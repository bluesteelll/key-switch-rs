# Context pack r1: p0-A3-suppression (KS-06 rest, KS-14; also KS-30, KS-32 rest)

Audience: tester and reviewer, round 1. Facts only; `UNVERIFIED` marks anything not checked by reading or by git. Line numbers
are on `fix/suppression-symmetry` `15e901d` unless prefixed `master:`. Lane inputs read: `D:/tmp/ks-orch/p0-A3-suppression/cut.md`
(plan + developer addendum at cut.md:413-505), `impl.md`, `critique.md` (headings and verdict only).

## Scope and delta

- Work tree `D:/key-switch-rs/key-switch-rs`, branch `fix/suppression-symmetry`, HEAD `15e901d`, tree clean at read time.
- `master` = `82f542f` (Merge branch 'fix/matching') = merge-base of `master` and the branch.
- `git log --oneline master..fix/suppression-symmetry` (oldest first), with the role each commit plays:

| Commit | Subject | Role | Files |
|---|---|---|---|
| `6661c9d` | Tests: scenario checks for stuck keys and per-press suppression symmetry | harness (green) | testing.rs +201/-, scenarios.rs +47 |
| `20d4844` | Engine: red-first: a modifier whose key-down was swallowed survives the resync (KS-14) | RED 1 | keys.rs tests +37 (hunk at `mod tests`), scenarios.rs +237 |
| `8772b73` | Engine: the modifier resync leaves keys whose key-down was swallowed alone (KS-14) | fix 1 (D1, D2) | keys.rs, mod.rs, env.rs, hook/keyboard_hook.rs (comment), hook/keyboard_hook_callback.rs (comment), README, CHANGELOG |
| `72ea8d6` | Engine: red-first: a key whose key-down was swallowed keeps its repeats swallowed across a focus change (KS-06) | RED 2 | keys.rs tests (1 assertion flipped, hunk at master-line 297), scenarios.rs +101 |
| `cc3892d` | Engine: a key-down a focus change forgot still swallows its repeats (KS-06) | fix 2 (I1 b) | keys.rs, mod.rs, scenarios.rs (comments), README, CHANGELOG |
| `afdb60c` | Engine: red-first: a key whose key-down reached Windows keeps its key-up (KS-06, KS-32) | RED 3 | mod.rs tests +36 (hunk inside `mod tests`), scenarios.rs +82 |
| `12be46b` | Engine: a key-down that reaches Windows always lets its key-up through (KS-06, KS-32, KS-30) | fix 3 (D3, D4, D5) | keys.rs, mod.rs, chords.rs (docs), sequences.rs (docs), README, CHANGELOG |
| `9dafc77` | Engine: property tests of the suppression invariant over generated key scripts (KS-06, KS-14) | acceptance 3 | properties.rs new +236, mod.rs +3 |
| `15e901d` | Tests: suppression scenarios auto-repeat only the key pressed last | test fix-up | scenarios.rs 51 lines |

- Red commits touch test code only (verified with `git show -U0`): `20d4844` keys.rs hunk `@@ -335,0 +336,37 @@ mod tests`;
  `72ea8d6` keys.rs hunk `@@ -297 +297,4 @@ mod tests`; `afdb60c` mod.rs hunk `@@ -552,0 +553,36 @@ mod tests`.
- Engine non-test code at `20d4844` equals `master` (`git diff 20d4844 master` over keys/mod/chords/sequences/gestures/matching
  shows only the keys.rs `mod tests` hunk).
- `git diff --stat master..fix/suppression-symmetry`: 12 files, +1168 / -164.

| File | +/- (stat) |
|---|---|
| CHANGELOG.md | 43 |
| README.md | 21 |
| src/engine/chords.rs | 8 (docs only) |
| src/engine/env.rs | 5 (docs only) |
| src/engine/keys.rs | 177 |
| src/engine/mod.rs | 133 |
| src/engine/properties.rs | +236 (new) |
| src/engine/scenarios.rs | 488 |
| src/engine/sequences.rs | 13 (docs only) |
| src/engine/testing.rs | 201 |
| src/hook/keyboard_hook.rs | 3 (comment only) |
| src/hook/keyboard_hook_callback.rs | 4 (comment only) |

## File map

