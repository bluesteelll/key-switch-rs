# Results analysis: p0-A3-suppression (Workstream A: "suppresses repeats symmetrically, makes `sync_modifiers` skip blocked keys"; KS-06 rest, KS-14, plus KS-30 and the rest of KS-32)

Branch `fix/suppression-symmetry` @ `5d0fada`. It was cut from `master` @ `82f542f`, and the merge-base is master HEAD. 10 commits, 12 files, +1191/-164. The tree is clean and the branch is not pushed.

## VERDICT: ACCEPT

**Summary**: Every acceptance line of the brief is backed by an `Engine::on_key` scenario or property test. Every lane red-first fails by assertion on the master engine; I replayed this myself (14 failed, with the predicted texts) and they pass on HEAD. The gate is green in my own run: fmt, clippy in a fresh target dir, and 286/286 tests against 263 on master. The design keeps `decide()` pure, touches the hook adapter only in comments, and makes the hot path cheaper. The review is APPROVED with 0 Critical and 0 Important, and the tester is GREEN. These items remain for the orchestrator and do not block:
- two README sentences that now overclaim the focus-transition repair (review m1, still open);
- the tester commit's Sonnet trailer, which is the second time this has happened (A2 P1 was never acted on);
- the book sync, together with the roadmap 2.3 debt notes.

## Quality gate (own run)

| Check | Result |
|---|---|
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK. The in-tree run was a stale 0.13 s "Finished", so I re-ran in a fresh `CARGO_TARGET_DIR` (`analyst-target`): full check of the crate, exit 0, no warnings |
| `cargo test --locked` | **286 passed, 0 failed, 0 ignored** (lib). The swch bin and doc-tests run 0 tests. Log: `D:/tmp/ks-orch/p0-A3-suppression/analyst-test.txt`. Master `82f542f`: 263 (my `--list` on a `git archive` export). Audit commit `7cb3521`: 120 |
| `cargo test --locked -- --list --ignored` | 0 tests in every target. No `#[ignore]` was added or removed |
| `cargo doc --no-deps` | 1 warning, `src/data/vk_name.rs:4` unclosed `<raw>`. It also appears on master, so it predates this lane |
| `mdbook build` | not run: the lane does not touch `book/` |
| Test-set diff vs master (names) | removed: `clear_all_keeps_passed_key_downs` and `clear_all_resets_every_flag` (renamed to `forget_blocked_*`, same subject), and `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck` (it pinned the KS-14 behaviour and is replaced by T5, as cut commit 2 declares). Added: 26 |
| Trailer count | 9 of 10 commits carry `Co-Authored-By: Claude Opus 5.5`. `5d0fada` carries `Claude Sonnet 5.5` (P2) |

**Independent red replay.** I exported master `82f542f` with `git archive`, copied in HEAD's `testing.rs`, `scenarios.rs` and `properties.rs`, and registered `mod properties`. I then ran `cargo test --lib engine::`: **14 failed**, 128 passed (`analyst-master-replay.txt`). The failures were the 12 lane scenarios plus both properties, each an assertion with the predicted text. Two examples:
- `repeat LAlt at +3079 ms: press of LAlt: key-down suppressed, repeat passed` / `context: config default, seed 0, Strict mode`;
- `up LCtrl at +4114 ms: VK 0xA2 is down in Windows but physically up` / `context: config gestures, seed 3, Focus mode`.

The three unit-level red-firsts live in production files (`keys.rs`, `mod.rs`). The tester replayed those at `20d4844`, `72ea8d6` and `afdb60c`; I checked that those three commits touch test code only.

**Own mutation.** On a HEAD export I narrowed the resync skip to `Suppressed` only, dropping `Forgotten`. Two tests caught it: `keys::tests::forgotten_modifier_is_left_alone_by_the_resync` and `scenarios::suppressed_modifier_stays_an_auto_repeat_across_a_focus_change` (T10). This adds to the tester's 12 mutations, which were all caught (`test_r1.md`).

## Acceptance criteria

