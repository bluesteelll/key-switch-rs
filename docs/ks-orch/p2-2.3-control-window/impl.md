# Implementation: p2-2.3-control-window round 1

Branch `feat/control-window` from `master` `c3dab6a`; HEAD `6f6edb3`; tree clean. 22 commits: the cut's C1..C21 in order, plus C22 (comment-only). The developer addendum is at the end of `cut.md`.

## Items

| C | sha | Subject | Red-first |
|---|---|---|---|
| C1 | `0ca09d7` | Executor: the module docs name the threads that hand it actions today | doc only |
| C2 | `a60aa65` | Tests: a waiting Tap whose window closed while its timer was late fires at its own key-down | pin, green; mutation run (drop `fire.extend(deferred_tap)`, `gestures.rs:193`): `scenario: down F13 at +381 ms: expected to fire [SwitchLanguage], fired []` |
| C3 | `19df9d4` | Tests: a gesture timer that cannot be moved earlier is set again by the loop | red: `assertion left == right failed: a failed set leaves the timer set for no deadline the loop knows / left: Some(Instant { .. }) / right: None` |
| C4 | `cb8dc1e` | Hook: a gesture timer that Windows refuses to move earlier is set again | `src/hook/main_thread.rs:395-398` (`self.armed = None` in `GestureTimers::start`); C3 green |
| C5 | `668e80e` | Tests: a refused gesture timer counts only the Holds it drops | red: `a_refused_window_timer_drops_no_gesture` and `a_tap_whose_window_timer_is_refused_...` `left: 1 right: 0`; `a_wake_that_cannot_be_set_counts_only_the_holds_it_drops` `only the Hold is dropped left: 3 right: 1` |
| C6 | `bec45b4` | Engine: a refused gesture timer reports whether it dropped a Hold | `gestures.rs:422` (`forget -> bool`), `mod.rs` `start_timers`/`forget_timer -> bool`, `main_thread.rs` `plan_wake`; C5 green |
| C7 | `a0ae250` | Tests: a passing Ctrl+Shift DoubleTap completes when its own keys go down again | red: `down LShift at +50 ms: expected to fire [ToggleCapsLock], fired []`; M1 scenario `down LCtrl at +50 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(27))]` |
| C8 | `7d2c22f` | Engine: a key of a waiting gesture's own combo does not cancel it | `gestures.rs:306` (`other_key_down(vk: VIRTUAL_KEY, ..)`, own-key arm); C7 green |
| C9 | `5388175` | Tests: the config watcher reports a thread it cannot start | red: `panic paths in src/hook/config_watcher.rs: / 126: .expect("spawn config-watcher thread");` |
| C10 | `5bf7f12` | Watcher: a thread that cannot be started is an error the daemon logs | `config_watcher.rs:158` (`.map_err(notify::Error::io)?`); C9 green |
| C11 | `dd53797` | Bench: R9 to R12 time a conditional key-down, a swallowed key-up and an unbound key | ignored bench; the baseline commit for R9-R12 |
| C12 | `0ef6a99` | Tests: the key-up of a swallowed key that Windows has down reaches Windows | red: `up CapsLock at +0 ms: VK 0x14 is down in Windows but physically up`; counting Env `a swallowed key-up asks the OS about its key once left: 0 right: 1`; property `up CapsLock at +10838 ms: VK 0x14 is down in Windows but physically up / context: config default, seed 8, Unseen mode`; pins (critic's script, remapping-off variant) green |
| C13 | `7eabce9` | Engine: the key-up of a swallowed key passes when Windows has the key down | `src/engine/mod.rs:885`; C12 green; OQ2 tests added (generic Shift, sided LShift) |
| C14 | `765e5b5` | Tests: a swallowed key-down keeps its key-up swallowed across focus changes and resets, and only input leaving releases keys | red: 7 flips `up <key> at +533 ms: expected the event to be suppressed, it was passed`; 3 with the lone check `up F13 at +0 ms: Windows got a lone key-up of F13`; `nothing_is_stuck_after_win_l` `down L at +0 ms: expected the event to be suppressed, it was passed`; `after_a_lock_the_next_press_of_a_passing_key_is_a_new_press` `down F13 at +1000 ms: expected to fire [SwitchLanguage], fired []`; FocusWhole `press of CapsLock: key-down suppressed, up passed (seed 0)`; Unlock `Windows got a lone key-up of CapsLock (seed 7)`; R2 pins, the R1 lock pin and the self-test green |
| C15 | `3246339` | Engine: a swallowed key-down's record survives focus changes and resets; input leaving the desktop releases every key | `mod.rs:381` `ResetReason`, `:414` `InputSignal`, `:608` `on_foreground_changed`, `:642` `reset(env, reason)`; `keys.rs:126` `resync_from_os`, `:148` `release_all`; `main_thread.rs:230` `reset_transient_state`; C14 green |
| C16 | `f7588e4` | Tests: a reload through the main thread drops an armed Tap | red (compile): `E0599 no method named reload ... &mut MainState`, `... MainThreadHandle`, `E0599 no variant named Reload ... Request` |
| C17 | `4fbcc37` | Hook: a reload swaps the bindings and resets the engine on the main thread | `main_thread.rs:240` `MainState::reload`, `:490` `MainThreadHandle::reload`, `:763` late line; `config_watcher.rs:43` `ApplyReload`, `:190` `reload_into`; `app.rs:253`; C16 green |
| C18 | `036a859` | Tests: a hundred conditional presses in one window query its process once | red (compile): `E0425 cannot find type ForegroundCache`, `E0405 cannot find trait WindowQuery`, `E0432 unresolved imports ...ForegroundCache, ...WindowQuery` |
| C19 | `a5140fe` | Hook: the foreground window's executable name is cached per window, outside the keyboard callback | `platform/foreground.rs:79` `ForegroundCache`, `keyboard_hook_callback.rs:146` `HookEnv<'a>`, `:224` split borrow, `main_thread.rs:260` `foreground_changed`, `:611` `refresh_foreground`, `app.rs:224`; C18 green |
| C20 | `afbe152` | Tests: input leaving the desktop asks the main thread for a reset that frees a swallowed key; a late return asks for none | red (compile): `E0425 cannot find function note_input_signal`, `E0432 unresolved import ...take_pending_signals`, `E0425 cannot find function control_window_proc`, `E0433 cannot find type ControlWindow`, `E0425 cannot find function desktop_switch_note` |
| C21 | `bc0a929` | Hook: a hidden control window and the desktop-switch hook reset the state when keyboard input leaves the desktop | `hook/control_window.rs` (new), `platform/desktop.rs:30`, `main_thread.rs:93` `PENDING_SIGNALS`, `:131/:147/:157`, `:710` `answer_input_signals`, `keyboard_hook.rs:37`, `:200`, `keyboard_hook_callback.rs:423`, `:436`, `app.rs:215`, `:296`, `Cargo.toml` (4 features); C20 green |
| C22 | `6f6edb3` | Hook: the docs name the third hook and the requests the main loop serves | comment only; removes a new rustdoc warning |

Tests added (by name): `engine::scenarios::{a_waiting_tap_whose_window_closed_while_its_timer_was_late_fires_at_its_own_key_down, a_passing_ctrl_shift_double_tap_is_not_cancelled_by_its_own_keys, a_waiting_tap_of_a_passing_ctrl_shift_double_tap_fires_with_ctrl_down_in_windows, a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_is_released_in_windows, ..._across_focus_changes_is_released_in_windows, a_swallowed_key_released_while_remapping_is_off_and_pressed_again_unseen_is_released_in_windows, a_swallowed_lshift_key_up_stays_swallowed_while_the_right_shift_is_down_in_windows, a_foreground_change_keeps_a_swallowed_key_up_swallowed (renamed), a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed, a_key_swallowed_before_remapping_off_and_released_after_remapping_on_keeps_its_key_up_swallowed, nothing_is_stuck_after_win_l, after_a_lock_the_next_press_of_a_passing_key_is_a_new_press, after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires, after_input_leaves_for_the_secure_desktop_the_next_press_of_a_swallowed_key_fires, a_late_unlock_notification_keeps_a_press_made_after_the_unlock, a_late_return_notification_keeps_a_press_made_after_the_return_in_either_order, a_hold_armed_between_two_return_notifications_fires, a_second_lock_releases_a_key_pressed_after_the_first, a_swallowed_key_pressed_again_on_the_lock_screen_is_released_in_windows, a_tap_armed_before_a_reload_fires_nothing_at_its_key_up (renamed), a_reload_drops_a_pending_double_tap_and_keeps_each_press_whole (renamed), press_symmetry_check_reports_a_repeat_after_a_lock_that_differs_from_its_key_down (rewritten)}`; `engine::tests::{a_refused_window_timer_drops_no_gesture, only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key, the_witness_of_a_generic_shift_key_up_reads_the_generic_state, only_a_signal_that_input_left_asks_for_a_reset}`; `engine::keys::tests::{resync_from_os_keeps_swallowed_records_frees_their_keys_and_takes_os_keys_as_passed, release_all_frees_every_key_and_keeps_every_record, a_swallowed_modifier_freed_by_a_reset_is_left_alone_by_the_modifier_resync, resync_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons}`; `engine::properties::{nothing_is_left_down_in_windows_when_the_hook_misses_key_downs, every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes, no_key_up_reaches_windows_alone_across_focus_changes_unseen_presses_and_desktop_switches}`; `hook::main_thread::tests::{a_gesture_timer_that_cannot_be_moved_earlier_is_set_again_by_the_loop, a_wake_that_cannot_be_set_counts_only_the_holds_it_drops, a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing, a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold, a_reload_the_watcher_stopped_waiting_for_is_logged_when_it_lands, a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key, a_late_return_signal_keeps_a_running_macro_and_an_armed_hold, the_drain_logs_one_line_per_input_signal}`; `hook::config_watcher::tests::the_watcher_thread_has_no_panic_path`; `hook::keyboard_hook_callback::tests::{a_hundred_conditional_presses_in_one_window_query_its_process_once, a_title_condition_reads_the_cached_windows_title_on_each_key_down, a_window_of_this_process_is_never_asked_for_its_title, a_desktop_switch_is_classified_by_where_input_is, a_desktop_switch_notes_a_signal_only_while_remapping_is_on}` (+ `remapping_off_callbacks_pass_without_asking_for_the_engine` extended); `hook::control_window::tests::{only_the_lock_and_the_unlock_are_input_signals, the_session_messages_note_the_lock_and_the_unlock, the_session_end_messages_are_answered_and_end_the_loop, the_control_window_is_a_hidden_top_level_window_released_on_drop}`; `platform::foreground::tests::{the_cache_queries_again_only_for_another_window, an_invalid_foreground_window_keeps_the_cache, process_image_name_resolves_the_current_process, app_equals_matches_the_current_process, the_win32_queries_read_a_window_of_this_process, a_window_of_this_process_has_no_app_and_is_not_asked_for_its_title, image_file_name_is_the_part_after_the_last_backslash}` (four moved from `data::condition`).

Mutations run by the developer (the tester repeats them):
- M2: `InputSignal::SessionLocked.reset()` returns `None` -> `nothing_is_stuck_after_win_l`: `scenario: down L at +0 ms: expected the event to be suppressed, it was passed`.
- R2: `SessionUnlocked.reset()` returns `Some(InputLeft)` -> `a_late_unlock_notification_...` and `a_late_return_notification_...`: `repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]`. (`a_hold_armed_between_two_return_notifications_fires` pins `DesktopSwitchedBack.reset()` instead.)
- Drain: the reset in `answer_input_signals` removed -> `a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key`: `the macro is gone: not within 2 s`.
- C2: listed above.

## Remarks

critique.md (round 1), as ruled:
- I1 - RESOLVED per R1: witness at `src/engine/mod.rs:885`; pinned by C12 (a) (red on master) and the critic's script C12 (b), `Mode::Unseen`, row 78.
- I2 - RESOLVED per R2: only input-left signals reset (`InputSignal::reset`, `src/engine/mod.rs`; drain `src/hook/main_thread.rs:710`); pinned by the four R2 scenarios (C14) and `a_late_return_signal_keeps_a_running_macro_and_an_armed_hold` (C20).
- M1 - RESOLVED (declared): `a_waiting_tap_of_a_passing_ctrl_shift_double_tap_fires_with_ctrl_down_in_windows`; row 68; triggers-conditions.md 11.4.
- M2 - RESOLVED: `nothing_is_stuck_after_win_l` asserts `expect_suppressed`; mutation result above.
- M3 - RESOLVED at `src/core/app.rs:224`.
- M4 - RESOLVED at `src/core/app.rs:215`/`:296` (with V2-m4).
- M5 - RESOLVED: both outcomes named in the window test.
- M6 - RESOLVED: `hook::control_window::tests::{the_session_messages_note_the_lock_and_the_unlock, the_session_end_messages_are_answered_and_end_the_loop}`.
- M7 - RESOLVED: `gestures.rs` module doc, `engine/mod.rs` purity paragraph and two test comments.
- M8 - RESOLVED: row 80, triggers-conditions.md 12.4.
- OQ1-OQ5 - answered in the cut; OQ4 implemented (`main_thread.rs:763`, pinned by `a_reload_the_watcher_stopped_waiting_for_is_logged_when_it_lands`).

critique_v2.md (round 2) and rulings round 2: all recorded in the cut's developer addendum (V2-m1..m4, critic OQ1..OQ3, architect Q1..Q3). In short: V2-m1 invariant stated, M1 left to the owner (needs a live lock); V2-m2 residual in row 79; V2-m3 in rows 63/79 and the CHANGELOG; V2-m4 at `app.rs:296`; OQ1 asserted (EnumThreadWindows: 0 windows after the drop, green here); OQ2 generic read + sided pin; OQ3 `CONTROL_CLASS` mutex.

## Deviations from the cut

See the developer addendum in `cut.md` (11 items). The ones that change shape: `session_notifications()` is test-only; the window drop sits after `uninstall` (V2-m4); three C14 reds show the lone-key-up message; C7 part (2)'s master red is traced, not observed.

## Unsafe / atomics added

- `src/hook/keyboard_hook.rs:37` `desktop_hook: AtomicPtr` - `store(Release)` in `install` (`:218`), `swap(null, AcqRel)` in `uninstall` (`:234-236`), both on the main thread, the pairing of the existing two handles.
- `src/hook/keyboard_hook.rs:200` SetWinEventHook - SAFETY: "WINEVENT_OUTOFCONTEXT delivers the events to this thread's message queue, which the main loop pumps, and the callback is a 'static extern "system" fn with no captured state"; `:241` UnhookWinEvent - "the handle came from SetWinEventHook in `install`, and the swap above took it out of the field, so it is unhooked at most once".
- `src/hook/control_window.rs`: `GetModuleHandleW(None)` ("a null module name asks for the handle of the file that made this process"), `RegisterClassW` ("`class` is a local that outlives the call ... a 'static extern "system" fn and its name a static wide literal"), `CreateWindowExW` ("WS_POPUP without WS_VISIBLE is never shown ..."), `WTSRegisterSessionNotification` / `WTSUnRegisterSessionNotification` ("the live window this registration was made for; the window guard drops after this one"), `DestroyWindow` ("HWND is not Send, so this drop runs on the creating thread"), `UnregisterClassW` ("no window of the class exists any more, because the window guard drops first"), `PostQuitMessage`, `DefWindowProcW` ("the exact arguments the system gave this window procedure").
- `src/platform/desktop.rs:41`, `:47` GetThreadDesktop / GetUserObjectInformationW - "You do not need to call the CloseDesktop function"; "`value` is a local BOOL that outlives the call, and the length passed is its size".
- `src/platform/foreground.rs` GetForegroundWindow, GetWindowThreadProcessId ("`pid` is a local that outlives the call"), GetWindowTextW ("the wrapper passes its length, so the call writes within it"); OpenProcess / QueryFullProcessImageNameW moved unchanged from `data::condition`.
- `src/hook/main_thread.rs:126` PostThreadMessageW(GetCurrentThreadId(), MAIN_WAKE) - "takes the thread id and the message by value, reads no memory of ours and returns without waiting; MAIN_WAKE carries no pointer".
- `src/platform/timers.rs` `not_a_timer` (test only) CreateEventW.
- Test-only: `control_window_proc` direct calls, `PeekMessageW(WM_QUIT..)`, `EnumThreadWindows` with a counting callback, IsWindow/IsWindowVisible/GetAncestor/GetDesktopWindow; `desktop_switched` direct call.
- `PENDING_SIGNALS` is a main-thread `thread_local` `Cell` (no atomics, no lock). No `Relaxed`, no `static mut`.

## Gates

- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass (`Cargo.lock` unchanged with the four features). `cargo clippy --all-targets --locked -- -D warnings`: pass. Every commit passes them except the declared compile-red C16, C18, C20.
- Tests run: per commit the touched modules; at HEAD `cargo test --locked --lib`: 676 passed, 0 failed, 7 ignored. Full `cargo test --locked` left to the tester.
- `cargo doc --lib --no-deps --locked`: 3 warnings, all pre-existing (`load`/`reload` link the private `ConfigError::Missing`; unclosed `raw` tag in `vk_name.rs`); the one this lane caused was fixed in C22.
- Bench: a debug run of `bench::latency_bench` at C11 and at C21 completes (R1-R13). The release runs on `master` (`c3dab6a`), C11 (`dd53797`) and HEAD (`6f6edb3`) are the tester's (Q7); the comparison rule is cut D14.
- Manual ignores added: `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad` (M10: focus Notepad within 5 s; expects notepad.exe), `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it` (M14: at the unlocked desktop; expects Ok(true)). The moved `data::condition` ignore (`foreground_context_names_the_focused_notepad`) is replaced by the first.

## cfgcheck (methodology.md#validation-harness)

Harness `D:/tmp/ks-orch/p2-2.3-control-window/cfgcheck` (verbatim from the book), built against `feat/control-window`; files in `cfg/`; output with `2>&1` in `cfgcheck-output.txt`:

```text
=== cfg/ctrl_shift_tap_double.ron      (triggers-conditions.md 11.4, README, row 68)
[INFO] loading config from cfg/ctrl_shift_tap_double.ron
OK: 2 bindings (after auto-blocker expansion)
  #0: kind=Combo(KeyCombination { keys: [AnySide(Ctrl), AnySide(Shift)] }) action=PressKey(VIRTUAL_KEY(27)) block_default=false block_original_combo=None condition=Always trigger=Tap(200)
  #1: kind=Combo(KeyCombination { keys: [AnySide(Ctrl), AnySide(Shift)] }) action=ToggleCapsLock block_default=false block_original_combo=None condition=Always trigger=DoubleTap(300)
