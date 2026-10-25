# Context pack r2: p0-A2-matching (round 2: I1 fix, tri-state key-down record, KS-06 docs narrowing)

Audience: tester and code-reviewer, round 2. Facts only; `UNVERIFIED` marks inference. Line numbers are on `fix/matching` `c8ca4a5` unless prefixed `master:` or another sha. Lane-wide facts unchanged since round 1 live in `D:/tmp/ks-orch/p0-A2-matching/pack_r1.md` (its line numbers are at `c609a69`; see "Stale r1 references" below).

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `fix/matching`, HEAD `c8ca4a5`, `git status --short` empty at read time.
- `master` = `e67e5cb` (merge-base, per triage_r1.md header).
- Round-2 commits, `git log --oneline c609a69..fix/matching` (newest first):
  - `c8ca4a5` Docs: the KS-06 entry and README name the focus-change and KS-14 gaps (README.md, CHANGELOG.md)
  - `938df54` Engine: a repeat whose swallowed key-down a focus change forgot follows its bindings (KS-06) (keys.rs, mod.rs)
  - `c59a110` Engine: red-first: a repeat after a focus change follows its bindings (KS-06) (scenarios.rs +112, tests only)
  - `965ac4d` Tests: deferred bindings follow the exact-modifier rule and allow_extra_modifiers (tester r1, scenarios.rs +44)
- `git diff --stat c609a69..fix/matching`: 5 files, +286 / -29

| File | +/- | Commit(s) |
|---|---|---|
| CHANGELOG.md | +7 | c8ca4a5 |
| README.md | 14 (+12/-2) | c8ca4a5 |
| src/engine/keys.rs | 96 | 938df54 |
| src/engine/mod.rs | 42 | 938df54 |
| src/engine/scenarios.rs | +156 | 965ac4d (+44), c59a110 (+112) |

- `git diff --stat 965ac4d..fix/matching`: same 5 files, +242 / -29 (scenarios.rs +112).
- Lane total `git diff --stat master...fix/matching`: 20 files, +2434 / -324 (r1 file list unchanged; no new file in round 2).
- Trailers: `c59a110`, `938df54`, `c8ca4a5` carry `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`; `965ac4d` carries `Claude Sonnet 5.5` (triage M2; no amend under the git flow).

## File map (round-2 delta)

