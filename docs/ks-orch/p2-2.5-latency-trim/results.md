# Results analysis: p2-2.5-latency-trim / roadmap 2.5 Latency trim

Analyst: results-analyst (Opus, xhigh). Branch `perf/latency-trim`, HEAD `d853a59`, master `3cde1d0` (merge base = master,
19 commits ahead, 0 behind, no merges). Inputs read: `spec.md`, `cut.md` (revision 2, Developer addendum rounds 3-4),
`rulings.md` (rounds 1-4), `critique.md`, `critique_v2.md`, `critique_v3.md`, `impl.md`, `pack_r1.md` (by reference),
`review_r1.md`, `test_r1.md`, the bench evidence in `bench/`, and the whole diff `git diff master...perf/latency-trim`
(24 files, +3326/-295), every changed source file read in full.

## VERDICT: ACCEPT

**Summary**: The lane does what roadmap 2.5 asks, as the binding rulings W1-W4, W2a, V3-I1, L3 and L4 reshaped it. The
per-event read of eleven modifier codes is gone from the key path: an ordinary key costs 0.2 us, against 4.7 us on the
parent. A held modifier costs one read of about 0.6 us. The first key after an idle watchdog (R16) went from 52.6 us to
0.9 us at the median. The witness is unchanged (W1). Every stuck-key and lone-key-up guarantee of 2.3 holds: 7 properties
pass, among them the new `Unnoticed` mode with later-hook presses, and D10 closes a lone-key-up hole that exists on
master. The gate is green in my own run. What remains is hand-off work: two Minor README remarks, two new rustdoc
warnings, the CLAUDE.md and agent-file chore, and the book sync. None of it blocks the merge.

One roadmap acceptance line is formally not met, by ruling. Roadmap `:595` says "R11 ... costs the same as an ordinary
key-up (R12)". Ruling W1 keeps the witness read, so R11 stays 0.6 us above R12. The owner declined to choose, the
orchestrator applied W1, and the owner can still overrule it. The roadmap text must be rewritten at the book sync.

