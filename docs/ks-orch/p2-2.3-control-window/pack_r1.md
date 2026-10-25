# Context pack, round 1: p2-2.3-control-window (tester + code-reviewer)

Scout: project-analyst (read-only). Facts only, each with `file:line` at HEAD `6f6edb3` unless a commit is named. Inferred items are marked UNVERIFIED.

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `feat/control-window`, HEAD `6f6edb3`, tree clean (`git status --short` empty). `master` = merge-base = `c3dab6a`.
- Lane inputs: `spec.md`, `cut.md` (Commits `:596-771`, D14 bench rule `:392-422`, Acceptance mapping `:790-807`, Manual checks `:809-828`, Developer addendum `:969-1017`), `rulings.md` (R1, R2, M1-M8, Q1-Q8, round 2 V2-m1..m4, critic OQ1-OQ3, architect Q1-Q3), `impl.md`.
- `git log --oneline master..feat/control-window` (22 commits, oldest last; C-number from `impl.md:7-30`):

| C | sha | subject |
|---|---|---|
| C22 | 6f6edb3 | Hook: the docs name the third hook and the requests the main loop serves |
| C21 | bc0a929 | Hook: a hidden control window and the desktop-switch hook reset the state when keyboard input leaves the desktop |
| C20 | afbe152 | Tests: input leaving the desktop asks the main thread for a reset that frees a swallowed key; a late return asks for none |
| C19 | a5140fe | Hook: the foreground window's executable name is cached per window, outside the keyboard callback |
| C18 | 036a859 | Tests: a hundred conditional presses in one window query its process once |
| C17 | 4fbcc37 | Hook: a reload swaps the bindings and resets the engine on the main thread |
| C16 | f7588e4 | Tests: a reload through the main thread drops an armed Tap |
| C15 | 3246339 | Engine: a swallowed key-down's record survives focus changes and resets; input leaving the desktop releases every key |
| C14 | 765e5b5 | Tests: a swallowed key-down keeps its key-up swallowed across focus changes and resets, and only input leaving releases keys |
| C13 | 7eabce9 | Engine: the key-up of a swallowed key passes when Windows has the key down |
| C12 | 0ef6a99 | Tests: the key-up of a swallowed key that Windows has down reaches Windows |
| C11 | dd53797 | Bench: R9 to R12 time a conditional key-down, a swallowed key-up and an unbound key |
| C10 | 5bf7f12 | Watcher: a thread that cannot be started is an error the daemon logs |
| C9 | 5388175 | Tests: the config watcher reports a thread it cannot start |
| C8 | 7d2c22f | Engine: a key of a waiting gesture's own combo does not cancel it |
| C7 | a0ae250 | Tests: a passing Ctrl+Shift DoubleTap completes when its own keys go down again |
| C6 | bec45b4 | Engine: a refused gesture timer reports whether it dropped a Hold |
| C5 | 668e80e | Tests: a refused gesture timer counts only the Holds it drops |
| C4 | cb8dc1e | Hook: a gesture timer that Windows refuses to move earlier is set again |
| C3 | 19df9d4 | Tests: a gesture timer that cannot be moved earlier is set again by the loop |
| C2 | a60aa65 | Tests: a waiting Tap whose window closed while its timer was late fires at its own key-down |
| C1 | 0ca09d7 | Executor: the module docs name the threads that hand it actions today |

- `git diff --stat master..feat/control-window`: 32 files, +3842 / -748. `Cargo.lock` unchanged (no stat line). Per file: CHANGELOG.md 134; Cargo.toml 4; README.md 91; book/src/config/{errors.md 16, kinds.md 5, known-issues.md 13, structure.md 4, triggers-conditions.md 23}; src/bench.rs 129; src/config/default_config.ron 3; src/core/app.rs 31; src/core/executor/mod.rs 5; src/data/condition.rs 320 (mostly removed); src/engine/{env.rs 27, gestures.rs 68, keys.rs 180, mod.rs 441, properties.rs 87, scenarios.rs 587, testing.rs 114}; src/hook/{config_watcher.rs 101, control_window.rs 460 (new), keyboard_hook.rs 64, keyboard_hook_callback.rs 385, main_thread.rs 699, mod.rs 1}; src/logging/{counters.rs 9, mod.rs 7}; src/platform/{desktop.rs 62 (new), foreground.rs 498 (new), mod.rs 5, timers.rs 17}.

## File map

