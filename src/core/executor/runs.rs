//! The macro runs of the action executor: bookkeeping only, with no Win32
//! call, no thread and no clock of its own (the executor passes `now` and
//! `live` in), so that iteration, `Delay` deadlines and epochs are tested
//! with a fake clock.
//!
//! A `Sequence` job becomes a run. A run goes step after step until its
//! next `Delay`, which stores the deadline and yields to the other runs and
//! jobs; once `now` has reached the deadline, the run goes on from the step
//! after the `Delay`. Runs advance oldest first, once per poll, so two
//! macros fired together interleave at their `Delay` steps and never inside
//! a stretch of steps between two of them.
//!
//! A `Paste` job is a run of one step, and a `Paste` step is a step of its
//! macro's run, which the runs' paste session works (`paste`): its waits (a
//! held modifier, a busy clipboard, the restore of the paste before it) are
//! deadlines of its run, as a `Delay`'s are, and the session's restore is
//! polled with the runs. One paste tries at a time, the oldest first; while
//! one waits, the jobs fired after it are held, in order, and a `Paste`
//! action fired meanwhile is dropped (cut D12).

use std::collections::VecDeque;
use std::ops::AddAssign;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::paste::{Attempt, Closing, PasteSession, Ports, Step};
use super::{QUEUE_CAPACITY, Runner, TargetWindow};
use crate::core::windows_actions::BindAction;
use crate::data::paste::PasteSpec;
use crate::data::sequence::SequenceStep;

/// One slow action, as the thread that fired it queued it.
pub(crate) struct Job {
    pub(crate) action: BindAction,
    /// The foreground window when it fired, for a `Sequence` only.
    pub(crate) target: TargetWindow,
    /// The epoch it fired in; a job of an older epoch never starts.
    pub(crate) epoch: u64,
}

/// The macros and pastes that run, oldest first, and the paste session.
pub(crate) struct Runs {
    active: Vec<Run>,
    /// The jobs that came while a paste waited, oldest first; they start
    /// once it is over.
    held: VecDeque<Job>,
    session: PasteSession,
}

/// What a run walks.
enum Body {
    /// A `Sequence`'s steps.
    Macro(Arc<Vec<SequenceStep>>),
    /// A `Paste` action: one step.
    Paste(Arc<PasteSpec>),
}

struct Run {
    body: Body,
    /// The index of the next step.
    next: usize,
    target: TargetWindow,
    epoch: u64,
    resume: Resume,
    /// The state of the paste at the next step while it waits.
    attempt: Option<Attempt>,
}

/// When a run goes on.
#[derive(Debug, Clone, Copy)]
enum Resume {
    /// At the next poll: a run that has not started.
    Now,
    /// Once `now` reaches the deadline of its `Delay` or of its paste's
    /// wait.
    At(Instant),
    /// Only by being cancelled: the deadline is past what `Instant` holds.
    WhenCancelled,
}

/// When the executor must poll again without a job or a wake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Next {
    /// No run waits for a deadline.
    Idle,
    /// At the nearest deadline of the waiting runs.
    At(Instant),
}

/// What the runs need from the executor for one `start` or `poll`.
pub(crate) struct Ctx<'a> {
    pub(crate) runner: &'a mut dyn Runner,
    /// Whether work fired in this epoch may still run.
    pub(crate) live: &'a dyn Fn(u64) -> bool,
    pub(crate) now: &'a dyn Fn() -> Instant,
}

/// The runner calls of a `start` or a `poll`.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Calls {
    /// How many calls were made, the ones that panicked included.
    pub(crate) made: u32,
    /// How many of them panicked.
    pub(crate) panics: u32,
    /// How many `Paste` actions were dropped because another one waited.
    pub(crate) dropped_pastes: u32,
}

impl AddAssign for Calls {
    fn add_assign(&mut self, other: Self) {
        self.made += other.made;
        self.panics += other.panics;
        self.dropped_pastes += other.dropped_pastes;
    }
}

/// What a poll did, and when the next one is due.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Polled {
    pub(crate) next: Next,
    pub(crate) calls: Calls,
}