| # | Criterion (brief / roadmap.md:98) | Evidence | Status |
|---|---|---|---|
| 1a | A repeat of a suppressed key is suppressed | `scenarios.rs:1649` `shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press` (two repeats after Alt's key-up; red on master). Also `:1616` `ctrl_shift_hold_with_ctrl_released_first_neither_leaks_nor_sticks_shift`, `:1586` `one_long_press_of_shift_does_not_fire_its_double_tap`, and the pre-existing `:1348` `repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired` and `:1373` `immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once`. "Whatever the kind or trigger" across a focus change: `:1728` (chord), `:1759` (Hold and Tap), `:1801` (Immediate combo). Tester M7 caught (12 tests) | PASS |
| 1b | ...and its key-up is suppressed exactly once | T5 `:1649`: key-up suppressed, then the next press passes on both edges. `:1682` `key_up_of_a_suppressed_press_is_suppressed_only_once`, the tester's `5d0fada`, which closes the gap left by mutation M12 (a record not spent at key-up). Under M12 it fails with `VK 0xA0 is down in Windows but physically up` | PASS |
| 2 | KS-06 default-config route (Alt+Shift auto-blocker, Alt released while Shift is held) leaves nothing stuck | T5 `:1649` on `with_alt_shift_blocker(DEFAULT_PAIR)`, which is the loader's order and the same bindings as `default_config.ron:5-11`, with `check_nothing_stuck()` after every step. Red on master. Tester M1 breaks it | PASS |
| 3 | Property-style test over many generated down/repeat/up sequences holds the invariant | `properties.rs:229` `every_press_is_suppressed_or_passed_as_a_whole` (Strict: whole-press symmetry plus nothing stuck after every step) and `:234` `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups` (Focus: adds foreground changes and unseen key-ups). Each runs 6 configs x 64 seeds x 250 steps, covering every kind and trigger, and takes 0.7 s. Both are red on the master engine (my replay). The checker's repeat branch has its own failing self-test, `scenarios.rs:1942`. Tester M1, M4 and M5 flip the properties | PASS |
| S1 | Spec invariant: key-up suppressed iff key-down suppressed, every kind and trigger | The strict property, plus T1/T2/T3/T4/T5/T6 and the chord/sequence completion scenarios. **Scoped**: across a foreground change, a swallowed press's key-up still passes unless a Tap or Hold is pending. That is the A2 policy and roadmap 2.3's, stated in the `properties.rs:14-17` module doc, README:365-368 and CHANGELOG KS-06. The repeat half of the invariant does hold across focus changes now (critique I1, option b) | PASS (key-up half after a focus change deferred to 2.3, recorded) |
| S2 | A key-down that passes clears the key's blocked state, so its key-up passes | `:1269` `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (pre-existing). Lane tests: `:1828` (KS-32), `:1851` (chord), `:1880` (unrecorded repeat that passes, D3), and `mod.rs:566` (D4, unit level, because no event route reaches it yet) | PASS |
| S3 | The resync must not overwrite a key the engine suppressed, nor reset its repeat detection | `keys.rs:397` `resync_leaves_a_modifier_whose_key_down_was_suppressed` (red on master), `:457` `forgotten_modifier_is_left_alone_by_the_resync`, `:445` `resync_still_repairs_a_modifier_whose_key_down_passed` (guards the repair that is kept), and scenarios T1 `:1517`, T2 `:1560`, T3 `:1586`, T10 `:1705` | PASS |

Manual checks: none are needed for acceptance and no `#[ignore]` stubs exist. `impl.md` M1-M9 is a human smoke list (live daemon, `osk.exe`). Nobody has run it. M9 needs a working `swch off/on`, which waits on KS-01.

## Issues claimed closed

| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-14 (all three audit cases) | Case 1 `Alt+Shift+X`, both key orders: `scenarios.rs:1517`. Case 2, DoubleTap Shift plus `Shift+CapsLock`: `:1560`. Case 3, `DoubleTap(600)` long press with a rollover key-up: `:1586`. Unit test: `keys.rs:397` | Yes: assertion reds at `20d4844` (tester) and on master (mine) | CLOSED |
| KS-06 (rest: the auto-blocker route; repeats after a focus change for every kind; holes H1-H3) | `:1649` (audit's auto-blocker route), `:1728`, `:1759`, `:1801` (roadmap 2.3's `Launch` example: no `jjj`/CapsLock into the new window), `:1880`, both properties | Yes (`20d4844`, `72ea8d6`, `afdb60c`; my master replay) | CLOSED (the key-up half after a focus change is still 2.3) |
| KS-32 (rest; unverified) | `:1828` `sequence_completion_keeps_the_key_up_of_a_key_whose_down_passed`: the audit's exact `["F14+F15","F14+F16"]` example | Yes: `up F14 ... VK 0x7D is down in Windows` | CLOSED |
| KS-30 (unverified) | Key-down-passed half: `:1851` (red-first). Released-key half: only the unit test `keys.rs:416` `completion_marks_only_held_keys_whose_key_down_did_not_pass`, written in the fix commit because `mark_completed` is new. Tester M11 shows that only this unit test catches the guard | Partly. The audit's reload route has been harmless since A2 (`mark_passed` overwrites the stale mark). An event-level red is still possible through `down_unseen` (P5) | CLOSED, with weaker evidence for one half (P5, not blocking) |

## Invariants

| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking or allocation, symmetry) | OK | `src/hook/*` changes are comments only (`keyboard_hook.rs:129-132`, `keyboard_hook_callback.rs:244-249`). `code < 0` and the sentinel check still come before the engine call. The added lines contain no `println`, `unsafe`, `Ordering`, `unwrap()` or `thread::spawn`. Per event, `sync_modifiers` gains one array read per modifier and skips `GetAsyncKeyState` for swallowed modifiers (`keys.rs:88-96`). `mark_completed` allocates nothing. Symmetry is proven by the strict property |
| `decide()` purity, matching per owner decisions | OK | `#![forbid(unsafe_code)]` is kept (`mod.rs:148`). No new `Env` call or clock read. `matching.rs` is untouched: exact modifiers and `allow_extra_modifiers` are unchanged, and repeat defaults remain 1.1's (KS-10 is still listed under the known defects, `mod.rs:143-146`) |
| Thread ownership, sentinel, atomics, SAFETY comments | OK | No new thread, hook, timer, atomic, `unsafe` or `SendInput`. The `SetWinEventHook` SAFETY block is intact |
| Config reload, IPC | N/A / OK | No schema, loader or IPC change. Records survive a reload mid-press on purpose (cut RK9) |
| No key logging | OK | No logging added |

