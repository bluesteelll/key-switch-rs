# Context pack r1: p0-A1-engine (tester + reviewer)

CONTEXT PACK READY. Facts only, `file:line` at branch HEAD unless prefixed `M:` (= `master` `73bbf40`).
Read by: project-analyst (scout), read-only. Inputs: `D:/tmp/ks-orch/p0-A1-engine/cut.md`, `impl.md` (also present: `critique.md`, not summarised here).

## Scope and delta

- Branch `refactor/engine`, HEAD `d3131fc`, tree clean. `master` = `73bbf40`; `git merge-base master refactor/engine` = `73bbf40` (master merged in by `3d785dc`), so `master..` and `master...` give the same diff.
- `git log --oneline master..refactor/engine` (newest first):
  ```
  d3131fc Hook: adapter docs link no private items
  0873abf Hook: remove tap_state, chord_state and sequence_state
  941f0a7 Hook: the callback becomes an adapter over the engine
  9e9057e Hook: red-first: the adapter routes physical events through the engine   (declared compile red)
  c72d85f Engine: forbid unsafe code
  255c9a0 Engine: forget a Hold whose timer cannot be started (KS-33)
  a50e8d7 Engine: a Hold released after its term fires from its timer (KS-25)
  eef8629 Engine: red-first: a Hold released after its term must still fire       (declared assertion red)
  3d785dc Merge branch 'master' into refactor/engine                               (only .claude/**, CLAUDE.md, docs/WORKFLOW.md, .gitattributes)
  1d12c7b Engine: scenario test harness (WIP, lane interrupted)                    (earlier lane run)
  e32169c Engine: pure decision core with Env, Decision and timer tokens           (earlier lane run)
  677ee50 Data: conditions evaluate against a ForegroundInfo trait                 (earlier lane run)
  ```
- `git diff --stat master..refactor/engine`:
  ```
  CHANGELOG.md                       |  22 ++
  README.md                          |  57 +--
  src/config/mod.rs                  |   5 +
  src/data/binding.rs                |   7 +-
  src/data/condition.rs              |  71 +++-
  src/data/trigger.rs                |   2 +-
  src/engine/chords.rs               | 205 +++++++++++
  src/engine/env.rs                  |  63 ++++
  src/engine/gestures.rs             | 542 +++++++++++++++++++++++++++
  src/engine/keys.rs                 | 284 +++++++++++++++
  src/engine/mod.rs                  | 441 ++++++++++++++++++++++
  src/engine/scenarios.rs            | 725 +++++++++++++++++++++++++++++++++++++
  src/engine/sequences.rs            | 230 ++++++++++++
  src/engine/testing.rs              | 395 ++++++++++++++++++++
  src/hook/chord_state.rs            | 266 --------------
  src/hook/keyboard_hook.rs          | 114 ++++--
  src/hook/keyboard_hook_callback.rs | 585 +++++++++++++-----------------
  src/hook/mod.rs                    |   3 -
  src/hook/sequence_state.rs         | 230 ------------
  src/hook/tap_state.rs              | 273 --------------
  src/lib.rs                         |   1 +
  21 files changed, 3347 insertions(+), 1174 deletions(-)
  ```
- Unchanged vs master (empty diff): `Cargo.toml`, `Cargo.lock`, `src/core/**`, `src/config/default_config.ron`, `book/**`.
- Test inventory (scout ran `cargo test --locked -- --list` only; nothing executed): 189 tests, 0 `#[ignore]` in `src/`. By prefix: `engine::` 75 (`engine::scenarios` 38, `engine::gestures` 17), `hook::` 9, `data::` 30. Matches impl.md "Expected test count: 189" (impl.md:75).

## File map

