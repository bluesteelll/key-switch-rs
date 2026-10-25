# Implementation: p0-A1-engine round 1

Verdict: DONE (deviations listed below; critique remarks answered in the addendum at the end of `cut.md`).
Branch `refactor/engine`, HEAD `d3131fc`, tree clean. Base of the remainder: `1d12c7b` (3 earlier commits kept).

## Items

- C0 / D0 - merge of `master` (`73bbf40`) into the branch: `3d785dc` (no conflicts; brings `CLAUDE.md`, `.claude/**`,
  `docs/WORKFLOW.md`, `.gitattributes`).
- C1 + C2 / D6' (KS-25 equal terms; critique I1) - `src/engine/testing.rs:243` `Scenario::advance_ms_stalled`;
  `src/engine/gestures.rs:57-200` (`HoldRelease`, `release_hold`, `pending_hold_index`, `PendingHold::{due, released}`);
  `src/engine/mod.rs:415-440` (`on_key_up`), docs `mod.rs` "Time" and "Decision rules".
  Red-first (commit `eef8629`, run on the branch head before the fix):
  - `engine::scenarios::hold_released_after_its_term_fires_when_its_late_timer_wakes`: red
    "scenario: advance 10 ms to +260 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []" -> green.
  - `engine::scenarios::equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one`: red
    "scenario: advance 50 ms to +250 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []" -> green.
  - `engine::scenarios::hold_left_to_a_late_timer_does_not_block_the_next_press`: red
    "scenario: advance 10 ms to +270 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []" -> green.
  - `engine::scenarios::hold_released_before_its_term_with_a_late_timer_is_cancelled`: green guard (kills "always fire at release").
  - unit (C2, `a50e8d7`): `engine::gestures::tests::hold_released_at_or_after_its_due_is_left_to_its_timer`,
    `engine::gestures::tests::hold_left_to_its_timer_does_not_block_a_new_arm`.
  - The 5 ported gesture tests that called `cancel_hold` call `release_hold` before the term, assertions unchanged.
  - Mutation run: `arm_hold` blocking on released entries (`self.holds.iter().any(|p| p.vk == vk.0)`) fails
    `hold_left_to_its_timer_does_not_block_a_new_arm` and `hold_left_to_a_late_timer_does_not_block_the_next_press`; reverted.
- C2b / D5 + D6' (KS-33 partly) - `Engine::forget_timer` `src/engine/mod.rs:277`, `Gestures::forget_hold`
  `src/engine/gestures.rs:185`, harness `Scenario::lose_timers` `src/engine/testing.rs:256`. Red-first (compile error,
  new API): "error[E0599]: no method named `forget_hold` found for struct `gestures::Gestures`" and
  "no method named `lose_timers` found for struct `Scenario`" -> green:
  `engine::gestures::tests::forgotten_hold_never_fires_and_frees_its_key`, `engine::scenarios::hold_whose_timer_cannot_start_never_fires`.
- C3 / D8 - `#![forbid(unsafe_code)]` at `src/engine/mod.rs:122`; docs `:26-32` (critique M1). Probe: a temporary
  `GetAsyncKeyState` call in `engine::keys` -> "error: usage of an `unsafe` block"; removed.
- C4 + C5 / D2-D5, D7, D10 - `src/hook/keyboard_hook.rs:14-98` (`DECIDING`, `DecidingGuard`, `engine: Mutex<Engine>`,
  `with_engine`, `update_bindings` -> `engine::sort_by_specificity`); `src/hook/keyboard_hook_callback.rs:1-260`
  (module docs, `HookEnv`, `key_event`, `decide_event`, `carry_out`, `start_hold_timer`, `sleep_until`, both callbacks).
  Red-first (commit `9e9057e`, compile error): "error[E0425]: cannot find function `decide_event` in this scope",
  "error[E0599]: no method named `with_engine` found for struct `keyboard_hook::KeyboardHook`" -> green:
  - `hook::keyboard_hook_callback::tests::physical_key_events_reach_the_engine` (all four key messages; ups follow a suppressed down; critique M2)
  - `hook::keyboard_hook_callback::tests::injected_and_unknown_events_bypass_the_engine`
  - `hook::keyboard_hook::tests::with_engine_refuses_reentry_on_the_same_thread` (mutation: guard disabled ->
    "re-entry deadlocked: Timeout" after 5 s; reverted)
  - `hook::keyboard_hook::tests::with_engine_works_again_after_a_call`
  Removed with C5: the 4 old callback helper tests (ported as `engine::keys::tests::{active_keys_dedup,
  active_keys_capacity_bound, blocked_flags_are_independent, clear_all_resets_every_flag}`).
