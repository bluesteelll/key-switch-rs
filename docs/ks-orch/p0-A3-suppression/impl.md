# Implementation: p0-A3-suppression round 1

Verdict: DONE. Branch `fix/suppression-symmetry`, base `master` @ `82f542f`, HEAD `15e901d`, tree clean.
The critique's I1 is resolved with option (b). That is the main deviation from the cut; see "Deviations" and the developer
addendum at the end of `cut.md`.

## Items

Line numbers are at HEAD.

- **Harness (cut D7, commit 1)**: `src/engine/testing.rs`.
  - `with_context` :168, `check_press_symmetry` :176, `check_nothing_stuck` :183, `down_unseen` :243 (new, see Deviations),
    `expect_nothing_stuck` :365, `check_press` :395, `push_step` :422, `fail_if_stuck` :436.
  - `fail` prints `context: ...` after the message.
  - Self-tests in `src/engine/scenarios.rs`: :1907 `press_symmetry_check_reports_a_key_up_that_differs_from_its_key_down`, :1919
    `press_symmetry_check_reports_a_repeat_that_differs_from_its_key_down`, :1934
    `nothing_stuck_holds_for_a_plain_press_and_an_unseen_release`, :1946 `context_is_printed_with_a_failure`. All green at
    `6661c9d`. The repeat self-test's key order was made physically possible in `15e901d`; it panics as expected on `6661c9d`,
    `8772b73`, `cc3892d` and HEAD.
- **KS-14 (D1, D2)**: `src/engine/keys.rs`.
  - `LastDown::Forgotten` :56; `sync_modifiers` skips `Suppressed`/`Forgotten` :88-104 (`key_down_swallowed` :100);
    `clear_all_blocked` -> `forget_blocked` :178.
  - Red-first in `20d4844`, green in `8772b73`. Each test ran red first, then green:
    - `alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier` (:1517): red "down X at +0 ms: expected the
      event to be suppressed, it was passed";
    - `shift_held_by_a_double_tap_still_counts_for_shift_capslock` (:1560): red "down CapsLock at +50 ms: expected to fire
      [ToggleCapsLock], fired [SwitchLanguage]";
    - `one_long_press_of_shift_does_not_fire_its_double_tap` (:1586): red "repeat LShift at +500 ms: expected to fire [],
      fired [ToggleCapsLock]";
    - `ctrl_shift_hold_with_ctrl_released_first_neither_leaks_nor_sticks_shift` (:1616): red "repeat LShift at +500 ms:
      expected the event to be suppressed, it was passed";
    - `shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press` (:1649, replaces
      `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck`): red "repeat LShift at +500 ms: expected the
      event to be suppressed, it was passed";
    - `suppressed_modifier_stays_an_auto_repeat_across_a_focus_change` (:1682, T10): red "repeat LShift at +500 ms: expected to
      fire [], fired [ToggleCapsLock]";
    - `keys::tests::resync_leaves_a_modifier_whose_key_down_was_suppressed` (keys.rs :397): red "a modifier whose key-down was
      swallowed stays held through the resync".
  - New unit tests, green: `resync_still_repairs_a_modifier_whose_key_down_passed` :445,
    `forgotten_modifier_is_left_alone_by_the_resync` :457. Renamed: `forget_blocked_resets_every_flag` :298,
    `forget_blocked_keeps_passed_key_downs` :308.
