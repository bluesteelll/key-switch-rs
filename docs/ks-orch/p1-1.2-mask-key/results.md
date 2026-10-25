# Results analysis: p1-1.2-mask-key / roadmap 1.2 (KS-12, rest of KS-05, KS-35)

## VERDICT: ACCEPT

**Summary**: The lane does what the brief's spec asks for. The engine decides `Decision::mask` purely: it is set when a suppressed event comes while Windows has an Alt or Win key down whose key-down the hook passed. The adapter sends vk 0xE8 down and up, tagged with the sentinel, outside the engine lock and before the hook returns. `block_original_combo: MaskOnly` ships opt-in and marked experimental, auto-blockers inherit their owners' `when:`, and the docs and CHANGELOG are in sync. Every claimed KS-ID has a red-first regression test, and the gate is green in my own run. What remains is the owner's manual checks M1-M9. Two of them can still overturn the design: if M3 (the Alt+Tab switcher with E8) fails, that is a RETHINK by prior agreement, and if M4/M5 (the v2 hypothesis) fail, `true` stays the default. Checkpoint-1 item 4 passes only with `MaskOnly` opted in.

Tree: `feat/mask-key` @ `0187cde`, merge-base = `master` = `ece1767`, 9 commits, `git status` clean.

## Quality gate (own run)
| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) - `analyst-fmt.txt` |
| `cargo clippy --all-targets --locked -- -D warnings` | OK. Re-run from a cold, separate target dir on a `git archive feat/mask-key` export, so cached results could not mask anything (`Finished ... in 19.29s`, exit 0, no warnings) |
| `cargo test --locked` | lib 470 passed / 0 failed / 2 ignored; `tests/ipc_roundtrip.rs` 6 passed; bins 0. **476 passed, 2 ignored** (`analyst-test.txt`). master `ece1767`: 438 lib + 6 = 444 listed, so +34 tests and none removed |
| Ignored list (listed, not run) | `data::condition::tests::foreground_context_names_the_focused_notepad` (pre-existing); `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream` (new, `manual: injects vk 0xE8 ... while Notepad has focus and check that it passes and nothing is typed and no menu opens`) |
| `cargo doc --no-deps` | 4 warnings, all of them also on master (same 4 lines: `load`/`reload` -> `ConfigError::Missing`, `set_enabled` -> `with_engine`, unclosed `raw`). None new |
| `mdbook build` (export, output in scratchpad) | OK. The new anchors `s15-5`, `s15-3`, `s15-4`, `s16-2`, `s18-6`, `s13-1`, `s12-4`, `s11-8` and the README anchors `#combo-syntax-the-keys-field` and `#triggers-trigger-field` resolve |
| Named acceptance tests | 40 of 40 named tests are `... ok` by name in `analyst-test.txt` (the S, L, E and H tests, the properties, the `keys` unit test, both Display tests, `readme_matching_examples_load`, `the_template_and_its_examples_have_no_duplicate_bindings`) |
| Trailers | 9/9 commits end with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` |
| Dependencies | `Cargo.toml`, `Cargo.lock` and `tests/` untouched |

## Acceptance criteria
| # | Criterion (roadmap 1.2 + brief) | Evidence | Status |
|---|---|---|---|
| 1 | Scenario tests show `mask = true` exactly when an event is suppressed after Alt or Win reached the OS, and false otherwise | true: `engine::scenarios::win_q_post_message_masks_the_swallowed_q_and_its_release`, `alt_then_shift_with_the_template_blocker_masks_the_swallowed_shift`, `right_alt_and_right_win_mask_a_swallowed_key`, `swallowed_repeats_and_key_up_while_win_is_held_are_masked`, `alt_held_before_the_hook_saw_it_masks_a_swallowed_key`, `win_held_through_remapping_on_masks_a_swallowed_key`, `win_held_across_a_foreground_change_masks_a_swallowed_key`, `sequence_and_chord_keys_swallowed_while_win_is_held_are_masked`. false: `a_swallowed_event_without_alt_or_win_in_windows_is_not_masked`, `a_swallowed_key_is_not_masked_when_the_alt_or_win_key_down_was_swallowed` (the modifier's own key-down suppressed), `a_passed_event_is_not_masked`, `a_hold_timer_never_masks` (+ the harness assert in `testing.rs` `advance_ms`). Oracle: `check_mask_rule` (`src/engine/testing.rs` `check_mask_of`) reads the modelled OS, not the engine's records, and runs in the 3 `engine::properties` tests. Unit: `engine::keys::tests::alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed`. Mutations M1, M6, M11 and M13 are killed | PASS |
| 2 | The opt-in v2 mode passes the modifier key-downs and sets mask on an exact match | `mask_only_blocker_lets_alt_and_shift_through_and_masks_the_exact_hotkey`, `mask_only_blocker_lets_ctrl_shift_t_reach_the_application`, `mask_only_blocker_does_not_mask_a_hotkey_pressed_with_another_key_held`, `mask_only_blocker_masks_only_the_first_event_after_the_match`, `alt_shift_x_fires_next_to_a_mask_only_blocker`, `remapping_off_drops_an_owed_mask`; config surface `config::loader::tests::block_original_combo_takes_true_false_and_mask_only`, `a_misspelled_block_original_combo_value_names_the_valid_ones`, `mask_only_on_a_hotkey_with_a_payload_key_blocks_it_as_true_and_warns`, `true_and_mask_only_on_one_hotkey_give_one_blocker_each`; property config `mask_only` (symmetry, nothing stuck). Mutations M2, M7, M8, M9 and M15 are killed | PASS |
| 3 | A blocker requested by a binding with `when: AppEquals("x.exe")` is inactive in other apps | `a_blocker_asked_for_by_a_binding_with_when_app_equals_is_inactive_in_other_apps` (y.exe passes, x.exe suppressed + masked), `blockers_act_where_any_owner_when_holds`, `the_blocker_takes_the_when_of_the_binding_that_asked_for_it`, `blockers_of_one_hotkey_combine_the_when_of_their_owners`, `a_hotkey_asked_for_by_true_and_mask_only_keeps_each_owners_when`; hot-path guard `engine::tests::a_blocker_when_condition_is_asked_only_once_its_hotkey_matches`. Mutations M3 and M12 are killed | PASS |
| 4 | On mask, the adapter injects vk 0xE8 down+up with `injected_sentinel()` via `SendInput` before it returns, outside the engine lock | `hook::keyboard_hook_callback::tests::menu_mask_inputs_are_vk_e8_down_then_up_tagged_with_the_sentinel` (M4), `carry_out_sends_the_menu_mask_only_when_the_decision_asks` (M5). Structure, read by me: `carry_out` runs after `decide_event` has returned (lock and snapshot dropped) and before `LRESULT(1)` / `CallNextHookEx` (`src/hook/keyboard_hook_callback.rs:339-349`); the mask is sent first in `carry_out` (`:202-213`). The injection itself: H3 `the_menu_mask_key_is_inserted_into_the_input_stream`, `#[ignore = "manual: ..."]` = M8 | PASS (injection manual) |
| 5 | Manual: Alt then Shift in Notepad does not open the menu; `Win+Q -> PostMessage` does not open Start | M1, M2 (cut.md "Manual checks", impl.md). Matches **Checkpoint 1 item 5** (`book/src/roadmap.md:309`). Also in the CHANGELOG Fixed KS-12 entry | MANUAL, listed |
| 6 | Manual (v2): `Ctrl+Shift+T` works and a bare Alt+Shift does not switch the language | M4, M5 (cut.md, impl.md, CHANGELOG Added entry, `book/src/config/matching.md` §15.5 steps 1-3). Matches **Checkpoint 1 item 4** (`roadmap.md:308`), **but only with `block_original_combo: MaskOnly` opted in**: with the template's `true`, Alt, Shift, Tab still swallows Shift (`shift_pressed_after_tab_was_released_is_still_swallowed`) and Ctrl, Shift, T still arrives as Ctrl+T (`ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing`) | MANUAL, listed (see note on item 4) |
| 7 | Update README, `default_config.ron`, the book config pages and CHANGELOG | All present (see Docs and hygiene) | PASS |

