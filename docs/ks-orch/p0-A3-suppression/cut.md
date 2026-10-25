# Cut: p0-A3-suppression — Workstream A "suppress repeats symmetrically; sync skips blocked keys" (KS-06 rest, KS-14; also KS-32 rest, KS-30)

Base: `master` @ `82f542f` (Merge branch 'fix/matching'). Branch: `fix/suppression-symmetry`. Work tree `D:/key-switch-rs/key-switch-rs`.

## Verdict
CUT READY

## Spec re-located

The A2 lane (`fix/matching`, merged at `82f542f`) already moved all decisions into `src/engine` and landed the core of KS-06
("a repeat follows its key-down"). What is left on master is KS-14 and four holes in the same invariant.

| Spec reference (book @7cb3521) | Current file:line | Status | Note |
|---|---|---|---|
| KS-06 #1 `was_repeat` from `active_keys.swap` (`keyboard_hook_callback.rs:191-195`) | `src/engine/keys.rs:88-93` (`press`), used `src/engine/mod.rs:309` | moved | unchanged semantics |
| KS-06 #2 deferred/sequence/chord arms `continue` on repeat (`:239,:244,:249,:256,:274`) | `src/engine/mod.rs:343-345`, `:355-357`, `:369-371`, `:385-387`, `:410-412` | moved | still `continue`; suppression of a repeat is now decided by the tail `mod.rs:431-450` |
| KS-06 #3 blocked bit set, never cleared by a passing key-down (`:294-297`) | `src/engine/mod.rs:451-461` (`mark_blocked` / `mark_passed`) | fixed on master (A2) for first key-downs | **hole H1**: a repeat with no record that passes records nothing (`mod.rs:446-450`) |
| KS-06 #4 key-up `take_blocked` (`:324`), gesture-bound key-up (`:314-315`) | `src/engine/mod.rs:465-490` | moved | **hole H2**: `mod.rs:481-485` suppresses a key-up whenever a Tap fired or a Hold was pending, even if the key's record is Passed |
| KS-06 #5 `foreground_changed` clears the bitmap (`:176-181`) | `src/engine/mod.rs:302-305` -> `keys.rs:140-146` (`clear_all_blocked`) | moved | policy kept (roadmap 2.3); see D1/D6 |
| KS-06 Immediate combo stops matching mid-hold; auto-blocker Shift after Alt released (audit `audit.md:253`) | pinned by `src/engine/scenarios.rs:1351-1379` and `:1381-1405` | partly fixed | `:1381-1405` pins the **KS-14 behaviour** (Shift's repeats pass after Alt's key-up); this lane flips it (T5) |
| KS-14 `sync_modifiers` overwrites suppressed modifiers (`:86-97`) | `src/engine/keys.rs:74-84` (unconditional store at `:82`) | moved, **defect present** | called per event `mod.rs:273` and on focus change `mod.rs:303` |
| KS-14 call per event (`:146`) | `src/engine/mod.rs:273` | moved | |
| KS-14 `was_repeat` shares the slot (`:192`) | `src/engine/keys.rs:88-93` | moved, defect present | same `active` array the resync overwrites |
| `MODIFIERS` (`:38-43`) | `src/engine/keys.rs:21-33` | moved | 11 codes |
| `GetAsyncKeyState` reader | `src/hook/keyboard_hook_callback.rs:116-122` (`HookEnv::os_key_down`), contract `src/engine/env.rs:32-40` | moved | unchanged by this lane |
| KS-32 sequence completion marks every last-step key (`sequence_state.rs:82-83`, `keyboard_hook_callback.rs:269`) | `src/engine/sequences.rs:82-88`, `:120-139` (`last_step_keys`), applied `src/engine/mod.rs:397-406` | partly fixed (A2: modifiers filtered) | **hole H3a**: a non-modifier last-step key whose key-down passed is re-marked blocked (`["F14+F15","F14+F16"]`) |
| KS-30 chord completion marks released keys (`keyboard_hook_callback.rs:285-288`) | `src/engine/chords.rs:94-100`, applied `src/engine/mod.rs:419-425` | consequence fixed on master (`mod.rs:460` `mark_passed` overwrites the stale mark) | **hole H3b**: a chord key whose repeat passed after a focus change is re-marked blocked -> stuck; stale marks on released keys remain |
| Roadmap A bullet (`roadmap.md:81`) | `book/src/roadmap.md:81` | unchanged | |
| Roadmap A acceptance "a repeat of a suppressed key is suppressed, and its key-up is suppressed exactly once" (`roadmap.md:98`) | `book/src/roadmap.md:98` | unchanged | partly covered by `scenarios.rs:1326-1349`, `:1351-1379` |
| README gap text | `README.md:355-367` (KS-14 gap bullet `:364-367`), `README.md:427` (`block_original_combo` row) | current | edited by this lane |
| CHANGELOG KS-06 entry naming the KS-14 gap | `CHANGELOG.md:87-104` (gap sentence `:102-104`), KS-32 partly `:119-121` | current | edited by this lane |

Verified mechanism on master (by reading, no build): `KeyState::sync_modifiers` writes `env.os_key_down(m)` into `active[m]` for
every modifier except the event key (`keys.rs:78-83`). Windows never updates the async state for a key-down a low-level hook
swallows (audit KS-14, two verifiers; Microsoft Learn: the hook "is called before the asynchronous state of the key is updated"),
so the next event of any other key clears a swallowed modifier, and that modifier's next auto-repeat reads as a fresh key-down.

## Goal and non-goals

**Goal (behaviour after the lane).**
- G1. The modifier resync never touches a key whose key-down the engine swallowed, including after a focus change: such a key
  stays held for matching and its auto-repeats stay auto-repeats (KS-14; spec item 3).
- G2. Without a focus change or a lost event, every press is decided as a whole: its key-down, every auto-repeat and its key-up are
  all suppressed or all passed, for every binding kind and trigger (spec item 1). Proven by a property test.
- G3. Any key-down that reaches Windows, first key-down or auto-repeat, makes the rest of that press reach Windows, key-up
  included; nothing (gesture resolution, chord or sequence completion) swallows that key-up (spec item 2). With focus changes and
  lost key-ups in the mix, no key is ever left down in Windows while it is physically up. Proven by a second property test.

**Non-goals (left to their items).**
- Focus-change policy (roadmap 2.3, KS-29/KS-31): a key whose swallowed key-down a focus change forgot keeps master's rules: its
  repeats follow the bindings, its key-up passes unless the press still has a Tap/Hold pending. Lost-key-up repair (Win+L, secure
  desktop, `swch off`/`on`) is 2.3's `reset_transient_state()`.
- KS-10 (roadmap 1.1): Immediate actions still fire on repeats, including passed ones.
- KS-05 rest / KS-12 (roadmap 1.2): the exact auto-blocker still swallows the second modifier; this lane makes that swallow
  consistent for the whole press.