| path:range | what lives there | why it matters this round |
|---|---|---|
| `src/hook/keyboard_hook_callback.rs:1-60` | adapter module docs: 6 steps, "Threads and locking", "Re-entry" (residual risk for a refused key-up) | acceptance "architecture documented"; critique M3 answer |
| `src/hook/keyboard_hook_callback.rs:77-89` | `KEYBOARD_HOOK` `LazyLock`, `get_hook()` | R3 |
| `src/hook/keyboard_hook_callback.rs:91-131` | `HookEnv` (`OnceCell<ForegroundContext>`), `impl Env` (`now`=`Instant::now()`, `os_key_down`=`GetAsyncKeyState` with SAFETY at 117-119) | D4, D7; the only Win32 reads the engine receives |
| `src/hook/keyboard_hook_callback.rs:133-143` | `key_event`: 4 key messages -> `KeyEvent`, else `None` | "every key event" |
| `src/hook/keyboard_hook_callback.rs:145-165` | `decide_event`: sentinel check (156-158) -> `key_event` (159) -> `bindings()` snapshot (162) -> `HookEnv::new` (163) -> `with_engine(on_key)` (164) | R1 order, R2, C4 tests drive it |
| `src/hook/keyboard_hook_callback.rs:167-177` | `carry_out`: timers first (171-173), then `action.execute()` in order (174-176); `mask` ignored (170) | Decision execution |
| `src/hook/keyboard_hook_callback.rs:179-213` | `start_hold_timer` (named `hold-timer` thread, `sleep_until`, `on_timer` under lock, `carry_out` on that thread; spawn error -> `eprintln!` + `forget_timer`), `sleep_until` loop | D5, KS-11, KS-33, KS-25 |
| `src/hook/keyboard_hook_callback.rs:215-242` | `keyboard_hook_callback` extern fn | R1 return paths |
| `src/hook/keyboard_hook_callback.rs:244-260` | `foreground_changed` extern fn -> `with_engine(on_foreground_changed)` | R10/KS-14 area (unchanged semantics) |
| `src/hook/keyboard_hook_callback.rs:262-335` | tests (2) | C4 red-first |
| `src/hook/keyboard_hook.rs:15-35` | `thread_local! DECIDING: Cell<bool>`, `DecidingGuard` (Drop resets) | re-entry guard |
| `src/hook/keyboard_hook.rs:37-64` | `KeyboardHook { bindings: ArcSwap, hook_handle, foreground_hook, engine: Mutex<Engine> }` | D2; R3 (spec-sanctioned Mutex) |
| `src/hook/keyboard_hook.rs:66-88` | `with_engine`: re-entry -> `None` (80-82), poison recovered (83), guard dropped before lock guard (84-86) | D2 |
| `src/hook/keyboard_hook.rs:94-99` | `update_bindings` -> `engine::sort_by_specificity` | R7 |
| `src/hook/keyboard_hook.rs:222-352` | tests: 5 master + 2 new (`with_engine_*` at 330-351) | C4 |
| `src/hook/mod.rs:1-4` | 4 modules; `tap_state`, `chord_state`, `sequence_state` removed | C6 |
| `src/engine/mod.rs:1-120` | module docs: Architecture, purity + `forbid` scope (26-32), State, Time (50-68), Threads and locking (70-86), Decision rules (88-120) | acceptance docs |
| `src/engine/mod.rs:122` | `#![forbid(unsafe_code)]` | D8 |
| `src/engine/mod.rs:130-134` | `testing` (`pub(crate)`, `cfg(test)`), `scenarios` (`cfg(test)`) | harness |
| `src/engine/mod.rs:153-224` | `KeyDirection`, `KeyEvent`, `TimerToken{vk, generation}` (private fields), `TimerRequest{token, due}`, `Decision{suppress, fire, timers, mask}` `#[must_use]`, `sort_by_specificity` (stable) | spec API |
| `src/engine/mod.rs:226-287` | `Engine` {keys, gestures, chords, sequences}; `on_key` (252-262), `on_timer` (266-272), `forget_timer` (277-279), `on_foreground_changed` (284-287) | spec API |
| `src/engine/mod.rs:289-413` | `on_key_down`: binding loop, Combo/Immediate/Tap/Hold/DoubleTap, Sequence, Chord; `mark_blocked` on suppress (409-411) | ported `M:keyboard_hook_callback.rs:186-318` |
| `src/engine/mod.rs:415-440` | `on_key_up`: `resolve_tap` + `release_hold`; suppress rules | D6' (KS-25) |
| `src/engine/env.rs:22-48` | `trait Env { now, os_key_down, foreground_app, foreground_title }` | spec Env |
| `src/engine/env.rs:53-63` | `EnvForeground` adapter to `ForegroundInfo` | lazy `when:` lookups |
| `src/engine/keys.rs:12-164` | `MODIFIERS` (11), `ActiveKeys` (cap 16), `normalize` (65-73), `slot` (<256), `KeyState { active, blocked: [bool;256] }`, `sync_modifiers`, `press`, `release`, `active_keys`, `mark_blocked`, `take_blocked`, `clear_all_blocked` | ported `M:keyboard_hook_callback.rs:34-125,345-365`; R8 |
| `src/engine/gestures.rs:1-27` | docs: deadline Tap/DoubleTap, generation-token Hold, D6' rule | KS-11, KS-25 |
| `src/engine/gestures.rs:36-82` | `PendingTap`, `PendingHold{.., generation, due, released}`, `HoldRelease{NotPending, Cancelled, Overdue}`, `PendingDoubleTap`, `Gestures` | D6' |
| `src/engine/gestures.rs:94-121` | `arm_tap` (first live wins), `resolve_tap` (`now < due`) | Tap |
| `src/engine/gestures.rs:128-193` | `arm_hold`, `release_hold`, `fire_hold`, `forget_hold`, `pending_hold_index` (ignores `released`) | Hold, KS-11/25/33 |
| `src/engine/gestures.rs:200-219` | `double_tap` | DoubleTap |
| `src/engine/gestures.rs:237-542` | 17 unit tests | ported + C2 + forget |
| `src/engine/chords.rs:26-107` | `Chords::on_key_down`, `ChordOutcome{NotInChord, Pending, Completed{action, keys}}` | ported `M:chord_state.rs:61-123` |
| `src/engine/sequences.rs:24-124` | `Sequences::on_key_down`, `SequenceOutcome{NotMatching, Advanced, Completed{action, last_step_keys}}` | ported `M:sequence_state.rs:55-124` |
| `src/engine/testing.rs:1-25` | harness docs | spec `Scenario::from_ron` |
| `src/engine/testing.rs:38-84` | `FakeEnv` (fake clock, modelled OS key state, foreground) | fake Env |
| `src/engine/testing.rs:120-268` | `from_ron` (124), `with_bindings` (131), `foreground` (146), `down` (154), `repeat` (163), `up` (172), `up_unseen` (182), `foreground_changed` (199), `advance_ms` (212-238), `advance_ms_stalled` (243-252), `lose_timers` (256-267) | C1/C2b harness steps |
| `src/engine/testing.rs:270-312` | `expect_suppressed`, `expect_passed`, `expect_fired`, `expect_nothing_fired`, `expect_total_fired` | assertions |
| `src/engine/scenarios.rs:1-725` | 38 scenario tests (list in Facts F40) | acceptance coverage |
| `src/data/condition.rs:59-78` | `Condition::evaluate(&dyn ForegroundInfo)` | 677ee50 |
| `src/data/condition.rs:81-88,170-178` | `trait ForegroundInfo`, impl for `ForegroundContext` | 677ee50 |
| `src/data/condition.rs:100-190` | `ForegroundContext::capture` (`GetForegroundWindow` 106, `OpenProcess` 144, `GetModuleBaseNameW` 149, `GetWindowTextW` 162) - unchanged code, now called lazily under the engine lock | D7, KS-03 untouched |
| `src/config/mod.rs:17-20` | `#[cfg(test)] pub(crate) use loader::from_ron_str` | harness input |
| `src/data/trigger.rs:7`, `src/data/binding.rs:63-66` | doc refs `tap_state` -> `crate::engine` | C6 |
| `src/lib.rs:8` | `pub mod engine;` | |
| `CHANGELOG.md:41-67` | Unreleased: Changed (44-51), Fixed KS-11 (55-58), KS-25 equal terms (59-64), KS-33 partly (65-67) | spec "note in CHANGELOG under Fixed" |
| `README.md` (new lines) 43-51, 53-54, 99, 103-106, 281-294, 512, 522, 529 | layout tree (engine/ added, 3 hook files removed), module bullets, "How resolution works", feature lines | docs |