| Range | What lives there | Why it matters this round |
|---|---|---|
| `src/engine/keys.rs:1-9` | module doc | rewritten: "whether each key's latest key-down was swallowed or passed" |
| `src/engine/keys.rs:41-51` | `enum LastDown { Unknown, Passed, Suppressed }` (private, `Copy`, `PartialEq`) | replaces the `blocked: [bool; 256]` flag |
| `src/engine/keys.rs:53-72` | `KeyState { active: [bool;256], last_down: [LastDown;256] }`, `new()` | all slots start `Unknown` |
| `src/engine/keys.rs:77-84` | `sync_modifiers` | writes `active` only; never touches `last_down` |
| `src/engine/keys.rs:88-100` | `press` (returns previous `active`, the repeat detector), `release` | unchanged; `was_repeat` source |
| `src/engine/keys.rs:102-147` | `mark_blocked`, `mark_passed` (new), `is_blocked`, `is_passed` (new), `take_blocked`, `clear_all_blocked` | the record API the decision reads |
| `src/engine/keys.rs:233-298` | unit tests `blocked_flags_are_independent` :234, `is_blocked_reads_without_clearing` :248, `clear_all_resets_every_flag` :260, new `clear_all_keeps_passed_key_downs` :270, new `latest_key_down_replaces_the_record_and_take_forgets_it` :281 | record semantics pinned at unit level |
| `src/engine/mod.rs:36-41` | module doc "State" bullet for `keys` | rewritten |
| `src/engine/mod.rs:115-124` | rule 5 (repeat / first key-down / focus change) | rewritten |
| `src/engine/mod.rs:298-305` | `on_foreground_changed`: `sync_modifiers(env, None)` then `clear_all_blocked()` | doc rewritten; body unchanged |
| `src/engine/mod.rs:307-463` | `on_key_down` | repeat tail `:431-461` rewritten |
| `src/engine/mod.rs:465-490` | `on_key_up` | unchanged; `take_blocked` at :484, :487 |
| `src/engine/scenarios.rs:1201-1243` | 2 tester tests (965ac4d) | deferred Tap + exact modifiers |
| `src/engine/scenarios.rs:1407-1517` | 5 round-2 scenarios (c59a110) | I1 red-first + guard |
| `src/engine/scenarios.rs:173-182` | `foreground_change_forgets_pending_key_up_suppression` (pre-lane) | pinned behaviour the fix preserves |
| `src/engine/scenarios.rs:731-748` | `chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` (pre-lane) | the other pre-existing focus-change pin |
| `src/engine/testing.rs:152-208` | harness `down`/`repeat`/`up`/`up_unseen`/`foreground_changed` | how the new scenarios drive the engine |
| `src/engine/testing.rs:314-337` | `key_event`: OS key state updated only when `!decision.suppress` | what `env.os_down` reflects in scenarios |
| `src/hook/keyboard_hook_callback.rs:243-260` | WinEvent `foreground_changed` -> `with_engine(|e| e.on_foreground_changed(&env))` (:259) | unchanged; production caller of the changed path; doc at :244-247 still says "forgets which key-ups were due to be swallowed" |
| `README.md:347-367` | "Behaviour rules" auto-repeat bullet + two gap sub-bullets | c8ca4a5 |
| `CHANGELOG.md:87-104` | Unreleased / Fixed / KS-06 entry, new sentences at :98-104 | c8ca4a5 |

## Facts

### Record API (`src/engine/keys.rs`)

1. `LastDown` has three variants: `Unknown` ("the key is up, or a foreground change forgot that its key-down was suppressed"), `Passed`, `Suppressed` (`keys.rs:41-51`).
2. `mark_blocked(vk)` sets `Suppressed`; `mark_passed(vk)` sets `Passed`; both ignore codes >= 256 via `slot` (`keys.rs:103-114`, `:36-39`).
3. `is_blocked` is true only for `Suppressed`; `is_passed` only for `Passed`; both read without changing the record (`keys.rs:118-125`).
4. `take_blocked` replaces the record with `Unknown` and returns whether it was `Suppressed`; a `Passed` record is also reset to `Unknown` (returns false) (`keys.rs:128-136`).
5. `clear_all_blocked` turns only `Suppressed` into `Unknown`; `Passed` survives (`keys.rs:140-146`).
6. `clear_all_blocked` is a 256-iteration loop; its only caller is `on_foreground_changed` (`mod.rs:304`), which the adapter runs from the WinEvent callback (`keyboard_hook_callback.rs:259`), not the keyboard callback.
7. No new allocation, lock, atomic, `unsafe` or Win32 call in the round-2 diff (diff read; `src/engine` keeps `#![forbid(unsafe_code)]` per fix_r1.md "Unsafe / atomics added").

### Callers of the record API (complete list, `rg` over `src`)

8. `mod.rs:304` `clear_all_blocked`; `mod.rs:404` `mark_blocked(key)` for each `last_step_keys` on sequence completion; `mod.rs:423` `mark_blocked(key)` for each chord key on chord completion; `mod.rs:446` `is_blocked`; `mod.rs:448` `is_passed`; `mod.rs:452` `mark_blocked(vk)`; `mod.rs:460` `mark_passed(vk)`; `mod.rs:484` and `:487` `take_blocked(vk)`. No caller outside `src/engine`.

### Decision path at HEAD (`src/engine/mod.rs`)

