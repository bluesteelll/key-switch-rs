# Independent check of the book sync after roadmap 2.5: coverage and integrity

13 findings: 0 critical, 0 important, 13 minor

- Role: project-analyst, read-only. Lens: coverage and integrity.
- Read: branch `docs/book-2.5`, head `24444e9` (24444e99d61a0043cec83b4a7c08a8d72cf93fa0), 5 commits on `master` `252daef`.
  The checkout stayed on `docs/book-2.5`, with a clean tree. Code facts come from `f4c1fb6`, whose source differs from
  `252daef` only in `src/engine/modifiers.rs` (2 doc-comment lines, `git diff --stat 252daef f4c1fb6 -- src`).
- Inputs: `git diff master...docs/book-2.5` (20 files under `book/src`, nothing else) and `git show docs/book-2.5:<path>`.
  Lane reports: `results.md:210-228`, `impl.md:293-371`, `cut.md:1511-1527, :1805-1828`, `test_r1.md`,
  `merge_gate_master.log`. The doc-writer's account `D:/tmp/ks-orch/book-2.5/book.md` was checked against these, not trusted.
- My own artefacts: the book build in `D:/tmp/ks-orch/book-2.5/check-book/`, its log in
  `D:/tmp/ks-orch/book-2.5/check-build.log`. The link and permalink checkers are mine, written fresh, not the
  doc-writer's tools: `chk_links.py`, `chk_perma.py` in the session scratchpad.

## Verdict per lens item

| # | Lens item | Result |
|---|---|---|
| 1 | Hand-off coverage (results.md "Hand-off: phase book sync", impl.md "Book pages ... left") | Every item is done or rightly marked moot. One cited line (architecture `:251`) got only part of what the hand-off asked (M8) |
| 2 | Stale text that 2.5 made false, in `book/src` and `README.md` | No stale current-tense claim remains. Every hit is qualified history or plan text. Small inaccuracies: M7, M9, M10 |
| 3 | `audit.md` changes only status lines | Yes: one line, the KS-14 **Status** line (`audit.md:409`). The findings are untouched |
| 4 | Build, internal links, anchors, SUMMARY | `mdbook build` exit 0, with only the mdbook-mermaid version warning. 20 pages, 2720 internal hrefs, 0 broken files, 0 missing anchors, 0 duplicate ids. `SUMMARY.md` registers all 20 `.md` files. Permalinks: 1787 checked, 0 bad (commit, path and line range), 768 of them to `f4c1fb6` |
| 5 | Manual-check rows 49-59 | All five cells are filled in every row, in the source and in the rendered HTML. Numbering runs 1-59 with no gap or duplicate. Every check-number reference outside the table points at the right row. Defects: M1, M2, M3, M12 |

## 1. Hand-off coverage

Line numbers in the hand-off are on `master` (`252daef`); I re-read each one on `master` and found its counterpart on the branch.

