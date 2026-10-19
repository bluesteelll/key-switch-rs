//! The hook watchdog (roadmap item 2.4, KS-08): it finds a keyboard hook
//! that Windows removed and has the main thread reinstall it.

mod decision;

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::Mutex;
    use std::sync::mpsc::{self, Receiver, SyncSender};
    use std::thread;
    use std::time::{Duration, Instant};

    use windows::Win32::Foundation::E_ACCESSDENIED;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_VOLUME_UP};

    use super::decision::SILENCE;
    use super::*;
    use crate::core::executor::testing::leaked_counters;
    use crate::hook::health::{HealthState, ReinstallOutcome, ReinstallRequest};
    use crate::hook::main_thread::MainThreadError;
    use crate::logging::counters::{Counter, Counters};

    // No test here registers for raw input or installs a hook: the thread
    // tests start the watchdog with synthetic raw events, which skip the
    // registration, and the loop tests drive its passes on this thread
    // with a fake timer and a fake clock.

    /// A requester that records each request with the time it came and
    /// keeps the reply open without answering, as a busy main thread does;
    /// or, made with `answering`, answers each at once.
    struct Recorder {
        sent: mpsc::Sender<(Instant, ReinstallRequest)>,
        waiting: Mutex<Vec<SyncSender<ReinstallOutcome>>>,
        answer: Option<ReinstallOutcome>,
    }

    impl Recorder {
        fn new() -> (Self, Receiver<(Instant, ReinstallRequest)>) {
            Self::answering(None)
        }

        fn answering(
            answer: Option<ReinstallOutcome>,
        ) -> (Self, Receiver<(Instant, ReinstallRequest)>) {
            let (sent, requests) = mpsc::channel();
            let recorder = Self {
                sent,
                waiting: Mutex::new(Vec::new()),
                answer,
            };
            (recorder, requests)
        }
    }

    impl RequestReinstall for Recorder {
        fn request(
            &self,
            request: ReinstallRequest,
        ) -> Result<Receiver<ReinstallOutcome>, MainThreadError> {
            let (reply, answer) = mpsc::sync_channel(1);
            if let Some(outcome) = self.answer {
                let _ = reply.try_send(outcome);
            }
            self.waiting
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(reply);
            let _ = self.sent.send((Instant::now(), request));
            Ok(answer)
        }
    }

    /// What the watchdog did to a [`FakeTimer`].
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum TimerCall {
        Set(Duration),
        Cancel,
    }

    /// A tick timer that Windows never sees: it records each call, and
    /// refuses as many sets as asked.
    #[derive(Default)]
    struct FakeTimer {
        calls: RefCell<Vec<TimerCall>>,
        refusals: std::cell::Cell<u32>,
    }

    impl TickTimer for FakeTimer {
        fn set(&self, after: Duration) -> windows::core::Result<()> {
            if self.refusals.get() > 0 {
                self.refusals.set(self.refusals.get() - 1);
                return Err(E_ACCESSDENIED.into());
            }
            self.calls.borrow_mut().push(TimerCall::Set(after));
            Ok(())
        }

        fn cancel(&self) -> windows::core::Result<()> {
            self.calls.borrow_mut().push(TimerCall::Cancel);
            Ok(())
        }
    }

    fn input_here() -> windows::core::Result<bool> {
        Ok(true)
    }

    fn input_unknown() -> windows::core::Result<bool> {
        Err(E_ACCESSDENIED.into())
    }

    /// The watchdog's loop on this thread, on a fake clock in whole
    /// milliseconds, with a fake timer that signals at its due time, or
    /// `early` ms before it the first time.
    struct Drive {
        ticker: Ticker<FakeTimer>,
        start: Instant,
        health: &'static HealthState,
        counters: &'static Counters,
        recorder: Recorder,
        sent: Receiver<(Instant, ReinstallRequest)>,
        input_here: fn() -> windows::core::Result<bool>,
        /// When the fake timer signals, in ms.
        due: Option<u64>,
        early: u64,
        /// When each request was sent, in ms.
        requests: Vec<u64>,
    }

    impl Drive {
        fn new(recorder: (Recorder, Receiver<(Instant, ReinstallRequest)>)) -> Self {
            let start = Instant::now();
            let health: &'static HealthState = Box::leak(Box::new(HealthState::new()));
            health.set_hook_installed(true);
            Self {
                ticker: Ticker::new(start, FakeTimer::default(), health),
                start,
                health,
                counters: leaked_counters(),
                recorder: recorder.0,
                sent: recorder.1,
                input_here,
                due: None,
                early: 0,
                requests: Vec::new(),
            }
        }

        fn at(&self, ms: u64) -> Instant {
            self.start + Duration::from_millis(ms)
        }

        /// One pass of the loop at `ms`, and what it did to the timer and
        /// the requester.
        fn pass(&mut self, ms: u64) {
            let context = Context {
                health: self.health,
                counters: self.counters,
                requests: &self.recorder,
                input_here: self.input_here,
            };
            self.ticker.pass(self.at(ms), &context);
            for call in self.ticker.timer.calls.borrow_mut().drain(..) {
                self.due = match call {
                    TimerCall::Set(after) => {
                        Some(ms + u64::try_from(after.as_millis()).expect("a short wait"))
                    }
                    TimerCall::Cancel => None,
                };
            }
            while self.sent.try_recv().is_ok() {
                self.requests.push(ms);
            }
        }

        /// Runs the clock from `from` to `to` ms: at each ms the raw events
        /// `raw` names, each followed by a pass, then the timer's signal, if
        /// it is due, followed by a pass.
        fn run(&mut self, from: u64, to: u64, raw: impl Fn(u64) -> bool) {
            for ms in from..=to {
                if raw(ms) {
                    let now = self.at(ms);
                    self.ticker.raw_event(now, self.health);
                    self.pass(ms);
                }
                if let Some(due) = self.due
                    && ms + self.early >= due
                {
                    self.early = 0;
                    self.due = None;
                    self.ticker.fired();
                    self.pass(ms);
                }
            }
        }
    }

    /// Raw key events every 30 ms from 0 to 300 ms, so that every tick
    /// after them is reached only through the timer.
    fn a_burst(ms: u64) -> bool {
        ms <= 300 && ms.is_multiple_of(30)
    }

    #[test]
    fn an_early_timer_signal_does_not_lose_the_tick() {
        // Ruling I1: a signal before the tick's due time is ignored by the
        // clock, and the pass sets the timer again for the same tick, as the
        // main loop's plan_wake does.
        let mut drive = Drive::new(Recorder::new());
        drive.early = 10;
        drive.run(0, 3000, a_burst);
        assert_eq!(drive.requests.len(), 1, "{:?}", drive.requests);
        assert!(
            (1000..=1500).contains(&drive.requests[0]),
            "{:?}",
            drive.requests
        );
    }

    #[test]
    fn a_timer_that_refuses_once_is_set_again_at_the_next_pass() {
        // Ruling I1: a failed set is counted and tried again at the next
        // pass, here the next raw event's.
        let mut drive = Drive::new(Recorder::new());
        drive.ticker.timer.refusals.set(1);
        drive.run(0, 3000, a_burst);
        assert_eq!(drive.counters.get(Counter::WatchdogTimerNotSet), 1);
        assert_eq!(drive.requests.len(), 1, "{:?}", drive.requests);
        assert!(
            (1000..=1500).contains(&drive.requests[0]),
            "{:?}",
            drive.requests
        );
    }

    #[test]
    fn a_reply_of_the_main_thread_is_taken_at_the_next_pass() {
        // The main thread answers each request at once with Reinstalled and
        // the callback stays silent: the answer frees the next request,
        // which waits for the doubled window.
        let mut drive = Drive::new(Recorder::answering(Some(ReinstallOutcome::Reinstalled)));
        drive.run(0, 6000, |ms| ms.is_multiple_of(30));
        assert_eq!(drive.requests.len(), 2, "{:?}", drive.requests);
        let gap = drive.requests[1] - drive.requests[0];
        assert!((2000..2600).contains(&gap), "{:?}", drive.requests);
    }

    #[test]
    fn raw_input_before_the_hook_is_installed_is_no_evidence() {
        // Ruling I3: the watchdog starts before the hooks, and counts raw
        // input only while the keyboard hook is installed, so a key typed
        // before `install` draws no reinstall of the new hook.
        let mut drive = Drive::new(Recorder::new());
        drive.health.set_hook_installed(false);
        drive.run(0, 600, |ms| ms.is_multiple_of(30));
        assert_eq!(drive.due, None, "no tick while no hook is installed");
        drive.health.set_hook_installed(true);
        drive.run(601, 4000, |ms| ms.is_multiple_of(30));
        assert_eq!(drive.requests.len(), 1, "{:?}", drive.requests);
        assert!(
            drive.requests[0] >= 610 + 1000,
            "the silence starts at the first raw event counted: {:?}",
            drive.requests
        );
    }

    #[test]
    fn a_failed_desktop_check_declines_and_is_counted() {
        // Ruling M2: a request declined because the desktop check failed
        // leaves a trace in the counters.
        let mut drive = Drive::new(Recorder::new());
        drive.input_here = input_unknown;
        drive.run(0, 3000, |ms| ms.is_multiple_of(30));
        assert_eq!(drive.requests, [] as [u64; 0]);
        assert!(
            drive.counters.get(Counter::WatchdogDesktopCheckFailed) >= 1,
            "the declined decision is counted"
        );
    }

    #[test]
    fn only_keyboard_events_the_hook_can_see_count() {
        assert!(qualifies(0x1E, VK_A.0), "A");
        assert!(!qualifies(0xFF, VK_A.0), "keyboard overrun");
        assert!(!qualifies(0x1E, 0), "no virtual key");
        assert!(!qualifies(0x1E, 255), "VK 255, which the hook never gets");
        assert!(
            qualifies(0, VK_VOLUME_UP.0),
            "a media key without a scan code"
        );
    }

    /// A health state of its own, with the keyboard hook taken as installed.
    fn leaked_health() -> &'static HealthState {
        let health: &'static HealthState = Box::leak(Box::new(HealthState::new()));
        health.set_hook_installed(true);
        health
    }

    #[test]
    fn the_watchdog_thread_starts_and_stops_without_raw_input() {
        let (recorder, _requests) = Recorder::new();
        let watchdog = HookWatchdog::start_synthetic(leaked_health(), recorder, leaked_counters())
            .expect("the watchdog thread starts");
        let begun = Instant::now();
        assert!(watchdog.stop(), "the thread is joined within 1 s");
        assert!(begun.elapsed() < HookWatchdog::STOP_TIMEOUT);
    }

    /// Posts a synthetic raw event every 20 ms for 3 s, calling `before`
    /// right before each, and returns the requests with when they came,
    /// from the first raw event.
    fn three_seconds_of_raw_events(
        health: &'static HealthState,
        before: impl Fn(),
    ) -> Vec<(Duration, ReinstallRequest)> {
        let (recorder, requests) = Recorder::new();
        let watchdog = HookWatchdog::start_synthetic(health, recorder, leaked_counters())
            .expect("the watchdog thread starts");
        let first = Instant::now();
        while first.elapsed() < Duration::from_secs(3) {
            before();
            watchdog.post_synthetic_raw();
            thread::sleep(Duration::from_millis(20));
        }
        assert!(watchdog.stop(), "the thread is joined within 1 s");
        requests
            .try_iter()
            .map(|(at, request)| (at.saturating_duration_since(first), request))
            .collect()
    }

    #[test]
    fn synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request() {
        // Roadmap item 2.4 below the hook: the thread, its loop and its
        // timer on the real clock. The requester never answers, so the
        // request stays outstanding: exactly one.
        let requests = three_seconds_of_raw_events(leaked_health(), || {});
        assert_eq!(requests.len(), 1, "{requests:?}");
        let (after, request) = requests[0];
        assert!(
            (Duration::from_secs(1)..=Duration::from_secs(2)).contains(&after),
            "requested {after:?} after the first raw event"
        );
        assert_eq!(request.seen, 0);
        assert!(request.silent_for >= SILENCE, "{request:?}");
    }

    #[test]
    fn synthetic_raw_events_with_a_live_heartbeat_send_no_request() {
        let health = leaked_health();
        let requests = three_seconds_of_raw_events(health, || health.callback_seen());
        assert_eq!(requests, []);
    }

    /// The non-test lines of `text`: those before its `#[cfg(test)]` and
    /// `mod tests {` pair, or every line if it has none.
    fn non_test_lines(text: &str) -> Vec<&str> {
        let lines: Vec<&str> = text.lines().collect();
        let end = lines
            .windows(2)
            .position(|pair| pair[0].trim() == "#[cfg(test)]" && pair[1].trim() == "mod tests {")
            .unwrap_or(lines.len());
        lines[..end].to_vec()
    }

    /// The non-test lines of the first function whose line contains
    /// `signature`, until its braces balance. Panics if there is none, so a
    /// renamed function fails the scan instead of passing it.
    fn fn_lines<'a>(lines: &[&'a str], signature: &str) -> Vec<&'a str> {
        let start = lines
            .iter()
            .position(|line| line.contains(signature))
            .unwrap_or_else(|| panic!("no function `{signature}`"));
        let mut depth = 0_i64;
        for (index, line) in lines.iter().enumerate().skip(start) {
            depth += line.matches('{').count() as i64 - line.matches('}').count() as i64;
            if depth == 0 && line.contains('}') {
                return lines[start..=index].to_vec();
            }
        }
        panic!("the function `{signature}` does not end");
    }

    fn source(file: &str) -> String {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(root.join(file)).expect("read a source file")
    }

    #[test]
    fn the_watchdog_injects_no_input() {
        // Roadmap item 2.4's acceptance "ten minutes of idle time produce no
        // injected input": the watchdog and the reinstall name no way to
        // inject input. The tokens are assembled, so this test does not
        // match itself.
        let tokens = [
            concat!("Send", "Input"),
            concat!("keybd", "_event"),
            concat!("mouse", "_event"),
            concat!("KEYBD", "INPUT"),
            concat!("INPUT", "_KEYBOARD"),
        ];
        let mut scanned: Vec<(String, Vec<String>)> = Vec::new();
        for file in [
            "src/hook/watchdog/mod.rs",
            "src/hook/watchdog/decision.rs",
            "src/hook/health.rs",
            "src/hook/installer.rs",
        ] {
            let text = source(file);
            let lines = non_test_lines(&text);
            assert!(lines.len() > 10, "{file} was read");
            scanned.push((
                file.to_owned(),
                lines.iter().map(|line| (*line).to_owned()).collect(),
            ));
        }
        for (file, signature) in [
            ("src/hook/keyboard_hook.rs", "fn reinstall_keyboard_hook("),
            (
                "src/hook/keyboard_hook.rs",
                "fn reinstall_keyboard_hook_with(",
            ),
            ("src/hook/main_thread.rs", "fn reinstall_if_silent("),
            ("src/hook/main_thread.rs", "fn dispatch_sent_messages("),
        ] {
            let text = source(file);
            let lines = non_test_lines(&text);
            scanned.push((
                format!("{file} {signature}"),
                fn_lines(&lines, signature)
                    .iter()
                    .map(|line| (*line).to_owned())
                    .collect(),
            ));
        }
        let mut sites = Vec::new();
        for (what, lines) in &scanned {
            for line in lines {
                if tokens.iter().any(|token| line.contains(token)) {
                    sites.push(format!("{what}: {}", line.trim()));
                }
            }
        }
        assert!(sites.is_empty(), "input injection:\n{}", sites.join("\n"));
    }

    #[test]
    fn no_key_data_leaves_the_raw_input_read() {
        // Roadmap item 2.4: no key data from raw input is kept or logged.
        // The fields that name a key are read only where the event is
        // classified, and no log line names them.
        let fields = [concat!("V", "Key"), concat!("Make", "Code")];
        let text = source("src/hook/watchdog/mod.rs");
        let lines = non_test_lines(&text);
        let mut allowed = fn_lines(&lines, "fn raw_key_event(");
        allowed.extend(fn_lines(&lines, "fn qualifies("));
        let mut found = 0;
        let mut sites = Vec::new();
        for line in &lines {
            if fields.iter().any(|field| line.contains(field)) {
                found += 1;
                let logs = line.contains(concat!("log", "::"));
                if logs || !allowed.contains(line) {
                    sites.push(line.trim().to_owned());
                }
            }
        }
        assert!(found > 0, "the raw input read names the key fields");
        assert!(
            sites.is_empty(),
            "key data outside the read:\n{}",
            sites.join("\n")
        );
    }
}
