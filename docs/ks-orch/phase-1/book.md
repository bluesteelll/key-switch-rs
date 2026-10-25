# Documentation update: book sync after roadmap Phase 1
Verdict: DOCS READY WITH GAPS
Pinned commit: 20af28d (content); merged into master as f7f3a68 (docs commit 37453b8)

Gaps behind the verdict, none of them a defect in the published book:
- mermaid diagrams were not rendered locally (no mermaid CLI); syntax follows the book's rules and the HTML build succeeded;
- the mdbook build prints one environment warning (`mdbook-mermaid` built against mdbook 0.5.3, local mdbook is 0.5.2), no content warning;
- three hand-off items touch `CHANGELOG.md`, which this brief does not open to the doc-writer; they are listed under Suggested follow-up;
- two owner questions are recorded, not decided (see Owner questions).

## Pages changed
- `book/src/audit.md` - header status at 20af28d (18 fixed, 2 partly, 6 open), summary table rows KS-05/10/12/13/17/20, status lines KS-05, KS-10, KS-12, KS-13, KS-17, KS-19, KS-20, KS-29, KS-33, KS-35, KS-40. Findings untouched.
- `book/src/roadmap.md` - progress line; Phase 0 notes that went stale (mask, KS-05 warning, LShift example, manual checks 6 and 7); Phase 1 "Status: done" line and a status block with acceptance evidence for each of 1.1 to 1.7; Checkpoint 1 items 4, 9 and new 10; "Manual checks after Phase 1" (18 rows); known-issues warning; status notes on 2.2, 2.3, 3.1, 3.5, 4.2; syntax table; infrastructure rows; issue coverage; open question 6 overtaken, new questions 9 and 10; new section "Follow-ups from Phase 1".
- `book/src/architecture.md` - re-described and re-pinned to 20af28d: crate layout (logging, single_instance, known_folder, winner), startup sequence (logger, guard, log file, pipe before hooks, serve after install), thread model (log-writer), key-event path (one winner, gesture group, repeat, block_default on deferred triggers, MaskOnly, menu mask, carry-out order), data structures (LOGGER, COUNTERS, mask_owed), DoubleTap note, conditions after the match, tri-state hotkey, four SendInput builders, loader steps (defaults, duplicate warning, blocker when), strengths, root-cause table.
- `book/src/overview.md` - top half re-described at 20af28d: features (single instance, logging rows), default bindings (KS-17), warning, binaries, mermaid, changes table (+ efea317 and the seven Phase 1 merges), build/test/lint numbers measured at 20af28d, line counts; audit-snapshot tense fixes.
- `book/src/build-and-run.md` - single-instance paragraph, startup flowchart, console-line table (already-running, hotkey not assigned, duplicate warning, MaskOnly warning, FATAL pipe, IPC thread FATAL), stopping, pipe bullet, `open` bullet, autostart NOTE, test counts per module (537), clippy line, troubleshooting rewritten (KS-20 fixed, KS-12 cause, new rows), anchors of old sections kept.
- `book/src/glossary.md` - facts at 20af28d; entries Auto-blocker, block_default, block_original_combo, Console control handler, Daemon, Decision, Default config, Deferred trigger, Gesture, Hook chain, Immediate trigger, Language hotkey, Menu-mask key, Sentinel, Specificity, Suppression, Typematic auto-repeat, Validation harness, Binding (four flags) rewritten; new entries Counters, Gesture group, Log file and log-writer thread, MaskOnly, repeat, Single-instance guard, Winner.
- `book/src/gaps.md` - status at 20af28d; Layout Hotkey status (side effect, Hotkey vs Language Hotkey, other admin account); conflict detection partly closed; re-entry counter note; new gaps rows (block_default ignored on sequence/chord, spawn_sequence panic); drift rows 182 and 185; file logging and single-instance rows done with merge commits; test-coverage note.
- `book/src/methodology.md` - harness output note re-run at 20af28d; new section "The book sync after Phase 1"; live-checks pointer to Phase 1 manual checks.
- `book/src/README.md` - intro, current state, warning and dates note at 20af28d.
- `book/src/config/*.md` - facts statements at 20af28d (10 pages); permalinks re-pinned; actions.md §13.9 precedence sentence (N1); known-issues: heading/status at 20af28d, merge commits in Where cells, rows 63/66/67/73 point to the roadmap, row 68 widened (Hold/DoubleTap output under a passed modifier), new row 74 (cross-press DoubleTap); matching §15.2 links show the tri-state parser, "console and daemon.log"; structure "logged" wording; kinds §8.3 duplicate warning; triggers §12.4 links.

