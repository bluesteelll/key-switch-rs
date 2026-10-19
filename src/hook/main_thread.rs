#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::sync::{Arc, mpsc};
    use std::thread;
    use std::time::{Duration, Instant};

    use windows::Win32::{
        Foundation::*,
        UI::{Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    };
    use windows::core::w;

    use super::*;
    use crate::core::executor::testing::{FakeRunner, Record, leaked_counters};
    use crate::core::executor::{Executor, QUEUE_CAPACITY, Stopped};
    use crate::core::windows_actions::BindAction;
    use crate::data::binding::Binding;
    use crate::data::key_combination::KeyCombination;
    use crate::data::sequence::SequenceStep;
    use crate::data::trigger::Trigger;
    use crate::engine::{Engine, Env, KeyEvent, sort_by_specificity};
    use crate::hook::keyboard_hook::KeyboardHook;
    use crate::hook::keyboard_hook_callback::decide_event;

    // No test installs a hook or injects input. The loop tests run
    // `MainThread::run` on a thread of their own with a local hook, send
    // only `WM_NULL` and `WM_CLOSE` to a message-only window of that thread,
    // and every action is `DoNothing` or a macro that a recording runner
    // takes; a fast action is only recorded under test anyway.

    /// An [`Env`] stopped at one instant with nothing held and no
    /// foreground window.
    struct QuietEnv(Instant);

    impl Env for QuietEnv {
        fn now(&self) -> Instant {
            self.0
        }
        fn os_key_down(&self, _vk: VIRTUAL_KEY) -> bool {
            false
        }
        fn foreground_app(&self) -> Option<&str> {
            None
        }
        fn foreground_title(&self) -> Option<&str> {
            None
        }
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// A local hook with `binding` on F13, which allows extra modifiers: the
    /// adapter reads the live key state of this machine.
    fn hook_with(binding: Binding) -> KeyboardHook {
        let hook = KeyboardHook::new();
        hook.update_bindings(vec![binding.with_allow_extra_modifiers(true)]);
        hook
    }

    fn f13_hold(term_ms: u64, action: BindAction) -> Binding {
        Binding::new(KeyCombination::new(VK_F13), action).with_trigger(Trigger::Hold(term_ms))
    }

    fn key_macro() -> BindAction {
        BindAction::Sequence(Arc::new(vec![SequenceStep::PressKey(VK_F13)]))
    }

    fn f13(msg: u32) -> (WPARAM, u32) {
        (WPARAM(msg as usize), u32::from(VK_F13.0))
    }

    /// A message-only `STATIC` window of the calling thread, as an integer,
    /// since an `HWND` is not `Send`.
    fn message_only_window() -> isize {
        // SAFETY: the class name is a static wide literal and the window
        // name is null; HWND_MESSAGE makes a message-only window, which no
        // other process enumerates or sends input to. No creation data is
        // passed.
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                None,
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
        }
        .expect("a message-only STATIC window is created");
        hwnd.0 as isize
    }

    fn hwnd(raw: isize) -> HWND {
        HWND(raw as *mut std::ffi::c_void)
    }

    #[test]
    fn with_main_refuses_reentry_on_the_same_thread() {
        // On its own thread, so that a broken refusal fails the test
        // instead of the suite.
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let _main = MainThread::new().expect("the main thread is set up");
            let before = REFUSED.with(Cell::get);
            let nested = with_main(|_| with_main(|_| ()));
            let _ = tx.send((nested, REFUSED.with(Cell::get) - before));
        });
        let (nested, refused) = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("re-entry panicked or hung");
        assert_eq!(nested, Some(None));
        assert_eq!(refused, 1, "the refusal is counted");
    }

    #[test]
    fn with_main_works_again_after_a_call() {
        let _main = MainThread::new().expect("the main thread is set up");
        assert_eq!(with_main(|_| 1), Some(1));
        assert_eq!(with_main(|_| 2), Some(2));
    }

    #[test]
    fn with_main_is_none_without_a_main_state() {
        assert_eq!(with_main(|_| ()), None);
        let main = MainThread::new().expect("the main thread is set up");
        assert_eq!(with_main(|_| ()), Some(()));
        drop(main);
        assert_eq!(with_main(|_| ()), None, "dropping it clears the state");
    }

    #[test]
    fn a_second_main_thread_on_one_thread_is_refused() {
        let _main = MainThread::new().expect("the main thread is set up");
        assert!(MainThread::new().is_err());
        assert_eq!(with_main(|_| ()), Some(()), "the first one stays");
    }

    #[test]
    fn plan_sets_the_timer_only_for_another_deadline() {
        // The gesture timer's signal is not a message, so nothing the loop
        // dispatched since it was set can have taken it: a timer set for the
        // earliest deadline stands.
        let now = Instant::now();
        let at = now + ms(100);
        let sooner = now + ms(50);

        assert_eq!(plan(None, None, now), Plan::Cancel);
        assert_eq!(plan(Some(at), None, now), Plan::Cancel);
        assert_eq!(plan(Some(at), Some(now), now), Plan::Due);
        assert_eq!(plan(None, Some(at), now), Plan::Arm { after: ms(100), at });
        assert_eq!(
            plan(Some(at), Some(at), now),
            Plan::Keep,
            "set for this deadline: the timer stands"
        );
        assert_eq!(
            plan(Some(at), Some(sooner), now),
            Plan::Arm {
                after: ms(50),
                at: sooner
            }
        );
    }

    #[test]
    fn a_timer_started_for_an_earlier_deadline_resets_the_wake() {
        // The wake is set inside `start`, which a hook callback calls, so the
        // loop's wait always holds the earliest deadline, whatever ran
        // inside the wait.
        let mut bindings = crate::config::from_ron_str(
            r#"( bindings: [
                (keys: "F13", action: DoNothing, trigger: Hold(100)),
                (keys: "F14", action: DoNothing, trigger: Hold(50)),
                (keys: "F15", action: DoNothing, trigger: Hold(200)),
            ] )"#,
        )
        .expect("config parses");
        sort_by_specificity(&mut bindings);
        let mut engine = Engine::new();
        let mut timers = GestureTimers::new().expect("the gesture timer is made");
        let env = QuietEnv(Instant::now());
        let mut press = |vk: VIRTUAL_KEY, timers: &mut GestureTimers| {
            let mut decision = engine.on_key(KeyEvent::down(vk), &bindings, &env);
            let due = decision.timers[0].due;
            assert_eq!(
                engine.start_timers(std::mem::take(&mut decision.timers), timers),
                0
            );
            due
        };

        assert_eq!(timers.armed, None);
        let first = press(VK_F13, &mut timers);
        assert_eq!(timers.armed, Some(first));
        let earlier = press(VK_F14, &mut timers);
        assert_eq!(timers.armed, Some(earlier), "an earlier deadline resets it");
        press(VK_F15, &mut timers);
        assert_eq!(timers.armed, Some(earlier), "a later one leaves it");
        assert_eq!(timers.queue.len(), 3);
    }

    #[test]
    fn a_message_posted_behind_a_wake_is_dispatched_without_more_input() {
        // Technical debt m-1 of roadmap item 2.1, for the main loop: `pump`
        // goes back to the drain at a MAIN_WAKE and leaves the messages
        // behind it, which are then no longer new input. Without
        // MWMO_INPUTAVAILABLE the wait would block on them until more input
        // came; here nothing else comes until the watchdog's WM_QUIT.
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let hook = KeyboardHook::new();
            let window = message_only_window();
            let handle = main.handle();
            // SAFETY: PostThreadMessageW and PostMessageW take their
            // arguments by value and read no memory of ours; both post to
            // this thread's own queue, a MAIN_WAKE first and the window's
            // WM_CLOSE behind it, and neither message carries a pointer.
            unsafe {
                PostThreadMessageW(handle.thread_id, MAIN_WAKE, WPARAM(0), LPARAM(0))
                    .expect("this thread has a message queue");
                PostMessageW(Some(hwnd(window)), WM_CLOSE, WPARAM(0), LPARAM(0))
                    .expect("the window takes a posted message");
            }
            let _ = ready_tx.send((window, handle));
            main.run(&hook)
        });
        let (window, handle) = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");

        let begun = Instant::now();
        let closed = loop {
            // SAFETY: IsWindow only reads the handle's value; a window that
            // was destroyed reads as no window.
            if !unsafe { IsWindow(Some(hwnd(window))) }.as_bool() {
                break true;
            }
            if begun.elapsed() > Duration::from_secs(1) {
                break false;
            }
            thread::sleep(ms(1));
        };
        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");
        assert!(
            closed,
            "the WM_CLOSE posted behind a MAIN_WAKE was not dispatched within 1 s"
        );
    }

    #[test]
    fn a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer() {
        // A low-level hook call is a message sent to the thread that
        // installed the hook. A wait blind to sent messages would leave each
        // one to the 1 s timeout below.
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let hook = hook_with(f13_hold(10_000, BindAction::DoNothing));
            let window = message_only_window();
            let (wparam, vk) = f13(WM_KEYDOWN);
            let armed = decide_event(&hook, wparam, vk, 0).is_some();
            let queued = with_main(|main| main.timers.queue.len());
            let _ = ready_tx.send((window, main.handle(), armed, queued));
            main.run(&hook)
        });
        let (window, handle, armed, queued) = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");
        assert!(armed, "the F13 key-down reached the engine");
        assert_eq!(queued, Some(1), "a Hold timer 10 s out is queued");

        let mut trips = Vec::new();
        let mut failed = 0;
        for _ in 0..100 {
            let begun = Instant::now();
            // SAFETY: `window` belongs to the loop thread, which keeps it
            // until it ends after the WM_QUIT below; WM_NULL carries no
            // pointer and no result pointer is passed.
            let answered = unsafe {
                SendMessageTimeoutW(
                    hwnd(window),
                    WM_NULL,
                    WPARAM(0),
                    LPARAM(0),
                    SMTO_ABORTIFHUNG,
                    1000,
                    None,
                )
            };
            trips.push(begun.elapsed());
            if answered == LRESULT(0) {
                failed += 1;
            }
        }
        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");

        assert_eq!(failed, 0, "every sent message is answered: {trips:?}");
        trips.sort();
        let median = trips[trips.len() / 2];
        assert!(
            median < ms(2),
            "a sent message took {median:?} in the median; sorted: {trips:?}"
        );
    }

    #[test]
    fn a_200_ms_hold_fires_within_a_few_ms_of_its_deadline() {
        // A wake on the system clock tick (about 15.6 ms) is about 7.8 ms
        // late on average, and a high-resolution timer under 1 ms. 5 ms sits
        // between the two; the median of five ignores one press in which a
        // loaded machine did not run the thread at once. The key-down's time
        // is read before the engine reads its own, so no lateness is
        // negative.
        let (tx, records) = mpsc::channel::<Record>();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        let hook = hook_with(f13_hold(200, key_macro()));
        hook.attach_executor(Some(executor.handle()));
        let main = MainThread::new().expect("the main thread is set up");
        let handle = main.handle();
        let (fired_tx, fired) = mpsc::channel();
        let waiter = thread::spawn(move || {
            for _ in 0..5 {
                let record = records.recv_timeout(Duration::from_secs(2));
                let _ = handle.request_exit();
                let _ = fired_tx.send(record.map(|record| record.at));
            }
        });

        let mut late = Vec::new();
        for _ in 0..5 {
            let pressed = Instant::now();
            let (wparam, vk) = f13(WM_KEYDOWN);
            assert!(decide_event(&hook, wparam, vk, 0).is_some());
            main.run(&hook).expect("the loop ends on WM_QUIT");
            let at = fired
                .recv_timeout(Duration::from_secs(5))
                .expect("the waiter answers")
                .expect("the Hold fired within 2 s");
            let (wparam, vk) = f13(WM_KEYUP);
            assert!(decide_event(&hook, wparam, vk, 0).is_some());
            let held = at.duration_since(pressed);
            assert!(
                held >= ms(200),
                "the Hold fired early, {held:?} after its key-down"
            );
            late.push(held - ms(200));
        }
        waiter.join().expect("the waiter does not panic");
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);

        late.sort();
        let median = late[late.len() / 2];
        assert!(
            median < ms(5),
            "a 200 ms Hold fired {median:?} late in the median; sorted: {late:?}"
        );
    }

    #[test]
    fn remapping_switches_through_the_main_thread_and_drops_pending_timers() {
        let (tx, records) = mpsc::channel::<Record>();
        let executor = Executor::start(FakeRunner::new(tx), QUEUE_CAPACITY, leaked_counters())
            .expect("the executor starts");
        // Leaked, so that the loop thread and this one share it, as the
        // daemon's threads share the process-wide hook.
        let hook: &'static KeyboardHook =
            Box::leak(Box::new(hook_with(f13_hold(300, key_macro()))));
        hook.attach_executor(Some(executor.handle()));
        let (ready_tx, ready) = mpsc::channel();
        let looper = thread::spawn(move || {
            let main = MainThread::new().expect("the main thread is set up");
            let (wparam, vk) = f13(WM_KEYDOWN);
            let armed = decide_event(hook, wparam, vk, 0).is_some();
            let _ = ready_tx.send((main.handle(), armed));
            main.run(hook)
        });
        let (handle, armed) = ready
            .recv_timeout(Duration::from_secs(5))
            .expect("the loop thread is ready");
        assert!(armed, "a 300 ms Hold is armed");

        assert_eq!(handle.set_enabled(false), Ok(true));
        assert!(!hook.is_enabled());
        assert_eq!(handle.set_enabled(false), Ok(false), "already off");
        assert!(
            records.recv_timeout(ms(500)).is_err(),
            "a Hold armed before remapping went off never fires"
        );
        assert_eq!(handle.set_enabled(true), Ok(true));
        assert!(hook.is_enabled());

        handle
            .request_exit()
            .expect("the loop thread takes WM_QUIT");
        looper
            .join()
            .expect("the loop thread does not panic")
            .expect("the loop ends on WM_QUIT without an error");
        hook.attach_executor(None);
        assert_eq!(executor.stop(), Stopped::Joined);
    }

    #[test]
    fn a_switch_request_to_an_ended_main_thread_fails_at_once() {
        let handle = thread::spawn(|| {
            MainThread::new()
                .expect("the main thread is set up")
                .handle()
        })
        .join()
        .expect("the scratch thread does not panic");

        let begun = Instant::now();
        assert_eq!(
            handle.set_enabled(false),
            Err(MainThreadError::ShuttingDown)
        );
        let took = begun.elapsed();
        assert!(took < ms(100), "the request waited {took:?}");
    }
}