| Range | What lives there | Why it matters this round |
|---|---|---|
| `src/engine/keys.rs:17-34` | `MODIFIERS` (11 codes: generic + sided Shift/Ctrl/Alt, LWin, RWin); doc now says swallowed modifiers are left out of the sync | KS-14 scope; generic slots 0x10-0x12 included |
| `src/engine/keys.rs:42-57` | `enum LastDown { Unknown, Passed, Suppressed, Forgotten }`; `Forgotten` new (`:52-56`) | D1; private, `derive(Clone, Copy, PartialEq, Eq)` (`:43`), no `Display` |
| `src/engine/keys.rs:59-71` | `KeyState { active: [bool;256], last_down: [LastDown;256] }` | per-key record the invariant reads |
| `src/engine/keys.rs:81-96` | `sync_modifiers(env, skip)`: skips `skip` and any modifier with `key_down_swallowed` (`:91`), else `active[i] = env.os_key_down(m)` (`:94`) | D2 / KS-14 core |
| `src/engine/keys.rs:98-105` | `key_down_swallowed(index)`: `Suppressed | Forgotten` | shared by sync skip and `is_blocked` |
| `src/engine/keys.rs:107-121` | `press` (returns previous `active`, the repeat detector) / `release` (clears `active` only, leaves `last_down`) | repeat detection the spec protects |
| `src/engine/keys.rs:123-128` | `mark_blocked` -> `Suppressed` (unconditional) | first suppressed key-down |
| `src/engine/keys.rs:130-141` | `mark_completed(vk)`: sets `Suppressed` only if in range, `active[i]`, and record != `Passed` | D5 (KS-30, KS-32) |
| `src/engine/keys.rs:143-148` | `mark_passed` -> `Passed` | D3 / first passed key-down |
| `src/engine/keys.rs:150-156` | `is_blocked`: true for `Suppressed` or `Forgotten` | I1(b): forgotten press keeps repeats swallowed |
| `src/engine/keys.rs:158-161` | `is_passed`: `Passed` only | D4 read |
| `src/engine/keys.rs:163-172` | `take_blocked`: replaces record with `Unknown` (any state), returns true only for `Suppressed` | key-up decision; `Forgotten` -> false |
| `src/engine/keys.rs:174-184` | `forget_blocked` (renamed from `clear_all_blocked`): `Suppressed` -> `Forgotten` | D1 |
| `src/engine/keys.rs:205-475` | unit tests (see Tests table) incl. `NothingDown` env `:378-395` | red-first + guards |
| `src/engine/mod.rs:36-43` | module doc, State bullet (forgotten records) | doc drift check |
| `src/engine/mod.rs:92-146` | module doc rules 1-5, key-up paragraph, "known defects" (KS-14 removed) | doc vs code |
| `src/engine/mod.rs:160-164` | `#[cfg(test)] mod scenarios;` and new `#[cfg(test)] mod properties;` | registration |
| `src/engine/mod.rs:282-293` | `on_key`: `sync_modifiers(env, Some(event.vk))` at `:287` before every key-down and key-up | KS-14 call site per event |
| `src/engine/mod.rs:312-327` | `on_foreground_changed`: `sync_modifiers(env, None)` `:325` then `forget_blocked()` `:326`; doc names RK1 / 2.3 | D1 order |
| `src/engine/mod.rs:329-487` | `on_key_down`: `press` `:331`; binding loop `:337-451`; deferred arms `continue` on repeat `:365-367`, `:377-379`, `:391-393`; sequence/chord `continue` on repeat `:407-409`, `:432-434`; completion -> `mark_completed` `:425-427`, `:444-446`; tail `:453-485` | D3 tail and D5 call sites |
| `src/engine/mod.rs:453-477` | repeat tail: `is_blocked` -> suppress `:471-472`; `is_passed` -> pass `:473-474`; else if not suppressed -> `mark_passed` `:475-476` | D3 + I1(b) |
| `src/engine/mod.rs:478-485` | first key-down: suppressed -> `mark_blocked` `:479`; passed -> `mark_passed` `:484` | spec item 2 |
| `src/engine/mod.rs:489-515` | `on_key_up`: `release` `:491`, `resolve_tap` `:498`, `release_hold` `:500`, `was_passed` read `:510` before `take_blocked` `:511`, `suppress = was_blocked || (gesture_bound && !was_passed)` `:512-513` | D4 |
| `src/engine/mod.rs:518-600` | unit tests; `FixedEnv` `:525`; new `key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending` `:566-599` | D4 pin (state set directly at `:583`) |
| `src/engine/env.rs:32-41` | `Env::os_key_down` contract: never asked about current key nor a modifier whose key-down was swallowed | D2 contract |
| `src/engine/chords.rs:10-17`, `:50-55`, `:98-100` | docs on completion marks; `Completed.keys` = `done.seen` codes (`:100`) | D5 input (sided event codes) |
| `src/engine/sequences.rs:12-17`, `:42-50`, `:122-139` | docs; `last_step_keys` (`:128`) returns last step's non-modifier keys | D5 input |
| `src/engine/testing.rs:1-39` | module doc incl. `down_unseen`, `up_unseen`, two continuous checks | harness contract |
| `src/engine/testing.rs:47-93` | `FakeEnv`: `os_down[256]` sided; `os_key_down` folds generic 0x10/0x11/0x12 to L||R (`:77-84`) | model of Windows state |
| `src/engine/testing.rs:96-106` | `physical_key`: generic names map to the left sided code | scripts press sided codes |
| `src/engine/testing.rs:117-136` | `Scenario` fields: `context`, `check_symmetry`, `press_suppressed[256]`, `check_stuck` | new state |
| `src/engine/testing.rs:167-186` | `with_context`, `check_press_symmetry`, `check_nothing_stuck` | new API |
| `src/engine/testing.rs:222-256` | `up_unseen` (clears `held`, `os_down`, `press_suppressed`) / `down_unseen` (sets `held`, `os_down`; `press_suppressed = Some(false)` if symmetry on) | lost/unseen events |
| `src/engine/testing.rs:258-265` | `foreground_changed` -> `engine.on_foreground_changed(&env)` | focus change step |
| `src/engine/testing.rs:362-390` | `expect_nothing_stuck`; `key_event`: OS state changes only when not suppressed (`:379-381`) | stuck model |
| `src/engine/testing.rs:392-418` | `check_press`: "down" records, repeat/up compared, "up" clears; text `"{label}: press of {key}: key-down {..}, {what} {..}"` `:409-413` | symmetry oracle |
| `src/engine/testing.rs:420-446` | `push_step` runs `fail_if_stuck` after every step when on; text `"{label}: VK 0x{vk:02X} is down in Windows but physically up"` `:442-444` | stuck oracle |
| `src/engine/testing.rs:486-507` | `fail`: `scenario: {message}\n` + optional `context: {..}\n` + transcript | failure format |
| `src/engine/properties.rs:1-236` | property tests (new); see Facts 30-38 | acceptance 3 |
| `src/engine/scenarios.rs:20-39` | `DEFAULT_PAIR` `:21-24`; `with_alt_shift_blocker` `:31-39` (blocker `KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT])`, then `from_ron_str`) | shared by scenarios and properties |
| `src/engine/scenarios.rs:750-760` | `chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` (`:752`), comment reworded | pinned A2 behaviour kept by D5 |
| `src/engine/scenarios.rs:1266-1512` | "Auto-repeat follows its key-down (KS-06)" (A2 pins); removed `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck` (was master:1381-1405); comments reworded at `:1403-1410`, `:1474-1479` | pinned set, one removal |
| `src/engine/scenarios.rs:1514-1887` | "Suppression symmetry (KS-06, KS-14)": 12 new scenarios | red-first + acceptance |
| `src/engine/scenarios.rs:1889-1951` | harness self-tests (4 new) | oracle validity |
| `src/hook/keyboard_hook_callback.rs:116-122` | `HookEnv::os_key_down` = `GetAsyncKeyState(vk) & 0x8000` | real Env behind D2 (unchanged code) |
| `src/hook/keyboard_hook_callback.rs:150-165` | `decide_event`: sentinel check `:156` before `with_engine(on_key)` `:164` | R1/R2 path (unchanged code) |
| `src/hook/keyboard_hook_callback.rs:244-262` | `foreground_changed` WinEvent callback, doc edited `:247-249`; calls `on_foreground_changed` `:261` | comment-only change |
| `src/hook/keyboard_hook.rs:129-133` | `install` comment on the foreground hook, edited `:131-132` | comment-only change |
| `README.md:355-370` | behaviour-rules bullet: symmetry rule, KS-14 resync sentence, "One gap remains" (focus change, 2.3) | user docs |
| `README.md:428` | `block_original_combo` row: blocker swallows Shift-after-Alt for the whole press until 1.2 | RK2 disclosure |
| `CHANGELOG.md:87-108` | KS-06 entry (focus-change repeats now swallowed for every kind; unseen-modifier sentence) | Keep a Changelog, Unreleased/Fixed |
| `CHANGELOG.md:123-144` | new KS-14, KS-30, KS-32 entries | same |