| path:lines | what lives there | why it matters this round |
|---|---|---|
| `src/hook/control_window.rs:52-57` | `CLASS_NAME` (`w!("key-switch-rs control")`), test-only `CONTROL_CLASS` mutex | critic OQ3: shared by the window test and bench R13 |
| `src/hook/control_window.rs:74-122` | `ControlWindow` + Drop guards `RegisteredClass` (`UnregisterClassW`), `OwnedWindow` (`DestroyWindow`), `SessionRegistration` (`WTSUnRegisterSessionNotification`) | handle release on every path (spec); field drop order `_session`, `_window`, `_class` |
| `src/hook/control_window.rs:129-184` | `create()`: `GetModuleHandleW(None)`, `RegisterClassW`, `CreateWindowExW(WS_EX_TOOLWINDOW\|WS_EX_NOACTIVATE, .., WS_POPUP, parent None)`, `WTSRegisterSessionNotification(NOTIFY_FOR_THIS_SESSION)`, failure -> `log::warn!` + `None` | not `HWND_MESSAGE` (spec, tray 4.6); M5 tolerated failure |
| `src/hook/control_window.rs:203-231` | `session_signal` (LOCK/UNLOCK only), `session_ends`, `end_session` (`uninstall`, info line, `logging::flush()`, `PostQuitMessage(0)`) | Q5; flush waits up to 1 s (CLAUDE.md principle 1) |
| `src/hook/control_window.rs:235-265` | `control_window_proc`: `WM_WTSSESSION_CHANGE` -> `note_input_signal` if enabled; `WM_QUERYENDSESSION` -> `LRESULT(1)`; `WM_ENDSESSION` -> `end_session` if wparam != 0; `WM_CLOSE` -> 0; else `DefWindowProcW` | M6 direct-call tests |
| `src/hook/control_window.rs:268-460` | tests (4) incl. `EnumThreadWindows` counter `:383-404` | critic OQ1, M5, M6 |
| `src/platform/desktop.rs:26-51` | `input_is_on_this_desktop()`: `GetThreadDesktop(GetCurrentThreadId())`, `GetUserObjectInformationW(UOI_IO, BOOL, size_of::<BOOL>())` | D16; called inside the desktop-switch WinEvent callback |
| `src/platform/desktop.rs:53-62` | ignored M14 test | manual |
| `src/platform/foreground.rs:36-75` | `WindowQuery` trait (foreground/owner/image_name/title) and `Win32Query` | the counting seam of the 100-press test |
| `src/platform/foreground.rs:79-164` | `ForegroundCache`: `window_changed` `:122-140` (skip invalid/pid 0/same window+pid; own process -> `app = None`), `resync` `:143-146`, `app` `:150-152`, `read_title` `:156-161` | Q8: exe cached per (HWND, pid), title live |
| `src/platform/foreground.rs:166-219` | `TITLE_CAPACITY = 512`, `window_title` (`GetWindowTextW`), `IMAGE_PATH_CAPACITY = 1024`, `process_image_name` (`OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` in `Owned`, `QueryFullProcessImageNameW`) | R9; moved from `data/condition.rs` |
| `src/platform/foreground.rs:228-498` | tests (7 + ignored M10 `:487-489`) | |
| `src/hook/main_thread.rs:85-173` | `PENDING_SIGNALS` thread-local `Cell<PendingSignals>` `:93`; `PendingSignals` `:101-125`; `note_input_signal` `:131-143`; `note_unknown_desktop_switch` `:147-154`; `take_pending_signals` `:157-162`; `wake_this_thread` `:167-172` (`PostThreadMessageW(GetCurrentThreadId(), MAIN_WAKE)`) | D7, R2 coalescing |
| `src/hook/main_thread.rs:182-273` | `MainState { engine, timers, foreground }`; `set_enabled` `:217-223`; `reset_transient_state` `:230-233` (`reset_engine` + `hook.cancel_actions()`); `reload` `:240-243` (`update_bindings` + `reset_engine(Reloaded)`); `reset_engine` `:246-255` (foreground `resync` only for `RemappingSwitched`); `foreground_changed` `:260-264`; `input_returned` `:269-271` | D1, D4, R3 |
| `src/hook/main_thread.rs:345-380` | `plan_wake`: `armed = None` before `set`; on `Err` drains the queue and counts `forget_timer == true` | C6 / m2 |
| `src/hook/main_thread.rs:388-404` | `Scheduler::start`: on failed `set`, removes the token and sets `armed = None` `:398` | C4 / OQ3 |
| `src/hook/main_thread.rs:412-505` | `Request::{SetEnabled, Reload}`; `ApplyReload for MainThreadHandle` `:450-454`; `MainThreadHandle::reload` `:490-505` (copy of `set_enabled`'s send/wake/wait body, deviation 7) | D4 |
| `src/hook/main_thread.rs:611-613` | `refresh_foreground` | M3 |
| `src/hook/main_thread.rs:618-686` | `run` loop; `drain` `:666-686` (requests, then `take_pending_signals` -> `answer_input_signals`, then due timers, until a pass is idle) | step order |
| `src/hook/main_thread.rs:688-698` | `Drop for MainThread` also clears `PENDING_SIGNALS` | |
| `src/hook/main_thread.rs:701-733` | `answer_input_signals`: returns if disabled; logs each signal in `ALL` order; warn for unknown; one reset via `find_map(InputSignal::reset)` inside `with_main`, then `log::info!("state reset: {reason}")` outside it; `input_returned` if any return signal | R2, V2-m1 order |
| `src/hook/main_thread.rs:747-770` | `serve`: `Reload` arm logs `config reload carried out late` when `reply.try_send` fails | critic OQ4 |
| `src/hook/main_thread.rs:794-1627` | tests; new ones `:1003`, `:1057`, `:1323`, `:1360`, `:1408`, `:1444-1610` | |
| `src/hook/keyboard_hook_callback.rs:139` | test-only `GLOBAL_HOOK` mutex at module level | |
| `src/hook/keyboard_hook_callback.rs:146-182` | `HookEnv<'a> { foreground: &ForegroundCache, title: OnceCell<Option<String>> }`; `os_key_down` = `GetAsyncKeyState & 0x8000` `:165-171`; `foreground_app` cache read `:173-175`; `foreground_title` lazy `read_title` `:177-181` | R1 hot path |
| `src/hook/keyboard_hook_callback.rs:203-240` | `decide_event`: sentinel `:209`, enabled `:216`, one `bindings()` snapshot `:222`, split borrow of `MainState` `:223-233` | R1, R2, R3 |
| `src/hook/keyboard_hook_callback.rs:339-391` | extern `keyboard_hook_callback` (code<0 `:344-349`, off gate `:352-357`) | R1 unchanged |
| `src/hook/keyboard_hook_callback.rs:393-408` | extern `foreground_changed` -> `with_main(main.foreground_changed(hwnd))` | process query now runs here (main thread, WinEvent callback) |
| `src/hook/keyboard_hook_callback.rs:411-453` | `DesktopSwitchNote`, `desktop_switch_note`, extern `desktop_switched` | D16 |
| `src/hook/keyboard_hook_callback.rs:455-1330` | tests; new `:628`, `:657`, `:684`, `:1004` (extended), `:1069`, `:1088` helper | |
| `src/hook/keyboard_hook.rs:34-37,62` | `desktop_hook: AtomicPtr` | R4 |
| `src/hook/keyboard_hook.rs:78-84,106-110` | `update_bindings` (sort, store, `cancel_actions`); `cancel_actions` -> `executor.cancel_pending()` | R7 |
| `src/hook/keyboard_hook.rs:141-221` | `install`: LL hook, foreground WinEvent `:170-190`, desktop-switch WinEvent `:200-218` (`WINEVENT_OUTOFCONTEXT \| WINEVENT_SKIPOWNPROCESS`; on failure `uninstall()` + `E_FAIL`) | R10, rollback |
| `src/hook/keyboard_hook.rs:233-270` | `uninstall`: desktop hook swap+`UnhookWinEvent` `:234-245`, foreground `:247-258`, LL hook `:260-267` | R10 |
| `src/engine/mod.rs:381-462` | `ResetReason::{RemappingSwitched, Reloaded, InputLeft}` + Display `:393-400`; `InputSignal` 4 variants, `ALL` `:427-432`, `reset()` `:435-442`, `input_returned()` `:445-450`, Display `:453-462` | R6 |
| `src/engine/mod.rs:555-595` | `start_timers -> u32` (counts only `forget_timer == true`); `forget_timer -> bool` | C6 |
| `src/engine/mod.rs:597-610` | `on_foreground_changed` = `keys.sync_modifiers(env, None)` only | D2 |
| `src/engine/mod.rs:612-655` | `reset(env, reason)`: clears gestures/chords/sequences/`mask_owed`; per reason `resync_from_os` / nothing / `release_all`; generation `wrapping_add(1)` | D1 |
| `src/engine/mod.rs:869-887` | key-up: `take_blocked`, `swallow`, witness `decision.suppress = swallow && !env.os_key_down(vk)` `:885` | R1 ruling |
| `src/engine/mod.rs:902-1500` | tests; new `:1231`, `:1279`, `:1308`, `:1339`; changed `:1163` | |
| `src/engine/keys.rs:112-150` | `resync_from_os` (0..=255, OS read for 1..=254 non-mouse; `Suppressed` record kept); `release_all` (`active = [false; 256]`) | D1, KS-14, R8 |
| `src/engine/gestures.rs:291-349` | `other_key_down(vk, ..)`: own-combo arm `:318-322` (`group.keys.keys.iter().any(|key| key.accepts(vk))` -> no cancel) | D10 / OQ2 |
| `src/engine/gestures.rs:417-436` | `forget -> bool` | C6 |
| `src/engine/env.rs:52-59` | `Env::{os_key_down, foreground_app, foreground_title}` docs changed | |
| `src/engine/testing.rs:256-300,344-425,449-509,601,625-640,716-750` | `check_nothing_stuck`, `check_no_lone_key_up` `:284`, `reload` `:295`, `up_unseen` `:344`, `down_unseen` `:362`, `foreground_changed` `:379`, four signal steps `:397-425`, `remapping_off/on` `:449/:470`, `advance_ms_stalled` `:509`, lone-key-up fail `:635`, stuck fail `:744` | harness seams |
| `src/engine/properties.rs:208-235,292,327-336,361-388` | `Mode` (`Strict`, `Focus`, `FocusWhole`, `Unseen`, `Unlock`); the Unlock signal draw `:292`; five `run_all` tests | R1/R2 properties |
| `src/hook/config_watcher.rs:40-62` | `ApplyReload` trait; `&'static T` impl; test-only `impl for KeyboardHook` | D4 |
| `src/hook/config_watcher.rs:126-160,188-213` | `spawn_watcher(.., sink: impl ApplyReload)`, `.map_err(notify::Error::io)?` `:158`; `reload_into` outcome lines | C10, D4 |
| `src/core/app.rs:207-224,285-297` | executor guard `:209`, `ControlWindow::create()` `:215`, `install` `:221`, `refresh_foreground` `:224`; `main.run` `:285`, `uninstall()?` `:292`, `drop(control_window)` `:296`, `executor.stop()` `:297` | M3, M4, V2-m4 |
| `src/core/app.rs:234-240` | IPC serve failure path: `uninstall`, return (control window then dropped by scope, before `executor`) | R10 |
| `src/data/condition.rs:1-244` | `ForegroundContext` and Win32 code removed; `ForegroundInfo` `:90`, `evaluate` `:55` | |
| `src/bench.rs:45-56,352-403,525-560,710-800` | R9-R13 docs, row fns, `latency_bench` (`#[ignore = "bench: ..."]` `:710`), `refresh_foreground` before R9/R10 `:769` | D14 |
| `src/platform/timers.rs:61-76` | test-only `WaitableTimer::not_a_timer()` (`CreateEventW`) | C3 seam |
| `src/logging/counters.rs:27-31` | `HoldTimerNotStarted` doc | m2 |
| `src/logging/mod.rs:436-445` | hot-path scan list: `foreground.rs` `fn app(`, `fn read_title(`, `fn window_title(`; `desktop.rs` whole | hot-path scan test |
| `Cargo.toml:41-57` | new `windows` features `Win32_Graphics_Gdi`, `Win32_System_LibraryLoader`, `Win32_System_RemoteDesktop`, `Win32_System_StationsAndDesktops` | Q6, architect Q1 |
| `book/src/config/known-issues.md:35,123,131,136,144-148` | rows 9, 55, 63, 68, 76, 77, 78, 79, 80 | docs |
| `book/src/config/triggers-conditions.md:39,58,126,199,218` | §11.3, §11.4, §11.8, §12.3, §12.4 | docs |
| `CHANGELOG.md:12-14,174,539` | `[Unreleased]`, Added, Changed, Fixed | cut `:773-788` entries |

## Facts

1. HEAD `6f6edb3`; merge-base with `master` `c3dab6a`; 22 commits; tree clean.
2. The witness is `decision.suppress = swallow && !env.os_key_down(vk)` (`src/engine/mod.rs:885`); `swallow` is computed at `:872-873` from `take_blocked` and `released.gesture_bound && !was_passed`; the OS is read only when `swallow` is true (short-circuit `&&`).
3. `HookEnv::os_key_down` calls `GetAsyncKeyState(i32::from(vk.0))` and tests bit `0x8000` (`src/hook/keyboard_hook_callback.rs:165-171`). The comment at `src/engine/mod.rs:874-884` claims the read is the state before the key-up; per `impl.md:147` this OS behaviour is unverified and left to manual M12/M13.
4. The witness reads the event's exact code; a generic `VK_SHIFT` reads the either-side state (`src/engine/mod.rs:882-884`); pinned by `engine::tests::the_witness_of_a_generic_shift_key_up_reads_the_generic_state` (`src/engine/mod.rs:1279`) and `engine::scenarios::a_swallowed_lshift_key_up_stays_swallowed_while_the_right_shift_is_down_in_windows` (`src/engine/scenarios.rs:2041`). Developer's OQ2 evidence for injected generic codes is "recalled, not re-read" (`cut.md` addendum, Critic OQ2).
5. `InputSignal::reset` returns `Some(InputLeft)` for `SessionLocked` and `DesktopSwitchedAway`, `None` for `SessionUnlocked` and `DesktopSwitchedBack` (`src/engine/mod.rs:435-442`); `input_returned` is true for the latter two (`:445-450`).
6. `Engine::reset(env, reason)` clears gestures, chords, sequences and `mask_owed`, then `RemappingSwitched` -> `keys.resync_from_os(env)`, `Reloaded` -> nothing, `InputLeft` -> `keys.release_all()`, then bumps the generation (`src/engine/mod.rs:642-655`).
7. `release_all` sets `active = [false; 256]` and keeps `last_down` (`src/engine/keys.rs:148-150`). `resync_from_os` keeps a `Suppressed` record and sets `active` from the OS for 1..=254 except mouse buttons (`src/engine/keys.rs:126-140`).
8. `on_foreground_changed` only calls `keys.sync_modifiers(env, None)` (`src/engine/mod.rs:608-610`); `Forgotten`/`forget_blocked` are removed (C15 per `cut.md:696`).
9. `PENDING_SIGNALS` is a main-thread `thread_local` `Cell<PendingSignals>` with `seen: [bool; 4]` and `unknown: Option<HRESULT>` (`src/hook/main_thread.rs:93,101-105`). A signal noted twice before a drain is one flag.
10. `note_input_signal` and `note_unknown_desktop_switch` set a flag then post `MAIN_WAKE` to the current thread (`src/hook/main_thread.rs:131-154,167-172`); a failed post is ignored (`let _`).
11. `drain` serves requests, then pending signals, then due timers, and loops until one pass is idle (`src/hook/main_thread.rs:666-686`).
12. `answer_input_signals` returns at once while remapping is off (`src/hook/main_thread.rs:711-713`); it calls `reset_transient_state` at most once per drain (`find_map`, `:723-727`) and logs `state reset: {reason}` after `with_main` returns (`:727`).
13. `reset_transient_state` = `reset_engine(reason)` + `hook.cancel_actions()` and runs inside `with_main` (`src/hook/main_thread.rs:230-233`, call site `:724`). `cancel_actions` -> `ExecutorHandle::cancel_pending` = `epoch.fetch_add(1, AcqRel)` + `PostThreadMessageW` (`src/hook/keyboard_hook.rs:106-110`; `src/core/executor/mod.rs:244-262`).
14. `MainState::reload` = `hook.update_bindings(bindings)` + `reset_engine(ResetReason::Reloaded)` inside `with_main` (`src/hook/main_thread.rs:240-243`, call `:758`). `update_bindings` sorts, stores and calls `cancel_actions` (`src/hook/keyboard_hook.rs:78-84`).
15. `reset_engine` resyncs the foreground cache only for `RemappingSwitched` (`src/hook/main_thread.rs:249-251`) and builds a `HookEnv` from the cache (`:252`).
16. `MainThreadHandle::reload` uses `try_send` (`Busy` when full; capacity `REQUEST_CAPACITY = 8` `:83`) and waits `REPLY_TIMEOUT = 2 s` (`src/hook/main_thread.rs:80,490-505`).
17. `reload_into` logs one of: `config reloaded ({count} bindings live)`, the NoAnswer warn, `config reload skipped: ...shutting down`, or `config reload failed: the daemon is busy` + `keeping previous bindings live` (`src/hook/config_watcher.rs:200-212`).
18. The daemon's watcher sink is `main.handle()` (`src/core/app.rs:253`); `impl ApplyReload for KeyboardHook` exists only under `#[cfg(test)]` (`src/hook/config_watcher.rs:55-62`).
19. `spawn_watcher` maps a failed thread spawn to `notify::Error::io` with `?` (`src/hook/config_watcher.rs:158`).
20. `ForegroundCache::window_changed` returns early for an invalid handle, for `pid == 0`, and for the same (window, pid); a window of this process gets `app = None` and no image query (`src/platform/foreground.rs:122-140`). `read_title` returns `None` for an invalid window or an own-process window (`:156-161`).
21. `HookEnv::foreground_title` reads the title at most once per `HookEnv` via `OnceCell` and allocates a `String` (`src/hook/keyboard_hook_callback.rs:177-181`; `src/platform/foreground.rs:169-179`). `master` also read the title lazily into a `String` (`git show master:src/data/condition.rs` `:113-114,160-170`). On HEAD the title is read from the cached HWND, on master from the HWND captured per event (`master:src/data/condition.rs:121-127`).
22. The process query (`OpenProcess` + `QueryFullProcessImageNameW`) now runs from the `EVENT_SYSTEM_FOREGROUND` WinEvent callback on the main thread (`src/hook/keyboard_hook_callback.rs:407` -> `src/hook/main_thread.rs:260-264` -> `src/platform/foreground.rs:135-139,184-219`), not in the keyboard callback.
23. `refresh_foreground()` runs right after `hook.install()?` (`src/core/app.rs:221-224`); `ForegroundCache::resync` also runs at `RemappingSwitched` (`src/hook/main_thread.rs:249-251`) and on a return signal (`:269-271`).
24. The desktop-switch hook is `SetWinEventHook(EVENT_SYSTEM_DESKTOPSWITCH, .., WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)`, stored with `Release`, removed with `swap(null, AcqRel)` + `UnhookWinEvent`; a failed install calls `uninstall()` and returns `E_FAIL` (`src/hook/keyboard_hook.rs:200-218,234-245`).
25. `desktop_switched` returns while remapping is off and otherwise classifies with `input_is_on_this_desktop()`: `Ok(true)` -> `DesktopSwitchedBack`, `Ok(false)` -> `DesktopSwitchedAway`, `Err` -> unknown note (`src/hook/keyboard_hook_callback.rs:423-453`).
26. `input_is_on_this_desktop` does not close the desktop handle, citing Microsoft Learn's GetThreadDesktop text (`src/platform/desktop.rs:31-35`), and passes `size_of::<BOOL>()` as the length (`:37-49`).
27. The control window is a top-level `WS_POPUP` without `WS_VISIBLE`, `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`, parent `None` (`src/hook/control_window.rs:151-166`); a failed `WTSRegisterSessionNotification` logs `could not register for session notifications (...)` and continues (`:169-178`).
28. Struct field order is `_session`, `_window`, `_class` (`src/hook/control_window.rs:74-78`), so Rust drops the session registration, then the window, then the class.
29. `control_window_proc` notes a session signal only while remapping is on (`src/hook/control_window.rs:242-247`); `WM_ENDSESSION` with `wparam != 0` calls `end_session` (`:252-256`), which calls `hook.uninstall()`, `log::info!`, `logging::flush()` and `PostQuitMessage(0)` (`:222-231`).
30. `App::run` creates the control window after the executor guard and before `install` (`src/core/app.rs:209-221`) and drops it after `hook.uninstall()?` and before `executor.stop()` (`:292-297`). If `uninstall()?` at `:292` returns `Err`, the window is dropped at scope end, before `executor` (declared earlier, `:209`).
31. `Scheduler::start` sets `armed = None` after a failed `set` (`src/hook/main_thread.rs:394-399`); `plan_wake` sets `armed = None` before `set` (`:356`).
32. `plan_wake` on a failed `set` drains the whole queue and counts only tokens where `engine.forget_timer` is true (`src/hook/main_thread.rs:362-374`); `start_timers` counts the same way (`src/engine/mod.rs:571-576`); `Gestures::forget` returns true only when a Hold was dropped (`src/engine/gestures.rs:423-436`).
33. `other_key_down` does not cancel an `AwaitSecond` press whose group's combo accepts the new key (`src/engine/gestures.rs:318-322`).
34. No `.unwrap()`, `.expect(`, `panic!` or `unreachable!` in the non-test part of any changed file in `src/hook`, `src/platform`, `src/engine/{mod,keys,gestures}.rs`, `src/core/app.rs` (scan of lines before each `mod tests {`).
35. No new `SendInput`, `dwExtraInfo`, `Ordering::Relaxed` or `static mut` in added lines of `src` (diff scan); new `thread::spawn`/`.lock()` hits are in `src/bench.rs` and test modules.
36. New `unsafe` sites with SAFETY comments are listed at `impl.md:61-71`; `control_window_proc` is `unsafe extern "system"` (`src/hook/control_window.rs:235`).
37. Ignored tests added: `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad` (`src/platform/foreground.rs:487-489`, M10), `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it` (`src/platform/desktop.rs:57-59`, M14); the moved `data::condition` ignore is gone (`impl.md:79`). Developer's lib run: 676 passed, 0 failed, 7 ignored (`impl.md:76`); the full `cargo test --locked` was not run by the developer.
38. Tests that create a real window/registration (not a hook): `the_control_window_is_a_hidden_top_level_window_released_on_drop` (`RegisterClassW`, `CreateWindowExW`, `WTSRegisterSessionNotification`), bench R13; `desktop_switched` direct call reads `GetThreadDesktop`/`GetUserObjectInformationW` (`src/hook/keyboard_hook_callback.rs:1088-1095`).
39. The bench engine rows (R1-R3) call `engine.on_key` directly (`src/bench.rs:223-290`); adapter rows R4, R5, R9-R12 call the extern callback with `DoNothing` bindings and no hook installed (`src/bench.rs:16-18,293-403`).
40. `cargo doc` warnings: 3 pre-existing, the lane's one fixed in C22 (`impl.md:77`).

## Red-first tests and acceptance proofs

Command form (lib tests): `cargo test --locked --lib <path> -- --exact` (e.g. `cargo test --locked --lib engine::scenarios::nothing_is_stuck_after_win_l -- --exact`). Compile reds: `cargo test --locked --lib --no-run`. Expected at HEAD: every test below green.

"Red tree" = the commit on which the test must be red: the test commit itself (= parent of its fix commit). "Master-replayable" = the test's file hunks are test-only and need no seam beyond master's harness (master has `check_nothing_stuck`, `reload`, `up_unseen`, `down_unseen`, `remapping_off`, `advance_ms_stalled`, `expect_down_in_windows`: `git show master:src/engine/testing.rs` `:239-518`; master `Mode` = `Strict`, `Focus` only).

| C (red tree) | fix | test(s) | expected red | master-replayable? |
|---|---|---|---|---|
| C2 `a60aa65` (pin) | none | `engine::scenarios::a_waiting_tap_whose_window_closed_while_its_timer_was_late_fires_at_its_own_key_down` (`scenarios.rs:988`) | green everywhere; red only under mutation: drop `fire.extend(deferred_tap)` (`gestures.rs:199` at HEAD; cut cites `:193`) -> `down F13 at +381 ms: expected to fire [SwitchLanguage], fired []` | yes (scenarios only) |
| C3 `19df9d4` | C4 `cb8dc1e` | `hook::main_thread::tests::a_gesture_timer_that_cannot_be_moved_earlier_is_set_again_by_the_loop` (`main_thread.rs:1003`) | `a failed set leaves the timer set for no deadline the loop knows / left: Some(Instant { .. }) / right: None` | needs seam `WaitableTimer::not_a_timer` (test-only, added in C3 `platform/timers.rs`); red tree = `19df9d4`. Hold terms 10000/5000 ms (deviation 6) |
| C5 `668e80e` | C6 `bec45b4` | `engine::tests::a_refused_window_timer_drops_no_gesture` (`mod.rs:1339`), `engine::tests::a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes` (`:1163`, assertion changed to 0), `hook::main_thread::tests::a_wake_that_cannot_be_set_counts_only_the_holds_it_drops` (`main_thread.rs:1057`) | first two `left: 1 right: 0`; third `only the Hold is dropped left: 3 right: 1` | third needs C3's `not_a_timer`; red tree = `668e80e` (hunks are in `mod tests` only) |
| C7 `a0ae250` | C8 `7d2c22f` | `engine::scenarios::a_passing_ctrl_shift_double_tap_is_not_cancelled_by_its_own_keys` (`scenarios.rs:4169`); `...a_waiting_tap_of_a_passing_ctrl_shift_double_tap_fires_with_ctrl_down_in_windows` (`:4230`, M1) | `down LShift at +50 ms: expected to fire [ToggleCapsLock], fired []`; M1: `down LCtrl at +50 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(27))]`. Part (2)'s red `expected to fire [], fired [SwitchLanguage]` traced, not observed (deviation 4) | yes (scenarios only) |
| C9 `5388175` | C10 `5bf7f12` | `hook::config_watcher::tests::the_watcher_thread_has_no_panic_path` (`config_watcher.rs:448`) | `panic paths in src/hook/config_watcher.rs:\n126: .expect("spawn config-watcher thread");` | yes (text scan) |
| C12 `0ef6a99` | C13 `7eabce9` | (a) `engine::scenarios::a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_is_released_in_windows` (`scenarios.rs:1986`); (d) `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key` (`mod.rs:1231`); (e) `engine::properties::nothing_is_left_down_in_windows_when_the_hook_misses_key_downs` (`properties.rs:376`) | (a) `up CapsLock at +0 ms: VK 0x14 is down in Windows but physically up`; (d) `a swallowed key-up asks the OS about its key once / left: 0 / right: 1`; (e) `up CapsLock at +10838 ms: VK 0x14 is down in Windows but physically up / context: config default, seed 8, Unseen mode` | yes: hunks are test-only (`mod.rs` `mod tests` `:1073+`, `properties.rs`, `scenarios.rs`); `Mode::Unseen` comes with the test |
| C12 pins | - | (b) `..._across_focus_changes_is_released_in_windows` (`scenarios.rs:2005`, the critic's R1 script); (c) `a_swallowed_key_released_while_remapping_is_off_and_pressed_again_unseen_is_released_in_windows` (`:2022`) | green on master and at `0ef6a99`; mutation at HEAD: remove the witness (`mod.rs:885` -> `swallow`) -> stuck (red of (a)) | yes |
| C14 `765e5b5` | C15 `3246339` | flips: `a_foreground_change_keeps_a_swallowed_key_up_swallowed` (`:206`), `immediate_key_held_across_a_focus_change_keeps_its_repeats_and_release_suppressed` (`:1852`), `capslock_held_across_a_focus_change_does_not_reach_windows` (`:1875`), `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed` (`:1898`), `capslock_held_across_a_focus_change_switches_the_language_once` (`:2521`), `chord_keys_held_across_a_focus_change_keep_their_repeats_suppressed` (`:2959`), `deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:2989`), `immediate_combo_that_stops_matching_after_a_focus_change_keeps_its_key_suppressed` (`:3031`), `press_after_a_lost_key_up_without_a_reset_reads_as_a_repeat_and_fires_only_a_repeating_binding` (`:1922`) | `up <key> at +<n> ms: expected the event to be suppressed, it was passed` (7 flips, `impl.md:22`); 3 tests carrying `check_no_lone_key_up` red with `up F13 at +0 ms: Windows got a lone key-up of F13` (deviation 3) | needs harness seams added in C14 (`testing.rs`: four signal steps, `check_no_lone_key_up`); red tree = `765e5b5` |
| C14 `765e5b5` | C15 | new: `a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed` (`:2066`), `a_key_swallowed_before_remapping_off_and_released_after_remapping_on_keeps_its_key_up_swallowed` (`:2080`), `nothing_is_stuck_after_win_l` (`:2095`), `after_a_lock_the_next_press_of_a_passing_key_is_a_new_press` (`:2125`); properties `every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes` (`properties.rs:381`), `no_key_up_reaches_windows_alone_across_focus_changes_unseen_presses_and_desktop_switches` (`:386`) | `nothing_is_stuck_after_win_l`: `down L at +0 ms: expected the event to be suppressed, it was passed`; passing-key: `down F13 at +1000 ms: expected to fire [SwitchLanguage], fired []`; FocusWhole: `press of CapsLock: key-down suppressed, up passed (seed 0)`; Unlock: `Windows got a lone key-up of CapsLock (seed 7)` | as above, red tree = `765e5b5` |
| C14 pins (green at `765e5b5` and HEAD) | - | `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires` (`:2146`), `after_input_leaves_for_the_secure_desktop_the_next_press_of_a_swallowed_key_fires` (`:2167`), `a_late_unlock_notification_keeps_a_press_made_after_the_unlock` (`:2187`), `a_late_return_notification_keeps_a_press_made_after_the_return_in_either_order` (`:2222`), `a_hold_armed_between_two_return_notifications_fires` (`:2259`), `a_second_lock_releases_a_key_pressed_after_the_first` (`:2277`), `a_swallowed_key_pressed_again_on_the_lock_screen_is_released_in_windows` (`:2298`), `press_symmetry_check_reports_a_repeat_after_a_lock_that_differs_from_its_key_down` (`:5435`, `should_panic(expected = "press of F13: key-down suppressed, repeat passed")`) | mutations, see below | - |
| C16 `f7588e4` (compile) | C17 `4fbcc37` | `hook::main_thread::tests::a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing` (`:1323`), `...a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold` (`:1360`), `...a_reload_the_watcher_stopped_waiting_for_is_logged_when_it_lands` (`:1408`) | `E0599 no method named reload ... &mut MainState`, `... MainThreadHandle`, `E0599 no variant named Reload ... Request` | no: needs the API the fix adds; red tree = `f7588e4` |
| C18 `036a859` (compile) | C19 `a5140fe` | `hook::keyboard_hook_callback::tests::a_hundred_conditional_presses_in_one_window_query_its_process_once` (`:628`), `...a_title_condition_reads_the_cached_windows_title_on_each_key_down` (`:657`), `...a_window_of_this_process_is_never_asked_for_its_title` (`:684`), `platform::foreground::tests::{the_cache_queries_again_only_for_another_window (:289), an_invalid_foreground_window_keeps_the_cache (:322)}` | observed (`impl.md:26`): `E0425 cannot find type ForegroundCache`, `E0405 cannot find trait WindowQuery`, `E0432 unresolved imports ...`; cut predicted `E0412`/`E0599` (`cut.md:730`) | no; red tree = `036a859` |
| C20 `afbe152` (compile) | C21 `bc0a929` | `hook::main_thread::tests::{a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key (:1455), a_late_return_signal_keeps_a_running_macro_and_an_armed_hold (:1526), the_drain_logs_one_line_per_input_signal (:1578)}`; `hook::control_window::tests::{only_the_lock_and_the_unlock_are_input_signals (:308), the_session_messages_note_the_lock_and_the_unlock (:327), the_session_end_messages_are_answered_and_end_the_loop (:352), the_control_window_is_a_hidden_top_level_window_released_on_drop (:407)}`; `hook::keyboard_hook_callback::tests::{a_desktop_switch_is_classified_by_where_input_is (:1069), a_desktop_switch_notes_a_signal_only_while_remapping_is_on}`; `remapping_off_callbacks_pass_without_asking_for_the_engine` (`:1004`, extended) | observed (`impl.md:28`): `E0425 cannot find function note_input_signal`, `E0432 unresolved import ...take_pending_signals`, `E0425 cannot find function control_window_proc`, `E0433 cannot find type ControlWindow`, `E0425 cannot find function desktop_switch_note` | no; red tree = `afbe152` |

Other new tests with no red-first claim (green from their commit): keys tests `resync_from_os_keeps_swallowed_records_frees_their_keys_and_takes_os_keys_as_passed` (`keys.rs:386`), `release_all_frees_every_key_and_keeps_every_record` (`:414`), `resync_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons` (`:592`), `a_swallowed_modifier_freed_by_a_reset_is_left_alone_by_the_modifier_resync` (`:695`); `engine::tests::only_a_signal_that_input_left_asks_for_a_reset` (`mod.rs:1308`); `engine::scenarios::{a_tap_armed_before_a_reload_fires_nothing_at_its_key_up (:4689), a_reload_drops_a_pending_double_tap_and_keeps_each_press_whole (:4038)}` (renamed in C17); the moved Win32 tests in `platform::foreground::tests`.

### Mutations named by the developer (tester repeats; `impl.md:34-38`)

| mutation at HEAD | test that must fail | recorded failure |
|---|---|---|
| M2: `InputSignal::SessionLocked.reset()` -> `None` (`mod.rs:437`) | `engine::scenarios::nothing_is_stuck_after_win_l` | `scenario: down L at +0 ms: expected the event to be suppressed, it was passed` |
| R2: `SessionUnlocked.reset()` -> `Some(InputLeft)` (`mod.rs:440`) | `a_late_unlock_notification_keeps_a_press_made_after_the_unlock`, `a_late_return_notification_keeps_a_press_made_after_the_return_in_either_order` | `repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]` |
| `DesktopSwitchedBack.reset()` -> `Some(InputLeft)` | `a_hold_armed_between_two_return_notifications_fires` | cut `:687`: `advance 300 ms to +300 ms: expected to fire [SwitchLanguage], fired []` (developer: "pins DesktopSwitchedBack instead", not recorded) |
| `DesktopSwitchedAway.reset()` -> `None` | `a_second_lock_releases_a_key_pressed_after_the_first` | cut `:688`, no recorded output |
| keep `active` for `Suppressed` keys in `release_all` (`keys.rs:148-150`) | `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires`, `after_input_leaves_..._fires` | cut `:675`: `down F13 at +1000 ms: expected to fire [SwitchLanguage], fired []`, no recorded output |
| remove the witness (`mod.rs:885` -> `decision.suppress = swallow`) | C12 (a), (b), (c), `a_swallowed_key_pressed_again_on_the_lock_screen_is_released_in_windows` | cut `:646-647,689`: stuck message |
| Drain: remove the reset in `answer_input_signals` (`main_thread.rs:723-728`) | `hook::main_thread::tests::a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key` | `the macro is gone: not within 2 s` |
| C2: drop `fire.extend(deferred_tap)` (`gestures.rs:199`) | C2 test | listed above |

### Acceptance -> proof (`cut.md:790-807`; rulings override)

| acceptance (spec) | proof |
|---|---|
| Tap armed, reload, key-up: old action does not run | `engine::scenarios::a_tap_armed_before_a_reload_fires_nothing_at_its_key_up`; `hook::main_thread::tests::a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing`, `...a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold`; manual M7 |
| Win+Alt+T held across a foreground change keeps T repeats suppressed | `engine::scenarios::win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed` (repeats green on master; release red at C14); property `every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes`; M8 |
| Nothing stuck after Win+L (request path, no session change). Declared change (Architect Q2): reset answers the lock / switch away, not `WTS_SESSION_UNLOCK` | `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key`; `the_session_messages_note_the_lock_and_the_unlock`; `a_desktop_switch_is_classified_by_where_input_is`; `nothing_is_stuck_after_win_l` (M2 mutation); property `no_key_up_reaches_windows_alone_...`; M1, M2, M5 |
| After Win+L the next press of a swallowed non-repeating trigger fires | `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires`; the F14 part of `a_lock_signal_drops_...`; M1 |
| (spec) passed case | `after_a_lock_the_next_press_of_a_passing_key_is_a_new_press`; M2 |
| 100 conditional presses -> one process-name query | `a_hundred_conditional_presses_in_one_window_query_its_process_once` |
| Lone key-up rule after a reset pinned | `a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed`, `a_key_swallowed_before_remapping_off_and_released_after_remapping_on_keeps_its_key_up_swallowed`, property `no_key_up_reaches_windows_alone_...`; M11 |
| R1 no stuck key | C12 (a)-(e), lock variant, property Unseen; M12, M13 |
| R2 reset never discards later state | the four R2 scenarios + `a_late_return_signal_keeps_a_running_macro_and_an_armed_hold`; M1 |
| Control window / Drop guards / features; desktop-switch hook; title current | `hook::control_window::tests::*` (M5: test accepts `Registered` or `Unavailable` + warn; OQ1: `EnumThreadWindows` 0 before create and 0 after drop); `a_desktop_switch_notes_a_signal_only_while_remapping_is_on`; `a_title_condition_reads_the_cached_windows_title_on_each_key_down`; M4, M5, M6, M9 |
| 2.2 leftovers | C1 (doc), C2, C3/C4, C5/C6, C7/C8, C9/C10 |
| Latency bench | `bench::latency_bench` R1-R13, rule D14 |

### Bench (Q7, D14, tester.md section 4b)

- Command (release, each commit in its own scratch worktree and `CARGO_TARGET_DIR`, 3 runs each): `KS_BENCH_OUT=<file> KS_BENCH_LABEL=<label> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1` (`src/bench.rs:10,710`). `tester.md:93` adds `KS_BENCH_LOAD=1` for R8 when the lane touches the main loop; this lane changes `src/hook/main_thread.rs` `run`/`drain` (`:618-686`).
- Commits: `master` `c3dab6a` (R1-R8 only), C11 `dd53797` (R1-R12), HEAD `6f6edb3` (R1-R13).
- Rule (`cut.md:405-414`), median of three p50s: R1-R8 vs master per 2.2 D10 (`D:/tmp/ks-orch/p2-2.2-timers/cut.md:194`), R4 may move by a quarter of R11's allowance; R9 HEAD p50 <= 0.5 x C11 p50 and <= HEAD R12 p50 + 2 us; R10 HEAD p50 <= C11 p50 + 0.5 us; R11 HEAD p50 <= C11 p50 + 0.5 us; R12 HEAD p50 <= 1.05 x C11 p50 + 100 ns and p99 <= 1.5 x C11 p99 + 2 us; R13 p50 <= HEAD R7 p50 + 20 us.
- Latency ruling (`rulings.md` "Latency"): `results.md` states before/after for R9/R10 vs C15 and master, an ordinary key, and R11. R9/R10 read the bench machine's real foreground window (`cut.md:403`); from C19 `main.refresh_foreground()` runs before them (`src/bench.rs:767-769`).
- Developer ran only a debug bench at C11 and C21 (`impl.md:78`).

### Gate (tester owns the full suite)

`cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked`. Developer: fmt/check/clippy pass at HEAD, every commit except the compile-red C16, C18, C20 (`impl.md:75`); C3, C5, C7, C9, C12, C14 are test-red commits by design (`cut.md:598`).

## Invariants in play

- **R1 hook callback**: `src/hook/keyboard_hook_callback.rs:339-391` (unchanged code<0/off/sentinel order; sentinel `:209`); new hot-path work: witness read `src/engine/mod.rs:885` + `keyboard_hook_callback.rs:165-171`; title read `:177-181` (allocates, as master did); exe name read from cache `:173-175`. Hot-path scan list `src/logging/mod.rs:436-445`.
- **R2 sentinel**: no new `SendInput` builder (Fact 35).
- **R3 shared state**: `PENDING_SIGNALS` thread-local `Cell` `src/hook/main_thread.rs:93`; signals are noted from a window procedure and a WinEvent callback that may run inside other message calls (`:127-130` doc); reset and reload run inside `with_main` from `drain`/`serve` (`:724`, `:758`); `with_main` refuses re-entry `:196-207`; `end_session` runs `uninstall` + `flush` from the window procedure (`src/hook/control_window.rs:222-231`; doc claim "outside the main state's borrow" `:220-221`, UNVERIFIED for every delivery path).
- **R4 orderings**: `desktop_hook` `Release` store / `AcqRel` swap `src/hook/keyboard_hook.rs:218,234-236`; `cancel_pending` `fetch_add(AcqRel)` `src/core/executor/mod.rs:245`.
- **R6 Display**: `ResetReason` `src/engine/mod.rs:393-400`, `InputSignal` `:453-462`; `MainThreadError` `src/hook/main_thread.rs:438-446`.
- **R7 specificity**: `update_bindings` sort `src/hook/keyboard_hook.rs:78-84`, now reached from `MainState::reload` on the main thread.
- **R8 key identity**: witness reads the raw (sided or generic) code `src/engine/mod.rs:882-885`; `resync_from_os` loop `src/engine/keys.rs:126-140` (KS-07).
- **R9 process reads**: `process_image_name` `src/platform/foreground.rs:184-219` (`Owned` guard, `PROCESS_QUERY_LIMITED_INFORMATION`, KS-03).
- **R10 message loop/shutdown**: control window lifetime `src/core/app.rs:215,292-297`; desktop hook install/uninstall `src/hook/keyboard_hook.rs:200-245`; `WM_ENDSESSION` exit `src/hook/control_window.rs:252-256` (KS-01, KS-20).
- **KS IDs**: KS-29 (`book/src/audit.md:622`), KS-31 (`:638`), KS-06 status (`:246`), KS-14 (Suppressed record kept, `src/engine/keys.rs:112-118`), KS-11 generation (`src/engine/mod.rs:654`), KS-33 (`HoldTimerNotStarted`, `src/logging/counters.rs:27-31`), KS-12 mask (`src/engine/mod.rs:886`).

## Open points (from impl.md, cut addendum, rulings)

1. Unverified OS behaviour, each with a manual check (`impl.md:147`): the witness through UIPI and the secure desktop (M12, M13); `UOI_IO` FALSE while the lock screen / Ctrl+Alt+Del / UAC has input (M1, M5, M14); whether the lock screen is a desktop of its own (M1); lock-signal latency (M1, V2-m2); whether the LL hook can report a generic modifier code for injected input (critic OQ2; generic read pinned either way).
2. V2-m1/V2-m2: M1 needs a live lock and is left to the owner; pass criterion is "every `state reset` line before the first return line"; a lock line after a return line is reported to the orchestrator (`impl.md:129`, `cut.md` addendum Round 2).
3. Declared residuals: row 68 (M1, Ctrl+Shift waiting Tap fires with Ctrl down), row 76 (key auto-repeating through a lock fires once more, Q4/architect Q3), row 77 (reload), row 78 (witness residual incl. injected generic release), row 79 (late lock after a short unlock; busy main thread), row 80 (cache race) (`book/src/config/known-issues.md:136,144-148`).
4. Declared deviation Q2: a reload does not resync the key table (`src/engine/mod.rs:628-632,651`); the roadmap wording is answered in CHANGELOG and the Phase-2 book sync.
5. Deviations from the cut (`cut.md` addendum, 11 items): `session_notifications()` test-only; window dropped after `uninstall` (V2-m4); C14 lone-key-up reds; C7 part (2) red traced, not observed; C5 (c) key order; C3 terms 10000/5000 ms; `MainThreadHandle::reload` duplicates `set_enabled`'s body; drain named `answer_input_signals`; `ForegroundCache: Default`; C12 uses `DEFAULT_PAIR`; C22 comment-only.
6. Phase-2 book sync list (non-config pages, not changed here): `book/src/architecture.md:130,146,161,181,186,207,304,321,342,417,543-546`; `book/src/build-and-run.md:111,139-140,146,355,475`; `book/src/overview.md:25`; `book/src/glossary.md:179,195-197,237`; `book/src/roadmap.md` 2.3 status, OQ9, `:808`, coverage rows, the `WTS_SESSION_UNLOCK` acceptance wording; `book/src/audit.md:249,624,640` status lines after merge; `book/src/gaps.md:280` (`impl.md:116-123`).
7. Pre-existing rustdoc warnings (`ConfigError::Missing` private link; unclosed `raw` tag in `src/data/vk_name.rs`) (`impl.md:146`).
8. Line endings: repository stores LF with `core.autocrlf=true`; some touched files LF in the working copy (`impl.md:148`).
9. Manual checks M1-M14 (`impl.md:127-142`); ignored tests M10, M14 only. M12 needs a non-elevated debug daemon run by the human.
10. cfgcheck on 4 lane configs + `default_config.ron`: all `exit 0` (`impl.md:83-109`, `cfgcheck-output.txt`); book build and link check: 0 broken (`linkcheck.txt`).

## Not covered

- Not read: full bodies of the new tests (only names, line anchors and the cut's steps); `src/engine/scenarios.rs` beyond the anchors; `src/engine/properties.rs` script generation beyond `:208-336`; `src/engine/testing.rs` outside the listed ranges; `src/bench.rs` row bodies `:352-560`; README.md, CHANGELOG.md and `book/src/config/*.md` text beyond the anchors; `src/config/default_config.ron` diff; `src/hook/keyboard_hook_callback.rs` `carry_out`/`dispatch` (unchanged by this lane except the scan lists, UNVERIFIED).
- Not run: any build, test, clippy, bench or mdbook command; no red-first or mutation was replayed by the scout.
- Not verified against Microsoft Learn: the pre-event state of `GetAsyncKeyState` in an LL hook, `UOI_IO` semantics, `GetWindowTextW` non-blocking for another process's window, `WM_ENDSESSION` timing.
- `critique.md` and `critique_v2.md` were not re-read; their outcomes are taken from `rulings.md` and the cut addendum.