9. `on_key` runs `sync_modifiers(env, Some(event.vk))` before every key event (`mod.rs:273`).
10. `was_repeat = self.keys.press(vk)` (`mod.rs:309`): true iff `active[vk]` was already set.
11. Immediate combo arm sets `decision.suppress = true` when `block_default` (`mod.rs:333-341`), including on repeats; fires at most one Immediate action per event (`:334-337`).
12. Tap / Hold / DoubleTap / Sequence / Chord arms `continue` on `was_repeat` (`mod.rs:343-345`, `:355-357`, `:369-371`, `:385-387`, `:410-412`), so on a repeat only Immediate arms can set `decision.suppress`.
13. Repeat tail (`mod.rs:431-450`): record `Suppressed` -> `suppress = true`; else record `Passed` -> `suppress = false`; else (`Unknown`) `decision.suppress` keeps what the bindings set, and nothing is recorded.
14. First key-down tail: `decision.suppress` -> `mark_blocked(vk)` (`mod.rs:451-452`); else `mark_passed(vk)` (`mod.rs:453-460`). At c609a69 the else-branch was `take_blocked(vk)` (diff).
15. `on_key_up`: if a Tap fired or a Hold was pending, `take_blocked(vk)` and `suppress = true` (`mod.rs:481-485`); otherwise `suppress = take_blocked(vk)` (`mod.rs:486-487`). Every key-up therefore leaves the record `Unknown` (fact 4).
16. `on_foreground_changed` = `sync_modifiers(env, None)` + `clear_all_blocked()` (`mod.rs:302-305`); body unchanged in round 2.
17. Master's tail was only `if decision.suppress { self.keys.mark_blocked(vk.0); }` with no repeat branch (`master:src/engine/mod.rs:409-411`); master's Immediate arm also set `suppress` on repeats (`master:src/engine/mod.rs:311-319`).
18. Consequence of facts 13-15 for an Immediate-suppressed key held across a focus change: repeats are suppressed by the binding; the record stays `Unknown`; the key-up passes. Pinned by `scenarios.rs:1408`, `:1433`, `:1456` (`up ... expect_passed`). Master suppressed that key-up (fix_r1.md "Deviations from the cut"; triage probes p1/p5 assert master and fail at HEAD only at the `up` step, `fix_r1/probe_head.txt:62`, `:74`).
19. Consequence for a key whose first key-down passed and whose key-up is lost with no key-up seen: the `Passed` record survives `clear_all_blocked` (fact 5) and `active` stays set, so the next physical press is a repeat and passes whatever the bindings ask (fact 13). Triage p3/p3b at HEAD: `down L at +0 ms: expected the event to be suppressed, it was passed` (`fix_r1/probe_head.txt:17`, `:32`); same as 965ac4d; DEFERRED as KS-29 / roadmap 2.3 (triage_r1.md "I1(c)").
20. Deferred binding (Tap/Hold/DoubleTap, chord, sequence) whose key-down was suppressed, held across a focus change: record becomes `Unknown`, its arm `continue`s on repeats (fact 12), so repeats pass (fact 13); its key-up is suppressed if a Tap/Hold is still pending (fact 15). Stated as a pre-existing open point in fix_r1.md "Open points"; master and 965ac4d behave the same per fix_r1.md (not re-run here, UNVERIFIED by this scout). No scenario pins it.

### Harness facts used by the new scenarios (`src/engine/testing.rs`)

21. `repeat(key)` requires the key held in the harness and sends `KeyDirection::Down` labelled `repeat` (`testing.rs:163-168`).
22. `up_unseen(key)` clears the harness `held` and `env.os_down` slots without telling the engine (`testing.rs:182-196`); the engine's `active` stays set, so the next `down` is a repeat to the engine while labelled `down`.
23. `foreground_changed()` calls `engine.on_foreground_changed(&env)` (`testing.rs:199-208`).
24. `key_event` writes `env.os_down[vk]` only when the decision passes (`testing.rs:321-323`); `expect_fired` compares the last step's fires exactly (`testing.rs:283-294`); a step with no `expect_fired` call has its fires unchecked (no implicit check in `key_event`/`record`, `testing.rs:314-337`).

