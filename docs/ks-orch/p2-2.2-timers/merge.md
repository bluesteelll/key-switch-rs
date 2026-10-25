# Merge: p2-2.2-timers (Timers on the main thread, KS-11, KS-25, KS-33)

## Verdict: MERGED

- Merge commit on `master`: `283d8aac1c06dccbbd19e3fad3780c927b625d1b`
- Subject: `Merge branch 'feat/main-thread-timers': Timers on the main thread (KS-11, KS-25, KS-33)`, blank line, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`
- Pushed: `c9c1373..283d8aa  master -> master` to `https://github.com/bluesteelll/key-switch-rs`; `origin/master` = `283d8aa`
- Lane branch deleted locally: `feat/main-thread-timers (was bfe931f)`. It was never pushed, so there is no remote branch to remove.

## Preconditions

- Results-analyst verdict: `results.md` line 3, `## VERDICT: ACCEPT` (HEAD `bfe931f`).
- `git status` on `feat/main-thread-timers`: `nothing to commit, working tree clean`.
- `master` = `origin/master` = `c9c1373` after `git fetch origin`, which is also the merge base. The lane is 19 commits ahead and 0 behind. One worktree, no other.

## Gate on the lane branch (HEAD bfe931f)

| Check | Result | Log |
|---|---|---|
| `cargo fmt --check` | exit 0 | `merge-branch-fmt.txt` |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 | `merge-branch-clippy.txt` |
| `cargo test --locked` | exit 0. lib: `running 636 tests`, `630 passed; 0 failed; 6 ignored`; `tests/ipc_roundtrip.rs`: `6 passed`; swch bin and doc-tests: 0 | `merge-branch-test.txt` |

## Merge

- `git checkout master`, then `git merge --no-ff feat/main-thread-timers`: the 'ort' strategy, no conflicts. 31 files, 4802 insertions, 1147 deletions. New files: `src/bench.rs`, `src/engine/schedule.rs`, `src/hook/main_thread.rs`, `src/platform/priority.rs`.
- `git diff feat/main-thread-timers master` is empty: the merge tree is identical to the lane tree.

## Gate on the merge commit (283d8aa)

| Check | Result | Log |
|---|---|---|
| `cargo fmt --check` | exit 0 | `merge-master-fmt.txt` |
| `cargo clippy --all-targets --locked -- -D warnings` | exit 0 | `merge-master-clippy.txt` |
| `cargo test --locked` | exit 0. lib: `running 636 tests`, `630 passed; 0 failed; 6 ignored`; `tests/ipc_roundtrip.rs`: `6 passed`; swch bin and doc-tests: 0 | `merge-master-test.txt` |

The 6 ignored tests were not run, so none counts as a pass:
- `bench::latency_bench` (`bench:` reason, counted apart, ruling Q5)
- `core::executor::tests::the_win32_runner_opens_a_url` (manual)
- `data::condition::tests::foreground_context_names_the_focused_notepad` (manual)
- `hook::ipc_server::tests::hold_the_daemon_pipe_name_for_a_manual_start` (manual)
- `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream` (manual)
- `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (manual)

## Not done (outside the merge step)

- No tag, release or GitHub settings change.
- The post-merge orchestrator updates from ruling Q1 (CLAUDE.md and the agent files describing `hook::main_thread::MAIN` and the removal of `Mutex<Engine>`/`with_engine`) belong to the orchestrator.
- The Minor hand-offs in `results.md` (reviewer m2, OQ3, one sentence of config-page drift, the Phase-2 book sync, H-B1, H-B5) belong to the orchestrator and later lanes.
