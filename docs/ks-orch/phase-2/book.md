# Documentation update: book sync after Phase 2
Verdict: DOCS READY WITH GAPS
Pinned commit: bf549c0 (master HEAD before the sync: 3e779f8, orchestrator files only; the src tree equals bf549c0)

Gaps behind the verdict: no Mermaid diagram was rendered in a browser (syntax checked by rule only). mdbook-mermaid warns that it was built for mdbook 0.5.3 (local 0.5.2; machine setup). Four owner questions are open (Q11-Q14). Two code-doc follow-ups belong to the developer (see Suggested follow-up). None of the 48 Phase 2 manual checks could be run (safety rules).

## Pages changed
- `book/src/audit.md` - header status at bf549c0 (20 fixed, KS-05 partly, 5 open), summary column, and status lines for KS-01, 06, 08, 10, 11, 25, 29, 31, 33 with merge/fix commits and tests. Findings untouched.
- `book/src/architecture.md` - rewritten for bf549c0 (action executor, main thread state, gesture timers, control window, three hooks, witness, foreground cache, reload on main, hook watchdog, installer, panic hook, counters). All existing anchors kept. 136 bf549c0 links.
- `book/src/build-and-run.md` - startup text and flowchart, console-line table (new lines of 2.1-2.4), stopping, `swch status` reply with hook health, `on`/`off` failure replies, tests at bf549c0 (808/7 ignored), module table, troubleshooting.
- `book/src/overview.md` - current state, features table (Hook health row), threads, changes table with the four Phase 2 merges, build numbers and line counts.
- `book/src/glossary.md` - entries updated. New: Action executor, Control window, Epoch, Fast action, Foreground cache, Heartbeat, Hook watchdog, Input signal, Latency bench, Main thread state, Panic hook, Raw Input, Rule L, Witness. Hot path entry gains the witness and the cached `when:`.
- `book/src/gaps.md` - status block, rows closed (Tap/DoubleTap, cancellation, hook health, reset, Sequence panic, panic messages), re-entry row rewritten, test-coverage note.
- `book/src/roadmap.md` - progress line, Phase 2 done, status blocks 2.1-2.4 with acceptance tables, Checkpoint 2, "Manual checks after Phase 2" (48 rows), Phase 1 manual checks 9 and 16, known-issues list, coverage tables, phase table, RC2 witness exception, open questions 5/7/9/10 implemented, new owner questions 11-14, Phase 1 follow-ups closed, new "Follow-ups from Phase 2", status notes for 3.4, 3.5, 4.6.
- `book/src/config/known-issues.md` - re-pins, fixed rows with merge commits, permalinks on 75-80, new rows 81-87 (2.4 residuals R-1..R-7).
- `book/src/config/{README,actions,errors,example,keys,kinds,matching,ron-syntax,structure,triggers-conditions}.md` - facts commit bf549c0, re-pinned CHANGED links. triggers-conditions §11.4 names the waiting Tap as timer-fired (2.2 P2). actions §13.1 Tap row. example.md header and F24 sentence.
- `book/src/README.md`, `book/src/methodology.md` - commit, test counts, new section "The book sync after Phase 2".
- `README.md` - module tree: `hook/debug_hold.rs`, `src/logging/` block, `system/known_folder.rs` (FU-3, R-m4).
- Not touched: `src/**`, `default_config.ron`, `CHANGELOG.md`, `CLAUDE.md`, `.claude/`.

## SUMMARY.md
Unchanged (no new page).

## Hand-off dispositions