## Quality gate (own run)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (`CLIPPY_EXIT=0`, `analyst-clippy.txt`) |
| `cargo test --locked` | lib `running 869 tests` / `ok. 862 passed; 0 failed; 7 ignored`; swch 0; `tests/ipc_roundtrip.rs` `7 passed`; doc 0 (`analyst-test.txt`, `TEST_EXIT=0`). Master: 807 `#[test]` attributes against 868 on HEAD (`git grep -h '#\[test\]'`), +61 |
| Ignored (listed, never run) | 7, the same 7 as master, all `manual:` or `bench:` (`cargo test --locked -- --list --ignored`). The diff adds and removes no `#[ignore` |
| `cargo doc --no-deps` | 5 warnings. **Two are new in this lane**: `src/engine/modifiers.rs:30` and `:127`, `[`MODIFIERS`]` is a private item linked from the docs of the public `ModifierSet` (`rustdoc::broken_intra_doc_links`). The other three predate the lane (`config::loader`, `vk_name.rs:4`). Not part of the CI gate; see P2 |
| `mdbook build book -d .../book-analyst` | OK (only the mermaid version warning); `htmllinks.py`: `pages=20 links=2566 broken=0 duplicate_ids=0` |
| Trailers | 19 of 19 commits end with `Co-Authored-By: Claude Opus 5.5` (17) or `Claude Sonnet 5.5` (the tester's `eb15d53`, `d853a59`) |
| Release test binary | `Cargo.toml:33` `test = false` on the daemon `[[bin]]`; no `--release` test was run by me |

The `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline` flake, which the tester saw in 3 of about 14
runs on HEAD and in 1 of 8 on master, did not occur in my run. The lane does not touch `src/core` (`git diff --stat` is
empty there).

## Acceptance criteria

The spec acceptance (`spec.md:15-20`) and roadmap `:595`, as amended by the rulings.

| # | Criterion | Evidence | Status |
|---|---|---|---|
| A1 | R11 within the D14 rule of R12 (roadmap `:595`, spec) | **Superseded by ruling W1**: the witness reads at every swallowed key-up. The line is unchanged: `src/engine/mod.rs:999` `decision.suppress = swallow && !env.os_key_down(vk);` is identical to master `:896`. Pin: `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key` (`mod.rs:1353`). Bench: the witness term is H 0.6 us against P 0.6 us, and H R11 is 0.8 us against the 2.3 bound of 5.4 us | MET AS RULED (W1); roadmap text to rewrite |
| A2 | A new row shows the witness reading after a foreground change | Moot under W1: the witness reads at every swallowed key-up, which R11 times | MOOT (W1) |
| A3 | Every stuck-key and lone-key-up test and property of 2.3 stays green, the unseen re-press after a focus change included | `engine::scenarios::a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows` (`scenarios.rs:2033`) ok. The 6 properties of 2.3 ok, with their assertions unchanged (`properties.rs` diff is additive; the Unlock-mode seeds keep their scripts). The scenarios diff removes only comment lines | PASS |
| A4 | Each hiding-count source triggers a resync: foreground change, reset, remapping switch, reinstall, refused event, watchdog gap, late callback | Engine: `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers` (`mod.rs:1581`), `swch_on_and_a_reinstall_read_everything_themselves_...`, `a_reload_makes_no_read`; scenarios `a_modifier_pressed_..._counts_after_a_foreground_change` (`:2461`), `..._after_the_unlock_signal` (`:2480`), `..._after_the_first_key_when_no_return_signal_comes` (`:2497`), `..._while_remapping_is_off_counts_after_swch_on` (`:2517`), `..._while_the_hook_was_gone_counts_after_the_reinstall` (`:2532`), `..._after_an_adapter_note` (`:2440`). Adapter: `a_refused_call_notes_hidden_input` (`main_thread.rs:2015`), `a_desktop_switch_to_an_unknown_destination_notes_hidden_input` (`:2025`), `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late` (`keyboard_hook_callback.rs:1026`), `an_awake_stretch_of_17_ms_notes_hidden_input` (`main_thread.rs:2056`), `a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread` (`watchdog/mod.rs:1635`), `the_drain_counts_the_watchdogs_notes` (`main_thread.rs:1977`), `a_foreground_change_queues_a_wake_for_the_drain` (`keyboard_hook_callback.rs:1347`). Each source has a mutation the tester ran (X5-X20c, X38-X45) | PASS |
| A5 | W2: a hidden modifier key-up followed by a binding press heals | R-C `another_programs_generic_shift_released_unnoticed_stops_counting_after_a_note` (`scenarios.rs:2676`), R-D (`:2692`), R-E `a_modifier_released_where_the_hook_cannot_see_it_stops_counting_at_the_next_key` (`:2706`). Red on `d5458d2` and green on HEAD (tester replay). Under W2a they heal at the press itself, before matching | PASS |
| A6 | W2: no stuck key and no lone key-up across the property modes | the 7 properties, among them `no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed` (`properties.rs:464`, with later-hook presses, Q12). Its named mutation X51 (witness term removed) fails it at config default, seed 8 (developer and tester). Also S9 `a_swallowed_modifier_stays_held_through_both_reads_and_its_next_press_stays_a_repeat`, which asserts `expect_nothing_fired()` (v2 M2), and R-A and R-B under both checks | PASS |
| A7 | W2a pins: the AutoHotkey trace, the blocker trace, I1's scenario, a read count, each with its named mutation | R-A `a_modifier_that_a_later_hook_swallows_does_not_count_for_matching` (`scenarios.rs:2619`), R-B `a_shift_that_a_later_hook_swallows_does_not_make_alt_the_language_hotkey` (`:2654`), R-C/R-D, R-F `a_key_event_asks_windows_once_for_each_modifier_held_for_it_and_never_for_its_own_key_or_a_swallowed_one` (`mod.rs:1492`). Tester's mutations: X26 (check removed), X27 (visits every slot / allowed to set), X28 (KS-14 skip), X29 (generic read made sided), X31a/b, X32, all caught | PASS |
| A8 | W2a: the drain read sets and clears the sided slots and clears a generic slot only when it reads up | R-G `the_drain_read_clears_a_generic_slot_only_when_its_code_reads_up_and_never_sets_one` (`keys.rs:823`); `a_reset_never_takes_a_generic_modifier_from_the_os`; X17, X22, X33 caught | PASS |
| A9 | W2: R12 and R1/R4 show the reads gone; a row for the resync on main | `a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier` (`mod.rs:1410`), red on `b0db237` with `left: 44, right: 0`. Bench below (R12 4.7 -> 0.2, R4 4.7 -> 0.2, the R1 gate, R18 4.5 us) | PASS |
| A10 | V3-I1: a nested decided event keeps master's reading | `a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it` (`mod.rs:1469`), X50 caught. `git grep` finds no `UNAPPLIED`, `Unapplied` or `Env::unapplied` | PASS |
| A11 | R16 meets a bound the cut sets from the evidence | L3 gate H R16 p50 <= H R12 p50 + 20 us: 0.9 <= 20.2 (median of 0.9, 0.9, 1.7); p99 7.5 us at most. Before: P 52.6 us | PASS |
| A12 | No other bench row regresses | tester's table, the D10/D14/2.4 rules: R1-R10, R13, R14/R14c, R7, R8 all pass; R6 within P + 0.5 ms. R12m2 meets its bound only to within 9 ns on the 100 ns grid (see Latency) | PASS (R12m2 marginal) |
| A13 | The 2.4 watchdog acceptance (fake-clock rows) stays green | `hook::watchdog::*` 79 ok in my run, among them the roadmap `:578-581` tests (`synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request`, `idle_asks_for_nothing_and_stops_ticking`, `key_events_with_a_silent_callback_for_more_than_1_s_ask_for_exactly_one_reinstall`, `a_busy_main_thread_gives_no_reinstall_loop`, `the_watchdog_injects_no_input`, ...) and the restored 2.4 expectations (`a_batch_read_at_a_tick_counts_all_its_events`, `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request`, `the_first_raw_key_event_after_idle_wakes_the_watchdog`). The diff against master changes no existing watchdog test | PASS |
| A14 | W4: `hook_late` triggers only, makes no `[WARN]` alone, ignores an injector's stamp, has its base marked unverified | `hook_late_alone_is_reported_at_info_and_with_another_count_at_warn` (`counters.rs:330`), `a_report_whose_only_count_is_hook_late_is_written_at_info` (`writer.rs:371`), `an_injected_events_own_stamp_is_not_its_age` (`keyboard_hook_callback.rs:1054`); `tick_count` doc (`main_thread.rs:153-160`) says the shared base is not stated. **Manual: M8** (5 minutes of typing on the release daemon, `hook_late` stays 0) | PASS + manual |
| A15 | Spec item 3: Phase 2 book-sync leftovers | `CHANGELOG.md:787-798` ("on the main thread when its gesture timer fires"), `:116` (`hold_timer_not_started` wording), `src/platform/timers.rs:1-6` | PASS |
| A16 | Docs: CHANGELOG (Changed, citing 2.5), the README latency notes, config pages where a cost changes | CHANGELOG Added/Changed/Fixed under `[Unreleased]`, each citing roadmap 2.5. README `:149`, `:558-570`, `:874-881`, `:955`, `:959`. Book config pages: `keys.md:194`, `errors.md:231`, `known-issues.md:156` (row 88). `default_config.ron` is untouched, which is right: no config meaning changed. Non-config pages go to the book sync (listed below) | PASS (Minor README drift, P1) |
| A17 | Manual checks (human only) | M1-M11: the cut's "Manual checks" table with the addendum's M11 and `impl.md` "Manual checks" (M5-M7 back to 2.4's checks 44, 39, 47 per L3; M9 with its latched step; M10 AutoHotkey `cfg/m10.ron`; M11 PowerToys `cfg/m11.ron`, both validated by cfgcheck). None needs an ignored test | LISTED, not verified; to enter the roadmap's Phase 2 checklist at the book sync |

## Issues claimed closed

The lane claims no KS-ID: roadmap 2.5 is an owner decision (open question 13), not an audit item. It claims one fix:

| Fix | Regression test | Fails on old code? | Status |
|---|---|---|---|
| D10: a swallowed auto-repeat of a modifier with no record kept its key-up passing, so Windows got a lone key-up (CHANGELOG Fixed, ruling Q13) | `engine::scenarios::a_swallowed_repeat_of_a_modifier_with_no_record_keeps_its_release_from_windows` (`scenarios.rs:2413`); `engine::tests::a_repeat_with_no_record_that_passes_after_a_swallowed_one_is_recorded_as_passed` | Yes. The tester replayed it on `9a9bdcf` (= P plus the test): "scenario: up LShift at +0 ms: Windows got a lone key-up of LShift", then green on `c6b186b`. The red commit comes before the fix in `git log` | CLOSED |

Red-first order is confirmed for every behaviour commit: `9a9bdcf` before `c6b186b`, `b0db237` before `3f0dc62`, and
`d5458d2` before `e84625e` (replayed on `3f0dc62`, per ruling Q9). The compile-red tests of `045b8b4`, `4703385` and
`aa86922` are carried by named mutations, which the tester ran: 49 of the cut's rows, plus 9 of its own (MY-a..MY-h,
MY-w1). Three gaps were closed with tester tests (`eb15d53`, `d853a59`): X12, X41b, MY-b.

## Rulings carried out

| Ruling | Carried out where | Status |
|---|---|---|
| W1 (witness stays) | `mod.rs:999` unchanged; R11's witness term 0.6 us at P and at H | done |
| W2 (resync off the key path; drain on the hiding count; failure mode traced; pins; row 88; bench) | `on_key` without `sync_modifiers` (`mod.rs:547-570`); `Resync` (`src/engine/resync.rs`, pure); drain step `main_thread.rs:860`, never in a callback (only caller `MainThread::run` `:790`); D11 in the cut; row 88; R12/R4/R1/R18 | done |
| W2 "slow resync while keys are active" | D7 dropped with `RESYNC_PERIOD` (round 2); `git grep RESYNC_PERIOD` is empty | done |
| W2a (clear-only check, own key and KS-14 skipped, generic code read; drain sets/clears sided, clear-only generic; W10 corrected; pins; R12m/R12m2) | `KeyState::clear_released_modifiers` (`keys.rs:127-146`), key-down check `mod.rs:565` before every flag reader (`:767-912`), key-up check `mod.rs:1005` before the mask (`:1016`), `resync_modifiers` (`keys.rs:104-117`) | done |
| W3 (D12) then L3 (D12 reverted, R16 gated) | D12 built (`ddb9f3a`, `ea944da`), then reverted with `git revert` (`47af1b1`, `b106db9`); HEAD source equals commit 11 but `//!` lines (`git diff aa86922 786e61f -- src`: 8+/6-, doc only); R16 gate in `bench.rs:82-90`; M5-M7 back to 2.4 wording | done |
| W4 (hook_late) | `Counter::HookLate` with `warns() == false` (`counters.rs:92-105`, no `_ =>`); writer picks INFO (`writer.rs:229-240`); injected stamp ignored (`keyboard_hook_callback.rs:442-446`); M8 | done |
| M1 (E5 ordering residual) | E5 before the wait with `continue` (`main_thread.rs:798-806`); residual in row 88 ("key events decided in the same burst of messages right after a stall") | done |
| M2, M4, v2 M2, v2 M4, v3 M2 (W-table, mutations) | tester mutation table, all caught; X51 for the `Unnoticed` property | done |
| v2 M1, v3 M4, v3 M5 (visible effects, "next key-down") | row 88 (`known-issues.md:156`) names all five effects, Ctrl+Alt+Shift+K -> Ctrl+Alt+K among them; `keys.md:194` | done |
| v2 M5 / "For the tester" (one session P, N, H; load recorded) | `bench/runs.log`: P1 N1 H1 P2 N2 H2 P3 N3 H3, load 1-51 %, no rerun needed | done |
| V3-I1 (D17 dropped, residual declared, pin, M11) | no D17 code; Re-entry docs `keyboard_hook_callback.rs:108-118`; row 88; `mod.rs:1469`; M11 | done |
| Q9 (replay tree), Q10 (P = commit 2), Q12 (`Unnoticed` + later hook), Q13 (CHANGELOG Fixed) | `impl.md` names `3f0dc62` and `62c9fd8`; the tester's Q10 diff shows only bench and doc lines; property `properties.rs:276-296`; CHANGELOG Fixed | done |
| L4 (0.6 us each) | CHANGELOG `:649-652`; addendum "Round 4, L4"; here below | done |

## Latency (owner decision: lowest latency)

The tester's single session (`bench/P1..H3.md`): P = `62c9fd8` (master plus the bench), N = P without the
`sync_modifiers` line (the probe), H = `d853a59`. The figures are the median of three p50s, in us.

| Row | Before (P) | Probe (N) | After (H) | Gate | Result |
|---|---|---|---|---|---|
| R12 ordinary key, nothing held | 4.7 | 0.2 | **0.2** | H <= N + R17t + 0.1 = 0.301; p99 0.2 <= 2.3 | pass |
| R12m one held modifier | 4.7 | 0.2 | **0.8** | H <= N + R17a + R17t + 0.1 = 0.846 | pass |
| R12m2 two held modifiers | 4.7 | 0.2 | **1.4** | H <= N + 2 R17a + R17t + 0.1 = 1.391 | formal excess of 9 ns on a 100 ns p50 grid. The means are 1.369 and 1.354 (1.577 in the loaded H3), so the run is within L4's accepted range of "1.1 to 1.4 us". Not a breach at the instrument's resolution |
| R1 engine typing script (mean) | 0.093 | 0.078 | **0.083** | H <= N + 0.5 Delta = 0.0855 (Delta 0.015) | pass, margin 2.5 ns |
| R4 | 4.7 | 0.2 | **0.2** | H <= N + 0.101; 2.2 D10 too | pass |
| R11 swallowed key-up (witness) | 5.3 | 0.8 | **0.8** | witness term H 0.6 = P 0.6 (<= P + 0.2); 0.8 <= 5.4 | unchanged, as W1 rules |
| R16 first key after idle watchdog | **52.6** (13.4-53.3) | 0.9 | **0.9** (0.9, 0.9, 1.7; p99 at most 7.5) | H <= H R12 + 20 us | pass |
| R18 the drain read on main | - | - | **4.5** (4.4-4.6; p99 at most 5.8) | evidence; <= 8 R17a + 0.5 | pass |
| R17a one `GetAsyncKeyState` | 0.552 | 0.546 | 0.545 | evidence (L4) | - |
| R17t one `GetTickCount` | 0.001 | 0.001 | 0.001 | stop rule <= 100 ns | pass: E4 and E5 keep `GetTickCount` |

- **The W2a trade in one line:** zero reads with nothing held, one `GetAsyncKeyState` per held modifier (about 0.6 us
  each, measured; the cut planned 0.36 us), none for a swallowed one or the event's own key. Master read eleven codes,
  4.7 us, on every event.
- **How often R18 runs:** once per change of the hiding count. That is the daemon's start, a foreground change, the
  return from a lock or the secure desktop (or the first decided key after it), `swch on` and a reinstall (which read
  everything themselves, so the drain reads nothing), a refused callback, a late callback or a main-thread stretch of
  17 ms or more (E4, E5), and a watchdog tick with raw input that no callback explains (E6). It never runs per key, and
  never inside a callback. The worst case in normal typing is about four per second. That happens only if raw input
  comes before its hook call, the order manual check 44 settles. Each read costs about 4.5 us on main, between events.
- **D17 dropped (V3-I1):** a passed modifier key-down has no TLS guard on the key path. It costs only the check's
  reads of the other held modifiers.
- **Added per physical event while remapping is on:** two field copies, one `GetTickCount` (1 ns) and two compares at
  the callback's exit (`keyboard_hook_callback.rs:426-449`). A sentinel-tagged modifier event adds one `with_main` and a
  store. A foreground change adds one `PostThreadMessageW` to itself. No allocation, lock, `log::` or blocking call.

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, outcomes, no blocking/alloc, symmetry) | OK | `code < 0` first (`keyboard_hook_callback.rs:399-404`), heartbeat, then the remapping gate; every path ends in `LRESULT(1)` or `CallNextHookEx`. No `log::` anywhere in the file. No allocation: `ModifierSet::codes()` is an iterator over a const array. Indexing by `vk` is guarded by `ModifierSet::ALL.contains` (`keys.rs:150-155`). Symmetry: no read writes `last_down`; records plus witness unchanged; D10 closes a hole; 7 properties green |
| Sentinel ordering (principle 3) | OK by ruling; CLAUDE.md text to update | `decide_event` still tests the sentinel first (`:246`), but the sentinel path now updates the engine's flag of a modifier (`track_injected`, `:287-296`; cut D2, ruling W2 "its own and the sentinel-tagged injected ones included"). Nothing fires and the event always passes, so no feedback loop. CLAUDE.md principle 3 ("before it mutates any state") must say so: hand-off |
| Engine purity, owner matching decisions | OK | `#![forbid(unsafe_code)]` (`mod.rs:251`); `resync.rs` is pure data with no clock; the OS reads go only through `Env`. Exact matching, `allow_extra_modifiers` and the repeat defaults are untouched; sided handling is in one place (`ModifierSet::touched_by`) |
| Thread ownership, sentinel, atomics, SAFETY | OK | No new thread, hook, window or timer. The drain read runs on main under `with_main` with `GetAsyncKeyState` only. The watchdog only bumps `unseen_input` (`fetch_add(AcqRel)`/`load(Acquire)`, `health.rs:132-142`) and posts a wake. No `Relaxed`, `SeqCst` or `static mut` added. New `unsafe`: `tick_count` (`main_thread.rs:161-165`), bench `GetTickCount`/`GetAsyncKeyState`, and the test `PeekMessageW` and extern foreground call. Each has a concrete `SAFETY:` |
| Config reload, IPC | OK (untouched) | `Reloaded` neither reads nor bumps (`a_reload_makes_no_read`) |
| No key logging | OK | the diff adds no `log::` call; `counters.rs` names no logging (`logging::tests::the_functions_on_the_hook_path_log_nothing`) |