## SUMMARY.md
Unchanged (no new pages).

## Hand-off table
| Lane | Item | File:line (after the sync) | Disposition |
|---|---|---|---|
| 1.1 | Reviewer M2: CHANGELOG.md:242 reflow | CHANGELOG.md | Not done: CHANGELOG is the developer's file and this brief does not open it to the doc-writer; follow-up |
| 1.1 | KS-29 corner vs roadmap 2.3 (ruling premise) | roadmap.md:788 (Q9), roadmap.md:493 (2.3 note), audit.md:624, known-issues.md:131 | Done in 37453b8, as an owner question; not decided |
| 1.1 | Critic note for 1.6 (focus change hands repeats to another binding) | roadmap.md:791 follow-ups (winner per press row) | Done in 37453b8 |
| 1.1 | Critic note for 2.3 (OS key state tells lost key-up from repeat) | roadmap.md:791 follow-ups, roadmap.md:788 | Done in 37453b8, labelled unverified |
| 1.1 | `check_repeat_fire` harness gap, for 3.4 | roadmap.md:791 follow-ups | Done in 37453b8 |
| 1.1 | scenarios.rs comment cross-reference | src/engine/scenarios.rs | Not applicable: code, developer |
| 1.1 | WORKFLOW.md:23 tester rule | docs/WORKFLOW.md | Not applicable: orchestrator chore |
| 1.1/1.3 | overview.md:23, architecture.md:207 pre-1.1 repeat text | overview.md:23, architecture.md:232 | Done in 37453b8 |
| 1.2 | audit KS-05, KS-12, KS-35 status and summary rows | audit.md:28-36, :229, :379, :672 | Done in 37453b8 |
| 1.2 | roadmap :117 mask reserved | roadmap.md:117 | Done in 37453b8 |
| 1.2 | roadmap Checkpoint item 4 "with MaskOnly" | roadmap.md:381 | Done in 37453b8 |
| 1.2 | roadmap :616, :644 coverage | roadmap.md:707, :718 tables | Done in 37453b8 |
| 1.2 | roadmap :670 open question 6 superseded | roadmap.md:785 | Done in 37453b8 (marked "overtaken", question restated for the owner) |
| 1.2 | architecture :268, :271, :544, :323 | architecture.md:292, :295, :583, :352 | Done in 37453b8 |
| 1.2 | build-and-run :415, :457-465 | build-and-run.md:435-481 | Done in 37453b8 |
| 1.2 | glossary :141, :313 | glossary.md:143, :327 | Done in 37453b8 |
| 1.2 | methodology.md:408-409 harness `{:?}` | methodology.md:420 | Already done by the 1.7 lane; rebuilt and run here, it compiles and prints the documented output |
| 1.2 | Row 66 has no roadmap owner | roadmap.md:791 follow-ups, known-issues.md:134 | Done in 37453b8 (candidate 3.1) |
| 1.2 | Mask per swallowed event at the repeat rate, 3.1/3.5 | roadmap.md:791 follow-ups | Done in 37453b8 |
| 1.2 | `menu_mask_inputs` fourth builder, 3.1 file list | roadmap.md:531, architecture.md:449 | Done in 37453b8 (status note, item text kept) |
| 1.2 | Mask-before-actions order unpinned (M14) | roadmap.md:791 follow-ups | Done in 37453b8 |
| 1.2 | E8 scan code 0 (KS-36) | roadmap.md:791 follow-ups, :531 | Done in 37453b8 |
| 1.2 | Hook-thread eprintln on short mask count | src/hook/keyboard_hook_callback.rs | Already done by the 1.7 lane (counter `menu_mask_short`) |
| 1.2 | v2 hypothesis unverified, M4/M5 into audit | audit.md:229 | Done in 37453b8 (status says not yet run) |
| 1.2 | m1 MaskOnly warning before the KS-18 skip | src/config/loader.rs:353-379 | Not applicable: code defect for the developer; follow-up |
| 1.2 | m2 CLAUDE.md / architect.md "mask reserved" | CLAUDE.md | Not applicable: orchestrator files |
| 1.2 | 2.2 note: on_timer keeps mask false | roadmap.md:473 | Done in 37453b8 |
| 1.3 | architecture :207-208, :238 always suppress | architecture.md:233, mermaid | Done in 37453b8 |
| 1.3 | glossary :83-85 block_default | glossary.md:83 | Done in 37453b8 |
| 1.3 | overview :23, :41 | overview.md:23, :41 | Done in 37453b8 |
| 1.3 | build-and-run :418 "remove until 1.3" | build-and-run.md:440 table | Done in 37453b8 (row replaced by the `<T>` case) |
| 1.3 | roadmap :151 (B "What") | roadmap.md:151 | Not applicable: the plan text of B is kept as written; the status line :158 says what changed |
| 1.3 | roadmap :158, :199, 1.3 status, :594 | roadmap.md:158, :199, :287, :619 | Done in 37453b8 |
| 1.3 | gaps.md:182 README says it swallows the left Shift | gaps.md:190 | Done in 37453b8 |
| 1.3 | I1 interrupt rule (owner) | roadmap.md:789 (Q10), known-issues.md:135 | Done in 37453b8, as an owner question; not decided |
| 1.3 | P1 lone Alt/Win attributed to 1.2 | known-issues.md:134, README.md:456 | Already done by the 1.2 lane (row 66 and README relabelled); the CHANGELOG sub-bullet "A Tap on Alt or Win" (CHANGELOG.md:272) still says Tap only: follow-up for the developer |
| 1.3 | P2 Breaking entry: modifier-only Hold output | known-issues.md:136 (row 68 widened), audit.md:484, roadmap.md:791 | Book part done in 37453b8; the CHANGELOG Breaking sub-bullet is a follow-up for the developer |
| 1.3 | F2 block_default ignored on sequence/chord | gaps.md:165 | Done in 37453b8 |
| 1.3 | F6 code-reviewer.md R8 text | .claude/agents | Not applicable: orchestrator chore |
| 1.3 | Note 4: 1.6 acceptance with a deferred winner | roadmap.md:345 table | Done in 37453b8 (deferred winner test cited) |
| 1.3 | Note 5: KS-19 regression case for 3.1 | roadmap.md:791 follow-ups | Done in 37453b8 |
| 1.3 | Note 6: Checkpoint smoke #6 and M1-M3 | roadmap.md:389 manual checks 9, 10 | Done in 37453b8 |
| 1.4 | audit :40, :453 | audit.md:28-36, :454 | Done in 37453b8 |
| 1.4 | roadmap :621 KS-17 open | roadmap.md:707 table | Done in 37453b8 |
| 1.4 | glossary :287 | glossary.md:293 | Done in 37453b8 |
| 1.4 | overview :39 | overview.md:41 | Done in 37453b8 |
| 1.4 | architecture :548 (and :409) | architecture.md:583, :441 | Done in 37453b8 |
| 1.4 | build-and-run :119 hotkey lines | build-and-run.md:127 | Done in 37453b8 |
| 1.4 | gaps :124 | gaps.md:126 | Done in 37453b8 |
| 1.4 | matching.md:46 permalinks showed `"3" => None` | config/matching.md:46 | Done in 37453b8 |
| 1.4 | Q1 which value Windows honours | gaps.md:126 block; roadmap manual check 11 records the values | Done in 37453b8 |
| 1.4 | F1 other admin account reads its own HKCU | gaps.md:126 block | Done in 37453b8 |
| 1.4 | Layout Hotkey side effect sentence | gaps.md:126 block | Done in 37453b8 (from code, not observed) |
| 1.4 | Manual checks M1-M3 | roadmap.md:389 check 11 | Done in 37453b8 (M2/M3 regressions not repeated as rows) |
| 1.5 | build-and-run :309 WARNING | build-and-run.md:341 | Done in 37453b8 (NOTE) |
| 1.5 | build-and-run :125, :441 removed lines; :401-414; :435-449 | build-and-run.md:127-160, :435-500 | Done in 37453b8 |
| 1.5 | build-and-run :242 | build-and-run.md:242 area | Not applicable: the on/off paragraph there is still accurate |
| 1.5 | architecture :99, :161, :549 | architecture.md:109-112, :183, :584 | Done in 37453b8 |
| 1.5 | audit :43, :499 | audit.md:28-36, :499 | Done in 37453b8 (KS-20 Fixed) |
| 1.5 | roadmap :220, :271-279, :624 | roadmap.md:224, :327, :707 | Done in 37453b8 |
| 1.5 | overview :26, :100, :212 | overview.md:27, changes table, :227 | Done in 37453b8 (:100 row is history and stays "partly") |
| 1.5 | glossary :221, :389 | glossary.md:227, :407 | Done in 37453b8 |
| 1.5 | gaps :225 | gaps.md:235 | Done in 37453b8 |
| 1.5 | Checkpoint item 9 "exits with code 2", pipe-held check | roadmap.md:381-387 (items 9, 10), manual checks 12, 13 | Done in 37453b8 |
| 1.5 | Note for 1.7 and 4.2: re-run M1/M1b | roadmap.md:582 (4.2 note) | 1.7: already done by the 1.7 lane (its M3); 4.2: done in 37453b8 |
| 1.5 | 1.7 acceptance rg must allow the already-running line | roadmap.md:364 | Not applicable: 1.7 made that line a log record, so the rg finds only the banner |
| 1.5 | F2 swch open waits / exit 2; F3 private namespace | roadmap.md:791 follow-ups | Done in 37453b8 |
| 1.5 | console_ctrl_handler unsafe without SAFETY | src/core/app.rs:248 | Not applicable: code, next lane touching app.rs |
| 1.5 | CLAUDE.md Layout single_instance.rs | CLAUDE.md | Not applicable: orchestrator |
| 1.6 | audit :36, :394 | audit.md:28-36, :394 | Done in 37453b8 |
| 1.6 | roadmap :283-289, :617 | roadmap.md:345, :707 | Done in 37453b8 |
| 1.6 | architecture :207, :212, :543 | architecture.md:228-233, :578 | Done in 37453b8 |
| 1.6 | glossary :85, :401 | glossary.md:83, :423, new Winner :519 | Done in 37453b8 |
| 1.6 | gaps :185 | gaps.md:193 | Done in 37453b8 |
| 1.6 | overview :17 | overview.md:23-25 | Done in 37453b8 |
| 1.6 | N1 actions.md §13.9 sentence | config/actions.md:257 | Done in 37453b8 |
| 1.6 | N2 Phase 0 manual check 7 | roadmap.md:200 | Done in 37453b8 |
| 1.6 | N3 cross-press DoubleTap | known-issues.md:142 (row 74), architecture.md:367, roadmap.md:791 | Done in 37453b8 |
| 1.6 | kinds.md:91 mention the warning | config/kinds.md:91 | Done in 37453b8 |
| 1.6 | Immediate shadowing a deferred group (row 69) | roadmap.md:791 follow-ups | Done in 37453b8 (row 69 already pointed to 3.4) |
| 1.6 | Re-election per repeat (row 73) | known-issues.md:141, roadmap.md:791 | Done in 37453b8 |
| 1.6 | Review m1-m3 | src/engine/winner.rs, mod.rs, loader.rs | Not applicable: code; m3 is partly open (parse_ron_str still logs "config contains no bindings") |
| 1.6 | M1-M5 into Phase 1 manual checks | roadmap.md:389 checks 14, 15 and check 7 of Phase 0 | Done in 37453b8 (M2/M3 not repeated as rows) |
| 1.7 | 1. roadmap 1.7 status block, smoke #8 | roadmap.md:364 | Done in 37453b8 |
| 1.7 | 2. gaps.md:222 merge commit | gaps.md:230 | Done in 37453b8 |
| 1.7 | 3. audit KS-33 status wording | audit.md:656 | Done in 37453b8 |
| 1.7 | 4. architecture startup steps, build-and-run IPC warn row | architecture.md:101-117, build-and-run.md:127-160 | Done in 37453b8 |
| 1.7 | 5. structure/matching "printed" | config/structure.md:38, :91, :104, config/matching.md:55 | Done in 37453b8 |
| 1.7 | 6. re-pin 1dfd80a and 0dc008e links | all descriptive pages | Done in 37453b8 |
| 1.7 | 7. gaps rows spawn_sequence panic, re-entry counter | gaps.md:166, :164 | Done in 37453b8 |
| 1.7 | Orchestrator items 1-11 (CLAUDE.md, agent files, manual checks, m1/m2, SAFETY) | CLAUDE.md, .claude/ | Not applicable: orchestrator or developer |
| 1.7 | Risk 8 (High label on files of the elevated daemon) | - | Not applicable: technical debt in the lane report, not handed to the sync; worth adding to gaps when M1 of 1.7 is run |