## Facts

Engine state and resync

1. `sync_modifiers` skips a modifier whose record is `Suppressed` or `Forgotten`; for such a modifier it neither calls
   `env.os_key_down` nor writes `active` (`src/engine/keys.rs:89-95`, `:100-104`).
2. `Passed` and `Unknown` modifiers are still overwritten from `env.os_key_down` on every event (`src/engine/keys.rs:94`), called
   from `src/engine/mod.rs:287` with `skip = Some(event.vk)` and from `src/engine/mod.rs:325` with `skip = None`.
3. `release` clears `active` only and does not touch `last_down` (`src/engine/keys.rs:117-121`); the record is reset only by
   `take_blocked` (to `Unknown`, `src/engine/keys.rs:167`), `mark_*` and `forget_blocked`.
4. `forget_blocked` turns every `Suppressed` into `Forgotten`; `Passed`, `Unknown` and `Forgotten` are untouched
   (`src/engine/keys.rs:178-184`). It is called only from `on_foreground_changed` (`src/engine/mod.rs:326`).
5. `is_blocked` is true for `Suppressed` and `Forgotten` (`src/engine/keys.rs:154-156`); `take_blocked` returns true only for
   `Suppressed` (`src/engine/keys.rs:167-168`).
6. `mark_completed` writes `Suppressed` only when `slot(vk)` is `Some`, `active[index]` is true and the record is not `Passed`
   (`src/engine/keys.rs:134-141`). A `Forgotten` or `Unknown` held key is therefore re-marked `Suppressed`.
