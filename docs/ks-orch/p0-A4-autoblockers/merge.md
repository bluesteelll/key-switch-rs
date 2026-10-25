# Merge: p0-A4-autoblockers (fix/auto-blockers -> master)

## Verdict: MERGED

- Merge commit: `b056b72af589ff6f1702d7cdb93ade68d5eed675` (`b056b72`), pushed: `44f79ac..b056b72  master -> master`.
- Message: `Merge branch 'fix/auto-blockers': Exact auto-blockers and dedup`, blank line, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Lane branch `fix/auto-blockers` deleted locally (`git branch -d`, was `3d65ac9`). It was never pushed, so there is no remote branch to remove.

## Preconditions

- Results-analyst verdict: **ACCEPT** (`results.md:5`), review r1 APPROVED, tester r1 GREEN.
- `fix/auto-blockers` @ `3d65ac9`: working tree clean (`git status`: nothing to commit).
- `master` == `origin/master` == merge-base == `44f79ac`: master did not move since the cut, no conflicts possible.

## Gate on the lane branch (3d65ac9)

| Check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 |
| `cargo test --locked` | lib: 307 passed, 0 failed, 0 ignored; `swch` bin and doc-tests: 0 tests. Log: `D:/tmp/ks-orch/p0-A4-autoblockers/merge-branch-test.txt` |

## Merge

- `git checkout master`, `git merge --no-ff fix/auto-blockers`: ort strategy, no conflicts, 13 files, +803/-49 (CHANGELOG.md, README.md, src/config/{default_config.ron,loader.rs,mod.rs}, src/data/binding.rs, src/engine/{env.rs,keys.rs,matching.rs,mod.rs,properties.rs,scenarios.rs,testing.rs}).
- `git diff 3d65ac9 b056b72` is empty: the merge tree is the tested branch tree.

## Gate on the merge commit (b056b72)

| Check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0, forced re-check (crate roots touched; "Checking key-switch-rs ... Finished") |
| `cargo test --locked` | lib: 307 passed, 0 failed, 0 ignored; `swch` bin and doc-tests: 0 tests. Log: `D:/tmp/ks-orch/p0-A4-autoblockers/merge-master-test.txt` |
| Working tree after gate | clean |

## Interpretation readings the lane merged under (results.md P2)

The orchestrator has not formally ruled on these; they are recorded here as the readings carried by `b056b72`, endorsed by the critic, reviewer, tester and results-analyst:
- **Q1**: acceptance A1 ("Alt+Shift+Tab and Ctrl+Shift+T pass through") is met for every order in which the third key is down before the second modifier. The modifiers-first order (Ctrl, Shift, T; Alt held, Tab released, Shift+Tab) is carved out to roadmap item 1.2 (mask key), per `book/src/roadmap.md:100-101` and smoke item 4. Master behaved the same; not a regression.
- **Q2 (D2)**: spec S1's "held set" means the keys Windows has down besides the hotkey (engine-held keys that Windows does not have down, e.g. after Win+L or a swallowed key, do not count). Pinned by scenarios T9, T10 and the M3 pin.

## Open points for the orchestrator (not blocking, from results.md)

- P1: `3d65ac9` (tester) carries `Co-Authored-By: Claude Sonnet 5.5`; history cannot be fixed without amending. Root cause in `.claude/agents/tester.md:22` and `.claude/workflows/ks-lane.js:134` still open.
- P3: book sync backlog: pages in `cut.md:342`, RK5 and the `block_default: false` consequence for `book/src/config/known-issues.md`, audit status lines (KS-18 "Fixed in b056b72 (fix/auto-blockers)", KS-05 "Partly fixed in b056b72 (fix/auto-blockers)"), stale agent texts `.claude/agents/code-reviewer.md:124` and `.claude/agents/project-analyst.md:60`.
- Owner FYI and manual checks M1-M5 (`impl.md:58-62`) with Checkpoint 1.
- The lane directory still holds the analyst's `wt/` and `target-wt/` scratch directories; not created by this step, left in place.

## Actions not taken

No tags, releases, GitHub settings changes, or remote branch operations. No daemon run, no hooks installed, no input injected, no registry writes.