## Review status

- `review_r1.md` at `786e61f`: **APPROVED (Critical 0, Important 0, Minor 2)**. No triage was needed. m1 and m2 are
  CONFIRMED Minor and still open at HEAD (P1).
- The two commits after the review (`eb15d53`, `d853a59`) are the tester's. `git diff --stat 786e61f d853a59` shows
  only `mod tests` additions in `engine/mod.rs` (+25) and `main_thread.rs` (+61). I read them: three tests that close
  mutation gaps, with no production change.
- `test_r1.md`: **GREEN**. Every critique Important finding (v1 I1/I2, v2 I1/I2, v3 I1) is resolved by a ruling and
  carried out in the code, as the table above shows.

## Docs and hygiene

- CHANGELOG follows Keep a Changelog: Added (R12m/R12m2/R17t/R17a, R18, `hook_late`), Changed (the modifier reads with
  0.6 us each and R16's drop; the INFO counters line) and Fixed (D10), each citing roadmap 2.5. The D12 entry is gone
  with the revert.
- Config pages: `keys.md:194` (generic codes), `errors.md:231` (`hook_late`, INFO rule), `errors.md:214-215` back to
  2.4's text, `known-issues.md:156` (row 88, "By design", with the five visible effects). No closed workaround to
  remove.
- `audit.md`: no status line to change (no KS-ID). `default_config.ron`: unchanged, correctly.
- Dependencies: only the `windows` feature `Win32_System_SystemInformation` (for `GetTickCount`), which the cut names.
  `Cargo.lock` is unchanged and `--locked` passes.
- Branch: cut from `master` at `3cde1d0`, only this lane's 19 commits, clean to merge with `--no-ff`.
- History: `47af1b1` fails the gate by one assertion (the reverts ran newest-first). The developer declared it, and
  `b106db9` restores green. The other red commits are red-first by design (`9a9bdcf`, `b0db237`, `d5458d2`, `ddb9f3a`).
  First-parent `master` will hold only green trees.

## Technical debt and residuals (where each is recorded)

- Known issue 88 (`known-issues.md:156`) declares the residuals of W2/W2a/V3-I1. The ones a user can see: a binding
  misses and its sibling acts (CapsLock switches the layout while an unseen Shift is held); the Alt+Shift blocker misses,
  so Windows switches the layout; Ctrl+Alt+Shift+K arrives as Ctrl+Alt+K; with an unseen Alt or Win, a swallowed key
  gets no menu mask, so the menu bar or Start opens; a binding on a lone modifier acts once more. Every case needs a
  modifier event that the hook missed with nothing the daemon noticed after it. No key sticks and no lone key-up results.
- Q11, the own-key skip that keeps row 63's case: `impl.md` "Follow-ups for 3.5".
- M9, a StickyKeys latch (direction B with no LL event): `impl.md`, 3.5 follow-up if the owner's check differs from master.
- D10 property coverage: the `Unnoticed` property passes under X4, so D10 rests on one scenario. `impl.md`, 3.5.
- FU-4 (watchdog first-pass user32 work): kept, likely moot (L3). `impl.md`, and roadmap `:999` at the book sync.
- U1, the `KBDLLHOOKSTRUCT.time` / `GetTickCount` base: unverified, settled by M8. If it fails, every key notes
  `hook_late` and the drain reads after every key: an extra cost on main, off the key path, which loses part of the
  lane's saving without breaking correctness. The fallback (E4 on run time alone) is in cut Q2.
- No new `TODO` or `FIXME` (grep over the added lines).

## Problems

None blocks the merge. These Minor items should be done while the context is fresh:

### P1. README text drift (review m1, m2; CONFIRMED Minor, open)
- **What**: `README.md:151` still says injected events are skipped "before the engine sees them". Since `3f0dc62`, a
  sentinel modifier updates the engine's flag (`keyboard_hook_callback.rs:287-296`). The README tree `:52-63` lists
  neither `engine/modifiers.rs` nor `engine/resync.rs`, and `keys.rs - ... modifier resync` is stale (`winner.rs` was
  missing already).
- **Impact**: a reader or a later lane (3.1) takes it from the README that the sentinel path changes no engine state.
- **Root cause**: the README edits of commits 7 and 9 covered the `Env` line (`:149`) but not the sentinel bullet or the tree.
- **Return to**: `doc-writer` at the phase book sync, or the orchestrator's post-merge chore. No re-run of the lane.
- **Re-check condition**: `README.md:151` says that a sentinel modifier only updates the engine's flag. The tree lists
  `modifiers.rs` and `resync.rs`.

### P2. Two new rustdoc warnings
- **What**: `src/engine/modifiers.rs:30` and `:127` link `[`MODIFIERS`]` (a `pub(super)` const) from the docs of the
  `pub` `ModifierSet`.
- **Root cause**: an intra-doc link to a private item; `cargo doc` is not in the CI gate, so nothing caught it.
- **Return to**: `developer`, as a one-line follow-up in the next lane that touches `engine/` (or the chore). Write the
  link as plain code text, or link `ModifierSet::ALL`.
- **Re-check condition**: `cargo doc --no-deps 2>&1 | grep MODIFIERS` is empty.

### P3. Stated R16 figure differs between sessions (Note)
- `README.md:955`, the module Cost doc (`watchdog/mod.rs:39-42`) and `CHANGELOG.md:664-668` give the developer's
  session: R16 4.4 us at the median, about 4 us above R12. The tester's session measured 0.9 us (0.7 us above R12).
  Both say "on the developer's machine" and are true upper figures. The book sync may restate them from the tester's
  session.

## Hand-off: phase book sync (doc-writer)

The non-config pages that `impl.md` "Book pages and agent files left to the phase book sync" lists, all checked
against HEAD:
- `roadmap.md:586-595`: rewrite 2.5's What/Acceptance/status: W1 (the witness kept, so R11 stays 0.6 us over R12),
  W2/W2a done, D12 withdrawn (L3), R16 gated and measured 52.6 -> 0.9 us. `:584` (2.4's key-path line: R12 4.7 us, R16
  "declared, not gated"). `:654` (check 44: "about 50 us" becomes about 1-4 us). `:739` (3.5: the witness follow-up
  lapses; Q11, M9 and D10 property coverage are candidates; FU-4 likely moot). `:952` (open question 13, the outcome).
  `:999` (FU-4 row), `:1005`.
- Add the lane's manual checks M1-M11 (cut "Manual checks", the addendum's M11, `impl.md` "Manual checks") to the
  roadmap's "Manual checks after Phase 2" / Checkpoint 2 list, with M8, M10 and M11 marked as the ones that check
  this lane's own assumptions (the clock base, the AutoHotkey and PowerToys later hooks).