## Facts

### Adapter (`src/hook`)
1. `code < 0` returns `CallNextHookEx` before anything else: `src/hook/keyboard_hook_callback.rs:220-225`.
2. The `KBDLLHOOKSTRUCT` deref happens at `:230` (SAFETY `:227-229`); only `vkCode` and `dwExtraInfo` are passed on (`:231`).
3. `decide_event` checks `constants::is_injected_event(extra_info)` (`:156-158`) before `key_event` (`:159`), the `ArcSwap` load (`:162`) and the lock (`:164`). `is_injected_event` is `src/core/constants.rs:30`.
4. `key_event` maps `WM_KEYDOWN|WM_SYSKEYDOWN` -> Down, `WM_KEYUP|WM_SYSKEYUP` -> Up, everything else -> `None` (`:135-143`). On master `sync_modifiers` ran for every `code >= 0` non-injected event before the message type was checked (`M:src/hook/keyboard_hook_callback.rs:149-152`); on the branch it runs only inside `Engine::on_key` (`src/engine/mod.rs:256`), i.e. only for the four key messages.
5. Return paths of `keyboard_hook_callback`: `LRESULT(1)` when `decision.suppress` (`:232-236`), otherwise `CallNextHookEx` (`:241`); `decide_event` returning `None` falls through to `:241`.
6. `carry_out` runs before the suppress return (`:233`), on the hook thread; it starts timers before firing actions (`:171-176`). On master Immediate actions ran inside the binding loop (`M:keyboard_hook_callback.rs:233-236`), chord/sequence actions inside their state handlers after dropping their mutex (`M:chord_state.rs:97-100`, `M:sequence_state.rs:84-86,107-108`), DoubleTap action from `tap_state` (`M:tap_state.rs:141-145`). On the branch all of them are collected into `Decision::fire` and run after `on_key` returns (`src/engine/mod.rs:313,356,380,399`).
7. The binding snapshot `bindings` and `HookEnv` are locals of `decide_event` (`:162-164`), dropped when it returns, before `carry_out` runs (`:231-233`).
8. `start_hold_timer` builds the thread name with `"hold-timer".into()` (`:188`) on the calling thread (the hook thread for key-downs). On spawn failure it calls `eprintln!` (`:197`) and `with_engine(forget_timer)` (`:198`) on the calling thread.
9. The `hold-timer` closure: `sleep_until(request.due)` (`:190`), then `get_hook().with_engine(on_timer)` (`:191`), then `carry_out` on that thread (`:193`). `Engine::on_timer` never pushes `timers` (`src/engine/mod.rs:266-272`), so a timer decision starts no further timer.
10. `sleep_until` loops on `Instant::now() >= due` around `thread::sleep(due - now)` (`:205-213`).
11. `foreground_changed` builds a `HookEnv` and calls `with_engine(|e| e.on_foreground_changed(&env))`; the `Option` result is discarded (`:257-259`). Master did the same work without a lock (`M:keyboard_hook_callback.rs:179-183`).
12. `HookEnv::foreground` uses `OnceCell::get_or_init(ForegroundContext::capture)` (`:106-108`); `Condition::Always` returns `true` without calling the trait (`src/data/condition.rs:61`). Master captured once per key-down when any binding had a non-`Always` condition (`M:keyboard_hook_callback.rs:210-214`).
13. Callback tests call `decide_event` only; no `carry_out`, no extern fn, no hook install (`src/hook/keyboard_hook_callback.rs:271-274`). `HookEnv::os_key_down` there reads the real `GetAsyncKeyState` (read-only).
14. Production `src/hook` (comments and `mod tests` excluded) has no `.unwrap(`/`.expect(` in `keyboard_hook.rs`/`keyboard_hook_callback.rs`; the only `.lock()` is `keyboard_hook.rs:83` with `unwrap_or_else(PoisonError::into_inner)`. `.expect(` remains in `config_watcher.rs:67,74` and `ipc_server.rs:80` (unchanged files).
15. `with_engine`: returns `None` without locking if `DECIDING` is set (`src/hook/keyboard_hook.rs:80-82`); `DecidingGuard` is declared after the `MutexGuard` (`:83-86`), so Rust drops it first (reverse declaration order). `DECIDING` is `thread_local!` (`:15-19`), so a `hold-timer` thread has its own flag.
16. `KeyboardHook` fields `active_keys`, `blocked_keys`, `tap_state`, `chord_state`, `sequence_state` (`M:src/hook/keyboard_hook.rs:34-41`) are replaced by `engine: Mutex<Engine>` (`src/hook/keyboard_hook.rs:51`). Atomic orderings left in `keyboard_hook.rs` are on `hook_handle`/`foreground_hook` only (`:109,127,156,163,187,199`, unchanged).

