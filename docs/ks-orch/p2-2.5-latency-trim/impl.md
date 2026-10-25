# Implementation: p2-2.5-latency-trim round 1 (developer, resumed after rulings round 4)

**Verdict: DONE.** The work tree is clean on `perf/latency-trim`, HEAD `786e61f`. The branch has 17 commits on
`master` (`3cde1d0`):

- commits 1-13 of the cut (`7cd8237..ea944da`), from the earlier developer runs;
- four follow-up commits from this run, which carry out `rulings.md` round 4:
  - L3: D12 is reverted with `git revert`, and R16 is gated;
  - L4: a held modifier costs about 0.6 µs.

The Developer addendum at the end of `cut.md` has two new sections, "Round 4, L3" and "Round 4, L4". `fmt` and
`clippy -D warnings` pass. The book builds, and its links check clean.

Fixed points for the tester:

- **Replay tree (rulings Q9).** Commit 7, `3f0dc62`, is the replay tree of commit 8's red-first tests (`d5458d2`).
- **The tester's parent P (Q10).** P is commit 2, `62c9fd8`. `git diff --stat master 62c9fd8 -- src` lists only
  `src/bench.rs` (+119/-2) and `src/platform/timers.rs` (commit 1's doc lines, +6/-3).
- **HEAD after the revert:** `786e61f`. Its sources match commit 11 (`aa86922`) except for doc comments in
  `src/bench.rs` and `src/hook/watchdog/mod.rs` (`git diff aa86922 786e61f -- src`: 9 lines in bench.rs, 5 in
  `watchdog/mod.rs`, all `//!`).

## This run (rulings round 4)

| # | Commit | Ruling | What | Gate |
|---|---|---|---|---|
| 14 | `47af1b1` Watchdog: an idle wake reads the raw input at once again (ruling L3) | L3 | `git revert --no-commit ea944da`, then a commit message that cites L3 and its data. It removes `defers_first_read`, `Ticker::woke_idle`, `WOKE_BY_INPUT`, `Watchdog::raw_pending` and `Drive::wake`, and three of the four D12 tests. The three loop-model expectations go back to 2.4's values. The module Cost doc, the R16 doc, `README.md:955`, `errors.md:214-215` and the CHANGELOG Changed entry return to their 2.4 text | **red**, see Deviations: its tree is commit 12's (`601c874`) |
| 15 | `b106db9` Watchdog: drop the test that the first raw key event waits for a tick (ruling L3) | L3 | `git revert --no-commit ddb9f3a`: removes `the_first_raw_key_event_after_idle_is_read_at_the_first_tick` | green. Its tree is commit 11's (`b6a18e3`) |
| 16 | `e391a06` Bench: R16 is gated against R12 and costs about 4 us without the per-event reads (ruling L3) | L3 | `src/bench.rs:82-90`: R16 is gated, HEAD R16 p50 ≤ R12 p50 + 20 µs, p99 reported, and 2.4's L2 is superseded. `src/hook/watchdog/mod.rs:39-42`: Cost doc, about 4 µs, gated, about 50 µs before item 2.5. `README.md:955`: about 4 microseconds (about 50 before item 2.5). `CHANGELOG.md:664-668`: the modifier-read entry names R16's drop | green |
| 17 | `786e61f` Docs: a modifier held for a key event costs about 0.6 us (ruling L4) | L4 | `CHANGELOG.md:649-652`: "one read of about 0.6 µs each (bench rows R12m and R12m2 against R12)" | green |

**The red-first rule for this run.** None of the four commits adds behaviour. 14 and 15 are reverts, and 16 and 17
are text. So no new red-first test is needed. The revert's regression guard is item 2.4's
`the_first_raw_key_event_after_idle_wakes_the_watchdog`, restored to "no pass in `2250..5000`". A deferral moves the
first read to the tick at 250 ms, so ticking would run to 2250 ms and the test would fail.

**Tests removed** (all four D12 tests):

- `hook::watchdog::tests::the_first_raw_key_event_after_idle_is_read_at_the_first_tick` (commit 12, removed by
  `b106db9`);
- `hook::watchdog::tests::an_idle_wake_with_no_hook_installed_still_reads_at_once` (commit 13, removed by `47af1b1`);
- `hook::watchdog::tests::a_deferred_wake_is_followed_by_a_wait_that_leaves_only_the_batched_input_out` (commit 13,
  removed by `47af1b1`);
- `hook::watchdog::decision::tests::a_deferred_first_read_starts_the_silence_at_the_wake` (commit 13, removed by
  `47af1b1`).

**Expectations restored** (item 2.4, Q6):

- `a_batch_read_at_a_tick_counts_all_its_events`: `(250, 8)`;
- `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request`: the last read at 749 ms;
- `the_first_raw_key_event_after_idle_wakes_the_watchdog`: no pass in `2250..5000`.

**Mutation rows removed:**

- **X46**: the deferral reads anyway;
- **X47**: the deferral ignores `hook_installed`;
- **X48**: `raw_pending` keeps the old `quiet_since`;
- **X49**: the deferred wait keeps the batched bit.

The code these four mutate is gone. X47's other catcher, `raw_input_before_the_hook_is_installed_is_read_and_not_counted`,
stays as an item 2.4 test.

## Stop rules (D13), closed by rulings L3 and L4

The earlier run stopped on two rules: D-R16d missed on P, and R17a sat at its 0.6 µs threshold. Ruling L3 reverts D12
and gates R16. Ruling L4 accepts R17a. The developer numbers are below. They are p50 in µs, except R1, which is a
mean. Load is the machine's CPU load before each run; the owner was using the machine. The raw runs are in
`bench-dev-raw.md`, `bench-dev-reads.md` and `bench-dev-resume.md`.

| Run | Load | R1 mean | R4 | R11 | R12 | R12m | R12m2 | R16 | D-R16d | R17t | R17a | R18 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| P a | ? | 0.192 | 8.4 | 10.0 | 8.8 | 8.8 | 8.6 | 85.8 | 131.2 | 0.001 | 0.968 | - |
| N a | ? | 0.111 | 0.3 | 0.9 | 0.2 | 0.2 | 0.2 | 12.8 | 10.9 | 0.001 | 1.017 | - |
| P b | ? | 0.099 | 4.9 | 5.4 | 4.8 | 4.8 | 4.8 | 58.3 | 28.7 | 0.001 | 0.583 | - |
| N b | ? | 0.081 | 0.2 | 0.8 | 0.2 | 0.2 | 0.2 | 1.2 | 2.5 | 0.001 | 0.805 | - |
| commit 11 a | 65 % | 0.093 | 0.2 | 0.8 | 0.2 | 1.0 | 1.6 | 4.3 | 2.4 | 0.001 | 0.777 | 4.8 |
| P 1 | 66 % | 0.143 | 5.6 | 5.7 | 4.9 | 4.9 | 5.1 | 66.8 | 43.6 | 0.001 | 0.602 | - |
| commit 11 1 | 79 % | 0.113 | 0.3 | 0.8 | 0.2 | 0.8 | 1.3 | 4.4 | 18.9 | 0.001 | 1.082 | 5.9 |
| P 2 | 90 % | 0.177 | 7.1 | 6.9 | 6.6 | 5.0 | 6.0 | 68.4 | 99.0 | 0.001 | 1.098 | - |
| commit 11 2 | 100 % | 0.172 | 0.4 | 1.1 | 0.2 | 1.1 | 1.9 | 21.7 | 19.3 | 0.001 | 1.116 | 6.4 |
| HEAD 1 (`ea944da`) | 100 % | 0.224 | 0.4 | 1.3 | 0.2 | 1.4 | 3.0 | 24.0 | - | 0.001 | 0.726 | 5.7 |
| P 3 | 47 % | 0.107 | 5.0 | 5.4 | 4.8 | 4.8 | 4.8 | 66.8 | 37.4 | 0.001 | 0.584 | - |
| HEAD 2 (`ea944da`) | 39 % | 0.080 | 0.2 | 0.8 | 0.2 | 0.8 | 1.6 | 5.1 | - | 0.001 | 0.606 | 4.6 |
| HEAD 3 (`ea944da`) | 39 % | 0.092 | 0.2 | 0.8 | 0.2 | 0.8 | 1.3 | 5.6 | - | 0.001 | 0.891 | 6.7 |

- **R16 after the revert.** The new HEAD runs commit 11's code. On commit 11, R16 was 4.3 and 4.4 µs at 65-79 % load,
  and 21.7 µs at 100 %, against R12's 0.2. L3's gate is HEAD R16 p50 ≤ HEAD R12 p50 + 20 µs: the two runs below full
  load pass by about 16 µs, and the 100 % run misses by 1.5 µs. The gate belongs to the tester's session. The
  tester's re-run rule for runs at 79 % load or more applies.
- **R17a (L4).** Over 19 runs the median p50 is 0.613 µs, and 0.561-0.576 µs at 7-16 % load. One held modifier costs
  about 0.6 µs on the callback (R12m 0.8 against R12 0.2). Two cost 1.1-1.4 µs (R12m2 1.3-1.6).
- **The other rules** passed in the earlier run and still apply:
  - (a) N R12 = 0.2 against 0.5 × P R12;
  - (b) Δ R1 = 0.081 and 0.018;
  - (c) R17t is 0.001 µs, so E4 and E5 keep `GetTickCount`.
- No bench was run in this run. HEAD's code is commit 11's (above), and the tester runs P, N and HEAD in one session
  (rulings, "For the tester").

## Items (commits 1-13, from the earlier runs)

| # | Commit | Cut | Main locations (HEAD) | Red-first |
|---|---|---|---|---|
| 1 | `7cd8237` Docs: a Hold's fast actions run on the main thread, and timers.rs serves three threads | D14 | `CHANGELOG.md:116`, `:799-802`; `src/platform/timers.rs:1-3` | none (docs) |
| 2 | `62c9fd8` Bench: rows R12m, R12m2, R17t and R17a; the callback's events carry a time stamp | D13 | `Cargo.toml` (`Win32_System_SystemInformation`); `src/bench.rs` (rows, `call_injected_callback`, time stamps); CHANGELOG Added | the bench is ignored. P tree |
| 3 | `9a9bdcf` Engine: a swallowed repeat of a modifier with no record keeps its release from Windows (red) | D10 | `src/engine/scenarios.rs` | `engine::scenarios::a_swallowed_repeat_of_a_modifier_with_no_record_keeps_its_release_from_windows`: red on master with "scenario: up LShift at +0 ms: Windows got a lone key-up of LShift" |
| 4 | `c6b186b` Engine: a swallowed repeat with no record is recorded, so its key-up asks Windows | D10 | `src/engine/keys.rs:70` (`RepeatSuppressed`), `:244` (`mark_repeat_suppressed`), `:283` (`take_blocked`); `src/engine/mod.rs:940-948`; README; CHANGELOG Fixed | 3 turns green. X1-X3 carry the compile-red unit tests |
| 5 | `045b8b4` Main thread: read the modifiers between events when something could have hidden them | D4-D6, D8 (E3u, E8), D15 | `src/engine/modifiers.rs` and `src/engine/resync.rs` (new). `Engine::resync_if_due` (`mod.rs:670`), `input_returned` (`:682`) and `on_foreground_changed` (`:657`). `KeyState::resync_modifiers` (`keys.rs:104`). `HIDDEN_INPUT` and the notes (`main_thread.rs:128-142`), `MainState::resync_if_due` (`:383`) and the drain step. The foreground callback's wake (`keyboard_hook_callback.rs:502`). The harness; R18 | compile-red (new API), carried by X5-X20 |
| 6 | `b0db237` Engine: a key event with no modifier held asks Windows about no modifier (red) | W2 | `src/engine/mod.rs` tests | `engine::tests::a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier`: red with `left: 44`, `right: 0` |
| 7 | `3f0dc62` Engine: the key path no longer reads every modifier from Windows | D1-D3 | `on_key` without `sync_modifiers` (`mod.rs:544-570`); `track_injected` (`mod.rs:693`, `keys.rs:150`, `keyboard_hook_callback.rs:240-296`); `resync_from_os` (`keys.rs:181`); the harness's generic and injected steps; `Mode::Unnoticed` | 6 turns green. X6-X24 carry S3-S9, G1, I-1 and the unit tests. **Replay tree of commit 8** |
| 8 | `d5458d2` Engine: a modifier Windows has up does not count for the next key event (red) | W2a | the harness's later-hook steps; R-A..R-H | red on `3f0dc62` by assertion; the eight messages are in the commit body |
| 9 | `e84625e` Engine: a key event asks Windows only about the modifiers held for it and drops those Windows has up | D16 without D17 (V3-I1); D3's clear in the drain | `KeyState::clear_released_modifiers` (`keys.rs:127`); the checks at `mod.rs:565` (key-down) and `:1005` (swallowed key-up); the generic clear-only read in `resync_modifiers`; `ModifierSet::MENU`; the Re-entry docs (`keyboard_hook_callback.rs:108-117`); row 88; `keys.md:194`; README; CHANGELOG | 8 turns green. X25-X34 and X50 carry C-1, C-2 and the V3-I1 pin |
| 10 | `4703385` Hook: a late callback or a long stretch away from messages makes the main thread read the modifiers | D8 (E4, E5), D9 | the callback's exit (`keyboard_hook_callback.rs:438-449`), `event_age_ms` (`:463`), `late_callback` (`:471`); `HOOK_LATE_MS`, `tick_count` and `note_if_awake_too_long` (`main_thread.rs:151-185`) and E5 in `MainThread::run`; `Counter::HookLate` and `warns` (`counters.rs:69-124`); `writer.rs`; README; `errors.md:231`; CHANGELOG | compile-red, carried by X38-X41 |
| 11 | `aa86922` Watchdog: raw key events no callback explains make the main thread read the modifiers | D8 (E6) | `health.rs` (`unseen_input`); `decision.rs` (`unseen_at_last_tick`); `watchdog/mod.rs` (`RequestReinstall::hidden_input`, `Ticker::pass`); `main_thread.rs:674` (`wake_for_hidden_input`) and the notes sum | compile-red, carried by X42-X45 |
| 12 | `ddb9f3a` Watchdog: the first raw key event after idle is read at the first tick (red) | D12 | **reverted** by `b106db9` (L3) | it was red with `left: Some((0, 1))`, `right: Some((250, 1))` |
| 13 | `ea944da` Watchdog: an idle wake only starts the ticks | D12 | **reverted** by `47af1b1` (L3) | the test of 12 turned green; X46-X49 were confirmed red. All of it is gone now |

The tests of commits 3-11 are listed in the cut and are all present at HEAD. The D17 tests C-3..C-7 were never written
(V3-I1). Commit 9 adds the V3-I1 pin,
`engine::tests::a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it`.
Commit 10 adds `logging::writer::tests::a_report_whose_only_count_is_hook_late_is_written_at_info`.

## Mutation checks run by the developer (still valid at HEAD)

The earlier run applied each mutation to the work tree, ran the named test and restored the file. It compared each
file's md5 before and after.

| # | Mutation | Test | Outcome |
|---|---|---|---|
| X50 (V3-I1) | the key-down check leaves out `touched_by(VK_LWIN)` (D17 restored) | `a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it` | red: "the exact F13 binding fires", `left: []`, `right: [DoNothing]` |
| X31a (v3 M1) | the key-up check over `ModifierSet::ALL` | R-F | red at "LWin up, swallowed": `left: 2`, `right: 0` |
| X31b (v3 M1) | the key-up check at a key-up that passes too | R-F | red at "A up with LAlt held, passing": `left: 1`, `right: 0` |
| X51 (v3 M2) | the witness term removed (`mod.rs:999`: `decision.suppress = swallow;`) | `engine::properties::no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed` | red: "scenario: up CapsLock at +6735 ms: VK 0x14 is down in Windows but physically up", config default, seed 8, `Unnoticed` mode |
| X4 (v3 M2 candidate) | D10 reverted (no `mark_repeat_suppressed`) | the same property | **green**: the property does not catch X4. Commit 3's scenario does ("Windows got a lone key-up of LShift"). So X51 is the property's named mutation |

X46-X49 are removed with D12 (above). Rows X1-X45 of the cut's mutation table stand. X35-X37 were dropped in round 3.

## Remarks

Every remark of the three critiques ends in one state. The rulings settled most of them in the cut, and the code follows
the rulings.

**critique.md (round 1):**

- **I1** (residuals against "an unknown case keeps the read"): RESOLVED by ruling W1. The witness reads at every
  swallowed key-up (`mod.rs:999`, unchanged). Pin: `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key`
  (unchanged).
- **I2** (the clock base): RESOLVED by W4. `hook_late` is a resync trigger only (`keyboard_hook_callback.rs:438-449`).
  It is logged at `[INFO]` when it is the only count (`counters.rs:92-124`). Pins:
  `hook_late_alone_is_reported_at_info_and_with_another_count_at_warn` and `an_injected_events_own_stamp_is_not_its_age`.
  Manual check M8.
- **M1** (E5 notes after the stretch's events): RESOLVED. E5 checks before the wait and `continue`s to the drain
  (`MainThread::run`). Pin: `the_main_loop_checks_its_awake_stretch_before_every_wait`. R-M1 is declared in row 88.
- **M2** (a short `SendInput` count, injected generic codes): RESOLVED by D2 and D3. Pins:
  `the_daemons_own_injected_modifiers_pass_and_are_tracked` and G1.
- **M3** (the deferral's cost): RESOLVED by ruling L3. No deferral is left, so there is no extra wake or tick.
  `errors.md:215` and the module Cost doc carry item 2.4's text (`47af1b1`).

**critique_v2.md:**

- **I1** (the generic phantom): RESOLVED by W2a. The drain clears a set generic slot that reads up (`keys.rs:104`), and
  the check reads the generic code (`keys.rs:127`). Pins: R-C, R-D, R-G and C-2.
- **I2** (W10, a later hook): RESOLVED by W2a and D16. Pins: R-A and R-B. M10 is the AutoHotkey manual check.
- **M1** (four visible effects): RESOLVED in row 88 (`known-issues.md:156`).
- **M2** (the scenario-to-mutation map): RESOLVED. S9 asserts `expect_nothing_fired()` on the second press.
- **M3** (one more wake only for a single-event burst): RESOLVED by ruling L3. With the deferral reverted, no burst
  costs a wake more than in item 2.4, and the texts are item 2.4's (`47af1b1`).
- **M4** (mutations): RESOLVED (X1-X3, X16, X23). X48 and X49 went with D12 (L3).
- **M5** (the bench gates): RESOLVED in the cut's D13. They are the tester's gates. R16's gate is L3's.

**critique_v3.md:**

- **I1** (D17 wrong in the PowerToys case): RESOLVED by ruling V3-I1. D17 is not built, and master parity holds.
  - Pin: `a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it`
    (`src/engine/mod.rs:1468-1489`). Mutation X50 was confirmed red.
  - The residual is declared in row 88 and in the Re-entry docs (`keyboard_hook_callback.rs:108-117`). Manual check
    M11 is below.
- **M1** (R-F and X31's second variant): RESOLVED. R-F has the step "A up with LAlt held, passing"
  (`mod.rs:1553`). X31a and X31b were both confirmed red.
- **M2** (the `Unnoticed` property has no mutation): RESOLVED with X51, confirmed red. The property does not catch X4
  (above).
- **M3** (M9 cannot fail): RESOLVED. M9 gains the latched step (Manual checks below).
- **M4** (direction B and the blocker): RESOLVED in row 88 (`known-issues.md:156`) and in the addendum.
- **M5** ("the next key-down"): RESOLVED in row 88, `keys.md:194`, the CHANGELOG and the addendum.
- **Open questions 1-3**: none of them is a ruling.
  - OQ1 goes to the tester. Δ was 0.018 and 0.081 here.
  - OQ2 stays as it is: the cut names `engine::ModifierSet`, and the two types live in different modules.
  - OQ3 is noted in the addendum.

**rulings.md round 4:**

- **L3**: RESOLVED.
  - The reverts are `47af1b1` and `b106db9`, and the gate and figures are in `e391a06`. Addendum: "Round 4, L3".
  - The guard is the restored `the_first_raw_key_event_after_idle_wakes_the_watchdog`.
  - R16's gate is the tester's.
- **L4**: RESOLVED in `786e61f` (CHANGELOG) and in the addendum, "Round 4, L4", which covers the cut and rulings texts.
  `results.md` is the analyst's.

## Deviations from the cut

- **`47af1b1` fails the gate.** The two reverts run newest first, as `git revert` of a range does.
  - So `47af1b1` has commit 12's tree (`601c874`): commit 12's test stays and its code is gone. It is red by that one
    assertion, which I re-ran there: "assertion `left == right` failed: read at the first tick: [(0, 1)]",
    `left: Some((0, 1))`, `right: Some((250, 1))`.
  - `b106db9` turns it green, because it has commit 11's tree.
  - I saw this only after committing. The other order would have kept every commit green. History is never rewritten,
    so it is declared here and in the addendum. It adds one red commit to the declared red commits 3, 6, 8 and 12.
- **Manual check M5** goes back to roadmap check 44 as written, together with M6 and M7, which L3 names. M5's
  amendment ("one tick (250 ms) after the raw input") came from D12.
- **R16's figures** in `README.md:955`, the module Cost doc and the CHANGELOG come from the developer's runs at commit
  11 (4.3 and 4.4 µs; 66.8 µs on P in the same session). They do not come from a new HEAD run, because HEAD runs
  commit 11's code. They are measured figures, as 2.4 V3-M2 asks.
- **Wording in commit messages, which are never amended:**
  - `786e61f` says "R12m2 is 1.1 to 1.4 us above it", meaning above R12.
  - `47af1b1` says "the next commits set R16's gate and its figures". That is `e391a06` alone.
- From the earlier runs, still valid:
  - **D17 not built** (ruling V3-I1);
  - **row 88's and `keys.md:194`'s links pin `045b8b4`** (commit 5), not the merge commit;
  - **D9:** `summary` returns `Option<Report>`;
  - **the probe files** are `bench-dev-raw.md`, `bench-dev-reads.md` and `bench-dev-resume.md`.
- The earlier deviations about D12 (D-R16d read on commit 11, `defers_first_read`, the commit 13 CHANGELOG text) are
  moot: the code is reverted.

## Unsafe and atomics added

None in this run. Commits 1-11, unchanged:

- `src/hook/main_thread.rs:161-165` (`tick_count`): "SAFETY: GetTickCount takes no arguments, reads no memory of ours
  and cannot fail."
- `src/bench.rs`: the same for `GetTickCount`. The R17a read has "SAFETY: GetAsyncKeyState takes the key code by
  value, writes no ...".
- `src/hook/keyboard_hook_callback.rs` tests: `PeekMessageW` on the test thread's own queue, and the extern foreground
  callback with a null window.
- `src/hook/health.rs`: `unseen_input` is `fetch_add(1, AcqRel)` on the watchdog thread and `load(Acquire)` on main.
  It publishes no other data. No `Relaxed`.

## Gates (this run, at HEAD `786e61f`)

- `cargo fmt --check`: pass (also after the reverts, at `b106db9`).
- `cargo clippy --all-targets --locked -- -D warnings`: pass (at `b106db9`, after `e391a06`'s edits, and at HEAD).
- Tests run:
  - `hook::watchdog`: 79 passed. That is 83 at `ea944da` minus the four D12 tests, run after the reverts and again at
    HEAD.
  - `config::loader`: 91 passed. It covers the README examples, since the README changed.
  - `bench`: 1 passed, 1 ignored.
  - commit 12's test at `47af1b1`: red, as above.
- The full suite is the tester's.
- Manual ignores added: none.

## cfgcheck

This run changed no published RON example. The manual-check configs were validated with the cfgcheck harness in an
earlier run. Its output is in `cfgcheck-output.txt`, read with `2>&1`:

```text
== m10.ron
[INFO] loading config from D:/tmp/ks-orch/p2-2.5-latency-trim/cfg/m10.ron
OK: 2 bindings (after auto-blocker expansion)
  #0: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(20))] }) action=SwitchLanguage block_default=true block_original_combo=None condition=Always trigger=Immediate
  #1: kind=Combo(KeyCombination { keys: [AnySide(Ctrl), Key(VIRTUAL_KEY(74))] }) action=SwitchLanguage block_default=true block_original_combo=None condition=Always trigger=Immediate
exit 0
== m11.ron
[INFO] loading config from D:/tmp/ks-orch/p2-2.5-latency-trim/cfg/m11.ron
OK: 2 bindings (after auto-blocker expansion)
  #0: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(20))] }) action=SwitchLanguage block_default=true block_original_combo=None condition=Always trigger=Immediate
  #1: kind=Combo(KeyCombination { keys: [Key(VIRTUAL_KEY(124))] }) action=SwitchLanguage block_default=true block_original_combo=None condition=Always trigger=Immediate
exit 0
```

## Docs

- **Book config pages:**
  - `book/src/config/keys.md:194` (generic codes; commit 9);
  - `known-issues.md:156` (row 88; commits 9-11);
  - `errors.md:231` (`hook_late`, the `[INFO]` rule; commit 10);
  - `errors.md:214-215`: back to item 2.4's text (`47af1b1`).
- **README:**
  - `:149` (`GetAsyncKeyState` uses);
  - `:558-570` (KS-14 and D10);
  - `:871-881` (`hook_late`);
  - `:955` (the watchdog's wake: about 4 microseconds, about 50 before item 2.5; `e391a06`);
  - `:959` (focus-transition repair).
- **CHANGELOG (`[Unreleased]`, all citing roadmap 2.5):**
  - Added: R12m/R12m2/R17t/R17a, R18, `hook_late`.
  - Changed: the modifier resync, with about 0.6 µs per held modifier (L4) and R16's drop to 4.4 µs (L3); the
    counters line level.
  - Fixed: D10.
  - The D12 entry is gone (`47af1b1`).
- **`default_config.ron`:** none, since no config meaning changed.
- **Build:** `mdbook build book -d D:/tmp/ks-orch/p2-2.5-latency-trim/book` builds, with only the mermaid
  preprocessor's version warning. The internal link check of the built HTML (`D:/tmp/ks-orch/phase-0/htmllinks.py`)
  gives `SUMMARY pages=20 links=2566 broken=0 duplicate_ids=0` (`linkcheck.txt`).

## Book pages and agent files left to the phase book sync (non-config, unchanged at `786e61f`)

- **`architecture.md`:**
  - `:237` (step 8, "Modifier resync": now the check of D16 and the drain read);
  - `:248` (step 15, the menu mask after the key-up check);
  - `:251` (the suppression invariant: D10's `RepeatSuppressed`);
  - `:265` (mermaid "resync modifiers from the OS");
  - `:294` (foreground changes: a hiding-count bump and a wake, no reads);
  - `:331` (the `KeyState` row);
  - `:350` (`Unknown`, and the new `RepeatSuppressed` row);
  - `:362` ("Only the modifier codes are resynced from the OS on every event");
  - `:566-567` (the per-event resync);
  - `:605` (the two sources of truth).
- **`glossary.md`:**
  - `:67` (the async key state);
  - `:377` (modifier: the generic codes come from events);
  - `:569` (WinEvent);
  - `:577` (the witness);
  - add "Hiding count". No "Unapplied key" (V3-I1).
- **`overview.md:16`.**
- **`gaps.md`:** `:167` (non-modifier key state), `:300`.
- **`roadmap.md`:**
  - `:584` (2.4's key-path line). It says R12 4.7 µs and R16 "about 50 µs ... declared, measured and not gated". Now
    R12 is about 0.2 µs, and R16 is about 4 µs and gated (L3).
  - `:586-595` (2.5 status: W1, W2 and W2a done; D12 withdrawn by L3; R16 gated; `:593`'s "about 50 µs" is the
    pre-2.5 figure).
  - `:654` (check 44's expected text: "the first key of each burst pays bench row R16, about 50 µs at the median"
    becomes about 4 µs). Check 44 is otherwise unchanged; no "one tick later" amendment, since D12 is reverted.
  - `:739` (3.5). The witness follow-up lapses (W1). FU-4 stays listed with the new numbers and is likely moot (L3).
    Q11, M9 and the property coverage of D10 are candidates.
  - `:952` (open question 13, the outcome).
  - `:999` (the FU-4 row). It stays, with R16 about 4 µs above R12 instead of about 50 µs, likely moot (L3).
  - `:1005`.
  - The re-runs M1-M11.
- **Not book pages; `CLAUDE.md` is not edited (brief).** For the orchestrator's chore after the merge (Q5):
  - `CLAUDE.md:72` says "R16, one wake per burst, about 50 us, is declared and not gated";
  - `.claude/agents/tester.md:101` says "R16 is a declared row, not gated".

  Both state 2.4's ruling L2, which L3 supersedes. The new gate: HEAD R16 p50 ≤ HEAD R12 p50 + 20 µs, median of 3
  runs, p99 reported.

## Manual checks (human only; no test is ignored for them)

- **Unchanged from the cut:** M1-M4, M8 and M10.
- **M5, M6 and M7:** roadmap checks 44, 39 and 47 as written for item 2.4 (L3). The cut's D12 notes no longer apply.
  - In check 44, `raw keyboard input arrives` is logged at the wake's read, as in item 2.4.
  - In check 39, W ≤ 100.
  - In check 47, W ≤ K/2, with `hook ok, reinstalls: 0`.
- **M9, latched step (v3 M3).**
  - Turn on Sticky keys (Settings, Accessibility, Keyboard), and turn it off afterwards.
  - Tap LShift, then press CapsLock with nothing in between. Expected: `ToggleCapsLock`, which master gives if
    `GetAsyncKeyState` reads the latch down. Record what HEAD does. A difference from master is a 3.5 follow-up.
  - Then the existing steps: LShift, `a`, CapsLock; LShift, a mouse click, CapsLock. Each CapsLock switches the
    layout.
- **M11 (V3-I1).**
  - Start PowerToys Keyboard Manager first, remapping Left Win to F13. Then run `swch open`, so that PowerToys' hook
    runs after the daemon's.
  - Use a temporary `( keys: "F13", action: SwitchLanguage )` (`cfg/m11.ron`), and remove it afterwards.
  - Expected: every LWin press switches the layout, and Start does not open.
- **M8 (W4).** Type normally for 5 minutes on the release daemon; `hook_late` stays 0. Input from Remote Desktop may
  count; that is expected and not a defect.

## Follow-ups for 3.5

- **Q11.** The own-key skip keeps row 63's modifier case: a modifier released unseen and pressed again, with no
  key-down in between, reads as an auto-repeat.
- **M9.** If the latched step differs from master, a StickyKeys latch is direction B with no LL event (W13).
- **Property coverage of D10.** The random scripts never take D10's path: the `Unnoticed` property passes under X4. A
  property case that does take it would pin D10 beyond commit 3's scenario.
- **FU-4 (from 2.4, kept by L3).** Trim the watchdog's first-pass user32 work, if check 44 shows that raw input comes
  before the hook call. R16 is now about 4 µs above R12 (4.3 and 4.4 against 0.2 at commit 11), so it is likely moot.

## Open points

- **The gate failure at `47af1b1`** (Deviations). It is declared and cannot be undone without rewriting history.
- **`CLAUDE.md:72` and `tester.md:101`**: the chore after the merge (above).
- **No pre-existing warnings.**
- **Build directories** `D:/wt/_targets/ks25-probe`, `ks25-probe11` and `ks25-head` from the earlier runs are left in
  place. They are build output, outside the repo, and no worktree of them is registered.
