# Results analysis: p0-A4-autoblockers (Workstream A, "exact auto-blockers and dedup": KS-18, KS-05 exact-match part)

Branch `fix/auto-blockers` @ `3d65ac9`. It was cut from `master` @ `44f79ac`, which is still master HEAD and the merge-base. It has 6 commits, 13 files, +803/-49. The tree is clean and the branch is not pushed.

## VERDICT: ACCEPT

**Summary**: Every spec bullet and acceptance line is backed by an `Engine::on_key` scenario test or a registry-free loader test. I replayed both red commits myself: `cae1ec5` gives 4 failed, `8a08253` gives 14 failed, all with the predicted texts, and all pass on HEAD. The gate is green in my own run (307/307 against 286 on master). The design keeps the engine pure and leaves the hook adapter alone. It adds only bounded table reads and a few `GetAsyncKeyState` calls, under the existing lock, and only on a blocker's own key-downs. The hot path gets cheaper: the blocker no longer fires `DoNothing`. Review r1 is APPROVED with 0 Critical and 0 Important, and tester r1 is GREEN. Four non-blocking items remain for the orchestrator: the tester's Sonnet trailer, now for the third lane in a row with the root cause still open; two interpretation rulings the orchestrator never recorded (cut Q1, Q2); an owner FYI about the 1.2 residue; and the book sync.

## Quality gate (own run, HEAD 3d65ac9)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (exit 0) after touching the three crate roots, so it was a real re-check: "Checking key-switch-rs ... Finished" |
| `cargo test --locked` | **307 passed, 0 failed, 0 ignored** (lib). The `swch` bin and doc-tests run 0 tests. Log: `D:/tmp/ks-orch/p0-A4-autoblockers/analyst-test.txt`. Master `44f79ac`: 286 (A3 results). Audit `7cb3521`: 120 |
| `cargo test --locked -- --list --ignored` | 0 tests in every target. No `#[ignore]` added or removed |
| `#[test]` attributes, master vs branch | 277 -> 298 (+21). No `#[test]` removed (`git diff` has no `-` line with `#[test]`) |
| `cargo doc --no-deps` | 1 warning, `src/data/vk_name.rs:4` unclosed `<raw>`. It was already there before this lane |
| `mdbook build` | not run, because the lane does not touch `book/` |
| Trailer count | 5 of 6 commits carry `Co-Authored-By: Claude Opus 5.5`. `3d65ac9` (tester) carries `Claude Sonnet 5.5` (P1) |