- **KS-06, focus-change repeats (critique I1 b)**: `keys.rs` :154, where `is_blocked` counts `Forgotten`.
  - Red-first in `72ea8d6`, green in `cc3892d`:
    - `chord_keys_held_across_a_focus_change_keep_their_repeats_suppressed` (:1705): red "repeat J at +500 ms: expected the
      event to be suppressed, it was passed". In `15e901d` it repeats K, the key pressed last, instead; replayed on `8772b73` it is
      red with "repeat K at +500 ms: expected the event to be suppressed, it was passed", and green from `cc3892d` on;
    - `deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (:1736): red "repeat CapsLock at +500 ms: expected
      the event to be suppressed, it was passed";
    - `immediate_combo_that_stops_matching_after_a_focus_change_keeps_its_key_suppressed` (:1778): red "repeat F13 at +500 ms:
      expected the event to be suppressed, it was passed";
    - `keys::tests::forget_blocked_keeps_passed_key_downs`, assertion flipped: red "a forgotten key-down was still swallowed, so
      its repeats are too".
- **D3, D4, D5 (KS-06 rest, KS-32 rest, KS-30)**: `src/engine/mod.rs` :426, :445 (`mark_completed`), :475-477 (D3), :505-514
  (D4); `keys.rs` :134 (`mark_completed`).
  - Red-first in `afdb60c`, green in `12be46b`:
    - `sequence_completion_keeps_the_key_up_of_a_key_whose_down_passed` (:1805, T6): red "up F14 at +0 ms: VK 0x7D is down in
      Windows but physically up";
    - `chord_completion_keeps_the_key_up_of_a_chord_key_whose_down_passed` (:1828): red "up J at +0 ms: VK 0x4A is down in
      Windows but physically up";
    - `modifier_held_before_the_engine_saw_it_keeps_passing_once_a_repeat_passed` (:1857): red "repeat LShift at +533 ms:
      expected the event to be passed, it was suppressed". In `15e901d` its key order became physically possible: Ctrl, Alt, A and
      Shift are held from before; A's key-up triggers the resync; Ctrl's key-up makes the blocker match. Replayed on `cc3892d` it
      is red with the same text;
    - `tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending` (mod.rs :566): red "a key-up whose key-down
      reached Windows must reach it too".
  - New unit test: `keys::tests::completion_marks_only_held_keys_whose_key_down_did_not_pass` (:416).
- **Property tests (acceptance 3)**: `src/engine/properties.rs` (new), registered at mod.rs :164; commit `9dafc77`.
  - `every_press_is_suppressed_or_passed_as_a_whole` :229 and
    `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups` :234.
  - Parameters as the cut specifies: SplitMix64, SEEDS 64, STEPS 250, six configs, the cut's generator. Runtime 0.70-0.74 s in
    debug, so SEEDS stays 64.
  - **Replayed on the master engine**: a scratch worktree at `20d4844`, whose engine is unchanged from master, plus this module.
    It was removed afterwards. Both properties are red there:
    - strict: "repeat LAlt at +3079 ms: press of LAlt: key-down suppressed, repeat passed" (config default, seed 0, Strict mode);
    - focus: "up LCtrl at +4114 ms: VK 0xA2 is down in Windows but physically up" (config gestures, seed 3, Focus mode).
  - Mutations, run locally and then reverted by copying the file back:
    - no D2 skip: strict fails with "repeat LAlt at +3079 ms: press of LAlt: key-down suppressed, repeat passed"; focus stays
      green.
    - `mark_completed` without its held and not-`Passed` guards: strict fails with "repeat F14 at +4403 ms: press of F14:
      key-down passed, repeat suppressed" (sequences, seed 1); focus fails with "up F16 at +8441 ms: VK 0x7F is down in
      Windows but physically up" (sequences, seed 0).
  - Not reachable by the generator: D3 (needs `down_unseen`), D4 (unreachable) and I1(b). Without (b) a forgotten repeat passes,
    and a passed key-up is never stuck. The scenario tests above pin these three.

## Remarks

- **I1**: RESOLVED with option (b) at `keys.rs:154` (`is_blocked` counts `Forgotten`), commit `cc3892d`. Pinned by the three
  focus-change scenarios (:1705, :1736, :1778) and `forget_blocked_keeps_passed_key_downs`. Details, D3/D4/D5 routes and the cost
  are in the `cut.md` addendum.
- **M1**: RESOLVED in commit 1. `nothing_stuck_holds_for_a_plain_press_and_an_unseen_release` (scenarios.rs :1934) calls
  `check_nothing_stuck`; clippy `-D warnings` passes at every commit.
- **M2**: RESOLVED with comment edits in `8772b73`:
  - `src/hook/keyboard_hook_callback.rs`, the `foreground_changed` doc;
  - `src/hook/keyboard_hook.rs`, the `install` comment;
  - `src/engine/mod.rs`, the `on_key` resync comment and the `on_foreground_changed` doc;
  - `src/engine/keys.rs`, the `MODIFIERS` doc.
- **M3**: RESOLVED. `press_symmetry_check_reports_a_repeat_that_differs_from_its_key_down` (scenarios.rs :1919) pins the
  checker's repeat branch. The D2 mutation and the master replay show the strict property catching a repeat mismatch.
- **M4**: RESOLVED. The corrected RK1 and Q3 are in the `cut.md` addendum.
- **Critique open questions 1-3**: answered in the addendum. The book is left to the phase sync, the generic codes need no change,
  and Q5 is under Fixed.

## Deviations from the cut

1. **I1(b) instead of the cut's focus-change repeat policy.** Two extra commits (`72ea8d6` red, `cc3892d` fix), so the cut's
   commits 4/5/6 are commits 6/7/8 here. T7, T8 and T9 of the cut are replaced by:
   - the three (b) scenarios;
   - new D5-chord and D3 routes (title change; `down_unseen`).

   The cut's commit-5 README sentence ("Its repeats are then swallowed only while an `Immediate` binding matches them; once one
   reaches Windows ...") is replaced by the (b) rule.
2. **Harness: `down_unseen` added** (`testing.rs:243`). It is needed for the M3 self-test and the D3 route under (b). Step methods
   now go through `push_step`, so the stuck check runs after every step. `#[track_caller]` is on the step methods. The
   `outcome()` free fn replaces the closure in `expect_suppression`.