### Engine (`src/engine`)
17. `#![forbid(unsafe_code)]` at `src/engine/mod.rs:122`; `mod.rs:26-32` documents that it does not exclude safe wrappers (`ForegroundContext::capture`, `BindAction::execute`, `Instant::now()`).
18. `Engine::on_key` resyncs modifiers with `skip = Some(event.vk)` before dispatch (`mod.rs:256`); `KeyState::sync_modifiers` overwrites `active[modifier]` from `env.os_key_down` for the 11 `MODIFIERS` (`keys.rs:104-111`, list `keys.rs:12-24`), same list as `M:keyboard_hook_callback.rs:34-46`.
19. `on_key_down` reads `env.now()` once (`mod.rs:290`), marks the key pressed and gets `was_repeat` (`:291`), builds `active_keys` once (`:292`).
20. Per binding: `condition.evaluate(&EnvForeground(env))` first (`:301`), then kind. Immediate: first matching Immediate fires (`:312-315`), any matching Immediate with `block_default` suppresses (`:316-318`). Tap/Hold/DoubleTap: `continue` on repeat (`:321,333,347`), always suppress (`:330,344,358`). Sequence/Chord: `continue` on repeat (`:363,388`); Advanced/Pending suppress; Completed suppresses, fires and `mark_blocked` each key (`:375-384`, `:397-403`).
21. `if decision.suppress { mark_blocked(vk) }` at `mod.rs:409-411` (master `M:keyboard_hook_callback.rs:312-315`).
22. `on_key_up` (`mod.rs:415-440`): `release(vk)`; `resolve_tap(vk, now)`; `release_hold(vk, now)`; fire = tap action only (`:429`); if tap fired or hold is `Cancelled`/`Overdue`, `take_blocked` and suppress (`:431-435`); else suppress = `take_blocked` (`:437`). Master suppressed when `tap_fired || hold_cancelled` (`M:keyboard_hook_callback.rs:332-340`).
23. `resolve_tap` removes the entry for `vk` and fires iff `now < due` (`gestures.rs:117-121`). `arm_tap` keeps a live entry (`now < due`) and replaces an expired one (`:101-106`).
24. `arm_hold` returns `None` if a non-released Hold is pending on the key (`gestures.rs:135-137`); otherwise increments `last_generation` with `wrapping_add` (`:138`), pushes the entry and returns `TimerRequest{token{vk, generation}, due = now + term}` (`:139-153`).
25. `release_hold`: no pending (non-released) entry -> `NotPending`; `now < due` -> entry removed, `Cancelled`; else `released = true`, `Overdue` (`gestures.rs:158-169`).
26. `fire_hold(token)` removes and returns the entry with matching `vk` and `generation`, released or not (`gestures.rs:173-179`). `forget_hold` removes it without firing (`:185-188`). `pending_hold_index` skips `released` entries (`:191-193`), so a released entry and a new pending entry can coexist for one key.
27. `double_tap`: an existing entry for `vk` is always removed; fires its stored (first-press) action iff `now < first.due`; otherwise a fresh entry is armed with this binding's action (`gestures.rs:207-218`).
28. Chord: membership check before allocation (`chords.rs:66-68`); canonical sorted `Vec<u16>` (`:72-73`); expired entries dropped with `expires_at > now` (`:77`); Completed returns `chord_keys` (`:87-93`); new entry `expires_at = now + window` (`:99-104`). Master order: canonical first, then membership (`M:chord_state.rs:70-75`).
29. Sequence: `retain(expires_at > now)` (`sequences.rs:71`); advance resets `expires_at = now + max_gap` (`:79`); mismatch removes the entry and falls through to the step-0 check (`:90-93`, same control flow as `M:sequence_state.rs:93-100`, whose comment says the opposite); 1-step sequence completes immediately (`:99-104`).
30. `on_foreground_changed`: `sync_modifiers(env, None)` + `clear_all_blocked()`; gestures, chords and sequences untouched (`mod.rs:284-287`).
31. `KeyState` bounds every index through `slot(vk) -> Option<usize>` (`keys.rs:76-79`); `sync_modifiers` indexes `active[modifier.0 as usize]` directly with the 11 constant modifier codes (`keys.rs:109`).
32. Engine production code allocations on key-down: Immediate `binding.action.clone()` + `Vec` push (`mod.rs:313`), gesture arm clones (`gestures.rs:109,142,215`), chord canonical `collect` + `vec!` (`chords.rs:72,101-102`), sequence `to_vec`/clones/`collect` (`sequences.rs:101,107,109,122`). (Zero allocation is roadmap 3.4, cut D11.)
33. `Decision::mask` is never set (`mod.rs:212-215`; no assignment in `src/engine` production code).

