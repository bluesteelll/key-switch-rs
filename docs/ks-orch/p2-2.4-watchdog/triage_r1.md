TRIAGE: 3 confirmed, 0 refuted, 0 deferred

# Triage r1: p2-2.4-watchdog @ 140b831 (feat/hook-watchdog)

Inputs:
- `D:/tmp/ks-orch/p2-2.4-watchdog/test_r1.md`: red item F1.
- `D:/tmp/ks-orch/p2-2.4-watchdog/review_r1.md`: Important I1 and I2, Minor m1-m4.

I checked each item against the branch HEAD through `git show feat/hook-watchdog:<path>`, against `spec.md`, and against `rulings.md` (which wins over `cut.md`).

The three CONFIRMED lines cover two distinct defects:
- **F1 and I2 are one defect**: R14 misses its bound. It needs an orchestrator ruling before any developer work.
- **I1 is a test-only fix** to the S4 scan.

## Verdicts

- **T-F1 CONFIRMED**: R14 misses cut D11's own bound, and no ruling waives it.
  - The bound is `cut.md:369`: `p50 <= R12 p50 + 300 ns; p99 <= R12 p99 + 2 us`, and its stated justification is only "the watchdog thread on another core".
  - The tester's raw tables (`bench-tester-head-{1,2,3}.md:19,21`) show:
    - R12 p50 4.7-4.8 us, p99 4.8-7.0 us.
    - R14 p50 21.5-23.1 us, p99 53.7-65.9 us.
  - The developer's runs (`impl.md:129`) agree: R14 22.2/91.7 and 22.8/92.1.
  - Refutation attempts:
    1. **Ruling?** No. The "Latency" section of `rulings.md` (written 12:13, before any bench data at 13:19) only tells the results analyst what to report. It calls R14 "the added atomic increment", which presumes the bound would be met. It does not waive or relax D11. No later ruling exists: `pack_r1.md:263` and `impl.md:165` both list it as "Needs a ruling".
    2. **Harness artefact?** No. `src/bench.rs:453-480` builds R14 exactly as `cut.md:357` specifies: post one synthetic raw event, then time the callback. So the row fails the acceptance measurement the cut itself chose. R14/settled (300 us spin after the post, `bench.rs:469-472`) returns to R12 +0.3 us. So the gap is the watchdog's wake and per-message work (`watchdog/mod.rs:592-620`, the `MsgWaitForMultipleObjectsEx` return, `PeekMessageW`, `DispatchMessageW`) running at the same instant as the callback, not the heartbeat.
    3. **Does the daemon ever hit that instant?** That is the cut's open question Q6, which cannot be settled without a live hook. A finding that cannot be refuted stands.
  - Facts that would settle it:
    - the orchestrator's ruling on `cut.md:745-746` option (a) accept, (b) batch raw input at the ticks, or (c) raw input on main (R15 3.9 us);
    - the owner's manual check M6 for Q6.
    - A web search found no authoritative source on when `WM_INPUT` is posted relative to the LL hook chain. The only lead is the gamedev.net thread the review already cites.