- `architecture.md:237, :248, :251, :265, :294, :331, :350, :362, :566-567, :605` (per-event resync -> the check plus
  the drain read; `RepeatSuppressed`; the foreground change bumps and wakes).
- `glossary.md:67, :377, :569, :577` and a new "Hiding count" entry (no "Unapplied key").
- `overview.md:16`; `gaps.md:167, :300`.
- Optional: `kinds.md` §10 could name D10's corner case (a repeat with no record that a binding swallows keeps its
  key-up from Windows unless Windows has the key down). README `:569` already states it.
- P1 (README) if the orchestrator routes it here.

## Hand-off: orchestrator (CLAUDE.md, agent files; post-merge chore, rulings Q5/Q9)

- `CLAUDE.md:72`: "R16, one wake per burst, about 50 us, is declared and not gated" becomes **R16 gated: HEAD R16
  p50 <= HEAD R12 p50 + 20 us, median of 3, p99 reported (ruling L3)**. Add this lane's bench rules: P on the lane's
  bench commit, N (P minus the removed line), H in one session; R12/R12m/R12m2/R4 against N; R1 with 0.5 Delta;
  R11's witness term; R17t/R17a/R18 as evidence.
- `.claude/agents/tester.md:101`: "R16 is a declared row, not gated" becomes the same gate. `tester.md` §3: a lane may
  name a branch commit as the replay tree of a red-first test (Q9).
