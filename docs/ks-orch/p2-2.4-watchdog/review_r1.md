# Code review: p2-2.4-watchdog @ 4a90d44 (plus tester commits to 140b831)
CHANGES_REQUESTED (Critical 0, Important 2, Minor 4)

Scope: `git diff master...feat/hook-watchdog` (merge base `fa20b30`). I read the developer's 28 commits at `4a90d44`. During the review the branch moved to `140b831`, with four test-only tester commits (`065c824`, `ec25787`, `7f10ecc`, `140b831`; Sonnet trailer). I read those too. They add tests on the fake installer, the fake clock and the fake timer. None of them reaches Win32, and none changes a line cited below. Every non-test line number below holds at both refs.

Files read in full through `git show`: `src/hook/{installer,health,debug_hold}.rs`, `src/hook/watchdog/{mod,decision}.rs`, `src/logging/panic.rs`, `src/platform/{timers,desktop}.rs`, and the changed regions plus their call sites in `keyboard_hook.rs`, `keyboard_hook_callback.rs` (callback 352-401), `main_thread.rs` (request queue, `run`, `drain`, `serve`, `reinstall_if_silent`, tests), `core/app.rs` (`App::run` 185-338), `ipc_server.rs`, `logging/{mod,counters,writer,sink}.rs`, `engine/{mod,keys,testing,scenarios,winner}.rs`, `bench.rs`, `tests/ipc_roundtrip.rs`, README, CHANGELOG and `book/src/config/errors.md`.

## Gate: fmt ✅ · clippy ✅
- `cargo fmt --check`: exit 0, at `4a90d44` and again at `140b831`.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0, at `4a90d44` and again at `140b831`. It ran with its own `CARGO_TARGET_DIR=D:/tmp/ks-orch/p2-2.4-watchdog/target-review`. Logs: `D:/tmp/ks-orch/p2-2.4-watchdog/review-clippy.log`.
- Extra check, not part of the gate: `cargo clippy --release --all-targets --locked -- -D warnings` exits 0 with 0 warnings, so the `cfg(debug_assertions)` split compiles clean (`review-clippy-release.log`).

## Remarks

### 🔴 Critical
None.

### 🟡 Important

#### I1. The S4 scan skips every `mod tests` block, the code that compiles only under test (ruling S4, critique I2)
**Where**: `src/hook/installer.rs:499` (`for (index, line) in lines[..tests_start(&lines)].iter().enumerate()`), with `tests_start` at `:410-415`. Token list at `:383-389`.  **Confidence**: CONFIRMED (code read; the counterexample below follows from it).

**Problem**: ruling S4 asks for a scan that asserts the installer calls "appear only in items compiled out under `cfg(test)`". The scan reads each file only up to its first `#[cfg(test)]` + `mod tests {` pair. Everything from that line to the end of the file is never scanned. That region is exactly the code that is compiled *only* in the test build. So the guard does not cover the place where a regression would put a live hook on the owner's machine. Two smaller holes add to it:
- the token `SetWindowsHookExW` does not match `SetWindowsHookExA`;
- `build.rs`, which `lib.rs:22-25` compiles into the lib test harness, is not scanned.

**Failure**: a later commit adds this to `src/hook/keyboard_hook.rs`'s `mod tests` (or any other file's):
`use windows::Win32::UI::WindowsAndMessaging::*; #[test] fn t() { unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_callback), None, 0) }.unwrap(); }`
- `the_hook_installers_are_compiled_out_of_the_tests` stays green.
- `cargo test` installs a global LL hook on the owner's desktop, on a test thread that does not pump.
- Every key the owner types then stalls until `LowLevelHooksTimeout`.

S1 holds today: `git grep` over `src` and `tests` finds the five names only in the `win32` block and in two string literals. The regression guard that S4 requires does not hold.

**What to do**:
- Scan the whole file, test modules included. `code_of` already strips comments and string literals, and the scan's own tokens are built with `concat!`, so nothing in the tree matches today.
- Match the prefix `SetWindowsHookEx`.
- Cover `build.rs`, or say why not.

#### I2. The latency acceptance is unmet, and nobody has ruled on it: R14 misses D11's bound by about 4.6x (owner latency decision, spec acceptance)
**Where**: bound at `cut.md:369` (`R14 p50 <= R12 p50 + 300 ns; p99 <= R12 p99 + 2 us`). Row at `src/bench.rs:453-480`. Data at `D:/tmp/ks-orch/p2-2.4-watchdog/bench-dev.md:105-107,127-129,169-172,192-195`. Open point at `cut.md:745-746` and `impl.md` open point 1. `rulings.md` "Latency" assigns the reporting to the results analyst but does not waive or rule on the R14 bound.  **Confidence**: CONFIRMED that the bound is missed, on the developer's recorded runs (the tester owns the official runs). PLAUSIBLE that the daemon pays it per key.

