//! Test-only: a [`Runner`] that records what the executor asks it to run,
//! and never runs a real action, and the fake desk its pastes go through.
//!
//! Each call is recorded first, with the thread it ran on, and then the
//! runner does what the test asked for: block, wait on a gate, panic, post
//! `WM_QUIT` to its own thread, or remove the executor's own wake-ups from
//! its queue. Its only Win32 calls are that `PostQuitMessage` and that
//! `PeekMessageW`, on the executor thread of the test that asks for them.
//! Its pastes go through a `FakeClipboard` and a [`FakeDesk`], which record
//! the keystroke as `keys <names>`.

use std::collections::VecDeque;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;
use windows::Win32::UI::WindowsAndMessaging::{MSG, PM_REMOVE, PeekMessageW, PostQuitMessage};

use super::paste::{Desk, Ports, Probe};
use super::{Runner, TargetWindow, WAKE};
use crate::core::windows_actions::BindAction;
use crate::data::sequence::{self, ComboSent, SequenceStep};
use crate::data::vk_name::vk_name;
use crate::logging::counters::Counters;
use crate::platform::clipboard::FakeClipboard;

/// How long a gated call waits for its release at most, so that a test that
/// fails before it opens the gate does not hang the suite.
const GATE_LIMIT: Duration = Duration::from_secs(10);

/// How long a test waits for something the executor should do at once.
pub(crate) const WAIT: Duration = Duration::from_secs(2);

/// What the runner was asked to run, as the startup list prints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Ran {
    /// `Runner::action`, with the action's `Display` text.
    Action(String),
    /// `Runner::step`, with the step's `Display` text and its target.
    Step(String, TargetWindow),
    /// A paste keystroke on the fake desk, as `keys <names>`.
    Keys(String),
}

/// One call of the runner.
#[derive(Debug)]
pub(crate) struct Record {
    pub(crate) thread: ThreadId,
    pub(crate) name: Option<String>,
    pub(crate) at: Instant,
    pub(crate) ran: Ran,
}

impl Record {
    fn now(ran: Ran) -> Self {
        Self {
            thread: thread::current().id(),
            name: thread::current().name().map(str::to_owned),
            at: Instant::now(),
            ran,
        }
    }

    /// The `Display` text of what ran.
    pub(crate) fn text(&self) -> &str {
        match &self.ran {
            Ran::Action(text) | Ran::Step(text, _) | Ran::Keys(text) => text,
        }
    }
}

/// Records every call on `records`; see the module docs.
pub(crate) struct FakeRunner {
    records: Sender<Record>,
    block: Duration,
    gate: Option<(Sender<()>, Receiver<()>)>,
    eat_wakes: bool,
    panic_on: Option<String>,
    quit_on: Option<String>,
    clipboard: FakeClipboard,
    desk: FakeDesk,
}

impl FakeRunner {
    /// A runner whose pastes go through an empty fake clipboard and a fake
    /// desk with its defaults, whose keystrokes are recorded on `records`.
    pub(crate) fn new(records: Sender<Record>) -> Self {
        let desk = FakeDesk::new();
        desk.record_on(records.clone());
        Self {
            records,
            block: Duration::ZERO,
            gate: None,
            eat_wakes: false,
            panic_on: None,
            quit_on: None,
            clipboard: FakeClipboard::new(),
            desk,
        }
    }

    /// Every call blocks for `block` after it is recorded, standing in for
    /// an `OpenUrl` against a cold browser.
    pub(crate) fn with_block(mut self, block: Duration) -> Self {
        self.block = block;
        self
    }

    /// Every call sends on `started` after it is recorded, and then waits
    /// until `release` receives or its sender is dropped.
    pub(crate) fn with_gate(mut self, started: Sender<()>, release: Receiver<()>) -> Self {
        self.gate = Some((started, release));
        self
    }

    /// Every call removes the executor's `WAKE` messages from its own
    /// thread's queue before it returns (after the gate, if any), as a
    /// modal loop inside a step (a `ShellExecuteW` dialog) would.
    pub(crate) fn with_eat_wakes(mut self) -> Self {
        self.eat_wakes = true;
        self
    }

    /// A call whose text is `text` panics before it is recorded.
    pub(crate) fn with_panic_on(mut self, text: &str) -> Self {
        self.panic_on = Some(text.to_owned());
        self
    }

