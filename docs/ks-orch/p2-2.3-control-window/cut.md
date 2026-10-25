# Cut: p2-2.3-control-window — roadmap 2.3 Control window and state reset (KS-29, KS-31, KS-06 status)

Tree: `D:/key-switch-rs/key-switch-rs`, `master` at `c3dab6a` (re-checked: `.git/refs/heads/master`). Branch `feat/control-window` is cut from it. Every `path:line` is on that tree unless it is marked `@7cb3521` or `@20af28d`. Revision 2: the rulings of 2026-10-19 (`rulings.md`) are applied, and the critique of round 1 (`critique.md`) is answered at the end.

## Revision after rulings

| Ruling / remark | What changed | Where |
|---|---|---|
| **R1** (I1, option b) | New **D15**: an OS witness at a swallowed key-up. A key-up that the engine would swallow passes when `GetAsyncKeyState` reports its key down before the event. The pre-event claim is quoted from Microsoft Learn. Whether the OS state includes key-downs delivered elsewhere is unverified and is settled by M12 and M13. D2 keeps its every-daemon scope, with its premise corrected. Question 3 of round 1 is withdrawn. New commits C12 and C13 come before the rule-L commits. New property mode `Unseen` presses keys the hook does not see. New known-issues row 78. Follow-up F1 is narrowed. | D2, D5, D15, C12, C13, rows 55 and 78 |
| **R2** (I2) | A lost-input reset now answers **only a signal that keyboard input left this desktop**: `WTS_SESSION_LOCK`, or `EVENT_SYSTEM_DESKTOPSWITCH` while `UOI_IO` says another desktop has input (new **D16**). It releases every key (`KeyState::release_all`) and does not read the OS. Signals that input came back (`WTS_SESSION_UNLOCK`, a switch back) reset nothing, however late they arrive; they only refresh the foreground cache. `ResetReason::{SessionUnlocked, DesktopSwitched}` and `may_have_lost_key_events` are replaced by `ResetReason::InputLeft` and the pure `engine::InputSignal`. `PENDING_RESET` is replaced by `PENDING_SIGNALS`, which logs one line per signal (M1). The coalescing sentence of D7 is removed. A fourth `windows` feature is needed (Question 1). The acceptance wording "the reset on WTS_SESSION_UNLOCK" becomes the reset on the lock (Question 2). | D1, D3, D7, D16, C14, C15, C20, C21, rows 63, 76 and 79 |
| M1 | The D10 cost names the Ctrl+Esc case. Known-issues row 68 and §11.4 grow. A scenario in C7 pins `expect_down_in_windows("LCtrl")` when the Tap fires. | D10, C7 |
| M2 | `nothing_is_stuck_after_win_l` asserts `expect_suppressed()` on `down L`. It is now **red** on today's reset (the early-lock race) and is fixed by C15. The mutation for `impl.md` is named. | C14, C15 |
| M3 | `refresh_foreground()` runs right after `hook.install()` succeeds, when the foreground hook is live. | D6, D9, C19 |
| M4 | The control window is created after the `AttachedExecutor` guard and dropped explicitly right after `main.run` returns, before the hooks are removed and before the 2 s executor wait. On early returns it drops before that guard. | D6, C21 |
| M5 | `ControlWindow::session_notifications()` reports `Registered` or `Unavailable`. The test accepts both outcomes, names them, and checks the warn line for `Unavailable`. | D6, C20 |
| M6 | The window procedure is called directly with `WM_WTSSESSION_CHANGE` (lock and unlock), `WM_QUERYENDSESSION` and `WM_ENDSESSION`. | C20 |
| M7 | `gestures.rs:25-26`, `mod.rs:31-38` and `mod.rs:590-594` are added to the touch set (C17, C19). | Touch set |
| M8 | The residual of the foreground race is stated exactly in D9 and in the new known-issues row 80. | D9, row 80 |
| Critic OQ1-OQ5 | OQ1 is answered with Microsoft Learn plus bench row R13. OQ2: the switch back is no longer load-bearing, and F5 names the plan. OQ3: the Risks R1 sentence is corrected. OQ4 is adopted (a log line when a late reload lands). OQ5 is left to 4.5. | Critique response |
| Latency | Bench rows R11 (the witness read), R12 (an ordinary key) and R13 (R7 with a control window). The bench commit moves to C11, before the witness and the cache, so that it is the baseline. | D14, C11 |
| Renumbering | The lane now has 21 commits. The declared red commits are C3, C5, C7, C9, C12, C14, C16, C18 and C20. The bench commit of round 1, which Q7 accepted as "C15", is now C11. | Commits |

## Verdict
CUT READY

All rulings can be met. The three new questions at the end do not block the work, and each has a recommendation.

## Spec re-located