### Round-2 scenarios (`src/engine/scenarios.rs`)

25. `deferred_binding_does_not_arm_while_shift_is_held` (`:1202-1226`): `CapsLock` `Tap(200)`; with LShift held, CapsLock down/up pass, nothing fires; after LShift up, CapsLock down/up suppressed and `SwitchLanguage` fires on up.
26. `deferred_binding_arms_while_shift_is_held_with_allow_extra_modifiers` (`:1228-1243`): same with `allow_extra_modifiers: true`: CapsLock down suppressed, up suppressed, fires.
27. `immediate_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:1408-1430`): `F13 -> PressKey("Left")`; down suppressed+fires; focus change; two repeats suppressed and each fires `PressKey(VK_LEFT)`; up passed, nothing fired.
28. `capslock_held_across_a_focus_change_does_not_reach_windows` (`:1433-1453`): default-config pair (`CapsLock`, `Shift+CapsLock`); repeats suppressed; up passed; fires checked only on the first down.
29. `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_suppressed` (`:1456-1476`): `Win+Alt+T -> Launch(exe: "wt.exe")`; Win, Alt pass; T suppressed+fires; focus change; repeats suppressed; up T passed. Mirrors roadmap 2.3 acceptance sentence (`book/src/roadmap.md:304`).
30. `press_after_a_lost_key_up_and_a_focus_change_follows_its_binding` (`:1479-1494`): `F13 -> SwitchLanguage`; down suppressed+fires; `up_unseen`; focus change; +1000 ms `down F13` (a repeat to the engine, fact 22) suppressed and fires.
31. `repeat_of_a_key_whose_down_passed_is_passed_across_a_focus_change` (`:1497-1517`, guard): `CapsLock -> SwitchLanguage`; LCtrl down, CapsLock down passed (exact matching, Ctrl held); focus change; LCtrl up; two CapsLock repeats passed; up passed. Fires on the repeats are not asserted (fact 24); the Immediate arm matches once LCtrl is up (KS-10 interplay, r1 open point 3).
32. Pre-lane pins still green at HEAD: `foreground_change_forgets_pending_key_up_suppression` (`:173-182`, key-up after focus change passes), `chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` (`:731-748`, chord completion re-marks J via `mod.rs:423`).

### Docs (c8ca4a5, 938df54)

33. `README.md:356-367`: "does not leak repeats" (was "never leaks"), plus two gap bullets: focus change (roadmap 2.3) and pending Tap/Hold modifier, `"Ctrl+Shift"` + `Hold(1000)`, Ctrl released first (KS-14).
34. `CHANGELOG.md:98-104`: KS-06 Fixed entry extended with the same two gaps.
35. Rustdoc: `mod.rs:38-41` (State), `:115-124` (rule 5), `:298-301` (`on_foreground_changed`), `:432-445` and `:454-459` (comments), `keys.rs:1-3`, `:41-63`, `:102-139`.
36. Not edited in round 2: `src/hook/keyboard_hook_callback.rs:244-247` and `:24` (adapter doc of `foreground_changed`); `book/` (cut D12/Q2, phase book sync); `book/src/audit.md` KS-06 entry (`:238`).

### Gate at HEAD c8ca4a5 (run by this scout)

37. `cargo fmt --check`: exit 0. `cargo clippy --all-targets --locked -- -D warnings`: `Finished`, no warning output. `cargo test --locked --lib`: `running 263 tests` / `test result: ok. 263 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Full `cargo test --locked` (bins, doc-tests) not run here. 263 = 256 (965ac4d, test_r1.md:13) + 5 scenarios + 2 `keys.rs` unit tests.
38. `#[ignore` count in `src`: 0 at 965ac4d (test_r1.md:15); round-2 diff adds none.

## Tests: red-first and acceptance proofs

### Commands

