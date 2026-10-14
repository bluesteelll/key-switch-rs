//! Scenario harness for the engine: a RON config, a script of key events on
//! a fake clock, and assertions on what the engine decided.
//!
//! ```ignore
//! Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
//!     .down("CapsLock").expect_suppressed().expect_fired(&[BindAction::SwitchLanguage])
//!     .advance_ms(80)
//!     .up("CapsLock").expect_suppressed().expect_nothing_fired();
//! ```
//!
//! Key names are parsed like the config's. The unsided modifier names press
//! the left key, because a real keyboard never reports an unsided code:
//! `"Shift"` is LShift, `"Ctrl"` LCtrl and `"Alt"` LAlt (`"Win"` already
//! parses to LWin). Use `"RShift"` and so on for the right-hand keys.
//!
//! [`FakeEnv`] models the OS key state the way Windows keeps it: a key-down
//! or key-up changes it only if the engine let the event through, and a
//! generic modifier code reads as down while either side is. Timers that the
//! engine requests fire during [`Scenario::advance_ms`], in due order;
//! [`Scenario::advance_ms_stalled`] holds them back to model a timer thread
//! that wakes late.
//!
//! The `expect_*` methods check the most recent step: a key event, or an
//! `advance_ms` together with the timers that fired during it. A failing
//! expectation panics with the whole transcript of the scenario.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::{Decision, Engine, Env, KeyDirection, KeyEvent, TimerRequest, sort_by_specificity};
use crate::config::from_ron_str;
use crate::core::windows_actions::BindAction;
use crate::data::binding::Binding;
use crate::data::vk_name::parse_vk;

/// An [`Env`] with a hand-driven clock, a modelled OS key state and a
/// configurable foreground window.
pub(crate) struct FakeEnv {
    now: Instant,
    /// What the OS believes is held, by physical (sided) code.
    os_down: [bool; 256],
    app: Option<String>,
    title: Option<String>,
}

impl FakeEnv {
    pub(crate) fn new() -> Self {
        Self {
            now: Instant::now(),
            os_down: [false; 256],
            app: None,
            title: None,
        }
    }

    fn os_down(&self, vk: VIRTUAL_KEY) -> bool {
        self.os_down.get(vk.0 as usize).copied().unwrap_or(false)
    }
}

impl Env for FakeEnv {
    fn now(&self) -> Instant {
        self.now
    }

    fn os_key_down(&self, vk: VIRTUAL_KEY) -> bool {
        match vk {
            VK_SHIFT => self.os_down(VK_LSHIFT) || self.os_down(VK_RSHIFT),
            VK_CONTROL => self.os_down(VK_LCONTROL) || self.os_down(VK_RCONTROL),
            VK_MENU => self.os_down(VK_LMENU) || self.os_down(VK_RMENU),
            other => self.os_down(other),
        }
    }

    fn foreground_app(&self) -> Option<&str> {
        self.app.as_deref()
    }

    fn foreground_title(&self) -> Option<&str> {
        self.title.as_deref()
    }
}

/// The physical key a scenario presses for `name`.
fn physical_key(name: &str) -> VIRTUAL_KEY {
    let vk = parse_vk(name).unwrap_or_else(|| panic!("scenario: unknown key name {name:?}"));
    let vk = match vk {
        VK_SHIFT => VK_LSHIFT,
        VK_CONTROL => VK_LCONTROL,
        VK_MENU => VK_LMENU,
        other => other,
    };
    assert!(vk.0 < 256, "scenario: key {name:?} has no 8-bit code");
    vk
}

/// One step of a scenario and what the engine decided for it.
struct Step {
    label: String,
    /// `None` for steps that are not key events (`advance_ms`).
    suppressed: Option<bool>,
    fired: Vec<BindAction>,
}

/// A scripted run of the engine. See the [module documentation](self).
pub(crate) struct Scenario {
    engine: Engine,
    bindings: Vec<Binding>,
    env: FakeEnv,
    start: Instant,
    /// Keys the scenario holds physically, to catch scripts that release a
    /// key they never pressed or repeat a key that is up.
    held: [bool; 256],
    timers: Vec<TimerRequest>,
    steps: Vec<Step>,
}