### Harness
34. `Scenario::from_ron` -> `config::from_ron_str` (auto-blockers included) -> `with_bindings` sorts with `sort_by_specificity` (`testing.rs:124-143`).
35. `physical_key` maps unsided `Shift/Ctrl/Alt` to the left code (`testing.rs:87-97`); `FakeEnv::os_key_down` reports a generic modifier as down if either side is (`:68-75`).
36. `key_event` updates the modelled OS state only when the event was not suppressed (`testing.rs:320-323`) and records `decision.timers` (`:331`).
37. `advance_ms` fires due timers earliest first with `env.now = max(now, due)`, asserts a timer never suppresses, and appends any new timers (`testing.rs:212-238`). `advance_ms_stalled` moves the clock and fires nothing (`:243-252`). `lose_timers` drains timers and calls `forget_timer` on each (`:256-267`).
38. In the harness a timer fires exactly at `due` unless stalled; in the adapter it fires once `Instant::now() >= due` after the thread wakes (Facts 9-10).

### Tests and coverage
39. Master hook tests -> branch (by name, scout-checked against the `--list` output): `tap_state` 8 -> `engine::gestures::tests::*` (7 same names; `resolve_tap_with_no_pending_returns_false` -> `resolve_tap_with_no_pending_returns_none`); `chord_state` 5 -> `engine::chords::tests::*` (same names); `sequence_state` 4 -> `engine::sequences::tests::*` (same names); `keyboard_hook_callback` 4 -> `engine::keys::tests::{active_keys_dedup, active_keys_capacity_bound}` (same) + `blocked_bitmap_independent_bits` -> `blocked_flags_are_independent`, `clear_all_resets_every_slot` -> `clear_all_resets_every_flag`; `keyboard_hook` 5 -> same names under `hook::keyboard_hook::tests`.
40. `engine::scenarios` (38) by acceptance kind, `src/engine/scenarios.rs` line of `fn`:
    - Immediate: `immediate_single_key_fires_on_down_and_swallows_both_edges` :22, `immediate_combo_fires_on_its_last_key_and_lets_the_modifiers_through` :35, `right_hand_modifiers_match_unsided_combo_keys` :58, `combo_does_not_fire_without_all_of_its_keys` :70, `unbound_key_passes_through` :87, `more_specific_immediate_binding_wins` :98, `press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed` :134, `modifier_lost_by_the_os_is_resynced_before_matching` :155, `foreground_change_forgets_pending_key_up_suppression` :171
    - block_default false: `block_default_false_fires_and_lets_both_edges_through` :121
    - when: `when_app_equals_matches_the_foreground_app_case_insensitively` :185, `when_title_contains_and_not_conditions` :207, `first_binding_whose_condition_holds_wins_on_a_shared_key` :234
    - Tap: `tap_fires_on_a_release_within_its_term` :253, `tap_held_past_its_term_fires_nothing_and_swallows_the_release` :268, `two_quick_taps_fire_twice` :282 (KS-11)
    - Hold: `hold_fires_once_its_term_passes_while_held` :305, `hold_released_early_is_cancelled` :324, `stale_hold_timer_does_not_fire_the_next_press_early` :339 (KS-11), `tap_and_hold_with_equal_terms_fire_exactly_one` :363, `hold_released_after_its_term_fires_when_its_late_timer_wakes` :390, `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` :414, `hold_released_before_its_term_with_a_late_timer_is_cancelled` :446, `hold_left_to_a_late_timer_does_not_block_the_next_press` :461, `hold_whose_timer_cannot_start_never_fires` :490 (KS-33)
    - DoubleTap: `double_tap_fires_on_the_second_press_within_its_term` :517, `double_tap_second_press_after_the_term_arms_again` :538, `four_quick_presses_double_tap_twice` :555 (KS-11)
    - Sequence: `sequence_fires_when_its_steps_follow_within_max_gap` :583, `sequence_times_out_after_max_gap` :602, `sequence_broken_by_another_key_does_not_fire` :618, `sequence_with_a_modifier_held_through_both_steps` :634
    - Chord: `chord_fires_when_all_keys_go_down_within_its_window` :659, `chord_keys_can_go_down_in_any_order` :678, `chord_does_not_fire_once_its_window_has_passed` :688, `key_outside_the_chord_passes` :701
    - Harness self-tests (`#[should_panic]`): `a_failed_expectation_panics` :715, `releasing_a_key_that_is_up_is_a_script_error` :723
    - Total 36 behaviour scenarios + 2 harness self-tests = 38 (`grep -c '#\[test\]' src/engine/scenarios.rs` = 38; matches `--list`).