| Spec reference | Current file:line | Status | Note |
|---|---|---|---|
| roadmap 2.3 Why: reload keeps pending entries, `src/hook/tap_state.rs:54` @7cb3521 | `src/hook/config_watcher.rs:158-170` (`reload_into` → `hook.update_bindings` only), `src/hook/keyboard_hook.rs:70-76` (sort, store, `cancel_actions`; no engine reset), `src/engine/mod.rs:59-62` ("a hot reload never has to touch the engine") | moved (the file was removed in `16a0461`) | Defect present: the reload swaps the list on the `config-watcher` thread and never resets the engine (the reload half of KS-31). |
| `src/hook/chord_state.rs:114` @7cb3521 | `src/engine/chords.rs` (a pending chord keeps a clone of its action); the only reset is `Engine::reset` at `src/engine/mod.rs:533-540` | moved | Same defect. |
| foreground change clears blocked bits, `keyboard_hook_callback.rs:176-181` @7cb3521 | `src/hook/keyboard_hook_callback.rs:367-383` → `Engine::on_foreground_changed` `src/engine/mod.rs:512-515` → `KeyState::forget_blocked` `src/engine/keys.rs:213-223` | moved | Repeats already stay swallowed (`keys.rs:193-195`). The key-up passes (`take_blocked`, `keys.rs:203-211`): that is the lone key-up of KS-06. It also rescues the critic's I1 case. D15 replaces that rescue. |
| KS-29 `keyboard_hook_callback.rs:38-43`, `:306` @7cb3521 | `src/engine/keys.rs:74-85` (`active`), `:146-153` (`press` is the repeat detector); key-up at `src/engine/mod.rs:734-763` | moved | A key-up the hook never saw leaves `active` set, so the next press reads as a repeat (known issue 63). |
| KS-31 `keyboard_hook.rs:83-102`, `:159-178` @7cb3521 | `src/hook/keyboard_hook.rs:132-184` (`install`), `:196-220` (`uninstall`, idempotent: `swap(null)`), `src/hook/main_thread.rs:122-136` (`MainState::set_enabled`) | moved; the off/on half was fixed in `a1b5a27` (2.2) | The reload half is still open (first row). |
| Status after Phase 0: `Engine::reset` @1dfd80a `mod.rs:342-363` | `src/engine/mod.rs:533-540`, `KeyState::reset_from_os` `src/engine/keys.rs:112-135` | moved | Today's reset forgets swallowed key-downs and reads every key from the OS. |
| KS-06 status (lone key-up after a focus change) | key-up rule `src/engine/mod.rs:754-760`, `keys.rs:213-223` | unchanged | Fixed here (D2, D5, D15). |
| the key-up of a swallowed key whose press Windows saw unseen (critique I1) | `src/engine/mod.rs:759-760` (`take_blocked` alone decides); harness `src/engine/testing.rs:309-343` (`up_unseen`, `down_unseen`), `:540-543` (the modelled OS changes after the decision) | unchanged | On master it is stuck unless a focus change came between (C12 (a) is red). |
| `Env::os_key_down` docs: "never asks about the key of the current event" | `src/engine/env.rs:32-45`; `HookEnv::os_key_down` `src/hook/keyboard_hook_callback.rs:146-152` | unchanged | D15 asks about it on purpose, at one point only. |
| every `SendInput` of the daemon | `src/system/system_function.rs:125-159`, `src/core/windows_actions.rs:257-288`, `src/data/sequence.rs:133-139`, `:163-169`, `:195-205`, `src/hook/keyboard_hook_callback.rs:258-283` | unchanged | Each sends its downs and ups in one call (D15). |
| "Latency" bullet (moved from 3.5); 3.5 Why `keyboard_hook_callback.rs:207-221` @7cb3521 | `HookEnv` `src/hook/keyboard_hook_callback.rs:120-161`, `ForegroundContext` `src/data/condition.rs:107-182`, `process_image_name` `:188-223` | moved | Every key-down that evaluates a `when:` runs `GetForegroundWindow`, `GetWindowThreadProcessId`, `OpenProcess`, `QueryFullProcessImageNameW` and `CloseHandle` (plus `GetWindowTextW`) on the hook path. |
| Files: `Cargo.toml` (feature `Win32_System_RemoteDesktop`) | `Cargo.toml:41-53` | absent | Also needs `Win32_Graphics_Gdi` (`RegisterClassW` and `WNDCLASSW` are `#[cfg(feature = "Win32_Graphics_Gdi")]`, `windows-0.62.2/src/Windows/Win32/UI/WindowsAndMessaging/mod.rs:1919-1924`, `:7198-7210`), `Win32_System_LibraryLoader` (`GetModuleHandleW`, `windows-0.62.2/src/Windows/Win32/System/LibraryLoader/mod.rs:224`) and, new in revision 2, `Win32_System_StationsAndDesktops` (`GetThreadDesktop` `windows-0.62.2/src/Windows/Win32/System/StationsAndDesktops/mod.rs:128-132`, `GetUserObjectInformationW` `:139-142`, `UOI_IO` `:377`; the crate's `Cargo.toml:645` declares the feature as `["Win32_System"]`). |
| 2.2 request queue and wake | `Request` `src/hook/main_thread.rs:273-280`, `MainThreadHandle` `:306-367`, `MAIN_WAKE` `:67`, `drain` `:484-499`, `serve` `:528-539`, `pump` `:544-563` | unchanged | `Request::Reload` and the pending signals plug in here. |
| `App::run` order (M3, M4) | `src/core/app.rs:196-211` (executor, `MainThread::new`, `AttachedExecutor::attach` at `:207`, `install` at `:211`), `:270-278` (`main.run`, `drop(raised)`, `drop(main)`, `uninstall`, `executor.stop()`), guard `Drop` `:61-65` (stop waits up to 2 s, `:45-58`) | unchanged | The window's place in this order is decided in D6. |
| m2: dropped-gesture overcount | `src/hook/main_thread.rs:227-240`, `:448-450`, `src/logging/counters.rs:27-30`, `book/src/config/errors.md:198`, `:208` | unchanged | `Engine::forget_timer` `src/engine/mod.rs:489-498` returns nothing. |
| OQ3: `armed` kept after a failed set | `src/hook/main_thread.rs:254-266` (`:258-261`) | unchanged | Defect present. |
| P1: stalled-timer scenario | `src/engine/scenarios.rs:722-725`, rule at `src/engine/gestures.rs:183-193` | no test | Pin added (C2). |
| OQ2: own keys cancel a passing multi-key DoubleTap | `src/engine/gestures.rs:296-334`, arm at `:312` | unchanged | Defect present. |
| `executor/mod.rs:6` names a hold-timer thread | `src/core/executor/mod.rs:5-6` | unchanged | Doc fix (C1). |
| `config_watcher.rs:126` `.expect` | `src/hook/config_watcher.rs:123-126` | unchanged | Becomes an error path (C10). |
| stale docs (M7) | `src/engine/gestures.rs:25-26` ("survives a reload"), `src/engine/mod.rs:31-38` (`ForegroundContext::capture`), `:590-594` ("may capture the foreground window") | unchanged | Rewritten in C17 and C19. |
| harness self-test relying on forgetting | `src/engine/scenarios.rs:4980-4990` (round 1 said `:4981`; the fn is at `:4982`) | unchanged | Rewritten in C14. |
| property modes | `src/engine/properties.rs:192-196` (`Mode`), `:233-251` (draws), `:257-283` (checks per mode), docs `:25-40` | unchanged | New modes `Unseen` (C12), `FocusWhole` and `Unlock` (C14). |
| known-issues rows | `book/src/config/known-issues.md:35` (row 9), `:123` (55), `:127` (59), `:131` (63), `:136` (68); last row 75 at `:143` | unchanged | New rows 76-80. |
| triggers-conditions `when:` | `book/src/config/triggers-conditions.md:50`, `:144`, `:201-215`, `:218-225` | unchanged | Updated. |

## Goal and non-goals

**After the lane:**
1. The main thread owns a hidden top-level control window (not `HWND_MESSAGE`). `WM_WTSSESSION_CHANGE` (`NOTIFY_FOR_THIS_SESSION`), `WM_QUERYENDSESSION` and `WM_ENDSESSION` reach it. A second WinEvent hook, `EVENT_SYSTEM_DESKTOPSWITCH`, sits next to the foreground hook. Drop guards or `uninstall` release every class, window, registration and hook on every exit path, and the window never outlives the pump (M4).
2. `MainState::reset_transient_state(hook, reason)` drops every pending gesture, chord, sequence and owed mask, starts a new timer generation, cancels macros through `KeyboardHook::cancel_actions`, and applies the key rule of D1. It always runs on the main thread, between two events, in three cases:
   - on `swch off` and `swch on` (`RemappingSwitched`);
   - on a reload (`Reloaded`, through a `Request::Reload` that also swaps the list);
   - on a signal that keyboard input left this desktop (`InputLeft`): `WTS_SESSION_LOCK`, or `EVENT_SYSTEM_DESKTOPSWITCH` while `UOI_IO` says another desktop has input.

   A signal that input came back resets nothing (R2).
3. Rule L applies while remapping is on. The key-up of a key whose key-down was swallowed is swallowed across focus changes, reloads, resets and `swch on`, unless Windows has that key down. Then the key-up passes (D15, R1), so no key-up reaches Windows alone and none stays down there.
4. After Win+L, Ctrl+Alt+Del or a UAC prompt, the next press of any key held when input left is a press (D3). A swallowed, non-repeating trigger therefore fires (owner decision of 2026-10-18).
5. The executable name of the foreground window is cached per (HWND, pid). The cache is filled in the foreground callback, after `install`, on `swch on` and when input comes back. The hook path never queries a process. The title is read live, at most once per key-down that evaluates a title condition (D9).
6. The 2.2 leftovers (m2, OQ3, P1, OQ2, the executor doc line and the watcher `.expect`) land as separate commits.

**Non-goals** (later items):
- `TaskbarCreated` and the tray: item 4.6 (F2).
- The Raw Input watchdog: item 2.4.
- `when:` after the match for `sequence`/`chord`, and the 100 µs budget: item 3.5.
- `swch reload`: item 4.3.
- A lost key-up that no input-left signal precedes (row 63's remainder, row 79).
- The lone key-up while remapping is off: row 59, unchanged.
- Repeats of a swallowed key that was pressed again unseen: row 78.
- Console or remote disconnects as input-left signals: F4.

**Items that depend on this one:** 2.4 (`RIDEV_INPUTSINK` needs this window), 4.2 (background mode uses `WM_ENDSESSION`), 4.6 (the tray lives on this window).

## Decisions

### D1: One reset, three reasons, a key rule per reason
**What.** In `engine` (pure):
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetReason { RemappingSwitched, Reloaded, InputLeft }   // Reloaded from C17
impl fmt::Display for ResetReason // "remapping switched" | "config reloaded" | "keyboard input left this desktop"
impl Engine { pub fn reset(&mut self, env: &dyn Env, reason: ResetReason) }
// keys.rs
impl KeyState {
    pub(super) fn resync_from_os(&mut self, env: &dyn Env); // replaces reset_from_os (keys.rs:122-135)
    pub(super) fn release_all(&mut self);                   // every active flag cleared, every record kept, the OS not asked
}
```
`Engine::reset` always does `gestures.clear()`, makes new `Chords` and `Sequences`, sets `mask_owed = false` and bumps `ids.generation` (wrapping). Keys:

| Reason | Key rule |
|---|---|
| `RemappingSwitched` | `resync_from_os`, per code 1..=254 except the five mouse buttons, by the table below |
| `Reloaded` | unchanged (Q2, accepted deviation) |
| `InputLeft` | `release_all` |

`resync_from_os` (`RemappingSwitched` only):

| OS reports the code down | record before | after |
|---|---|---|
| yes | any | held, `Passed` |
| no | `Suppressed` | not held, `Suppressed` kept: the next key-down is a first press, and a key-up still due is swallowed (rule L) unless Windows has the key down by then (D15) |
| no | `Passed`/`Unknown` | not held, `Unknown` |

**Why.**
- `RemappingSwitched`: while remapping was off the engine saw nothing and Windows saw everything, so the OS is the witness for passed keys. It is no witness for swallowed keys (KS-14), so their records survive. A record that has gone wrong meanwhile, because the key was released and pressed again while off, is caught at the key-up by D15.
- `Reloaded`: a reload loses no key event. Reading the OS then races the update of the async state for a key-down that has just passed (`env.rs:36-38`; LowLevelKeyboardProc: "the callback function is called before the asynchronous state of the key is updated"). A passed held key that read as up would become a first press on its next repeat.
- `InputLeft`: every key held may lose its key-up on the other desktop ("the hook procedure of a process running on a particular desktop can only receive messages intended for windows created in the same desktop", Desktops). The OS cannot help here:
  - While input is elsewhere, `GetAsyncKeyState` returns zero ("Reasons for failure include … The current desktop is not the active desktop").
  - A `WTS_SESSION_LOCK` noted before the switch would read keys that are about to lose their key-ups as held. C14's `nothing_is_stuck_after_win_l` is red on today's reset for exactly this reason.

  Releasing every key is the only rule that does not depend on when the reset runs relative to the switch. That is also what makes a second `InputLeft` reset of the same episode idempotent (D7). Records stay, so a key-up that does come after the return passes or is swallowed as its key-down was, and D15 handles a key that Windows has down.

**Rejected:**
- (a) Today's `reset_from_os` for every reason: it forgets swallowed records, which gives lone key-ups.
- (b) An OS read at `InputLeft` (round 1): the early-lock race above.
- (c) Keeping `active` for swallowed keys at `InputLeft`: KS-29, against the owner decision.
- (d) "Resync all 256 keys" on reload: the race above (Q2, accepted).

**Cost.** `RemappingSwitched` makes at most 254 `GetAsyncKeyState` calls on the serve path, as today. `InputLeft` is 256 stores. Nothing is added per key event.

### D2: A foreground change resyncs the modifiers and forgets nothing
**What.** `Engine::on_foreground_changed(env)` becomes `keys.sync_modifiers(env, None)` only. `LastDown::Forgotten` and `KeyState::forget_blocked` are removed (`keys.rs:66-70`, `:213-223`), and `key_down_swallowed` (`keys.rs:139-144`) becomes `== Suppressed`. This applies to every daemon, elevated or not.

**Why (premise corrected, critique I1).** Round 1 said that forgetting "never prevents a stuck key". That was wrong. When a swallowed key's key-up is lost and the key is pressed again where the hook cannot see it, Windows has the key down while the engine still has the old `Suppressed` record. Master's forgetting let that key-up pass, provided a foreground change came in between. D15 catches exactly that case, in every daemon and whether or not a focus change came between: the key-up passes when Windows reports the key down. With D15 in place, forgetting adds nothing but lone key-ups (KS-06 status), so it is dropped everywhere (ruling R1). No elevation query is needed.

**Rejected:**
- Keep forgetting in a non-elevated daemon: the lone key-up stays, and D15 already covers the stuck key.
- Forget only after a focus change into a higher-integrity window: needs `OpenProcessToken` on a process the daemon may not be able to open.

**Cost.** None. A loop over 256 entries per focus change disappears.

### D3: Open question 9 (owner): the press after Win+L fires
**What.** Win+L is answered by an `InputLeft` reset (D1, D7) on `WTS_SESSION_LOCK`, and again on the switch away if the lock screen has a desktop of its own (unverified; M1 records it). `release_all` frees every key, so the next key-down of a key held at the lock is a first press. A swallowed, non-repeating trigger fires once, and a passed key gets a new press. Ctrl+Alt+Del and the UAC prompt are answered the same way through the switch away ("During the user's session, the system switches to the Winlogon desktop when the user presses the CTRL+ALT+DEL key sequence, or when the User Account Control (UAC) dialog box is open", Desktops). Both cases are pinned by scenarios (C14) and by the request-path test (C20).

**Why.** Key-ups on the other desktop never reach the hook. The hook path cannot tell a press after a lost key-up from an auto-repeat (rejected alternatives below). The moment input leaves is the one moment the engine knows that key-ups may be lost. The reset reads no OS state (D1). Round 1's claim that a key-up on the Winlogon desktop clears the key in the async state of the default desktop no longer carries the reset. It still matters for the modifier resync of the next key event (`keys.rs:102-110`), which M2 checks.

**Rejected:**
- (a) An OS check per repeat on the hook path (the roadmap follow-up, `roadmap.md:808`): a `PressKey` of a physically held key, or any other injector, makes the OS see the key up while it is held. It also adds a Win32 call per repeat.
- (b) A time-gap heuristic: FilterKeys stretches the delay, and a key held while another key repeats gets no repeats of its own.
- (c) Round 1: a reset on the unlock that reads the OS. It fails I2 and the early-lock race.
- (d) Keeping `active` for swallowed keys: KS-29.

**Cost.** Nothing on the key path. Residual, row 76 (it now covers every key, not only swallowed ones): a key held down through the whole episode and still auto-repeating afterwards reads its first repeat as a press, and a binding on it fires once more. Its release reaches Windows if Windows has the key down (D15). Question 3.

### D4: A reload is one main-thread request: swap and reset between two events
**What.** `config-watcher` parses on its own thread (unchanged) and hands the list to an `ApplyReload` sink:
```rust
// src/hook/config_watcher.rs
pub(crate) trait ApplyReload: Send + Sync + 'static { fn apply(&self, bindings: Vec<Binding>) -> Result<(), MainThreadError>; }
impl<T: ApplyReload + ?Sized> ApplyReload for &'static T { .. }   // the watcher tests pass leaked_hook(): &'static KeyboardHook
#[cfg(test)] impl ApplyReload for KeyboardHook { /* update_bindings; Ok(()) */ }   // test only: the daemon has no reload without a reset
pub(crate) fn spawn_watcher(config_path: PathBuf, sink: impl ApplyReload) -> notify::Result<RecommendedWatcher>  // was pub
fn reload_into(config_path: &Path, sink: &dyn ApplyReload)
// src/hook/main_thread.rs
enum Request { SetEnabled { enabled: bool, reply: SyncSender<bool> }, Reload { bindings: Vec<Binding>, reply: SyncSender<()> } }
impl MainThreadHandle { pub(crate) fn reload(&self, bindings: Vec<Binding>) -> Result<(), MainThreadError> }  // try_send, wake, recv_timeout(REPLY_TIMEOUT)
impl ApplyReload for MainThreadHandle { .. }
impl MainState { pub(crate) fn reload(&mut self, hook: &KeyboardHook, bindings: Vec<Binding>) }  // hook.update_bindings(bindings) (sort, store, cancel_actions), then self.reset_engine(ResetReason::Reloaded)
```
`serve`'s `Request::Reload { bindings, reply }` arm runs `with_main(|m| m.reload(hook, bindings))`, then `reply.try_send(())`. If that send fails because the watcher stopped waiting (its receiver is gone), it logs `[INFO] config reload carried out late: N bindings live` after the borrow (critic OQ4). `App::run` passes `main.handle()`. The watcher thread logs one outcome:
- `Ok` → `[INFO] config reloaded (N bindings live)` (unchanged);
- `NoAnswer` → `[WARN] config reload: the daemon's main thread did not answer within 2 s; the new bindings go live when it does`;
- `ShuttingDown` → `[INFO] config reload skipped: the daemon is shutting down`;
- `Busy` → `[ERROR] config reload failed: the daemon is busy; try again` and `[INFO] keeping previous bindings live`.

**Why.** If the watcher thread swapped the list and a reset followed separately, a key-up could be decided between the two with the new list and the old pending press, and the old action would fire (the acceptance line). On the main thread, between two events, the swap and the reset are atomic with respect to every key event (`with_main`, `main_thread.rs:100-111`). `update_bindings` stays the only publisher (R7). It already cancels the executor epoch, so `reload` calls `reset_engine`, which does not cancel again.

**Rejected.** Swapping on the watcher thread plus a separate reset request has the race above. Resetting first and swapping later lets a key-down between the two arm from the old list.

**Cost.** One sort (tens of bindings, microseconds) and the drop of the old list on the main thread per reload. The watcher thread waits at most 2 s for the reply.

### D5: Rule L, the witness and the lone key-up of KS-06
**What.** While remapping is on, a key-up passes if and only if its key's latest key-down passed or Windows has the key down (D15). With D1 and D2, nothing erases a `Suppressed` record except that key's own next key-down (`mark_blocked`/`mark_passed`) or its key-up (`take_blocked`). The harness gets `Scenario::check_no_lone_key_up()`: it fails when a key-up passes while the modelled OS had that key up before the event, with remapping on. There are three new property modes:
- `Unseen` (C12): key-downs the hook does not see;
- `FocusWhole` (C14): focus changes only, whole-press symmetry;
- `Unlock` (C14): the `Unseen` steps plus the four input signals.

**Why.** Whether a program acts on a lone key-up (PrintScreen, `VK_APPS`: unverified) does not matter once none reaches Windows. D15 closes the other side: no key-up that Windows needs is swallowed. Residual: while remapping is off the hook passes every event without reading it (`keyboard_hook_callback.rs:325-331`), so a key swallowed before `swch off` and released while off still gives Windows a lone key-up (row 59). The off path stays the fast gate pinned by `remapping_off_keyboard_callback_returns_before_it_times_itself`.

**Rejected.** Reading the event while off, to swallow such key-ups, breaks the 2.2 fast gate and the "remapping off passes untouched" contract.

**Cost.** None beyond D15.

### D6: The control window
**What.** New `src/hook/control_window.rs`, used on the main thread only. It is `!Send` because it holds an `HWND`.
```rust
pub(crate) struct ControlWindow { session: Option<SessionRegistration>, window: OwnedWindow, class: RegisteredClass } // field order = drop order
struct RegisteredClass { hinstance: HINSTANCE }       // Drop: UnregisterClassW(CLASS_NAME, Some(hinstance)), warn on error
struct OwnedWindow(HWND);                              // Drop: DestroyWindow, warn on error
struct SessionRegistration(HWND);                      // Drop: WTSUnRegisterSessionNotification, warn on error
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionNotifications { Registered, Unavailable }
const CLASS_NAME: PCWSTR = w!("key-switch-rs control");
impl ControlWindow {
    pub(crate) fn create() -> windows::core::Result<Self>;
    pub(crate) fn session_notifications(&self) -> SessionNotifications;
    #[cfg(test)] pub(crate) fn hwnd(&self) -> HWND;
}
unsafe extern "system" fn control_window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT; // the tests of this file call it directly (M6)
fn session_signal(wparam: WPARAM) -> Option<InputSignal>;  // WTS_SESSION_LOCK → SessionLocked, WTS_SESSION_UNLOCK → SessionUnlocked, else None
fn session_ends(wparam: WPARAM) -> bool;                    // WM_ENDSESSION: wParam != 0
fn end_session(hook: &KeyboardHook);                        // uninstall (warn on error), one [INFO] line, logging::flush(), PostQuitMessage(0)
```
`create` runs these steps in order:
1. `GetModuleHandleW(None)`.
2. `RegisterClassW(&WNDCLASSW { lpfnWndProc: Some(control_window_proc), hInstance, lpszClassName: CLASS_NAME, .. })`. A return of 0 becomes `Error::from_thread()`, for example `ERROR_CLASS_ALREADY_EXISTS`.
3. `CreateWindowExW(WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, CLASS_NAME, w!("key-switch-rs"), WS_POPUP, 0, 0, 0, 0, None, None, Some(hinstance), None)`. The window is never shown, has no taskbar or Alt+Tab entry and is never activated. It is top-level, so it receives broadcasts.
4. `WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION)`. A failure (`RPC_S_INVALID_BINDING` before Remote Desktop Services have started) is not fatal. It logs `[WARN] could not register for session notifications ({e}); a lock is answered only by the switch to the lock screen's desktop`, sets `session = None`, and `session_notifications()` then returns `Unavailable`.

If a later step fails, the guards already made are dropped.

Window procedure:
- `WM_WTSSESSION_CHANGE`: if `session_signal` gives `Some(s)` and `get_hook().is_enabled()`, call `main_thread::note_input_signal(s)`. Returns 0.
- `WM_QUERYENDSESSION`: returns `LRESULT(1)`.
- `WM_ENDSESSION` with `session_ends`: `end_session(get_hook())` logs `[INFO] the session is ending; removing the keyboard hook and exiting`. Returns 0.
- `WM_CLOSE`: returns 0. Only the Drop guard destroys the window.
- Anything else goes to `DefWindowProcW`.

Nothing in the procedure can panic.

`App::run` (`src/core/app.rs`), in this order:
1. Right after `AttachedExecutor::attach` (`:207`): `let control_window = ControlWindow::create().map_err(|e| format!("could not create the control window: {e}"))?;`. This is fatal, before any hook.
2. `hook.install()?` (`:211`), then `main.refresh_foreground()` (D9, M3).
3. Right after `let ran = main.run(hook);` (`:270`), `drop(control_window)` as the first statement, before `drop(raised)`, `drop(main)`, `hook.uninstall()` and `executor.stop()`.

On every early return (`install` failing at `:211`, `serve` failing at `:222-228`) the window is declared after the `AttachedExecutor` guard, so it drops before that guard's ≤ 2 s stop (`app.rs:45-65`). The window therefore never exists while no thread pumps it (M4). `MAIN_WAKE` stays a thread message (Q1, accepted).

**Why:**
- Message-only windows "do not receive broadcast messages", and `TaskbarCreated` is a broadcast.
- `WTSRegisterSessionNotification`: "When a window no longer requires these notifications, it must call WTSUnRegisterSessionNotification before being destroyed". So the registration guard drops first.
- `UnregisterClassW` fails "if a window still exists that was created with the class". So the class guard drops last.
- `DestroyWindow` must run on the creating thread. The window is `!Send` and is created and dropped in `App::run`.
- M4: "Messages sent between threads are processed only when the receiving thread executes message retrieval code. The sending thread is blocked until the receiving thread processes the message" (SendMessage). `HWND_BROADCAST` reaches "invisible unowned windows". A hidden top-level window that outlives the pump would therefore hang every plain `SendMessage(HWND_BROADCAST, …)` sender for up to 2 s. After `DestroyWindow` the window is no longer a target.
- `WM_ENDSESSION`: "the session can end any time after all applications have returned from processing this message". So the hook goes and the log is flushed inside the handler. The flush (at most 1 s) runs outside `with_main` and off the hook path, with the hook already removed.
- IME (critic OQ1): "The operating system creates a default IME window for every thread" (ImmGetDefaultIMEWnd). A window of this thread, the IME's included, runs code only inside this thread's message calls (the SendMessage quote above) or in calls this thread makes. The control window is never shown or activated, so it takes no focus and no input context. Bench row R13 measures R7 with the control window on the loop thread.

**Rejected:**
- A subclassed `STATIC` window: a test trick, and not a window the tray can own.
- A message-only window: no broadcasts.
- Creating the window inside `MainThread::new()`: every test that makes a main thread would own a top-level window that must pump.
- Creating the window before `AttachedExecutor` (round 1): it would outlive the pump during the executor stop (M4).

**Cost.** Three of the lane's four new `windows` features (D16 adds the fourth; compile time only), one window, one class. The main thread also receives broadcasts (`WM_SETTINGCHANGE` and the like), each one `DefWindowProcW` call, and they are rare.

### D7: Input signals are noted on the main thread; only "input left" resets
**What.**
```rust
// src/engine/mod.rs (pure, from C15)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSignal { SessionLocked, SessionUnlocked, DesktopSwitchedAway, DesktopSwitchedBack }
impl InputSignal {
    pub const ALL: [InputSignal; 4];              // the drain's log order
    pub fn reset(self) -> Option<ResetReason>;    // SessionLocked | DesktopSwitchedAway => Some(InputLeft); else None
    pub fn input_returned(self) -> bool;          // SessionUnlocked | DesktopSwitchedBack
}
impl fmt::Display for InputSignal // "session locked" | "session unlocked" | "keyboard input switched to another desktop" | "keyboard input switched back to this desktop"
// src/hook/main_thread.rs (from C21)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PendingSignals { seen: [bool; 4], unknown: Option<HRESULT> }   // index = position in InputSignal::ALL; const NONE
thread_local! { static PENDING_SIGNALS: Cell<PendingSignals> = const { Cell::new(PendingSignals::NONE) }; }
pub(crate) fn note_input_signal(signal: InputSignal);        // main thread only: sets its flag, then PostThreadMessageW(GetCurrentThreadId(), MAIN_WAKE, 0, 0), result ignored
pub(crate) fn note_unknown_desktop_switch(error: HRESULT);   // the same, for a switch D16 could not classify
pub(crate) fn take_pending_signals() -> Option<PendingSignals>;  // pub(crate) for the adapter's tests
impl MainState {
    pub(crate) fn reset_transient_state(&mut self, hook: &KeyboardHook, reason: ResetReason); // reset_engine + hook.cancel_actions()
    fn reset_engine(&mut self, reason: ResetReason);   // engine.reset(&env, reason); timers.retain(live); RemappingSwitched first resyncs the foreground (from C19)
    pub(crate) fn input_returned(&mut self);           // foreground.resync() (from C21)
}
```
`set_enabled(&mut self, hook, enabled) -> bool` (no `env` parameter any more) calls `reset_transient_state(hook, RemappingSwitched)` when the switch really changes.

Signals come from two sources:
- `control_window_proc` notes the lock and the unlock (D6).
- The WinEvent callback `desktop_switched` (in `keyboard_hook_callback.rs`, which names no `log::`) returns at once while remapping is off. Otherwise it maps `platform::desktop::input_is_on_this_desktop()` (D16) through `desktop_switch_note` to `DesktopSwitchedBack` (`Ok(true)`), `DesktopSwitchedAway` (`Ok(false)`) or `note_unknown_desktop_switch(e.code())` (`Err`).

`KeyboardHook` gets `desktop_hook: AtomicPtr<c_void>`:
- `install` (`keyboard_hook.rs:132-184`) adds `SetWinEventHook(EVENT_SYSTEM_DESKTOPSWITCH, EVENT_SYSTEM_DESKTOPSWITCH, None, Some(keyboard_hook_callback::desktop_switched), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)` after the foreground hook. An invalid handle leads to `uninstall()` and `Err(E_FAIL, "SetWinEventHook failed for EVENT_SYSTEM_DESKTOPSWITCH")`, as for the foreground hook.
- `uninstall` (`:196-220`) unhooks it first (`swap(null, AcqRel)`) and warns `could not remove the desktop-switch hook: …` on error.

The drain (`main_thread.rs:484-499`) serves the requests first. Then, if `take_pending_signals()` gives something:
1. If remapping is off, the signals are dropped: the `swch on` reset resyncs anyway.
2. Otherwise, after the borrow, one `[INFO] {signal}` line per flag in `ALL` order.
3. For a switch that could not be classified: `[WARN] keyboard input switched desktops, but the daemon could not tell where it went ({error}); nothing was reset`.
4. If any flagged signal has `reset() == Some(r)`: `with_main(|m| m.reset_transient_state(hook, r))`, then `[INFO] state reset: keyboard input left this desktop`.
5. If any flagged signal has `input_returned()`: `with_main(|m| m.input_returned())`.

Then the due timers run. The signals count as work for the "busy" pass. `MainThread::new()` and `Drop for MainThread` clear `PENDING_SIGNALS`.

**Why (ruling R2).** The invariant is that a lost-input reset clears only what existed before the input loss it answers. The mechanism has three parts:
1. **Only "input left" resets.** That is `WTS_SESSION_LOCK` ("The session identified by lParam has been locked", WM_WTSSESSION_CHANGE), or a desktop switch noted while another desktop has input (D16). Once input is on another desktop the hook sees nothing (Desktops, quoted in D1). So nothing a key event creates can appear between that moment and the reset. A second "left" signal of the same episode finds only what the first one left: `release_all` twice is `release_all`, and dropping no gestures drops nothing.
2. **"Input came back" resets nothing**, however late it arrives. State built after the return survives any number of such signals in any order: the ruling's pins in C14 and C20.
3. **A new episode always begins with input leaving again**, and a new "left" signal answers it (pin `a_second_lock_releases_a_key_pressed_after_the_first`).

Nothing relies on coalescing or on the order of the two notifications.

`WTS_SESSION_LOCK` is taken by its kind, without asking D16. The lock may be noted before the desktop switches; the order is undocumented. A reset then clears only state from before the lock, and the switch away, if reported, clears again. The other order, a lock noted after the unlock, needs the main thread stalled through the whole lock. Keys typed after the unlock would wait on that same stall.

The reset runs in the drain, never inside a callback. A sent `WM_WTSSESSION_CHANGE` or a WinEvent may be dispatched inside another message call. The posted `MAIN_WAKE` makes the wait return even if the callback ran inside `MsgWaitForMultipleObjectsEx`. `UnhookWinEvent` "fails if called from a thread different from" the installer, so both WinEvent hooks stay in `install`/`uninstall` on the main thread (KS-01).

**Residual (row 79).** A switch away that is noted only after input has come back (the main thread busy for the whole Ctrl+Alt+Del or UAC episode) reads as a return and resets nothing. A key-up lost there makes the next press read as a repeat (row 63's remainder). Without session notifications, a lock is answered only by the desktop switch.

**Rejected:**
- Round 1: a reset on every notification. It breaks I2, and its coalescing claim had no evidence.
- An episode flag cleared by the first key event after a reset: a return notification after that key event resets again. That is exactly the ruling's pin `session_unlocked; down F13; session_unlocked; repeat F13`.
- Event timestamps: `dwmsEventTime` exists for WinEvents only, and `WM_WTSSESSION_CHANGE` carries no time of its own. A boundary would be needed exactly for the notification that has none.
- Treating every desktop switch as "left": the switch back would wipe state made after the return.
- `Request::Reset` through the `SyncSender` (capacity 8): a full queue would lose the reset.
- A reset inside the window procedure.
- One WinEvent hook over the range 3..0x20.

**Cost.** One more WinEvent hook, one `Cell` read per drain, two user32 calls per desktop switch (D16).

### D8: Session end
`WM_QUERYENDSESSION` returns TRUE ("Applications should respect the user's intentions and return TRUE"). `WM_ENDSESSION(TRUE)` is handled as in D6 (Q5, accepted). There is no `ShutdownBlockReasonCreate`. The console `CTRL_CLOSE_EVENT` handler (`app.rs:306-333`) is unchanged.

### D9: Foreground cache: exe name per (HWND, pid), title read live
**What.** New `src/platform/foreground.rs` (it imports nothing from the crate outside tests):
```rust
pub trait WindowQuery {
    fn foreground(&self) -> HWND;                      // GetForegroundWindow
    fn owner(&self, window: HWND) -> u32;              // GetWindowThreadProcessId, 0 = none
    fn image_name(&self, pid: u32) -> Option<String>;  // OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION) + QueryFullProcessImageNameW(PROCESS_NAME_NATIVE), moved from condition.rs:188-231
    fn title(&self, window: HWND) -> Option<String>;   // through fn window_title(): GetWindowTextW into [u16; 512]
}
pub struct Win32Query;
pub struct ForegroundCache { query: Box<dyn WindowQuery>, window: HWND, pid: u32, own: bool, app: Option<String> }
impl ForegroundCache {
    pub fn new() -> Self;                                         // Win32Query, no window
    #[cfg(test)] pub(crate) fn with_query(query: Box<dyn WindowQuery>) -> Self;
    pub fn window_changed(&mut self, window: HWND);               // invalid HWND or owner 0: keep the cache; (window, pid) unchanged: nothing; pid == std::process::id(): own = true, app = None; else app = query.image_name(pid)
    pub fn resync(&mut self);                                     // window_changed(query.foreground())
    pub fn app(&self) -> Option<&str>;
    pub fn read_title(&self) -> Option<String>;                   // None if no window or own; else query.title(window)
}
```
- `MainState { engine, timers, foreground: ForegroundCache }`.
- `MainState::foreground_changed(&mut self, hwnd)` calls `foreground.window_changed(hwnd)`, then `engine.on_foreground_changed(&HookEnv::new(&self.foreground))`.
- `MainThread::refresh_foreground(&self)` is `with_main(|m| m.foreground.resync())`. It is called after `install` (M3), from `reset_engine` for `RemappingSwitched` (foreground events are ignored while remapping is off), and from `input_returned` (D7).
- The extern `foreground_changed` (`keyboard_hook_callback.rs:367-383`) passes its `hwnd` to `MainState::foreground_changed`, under `with_main`, while remapping is on.
- `HookEnv<'a> { foreground: &'a ForegroundCache, title: OnceCell<Option<String>> }`:
  - `foreground_app` is `foreground.app()`: no Win32 call, no allocation;
  - `foreground_title` is `title.get_or_init(|| foreground.read_title())`;
  - `decide_event` splits the borrow: `let MainState { engine, timers, foreground } = main;`.
- `data::condition` loses `ForegroundContext` and its Win32 code (`condition.rs:16-26`, `:107-231`), and no longer imports `windows`.

**Why.** The spec acceptance (100 presses, one query) and the owner's latency rule. On title currency: `GetWindowTextW` on another process's window does not send `WM_GETTEXT` ("This behavior is by design. It allows applications to call GetWindowText without becoming unresponsive if the process that owns the target window is not responding"), so it does not block. A window of this process would get a `WM_GETTEXT`, so `own` skips it. Keying by (HWND, pid) handles a recycled HWND. An invalid HWND keeps the cache.

**Rejected:**
- `EVENT_OBJECT_NAMECHANGE`, scoped or unscoped: it misses console titles (unverified) or wakes the hook thread for every name change on the desktop.
- A multi-entry cache: invalidation work for no acceptance need.
- **`GetForegroundWindow` compared on every evaluation**: one more call per conditional key-down. The residual is stated exactly (M8, row 80). The cache follows `EVENT_SYSTEM_FOREGROUND`, so a key decided before the event of an activation that has just happened sees the previous window's executable, where master read `GetForegroundWindow` live. Typing by hand after Alt+Tab is expected to be decided after the event: the event is queued at the activation, the main thread runs at `THREAD_PRIORITY_HIGHEST`, and the next keystroke comes much later (not measured). But the relative order of a WinEvent and a later low-level hook call in one wake is undocumented. A program that activates a window and injects a key at once, such as AutoHotkey `WinActivate notepad` followed by `Send {F9}`, can be decided against the previous window. `when: AppEquals("notepad.exe")` on F9 then does not fire in Notepad, and a binding of the previous application may fire instead. This is measured under 3.5 (F3).

**Cost.** Hook path: no Win32 calls and no allocations for `AppEquals` (today: 5 calls and 1 `String`); for a title condition, 1 call and 1 `String` (today: 2 and 1). The foreground callback makes at most one process query per real window change, under the borrow.

### D10: OQ2: a passing multi-key DoubleTap is not cancelled by its own keys
**What.** `Gestures::other_key_down(vk: VIRTUAL_KEY, ..)` (was `u16`; the caller at `mod.rs:559-560` passes `vk`). A press in `Phase::AwaitSecond` is skipped, neither cancelled nor its waiting Tap fired, when a key of its own group accepts `vk`: `group.keys.keys.iter().any(|k| k.accepts(vk))` (`key_spec.rs:134-139`). `Phase::Held` is unchanged.

**Why.** Reviewer OQ2 and the spec. Exempting held presses would leave two pending presses of one group. The check costs at most 4 `accepts` calls per waiting press per first key-down, with no allocation.

**Rejected.** A `[u64; 4]` own-key set stored per press is not needed: `AwaitSecond` already holds `GroupKey.keys` (`gestures.rs:78-84`).

**Cost.**
- The second press completes only in the order that arms the group under the same key; the other order ends the window (§11.4).
- **The declared residual (M1).** A waiting Tap that an own key no longer fires now fires at the next key that is not its own, or on the window timer. By then that own key may be down in Windows. On master, the own key's key-down fired the Tap inside the callback, before Windows saw the key. Example: a `Ctrl+Shift` Tap of `PressKey("Escape")` with a DoubleTap, tapped once, then Ctrl+C within the window. Windows gets Ctrl+Esc and Start opens. Known-issues row 68 and §11.4 say so; the modifiers an output is sent with belong to item 3.1. A scenario in C7 pins it.

### D11: m2: count only the Holds that a refused timer drops
- `Gestures::forget(vk, seq) -> bool` returns true if and only if a Hold, held or due, was dropped.
- `Engine::forget_timer(token) -> bool`.
- `Engine::start_timers` returns how many Holds it dropped. The callers (`keyboard_hook_callback.rs:206-213`, `main_thread.rs:520-522`) keep bumping `HoldTimerNotStarted` once per unit.
- `GestureTimers::plan_wake` counts `forget_timer(..) == true` into `Wake::Failed { dropped }`.
- The log line becomes `could not set the gesture timer ({error}); {dropped} pending Holds dropped`.
- The `counters.rs:27-30` doc becomes: "a Hold whose gesture timer could not be set on the main thread never fired".

### D12: OQ3: a failed set clears `armed`
`GestureTimers::start` (`main_thread.rs:256-263`) sets `self.armed = None` on a `wake.set` error, before returning false. Test seam: `#[cfg(test)] pub(crate) fn WaitableTimer::not_a_timer() -> Self`, an event from `CreateEventW`. That `SetWaitableTimer` fails on an event is unverified, so the test asserts that failure as its precondition.

### D13: The watcher thread spawn is an error
`config_watcher.rs:123-126`: `.spawn(..).map_err(notify::Error::io)?` (`notify-8.2.0/src/error.rs:80`). `App::run` already logs the failure and goes on without hot reload (`app.rs:243-249`).

### D14: Bench rows and the comparison rule
New rows in `src/bench.rs`, 2 000 samples each unless stated:

| Row | What it times | Added in |
|---|---|---|
| R9 | `keyboard_hook_callback` key-down of `F13 → DoNothing, when: AppEquals("no-such.exe"), allow_extra_modifiers: true` (the key-up is called but not timed) | C11 |
| R10 | The same with `when: TitleContains("no-such-title")` | C11 |
| R11 | The key-up callback of a swallowed `F13 → DoNothing` press (the key-down is called, not timed): the D15 read | C11 |
| R12 | Down and up of the unbound A through the callback: an ordinary key | C11 |
| R13 | R7 with a `ControlWindow` created on the loop thread before it runs | C21 |

R9 and R10 read the bench machine's real foreground window (read-only). From C19 the bench calls `main.refresh_foreground()` before them, outside the timing.

Comparison rule (tester section 4b; the median of three runs' p50s, as in 2.2 D10):

| Rows | Compared against | Bound | What it bounds |
|---|---|---|---|
| R1-R8 | master | The 2.2 D10 rule (`D:/tmp/ks-orch/p2-2.2-timers/cut.md:194`) | Existing rows. R4 holds one swallowed F13 key-up in four events, so it may move by a quarter of R11's allowance. |
| R9 | C11 | HEAD p50 ≤ 0.5 × C11 p50 and ≤ HEAD R12 p50 + 2 µs | The cache |
| R10 | C11 | HEAD p50 ≤ C11 p50 + 0.5 µs | The title read |
| R11 | C11 | HEAD p50 ≤ C11 p50 + 0.5 µs | The witness: one `GetAsyncKeyState` |
| R12 | C11 | HEAD p50 ≤ 1.05 × C11 p50 + 100 ns, and p99 ≤ 1.5 × C11 p99 + 2 µs | An ordinary key must not get slower |
| R13 | HEAD R7 | HEAD R13 p50 ≤ HEAD R7 p50 + 20 µs | The control window and IME on the loop thread |

C11 is master's key path plus C1-C10, none of which runs in R9-R12:
- C4 changes `GestureTimers::start` only after a failed set;
- C6 changes a refused-timer count;
- C8 visits pending presses, and R9-R12 have none.

The tester runs the bench on master, C11 and HEAD (Q7, accepted, renumbered). `results.md` states the hook-path cost before and after for a conditional binding (R9, R10), for an ordinary key (R12, R1) and for the witness read (R11) (latency ruling). Justified extra work on the hook path: R11's one read, R10's title read (as today).

### D15: The OS witness at a swallowed key-up (ruling R1)
**What.** `Engine::on_key_up` (`src/engine/mod.rs:754-760`) becomes:
```rust
let was_blocked = self.keys.take_blocked(vk.0);          // the record is consumed either way
let swallow = was_blocked || (released.gesture_bound && !was_passed);
// Windows has this key down although the engine swallowed its latest key-down: it saw a key-down the hook
// did not (a higher-integrity window, the secure desktop, a press while remapping was off, an injector).
decision.suppress = swallow && !env.os_key_down(vk);
```
The `&&` short-circuits: a key-up that passes asks the OS nothing, and a key-up the engine would swallow asks once. Gesture resolution is unchanged: a Tap still fires at that key-up. The `Env::os_key_down` docs (`env.rs:32-45`) change from "the engine therefore never asks about the key of the current event" to "… except at a key-up it would swallow, where the state before the event is exactly the question".

**Why.**
- LowLevelKeyboardProc: "When this callback function is called in response to a change in the state of a key, the callback function is called before the asynchronous state of the key is updated." At a key-up, the asynchronous state of that key is therefore the state before the key-up. Down means that Windows holds a key-down of it which the engine did not swallow. Swallowing the key-up would leave the key down in Windows with nothing to repair it. Every later press of that key would be swallowed whole (critique I1).
- The harness models the same ordering: `FakeEnv` changes `os_down` only after the decision (`testing.rs:540-543`).
- One check covers every source of an unseen key-down: UIPI in a non-elevated daemon, the secure desktop, a key released and pressed again while remapping was off, a record kept by D1, and a key `mark_completed` marked while Windows had it down (F1).
- The daemon's own output cannot fool it. Each of its six `SendInput` sites sends its downs and ups in one call (`system_function.rs:125-159`, `windows_actions.rs:257-288`, `sequence.rs:133-139`, `:163-169`, `:195-205`, `keyboard_hook_callback.rs:258-283`; no macro step sends a key-down alone, `sequence.rs:25-61`). "These events are not interspersed with other keyboard or mouse input events inserted either by the user … or by calls to keybd_event, mouse_event, or other calls to SendInput" (SendInput).
- At a key-up that the hook sees, our desktop has input and the foreground window is one the hook can see. So the documented failures of `GetAsyncKeyState` ("The current desktop is not the active desktop", "UIPI prevents the calling thread from accessing the foreground thread") do not apply.

**Unverified:** that the asynchronous state includes a key-down that Windows delivered to a higher-integrity window or to the secure desktop. M12 (UIPI, non-elevated daemon) and M13 (secure desktop) settle it. If it does not, the I1 case stays open for that source only, and row 78 says so.

**Rejected:**
- (a) Forgetting at a focus change: it covers only presses after a focus change and adds lone key-ups.
- (b) An OS read on every key-up: a passed key-up needs no witness, and the read would cost every key-up.
- (c) Also passing the repeats of such a key: not needed for the stuck-key invariant, and it would change KS-06's repeat rule (`mod.rs:697-721`). Left as row 78.

**Cost.** One `GetAsyncKeyState` (~0.1 µs; bench R11) on a key-up the engine would otherwise swallow. Nothing on any other event, and no allocation.

**Residual (row 78):**
- Such a key's auto-repeats stay swallowed until its release, because they follow the old record (`mod.rs:715-716`).
- An Alt or Win in that state does not count for the menu mask, so the property modes `Unseen` and `Unlock` do not check the mask rule.
- A release while another program holds the same key down by injection reaches Windows. A short `SendInput` count that left one of the daemon's own keys down would be released the same way.

### D16: Which desktop has input: `UOI_IO` of the thread's own desktop
**What.** New `src/platform/desktop.rs` (no crate imports):
```rust
/// Whether the desktop of the calling thread is the one that receives user input now.
pub fn input_is_on_this_desktop() -> windows::core::Result<bool>
```
1. `GetThreadDesktop(GetCurrentThreadId())?`. The handle is not closed: "You do not need to call the CloseDesktop function to close the returned handle".
2. `GetUserObjectInformationW(HANDLE(desk.0), UOI_IO, Some(&mut value as *mut BOOL as *mut c_void), size_of::<BOOL>() as u32, None)?`, giving `value.as_bool()`.

The pure classification, `desktop_switch_note(windows::core::Result<bool>) -> DesktopSwitchNote { Signal(InputSignal), Unknown(HRESULT) }`, lives in `keyboard_hook_callback.rs` and is tested alone.

**Why.**
- `UOI_IO`: "TRUE if the hObj parameter is a handle to the desktop object that is receiving input from the user. FALSE otherwise." GetThreadDesktop: "The calling process can use the returned handle in calls to the GetUserObjectInformation … functions."
- Asking about our own desktop needs no access to the other one. "The Winlogon desktop's security descriptor allows access to a very restricted set of accounts, including the LocalSystem account. Applications generally do not carry any of these accounts' SIDs in their tokens and therefore cannot access the Winlogon desktop" (Desktops). So `OpenInputDesktop` fails there for the daemon, and a failure would be ambiguous.
- The secure screen saver desktop ("the system automatically switches to the ScreenSaver desktop") also reads as another desktop.
- NVDA classifies the same event with `OpenInputDesktop(0, False, 0)` in `processDesktopSwitchWinEvent` (https://github.com/nvaccess/nvda/blob/master/source/IAccessibleHandler/__init__.py): non-null means the user desktop, null means the secure desktop. NVDA's WinEvent hooks are out-of-context. This is evidence that the event reaches an out-of-context hook of the default desktop in both directions.

**Unverified:** that `UOI_IO` of the Default desktop reads FALSE while the lock screen, the Ctrl+Alt+Del screen or the UAC prompt is up. M1, M5 and M14 settle it.

**Rejected:**
- `OpenInputDesktop` plus a name comparison: needs access to the other desktop, and makes more calls.
- Classifying by event parity: one undelivered event would invert every later classification.
- Treating every switch as "left": I2.

**Cost.** Two user32 calls per desktop switch, on the main thread in the WinEvent callback. None on the keyboard path.

## Threads

| Code path | Thread | Notes |
|---|---|---|
| `ControlWindow::create`, its Drop | main (`App::run`) | Created after `AttachedExecutor::attach` and before `install`; dropped right after `main.run` returns, before `uninstall` and `executor.stop()`. |
| `control_window_proc` | main, inside `pump`'s `DispatchMessageW` (posted) or a message call (sent) | Only notes signals. `WM_ENDSESSION` uninstalls and flushes, outside `with_main`. |
| `foreground_changed`, `desktop_switched` (WinEvent) | main (out-of-context: "the event is delivered on the same thread that called SetWinEventHook") | `foreground_changed` fills the cache under the borrow. `desktop_switched` makes two user32 calls (D16) and notes a signal. |
| signal drain, `reset_transient_state` | main, `drain` (signals) and `serve` (`swch on/off`, reload), between events | Logs after the borrow. |
| witness read (D15) | main, keyboard callback, under the borrow | Only at a key-up the engine would swallow. |
| `MainThreadHandle::reload` | `config-watcher` | Waits at most 2 s for the reply. |
| `MainState::reload` | main, `serve` | `update_bindings` plus the engine reset; the late line after the borrow. |
| title read (`ForegroundCache::read_title`) | main, keyboard callback, under the borrow | Other processes only, so no message is sent. |
| bench R9-R13 | the bench's test thread as a main thread (R13: its loop helper) | Read-only Win32 calls. |

There is no new thread, timer, COM apartment or `static mut`. New shared state: `KeyboardHook::desktop_hook: AtomicPtr<c_void>`, written with `store(Release)` in `install` and `swap(null, AcqRel)` in `uninstall`. `PENDING_SIGNALS` and `MainState::foreground` are main-thread locals. The test-only `GLOBAL_HOOK` mutex moves to module level in `keyboard_hook_callback.rs`.

## Win32 API choices

| Purpose | `windows` path | Feature (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| module handle for the class | `Win32::System::LibraryLoader::GetModuleHandleW(None) -> Result<HMODULE>`; `HINSTANCE::from` (`Foundation/mod.rs:5548`) | `Win32_System_LibraryLoader` (no: add) | main | Err makes `create` fail, which is fatal at startup | https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew ("If this parameter is NULL, GetModuleHandle returns a handle to the file used to create the calling process") |
| register and unregister the class | `Win32::UI::WindowsAndMessaging::{RegisterClassW(*const WNDCLASSW) -> u16, UnregisterClassW(PCWSTR, Option<HINSTANCE>) -> Result<()>}` | `Win32_UI_WindowsAndMessaging` (yes) + `Win32_Graphics_Gdi` (no: add) | main | 0 becomes `Error::from_thread()`; an unregister error is a warn in Drop | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerclassw, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unregisterclassw ("Before calling this function, an application must destroy all windows created with the specified class") |
| create and destroy the window | `CreateWindowExW(..) -> Result<HWND>`, `DestroyWindow(HWND) -> Result<()>`, `DefWindowProcW`, `PostQuitMessage` | WindowsAndMessaging (yes) | main | create Err is fatal; destroy Err is a warn | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-destroywindow ("A thread cannot use DestroyWindow to destroy a window created by a different thread"), https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#message-only-windows ("does not receive broadcast messages") |
| why the window must not outlive the pump | (no call) | n/a | n/a | n/a | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagew ("Messages sent between threads are processed only when the receiving thread executes message retrieval code. The sending thread is blocked until the receiving thread processes the message"; HWND_BROADCAST reaches "invisible unowned windows") |
| session notifications | `Win32::System::RemoteDesktop::{WTSRegisterSessionNotification(HWND, u32) -> Result<()>, WTSUnRegisterSessionNotification(HWND) -> Result<()>, NOTIFY_FOR_THIS_SESSION}` (`RemoteDesktop/mod.rs:256`, `:355`, `:9987`); `WM_WTSSESSION_CHANGE`, `WTS_SESSION_LOCK`, `WTS_SESSION_UNLOCK` in WindowsAndMessaging (`mod.rs:7144`, `:7314`, `:7319`) | `Win32_System_RemoteDesktop` (no: add) | main | register Err: warn, `Unavailable`; unregister Err: warn | https://learn.microsoft.com/en-us/windows/win32/api/wtsapi32/nf-wtsapi32-wtsregistersessionnotification, https://learn.microsoft.com/en-us/windows/win32/termserv/wm-wtssession-change ("WTS_SESSION_LOCK (0x7) The session identified by lParam has been locked") |
| session end | `WM_QUERYENDSESSION`, `WM_ENDSESSION` (`mod.rs:7090`, `:6947`) | WindowsAndMessaging (yes) | main | n/a | https://learn.microsoft.com/en-us/windows/win32/shutdown/wm-queryendsession, https://learn.microsoft.com/en-us/windows/win32/shutdown/wm-endsession |
| desktop-switch hook | `Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK}`, `EVENT_SYSTEM_DESKTOPSWITCH` (`WindowsAndMessaging/mod.rs:3461`) | `Win32_UI_Accessibility` (yes) | main (install, uninstall, callback) | invalid handle: `uninstall` + Err (fatal); unhook Err: warn | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook ("The client thread that calls SetWinEventHook must have a message loop in order to receive events"), https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwinevent, https://learn.microsoft.com/en-us/windows/win32/winauto/event-constants ("The active desktop has been switched") |
| which desktop has input (D16) | `Win32::System::StationsAndDesktops::{GetThreadDesktop(u32) -> Result<HDESK>, GetUserObjectInformationW(HANDLE, USER_OBJECT_INFORMATION_INDEX, Option<*mut c_void>, u32, Option<*mut u32>) -> Result<()>, UOI_IO}`; `HANDLE(hdesk.0)` (no `From` impl) | `Win32_System_StationsAndDesktops` (no: add; Question 1) | main (WinEvent callback) | Err: `note_unknown_desktop_switch(code)`, then a warn in the drain, no reset | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getuserobjectinformationw ("UOI_IO … TRUE if the hObj parameter is a handle to the desktop object that is receiving input from the user"), https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getthreaddesktop, https://learn.microsoft.com/en-us/windows/win32/winstation/desktops |
| wake its own loop | `PostThreadMessageW(GetCurrentThreadId(), MAIN_WAKE, ..)` | WindowsAndMessaging, Threading (yes) | main | ignored (the drain also runs when `pump` empties the queue) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew |
| foreground window, owner, exe | `GetForegroundWindow`, `GetWindowThreadProcessId`, `System::Threading::{OpenProcess, QueryFullProcessImageNameW}`, `Owned<HANDLE>` | present | main (callback, reset, startup, return) | `None` | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow, https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew |
| title | `GetWindowTextW(HWND, &mut [u16]) -> i32` | present | main (keyboard callback) | 0 becomes `None` | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowtextw |
| OS key state: witness (D15) and `swch on/off` resync | `GetAsyncKeyState` (the existing `HookEnv::os_key_down`, `keyboard_hook_callback.rs:146-152`) | present | main | 0 on failure ("The current desktop is not the active desktop"; UIPI) | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc ("the callback function is called before the asynchronous state of the key is updated"), https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate |
| own output never interleaves (D15) | (no call) | n/a | n/a | n/a | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput ("These events are not interspersed with other keyboard or mouse input events") |
| test seam: a handle that `SetWaitableTimer` refuses | `System::Threading::CreateEventW` | `Win32_System_Threading` (yes) | test | (unverified) that `SetWaitableTimer` fails on an event; the test asserts it | https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer |
| tests only | `GetAncestor(GA_PARENT)`, `GetDesktopWindow`, `IsWindow`, `IsWindowVisible`, `PeekMessageW(.., WM_QUIT, WM_QUIT, PM_REMOVE)` | present | test | n/a | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getancestor |

No `SendInput` path is added. Elevation and UIPI:
- `WM_WTSSESSION_CHANGE` and the session-end messages come from the system.
- The pipe DACL and token inheritance are untouched.
- D15 depends on UIPI not hiding the key state at a key-up the hook sees (M12).
- D16 needs no access to the Winlogon desktop.

## Touch set

| File | Change | Why |
|---|---|---|
| `Cargo.toml` | Features `Win32_Graphics_Gdi`, `Win32_System_LibraryLoader`, `Win32_System_RemoteDesktop`, `Win32_System_StationsAndDesktops`, sorted into `:41-53`. `Cargo.lock` unchanged: features of an existing dependency (Q6, plus Question 1). | D6, D16 |
| `src/engine/mod.rs` | `ResetReason` (RemappingSwitched and InputLeft in C15, Reloaded in C17, with `Display`); `InputSignal` (`Display`, `ALL`, `reset`, `input_returned`); `pub use`; `reset(env, reason)`; `on_foreground_changed` = sync only; the `on_key_up` witness (C13); `forget_timer -> bool`; `start_timers` doc and count; the `other_key_down(vk, ..)` call. Module docs: `:31-38` (M7, C19), State `:39-62`, Time, Threads, the key-up paragraph of the Decision rules `:191-202` (C13, C15), `:590-594` (M7, C19). Tests. | D1, D2, D7, D11, D15 |
| `src/engine/keys.rs` | Remove `Forgotten` and `forget_blocked`; `reset_from_os` becomes `resync_from_os` with the D1 table; new `release_all`; docs `:11-12`, `:20-24`, `:95-101`, `:137-144`, `:189-195`; tests migrated (list in C15) | D1, D2 |
| `src/engine/gestures.rs` | `other_key_down(vk: VIRTUAL_KEY, ..)` with the own-key skip (C8); `forget -> bool` (C6); docs `:28-32` (C8), `:25-26` ("a reload drops every pending press", M7, C17); unit tests | D10, D11, D4 |
| `src/engine/env.rs` | Docs `:19-21` (cache, C19), `:32-45` (the witness asks about the event's own key at a key-up it would swallow (C13); the reset reasons (C15)) | D9, D15, D1 |
| `src/engine/testing.rs` | `session_locked`, `session_unlocked`, `desktop_switched_away`, `desktop_switched_back`, `check_no_lone_key_up` (C14; routed through `InputSignal::reset` in C15); `reload` resets with `Reloaded` and drops stale timers (C17); `remapping_off/on` pass `RemappingSwitched`; docs `:16-23`, `:44-53` | D1, D4, D5, D7 |
| `src/engine/scenarios.rs` | Flipped and new scenarios (C2, C7, C12, C14, C17); self-test `:4980-4990` rewritten (C14) | acceptance |
| `src/engine/properties.rs` | Modes `Unseen` (C12), `FocusWhole` and `Unlock` (C14); `run` checks per mode (no mask rule in `Unseen` and `Unlock`; `check_no_lone_key_up` in `Unlock`); docs `:1-40` | D5, D15 |
| `src/hook/main_thread.rs` | `MainState.foreground`; `set_enabled(hook, enabled)`; `reset_transient_state`, `reset_engine`, `reload`, `foreground_changed`, `input_returned`; `Request::Reload`; `MainThreadHandle::reload` (+ `ApplyReload`); the late reload line in `serve`; `PendingSignals`, `PENDING_SIGNALS`, `note_input_signal`, `note_unknown_desktop_switch`, `take_pending_signals`; the drain step; `refresh_foreground`; `plan_wake` count; the `start` armed fix; module docs; tests | D4, D7, D9, D11, D12 |
| `src/hook/keyboard_hook.rs` | `desktop_hook` field, install and uninstall; `update_bindings` doc (the daemon calls it on the main thread); struct doc; test helper `set_enabled` | D7, D4 |
| `src/hook/keyboard_hook_callback.rs` | `HookEnv<'a>`; the `decide_event` split borrow; `foreground_changed` → `MainState::foreground_changed(hwnd)`; new `desktop_switched` and `desktop_switch_note`; `#[cfg(test)] pub(crate) static GLOBAL_HOOK` moved out of the tests module (`:417-420`); module docs (`:34-36`, Threads, Re-entry `:83-86`); thread-scan list `:1017-1023` (+ `src/platform/foreground.rs`, `src/platform/desktop.rs`); tests | D7, D9, D16 |
| `src/hook/control_window.rs` (new) | D6, plus tests | D6 |
| `src/hook/mod.rs` | `pub(crate) mod control_window;` | D6 |
| `src/hook/config_watcher.rs` | `ApplyReload`; `spawn_watcher(path, sink)` `pub(crate)`; outcome logs of `reload_into(path, &dyn ApplyReload)`; spawn error; docs `:15-21`; scan test | D4, D13 |
| `src/core/app.rs` | `ControlWindow::create` after `:207`; `refresh_foreground` after `:211`; `drop(control_window)` right after `:270`; `spawn_watcher(path, main.handle())`; `run` docs `:165-179` | D4, D6, D9 |
| `src/core/executor/mod.rs` | `:5-6`: "the hook callback, or the main loop's timer drain for a Hold or a waiting Tap" | spec leftover |
| `src/platform/foreground.rs` (new) | D9, plus tests moved from `condition.rs` | D9 |
| `src/platform/desktop.rs` (new) | `input_is_on_this_desktop`, plus an ignored test (M14) | D16 |
| `src/platform/mod.rs` | `pub mod foreground;` (C18), `pub mod desktop;` (C21), doc `:1-7` | D9, D16 |
| `src/platform/timers.rs` | `#[cfg(test)] not_a_timer()` | D12 |
| `src/data/condition.rs` | Remove `ForegroundContext`, `process_image_name`, `image_file_name` and the Win32 imports; module doc `:1-14`; tests use `FakeForeground`; the Win32 tests move out | D9 |
| `src/logging/counters.rs` | `HoldTimerNotStarted` doc `:27-30` | D11 |
| `src/logging/mod.rs` | `the_functions_on_the_hook_path_log_nothing` list `:393-442`: add `("src/platform/foreground.rs", Some("fn read_title("))`, `(…, Some("fn window_title("))`, `(…, Some("fn app("))` (C19), and `("src/platform/desktop.rs", None)` (C21) | scan |
| `src/bench.rs` | R9-R12 (C11), the cache fill (C19), R13 (C21); module doc rows `:20-44` | D14 |
| `README.md` | Tree `:46`, `:64-72` (+ `control_window.rs`, `platform/foreground.rs`, `platform/desktop.rs`); `keyboard_hook.rs` `:123`, `:126`; `keyboard_hook_callback.rs` `:134`; `main_thread.rs` `:142-145`; `app.rs` `:152-155`; `:145`; deferred rules `:491-501` (own keys), `:514-516`, `:530-535`; `Sequence` `:545-549` (reload); conditions `:609-611`; features `:899-904`; Known Limitations `:918-923` (rows 76, 78, 79, 80) | every change of config meaning, in its commit |
| `src/config/default_config.ron` | Header `:3-7`: a save that loads drops gestures still pending and stops running macros | D4 |
| `CHANGELOG.md` | `[Unreleased]` Added `:14`, Changed `:143`, Fixed `:460` (entries table below) | rule |
| `book/src/config/triggers-conditions.md` | `:50`, `:63` (§11.4 own keys + M1), `:67` (counter wording), `:135` (§11.8), `:144` (reload), `:201-215` (§12.3 test names), `:218-225` (§12.4 + row 80) | config semantics |
| `book/src/config/known-issues.md` | Rows 9 `:35`, 55 `:123`, 59 `:127` (text unchanged, re-checked), 63 `:131`, 68 `:136`; new rows 76-80 (texts below) | config semantics |
| `book/src/config/kinds.md` | §10 `:178-179` (rule L + witness) | D2, D1, D15 |
| `book/src/config/structure.md` | §2.3 `:76`, `:106` | D4 |
| `book/src/config/errors.md` | `:198` wording; new rows (messages below) | messages |

Non-config book pages go to the Phase-2 book sync. The developer lists them in `impl.md` with file:line:
- `book/src/architecture.md:130`, `:146`, `:161`, `:181`, `:186`, `:207`, `:304-305`, `:321`, `:342`, `:417`, `:543-546`;
- `book/src/build-and-run.md:111`, `:139-140`, `:146`, `:355`, `:475`;
- `book/src/overview.md:25`;
- `book/src/glossary.md:179`, `:195-197`, `:237`;
- `book/src/roadmap.md`: 2.3 status, open question 9, follow-up `:808`, the coverage rows, and the acceptance wording "on WTS_SESSION_UNLOCK" (Question 2);
- `book/src/audit.md` status lines KS-06 `:249`, KS-29 `:624`, KS-31 `:640`;
- `book/src/gaps.md:280`.

**New known-issues rows:**

| # | Text | Status | Ref |
|---|---|---|---|
| 76 | A key held down through a lock, a Ctrl+Alt+Del screen or a UAC prompt, and still auto-repeating afterwards: its first repeat counts as a new press, so a binding on it fires once more. Its release reaches Windows when Windows has the key down | By design (roadmap 2.3, open question 9) | `src/engine/keys.rs` (`release_all`); §11.3; KS-29 |
| 77 | A config reload drops every pending `Tap`, `Hold`, `DoubleTap`, chord and sequence and stops running macros. The release of a key whose press the old bindings swallowed stays swallowed. A reload does not re-read the keys from Windows | By design (roadmap 2.3) | `src/hook/main_thread.rs` (`reload`); §2.3; KS-31 |
| 78 | A swallowed key that is pressed again where the daemon cannot see it (a higher-integrity window in a daemon that is not elevated, the secure desktop) keeps its auto-repeats swallowed until it is released. Its release reaches Windows. Such an Alt or Win does not count for the menu mask key. A release while another program holds the same key down by injection also reaches Windows | By design (roadmap 2.3); relies on OS behaviour checked by hand (M12, M13) | `src/engine/mod.rs` (`on_key_up`); §10 |
| 79 | A Ctrl+Alt+Del screen or UAC prompt that comes and goes while the daemon's main thread is busy for its whole duration is not answered: a key released there makes its next press read as an auto-repeat (row 63). When session notifications cannot be registered at startup, a lock is answered only by the switch to the lock screen's desktop | Open | `src/hook/main_thread.rs` (drain), `src/hook/control_window.rs` |
| 80 | A `when:` reads the foreground window's executable from a cache that the foreground event fills. A key that a program sends right after it activates a window (AutoHotkey `WinActivate`, then `Send`) can be decided before that event, against the previous window | Open until roadmap 3.5 | `src/platform/foreground.rs`; §12.4 |

**Changed rows:**
- Row 63: **Fixed** for a lock, a Ctrl+Alt+Del screen and a UAC prompt on `feat/control-window`. Input leaving the desktop releases every key, so the next press is a press (owner decision of 2026-10-18). A key-up lost without a desktop switch, or in row 79's case, still makes the next press read as a repeat.
- Row 55: **Fixed**. The release of a swallowed key stays swallowed across a foreground change, and reaches Windows when Windows has the key down.
- Row 68: gains D10's case (M1).

**errors.md, new rows:**
- `could not create the control window: {e}` (fatal);
- `SetWinEventHook failed for EVENT_SYSTEM_DESKTOPSWITCH` (fatal);
- `[WARN] could not register for session notifications ({e}); a lock is answered only by the switch to the lock screen's desktop`;
- `[INFO] session locked` / `session unlocked` / `keyboard input switched to another desktop` / `keyboard input switched back to this desktop`;
- `[INFO] state reset: keyboard input left this desktop`;
- `[WARN] keyboard input switched desktops, but the daemon could not tell where it went ({e}); nothing was reset`;
- `[INFO] the session is ending; removing the keyboard hook and exiting`;
- the reload rows: `NoAnswer`, `ShuttingDown`, `Busy`, and `config reload carried out late: N bindings live`;
- the Drop-guard warns: `could not unregister the session notifications`, `could not destroy the control window`, `could not unregister the control window class`, `could not remove the desktop-switch hook`;
- `:198` becomes `… N pending Holds dropped`.

## Commits, in order

Subjects are `<Area>: <what>`, and every message ends with the Opus trailer. Every commit passes the gate except the declared red ones: C3, C5, C7, C9, C12, C14, C16, C18 and C20. A "pin" is green on master by design; each pin names the mutation that turns it red, for the tester.

1. **`Executor: the module docs name the threads that hand it actions today`**: `src/core/executor/mod.rs:5-6` only. Green.

2. **`Tests: a waiting Tap whose window closed while its timer was late fires at its own key-down`**: pin (results P1).
   - Test: `engine::scenarios::a_waiting_tap_whose_window_closed_while_its_timer_was_late_fires_at_its_own_key_down` on `TAP_AND_DOUBLE_TAP_F13`.
   - Steps: down F13; +80 up (nothing); `advance_ms_stalled(301)`; down F13 → suppressed, fired `[SwitchLanguage]`, no `ToggleCapsLock`; +50 up → nothing; `advance_ms(300)` → `[SwitchLanguage]`; total `[SwitchLanguage, SwitchLanguage]`.
   - Mutation: drop `fire.extend(deferred_tap)` at `gestures.rs:193`, which gives the red `down F13 at +381 ms: expected to fire [SwitchLanguage], fired []`.

3. **`Tests: a gesture timer that cannot be moved earlier is set again by the loop`**: red.
   - `platform/timers.rs` gets `#[cfg(test)] pub(crate) fn not_a_timer() -> Self`.
   - Test: `hook::main_thread::tests::a_gesture_timer_that_cannot_be_moved_earlier_is_set_again_by_the_loop`, shaped like `a_timer_started_for_an_earlier_deadline_resets_the_wake` (`:735-770`):
     1. Arm an F13 Hold(100).
     2. `timers.wake = WaitableTimer::not_a_timer()`.
     3. Assert that `not_a_timer().set(..)` is `Err` (the precondition).
     4. Start the F14 Hold(50) → `false`.
     5. After the red assert: restore `WaitableTimer::new()?`; `plan_wake` → `Wake::Wait` with `armed == Some(F13 due)`.
   - **PREDICTED RED:** `assert_eq!(timers.armed, None)` → `left: Some(Instant { .. }), right: None`.

4. **`Hook: a gesture timer that Windows refuses to move earlier is set again`**: D12 (`main_thread.rs:258-261`). Makes C3 green. CHANGELOG Fixed (entries table).

5. **`Tests: a refused gesture timer counts only the Holds it drops`**: red.
   - (a) `engine::tests::a_refused_window_timer_drops_no_gesture`. **PREDICTED RED:** `left: 1, right: 0`.
   - (b) The count in `engine::tests::a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes` (`mod.rs:1059-1063`) becomes `0`. Same red.
   - (c) `hook::main_thread::tests::a_wake_that_cannot_be_set_counts_only_the_holds_it_drops`: an F13 Hold(100), an F14 Tap+DoubleTap window timer and a J+K chord expiry; then `wake = not_a_timer()`, `armed = None`, `plan_wake`. **PREDICTED RED:** `dropped == 3`, expected `1`. Then an F15 key-down fires the waiting Tap `[SwitchLanguage]`.

6. **`Engine: a refused gesture timer reports whether it dropped a Hold`**: D11. Makes C5 green. Updates the `counters.rs` doc and `errors.md:198`. CHANGELOG Changed.

7. **`Tests: a passing Ctrl+Shift DoubleTap completes when its own keys go down again`**: red. All in `engine::scenarios::a_passing_ctrl_shift_double_tap_is_not_cancelled_by_its_own_keys` (`check_press_symmetry`, `check_nothing_stuck`), except (3):
   - (1) `( keys: "Ctrl+Shift", action: ToggleCapsLock, trigger: DoubleTap(300) )`: down LCtrl, down LShift, +20 up LShift, up LCtrl, +30 down LCtrl (passed, nothing fired), down LShift. **PREDICTED RED:** `down LShift at +50 ms: expected to fire [ToggleCapsLock], fired []`.
   - (2) The same keys with `Tap(200)` `SwitchLanguage` + `DoubleTap(300)` `ToggleCapsLock`: down LCtrl after the first press → nothing fired. **RED on master:** `expected to fire [], fired [SwitchLanguage]`. Then down LShift → `[ToggleCapsLock]`.
   - (3) **(M1)** `…::a_waiting_tap_of_a_passing_ctrl_shift_double_tap_fires_with_ctrl_down_in_windows` with `( keys: "Ctrl+Shift", action: PressKey("Escape"), trigger: Tap(200) )` and `( keys: "Ctrl+Shift", action: ToggleCapsLock, trigger: DoubleTap(300) )`:
     - Steps: down LCtrl, down LShift, +20 up LShift, up LCtrl, +30 down LCtrl → nothing fired. **RED on master:** `down LCtrl at +50 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(27))]`.
     - Then down C → fired `[PressKey(VIRTUAL_KEY(27))]` and `expect_down_in_windows("LCtrl")`. This pins the declared residual of row 68.
   - `…cancels_the_double_tap` (`:3724`) and `selecting_words_with_ctrl_shift_held_cancels_a_passing_ctrl_shift_hold` (`:3783`) stay green.

8. **`Engine: a key of a waiting gesture's own combo does not cancel it`**: D10. Makes C7 green. Docs: `triggers-conditions.md:63` (§11.4 + M1), `:135`; README `:491-501`; known-issues row 68 (M1). CHANGELOG Fixed.

9. **`Tests: the config watcher reports a thread it cannot start`**: red.
   - Test: `hook::config_watcher::tests::the_watcher_thread_has_no_panic_path` (the non-test lines contain neither `concat!(".exp", "ect(")` nor `concat!(".unw", "rap()")`).
   - **PREDICTED RED:** `panic paths in src/hook/config_watcher.rs:\n126: .expect("spawn config-watcher thread");`.

10. **`Watcher: a thread that cannot be started is an error the daemon logs`**: D13. Makes C9 green. CHANGELOG Fixed.

11. **`Bench: R9 to R12 time a conditional key-down, a swallowed key-up and an unbound key`**: D14. Green (ignored test). This is the baseline for the witness and the cache. Updates the module doc rows `bench.rs:20-44`.

12. **`Tests: the key-up of a swallowed key that Windows has down reaches Windows`**: red (R1).
    - (a) `engine::scenarios::a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_is_released_in_windows`, on the template's `(keys: "CapsLock", action: SwitchLanguage)` with `check_nothing_stuck()`. Steps: down CapsLock → suppressed, `[SwitchLanguage]`; `up_unseen CapsLock`; `down_unseen CapsLock`; up CapsLock → `expect_passed()`. **PREDICTED RED:** `scenario: up CapsLock at +0 ms: VK 0x14 is down in Windows but physically up`. The stuck check in `push_step` (`testing.rs:641-643`) runs before `expect_passed`.
    - (b) Pin: `…_across_focus_changes_is_released_in_windows`, the critic's script verbatim with `check_nothing_stuck()`: down CapsLock, `foreground_changed`, `up_unseen CapsLock`, `down_unseen CapsLock`, `foreground_changed`, up CapsLock → passed. Green on master, because forgetting makes `take_blocked` false (`keys.rs:203-211`). Mutation once C15 lands: remove the witness → the red of (a).
    - (c) Pin: `a_swallowed_key_released_while_remapping_is_off_and_pressed_again_unseen_is_released_in_windows`. Steps: down CapsLock (suppressed), `remapping_off`, up CapsLock (passes unread), `remapping_on`, `down_unseen CapsLock`, up CapsLock → passed, nothing stuck. Green on master (`reset_from_os` forgets the record). Same mutation once C15 keeps `Suppressed` across `RemappingSwitched`.
    - (d) `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key`. A counting `Env` (`RefCell<Vec<VIRTUAL_KEY>>`, nothing down); F13 Immediate `DoNothing` and F14 `DoNothing` `block_default: false`; down/up F13, then down/up F14; asked(F13) == 1, asked(F14) == 0. **PREDICTED RED:** `assertion \`left == right\` failed: a swallowed key-up asks the OS about its key once\n  left: 0\n right: 1`.
    - (e) Properties: `Mode::Unseen` takes the Focus steps, and in `0..35` presses an up key unseen when `rng.below(10) == 0` (that extra draw happens only in this mode). Checks: nothing stuck. No mask rule (row 78), symmetry or repeat check. Test: `nothing_is_left_down_in_windows_when_the_hook_misses_key_downs`. The Strict and Focus scripts are unchanged. **PREDICTED RED:** the first failing seed panics with `up <key> at +<n> ms: VK 0x<code> is down in Windows but physically up`, `context: config <name>, seed <s>, Unseen mode`.

13. **`Engine: the key-up of a swallowed key passes when Windows has the key down`**: D15. Makes C12 green; the rest of the suite is unchanged. The existing `down_unseen` scenarios (`scenarios.rs:2571`, `:2765-2768`, `:4612`, `:5002-5003`) press keys with no `Suppressed` record. Docs: `env.rs:32-45`, the `mod.rs` Decision rules key-up paragraph `:191-202`, `kinds.md` §10, known-issues row 78, README Known Limitations. CHANGELOG Fixed.

14. **`Tests: a swallowed key-down keeps its key-up swallowed across focus changes and resets, and only input leaving releases keys`**: red.
    - **Harness.** In this commit `Scenario::session_locked()` and `desktop_switched_away()` call today's `engine.reset(&env)` plus `drop_stale_timers`; `session_unlocked()` and `desktop_switched_back()` record a step only. All four fail the script while remapping is off, like `foreground_changed`. `Scenario::check_no_lone_key_up()` fails with `"{label}: Windows got a lone key-up of {key}"` when a passed key-up meets a modelled OS that had the key up before the event, with remapping on.
    - **Flips.** Each is **PREDICTED RED:** `up <key> at +<n> ms: expected the event to be suppressed, it was passed`.
      - Rename `foreground_change_forgets_pending_key_up_suppression` (`:206`) to `a_foreground_change_keeps_a_swallowed_key_up_swallowed`; up F13 is expected suppressed.
      - Key-up asserts become `expect_suppressed()` in:
        - `immediate_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:1826`, renamed `…_keeps_its_repeats_and_release_suppressed`);
        - `capslock_held_across_a_focus_change_does_not_reach_windows` (`:1851`);
        - `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_suppressed` (`:1874`, renamed `…_keeps_the_t_repeats_and_release_suppressed`);
        - `capslock_held_across_a_focus_change_switches_the_language_once` (`:2164`);
        - `chord_keys_held_across_a_focus_change_keep_their_repeats_suppressed` (`:2603`);
        - `deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:2634`);
        - `immediate_combo_that_stops_matching_after_a_focus_change_keeps_its_key_suppressed` (`:2676`);
        - `press_after_a_lost_key_up_reads_as_a_repeat_and_fires_only_a_repeating_binding` (`:1897`, renamed `…_without_a_reset_…`; its first up F13 is suppressed).
      - Comment-only updates: `:1166`, `:2580-2600`.
    - **New, same red:**
      - `a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed`: down F13, `session_locked`, `session_unlocked`, up F13 → suppressed.
      - `a_key_swallowed_before_remapping_off_and_released_after_remapping_on_keeps_its_key_up_swallowed`.
    - **`nothing_is_stuck_after_win_l`** (KS-29's example, M2): `( keys: "Ctrl+Alt+L", action: PressKey("F13") )`, `check_nothing_stuck`.
      - Steps: down LWin, down L (both pass), `session_locked`, `up_unseen` LWin, `up_unseen` L, `session_unlocked`, down LCtrl, down LAlt → nothing fired; down L → `expect_fired(&[PressKey(VIRTUAL_KEY(124))])`, **`expect_suppressed()`**; up L, up LAlt, up LCtrl.
      - **PREDICTED RED:** `down L at +0 ms: expected the event to be suppressed, it was passed`. Today's reset reads L as held at the lock (`keys.rs:122-135`), so the next L press is a passed repeat.
      - M2 mutation for `impl.md`, which the tester repeats: after C15, make `InputSignal::SessionLocked.reset()` return `None` → the same red.
    - **`after_a_lock_the_next_press_of_a_passing_key_is_a_new_press`**: `( keys: "F13", action: SwitchLanguage, block_default: false )`, `check_no_lone_key_up`. Steps: down F13 (passed, fired), `session_locked`, `up_unseen F13`, `session_unlocked`, +1000 down F13 → passed, `[SwitchLanguage]`; up passed. **PREDICTED RED:** `down F13 at +1000 ms: expected to fire [SwitchLanguage], fired []`.
    - **Pins, green on today's reset** (it clears a swallowed key whose OS state is up). Mutation after C15: keep `active` for `Suppressed` keys in `release_all` → red `down F13 at +1000 ms: expected to fire [SwitchLanguage], fired []`.
      - `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires`: down F13 fires, `session_locked`, `up_unseen`, `session_unlocked`, +1000 down F13 → suppressed, `[SwitchLanguage]`; up suppressed; `check_no_lone_key_up`.
      - `after_input_leaves_for_the_secure_desktop_the_next_press_of_a_swallowed_key_fires`: the same with `desktop_switched_away` and `desktop_switched_back`.
    - **R2 pins**, green here because the return signals only record a step. Mutations after C15: make `SessionUnlocked.reset()` or `DesktopSwitchedBack.reset()` return `Some(InputLeft)`.
      - `a_late_unlock_notification_keeps_a_press_made_after_the_unlock`, on F13 `SwitchLanguage` with `check_no_lone_key_up`. Two parts:
        - (1) the ruling's sequence verbatim: `session_unlocked`, down F13 (fires), `session_unlocked`, +500 repeat F13 → suppressed, nothing fired; up suppressed;
        - (2) the same preceded by `session_locked`.
        - Mutation red: `repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]`.
      - `a_late_return_notification_keeps_a_press_made_after_the_return_in_either_order`. Two parts:
        - (1) `session_locked`, `desktop_switched_away`, `desktop_switched_back`, down F13, `session_unlocked`, +500 repeat → nothing;
        - (2) `session_locked`, `desktop_switched_away`, `session_unlocked`, down F13, `desktop_switched_back`, +500 repeat → nothing.
        - Same mutation red.
      - `a_hold_armed_between_two_return_notifications_fires`: `( keys: "F13", action: SwitchLanguage, trigger: Hold(300) )`. Steps: `session_locked`, `session_unlocked`, down F13 (suppressed), `desktop_switched_back`, `advance_ms(300)` → `[SwitchLanguage]`; up suppressed. Mutation red: `advance 300 ms to +300 ms: expected to fire [SwitchLanguage], fired []`.
      - `a_second_lock_releases_a_key_pressed_after_the_first` (a genuine new episode): `session_locked`, `session_unlocked`, down F13 (fires), `desktop_switched_away`, `up_unseen F13`, `desktop_switched_back`, +1000 down F13 → fires. Mutation: `DesktopSwitchedAway.reset()` returns `None`.
    - **R1 pin after a reset that never saw the key go down again:** `a_swallowed_key_pressed_again_on_the_lock_screen_is_released_in_windows`, on the template CapsLock with `check_nothing_stuck`. Steps: down CapsLock, `session_locked`, `up_unseen CapsLock`, `down_unseen CapsLock`, `session_unlocked`, up CapsLock → passed. Green (today's reset forgets the record). After C15 it is green through D15. Mutation: remove the witness → stuck.
    - **Harness self-test** `:4982`: rename to `press_symmetry_check_reports_a_repeat_after_a_lock_that_differs_from_its_key_down`. `Ctrl+F13` Immediate `DoNothing`; down LCtrl, down F13 (suppressed), up LCtrl, `session_locked`, repeat F13 → `should_panic(expected = "press of F13: key-down suppressed, repeat passed")`. It also documents row 76. Green on master's reset and after C15.
    - **Properties.**
      - `Mode::FocusWhole`: focus changes, no lost or unseen events; symmetry, nothing stuck, mask rule. Test: `every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes`. **PREDICTED RED:** `press of <key>: key-down suppressed, up passed`.
      - `Mode::Unlock`: the `Unseen` steps; the `95..97` branch draws, with one more draw, among `foreground_changed`, `session_locked`, `session_unlocked`, `desktop_switched_away` and `desktop_switched_back`. Checks: nothing stuck, no lone key-up. Test: `no_key_up_reaches_windows_alone_across_focus_changes_unseen_presses_and_desktop_switches`. **PREDICTED RED:** `Windows got a lone key-up of <key>`. Nothing is stuck, because C13 already holds.

15. **`Engine: a swallowed key-down's record survives focus changes and resets; input leaving the desktop releases every key`**: D1, D2, and the engine half of D7. Makes C14 green.
    - **Engine:** `ResetReason::{RemappingSwitched, InputLeft}` with `Display`; `InputSignal` with `Display`, `ALL`, `reset` and `input_returned`; `reset(env, reason)`; `on_foreground_changed`; in `keys.rs`, `resync_from_os` and `release_all`, with `Forgotten` and `forget_blocked` removed.
    - **Harness:** the four signals go through `InputSignal::reset`; `remapping_off/on` pass `RemappingSwitched`.
    - **Main thread:** `MainState::set_enabled(hook, enabled)` builds its env inside from today's `HookEnv::new()`. The callers and test helpers are `keyboard_hook.rs:259-262`, `:548-549`, `keyboard_hook_callback.rs:424-438` and `serve`. Also `reset_transient_state` and `reset_engine`, without the foreground part, which comes in C19.
    - **Migrated keys tests:**
      - `forget_blocked_resets_every_flag` and `forget_blocked_keeps_passed_key_downs` become `resync_from_os_keeps_swallowed_records_frees_their_keys_and_takes_os_keys_as_passed`;
      - `reset_from_os_takes_held_keyboard_keys_as_passed_and_skips_mouse_buttons` uses `resync_from_os`, with J not held and `is_blocked` true;
      - new `release_all_frees_every_key_and_keeps_every_record`;
      - `completion_marks_only_held_keys_whose_key_down_did_not_pass` loses its `forget_blocked` line;
      - in `alt_or_win_down_in_windows_…`, "a Win freed by `release_all` does not count" replaces the forgotten case;
      - `forgotten_modifier_is_left_alone_by_the_resync` becomes `a_swallowed_modifier_freed_by_a_reset_is_left_alone_by_the_modifier_resync`;
      - `a_token_from_before_a_reset_is_stale` passes `RemappingSwitched`.
    - **Docs:** README `:145`, `:530-535`; `kinds.md:178-179`; known-issues row 55 → Fixed; row 9 text. CHANGELOG Fixed and Changed.

16. **`Tests: a reload through the main thread drops an armed Tap`**: red (compile).
    - `hook::main_thread::tests::a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing`.
    - `…::a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold`, shaped like `remapping_switches_through_the_main_thread_and_drops_pending_timers` (`:945`).
    - `…::a_reload_the_watcher_stopped_waiting_for_is_logged_when_it_lands` (critic OQ4): `MainThread::new()`; `serve(&hook, Request::Reload { bindings, reply })` with the receiver dropped, under `logging::testing::capture` → a line contains `config reload carried out late`.
    - **PREDICTED RED:** `error[E0599]: no method named \`reload\` found for mutable reference \`&mut MainState\``, `… for struct \`MainThreadHandle\``, and `error[E0599]: no variant named \`Reload\` found for enum \`Request\``.

17. **`Hook: a reload swaps the bindings and resets the engine on the main thread`**: D4. Makes C16 green.
    - Code: `ResetReason::Reloaded` with its `Display` arm; `Request::Reload`, `MainThreadHandle::reload`, `MainState::reload` and `ApplyReload`; the watcher sink and its outcome logs; the late line; `App::run`. The harness `reload` calls `reset(.., Reloaded)` and drops stale timers.
    - Scenarios:
      - `:4237` → `a_tap_armed_before_a_reload_fires_nothing_at_its_key_up`;
      - `:3683` → `a_reload_drops_a_pending_double_tap_and_keeps_each_press_whole`;
      - the comment at `:2209-2211`.
    - Docs: `gestures.rs:25-26` (M7); `default_config.ron:3-7`; README `:545-549`, `:126`; `triggers-conditions.md:144`; `structure.md:76`, `:106`; known-issues row 77; errors.md reload rows. CHANGELOG Changed and Fixed.

18. **`Tests: a hundred conditional presses in one window query its process once`**: red (compile). `src/platform/foreground.rs` holds only its tests module, and `pub mod foreground;` goes into `platform/mod.rs`.
    - `hook::keyboard_hook_callback::tests::a_hundred_conditional_presses_in_one_window_query_its_process_once`: a counting `WindowQuery` (fake HWND, pid 4242, `"x.exe"`); F13 `DoNothing` `when: AppEquals("x.exe")`; 100 rounds of down/up → `image_queries == 1`.
    - `…::a_title_condition_reads_the_cached_windows_title_on_each_key_down`.
    - `…::a_window_of_this_process_is_never_asked_for_its_title`.
    - `platform::foreground::tests::the_cache_queries_again_only_for_another_window`.
    - `…::an_invalid_foreground_window_keeps_the_cache`.
    - The moved Win32 tests, plus the ignored `the_foreground_cache_names_the_focused_notepad` (M10).
    - **PREDICTED RED:** `error[E0412]: cannot find type \`ForegroundCache\``, `error[E0599]: no method named \`foreground_changed\` found`.

19. **`Hook: the foreground window's executable name is cached per window, outside the keyboard callback`**: D9. Makes C18 green.
    - Code: `platform/foreground.rs`; `MainState.foreground`; `HookEnv<'a>`; `decide_event`; `foreground_changed(hwnd)`; the `RemappingSwitched` foreground resync in `reset_engine`; `App::run` `refresh_foreground` right after `hook.install()?` (M3); the `condition.rs` cleanup; the bench fill line; the scan lists.
    - Docs: `mod.rs:31-38` and `:590-594` (M7); `env.rs:19-21`; `triggers-conditions.md:201-215`, `:218-225` (+ row 80, M8); README tree, `:46`, `:134`, `:609-611`. CHANGELOG Changed.

20. **`Tests: input leaving the desktop asks the main thread for a reset that frees a swallowed key; a late return asks for none`**: red (compile).
    - **`hook::main_thread::tests::a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key`.** Leaked hook with an F13 `Hold(300)` macro and an F14 Immediate `SwitchLanguage`; FakeRunner executor.
      - On the loop thread: F13 down, F14 down; a `[F15, Delay(10000), F16]` macro submitted and waited for until `active_runs() == 1`; then `note_input_signal(InputSignal::SessionLocked)`; then `main.run`.
      - The waiter checks `active_runs() == 0` within 100 ms and no Hold record within 500 ms, then ends the loop.
      - After `run`: F14 down → `fire == [SwitchLanguage]`, suppressed; F14 up suppressed.
    - **`…::a_late_return_signal_keeps_a_running_macro_and_an_armed_hold`** (R2, adapter level).
      - On the loop thread: `note_input_signal(SessionLocked)`, drained; then F13 down (the Hold arms) and a submitted macro, waited for until running; then `note_input_signal(SessionUnlocked)` and `note_input_signal(DesktopSwitchedBack)`.
      - The waiter sees `active_runs() == 1` after 100 ms and the Hold's record within 500 ms.
    - **`…::the_drain_logs_one_line_per_input_signal`** (M1 support): under `capture`, all four signals and one unknown switch are noted, then `drain` runs. The lines are `session locked`, `session unlocked`, `keyboard input switched to another desktop`, `keyboard input switched back to this desktop`, the warn line, and exactly one `state reset: keyboard input left this desktop`.
    - **`hook::control_window::tests::the_session_messages_note_the_lock_and_the_unlock`** (M6). Under `GLOBAL_HOOK`, with `MainThread::new()`, calls `control_window_proc(HWND::default(), WM_WTSSESSION_CHANGE, WPARAM(WTS_SESSION_LOCK), LPARAM(0))` directly:
      - lock → `take_pending_signals()` holds `SessionLocked` only;
      - `WTS_SESSION_UNLOCK` → `SessionUnlocked`;
      - `WTS_SESSION_LOGON` → `None`;
      - with remapping off (local guard), `WTS_SESSION_LOCK` → `None`.
    - **`…::the_session_end_messages_are_answered_and_end_the_loop`** (M6, `GLOBAL_HOOK`):
      - `WM_QUERYENDSESSION` → `LRESULT(1)`;
      - `WM_ENDSESSION(WPARAM(0))` → 0, with no `WM_QUIT` queued (`PeekMessageW(.., WM_QUIT, WM_QUIT, PM_REMOVE)` false);
      - `WM_ENDSESSION(WPARAM(1))` under `capture` → 0, `WM_QUIT` queued and removed, a line containing `the session is ending`, and `!get_hook().is_installed()`.
      - Nothing is installed in tests, so `uninstall` is a no-op swap; `flush` returns at once with no writer (`logging/mod.rs:162-167`).
    - **`…::the_control_window_is_a_hidden_top_level_window_released_on_drop`** (M5). Under a test mutex, `create()` runs under `capture`. Checks: `IsWindow`, `!IsWindowVisible`, and `GetAncestor(hwnd, GA_PARENT) == GetDesktopWindow()`. The session registration must be one of the two named outcomes:
      - `Registered`;
      - `Unavailable`, with the captured warn `could not register for session notifications` present.

      After drop, `!IsWindow`, and `create()` succeeds again because the class was unregistered.
    - **`…::only_the_lock_and_the_unlock_are_input_signals`**: `session_signal` for lock, unlock, logon, console connect and remote disconnect.
    - **`hook::keyboard_hook_callback::tests::a_desktop_switch_is_classified_by_where_input_is`**: `desktop_switch_note(Ok(true))` → `Signal(DesktopSwitchedBack)`, `Ok(false)` → `Signal(DesktopSwitchedAway)`, `Err(E_ACCESSDENIED.into())` → `Unknown(E_ACCESSDENIED)`.
    - **`…::a_desktop_switch_notes_a_signal_only_while_remapping_is_on`** (`GLOBAL_HOOK`): the extern `desktop_switched` with null handles leaves exactly one pending note while remapping is on: away, back or unknown, whichever this machine's desktop reports. All three are named. With remapping off, nothing is noted.
    - **`platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it`**: `#[ignore = "manual: run at the unlocked desktop; expects Ok(true)"]` (M14).
    - **PREDICTED RED:** `error[E0432]: unresolved import \`crate::hook::control_window\``, `error[E0425]: cannot find function \`note_input_signal\` in module \`crate::hook::main_thread\``.

21. **`Hook: a hidden control window and the desktop-switch hook reset the state when keyboard input leaves the desktop`**: D6, D7, D8, D16. Makes C20 green.
    - Code: `Cargo.toml` (4 features); `control_window.rs`; `platform/desktop.rs`; `hook/mod.rs`; `platform/mod.rs`; `PendingSignals`, `note_input_signal` and the drain step; `MainState::input_returned`; `desktop_hook` install and uninstall; `desktop_switched`; `App::run` (window after the executor guard, dropped right after the loop); `GLOBAL_HOOK` at module level; bench R13.
    - `remapping_off_callbacks_pass_without_asking_for_the_engine` (`:798`) also asserts that `desktop_switched` leaves no pending note while remapping is off.
    - Docs: `triggers-conditions.md:50`; known-issues row 63 → Fixed, rows 76 and 79; README `:123`, `:142-145`, `:152-155`, tree, `:904`, `:918-923`; errors.md rows. CHANGELOG Added and Fixed.

After C21: the bench runs three times at master, C11 and HEAD (`bench-*.md` in the lane folder). The numbers and the D14 comparison go into `impl.md`, with the non-config book list, M2's mutation result and M1's log lines.

## CHANGELOG entries (`[Unreleased]`)

| Commit | Section | Entry |
|---|---|---|
| C4 | Fixed | A gesture timer that Windows refused to move to an earlier Hold no longer leaves the main loop waiting on the old deadline (2.2 review OQ3). |
| C6 | Changed | When the gesture timer cannot be set, the log line and the `hold_timer_not_started` counter count only the Holds that were dropped; a waiting Tap fires at the next key event (2.2 review m2). |
| C8 | Fixed | A passing multi-key DoubleTap such as `Ctrl+Shift` is no longer cancelled by its own keys going down again. Its waiting Tap fires at the next other key, while that modifier is down in Windows (known issue 68; 2.2 review OQ2). |
| C10 | Fixed | The config watcher reports a thread it cannot start instead of panicking; the daemon goes on without hot reload (2.2 hand-off H-B5). |
| C13 | Fixed | The release of a key whose press the daemon swallowed reaches Windows when Windows has that key down. A key pressed again where the daemon cannot see it (a higher-integrity window, the secure desktop) no longer stays stuck (roadmap 2.3; known issue 78). |
| C15 | Fixed | The release of a swallowed key no longer reaches Windows alone after a foreground change or `swch on` (KS-06 status, known issue 55). |
| C15 | Changed | `swch off` and `swch on` keep a swallowed key's release swallowed. A foreground change no longer forgets swallowed key-downs, in every daemon (roadmap 2.3). |
| C17 | Changed | A config reload drops pending `Tap`, `Hold`, `DoubleTap`, chord and sequence entries and is applied on the daemon's main thread between two key events. It does not re-read the keys from Windows, because no key event is lost on a reload (roadmap 2.3; known issue 77). |
| C17 | Fixed | A `Tap` armed before a reload no longer fires the old action at its key-up (KS-31). |
| C19 | Changed | A `when:` condition reads the executable name from a per-window cache that the foreground event fills, so no process is queried on the key path; titles are read once per key-down that needs one (roadmap 2.3 latency; known issue 80). |
| C21 | Added | A hidden control window owned by the main thread (session notifications, session end), and an `EVENT_SYSTEM_DESKTOPSWITCH` hook. On a lock, Ctrl+Alt+Del or a UAC prompt, the daemon drops pending gestures, stops macros and releases every held key when keyboard input leaves the desktop, not when it returns, so nothing done after the return is lost. Each such event is logged. At sign-out the daemon removes its hook, flushes the log and exits (roadmap 2.3). |
| C21 | Fixed | After Win+L, Ctrl+Alt+Del or a UAC prompt, the next press of a key held when input left is a press: a swallowed, non-repeating trigger fires (KS-29, known issue 63, owner decision 2026-10-18). |

## Acceptance mapping

| Acceptance line | Covered by |
|---|---|
| With a Tap armed, a reload followed by the key-up does not run the old action | `engine::scenarios::a_tap_armed_before_a_reload_fires_nothing_at_its_key_up` (C17); `hook::main_thread::tests::a_reload_on_the_main_thread_drops_an_armed_tap_so_its_key_up_fires_nothing` and `…::a_reload_request_is_carried_out_by_the_main_thread_and_drops_a_pending_hold` (C16 → C17); manual M7 |
| Holding Win+Alt+T while the foreground window changes keeps the T repeats suppressed | `engine::scenarios::win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_and_release_suppressed` (the repeats are green on master; the release is red in C14 and green from C15); property `every_press_is_suppressed_or_passed_as_a_whole_across_focus_changes`; manual M8 |
| Nothing is stuck after Win+L: the reset clears gestures, cancels macros and resyncs keys (the request path tested without a session change). **Declared trigger change (R2, Question 2):** the reset answers `WTS_SESSION_LOCK` and the switch away, not `WTS_SESSION_UNLOCK`. By the unlock, everything from before the loss is gone and everything since is new. | `hook::main_thread::tests::a_lock_signal_drops_gestures_cancels_macros_and_releases_a_swallowed_key` (C20 → C21); `hook::control_window::tests::the_session_messages_note_the_lock_and_the_unlock`; `hook::keyboard_hook_callback::tests::a_desktop_switch_is_classified_by_where_input_is`; `engine::scenarios::nothing_is_stuck_after_win_l` (red in C14 on today's reset, green from C15; M2 mutation); property `no_key_up_reaches_windows_alone_…`; live: manual M1, M2, M5 |
| After Win+L with a swallowed, non-repeating trigger key held, the next press fires | `engine::scenarios::after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires` (pin from C14, D1 semantics from C15); the F14 part of the C20 request-path test; manual M1 |
| (spec) the passed case | `engine::scenarios::after_a_lock_the_next_press_of_a_passing_key_is_a_new_press` (red in C14, green from C15); manual M2 |
| 100 presses of a conditional binding in one window cause one process-name query | `hook::keyboard_hook_callback::tests::a_hundred_conditional_presses_in_one_window_query_its_process_once` (C18 → C19) |
| The lone key-up rule after a reset is pinned by a test | `engine::scenarios::a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed`, `…_released_after_remapping_on_keeps_its_key_up_swallowed`, property `no_key_up_reaches_windows_alone_…` (C14 → C15); manual M11 |
| Latency bench: no unjustified regression; the cache shows a lower cost; a bench row | `bench::latency_bench` R1-R13, rule D14 (tester 4b); `results.md` per the latency ruling |
| (R1) No stuck key when the hook misses a press of a swallowed key | `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_is_released_in_windows` (red in C12), the critic's script and the remapping-off variant (pins C12), the lock variant (pin C14), `only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key`, property `nothing_is_left_down_in_windows_when_the_hook_misses_key_downs`; manual M12, M13 |
| (R2) A reset never discards state created after input came back | `a_late_unlock_notification_keeps_a_press_made_after_the_unlock`, `a_late_return_notification_keeps_a_press_made_after_the_return_in_either_order`, `a_hold_armed_between_two_return_notifications_fires`, `a_second_lock_releases_a_key_pressed_after_the_first` (C14), `a_late_return_signal_keeps_a_running_macro_and_an_armed_hold` (C20); manual M1 (timestamped lines) |
| (spec) the control window is not `HWND_MESSAGE`; WTS, `WM_QUERYENDSESSION` and `WM_ENDSESSION` go through it; Drop guards; the feature is named | `hook::control_window::tests::*`; `Cargo.toml`; manual M6, M9 |
| (spec) `EVENT_SYSTEM_DESKTOPSWITCH` is hooked like the foreground hook | `a_desktop_switch_notes_a_signal_only_while_remapping_is_on`; review of `install`/`uninstall`; manual M5 |
| (spec) the title stays current inside one window | `…::a_title_condition_reads_the_cached_windows_title_on_each_key_down`; manual M4 |
| (spec) m2, OQ3, P1, OQ2, the executor doc, the watcher spawn | C5/C6, C3/C4, C2, C7/C8, C1, C9/C10 |

## Manual checks (human only)

These use the release daemon, elevated, unless stated. `daemon.log` is `%LOCALAPPDATA%\key-switch-rs\daemon.log`, and its lines are timestamped.

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | Default config, two layouts | Hold CapsLock, press Win+L, release everything on the lock screen, unlock, press CapsLock once | The layout switches once. `daemon.log` has one timestamped line per signal: `session locked`, `keyboard input switched to another desktop` (if the lock screen is a desktop of its own), exactly one `state reset: keyboard input left this desktop` before the unlock, then `keyboard input switched back to this desktop` and/or `session unlocked`. Paste the lines and their timestamps into `impl.md` (ruling R2). | none |
| M2 | Add `( keys: "F9", action: SwitchLanguage, block_default: false )` | Hold F9, Win+L, release on the lock screen, unlock, press F9; then hold Shift, Win+L, release Shift on the lock screen, unlock, press F9 | The layout switches each time, and F9 is not read as Shift+F9 (the modifier resync reads Shift as up after the return) | none |
| M3 | `( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") )` | In Notepad press F9; Alt+Tab to another app and press F9 at once; back to Notepad, F9 | `a` in Notepad; F9 reaches the other app; `a` again | none |
| M4 | `( keys: "F9", action: PressKey("A"), when: TitleContains("GitHub") )`; a browser with a GitHub tab and another tab | Switch tabs with Ctrl+Tab and press F9 in each | `a` only in the GitHub tab, without changing windows | none |
| M5 | Default config | Start an elevated program (UAC prompt) and cancel it; press Ctrl+Alt+Del, then Esc; type in Notepad | Each time, `daemon.log` shows `keyboard input switched to another desktop` with `state reset: …`, then `keyboard input switched back to this desktop`. Typing and CapsLock work. | none |
| M6 | Daemon running | Sign out, sign in, read `daemon.log` | It ends with `[INFO] the session is ending; removing the keyboard hook and exiting` before the next start | none |
| M7 | `( keys: "F9", action: Launch(exe: "notepad.exe"), trigger: Hold(3000) )` in an editor's `config.ron` | Hold F9, press Ctrl+S in the editor (the save loads), release F9 after 4 s | No Notepad starts; `config reloaded` is in the log | none |
| M8 | `( keys: "Win+Alt+T", action: Launch(exe: "wt.exe") )` | Hold Win+Alt+T until the terminal opens, keep T held 1 s, release | No `t` in the terminal; no stuck key (`osk.exe`) | none |
| M9 | Daemon running | Alt+Tab, taskbar | No daemon window other than its console | none |
| M10 | Notepad focused | `cargo test --locked -- --ignored --exact platform::foreground::tests::the_foreground_cache_names_the_focused_notepad`, focus Notepad within 5 s | passes | `platform::foreground::tests::the_foreground_cache_names_the_focused_notepad` |
| M11 | `( keys: "PrintScreen", action: DoNothing )` | Hold PrintScreen, Ctrl+Alt+Del, Esc, release PrintScreen | No screenshot, no Snipping Tool | none |
| M12 | **Non-elevated debug daemon** (`cargo run`, run by the human) with the default config; Notepad and an elevated Task Manager | Hold CapsLock in Notepad; click into Task Manager; release CapsLock; press CapsLock again and keep it held; click back into Notepad; release | CapsLock is not stuck: `osk.exe` shows it up, typing is normal, the next CapsLock press switches the layout once (D15 through UIPI) | none |
| M13 | Release daemon, default config | Hold CapsLock; Ctrl+Alt+Del; on that screen release CapsLock and press it again, keep it held; Esc; release CapsLock on the desktop | CapsLock is not stuck (`osk.exe`); the next press switches the layout once (D15 across the secure desktop) | none |
| M14 | At the unlocked desktop | `cargo test --locked -- --ignored --exact platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it` | passes (`Ok(true)`) | `platform::desktop::tests::input_is_on_this_desktop_while_a_human_uses_it` |

## Risks and edge cases

- **R1 hook callback and engine.**
  - Under the borrow, the callback may now read a title (other processes only) and, at a key-up it would swallow, make one `GetAsyncKeyState` (D15). It never queries a process, and it adds no allocation (`AppEquals` loses one).
  - `code < 0`, the off gate, sentinel-first handling and the one-outcome rule are untouched.
  - Re-entry: a `WM_WTSSESSION_CHANGE` or a WinEvent dispatched inside another message call only notes a signal (D7).
  - `foreground_changed` under a refused borrow updates nothing. With the cache, that would leave the cache stale until the next focus change, `swch on` or return signal, where master's live lookup lost nothing. This is unreachable: the borrow is refused only on re-entry, and nothing under the borrow pumps messages or injects input (`keyboard_hook_callback.rs:62-86`; critic OQ3).
- **R2 sentinel.** No new `SendInput`. D15 cannot be fooled by the daemon's own output (SendInput is not interspersed).
- **R3 shared state.** `desktop_hook` is atomic. `PENDING_SIGNALS` and `MainState.foreground` are main-thread locals. No new lock. `install` and `uninstall` stay on the main thread; `WM_ENDSESSION` calls `uninstall` there, and the later explicit `uninstall` is an idempotent swap (`keyboard_hook.rs:196-220`).
- **R4 orderings.** `desktop_hook`: `store(Release)`, `swap(AcqRel)`. Nothing is `Relaxed` or `SeqCst`.
- **R5 builders.** No setters. `ForegroundCache::with_query` is test-only.
- **R6 Display.** `ResetReason` has three arms and `InputSignal` four, each added with its variant. `Request::Reload` is private and needs none.
- **R7 specificity.** The reload path still publishes through `update_bindings`.
- **R8 key identity.** Own keys use `KeySpec::accepts`. The resync and `release_all` work by raw code. D15 reads the sided code that the hook reported, and `GetAsyncKeyState` distinguishes sides (`VK_LSHIFT` … `VK_RMENU`). Two physical keys sharing one VK (numpad Enter and Enter) share a record, as today.
- **R9 process reads.** The same rights and calls as `condition.rs:189-223`; reads only.
- **R10 loop and shutdown.** The window is created after the executor guard and before `install`, and destroyed right after the loop, before `uninstall` and the executor stop (M4). `WM_ENDSESSION` removes the hook before its flush of at most 1 s. A late watcher reload gets `ShuttingDown` after `drop(main)`.
- **Repeats.** A key held through a whole lock episode and still repeating reads one repeat as a press (D3, row 76). A reload keeps held keys' records, so repeats stay repeats. A swallowed key pressed again unseen keeps its repeats swallowed (row 78).
- **Sided modifiers.** Matching is unchanged. `resync_from_os` writes generic codes only from the OS. After `release_all`, the next event's `sync_modifiers` re-reads every passed modifier from the OS (`keys.rs:102-110`), so a modifier really held after the return is held again (M2).
- **Reload mid-gesture.** Dropped by design (row 77). The key-up of a swallowed press stays swallowed unless Windows has the key down.
- **`swch off/on` during a gesture.** The drop is unchanged. Signals noted while off are dropped. A swallowed key released while off still gives a lone key-up (row 59). D15 covers a key that was pressed again while off.
- **Late or repeated input signals.** Return signals never reset, so a late return is harmless. A second "left" signal of one episode is idempotent. A "left" signal noted after the return reads as a return (row 79). A lock noted after the unlock needs a stall of the main thread for the whole lock.
- **Foreground race.** D9, row 80; F3.
- **UIPI.** `TaskbarCreated` from a medium-integrity Explorer to the elevated window will need `ChangeWindowMessageFilterEx` in 4.6 (F2). D15 relies on the key state at a key-up the hook sees, M12.
- **IME/TSF.** A default IME window exists per thread. Its work runs only inside this thread's message calls. R13 bounds the effect on the loop.
- **Tests that touch real input state.** `HookEnv` reads the machine's keyboard, so adapter tests that decide a swallowed key-up use F13 (absent on most keyboards). A held F13 would make D15 pass the key-up, and such a test would fail visibly, not silently. Only the control-window test creates a top-level window, briefly, under a test mutex. The proc tests call the procedure directly with no window. Every other window in the tests stays message-only.

**Follow-ups (not in scope):**
- **F1.** `KeyState::mark_completed` (`keys.rs:173-180`) marks a held key with no record as swallowed. D15 now saves its key-up when Windows has it down, but its repeats stay swallowed. Goes to 3.2.
- **F2.** In 4.6: `ChangeWindowMessageFilterEx(hwnd, RegisterWindowMessageW("TaskbarCreated"), MSGFLT_ALLOW, None)` (https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-changewindowmessagefilterex).
- **F3.** In 3.5: measure the foreground race. `TargetWindow::for_action` (`executor/mod.rs:81-98`) could take the cached HWND.
- **F4.** `WTS_CONSOLE_DISCONNECT` and `WTS_REMOTE_DISCONNECT` as input-left signals (fast user switching, RDP): goes to 4.1.
- **F5.** If M1 or M5 shows that the switch away is not reported, a lock is still answered by `WTS_SESSION_LOCK`. Ctrl+Alt+Del and UAC would then be answered by nothing (row 79 grows). The plan is a second look in 2.4, whose Raw Input sink on this window sees input on the desktop it belongs to.
- **F6.** `WTSRegisterSessionNotification` retry at autostart: goes to 4.5 (critic OQ5).

## Questions for the orchestrator

1. **A fourth `windows` feature, `Win32_System_StationsAndDesktops`, for `GetThreadDesktop` and `GetUserObjectInformationW(UOI_IO)` (D16).** Recommendation: accept, on the same grounds as Q6. It is a feature of an existing dependency, and `Cargo.lock` is unchanged. It is the only documented way to ask whether this desktop has input without opening the Winlogon desktop, which the daemon cannot open.
2. **The acceptance wording "the reset on WTS_SESSION_UNLOCK clears gestures, cancels macros and resyncs keys".** Under R2 the reset answers `WTS_SESSION_LOCK` and the switch away, and releases keys instead of reading them from the OS. The unlock and the switch back only refresh the foreground cache. Recommendation: accept this as the mechanism R2 asks the architect to choose. The intent of the acceptance line, nothing stuck after Win+L with the request path tested without a session change, is met by C20. A reset at the unlock is exactly what I2 shows to be unsafe. The CHANGELOG (C21) and the Phase-2 roadmap sync state it.
3. **Row 76 widens from swallowed keys to every key.** A passed key held through a whole lock episode and still repeating reads one repeat as a press. Recommendation: accept. The only alternative is an OS read at the reset, which brings KS-29 back for a lock noted before the switch (C14's `nothing_is_stuck_after_win_l`). Holding a key through password entry is rare, and D15 keeps its release from sticking.

The round-1 questions are ruled in `rulings.md`: Q1, Q2 and Q4-Q8 are accepted, and Q3 is superseded by R1.

## Readiness checklist

- [x] KS-06 (status), KS-29 and KS-31 are read. Their scenarios are tests (C12, C14, C17, C20) and manual checks (M1, M2, M7, M8, M11, M12, M13).
- [x] Book references are re-located (table above), and removed files are named.
- [x] Goal, non-goals and dependent items are stated.
- [x] Each decision has a rationale, rejected alternatives and a cost.
- [x] Owner decisions:
  - exact modifiers untouched;
  - repeat defaults untouched;
  - RON unchanged;
  - `log` + sink only (new lines through `log::`, none in `keyboard_hook_callback.rs`);
  - licence n/a;
  - open question 9 implemented (D3);
  - open question 10 refined only for own keys (D10, with M1 declared).
- [x] The thread is named for every new path (Threads table). The LL callback gains only D15's read on a swallowed key-up.
- [x] Install, uninstall and timers are on the main thread: both WinEvent hooks are in `install`/`uninstall`, and the window is on the main thread.
- [x] N/A: new `SendInput` paths. There are none, and D15 relies on the existing six being atomic.
- [x] Sided and unsided keys: own keys through `KeySpec::accepts`; the resets and D15 by raw code.
- [x] Reload, `swch off/on`, foreground change, lock, desktop switches (late and repeated), shutdown and session end during a pending gesture: D1, D4, D5, D7, D8, Risks.
- [x] Orderings are stated; no `static mut`.
- [x] No new module cycle: `platform::{foreground, desktop}` import nothing from the crate, and `engine` gains no `hook`/`system` import. The decisions sit in `engine` (`InputSignal::reset`, D15) and are reachable by scenarios.
- [x] Win32 table: paths, features checked against `Cargo.toml:41-53` and `windows-0.62.2`, threads, errors, links. Unverified items are marked with their manual checks (M1, M5, M12, M13, M14).
- [x] Elevation and UIPI are considered (D15, D16, Risks, F2).
- [x] Red-first with predicted reds. The pins name their mutations, including M2's.
- [x] Tests run through `Engine`, `Env`, the fake clock, `decide_event` and direct procedure calls. Nothing installs a hook or injects input, and one hidden window appears in one test.
- [x] Manual tests: two are ignored tests (M10, M14); the others need the live daemon and are listed.
- [x] Each commit is gate-green except the declared reds, and docs go with the commit that changes the meaning.
- [x] Every acceptance line is mapped, the changed trigger declared (Question 2).
- [x] Every question has a recommendation.

## Critique response

Round 1 (`critique.md`). Each remark is answered here: what was fixed, where, and what depends on it. The removed text is quoted from round 1.

**I1. Rule L swallows the key-up of a key-down that Windows saw and the hook did not.** Fixed per ruling R1 (option b).
- **D2, Why.** Removed: "In the current engine a swallowed record is consulted only by its own key's repeats and key-up. A lost key-up leaves `active` set (`keys.rs:146-153`), so the next press reads as a repeat whatever the record says; forgetting therefore never prevents a stuck key and only turns a key-up that does arrive into a lone key-up (KS-06 status)." Added: the corrected premise naming the I1 case, and D15 as what covers it in every daemon.
- **Question 3 (round 1)** is withdrawn. Removed: "Recommendation: accept (D2): forgetting never prevents a stuck key in the current engine; dropping it everywhere avoids a token query and a second behaviour."
- **New D15** (the witness, with the Learn quote, the six atomic `SendInput` sites and the M12/M13 unverified items) and **D5** (rule L plus the witness).
- **Tests.** C12 (a) is red on master, which shows the defect is older than D2. C12 (b) is the critic's script verbatim, with `check_nothing_stuck`. C12 (c) and the C14 lock variant cover a reset that did not see the key go down again. C12 (e) adds the property mode with unseen presses. Known-issues row 78 is new.
- **Depends on it:** D1 (the `Suppressed kept` rows rely on D15), D3, the C15 migrations, F1.

**I2. A second or late lost-input reset wipes state built after input came back.** Fixed per ruling R2.
- **D7, Why.** Removed: "the cell coalesces the unlock notification and the desktop switch of one unlock into one reset." Added: only a signal that input left resets; a return signal resets nothing, however late; a new episode starts with input leaving again; nothing relies on coalescing or order. The mechanism is classified by `UOI_IO` (new D16, Learn-quoted, NVDA as corroborating evidence).
- **D1.** `InputLeft` uses `release_all` without an OS read, which makes repeated resets idempotent. The removed round-1 table row now applies to `RemappingSwitched` only.
- **The episode flag and timestamps are rejected with reasons.** The flag fails the ruling's own first pin.
- **Pins.** All of the ruling's pins are in C14 (engine), and the macro and Hold pins also in C20 (adapter). The critic's exact script `session_unlocked; down F13; session_unlocked; repeat F13` is part (1) of `a_late_unlock_notification_keeps_a_press_made_after_the_unlock`.
- **M1** now asks for one timestamped line per signal in `impl.md`. Removed: "`daemon.log` has `[INFO] state reset: session unlocked` and/or `… input desktop switched`".
- **Depends on it:** D3, D16, Questions 1-3, rows 63, 76 and 79, C20/C21.

**M1. D10 makes a waiting Tap fire while an own modifier is down in Windows.** Declared, not fixed (ruling).
- **D10 Cost.** Kept: "The second press completes only in the order that arms the group under the same key …". Added: the Ctrl+Esc residual.
- Row 68 and §11.4 grow in C8. C7 (3) pins `expect_down_in_windows("LCtrl")` at the Tap's fire and is red on master, because master fires the Tap at the LCtrl key-down.
- Item 3.1 owns the output modifier state.

**M2. `nothing_is_stuck_after_win_l` passes with or without the reset.** Fixed.
- **C14.** Removed: "down LCtrl, down LAlt → nothing fired; down L → fired; `check_nothing_stuck`". Added: `down L` asserts `expect_suppressed()`. With the new `session_locked` placed before the `up_unseen` steps, the test is red even on today's reset: `down L at +0 ms: expected the event to be suppressed, it was passed`. That is new evidence for D1's no-OS-read rule.
- The mutation for `impl.md` is named: `SessionLocked.reset()` returns `None`.

**M3. `refresh_foreground()` runs before the foreground hook exists.** Fixed.
- **D6 and D9.** Removed: "right after `MainThread::new()`, `let _control = ControlWindow::create()…?;` (fatal, before any hook), then `main.refresh_foreground()` (D9)." Added: `refresh_foreground()` right after `hook.install()?`. Any later window change is a queued foreground event, so no change can fall between the refresh and the hook.
- Done in C19.

**M4. The control window outlives the pump at exit.** Fixed.
- **D6.** Removed: "Declared before `AttachedExecutor::attach` (`app.rs:207`), so on every return it drops after the `AttachedExecutor` guard (executor stopped) and after the explicit `uninstall` (`:224`, `:277`), on the main thread." Added: the window is created after the guard and dropped explicitly as the first statement after `main.run` returns. On early returns it drops before the guard. The SendMessage quote explains why.
- **Depends on it:** R10, Threads, C21.

**M5. The control-window test asserts a registration that production tolerates failing.** Fixed.
- **D6 and C20.** Removed: "`has_session_notifications()`". Added: `session_notifications() -> SessionNotifications { Registered, Unavailable }`. The test accepts both, names them, and checks the warn line for `Unavailable`.
- The same principle is applied to D16: the real `UOI_IO` query is tested only as an ignored manual test (M14), and the classification is tested as a pure function.

**M6. The window procedure's session wiring has no test.** Fixed.
- **C20.** `the_session_messages_note_the_lock_and_the_unlock` (with the `is_enabled` gate) and `the_session_end_messages_are_answered_and_end_the_loop` call `control_window_proc` directly with `WM_WTSSESSION_CHANGE`, `WM_QUERYENDSESSION` and `WM_ENDSESSION`. The latter asserts that `WM_QUIT` is queued for `TRUE` and not for `FALSE`.
- No session change or injection is involved. `GLOBAL_HOOK` moves to module level so these tests share it.

**M7. Doc lines the change makes false.** Fixed: `gestures.rs:25-26` (C17), `mod.rs:31-38` and `:590-594` (C19) are in the touch set.

**M8. The foreground-race rejection understates the residual.** Fixed.
- **D9, Rejected.** Removed: "`GetForegroundWindow` compared on every evaluation: one more call per conditional key-down for a race of microseconds (Risks)." Added: the exact residual (a programmatic activation followed by an injected key, the AutoHotkey example and its effect on `when:`).
- New known-issues row 80 and §12.4 (C19). F3 measures it in 3.5.

**Open question 1 (IME/TSF).**
- Answered with "The operating system creates a default IME window for every thread" (ImmGetDefaultIMEWnd). That window, like every window of the thread, runs only inside this thread's message calls: "Messages sent between threads are processed only when the receiving thread executes message retrieval code" (SendMessage).
- The control window is never shown or activated, so it takes no input context.
- Measured by the new bench row R13 (R7 with the control window on the loop thread, bound in D14). `ImmDisableIME` is not adopted: it needs a new feature for a need not shown.

**Open question 2 (no switch back).**
- Under R2 the switch back no longer carries any reset. Only the switch away does, and NVDA's handler is evidence that it is delivered.
- If M1 or M5 shows it is not, `WTS_SESSION_LOCK` still answers Win+L, Ctrl+Alt+Del and UAC are left unanswered (row 79), and F5 names the plan.

**Open question 3 (a refused foreground update has a lasting cost).** Fixed in Risks R1. Removed: "`foreground_changed` under a refused borrow updates nothing (as today)." Added: the lasting cost, and why it is unreachable (the refusal happens only on re-entry, and nothing under the borrow pumps or injects).

**Open question 4 (a late reload logs nothing).** Adopted. In D4, `serve` logs `config reload carried out late: N bindings live` when the reply cannot be delivered, and the C16 test pins it. An errors.md row is added.

**Open question 5 (retry of the session registration at autostart).** Left to 4.5 (F6). Without it, a lock is still answered by the desktop switch away (`session_notifications() == Unavailable` is logged).

**Strengths and absent topics.** Nothing to answer. The strengths are kept: D4 is unchanged, D1's no-resync-on-reload is kept, D6's guard order is kept, and the reset still never runs in a callback.

## Developer addendum

Branch `feat/control-window`, from `master` `c3dab6a`, 22 commits (`0ca09d7` .. `6f6edb3`); the cut's C1..C21 in order plus one comment-only follow-up (C22, `6f6edb3`). Full report: `D:/tmp/ks-orch/p2-2.3-control-window/impl.md`.

### Rulings of round 1 (applied by the revised cut, implemented)

| Ruling | How | Where |
|---|---|---|
| R1 witness | `decision.suppress = swallow && !env.os_key_down(vk)` | `src/engine/mod.rs:885` (C13 `7eabce9`); red in C12 `0ef6a99` |
| R1 red-first script | the critic's script verbatim: `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows` (pin, green on master, mutation: remove the witness after C15); the lock variant `a_swallowed_key_pressed_again_on_the_lock_screen_is_released_in_windows` | `src/engine/scenarios.rs` |
| R1 property mode | `Mode::Unseen` (C12), `Mode::Unlock` (C14) | `src/engine/properties.rs` |
| R2 | only `InputSignal::{SessionLocked, DesktopSwitchedAway}` reset (`InputLeft` = `release_all`, no OS read); return signals reset nothing; all ruling pins in C14 and C20 | `src/engine/mod.rs` (`InputSignal::reset`), `src/hook/main_thread.rs:710` (`answer_input_signals`) |
| M1 | declared: C7 part (3) `a_waiting_tap_of_a_passing_ctrl_shift_double_tap_fires_with_ctrl_down_in_windows`, row 68, §11.4 | C7/C8 |
| M2 | `nothing_is_stuck_after_win_l` asserts `expect_suppressed`; mutation run: `SessionLocked.reset()` -> `None` gives `scenario: down L at +0 ms: expected the event to be suppressed, it was passed` | C14/C15 |
| M3 | `main.refresh_foreground()` right after `hook.install()?` | `src/core/app.rs:224` |
| M4 + V2-m4 | window created after the `AttachedExecutor` guard, dropped after `hook.uninstall()` and before `executor.stop()` | `src/core/app.rs:215`, `:296` |
| M5 | test accepts `Registered` or `Unavailable` (+ the warn line) | `hook::control_window::tests::the_control_window_is_a_hidden_top_level_window_released_on_drop` |
| M6 | procedure called directly with `WM_WTSSESSION_CHANGE` (lock, unlock, logon, remapping off), `WM_QUERYENDSESSION`, `WM_ENDSESSION(0/1)` | `hook::control_window::tests` |
| M7 | `gestures.rs` reload line, `mod.rs` `ForegroundContext` lines rewritten | C17, C19 |
| M8 | row 80, §12.4 | C19 |
| Q1-Q8 | as accepted; Q6: four features, `Cargo.lock` unchanged (`cargo check --locked` passes) | C21 |

### Round 2 (rulings.md)

- **V2-m1** M1 passes on the invariant: every `state reset` line comes before the first return-signal line. Lines of one drain share a time and come in `InputSignal::ALL` order (lock, unlock, switch away, switch back), then the unknown-switch warn, then `state reset`. M1 cannot be run by an agent (it needs the live daemon and a real lock): the Win+L and Ctrl+Alt+Del timestamps are left to the owner as manual check M1/M5 (impl.md, Manual checks).
- **V2-m2** the lock is still taken by its kind. D7's sentence "The other order, a lock noted after the unlock, needs the main thread stalled through the whole lock" is corrected: the lock notification comes from the session service on its own channel, and its latency is unverified; a lock delivered after a very short unlock (Windows Hello) resets after the return, so the next repeat of a held, swallowed, non-repeating trigger fires once more. Declared in known-issues row 79 (`book/src/config/known-issues.md`). If M1's timestamps show a lock after the return, report it; a later lane decides.
- **V2-m3** row 63, row 79 and the CHANGELOG `Added` entry say that a UAC prompt counts as input leaving only while "Switch to the secure desktop when prompting for elevation" is on (the default); M5's precondition is stated in impl.md.
- **V2-m4** see M4 above.
- **Critic OQ1** `the_control_window_is_a_hidden_top_level_window_released_on_drop` asserts `EnumThreadWindows` finds 0 windows before the create and 0 after the drop (the default IME window included). Green on this machine.
- **Critic OQ2** evidence: Microsoft Learn's KBDLLHOOKSTRUCT documents `vkCode` only as 1..=254 and says nothing about sides; the daemon's own adapter passes the raw code (`key_event`); a real keyboard reports sided codes (harness rule). AutoHotkey's hook translates a neutral `VK_SHIFT`/`VK_CONTROL`/`VK_MENU` that `keybd_event`/`SendInput` callers inject (from its source, recalled, not re-read here), so the hook is taken as able to report a generic code for another program's injected input. The witness reads the exact code of the event: a generic code reads the generic (either-side) state, because Windows applies the injected key to a side the event does not name (comment at `src/engine/mod.rs:874-885`). Pinned by `engine::tests::the_witness_of_a_generic_shift_key_up_reads_the_generic_state` (an engine test with its own Env: the harness presses only physical sided keys) and, for the rule "never let a generic read pass a sided key-up", by `engine::scenarios::a_swallowed_lshift_key_up_stays_swallowed_while_the_right_shift_is_down_in_windows`. The residual (an injected generic release passes while either side is down in Windows) is in row 78.
- **Critic OQ3** `hook::control_window::CONTROL_CLASS` (test-only mutex) is taken by the window test and by bench row R13.
- **Architect Q1** fourth feature `Win32_System_StationsAndDesktops` added (`Cargo.toml`). **Q2** the roadmap wording is listed for the Phase-2 book sync (impl.md). **Q3** row 76 covers every key held through a lock.

### Deviations from the cut

1. `ControlWindow::session_notifications()` and `SessionNotifications` are `#[cfg(test)]`, and the guard fields are `_session`, `_window`, `_class`: in the daemon nothing reads them (the warn of a failed registration is logged by `create`), and `-D warnings` rejects dead code. The test still names both outcomes.
2. The window drop sits after `hook.uninstall()` (ruling V2-m4), not as the first statement after `main.run` (cut D6 step 3).
3. C14: three new or flipped scenarios carry `check_no_lone_key_up`, so their red is `Windows got a lone key-up of F13` rather than the predicted `expected the event to be suppressed, it was passed`; the defect is the same.
4. C7 part (2)'s master red (`expected to fire [], fired [SwitchLanguage]`) is traced, not observed: part (1) of the same test fails first. Part (3) (M1) is red as predicted.
5. C5 (c): J goes down before F14, because a first key-down fires a Tap that waits for its window (2.2 rule), which would have taken the window timer out of the count.
6. C3: Hold terms 10000/5000 ms instead of 100/50, so the plan never meets a due timer on a slow machine.
7. `MainThreadHandle::reload` repeats `set_enabled`'s send-wake-wait body instead of extracting a helper (no extraction beyond the cut).
8. The drain step is the function `answer_input_signals`; `PendingSignals` gets the accessors `signals()` and `unknown()`; `note_input_signal` finds the flag with `position` and returns without a panic path (it runs in a window procedure).
9. `ForegroundCache` gets `impl Default` (clippy `new_without_default`).
10. C12 uses the template's two bindings (`DEFAULT_PAIR`) for the CapsLock scenarios.
11. C22 (`6f6edb3`, comment-only): the adapter's Threads section, `KeyboardHook::install`, the main-thread module docs and README name the third hook; a rustdoc warning (public docs linking the private `ApplyReload`) is removed.