3. **D4 has no event-level red-first**. A `Passed` record with a pending Tap/Hold is unreachable under (b); the argument is in the
   addendum. D4 is implemented as the cut says (roadmap 1.3 will reach it) and pinned by a unit test in `mod.rs` that sets the
   record directly.
4. **T10** is kept as the cut defines it, with no suppression expectation on the repeat. Under (b) the repeat is suppressed;
   the three (b) scenarios pin that.
5. **`forget_blocked_keeps_passed_key_downs`**: one assertion was flipped (`!is_blocked(20)` -> `is_blocked(20)`). It pinned A2's
   reading of a forgotten key-down; (b) reverses that reading on purpose. The test's subject, passed records surviving
   `forget_blocked`, is unchanged.
6. **D5 predicate** is as the cut says: held and not `Passed`. A held `Unknown` modifier (resync-introduced, so Windows has it
   down) would still be marked by a completed chord. This is reachable only through a chord key that is pressed, released, then
   pressed again while the hook does not see it, all inside the chord window (RK7 class). It is left for 2.3; see Open points.
7. **Comment edits in `src/hook/*`** (M2), outside the cut's touch set. They are comment-only.
8. **Follow-up commit `15e901d`.** A keyboard auto-repeats only the key pressed last, and pressing another key ends that repeat
   for good. Three of this lane's scenarios repeated a key after another key went down; `15e901d` gives them a possible key
   order. The property generator keeps repeating any held key, on purpose: the engine must not depend on typematic order
   (injected input, several keyboards).
9. **CHANGELOG**: the KS-32 entry loses "partly", because A2 fixed the modifier half and this lane the rest. KS-30 gets its own
   entry.

## Unsafe / atomics added

None. No `unsafe`, atomic, lock, allocation, thread or Win32 call was added. The engine stays `#![forbid(unsafe_code)]`. On the
hook path the resync now reads one array slot per modifier before it calls `GetAsyncKeyState`, and skips the call for swallowed
modifiers.

## Gates

- `cargo fmt --check`: pass at every commit.
- `cargo check --all-targets --locked`: pass at HEAD.
- `cargo clippy --all-targets --locked -- -D warnings`: pass at every commit, including the red commits 2, 4 and 6.
- Tests run (lib): `engine::` (146 at HEAD), `hook::` and `readme`, 157 tests in total, all green at HEAD. At each red commit
  exactly the listed tests failed and every other `engine::` test passed. The full `cargo test --locked` run is left to the
  tester.
- Manual `#[ignore]` tests added: none. Nothing automatable needs a live hook.

## Manual checks (human only, release daemon, `osk.exe` visible, language hotkey Alt+Shift)

The cut's M1-M5 are unchanged. M6 has a new expected result, and M7-M9 are new for I1(b) and D3.

