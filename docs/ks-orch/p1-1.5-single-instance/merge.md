# Merge: p1-1.5-single-instance (roadmap 1.5, KS-20, A5-m1)

## Verdict: MERGED
`master` @ `1058ddb4c5a1da3b2f9fe358efeb9eac6ef2843e`, pushed to `origin/master` (`f0048a3..1058ddb`).

## Preconditions
- Results-analyst verdict: `D:/tmp/ks-orch/p1-1.5-single-instance/results.md:3` `## VERDICT: ACCEPT`. It names `2a91ec6` as the merge target.
- `feat/single-instance` @ `2a91ec67c4061a8b2a1009920cc006b2b64a1eed`. `git status` showed a clean tree.
- `master` = `origin/master` = merge base = `f0048a3` (fetched before the merge). There was no divergence, so no conflicts were possible.

## Gate on the lane branch (`2a91ec6`)
`CARGO_TARGET_DIR=D:/tmp/ks-orch/p1-1.5-single-instance/target-merge` was a fresh directory. It avoids the work tree's `target/debug/incremental` ICE that the tester reported.
- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0, no warnings (`merge_branch_clippy.log`)
- `cargo test --locked`: exit 0 (`merge_branch_test.log`)
  - lib: 492 passed, 0 failed, 4 ignored
  - swch: 0
  - `ipc_roundtrip`: 6 passed
  - doctests: 0

## Merge
- `git checkout master`
- `git merge --no-ff feat/single-instance` with the message "Merge branch 'feat/single-instance': Single daemon instance (KS-20)", a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Strategy `ort`, no conflicts. The merge commit `1058ddb` has parents `f0048a3` and `2a91ec6`.
- `git diff --stat feat/single-instance HEAD` is empty, so the merge tree equals the branch tree.
- Files: CHANGELOG.md, Cargo.toml, README.md, src/core/app.rs, src/hook/ipc_server.rs, src/hook/keyboard_hook_callback.rs, src/main.rs, src/system/mod.rs, src/system/single_instance.rs (new), tests/ipc_roundtrip.rs. That is 667 insertions and 98 deletions.

## Gate on the merge commit (`1058ddb`)
- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0, 0 warning/error lines (`merge_master_clippy.log`)
- `cargo test --locked`: exit 0 (`merge_master_test.log`)
  - lib: 492 passed, 0 failed, 4 ignored
  - swch: 0
  - `ipc_roundtrip`: 6 passed
  - doctests: 0
- Work tree clean afterwards.

## Publish
- `git push origin master`: `f0048a3..1058ddb  master -> master`
- `git branch -d feat/single-instance`: deleted (was `2a91ec6`)
- After a fetch, `master` = `origin/master` = `1058ddb`.
- Not done: no tags, releases or settings changes.

## Manual checks still open (not counted as passes)
- M1/M1b: a second daemon start in the same session prints "already running" and exits with code 2. The mutex is taken before the config loads. This is also Checkpoint 1 item 9 (`book/src/roadmap.md`).
- M4: the pipe is created before either hook is installed. A pipe that cannot be created is a clean fatal exit with no hook installed.
- 4 `#[ignore = "manual: ..."]` tests, one of them new: `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start`.

## Open points carried from results.md
- The acceptance check for roadmap 1.7, `rg "println!|eprintln!"`, must allow the already-running line (cut risk 15). It is not yet in the text of `roadmap.md` 1.7.
- The 4 `cargo doc --no-deps` warnings are pre-existing, in files this lane did not touch.
