12 findings: 0 critical, 3 important, 9 minor

# Independent fact check: book sync after roadmap item 2.5 (docs/book-2.5)

Checker: project-analyst, lens **facts**. Read only; nothing in the work tree was edited, committed or checked out.

- Branch `docs/book-2.5`, head `24444e99d61a0043cec83b4a7c08a8d72cf93fa0` (5 commits on `master` `252daef`).
- Diff read with `git diff master...docs/book-2.5 -- book/src`: 20 files, +732/-621.
- Code facts were checked against `f4c1fb6` (the pin). `git diff f4c1fb6 252daef -- src` changes only two `///` lines of
  `src/engine/modifiers.rs` (`[`MODIFIERS`]` written as plain code), so every code fact holds at `252daef` too.
- Lane evidence used: `D:/tmp/ks-orch/p2-2.5-latency-trim/{results,rulings,test_r1,impl,cut,merge}.md`, `merge_gate_master.log`.
- I checked the doc-writer's account (`D:/tmp/ks-orch/book-2.5/book.md`) claim by claim rather than taking it on trust.

## Findings

### Important

**I1. `book/src/config/actions.md:69`: the sentinel sentence is stale. The sync re-pinned this line but did not correct it.**
- **Says:** "the daemon's own hook passes them through before the decision core sees them
  ([src/hook/keyboard_hook_callback.rs:246-249])".
- **True:** since `f4c1fb6`, a sentinel-tagged modifier event does reach the decision core. `decide_event` calls
  `track_injected` before it returns `None` (`src/hook/keyboard_hook_callback.rs:246-249`, the lines the link itself
  shows). That calls `Engine::track_injected` (`:287-296`), which sets or clears the engine's flag
  (`src/engine/keys.rs:150-159`) and marks the code fresh for the drain (`src/engine/mod.rs:693-696`). The event still
  passes and fires nothing, so "no chaining and no looping" is still true.
- **Evidence:**
  - results.md P1 (`:185-194`) flagged exactly this wording in `README.md:151`, and the chore `28566a4` fixed it there
    (`README.md:154` at 252daef: "without deciding them; for a modifier among them it only updates the engine's flag").
  - book.md "Claims removed or corrected" says the wording was corrected in architecture and glossary. `actions.md:69`
    is the one place the sync missed (git grep "before the decision core" finds only this line).

**I2. `book/src/roadmap.md:987` (outcome of open question 13): "R11 is unchanged" contradicts the R11 figures in the book.**
- **Says:** "the witness read stays at every swallowed key-up (ruling W1 ...), so R11 is unchanged."
- **True:** row R11 fell from 5.3 µs to 0.8 µs (P to H, the median of three p50s). What stayed the same is the witness
  term, R11 minus R12: 0.6 µs before and after.
- **Evidence:**
  - test_r1.md:195 (R11 5.3 / 0.8 / 0.8) and test_r1.md:217 (witness term 0.6 against P 0.6); results.md:107.
  - The book itself gives the figure elsewhere: architecture.md:341 has `R11 ... | 5.3 | 0.8`, and roadmap.md:608
    says "0.8 against 0.2 µs at the head; the witness term is 0.6 µs before and after".
- **Why it matters:** the brief asks for R11 to be stated the same way everywhere; this is the one place it is not. The
  shorthand comes from results.md:107 and rulings.md:17/:186 ("R11 unchanged").
- **Fix:** "so the witness term of R11 is unchanged (0.6 µs over R12)".

**I3. `book/src/architecture.md:309`: the owner decision is misattributed.**
- **Says:** "Until roadmap item 2.5 the engine read all 11 modifier codes ... That read was most of the 4.7 µs ... The
  owner decided to trim it (open question 13)."
