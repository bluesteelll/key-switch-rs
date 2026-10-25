# Merge: p1-1.3-deferred-block-default (roadmap item 1.3)

## VERDICT: MERGED

- Merge commit: `9357cfa48be0d2c7801dbaaef529541e04a742d2` on `master`, pushed (`1afa720..9357cfa  master -> master`).
- Parents: `1afa720` (master, = origin/master, = merge-base) and `1562acd` (feat/deferred-block-default).
- Message: `Merge branch 'feat/deferred-block-default': block_default for Tap, Hold and DoubleTap`, a blank line, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Branch `feat/deferred-block-default` deleted locally with `git branch -d` (was 1562acd). It was never pushed, so there is no remote branch.

## Preconditions

- Results-analyst verdict: `results.md` line 3, `## VERDICT: ACCEPT` (branch @ 1562acd).
- `git status` on `feat/deferred-block-default`: `nothing to commit, working tree clean`.
- After `git fetch origin`: `master` = `origin/master` = `1afa720` = `merge-base master feat/deferred-block-default`. No commits on either side that the other lacks, so the merge was conflict-free.

## Gate on the branch (1562acd)

| Check | Result | Log |
|---|---|---|
| `cargo fmt --check` | exit 0 | `logs/merge_branch_fmt.txt` |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 | `logs/merge_branch_clippy.txt` |
| `cargo test --locked` | exit 0. lib `412 tests`: 411 passed, 0 failed, 1 ignored; swch 0; `ipc_roundtrip` 6 passed; doc-tests 0 | `logs/merge_branch_test.txt` |

## Merge

`git checkout master` then `git merge --no-ff feat/deferred-block-default` with the message above, using the `ort` strategy. There were no conflicts: 22 files changed, 897 insertions(+), 111 deletions(-).
The tree of merge commit `9357cfa` (`fa42d55`) is identical to the branch tip's tree (`1562acd^{tree}` = `fa42d55`).

## Gate on the merge commit (9357cfa)

| Check | Result | Log |
|---|---|---|
| `cargo fmt --check` | exit 0 | `logs/merge_master_fmt.txt` |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 | `logs/merge_master_clippy.txt` |
| `cargo test --locked` | exit 0. lib `412 tests`: 411 passed, 0 failed, 1 ignored; swch 0; `ipc_roundtrip` 6 passed; doc-tests 0. Total 417 passed, 1 ignored | `logs/merge_master_test.txt` |

The ignored test (not counted as a pass) is the existing `data::condition::tests::foreground_context_names_the_focused_notepad` (manual). This lane adds no `#[ignore]`.

## Push and cleanup

- `git push origin master`: exit 0, `1afa720..9357cfa`.
- `git branch -d feat/deferred-block-default`: exit 0.
- After another fetch: `master` = `origin/master` = `9357cfa`, and the work tree is clean.
- Nothing else was done: no tags, no releases, no settings changes.

## Open points carried from results.md (for the orchestrator, not blocking)

- I1 owner question: should a pass-through gesture be cancelled when another key goes down? It is pinned at `src/engine/scenarios.rs:3341` and `:3376`, and listed as known-issues row 67.
- P1 (review m1): the lone Alt/Win tap with a passing gesture is attributed to item 1.2 in CHANGELOG, README and known-issues row 66, but 1.2 as written does not fix it. Decide when 1.2 is cut, or hand it to doc-writer.
- P2 (review m2): the Breaking entry does not say that a modifier-only Hold now sends its output with that modifier down (KS-19, item 3.1). Hand it to doc-writer.
- Phase-1 book sync: the stale lines outside `book/src/config/` (cut.md:176), plus `gaps.md:182`.
- F2 (block_default is ignored on sequence/chord) and F6 (stale `.claude/agents/code-reviewer.md:126`) are recorded only in lane reports.
- Checkpoint 1 manual checks: smoke #6 and impl.md M1-M3.