**Problem**: the spec's acceptance line "The latency bench shows no unjustified regression on the key path with the watchdog active" is not met as built:
- R12 p50 is 4.7-4.8 us. R14 p50 is 22.2-22.8 us, and R14 p99 is 91.7-92.1 us against R12's 5.9-10.9 us.
- R14/settled is within 0.5 us of R12. The developer's uncommitted run at `THREAD_PRIORITY_HIGHEST` shows the same gap (`impl.md` Bench).
- So the cost is not scheduler preemption on the bench thread. It is the callback contending with the watchdog thread's own `PeekMessageW`/`GetRawInputData`/`DispatchMessageW`.

The justification ("raw input follows the hook chain") rests on Q6, which nobody has verified. The module docs disclose the cost (`src/hook/watchdog/mod.rs:31-35`), but disclosing a cost is not the ruling the owner decision needs.

**Failure**:
- If Windows posts a key's `WM_INPUT` before or while it calls the LL hook chain (Q6), every key event's callback overlaps the watchdog's handling of that key: about +18 us at the median and about +80 us at p99 on the hot path, against about 4.8 us today.
- Even if raw input follows the chain, the case still occurs for events that come within the watchdog's handling time of each other:
  - the key-up after a key-down;
  - the down and up that one `SendInput` of a `PressKey` injects, whose second callback runs while the watchdog reads the first one's raw input;
  - chords.

**What to do**: get the orchestrator's ruling before the merge, with evidence. Either:
- (a) accept, with Q6 settled by the researcher query in `cut.md:691` or by manual check M6; or
- (b) take raw input in batches at the ticks, so the watchdog does no work per key.

Which one is the orchestrator's call. This remark closes when one of them is recorded.

### 🟢 Minor

#### m1. The doc of `Counter::WatchdogTimerNotSet` says the retry waits for a raw key event, and the code does not
**Where**: `src/logging/counters.rs:51-53` ("it tries again at its next wake, which the next raw key event brings"). Code: `src/hook/watchdog/mod.rs:193-202`, `:593-595`.  **Confidence**: CONFIRMED.
**Failure**: a reader of the doc believes ruling I1 is not met, that is, that a failed set with no further typing loses the tick. In fact `wake_by` makes the wait time out at the tick, and `a_timer_that_refuses_once_is_set_again_at_the_next_pass` pins that. `errors.md` describes it correctly ("ticked by its wait's timeout instead").
**What to do**: make the doc match `wake_by`.

#### m2. With the log writer gone, a panic's message is written nowhere
**Where**: `src/logging/panic.rs:83-96`. `report_on`'s result is dropped at `:93`. `send_by` returns `SendFailure::Gone` and drops the message (`src/logging/writer.rs:99-117`). The default hook is not chained (`panic.rs:57-59`).  **Confidence**: CONFIRMED by trace. It needs the writer thread to have died first.
**Failure**:
1. The `log-writer` thread panics. Its own line goes to stderr: `name == Some("log-writer")`.
2. Later, the main thread panics.
3. `LOGGER.writer.get()` is `Some(tx)` and `try_send` is `Disconnected`, so the line is dropped.
4. No stderr fallback runs, and Rust's default message is replaced, so the console shows nothing at all.

The ordinary log path falls back to stderr in the same situation (`src/logging/mod.rs:79-85`).
**What to do**: fall back to `write_to_stderr` when `report_on` reports `Gone`, as `DaemonLogger::log` does.

#### m3. "Never two hooks, never none" holds only while no other process's LL hook sits above the daemon's
**Where**: `src/hook/keyboard_hook.rs:263-267`. `CHANGELOG.md:189` ("so a key never sees two hooks of the daemon or none").  **Confidence**: PLAUSIBLE. It depends on win32k resolving `CallNextHookEx`'s next hook at call time.
**Failure**:
1. Another program's LL hook (AutoHotkey, PowerToys, mstsc) was installed after the daemon, so it sits above the daemon's hook.
2. A key event is inside that program's hook while main runs `reinstall_keyboard_hook_with` (`:283-288`) with the old hook still installed (`StillInstalled`).
3. The new hook goes in at the head, which that event has already passed.
4. If the program calls `CallNextHookEx` after the old hook is removed, the event skips the removed hook and reaches neither hook of the daemon.
5. That one key passes unremapped. If it is the key-up of a swallowed key-down, a lone key-up reaches Windows.