- KS-15/KS-16 sequence matching (3.2), one winner per event (1.6), `block_default` for deferred triggers (1.3).
- No adapter, Win32, config-schema or RON change.

**Later items that depend on this one:** 1.1 (a non-repeating binding's repeat "is still suppressed when the original key-down was
suppressed": reads the per-key record), 1.2 (mask "after Alt or Win reached the OS": reads Passed), 1.3 (deferred `block_default`),
2.3 (stops converting Suppressed to Forgotten in an elevated daemon; its reset "resyncs all 256 keys, except blocked ones" = D2's
rule), 3.4 (`KeyState { physical, suppressed }`).

## Decisions

### D1: A fourth per-key record, `Forgotten`
**What.** `keys::LastDown` gains `Forgotten`: "the key-down was suppressed, then a foreground change forgot it". `on_foreground_changed`
turns every `Suppressed` into `Forgotten` (rename `clear_all_blocked` -> `forget_blocked`, `keys.rs:138-146`). For repeat and key-up
decisions `Forgotten` behaves exactly like today's `Unknown` (`is_blocked` false, `is_passed` false, `take_blocked` returns false);
it differs only in D2 (the resync leaves it alone) and D5 (completion may re-mark it).
**Why.** Spec item 3 says the resync must not overwrite the state of a key the engine itself suppressed nor reset its repeat
detection. A focus change does not make Windows see that key-down, so the OS view stays meaningless for it. Today the focus change
erases the record to `Unknown` and the next event's resync clears the key: `T10` (one long Shift press fires its `DoubleTap(600)`
across a focus change) stays red with a Suppressed-only skip.
**Alternatives rejected.** (A) Skip only `Suppressed`: fails T10, violates spec item 3 after any focus change (Alt+Shift+Tab with
the auto-blocker causes exactly that). (C) Keep `Suppressed` across focus changes: changes 2.3's policy and flips four pinned tests
(`scenarios.rs:172-182`, `:1407-1476`). (D) Detect a lost key-up by silence: typematic repeat stops when another key goes down, so
silence proves nothing.
**Cost.** A suppressed modifier whose key-up is lost (secure desktop, `swch off`) stays held in the engine until its next press and
release; today the next event repairs it. Risk RK1, question Q3.

