# Code review: p2-2.3-control-window @ 6f6edb3
CHANGES_REQUESTED (Critical 0, Important 1, Minor 1)

Scope: `git diff master...feat/control-window` at the pack's HEAD `6f6edb3` (22 developer commits), read through `git show`/`git diff`. The branch tip has since moved to `4539306`, with three tester commits (`c7ab5c7`, `7ab78ab`, `4539306`). They are test-only (`src/engine/scenarios.rs`, `src/hook/main_thread.rs` tests), carry the Sonnet trailer, and install no hook and inject nothing. I read them but did not review them as lane code.

## Gate: fmt ✅ · clippy ✅
- `6f6edb3`, exported with `git archive` into a scratch tree: `cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -- -D warnings` exit 0 (re-run after touching `src/lib.rs`: `Checking key-switch-rs ... Finished`, exit 0).
- `4539306` (tip, same method): fmt exit 0, clippy exit 0.
- `cargo test` was not run (the tester owns it).

## Remarks

### 🔴 Critical
None.

### 🟡 Important

#### I1. The pin of the watcher-spawn fix scans only the first 36 lines of the file, so it can no longer fail (spec leftover "config_watcher.rs:126 .expect becomes an error path", KS-33-like)
**Where**: `src/hook/config_watcher.rs:448-463` (the test `the_watcher_thread_has_no_panic_path`), `:37`, `:158`  **Confidence**: CONFIRMED (path traced; an `awk` replay of the `take_while` on `git show 6f6edb3:src/hook/config_watcher.rs` prints "scan stops before line 37")

**Problem**: The test keeps only the lines before the first line whose trimmed text is `#[cfg(test)]`:
```rust
.take_while(|(_, line)| line.trim() != "#[cfg(test)]")
```
At C9/C10 (`5388175`/`5bf7f12`) that first line was the `mod tests` attribute at `:175`, so the test was red, then green, as `impl.md` records. C17 (`4fbcc37`) added a test-only import near the top of the file:
```rust
37: #[cfg(test)]
38: use crate::hook::keyboard_hook::KeyboardHook;
```
From then on the scan stops at line 37. `spawn_watcher` (`:126-162`, with `.map_err(notify::Error::io)?` at `:158`), `debounce_loop` and `reload_into` are never scanned. The CHANGELOG's Fixed entry and the cut name this test as the proof of the fix, and at HEAD it proves nothing.

**Failure**: Mutation: put `.expect("spawn config-watcher thread")` back in place of `.map_err(notify::Error::io)?` at `:158`. Or add `.unwrap()` anywhere in `reload_into`. `the_watcher_thread_has_no_panic_path` stays green. The KS-33-like panic on a failed `config-watcher` spawn can come back with no test failing.

**What to do**: End the non-test region at the `#[cfg(test)]` + `mod tests {` pair, as `non_test_lines` in `keyboard_hook_callback.rs:1254-1261` and the scans in `logging/mod.rs:276,356` already do. Also assert that the scanned region contains `fn spawn_watcher(`, so that a scan which stops early fails visibly. Then replay the mutation above to show the test is red.

### 🟢 Minor

#### m1. Loop docs that C17/C21 made stale: the drain and pump steps do not name reloads or input signals, and the adapter still says "live Win32 calls"
**Where**: `src/hook/main_thread.rs:28-30`, `:36-37`, `:664-665`; `src/hook/keyboard_hook_callback.rs:16-19`  **Confidence**: CONFIRMED (text read at `6f6edb3`)

**Problem**:
- Module doc step 1 says "Drain, until a whole pass finds nothing: the requests (`swch on` and `off`), then every gesture timer whose due time has passed". Step 4 says "a `MAIN_WAKE` comes (a request waits: back to 1)".
- `drain`'s doc says "serves the requests and carries out the due timers until a whole pass finds neither".
- None of these names the `Reload` request or the input-signal step (`take_pending_signals` → `answer_input_signals`, `:673-676`), although the drain now runs them, and a noted signal also posts `MAIN_WAKE`.
- The adapter's step 5 still says that `HookEnv` "answers the engine's questions with live Win32 calls". The executable name now comes from the cache, with no call.