**My own red replay.** I exported each red commit with `git archive` into the session scratchpad and gave each its own `CARGO_TARGET_DIR`. A shared target dir gave a stale binary for the second export, so I discarded that run and redid it.
- `cae1ec5` (master + the behaviour-neutral seam `91937a0` + tests): `288 passed; 4 failed`. The 4 are L1, L2, L3 (`left: 1 / right: 0`, `the file's bindings come first`) and T1 (`down LShift at +0 ms: expected to fire [SwitchLanguageBackward], fired [DoNothing]`, `context: flagged binding above`).
- `8a08253` (master engine + the loader fix + tests): 14 FAILED, exactly the set in `impl.md:15-22`. Examples: `down LShift at +500 ms: expected the event to be passed, it was suppressed` (T2); ``down ` at +0 ms: expected the event to be passed, it was suppressed`` (T5); `down RShift at +0 ms: expected to fire [], fired [DoNothing]` (T8); `expected the event to be suppressed, it was passed` (M3 pin).
- I checked that `91937a0` only threads the lookup through (`git show 91937a0`), so `cae1ec5` is master's behaviour.

## Acceptance criteria

| # | Criterion | Evidence | Status |
|---|---|---|---|
| S1 | A blocker fires only when the held set exactly equals its combo and the pressed key belongs to it | `engine::matching::tests::blocker_matches_only_what_windows_sees_as_its_hotkey` (`src/engine/matching.rs:241`): pressed `A` -> false, Alt only -> false, an extra Tab or LCtrl down in Windows -> false, grave with A or Shift -> false. `blocker_never_asks_windows_about_its_own_keys` (`:298`). Scenarios T2 `src/engine/scenarios.rs:1965`, T3 `:1997`, T4 `:2030`, T5 `:2051`; F1 second half `:1170` (passed LCtrl -> Shift passes). D2 reading: keys Windows does not have down do not count, T9 `:2198`, T10 `:2223`, M3 pin `:2247` | MET, under the D2 reading (cut Q2, see Notes) |
| S2 | Dedup checks the whole list, requester included; never shadows a user binding on the same combo; if one exists, the blocker is skipped | `config::loader::tests::auto_blocker_is_skipped_when_a_binding_uses_the_hotkey_keys_in_either_order` (`src/config/loader.rs:650`: above, below, Tap + `when:`), `..._when_the_flagged_binding_uses_the_hotkey_keys` (`:689`), `auto_blocker_stays_when_a_binding_only_overlaps_the_hotkey` (`:728`). No shadowing: T7 `:2135`, T8 `:2165`. Through the loader: T1 `:1915` | MET |
| S3 | Blockers stay distinguishable (flag or separate list) | The `pub(crate) is_auto_blocker` flag (`src/data/binding.rs:75`) is read by the engine branch (`src/engine/mod.rs:353`) and asserted in `auto_blockers_follow_the_bindings_of_the_file` (`loader.rs:700`), `auto_blocker_keys_come_from_the_hotkey_lookup` (`:608`) and L4 | MET |
| A1 | Alt+Shift+Tab and Ctrl+Shift+T pass through when the blocker exists (scenario tests) | T2 `shift_pressed_while_alt_and_tab_are_held_reaches_windows` (`:1965`, every event passes, repeats included). T3 `shift_pressed_while_ctrl_and_t_are_held_reaches_windows` (`:1997`, both sides). Third key in the modifiers-first order: T6 `ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing` (`:2082`, T passes) and `alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier` (`:1518`). Residue pinned: `shift_pressed_after_tab_was_released_is_still_swallowed` (`:2106`). Manual M3, M4 (`impl.md:60-61`) | MET for every order where the third key is down before the second modifier. The modifiers-first order (Ctrl, Shift, T -> Ctrl+T; Alt held, Tab released, Shift+Tab -> forward) is carved out to item 1.2 by `book/src/roadmap.md:100-101` and smoke item 4 "(A, 1.2)" (`:232`). It is unreachable with exact matching, because the decision is made at Shift's key-down. It is not a regression: master behaves the same |
| A2 | A user binding on Alt+Shift still fires with the blocker present, in either file order | T7 `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order` (`:2135`, blocker first and last, both with `check_press_symmetry`); T1 (`:1915`, both file orders and the flagged binding itself, through the loader); T8 (`:2165`, sided binding next to a kept blocker). Manual M2 | MET |
| A3 | Tests for loader dedup | L1 `:650`, L2 `:689`, L3 `:700`, L4 `:728`, C1 `:608`. Existing ones: `auto_blocker_deduped_when_two_bindings_want_the_same_combo` (`:582`), `auto_blocker_not_added_without_flag` (`:548`), `auto_blocker_not_added_for_non_system_actions` (`:563`). All registry-free through `from_ron_str_with_hotkeys`, except the three existing ones | MET |

The manual checks M1-M5 (`impl.md:58-62`) each have an automated twin. None needs an `#[ignore]` stub, because none of them is a test the suite could run. They go to the owner as a Checkpoint 1 smoke supplement, with smoke item 4 (`roadmap.md:232`).

## Issues claimed closed

| KS-ID | Regression test (audit scenario) | Fails on old code? | Status |
|---|---|---|---|
| KS-18 | T1 `user_binding_on_the_language_hotkey_fires_in_either_file_order` (`scenarios.rs:1915`) is the audit's repro: flagged CapsLock + `Alt+Shift` below (did nothing), swapped, and the flagged `Alt+Shift` alone (never fired). Plus L1/L2/L3 for the loader and T7/T8 for the "never shadows" half | Yes, in my replay of `cae1ec5` (T1, L1-L3) and `8a08253` (T7, T8) | CLOSED |
| KS-05 (partly; the exact-match part) | T2 (Alt+Shift+Tab, Tab held), T3 (Ctrl+Shift+T, T held), T4 (rollover), T5 (grave). The audit's modifiers-first orders stay with 1.2 and are pinned as residue (T6, `:2106`, F2) | Yes, in my replay of `8a08253`: T2-T5 fail on suppression | PARTLY CLOSED, as claimed. The CHANGELOG says "partly; roadmap item 1.2 fixes the rest" |

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | `src/hook/**` is untouched (`git diff --stat`). The new work runs inside `with_engine`: the blocker branch `src/engine/mod.rs:353-365` reads a 256-bool table (`keys.rs:204-210`, no allocation) and calls `GetAsyncKeyState` only for held keys outside the hotkey, only after `is_trigger_key` and `keys_down` pass (`matching.rs:98-101`, `\|\|` short-circuit). The hot path loses work: no `DoNothing` push or execute per blocker match. Symmetry: the record logic `mod.rs:473-506` and `on_key_up` are unchanged. Property tests `default` and `mixed` run the blocker in both modes and pass (`properties.rs:64`, `:113`) |
| Engine purity, matching per owner decisions | OK | Engine still `#![forbid(unsafe_code)]`. Win32 only through `Env::os_key_down` (contract updated in `env.rs:36-42`). Exact modifiers for user bindings are unchanged (`combo_matches`), and blockers stay exact (D2). Repeat defaults, RON and `println!` usage are unchanged. `VIRTUAL_KEY(code)` at `keys.rs:209` is built from the engine's own table index (0..256). The reviewer ruled it no principle-7 breach and I agree (Notes 5) |
| Thread ownership, sentinel, atomics, SAFETY comments | OK | No hook, timer, thread, atomic, `SendInput` or `unsafe` added. The existing `HookEnv::os_key_down` SAFETY note (`keyboard_hook_callback.rs:116-122`) covers the new callers |
| Config reload, IPC | OK | The loader is still all-or-nothing on errors. The `[INFO]` index is the 0-based file index, the same as the load errors (`loader.rs:160`, `:238`). The new `println!` runs on the main/`config-watcher` thread, never in the hook. IPC is untouched |
| No key logging | OK | `[INFO] no auto-blocker for Alt+Shift: binding #N uses the same keys` names the configured hotkey at load, not typed keys |

