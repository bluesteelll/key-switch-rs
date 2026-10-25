# Code review: p2-2.3-control-window @ 81bb38c (round 2)
APPROVED (Critical 0, Important 0, Minor 1)

Scope: `git diff 6f6edb3..feat/control-window` (tip `81bb38c`, 5 commits: tester `c7ab5c7`, `7ab78ab`, `4539306`; developer `c4f1bca`, `81bb38c`), read through `git show` / `git diff` only. Inputs: `pack_r2.md`, `triage_r1.md`, `review_r1.md`, `fix_r1.md`, `rulings.md`, and the developer's and tester's mutation logs (`fix_r1_*.txt`, `mut_M12b.txt`, `mut_M13b.txt`, `mut_M20b.txt`).

## Gate: fmt ✅ · clippy ✅
- `git archive 81bb38c` exported into a scratch tree (session scratchpad `r2tree`) with its own `CARGO_TARGET_DIR`. `cargo fmt --check` exit 0. `cargo clippy --all-targets --locked -- -D warnings`: `Finished dev profile`, no warning, exit 0.
- `cargo test` was not run (the tester owns it).

## Previous remarks
- **R-I1 / review r1 I1** (`the_watcher_thread_has_no_panic_path` scanned lines 1-36 only): ✅ closed by `c4f1bca`.
  - `src/hook/config_watcher.rs:455-460`: the region now ends at the first `windows(2)` pair whose trimmed lines are `#[cfg(test)]` and `mod tests {`. At `81bb38c` that pair is `:215-216`, and the module runs to EOF (`:506`), so lines 1-214 are scanned. The lone `#[cfg(test)]` lines at `:37` and `:56` no longer end the scan.
  - `:461-466`: a coverage assert requires `fn spawn_watcher(` inside the region, with the signature built by `concat!`, so the test cannot satisfy its own check. A scan that stops early now fails visibly (`fix_r1_redA.txt`: "ends after line 36 and misses `fn spawn_watcher(`", panic at `:456`).
  - Non-vacuity: `fix_r1_mutA.txt` (`:158` back to `.expect(..)`, red at `:473`) and `fix_r1_mutB.txt` (`.unwrap()` added in `reload_into`, red at `:474`, one line lower because of the insert). Both line numbers match the test at `81bb38c`.
  - `CHANGELOG.md:874-879` (`[Unreleased]` / Fixed) cites this test as the proof of the watcher fix, and that claim now holds.
- **R-m1 / review r1 m1** (stale loop docs): ✅ closed by `81bb38c`.
  - `src/hook/main_thread.rs:28-32` gives the order requests (including the reload) -> noted input signals -> due timers, which is the order `drain` runs at `:673-684`.
  - `:38-40`: a noted signal posts `MAIN_WAKE` (`note_input_signal` -> `wake_this_thread`, `:145`, `:170-175`), so "a request or a noted input signal waits" is right.
  - `:667-669`: the `drain` doc names all three steps.
  - `src/hook/keyboard_hook_callback.rs:16-22` step 5: `foreground_app` is `self.foreground.app()` (`:175-177`, a cached `Option<String>`, `platform/foreground.rs:150-152`), and the title (`read_title` -> `GetWindowTextW`) and `os_key_down` (`GetAsyncKeyState`, `:171`) are live calls. The doc matches the code.
- **T-N1** (`stopped < ms(100)`): unchanged, as the triage left it optional. No remark.

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor

#### m1. `MovingQuery` maps windows to PIDs 4096 and 8192, which can be the test process's own PID
**Where**: `src/hook/main_thread.rs:1645-1647`, `:1667`, `:1674`, `:1675-1679`, `:1684-1688`; `src/platform/foreground.rs:132-137`  **Confidence**: CONFIRMED (path traced; the trigger is a PID collision, rare per run)

**Problem**: `owner` returns `window.0 as usize as u32`, so windows `0x1000` and `0x2000` belong to PIDs 4096 and 8192. Windows PIDs are multiples of 4, so both are real PIDs. `ForegroundCache::window_changed` sets `self.own = pid == std::process::id()` and then `self.app = if self.own { None } else { .. }` (`foreground.rs:132-137`).

**Failure**: suppose the libtest binary runs with PID 4096. Then `foreground_changed(HWND(0x1000))` caches `app = None`, and the first assert (`:1675-1679`) fails with `left: Some(None)`, `right: Some(Some("p4096.exe"))`. With PID 8192, the assert after the drain (`:1684-1688`) fails the same way. No product code is involved. This is a spurious red of a correct product, roughly 2 in the number of live PID slots per run, and more likely on a freshly booted CI runner, where PIDs stay low.