7. `slot` bounds every index to `< 256` (`src/engine/keys.rs:37-40`); `sync_modifiers` indexes by `MODIFIERS` codes, all < 256
   (`src/engine/keys.rs:22-34`, `:90`).

Decision rules

8. Deferred triggers (`Tap`, `Hold`, `DoubleTap`), sequences and chords `continue` on a repeat (`src/engine/mod.rs:365-367`,
   `:377-379`, `:391-393`, `:407-409`, `:432-434`); only `Immediate` combos can set `suppress` on a repeat (`:355-363`).
9. A repeat is suppressed when `is_blocked` (`src/engine/mod.rs:471-472`), passed when `is_passed` (`:473-474`), otherwise
   keeps the bindings' `suppress` and, if that is false, records `Passed` (`:475-476`). A no-record repeat that is suppressed
   records nothing.
10. A first key-down records `Suppressed` if `suppress` (`src/engine/mod.rs:478-479`), else `Passed` (`:484`), overwriting any
    earlier record.
11. Sequence and chord completion call `mark_completed` for `last_step_keys` / `keys` (`src/engine/mod.rs:425-427`,
    `:444-446`); the completing key itself is then marked by the first-key-down tail (`:478-479`), because completion sets
    `suppress = true` (`:423`, `:442`).
12. `on_key_up` reads `was_passed` (`src/engine/mod.rs:510`) before `take_blocked` (`:511`), and suppresses iff
    `was_blocked || (gesture_bound && !was_passed)` with `gesture_bound = tap_fired || hold != HoldRelease::NotPending`
    (`:512-513`). Truth table per the code: `Suppressed` -> suppress; `Passed` -> pass; `Unknown`/`Forgotten` -> suppress iff
    gesture-bound.
13. Tap and Hold resolution on key-up is unchanged from master: `resolve_tap` `src/engine/mod.rs:498`, `release_hold` `:500`.
14. `on_foreground_changed` keeps the order resync then forget (`src/engine/mod.rs:325-326`).
15. Chord completion `keys` are the codes seen for the chord (`done.seen`, `src/engine/chords.rs:100`); sequence
    `last_step_keys` are the last step's non-modifier keys (`src/engine/sequences.rs:122-139`).
16. The adapter is unchanged in code: `decide_event` checks `is_injected_event` (`src/hook/keyboard_hook_callback.rs:156`)
    before `with_engine(on_key)` (`:164`); the WinEvent callback calls `on_foreground_changed` (`:261`). Only doc comments
    changed in `src/hook/*` (diff hunks at `keyboard_hook_callback.rs:244-249`, `keyboard_hook.rs:128-133`).
17. `src/engine/mod.rs` keeps `#![forbid(unsafe_code)]` (`src/engine/mod.rs:148`).
18. No added line in `git diff master..HEAD -- src/engine` matches `Mutex|thread::|unsafe|Ordering::|AtomicU|AtomicBool`
    (grep, zero matches); impl.md "Unsafe / atomics added" says the same.

Harness (`src/engine/testing.rs`)

19. The modelled OS key state changes only on a passed event (`src/engine/testing.rs:379-381`); `held` follows the script
    (`:382`).
20. `fail_if_stuck` fails on the first `vk` with `os_down[vk] && !held[vk]` (`src/engine/testing.rs:437`), run after every
    step when `check_nothing_stuck` is on (`:428-430`).
21. `check_press` stores the key-down's outcome on "down" (`src/engine/testing.rs:397-399`), compares on "repeat" and "up"
    (`:401-414`) and clears on "up" (`:415-417`); a key with `None` (pressed before the check was enabled) is not checked.
22. `down_unseen` sets `held` and `os_down` and, with symmetry on, records the press as passed (`src/engine/testing.rs:243-256`);
    `up_unseen` clears `held`, `os_down` and `press_suppressed` (`:225-236`). Neither calls the engine.
23. `FakeEnv::os_key_down` folds `VK_SHIFT`/`VK_CONTROL`/`VK_MENU` to the OR of both sided codes (`src/engine/testing.rs:77-84`);
    `physical_key` maps generic names to the left code (`:96-106`).
24. `fail` prints `scenario: {message}`, then `context: {context}` if set, then the transcript (`src/engine/testing.rs:501-506`).
25. Step methods carry `#[track_caller]` (e.g. `src/engine/testing.rs:196`, `:205`, `:214`, `:224`, `:242`, `:259`, `:269`).

Scenario helpers

26. `with_alt_shift_blocker` puts `Binding::new_auto_blocker(KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT]))` first, then the
    RON bindings, without reading the registry (`src/engine/scenarios.rs:31-39`).