**Failure**: No runtime failure. A reader who checks the step order the cut and R2 rely on (requests, then signals, then timers) finds the signal step missing from the documented loop.

**What to do**: Name the reload request and the signal step in steps 1 and 4 and in `drain`'s doc. Say in the adapter's step 5 that the executable name comes from the cache and the title is read live.

## Checklist walked (items absent with no reachable failure, one line each)
- **R1 hook callback.** `code < 0`, the off gate, the sentinel first and the single outcome are unchanged (`keyboard_hook_callback.rs:339-383`). New work on the hook path:
  - the witness `GetAsyncKeyState`, only when `swallow` is true (short-circuit `&&`, `engine/mod.rs:885`);
  - the cached `app()` (no call, no allocation);
  - the title read, which allocates one `String` as master did and is skipped for windows of this process.
  - No `log::`, no lock, no new allocation, no panic path (`window_title` clamps `len.min(TITLE_CAPACITY)`; `try_from(..).ok()`).
- **Witness semantics.** LowLevelKeyboardProc documents that the asynchronous state is updated after the callback. A false "up" swallows the key-up, as master did. A true "down" means Windows holds a key-down, so the key-up is needed. The witness never creates a stuck key. The generic and sided reads are pinned (`engine/mod.rs:1279`, `scenarios.rs:2041`). No chord or sequence replay exists that could feed the async state (`chords.rs:17`, `sequences.rs:23`).
- **Suppression invariant.** `release_all` and `resync_from_os` keep `Suppressed` records (`keys.rs:126-150`), and only the key's own next key-down or key-up replaces one. I found no path that swallows a key-up whose key-down passed (a `Passed` record always passes, `mod.rs:870-873`).
- **R2 sentinel.** No new `SendInput` site and no new constant.
- **R3 shared state.** `PENDING_SIGNALS` is a const thread-local `Cell` with no destructor; its `try_with` cannot panic. `MainState.foreground` is main-thread state. No `static mut` and no new `Mutex`. Under `with_main`, only the engine decides, `PostThreadMessageW` runs (`cancel_actions`, the existing pattern) and the process/title reads run; nothing pumps or injects.
- **R4.** `desktop_hook` is `store(Release)` and `swap(null, AcqRel)`. No `Relaxed` and no `SeqCst` were added.
- **R5.** No setters. `ForegroundCache::with_query` is test-only.
- **R6.** `ResetReason` (3 arms) and `InputSignal` (4 arms) have explicit `Display`. No `_ =>` arm hides a variant.
- **R7.** Reloads still publish only through `update_bindings` (sort, store, `cancel_actions`), now on the main thread inside `MainState::reload`. The engine is reset in the same borrow.
- **R8.** The witness reads the event's raw code, and the resyncs work by raw code. Own-combo keys use `KeySpec::accepts` (`gestures.rs:318-322`).
- **R9.** `process_image_name` moved unchanged: `Owned` guard, `PROCESS_QUERY_LIMITED_INFORMATION`, size in UTF-16 units, clamped. `GetUserObjectInformationW` is passed `size_of::<BOOL>()` bytes into a local `BOOL`. `GetThreadDesktop`'s handle is not closed, as documented. No registry access.
- **R10 shutdown and lifetimes.**
  - Startup order: control window after the executor guard, then `install`, then `refresh_foreground` (M3).
  - Normal exit: `uninstall` before `drop(control_window)`, before `executor.stop()` (V2-m4, M4).
  - Every early return (`install`, `serve`, `uninstall()?`) drops the window before the `AttachedExecutor` guard, by declaration order.
  - `WM_ENDSESSION` runs `uninstall` on the main thread outside `with_main`, then the ≤1 s flush, then `PostQuitMessage`. The later `uninstall` is an idempotent swap.
  - Drop guards release in the documented order (fields `_session`, `_window`, `_class`).