- **True:** open question 13 was about two other costs: the witness read (R11) and the watchdog's first-key wake (R16).
  The owner decided "trim both now". Taking the per-event modifier read off the key path was the orchestrator's ruling
  W2 (the cut's Q5, folded into the lane). The owner was asked about the lane's trade-offs, declined to choose, and can
  still overrule.
- **Evidence:**
  - rulings.md:8-9 (owner declined) and :18-19 ("W2 Modifier resync leaves the key path. This is the cut's Q5, folded
    into this lane").
  - Commit af87cd8 (merged in 50b56a8): "Question 13: trim the witness read and the watchdog's first-key wake now".
  - The branch's own roadmap.md:600: "(rulings W2 and W2a, folded into this item)".
  - roadmap.md:987, the question's text.
- **Fix:** "Ruling W2 of item 2.5, under the owner's latency priority, took it off the key path".

### Minor

**m1. `book/src/roadmap.md:611`: R12m2 is worded as a pass.**
- **Says:** "R12m2, two held modifiers, met its new bound only within 9 ns, under the 100 ns resolution of a p50".
- **True:** the head's p50 of 1.4 µs is 9 ns *above* the bound of 1.391 µs. The tester and the analyst judged that
  excess to be below the instrument's resolution and not a breach (the unrounded means 1.369 and 1.354 are within it).
- **Evidence:** test_r1.md:216 ("formal excess of 9 ns"); results.md:104.
- **Fix:** "exceeded its new bound by 9 ns at the p50, below the 100 ns resolution of a p50; not a breach".

**m2. `book/src/roadmap.md:631`: the list of rows that use a debug build is incomplete.**
- **Says:** "Rows 35, 38, 44, 49 and 52 use a debug build".
- **True:** row 53, new in this sync, uses one too: "As check 44, with a debug build of `f4c1fb6` or later"
  (roadmap.md:687). So does row 46, "The debug daemon of check 38" (:680); that omission is older than this sync.

**m3. `book/src/overview.md:128` (the `f4c1fb6` row): the row contradicts itself.**
- **Says:** "No key event reads the modifiers from Windows: ... a key event checks only the modifiers it holds".
- **True:** the check is a `GetAsyncKeyState` per held modifier (`src/engine/keys.rs:127-143`, called at
  `src/engine/mod.rs:564-565`). Every other page says "no key event reads them all" or "every modifier"
  (architecture.md:237, audit.md:409, gaps.md:167).

**m4. The sources of the hiding count are drawn and listed inconsistently.**
- In the diagram at `architecture.md:320`, "input back on this desktop, or the next decided key" points to the hiding
  count, and leaving the desktop does not.
- In the code, `input_left` and `input_returned` bump the count (`src/engine/resync.rs:77-86`). A decided event only
  clears `input_away` (`:69-74`), which lets the read that the departure made due happen. The list at
  architecture.md:325-328 gets this right.
- `roadmap.md:600` names "`swch on`, a reinstall" among the moves of the hiding count. Their reset calls
  `read_everything` (`src/engine/mod.rs:750-753`, `resync.rs:88-94`), which reads everything and moves nothing, as
  architecture.md:328 says.

**m5. The rule for `[INFO]` against `[WARN]` leaves out the slow-callback term.**
- **Where:** `build-and-run.md:147`, `glossary.md:131`, `roadmap.md:602`.
- **Says:** the line is `[INFO]` "when `hook_late` is the only count that is not 0", and `hook_late` "alone makes an
  `[INFO]`".
- **True:** a report warns while any warning counter is non-zero, *or while the longest callback was 5 ms or more*
  (`src/logging/counters.rs:252-261`). A `hook_late` caused by a callback's run time (17 ms or more) always comes with a
  slow callback, so that report is a `[WARN]`. Only a late time stamp alone gives `[INFO]`.
- The wording is inherited from the lane's `config/errors.md:231`. The sync's roadmap check 52 handles it correctly: it
  says a counters line, not the cut's `[INFO]` line (cut.md:1521).
- In addition, build-and-run.md:147 says "late callbacks" but leaves out the main-thread stretch of 17 ms or more (E5),
  which `hook_late` also counts (`counters.rs:64-69`).

**m6. `book/src/glossary.md:301` (KBDLLHOOKSTRUCT): the permalink misses the line it cites for.**
- **Says:** "copies four of them ([...keyboard_hook_callback.rs:422-427])".
- **True:** that range ends before line 430, where `vkCode` and `dwExtraInfo` are read. architecture.md:234, keys.md:178
  and known-issues.md:41-42 cite `#L425-L430` for the same fact.

**m7. `book/src/roadmap.md:1045` (sentinel follow-up): the stated consequence is not what the code does.**
- **Says:** "Every key the daemon injects must keep the sentinel, or the engine stops following the daemon's own
  modifiers".
- **True:** an injected event without the sentinel is *decided*: `decide_event` passes it to `on_key`, which calls
  `press`/`release` (`src/engine/keys.rs:220-232`) and `resync.event(vk, true)` (`mod.rs:554`). So its flag is still
  followed, and the event can also fire bindings: the feedback loop of CLAUDE.md principle 3.
- The sentence is copied from results.md:264-265.

**m8. `book/src/roadmap.md:9` (progress line): "item 2.5 carried out question 13" overstates.**
- The owner decided "trim both". Item 2.5 kept the witness read (ruling W1), and R16 fell without the planned watchdog
  change (ruling L3).
- The outcome at roadmap.md:987 says this accurately. The progress line could say "acted on question 13".

**m9. `book/src/overview.md:130`: the list of excluded commits describes `252daef` incompletely.**
- **Says:** the excluded commits are those "that update `CLAUDE.md`, the agent files and, for owner decisions, the
  roadmap".
- **True:** `252daef` also merges `28566a4`, "Docs: README and rustdoc after roadmap 2.5", which touches `README.md`
  and `src/engine/modifiers.rs`.

## Verified (no defect)

- **Commits:**
  - Every commit cited on the changed lines exists, with the subject the text gives it: c6b186b, 045b8b4, 3f0dc62,
    e84625e, 4703385, aa86922, 62c9fd8, e391a06 ("R16 is gated against R12"), ddb9f3a (the red test), ea944da,
    47af1b1, b106db9, 9a9bdcf (red), 7c26671 (2026-10-19), 50b56a8, 3cde1d0, 252daef, f4c1fb6 (2026-10-20).
  - The story of the two reverts matches the commit graph: 47af1b1 reverts ea944da's code and leaves ddb9f3a's test
    red; b106db9 drops that test.
- **Test names:** all 24 test paths cited on the changed lines exist as `fn` in the module file named, at 252daef. That
  includes the three KS-14 scenarios of audit.md:409 and the seven acceptance rows of roadmap.md:606-616.
- **Test counts:**
  - From `#[test]` attributes at 252daef: 869 in the library, after excluding one `"    #[test]"` string literal in
    `installer.rs:536`.
  - Every row of the per-module table at build-and-run.md:384-409 matches (scenarios 235, watchdog 55, main_thread 38,
    callback 29, executor 26, engine 26, keys 24, decision 24, writer 11, health 10, counters 9, properties 7, resync 6,
    modifiers 1, ...).
  - merge_gate_master.log:5/:876/:895 gives 869 run, "862 passed; 0 failed; 7 ignored", and 7 integration tests.
  - The hook::watchdog total of "79" (roadmap.md:616) is 55 + 24.
- **Line counts (overview.md:149):** I reproduced them with my own script at f4c1fb6:
  - 40,201 lines in 67 files, 14,568 of them outside tests;
  - engine 13,565 / 2,883; hook 11,536 / 4,793; platform 883 / 475;
  - `tests/ipc_roundtrip.rs` 281 and `build.rs` 214;
  - the bf549c0 figures 37,234 / 13,816 in 65 files.
- **Bench figures:**
  - Architecture table (:337-343) and roadmap.md:618 against test_r1.md:177-206: R12 4.7→0.2, R12m 4.7→0.8,
    R12m2 4.7→1.4, R11 5.3→0.8, R16 52.6→0.9, R18 4.5, R17a 0.545-0.552, R17t 0.001.
  - Session labels P=62c9fd8 and H=d853a59 (test_r1.md:171-172).
  - The developer's R16 of 4.3/4.4 µs (impl.md:358, rulings.md:275), "about 4 µs above R12", as README.md:958 and
    `watchdog/mod.rs:39-42` say.
  - "Factor of about five" (methodology.md): 4.3-4.4 against 0.9.
  - The bound "0.9 against 20.2" (test_r1.md:218).
  - R12 and R16 are stated consistently on every page (R16 is "about 1 to 4 µs depending on the session").
- **Code facts in the changed text:**
  - Steps 1-5, 8, 13-15 and 17 of the key-event path:
    - the four copied fields: `keyboard_hook_callback.rs:425-430`;
    - the age of an injected event is 0: `:440-446`;
    - the 17 ms threshold: `main_thread.rs:151-154` and `callback.rs:471-473`;
    - the key-down check: `mod.rs:564-565`;
    - the key-up check only at a swallowed key-up, on `ModifierSet::MENU`: `mod.rs:999-1006`;
    - `RepeatSuppressed`: `mod.rs:940-948`, `keys.rs:64-70`, `:195-200` and `:283-291`.
  - The drain read: sided codes are set from Windows, generic codes only cleared (`keys.rs:104-116`). It is the last
    step of the drain (`main_thread.rs:860`), and nothing is read while remapping is off (`:383-390`).
  - E5 sends the loop back to the drain (`main_thread.rs:798-806`).
  - `Resync` is pure data with owed codes (`resync.rs:15`, `:96-116`). The first plan reads (`hiding: 1`, `:49-58`).
  - The reset reads no generic codes or mouse buttons (`keys.rs:173-191`).
  - Watchdog E6 (`watchdog/mod.rs:281-285`); `unseen_input` AcqRel/Acquire (`health.rs:132-142`).
  - Twelve counters (`counters.rs:19`, `:74-87`).
  - The 11 modifier codes (`modifiers.rs:13-28`).
  - The windows feature `Win32_System_SystemInformation` is the only Cargo change (`git diff bf549c0 f4c1fb6 --
    Cargo.toml Cargo.lock`).
  - The run loop before 2.5 (`bf549c0:src/engine/mod.rs:511-528`) read at every key event, key-ups included, as
    architecture.md:309 says.
- **Permalinks:**
  - All 731 links on changed lines resolve at their commit, with the range inside the file.
  - I paired old and new links on changed lines myself:
    - 587 re-pins cite identical lines;
    - 57 re-pins cite changed lines, and I read each one's first and last lines: every one still brackets the same
      item (MAIN at `main_thread.rs:112-114` included), except m6;
    - about 90 hand-written links (the new section, steps 5/8/13/15/17, known issues 89/90, the 3.5 follow-ups) show
      what their sentence says.
  - Links of status notes of the roadmap's Phase 2 follow-ups kept at bf549c0, as methodology.md says.
- **Manual checks 49-59 against cut.md:1518-1527 and impl.md:334-353:**
  - M1-M4, M8, M10 and M11 are carried over, and M5-M7 point back to 2.4's checks 44, 39 and 47 (L3).
  - M9 has its latched step.
  - M8, M10 and M11 are marked as checks of the lane's assumptions; M9 is marked as settling known issue 89.
  - Checkpoint 2 item 4 names 49, 50 and 56-59.
- **Rulings:** W1, W2/W2a, L3, L4 and W4 are each cited for what they rule. The roadmap progress line's "questions 3 and
  6 are open, 8 to 14 decided" matches roadmap.md:977-988; question 14 is the owner's decision of af87cd8.
- **Microsoft Learn:**
  - STICKYKEYS: "Pressing a modifier key once *latches* the key down until the user presses a non-modifier key or clicks
    a mouse button" supports known issue 89.
  - GetMessageTime: milliseconds since system start, with no mention of GetTickCount; this supports step 17 and known
    issue 90 ("not that it counts on the clock of `GetTickCount`").
- **Hand-off line :251 of impl.md (the suppression invariant):** step 13/14 and the record table carry
  `RepeatSuppressed`. The invariant sentence at architecture.md:252 is still true with the witness clause: an unseen
  re-press read as a repeat is a swallowed key-down, and a genuine repeat has Windows holding the key down.

## Positives

- The key-path flowcharts are repaired: the decision nodes of master's architecture.md (`C0code below zero?` and so
  on) are proper `{"..."}` nodes again.
- The re-pin is careful. Every changed-content re-pin I read brackets the right item, and the hand-written links are
  exact (for example step 15's `L1000-L1006` and the E5 `L798-L806`).
- R16 is quoted with its session everywhere, the developer's 4.3/4.4 µs beside the tester's 0.9 µs, as results.md P3
  asked.
- Check 52 avoids the cut's wrong "`[INFO]` counters" line. A 1.5 s hold is a slow callback, so that line is a `[WARN]`.
- The test counts, line counts and merge-gate figures are exact and traceable.
