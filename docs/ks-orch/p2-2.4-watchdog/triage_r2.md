TRIAGE: 2 confirmed, 0 refuted, 0 deferred

# Triage r2: p2-2.4-watchdog @ f51dc7d (feat/hook-watchdog)

Inputs:
- `D:/tmp/ks-orch/p2-2.4-watchdog/test_r2.md`: one red item, F1.
- `D:/tmp/ks-orch/p2-2.4-watchdog/review_r2.md` (reviewed at `959fce1`): Important I1, Minor m1 and m2.

I checked each item against `feat/hook-watchdog` HEAD `f51dc7d` with `git show` and `git diff`, and against `spec.md` and `rulings.md` (rounds 1-3), which win over `cut.md`.

`git diff 959fce1 f51dc7d` touches only the `mod tests` of `src/hook/watchdog/{decision,mod}.rs`, +70 lines in three tester rows. Those rows are pure (a `Feeder`, a `Drive`, `wait_ms`), with no Win32 call, hook or injection. So the bench tree `959fce1` has the same production code as HEAD.

Both CONFIRMED lines are **one defect**: bench row R16 misses the sanity bound that the orchestrator accepted. As in round 1 (T-F1 = R-I2), this needs an **orchestrator ruling before any developer work**. Neither the tester nor the reviewer found a code defect.

## Verdicts

