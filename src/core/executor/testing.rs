//! Test-only: a [`Runner`] that records what the executor asks it to run,
//! and never runs a real action.
//!
//! Each call is recorded first, with the thread it ran on, and then the
//! runner does what the test asked for: block, wait on a gate, panic or
//! post `WM_QUIT` to its own thread. The only Win32 call it makes is that
//! `PostQuitMessage`, on the executor thread of the test that asks for it.

use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant};

use windows::Win32::UI::WindowsAndMessaging::PostQuitMessage;

use super::{Runner, TargetWindow};
use crate::core::windows_actions::BindAction;
use crate::data::sequence::SequenceStep;
use crate::logging::counters::Counters;

/// How long a gated call waits for its release at most, so that a test that
/// fails before it opens the gate does not hang the suite.
const GATE_LIMIT: Duration = Duration::from_secs(10);

/// What the runner was asked to run, as the startup list prints it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Ran {
    /// `Runner::action`, with the action's `Display` text.
    Action(String),
    /// `Runner::step`, with the step's `Display` text and its target.
    Step(String, TargetWindow),
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
    /// The `Display` text of what ran.
    pub(crate) fn text(&self) -> &str {
        match &self.ran {
            Ran::Action(text) | Ran::Step(text, _) => text,
        }
    }
}

/// Records every call on `records`; see the module docs.
pub(crate) struct FakeRunner {
    records: Sender<Record>,
    block: Duration,
    gate: Option<(Sender<()>, Receiver<()>)>,
    panic_on: Option<String>,
    quit_on: Option<String>,
}

impl FakeRunner {
    pub(crate) fn new(records: Sender<Record>) -> Self {
        Self {
            records,
            block: Duration::ZERO,
            gate: None,
            panic_on: None,
            quit_on: None,
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

    fn call(&mut self, ran: Ran) {
        let record = Record {
            thread: thread::current().id(),
            name: thread::current().name().map(str::to_owned),
            at: Instant::now(),
            ran,
        };
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
    }
}

impl Runner for FakeRunner {
    fn action(&mut self, action: &BindAction) {
        self.call(Ran::Action(action.to_string()));
    }

    fn step(&mut self, step: &SequenceStep, target: TargetWindow) {
        self.call(Ran::Step(step.to_string(), target));
    }
}

/// Counters of one test's own, so that no test takes the daemon's.
pub(crate) fn leaked_counters() -> &'static Counters {
    Box::leak(Box::new(Counters::new()))
}