The new-first order is still the right one. Only the claim is too strong.
**What to do**: qualify the doc comment and the CHANGELOG line, or list it with residuals R-1/R-7.

#### m4. The README source tree does not list the new `hook/debug_hold.rs`
**Where**: `README.md:64-74`. The tree gained `health.rs`, `installer.rs` and `watchdog/`, but not `debug_hold.rs`. (`logging/panic.rs` falls under the known FU-3 gap.)  **Confidence**: CONFIRMED.
**Failure**: none at runtime. A reader of the tree does not find the debug-only module that the Hook watchdog bullet (`README.md:939`) describes.
**What to do**: add the line, and mark it debug builds only.

## Checklist walked (no reachable failure, one line each)
- **Rulings S1-S3**:
  - S1: the five Win32 installer calls exist only in `#[cfg(not(test))] mod win32` (`installer.rs:60-159`), and under test `System` is `unreachable!` (`:170-194`).
  - S2: `29060a1`/`2904d04` land before `a8aa0f7`.
  - S3: four fake-installer tests: new hook first with counts `[2, 1]`, a refused install keeps the old hook, not installed installs nothing, Gone reported. All on leaked hooks, so no drop reaches `SYSTEM`.
  - The watchdog thread tests use `start_synthetic`, which skips `RegisterRawInputDevices`.
  - The tester's `ec25787` adds the `hook_installed` set/clear tests on the fake.
- **I1 (loop)**: `Ticker::pass` re-arms from `Watchdog::next_tick` on every wake. `plan`'s `Keep` is safe: SetWaitableTimer keeps a pending signal, and the clock check ignores an early signal. A failed set times the wait out at the tick. Synchronization timer, so no busy loop (`timers.rs:29-31`).
- **I3 (order)**: `HookWatchdog::start` (`app.rs:220`) runs before `ControlWindow::create` (`:235`) and `install` (`:245`). Stop (`:321`) runs after `uninstall()?` (`:316`) and `drop(control_window)` (`:320`).
  - Raw input is gated on `hook_installed` (`watchdog/mod.rs:177`).
  - Early returns after `install` uninstall first (`:259-265`).
  - A panic unwind with the hooks live is the known FU-1, and the watchdog's drop answers WM_QUIT at once.
- **M1-M7, Q1-Q8**: carried out as in the addendum. I traced `keeps_ticking` (`decision.rs:188-195`), the M2 counter (`watchdog/mod.rs:267-270`) and the M3 clear (`health.rs:109-113`, `watchdog/mod.rs:211-217`). Ruling M4: the refusal test holds `GLOBAL_HOOK`, the subtraction is saturating, and every test writer uses leaked counters.
- **R1 callback**:
  - `code < 0` is still first (`keyboard_hook_callback.rs:357-362`).
  - The heartbeat is one `fetch_add` (`:367`, `health.rs:57-61`), with no allocation, lock, log or Win32 call. It counts before the sentinel check by design (D2), and it feeds no engine state, so no feedback loop.
  - The debug hold is `cfg(debug_assertions)` only (`:383-384`), and the release clippy above confirms the split.
  - Every path still ends in `LRESULT(1)` or `CallNextHookEx`.
  - The hook-path scan lists `callback_seen`, `hold_if_asked` and `should_hold` (`logging/mod.rs:462-466`).
- **R2 sentinel**: no new `SendInput`/`keybd_event`. `the_watchdog_injects_no_input` scans the watchdog, the health and installer files and the reinstall functions, and grep agrees.
- **R3 shared state**:
  - `HealthState` is atomics only. It is 128 bytes with align 64, so the bools the watchdog writes do not share a line with `KeyboardHook`'s hot fields.
  - The watchdog holds `&'static HealthState` and a `MainThreadHandle`, never `&KeyboardHook`.
  - Reinstall runs only from `serve` (`main_thread.rs:825-828`), in the drain, outside `with_main`. The one `with_main` call (`:861`) only resets.
