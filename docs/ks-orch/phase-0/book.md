# Documentation update: book sync after roadmap Phase 0
Verdict: DOCS READY WITH GAPS
Pinned commit: 1dfd80a (master after the six Phase 0 lanes). The audit stays pinned to 7cb3521.
Merged: docs/book-phase-0 (70b6321, 20d2189) into master as efea317 (--no-ff). Pushed: 1dfd80a..efea317. Branch deleted.

## Pages changed
- `book/src/audit.md`: status note (13 fixed, 3 partly: KS-05/20/25, 10 open), a "Status at 1dfd80a" column in the summary table, a `Status:` line per KS-01..26, and a status appended to the "not independently verified" lines of KS-27..42. Findings unchanged. KS-06 residual: a lone key-up after a focus change unless a Tap/Hold is pending (item 2.3).
- `book/src/roadmap.md`: progress note. Workstream A (six-lane table, scope as built, acceptance evidence) and Workstream B marked done with shas, plus the 0.1 CI status. New section "Manual checks after Phase 0" (13 rows). Status notes on 1.6, 1.7, 2.1, 2.2, 2.3, 3.2, 3.4, 3.5 and 4.1. Issue-coverage tables gain a status column. Open questions 1, 2, 4 and 8 are marked as decided by the owner.
- `book/src/config/*.md` (all 11 pages): re-described and re-pinned to 1dfd80a, with no 7cb3521 permalinks left. Covered: exact modifier matching and allow_extra_modifiers, KeySpec, the trigger-key rule, the reworked auto-blockers, deny_unknown_fields, required `bindings`, the BOM strip, the 1..=10000 ms timings, chord overlap errors, the Missing error, AppEquals, and load vs reload. known-issues.md: Status column, rows 1-53 kept, new section 19.9 (rows 54-61). example.md: 24 bindings + 1 blocker = 25 (validated).
- `book/src/architecture.md`: rewritten at 1dfd80a (engine, Decision, Env, Mutex<Engine> via with_engine, the enabled flag, the threads and locking section, the root-cause status table). Old anchors kept as `<a id>` aliases.
- `book/src/build-and-run.md`: rewritten at 1dfd80a (swch on/off/status, pipe SDDL, 5 s request deadline, CI section). Old troubleshooting anchors kept as aliases. KS-05 workaround: `block_original_combo: false`.
- `book/src/overview.md`: current state at 1dfd80a, a "Changes since the audit" merge table, and the audit snapshot sections kept.
- `book/src/glossary.md`: rewritten. New terms: allow_extra_modifiers, DaemonControl, Decision, Enabled flag, Engine, Env, Exact modifier matching, KeySpec, Per-key table, Suppression record.
- `book/src/gaps.md`: status note and status columns. The swch read-timeout gap is closed. A re-entry row was added.
- `book/src/README.md`, `book/src/methodology.md`: intro updated; "The book sync after Phase 0" added.
- `README.md`: sample session output, the pipe/timeout paragraph, and the real unknown-variant error text (`FlyToMars`). The schema comment now matches the default config.

## SUMMARY.md
Unchanged (no new pages).

## RON validation
cfgcheck (config::reload on the checkout) at D:/tmp/ks-orch/phase-0/cfgcheck. Records: D:/tmp/ks-orch/phase-0/ron-final.tsv.
- 25 page blocks (config pages + README): 25 PASS after fixing two wrapper artifacts.
  - 21 loaded directly with the binding count the page states.
  - 4 first runs exited 1 because of how I wrapped them, not because of the pages. ron-syntax #2 and #3 are field fragments. README #3 and #6 are lists printed without commas on purpose. With correct wrapping or commas they load OK.
- 40 probe files (ron/probe/p01..p40) plus `--missing`: every error text the book quotes was reproduced verbatim.
- `cargo test --locked --lib -- readme_matching_examples_load`: pass.
- Binding counts that include auto-blockers depend on the registry language hotkey (Alt+Shift fallback). The pages say so.

## Links
Internal (built HTML, htmllinks.py, rerun on master after the merge): 20 pages, 1990 links, 0 broken, 0 duplicate ids.
Permalinks (permalinks.py): 1406 checked, 0 bad. 618 point at 1dfd80a and were reviewed against the cited lines. 601 point at 7cb3521 and are historical.
External: 3 fetched with WebFetch (a matching.rs blob, commit a1b5a27, the MS schtasks page): all OK.

## Build
`mdbook build book -d D:/tmp/ks-orch/phase-0/book` on master efea317: OK.
One warning: "mdbook-mermaid preprocessor was built against version 0.5.3 of mdbook, but we're being called from version 0.5.2". It comes from the local tools only (CI pins 0.5.3).
Quality gate on master before the push: fmt OK; clippy -D warnings OK; cargo test: 362 passed + 1 ignored (manual) in lib, 6 passed in integration.

## Claims removed or corrected in self-check (about 9)
- KS-06 residual: the A2-era description was wrong. Corrected to a lone key-up after a focus change. Repeats stay swallowed (Forgotten state).
- KS-05 workaround "press Shift first" was wrong, because the blocker swallows whichever modifier comes second. Replaced with block_original_combo: false in build-and-run, example and matching.
- binding.rs ranges fixed: new_auto_blocker L98-L109, Binding L49-L76, Combo variant L21-L23.
- a1b5a27 merge date corrected to 2026-10-16. The merge count excludes the three agent-workflow merges.
- Elevated-process claim now sourced.
- README: the invented FlyToMars message and the "OK:" prefixes on swch output were replaced with real text.
- Five duplicate explicit anchors removed.
- A mermaid label with ';' changed to use a comma.

## Open questions
- PrintScreen lone key-up behaviour after a focus change was inferred from the code, not observed.
- The roadmap still says "one pull request" in places, while the flow has no PRs. That wording belongs to the analysis roles, so I left it.

## Gaps (why WITH GAPS)
- Mermaid diagrams were linted for risky characters but not rendered: no renderer here.
- Runtime behaviour (matching, suppression, timing, swch on/off, pipe ACL) is described from the code and labelled "inferred from the code, not observed on a live daemon". The 13 manual checks in roadmap "Manual checks after Phase 0" have not been run.
- The Pages deploy was triggered by the push; I did not check the result.

## Suggested follow-up
- The owner runs the 13 manual checks and the book updates their status.
- CLAUDE.md layout does not list src/data/key_spec.rs or src/engine/matching.rs; the orchestrator should update it.
- Upgrade local mdbook to 0.5.3 to silence the warning.
