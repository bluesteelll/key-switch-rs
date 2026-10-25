# Merge: p0-A6-appequals

## Verdict: MERGED

- Merge commit: `4177540fa939feec709a88e98af20db8ae2f39b4` on `master`, pushed (`a1b5a27..4177540  master -> master`).
- Message: `Merge branch 'fix/app-equals': AppEquals via QueryFullProcessImageNameW`, blank line, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Lane branch `fix/app-equals` (`32703a8`) deleted locally with `git branch -d` (it never existed on the remote).

## Preconditions

- Results-analyst verdict: `## VERDICT: ACCEPT` (`D:/tmp/ks-orch/p0-A6-appequals/results.md:7`).
- `fix/app-equals` @ `32703a8`: working tree clean, `git worktree list` shows only the main tree.
- After `git fetch origin`: `master` = `origin/master` = merge-base = `a1b5a27`. No upstream drift.

## Gate on the lane branch (`32703a8`)

| Check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0, no warnings |
| `cargo test --locked` | exit 0: lib 338 passed / 0 failed / 1 ignored; `tests/ipc_roundtrip.rs` 6 passed; swch and doc-tests 0 |

## Merge

- `git checkout master`, `git merge --no-ff fix/app-equals`: ort strategy, no conflicts (master was the merge-base).
- Files: `CHANGELOG.md` (+23), `Cargo.toml` (-1), `README.md` (11), `src/config/default_config.ron` (4), `src/data/condition.rs` (333), `src/engine/scenarios.rs` (+37); 351 insertions, 58 deletions.
- `git diff --stat fix/app-equals HEAD`: empty (merge tree identical to the lane tree).

## Gate on the merge commit (`4177540`)

| Check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0, no warnings |
| `cargo test --locked` | exit 0: lib 338 passed / 0 failed / 1 ignored; `tests/ipc_roundtrip.rs` 6 passed; swch and doc-tests 0. Total 344 passed |

Manual ignore (not run, not counted as a pass): `data::condition::tests::foreground_context_names_the_focused_notepad`
(a human focuses Notepad and checks that `ForegroundContext` names `notepad.exe`). The CHANGELOG `[Unreleased]` entry carries
the manual-check note for `AppEquals("notepad.exe")`.

## Push and cleanup

- `git push origin master`: exit 0; after `git fetch`, `HEAD` = `origin/master` = `4177540`.
- `git branch -d fix/app-equals`: exit 0. Working tree clean on `master`.
- No tags, releases or GitHub settings touched.

## Open points (carried from results.md, for the orchestrator)

- `results.md` "Notes for later lanes" (N1, N2): two optional Minor items, the book sync list, a CHANGELOG classification
  ruling, owner FYIs.
- Book: untouched by the lane and deferred to the phase book sync (cut Q1, `docs/WORKFLOW.md:104-107`). Until that sync runs,
  the published pages still say `AppEquals` never matches (known drift, `results.md:86-89`). `book.yml` redeploys the
  unchanged book on this push.
