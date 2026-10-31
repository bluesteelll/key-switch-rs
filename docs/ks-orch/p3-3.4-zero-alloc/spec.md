# Lane spec: p3-3.4-zero-alloc - Zero-allocation decision core (roadmap 3.4, acceptance A4)

Branch: perf/zero-alloc-engine

Roadmap item 3.4 (book/src/roadmap.md 'Core consolidation', :708-725 at 252daef), acceptance 'With a counting
`#[global_allocator]`, `Engine::on_key` does not allocate' (:725), plus the 2.1 hand-off '`Binding.action: Arc`'
(p2-2.1-executor/results.md:162), the A6 note on zero allocation (p0-A6-appequals/cut.md:59) and the title read of a
`when:` without a heap `String` (3.5 scope, lane B item 4). It comes last in Phase 3: after p3-3.4-compiled-bindings,
whose `CompiledBindings` it extends, and after p3-3.1-held-modifiers and p3-3.3-launch-de-elevate, which change
`Decision`, `FastAction` and the `Launch` payload. Binding: lowest key-to-action latency; no key may ever stick. Scope
report: <orch>/phase-3/scope/3.4-core.md (sections 1.4, 4, 8 forks T1-T3, 9 lane B).

1. Shared action handles (scope fork T1): `CompiledBindings` holds `Arc<BindAction>`; `Decision::fire`
   (`src/engine/mod.rs:396`), `carry_out` and `dispatch` (`src/hook/keyboard_hook_callback.rs:303-334`),
   `KeyboardHook::submit` (`src/hook/keyboard_hook.rs:104`) and the executor's `Job` and `submit`
   (`src/core/executor/mod.rs:217-238`) move them; `BindAction::as_fast` (`src/core/windows_actions.rs:187-206`) works
   on the handle. The cut decides whether the inner `Arc` that p3-3.3 put on the `Launch` payload stays or is subsumed.
   No `ArcSwap` guard is held across `carry_out` (keyboard_hook_callback.rs:258-259).

   Note (lane feat-text-action): `BindAction::Text` carries an `Arc<str>`; a firing only bumps its refcount
   (`engine::scenarios::a_text_firing_shares_the_text_of_its_loaded_binding` pins it). The cut decides whether it stays
   inside `Arc<BindAction>`, as for the `Launch` payload.

   Note (lane feat-clipboard-paste): `BindAction::Paste` carries an `Arc<PasteSpec>` (`src/data/paste.rs`), its text
   encoded once at load; a firing only bumps its refcount
   (`engine::scenarios::a_paste_firing_shares_the_spec_of_its_loaded_binding` pins it). The cut decides whether it
   stays inside `Arc<BindAction>`, as for `Text` and the `Launch` payload.
2. `Decision` without per-event heap growth (scope fork T2: inline fixed capacity sized from a worst case computed at
   compile, or recycled buffers, the cut's choice with its copy cost stated): the pushes at `src/engine/mod.rs:796-798`,
   :846, :875-877, :904-906, :974, :976-978; `Engine::start_timers` takes the new shape (:608-624). A `Decision` that
   exceeds its capacity is counted in `logging::counters`, never silently allocated on the hook path.
3. Pending entries hold indices or handles into `CompiledBindings`, not copies: `src/engine/gestures.rs:217-229` (three
   `action.clone()` and a `GroupKey` clone per arming; the DoubleTap window identity becomes a group id),
   `src/engine/chords.rs:98-121`, `src/engine/sequences.rs:119-128` (after p3-3.2 deleted the completion key lists).
   The per-key press table and the due Holds are preallocated or fixed in size (`gestures.rs:152-153`). Every list swap
   resets the engine in the same main-thread step (`MainState::reload`, `src/hook/main_thread.rs:341-344`), which makes
   indices sound.
4. Conditions compare without allocating: needles case-folded once at compile, the haystack folded on the fly
   (`src/data/condition.rs:63`, :80-86). The Unicode rule stays as the existing tests pin it
   (`data::condition::tests::equals_conditions_fold_case_like_title_contains`, condition.rs:204;
   `engine::scenarios::when_title_equals_and_app_equals_ignore_case_beyond_ascii`, scenarios.rs:288).
5. The title of a `when:` is read without a heap `String`: today `HookEnv` keeps `OnceCell<Option<String>>`
   (`src/hook/keyboard_hook_callback.rs:184`) filled by `window_title` with `String::from_utf16_lossy`
   (`src/platform/foreground.rs:169-177`). A lazily filled fixed buffer is one option; a title longer than the buffer
   follows a rule the cut states (no false match). The title stays read live, at most once per key-down (owner
   decision 2026-10-19).
6. A counting `#[global_allocator]` in the library's `cfg(test)` build, with a thread-local count armed only around the
   measured calls (scope fork T3), so `engine::testing::Scenario` and `config::from_ron_str` (both `pub(crate)`) can
   drive it. The cut states that the release bench binary carries it unarmed.

Acceptance:
- Unit (counting allocator), red first on the parent with counts above 0 on firing and arming events: `Engine::on_key`
  and `Engine::on_timer` allocate nothing, after compile and one warm-up press per key, over the property scripts of
  every config of p3-3.4-groundwork and over firing configs with `Launch`, `OpenUrl`, `Text`, `Paste`, `Sequence`,
  Tap+Hold+DoubleTap groups, chords, sequences, and `TitleContains`/`TitleEquals` with a non-ASCII title from a fake
  `Env`.
- Unit: the adapter's key-down with a title condition allocates nothing (the counting allocator around
  `decide_event` with a fake foreground), or the cut shows why the title read stays one allocation and declares it.
- Unit: an overflowing `Decision` bumps its counter and drops nothing silently.
- Unit and real-thread tests: all scenarios, properties and adapter tests stay green, including
  `carry_out_queues_the_slow_actions_in_order_and_runs_none_here` (keyboard_hook_callback.rs:1114) and the executor
  tests; the Cyrillic and Greek condition tests stay green.
- Bench (counted apart): R2, R3, R4, R5 and R6 H <= P within the rule; R1, R9-R12, R12m and the R19 family within the
  CLAUDE.md rules; new row R26, `Engine::on_key` of a key-down that fires `Launch` and of one that arms a
  Tap+Hold+DoubleTap group, H < P. The change in `Decision` size and copy cost is stated.
- Manual (owner, later batch) M-B1: a `Launch`, an `OpenUrl`, a `Text`, a `Paste` and a `Sequence` bound to keys still
  run once each.
- Gate green.

Latency: removes the heap allocations of arming and firing events, and with them the waits they can take on the process
heap's lock, which the log-writer, executor and watcher threads share (a lock by construction; contention UNVERIFIED).
A fire costs one atomic increment per action. No new Win32 call, lock, timer or wake-up. The `Decision` copy size is set
and measured by the cut against R1's 0.083 us mean.

Docs: CHANGELOG `[Unreleased]` Changed: the decision core allocates nothing per key event (roadmap 3.4); README.md
hot-path line, scoped to `Engine::on_key` and `on_timer` (the old 'Hot-path allocations: none' claim is gone,
gaps.md:193); `book/src/config/errors.md` for the overflow counter. Book-sync list in impl.md with file:line:
architecture.md 'Core data structures', gaps.md:193, the roadmap 3.4 status (A4 done). Deferred, not this lane: moving
`BindAction` into `data` (p0-A1-engine/results.md:103; a mechanical move across 21 files, a chore after Phase 3).