27. `Scenario::with_bindings` sorts with `sort_by_specificity` (`src/engine/testing.rs:149-150`).
28. The A2 test `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck` (master `scenarios.rs:1381-1405`, which
    expected Shift's repeats and key-up to pass after Alt's key-up) is deleted; T5 at `src/engine/scenarios.rs:1649` replaces it
    with the opposite expectations (diff hunk `@@ -1378,38 +1399,12 @@`).
29. Three pinned tests have comment-only edits: `src/engine/scenarios.rs:752` (`chord_completed_after_a_focus_change_...`),
    `:1403`, `:1474` (diff hunks `@@ -729,8`, `@@ -1378,38` tail, `@@ -1478,9`).

Property tests (`src/engine/properties.rs`)

30. Generator: SplitMix64 with the standard constants (`src/engine/properties.rs:24-40`); `SEEDS = 64`, `STEPS = 250` (`:42-43`);
    13 keys (`:45-48`); 13 delays incl. 0, 1, 33, 199-201, 500, 600, 1000 ms (`:52`).
31. Six configs in order: `default` (blocker + `DEFAULT_PAIR` + `Alt+Shift+K`), `immediate`, `gestures`, `chords`, `sequences`,
    `mixed` (blocker) (`src/engine/properties.rs:62-125`).
32. Step mix (`roll = below(100)`): `0..35` down an up key (else repeat), `35..60` repeat a held key (else down), `60..85` up a held
    key (else down), in Focus mode 1 in 10 ups is `up_unseen`; `85..95` advance; `95..97` focus change in Focus mode else advance;
    rest `advance_ms_stalled` (`src/engine/properties.rs:153-186`).
33. The generator repeats any held key, not only the key pressed last (`src/engine/properties.rs:161-162`); impl.md Deviation 8
    states this is deliberate.
34. Seed per run = `seed.wrapping_mul(0x100) + config_index` (`src/engine/properties.rs:192`); context string
    `"config {name}, seed {seed}, {mode:?} mode"` (`:199-202`).
35. Strict mode turns on `check_nothing_stuck` and `check_press_symmetry` (`src/engine/properties.rs:203-206`); Focus mode only
    `check_nothing_stuck`.
36. After STEPS, every held key in `KEYS` order gets `up`, then `advance_ms(2000)` and `expect_nothing_stuck()`
    (`src/engine/properties.rs:212-217`).
37. Tests: `every_press_is_suppressed_or_passed_as_a_whole` (`src/engine/properties.rs:228-231`, Strict) and
    `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups` (`:233-236`, Focus); 6 x 64 runs each.
38. The generator never calls `down_unseen` (no call in `src/engine/properties.rs:146-187`), so D3 (no-record repeat) is reachable
    only via resync-introduced `Unknown` modifiers; impl.md states D3, D4 and I1(b) are "not reachable by the generator".

Docs

39. README behaviour bullet states the symmetry rule, the KS-14 resync rule, and one remaining gap (focus change: release passes
    unless a Tap/Hold is pending, roadmap 2.3) (`README.md:355-370`).
40. CHANGELOG Unreleased/Fixed has new entries KS-14 (`CHANGELOG.md:123-135`), KS-30 (`:136-140`), KS-32 (`:141-144`) and an
    amended KS-06 entry (`:87-108`).
41. Book pages are not edited in this lane (no `book/` path in the diff stat); impl.md "Docs" lists the pages left to the phase
    book sync: `book/src/config/known-issues.md` rows KS-06/14/30/32, `triggers-conditions.md` 11.8, `kinds.md:96` and 9.3,
    `architecture.md:184`, `:208`, `:491-494`, `:524`, `glossary.md:287`, `:343`, `audit.md` status lines.
42. Roadmap acceptance for Workstream A: "A repeat of a suppressed key is suppressed, and its key-up is suppressed exactly once"
    (`book/src/roadmap.md:98`); the A bullet "makes `sync_modifiers` skip blocked keys" (`book/src/roadmap.md:81`); roadmap 2.3
    acceptance "Holding `Win+Alt+T` while the foreground window changes keeps the T repeats suppressed" (`book/src/roadmap.md:304`).

## Tests: red-first and acceptance proofs

All tests live in the lib crate, path prefix `key_switch_rs::engine::` (crate `src/lib.rs:8` `pub mod engine;`). Red texts are
quoted from impl.md (developer runs); the scout did not re-run them (UNVERIFIED here).

### Commands

```text
# full gate on HEAD (work tree D:/key-switch-rs/key-switch-rs)
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
# targeted
cargo test --locked --lib engine::scenarios
cargo test --locked --lib engine::keys::tests
cargo test --locked --lib engine::tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending
cargo test --locked --lib engine::properties
# red replay at a red commit (scratch worktree per docs/WORKFLOW.md:69-70, scratch dirs per :134)
git worktree add D:/tmp/ks-orch/p0-A3-suppression/wt/<commit> <commit>
CARGO_TARGET_DIR=D:/tmp/ks-orch/p0-A3-suppression/target-wt cargo test --locked --lib <test path>   (run inside the worktree)
```