- **Threads, timers and COM.** There is no new production thread, timer or COM apartment. Both WinEvent hooks and the window belong to the main thread (KS-01).
- **Lost wakeups.** Each note sets its flag and then posts `MAIN_WAKE`. A failed post is covered because `drain` runs after every `pump`. A signal noted during a drain pass marks that pass busy, so another pass follows. No busy loop.
- **Cross-thread reload.** It uses the 2.2 request pattern (`try_send`, wake, 2 s `recv_timeout`). At shutdown, dropping the receiver yields `ShuttingDown`.
- **Key names and text in logs.** The new lines carry signal names, `HRESULT`s, counts and reasons only. None carries a key name or typed text.
- **Test safety.** No test installs a hook, injects input or runs a real action, and none calls `logging::init` or `attach_file`. The window test and bench R13 share `CONTROL_CLASS`. The direct procedure calls (M6) use no window. `end_session` in a test uninstalls nothing and posts a `WM_QUIT` that the test removes again. The only new ignores are the manual M10 and M14 tests.
- **Unsafe.** Every new `unsafe` block has a concrete `// SAFETY:` comment, including the test callbacks. There is no `.as_ptr()` on a temporary; `CLASS_NAME` is a `w!` static.
- **Engine purity.** `src/engine` gains only a `std::fmt` import and `VIRTUAL_KEY` in `gestures.rs`. `platform::{foreground, desktop}` import nothing from the crate outside tests.
- **Rulings.** R1, R2, M1-M8, Q1-Q8, V2-m2..m4, critic OQ1-OQ3 and architect Q1-Q3 are carried out as the addendum states. V2-m1's timestamps need a live lock and are left to the owner (see Open questions).
- **Docs.** README, `default_config.ron`, `triggers-conditions.md`, `known-issues.md` (rows 9, 55, 63, 68, 76-80), `kinds.md`, `structure.md`, `errors.md` and the CHANGELOG (`[Unreleased]` Added/Changed/Fixed) match the code's log strings and behaviour. The CHANGELOG cites the I1 test as the proof of the watcher fix.
- **Commits.** All 22 developer commits carry the Opus 5.5 trailer.

## Positive
- The witness is one short-circuited read with a precise comment. It covers every source of an unseen key-down without a per-event cost, and C12's critic script is red on master, so the defect it fixes is real.
- R2 is solved structurally: only "input left" resets, and it releases without an OS read. That is idempotent across repeated departures and order-independent, and it is pinned by the four ruling scenarios plus the adapter test with a macro and a Hold.
- The control window's guard order, its `!Send` ownership and its destruction before the executor's bounded stop are right. `WM_ENDSESSION` removes the hook before it flushes.
- The foreground cache keyed by (HWND, pid), the own-process skip (no `WM_GETTEXT` under the borrow) and the counting `WindowQuery` seam make the latency acceptance testable without a hook.

## Open questions
- **V2-m1 / M1.** The ruling asks that `impl.md` record one Win+L and one Ctrl+Alt+Del round trip with timestamps. Agents may not run the daemon, so `impl.md` defers this to the owner. The orchestrator should confirm that the deferral satisfies the ruling. Settled by: the owner's `daemon.log` lines.
- **Unverified OS behaviour.** Each item has a manual check (M1, M5, M12, M13, M14):
  - that the `EVENT_SYSTEM_DESKTOPSWITCH` away/back events reach an out-of-context hook on the Default desktop;
  - that `UOI_IO` reads FALSE while Winlogon has input;
  - that the async key state shows a key pressed in a higher-integrity window or on the secure desktop.

  None can be settled from code.
- **Tester commits.** The tester's commits after `6f6edb3` show that three behaviours had no failing test at `6f6edb3`:
  - the drain's remapping-off gate (M12);
  - `input_returned`'s foreground re-read (M13);
  - the second-Launch damage of a freed key (M20).

  They are pinned at `4539306`. The triage of those findings is not part of this review.
