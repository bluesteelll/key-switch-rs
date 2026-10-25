# Merge: p2-2.3-control-window (Control window and state reset, KS-29, KS-31)

## Verdict: MERGED

- Merge commit: `3befec75f342197e2415e8e2bed13d16c43e6fe0` (`3befec7`) on `master`, pushed (`c3dab6a..3befec7  master -> master`).
- Lane branch `feat/control-window` (`81bb38c`) deleted with `git branch -d` (was fully merged).

## Preconditions

- Results-analyst verdict: `D:/tmp/ks-orch/p2-2.3-control-window/results.md` line 5, `## VERDICT: ACCEPT`.
- `feat/control-window`: `git status` reported "nothing to commit, working tree clean".
- `git fetch origin`: `master` = `origin/master` = `c3dab6a`. `git merge-base master feat/control-window` = `c3dab6a`, so `master` had not moved since the lane was cut.

## Gate on the lane branch (`81bb38c`)

| Check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0, no warnings |
| `cargo test --locked` | exit 0. lib: 679 passed, 0 failed, 7 ignored. `tests/ipc_roundtrip.rs`: 6 passed. bins and doc-tests: 0. Log: `D:/tmp/ks-orch/p2-2.3-control-window/merge_branch_test.txt` |

## Merge

- `git checkout master`, then `git merge --no-ff feat/control-window -m "Merge branch 'feat/control-window': Control window and state reset (KS-29, KS-31)" -m "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>"`.
- No conflicts. 32 files changed, 3975 insertions, 755 deletions. New files: `src/hook/control_window.rs`, `src/platform/desktop.rs`, `src/platform/foreground.rs`.
- The merged tree is the lane tree: `git diff feat/control-window master` is empty, and both trees are `d04505148ba5d39c1107ba2a6a8396fe137c52af`.
- Commit message:
  ```
  Merge branch 'feat/control-window': Control window and state reset (KS-29, KS-31)

  Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
  ```

## Gate on the merge commit (`3befec7`)

| Check | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 (`Compiling key-switch-rs ... Finished`), no warnings |
| `cargo test --locked` | exit 0. lib: 679 passed, 0 failed, 7 ignored. `tests/ipc_roundtrip.rs`: 6 passed. bins and doc-tests: 0. **685 passed / 7 ignored.** Log: `D:/tmp/ks-orch/p2-2.3-control-window/merge_master_test.txt` |

The 7 ignored tests are the `manual:`/`bench:` tests that results.md lists (line 21). They are not counted as passes.

## Push and cleanup

- `git push origin master`: `c3dab6a..3befec7  master -> master`. Afterwards `HEAD` = `origin/master` = `3befec7`.
- `git branch -d feat/control-window`: "Deleted branch feat/control-window (was 81bb38c)."
- `git worktree list` shows only the main work tree (`D:/key-switch-rs/key-switch-rs 3befec7 [master]`). The work tree is clean and on `master`.
- No tags, releases or settings changes.

## CI on the pushed master (the CI watch results.md:223 asks for)

- `CI` run `39219217631`: success in 3m10s. Steps passed: fmt, clippy, test, release build, the UAC manifest check, and the KS-38 harness check. Watch log: `D:/tmp/ks-orch/p2-2.3-control-window/merge_ci_watch.txt`. The test `the_control_window_is_a_hidden_top_level_window_released_on_drop`, which creates a real hidden top-level window and tries a WTS registration, passed on the runner.
- `Book` run `39219217484`: success in 26s, so the book is deployed to Pages.

## Open points carried forward (from results.md, not acted on at the merge step)

- The book sync after the merge: the audit status lines KS-06 / KS-29 / KS-31 and the roadmap 2.3 status, each with merge sha `3befec7`, plus the pages listed in results.md:179-184.
- review r2 m1 (Minor, owner phase: tester): the `MovingQuery` fake window values `0x1000`/`0x2000` in `main_thread.rs:1645-1688` can match a real PID.
- Owner hand-off: the timestamped V2-m1/R2 M1 log needs a live daemon. The R1 witness costs +0.6 us per swallowed key-up.