impl Scenario {
    /// Loads `text` like a `config.ron` (auto-blockers included, so avoid
    /// `block_original_combo`, which reads the registry) and starts at +0 ms
    /// with nothing held and no foreground window.
    pub(crate) fn from_ron(text: &str) -> Self {
        let bindings =
            from_ron_str(text).unwrap_or_else(|e| panic!("scenario: config does not load: {e}"));
        Self::with_bindings(bindings)
    }

    /// Starts from a ready-made binding list, sorted like a hot reload does.
    pub(crate) fn with_bindings(mut bindings: Vec<Binding>) -> Self {
        sort_by_specificity(&mut bindings);
        let env = FakeEnv::new();
        Self {
            engine: Engine::new(),
            bindings,
            start: env.now,
            env,
            held: [false; 256],
            timers: Vec::new(),
            steps: Vec::new(),
        }
    }

    /// Sets the foreground window that `when:` conditions see.
    pub(crate) fn foreground(mut self, app: &str, title: &str) -> Self {
        self.env.app = Some(app.to_owned());
        self.env.title = Some(title.to_owned());
        self
    }

    /// Presses `key`, which must be up.
    #[track_caller]
    pub(crate) fn down(self, key: &str) -> Self {
        if self.is_held(key) {
            self.fail(&format!("down {key}: the key is already held; use repeat"));
        }
        self.key_event(key, KeyDirection::Down, "down")
    }

    /// Sends an auto-repeat key-down for `key`, which must be held.
    #[track_caller]
    pub(crate) fn repeat(self, key: &str) -> Self {
        if !self.is_held(key) {
            self.fail(&format!("repeat {key}: the key is not held"));
        }
        self.key_event(key, KeyDirection::Down, "repeat")
    }

    /// Releases `key`, which must be held.
    #[track_caller]
    pub(crate) fn up(self, key: &str) -> Self {
        if !self.is_held(key) {
            self.fail(&format!("up {key}: the key is not held"));
        }
        self.key_event(key, KeyDirection::Up, "up")
    }

    /// Releases `key` without the hook seeing it, as when the key-up goes to
    /// the secure desktop: the OS key state changes, the engine is not told.
    #[track_caller]
    pub(crate) fn up_unseen(mut self, key: &str) -> Self {
        if !self.is_held(key) {
            self.fail(&format!("up_unseen {key}: the key is not held"));
        }
        let vk = physical_key(key);
        self.held[vk.0 as usize] = false;
        self.env.os_down[vk.0 as usize] = false;
        let label = format!("up {key} unseen by the hook at {}", self.clock());
        self.steps.push(Step {
            label,
            suppressed: None,
            fired: Vec::new(),
        });
        self
    }

    /// Reports a foreground window change to the engine.
    pub(crate) fn foreground_changed(mut self) -> Self {
        self.engine.on_foreground_changed(&self.env);
        let label = format!("foreground change at {}", self.clock());
        self.steps.push(Step {
            label,
            suppressed: None,
            fired: Vec::new(),
        });
        self
    }

    /// Moves the clock forward by `ms`, firing every timer that falls due on
    /// the way, earliest first, with the clock set to its due time.
    pub(crate) fn advance_ms(mut self, ms: u64) -> Self {
        let target = self.env.now + Duration::from_millis(ms);
        let mut fired = Vec::new();
        while let Some(index) = self
            .timers
            .iter()
            .enumerate()
            .filter(|(_, timer)| timer.due <= target)
            .min_by_key(|(_, timer)| timer.due)
            .map(|(index, _)| index)
        {
            let timer = self.timers.remove(index);
            self.env.now = self.env.now.max(timer.due);
            let decision = self.engine.on_timer(timer.token);
            assert!(!decision.suppress, "a timer has no event to suppress");
            self.timers.extend(decision.timers);
            fired.extend(decision.fire);
        }
        self.env.now = target;
        let label = format!("advance {ms} ms to {}", self.clock());
        self.steps.push(Step {
            label,
            suppressed: None,
            fired,
        });
        self
    }

    /// Moves the clock forward by `ms` without firing any timer, as when a
    /// timer thread wakes late. The next [`advance_ms`](Self::advance_ms)
    /// fires every timer that fell due in the meantime, at its own clock.
    pub(crate) fn advance_ms_stalled(mut self, ms: u64) -> Self {
        self.env.now += Duration::from_millis(ms);
        let label = format!("advance {ms} ms to {}, timers stalled", self.clock());
        self.steps.push(Step {
            label,
            suppressed: None,
            fired: Vec::new(),
        });
        self
    }