## Owner questions
1. Roadmap open question 9 (roadmap.md:788): a press after a lost key-up of a swallowed, non-repeating trigger key fires nothing since item 1.1; item 2.3 as written does not repair it. Add a 2.3 acceptance line, or accept the behaviour as documented. Evidence: 1.1 critique I1, results_v2 technical debt, known-issues row 63.
2. Roadmap open question 10 (roadmap.md:789): should another key cancel a pending gesture whose key passes (1.3 interrupt rule I1)? If yes: a 2.2 acceptance line or a new item; two scenarios flip. Evidence: 1.3 critique I1, results Notes 1, known-issues row 67.
3. Restated, not new: open question 6 (roadmap.md:785): after manual checks 4 and 5, should the template switch to `MaskOnly`? Checkpoint 1 item 4 fails by design until then.
4. For the orchestrator to mention, no roadmap question added because roadmap 1.5 specified it: a second signed-in user's daemon now stops at the pipe (KS-40) instead of remapping without IPC.

## RON validation
No `ron` block was added or changed by this sync. As a regression check, every `ron` block of `book/src/config/*.md` and `README.md` ran through `cfgcheck` rebuilt from methodology.md against 20af28d (`D:/tmp/ks-orch/phase-1/cfgcheck/`, files in `D:/tmp/ks-orch/phase-1/ron/`, output `D:/tmp/ks-orch/phase-1/ronrun.txt`):
29 blocks: 27 run, 27 OK (0 expected to fail), 2 fragments of RON syntax (ron-syntax.md:184, :191) not runnable. README.md:333 and :555 hold several bindings without commas and loaded once split into a list (4 and 2 bindings); actions.md:208 loaded wrapped as one list (4 bindings). The shipped template: `OK: 3 bindings`, identical to methodology.md apart from the path (Alt+Shift fallback; this machine's registry has no Toggle values, so the count can differ elsewhere). The template's commented examples are covered by `default_config_commented_examples_load` in the gate run.

## Links
Internal: 1824 links checked, 0 broken (source slugs and explicit ids); 1714 anchored links checked against the ids of the built HTML, 0 missing. Permalinks: 1565 checked (commit resolves, path at commit, range inside the file), 0 bad; every 20af28d range's first and last line was printed and read against its sentence, 5 ranges corrected. External: 2 fetched (blob 20af28d/src/engine/winner.rs, commit 1058ddb), both resolve on GitHub after the push.

## Build
`mdbook build book -d D:/tmp/ks-orch/phase-1/book`: OK. Warnings: only `The mdbook-mermaid preprocessor was built against version 0.5.3 of mdbook, but we're being called from version 0.5.2` (local tool versions; CI pins 0.5.3). Mermaid: not rendered locally.

## Quality gate on merged master (f7f3a68)
`cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -- -D warnings` exit 0, 0 warnings; `cargo test --locked` exit 0: 533 passed, 0 failed, 4 ignored (manual), plus 6 integration tests passed. Logs: `D:/tmp/ks-orch/phase-1/gate_*.log`. Pushed `20af28d..f7f3a68`; branch `docs/book-phase-1` deleted (never pushed).

## Claims removed or corrected in self-check
12 corrected: KS-20 status order (now "Fixed in 1058ddb ... Earlier, in a1b5a27"); KS-40 "exits" -> "stops" (it waits at the prompt); intro "fit for daily use" overclaim replaced by "Checkpoint 1 still needs a manual run"; duplicated intro sentence; methodology "same lines" -> "apart from the path"; follow-up on the OS key state labelled unverified; the 1.2 NOTE that said v2 is not built yet; 5 permalink ranges off by one line (keyboard_hook install, keys::press, single_instance Display, registry hotkey_from_values, template bindings). 1 historical link kept on 1dfd80a on purpose (kinds.md §10 "it used to be ignored", now marked "at `1dfd80a`").

## Open questions
None for the orchestrator beyond the owner questions above.

## Suggested follow-up
- Developer: CHANGELOG.md:242 reflow (1.1 M2); CHANGELOG Breaking entry for block_default (1.3 P2: modifier-only Hold/DoubleTap output under the passed modifier, `block_default: true` remedy) and the "A Tap on Alt or Win" sub-bullet at CHANGELOG.md:272 (1.3 P1 wording).
- Developer: loader MaskOnly warning before the KS-18 skip (1.2 m1); `parse_ron_str` still logs "config contains no bindings" (1.6 m3); `// SAFETY:` in `console_ctrl_handler` (app.rs:248); `run`'s doc comment on `_instance` (1.5 m1).
- Orchestrator: CLAUDE.md and agent files (1.2 m2, 1.5 Layout, 1.7 hand-offs 1-8), WORKFLOW.md:23.
- Owner: the four manual-check tables (Phase 0 and Phase 1) are still unrun; results go into the audit status lines.
