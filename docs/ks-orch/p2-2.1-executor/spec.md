# Lane spec: p2-2.1-executor - Action executor (KS-08)

Branch: feat/action-executor

Roadmap item 2.1 (book/src/roadmap.md, 'Action executor', including its 'Status after Phase 0'), KS-08; book/src/gaps.md 'No cancellation for macro workers'.
- An executor thread with its own GetMessageW loop and single-threaded COM (CoInitializeEx with COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE, balanced CoUninitialize), fed by a bounded queue. On overflow the job is dropped and counted (src/logging counters; the hook path never calls log::).
- A job is (Arc<BindAction>, hwnd, epoch). Sequence runs on the executor as well: no thread per macro run, so spawn_sequence and its panicking thread::spawn go away. A change of epoch (swch off, reload, exit) interrupts a Delay step within 100 ms (a wait with a timeout, not thread::sleep).
- Only the fast actions stay in the callback, so that their output stays in order with the keys that follow: PressKey, Media, ToggleCapsLock, SwitchLanguage* and PostMessage. Everything else (Launch, OpenUrl, Sequence, the system functions that spawn or call the shell) goes to the executor. A Hold that fires on its hold-timer thread follows the same split (2.2 moves that timer; do not move it here).
- A counter records the longest callback duration and is reported by the log writer.
- The main thread starts the executor before the hooks and stops it after they are removed; exit stops pending work deterministically and joins the thread before the final logging::flush.
- The executor takes an injectable job runner, so tests run fake jobs. No test executes a real action, calls SendInput, ShellExecuteW or starts a program.
Acceptance:
- rg "\.execute\(\)" src/hook finds nothing (the fast actions are reached through a differently named entry);
- 100 Sequence runs in a row create no threads (a test with the fake runner shows every job runs on the one executor thread);
- Delay(10000) stops within 100 ms of an epoch change (a test on a real executor thread with a Delay-only job and the fake runner);
- with 20 slow jobs queued (the fake runner blocks 500 ms each, standing in for OpenUrl against a cold browser) each enqueue returns in under 5 ms; the cold-browser run itself is a manual check;
- a full queue drops the job and bumps the counter.
Leftovers from Phase 1, as separate small commits (each report path names the item; line numbers may have moved):
- CHANGELOG.md reflow of the line D:/tmp/ks-orch/p1-1.1-repeat/review_r1.md M2 names;
- CHANGELOG Breaking entry for block_default (a modifier-only Hold or DoubleTap sends its output with that modifier down; remedy block_default: true) and the 'A Tap on Alt or Win' sub-bullet wording: D:/tmp/ks-orch/p1-1.3-deferred-block-default/results.md P1 and P2;
- the loader's MaskOnly fallback warning is pushed before the KS-18 skip: D:/tmp/ks-orch/p1-1.2-mask-key/results.md m1;
- parse_ron_str still logs 'config contains no bindings' though documented as the non-logging half: D:/tmp/ks-orch/p1-1.6-one-winner/review_r1.md m3;
- the doc comment of run on _instance: D:/tmp/ks-orch/p1-1.5-single-instance/results.md m1;
- a // SAFETY: comment on the unsafe PostThreadMessageW block in console_ctrl_handler (src/core/app.rs);
- logging review m1 (a second init must not spawn a writer) and m2 (the attach_file doc): D:/tmp/ks-orch/p1-1.7-logging/review_r1.md.
Update README (which actions run off the hook thread; swch off cancels a running macro), book/src/config/actions.md and CHANGELOG.