- Gate: `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked` (full, tester).
- Round-2 red replay as run by this scout (scratch export, own target dir; never checkout in the lane tree):
  - `git -C D:/key-switch-rs/key-switch-rs archive c59a110 | tar -x -C <scratch>/c59`
  - in `<scratch>/c59`: `CARGO_TARGET_DIR=<scratch>/c59target cargo test --locked --lib across_a_focus_change` and `... --lib press_after_a_lost_key_up`
- fix_r1.md says each red test was run alone on 965ac4d plus the scenarios; the exact command line is not recorded there.
- HEAD green: `cargo test --locked --lib across_a_focus_change` (4 tests), `cargo test --locked --lib press_after_a_lost_key_up`, `cargo test --locked --lib engine::keys::tests` (unit tests).
- Panic text format: `scenario: <label>: expected the event to be <suppressed|passed>, it was <...>` with labels `{down|repeat|up} {key} at +N ms` (`testing.rs:325`; panic prefix `testing.rs:393`).

### Round-2 red-first table (replayed by this scout on c59a110; green on 938df54 per fix_r1.md and at HEAD per fact 37)

| Test (HEAD line) | Red on | Red text (verified) |
|---|---|---|
| `immediate_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:1408`) | c59a110 | `repeat F13 at +500 ms: expected the event to be suppressed, it was passed` |
| `capslock_held_across_a_focus_change_does_not_reach_windows` (`:1433`) | c59a110 | `repeat CapsLock at +500 ms: expected the event to be suppressed, it was passed` |
| `win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_suppressed` (`:1456`) | c59a110 | `repeat T at +500 ms: expected the event to be suppressed, it was passed` |
| `press_after_a_lost_key_up_and_a_focus_change_follows_its_binding` (`:1479`) | c59a110 | `down F13 at +1000 ms: expected the event to be suppressed, it was passed` |
| guard `repeat_of_a_key_whose_down_passed_is_passed_across_a_focus_change` (`:1497`) | (green on c59a110, verified) | - |

Scout replay output: `across_a_focus_change` -> `1 passed; 3 failed ... 257 filtered out`; `press_after_a_lost_key_up` -> `0 passed; 1 failed ... 260 filtered out`. On c59a110 the lib has 261 tests.