- **R-I1 CONFIRMED**: the S4 scan does not cover the code that compiles only under test, which is the code ruling S4 is about.
  - The scan loop reads each file only up to its first `#[cfg(test)]` + `mod tests {` pair: `src/hook/installer.rs:499` (`lines[..tests_start(&lines)]`, helper at `:410-415`).
  - No second guard exists:
    - there is no `clippy.toml` / `disallowed-methods` in the tree (`git ls-tree feat/hook-watchdog`);
    - the `cfg(test)` `System` panic (`:169-194`) guards only calls through the `Installer` trait, not a direct `windows::...::SetWindowsHookExW` call.
  - The addendum (`cut.md:712`) restates "before its tests module" without a reason. It is not a ruling.
  - **Failure**: a later `#[test]` inside `src/hook/keyboard_hook.rs`'s `mod tests` calls `SetWindowsHookExW(WH_KEYBOARD_LL, ...)` directly.
    - `the_hook_installers_are_compiled_out_of_the_tests` stays green.
    - `cargo test` then puts a live LL hook on the owner's desktop from a test thread that does not pump, so typing stalls up to `LowLevelHooksTimeout` per key.
  - S1 itself holds today. `git grep -E "SetWindowsHookEx|UnhookWindowsHookEx|SetWinEventHook|UnhookWinEvent|RegisterRawInputDevices"` over `src`, `tests` and `build.rs` finds calls only in `installer.rs:60-159`. The other hits are two string literals (`keyboard_hook.rs:184,199`) and comments.
  - Scanning whole files is therefore feasible at HEAD with no false positives: `code_of` strips strings and comments, and the token list is built with `concat!`.
  - The two sub-points also hold:
    - the token `SetWindowsHookExW` does not match `SetWindowsHookExA`;
    - `build.rs` is compiled into the lib test harness (`src/lib.rs:20-25`) but is not under `src/`, so it is never scanned.
  - Direction: scan whole files, match the prefix `SetWindowsHookEx`, add `build.rs`. The red proof is a probe call inside some `mod tests` that must turn the scan red.

- **R-I2 CONFIRMED (the same defect as T-F1)**: the spec's acceptance "no unjustified regression on the key path with the watchdog active" is unmet as built, for the reasons under T-F1. It closes with the same ruling.
  - One sub-claim is weaker than stated: "the key-up after a key-down" of human typing does not plausibly overlap.
    - Physical events are milliseconds apart, while the watchdog's handling takes tens of microseconds.
    - If raw input follows the hook chain, the realistic overlap is a burst of injected events in one `SendInput`: the down and up of a `PressKey`, or the events of a `type_text`. Event N+1's hook call can then run while the watchdog reads event N's `WM_INPUT`.
    - This narrows what the ruling has to accept but does not refute the remark.

- **R-m1 NOTED**: `src/logging/counters.rs:51-53` says the retry "is brought by the next raw key event". `Ticker::wake_by` (`watchdog/mod.rs:197-202`) and the wait timeout (`:593-595`) retry at the tick. Only the doc is wrong.
- **R-m2 NOTED**: `src/logging/panic.rs:93` drops `report_on`'s result.
  - `writer::send_by` returns `Gone` on a disconnected queue (`writer.rs:104-106`), and the hook does not chain to Rust's default. So with a dead `log-writer` and the `OnceLock` still set, a later panic's line goes nowhere.
  - The module doc (`panic.rs:24-25`) promises stderr only for "no writer". This is reachable only after the writer thread itself died.
- **R-m3 NOTED (PLAUSIBLE)**: the "never two hooks or none" wording at `keyboard_hook.rs:263-267` and `CHANGELOG.md` ignores a foreign LL hook above the daemon's whose `CallNextHookEx` runs across the swap. The new-first order is right; only the claim is too strong.
- **R-m4 NOTED**: the `hook/` tree in `README.md:64-74` lists `health.rs`, `installer.rs` and `watchdog/` but not `debug_hold.rs`.
- **T-note NOTED** (not a red item; the tester's "Noticed", `impl.md:167` open point 3):
  - `HookWatchdog::spawn` gives up after 5 s (`watchdog/mod.rs:400-402`).
  - If the thread's `ready.send` lands in the one-slot buffer just before the receiver drops, the thread runs on, unstopped, with `watched = true`, while `App::run` logs "the hook watchdog is off" (`core/app.rs:220-228`).
  - The executor has the same window. It is cosmetic for `swch status`, and it needs a 5 s thread start.

## For the orchestrator
- Fix round, developer: R-I1 (test-only change to the S4 scan).
- Ruling needed before the merge: T-F1 = R-I2.
  - A ruling (a) closes both with a documented, accepted R14. The tester re-states the bound or marks R14 as an evidence row.
  - A ruling (b) or (c) is a design change for the developer, followed by a new bench run.

Sources consulted for Q6: [gamedev.net 550264](https://gamedev.net/forums/topic/550264-dinput-and-global-key-hooks-vistawin7/), [LowLevelKeyboardProc, Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc). Neither settles it.
