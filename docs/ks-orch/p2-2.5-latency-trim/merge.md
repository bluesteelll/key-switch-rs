# Merge: p2-2.5-latency-trim

VERDICT: MERGED. master is at `f4c1fb69d278bd8ca70f60738dc0c5f19d2bcc1b` and has been pushed to origin.

## Preconditions
- Results-analyst verdict: `results.md` says `## VERDICT: ACCEPT` (HEAD `d853a59`, master `3cde1d0`).
- `perf/latency-trim` at `d853a59`, working tree clean. The merge base is master `3cde1d0`, which also equalled
  `origin/master` (`git fetch --dry-run` showed nothing new).

## Gate on perf/latency-trim (d853a59)
- `cargo fmt --check`: pass (exit 0)
- `cargo check --all-targets --locked`: pass
- `cargo clippy --all-targets --locked -- -D warnings`: pass (exit 0)
- `cargo test --locked`: pass (exit 0). lib: 862 passed, 0 failed, 7 ignored. swch: 0. ipc_roundtrip: 7 passed.
  Doc-tests: 0. Log: `D:/tmp/ks-orch/p2-2.5-latency-trim/merge_gate_branch.log`.

## Merge
- `git checkout master`, then `git merge --no-ff perf/latency-trim`, which made `f4c1fb6` with the ort strategy and
  no conflicts. 24 files changed, +3326/-295, and `src/engine/modifiers.rs` and `src/engine/resync.rs` are new.
- Message: `Merge branch 'perf/latency-trim': Latency trim: modifier resync off the key path, the watchdog's first-key wake`,
  a blank line, and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- `git diff perf/latency-trim master` is empty: the merge tree is identical to the lane tree.

## Gate on the merge commit (f4c1fb6)
- `cargo fmt --check`: pass (exit 0)
- `cargo clippy --all-targets --locked -- -D warnings`: pass (exit 0)
- `cargo test --locked`: pass (exit 0). lib: 862 passed, 0 failed, 7 ignored. swch: 0. ipc_roundtrip: 7 passed.
  Doc-tests: 0. Log: `D:/tmp/ks-orch/p2-2.5-latency-trim/merge_gate_master.log`.
- The 7 ignored tests were not run and are not counted as passes. One is the bench, `bench::latency_bench`. The other
  six are manual checks: `the_win32_runner_opens_a_url`, `hold_the_daemon_pipe_name_for_a_manual_start`,
  `the_menu_mask_key_is_inserted_into_the_input_stream`, `input_is_on_this_desktop_while_a_human_uses_it`,
  `the_foreground_cache_names_the_focused_notepad` and
  `live_registry_with_the_language_hotkey_not_assigned_has_no_combination`.

## Push and cleanup
- `git push origin master`: `3cde1d0..f4c1fb6  master -> master`.
- `git branch -d perf/latency-trim`: deleted (it was `d853a59`).
- I made no tags, releases or settings changes.

## Open points
- The local branch `chore/rust-analyzer-lsp` still exists. It is not this lane's branch: it was merged into master
  earlier, in `3cde1d0`. I left it alone.
- The orchestrator still has the hand-off items from `results.md`: the two Minor README remarks, the two new rustdoc
  warnings, the CLAUDE.md and agent-file chore (rulings Q5/Q9), and the phase book sync.
