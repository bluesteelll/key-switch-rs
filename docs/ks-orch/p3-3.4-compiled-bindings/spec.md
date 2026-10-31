# Lane spec: p3-3.4-compiled-bindings - Compiled bindings: a per-key index, auto-blockers apart, load-time diagnostics (roadmap 3.4)

Branch: feat/compiled-bindings

Roadmap item 3.4 (book/src/roadmap.md 'Core consolidation', :708-725 at 252daef), bullet W2, and known issues 69 and 73
(`book/src/config/known-issues.md:137`, :141), the Phase 1 follow-ups that name 3.4 (roadmap :967-968), the Phase 2
follow-up on unequal Tap and Hold terms (:986) and the gap '`block_default` is accepted and ignored on `sequence` and
`chord`' (book/src/gaps.md:169). It starts after p3-3.2-seq-chord-matching (it restructures the same loop,
`src/engine/mod.rs:805-918`) and after p3-3.1-held-modifiers. The owner's answer to plan_draft.md section 5 Q8 decides
item 5. Binding: lowest key-to-action latency; no key may ever stick; the 1.6 precedence (keys, `when:`, file order).
Scope report: <orch>/phase-3/scope/3.4-core.md (sections 1-9, lane A).

1. `CompiledBindings` (new `src/engine/compiled.rs`, pure, under the engine's `forbid(unsafe_code)`), built once per
   load on the loading thread (startup or `config-watcher`), holding:
   - the user bindings in stable key-count order (today `sort_by_specificity`, `src/engine/mod.rs:504-506`, R7);
   - `os_guards`, the auto-blockers that `src/config/loader.rs:340-416` produces, which today ride in the same list
     (`loader.rs:413-415`, `src/data/binding.rs:132`);
   - a bindings `generation` that moves on with each store (the engine's `Ids.generation`, `mod.rs:350-357`, is a
     timer generation and stays);
   - `by_vk[256]`, built with `matching::is_trigger_key` (`src/engine/matching.rs:62-67`), so `Shift` indexes LShift,
     RShift and the generic code (KS-07, R8);
   - the precomputed gesture groups (today scanned at `src/engine/winner.rs:110-145`).

   Note (lane feat-text-action): `BindAction::Text(Arc<str>)` is one more slow action; `CompiledBindings` treats it like
   `Launch` and `OpenUrl`. Pre-encoding its UTF-16 once at compile, for the input module's builder, is optional.

   Note (lane feat-clipboard-paste): `BindAction::Paste(Arc<PasteSpec>)` is one more slow action, treated like `Text`;
   its clipboard text is already encoded once at load (`src/data/paste.rs`), so compiling adds nothing for it.
   `KeyboardHook` holds `ArcSwap<CompiledBindings>` (`src/hook/keyboard_hook.rs:34`); `update_bindings` (:85-91) stores
   it with one atomic store and no longer sorts; `MainState::reload` (`src/hook/main_thread.rs:341-344`) swaps and
   resets. On a generation mismatch without a reset (bench, tests) `on_key` treats it as a reload, with a
   `debug_assert` (scope fork T8).
2. `Engine::on_key(event, &CompiledBindings, env)`. The election (`winner.rs:59-88`) and the blocker pass
   (`mod.rs:808-836`) visit `by_vk[vk]` only. Whether chords and sequences join the index (scope 3.5 TF-6: chords by
   chord keys, sequences by step-0 trigger keys plus the pending list) is the cut's choice; either way the
   `when:`-after-match rule of p3-3.2 holds and is pinned by its tests.
3. Load warnings in `loader.rs`, next to `duplicate_warnings` (:305-320), naming entries as `#n [keys as written]`
   (`format_raw_binding_keys`, :604), logged on the loading thread; the config still loads (the 1.6 precedent,
   errors.md section 18.6):
   - (a) a `keys` binding that never acts because an earlier binding of the other trigger family, on the same keys and
     `when:` with an `allow_extra_modifiers` at least as wide, wins every press it could win (known issue 69;
     `winner.rs:46-48`, :165-167);
   - (b) a gesture group with `Tap(t)` and `Hold(h)` where t < h (KS-25; `tap_100_and_hold_200_released_at_150_fire_nothing`,
     `src/engine/scenarios.rs:759`);
   - (c) the suppression trap: a `chord`, or a `sequence` whose first step, holds keys typed in text (letters, digits,
     OEM punctuation, Space, Enter, Tab, Backspace) with no modifier (scope fork T10; worded so that a future
     `replay: true`, new feature 5, silences it);
   - (d) `block_default` on a `sequence` or `chord`, which those kinds ignore (gaps.md:169; `src/data/binding.rs:183-188`;
     errors.md section 18.7). A warning, not a load error, so no config stops loading.
4. `swch status` (`src/core/app.rs:90`) and the reload line (`src/hook/config_watcher.rs:201`) keep counting user
   bindings plus guards (the default config prints 3).
5. Only if Q8 says lock (option a or c): a per-key winner slot, keyed by the bindings generation, set at a first
   key-down and read at its repeats (`mod.rs:770-774`, `src/engine/keys.rs`); (c) also stops the repeats once the
   winner's keys or `when:` no longer match.

Acceptance:
- Unit (pure): for every code 0..256 and every binding, `by_vk` membership equals `is_trigger_key`; guards are never
  user candidates; file order is kept among equal key counts (the port of `bindings_sorted_by_specificity`,
  `keyboard_hook.rs:382`).
- Scenario and property: all `engine::scenarios` tests and the 10,000-script net of p3-3.4-groundwork, plus a new
  property config with many bindings over the 13 keys, are green with no expectation changed, except the row-73 pin
  `repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down` (`scenarios.rs:5214`) under a lock
  answer; `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (:2923) and
  `reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat` (:2990) stay green.
- Loader unit (through `warnings_of`, `loader.rs:2542`), each red first: warnings (a) to (d); one case that must not warn
  for (a): a later binding with a wider `allow_extra_modifiers` is not shadowed. The template and every published RON
  example load with no new warning, except where the page documents one (`cfgcheck`, output read with `2>&1`).
- Unit: `swch status` prints the same count for the default config, 3 (`DaemonStatus`).
- Bench (counted apart): R19, R19b, R19c and R19e H <= P (50 bindings, rows of p3-3.5-budget-bench); R1-R18 and the rest
  within the CLAUDE.md rules.
- Manual (owner, later batch; only if the lock ships) M-A1: in Notepad, hold Ctrl+J with the row-73 config, release
  Ctrl and keep J held: the chosen behaviour holds.
- Gate green.

Latency: per key-down the election and the blocker pass drop from O(all bindings) to O(bindings indexed under vk);
still one `ArcSwap` load per event (`src/hook/keyboard_hook_callback.rs:260`); no new allocation, lock, Win32 call,
timer or wake-up. Main does less work at a reload: the sort and compile move to the loading thread. A lock adds one slot
write at a first key-down and replaces the election at a repeat. With the default 3 bindings the gain is near zero
(R1 mean 0.083 us); the R19 family shows it.

Docs: CHANGELOG `[Unreleased]`: Added (warnings a-d; KS-25, known issue 69, roadmap 3.4); Changed, **Breaking:** the
winner per press, if Q8 locks (known issue 73). README.md (module tree `engine/compiled.rs`, 'How
resolution works', the warnings); `src/config/default_config.ron:124-140`; `book/src/config/errors.md` section 18.6
(new rows), section 18.7 (drop the Immediate-versus-deferred and `block_default` rows); `known-issues.md` rows 69 and 73;
`triggers-conditions.md` sections 11.3 and 11.6; `matching.md` section 16.3; every new RON example through `cfgcheck`.
Book-sync list in impl.md with file:line: architecture.md (core data structures), gaps.md :162, :169, the roadmap 3.4
status and the Follow-ups rows :967, :968, :986; a roadmap 'Planned changes' row if Q8 locks.