- CLAUDE.md principle 1: the sanctioned additions on the hook path now include the clear-only check (D16, one
  `GetAsyncKeyState` per held unswallowed modifier), E4's `GetTickCount` and `note_late` (a TLS add plus a counter
  bump), and the sentinel modifier's `with_main` update. The sanctioned key-state reads are the witness, the KS-05
  blocker read and the check.
- Principle 3: a sentinel-tagged modifier event updates the engine's flag (D2); it is never decided and always passes.
- Principle 5: `HIDDEN_INPUT` (main-thread TLS `Cell`) and `HealthState::unseen_input` (atomic, AcqRel/Acquire).
- Principle 6: the drain ends with the modifier read on the hiding count; the foreground WinEvent callback posts
  `MAIN_WAKE`.
- Layout: `src/engine/modifiers.rs` (`ModifierSet`), `src/engine/resync.rs` (`Resync`, the hiding count).
- `.claude/agents/code-reviewer.md`: the hot-path checklist gains the same sanctioned reads (cut Q5).
- For the owner's report: W1 (R11 unchanged, the roadmap's R11 line deliberately not met), the known issue 88 effects
  listed above, and the manual checks M8 (clock base), M9 (StickyKeys latch), M10 (AutoHotkey) and M11 (PowerToys).
  These are the checks whose failure would change a declared residual.