    /// A call whose text is `text` posts `WM_QUIT` to its own thread after
    /// it is recorded, as foreign code on the executor thread could.
    pub(crate) fn with_quit_on(mut self, text: &str) -> Self {
        self.quit_on = Some(text.to_owned());
        self
    }

    /// Its pastes go through `clipboard`, a clone of the test's.
    pub(crate) fn with_clipboard(mut self, clipboard: FakeClipboard) -> Self {
        self.clipboard = clipboard;
        self
    }

    /// Its pastes go through `desk`, a clone of the test's, whose
    /// keystrokes are recorded on this runner's records.
    pub(crate) fn with_desk(mut self, desk: FakeDesk) -> Self {
        desk.record_on(self.records.clone());
        self.desk = desk;
        self
    }

    fn call(&mut self, ran: Ran) {
        let record = Record::now(ran);
        if self.panic_on.as_deref() == Some(record.text()) {
            panic!("the fake runner panics on {}", record.text());
        }
        let quit = self.quit_on.as_deref() == Some(record.text());
        let _ = self.records.send(record);
        if quit {
            // SAFETY: PostQuitMessage takes the exit code by value, reads
            // no memory of ours and posts WM_QUIT only to the calling
            // thread's own queue: the executor thread of this test.
            unsafe { PostQuitMessage(0) };
        }
        if !self.block.is_zero() {
            thread::sleep(self.block);
        }
        if let Some((started, release)) = &self.gate {
            let _ = started.send(());
            let _ = release.recv_timeout(GATE_LIMIT);
        }
        if self.eat_wakes {
            let mut msg = MSG::default();
            // SAFETY: `msg` is a local that outlives every call, which only
            // writes it; the filter removes only WAKE messages, and only
            // from the calling thread's own queue.
            while unsafe { PeekMessageW(&mut msg, None, WAKE, WAKE, PM_REMOVE) }.as_bool() {}
        }
    }
}

impl Runner for FakeRunner {
    fn action(&mut self, action: &BindAction) {
        self.call(Ran::Action(action.to_string()));
    }

    fn step(&mut self, step: &SequenceStep, target: TargetWindow) {
        self.call(Ran::Step(step.to_string(), target));
    }

    fn paste(&mut self) -> Ports<'_> {
        Ports {
            clipboard: &self.clipboard,
            desk: &self.desk,
        }
    }
}

/// What a [`FakeDesk`] was asked, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DeskCall {
    Foreground,
    ModifierDown,
    Probe(isize, Duration),
    Press(Vec<VIRTUAL_KEY>),
    ReleaseAfter(Vec<VIRTUAL_KEY>, u32),
}

/// Where a [`FakeDesk`] reports each keystroke, as `keys <names>`.
type Sink = Box<dyn Fn(String) + Send>;

/// The state of a [`FakeDesk`]. Each script answers its calls in order and
/// repeats its last answer.
struct DeskState {
    foregrounds: VecDeque<TargetWindow>,
    modifiers: VecDeque<bool>,
    /// The probe answers; [`Probe::Answered`] once they run out.
    answers: VecDeque<Probe>,
    press_result: Option<ComboSent>,
    calls: Vec<DeskCall>,
    sink: Option<Sink>,
    /// Runs at every probe, before it answers.
    on_probe: Option<Box<dyn Fn() + Send>>,
}

/// A desk that records every call and sends nothing, for the tests of
/// pastes. Its defaults: the foreground window `0x1234`, no modifier down,
/// every probe answered, every keystroke sent whole. Clones share it, so a
/// test keeps one and hands the other to the session.
#[derive(Clone)]
pub(crate) struct FakeDesk(Arc<Mutex<DeskState>>);

/// The foreground window of a [`FakeDesk`] that is told no other.
pub(crate) const FOREGROUND: TargetWindow = TargetWindow::from_raw(0x1234);

/// The next answer of `script`, which repeats its last one.
fn next_of<T: Copy>(script: &mut VecDeque<T>, otherwise: T) -> T {
    if script.len() > 1 {
        script.pop_front().unwrap_or(otherwise)
    } else {
        script.front().copied().unwrap_or(otherwise)
    }
}

