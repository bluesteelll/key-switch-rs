# Merge: p0-A5-ipc (IPC: on/off gate, pipe ACL, swch timeouts)

## Verdict: MERGED

- Merge commit: `a1b5a27a1ddeb339679035a03281b3a9e813a0d4` on `master`, pushed (`b056b72..a1b5a27 master -> master`).
- Parents: `b056b72` (master) and `2baadf1` (fix/ipc-control).
- Message: `Merge branch 'fix/ipc-control': IPC: on/off gate, pipe ACL, swch timeouts`, blank line, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `fix/ipc-control` deleted locally with `git branch -d` (was `2baadf1`). It was never pushed, so no remote branch exists.

## Preconditions

- Results-analyst verdict: ACCEPT (`D:/tmp/ks-orch/p0-A5-ipc/results.md:5`).
- `fix/ipc-control` @ `2baadf1`, working tree clean, only one worktree.
- After `git fetch origin`: `master` = `origin/master` = merge-base = `b056b72`. The merge had no conflicts. Merge tree `360d67f` is identical to the lane branch tree `360d67f`.
- 15 files, +1561/-236, one new file `tests/ipc_roundtrip.rs`.

## Gate on the lane branch (2baadf1)

| Check | Result | Log |
|---|---|---|
| `cargo fmt --check` | rc 0 | `logs/merge-branch-fmt.txt` |
| `cargo clippy --all-targets --locked -- -D warnings` | rc 0 | `logs/merge-branch-clippy.txt` |
| `cargo test --locked` | rc 0: lib 332 passed, swch 0, `ipc_roundtrip` 6 passed (5.01 s), doc 0; 0 failed, 0 ignored | `logs/merge-branch-test.txt` |

## Gate on the merge commit (a1b5a27)

| Check | Result | Log |
|---|---|---|
| `cargo fmt --check` | rc 0 | `logs/merge-master-fmt.txt` |
| `cargo clippy --all-targets --locked -- -D warnings` | rc 0 | `logs/merge-master-clippy.txt` |
| `cargo test --locked` | rc 0: lib 332 passed, swch 0, `ipc_roundtrip` 6 passed (5.01 s), doc 0; 338 total, 0 failed, 0 ignored | `logs/merge-master-test.txt` |

The logs are in `D:/tmp/ks-orch/p0-A5-ipc/logs/`.

## Conflicts

None. `master` had not moved since the branch was cut, so CHANGELOG and README needed no resolution.

## Not done (by brief)

- No tags, releases or GitHub settings changes.
- The non-blocking items from `results.md` stay open for the orchestrator: P1 (the tester's Sonnet trailer on `2baadf1`, now in `master` history), P2 (no test pins the callback's fast gate), the new rustdoc warning at `src/hook/keyboard_hook.rs:132`, a README sentence, and the book-sync and manual-check backlog (M1, M2, M3, M5).
- `book.yml` deploys the book on push to `master`. The lane did not touch `book/src/**`.
