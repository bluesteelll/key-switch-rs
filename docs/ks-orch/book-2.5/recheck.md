3 findings: 0 critical, 0 important, 3 minor

# Re-check of the book sync after roadmap 2.5: fix round 2a942a9

- Role: project-analyst, read-only. Lens: a delta re-check after the fix round. Nothing in the work tree was edited,
  committed or checked out. The checkout stayed on `docs/book-2.5` and was clean (`git status --short` was empty).
- Read: branch `docs/book-2.5`, head `2a942a9f4e300380765759a278eedbfacef86138`, one commit on `24444e9`, which sits 5
  commits above `master` `252daef`. The fix delta (`git diff 24444e9..docs/book-2.5`) touches 10 files, +33/-31:
  `README.md`, and `architecture`, `audit`, `build-and-run`, `config/actions`, `config/errors`, `gaps`, `glossary`,
  `overview` and `roadmap` under `book/src`. I read every hunk of it.
- Code facts were checked at `f4c1fb6`. Its `src/` differs from `252daef` only in two rustdoc lines of
  `src/engine/modifiers.rs` (`28566a4`).
- Inputs: `check_facts.md` (I1-I3, m1-m9), `check_coverage.md` (M1-M13), and the "Fix round" table of `book.md`.
  I checked that table claim by claim and did not take it on trust. Lane evidence came from
  `D:/tmp/ks-orch/p2-2.5-latency-trim/{rulings,results,test_r1,cut,impl}.md`.

## 1. Prior findings: is each one fixed?

Four pairs are the same defect (m1 = M6, m2 = M1, m3 = M9, m6 = M11), so there are 21 distinct defects. Every one is
fixed correctly, and none was refuted. A fix that left a residual is marked as such and counted in section 2.

