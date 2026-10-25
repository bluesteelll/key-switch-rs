# Merge: p1-1.4-hotkey-tristate (roadmap 1.4, KS-17)

## VERDICT: MERGED

- Merge commit: `f0048a3b9f5da5e07234a390b52304e875bdfebd` on `master`, pushed (`f28da35..f0048a3 master -> master`).
- `origin/master` after fetch: `f0048a3b9f5da5e07234a390b52304e875bdfebd`.
- Lane branch `fix/language-hotkey-tristate` deleted locally (was `9fcbc93`). It was never pushed, so there is no remote branch to delete.

## Preconditions
- Results-analyst verdict: `D:/tmp/ks-orch/p1-1.4-hotkey-tristate/results.md`, `## VERDICT: ACCEPT`.
- Lane branch clean (`git status`: nothing to commit, working tree clean), HEAD `9fcbc93`.
- `master` == `origin/master` == merge-base == `f28da35` (after `git fetch origin`). `master` had no commits outside the lane branch, so no conflicts were possible.
- Lane commits merged (5):
  - `06f385d` Tests: loader tests that count auto-blockers no longer read the registry (1.4)
  - `5493b3a` System: the language hotkey parser returns Combo, Disabled or Unknown; the resolution takes injected values (1.4, KS-17)
  - `8ef0ab7` System: red-first: a language hotkey not assigned in Windows gives no hotkey and no auto-blocker (1.4, KS-17)
  - `38029df` System: "3" (not assigned) decides the language hotkey: no auto-blocker, no Alt+Shift fallback (1.4, KS-17)
  - `9fcbc93` Book: the language hotkey not assigned in Windows in the config reference (1.4, KS-17)

## Gate on the lane branch (HEAD 9fcbc93)
| Check | Result |
|---|---|
| cargo fmt --check | exit 0 |
| cargo clippy --all-targets --locked -- -D warnings | exit 0 (log `merge_branch_clippy.log`) |
| cargo test --locked | exit 0; lib 479 passed, 0 failed, 3 ignored; ipc_roundtrip 6 passed; swch 0; doc-tests 0 (log `merge_branch_test.log`) |

## Merge
- `git checkout master`, `git merge --no-ff fix/language-hotkey-tristate`, ort strategy, no conflicts.
- Message: `Merge branch 'fix/language-hotkey-tristate': Tri-state language hotkey (KS-17)`, blank line, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- 9 files changed, 380 insertions, 64 deletions: `CHANGELOG.md`, `README.md`, `book/src/config/{errors,example,known-issues,matching}.md`, `src/config/default_config.ron`, `src/config/loader.rs`, `src/system/registry.rs`.
- `git diff 9fcbc93 f0048a3` is empty: the merge tree equals the lane tree.

## Gate on the merge commit (f0048a3)
| Check | Result |
|---|---|
| cargo fmt --check | exit 0 |
| cargo clippy --all-targets --locked -- -D warnings | exit 0 (log `merge_master_clippy.log`) |
| cargo test --locked | exit 0; lib 479 passed, 0 failed, 3 ignored; ipc_roundtrip 6 passed; swch 0; doc-tests 0 (log `merge_master_test.log`) |

## Manual checks (ignored, not counted as passes)
- `data::condition::tests::foreground_context_names_the_focused_notepad` (pre-existing)
- `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream` (pre-existing)
- `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (new in this lane): with Windows set to "Not assigned" for the input language switch, a human checks that the live registry gives no combination and that the daemon adds no auto-blocker.

## Push and cleanup
- `git push origin master`: `f28da35..f0048a3`, exit 0.
- `git branch -d fix/language-hotkey-tristate`: exit 0.
- No tags, releases or settings changes.

## Open points
- From results.md: book status lines outside `book/src/config/` (audit.md KS-17 status, roadmap 1.4 status) are owned by the Phase-1 book sync.