    /// Drops every timer that has not fired yet, as when the caller cannot
    /// start them, and tells the engine through [`Engine::forget_timer`].
    pub(crate) fn lose_timers(mut self) -> Self {
        for timer in std::mem::take(&mut self.timers) {
            self.engine.forget_timer(timer.token);
        }
        let label = format!("timers lost at {}", self.clock());
        self.steps.push(Step {
            label,
            suppressed: None,
            fired: Vec::new(),
        });
        self
    }

    /// The last key event was swallowed.
    #[track_caller]
    pub(crate) fn expect_suppressed(self) -> Self {
        self.expect_suppression(true)
    }

    /// The last key event was passed on to the OS.
    #[track_caller]
    pub(crate) fn expect_passed(self) -> Self {
        self.expect_suppression(false)
    }

    /// The last step fired exactly `expected`, in order.
    #[track_caller]
    pub(crate) fn expect_fired(self, expected: &[BindAction]) -> Self {
        let step = self.last_step();
        if step.fired != expected {
            let message = format!(
                "{}: expected to fire {expected:?}, fired {:?}",
                step.label, step.fired
            );
            self.fail(&message);
        }
        self
    }

    /// The last step fired nothing.
    #[track_caller]
    pub(crate) fn expect_nothing_fired(self) -> Self {
        self.expect_fired(&[])
    }

    /// Every step so far fired exactly `expected` between them, in order.
    #[track_caller]
    pub(crate) fn expect_total_fired(self, expected: &[BindAction]) -> Self {
        let total: Vec<&BindAction> = self.steps.iter().flat_map(|s| &s.fired).collect();
        let expected: Vec<&BindAction> = expected.iter().collect();
        if total != expected {
            self.fail(&format!(
                "expected the scenario to fire {expected:?} in total, it fired {total:?}"
            ));
        }
        self
    }

    fn key_event(mut self, key: &str, direction: KeyDirection, what: &str) -> Self {
        let vk = physical_key(key);
        let decision = self
            .engine
            .on_key(KeyEvent { vk, direction }, &self.bindings, &self.env);
        let down = direction == KeyDirection::Down;
        // The OS only learns about events the hook passes on.
        if !decision.suppress {
            self.env.os_down[vk.0 as usize] = down;
        }
        self.held[vk.0 as usize] = down;
        let label = format!("{what} {key} at {}", self.clock());
        self.record(label, decision);
        self
    }

    fn record(&mut self, label: String, decision: Decision) {
        self.timers.extend(decision.timers);
        self.steps.push(Step {
            label,
            suppressed: Some(decision.suppress),
            fired: decision.fire,
        });
    }

    fn is_held(&self, key: &str) -> bool {
        self.held[physical_key(key).0 as usize]
    }

    fn clock(&self) -> String {
        format!("+{} ms", (self.env.now - self.start).as_millis())
    }

    #[track_caller]
    fn last_step(&self) -> &Step {
        match self.steps.last() {
            Some(step) => step,
            None => self.fail("no step to check yet"),
        }
    }

    #[track_caller]
    fn expect_suppression(self, expected: bool) -> Self {
        let step = self.last_step();
        let word = |s: bool| if s { "suppressed" } else { "passed" };
        match step.suppressed {
            Some(actual) if actual == expected => {}
            Some(actual) => {
                let message = format!(
                    "{}: expected the event to be {}, it was {}",
                    step.label,
                    word(expected),
                    word(actual)
                );
                self.fail(&message);
            }
            None => {
                let message = format!("{}: not a key event, nothing to suppress", step.label);
                self.fail(&message);
            }
        }
        self
    }

    #[track_caller]
    fn fail(&self, message: &str) -> ! {
        let mut transcript = String::new();
        for step in &self.steps {
            let outcome = match step.suppressed {
                Some(true) => "suppressed",
                Some(false) => "passed",
                None => "-",
            };
            let _ = writeln!(
                transcript,
                "  {}: {outcome}, fired {:?}",
                step.label, step.fired
            );
        }
        panic!("scenario: {message}\ntranscript:\n{transcript}");
    }
}