impl FakeDesk {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Mutex::new(DeskState {
            foregrounds: VecDeque::from([FOREGROUND]),
            modifiers: VecDeque::from([false]),
            answers: VecDeque::new(),
            press_result: None,
            calls: Vec::new(),
            sink: None,
            on_probe: None,
        })))
    }

    fn state(&self) -> MutexGuard<'_, DeskState> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The foreground window answers `windows` in turn, then the last one.
    pub(crate) fn with_foregrounds(self, windows: &[isize]) -> Self {
        self.state().foregrounds = windows
            .iter()
            .map(|&raw| TargetWindow::from_raw(raw))
            .collect();
        self
    }

    /// The modifier reads answer `held` in turn, then the last one.
    pub(crate) fn with_modifiers(self, held: &[bool]) -> Self {
        self.state().modifiers = held.iter().copied().collect();
        self
    }

    /// The probes answer `answers` in turn, then `Answered`.
    pub(crate) fn with_answers(self, answers: &[Probe]) -> Self {
        self.state().answers = answers.iter().copied().collect();
        self
    }

    /// Every keystroke reports `result`.
    pub(crate) fn with_press_result(self, result: ComboSent) -> Self {
        self.state().press_result = Some(result);
        self
    }

    /// The user holds a modifier from now on, or releases every one.
    pub(crate) fn hold_modifiers(&self, held: bool) {
        self.state().modifiers = VecDeque::from([held]);
    }

    /// The foreground window is `raw` from now on.
    pub(crate) fn set_foreground(&self, raw: isize) {
        self.state().foregrounds = VecDeque::from([TargetWindow::from_raw(raw)]);
    }

    /// `hook` runs at every probe, before it answers, so that a test can
    /// see what the clipboard was asked by then.
    pub(crate) fn with_probe_hook(self, hook: impl Fn() + Send + 'static) -> Self {
        self.state().on_probe = Some(Box::new(hook));
        self
    }

    /// Each keystroke is also reported to `sink`.
    pub(crate) fn with_sink(self, sink: impl Fn(String) + Send + 'static) -> Self {
        self.state().sink = Some(Box::new(sink));
        self
    }

    /// Each keystroke is also recorded on `records`, from the thread that
    /// sends it.
    fn record_on(&self, records: Sender<Record>) {
        self.state().sink = Some(Box::new(move |text| {
            let _ = records.send(Record::now(Ran::Keys(text)));
        }));
    }

    /// Every call so far.
    pub(crate) fn calls(&self) -> Vec<DeskCall> {
        self.state().calls.clone()
    }

    /// How many times the foreground window was read so far, without a
    /// copy of the calls, for a bench that counts the attempts that start.
    pub(crate) fn foreground_reads(&self) -> usize {
        self.state()
            .calls
            .iter()
            .filter(|call| matches!(call, DeskCall::Foreground))
            .count()
    }

    /// The keystrokes sent so far.
    pub(crate) fn pressed(&self) -> Vec<Vec<VIRTUAL_KEY>> {
        self.state()
            .calls
            .iter()
            .filter_map(|call| match call {
                DeskCall::Press(keys) => Some(keys.clone()),
                _ => None,
            })
            .collect()
    }

    /// The windows probed so far, with the time each probe could take.
    pub(crate) fn probes(&self) -> Vec<(isize, Duration)> {
        self.state()
            .calls
            .iter()
            .filter_map(|call| match call {
                DeskCall::Probe(window, within) => Some((*window, *within)),
                _ => None,
            })
            .collect()
    }

    /// The key-ups sent after a keystroke that was cut short.
    pub(crate) fn released(&self) -> Vec<(Vec<VIRTUAL_KEY>, u32)> {
        self.state()
            .calls
            .iter()
            .filter_map(|call| match call {
                DeskCall::ReleaseAfter(keys, sent) => Some((keys.clone(), *sent)),
                _ => None,
            })
            .collect()
    }
}

impl Desk for FakeDesk {
    fn foreground(&self) -> TargetWindow {
        let mut state = self.state();
        state.calls.push(DeskCall::Foreground);
        next_of(&mut state.foregrounds, TargetWindow::NONE)
    }

    fn responds(&self, window: isize, within: Duration) -> Probe {
        let mut state = self.state();
        state.calls.push(DeskCall::Probe(window, within));
        if let Some(on_probe) = &state.on_probe {
            on_probe();
        }
        state.answers.pop_front().unwrap_or(Probe::Answered)
    }

    fn modifier_down(&self) -> bool {
        let mut state = self.state();
        state.calls.push(DeskCall::ModifierDown);
        next_of(&mut state.modifiers, false)
    }