- C6 / D9 - `src/hook/{tap_state,chord_state,sequence_state}.rs` deleted, `src/hook/mod.rs`, doc refs
  `src/data/trigger.rs:7`, `src/data/binding.rs:63`, README layout. All 17 removed tests have ported counterparts
  (checked by name; `resolve_tap_with_no_pending_returns_false` is `..._returns_none` in the engine, as D9 lists).

## Remarks (critique.md round 1; full text in the cut.md addendum)

- I1 - RESOLVED (option (a) as D6': Hold fires only from its timer; key-up at/after the term leaves it armed) at
  `src/engine/gestures.rs:158`, `src/engine/mod.rs:415-440`, `src/hook/keyboard_hook_callback.rs:186-200`; pinned by
  `hold_released_after_its_term_fires_when_its_late_timer_wakes` (key-up fires nothing), `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one`,
  `hold_left_to_a_late_timer_does_not_block_the_next_press`, `hold_whose_timer_cannot_start_never_fires`. Manual check M9 added.
- I2 - RESOLVED: the acceptance check is redefined on production code (test modules and comment lines excluded), with a
  runnable command in the addendum; it returns nothing at `d3131fc`. The C4 test is unchanged.
- M1 - RESOLVED: `src/engine/mod.rs:26-32` (what `forbid(unsafe_code)` covers, `BindAction` as data); stricter engine
  check in the addendum, no output at `d3131fc`. `BindAction` -> `data` recorded as a 3.4 follow-up.
- M2 - RESOLVED at `src/hook/keyboard_hook_callback.rs:287` (`physical_key_events_reach_the_engine` loops over both pairs).
- M3 - RESOLVED (docs): adapter module docs, section "Re-entry", state the key-up asymmetry, the residual risk and the hand-over to 2.2/3.4.
- Orchestrator questions Q1-Q5 of the cut: not developer remarks; implemented as recommended (Mutex per D2, D0 merge,
  D6 kept in its D6' form, book left to the phase sync, CLAUDE.md/architect.md "src/engine does not exist" lines left to the orchestrator).

## Deviations from the cut

- D6 -> D6' (critique I1): an overdue Hold is left to its timer instead of firing in the key-up `Decision`. C1 test names
  and expectations differ accordingly (key-up fires nothing; the next `advance_ms` fires the Hold); one extra scenario and
  one extra unit test. Alternative considered: a separate `Decision` field for actions that must leave the hook thread
  (critic option (a) literally) - rejected: changes the D1 API and still needs a thread per overdue Hold; D6' reuses the
  thread that already exists.
- D5 fallback: a failed `hold-timer` spawn now drops the Hold (`Engine::forget_timer`, new commit `255c9a0`) instead of
  firing it at the key-up, which would run it on the hook thread.
- Extra commit `d3131fc`: rustdoc warnings from the new docs (links to private items) fixed.
- README line 520 "Bitmap-blocked keys" -> "Independently blocked keys" (the bitmap no longer exists).
- Expected test count: 189 (`cargo test --locked -- --list` at HEAD), not 185.

## Unsafe / atomics added

- `src/hook/keyboard_hook_callback.rs:117` - SAFETY: "GetAsyncKeyState takes the key code by value, writes no memory of
  ours and may be called from any thread; a code outside the virtual-key range only reads as up."
- `src/hook/keyboard_hook_callback.rs:221` and `:239` - SAFETY: "passes the event on with the exact arguments the system
  gave this hook procedure" (`CallNextHookEx`).
- `src/hook/keyboard_hook_callback.rs:227` - SAFETY: "for code >= 0 (HC_ACTION) lparam points to a valid KBDLLHOOKSTRUCT
  supplied by the hook dispatcher for the duration of this call; two fields are copied out of it here."
- No new atomics. `DECIDING` is a `thread_local!` `Cell<bool>` (`src/hook/keyboard_hook.rs:18`). Removed: `active_keys`
  (`[AtomicBool; 256]`), `blocked_keys` (`[AtomicU64; 4]`), `ATOMIC_FALSE`.
- New lock: `KeyboardHook::engine: Mutex<Engine>` (`src/hook/keyboard_hook.rs:51`), sanctioned by the spec and cut D2;
  replaces the five gesture mutexes of master. Held only inside `with_engine`; no action, spawn or `ArcSwap` guard across it.

## Gates

- `cargo fmt --check`: pass. `cargo check --all-targets --locked`: pass. `cargo clippy --all-targets --locked -- -D warnings`: pass (HEAD `d3131fc`).
  Per commit: pass for every commit except the declared reds `eef8629` (assertion red) and `9e9057e` (compile red).
- Tests run by the developer: `cargo test --locked --lib -- engine:: hook:: data::` at HEAD: 114 passed, 0 failed.
  Full suite left to the tester (189 tests listed, 0 ignored).
- Acceptance greps at HEAD: spec grep over `src/engine` -> one `//!` comment hit (`src/engine/mod.rs:74`); stricter
  engine check and the `src/hook` decision-logic check (addendum) -> no output; `rg '\.execute\(' src/engine` -> no hits.
- Manual ignores added: none (no automated test needs a live hook). Human checks: cut M1-M8 plus addendum M9
  (Hold + Launch released just after its term under load: fires once per press held >= term, no input stall).

## Docs

- Module docs: `src/engine/mod.rs` (purity, Time incl. late timers and KS-25, Threads and locking incl. `with_engine`,
  re-entry refusal and the clock read under the lock, Decision rules for key-up), `src/engine/env.rs` (`now`),
  `src/engine/gestures.rs` (Hold resolution), `src/engine/testing.rs` (stalled timers), new `//!` header of
  `src/hook/keyboard_hook_callback.rs` (adapter steps, threads and locking, re-entry and residual risk).
- `CHANGELOG.md` Unreleased: Changed (decision core) and Fixed KS-11, KS-25 (equal terms), KS-33 (partly).
- `README.md`: layout tree (engine/ added, three hook files removed), `keyboard_hook.rs` and `keyboard_hook_callback.rs`
  bullets, "How resolution works", the hot-path, thread-safety and blocked-keys feature lines.
- `default_config.ron`, `book/src/**`: unchanged (config meaning unchanged; book left to the phase sync, list in the
  cut's Docs impact plus the §11.5/§11.6 notes in the addendum).

## Commits

- `3d785dc` Merge branch 'master' into refactor/engine
- `eef8629` Engine: red-first: a Hold released after its term must still fire
- `a50e8d7` Engine: a Hold released after its term fires from its timer (KS-25)
- `255c9a0` Engine: forget a Hold whose timer cannot be started (KS-33)
- `c72d85f` Engine: forbid unsafe code
- `9e9057e` Hook: red-first: the adapter routes physical events through the engine
- `941f0a7` Hook: the callback becomes an adapter over the engine
- `0873abf` Hook: remove tap_state, chord_state and sequence_state
- `d3131fc` Hook: adapter docs link no private items

## Open points / pre-existing warnings

- Pre-existing rustdoc warning (not a gate): `src/data/vk_name.rs:4` "unclosed HTML tag `raw`".
- `Condition::is_always` is no longer used by the hook; still used by `Binding`'s `Display` and tests, so not dead.
- `CLAUDE.md` "Layout" bullet and `.claude/agents/architect.md` still say `src/engine` does not exist and list the
  deleted hook files (cut Q5: the orchestrator updates them after the merge).
- Follow-up candidate for 3.4: move `BindAction` (data) out of `core::windows_actions` so the engine depends only on `data`.
- Working-tree line endings: files written in this lane are LF in the working copy (`core.autocrlf=true`); the index is
  LF as before, so `git status` is clean and no content change results.