exit 0
=== cfg/ctrl_shift_double.ron          (triggers-conditions.md 11.8 note)
OK: 1 bindings (after auto-blocker expansion)
  #0: ... action=ToggleCapsLock block_default=false ... trigger=DoubleTap(300)
exit 0
=== cfg/when_notepad.ron               (12.4, row 80; manual M3, M4)
OK: 2 bindings (after auto-blocker expansion)
  #0: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(120))] }) action=PressKey(VIRTUAL_KEY(65)) block_default=true block_original_combo=None condition=AppEquals("notepad.exe") trigger=Immediate
  #1: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(121))] }) action=PressKey(VIRTUAL_KEY(65)) block_default=true block_original_combo=None condition=TitleContains("GitHub") trigger=Immediate
exit 0
=== cfg/manual_checks.ron              (manual M2, M7, M8, M11)
OK: 4 bindings (after auto-blocker expansion)
exit 0
=== src/config/default_config.ron      (comment changed in C17)
[INFO] loading config from D:/key-switch-rs/key-switch-rs/src/config/default_config.ron
[INFO] SwitchLanguage: using default combination
OK: 3 bindings (after auto-blocker expansion)
exit 0
```

## Docs

- Changed in this lane (config semantics): `book/src/config/triggers-conditions.md` (11.3 row-63 bullet, 11.4 own keys + M1, 11.8 note, the reload note, 12.3, 12.4), `known-issues.md` (rows 9, 55 fixed, 63 fixed, 68 grows; new 76, 77, 78, 79, 80), `kinds.md` 10, `structure.md` 2.3 (point 5, "Further points"), `errors.md` 18.6 (gesture-timer row; reload rows; control-window, signal, session-end and guard rows); `README.md`; `src/config/default_config.ron` header; `CHANGELOG.md` `[Unreleased]` (Added 1, Changed 4, Fixed 8).
- `mdbook build book -d D:/tmp/ks-orch/p2-2.3-control-window/book`: built; internal links and anchors (`D:/tmp/ks-orch/phase-0/htmllinks.py`): 20 pages, 2360 links, 0 broken, 0 duplicate ids (`linkcheck.txt`). mdbook-mermaid warns it was built for mdbook 0.5.3 (machine setup, pre-existing).

Left to the Phase-2 book sync (non-config pages, at this tree):
- `book/src/architecture.md:130` (mermaid: add `desktop_switched` and the control window), `:146` (reload goes to the main thread), `:161` ("under the lock" is gone since 2.2; now the cache and the modifier resync), `:181` (main-thread duties: the control window, signals, reloads, the cache), `:186` (`config-watcher` hands the list to the main thread; the spawn is an error path), `:207` (remapping off: `desktop_switched` too), `:304` (`bindings` written by the main thread on reload), `:321` and `:342` (`LastDown::Forgotten` is gone; `release_all`, `resync_from_os`), `:417` (`ForegroundContext::capture` is gone: `ForegroundCache`), `:543-546` (Secure Desktop: input-left reset, the witness).
- `book/src/build-and-run.md:111` (startup: control window, three hooks), `:139-140` (reload lines: the new outcome lines), `:146` (`could not remove the desktop-switch hook` and the guard warns), `:355` (the ignored Notepad check is now `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad`; add `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it`), `:475` (the lone key-up after a foreground change is fixed).
- `book/src/overview.md:25` (hot reload resets pending gestures on the main thread).
- `book/src/glossary.md:179` (Env: cached exe, live title), `:195-197` (Foreground window: the cache), `:237` (hook path: the witness read, no process query).
- `book/src/roadmap.md`: 2.3 status; open question 9 (implemented); the follow-up at `:808` (the cut rejects the per-repeat OS check); the coverage rows; the acceptance wording "the reset on WTS_SESSION_UNLOCK" -> the reset answers the lock and the switch away (Architect Q2; the CHANGELOG `Added` entry states it); 2.2's "Tap and DoubleTap ... survives a reload" if present.
- `book/src/audit.md` status lines KS-06 `:249` (lone key-up after a focus change fixed), KS-29 `:624` (fixed for a lock, Ctrl+Alt+Del and UAC on the secure desktop), KS-31 `:640` (reload half fixed) - "Fixed in <sha> (feat/control-window)" once merged.
- `book/src/gaps.md:280` (`ForegroundContext::app()` is gone; the cache and the Win32 query tests in `platform::foreground`).

## Manual checks (human only; none run by an agent)

| # | Precondition | Steps | Expected |
|---|---|---|---|
| M1 | release daemon, default config, two layouts | hold CapsLock, Win+L, release everything on the lock screen, unlock, press CapsLock once | the layout switches once. `daemon.log`: `session locked` and/or `keyboard input switched to another desktop`, one or two `state reset: keyboard input left this desktop`, then `keyboard input switched back to this desktop` and/or `session unlocked`. Pass criterion (V2-m1): every `state reset` line comes before the first return line. Lines of one drain share a time and come in the order lock, unlock, switch away, switch back, then the reset. Paste the lines with their times into the lane's results; a lock line after a return line is reported to the orchestrator (V2-m2) |
| M2 | add `( keys: "F9", action: SwitchLanguage, block_default: false )` | hold F9, Win+L, release on the lock screen, unlock, press F9; then hold Shift, Win+L, release Shift there, unlock, press F9 | the layout switches each time; F9 is not read as Shift+F9 |
| M3 | `( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") )` | F9 in Notepad; Alt+Tab to another app, F9 at once; back to Notepad, F9 | `a`; F9 reaches the other app; `a` |
| M4 | `( keys: "F10", action: PressKey("A"), when: TitleContains("GitHub") )`, a browser with a GitHub tab | Ctrl+Tab between tabs, F10 in each | `a` only in the GitHub tab |
| M5 | default config; "User Account Control: Switch to the secure desktop when prompting for elevation" on (the default; readable at `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System`, value `PromptOnSecureDesktop` = 1) | start an elevated program and cancel the UAC prompt; Ctrl+Alt+Del, Esc; type in Notepad | each time `keyboard input switched to another desktop`, `state reset: ...`, then `keyboard input switched back to this desktop`, with times; typing and CapsLock work |
| M6 | daemon running | sign out, sign in, read `daemon.log` | it ends with `[INFO] the session is ending; removing the keyboard hook and exiting` |
| M7 | `( keys: "F9", action: Launch(exe: "notepad.exe"), trigger: Hold(3000) )` | hold F9, save the config in the editor (it loads), release F9 after 4 s | no Notepad; `config reloaded` in the log |
| M8 | `( keys: "Win+Alt+T", action: Launch(exe: "wt.exe") )` | hold Win+Alt+T until the terminal opens, keep T held 1 s, release | no `t` in the terminal; no stuck key (`osk.exe`) |
| M9 | daemon running | Alt+Tab, taskbar | no daemon window besides its console |
| M10 | Notepad at hand | `cargo test --locked -- --ignored --exact platform::foreground::tests::the_foreground_cache_names_the_focused_notepad`, focus Notepad within 5 s | passes |
| M11 | `( keys: "PrintScreen", action: DoNothing )` | hold PrintScreen, Ctrl+Alt+Del, Esc, release PrintScreen | no screenshot, no Snipping Tool |
| M12 | non-elevated debug daemon, default config; Notepad and an elevated Task Manager | hold CapsLock in Notepad; click into Task Manager; release CapsLock; press and hold it again; click back into Notepad; release | CapsLock not stuck (`osk.exe`); the next CapsLock switches the layout once |
| M13 | release daemon, default config | hold CapsLock; Ctrl+Alt+Del; there release CapsLock and press it again, keep it held; Esc; release on the desktop | CapsLock not stuck; the next press switches the layout once |
| M14 | at the unlocked desktop | `cargo test --locked -- --ignored --exact platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it` | passes (`Ok(true)`) |

## Open points / pre-existing warnings

- Pre-existing rustdoc warnings (not this lane): `load`/`reload` link the private `ConfigError::Missing`; unclosed HTML tag `raw` in `src/data/vk_name.rs`.
- Unverified OS behaviour the design relies on, each with its manual check: the witness through UIPI and the secure desktop (M12, M13); `UOI_IO` reading FALSE while the lock screen, Ctrl+Alt+Del or UAC has input (M1, M5, M14); whether the lock screen is a desktop of its own (M1); the lock signal's latency (M1, V2-m2); that the LL hook can report a generic modifier code for injected input (critic OQ2; the generic read is pinned either way).
- The working tree's line endings: the repository stores LF (`core.autocrlf=true`); some touched files are LF in the working copy, which git normalises; no content effect.