| Hand-off item | Source | Branch location | Status |
|---|---|---|---|
| roadmap `:586-595` 2.5 What/Acceptance/status: W1, W2/W2a, D12 withdrawn (L3), R16 gated, 52.6 -> 0.9 us | R, I | `roadmap.md:597-618` (status block, acceptance table, effect line); the plan text at `:588-595` is kept as for 2.1-2.4 | Done. Figures match `test_r1.md:182-218` (R12 4.7->0.2, R11 5.3->0.8, R12m 0.8, R12m2 1.4, R16 52.6->0.9, R18 4.5, R17t 1 ns). Commit list matches `git log f4c1fb6^1..f4c1fb6^2`. "Owner declined" matches `rulings.md:8-9` |
| roadmap `:584` 2.4's key-path line | R, I | `roadmap.md:584` | Done: 2.4's record is kept, and a "Since item 2.5" sentence is added |
| roadmap `:654` check 44 "about 50 us" | R, I | `roadmap.md:678` | Done: "about 50 us at item 2.4 and about 1 to 4 us since item 2.5", FU-4 "likely moot"; no D12 amendment |
| roadmap `:739` 3.5 | R, I | `roadmap.md:774` | Done: witness follow-up lapsed (W1); Q11, check 57, D10 property case, U1 fallback; FU-4 likely moot |
| roadmap `:952` OQ13 outcome | R, I | `roadmap.md:987` | Done |
| roadmap `:999` FU-4 row | R, I | `roadmap.md:1034` | Done |
| roadmap `:1005` | R, I | `roadmap.md:1040` (Q6 row: checks 43, 44 and 53) | Done |
| M1-M11 into "Manual checks after Phase 2", with M8/M10/M11 marked | R, I | `roadmap.md:683-693`, closing `:695` | Done. Each row matches `cut.md:1518-1527`, impl.md "Manual checks" (M5-M7 as written for 2.4, per L3; M9's latched step) and the M11 addendum (`cut.md:1805-1808`). Rows 56, 58 and 59 say "checks an assumption of item 2.5" |
| Checkpoint 2 | brief | `roadmap.md:627` item 4 (checks 49, 50, 56-59) | Done |
| architecture `:237` (step 8) | R, I | `architecture.md:237` | Done (matches `engine/mod.rs:557-566`, `keys.rs:118-143`) |
| architecture `:248` (step 15) | R, I | `architecture.md:248` | Done (key-up check `mod.rs:1000-1006`, `ModifierSet::MENU`) |
| architecture `:251` (invariant: D10's `RepeatSuppressed`) | R, I | `architecture.md:252` | **Partly**: only a W1 sentence was added; see M8 |
| architecture `:265` (mermaid) | R, I | `architecture.md:269` (CHK node) | Done |
| architecture `:294` (foreground: a bump and a wake) | R, I | `architecture.md:296` | Done (`keyboard_hook_callback.rs:475-503`) |
| architecture `:331`, `:350`, `:362` | R, I | `architecture.md:376`, `:396-399`, `:409` | Done (`RepeatSuppressed` row added) |
| architecture `:566-567`, `:605` | R, I | `architecture.md:613-614`, `:652` | Done |
| glossary `:67`, `:377`, `:569`, `:577`, new "Hiding count", no "Unapplied key" | R, I | `glossary.md:67`, `:381`, `:573`, `:581`, `:243-245`; `grep -i unapplied` finds nothing in `book/src` or `README.md` | Done |
| overview `:16` | R, I | `overview.md:16` | Done |
| gaps `:167`, `:300` | R, I | `gaps.md:167`; `:300` keeps the audit column and its status line `:304` is updated | Done (wording: M10) |
| Optional kinds.md §10 (D10 corner case) | R | `config/kinds.md`, new bullet in §10 citing `mod.rs:940-948` and the pinning scenario | Done |
| P1 README, P2 rustdoc | R | `28566a4`: `README.md:57` lists `resync.rs`, `README.md:154` the sentinel bullet | Moot, rightly (done in the chore) |
| P3 R16 figure per session | R | `architecture.md:345`, `roadmap.md:618` give both sessions; `README.md:958` is left, labelled "on the developer's machine" | Done (results.md says "may") |
| CLAUDE.md / tester.md / code-reviewer.md | R, I | `252daef:CLAUDE.md:74`, `tester.md:102` | Not for the book; done by the orchestrator, rightly marked |
| audit status for KS issues closed | brief | results.md "Issues claimed closed": none; KS-14 status sentence added | Correct |

Spot checks behind "Done": every test name that the added text cites (74 identifiers; the non-test hits are module paths)
appears as `ok` or `ignored` in `merge_gate_master.log`. Counts 869/862/7 ignored, 7 integration, 235 scenarios,
7 properties and 79 `hook::watchdog` tests match the log (`merge_gate_master.log:5, :876, :895`). The per-module table
of `build-and-run.md` matches the test names in the log (watchdog 55 + decision 24, executor 26 + runs 9). The first and
last lines of 16 new `f4c1fb6` ranges (keys.rs, resync.rs, mod.rs, main_thread.rs, callback, watchdog, counters,
writer, bench.rs) bracket the cited code.

## 2. Stale-text search (book/src and README.md on the branch)

| Pattern from the brief | Hits | Assessment |
|---|---|---|
| modifier reads / a resync on every key event | `roadmap.md:523`, `:535`, `:543` (2.3 plan and its status), `config/kinds.md:187` (KS-06/14 history), `gaps.md:167` and `:300` (audit-era columns), `architecture.md:309`, `roadmap.md:600`, `config/known-issues.md:157`, `README.md:958` (each says "until/before item 2.5") | None stale: each is history, a plan text, or an audit column with an updated status. Wording: M9 (`overview.md:128`) |
| R16 "about 50 us" | `roadmap.md:584`, `:593` (2.5 plan "Why"), `:678`, `:987`, `:1034`; `README.md:958`; `CHANGELOG.md:665` | All qualified "at item 2.4" or "before item 2.5", or plan text |
| R16 "declared, not gated" | `roadmap.md:584` only (2.4's record), followed by "R16 is gated" | Qualified |
| the witness reading on every swallowed key-up | `architecture.md:252`, `glossary.md:581`, `roadmap.md:599, :608, :987, :1031` | **Not stale: true after 2.5.** Ruling W1 kept the read: `src/engine/mod.rs:999` `decision.suppress = swallow && !env.os_key_down(vk);`. The only text claiming a skip is the 2.5 plan bullet (`roadmap.md:590`), which the status block (`:599`) marks "not done (W1)" |
| "Unapplied key" | none | Clean |
| FU-4 still open / witness-skip follow-up still open | FU-4: `roadmap.md:678, :695, :774, :1034, :1040`, each "likely moot"; witness skip: `:1031` "Closed, not taken up", `:774` "lapsed" | Covered (table consistency: M4) |
| also checked: "only vkCode and dwExtraInfo", "eleven counts", "passed on before the engine", "resyncs the modifiers" at a foreground change | `gaps.md:164` (audit column, status updated), `config/keys.md:178` (updated), `glossary.md:301`, `architecture.md:234` | Updated; `build-and-run.md:147` says twelve counts |

## 3. audit.md

`git diff master...docs/book-2.5 -- book/src/audit.md` changes one line, `audit.md:409`, the **Status** line of KS-14:
one sentence plus the link to the new architecture section. Its range `keys.rs:96-143` at `f4c1fb6` covers
`resync_modifiers` (L95-116) and `clear_released_modifiers` (L118-143), and both `continue` past `key_down_swallowed`.
The three tests it names pass in `merge_gate_master.log`. Wording: M10.

## 4. Build, links, anchors, SUMMARY

- `mdbook build book -d D:/tmp/ks-orch/book-2.5/check-book`: exit 0. The only warning is mdbook-mermaid "built against
  0.5.3 ... called from 0.5.2" (`check-build.log`).
- My HTML link checker over every page except `print.html`, `toc.html` and `404.html` found: pages 20, internal hrefs
  2720, broken 0 (missing file or missing anchor), duplicate ids 0. The doc-writer counted 2624 with another tool.
- `SUMMARY.md` on the branch is unchanged, and it lists all 20 `.md` files of `git ls-tree docs/book-2.5 book/src`.
- Permalinks: all 1787 `github.com/.../blob|tree|commit/<sha>` links resolve. Every commit exists, every path exists at
  its commit, and every `#Lx-Ly` lies inside the file. 768 point to `f4c1fb6`.
- Mermaid: none rendered (no `mmdc`; nothing installed). I scanned every block of the 8 pages that have one. No node
  id is followed by bare text, which was the `C0code below zero?` defect that master has at `252daef:architecture.md:257,
  :260, :262` and this branch fixes (`architecture.md:258, :261, :263`).

## 5. Manual-check rows 49-59

- **Cells.** I split the source on unescaped `|`. Rows `roadmap.md:683-693` have 5 cells each, and none is empty. The
  rendered `roadmap.html` gives 5 `<td>` for rows 44-59, so the escaped `\|` of row 56 renders correctly.
- **Numbering.** The Phase 2 table runs 1..59 at `roadmap.md:635-693`, with no gap and no duplicate. The Phase 0 (1-13)
  and Phase 1 (1-18) tables are separate.
- **References.** Outside the roadmap: `architecture.md:250` and `build-and-run.md:489` cite check 56 (the clock base);
  `known-issues.md:157` cites 57 (Sticky Keys) and `:158` cites 56. Older references are unchanged (`:149` checks 43/44,
  `:151` check 38, `:152` checks 41/42, `gaps.md:131` Phase 1 check 11). Inside the roadmap, the new references point
  at the right rows: `:601` (44 and 53), `:602` (56), `:609` (49-51), `:612` (49-52), `:614` (58, 59), `:615` (56),
  `:627` (49, 50, 56-59), `:774` (57, 56), `:1040` (43, 44, 53) and `:1043-1044` (56, 57). The old rows were not
  renumbered, because the new rows are appended.

## Findings (each Minor)

**M1. The intro's list of debug-build rows omits row 53.** `roadmap.md:631` says "Rows 35, 38, 44, 49 and 52 use a
debug build, which needs the release daemon stopped first". Row 53 (`:687`) is "As check 44, with a debug build of
`f4c1fb6` or later". The sync extended this sentence from master's "Rows 38 and 44" and missed 53. Row 46 ("The debug
daemon of check 38", `:680`) was already missing on master.

**M2. Row 59's Setup does not stop a running daemon, so the check can pass without testing its assumption.**
`roadmap.md:693` says "PowerToys Keyboard Manager started first ...; then `swch open`, so that PowerToys' hook runs after
the daemon's". Row 58 (`:692`) begins with `swch exit`; row 59 does not. If the daemon from check 58 is still running,
`swch open` only answers "already running" (check 45). PowerToys' hook, installed later, is then called first, and the
daemon sees only the injected F13. The binding then fires whether or not the D16 check works, so the row passes even
if the V3-I1 path is broken. The source has the same gap (`impl.md` M11, `cut.md:1805-1807`). Of the 13 findings, this
one has the largest effect.

**M3. The closing paragraph leaves out the re-runs.** `roadmap.md:695` says "Checks 43 and 44 also decide follow-up FU-4,
and check 47 decides whether item 2.4 needs its kernel-wait fallback". The same sync names "Manual checks 43, 44 and 53"
for the same question (`:1040`), and row 55 re-runs 47 (`:689`).

**M4. A Closed row still names its candidate item.** `roadmap.md:1031` marks the witness-skip follow-up "**Closed**, not
taken up", but its Candidate column still says `[3.5]`. The other Closed rows of the table say "done in 2.3", "done in
2.4" or "done" (`:1023-1025`, `:1033`). The table intro says "Rows marked **Closed** were done by a later lane of the
phase or by its book sync" (`:1011`), which does not fit a follow-up that was not taken up.

**M5. New rows break the table's ordering rule.** `roadmap.md:1011` says "The unverified points at the end each name
the manual check that settles them". Five of the six new rows are no "Not verified" point: Q11, the D10 property
coverage, the U1 fallback, the sentinel note and the flaky test. They sit inside the "Not verified" block, at
`:1041-1043` and `:1045-1046`, between `:1040` and `:1047`.

**M6. The R12m2 margin reads as a pass.** `roadmap.md:611` says "R12m2, two held modifiers, met its new bound only
within 9 ns, under the 100 ns resolution of a p50". The tester found a "formal excess of 9 ns" (1.4 against a bound of
1.391 us; `test_r1.md:216`), not a margin. Accurate wording: it exceeded the bound by 9 ns, below the p50's 100 ns grid,
so the excess is not counted as a breach. `results.md:57` has the same ambiguous wording.

**M7. "Once per move" survives in the roadmap.** `roadmap.md:618` says "The main thread's read between key events costs
4.5 us (R18), once per move of the hiding count". The doc-writer's self-check lists "a drain read runs once per move"
as corrected (`book.md`, "Claims removed"), and `architecture.md:345` was fixed ("with a short second read for the codes
it owed"). The roadmap was not. Code: `Resync::plan` leaves fresh codes out and reads them as `owed` at the next drain
(`src/engine/resync.rs:98-116` at `f4c1fb6`).

**M8. The suppression invariant sentence does not cover `RepeatSuppressed` (hand-off `:251`).** `architecture.md:252`
says "a key-up is suppressed if and only if its key-down was, unless Windows has the key down at the key-up", and gains
only a W1 sentence. Since D10 a key-up is also swallowed when the engine never saw the key-down but swallowed a repeat
(`RepeatSuppressed`; `mod.rs:940-948`, `:986-999`). Steps 13-14 (`:246-247`), the per-key table (`:399`) and the second
diagram (`:288`) say so, and the glossary's Suppression record (`glossary.md:497`) words the invariant with "or that
repeat". The architecture sentence alone is literally incomplete in that corner. CLAUDE.md principle 2 uses the same
wording.

**M9. The changes table contradicts itself.** `overview.md:128` (the `f4c1fb6` row) says "No key event reads the
modifiers from Windows: ..., a key event checks only the modifiers it holds". The check is a `GetAsyncKeyState` per
held modifier (`architecture.md:237`; `keys.rs:132-140`). Elsewhere the book says "no key event reads them all"
(`gaps.md:167`) or "no key event reads every modifier" (`audit.md:409`).

**M10. "A decided key event" drops more than it does.** `audit.md:409` (KS-14 status) and `gaps.md:167` say "a decided
key event drops a held modifier that Windows has up". In fact a key-down checks every held modifier
(`mod.rs:557-566`), a swallowed key-up only Alt and Win (`ModifierSet::MENU`, `mod.rs:1000-1006`), and a key-up that
passes checks nothing. `architecture.md:311` states this precisely.

**M11. One claim has two different ranges.** `glossary.md:301` (KBDLLHOOKSTRUCT) cites
`keyboard_hook_callback.rs:422-427` for "copies four of them ... `vkCode` and `dwExtraInfo` for the decision". At
`f4c1fb6`, lines 422-427 are the SAFETY comment, the `event` cast, `stamp` and `injected`. `vkCode` and `dwExtraInfo` are
read at L430 (`decide_event(hook, wparam, event.vkCode, event.dwExtraInfo)`). `architecture.md:234` and
`known-issues.md:42` cite `L425-L430`, which includes that line.

**M12. Row 57 blames every layout switch on issue 89.** `roadmap.md:691` (Expected, step 1) says "a switch of the layout
on this daemon is [known issue 89]". Known issue 89 (`known-issues.md:157`) is the case where Windows reports the latch
only in its key state, which shows as a difference from a daemon from before item 2.5. If the older daemon also switches
the layout, the latch is not visible to the daemon in either form, and the cause is not issue 89. The row's next
clause, "a difference from the older daemon goes to item 3.5", says this, so the impact is small.

**M13. The status block's lead-in differs from its siblings.** `roadmap.md:597` opens with "**Status after Phase 2: done**
in ...", while the blocks of 2.1-2.4 open with "**Status: done** in ..." (`:460`, `:499`, `:537`, `:567`).

## Doc-writer's account (`book.md`): verified or not

- The account is accurate on: pages changed; hand-off dispositions 1-24 (apart from M8 on item 12's `:251`); no change to
  `src/`, `README.md`, `CHANGELOG.md` or `.claude/`; SUMMARY unchanged; test counts; 768 permalinks to `f4c1fb6`; 0
  broken links; all five commits carry the `Co-Authored-By: Claude Opus 5.5` trailer.
- One self-check claim is only partly true: "a drain read runs once per move" was corrected in the architecture chapter
  but not in the roadmap (M7).
- Not re-verified by me: the cfgcheck RON runs (32 PASS) and the Microsoft Learn fetches.

## Positives

- The key-path flowcharts, broken on master (`252daef:architecture.md:257-263`, the decision nodes had lost their
  `{"..."}`), are written again with correct node syntax. No other mermaid block anywhere in the book has that defect.
- Bench figures are given with their session, both the tester's and the developer's, and match `test_r1.md` exactly. The
  R11 acceptance line is marked "Not met, by ruling W1" rather than glossed over.
- The plan texts of 2.4 and 2.5 are kept as records, and each change is added as a dated, qualified sentence. History
  and current state stay apart.
- Every new permalink resolves, and every cited test exists and passes in the merge gate log.

## Not covered

- Rendering the mermaid diagrams in a browser.
- The RON validation of the doc-writer (cfgcheck) and the external Microsoft Learn claims (`GetMessageTime`, `STICKYKEYS`).
- A line-by-line accuracy review of every re-pinned permalink. I checked structure for all 1787 and content for 16
  2.5-specific ranges.
- Accuracy of untouched config pages beyond the stale-text patterns above.
- `CHANGELOG.md`, which is outside the brief.