41. `hold_released_after_its_term_fires_when_its_late_timer_wakes` asserts the key-up fires nothing and the next `advance_ms(10)` fires `PressKey(VK_LSHIFT)` (`scenarios.rs:398-410`).
42. `hold_whose_timer_cannot_start_never_fires`: `lose_timers` after the down; `advance_ms(300)` fires nothing; up is suppressed; the next press fires after 200 ms (`scenarios.rs:494-511`).

### Docs and drift
43. `CHANGELOG.md:68` is a blank line inside the `### Fixed` list, between the KS-33 item (`:65-67`) and the KS-38 item (`:69`).
44. impl.md says the spec grep hit is `src/engine/mod.rs:74` (impl.md:96; cut.md addendum line 560); the actual hit is `src/engine/mod.rs:73` (`//! adapter keeps it behind a \`Mutex\` ...`).
45. `CLAUDE.md:21-22` lists `tap_state.rs`, `chord_state.rs`, `sequence_state.rs` under `src/hook/`; `CLAUDE.md:24` says `src/engine/` does not exist yet; `CLAUDE.md:65` forbids a "hook-wide `Mutex`". `.claude/agents/architect.md:35,37,42` say the same (impl.md:129-130 leaves them to the orchestrator, cut Q5).
46. Pre-existing rustdoc warning `src/data/vk_name.rs:4` "unclosed HTML tag `raw`" (impl.md:127).
47. `Condition::is_always` (`src/data/condition.rs:55-57`) is no longer called by `src/hook` (impl.md:128).

## Red-first tests and acceptance proofs (from cut.md / impl.md)

Commands are as written in the inputs; the tester owns execution. Never `--ignored`/`--include-ignored`. Replaying a red needs a scratch worktree at that commit (e.g. `git worktree add <scratch> eef8629`), not a checkout of the shared tree.