- **R4 orderings**: Release/Acquire for the bools, AcqRel for the RMWs. The time-then-count store in `record_reinstall` pairs with the count-then-time load in `snapshot` (`health.rs:89-99,124-135`). No `Relaxed`, no `SeqCst`. The load-then-store in `hook_answered` and `set_timer_failing` races only with main's single-request answer, and an interleaving leaves at most a transient flag.
- **R5 builders**: no config type changed. `HookHealth`/`DaemonStatus` are plain public data, as `DaemonStatus` already was.
- **R6 Display**: new `Display` arms for `ResetReason::HookReinstalled`, the four `Counter`s, `ReinstallOutcome`, `OldHook` and `HookHealth`. No new `_ =>` in non-test code.
- **R7 specificity**: untouched. The reinstall does not call `update_bindings`.
- **R8 key identity**: raw `VKey` is used only in `qualifies` (`watchdog/mod.rs:706-708`) and never matched against bindings.
- **R9 registry/process**: no new reads. `input_is_on_this_desktop` reads `UOI_IO` of the calling thread's desktop, which is the process's.
- **R10 loop/shutdown**:
  - The watchdog has its own `MsgWaitForMultipleObjectsEx` pump with `MWMO_INPUTAVAILABLE`, so no wake is lost.
  - Its window, registration and timer are made and dropped on its own thread.
  - `WAIT_FAILED` logs and ends the loop with `watched = false`.
  - A request after `uninstall`, `drop(main)` or `WM_ENDSESSION` gets `ShuttingDown` or `NotInstalled`.
  - A note on the addendum: its deviation 8 ("Watched drops first") is inaccurate. `ticker`, which holds the timer, drops before `_watched` (`watchdog/mod.rs:550-578`). This is harmless.
- **unsafe**: every new block has a concrete `// SAFETY:`:
  - `installer.rs:90,101,109,127,141,156`;
  - `watchdog/mod.rs:417,434,461,486,546,549,599,645,659,678,696`;
  - `main_thread.rs:885`;
  - the bench and test blocks.
  - `GetRawInputData` gets the `RAWINPUT` size in bytes and the header size. The union is read only after the `dwType` check, from a zero-initialised local, before `DispatchMessageW` frees the input.
- **No key data in logs**: watchdog lines carry only durations and counts. `no_key_data_leaves_the_raw_input_read` pins the read, and no non-test panic payload formats a key.
- **Tests**: none installs a hook, registers raw input, injects input or runs a real action. None calls `logging::init` or `attach_file`: the panic child uses `writer::attach_on` in its own process, with stdio null. The red-first chain and the mutations named in `impl.md` match the commits.
- **IPC**: the reply still starts `OK: running (N bindings live`, and `swch open`'s probe accepts any `Ok`. The protocol has no length cap. `tests/ipc_roundtrip.rs:102-170` covers it.
- **Docs**:
  - CHANGELOG: entries under Added, Changed and Fixed, citing 2.4 and KS-08.
  - `errors.md`: rows for every new log line, and their text matches the code.
  - No config page changes, and none was needed.
  - Every commit carries the right trailer: Opus on the developer's 28, Sonnet on the tester's 4.

## Positive
- The S1 seam is clean. The Win32 calls live in one `cfg(not(test))` module, and the test `System` panics before Windows sees anything. The fake models handles and keyboard counts well enough to pin the order of every step.
- The pure decision is small, well reasoned and heavily pinned: 15+ fake-clock rows, including ruling M5's ordering row and the 64 s backoff cap. The anti-loop rule is stated in the module docs.
- `reinstall_if_silent` dispatches pending sent messages before it compares the heartbeat with the request's snapshot. That makes a busy main thread harmless, and a `WM_ENDSESSION` dispatched inside it falls into the null guard.
- `Ticker`/`TickTimer`/`plan`/`wake_by` give ruling I1 one source of truth for the tick. The tests drive the real pass on a fake timer that signals early and refuses a set.
- The panic hook's bound is stated, shared across the nested panic, and tested in a child process without an abort.

## Open questions
- **Q6**: in which order relative to the LL hook chain does Windows post `WM_INPUT`, and does a hook that returns 1 suppress it? This settles I2 and residual R-1.
  - A gamedev.net report says a WH_KEYBOARD_LL hook blocked Raw Input reads on XP and Vista/7, which suggests the hook comes first. It is one report and a reply doubts it (https://gamedev.net/forums/topic/550264-dinput-and-global-key-hooks-vistawin7/).
  - Manual check M6 shows it on the owner's machine.
- **Critic OQ2**: does Windows still deliver a hook call it timed out, once main pumps again? If it does, the first request after the debug hold answers `Alive`, the reinstall comes about 1.25 s later, and M1's "within 2 s" can fail without a defect. The M1 log timestamps settle it.