## Review status

- Critique: APPROVED, 0 Critical, 0 Important, 5 Minor. All 5 are RESOLVED in the tree (cut addendum, `cut.md:387-397`). I checked them: M3, the U1 LCtrl-up case (`matching.rs:261-268`) plus the scenario `:2247`; M4, the switcher order in the CHANGELOG KS-05 entry and the README row, pinned at `:2106`; M5, the **Breaking:** entry under Changed (`CHANGELOG.md:77-85`).
- Review r1: APPROVED, 0 Critical, 0 Important, 1 Minor (m1, a process item about the tester commit, see P1). No triage was needed.
- Tester r1: GREEN, with 8 mutations. The 8th (`0..255`) survived until the tester added `all_held_visits_the_highest_code_the_table_holds` (`keys.rs:276`, `3d65ac9`).
- Open Minor or NOTED items for the orchestrator: see Problems P1-P3 and Notes.

## Docs and hygiene

- In sync in this branch: `README.md:428` (`block_original_combo` row: exactness, dedup, no fire, listed last, 1.2 residue including the switcher order); `src/config/default_config.ron:126-130` (ASCII); `CHANGELOG.md` Unreleased: Changed **Breaking** (`:77-85`), Fixed KS-05 partly (`:96-106`), KS-18 (`:156-169`). The README's "listed last at startup" claim is true: `print_welcome` prints the pre-sort list (`src/core/app.rs:56-62`).
- `book/src/**`, `audit.md` status lines (KS-18 fixed, KS-05 partly) and `roadmap.md:80` are deferred to the phase book sync (`docs/WORKFLOW.md:104-107`), as in A2 and A3. The page list is in `cut.md:342`.
- No new dependency. `Cargo.toml` and `Cargo.lock` are untouched, and `--locked` passes. Every source line the branch adds is ASCII. No new TODO, FIXME, `unwrap()`, `SeqCst` or `unsafe`.
- The branch contains only this lane's work, and master has not moved since the cut, so `--no-ff` merges cleanly. Trailers: 5 of 6 (P1).

## Technical debt

Nothing new is marked TODO or FIXME in code. Accepted limitations and deferrals:
- The modifiers-first residue (KS-05 rest, KS-12) belongs to item 1.2. It is recorded in `roadmap.md:100-101`, smoke item 4, the README row and the CHANGELOG.
- Blockers do not inherit the requester's `when:` (KS-35). Recorded under roadmap 1.2.
- Dedup ignores `when:` and `trigger` (RK3). The CHANGELOG Breaking entry and the README record it. Book §15.4 is pending.
- RK5: an overlapping user binding with `block_default: false` (for example `LAlt+LShift`) still has its key swallowed by the blocker. **Recorded only in `cut.md`.** It needs a line in `book/src/config/known-issues.md` at the book sync.
- A binding on exactly the hotkey's keys with `block_default: false` now lets the Windows hotkey through, because the blocker is skipped. The loader doc says so (`loader.rs:181-186`). The README and CHANGELOG mention only `when:`. Book sync.
- RK6/Q7: whether Windows' language hotkey fires when a non-modifier key is down or is released in between (rollover, T4) has not been verified. The grave case is in the CHANGELOG. The research query is `cut.md:352`, for the 1.2 spike.
- Pre-existing, with no lane change: the existing loader tests `default_config_parses`, `parse_simple_binding` and `auto_blocker_deduped_when_two_bindings_want_the_same_combo` still read HKCU. The reviewer showed that they cannot fail for values 1-4. Moving them to `from_ron_str_with_hotkeys` is optional hygiene and is not recorded anywhere.
- Reviewer open question 5: Alt pressed again while a swallowed Shift is still held is swallowed too. This is pre-existing, belongs to 1.2, and is partly covered by the README's "still counts as held".