| Commit / test | Command | Expected (from impl.md) |
|---|---|---|
| `eef8629` red: `engine::scenarios::hold_released_after_its_term_fires_when_its_late_timer_wakes` | `cargo test --locked engine::scenarios::hold_released_after_its_term_fires_when_its_late_timer_wakes` | red at `eef8629`: panic `scenario: advance 10 ms to +260 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []`; green at HEAD |
| `eef8629` red: `engine::scenarios::equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` | same pattern | red: `scenario: advance 50 ms to +250 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []`; green at HEAD |
| `eef8629` red: `engine::scenarios::hold_left_to_a_late_timer_does_not_block_the_next_press` | same pattern | red: `scenario: advance 10 ms to +270 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []`; green at HEAD |
| `eef8629` guard: `engine::scenarios::hold_released_before_its_term_with_a_late_timer_is_cancelled` | same pattern | green at `eef8629` and HEAD |
| `a50e8d7` unit: `engine::gestures::tests::hold_released_at_or_after_its_due_is_left_to_its_timer`, `engine::gestures::tests::hold_left_to_its_timer_does_not_block_a_new_arm` | `cargo test --locked engine::gestures::` | green at HEAD. Mutation recorded in impl.md:24-25: `arm_hold` blocking on released entries fails both `hold_left_to_*` tests |
| `255c9a0` (red = compile error, new API): `engine::gestures::tests::forgotten_hold_never_fires_and_frees_its_key`, `engine::scenarios::hold_whose_timer_cannot_start_never_fires` | `cargo test --locked engine::` | red before: `error[E0599]: no method named \`forget_hold\` found for struct \`gestures::Gestures\`` and `no method named \`lose_timers\` found for struct \`Scenario\``; green at HEAD |
| `c72d85f` forbid probe | (manual probe, reverted) temporary `GetAsyncKeyState` call in `engine::keys` | `error: usage of an \`unsafe\` block` |
| `9e9057e` red (compile): `hook::keyboard_hook_callback::tests::physical_key_events_reach_the_engine`, `hook::keyboard_hook_callback::tests::injected_and_unknown_events_bypass_the_engine`, `hook::keyboard_hook::tests::with_engine_refuses_reentry_on_the_same_thread`, `hook::keyboard_hook::tests::with_engine_works_again_after_a_call` | `cargo test --locked hook::` | red at `9e9057e`: `error[E0425]: cannot find function \`decide_event\` in this scope`, `error[E0599]: no method named \`with_engine\` found for struct \`keyboard_hook::KeyboardHook\``; green at HEAD. Mutation (impl.md:40-41): guard disabled -> `re-entry deadlocked: Timeout` after 5 s |
| On `master` every engine/adapter test | - | compile error (no `src/engine`, no `decide_event`) (cut.md:277-279) |
| Optional KS-11 evidence on master (cut.md:357-360) | scratch `master` worktree only, test in `src/hook/tap_state.rs` with `DoNothing`: `arm_tap(F13,..,100)`; sleep 40; resolve; sleep 30; `arm_tap`; sleep 60; `assert!(resolve_tap_on_keyup(F13), "KS-11")` | fails on master |

Acceptance proofs:

| Acceptance line | Command / evidence | Expected |
|---|---|---|
| quality gate | `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`; `cargo test --locked` | pass; 189 tests, 0 ignored (impl.md:75,92-95). Developer ran only `cargo test --locked --lib -- engine:: hook:: data::` = 114 passed (impl.md:94); full suite not yet run by anyone |
| spec grep | `rg 'GetAsyncKeyState\|GetForegroundWindow\|SendInput\|Mutex\|thread::' src/engine` | one hit, a `//!` comment: scout observed `src/engine/mod.rs:73` |
| stricter engine purity (cut.md:541-557) | Git Bash, repo root: `check() { for f in "$1"/*.rs; do case "$(basename "$f")" in testing.rs\|scenarios.rs) continue;; esac; awk '/^mod tests \{/{exit} !/^[[:space:]]*\/\//{print FILENAME":"FNR": "$0}' "$f"; done \| grep -E "$2"; }` then `check src/engine 'GetAsyncKeyState\|GetForegroundWindow\|SendInput\|Mutex\|thread::\|Instant::now\|ForegroundContext\|\.execute\(\|unsafe *(\{\|fn\|impl)'` | no output, exit 1 (scout reproduced at `d3131fc`) |
| engine executes nothing | `rg '\.execute\(' src/engine` | no hits (scout reproduced) |
| no decision logic in `src/hook` (redefined, critique I2) | `check src/hook 'Trigger::\|BindingKind::\|\.matches\(\|was_repeat\|block_default\|sync_modifiers\|mark_blocked\|take_blocked\|ActiveKeys\|TapState\|ChordState\|SequenceState'` | no output (scout reproduced). Raw `rg` of the same pattern over `src/hook` has one hit: `src/hook/keyboard_hook_callback.rs:318` (`Trigger::Tap(10_000)` in `mod tests`) |
| gesture files gone | `ls src/hook` | no `tap_state.rs`, `chord_state.rs`, `sequence_state.rs` (scout: `src/hook/mod.rs:1-4`) |
| callback calls the engine for every key event | the two `hook::keyboard_hook_callback::tests` + reading `decide_event` (`:150-165`) | single path `decide_event -> with_engine -> on_key` |
| scenario coverage | Fact 40 list | each kind present |
| docs | `src/engine/mod.rs:1-120`, `env.rs:9-48`, `gestures.rs:1-27`, `testing.rs:1-25`, `keyboard_hook_callback.rs:1-60` | present |
| CHANGELOG Fixed + test | `CHANGELOG.md:55-67`; KS-11 tests (Fact 40, plus `engine::gestures::tests::stale_hold_timer_does_not_fire_a_newer_press`), KS-25 C1 tests, KS-33 `hold_whose_timer_cannot_start_never_fires` + `forgotten_hold_never_fires_and_frees_its_key` | present |