| Finding | Branch location | Status | Evidence |
|---|---|---|---|
| I1 | `config/actions.md:69` | Fixed | The text now says sentinel events pass "without deciding them; for a modifier among them it only updates the decision core's flag". This matches `decide_event` at `f4c1fb6:src/hook/keyboard_hook_callback.rs:246-249` (`is_injected_event`, then `track_injected`, then `return None`) and `track_injected` at `:287-296`. "No chaining and no looping" still holds, because the event passes and fires nothing |
| I2 | `roadmap.md:987` | Fixed | "the witness term of R11 is unchanged, about 0.6 µs over R12". From `test_r1.md:217`: P 5.3-4.7 = 0.6 and H 0.8-0.2 = 0.6. It now agrees with `roadmap.md:608` and `architecture.md:341` |
| I3 | `architecture.md:309` | Fixed | The text names ruling W2 of item 2.5, says it was the orchestrator's ruling and that the owner declined to choose and can overrule, and says that open question 13 asked for two other costs. This matches `rulings.md:8-9` and `:18-19`. Dropping "under the owner's latency priority" was right: rulings.md does not tie W2 to that priority |
| m1 / M6 | `roadmap.md:611` | Fixed | "exceeded its new bound by 9 ns at the p50 (1.4 against 1.391 µs) ... counted no breach". This matches the tester (`test_r1.md:216`, "formal excess of 9 ns ... Not a breach at the instrument's resolution") and the analyst (`results.md:104`) |
| m2 / M1 | `roadmap.md:631` | Fixed | "Rows 35, 38, 44, 46, 49, 52 and 53". I re-read every Setup cell of rows 1-59. The debug rows are 35, 38 and 44 (`cargo build --locked`), 46 ("The debug daemon of check 38"), 49, 52 and 53. No other row builds or names a debug daemon |
| m3 / M9 | `overview.md:128` | Fixed | "No key event reads every modifier from Windows: ... a key-down asks Windows only about the modifiers it holds". This is true (`f4c1fb6:src/engine/mod.rs:557-566`) |
| m4 | `architecture.md:320-321`, `:329`; `roadmap.md:600` | Fixed | The diagram now has separate LEFT and BACK nodes, both pointing at the hiding count. The bullet says that a decided key "moves nothing". This matches `resync.rs:69-86`: `event(.., true)` only clears `input_away`, and `input_left` and `input_returned` each `bump()`. The roadmap no longer lists `swch on` or a reinstall as movers; their reset calls `read_everything` (`mod.rs:750-753`, `resync.rs:88-94`). I grepped every `resync.` call in `mod.rs`: there are only 6, at `:554`, `:658`, `:683`, `:695`, `:752` and `:759` |
| m5 | `build-and-run.md:147`, `glossary.md:131`, `roadmap.md:602`, `config/errors.md:231`, `README.md:884-885` | Fixed | Each says `[INFO]` only "and no callback took 5 ms or more", which matches `summary` in `f4c1fb6:src/logging/counters.rs:252-261` and `SLOW_CALLBACK` at `:132`. The roadmap's "a callback that ran 17 ms also took the 5 ms" holds: `keyboard_hook_callback.rs:438-447` passes the same `ran` to `record_callback` and to `late_callback`. build-and-run now names the main-thread stretch, as `counters.rs:64-69` does. No other `[INFO]` rule for counters remains in `book/src` or `README.md` (grep) |
| m6 / M11 | `glossary.md:301` | Fixed | The permalink is now `#L425-L430`. Line 430 holds `decide_event(hook, wparam, event.vkCode, event.dwExtraInfo)` |
| m7 | `roadmap.md:1042` | Fixed | "an injected key without the sentinel is decided like a typed one, so it can fire a binding". This is true: `decide_event` does not look at `LLKHF_INJECTED`, and only the late note reads it (`:427`, `:442-446`). The widened `#L240-L296` covers `decide_event` and `track_injected` |
| m8 | `roadmap.md:9` | Fixed | "item 2.5 acted on question 13" |
| m9 | `overview.md:130` | Fixed | `252daef` has the parents `f4c1fb6` and `28566a4`, and `28566a4` changes `README.md` (+5/-2) and `src/engine/modifiers.rs` (2 lines). Both checked with `git show` |
| M2 | `roadmap.md:693` (check 59) | Fixed | The Setup now starts with `swch exit`, then `swch status`, then PowerToys, then `swch open`. "says that the daemon is not running" matches `src/bin/swch.rs:157` |
| M3 | `roadmap.md:695` | Fixed | "Checks 43, 44 and 53 ... and checks 47 and 55". Row 55 is "As check 47" (`:689`) |
| M4 | `roadmap.md:1031`, intro `:1011` | Fixed | The candidate cell now reads "none: closed by ruling W1 of item 2.5, not done", and the intro allows "settled by a ruling without being done" |
| M5 | `roadmap.md:1039-1053` | Fixed | The five rows that are not "Not verified" points now sit at `:1039-1043`. Every "Not verified" row is contiguous at `:1044-1053`, the Sticky Keys row (`:1046`) included |
| M7 | `roadmap.md:618` | Fixed | "after a move of the hiding count, with a short second read for the codes it owed". This matches `resync.rs:98-116` (`owed = planned.without(read)`) and `architecture.md:346` |
| M8 | `architecture.md:252` | Fixed | The invariant adds "or, for a press whose key-down the engine never saw, one of its auto-repeats was (... `RepeatSuppressed` ...)". This matches `keys.rs:281-291` (`take_blocked` matches `Suppressed \| RepeatSuppressed`) and `mod.rs:940-948`. The anchor `#atomicbool-256-active-keys-table` exists (`architecture.md:389`). The glossary still states the invariant the old way, see N2 |
| M10 | `audit.md:409` (Status line only), `gaps.md:167` | Fixed on both cited lines | "a key-down asks Windows about each modifier the engine holds, and a swallowed key-up about each held Alt and Win". This matches `mod.rs:564-565` (`ModifierSet::ALL`) and `:999-1006` (`ModifierSet::MENU`, only `if decision.suppress`). `git diff master...docs/book-2.5 -- book/src/audit.md` changes 1 line, the KS-14 **Status** line. The same claim survives at `glossary.md:67`, see N1 |
| M12 | `roadmap.md:691` (check 57) | Fixed | Only "that daemon toggles and this one switches" is now called known issue 89, which agrees with `known-issues.md:157`. The new wording relies on a run of the older daemon that the Setup does not provide, see N3 |
| M13 | `roadmap.md:597` | Fixed | The block opens with "**Status: done** in", as the blocks of 2.1-2.4 do |