Mutation checks reported by fix_r1.md (run by the developer, reverted, not re-run here):
- M-a: `clear_all_blocked` also forgetting `Passed` -> fails the guard (`:1497`) and `clear_all_keeps_passed_key_downs` (`keys.rs:270`).
- M-b: an `Unknown` repeat re-marked as blocked (master's rule) -> fails `:1408`, `:1433`, `:1456` at their `up` step.

Tester tests 965ac4d (`:1202`, `:1229`): added green on c609a69 (test_r1.md "Tests added by the tester"); no red-first.

Unit tests (938df54): `clear_all_keeps_passed_key_downs` (`keys.rs:270-279`), `latest_key_down_replaces_the_record_and_take_forgets_it` (`keys.rs:281-298`).

### Brief acceptance line -> proof at HEAD (names verified present; r1 rows re-lined)

| Acceptance line | Scenario / test (HEAD line) |
|---|---|
| with CapsLock held, `a` passes and nothing fires | `key_pressed_while_an_immediate_binding_key_is_held_passes` (`:968`), `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` (`:986`) |
| CapsLock binding does not fire while Shift is held; does with `allow_extra_modifiers: true` | `capslock_binding_does_not_fire_while_shift_is_held` (`:1079`), `allow_extra_modifiers_lets_capslock_fire_while_shift_is_held` (`:1096`), deferred: `:1202`, `:1229`; loader `allow_extra_modifiers_true_is_parsed` (`loader.rs:1297`), `allow_extra_modifiers_defaults_to_false` (`loader.rs:1289`) |
| Ctrl+J does not fire on Ctrl+Shift+J | `ctrl_j_does_not_fire_on_ctrl_shift_j` (`:1108`), `allow_extra_modifiers_lets_ctrl_j_fire_on_ctrl_shift_j` (`:1124`) |
| LShift and RWin fire only for that side; Shift for both | `lshift_in_keys_fires_only_for_the_left_shift` (`:753`), `rwin_and_lwin_in_keys_fire_only_for_their_side` (`:776`), `unsided_shift_and_win_in_keys_fire_for_either_side` (`:811`), `lshift_double_tap_fires_only_for_the_left_shift` (`:842`) |
| Shift in a chord fires | `unsided_shift_in_a_chord_fires_for_either_side` (`:872`), `sided_shift_in_a_chord_is_only_its_side` (`:901`) |
| Shift+CapsLock fires ToggleCapsLock | `shift_capslock_fires_toggle_capslock_with_either_shift` (`:1173`), `more_specific_immediate_binding_wins` (`:100`) |
| sided rule in sequence steps | `sided_shift_in_a_sequence_step_matches_only_its_side` (`:913`), `win_in_the_last_sequence_step_does_not_swallow_its_key_up` (`:945`) |
| deferred arms keyed on the trigger key | `deferred_combo_arms_only_on_its_trigger_key` (`:1046`) |
| README / default_config / CHANGELOG | `default_config_parses` (`loader.rs:478`), `readme_matching_examples_load` (`loader.rs:1312`) |
| KS-06 repeat rule (lane addition) | `:1248`, `:1275`, `:1296`, `:1327`, `:1352`, `:1382` (r1 C7a-e, C8) and round-2 `:1408`-`:1497` |

Other loader tests unchanged since r1: `chord_keys_that_overlap_are_rejected` (`loader.rs:1267`), `allow_extra_modifiers_is_only_for_keys_bindings` (`loader.rs:1339`).

## Open points from triage_r1.md (status after fix_r1.md)

| Item | Triage verdict | fix_r1.md status | Where |
|---|---|---|---|
| I1 (a)(b): repeat after focus change passed and fired (regression vs master) | CONFIRMED | RESOLVED by tri-state record | `keys.rs:41-147`, `mod.rs:431-461`; tests `:1408`, `:1433`, `:1456`, `:1479`, `:1497` |
| I1 doc claims (README "never leaks", CHANGELOG KS-06, rule 5, cut.md addendum) | CONFIRMED | RESOLVED in README/CHANGELOG/rustdoc; `cut.md` not edited (fix_r1.md names itself the correction) | `README.md:356-367`, `CHANGELOG.md:87-104`, `mod.rs:115-124` |
| I1(c): lost key-up then combo (KS-29) | DEFERRED-OUT-OF-SCOPE (roadmap 2.3) | unchanged; p3/p3b same as 965ac4d | fact 19 |
| M1: pending Tap/Hold swallows a modifier key-up (`"Ctrl+Shift"` + `Hold(1000)`, Ctrl first) | NOTED, pre-existing KS-14 | docs narrowed only; not pinned (scenarios header excludes known defects); p4 unchanged `up LShift at +800 ms: ... suppressed` | `mod.rs:454-459`, `README.md:363-367`, `CHANGELOG.md:102-104`, `fix_r1/probe_head.txt:47` |
| M2: 965ac4d trailer `Claude Sonnet 5.5` | NOTED | no action possible (no amend); round-2 commits carry Opus trailer | `git log -1 965ac4d` |
| T-note: trigger rule stricter than brief wording (CapsLock-then-Shift order) | NOTED, not a defect (cut D2/Q1) | REFUTED as defect; owner confirmation is an orchestrator question | `src/engine/matching.rs:46`, `:1173` |
| Fix shape question (one bit cannot tell "no record" from "passed") | architect/orchestrator call | answered by `LastDown` tri-state; behaviour change: Unknown-record repeat leaves no mark, key-up passes | fact 13, 18 |

Additional open points stated in fix_r1.md "Open points":
- Deferred-binding key held across a focus change passes its repeats; a pending Tap/Hold still swallows its key-up (fact 20). Pre-existing; roadmap 2.3.
- Pre-existing rustdoc warning `src/data/vk_name.rs:4`, untouched.

Triage probe sources (for reproduction, scratch only, not in repo): `D:/tmp/ks-orch/p0-A2-matching/triage/probe.rs` (p1-p4), `triage/probe5.rs` (p5); they assert master's behaviour; HEAD output `fix_r1/probe_head.txt` (`1 passed; 5 failed`, line 95). The triage notes a shared-target-dir artefact: use a separate `CARGO_TARGET_DIR` per exported tree and confirm the filtered-out count.

## Invariants in play

| Item | Where | Fact |
|---|---|---|
| R1 hook callback | `mod.rs:431-461`, `keys.rs:103-136` | O(1) array reads/writes per key event; vk bounded by `slot` (`keys.rs:36-39`); no allocation added (fact 7) |
| R1 (WinEvent side) | `mod.rs:302-305`, `keys.rs:140-146`, `keyboard_hook_callback.rs:259` | 256-entry loop on focus change, not on the keyboard path (fact 6) |
| R2 sentinel | none in round-2 diff | no `SendInput` touched |
| R3 shared state | `keyboard_hook_callback.rs:259` `with_engine` | no new static/lock |
| R4 orderings | none | no atomics added |
| R5-R7, R9, R10 | not touched in round 2 | see pack_r1.md |
| R8 key identity | `keys.rs:155+` `is_down` | unchanged in round 2 |
| KS-06 (suppression symmetry) | `mod.rs:431-461`, `:481-488` | facts 13-18 |
| KS-10 | `mod.rs:333-341` | repeats still fire Immediate actions, including on the `Passed` path (scenario `:1497` does not assert fires) |
| KS-14 | `keys.rs:77-84`, `mod.rs:454-459` | resync writes `active` only; M1 docs narrowing |
| KS-29 | fact 19 | lost key-up with `Passed` record; deferred to roadmap 2.3 |
| Roadmap 2.3 | `book/src/roadmap.md:292-304` | items: `reset_transient_state`, "foreground_changed no longer clears every blocked bit"; acceptance "Holding Win+Alt+T ... keeps the T repeats suppressed" (`:304`), pinned for Immediate by `:1456` |

## Stale r1 references (pack_r1.md, lines at c609a69)

- `src/engine/mod.rs`: r1 `303-445` (on_key_down) is now `307-463`; r1 `427-443` (repeat tail) is now `431-461`; r1 `447-471` (on_key_up) is now `465-490`; r1 `329-333` (Immediate arm) is now `333-341`; r1 `314` (condition evaluate) is now `318`; r1 `319-327` is now `323-331`.
- `src/engine/keys.rs`: r1 `95-97`, `117-129`, `138`, `164`, `185`, `211` shifted; HEAD: `is_blocked` `:118`, `is_down` `:155`, tests `:175`, `:201`, `:222`, `:248`.
- `src/engine/scenarios.rs`: r1 rows at or below `:1173` unchanged; r1 `:1204`..`:1338` are now `:1248`..`:1382` (+44 from 965ac4d).
- All other files: no change since c609a69, r1 line numbers hold.

## Not covered

- Did not re-run the developer's two mutations (M-a, M-b) or any tester r1 mutation; did not run the full `cargo test --locked` (bins, doc-tests) or `cargo doc`.
- Did not re-run triage probes on HEAD; probe results are from `fix_r1/probe_head.txt`.
- Did not verify fact 20 (deferred binding across focus change) by a run; it is derived from code and fix_r1.md.
- Did not read `src/engine/gestures.rs`, `chords.rs`, `sequences.rs`, `matching.rs` bodies in round 2 (unchanged since c609a69; see pack_r1.md).
- Did not read `cut.md`, `impl.md`, `critique.md`, `review_r1.md` beyond headings, nor `book/src/audit.md` KS-06/KS-14/KS-29 bodies.
- Did not check the README sections outside `:347-367` or CHANGELOG outside `:78-110` for round 2 (unchanged since c609a69 per diff).