## Review status

- The last review, `review_r1.md` @ `5d0fada`, is **APPROVED** (Critical 0, Important 0, Minor 2). No triage round was needed. No CONFIRMED Critical or Important item is open.
- The last tester round, `test_r1.md`, is **GREEN**. It covers the full gate on `15e901d` and `5d0fada`, 15 red-firsts replayed on their parents in separate target dirs, a whole-lane replay on master, and 12 mutations, all caught after `5d0fada`.
- `critique.md` gave I1 (Important) plus M1-M4. I1 was resolved with option (b): repeats of a forgotten swallowed press stay suppressed. That supersedes the cut's T7-T9. M1-M4 are resolved (cut.md developer addendum, impl.md). The deviation from the cut was not re-critiqued. The code-reviewer traced it, and it follows the brief's spec item 1 and roadmap 2.3's acceptance ("Holding `Win+Alt+T` while the foreground window changes keeps the T repeats suppressed").
- Open Minor items worth doing while the context is fresh: review m1 (P1), review m2 (P4), and the KS-30 event-level scenario (P5).

## Docs and hygiene

- **README**: the behaviour rules (`README.md:355-368`) and the `block_original_combo` row (`:428`, RK2 behaviour until 1.2) are updated. This also fixes A2 results P2, the "release reaches Windows" overclaim for deferred keys. **Still stale**: `README.md:593` and `:609-612` (P1).
- **CHANGELOG**: entries sit under `## [Unreleased]` / `### Fixed`: KS-06 updated (`:87`), new KS-14 (`:123`), KS-30 (`:136`), KS-32 (`:141`, "partly" dropped). The RK2 behaviour change is under Fixed/KS-14. The cut's Q5 leaves that placement to the orchestrator; it is defensible because the old behaviour was a timing-dependent leak.
- **`default_config.ron`**: no text covers repeats or the resync, so it needs no change (`:115-125` checked).
- **Book**: not edited and deferred to the phase book sync (cut D8, critique open question 1), as in A2 (D12). CLAUDE.md:163-164 asks the developer to update the matching `book/src/config/*.md` page in the same lane. That deferral needs the orchestrator to confirm that a book sync runs. The book has been pinned at `7cb3521` since A1: `audit.md` status lines for KS-04/06/07 are still not marked fixed after A2.
- **Dependencies**: none. `Cargo.toml` and `Cargo.lock` are unchanged.
- **Branch**: cut from current master, contains only this lane's work, and merges cleanly with `--no-ff`. Trailers: 9 of 10 (P2).

## Technical debt