## Problems (none blocking)

### P1. `3d65ac9` carries `Co-Authored-By: Claude Sonnet 5.5`: third lane in a row, root cause still open
- **What**: the tester's `Tests:` commit ends with the Sonnet trailer. `CLAUDE.md` (Git flow) and `docs/WORKFLOW.md:234-236` require `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` on every commit.
- **Impact**: hygiene only. The content is test-only and correct: the test pins the last table slot and kills mutation M8. The trailer cannot be fixed without amending, which is forbidden. A revert plus a re-commit would leave the Sonnet commit in history anyway.
- **Root cause**: A2 results P1 and A3 results P2 both asked for the literal trailer string in the tester's instructions, and that was not done. `.claude/agents/tester.md:22` and `.claude/workflows/ks-lane.js:134` still say only "with the Co-Authored-By trailer", and the Sonnet tester writes its own model name. `docs/WORKFLOW.md:23` also still says the tester makes "no commits", which contradicts `tester.md:22` and `936b954`.
- **Return to**: none for this lane (orchestrator). Either put the literal Opus trailer into `tester.md:22` and the ks-lane tester brief, or sanction the Sonnet trailer for tester commits in CLAUDE.md. Fix `WORKFLOW.md:23` too.
- **Re-check condition**: the next lane has 100% compliant trailers, or the documented exception.

### P2. Cut Q1 and Q2 were never ruled on by the orchestrator
- **What**: the lane reads acceptance A1 with the roadmap's 1.2 carve-out (Q1), and spec S1's "held set" as "the keys Windows has down besides the hotkey" (D2, Q2). The critic, the reviewer and the tester all endorse both readings, and so do I. D2 is the reading that does not fail open after Win+L or a swallowed key (T9, T10), and the literal reading would be a regression against master. But `impl.md:97` and `pack_r1.md:190` record that nobody ruled on them.
- **Return to**: none (orchestrator). Record both rulings in the merge message or in `merge.md`.
- **Re-check condition**: the rulings are written down at merge.

### P3. Book sync backlog grows
- **What**: the pages in `cut.md:342`, plus RK5 and the `block_default: false` consequence above, the audit status lines (KS-18 "Fixed in <merge sha> (fix/auto-blockers)", KS-05 "Partly fixed ..."), and the stale agent rule texts `.claude/agents/code-reviewer.md:124` and `.claude/agents/project-analyst.md:60` ("auto-blockers rely on load order among equal counts"), which are no longer true after D3.
- **Return to**: `doc-writer` at the phase book sync; the orchestrator for the agent files.

## Positives

- D3, "a blocker only suppresses", removes the order dependence at its source. That closes KS-18 even for overlapping bindings (`LAlt+LShift`, `allow_extra_modifiers`), which equality dedup alone cannot reach. It also takes the per-match `DoNothing` execution off the hook thread.
- D2 decides by what Windows will actually see. It is immune to stale engine-held keys (T9) and to keys a binding swallowed (T10, M3 pin), and it never asks about the key being delivered, whose async state lags (U2 panics if it does).
- The D6 seam makes every new test independent of the machine's registry hotkey.
- The tests are strong. Every lane scenario carries `check_nothing_stuck`, and most carry `check_press_symmetry`. The tester's 8 mutations are each killed by a named test, and the flips F1-F3 are declared, not discovered.

## Notes for the orchestrator and later lanes

1. **Owner FYI (Russian in chat)**: after this merge, Ctrl, Shift, T with the Ctrl+Shift hotkey still reaches the app as Ctrl+T, and the usual Alt+Tab-back gesture (Alt held, Tab released, then Shift+Tab) still moves forward. What works now is Shift pressed while the third key is already held. Item 1.2 (the mask key) fixes the rest. Smoke item 4 stays half-open until then.
2. Run the manual checks M1-M5 (`impl.md:58-62`) with Checkpoint 1.
3. Item 1.2 should reuse `matching::blocker_matches` and hook the mask key into the blocker branch at `src/engine/mod.rs:353-365`. Run the cut's Q7 research (`cut.md:352`) first.
4. Item 1.4 (KS-17) must keep building hotkeys with `AnySide` modifiers. `blocker_matches` relies on that for the generic slots (doc at `matching.rs:81-91`, RK7).
5. CLAUDE.md principle 7 could name the table-index `VIRTUAL_KEY(code)` construction (`keys.rs:209`) as a sanctioned exception, so later critics do not re-raise it.
6. Optional pin: an AltGr+Shift scenario (fake LCtrl + RAlt, then Shift, passes) for the AltGr case KS-05 names. The same logic is already covered by F1's passed-LCtrl half.