    fn press(&self, keys: &[VIRTUAL_KEY]) -> ComboSent {
        let mut state = self.state();
        state.calls.push(DeskCall::Press(keys.to_vec()));
        let names: Vec<String> = keys.iter().map(|&key| vk_name(key)).collect();
        if let Some(sink) = &state.sink {
            sink(format!("keys {}", names.join("+")));
        }
        let whole = u32::try_from(keys.len() * 2).unwrap_or(u32::MAX);
        state.press_result.unwrap_or(ComboSent {
            sent: whole,
            of: whole,
        })
    }

    fn release_after(&self, keys: &[VIRTUAL_KEY], sent: u32) -> u32 {
        let mut state = self.state();
        state
            .calls
            .push(DeskCall::ReleaseAfter(keys.to_vec(), sent));
        u32::try_from(sequence::unreleased_keys(keys, sent).len()).unwrap_or(u32::MAX)
    }
}

/// Counters of one test's own, so that no test takes the daemon's.
pub(crate) fn leaked_counters() -> &'static Counters {
    Box::leak(Box::new(Counters::new()))
}

/// The next call of the runner, waited for up to [`WAIT`].
pub(crate) fn next_record(records: &Receiver<Record>) -> Record {
    records
        .recv_timeout(WAIT)
        .expect("the runner is called within 2 s")
}

/// Waits up to [`WAIT`] for `condition`, and fails naming `what`.
pub(crate) fn wait_until(what: &str, condition: impl Fn() -> bool) {
    let begun = Instant::now();
    while !condition() {
        assert!(begun.elapsed() < WAIT, "{what}: not within 2 s");
        thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;

    #[test]
    fn the_fake_desk_records_each_keystroke_as_keys_and_its_names() {
        let (tx, records) = mpsc::channel();
        let desk = FakeDesk::new()
            .with_foregrounds(&[0x10, 0x20])
            .with_modifiers(&[true, false])
            .with_answers(&[Probe::NoAnswer]);
        let clipboard = FakeClipboard::new().with_content(0x77, &[]);
        let mut runner = FakeRunner::new(tx)
            .with_desk(desk.clone())
            .with_clipboard(clipboard.clone());

        let ports = runner.paste();
        assert_eq!(ports.clipboard.owner(), Some(0x77), "the test's clipboard");
        assert_eq!(ports.desk.foreground(), TargetWindow::from_raw(0x10));
        assert_eq!(ports.desk.foreground(), TargetWindow::from_raw(0x20));
        assert_eq!(
            ports.desk.foreground(),
            TargetWindow::from_raw(0x20),
            "the last repeats"
        );
        assert!(ports.desk.modifier_down());
        assert!(!ports.desk.modifier_down());
        assert_eq!(
            ports.desk.responds(0x77, Duration::from_millis(100)),
            Probe::NoAnswer
        );
        assert_eq!(
            ports.desk.responds(0x77, Duration::from_millis(20)),
            Probe::Answered
        );
        assert_eq!(
            ports.desk.press(&[VK_SHIFT, VK_INSERT]),
            ComboSent { sent: 4, of: 4 }
        );
        assert_eq!(ports.desk.release_after(&[VK_CONTROL, VK_V], 2), 2);

        let record = records.try_recv().expect("the keystroke is recorded");
        assert_eq!(record.ran, Ran::Keys("keys Shift+Insert".to_owned()));
        assert_eq!(record.text(), "keys Shift+Insert");
        assert_eq!(
            record.thread,
            thread::current().id(),
            "on the thread that sent it"
        );
        assert_eq!(desk.pressed(), [vec![VK_SHIFT, VK_INSERT]]);
        assert_eq!(
            desk.probes(),
            [
                (0x77, Duration::from_millis(100)),
                (0x77, Duration::from_millis(20))
            ]
        );
        assert_eq!(desk.released(), [(vec![VK_CONTROL, VK_V], 2)]);
        assert_eq!(desk.calls().len(), 9);
        assert_eq!(desk.foreground_reads(), 3);

        desk.hold_modifiers(true);
        desk.set_foreground(0x30);
        let cut_short = FakeDesk::new()
            .with_press_result(ComboSent { sent: 1, of: 4 })
            .with_sink(|_| {});
        assert!(desk.modifier_down());
        assert_eq!(desk.foreground(), TargetWindow::from_raw(0x30));
        assert_eq!(
            cut_short.press(&[VK_CONTROL, VK_V]),
            ComboSent { sent: 1, of: 4 }
        );
    }
}