- **RK1** (accepted trade-off, spec item 3). A swallowed modifier whose key-up is lost (secure desktop, Win+L, `swch off` while held) stays held for the engine until its next press and release. That next press is swallowed. Meanwhile, exact matching sees the phantom modifier: with the default config, CapsLock fires `Shift+CapsLock` instead of `SwitchLanguage`. Recorded in the `mod.rs:319-323` doc comment, `keyboard_hook_callback.rs:247-249`, cut.md addendum M4 and impl.md. **Not recorded** in README, CHANGELOG or the roadmap. Roadmap 2.3 (`roadmap.md:297`) says its reset "resyncs all 256 keys, **except blocked ones**", so as written 2.3 would not repair this either (P3).
- **I1(b) cost**: after a lost key-up plus a focus change, the key's next physical press is swallowed once and its release passes. Recorded in `mod.rs:319-323` and cut.md. Not in the roadmap (P3).
- **RK7 class / Deviation 6**: a key held since before the hook saw it reads as a first key-down at its next repeat, and if a binding swallows it, its key-up is swallowed while Windows has it down. A held `Unknown` modifier can also be re-marked by a chord completion. Both predate this lane and need a hook-unseen key-down. Recorded only in impl.md "Open points" and cut RK7 (P3).
- **RK6**: in a non-elevated debug daemon with an elevated foreground, `GetAsyncKeyState` returns 0 and the resync clears every `Passed` modifier on each event. This predates the lane and is recorded in cut.md only.
- **D4**: currently unreachable. It becomes live with 1.3 (`block_default` on deferred triggers). Pinned by `mod.rs:566`; recorded in the test comment.
- No new `TODO`/`FIXME` in the diff.
- **Smoke lines**: impl.md M1-M9 are human checks with no line in a roadmap smoke checklist yet. Checkpoint 1 smoke 2 covers M5 only (P3).

## Problems

### P1. README still promises a full focus-transition repair for modifiers (review m1, open)
- **What**: `README.md:593` ("so phantom-held modifiers do not leak across the transition") and `:609-612` (Secure Desktop limitation: the same claim). The KS-14 CHANGELOG entry (`CHANGELOG.md:123-135`) does not name the cost either.
- **Impact**: a doc overclaim for a rare route (a swallowed modifier held while a secure desktop appears). Behaviour is not affected. The trade-off is mandated by spec item 3.
- **Root cause**: D2 deliberately skips `Suppressed`/`Forgotten` modifiers in the resync (`keys.rs:91`), also on a foreground change (`mod.rs:325`). The developer qualified the code comments (critique M2) but not the two README feature/limitation lines.
- **Return to**: `doc-writer`, as one follow-up `Docs:` commit on this branch before the merge, or in the phase book sync. Not blocking.
- **Re-check condition**: both README lines say that a modifier whose key-down was swallowed is not repaired and that roadmap 2.3 owns the repair, and the CHANGELOG KS-14 entry has one sentence on it.

### P2. `5d0fada` carries `Co-Authored-By: Claude Sonnet 5.5`, and A2's fix was never applied
- **What**: CLAUDE.md (Git flow) requires `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` on every commit. The count is 9 of 10.
- **Impact**: hygiene only. This is the second occurrence: `965ac4d` on master (A2 results P1) had the same defect.
- **Root cause**: A2 results P1 asked the orchestrator to spell out the literal trailer for the tester. It was not done. `.claude/agents/tester.md:22` and `.claude/workflows/ks-lane.js:134` still say only "with the Co-Authored-By trailer". The tester role runs on Sonnet and writes its own model name.
- **Return to**: none of the lane phases, because the git flow forbids amend and a follow-up commit cannot change it. Orchestrator and owner choose one of two options:
  - (a) record the exception, as for `965ac4d`;
  - (b) `5d0fada` is the unpushed branch tip, so a message-only amend would fix it, but only with the owner's explicit authorisation, since CLAUDE.md forbids amend.

  Either way, put the literal trailer string into `tester.md:22` and the ks-lane tester brief before the next lane.
- **Re-check condition**: the next lane counts 100% Opus trailers.

### P3. Book sync, roadmap 2.3 debt and smoke lines
- **What**: the book pages in cut.md "Docs impact" and impl.md "Docs" are not updated:
  - `known-issues.md` rows 9, 20, 21 and 27;
  - `triggers-conditions.md` §11.8;
  - `kinds.md` §8.4 and §9.3;
  - `architecture.md:184`, `:208`, `:491-494`, `:524`;
  - `glossary.md:287`, `:343`;
  - `audit.md` status lines for KS-06, KS-14, KS-30 and KS-32.

  Roadmap 2.3 also does not say that its reset must release keys whose key-down was swallowed (RK1, I1(b) cost), and it says the opposite ("except blocked ones"). RK7 and Deviation 6 are recorded nowhere durable.