Expected on HEAD (impl.md "Gates"): fmt, check and clippy `-D warnings` pass; `engine::` 146 tests, and `engine::` + `hook::` +
`readme` 157 tests, all green; the full `cargo test --locked` was NOT run by the developer ("left to the tester"). Property
module runtime 0.70-0.74 s in debug (impl.md). No `#[ignore]` test added.

### RED 1 (`20d4844`; engine = master) -> green at `8772b73` (KS-14, D1, D2)

| Test (HEAD line) | Expected red text at `20d4844` |
|---|---|
| `engine::scenarios::alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier` (`scenarios.rs:1517`) | `down X at +0 ms: expected the event to be suppressed, it was passed` |
| `engine::scenarios::shift_held_by_a_double_tap_still_counts_for_shift_capslock` (`:1560`) | `down CapsLock at +50 ms: expected to fire [ToggleCapsLock], fired [SwitchLanguage]` |
| `engine::scenarios::one_long_press_of_shift_does_not_fire_its_double_tap` (`:1586`) | `repeat LShift at +500 ms: expected to fire [], fired [ToggleCapsLock]` |
| `engine::scenarios::ctrl_shift_hold_with_ctrl_released_first_neither_leaks_nor_sticks_shift` (`:1616`) | `repeat LShift at +500 ms: expected the event to be suppressed, it was passed` |
| `engine::scenarios::shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press` (`:1649`, T5) | `repeat LShift at +500 ms: expected the event to be suppressed, it was passed` |
| `engine::scenarios::suppressed_modifier_stays_an_auto_repeat_across_a_focus_change` (`:1682`, T10) | `repeat LShift at +500 ms: expected to fire [], fired [ToggleCapsLock]` |
| `engine::keys::tests::resync_leaves_a_modifier_whose_key_down_was_suppressed` (`keys.rs:397`) | `a modifier whose key-down was swallowed stays held through the resync` |

New green guards at `8772b73`: `keys::tests::resync_still_repairs_a_modifier_whose_key_down_passed` (`keys.rs:445`),
`keys::tests::forgotten_modifier_is_left_alone_by_the_resync` (`keys.rs:457`); renamed `forget_blocked_resets_every_flag`
(`keys.rs:298`), `forget_blocked_keeps_passed_key_downs` (`keys.rs:308`).

### RED 2 (`72ea8d6`; engine = `8772b73`) -> green at `cc3892d` (KS-06 focus change, I1 b)

| Test (HEAD line) | Expected red text |
|---|---|
| `engine::scenarios::chord_keys_held_across_a_focus_change_keep_their_repeats_suppressed` (`:1705`) | at `72ea8d6`: `repeat J at +500 ms: expected the event to be suppressed, it was passed`; HEAD version (repeats K, `15e901d`) replayed on `8772b73`: `repeat K at +500 ms: expected the event to be suppressed, it was passed` |
| `engine::scenarios::deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:1736`) | `repeat CapsLock at +500 ms: expected the event to be suppressed, it was passed` |
| `engine::scenarios::immediate_combo_that_stops_matching_after_a_focus_change_keeps_its_key_suppressed` (`:1778`) | `repeat F13 at +500 ms: expected the event to be suppressed, it was passed` |
| `engine::keys::tests::forget_blocked_keeps_passed_key_downs` (`keys.rs:308`, assertion flipped `!is_blocked(20)` -> `is_blocked(20)`) | `a forgotten key-down was still swallowed, so its repeats are too` |

### RED 3 (`afdb60c`; engine = `cc3892d`) -> green at `12be46b` (D3, D4, D5; KS-06 rest, KS-32 rest, KS-30)

| Test (HEAD line) | Expected red text |
|---|---|
| `engine::scenarios::sequence_completion_keeps_the_key_up_of_a_key_whose_down_passed` (`:1805`, T6) | `up F14 at +0 ms: VK 0x7D is down in Windows but physically up` |
| `engine::scenarios::chord_completion_keeps_the_key_up_of_a_chord_key_whose_down_passed` (`:1828`) | `up J at +0 ms: VK 0x4A is down in Windows but physically up` |
| `engine::scenarios::modifier_held_before_the_engine_saw_it_keeps_passing_once_a_repeat_passed` (`:1857`) | `repeat LShift at +533 ms: expected the event to be passed, it was suppressed` (HEAD key order from `15e901d`, replayed on `cc3892d`, same text) |
| `engine::tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending` (`mod.rs:566`) | `a key-up whose key-down reached Windows must reach it too` |

New green guard at `12be46b`: `keys::tests::completion_marks_only_held_keys_whose_key_down_did_not_pass` (`keys.rs:416`).

### Harness self-tests (`6661c9d`, green; `src/engine/scenarios.rs`)