| Lane | Item | Where (lane file:line) | Disposition |
| --- | --- | --- | --- |
| 2.1 | roadmap known issue "OpenUrl can freeze input" | results.md:138 | Done: known-issues list rewritten |
| 2.1 | 2.1 status after Phase 2 | results.md:138 | Done: 2.1 status block |
| 2.1 | KS-08 coverage row | results.md:138 | Done |
| 2.1 | M1-M13 table (M13 from cut.md:660) | results_v3.md:180, merge.md:41 | Done: Manual checks after Phase 2, rows 1-13 |
| 2.1 | Extend open question 7 to macro Delay (P1) | results.md:138 | Moot: Q7 decided (62e36a6), R3-T fixed the precision (results_v2.md:126) |
| 2.1 | R-7 and m-1 into 2.2's notes | results_v3.md:178 | Done: Follow-ups from Phase 2 rows (R-7, MWMO pin) and 2.2 status text |
| 2.1 | roadmap 2.2 "Files" reuses `platform/timers.rs` | results_v3.md:177 | Done: 2.2 status block sentence (plan text kept) |
| 2.1 | audit KS-08 partly, KS-33 Sequence half | results.md:139 | Done, then superseded by 2.2 (KS-33 fixed) and 2.4 (KS-08 fixed) |
| 2.1 | gaps.md:35, :145-147, :166 | results.md:140 | Done |
| 2.1 | architecture thread table, KS-08 section, executor wait and timer | results.md:141, results_v3.md:176 | Done: architecture rewritten |
| 2.1 | glossary :107, :317; overview :20-21, :68; build-and-run executor lines | results.md:142 | Done |
| 2.1 | SystemFunction variants without fast/slow placement | results_v3.md:149 | Done: follow-up row |
| 2.1 | Blocking steps serialise the executor | results_v3.md:150 | Done: follow-up row |
| 2.1 | Window(Restore) to front unknown (M12) | impl.md | Done: follow-up row + manual check 12 |
| 2.1 | executor/mod.rs:6 code doc | 2.2 results H-B4 | Done by 2.3 C1 (not a book item) |
| 2.1 | CI watch of the real-timer tests | results_v3.md:181 | Not applicable: orchestrator item |
| 2.2 | H-B1 M1-M10 into the manual-check table; Phase 1 check 9 | results.md:161 | Done: rows 14-23; check 9 updated |
| 2.2 | H-B2 triggers-conditions.md:77 (P2) | results.md:162 | Done: §11.4 names the waiting Tap |
| 2.2 | H-B2 errors.md/counter text for a refused window timer | results.md:162 | Already done by 2.3 (counters.rs:27-31 and errors.md agree: a waiting Tap is not counted) |
| 2.2 | H-B3 passing multi-key DoubleTap | results.md:163 | Superseded by 2.3 C8 (known issue 68 and §11.4 updated in lane 2.3) |
| 2.2 | H-B4 architecture/overview/glossary/build-and-run/roadmap/audit/gaps list | results.md:164, impl.md:217-227 | Done (overview :223 is the audit snapshot, kept historical) |
| 2.2 | H-B5 follow-ups (watcher `.expect`, MWMO pin, re-entry counter, Tap/Hold warning, KBDLLHOOKSTRUCT.time, R-7, m2, OQ3) | results.md:165 | Done: rows in Follow-ups from Phase 2 (watcher, counter, m2/OQ3 closed by 2.3/2.4) |
| 2.2 | H-B6 latency bench in the acceptance | results.md:166 | Done: 2.2 acceptance table row |
| 2.3 | impl.md "Left to the Phase-2 book sync" (architecture, build-and-run, overview:25, glossary, roadmap, audit KS-06/29/31, gaps:280) | impl.md:116-123 | Done |
| 2.3 | roadmap 2.3 status, What bullet, latency bullet (title live) | results.md:180 | Done: status block; title live also as owner Q12 |
| 2.3 | Open question 9 implemented; follow-up :808 closed | results.md:181 | Done |
| 2.3 | 3.5 per-window cache done | results.md:182 | Done: 3.5 status note |
| 2.3 | Coverage rows KS-06/29/31; phase table 2.3 row | results.md:183 | Done (KS-06 rest added to the phase row) |
| 2.3 | Follow-ups F1, F3, F4, F5, F6, witness cost | results.md:140, :184 | Done: follow-up rows |
| 2.3 | M1-M14 with M1 pass criterion and M5 precondition | results.md:185 | Done: rows 24-37 |
| 2.3 | Suppression invariant wording (RC2, architecture, glossary) | results.md:186 | Done |
| 2.3 | gaps :156 closed, :164 re-entry | results.md:187 | Done |
| 2.3 | README `when::` typo (P3) | results.md:188 | Already done by 2.4 (4f2d406) |
| 2.3 | `MainThreadHandle::reload` duplicates `set_enabled` | results.md:143 | Done: follow-up row |
| 2.3 | winner.rs:57 stale doc | results.md:144 | Not applicable: code doc; already fixed at bf549c0 (winner.rs:55-58 names the cache) |
| 2.3 | Owner: witness +0.6 us, title live, timestamped M1 | results.md:195-198 | Owner questions Q12, Q13; M1 in manual check 24 |
| 2.4 | impl.md "Book pages left" (architecture, roadmap, audit, gaps, build-and-run, glossary, overview) | impl.md:136-145 | Done |
| 2.4 | roadmap "on the 2.3 window" -> own message-only window (D1); acceptance reading Q1 | results.md:168 | Done in the 2.4 status block (plan text kept) |
| 2.4 | audit KS-08 status lines | results.md:169 | Done |
| 2.4 | FU-1..FU-4 | results.md:176 | Done: follow-up rows (FU-3 closed) |
| 2.4 | M1-M10 with fix_r1/fix_r2 rules, no L1.12 numbers | results.md:177-184 | Done: rows 38-48 (M6 as 43/44) |
| 2.4 | R-1..R-7 to known-issues | results.md:131, :185 | Done: rows 81-87 |
| 2.4 | Unverified Q5-Q8, E2-E4, E9, E11, OQ4s | results.md:185 | Done: "Not verified" follow-up rows |
| 2.4 | R-m2 panic hook without fallback, R-m3 wording | results.md:112-113 | Done: follow-up rows |
| 2.4 | README debug_hold.rs (R-m4), src/logging/ (FU-3) | results.md:186 | Done |
| 2.4 | Owner H-O1/H-O2, CLAUDE.md layout | results.md:190+ | Not applicable: orchestrator |