| # | Setup | Steps | Expected |
|---|---|---|---|
| M1 | default config, Notepad | LAlt, LShift, release LAlt, hold LShift 2 s, release; Esc; type `a` | Shift never lights in osk; layout unchanged; `a` lowercase; nothing stuck |
| M2 | default + `( keys: "Alt+Shift+X", action: ToggleCapsLock )` | Alt, Shift, X, release all; then Shift, Alt, X | CapsLock toggles once per round; no `x`; nothing stuck |
| M3 | `( keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600) )` | A, LShift, release A, hold LShift 1.5 s, release | CapsLock does not toggle; nothing stuck |
| M4 | `( keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000) )` | LCtrl, LShift, release LCtrl within 0.2 s, hold LShift to 0.8 s, release; type `a` | Shift never lights; layout unchanged; `a` lowercase |
| M5 | README Tap/Hold pair on CapsLock | hold CapsLock 2 s, release | CapsLock light unchanged; nothing stuck |
| M6 | `( keys: "CapsLock", action: SwitchLanguage, trigger: Tap(3000) )` | hold CapsLock, click another window, keep holding 1 s, release | CapsLock light unchanged (repeats swallowed, I1 b); the layout switches (Tap fires on release); nothing stuck |
| M7 | `( chord: (keys: ["J", "K"], window_ms: 50), action: Launch(exe: "notepad.exe") )` | press J+K together, hold both 2 s while Notepad opens and takes focus, release | no `j`/`k` typed into Notepad; nothing stuck |
| M8 | `( keys: "CapsLock", action: Launch(exe: "wt.exe"), trigger: Hold(300) )` | hold CapsLock 2 s while the terminal opens, release | CapsLock light unchanged in the terminal; nothing stuck |
| M9 | default config | `swch off`, hold LShift, `swch on`, type `a` (still holding), press LAlt, release LAlt, release LShift | `A` uppercase; osk shows Shift held until it is released; nothing stuck |

## Docs

- README: the behaviour-rules bullet has no KS-14 gap, gains the resync sentence and the chord/sequence clause, and states the
  focus-change rule (repeats stay swallowed, the release passes unless a Tap/Hold is pending). The `block_original_combo` row
  gains the 1.2 note.
- CHANGELOG, Unreleased/Fixed:
  - new KS-14, KS-30 and KS-32 entries;
  - the KS-06 entry is updated: focus-change repeats stay swallowed, a modifier pressed while remapping was off is covered, and
    the KS-14 gap sentence is removed.
- Engine docs: `mod.rs` module rules 1 and 5, the key-up paragraph, State and known defects; `keys.rs`, `env.rs`,
  `sequences.rs`, `chords.rs`, `testing.rs`; new module doc in `properties.rs`.
- `default_config.ron`: no change (no text on this behaviour).
- Book: not edited, left to the phase book sync. Pages: `book/src/config/known-issues.md` rows for KS-06, KS-14, KS-30 and
  KS-32; `triggers-conditions.md` section 11.8; `kinds.md:96` and section 9.3; `architecture.md:184`, `:208`, `:491-494`, `:524`;
  `glossary.md:287`, `:343`; `audit.md` status lines of KS-06, KS-14, KS-30 and KS-32 (all fixed at this lane's merge). Add the
  I1(b) focus-change repeat rule where `architecture.md` describes `foreground_changed`.

## Commits

- `6661c9d` Tests: scenario checks for stuck keys and per-press suppression symmetry
- `20d4844` Engine: red-first: a modifier whose key-down was swallowed survives the resync (KS-14)
- `8772b73` Engine: the modifier resync leaves keys whose key-down was swallowed alone (KS-14)
- `72ea8d6` Engine: red-first: a key whose key-down was swallowed keeps its repeats swallowed across a focus change (KS-06)
- `cc3892d` Engine: a key-down a focus change forgot still swallows its repeats (KS-06)
- `afdb60c` Engine: red-first: a key whose key-down reached Windows keeps its key-up (KS-06, KS-32)
- `12be46b` Engine: a key-down that reaches Windows always lets its key-up through (KS-06, KS-32, KS-30)
- `9dafc77` Engine: property tests of the suppression invariant over generated key scripts (KS-06, KS-14)
- `15e901d` Tests: suppression scenarios auto-repeat only the key pressed last

## Open points / pre-existing warnings

- No pre-existing warnings: clippy `-D warnings` is clean.
- RK7 class, for 2.3: a non-modifier held since before the hook saw it reads as a first key-down at its next repeat; if a binding
  swallows it, its key-up is swallowed while Windows has it down. A held `Unknown` modifier can still be marked by a chord
  completion, as Deviation 6 describes. Under I1(b), `Unknown` plus held means Windows has the key down, so 2.3 could treat every
  `Unknown` repeat as passed.
- The critique's I1(b) cost (the next press after a lost key-up and a focus change is swallowed once) is documented in
  `Engine::on_foreground_changed` and in the addendum. 2.3's reset repairs it.
- CLAUDE.md "Layout" should mention property tests in `engine::properties` (cut Q6, orchestrator after merge).