## Issues claimed closed
| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-12 | `engine::scenarios::win_q_post_message_masks_the_swallowed_q_and_its_release` (the audit's `Win+Q -> PostMessage(WM_CLOSE)`), `alt_then_shift_with_the_template_blocker_masks_the_swallowed_shift` (the audit's default-config Alt-then-Shift case) + 6 more true-cases and the property oracle | Yes: `red1.log` at `54e0673` = 11 failed (S1-S4, S8-S11, 3 properties), `expected the menu mask, got none` | Fixed for swallowed events. Not covered, and documented as such: a passing Tap/Hold/DoubleTap on lone Alt/Win (known-issues row 66) |
| KS-05 (rest) | `mask_only_blocker_lets_ctrl_shift_t_reach_the_application` (the audit's Ctrl+Shift+T -> Ctrl+T), `mask_only_blocker_lets_alt_and_shift_through_and_masks_the_exact_hotkey`, `alt_shift_x_fires_next_to_a_mask_only_blocker` | Yes: `red_a3e1149.log`, 10 failed (`Expected boolean`: the feature did not exist) | Opt-in and experimental, as the brief specifies. With the default `true`, the rest of KS-05 stays as before. The audit status should read "partly fixed; rest opt-in (`MaskOnly`), unverified" |
| KS-35 | `a_blocker_asked_for_by_a_binding_with_when_app_equals_is_inactive_in_other_apps`, `config::loader::tests::the_blocker_takes_the_when_of_the_binding_that_asked_for_it` | Yes: `red_047da43.log`, 6 failed (`left: Always right: AppEquals("x.exe")`; `expected the event to be passed, it was suppressed`) | Fixed for `when`; `trigger` not inherited, by design (D7, known-issues row 43) |

The red commits (`54e0673`, `44adabe`, `a3e1149`, `047da43`) each come before their green commit in `git log --oneline master..feat/mask-key`. `44adabe` is compile-red (`E0425 cannot find value MASKS_SENT` / `menu_mask_inputs`), and that is legitimate for a new API.

## Invariants
| Area | Status | Notes (file:line @0187cde) |
|---|---|---|
| Hook callback: passthrough, exactly one outcome | OK | `code < 0` and the remapping-off early returns are unchanged (`keyboard_hook_callback.rs:321-334`); the sentinel check comes before snapshot or lock (`:179`); every path ends in `LRESULT(1)` or `CallNextHookEx` (`:339-349`) |
| Hook callback: no new blocking or allocating work | OK (WARN noted) | `menu_mask_inputs` is a stack `[INPUT; 2]`; the engine adds a 5-entry constant scan (`keys.rs` `alt_or_win_down_in_windows`) and two bools. `SendInput` on the hook thread is what the spec prescribes (AutoHotkey's technique). Under the recursive model it runs other LL hooks for E8 inside our callback (cut R1), which is accepted. The new `eprintln!` runs only on a short count (`:256-261`), the same pattern as `press_vk`; item 1.7 has to replace it (its acceptance grep forces that) |
| Suppression symmetry (RC2) | OK | `mask` never feeds `suppress`; `MaskOnly` never sets `suppress` (`engine/mod.rs:451-465`); `check_press_symmetry` passes in every new scenario and in the `mask_only` property config |
| Engine purity | OK | No new `Env` call; the mask is computed from records the resync already keeps (`keys.rs:229-241`, `mod.rs:646-654`). `mask_owed` is plain engine data, cleared by `reset` |
| Matching per owner decisions | OK | `blocker_matches` and the election are unchanged; exact modifiers, `allow_extra_modifiers` and the repeat defaults are untouched; RON is kept (`RawBlockOriginalCombo` keeps `true`/`false` loading, pinned by L1) |
| Threads, sentinel, atomics, SAFETY | OK | No install, uninstall or thread added. Both E8 records carry `injected_sentinel()` (H1). No atomics, no `static mut`/`RefCell`/new `Mutex`; `MASKS_SENT` is a `cfg(test)` thread-local. Two new `unsafe` blocks (`:246-250`, test H1 `:554-556`), each with a concrete `// SAFETY:` |
| Test safety seam | OK | `#[cfg(test)] send_menu_mask` only counts (`:268-271`), so `cargo test` cannot inject E8 even if the owner holds Win/Alt. `tests/ipc_roundtrip.rs` never reaches `carry_out` |
| Config reload, IPC | OK | Unknown fields are still denied (`schema.rs` `deny_unknown_fields`); the new warning names `binding #N`; reload semantics are unchanged; IPC is untouched |
| No key logging | OK | Nothing new logs keys; the short-count warning prints only a count |

## Review status
- code-reviewer `review_r1.md`: **APPROVED** (Critical 0, Important 0, Minor 2). No `triage_r*.md` was needed. No CONFIRMED Critical/Important item is open.
- tester `test_r1.md`: **GREEN**. Red-first replayed on every red commit; 14 of 15 own mutations killed. M14 (mask after the actions instead of before) is not caught. That order is a design claim, not an acceptance bullet.
- architecture-critic `critique.md`: APPROVED (Minor 2). M1 is resolved in `0187cde`. M2 (Checkpoint item 4) is resolved by statement and carried here.
- Open Minor items worth doing while the context is fresh (orchestrator's call, not merge-blocking):
  - **m1** (`src/config/loader.rs:353-360` vs `:373-379`): the `MaskOnly` fallback `[WARN] ... blocked as with true` is pushed before the KS-18 skip, so it can state a blocker that is never added. The text is wrong; behaviour is not affected. A one-line move or rewording by the developer.
  - **m2**: `CLAUDE.md:26` and `.claude/agents/architect.md:37` still say "`mask` is reserved for roadmap 1.2". These are orchestrator-owned files; update them at merge so later architects do not plan to set `mask` from scratch.

## Docs and hygiene
- In this lane: CHANGELOG `[Unreleased]` has **Added** (`MaskOnly`, experimental, with its manual check), **Changed** "**Breaking:**" (KS-35 inheritance) and **Fixed** (KS-12), plus edits to stale entries. README, `src/config/default_config.ron`, and the book config pages `matching.md` (§15.1, §15.3 with a validated example, §15.4, new §15.5, §16.2), `structure.md`, `README.md`, `ron-syntax.md`, `errors.md`, `actions.md`, `triggers-conditions.md`, `known-issues.md` (rows 31 Fixed, 39, 43, 66) and `example.md` are all updated. Code links are pinned to `ad7fe86`, which resolves only after the merge push.
- Left for the phase book sync (WORKFLOW.md: the `doc-writer` book sync owns audit status lines, architecture and roadmap). These are recorded in impl.md except where marked **new**:
  - `book/src/audit.md`: status lines of KS-05 ("partly fixed; rest opt-in `MaskOnly`, unverified"), KS-12 (fixed for swallowed events; row 66 open) and KS-35 (fixed for `when`); the summary table rows at `:35`, `:9`.
  - `book/src/roadmap.md`: `:117` (mask reserved); `:308` Checkpoint item 4 -> "with `block_original_combo: MaskOnly`" until M4/M5 decide the default; `:616`, `:644` status column; `:670` open question 6 (superseded: v2 shipped opt-in for the manual check).
  - `book/src/architecture.md:268`, `:271`, `:544` and **`:323`** (new: "a `mask` flag reserved for item 1.2"; not in impl.md's list).
  - `book/src/build-and-run.md:415`, `:457-465`; `book/src/glossary.md:141`, `:313`.
  - `book/src/methodology.md:408-409`: the published `cfgcheck` harness formats `b.block_original_combo` with `{}`, which **no longer compiles** (`Option<BlockerMode>`); it must become `{:?}`. The book sync has to fix this first, because it validates RON examples with that harness.
- Until the book sync merges, Pages publishes these stale non-config pages next to an accurate config reference. This is the established phase pattern.
- Branch: cut from `master` (`ece1767` = merge-base), only this lane's work, clean to merge with `--no-ff`. If other Phase-1 lanes (1.4, 1.5, 1.7) merge first, expect `loader.rs`/README/CHANGELOG conflicts (cut "Merge conflicts"; keep both sides).

## Technical debt
- `book/src/config/known-issues.md` row 66: a passing `Tap`/`Hold`/`DoubleTap` on lone Alt/Win still opens the menu bar or Start. It is recorded in known-issues and the CHANGELOG, but **no roadmap item owns it** (the cut says "candidate 3.1"). The book sync should add it to roadmap 3.1 or to `gaps.md`.
- `engine/mod.rs:646-654`: a mask per swallowed event, so a swallowed auto-repeat under Alt/Win masks at the repeat rate (cut Q2). Recorded only in cut.md; it should be noted under roadmap 3.1/3.5 ("revisit with measurements").
- `keyboard_hook_callback.rs:222-240` `menu_mask_inputs`: a fourth `SendInput` builder that item 3.1's `build_inputs` absorbs. Recorded in cut.md only; add it to roadmap 3.1's file list.
- `keyboard_hook_callback.rs:256-261`: the new hook-thread `eprintln!` on a short count. Item 1.7's acceptance grep covers it.
- `carry_out` ordering (mask before actions) is not pinned by a test (tester M14), because observing it needs a test-only action seam. A candidate for 3.1, when the input module gives one.
- E8 is sent with scan code 0 and no extended flag (KS-36), so RDP/VM clients see it that way. Owned by 3.1.
- The v2 hypothesis and the cross-hook injection order are unverified. Recorded in the CHANGELOG, `matching.md` §15.5 and the cut; the M4/M5 outcome goes into the audit status.

## Problems
None blocking. The two Minor review items (m1, m2) and the book-sync list above are follow-ups, not REWORK: every acceptance bullet is met by the brief's own definition, and no CONFIRMED Critical/Important item is open.

## Manual checks for the owner (release build via `swch open`, `osk.exe` open)
| # | Setup | Steps | Expected | If it fails |
|---|---|---|---|---|
| M1 | Default config, hotkey Alt+Shift | Notepad: Alt down, Shift down, Shift up, Alt up; type `fx` | No menu bar; `fx` typed; layout unchanged | REWORK (KS-12 not fixed) |
| M2 | `( keys: "Win+Q", action: PostMessage(msg: "WM_CLOSE") )` | Win+Q on Notepad; release Win | Notepad closes; Start stays closed | REWORK (KS-12 not fixed) |
| M3 | Default config, three windows | Hold Alt, tap Tab, press Shift, tap Tab, release all | Switcher stays open and steps forward; no menu bar | **RETHINK** (pre-agreed): E8 in the switcher would make the default config worse |
| M4 | CapsLock with `block_original_combo: MaskOnly`, hotkey Ctrl+Shift | Browser: close a tab, Ctrl+Shift+T; bare Ctrl+Shift x5 | Tab reopens; layout never switches; CapsLock still switches | Hypothesis false: keep `true` (already the default), record it in the audit |
| M5 | Same, hotkey Alt+Shift | Bare Alt+Shift x5; Alt+Shift+Tab with 3 windows; Notepad Alt, Shift, release | No switch; switcher steps back; no menu bar | As M4 |
| M6 | `( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("notepad.exe") )` | Alt+Shift in Notepad, then elsewhere | Notepad: no switch, no menu; elsewhere: Windows switches | REWORK (KS-35) |
| M7 | Hotkey `` ` `` (value "4") with `MaskOnly` | Start the daemon | The `[WARN] binding #0: block_original_combo: MaskOnly needs ...` line; a lone `` ` `` is swallowed | REWORK (cosmetic if only the text) |
| M8 | Debug build, Notepad focused | `cargo test --lib -- --ignored the_menu_mask_key_is_inserted_into_the_input_stream` | Passes; nothing typed; no menu | REWORK (adapter) |
| M9 | After M1-M6 | Look at `osk.exe` | No modifier held | REWORK (stuck key) |

Checkpoint 1 item 5 = M1 + M2. Checkpoint 1 item 4 = M4 + M5, which pass **only with `MaskOnly` opted in**. With the template's `true`, item 4 fails by design until the owner decides to change the default.

## Positives
- The mask rule is computed from records the resync already maintains: no new Win32 call on the hook thread, and the engine stays pure. The property oracle checks it against the modelled OS, which makes it an independent second opinion.
- `carry_out` destructures `Decision` without `..`, so a future field fails to compile until someone carries it out.
- The `cfg(test)` seam makes it impossible for `cargo test` to inject E8 on the owner's machine, whatever keys the owner holds.
- D8 moves the blocker's `when:` after the hotkey match, which keeps the KS-35 fix from becoming a KS-08 regression (pinned by E1 with a foreground-forbidding Env).
- `RawBlockOriginalCombo` keeps existing `true`/`false` configs loading. The choice rests on reading the locked ron 0.10.1 source, and the error text is pinned.
- Clean red/green alternation that bisects, a mutation-tested suite (14/15 killed), and docs that land with the commits that change meaning.

## Notes for later lanes
- 3.1 (single input module): absorb `menu_mask_inputs`; add an action seam so the mask-before-actions order can be pinned (M14); decide once-per-press versus per-event masking with measurements; consider row 66 (re-inject a lone Alt/Win key-up after a mask); scan code and extended flag for E8 (KS-36).
- 2.2 (timers on the main thread): `on_timer` must keep returning `mask = false`. The harness assert in `advance_ms` enforces that.
- Feature 7 (more blocker hotkeys) builds on `BlockerMode`; `expand_with_auto_blockers` now groups by (hotkey, mode) and ORs the owners' conditions.
- 1.4 (tri-state hotkey) will meet `expand_with_auto_blockers` (with its `warnings` parameter) in `loader.rs`. A disabled hotkey just yields no request.
- 1.7 (logging): convert the `[WARN] SendInput inserted N of 2 menu mask key events` `eprintln!` on the hook thread into a counter.
