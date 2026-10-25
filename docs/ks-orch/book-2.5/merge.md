# Merge step: book sync after roadmap 2.5
Verdict: MERGED
Merge commit: 1c1f3a0fc5dabdc2a7332e239325df5296b04a0a (`git merge --no-ff docs/book-2.5` into `master`)

## Preconditions
- `origin/master` and `master` were 252daef05a583bcc3f55bc427a4d71380a0c2ea6 at fetch time, as the brief expected.
- `docs/book-2.5` head 2a942a9 sat 6 commits on 252daef (d54de8f, 36e35df, 88aa498, d7e42b6, 24444e9, 2a942a9).
  The diff touched `book/src/**` and `README.md` only (21 files, +736/-623). Nothing under `src/`.
- Sync verdict (book.md): DOCS READY WITH GAPS. Delta re-check (recheck.md): 0 critical, 0 important, 3 minor
  (N1 glossary.md:67, N2 glossary.md:497, N3 roadmap.md:691 check 57). These are open and carried as follow-up.
- Merge message: "Merge branch 'docs/book-2.5': book sync after roadmap 2.5", a blank line, and
  "Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>". No conflicts.

## Gate on the merge commit
| Gate | Command | Result | Log |
|---|---|---|---|
| 1 | `cargo fmt --check` | exit 0 | gate_fmt.log |
| 2 | `cargo clippy --all-targets --locked -- -D warnings` | exit 0, no warnings | gate_clippy.log |
| 3 | `cargo test --locked` | exit 0 | gate_test.log |
| book | `mdbook build book -d D:/tmp/ks-orch/book-2.5/book-final` | exit 0 | build_final.log |

`test result:` lines (gate_test.log):
- lib: ok. 862 passed; 0 failed; 7 ignored (manual/bench, not counted as passes)
- bin key-switch-rs: ok. 0 passed
- bin swch: ok. 7 passed
- doc-tests: ok. 0 passed

mdbook: only the known mdbook-mermaid warning ("built against version 0.5.3 of mdbook, ... called from version
0.5.2"), the local machine setup. CI pins mdbook 0.5.3.

## Push
```
To https://github.com/bluesteelll/key-switch-rs
   252daef..1c1f3a0  master -> master
```
`git branch -d docs/book-2.5`: Deleted branch docs/book-2.5 (was 2a942a9).
The push touches `book/**`, so `book.yml` should redeploy Pages; the deploy run was not observed here.
No tag, no release. The checkout is left on `master` at 1c1f3a0, clean.

## Follow-up
- Recheck minors N1-N3 (glossary.md:67, glossary.md:497, roadmap.md:691) for the next book sync.
- Mermaid flowcharts of architecture.md were rewritten but never rendered; look at the published page once Pages has deployed.
