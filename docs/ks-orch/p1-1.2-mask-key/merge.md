# Merge: p1-1.2-mask-key (feat/mask-key -> master)

## Verdict: MERGED

- Merge commit: `f28da35857479a8d5076bab5b50d498af08d4703` on `master`, pushed (`ece1767..f28da35  master -> master`).
- Parents: `ece1767` (master) + `0187cde` (feat/mask-key, 9 commits).
- Message: `Merge branch 'feat/mask-key': Mask key and auto-blocker v2 (KS-12, KS-05, KS-35)`, blank line, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Lane branch `feat/mask-key` deleted locally (`git branch -d`, was 0187cde). No remote lane branch existed.

## Preconditions
- results-analyst verdict: `D:/tmp/ks-orch/p1-1.2-mask-key/results.md` = ACCEPT.
- `feat/mask-key` @ `0187cde`: `git status` clean, single worktree.
- `master` = `origin/master` = merge-base = `ece1767` after `git fetch origin`: no upstream drift, no conflicts.

## Gate on feat/mask-key (0187cde)
- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0, no warnings (`merge-branch-clippy.txt`)
- `cargo test --locked`: exit 0; lib 470 passed / 0 failed / 2 ignored, swch bin 0, `tests/ipc_roundtrip.rs` 6 passed, doc-tests 0 (`merge-branch-test.txt`)

## Merge
- `git merge --no-ff feat/mask-key`: ort strategy, no conflicts. 24 files, +1749 / -236.
- `git diff feat/mask-key HEAD`: empty (merge tree identical to the lane tree, since master had not moved).

## Gate on merge commit (f28da35)
- `cargo fmt --check`: exit 0
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0, no warnings (`merge-master-clippy.txt`)
- `cargo test --locked`: exit 0; lib 470 passed / 0 failed / 2 ignored, swch bin 0, `tests/ipc_roundtrip.rs` 6 passed, doc-tests 0 (`merge-master-test.txt`). Total 476 passed.
- Ignored (manual, not counted as passes):
  - `data::condition::tests::foreground_context_names_the_focused_notepad` (pre-existing)
  - `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream` (new: inject vk 0xE8 down+up with Notepad focused; nothing typed, no menu opens)

## Push / cleanup
- `git push origin master`: ok, `HEAD` = `origin/master` = `f28da35`.
- `git branch -d feat/mask-key`: ok.
- No tags, releases or settings touched.

## Open points (owner manual checks, from results.md / cut.md)
- M1: Alt then Shift in Notepad does not open the menu.
- M2: Win+Q -> PostMessage does not open Start.
- M3: Alt+Tab switcher with E8 (failure = RETHINK by prior agreement).
- M4/M5 (v2, `block_original_combo: MaskOnly`): Ctrl+Shift+T works; a bare Alt+Shift does not switch the language. If they fail, `true` stays the default.
- M8: the ignored injection test above.
- `book.yml` will redeploy the book from this push.
