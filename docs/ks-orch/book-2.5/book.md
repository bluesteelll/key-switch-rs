# Documentation update: book sync after roadmap item 2.5 (Latency trim)
Verdict: DOCS READY WITH GAPS
Pinned commit: f4c1fb6 (the merge of `perf/latency-trim`; master is 252daef, whose source differs only in two doc-comment lines of `src/engine/modifiers.rs`)
Branch: docs/book-2.5, head 24444e9 (5 commits on 252daef). Not merged, not pushed.

Gaps behind the verdict: no Mermaid diagram was rendered in a browser (syntax checked by rule only). This matters more than usual: the two key-path flowcharts of `architecture.md` had been broken on the published book since the Phase 2 sync (9f652c0). Its generator dropped the `{"..."}` of every decision node, which left text like `C0code below zero?`. They are rewritten here, but nobody has rendered them. mdbook-mermaid warns that it was built for mdbook 0.5.3 (local 0.5.2), a machine setup issue. None of the 11 new manual checks (49-59) could be run (safety rules). The README's R16 sentence (README.md:958) still quotes only the developer's session; it is labelled "on the developer's machine" and true, so I left it (results.md P3 says "may").

## Pages changed
- `book/src/architecture.md` - re-pinned to f4c1fb6 (all 223 links; 37 changed targets re-read). Module table (modifiers.rs, resync.rs) and the windows feature `Win32_System_SystemInformation`. The thread diagram and table (drain read, watchdog wake). Threads and shared state (the drain's read under the borrow, a refusal noted as hidden input, E5 in the loop). Key-event path: step 5 (a sentinel modifier updates its flag; four fields copied), step 8 rewritten as the modifier check, step 13 (RepeatSuppressed), steps 14-15 (swallowed repeat, the key-up check of Alt/Win), new step 17 (the late note). Both key-path diagrams rewritten, decision nodes restored. Foreground and input-left notes. New section "Modifier state between key events" (problem, design, hiding-count diagram, sources, bench table with its session, trade-offs, known issues 88-90). Data structures: `HIDDEN_INPUT` row, `unseen_input`, twelve counters, the `resync` part, the `RepeatSuppressed` record, the reset note. Elevation repairs. Strengths (869/235 tests). Root-cause table at f4c1fb6.
- `book/src/glossary.md` - re-pinned. New entry "Hiding count" (no "Unapplied key"). Updated: Async key state, Counters (twelve, hook_late, INFO rule), Hot path, Injected event, KBDLLHOOKSTRUCT, Modifier key, Sentinel, Suppression record, WinEvent hook, Witness, AltGr, dwExtraInfo.
- `book/src/roadmap.md` - progress line, Phase 2 status (five items), item 2.4's key-path line (one sentence added; 2.4's own figures kept). Item 2.5: a "Status after Phase 2: done" block with points that differ from the plan and an acceptance table. The R11 line is "Not met, by ruling W1", with the reason, and four rows the rulings added. Checkpoint 2 item 4. Manual checks: intro (debug-build rows), check 44 (R16 now 1-4 µs), new rows 49-59, closing paragraph. Item 3.5 status after Phase 2. Open question 13 outcome. Follow-ups: the witness-skip row closed, FU-4 likely moot, the Q6 row, six new rows (Q11, D10 property coverage, U1 fallback, Sticky Keys, the sentinel note for 3.1 and feature 6, the flaky executor test). Issue-coverage headers at f4c1fb6.
- `book/src/config/known-issues.md` - header and status bullets at f4c1fb6, re-pinned. New rows 89 (Sticky Keys latch, unverified, check 57) and 90 (clock of the time stamps, unverified, check 56). Row 16 wording ("only `vkCode` decides the key"). Row 88 (the lane's) unchanged apart from its link re-pin 045b8b4 -> f4c1fb6.
- `book/src/config/{README,actions,errors,example,keys,kinds,matching,ron-syntax,structure,triggers-conditions}.md` - facts commit f4c1fb6, re-pinned. kinds.md section 10: the swallowed repeat with no record (D10). keys.md: what the hook copies from KBDLLHOOKSTRUCT. example.md: the commit in its leading comment and the warning's commit.
- `book/src/overview.md` - commit note, features table (interception, hook health, known issues 88-90), changes table (7c26671, f4c1fb6; orchestrator merges 50b56a8, 3cde1d0, 252daef in the excluded list), build/test status (862/7/7, 869 listed, 235 scenarios), line counts at f4c1fb6 (40,201 lines in 67 files, 14,568 outside tests), high-severity table header.
- `book/src/build-and-run.md` - commit note, the counters console line (twelve counts, `hook_late`, INFO rule), test counts and the per-module table at f4c1fb6 (869), clippy/fmt sentence, troubleshooting header and two new rows (known issues 88 and 90).
- `book/src/gaps.md` - status header and table headers at f4c1fb6. Rows "Hook flags and scan code ignored", "Non-modifier key state is not repaired" (:167) and "Re-entry" updated. Test-coverage note (862). Secure Desktop status line (:304; the audit-notes column of :300 kept as the audit's).
- `book/src/audit.md` - KS-14 status line only: one sentence on where its fix lives since f4c1fb6 (the check and the drain read both skip a swallowed modifier; the three tests still pass). Findings untouched.
- `book/src/README.md` - commit, a Phase 2 sentence on item 2.5, test count (also corrects "808 passing" at bf549c0, which was 801 passed + 7 ignored), the dates note.
- `book/src/methodology.md` - new section "The book sync after item 2.5", the template-output sentence (252daef), the "No measurements" limitation (session-specific figures).
- Not touched: `src/**`, `README.md`, `default_config.ron`, `CHANGELOG.md`, `CLAUDE.md`, `.claude/`.

## SUMMARY.md
Unchanged (no new page).

## Hand-off dispositions

Sources: results.md "Hand-off: phase book sync (doc-writer)" (R), impl.md "Book pages and agent files left to the phase book sync" (I), the brief's standard duties (B). The hand-off's line numbers were taken on the lane branch; master 252daef has the same book text, because the lane touched only errors.md, keys.md and known-issues.md, so every one was found at the same place.

| # | Item | Source | Disposition |
| --- | --- | --- | --- |
| 1 | roadmap :586-595, 2.5 What/Acceptance/status (W1, W2/W2a, D12 withdrawn, R16 gated and 52.6 -> 0.9 µs) | R, I | Done: status block plus acceptance table; plan text kept, as for 2.1-2.4; the R11 line marked "Not met, by ruling W1" with the reason |
| 2 | roadmap :584, 2.4's key-path line | R, I | Done: one sentence added (0.2 µs, 1-4 µs, R16 gated); 2.4's measured figures kept as its record |
| 3 | roadmap :654, check 44 "about 50 µs" | R, I | Done: "about 50 µs at item 2.4 and about 1 to 4 µs since item 2.5", FU-4 likely moot; no "one tick later" amendment (D12 reverted) |
| 4 | roadmap :739, 3.5 status | R, I | Done: witness follow-up lapsed (W1); Q11, check 57 (M9), D10 property coverage, the U1 fallback as candidates; FU-4 likely moot |
| 5 | roadmap :952, open question 13 outcome | R, I, B | Done |
| 6 | roadmap :999, FU-4 row | R, I, B | Done: kept, with the new figures, likely moot (L3) |
| 7 | roadmap :1005 | R, I | Done: Q6 row now names checks 43, 44 and 53, FU-4 likely moot |
| 8 | Manual checks M1-M11 into "Manual checks after Phase 2", M8/M10/M11 marked | R, I, B | Done: rows 49-59 (M1-M11 in order). M5-M7 as checks 44, 39, 47 as written (L3). M9 with its latched step. M8, M10, M11 (56, 58, 59) marked as checks of the lane's own assumptions; M9 (57) marked as settling known issue 89 |
| 9 | Checkpoint 2 | B | Done: item 4 (checks 49, 50, 56-59) |
| 10 | Progress line and phase table | B | Done: progress line; Phase 2 status paragraph (five items). The phase table's 2.5 row was already right |
| 11 | Follow-ups from Phase 2: witness-skip row, FU-4 | B | Done: witness-skip **Closed**, not taken up (W1); FU-4 updated. Plus six new rows from impl.md "Follow-ups for 3.5" and results.md "Notes for later lanes" |
| 12 | architecture :237 (step 8), :248 (step 15), :251, :265 (mermaid), :294, :331, :350, :362, :566-567, :605 | R, I | Done, each; plus the steps the lane changed that the list missed (5, 13, 14, new 17), the module and thread tables, and the shared-state table |
| 13 | glossary :67, :377, :569, :577, new "Hiding count", no "Unapplied key" | R, I | Done; also Counters, Hot path, Injected event, KBDLLHOOKSTRUCT, Sentinel, Suppression record, AltGr, dwExtraInfo, which were stale too |
| 14 | overview :16 | R, I | Done; plus the commit note, changes table, test and line counts |
| 15 | gaps :167, :300 | R, I | Done (:300's notes column is the audit's; its status line :304 updated) |
| 16 | Optional: kinds.md section 10, D10's corner case | R | Done: one bullet, with the pinning scenario |
| 17 | P1 README (sentinel bullet, module tree) | R | Moot: done in 28566a4 (README.md:154 and :56-57 checked) |
| 18 | P2 rustdoc warnings | R | Moot for the book: done in 28566a4 (`src/engine/modifiers.rs`) |
| 19 | P3 R16 figure with its session | R, brief | Done in the book: the tester's 0.9 µs and the developer's 4.3/4.4 µs stated together, each with its session, in architecture and roadmap. README.md:958 left: it says "on the developer's machine" and is true |
| 20 | CLAUDE.md:72, tester.md:101, principles 1/3/5/6, layout, code-reviewer.md | R, I | Not applicable (orchestrator): done in 252daef (CLAUDE.md:74, tester.md:102 checked) |
| 21 | audit.md status lines for KS issues the lane closed | B | No KS claimed (results.md "Issues claimed closed": none). One status sentence added to KS-14, whose fix the lane re-implemented (check and drain read both skip a swallowed modifier); finding untouched |
| 22 | known-issues: every residual of results.md "Technical debt and residuals" | B | Row 88 (W2/W2a/V3-I1 effects): the lane's, checked, kept. Q11 (own-key skip): covered by row 88's text ("pressed again before any other key-down, it reads as an auto-repeat (row 63)"); also a roadmap follow-up. M9 Sticky Keys: new row 89. U1 clock base: new row 90. D10 property coverage and FU-4: not user-visible, roadmap follow-ups only |
| 23 | build-and-run test counts at 252daef from merge_gate_master.log | B | Done: 862/0/7 and 7 integration from the log (taken on f4c1fb6); per-module counts from the log's test names; confirmed by `cargo test --locked -- --list` at 252daef (869 lib, 235 scenarios, 7 integration), target dir `D:/tmp/ks-orch/book-2.5/target-list` |
| 24 | methodology section; book README commit and counts | B | Done |

## RON validation
cfgcheck rebuilt verbatim from methodology.md#validation-harness in `D:/tmp/ks-orch/book-2.5/cfgcheck/` (dependency: the work tree, source = 252daef), `CARGO_TARGET_DIR=D:/tmp/ks-orch/book-2.5/target-cfgcheck`. Outputs: `ronrun.txt`, `template_run.txt`, `ron/manual_runs.txt`, files in `ron/`.

| File | Expected | Actual | Result |
| --- | --- | --- | --- |
| example.md:17 (comment changed) | OK, 25 + 1 blocker | OK: 26 bindings | PASS |
| roadmap check 58 config (`Ctrl+J` with the template's two bindings) | OK | OK: 4 bindings (blocker Alt+Shift, fallback) | PASS |
| roadmap check 59 config (`F13` with the template's two bindings) | OK | OK: 4 bindings | PASS |
| default_config.ron | output quoted in methodology | identical lines, exit 0 | PASS |
| actions.md:210 (raw strings; the splitter cannot wrap it) | OK, 4 | OK: 4 bindings (by hand, `ron/actions.md-210-manual.ron`) | PASS |
| the other 27 blocks of README.md and config/*.md (unchanged) | OK | all OK (`ronrun.txt`) | PASS |
| ron-syntax.md:184, :191 | fragments, not configs | not run | - |

32 run, 32 PASS, 0 MISMATCH. Binding counts include the auto-blocker, which comes from the registry (here the Alt+Shift fallback) and can differ by machine.

## Links
Internal: 2150 Markdown links checked (links.py), 0 broken; 2039 anchored links against the built HTML, 0 missing; htmllinks.py over the built book: 20 pages, 2624 links, 0 broken, 0 duplicate ids. Permalinks: permalinks.py over every link of the book (commit resolves, path exists, range inside the file): bad=0, 768 to f4c1fb6. The 48 changed targets were proposed by a line diff and reviewed by their first and last lines (`review_changed.txt`; 3 overrides). Every link I wrote by hand was checked against the source at f4c1fb6. Every cited test name was found in the test list at 252daef. External: KBDLLHOOKSTRUCT and GetMessageTime on Microsoft Learn fetched; they support "the stamp is what GetMessageTime would return" and "no word on GetTickCount". Full record: `D:/tmp/ks-orch/book-2.5/linkcheck.txt`.

## Build
`mdbook build book -d D:/tmp/ks-orch/book-2.5/book`: OK. Warnings: only the mdbook-mermaid version warning (built against 0.5.3, called from 0.5.2), the machine setup.

## Claims removed or corrected in self-check
About 15:
- book README "808 passing tests" at bf549c0, which was 801 passed + 7 ignored; now 862 passed.
- "Only `vkCode` and `dwExtraInfo` are read", in architecture step 5, the glossary (AltGr, KBDLLHOOKSTRUCT), keys.md and known issue 16: the callback now also copies `time` and `LLKHF_INJECTED`, for the late note only.
- "passed on before the engine sees it", for sentinel events (architecture, glossary dwExtraInfo, Sentinel, Injected event): a sentinel modifier updates the engine's flag.
- "the engine resyncs the modifiers" at a foreground change (architecture, glossary WinEvent hook, overview): it moves the hiding count.
- "resyncs 11 codes on every event" (architecture step 8, glossary): replaced.
- The developer's R16 described as "the about 4 µs": it is R16 4.3/4.4 µs, about 4 µs above R12.
- "a drain read runs once per move": the owed codes give a second read.
- "every key counts as late" if the clock differs: now "can count, in the worst case every key".
- "Microsoft documents that clock for GetMessageTime": now the wording the two pages support.
- Step 15's citation order.
- "the random scripts of engine::properties" for D10 coverage: now the one property named in impl.md.
- An overstated "every key" in known issue 90's cost.

## Open questions
None for the orchestrator. For the owner, already in the lane's report: W1 (R11 deliberately unchanged) and the manual checks 56-59.

## Suggested follow-up
- Render the architecture diagrams once after the Pages deploy (the key-path flowcharts were broken on the live site since 9f652c0).
- Orchestrator: `D:/tmp/ks-orch/phase-2/tools/gen_arch.py` turned `{"..."}` into plain text in an f-string; any reuse must escape the braces.
- Developer (optional): README.md:958 could also give the tester's R16 session (results.md P3).
- Owner: manual checks 49-59 and Checkpoint 2 item 4.

## Fix round
Verdict after the fix round: DOCS READY WITH GAPS (the same gaps as above: no Mermaid rendering, manual checks 49-59 not run).
Head: `2a942a9` on `docs/book-2.5` (one follow-up commit on `24444e9`, nothing amended). Not merged, not pushed.
Inputs: `check_facts.md` (12 findings: I1-I3, m1-m9) and `check_coverage.md` (13 findings: M1-M13). Four pairs are the same defect (m1 = M6, m2 = M1, m3 = M9, m6 = M11), so 21 distinct defects. All 25 rows are Fixed; none is refuted.

| Finding | Where | Resolution |
| --- | --- | --- |
| I1 | `config/actions.md:69` | Fixed (2a942a9): sentinel events pass "without deciding them; for a modifier among them it only updates the decision core's flag", as `keyboard_hook_callback.rs:246-249` and `README.md:154` say |
| I2 | `roadmap.md:987` | Fixed (2a942a9): "so the witness term of R11 is unchanged, about 0.6 µs over R12" (test_r1.md:217) |
| I3 | `architecture.md:309` | Fixed (2a942a9): ruling W2 of item 2.5 took the read off the key path; the ruling was the orchestrator's, the owner declined to choose and can overrule; open question 13 asked for two other costs (rulings.md:8-9, :18-19). "Under the owner's latency priority" left out: no lane report states it |
| m1 / M6 | `roadmap.md:611` | Fixed (2a942a9): "exceeded its new bound by 9 ns at the p50 (1.4 against 1.391 µs), below the 100 ns resolution of a p50, so the tester and the analyst counted no breach" (test_r1.md:216, results.md:104) |
| m2 / M1 | `roadmap.md:631` | Fixed (2a942a9): "Rows 35, 38, 44, 46, 49, 52 and 53 use a debug build" (each row's setup re-read) |
| m3 / M9 | `overview.md:128` | Fixed (2a942a9): "No key event reads every modifier from Windows: ... a key-down asks Windows only about the modifiers it holds" |
| m4 | `architecture.md:320`, `:325-327`; `roadmap.md:600` | Fixed (2a942a9): diagram nodes "keyboard input leaves this desktop" and "keyboard input comes back" both feed the count; the bullet says the next decided key ends the wait and moves nothing (`resync.rs:65-86`, new permalink). Roadmap 2.5 status now lists the daemon's start, a foreground change, input leaving and coming back, a note of hidden input, and says `swch on` and a reinstall read every key in their reset (`mod.rs:750-753`, `resync.rs:88-94`) |
| m5 | `build-and-run.md:147`, `glossary.md:131`, `roadmap.md:602`; also `config/errors.md:231` and `README.md:884`, same wording | Fixed (2a942a9): `[INFO]` only when `hook_late` is the only non-zero count "and no callback took 5 ms or more" (`counters.rs:252-261`, `SLOW_CALLBACK` 5 ms at `:132`; `record_callback(ran)` at `keyboard_hook_callback.rs:439` before `late_callback`). Roadmap 2.5 says a callback that ran 17 ms also took the 5 ms of a `[WARN]`. build-and-run.md now names the main-thread stretch (`counters.rs:64-69`) |
| m6 / M11 | `glossary.md:301` | Fixed (2a942a9): permalink `L422-L427` -> `L425-L430`, which includes the `vkCode`/`dwExtraInfo` read at L430 |
| m7 | `roadmap.md:1045` | Fixed (2a942a9): "Only a sentinel-tagged modifier updates the engine's flag without being decided; an injected key without the sentinel is decided like a typed one, so it can fire a binding, the feedback loop the sentinel prevents" (`decide_event` and `track_injected`, permalink widened to `L240-L296`) |
| m8 | `roadmap.md:9` | Fixed (2a942a9): "item 2.5 acted on question 13" |
| m9 | `overview.md:130` | Fixed (2a942a9): added "`252daef` also brings `28566a4`, which updates `README.md` and two rustdoc lines of `src/engine/modifiers.rs` after item 2.5 and changes no behaviour" (`git show --stat 28566a4`) |
| M2 | `roadmap.md:693` (check 59) | Fixed (2a942a9): Setup starts with `swch exit` and `swch status`, then PowerToys, then `swch open`, as check 58 does. The source has the same gap (cut.md:1805-1807); the book row is the published step |
| M3 | `roadmap.md:695` | Fixed (2a942a9): "Checks 43, 44 and 53 also decide follow-up FU-4, and checks 47 and 55 decide whether item 2.4 needs its kernel-wait fallback" |
| M4 | `roadmap.md:1031`, intro `:1011` | Fixed (2a942a9): candidate column "none: closed by ruling W1 of item 2.5, not done"; the intro adds "or, where the row says so, settled by a ruling without being done" |
| M5 | `roadmap.md:1041-1046` | Fixed (2a942a9): the five rows that are no "Not verified" point (Q11, D10 property coverage, U1 fallback, sentinel, flaky executor test) moved above the "Not verified" block; the Sticky Keys row stays in the block |
| M7 | `roadmap.md:618` | Fixed (2a942a9): "It runs after a move of the hiding count, with a short second read for the codes it owed, and never inside a callback" (`resync.rs:98-116`), as architecture.md says |
| M8 | `architecture.md:252` | Fixed (2a942a9): the invariant now reads "... if and only if its key-down was, or, for a press whose key-down the engine never saw, one of its auto-repeats was (a swallowed repeat, step 13; the `RepeatSuppressed` record ...), unless Windows has the key down" (`keys.rs:281-291` `take_blocked`, `mod.rs:940-948`) |
| M10 | `audit.md:409` (KS-14 status line only), `gaps.md:167` | Fixed (2a942a9): "a key-down asks Windows about each modifier the engine holds, and a swallowed key-up about each held Alt and Win, and drops those Windows has up" (`mod.rs:557-566`, `:999-1006`). The KS-14 finding text is untouched |
| M12 | `roadmap.md:691` (check 57) | Fixed (2a942a9): "If that daemon toggles caps lock and this one switches the layout, that is known issue 89; any difference from the older daemon goes to item 3.5, and the same result on both is no change of item 2.5" |
| M13 | `roadmap.md:597` | Fixed (2a942a9): "**Status: done** in ...", as items 2.1-2.4 |

### Verification of the fix round
- RON: no `ron` block, README snippet or `default_config.ron` line changed (README change is prose at :884-885), so nothing to re-run; the 32 earlier runs stand.
- Build: `mdbook build book -d D:/tmp/ks-orch/book-2.5/book`: OK, only the mdbook-mermaid version warning (0.5.3 vs local 0.5.2).
- Links (`linkcheck.txt`, section "Fix round"): 2151 Markdown links, 0 broken; 2040 anchored links against the HTML, 0 missing; built book 20 pages, 2625 links, 0 broken, 0 duplicate ids; permalinks bad=0, 769 to `f4c1fb6` (one new: `resync.rs#L65-L86`). The three new or changed ranges (`resync.rs:65-86`, `keyboard_hook_callback.rs:240-296`, `:425-430`) were read at `f4c1fb6` and show what the text says.
- Mermaid: the changed hiding-count diagram keeps quoted labels with no parentheses; not rendered.
- `audit.md`: only the KS-14 **Status** line changed.

### Claims removed or corrected in self-check (fix round)
- Removed "under the owner's latency priority" from the I3 fix: no lane report ties ruling W2 to it.
- The M8 sentence first named only `RepeatSuppressed` for step 13, which says "a swallowed repeat"; it now gives both names and links the per-key table.

### Follow-up found while checking (not a book defect)
- Developer: the rustdoc of `src/engine/resync.rs:6-11` lists "`swch on`, a hook reinstall" among the sources that move the hiding count, but their reset calls `Resync::read_everything` (`resync.rs:88-94`, `mod.rs:750-753`), which moves nothing. The book follows the code.