## Positives

- The largest key-path cost is gone where it matters most: zero Win32 reads for ordinary typing. Every case costs less
  than master, and a shortcut pays at most one read per modifier it holds.
- Correctness rests on records and the witness, never on the reads: no read writes `last_down`, and the check only
  clears, so the check reads a subset of what master read. D10 fixes a real lone-key-up hole on master.
- The `Unnoticed` property mode, with later-hook presses, guards stuck keys without the harness's idealized notes.
- The work was evidence-driven: D12 was measured, found to buy nothing, and reverted by `git revert` instead of kept.
  The tester's mutation run found three untested wirings (X12, X41b, MY-b) and closed each with a test.

## Notes for later lanes

- **3.1 (single input module), new feature 6 (`HoldModifier`):** every injection must keep the sentinel, or D2 stops
  tracking the daemon's own modifiers. A held injected modifier must be released the same way on `off`, on reload and
  at Win+L.
- **R16's construction self-limits.** With a callback shorter than a thread wake, the synthetic watchdog pumps its
  first pass after the timed callback and the stop. On HEAD a bench run takes about 12 s. On P it takes about 4.4 min,
  because the ticking stop waits a tick per sample. So R16 now times the start of the wake, not an overlap with its
  user32 pass. It still catches a regression: a callback that grows back past a few microseconds overlaps again. No row
  times the first key after idle with a held modifier (one read). That press is rare, since a burst usually starts with
  the modifier itself, which asks nothing.
- **`core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline`** is a real-clock test that flakes on master
  and HEAD. The next lane that touches `core/executor` should widen it or move it to a fake clock.
- **3.5:** Q11 (the own-key read), M9's outcome, D10 property coverage, FU-4 (likely moot), U1 if M8 fails (E4 falls
  back to run time).