**What to do** (optional): give the fake windows owners that cannot equal a Windows PID, for example values that are not multiples of 4. The existing `FakeQuery` in `platform/foreground.rs:261-267` uses small PIDs.

## Checklist walked (items absent with no reachable failure, one line each)
- **Delta shape.** Every hunk is a `//!` or `///` doc line, inside `mod tests` (`config_watcher.rs:216-506`, `main_thread.rs:798-`), or in `src/engine/scenarios.rs` (test-only module). No product statement changed, so the r1 checklist for R1-R10 at `6f6edb3` stands for the product.
- **R1 hook callback.** Only the step-5 doc changed (`keyboard_hook_callback.rs:16-22`). No `log::`, allocation, lock, I/O or panic path was added.
- **R2 sentinel.** No `SendInput` site and no constant in the delta (the added lines contain no `SendInput`, `dwExtraInfo` or `unsafe`).
- **R3 shared state.** The new tests reach the engine only through `with_main` on their own `MainThread::new()`. `MovingQuery` is a test-only `Rc<Cell<_>>` inside the thread-local `MainState` (not `Send`, never shared). No `static mut` and no new `Mutex`.
- **R4 atomics.** None added.
- **R5-R7.** No builder, `Display` arm or binding-publish path changed. The c7ab5c7 test reaches `update_bindings` only indirectly (the `set_enabled` reset).
- **R8 key identity.** Unchanged. The 7ab78ab scenario uses the canonical `Win` / `Alt` names.
- **R9 registry and process.** No registry access. In the c7ab5c7 test, `set_enabled(false)` runs `reset_engine(RemappingSwitched)` -> `foreground.resync()` with the real `Win32Query`. That is `GetForegroundWindow`, `GetWindowThreadProcessId`, `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION)` and `QueryFullProcessImageNameW` (reads, closed by `Owned`), plus the engine's `GetAsyncKeyState` reads. All of these are allowed.
- **R10 loop and shutdown.** `note_input_signal` posts `MAIN_WAKE` to the test's own thread, whose queue `MainThread::new` created (`:590`). Each libtest test runs on its own thread (`fix_r1_*` panic lines show distinct thread ids), so a leftover `MAIN_WAKE` dies with that thread. In the 4539306 loop, `main` drops at the end of each iteration (`Drop` takes the slot, `:692-702`), so the next `MainThread::new()` finds the slot empty.
- **Logs: no key names or typed text.** The delta adds no log statement. Captures are thread-local (`logging/testing.rs:16-20`, `:57-63`), so a parallel test cannot leak "session locked" into the c7ab5c7 capture.
- **Test safety.** No test installs a hook, injects input, runs a real action or calls `logging::init` / `attach_file`. No `#[ignore]` was added or removed. Source stays ASCII, and the added lines contain no print macros.
- **Tests can fail.** M12 (`mut_M12b.txt`: both lines captured), M13 (`mut_M13b.txt`: `p4096.exe` against `p8192.exe`), M20 (`mut_M20b.txt`: a second `Launch` at `scenarios.rs:1938`), and the watcher mutations A and B above.
- **Rulings.** No ruling-bearing code changed. The r1 verdict on R1, R2, M1-M8, Q1-Q8, V2-m1..m4, critic OQ1-OQ3 and architect Q1-Q3 stands.
- **Docs and CHANGELOG.** No config semantics changed, so no book, README or `default_config.ron` change is due. The CHANGELOG's cited proof is now valid (R-I1).
- **Commits.** Tester commits carry the Sonnet 5.5 trailer and developer commits the Opus 5.5 trailer. Subjects are `<Area>: <what>`, and no commit was amended.

## Positive
- The R-I1 fix copies the house pattern (`non_test_lines` in `keyboard_hook_callback.rs`), and it adds a coverage assert, so the same class of silent shrink cannot recur. Both reviewer mutations were replayed red.
- The tester's three tests close real mutation gaps (M12, M13, M20) without a new seam in product code. `MovingQuery` drives the return-signal re-read with no Win32 window query.
- The doc fix states the drain order that ruling R2 relies on, in the module doc and on `drain` itself.

## Open questions
- **V2-m1 / manual check M1** (carried from r1): the Win+L and Ctrl+Alt+Del timestamps need a live daemon, which agents may not run, so `impl.md` defers them to the owner. The orchestrator confirms that the deferral satisfies the ruling. Settled by the owner's `daemon.log` lines.
- **T-B1 carry-over** (not a code item): `results.md` must state the measured +0.6 us of the witness read (R11), against the "~0.1 us" in `cut.md:448` and the "~100 ns" in ruling R1.