### D2: The resync skips keys the OS cannot know about
**What.** `KeyState::sync_modifiers(env, skip)` skips `skip` (as today) and every modifier whose record is `Suppressed` or
`Forgotten`; for those it neither calls `env.os_key_down` nor writes `active`. `Passed` and `Unknown` modifiers and the generic
slots (`VK_SHIFT`, `VK_CONTROL`, `VK_MENU`) are resynced as today.
**Why.** The OS view is authoritative only for key-downs Windows saw. Keeping the per-event resync for `Passed`/`Unknown` keeps the
repair of a passed modifier whose key-up went elsewhere (`modifier_lost_by_the_os_is_resynced_before_matching`,
`scenarios.rs:156-170`). Skipping also saves up to 11 `GetAsyncKeyState` calls per event for swallowed modifiers.
**Alternatives rejected.** Remove the per-event resync and repair only on focus change (KS-14's "better" fix): loses the repair above
when no foreground event follows, and it is 3.4's restructuring. Resync generic slots conditionally: unnecessary, the sided slot
already carries the engine's knowledge (`keys.rs:154-165`).
**Cost.** One array read per modifier per event.

### D3: A repeat with no record that reaches Windows is recorded `Passed`
**What.** In the repeat branch of `on_key_down` (`mod.rs:446-450`): record `Suppressed` -> suppress; `Passed` -> pass; otherwise
(`Unknown`/`Forgotten`) keep what the bindings asked, and if that is "pass", `mark_passed(vk)`. A no-record repeat the bindings
swallow records nothing (master policy, `mod.rs:439-445`).
**Why.** Spec item 2 ("a key-down that passes clears that key's blocked state, so its key-up passes too") covers repeats: such a
repeat is the first key-down Windows sees. Unrecorded, the key-up can still be swallowed by H2/H3 (T7, T8) and later repeats can
flip to suppressed (T9).
**Alternatives rejected.** Also cancel the key's pending Tap/Hold when its repeat passes: changes gesture semantics after focus
changes (2.2/2.3 scope); the Tap may still fire on the key-up (T8 pins it).
**Cost.** None.

### D4: A key-up whose key-down reached Windows always passes
**What.** `on_key_up` (`mod.rs:465-490`) becomes: resolve Tap and Hold as today (fire / cancel / leave overdue); then
`suppress = was_blocked || (gesture_bound && !was_passed)`, where `was_passed = is_passed(vk)` is read before
`was_blocked = take_blocked(vk)` (which resets the record), and `gesture_bound = tap_fired || hold != HoldRelease::NotPending`
(today's predicate). Truth table: `Suppressed` -> suppress; `Passed` -> pass; `Unknown`/`Forgotten` -> suppress iff gesture-bound
(today's rule).
**Why.** Spec item 2. A pending Tap/Hold is only ever armed by a suppressed first key-down (`mod.rs:342-381` set `suppress`), so
for a key without focus change the record is already `Suppressed`; the override only matters after D1/D3, where it left the key
stuck (T8).
**Alternatives rejected.** Decide key-ups by the record alone (drop the gesture override): turns master's swallowed key-up of a
forgotten gesture press into an orphan key-up with no spec need. Keep master's rule: stuck key (T8).
**Cost.** None.

### D5: Completion marks only held keys whose key-down did not reach Windows
**What.** New `KeyState::mark_completed(vk: u16)`: if `vk` is in range, `active[vk]` is set, and the record is not `Passed`, set
`Suppressed`; otherwise do nothing. `mod.rs:403-405` (sequence `last_step_keys`) and `mod.rs:422-424` (chord `keys`) call it
instead of `mark_blocked`. The completing key itself is still marked by the tail (`mod.rs:451-452`).
**Why.** Spec items 1-2 "whatever the binding kind": KS-32's rest (T6) and the chord-after-focus-change stuck key (T7) both come
from re-marking a key whose key-down passed. Re-marking a `Forgotten` held key keeps the A2 policy pinned by
`chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` (`scenarios.rs:730-748`). The held-only guard removes
KS-30's stale mark on released keys, which also keeps the invariant "`Suppressed`/`Forgotten` implies held" that D2 relies on.
**Alternatives rejected.** Drop completion marks: flips the pinned chord test. Keep `mark_blocked`: T6/T7 stuck keys.
**Cost.** One extra `active` read per marked key.

### D6: Everything else stays
Matching (`matching.rs`), gestures (`gestures.rs`), the binding loop, the adapter, timers, locking and `Decision` are unchanged.
`on_foreground_changed` keeps its order (resync, then forget); with D2 the order no longer matters.

### D7: Tests: targeted scenarios, harness checks, in-house property generator
- Harness (`testing.rs`): `with_context`, `check_press_symmetry`, `check_nothing_stuck`, `expect_nothing_stuck` (signatures below).
- Targeted red-first scenarios T1-T10 (exact reds below).
- New `src/engine/properties.rs` (`#[cfg(test)]`): SplitMix64 PRNG, fixed seeds, six configs, two properties. No `proptest`: the
  roadmap item names no dependency (CLAUDE.md principle 8).

### D8: Docs
README (behaviour rules, `block_original_combo` row), CHANGELOG (Fixed), engine module docs, in the commits that change behaviour.
Book pages are left to the phase book sync (same as A2's D12); list under Docs impact. `default_config.ron` has no text about
repeats or the resync: no change.

### Type and signature sketch (no bodies)

```rust
// src/engine/keys.rs
#[derive(Clone, Copy, PartialEq, Eq)]
enum LastDown { Unknown, Passed, Suppressed, Forgotten }      // Forgotten: new (D1)

impl KeyState {
    pub(super) fn sync_modifiers(&mut self, env: &dyn Env, skip: Option<VIRTUAL_KEY>); // D2: also skips Suppressed/Forgotten
    pub(super) fn mark_completed(&mut self, vk: u16);           // D5, new
    pub(super) fn forget_blocked(&mut self);                     // D1, renamed from clear_all_blocked: Suppressed -> Forgotten
    // unchanged: new, press, release, mark_blocked, mark_passed, is_blocked, is_passed, take_blocked, is_active
}

// src/engine/testing.rs
impl Scenario {
    pub(crate) fn with_context(self, context: impl Into<String>) -> Self; // printed as "context: ..." under the failure message
    pub(crate) fn check_press_symmetry(self) -> Self; // from now on: every repeat and key-up of a press has its key-down's outcome
    pub(crate) fn check_nothing_stuck(self) -> Self;  // from now on, after every step: expect_nothing_stuck
    #[track_caller] pub(crate) fn expect_nothing_stuck(self) -> Self; // no key down in FakeEnv while the script holds it up
}
```
Failure texts (the red predictions depend on them):
- stuck: `"{label}: VK 0x{vk:02X} is down in Windows but physically up"` (label of the last step);
- symmetry: `"{label}: press of {key}: key-down {suppressed|passed}, {repeat|up} {suppressed|passed}"`;
- `fail` prints `scenario: {message}`, then `context: {context}` if set, then the transcript (existing substring expectations stay valid).
`check_press_symmetry` records the outcome at each `down`, compares at each `repeat`/`up`, clears at `up` and `up_unseen`; keys
pressed before it was enabled are not checked.

## Win32 API choices

No new Win32 call, feature or thread. The one API this lane reasons about:

| Purpose | `windows` path | Feature flag (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| OS key state for the resync (now skipped for swallowed keys) | `windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState` (`keyboard_hook_callback.rs:120`) | `Win32_UI_Input_KeyboardAndMouse` (present, `Cargo.toml:44`) | main thread (hook callback, WinEvent callback) | returns 0 = "up" on failure: "The current desktop is not the active desktop", UIPI, missing `DESKTOP_HOOKCONTROL` | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate |
| Why the OS state lags / never sees a swallowed key-down | `WH_KEYBOARD_LL` callback contract | n/a | main thread | "the callback function is called before the asynchronous state of the key is updated"; returning nonzero prevents passing the message on | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc |
| Repeat vs fresh press cannot be told apart in the hook | `KBDLLHOOKSTRUCT::flags` | present | main thread | the documented bits are extended, lower-IL-injected, injected, Alt context, transition: no previous-state bit | https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct |

`LLKHF_INJECTED`, the extended flag and `scanCode` do not matter here: the adapter's sentinel filter (`keyboard_hook_callback.rs:156`)
is unchanged, and other injectors' events (AltGr's fake LCtrl, NumLock's fake Shift up/down around numpad keys) are seen events with
their own per-key records.

**Threading.** Unchanged. The main thread installs `WH_KEYBOARD_LL` and the `EVENT_SYSTEM_FOREGROUND` WinEvent hook
(`core/app.rs:91` -> `hook/keyboard_hook.rs:108-159`) and pumps `GetMessageW` (`core/app.rs:127`). `Engine::on_key` and
`Engine::on_foreground_changed` run on it under `KeyboardHook::with_engine` (`keyboard_hook.rs:79-88`); `hold-timer` threads call only
`on_timer`/`forget_timer`, which never touch `KeyState`. No window, timer, COM apartment, lock, atomic or allocation is added on the
hook path. **Config/RON:** none; no compatibility impact.

## Touch set

| File | Change | Why |
|---|---|---|
| `src/engine/keys.rs` | `LastDown::Forgotten`; `sync_modifiers` skip rule; `mark_completed`; `clear_all_blocked` -> `forget_blocked`; module/field docs; unit tests (new + 2 renamed) | D1, D2, D5 |
| `src/engine/mod.rs` | repeat branch (D3), `on_key_up` (D4), completion calls (D5), `on_foreground_changed` calls `forget_blocked`; module docs: rule 1 (`:94-95`), rule 5 (`:117-124`), key-up paragraph (`:126-131`), "known defects" (`:133-136`, drop KS-14), State bullet (`:38-41`), `on_foreground_changed` doc (`:298-301`) | D1-D5 |
| `src/engine/env.rs` | `os_key_down` doc (`:32-40`): the engine never asks about a key whose key-down it swallowed | D2 contract |
| `src/engine/sequences.rs` | docs only: module doc `:12-16`, `Completed` doc `:43-48`, `last_step_keys` doc `:120-124` (key-ups swallowed only for keys whose key-down was not passed) | D5 |
| `src/engine/chords.rs` | docs only: module doc `:10-14`, `Completed` doc `:50-53` | D5 |
| `src/engine/testing.rs` | four harness methods, `context`/per-key press and stuck-check fields, `fail` prints context, module doc | D7 |
| `src/engine/scenarios.rs` | helper `with_alt_shift_blocker`; T1-T10; T5 replaces `:1381-1405`; harness self-tests; section header "Suppression symmetry (KS-06, KS-14)" | acceptance |
| `src/engine/properties.rs` (new) + `#[cfg(test)] mod properties;` in `mod.rs` (`:150-151` area) | property tests | acceptance 3 |
| `README.md` | `:355-367` behaviour rules; `:427` `block_original_combo` row | user-visible behaviour |
| `CHANGELOG.md` | Unreleased/Fixed: KS-14, KS-06 rest, KS-32 rest, KS-30; trim `:102-104`; fold `:119-121` | Keep a Changelog |
| `src/config/default_config.ron` | none (no text on this behaviour) | checked: `:1-143` |
| `src/hook/*` | none | adapter unchanged |
| `book/src/*` | none in this lane (phase book sync) | CLAUDE.md: doc-writer owns the book |

## Commits, in order

Gate per commit: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Commits 2 and 4
are the declared red commits: fmt and clippy pass, and exactly the listed tests fail with the predicted text; every other test
passes. If any other existing test turns red in any commit, stop and report; do not edit its expectations. Every message ends with a
blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`; red commits quote the failing line in the body.

**1. `Tests: scenario checks for stuck keys and per-press suppression symmetry`** (green)
- `testing.rs`: the four methods and failure texts above.
- `scenarios.rs` harness self-tests:
  - `press_symmetry_check_reports_a_key_up_that_differs_from_its_key_down`, `#[should_panic(expected = "key-down suppressed, up passed")]`:
    `(keys: "F13", action: SwitchLanguage)`; `check_press_symmetry()`; down F13; `foreground_changed()`; up F13 (passes by the focus-change policy).
  - `nothing_stuck_holds_for_a_plain_press_and_an_unseen_release`: `( bindings: [] )`; A down/up, then `Shift` down + `up_unseen`, `expect_nothing_stuck()`.
  - `context_is_printed_with_a_failure`, `#[should_panic(expected = "context: seed 7")]`: `with_context("seed 7")`, CapsLock binding, down CapsLock, `expect_passed()`.

**2. `Engine: red-first: a modifier whose key-down was swallowed survives the resync (KS-14)`** (RED)
Helper in `scenarios.rs`: `fn with_alt_shift_blocker(ron: &str) -> Vec<Binding>` = `vec![Binding::new_auto_blocker(KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT]))]`
followed by `from_ron_str(ron)` (the loader's order, `loader.rs:170-189`; no registry read). `DEFAULT_PAIR` =
`( bindings: [ (keys: "CapsLock", action: SwitchLanguage), (keys: "Shift+CapsLock", action: ToggleCapsLock) ] )`.

| Test | Script (all on `Scenario`) | PREDICTED RED on master (`82f542f`) |
|---|---|---|
| T1 `alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier` (KS-14 case 1) | bindings `with_alt_shift_blocker("( bindings: [ (keys: \"Alt+Shift+X\", action: ToggleCapsLock) ] )")`, `check_nothing_stuck()`. Run A: down LAlt passed; down LShift suppressed, fired [DoNothing]; down X suppressed, fired [ToggleCapsLock]; up X suppressed; up LShift suppressed; up LAlt passed. Run B (new Scenario): down LShift passed; down LAlt suppressed, fired [DoNothing]; down X suppressed, fired [ToggleCapsLock]; up X suppressed; up LAlt suppressed; up LShift passed | `down X at +0 ms: expected the event to be suppressed, it was passed` (run A) |
| T2 `shift_held_by_a_double_tap_still_counts_for_shift_capslock` (KS-14 case 2) | `( bindings: [ (keys: "Shift", action: SwitchLanguageBackward, trigger: DoubleTap(300)), (keys: "CapsLock", action: SwitchLanguage), (keys: "Shift+CapsLock", action: ToggleCapsLock) ] )`: down LShift suppressed, nothing fired; advance 50; down CapsLock suppressed, fired [ToggleCapsLock]; up CapsLock suppressed; up LShift suppressed; total [ToggleCapsLock]; nothing stuck | `down CapsLock at +50 ms: expected to fire [ToggleCapsLock], fired [SwitchLanguage]` |
| T3 `one_long_press_of_shift_does_not_fire_its_double_tap` (KS-14 case 3) | `(keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600))`: down A passed; advance 10; down LShift suppressed; advance 10; up A passed; advance 480; repeat LShift suppressed, nothing fired; advance 33; repeat LShift suppressed, nothing fired; up LShift suppressed; total []; nothing stuck | `repeat LShift at +500 ms: expected to fire [], fired [ToggleCapsLock]` |
| T4 `ctrl_shift_hold_with_ctrl_released_first_neither_leaks_nor_sticks_shift` (README/CHANGELOG gap) | `(keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000))`, `check_nothing_stuck()`: down LCtrl passed; advance 20; down LShift suppressed; advance 100; up LCtrl passed; advance 380; repeat LShift suppressed; advance 33; repeat LShift suppressed; advance 67; up LShift suppressed, nothing fired; advance 1000 nothing fired; total [] | `repeat LShift at +500 ms: expected the event to be suppressed, it was passed` |
| T5 `shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press` (acceptance 1 and 2; **replaces** `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck`, `scenarios.rs:1381-1405`) | `with_alt_shift_blocker(DEFAULT_PAIR)`, `check_nothing_stuck()`: down LAlt passed; down LShift suppressed, fired [DoNothing]; up LAlt passed; advance 500; repeat LShift suppressed, nothing fired; advance 33; repeat LShift suppressed, nothing fired; up LShift suppressed; advance 100; down LShift passed; up LShift passed; total [DoNothing] | `repeat LShift at +500 ms: expected the event to be suppressed, it was passed` |
| T10 `suppressed_modifier_stays_an_auto_repeat_across_a_focus_change` (D1) | `(keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600))`, `check_nothing_stuck()`: down A; down LShift suppressed; `foreground_changed()`; up A passed; advance 500; repeat LShift nothing fired; up LShift; total []. No suppression expectation on the repeat or key-up (focus-change policy, 2.3) | `repeat LShift at +500 ms: expected to fire [], fired [ToggleCapsLock]` |
| `keys::tests::resync_leaves_a_modifier_whose_key_down_was_suppressed` | `press(VK_LSHIFT)`; `mark_blocked(VK_LSHIFT.0)`; `sync_modifiers(&NothingDown, Some(VK_A))`; `assert!(state.is_active(VK_LSHIFT), "a modifier whose key-down was swallowed stays held through the resync")`; `assert!(state.press(VK_LSHIFT), "its next key-down is still an auto-repeat")`. `NothingDown`: a test `Env` reporting every key up | panic `a modifier whose key-down was swallowed stays held through the resync` |

**3. `Engine: the modifier resync leaves keys whose key-down was swallowed alone (KS-14)`** (green; makes 2 green)
- `keys.rs`: D1 (`Forgotten`, `forget_blocked`), D2. `mod.rs:304` calls `forget_blocked`. Docs in `keys.rs`, `mod.rs` rule 1,
  State bullet, `on_foreground_changed` doc, "known defects" (remove KS-14), `env.rs`.
- Unit tests: `resync_still_repairs_a_modifier_whose_key_down_passed` (press LCtrl, `mark_passed`, sync with `NothingDown`, not
  active); `forgotten_modifier_is_left_alone_by_the_resync` (press LShift, `mark_blocked`, `forget_blocked`, sync: still active;
  `is_blocked`/`is_passed` false; `take_blocked` false); rename `clear_all_resets_every_flag` / `clear_all_keeps_passed_key_downs`
  to `forget_*` with the same assertions.
- README: `:364-367` bullet removed, "Two gaps remain" -> "One gap remains"; add to the rule: "The resync of modifiers from the
  Windows key state leaves alone a modifier whose key-down was swallowed, because Windows never saw it go down: it still counts as
  held for matching, and its auto-repeats stay auto-repeats (KS-14)." `:427` row: append "Until roadmap item 1.2 the blocker
  swallows the hotkey's second modifier (Shift pressed after Alt) for that key's whole press, auto-repeats and release included,
  even after Alt is released; it still counts as held, so `Alt+Shift+X` bindings fire."
- CHANGELOG Fixed: new KS-14 entry (cases T1-T4, T10; the Shift-after-Alt behaviour change: it used to reach Windows from its
  first auto-repeat on). Delete `:102-104` ("A modifier whose `Tap` or `Hold` ... KS-14").
- Invariant kept: no new lock/alloc on the hook path; `Passed`/`Unknown` repair unchanged.

**4. `Engine: red-first: a key whose key-down reached Windows keeps its key-up (KS-06, KS-32)`** (RED; predicted texts are the same
on commit 3 and on master, because commit 3 does not touch these paths)

| Test | Script | PREDICTED RED |
|---|---|---|
| T6 `sequence_completion_keeps_the_key_up_of_a_key_whose_down_passed` (KS-32 rest) | `( bindings: [ (sequence: (steps: ["F14+F15", "F14+F16"]), action: SwitchLanguage) ] )`, `check_nothing_stuck()`: down F14 passed; down F15 suppressed; up F15 suppressed; down F16 suppressed, fired [SwitchLanguage]; up F16 suppressed; up F14 passed | `up F14 at +0 ms: VK 0x7D is down in Windows but physically up` |
| T7 `chord_completed_after_a_focus_change_keeps_the_key_up_of_a_key_whose_repeat_passed` (D3+D5) | `(chord: (keys: ["J", "K"], window_ms: 2000), action: SwitchLanguage)`, `check_nothing_stuck()`: down J suppressed; `foreground_changed()`; advance 500; repeat J passed; advance 100; down K suppressed, fired [SwitchLanguage]; up K suppressed; up J passed | `up J at +600 ms: VK 0x4A is down in Windows but physically up` |
| T8 `deferred_key_whose_repeat_passed_after_a_focus_change_keeps_its_key_up` (D3+D4) | `check_nothing_stuck()`. Run 1 `(keys: "CapsLock", action: SwitchLanguage, trigger: Tap(2000))`: down CapsLock suppressed; `foreground_changed()`; advance 500; repeat CapsLock passed; advance 100; up CapsLock passed, fired [SwitchLanguage]. Run 2 `(keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(2000))`: same script; up passed, nothing fired; advance 2000 nothing fired | `up CapsLock at +600 ms: VK 0x14 is down in Windows but physically up` (run 1) |
| T9 `repeat_that_reaches_windows_after_a_focus_change_makes_the_rest_of_the_press_pass` (D3) | `(keys: "Ctrl+F13", action: SwitchLanguage)`, `check_nothing_stuck()`: down LCtrl passed; down F13 suppressed, fired [SwitchLanguage]; `foreground_changed()`; up LCtrl passed; advance 500; repeat F13 passed; advance 33; down LCtrl passed; repeat F13 passed (no fire expectation: KS-10); up F13 passed; up LCtrl passed | `repeat F13 at +533 ms: expected the event to be passed, it was suppressed` |

**5. `Engine: a key-down that reaches Windows always lets its key-up through (KS-06, KS-32, KS-30)`** (green; makes 4 green)
- `mod.rs`: D3 (repeat branch), D4 (`on_key_up`), D5 (`mark_completed` at `:404`, `:423`); module docs rule 5 and key-up paragraph.
- `keys.rs`: `mark_completed` + unit test `completion_marks_only_held_keys_whose_key_down_did_not_pass` (Passed held key unchanged;
  `Forgotten` held key -> blocked; released key not marked; code 300 ignored).
- `sequences.rs`, `chords.rs`: doc comments.
- README `:360-363` focus-change bullet: "Its repeats are then swallowed only while an `Immediate` binding matches them; once one
  reaches Windows, the rest of the press does too, its release included. Otherwise its release reaches Windows unless a `Tap` or
  `Hold` of the press is still pending (roadmap item 2.3)."
- CHANGELOG Fixed: KS-06 (rest: after a foreground change, a repeat that reaches Windows makes the rest of the press reach it,
  even with a pending `Tap`/`Hold` or a `chord` completed on another key; the release used to be swallowed and the key left held);
  KS-32 (rest; unverified report; fold `:119-121` into one entry: `["F14+F15", "F14+F16"]` with F14 pressed first no longer leaves
  F14 held); KS-30 (unverified report: a completed `chord` no longer marks a key already released).

**6. `Engine: property tests of the suppression invariant over generated key scripts (KS-06, KS-14)`** (green at commit 5)
`src/engine/properties.rs`, registered as `#[cfg(test)] mod properties;`:
- `struct SplitMix64(u64)` with `next_u64` (standard SplitMix64 constants `0x9E3779B97F4A7C15`, `0xBF58476D1CE4E5B9`,
  `0x94D049BB133111EB`, shifts 30/27/31) and `below(n)`. Seed per run = `seed.wrapping_mul(0x100) + config_index`.
- `const SEEDS: u64 = 64; const STEPS: usize = 250;`
  `KEYS = ["LShift", "RShift", "LCtrl", "LAlt", "CapsLock", "F13", "F14", "F15", "F16", "J", "K", "Space", "A"]`;
  `DELAYS_MS = [0, 1, 20, 33, 49, 50, 199, 200, 201, 250, 500, 600, 1000]`.
- Configs, in this order (names go into `with_context`):
  1. `default`: `with_alt_shift_blocker` of a RON list with the two `DEFAULT_PAIR` bindings plus
     `(keys: "Alt+Shift+K", action: ToggleCapsLock)`;
  2. `immediate`: `"Ctrl+Alt+J"` ToggleCapsLock; `"F13"` PressKey("Left"); `"F14"` ToggleCapsLock `block_default: false`;
     `"CapsLock"` SwitchLanguage `allow_extra_modifiers: true`; `"Shift+Space"` SwitchLanguage;
  3. `gestures`: CapsLock `Tap(200)` SwitchLanguage + CapsLock `Hold(200)` PressKey("LShift"); `"LShift"` `DoubleTap(600)`
     ToggleCapsLock; `"Ctrl+Shift"` `Hold(1000)` SwitchLanguage; `"RShift"` `Tap(300)` SwitchLanguage;
  4. `chords`: `["Shift","Space"]` 500 ms; `["J","K"]` 50 ms; `["F13","F14"]` 2000 ms;
  5. `sequences`: `["F13","F14"]`; `["F14+F15","F14+F16"]`; `["Ctrl+J","Ctrl+K"]`;
  6. `mixed`: blocker + `DEFAULT_PAIR` + `"LShift"` `DoubleTap(600)` + `"Ctrl+Shift"` `Hold(1000)` + chord `["J","K"]` 50 ms +
     sequences `["F13","F14"]` and `["F14+F15","F14+F16"]`.
- Step generator (`roll = below(100)`): `0..35` down a random key that is up (none up -> repeat); `35..60` repeat a random held key
  (none -> down); `60..85` up a random held key (none -> down), and in focus mode one in ten of these (`below(10) == 0`) is
  `up_unseen`; `85..95` `advance_ms(DELAYS_MS[..])`; `95..97` `foreground_changed()` in focus mode, else `advance_ms`; `97..100`
  `advance_ms_stalled(DELAYS_MS[..])`. After STEPS: `up` every held key in `KEYS` order, `advance_ms(2000)`, `expect_nothing_stuck()`.
- `every_press_is_suppressed_or_passed_as_a_whole` (strict: no focus change, no `up_unseen`): `check_press_symmetry()` +
  `check_nothing_stuck()`. PREDICTED RED on master (tester replays): config `default`, a modifier (`LShift`, `RShift` or `LAlt`):
  `press of <key>: key-down suppressed, repeat passed`.
- `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups` (focus mode): `check_nothing_stuck()` only. PREDICTED RED
  on master: config `gestures` (KS-14 + H2 via `"Ctrl+Shift"` `Hold`, or H2 via the CapsLock pair) with
  `VK 0xA0|0xA1|0x14 is down in Windows but physically up`, or config `chords`/`sequences` (`0x4A`, `0x7D`).
- Budget: the module must finish under 10 s in debug `cargo test --locked`; otherwise halve `SEEDS` and record it in `impl.md`.
  If either property is green when replayed on master, record it in `impl.md`; do not tune the generator toward the fix (T1-T10
  carry the red-first).

## Acceptance mapping

| Acceptance line | Covered by |
|---|---|
| Roadmap A (`roadmap.md:98`) / brief 1: "a repeat of a suppressed key is suppressed, and its key-up is suppressed exactly once" | T5 (two repeats suppressed, key-up suppressed, next press passes both edges), T4, T3; existing `repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired` (`scenarios.rs:1326`), `immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once` (`:1351`); `every_press_is_suppressed_or_passed_as_a_whole`; `completion_marks_only_held_keys_whose_key_down_did_not_pass` (no stale mark); manual M1, M5 |
| Brief 2: KS-06 default-config route (Alt+Shift blocker, Alt released, Shift held) leaves nothing stuck | T5 with `check_nothing_stuck`; manual M1 |
| Brief 3: property-style test over generated down/repeat/up sequences | `every_press_is_suppressed_or_passed_as_a_whole`, `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups` |
| Spec 1 invariant, every kind/trigger | strict property over six configs; T6 (sequence), T2/T3/T10 (DoubleTap), T4 (Hold), T1/T5 (Immediate blocker) |
| Spec 2 "a passing key-down clears the blocked state" | T6, T7, T8, T9; existing `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (`:1247`) |
| Spec 3 resync must not overwrite / reset repeat detection | `resync_leaves_a_modifier_whose_key_down_was_suppressed`, `forgotten_modifier_is_left_alone_by_the_resync`, T1, T2, T3, T4, T10 |
| Roadmap A "makes `sync_modifiers` skip blocked keys" (`roadmap.md:81`) | same as spec 3; `resync_still_repairs_a_modifier_whose_key_down_passed` guards the repair |
| Checkpoint 1 smoke 2 (`roadmap.md:230`, "nothing is stuck"; "one switch" needs 1.1) | manual M5 |

## Manual checks (human only)

No automatable test needs a live hook; no `#[ignore]` stubs are added (as in A2). All with the release daemon (`swch open`),
`osk.exe` visible to watch modifier state, Windows language hotkey Alt+Shift (registry `Toggle` "1" or absent; read-only check).

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | default config, Notepad focused | press LAlt, press LShift, release LAlt, hold LShift 2 s, release; Esc (menu bar from the lone Alt is KS-12, 1.2); type `a` | Shift never lights in osk.exe; layout unchanged; `a` is lowercase; nothing stuck | none (T5 is the automated twin) |
| M2 | default config + `( keys: "Alt+Shift+X", action: ToggleCapsLock )` | Alt, Shift, X; release all. Then Shift, Alt, X | CapsLock toggles once per round; no `x` typed; nothing stuck | none (T1) |
| M3 | `( keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600) )`, repeat delay at the default (setting 1, about 500 ms) | press A, press LShift, release A, hold LShift 1.5 s, release | CapsLock does not toggle; nothing stuck | none (T3) |
| M4 | `( keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000) )` | press LCtrl, press LShift, release LCtrl within 0.2 s, hold LShift until 0.8 s, release; type `a` | Shift never lights in osk.exe; layout unchanged; `a` lowercase | none (T4) |
| M5 | README Tap/Hold pair on CapsLock | hold CapsLock 2 s, release | CapsLock light unchanged after release; nothing stuck in osk.exe | none |
| M6 | `( keys: "CapsLock", action: SwitchLanguage, trigger: Tap(3000) )` | hold CapsLock, click another window with the mouse while holding, keep holding 1 s, release | nothing stuck in osk.exe (CapsLock may toggle and the layout may switch: the documented focus-change gap) | none (T8) |

## Risks and edge cases

| Id | Risk | Handling |
|---|---|---|
| RK1 | D1/D2: a swallowed modifier whose key-up is lost (secure desktop, Win+L, `swch off` while holding it) stays held in the engine; exact matching then rejects bindings without it until the user presses and releases it (that press is swallowed as a "repeat"). Master repaired it at the next event | accepted (Q3); owner of the repair is 2.3 (`reset_transient_state` on unlock / desktop switch / off-on; KS-29, KS-31) |
| RK2 | Shift after Alt with the default blocker now stays swallowed for its whole press: Alt+Shift, release Alt, keep Shift past the repeat delay, type -> lowercase (master: Shift reached Windows from its first repeat) | spec-mandated; README `:427` + CHANGELOG KS-14 say it; 1.2 removes the swallow (Q5) |
| RK3 | After a focus change a no-record repeat that passes is now `Passed`: later repeats pass even when a binding matches again, and an `Immediate` action still fires on them (KS-10) | consistent with master's `Passed` keys (`scenarios.rs:1247-1272`); 1.1 |
| RK4 | T8: after a focus change a pending Tap still fires on a key-up that now passes (Windows also saw the key) | pinned and documented; 2.2/2.3 gesture semantics |
| RK5 | Re-entry: a refused key-up (`with_engine` -> `None`, `keyboard_hook_callback.rs:47-60`) still leaves the record | unchanged, documented in the adapter; no re-entry exists |
| RK6 | Non-elevated debug daemon with an elevated foreground: `GetAsyncKeyState` returns 0 (UIPI, doc link above), so the resync clears every `Passed` modifier on each event | pre-existing; swallowed modifiers are now protected; release daemon is elevated |
| RK7 | A key held since before the daemon started (or across `swch on`) reads as a first key-down at its next repeat; if a binding swallows it, its key-up is swallowed while Windows has it down | pre-existing; follow-up for 2.3 (resync of all keys at start/on) |
| RK8 | Sided/unsided: records are per sided code; generic slots are resynced from the OS only; chords mark sided event codes, sequences non-modifier codes | D2/D5; R8 below |
| RK9 | Reload mid-press: records survive a reload, so a press swallowed under the old bindings stays swallowed to its key-up | intended (the invariant is per press) |
| RK10 | Property test runtime in debug | budget rule in commit 6 |

**R1-R10 (code-reviewer areas).** R1: no adapter change; new indexing uses `slot()` (`keys.rs:36-39`) or `MODIFIERS` (< 256); every
event still ends in exactly one outcome. R2: no `SendInput` path. R3: `KeyState` stays inside `Mutex<Engine>`; no static, `Cell` or
new lock. R4: no atomics. R5: harness methods consume `self`; no `Binding` change. R6: `LastDown` is private and has no `Display`;
no public variant added. R7: `Scenario::with_bindings` sorts (`testing.rs:131-143`), blocker first like the loader. R8: see RK8.
R9: tests build the blocker directly; no registry access. R10: message loop and shutdown untouched.

## Docs impact

- **This lane (developer):** `README.md:355-367` and `:427` (commits 3, 5); `CHANGELOG.md` Fixed KS-14, KS-06 rest, KS-32 rest,
  KS-30 (commits 3, 5); engine doc comments. `default_config.ron`: none.
- **Phase book sync (doc-writer), not this lane:** `book/src/config/known-issues.md` rows 9 (KS-06), 20 (KS-32), 21 (KS-30),
  27 (KS-14); `book/src/config/triggers-conditions.md` §11.8; `book/src/config/kinds.md:96` (§8.4) and §9.3;
  `book/src/architecture.md:184`, `:208`, `:491-494`, `:524`; `book/src/glossary.md:287`, `:343`; `book/src/audit.md` status lines of
  KS-06, KS-14, KS-30, KS-32.

## Questions for the orchestrator

1. Scope: commit 5 also closes the rest of KS-32 and KS-30 (roadmap 3.2). Recommendation: keep. Both break the invariant this lane
   owns ("whatever the binding kind"), the change is one guarded method, and the property test would otherwise have to exclude
   multi-key sequence steps and chords across focus changes.
2. Focus-change policy stays as A2 set it (forgotten key: repeats follow the bindings, key-up passes unless gesture-bound).
   Recommendation: keep; roadmap 2.3 owns it, and four pinned scenarios depend on it.
3. D1 trade-off (RK1): protect a swallowed modifier from the resync even after a focus change, at the cost of no repair when its
   key-up is lost. Recommendation: accept; the spec forbids the resync from overwriting it, and 2.3's reset is the planned repair.
4. Book pages are left to the phase book sync (list above). Recommendation: yes, same as A2 (D12).
5. CHANGELOG section for RK2 (Shift after the blocker stays swallowed for its whole press): Recommendation: "Fixed" under KS-14, not
   "Changed / Breaking": it is the audit's prescribed fix and master's behaviour depended on the repeat delay.
6. `CLAUDE.md` "Layout" mentions scenario tests in `engine::scenarios` only. Recommendation: the orchestrator adds
   "property tests in `engine::properties`" after merge (agents do not edit CLAUDE.md).

## Readiness checklist

Spec
- [x] KS-06, KS-14 read (`audit.md:237-276`, `:389-401`); KS-32, KS-30, KS-29 read (`audit.md:592-622`); every KS-14 scenario is T1/T2/T3, KS-06 routes are T4/T5, KS-32 is T6
- [x] Every book reference re-located (table above); KS-06 core marked fixed on master
- [x] Goal, non-goals, dependent items (1.1, 1.2, 1.3, 2.3, 3.4) stated
Design
- [x] Every decision has rationale and rejected alternatives (D1-D5)
- [x] Owner decisions respected: exact modifiers untouched, repeat defaults untouched (KS-10 left to 1.1), RON unchanged, no logging change, licence n/a
- [x] Thread named for every path (main thread; timers untouched); nothing heavy added to the LL callback
- [x] N/A install/uninstall/timers: none added
- [x] N/A `SendInput`: none added
- [x] Sided/unsided handled per sided slot in combo, chord, sequence (RK8)
- [x] Reload, `swch off`/`on`, foreground change, shutdown covered (RK1, RK9, D1/D6, R10)
- [x] No new shared state, no atomics, no `static mut`
- [x] No new module cycle; all logic in `engine`, reached by `Engine::on_key` scenario tests
Win32
- [x] Each API: path, feature flag vs `Cargo.toml`, thread, errors, doc link
- [x] Elevation/UIPI considered (RK6); pipe DACL and token inheritance N/A (no IPC or process change)
Commits and tests
- [x] Red-first with exact predicted text for every fix (commits 2, 4)
- [x] Tests run through `Engine::on_key`, `FakeEnv`, fake clock; nothing installs a hook or injects input
- [x] N/A `#[ignore]` tests: nothing automatable needs a live hook; M1-M6 are human checks
- [x] Every commit passes fmt, clippy, test except the declared red commits 2 and 4
- [x] Bisectable order; README/CHANGELOG ride with the commits that change behaviour (3, 5)
Acceptance
- [x] Every acceptance line mapped
- [x] Every question has a recommendation

## Developer addendum: critique round 1 (developer, 2026-10-15, branch `fix/suppression-symmetry`)

Every remark of `critique.md` ends in one state below. Commits are on `fix/suppression-symmetry` (base `82f542f`).

### I1 (Important): RESOLVED in code, option (b)

- **Decision.** Brief spec item 1 says "whatever the binding kind or trigger"; option (a) needs the orchestrator to accept the
  narrowing explicitly, and no such acceptance is on record. Option (b) implements the spec as written and moves in roadmap 2.3's
  direction (`roadmap.md:298`, `:300`, acceptance "Holding `Win+Alt+T` while the foreground window changes keeps the T repeats
  suppressed").
- **Change.** `KeyState::is_blocked` counts `Forgotten` as well as `Suppressed` (`src/engine/keys.rs`, `is_blocked` ->
  `key_down_swallowed`). A repeat whose swallowed key-down a focus change forgot is therefore suppressed whatever the bindings ask,
  and the record stays `Forgotten`. The key-up rule is unchanged: `take_blocked` is true only for `Suppressed`, so a forgotten
  press's key-up passes unless a Tap or Hold is pending. Red-first `72ea8d6`, fix `cc3892d`.
- **D1 text, amended.** For repeats `Forgotten` behaves like `Suppressed`; for key-ups like `Unknown`; the resync skips it (D2);
  completion re-marks it (D5).
- **T7-T9 of the cut are replaced** (their routes assert leaked repeats, which (b) forbids):
  - `chord_keys_held_across_a_focus_change_keep_their_repeats_suppressed` (the critique's `jjj` example; since `15e901d` it
    repeats K, the key pressed last; red on master and on `8772b73`: "repeat K at +500 ms: expected the event to be suppressed, it
    was passed");
  - `deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (CapsLock `Hold(300)` -> `Launch`, then a `Tap(2000)`
    run whose key-up stays swallowed; red: "repeat CapsLock at +500 ms: expected the event to be suppressed, it was passed");
  - `immediate_combo_that_stops_matching_after_a_focus_change_keeps_its_key_suppressed` (`Ctrl+F13`, Ctrl released after the
    focus change; red: "repeat F13 at +500 ms: expected the event to be suppressed, it was passed");
  - `keys::tests::forget_blocked_keeps_passed_key_downs`: its `!is_blocked(20)` line pinned A2's reading and now asserts
    `is_blocked(20)` (red: "a forgotten key-down was still swallowed, so its repeats are too").
- **D3, D4, D5 under (b)** need other routes, because a forgotten press no longer passes a repeat:
  - D5 sequence: T6 unchanged (`sequence_completion_keeps_the_key_up_of_a_key_whose_down_passed`).
  - D5 chord: `chord_completion_keeps_the_key_up_of_a_chord_key_whose_down_passed`. J starts the chord, a title change (no
    foreground event) makes the chord's `when:` false, J is pressed again and passes, the title changes back and K completes the
    chord. Red: "up J at +0 ms: VK 0x4A is down in Windows but physically up".
  - D3: `modifier_held_before_the_engine_saw_it_keeps_passing_once_a_repeat_passed`, on a new harness step `down_unseen`.
    Ctrl, Alt, A and Shift are held from before the hook saw them. A's key-up makes the resync find Shift held with no record,
    and Ctrl's key-up makes the blocker match Shift's repeat. Red: "repeat LShift at +533 ms: expected the event to be passed, it
    was suppressed".
  - D4: no event sequence reaches "record `Passed` and a Tap/Hold pending" any more. A Tap/Hold is armed only under a suppressed
    first key-down. Until that key's next seen key-up, `active` stays set: the resync skips `Suppressed`/`Forgotten`, and the
    key-up is the only other writer. The record can then only become `Forgotten` (`forget_blocked`) or `Suppressed`
    (`mark_completed`). `mark_passed` needs a first key-down or an `Unknown` repeat. D4 is implemented as specified (roadmap 1.3,
    a non-blocking deferred trigger, will reach it) and pinned by the unit test
    `tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending`, which sets the record directly. Red: "a key-up whose
    key-down reached Windows must reach it too".
- **Pinned scenarios:** no expectation changed. `:173`, `:731`, `:1408`, `:1433`, `:1456`, `:1479` and `:1497` pass at every commit.
  The comments of `:731`, `:1408` and `:1479` were reworded, because the record now decides the repeat, not the binding.
- **Cost (accepted, as stated in the critique's option b).** After a lost key-up and a focus change, the key's next physical
  press reads as an auto-repeat and is swallowed once. Its key-up then passes as a lone key-up and clears the state. Example:
  `Ctrl+F13` with F13 swallowed, Win+L while F13 is held, then a plain F13 press is eaten instead of typed.

### M1 (Minor): RESOLVED

`nothing_stuck_holds_for_a_plain_press_and_an_unseen_release` calls `check_nothing_stuck()` in commit 1 (`6661c9d`).
`cargo clippy --all-targets --locked -- -D warnings` passes at every commit of the lane.

### M2 (Minor): RESOLVED

Comment-only edits in commit `8772b73`:
- `src/hook/keyboard_hook_callback.rs`, `foreground_changed` doc;
- `src/hook/keyboard_hook.rs`, the `SetWinEventHook` comment in `install`;
- `src/engine/mod.rs`, the resync comment in `on_key` and the `on_foreground_changed` doc (which now names RK1 and 2.3);
- `src/engine/keys.rs`, the `MODIFIERS` doc.

### M3 (Minor): RESOLVED

The repeat branch of the checker has its own failing self-test,
`press_symmetry_check_reports_a_repeat_that_differs_from_its_key_down`, which should_panic with
"press of LShift: key-down passed, repeat suppressed". It uses the D3 policy route (a press the hook never saw begin: a repeat
with no record that a binding matches is swallowed). Evidence that the strict property is not vacuous for repeats:
- the mutation "resync without the D2 skip" fails it with "repeat LAlt at +3079 ms: press of LAlt: key-down suppressed, repeat
  passed" (config default, seed 0);
- the same text appears when it is replayed on the master engine (impl.md).

### M4 (Minor): RESOLVED (text below replaces RK1 and Q3)

- **RK1 (corrected).** A key whose key-down was swallowed stays held for the engine when its key-up is lost (secure desktop,
  Win+L, `swch off` while holding it). This holds with or without a focus change: a `Suppressed` or `Forgotten` modifier is not
  resynced (D2), and a non-modifier never is (KS-29). Under I1(b), its next physical press reads as an auto-repeat and is
  swallowed whatever its record. The release of that press clears the state; it passes for `Forgotten` and is swallowed for
  `Suppressed`, and in both cases Windows never had the key down. While a swallowed Shift is phantom-held, the default config's
  CapsLock matches `Shift+CapsLock` (held families {Shift}) and fires `ToggleCapsLock` instead of `SwitchLanguage`, until Shift
  is pressed and released. Master repaired a modifier at the next event. The repair belongs to 2.3 (`reset_transient_state` on
  unlock, desktop switch and off/on).
- **Q3 (corrected).** Accept. Spec item 3 forbids the resync from overwriting a key the engine suppressed, and the cost is the
  corrected RK1 above.

### Open questions of the critique (not remarks; status for the orchestrator)

1. **Book pages.** Not edited, as in A2 (D12). The brief lists CHANGELOG, README and `default_config.ron`; the
   last needed no change. The pages listed under "Docs impact" go to the phase book sync. If this lane does not run inside a
   `ks-phase` with the book sync on, the orchestrator schedules one.
2. **Generic codes 0x10-0x12.** No conflict. The skip is keyed on the record, and a generic slot gets a record only if the engine
   itself decided an event reported with that code. If the engine swallowed such an event, Windows did not see it, so skipping its
   resync is right whatever the hook reports for injected input.
3. **Q5 (RK2 under Fixed).** Implemented under Fixed / KS-14. This is the orchestrator's call.