| Test | Expectation |
|---|---|
| `press_symmetry_check_reports_a_key_up_that_differs_from_its_key_down` (`:1907`) | `should_panic(expected = "press of F13: key-down suppressed, up passed")` (`:1906`) |
| `press_symmetry_check_reports_a_repeat_that_differs_from_its_key_down` (`:1919`) | `should_panic(expected = "press of LShift: key-down passed, repeat suppressed")` (`:1918`); key order changed in `15e901d`; impl.md: panics as expected at `6661c9d`, `8772b73`, `cc3892d`, HEAD |
| `nothing_stuck_holds_for_a_plain_press_and_an_unseen_release` (`:1934`) | green |
| `context_is_printed_with_a_failure` (`:1946`) | `should_panic(expected = "context: seed 7")` (`:1945`) |

### Property tests (`9dafc77`; acceptance 3)

- On HEAD: both green (impl.md).
- Replayed by the developer on the master engine (worktree at `20d4844` + `properties.rs`, removed afterwards), both red:
  - Strict: `repeat LAlt at +3079 ms: press of LAlt: key-down suppressed, repeat passed` (config default, seed 0, Strict mode);
  - Focus: `up LCtrl at +4114 ms: VK 0xA2 is down in Windows but physically up` (config gestures, seed 3, Focus mode).
  Replay route per Facts above: `20d4844` has `with_alt_shift_blocker`, `DEFAULT_PAIR` (`git show 20d4844:src/engine/scenarios.rs`
  `:22`, `:31`) and `with_context`, `check_press_symmetry`, `down_unseen` in `testing.rs` (`git show 6661c9d:...` `:168`, `:176`,
  `:243`); `properties.rs` additionally needs `#[cfg(test)] mod properties;` in `mod.rs` (UNVERIFIED that nothing else is needed).
- Developer mutations (reverted):
  - D2 skip removed: Strict red `repeat LAlt at +3079 ms: press of LAlt: key-down suppressed, repeat passed`; Focus green.
  - `mark_completed` without its guards: Strict red `repeat F14 at +4403 ms: press of F14: key-down passed, repeat suppressed`
    (sequences, seed 1); Focus red `up F16 at +8441 ms: VK 0x7F is down in Windows but physically up` (sequences, seed 0).

### Acceptance mapping (brief -> test)

| Acceptance | Tests (HEAD) |
|---|---|
| 1. a repeat of a suppressed key is suppressed, and its key-up is suppressed exactly once | T5 `scenarios.rs:1649` (two repeats `:1664-1670` suppressed, key-up `:1671-1672` suppressed, next press passes both edges `:1674-1677`); T4 `:1616`; T3 `:1586`; A2 pins `repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired` `:1348`, `immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once` `:1373`; Strict property `properties.rs:229` |
| 2. KS-06 default-config route (Alt+Shift blocker, release Alt while holding Shift) leaves nothing stuck | T5 `scenarios.rs:1649` with `check_nothing_stuck()` `:1655`, on `with_alt_shift_blocker(DEFAULT_PAIR)` `:1654` |
| 3. property-style test over generated down/repeat/up sequences | `properties.rs:229`, `:234` |
| Spec 2: passing key-down clears blocked state | `mod.rs:484` (first key-down), `:475-476` (no-record repeat); tests T6 `:1805`, `:1828`, `:1857`, `mod.rs:566`; A2 pin `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` `scenarios.rs:1269` |
| Spec 3: resync must not overwrite a suppressed key nor reset its repeat detection | `keys.rs:397`, `:457`; T1 `:1517`, T2 `:1560`, T3 `:1586`, T4 `:1616`, T10 `:1682`; repair guard `keys.rs:445`; A2 pin `modifier_lost_by_the_os_is_resynced_before_matching` `scenarios.rs:178` |

### Manual checks (human only; impl.md table M1-M9)

M1-M5 as cut.md:332-336; M6 changed expectation (CapsLock light unchanged, layout switches on release); M7 (chord J+K ->
Notepad), M8 (CapsLock `Hold(300)` -> `wt.exe`), M9 (`swch off`, hold LShift, `swch on`, type `a`, press/release LAlt) new. None
has an `#[ignore]` stub (impl.md "Gates").

## Open points (from impl.md "Open points" and "Deviations")

1. RK7 class, for roadmap 2.3: a non-modifier held since before the hook saw it reads as a first key-down at its next repeat; if
   a binding swallows it, its key-up is swallowed while Windows has it down (impl.md Open points; cut.md RK7 at cut.md:349).
2. A held `Unknown` modifier (resync-introduced, Windows has it down) can still be marked `Suppressed` by a chord completion,
   because `mark_completed` excludes only `Passed` (`src/engine/keys.rs:137`); reachable only via a chord key pressed, released
   and pressed again unseen inside the chord window (impl.md Deviation 6). Left for 2.3.