- **Root cause**: book work is deferred to the phase sync by design (cut D8). The 2.3 wording predates this lane's `Forgotten` record.
- **Return to**: `doc-writer` (phase book sync). If the lane merges outside a `ks-phase` run, the orchestrator schedules the sync explicitly. A2's sync is still pending too.
- **Re-check condition**:
  - the pages above match `src/engine`;
  - `audit.md` marks KS-06, KS-14, KS-30 and KS-32 "Fixed in <sha> (fix/suppression-symmetry)";
  - roadmap 2.3 states that `reset_transient_state` on unlock, desktop switch and `swch on` also releases `Suppressed`/`Forgotten` keys, and it lists RK7 and the I1(b) cost;
  - impl.md M1-M4 and M6-M9 appear in a smoke checklist.

### P4. Two test names now state the policy this lane reversed (review m2)
- **What**: `scenarios.rs:1474` `press_after_a_lost_key_up_and_a_focus_change_follows_its_binding`. The press is now swallowed by its record, not by its binding. Also `keys.rs:298` `forget_blocked_resets_every_flag`: only the key-up side is reset, because `is_blocked` stays true for `Forgotten`.
- **Return to**: `developer`, as an optional follow-up rename commit. Not blocking.
- **Re-check condition**: the names describe the rule they pin.

### P5. KS-30's released-key half has no event-level regression
- **What**: only `keys.rs:416` catches dropping the `active` guard in `mark_completed` (tester M11).
- **Return to**: `tester`, optional. Write a scenario on chord `["J","K"]` with `window_ms` 50:
  1. down J, up J (both suppressed), then down K inside the window (fires);
  2. up K;
  3. `down_unseen("J")`, then `up("J")`, and expect `passed` with `check_nothing_stuck()`.

  It should be red on master (stale `Suppressed` on J) and green on HEAD.
- **Re-check condition**: the scenario exists and is red with mutation M11.

## Positives

- **The `Forgotten` record**: the smallest state change that gives both halves of the spec. The resync skips swallowed keys, also after a focus change, and repeats follow the record for every kind. It flips no pinned focus-change expectation.
- **D5 (`mark_completed`) is roadmap 3.2's rule in one guarded method**: it closes KS-32's rest and KS-30's stale marks, and it keeps the A2 chord pin.
- **The harness oracles fit acceptance 3**: "nothing stuck" and "whole-press symmetry" each have a failing self-test, `down_unseen` models RK7, and the seeded SplitMix64 properties are deterministic, cost no new dependency and print their seed and config.
- **The evidence is strong**:
  - every red-first fails by assertion with the predicted text, confirmed by three parties;
  - the tester's 12 mutations plus mine were all caught;
  - the tester's M12 found a real gap, and the tester closed it.
- **The hot path gets cheaper**: fewer `GetAsyncKeyState` calls for swallowed modifiers.

## Notes for later lanes

1. **1.1 (`repeat` flag)**: the per-key record already decides repeat suppression. 1.1 only stops `fire` on repeats. Immediate actions still fire on repeats of `Forgotten` and `Passed` keys today (KS-10).
2. **1.2 (mask / blocker v2)**: RK2 (Shift after Alt swallowed for its whole press) disappears when the blocker stops swallowing. The mask rule "after Alt or Win reached the OS" can read `is_passed`.
3. **1.3**: D4 (`mod.rs:510-513`) becomes reachable. Keep `key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending` and add an event-level twin.
4. **2.3**: P3's reset semantics:
   - decide the key-up half of the focus-change policy (the symmetry property's Focus mode skips it);
   - repair RK1, RK7 and Deviation 6;
   - consider treating every `Unknown` repeat as passed (impl.md Open points).
5. **KS-01 (on/off as an `AtomicBool`)**: with D2, a swallowed modifier held across `swch off/on` stays phantom-held unless `on` resets the engine (review open question 1).
6. **CLAUDE.md "Layout"**: mention `engine::properties` (cut Q6, an orchestrator edit after merge).
7. **Manual smoke for the owner**: impl.md M1-M9 (release daemon, `osk.exe` visible). M1 (Alt, Shift, release Alt, hold Shift) and M8 (CapsLock `Hold(300)` -> `wt.exe`) are the two most relevant to this lane.

Scratch artefacts (outside the repo): `analyst-test.txt`, `analyst-master-replay.txt`, `analyst-head-tests.txt`, `analyst-master-tests.txt` in this directory.