/// A call of the paste session on its ports at `now`; `true` if it asked a
/// port.
type SessionCall = fn(&mut PasteSession, &Ports<'_>, Instant) -> bool;

impl Runs {
    pub(crate) fn new() -> Self {
        Self {
            active: Vec::new(),
            held: VecDeque::new(),
            session: PasteSession::new(),
        }
    }

    /// Starts `job` unless its epoch is no longer live: a `Sequence` or a
    /// `Paste` becomes a run, which the next poll starts, and any other
    /// action runs at once. While a paste waits, the job is held until it
    /// is over, so that jobs run in firing order (ruling 7); a `Paste`
    /// fired while another one waits or is held is dropped and counted.
    pub(crate) fn start(&mut self, job: Job, ctx: &mut Ctx<'_>) -> Calls {
        if !(ctx.live)(job.epoch) {
            return Calls::default();
        }
        if matches!(job.action, BindAction::Paste(_)) && self.paste_waits() {
            return Calls {
                dropped_pastes: 1,
                ..Calls::default()
            };
        }
        if self.attempt_waits() {
            self.held.push_back(job);
            return Calls::default();
        }
        self.begin(job, ctx)
    }

    /// Makes `job` a run, or runs it at once if it is no `Sequence` and no
    /// `Paste`.
    fn begin(&mut self, job: Job, ctx: &mut Ctx<'_>) -> Calls {
        let body = match job.action {
            BindAction::Sequence(steps) => Body::Macro(steps),
            BindAction::Paste(spec) => Body::Paste(spec),
            action => return call(|| ctx.runner.action(&action)),
        };
        self.active.push(Run {
            body,
            next: 0,
            target: job.target,
            epoch: job.epoch,
            resume: Resume::Now,
            attempt: None,
        });
        Calls::default()
    }

    /// Whether a `Paste` action waits or is held: one more would only paste
    /// the same phrase again once the key that fires it is released, as a
    /// repeating key does (cut D12). A macro's `Paste` step does not count.
    fn paste_waits(&self) -> bool {
        self.active
            .iter()
            .any(|run| matches!(run.body, Body::Paste(_)))
            || self
                .held
                .iter()
                .any(|job| matches!(job.action, BindAction::Paste(_)))
    }

    /// Whether the paste of a run waits between its tries.
    fn attempt_waits(&self) -> bool {
        self.active.iter().any(|run| run.attempt.is_some())
    }

    /// Whether the executor may take another job from its queue: the held
    /// jobs are at most a queue's worth, so that the next ones stay in the
    /// queue, where `submit` counts what overflows.
    pub(crate) fn can_take(&self) -> bool {
        self.held.len() < QUEUE_CAPACITY
    }

    /// Drops every run and held job that is no longer live, decides whether
    /// the paste session's restore is due, advances each due run once,
    /// oldest first, up to its next `Delay`, its paste's next wait or its
    /// end, starts the held jobs once no paste waits, and puts the
    /// clipboard back if that is due.
    pub(crate) fn poll(&mut self, ctx: &mut Ctx<'_>) -> Polled {
        let mut calls = Calls::default();
        self.active.retain(|run| (ctx.live)(run.epoch));
        self.held.retain(|job| (ctx.live)(job.epoch));
        calls += self.session_call(ctx, PasteSession::tick);
        self.advance_from(0, ctx, &mut calls);
        // Oldest first, each advanced in this poll, until one leaves its
        // paste waiting.
        while !self.attempt_waits()
            && let Some(job) = self.held.pop_front()
        {
            let first = self.active.len();
            calls += self.begin(job, ctx);
            self.advance_from(first, ctx, &mut calls);
        }
        calls += self.session_call(ctx, PasteSession::restore_if_due);
        Polled {
            next: self.next(),
            calls,
        }
    }

    /// Advances each due run from `first` on, oldest first, and drops the
    /// runs that end.
    fn advance_from(&mut self, first: usize, ctx: &mut Ctx<'_>, calls: &mut Calls) {
        let mut index = first;
        while index < self.active.len() {
            let Runs {
                active, session, ..
            } = self;
            let (older, rest) = active.split_at_mut(index);
            let run = &mut rest[0];
            let due = match run.resume {
                Resume::Now => true,
                Resume::At(deadline) => (ctx.now)() >= deadline,
                Resume::WhenCancelled => false,
            };
            // The head rule (cut D6): only the oldest paste that waits
            // tries; a younger run at a Paste step waits until its
            // deadline.
            let head = older
                .iter()
                .find_map(|run| match (&run.attempt, run.resume) {
                    (Some(_), Resume::At(deadline)) => Some(deadline),
                    _ => None,
                });
            if due && !run.advance(ctx, calls, session, head) {
                active.remove(index);
            } else {
                index += 1;
            }
        }
    }

    /// Makes `f`, a call of the paste session, if a restore waits, under
    /// `call`, and counts it only if it asked a port. A panic ends the
    /// session, so that no later poll panics on it again.
    fn session_call(&mut self, ctx: &mut Ctx<'_>, f: SessionCall) -> Calls {
        if !self.session.is_pending() {
            return Calls::default();
        }
        let now = (ctx.now)();
        let ports = ctx.runner.paste();
        let session = &mut self.session;
        let mut asked = false;
        let made = call(|| asked = f(session, &ports, now));
        if made.panics > 0 {
            let _ = call(|| session.give_up(&ports));
            return made;
        }
        if asked { made } else { Calls::default() }
    }

    fn next(&self) -> Next {
        self.active
            .iter()
            .filter_map(|run| match run.resume {
                Resume::At(deadline) => Some(deadline),
                Resume::Now | Resume::WhenCancelled => None,
            })
            .chain(self.session.next())
            .min()
            .map_or(Next::Idle, Next::At)
    }

    /// Drops every run that waits for a deadline, which nothing would wake
    /// without a timer, and returns how many; a pending restore of the
    /// paste session is tried at once.
    pub(crate) fn drop_timed(&mut self, ctx: &mut Ctx<'_>) -> usize {
        let before = self.active.len();
        self.active
            .retain(|run| !matches!(run.resume, Resume::At(_)));
        self.session.restore_now(&ctx.runner.paste());
        before - self.active.len()
    }

    /// The restore of the paste session as the executor stops, tried until
    /// `until`.
    pub(crate) fn close(&mut self, ctx: &mut Ctx<'_>, until: Instant) -> Closing {
        let now = (ctx.now)();
        self.session.close(&ctx.runner.paste(), now, until)
    }

    /// The last try at the paste session's restore, and the end of its
    /// windows, as the executor thread ends.
    pub(crate) fn abandon(&mut self, runner: &mut dyn Runner) {
        self.session.abandon(&runner.paste());
    }

    pub(crate) fn len(&self) -> usize {
        self.active.len()
    }
}

impl Run {
    /// The spec of the run's next step, if that step is a `Paste`.
    fn paste_at_next(&self) -> Option<&Arc<PasteSpec>> {
        match &self.body {
            Body::Paste(spec) => (self.next == 0).then_some(spec),
            Body::Macro(steps) => match steps.get(self.next) {
                Some(SequenceStep::Paste(spec)) => Some(spec),
                _ => None,
            },
        }
    }

    /// Runs steps up to the next `Delay` or wait of a paste, which sets when
    /// the run goes on, and returns `true`; or up to the run's end (past its
    /// last step, its epoch no longer live, a step that panicked) and
    /// returns `false`. At a Paste step it waits until `head`, the deadline
    /// of an older run's paste, if there is one.
    fn advance(
        &mut self,
        ctx: &mut Ctx<'_>,
        calls: &mut Calls,
        session: &mut PasteSession,
        head: Option<Instant>,
    ) -> bool {
        loop {
            if !(ctx.live)(self.epoch) {
                return false;
            }
            if let Some(spec) = self.paste_at_next().cloned() {
                if let Some(deadline) = head {
                    self.resume = Resume::At(deadline);
                    return true;
                }
                let epoch = self.epoch;
                let Ctx { runner, live, now } = ctx;
                let ports = runner.paste();
                let live_now = || live(epoch);
                let attempt = &mut self.attempt;
                let mut step = Step::Done;
                let made = call(|| step = session.attempt(attempt, &spec, &ports, *now, &live_now));
                *calls += made;
                if made.panics > 0 {
                    return false;
                }
                match step {
                    Step::Done => self.next += 1,
                    Step::WaitUntil(at) => {
                        self.resume = Resume::At(at);
                        return true;
                    }
                }
                continue;
            }
            let Body::Macro(steps) = &self.body else {
                return false;
            };
            let Some(step) = steps.get(self.next) else {
                return false;
            };
            self.next += 1;
            if let SequenceStep::Delay(ms) = step {
                // Taken when the step is reached, after any slow step
                // before it.
                self.resume = (ctx.now)()
                    .checked_add(Duration::from_millis(*ms))
                    .map_or(Resume::WhenCancelled, Resume::At);
                return true;
            }
            let target = self.target;
            let made = call(|| ctx.runner.step(step, target));
            *calls += made;
            if made.panics > 0 {
                return false;
            }
        }
    }
}

/// Makes one runner call and catches its panic: one thread carries every
/// slow action, so an action that panics must end only itself, as its own
/// thread did before the executor.
fn call(f: impl FnOnce()) -> Calls {
    let panicked = panic::catch_unwind(AssertUnwindSafe(f)).is_err();
    Calls {
        made: 1,
        panics: u32::from(panicked),
        dropped_pastes: 0,
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::{Arc, Mutex, PoisonError};
    use std::time::{Duration, Instant};

    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;
    use crate::core::executor::testing::FakeDesk;
    use crate::core::executor::{Runner, TargetWindow};
    use crate::core::windows_actions::BindAction;
    use crate::data::sequence::SequenceStep;
    use crate::platform::clipboard::{CF_UNICODETEXT, Call, FakeClipboard};

    /// A fake clock: the base instant plus an offset that the test, or the
    /// runner inside a call, moves forward.
    type Offset = Arc<Mutex<Duration>>;

    /// The calls a [`Recorder`] made, shared with its desk's keystrokes.
    type Log = Arc<Mutex<Vec<String>>>;

    /// Records the `Display` text of every call, and every paste keystroke
    /// of its fake desk as `keys <names>`; panics on `panic_on` before
    /// recording, and moves the clock by `advance` inside the call named in
    /// it.
    struct Recorder {
        log: Log,
        panic_on: Option<&'static str>,
        advance: Option<(&'static str, Duration, Offset)>,
        clipboard: FakeClipboard,
        desk: FakeDesk,
    }

    impl Recorder {
        fn new() -> Self {
            let log = Log::default();
            let keys = Arc::clone(&log);
            Self {
                log,
                panic_on: None,
                advance: None,
                clipboard: FakeClipboard::new(),
                desk: FakeDesk::new().with_sink(move |text| {
                    keys.lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .push(text);
                }),
            }
        }

        fn call(&mut self, text: String) {
            if let Some((on, by, offset)) = &self.advance
                && *on == text
            {
                *offset.lock().unwrap_or_else(PoisonError::into_inner) += *by;
            }
            if self.panic_on == Some(text.as_str()) {
                panic!("the recorder panics on {text}");
            }
            self.log
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(text);
        }
    }

    impl Runner for Recorder {
        fn action(&mut self, action: &BindAction) {
            self.call(action.to_string());
        }

        fn step(&mut self, step: &SequenceStep, _target: TargetWindow) {
            self.call(step.to_string());
        }

        fn paste(&mut self) -> Ports<'_> {
            Ports {
                clipboard: &self.clipboard,
                desk: &self.desk,
            }
        }
    }

    /// A run list with a recording runner, a fake clock and an epoch.
    struct Fixture {
        runs: Runs,
        runner: Recorder,
        base: Instant,
        offset: Offset,
        epoch: Cell<u64>,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                runs: Runs::new(),
                runner: Recorder::new(),
                base: Instant::now(),
                offset: Arc::new(Mutex::new(Duration::ZERO)),
                epoch: Cell::new(0),
            }
        }

        fn ms(&self, ms: u64) -> Instant {
            self.base + Duration::from_millis(ms)
        }

        /// Sets the clock to the base plus `ms`.
        fn at(&self, ms: u64) {
            *self.offset.lock().unwrap_or_else(PoisonError::into_inner) = Duration::from_millis(ms);
        }

        fn start_in(&mut self, action: BindAction, epoch: u64) -> Calls {
            let Fixture {
                runs,
                runner,
                base,
                offset,
                epoch: live_epoch,
            } = self;
            let live = |job_epoch: u64| job_epoch == live_epoch.get();
            let now = || *base + *offset.lock().unwrap_or_else(PoisonError::into_inner);
            let mut ctx = Ctx {
                runner,
                live: &live,
                now: &now,
            };
            let job = Job {
                action,
                target: TargetWindow::NONE,
                epoch,
            };
            runs.start(job, &mut ctx)
        }

        fn start(&mut self, action: BindAction) -> Calls {
            let epoch = self.epoch.get();
            self.start_in(action, epoch)
        }

        fn poll(&mut self) -> Polled {
            let Fixture {
                runs,
                runner,
                base,
                offset,
                epoch,
            } = self;
            let live = |job_epoch: u64| job_epoch == epoch.get();
            let now = || *base + *offset.lock().unwrap_or_else(PoisonError::into_inner);
            let mut ctx = Ctx {
                runner,
                live: &live,
                now: &now,
            };
            runs.poll(&mut ctx)
        }

        /// Polls at each next deadline, as the executor's timer wakes it,
        /// until no run and no restore waits, or the next deadline is past
        /// `until_ms`.
        fn run_until_idle(&mut self, until_ms: u64) {
            for _ in 0..100_000 {
                let Next::At(at) = self.poll().next else {
                    return;
                };
                if at > self.ms(until_ms) {
                    return;
                }
                let mut offset = self.offset.lock().unwrap_or_else(PoisonError::into_inner);
                *offset = (*offset).max(at - self.base);
            }
            panic!("the runs did not settle");
        }

        /// The calls recorded since the last `taken`.
        fn taken(&mut self) -> Vec<String> {
            std::mem::take(
                &mut *self
                    .runner
                    .log
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner),
            )
        }
    }

    fn sequence(steps: Vec<SequenceStep>) -> BindAction {
        BindAction::Sequence(Arc::new(steps))
    }

    fn key(vk: VIRTUAL_KEY) -> SequenceStep {
        SequenceStep::PressKey(vk)
    }

    fn delay(ms: u64) -> SequenceStep {
        SequenceStep::Delay(ms)
    }

    /// The spec of a `Paste` of `text` with `Ctrl+V` and `restore_delay`.
    fn spec(text: &str, restore_delay: u64) -> Arc<PasteSpec> {
        Arc::new(PasteSpec::new(text, vec![VK_CONTROL, VK_V], restore_delay).expect("a valid spec"))
    }

    fn paste(text: &str) -> BindAction {
        BindAction::Paste(spec(text, 50))
    }

    fn paste_step(text: &str) -> SequenceStep {
        SequenceStep::Paste(spec(text, 50))
    }

    #[test]
    fn a_paste_never_reaches_runner_action_or_step() {
        // The runs' paste session works a Paste, the action and the step;
        // the runner's `action` and `step` never see one.
        let mut fixture = Fixture::new();
        fixture.start(paste("ab"));
        fixture.start(sequence(vec![paste_step("cd")]));
        fixture.run_until_idle(10_000);
        let taken = fixture.taken();
        assert_eq!(taken, ["keys Ctrl+V", "keys Ctrl+V"], "both pasted");

        fixture.start(sequence(vec![key(VK_F13), paste_step("ef"), key(VK_F14)]));
        fixture.run_until_idle(20_000);
        assert_eq!(fixture.taken(), ["key F13", "keys Ctrl+V", "key F14"]);
    }

    fn text(text: &str) -> BindAction {
        BindAction::Text(Arc::from(text))
    }

    /// `text` as `CF_UNICODETEXT` bytes, its NUL included.
    fn utf16(text: &str) -> Vec<u8> {
        text.encode_utf16()
            .chain([0])
            .flat_map(u16::to_le_bytes)
            .collect()
    }

    /// A clipboard that holds "ORIG", put there by another program.
    fn original() -> FakeClipboard {
        FakeClipboard::new().with_content(0x77, &[(CF_UNICODETEXT, &utf16("ORIG"))])
    }

    /// Whether `clipboard` holds "ORIG" again.
    fn holds_the_original(clipboard: &FakeClipboard) -> bool {
        clipboard
            .content()
            .first()
            .is_some_and(|first| *first == (CF_UNICODETEXT, utf16("ORIG")))
    }

    impl Fixture {
        /// A fixture whose runner pastes through `clipboard`.
        fn with_clipboard(clipboard: &FakeClipboard) -> Self {
            let mut fixture = Self::new();
            fixture.runner.clipboard = clipboard.clone();
            fixture
        }

        /// Starts `action` and polls once, as the drain does.
        fn submit(&mut self, action: BindAction) -> Calls {
            let mut calls = self.start(action);
            calls += self.poll().calls;
            calls
        }

        fn drop_timed(&mut self) -> usize {
            let Fixture {
                runs,
                runner,
                base,
                offset,
                epoch,
            } = self;
            let live = |job_epoch: u64| job_epoch == epoch.get();
            let now = || *base + *offset.lock().unwrap_or_else(PoisonError::into_inner);
            let mut ctx = Ctx {
                runner,
                live: &live,
                now: &now,
            };
            runs.drop_timed(&mut ctx)
        }
    }

    #[test]
    fn jobs_after_a_waiting_paste_run_after_it() {
        // Ruling 7: firing order, even behind a Paste that waits.
        let mut fixture = Fixture::new();
        fixture.runner.desk.hold_modifiers(true);
        fixture.submit(paste("ab"));
        fixture.submit(text("cd"));

        fixture.runner.desk.hold_modifiers(false);
        fixture.at(5);
        fixture.poll();

        assert_eq!(fixture.taken(), ["keys Ctrl+V", "text (2 chars)"]);
    }

    #[test]
    fn a_run_started_before_a_waiting_paste_goes_on() {
        let mut fixture = Fixture::new();
        fixture.runner.desk.hold_modifiers(true);
        fixture.submit(sequence(vec![key(VK_F13), delay(10), key(VK_F14)]));
        fixture.submit(paste("ab"));
        assert_eq!(fixture.taken(), ["key F13"]);

        fixture.at(10);
        fixture.poll();

        assert_eq!(fixture.taken(), ["key F14"], "the Paste still waits");
    }

    #[test]
    fn extra_standalone_pastes_while_one_waits_are_dropped_and_counted() {
        let mut fixture = Fixture::new();
        fixture.runner.desk.hold_modifiers(true);
        let mut calls = Calls::default();
        for phrase in ["a", "b", "c"] {
            calls += fixture.submit(paste(phrase));
        }
        fixture.runner.desk.hold_modifiers(false);
        fixture.at(5);
        calls += fixture.poll().calls;
        fixture.run_until_idle(10_000);

        assert_eq!(calls.dropped_pastes, 2);
        assert_eq!(fixture.taken(), ["keys Ctrl+V"]);
    }

    #[test]
    fn a_repeat_burst_of_pastes_leaves_no_backlog() {
        // A held key repeats every 33 ms; each repeat that fires a Paste
        // while one waits adds nothing.
        let mut fixture = Fixture::new();
        fixture.runner.desk.hold_modifiers(true);
        let mut calls = Calls::default();
        for repeat in 0..30 {
            fixture.at(repeat * 33);
            calls += fixture.submit(paste("x"));
        }
        fixture.runner.desk.hold_modifiers(false);
        fixture.at(30 * 33);
        calls += fixture.poll().calls;
        fixture.run_until_idle(10_000);

        assert_eq!(calls.dropped_pastes, 29);
        assert_eq!(fixture.taken(), ["keys Ctrl+V"]);
    }

    #[test]
    fn a_macro_paste_step_is_never_dropped() {
        let mut fixture = Fixture::new();
        fixture.runner.desk.hold_modifiers(true);
        let mut calls = Calls::default();
        calls += fixture.submit(paste("a"));
        calls += fixture.submit(sequence(vec![paste_step("b")]));
        calls += fixture.submit(sequence(vec![paste_step("c")]));
        fixture.runner.desk.hold_modifiers(false);
        fixture.at(5);
        fixture.run_until_idle(10_000);

        assert_eq!(calls.dropped_pastes, 0);
        assert_eq!(fixture.taken(), ["keys Ctrl+V"; 3]);
    }

    #[test]
    fn an_epoch_change_drops_waiting_pastes_and_held_jobs_but_keeps_the_restore() {
        let clipboard = original();
        let mut fixture = Fixture::with_clipboard(&clipboard);
        fixture.submit(paste("A"));
        fixture.at(10);
        fixture.submit(paste("B"));
        fixture.submit(text("cd"));
        assert_eq!(fixture.taken(), ["keys Ctrl+V"], "B waits for A's restore");

        fixture.epoch.set(1);
        fixture.run_until_idle(10_000);

        assert_eq!(
            fixture.taken(),
            Vec::<String>::new(),
            "B and the Text never run"
        );
        assert!(holds_the_original(&clipboard), "{:?}", clipboard.content());
    }

    #[test]
    fn held_jobs_stop_at_the_queue_capacity() {
        let mut fixture = Fixture::new();
        fixture.runner.desk.hold_modifiers(true);
        fixture.submit(paste("a"));
        for _ in 0..QUEUE_CAPACITY {
            fixture.submit(text("t"));
        }
        assert!(!fixture.runs.can_take());

        fixture.runner.desk.hold_modifiers(false);
        fixture.at(5);
        fixture.poll();

        assert!(fixture.runs.can_take());
        assert_eq!(fixture.taken().len(), 1 + QUEUE_CAPACITY);
    }

    #[test]
    fn paste_attempts_go_in_firing_order() {
        // Only the oldest run at a Paste step tries; a younger one waits
        // for it, even when the clipboard would open for the younger one.
        let mut fixture = Fixture::new();
        fixture.runner.clipboard.busy_opens(1);
        let shift_insert =
            Arc::new(PasteSpec::new("b", vec![VK_SHIFT, VK_INSERT], 50).expect("a valid spec"));
        fixture.start(sequence(vec![paste_step("a")]));
        fixture.start(sequence(vec![SequenceStep::Paste(shift_insert)]));
        fixture.run_until_idle(10_000);

        assert_eq!(fixture.taken(), ["keys Ctrl+V", "keys Shift+Insert"]);
    }

    #[test]
    fn a_failed_timer_drops_the_waiting_pastes_and_restores_once() {
        let clipboard = original();
        let mut fixture = Fixture::with_clipboard(&clipboard);
        fixture.submit(paste("A"));
        fixture.runner.desk.hold_modifiers(true);
        fixture.submit(paste("B"));
        assert_eq!(fixture.taken(), ["keys Ctrl+V"]);

        let (dropped, lines) = crate::logging::testing::capture(|| fixture.drop_timed());

        assert_eq!(dropped, 1, "B waited");
        assert!(holds_the_original(&clipboard), "{:?}", clipboard.content());
        assert!(
            lines.iter().any(|line| line.contains(
                "paste: the executor's timer failed; the clipboard was put back at once"
            )),
            "{lines:?}"
        );
        fixture.runs.abandon(&mut fixture.runner);
        let empties = clipboard
            .calls()
            .into_iter()
            .filter(|call| *call == Call::Empty)
            .count();
        assert_eq!(empties, 2, "the paste's and the one restore");
        assert_eq!(fixture.taken(), Vec::<String>::new());
    }

    #[test]
    fn a_run_goes_up_to_its_first_delay_and_resumes_at_its_deadline() {
        let mut fixture = Fixture::new();
        fixture.start(sequence(vec![key(VK_F13), delay(100), key(VK_F14)]));

        let first = fixture.poll();
        assert_eq!(fixture.taken(), ["key F13"]);
        assert_eq!(first.next, Next::At(fixture.ms(100)));
        assert_eq!(first.calls.made, 1);

        fixture.at(99);
        let early = fixture.poll();
        assert_eq!(
            fixture.taken(),
            Vec::<String>::new(),
            "a Delay never ends early"
        );
        assert_eq!(
            early.calls.made, 0,
            "a poll that runs nothing reports no call"
        );
        assert_eq!(early.next, Next::At(fixture.ms(100)));

        fixture.at(100);
        let done = fixture.poll();
        assert_eq!(fixture.taken(), ["key F14"]);
        assert_eq!(done.next, Next::Idle);
        assert_eq!(fixture.runs.len(), 0, "a run past its last step is gone");
    }

    #[test]
    fn the_nearest_deadline_of_several_waiting_runs_is_next() {
        let mut fixture = Fixture::new();
        fixture.start(sequence(vec![delay(200), key(VK_F13)]));
        fixture.start(sequence(vec![delay(50), key(VK_F14)]));
        fixture.start(sequence(vec![delay(900), key(VK_F15)]));

        let waiting = fixture.poll();
        assert_eq!(
            waiting.next,
            Next::At(fixture.ms(50)),
            "the timer is armed for the earliest deadline, not the oldest or the latest run's"
        );

        fixture.at(50);
        let after_first = fixture.poll();
        assert_eq!(fixture.taken(), ["key F14"]);
        assert_eq!(
            after_first.next,
            Next::At(fixture.ms(200)),
            "with the nearest run gone the next one is the nearest"
        );
    }

    #[test]
    fn two_macros_fired_together_interleave_at_their_delays_and_keep_their_order() {
        let mut fixture = Fixture::new();
        fixture.start(sequence(vec![
            key(VK_F13),
            delay(30),
            key(VK_F14),
            delay(30),
            key(VK_F15),
        ]));
        fixture.start(sequence(vec![
            key(VK_F16),
            delay(30),
            key(VK_F17),
            delay(30),
            key(VK_F18),
        ]));

        for ms in [0, 30, 60] {
            fixture.at(ms);
            fixture.poll();
        }

        assert_eq!(
            fixture.taken(),
            [
                "key F13", "key F16", "key F14", "key F17", "key F15", "key F18"
            ]
        );
        assert_eq!(fixture.runs.len(), 0);
    }

    #[test]
    fn a_cancelled_waiting_run_never_resumes() {
        let mut fixture = Fixture::new();
        fixture.start(sequence(vec![key(VK_F13), delay(100), key(VK_F14)]));
        fixture.poll();
        assert_eq!(fixture.taken(), ["key F13"]);

        fixture.epoch.set(1);
        fixture.at(200);
        let polled = fixture.poll();

        assert_eq!(fixture.taken(), Vec::<String>::new());
        assert_eq!(fixture.runs.len(), 0, "the cancelled run is dropped");
        assert_eq!(polled.next, Next::Idle);
    }

    #[test]
    fn a_job_of_an_old_epoch_never_starts() {
        let mut fixture = Fixture::new();
        fixture.epoch.set(1);

        let calls = fixture.start_in(sequence(vec![key(VK_F13)]), 0);
        assert_eq!(calls, Calls::default());
        let calls = fixture.start_in(BindAction::OpenUrl("https://example.com".to_owned()), 0);
        assert_eq!(calls, Calls::default());
        fixture.poll();

        assert_eq!(fixture.taken(), Vec::<String>::new());
        assert_eq!(fixture.runs.len(), 0);
    }

    #[test]
    fn the_delay_starts_when_its_step_is_reached() {
        let mut fixture = Fixture::new();
        fixture.runner.advance = Some((
            "open https://example.com",
            Duration::from_millis(500),
            Arc::clone(&fixture.offset),
        ));
        fixture.start(sequence(vec![
            SequenceStep::OpenUrl("https://example.com".to_owned()),
            delay(100),
            key(VK_F14),
        ]));

        let polled = fixture.poll();

        assert_eq!(fixture.taken(), ["open https://example.com"]);
        assert_eq!(polled.next, Next::At(fixture.ms(600)));
    }

    #[test]
    fn a_panicking_step_ends_only_its_macro() {
        let mut fixture = Fixture::new();
        fixture.runner.panic_on = Some("key F13");
        fixture.start(sequence(vec![key(VK_F13), key(VK_F14)]));
        fixture.start(sequence(vec![key(VK_F15)]));

        let polled = fixture.poll();

        assert_eq!(fixture.taken(), ["key F15"]);
        assert_eq!(polled.calls.panics, 1);
        assert_eq!(polled.calls.made, 2);
        assert_eq!(fixture.runs.len(), 0);
    }

    #[test]
    fn a_zero_delay_lets_the_other_runs_go_first() {
        let mut fixture = Fixture::new();
        fixture.start(sequence(vec![key(VK_F13), delay(0), key(VK_F14)]));
        fixture.start(sequence(vec![key(VK_F15)]));

        let first = fixture.poll();
        assert_eq!(fixture.taken(), ["key F13", "key F15"]);
        let Next::At(at) = first.next else {
            panic!("a Delay(0) waits for a deadline: {:?}", first.next);
        };
        assert!(at <= fixture.ms(0), "the deadline has passed already");

        fixture.poll();
        assert_eq!(fixture.taken(), ["key F14"]);
    }

    #[test]
    fn launch_and_open_url_run_when_their_job_starts() {
        let mut fixture = Fixture::new();
        let launched = fixture.start(BindAction::Launch {
            exe: "no-such.exe".to_owned(),
            args: Vec::new(),
        });
        let opened = fixture.start(BindAction::OpenUrl("https://example.com".to_owned()));

        assert_eq!(
            fixture.taken(),
            ["launch no-such.exe", "open https://example.com"]
        );
        assert_eq!(
            launched,
            Calls {
                made: 1,
                panics: 0,
                dropped_pastes: 0
            }
        );
        assert_eq!(
            opened,
            Calls {
                made: 1,
                panics: 0,
                dropped_pastes: 0
            }
        );
        assert_eq!(fixture.runs.len(), 0);
    }

    #[test]
    fn a_text_runs_when_its_job_starts_and_makes_no_run() {
        let mut fixture = Fixture::new();
        let typed = fixture.start(BindAction::Text(Arc::from("abc")));

        assert_eq!(
            typed,
            Calls {
                made: 1,
                panics: 0,
                dropped_pastes: 0
            }
        );
        assert_eq!(fixture.taken(), ["text (3 chars)"]);
        assert_eq!(fixture.runs.len(), 0, "a Text is not walked as a macro");

        fixture.epoch.set(1);
        let stale = fixture.start_in(BindAction::Text(Arc::from("abc")), 0);
        assert_eq!(stale, Calls::default());
        assert_eq!(fixture.taken(), Vec::<String>::new());
    }
}