3. I1(b) cost: after a lost key-up and a focus change, the key's next physical press reads as an auto-repeat and is swallowed once
   (`is_blocked` counts `Forgotten`, `src/engine/keys.rs:154-156`); documented in `on_foreground_changed` doc
   (`src/engine/mod.rs:319-323`) and cut.md:457-459. Repair owned by 2.3.
4. RK1 (corrected, cut.md:486-493): a `Suppressed`/`Forgotten` modifier whose key-up is lost stays held for the engine (no resync,
   `src/engine/keys.rs:91`); with the default config a phantom-held Shift makes CapsLock match `Shift+CapsLock` until Shift is
   pressed and released.
5. D4 has no event-level red-first; it is pinned only by the unit test that sets `Passed` directly (`src/engine/mod.rs:566-599`,
   `:583`); developer argues the state is unreachable until roadmap 1.3 (cut.md:448-454).
6. T10 (`scenarios.rs:1682`) has no suppression expectation on its repeat or key-up (impl.md Deviation 4); the three I1(b) scenarios
   pin suppression of forgotten repeats.
7. The property generator repeats any held key (Fact 33), while scenarios since `15e901d` repeat only the key pressed last
   (impl.md Deviation 8).
8. The full `cargo test --locked` run on HEAD is left to the tester (impl.md Gates).
9. `CLAUDE.md` "Layout" names scenario tests in `engine::scenarios` only (`CLAUDE.md:28`, `:88`); cut Q6 / impl.md ask the
   orchestrator to add `engine::properties` after merge.
10. CHANGELOG section for RK2 (Shift after the blocker stays swallowed for its whole press) is placed under Fixed/KS-14
    (`CHANGELOG.md:131-135`); cut Q5 marks this as the orchestrator's call.
11. Book pages listed in Fact 41 are not updated in this lane (phase book sync).

## Invariants in play

| Item | Where | What this round touches |
|---|---|---|
| R1 hook callback | `src/hook/keyboard_hook_callback.rs:215-242`, `:150-165`; engine path `src/engine/mod.rs:282-293` | no adapter code change; per-event `sync_modifiers` now reads `last_down` before `GetAsyncKeyState` (`keys.rs:91-94`); new indexing bounded (`keys.rs:37-40`, `:90`) |
| R2 sentinel | `src/hook/keyboard_hook_callback.rs:156` | untouched |
| R3 shared state | `src/hook/keyboard_hook.rs:79-88` (`with_engine`) | `KeyState` stays inside the engine; no new static or lock (impl.md) |
| R4 atomics | none in `src/engine` diff (Fact 18) | none |
| R5 builders | harness methods consume `self` (`src/engine/testing.rs:168-186`) | test-only API |
| R6 Display | `LastDown` private, no `Display` (`src/engine/keys.rs:43-57`) | no public variant added |
| R7 specificity | `Scenario::with_bindings` sorts (`src/engine/testing.rs:149-150`); blocker first in `with_alt_shift_blocker` (`scenarios.rs:31-39`) | blocker order among equal counts (KS-18) |
| R8 key identity | records per sided code; `MODIFIERS` include generic 0x10-0x12 (`keys.rs:22-34`); `FakeEnv` generic folding (`testing.rs:77-84`); chord marks `done.seen` codes (`chords.rs:100`) | D2/D5 key codes |
| R10 message loop | WinEvent `foreground_changed` on main thread (`keyboard_hook_callback.rs:250-262`) | doc-only change |
| KS-06 | `mod.rs:453-485`, `:505-513` | repeat tail, key-up rule |
| KS-14 | `keys.rs:88-105`, `mod.rs:287`, `:325` | resync skip |
| KS-30 / KS-32 | `keys.rs:134-141`, `mod.rs:425-427`, `:444-446` | completion marks |
| KS-29 / KS-31 (roadmap 2.3) | `mod.rs:312-327` | focus-change policy, lost key-ups (Open points 1-4) |
| KS-10 (roadmap 1.1) | `mod.rs:355-359` | Immediate still fires on repeats, incl. passed ones (scenario `:1857` comment) |
| KS-05 / KS-12 (roadmap 1.2) | blocker swallows Shift-after-Alt for the whole press (`README.md:428`) | RK2 |

## Not covered

- Not read: `src/engine/gestures.rs`, `src/engine/matching.rs` (unchanged in the diff), `src/engine/chords.rs` and
  `src/engine/sequences.rs` beyond the lines cited, `src/engine/scenarios.rs:40-1265` beyond the pinned tests cited,
  `src/engine/keys.rs:205-375` (pre-existing unit tests), `critique.md` body beyond headings and "Strengths".
- Not run: any `cargo` command; red texts, test counts, runtimes and mutation results are as reported by impl.md.
- Not checked: book pages listed in Fact 41 for current wording.
- `D:/tmp/ks-orch/p0-A3-suppression/dev-target/` exists (a developer `CARGO_TARGET_DIR`; contents not inspected).