- **T-F1 CONFIRMED**: R16 misses its accepted sanity bound on every run, at p50, and no ruling restates the bound.
  - **The bound.** `cut.md:984` (L1.6): R16 p50 <= HEAD R12 p50 + 25 us, p99 <= R12 p99 + 100 us.
    - `rulings.md:133-135` accepts it ("Delta Q1-Q6 accepted as the cut recommends: ... R16's sanity bound").
    - `cut.md:1118` gives its purpose: "it catches a first-key pass that grew".
  - **The raw files.** I checked `bench-r2-head-{1,2,3}.md`, row 24 in each:

    | Run | R12 p50 / p99 (us) | Bound p50 / p99 (us) | R16 p50 (us) | R16 p99 (us) |
    |---|---|---|---|---|
    | 1 | 4.7 / 4.8 | 29.7 / 104.8 | 54.6 | 107.6 |
    | 2 | 4.7 / 4.8 | 29.7 / 104.8 | 52.7 | 101.2 |
    | 3 | 4.7 / 4.8 | 29.7 / 104.8 | 46.1 | 121.9 |

    - So p50 misses its bound 3/3, by about 2x, and p99 misses 2/3.
    - The developer's run (`bench-dev-r1.md:24`) agrees: 53.5 / 168.4.
  - **Refutation attempts:**
    1. **A later ruling?** No. `rulings.md` ends at round 3. The developer lists R16 as "Open points 1, needs an orchestrator ruling" (`fix_r1.md:1`, `:147`), and the Developer addendum does too (`cut.md:764`).
    2. **A harness artefact or a wrong ref?** No.
       - `src/bench.rs:576-606` (`r16_first_key_of_a_burst`) follows `cut.md:972-976` step by step:
         - `start_synthetic`, then a 2 ms spin;
         - `post_synthetic_raw`, then at once the timed `call_keyboard_callback(VK_A, WM_KEYDOWN)`;
         - an untimed key-up, then `stop`.
       - Ruling V3-M2 is met: `set_hook_installed(true)` and n = 1000 (`bench.rs:1128`).
       - The runs were release builds with `--test-threads=1`, each tree in its own worktree and target dir. They ran one after another, never concurrently (`bench_run_r2.sh` alternates prev and head in one loop), and all six exited 0.
       - The tree is the right one (see above).
    3. **"Declared" means "justified", so it is not RED?** No. The cut justifies the per-burst wake only "with a sanity bound" (`cut.md:984`, "Justified extra work" column). `tester.md:93` says "A breach is RED unless the cut justified that extra work". Beyond the bound, the cut justifies nothing.
    4. **A lane-code cause that a fix could remove, so this is a code remark and not a bench remark?** No user-mode contention exists.
       - The callback's pass path for the unbound A uses only the heartbeat RMW and `CallNextHookEx` (`keyboard_hook_callback.rs:352-400`).
       - `HookHealth` is atomics only (`health.rs:15-50`). No lock is shared with the watchdog's pass.
       - So the extra time comes from the OS: the cross-thread wake and the watchdog's user32/win32k calls running at the same instant. A daemon key whose callback overlaps the pass pays it too.
       - D-D in `bench-dev-diag.md` (a warm thread idle 2.4 s, the daemon's real state at the first key of a burst) measured 79.7 us. So R16 is not pessimistic about the thread's state. It is pessimistic only in the order of cut Q6.
    5. **Does cut Q6 make the overlap impossible for a real first key?** Not settled.
       - [Raymond Chen, "When something gets added to a queue, it takes time for it to come out the front of the queue"](https://devblogs.microsoft.com/oldnewthing/20140213-00/?p=1773) says the Raw Input Thread consults every low-level hook before it modifies input state and delivers the input. It does not name `WM_INPUT`.
       - The [gamedev.net thread](https://gamedev.net/forums/topic/550264-dinput-and-global-key-hooks-vistawin7/) has contradicting posts.
       - If the order is hook first, the first key's own callback never meets the wake it causes. The events exposed would then be the ones that follow within the pass: the injected down and up of a `PressKey`, or a `type_text` burst, as `cut.md:1101` already states.
       - This narrows what the ruling has to accept. It does not change whether the accepted bound was met.
  - **What would settle it:** the orchestrator's ruling on `fix_r1.md:147`'s options:
    - (a) restate R16 as a declared evidence row with the measured numbers, p50 about 46-55 us, once per burst of keys after 2 s without keys;
    - (b) a delta that trims the idle wake. D-B (53.6 us with no `raw_batch` and no timer set) and D-pre (35.6 us before L1) give no assurance that +25 us is reachable.

    The real-key exposure needs the owner's manual check of Q6.

- **R-I1 CONFIRMED (the same defect as T-F1)**: the measurement and the refutation attempts are the same as above. The review states the facts correctly:
  - D-A 51.7, D-B 53.6, `HIGHEST` 58.5, D-D 79.7 and D-pre 35.6 us match `bench-dev-diag.md`;
  - `README.md:939` says "about 50 microseconds".

  It closes with the same ruling.

- **R-m1 NOTED**: the wording of `book/src/config/errors.md` is wrong for isolated bursts.
  - The page says of the stop line "N stays far below M ... N about as large as M says that Windows wakes it for every key event, which is worth reporting".
  - `decision.rs:68-71` sets `TICK` = 250 ms and `ACTIVE_FOR` = 2 s. So each isolated burst costs 1 wake plus about 8 ticks, whatever its length.
  - The review's example holds: 100 isolated Ctrl+C presses give M = 400 and N about 900-1000 on a correct build.
  - Doc only, no behaviour.
- **R-m2 NOTED (PLAUSIBLE)**: key codes of earlier batches outlive their batch.
  - `RawBuffer([0; RAW_BUFFER_BYTES])` (`watchdog/mod.rs:659`) lives as long as the thread. Each `GetRawInputBuffer` writes from offset 0, and the walk leaves the bytes in place.
  - So the VKey/MakeCode of a batch larger than the next ones stay on the thread's stack until a batch as large overwrites them.
  - Nothing logs or exposes them: `RawBuffer` has no `Debug`, and only counts leave the walk. The exposure is a memory dump of the elevated daemon.
  - Whether the spec's "no key data ... is kept" covers transient process memory is a reading question. Zeroing the written prefix or a sentence in the module's "No key data" docs closes it.

## Not triaged (not red items or remarks), recorded for the orchestrator

- **Tester's R14 p99 note**: not a red item.
  - Under `tester.md:93` (the largest p99 per row), R14 66.9 us against R14c 107.1 us passes V3-I2's p99 rule.
  - Per run, run 3 is +10.5 us. The control row with no watchdog itself swings 9.1-107.1 us between runs, so these 1 ms paced p99s are scheduler noise.
  - The p50 acceptance line (L1) is met in all three runs: 4.8 against 4.8 us.
- **Review Open questions**:
  - OQ1 is answered by the tester's three runs (above).
  - OQ2: hand the owner only `fix_r1.md`'s M9 rule, not `cut.md:1092`'s older K >= 150 / W <= 130.
  - OQ3 (E4) and OQ4 (does raw input report a `KEYEVENTF_UNICODE` event's `wScan` as MakeCode, which weighs m2) remain manual or research questions.
- **The tester's three `Tests:` commits** (`7300822`, `995892f`, `f51dc7d`):
  - All three are inside `mod tests` and pure.
  - No test installs a hook, injects input or touches the registry.
  - They pin the M10-M12 gaps the tester found by mutation.