## Owner questions
Recorded in `book/src/roadmap.md#open-questions-for-the-owner`:
- Q11: a waiting Tap fires when another key goes down (2.2), instead of being cancelled.
- Q12: the window title of a `when:` is read live, not cached (2.3).
- Q13: two key-path costs above estimate: witness +0.6 us per swallowed key-up (R11), first key after idle about 50 us slower at the median (R16).
- Q14: main thread at THREAD_PRIORITY_HIGHEST (2.2).

## RON validation
30 blocks of README.md and book/src/config/*.md through cfgcheck at bf549c0 (`ronrun_final.txt`): 28 OK as expected, 2 fragments in ron-syntax.md not run (they are not configs), 0 MISMATCH. actions.md:210 (a raw-string block the parser cannot split) validated by hand: OK: 4 bindings (`ron/actions.md-210-manual.ron`). example.md: OK: 26 bindings. 21 manual-check configs in roadmap.md (`ron/roadmap-mc*.ron`, `ron/roadmap_runs.txt`): all OK. Template: OK: 3 bindings with the Alt+Shift fallback. Binding counts include auto-blockers and can differ by machine.

## Links
Internal: 2090 Markdown links checked by links.py, 0 broken; htmllinks.py over the built book: 20 pages, 2560 links, 0 broken, 0 duplicate ids. Permalinks: all checked by permalinks.py (commit resolves, path exists, range inside the file), bad=0; 758 to bf549c0; the CHANGED targets reviewed by hand (show_all.txt, review_other.txt). External: GitHub commit and blob permalinks only, verified with git locally; no new external sites.

## Build
`mdbook build book -d D:/tmp/ks-orch/phase-2/book`: OK. Warnings: only the mdbook-mermaid version warning (built against 0.5.3, called from 0.5.2), pre-existing machine setup.

## Claims removed or corrected in self-check
About 20: range ends (keys.rs alt_or_win 228-239, press 158-165, gestures 54-154, condition 18-45 and 77-86, env 62-75, foreground 184-225, sequence run_step 104-117, sequence 123-227, windows_actions 208-215, watchdog 246-251, 456-479, 675-683); example.md F24 sentence (a binding with `when` beats one without); "Hold is the only timer-fired trigger" (now also the waiting Tap); RC2 suppression invariant (witness exception); stale "engine lock"/`hold-timer`/`GetMessageW`-loop wording removed from architecture, overview, glossary and build-and-run.

## Open questions
None for the orchestrator. Owner questions Q11-Q14 above.

## Suggested follow-up
- Developer: `CHANGELOG.md` [Unreleased] KS-08 entry (CHANGELOG.md:744-745) still says a `Hold` runs fast actions "on its timer thread as before" (stale since 2.2; they run on the main thread).
- Developer: `src/platform/timers.rs:2-3` module doc says the gesture timers of 2.2 come "later" (stale since 2.2).
- Owner: run the 48 manual checks after Phase 2 and Checkpoint 2.
- Render the Mermaid diagrams once in a browser after the Pages deploy.

## Merge
Commit `9f652c0` on `docs/book-phase-2`, merged `--no-ff` into master as `7c26671`. Gate on master: `cargo fmt --check` OK (gate_fmt.log), `cargo clippy --all-targets --locked -- -D warnings` OK (gate_clippy.log), `cargo test --locked` OK: 801 passed, 7 ignored, 7 integration passed (gate_test.log). Pushed `3e779f8..7c26671` to origin/master; branch deleted.