The doc-writer's follow-up for the developer is valid (not a book defect). The rustdoc at
`f4c1fb6:src/engine/resync.rs:7-8` lists "`swch on`, a hook reinstall" among the sources that move the hiding count.
Their reset calls `read_everything` (`mod.rs:750-753`), which sets `resynced = hiding` and moves nothing
(`resync.rs:88-94`). The book's citation `resync.rs:1-46` (`architecture.md:314`, `glossary.md:245`) shows that stale
doc comment to a reader. The book follows the code.

## 2. Findings still open or new (all Minor)

**N1 (Minor; M10 left in a third place, new). `glossary.md:67` (Async key state) still says what M10 fixed elsewhere.**
- **Says:** "A key event the engine decides asks about each modifier the engine holds and drops the ones Windows has
  up".
- **True:** a key-down asks about every held modifier except its own key and the swallowed ones
  (`f4c1fb6:src/engine/mod.rs:564-565`, `keys.rs:127-143`). A swallowed key-up asks only about held Alt and Win
  (`mod.rs:1000-1006`, `ModifierSet::MENU`). A key-up that passes asks nothing.
- **Evidence:** the fix round corrected this claim at `audit.md:409` and `gaps.md:167`, the two places M10 named, and
  at `roadmap.md:600`. The glossary entry has the same claim and was not touched; `check_coverage.md` M10 did not list
  it either. `architecture.md:311` states the rule correctly.
- **Fix direction:** use the `gaps.md:167` wording: "a key-down asks Windows about each modifier the engine holds, and a
  swallowed key-up about each held Alt and Win, and drops those Windows has up".

**N2 (Minor; consistency, new, a side effect of the M8 fix). `glossary.md:497` (Suppression record) now states the
invariant differently from `architecture.md:252`.**
- **Says:** "A key-up whose key-down, or that repeat, was suppressed is suppressed too, unless Windows has the key
  down ..., so the invariant reads: a key-up is suppressed if and only if its key-down was, unless Windows has the key
  down".
- **True:** the first half of that sentence is right, but its "so the invariant reads" clause drops the repeat. That is
  the literal gap M8 found in the architecture sentence. Since `2a942a9`, `architecture.md:252` states the full
  invariant, with the `RepeatSuppressed` case (`keys.rs:281-291`), so the two pages now quote "the invariant" in two
  forms.
- **Fix direction:** add "or, for a press whose key-down the engine never saw, that repeat was" to the glossary's
  invariant clause, or drop the clause and link `architecture.md#the-key-event-path`.

**N3 (Minor; manual-check usability, new on a line the fix changed). `roadmap.md:691` (check 57): the Expected cell
compares with an older daemon that the Setup never starts.**
- **Says:** the Setup is "Release daemon, default config, Notepad" plus Sticky Keys. Step 1's Expected is: "If that
  daemon [from before item 2.5] toggles caps lock and this one switches the layout, that is known issue 89; any
  difference from the older daemon goes to item 3.5, and the same result on both is no change of item 2.5".
- **True:** step 1 can only be judged against a daemon from before item 2.5, run in the same way. On this daemon alone,
  a layout switch is ambiguous: either the latch reaches Windows' key state only (known issue 89), or it does not reach
  it at all, and then there is no change. The fix of M12 made the comparison explicit, but no row says which build to
  use (for example a release build of `bf549c0`) or to run step 1 on it first.
- **Source:** `cut.md:1825-1828` (v3 M3) says "Expected: `ToggleCapsLock`, which master gives if `GetAsyncKeyState`
  reads the latch down. The owner records what HEAD does. A difference from master is a follow-up". So the source has
  the same gap.
