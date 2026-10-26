# Lane spec: p3-3.5-budget-bench - Hot-path budget: the 50-binding bench rows and the field counter (roadmap 3.5)

Branch: perf/budget-bench

Roadmap item 3.5 (book/src/roadmap.md 'Hot-path budget', :727-739 at 252daef), acceptance (c) "The callback stays under
100 us with 50 bindings". Acceptance (a) and (b) are done (2.3 `3befec7`, A `4177540`). The `when:`-after-match rule for
`sequence`/`chord` moved to p3-3.2-seq-chord-matching (plan_draft.md section 2, C1); there is no second 3.5 lane.
Binding owner decision: binds run with the lowest possible latency (CLAUDE.md). This lane is first in Phase 3 so that
every later lane runs its parent and HEAD bench against the 50-binding rows. Scope report:
<orch>/phase-3/scope/3.5-budget.md (sections 4, 8, 9).

1. New bench rows in `src/bench.rs` (header list :1-103, configs :229-252, adapter helpers :350-390, driver
   :1144-1299). Row ids are allocated by the phase plan (plan_draft.md section 4); do not reuse R20-R26.
   - `BUDGET_CONFIG`: 50 bindings: the template's 2; about 28 `keys` combos across letters and F-keys with
     Ctrl/Alt/Shift/Win; about 8 with `when:` (AppEquals, TitleContains, And/Or/Not); 8 deferred Tap/Hold/DoubleTap;
     4 auto-blockers built with `Binding::new_auto_blocker` (no registry read); 4 chords and 4 sequences with no
     `when:`. `BUDGET_TITLE_CONFIG`: the same, with 2 chords and 2 sequences under
     `when: TitleContains("no-such-title")`. Load through `from_ron_str` and `sort_by_specificity` (:229-233).
   - R19: `keyboard_hook_callback`, unbound A down and up, `BUDGET_CONFIG`.
   - R19b: a bound key-down whose winner has a `when:`, `BUDGET_CONFIG`.
   - R19c: a chord key-down (suppressed, pending), `BUDGET_CONFIG`.
   - R19w: unbound A key-down, `BUDGET_TITLE_CONFIG` (what the `when:`-before-match rule costs every key; the gate
     row of p3-3.2).
   - R19e: `Engine::on_key` typing script on `BUDGET_CONFIG`, batch-timed (mean of 1000 events per sample), because
     R1-R3 sit at the 0.1 us `Instant` floor (p2-2.2-timers/results.md:191).
   - R17f: one `GetForegroundWindow` (read only), batch-timed; evidence for known issue 80 (F3).
   - R17p: one `PostMessageW` to the bench's own message-only window, drained outside the timing; evidence for the
     cost of `SwitchLanguage`/`PostMessage` actions.
   - Optional, evidence only: R8 with the two priorities interleaved (ABAB), to settle whether R8's run order
     confounds the priority comparison (scope F-17; it reopens nothing); R16m (R16 with one held modifier, 2.5 note).
   - Hygiene: the R19 family runs after `main.refresh_foreground()` (:1200-1202), or the title path reads `None`
     (`src/platform/foreground.rs:157-158`); drain `fast_actions_run()` between samples.
2. A field counter for the 100 us budget: `Counter::CallbackOverBudget` (or two counters split by whether a fast
   action ran, the cut's choice), an INFO counter (`warns` false) with a `Display` arm (R6), in
   `src/logging/counters.rs` (enum :23, `ALL` :74, `warns` :92-106, Display :110-128, `summary` :248-271). It is
   bumped in `keyboard_hook_callback` next to `record_callback(ran)` (`src/hook/keyboard_hook_callback.rs:438-439`)
   through a pure `over_budget(ran)`. No `log::` in that file. Evidence for the cut: the owner's log at 252daef shows
   `longest_callback_us` of 100 us or more in 22 of 74 reported minutes, median 56 us, maximum 4246 us, with 2 bindings
   (`%LOCALAPPDATA%\key-switch-rs\daemon.log`, read only; the writer reports only minutes with a non-zero counter, so
   the sample is biased toward hiccups). The split tells an action's `SendInput`/`PostMessageW` from decision work.
3. No other product change. No new dependency (`Win32_UI_WindowsAndMessaging` is present, `Cargo.toml:45`).

Acceptance:
- Bench (counted apart, never a pass): R19, R19b, R19c and R19w have p99 <= 100 us on HEAD (median of 3 runs' p99,
  tester session). The cut states R19's first-run figure as its probe (CLAUDE.md: a new bound is probed first). R19e,
  R17f and R17p are reported as evidence. No existing row R1-R18 regresses under the D10/D14/L3/D13 rules, parent and
  HEAD, 3 runs each.
- Unit: `over_budget` at 99, 100 and 101 us; `counters::summary` names the new counter(s) and does not warn on them.
- Unit (scan): `logging::tests::the_functions_on_the_hook_path_log_nothing` and the `keyboard_hook_callback.rs`
  no-`log::` scan stay green with the new call.
- Review: the bench still installs no hook and injects nothing (`src/bench.rs:14-18` stays true).
- Manual (owner, later batch): after a week of daily use, read the counters lines of `daemon.log` for the new
  counter(s) next to `longest_callback_us`.
- Gate green (fmt, clippy -D warnings, test).

Latency: the bench changes nothing on the key path. The counter adds one `Duration` compare per callback on the
already computed `ran`, plus one atomic RMW only for a callback of 100 us or more. No allocation, lock, Win32 call,
timer or wake-up. CLAUDE.md principle 1 lists this compare among the sanctioned hook-path additions after the merge
(orchestrator chore).

Docs: CHANGELOG `[Unreleased]` Added (the rows and the counter, roadmap 3.5); `book/src/config/errors.md` (the counter
in the counters line, near :231); README latency notes only if the counter is mentioned there. For the orchestrator's
post-merge chore: the R19 family's gate lines for CLAUDE.md (bench rules) and `.claude/agents/tester.md` (p99 <= 100 us;
later lanes compare p50 to their parent by D10's R4 rule). Non-config pages (roadmap 3.5 status, F3/witness/FU-4/U1
closures, the field evidence) go to the phase book sync, listed in impl.md with file:line.
