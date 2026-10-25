# Merge: p0-A2-matching

Verdict: MERGED

## Preconditions
- results-analyst verdict: `## VERDICT: ACCEPT` (`D:/tmp/ks-orch/p0-A2-matching/results.md:5`).
- `fix/matching` @ `c8ca4a5`, working tree clean (`git status`: nothing to commit; only ignored `target/`, `book/book/`, `.claude/settings.local.json`).
- `master` = `origin/master` = merge-base = `e67e5cb` after `git fetch origin`.

## Gate on the lane branch (`c8ca4a5`)
- `cargo fmt --check`: exit 0
- `cargo check --all-targets --locked`: exit 0
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0, 0 warnings
- `cargo test --locked`: exit 0; lib `263 passed; 0 failed; 0 ignored`, swch bin 0, doc-tests 0.
  Test names identical to the analyst's run (`analyst-test.txt`), 263 names incl. two `should panic` tests.
- Logs: `D:/tmp/ks-orch/p0-A2-matching/merge/lane-{fmt,check,clippy,test}.txt`

## Merge
- `git checkout master`, `git merge --no-ff fix/matching`, ort strategy, no conflicts.
- Message: `Merge branch 'fix/matching': Matching: trigger-key rule, exact modifiers, sided keys`, blank line, `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Merge commit `82f542f55d68a1d2ba352f46d2e2900085e856e7`, parents `e67e5cb` `c8ca4a5`.
- 20 files, +2434 / -324. Tree `bc41749` is identical to the `fix/matching` tree (master was an ancestor).

## Gate on the merge commit (`82f542f`)
- `cargo fmt --check`: exit 0
- `cargo check --all-targets --locked`: exit 0
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0, 0 warnings; re-run after `touch src/lib.rs src/main.rs src/bin/swch.rs` (fresh `Checking key-switch-rs`), exit 0
- `cargo test --locked`: exit 0; lib `263 passed; 0 failed; 0 ignored`, swch bin 0, doc-tests 0; name set identical to the lane run
- mdbook: not run; the lane changes no file under `book/`
- Logs: `D:/tmp/ks-orch/p0-A2-matching/merge/master-{fmt,check,clippy,clippy-fresh,test}.txt`

## Publish
- `git push origin master`: `e67e5cb..82f542f  master -> master`
- `git branch -d fix/matching`: deleted (was `c8ca4a5`)
- After `git fetch`: `master` = `origin/master` = `82f542f`; tree clean
- No tags, releases or settings changes

## Open points carried from results.md (not acted on at the merge step)
- P1: `965ac4d` carries a `Claude Sonnet 5.5` trailer; it cannot be fixed without rewriting history. The tester brief needs the literal trailer.
- P2: README/CHANGELOG/`src/engine/mod.rs` rule 5 overclaim the release after a focus change for a key with a pending Tap/Hold (docs only, for doc-writer / roadmap 2.3).
- Book sync is still pending (cut.md "Docs impact" plus the impl.md KS-06 rows); the published book still describes side folding and superset matching.
- CLAUDE.md "Layout" (`key_spec.rs`, `matching.rs`) and code-reviewer R8 are stale after this merge (cut Q4, orchestrator).
- Owner FYI: with a payload key, only that key triggers (CapsLock then Shift does not fire `Shift+CapsLock`).
- Pre-existing `cargo doc` warning `src/data/vk_name.rs:4` (unclosed `<raw>`), untouched.
