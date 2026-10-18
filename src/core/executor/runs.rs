#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::{Arc, Mutex, PoisonError};
    use std::time::{Duration, Instant};

    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;
    use crate::core::executor::{Runner, TargetWindow};
    use crate::core::windows_actions::BindAction;
    use crate::data::sequence::SequenceStep;

    /// A fake clock: the base instant plus an offset that the test, or the
    /// runner inside a call, moves forward.
    type Offset = Arc<Mutex<Duration>>;

    /// Records the `Display` text of every call; panics on `panic_on`
    /// before recording, and moves the clock by `advance` inside the call
    /// named in it.
    struct Recorder {
        log: Vec<String>,
        panic_on: Option<&'static str>,
        advance: Option<(&'static str, Duration, Offset)>,
    }

    impl Recorder {
        fn call(&mut self, text: String) {
            if let Some((on, by, offset)) = &self.advance
                && *on == text
            {
                *offset.lock().unwrap_or_else(PoisonError::into_inner) += *by;
            }
            if self.panic_on == Some(text.as_str()) {
                panic!("the recorder panics on {text}");
            }
            self.log.push(text);
        }
    }

    impl Runner for Recorder {
        fn action(&mut self, action: &BindAction) {
            self.call(action.to_string());
        }

        fn step(&mut self, step: &SequenceStep, _target: TargetWindow) {
            self.call(step.to_string());
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
                runner: Recorder {
                    log: Vec::new(),
                    panic_on: None,
                    advance: None,
                },
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

        /// The calls recorded since the last `taken`.
        fn taken(&mut self) -> Vec<String> {
            std::mem::take(&mut self.runner.log)
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
        assert_eq!(launched, Calls { made: 1, panics: 0 });
        assert_eq!(opened, Calls { made: 1, panics: 0 });
        assert_eq!(fixture.runs.len(), 0);
    }
}