Manual checks (human only, no ignored test names them; cut.md:362-373 + addendum M9 cut.md:526-528): M1 default config CapsLock/Shift+CapsLock; M2 README Tap(200)+Hold(200) pair, 20 taps + 3 holds; M3 DoubleTap(200) four presses x2; M4 chord J+K / sequence F13,F14; M5 `TitleContains("Notepad")`; M6 Hold(300) Launch notepad no stall; M7 thread count (no `tap-gc`); M8 `PressKey("Left")` on F13 held (sentinel); M9 Hold(300) Launch notepad under CPU load, released at ~300-320 ms, 10 times.

## Open points (impl.md:125-133 and deviations impl.md:64-75)

1. Pre-existing rustdoc warning `src/data/vk_name.rs:4` (not a gate).
2. `Condition::is_always` unused by the hook, still used by `Binding` `Display` and tests.
3. `CLAUDE.md` Layout bullet and `.claude/agents/architect.md` still describe pre-engine layout (Fact 45); left to the orchestrator (cut Q5).
4. Follow-up candidate for 3.4: move `BindAction` out of `core::windows_actions` (engine imports it at `src/engine/mod.rs:141`, `gestures.rs:34`, `chords.rs:24`, `sequences.rs:21`).
5. Working-tree line endings: LF in working copy with `core.autocrlf=true`; index LF; `git status` clean.
6. Deviations: D6 -> D6' (overdue Hold left to its timer, key-up fires nothing; critique I1); D5 fallback is now "Hold dropped" via `forget_timer` (`255c9a0`), not "fires at release"; extra doc commit `d3131fc`; README "Bitmap-blocked keys" -> "Independently blocked keys" (`README.md:529`); expected count 189 not 185.
7. Critique remarks I1, I2, M1, M2, M3 marked RESOLVED by the developer (impl.md:51-60; cut.md:488-578). Cut questions Q1-Q5 implemented as recommended (impl.md:61-62); Q1 (Mutex vs CLAUDE.md principle 5 / code-reviewer R3) is a wording conflict with `CLAUDE.md:65`.

## Invariants in play

- R1 hook callback: `keyboard_hook_callback.rs:220-241` (return paths), `:156-158` (sentinel before state), `:83` of `keyboard_hook.rs` (lock in the hot path, poison recovered), `:188,197` (allocation / I/O on the hook thread on Hold arm / spawn failure), `src/data/condition.rs:100-190` (foreground Win32 calls under the lock). KS-08 budget.
- R2 sentinel: `keyboard_hook_callback.rs:156`; no new `SendInput` (`src/core` unchanged).
- R3 shared state: `keyboard_hook.rs:37-52` (`Mutex<Engine>` beside lock-free `ArcSwap` and atomics), `keyboard_hook_callback.rs:80-89`; `CLAUDE.md:65` wording (Open point 7).
- R4 atomics: none added; `active_keys`/`blocked_keys` atomics removed (`M:keyboard_hook.rs:34-35`); `DECIDING` thread-local (`keyboard_hook.rs:15-19`).
- R7 specificity: `src/engine/mod.rs:222-224` (stable `sort_by_key`), used by `keyboard_hook.rs:97` and `testing.rs:132`; pinned by `hook::keyboard_hook::tests::equal_specificity_keeps_config_order`.
- R8 key identity: `keys.rs:65-73` folding for combos/sequences; chords use raw `vk` (`chords.rs:66,82`), KS-07 unchanged (`chords.rs:16-18`).
- R10 shutdown: `hold-timer` firing after uninstall unchanged (cut.md:390, KS-31).
- Suppression symmetry: `mod.rs:409-411` / `:431-438`; re-entry asymmetry documented `keyboard_hook_callback.rs:47-60`.
- Timers: KS-11 (`gestures.rs:138-153,173-179`), KS-25 equal terms (`gestures.rs:158-169`, `mod.rs:415-440`), KS-33 partly (`keyboard_hook_callback.rs:186-200`, `mod.rs:277-279`).
- Out of scope, unchanged by design (cut.md:60-64): KS-04/05/06/07/10/13/14/15/16/18, Decision::mask (1.2), zero-alloc (3.4).

## Not covered

- Scout did not run the gate (`fmt`, `clippy`, `cargo test`); only `cargo test --locked -- --list` (compiles, executes nothing).
- Not read: `src/engine/scenarios.rs` bodies other than `:385-512`; unit-test bodies in `chords.rs`, `sequences.rs`, `keys.rs`, most of `gestures.rs` tests; `critique.md`; `src/core/**`, `src/config/loader.rs` (`from_ron_str`), `src/hook/config_watcher.rs`, `src/hook/ipc_server.rs`; README outside the diff hunks; book pages.
- Behavioural equivalence of `Immediate` actions now running after the binding loop rather than inside it (cut.md:393-394 claims no action changes matching state synchronously): UNVERIFIED by the scout.
- Red commits were not replayed by the scout; red messages above are quoted from impl.md.