- **Fix direction:** add to the Setup "for step 1, also a release build from before item 2.5 (`bf549c0`), run first
  after `swch exit`". Otherwise, reword step 1 to "record the result; it is compared with the behaviour before item 2.5".

## 3. Build and links (my own run)

- **Build:** I exported the committed tree with `git archive 2a942a9 book README.md` into my scratchpad and ran
  `mdbook build book -d D:/tmp/ks-orch/book-2.5/recheck-book`. It exited 0. The only warning is mdbook-mermaid's
  "built against version 0.5.3 ... called from version 0.5.2" (`D:/tmp/ks-orch/book-2.5/recheck-build.log`).
- **Internal links:** my own HTML checker ran over every page except `print`, `toc` and `404`: 20 pages, 2721 internal
  hrefs, 0 missing files, 0 missing anchors, 0 duplicate ids (`D:/tmp/ks-orch/book-2.5/recheck-links.txt`). The new
  links resolve: `#atomicbool-256-active-keys-table`, `roadmap.md#25-latency-trim` from `architecture.md:309`, and
  `#the-key-event-path`.
- **Permalinks:** my own checker covered every `github.com/.../blob|tree|commit/<sha>` link in `book/src/**/*.md` and
  `README.md` at `2a942a9`. It found 1788 links and 0 bad ones: each commit exists, each path exists at its commit, and
  each `#Lx-Ly` lies inside the file. 769 of them point to `f4c1fb6`, the count `book.md` gives
  (`D:/tmp/ks-orch/book-2.5/recheck-permalinks.txt`).
- **New or changed ranges:** I read the three of them at `f4c1fb6`:
  - `resync.rs#L65-L86` shows `event`, `input_left` and `input_returned`.
  - `keyboard_hook_callback.rs#L240-L296` shows `decide_event` through the end of `track_injected`.
  - `#L425-L430` shows the struct read through the `decide_event` call.
- **Mermaid:** I checked the changed hiding-count diagram by eye: quoted labels, no parentheses, and node ids `LEFT`
  and `BACK` with no reserved words. It was not rendered, because no renderer is installed.

## 4. The doc-writer's account (`book.md`, Fix round)

- **Accurate:** the 25-row table, the 21-distinct-defects arithmetic, "none is refuted", the head `2a942a9` (one
  follow-up commit, nothing amended), the 769 permalinks to `f4c1fb6`, the build result, and "audit.md: only the KS-14
  Status line". The commit carries `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- **Out of date, not a book defect:**
  - The round-1 "Pages changed" list still says "Not touched: ... `README.md`". The fix round edited
    `README.md:884-885`, and its m5 row and "Verification" say so.
  - The 2.5 section of `methodology.md` (`:559`) also does not mention that the sync edited the README. The earlier
    sync sections do not describe their check rounds either, so this matches their format.
- **Not covered by the account:** the fix of M10 missed `glossary.md:67` (N1), and the fix of M8 left the glossary's
  invariant clause as it was (N2).

## Positives

- Every fix is the smallest edit that makes its sentence true. The plan texts stay as records, and the status line of
  `audit.md` is the only audit change.
- The m5 fix was carried to every page that states the `[INFO]` rule, including `errors.md` and `README.md`, which the
  checker had not listed. The same rule now reads the same in five places.
- The I3 fix turned down wording that no lane report supports ("under the owner's latency priority").
- The doc-writer found a stale rustdoc (`resync.rs:7-8`) and recorded it as a hand-off for the developer instead of
  copying it into the book.
- The reordered follow-ups table now keeps all ten "Not verified" rows together at the end, as its intro says.

## Not covered

- Mermaid rendering in a browser (no renderer installed; nothing was installed).
- RON validation: the fix delta changes no `ron` block, README snippet or template line, so nothing was re-run. The
  doc-writer's 32 cfgcheck runs were not repeated.
- Pages and lines outside the fix delta, except the greps for the residual phrasings of I1-I3, m3-m5, M7 and M10
  (`book/src` and `README.md`), which found N1 and N2.
- `CHANGELOG.md`, `CLAUDE.md` and `.claude/`, which are outside the book sync.
