mod runs;
#[cfg(test)]
pub(crate) mod testing;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::mpsc::{self, Receiver};
    use std::thread;
    use std::time::{Duration, Instant};

    use windows::Win32::System::Com::{APTTYPE, APTTYPE_STA, APTTYPEQUALIFIER, CoGetApartmentType};
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    use windows::Win32::UI::WindowsAndMessaging::USER_TIMER_MAXIMUM;

    use super::runs::{Ctx, Job, Next, Runs};
    use super::testing::{FakeRunner, Ran, Record, leaked_counters};
    use super::*;
    use crate::core::windows_actions::BindAction;
    use crate::data::sequence::SequenceStep;
    use crate::logging::counters::{Counter, Counters};

    /// How long a test waits for something the executor should do at once.
    const WAIT: Duration = Duration::from_secs(2);

    fn sequence(steps: Vec<SequenceStep>) -> BindAction {
        BindAction::Sequence(Arc::new(steps))
    }

    fn key(vk: VIRTUAL_KEY) -> SequenceStep {
        SequenceStep::PressKey(vk)
    }

    fn delay(ms: u64) -> SequenceStep {
        SequenceStep::Delay(ms)
    }

    fn open(url: &str) -> BindAction {
        BindAction::OpenUrl(url.to_owned())
    }

    /// A real executor thread with `runner` and counters of its own.
    fn start(runner: FakeRunner, capacity: usize) -> (Executor, &'static Counters) {
        let counters = leaked_counters();
        let executor = Executor::start(runner, capacity, counters).expect("the executor starts");
        (executor, counters)
    }

    fn next(records: &Receiver<Record>) -> Record {
        records
            .recv_timeout(WAIT)
            .expect("the runner is called within 2 s")
    }

    /// Waits up to 2 s for `condition`, and fails naming `what`.
    fn wait_until(what: &str, condition: impl Fn() -> bool) {
        let begun = Instant::now();
        while !condition() {
            assert!(begun.elapsed() < WAIT, "{what}: not within 2 s");
            thread::sleep(Duration::from_millis(1));
        }
    }

    /// The counts since the last call, each with its counter.
    fn take_counts(counters: &Counters) -> Vec<(Counter, u64)> {
        Counter::ALL.into_iter().zip(counters.take()).collect()
    }

    fn count_of(counts: &[(Counter, u64)], counter: Counter) -> u64 {
        counts
            .iter()
            .find(|(each, _)| *each == counter)
            .map_or(0, |(_, count)| *count)
    }

    #[test]
    fn a_hundred_sequence_runs_use_the_one_executor_thread() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();

        for _ in 0..100 {
            assert_eq!(
                handle.submit(sequence(vec![key(VK_F13)]), TargetWindow::NONE),
                Submitted::Queued
            );
            let record = next(&records);
            assert_eq!(record.text(), "key F13");
            assert_eq!(record.thread, executor.thread().id());
            assert_ne!(record.thread, thread::current().id());
            assert_eq!(record.name.as_deref(), Some("action-executor"));
        }
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn delay_10000_stops_within_100_ms_of_an_epoch_change() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![delay(10_000), key(VK_F14)]),
            TargetWindow::NONE,
        );
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        let t0 = Instant::now();
        handle.cancel_pending();
        wait_until("the cancelled macro is gone", || handle.active_runs() == 0);
        let stopped = t0.elapsed();
        assert!(
            stopped < Duration::from_millis(100),
            "the Delay stopped {stopped:?} after the epoch change"
        );

        handle.submit(sequence(vec![key(VK_F15)]), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "key F15", "F14 never runs");
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_recv().is_err());
    }

    #[test]
    fn a_delay_5000_macro_holds_up_no_other_job() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![key(VK_F13), delay(5_000), key(VK_F14)]),
            TargetWindow::NONE,
        );
        assert_eq!(next(&records).text(), "key F13");
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        let t0 = Instant::now();
        handle.submit(open("https://example.com"), TargetWindow::NONE);
        let opened = next(&records);
        assert_eq!(opened.text(), "open https://example.com");
        let waited = opened.at.duration_since(t0);
        assert!(
            waited < Duration::from_millis(100),
            "the OpenUrl waited {waited:?} behind the Delay"
        );

        handle.cancel_pending();
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_recv().is_err(), "F14 never runs");
    }

    #[test]
    fn two_macros_fired_together_both_run_in_their_own_order() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![key(VK_F13), delay(300), key(VK_F14)]),
            TargetWindow::NONE,
        );
        handle.submit(
            sequence(vec![key(VK_F15), delay(300), key(VK_F16)]),
            TargetWindow::NONE,
        );

        let texts: Vec<String> = (0..4).map(|_| next(&records).text().to_owned()).collect();
        assert_eq!(texts, ["key F13", "key F15", "key F14", "key F16"]);
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_cancelled_waiting_run_never_resumes() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![key(VK_F13), delay(150), key(VK_F14)]),
            TargetWindow::NONE,
        );
        assert_eq!(next(&records).text(), "key F13");
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        handle.cancel_pending();
        thread::sleep(Duration::from_millis(400));
        assert!(records.try_recv().is_err(), "F14 must not run");

        handle.submit(sequence(vec![key(VK_F15)]), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "key F15");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_delay_resumes_its_run_after_its_deadline() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        executor.handle().submit(
            sequence(vec![key(VK_F13), delay(50), key(VK_F14)]),
            TargetWindow::NONE,
        );

        let first = next(&records);
        let second = next(&records);
        assert_eq!(second.text(), "key F14");
        let between = second.at.duration_since(first.at);
        assert!(between >= Duration::from_millis(50), "{between:?}");
        assert!(between < Duration::from_secs(1), "{between:?}");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_delay_of_u64_max_waits_until_cancelled() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        handle.submit(
            sequence(vec![delay(u64::MAX), key(VK_F14)]),
            TargetWindow::NONE,
        );
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);

        thread::sleep(Duration::from_millis(50));
        assert!(records.try_recv().is_err(), "nothing runs");
        assert!(!executor.is_finished(), "the executor still runs");
        assert_eq!(handle.active_runs(), 1);

        handle.cancel_pending();
        handle.submit(sequence(vec![key(VK_F15)]), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "key F15");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_delay_of_u64_max_arms_no_timer_longer_than_u32_max_minus_1_ms() {
        let (tx, records) = mpsc::channel();
        let mut runner = FakeRunner::new(tx);
        let base = Instant::now();
        let mut runs = Runs::new();
        let live = |_: u64| true;
        let now = || base;
        let mut ctx = Ctx {
            runner: &mut runner,
            live: &live,
            now: &now,
        };
        runs.start(
            Job {
                action: sequence(vec![delay(u64::MAX), key(VK_F14)]),
                target: TargetWindow::NONE,
                epoch: 0,
            },
            &mut ctx,
        );
        let polled = runs.poll(&mut ctx);

        match polled.next {
            Next::Idle => {}
            Next::At(at) => {
                let ms = timer_ms(base, at);
                assert_eq!(ms, USER_TIMER_MAXIMUM);
                assert!(ms <= u32::MAX - 1);
                assert!(
                    matches!(plan(None, polled.next, base, false), Plan::Arm { ms, .. } if ms == USER_TIMER_MAXIMUM),
                    "the timer is armed for at most USER_TIMER_MAXIMUM ms"
                );
            }
        }

        let a_day_later = base + Duration::from_secs(86_400);
        let later = || a_day_later;
        let mut ctx = Ctx {
            runner: &mut runner,
            live: &live,
            now: &later,
        };
        let polled = runs.poll(&mut ctx);
        assert_eq!(polled.calls.made, 0);
        assert!(records.try_recv().is_err(), "nothing runs a day later");
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn timer_ms_rounds_up_and_clamps() {
        let base = Instant::now();
        assert_eq!(timer_ms(base, base + Duration::from_micros(400)), 1);
        assert_eq!(timer_ms(base, base + Duration::from_micros(10_200)), 11);
        assert_eq!(timer_ms(base, base), 1);
        assert_eq!(timer_ms(base + Duration::from_millis(5), base), 1);
        assert_eq!(
            timer_ms(base, base + Duration::from_secs(30 * 86_400)),
            USER_TIMER_MAXIMUM
        );
    }

    #[test]
    fn the_delay_timer_is_kept_only_while_nothing_ran() {
        // Code that ran on the executor thread (a runner call, a dispatched
        // message) may have pumped its queue and eaten the armed timer's
        // WM_TIMER; only a timer nothing could have touched is kept.
        let now = Instant::now();
        let at = now + Duration::from_millis(100);
        let sooner = now + Duration::from_millis(50);

        assert_eq!(plan(None, Next::Idle, now, false), Plan::Kill);
        assert_eq!(plan(Some(at), Next::Idle, now, true), Plan::Kill);
        assert_eq!(plan(Some(at), Next::At(now), now, false), Plan::Due);
        assert_eq!(
            plan(None, Next::At(at), now, false),
            Plan::Arm { ms: 100, at }
        );
        assert_eq!(
            plan(Some(at), Next::At(at), now, false),
            Plan::Keep,
            "nothing ran: the armed timer stands"
        );
        assert_eq!(
            plan(Some(at), Next::At(at), now, true),
            Plan::Arm { ms: 100, at },
            "something ran: the timer is made again"
        );
        assert_eq!(
            plan(Some(at), Next::At(sooner), now, false),
            Plan::Arm { ms: 50, at: sooner }
        );
    }

    /// The enqueue times of 20 `OpenUrl` jobs on a fresh executor whose
    /// runner blocks 500 ms per call. The executor is stopped before this
    /// returns, after at most one call.
    fn twenty_enqueue_times() -> Vec<Duration> {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(
            FakeRunner::new(tx).with_block(Duration::from_millis(500)),
            64,
        );
        let handle = executor.handle();
        let times = (0..20)
            .map(|_| {
                let begun = Instant::now();
                let submitted = handle.submit(open("https://example.com"), TargetWindow::NONE);
                let took = begun.elapsed();
                assert_eq!(submitted, Submitted::Queued);
                took
            })
            .collect();
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_iter().count() <= 1, "the stop drops the queue");
        times
    }

    #[test]
    fn twenty_slow_jobs_each_enqueue_in_under_5_ms() {
        // The rule of the lane: every one of the 20 under 5 ms, and one
        // retry of the whole batch for a scheduler hiccup.
        let limit = Duration::from_millis(5);
        let first = twenty_enqueue_times();
        if first.iter().all(|took| *took < limit) {
            return;
        }
        let second = twenty_enqueue_times();
        assert!(
            second.iter().all(|took| *took < limit),
            "an enqueue took 5 ms or more twice: {first:?}, then {second:?}"
        );
    }

    #[test]
    fn a_full_queue_drops_the_job_and_counts_it() {
        let (tx, records) = mpsc::channel();
        let (started_tx, started) = mpsc::channel();
        let (release, release_rx) = mpsc::channel::<()>();
        let (executor, counters) = start(FakeRunner::new(tx).with_gate(started_tx, release_rx), 1);
        let handle = executor.handle();

        assert_eq!(
            handle.submit(open("A"), TargetWindow::NONE),
            Submitted::Queued
        );
        started.recv_timeout(WAIT).expect("A runs");
        assert_eq!(
            handle.submit(open("B"), TargetWindow::NONE),
            Submitted::Queued
        );
        assert_eq!(
            handle.submit(open("C"), TargetWindow::NONE),
            Submitted::Dropped
        );
        let counts = take_counts(counters);
        assert_eq!(
            count_of(&counts, Counter::ExecutorJobsDropped),
            1,
            "{counts:?}"
        );

        drop(release);
        assert_eq!(next(&records).text(), "open A");
        assert_eq!(next(&records).text(), "open B");
        assert_eq!(executor.stop(), Stopped::Joined);
        assert!(records.try_recv().is_err(), "C never runs");
    }

    #[test]
    fn a_job_queued_before_an_epoch_change_does_not_run() {
        let (tx, records) = mpsc::channel();
        let (started_tx, started) = mpsc::channel();
        let (release, release_rx) = mpsc::channel::<()>();
        let (executor, _) = start(
            FakeRunner::new(tx).with_gate(started_tx, release_rx),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();

        handle.submit(open("A"), TargetWindow::NONE);
        started.recv_timeout(WAIT).expect("A runs");
        handle.submit(open("B"), TargetWindow::NONE);
        handle.cancel_pending();
        drop(release);
        handle.submit(open("C"), TargetWindow::NONE);

        assert_eq!(next(&records).text(), "open A");
        assert_eq!(next(&records).text(), "open C", "B never runs");
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn jobs_run_in_the_order_they_were_queued_with_their_target() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();
        let target = TargetWindow::from_raw(0x1234);

        handle.submit(
            BindAction::Launch {
                exe: "no-such.exe".to_owned(),
                args: Vec::new(),
            },
            TargetWindow::NONE,
        );
        handle.submit(open("https://example.com"), TargetWindow::NONE);
        handle.submit(sequence(vec![key(VK_F13), key(VK_F14)]), target);

        let ran: Vec<Ran> = (0..4).map(|_| next(&records).ran).collect();
        assert_eq!(
            ran,
            [
                Ran::Action("launch no-such.exe".to_owned()),
                Ran::Action("open https://example.com".to_owned()),
                Ran::Step("key F13".to_owned(), target),
                Ran::Step("key F14".to_owned(), target),
            ]
        );
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn stop_interrupts_a_delay_drops_the_queue_and_joins() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(
            FakeRunner::new(tx).with_block(Duration::from_millis(100)),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();
        handle.submit(
            sequence(vec![key(VK_F13), delay(10_000), key(VK_F14)]),
            TargetWindow::NONE,
        );
        assert_eq!(next(&records).text(), "key F13");
        wait_until("the macro waits in its Delay", || handle.active_runs() == 1);
        handle.submit(open("A"), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "open A");
        for url in ["B", "C", "D"] {
            handle.submit(open(url), TargetWindow::NONE);
        }

        let begun = Instant::now();
        assert_eq!(executor.stop(), Stopped::Joined);
        let took = begun.elapsed();
        assert!(took < Duration::from_secs(1), "the stop took {took:?}");
        assert!(records.try_recv().is_err(), "nothing after A runs");
    }

    #[test]
    fn a_submit_after_stop_is_closed_and_not_counted() {
        let (tx, _records) = mpsc::channel();
        let (executor, counters) = start(FakeRunner::new(tx), QUEUE_CAPACITY);
        let handle = executor.handle();

        assert_eq!(executor.stop(), Stopped::Joined);

        assert_eq!(
            handle.submit(open("A"), TargetWindow::NONE),
            Submitted::Closed
        );
        let counts = take_counts(counters);
        assert!(counts.iter().all(|(_, count)| *count == 0), "{counts:?}");
    }

    #[test]
    fn an_executor_ended_by_wm_quit_counts_every_later_submit() {
        let (tx, records) = mpsc::channel();
        let (executor, counters) = start(
            FakeRunner::new(tx).with_quit_on("open quit"),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();

        handle.submit(open("quit"), TargetWindow::NONE);
        assert_eq!(next(&records).text(), "open quit");
        wait_until("the executor ends on WM_QUIT", || executor.is_finished());

        assert_eq!(
            handle.submit(open("later"), TargetWindow::NONE),
            Submitted::Unavailable
        );
        let counts = take_counts(counters);
        assert_eq!(
            count_of(&counts, Counter::ExecutorUnavailable),
            1,
            "{counts:?}"
        );
        assert_eq!(
            count_of(&counts, Counter::ExecutorJobsDropped),
            0,
            "{counts:?}"
        );

        assert_eq!(executor.stop(), Stopped::Joined);
        assert_eq!(
            handle.submit(open("after stop"), TargetWindow::NONE),
            Submitted::Closed
        );
        let counts = take_counts(counters);
        assert!(counts.iter().all(|(_, count)| *count == 0), "{counts:?}");
        assert!(records.try_recv().is_err());
    }

    #[test]
    fn a_panicking_job_leaves_the_executor_running() {
        let (tx, records) = mpsc::channel();
        let (executor, _) = start(
            FakeRunner::new(tx).with_panic_on("open boom"),
            QUEUE_CAPACITY,
        );
        let handle = executor.handle();

        handle.submit(open("boom"), TargetWindow::NONE);
        handle.submit(open("next"), TargetWindow::NONE);

        assert_eq!(next(&records).text(), "open next");
        assert!(!executor.is_finished());
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    /// The apartment of the calling thread, or `None` without COM.
    fn apartment_type() -> Option<APTTYPE> {
        let mut kind = APTTYPE::default();
        let mut qualifier = APTTYPEQUALIFIER::default();
        // SAFETY: both pointers are to locals of this frame that outlive
        // the call, which only writes them; it changes no COM state.
        unsafe { CoGetApartmentType(&mut kind, &mut qualifier) }
            .ok()
            .map(|()| kind)
    }

    /// Reports the apartment type inside every `action`.
    struct ApartmentProbe(mpsc::Sender<Option<APTTYPE>>);

    impl Runner for ApartmentProbe {
        fn action(&mut self, _action: &BindAction) {
            let _ = self.0.send(apartment_type());
        }

        fn step(&mut self, _step: &SequenceStep, _target: TargetWindow) {}
    }

    #[test]
    fn the_executor_runs_jobs_in_a_single_threaded_apartment() {
        let (tx, kinds) = mpsc::channel();
        let executor = Executor::start(ApartmentProbe(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");

        executor
            .handle()
            .submit(open("https://example.com"), TargetWindow::NONE);

        let kind = kinds.recv_timeout(WAIT).expect("the probe runs");
        assert_eq!(kind, Some(APTTYPE_STA));
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn the_apartment_guard_uninitialises_com_on_drop() {
        let (inside, after) = thread::spawn(|| {
            let apartment = Apartment::enter_sta();
            let inside = apartment_type();
            drop(apartment);
            (inside, apartment_type())
        })
        .join()
        .expect("the scratch thread does not panic");

        assert_eq!(inside, Some(APTTYPE_STA));
        assert_ne!(
            after,
            Some(APTTYPE_STA),
            "after the guard the thread has no apartment of its own (or the implicit MTA)"
        );
    }

    #[test]
    #[ignore = "manual: opens https://example.com in the default browser through Win32Runner on a real executor thread; check that the page opens and the test passes"]
    fn the_win32_runner_opens_a_url() {
        let executor = Executor::start(Win32Runner, QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        assert_eq!(
            executor
                .handle()
                .submit(open("https://example.com"), TargetWindow::NONE),
            Submitted::Queued
        );
        thread::sleep(Duration::from_secs(3));
        assert_eq!(executor.stop(), Stopped::Joined);
    }
}
