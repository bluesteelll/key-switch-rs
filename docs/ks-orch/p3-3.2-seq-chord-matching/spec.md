# Lane spec: p3-3.2-seq-chord-matching - Sequence and Chord matching (roadmap item 3.2)

Branch: fix/sequence-chord-matching

Roadmap item 3.2 (book/src/roadmap.md 'Sequence and Chord matching', :686-696 at 252daef), plus the `when:`-after-match
rule for `sequence` and `chord` that item 3.5 carried (:737, :739; moved here, plan_draft.md section 2, C1), F1 of item
2.3 (:991) and known issue 19. Binding owner decisions: exact modifier matching with a per-binding
`allow_extra_modifiers` (CLAUDE.md; applied here to sequence steps, plan_draft.md section 5 'Decided'); one binding wins
each key press (item 1.6); the lowest key-to-action latency; no key may ever stick. Scope report:
D:/tmp/ks-orch/phase-3/scope/3.2-seq-chord.md (sections 1.5, 8, 9). Do not confuse `src/engine/sequences.rs` (the
`sequence` binding kind, this lane) with `src/data/sequence.rs` (the `Sequence([...])` macro action, 3.1 and 3.3).

1. KS-15, the trigger-key rule for steps (`src/engine/sequences.rs:71-136`, the tests at :91 and :114;
   `src/engine/mod.rs:857-890`). `Sequences::on_key_down` takes the event's vk. A step, the next one or step 0 at a
   fresh start, counts only when the pressed key is one of its trigger keys and its keys match by the combo rule
   (`matching::combo_matches`, `src/engine/matching.rs:74-83`, with the binding's `allow_extra_modifiers`). An
   unrelated key pressed while a step's keys are held neither completes nor re-arms anything, and it passes.
2. Exact modifiers on steps (owner decision on exact matching). A step no longer matches while a modifier it does not
   name is held; `allow_extra_modifiers: true` becomes legal on `sequence` and stays a load error on `chord`
   (`src/config/loader.rs:489-497`, test `allow_extra_modifiers_is_only_for_keys_bindings` at :2326). Chords keep
   matching whatever modifiers are held (`book/src/config/kinds.md:144`). Rewrite the two scenarios that hold an
   unnamed modifier across step 0 (`src/engine/scenarios.rs:1378-1407`, :1410-1428) with the modifier pressed after
   step 0, which item 3 makes possible.
3. KS-16, the modifier wait (`src/engine/sequences.rs:108-110`). While an entry waits for step N, a key-down of a
   modifier that step N names (for a modifier-only step, any of its keys that does not complete it) neither aborts nor
   advances the entry, and it passes. Any other key-down aborts and is then checked as step 0, as today. The deadline
   does not move and no timer is asked for (keeps 2.2's `seq` token discipline, p2-2.2-timers/cut.md:52).
4. F1 of item 2.3 (`src/engine/keys.rs:249-261`). Delete `KeyState::mark_completed`, the key lists of
   `SequenceOutcome::Completed` and `ChordOutcome::Completed` (`src/engine/sequences.rs:57-60`, `last_step_keys`
   :156-170; `src/engine/chords.rs:62`, :107-110) and the loops at `src/engine/mod.rs:885-887` and :911-913. The scope
   report traced that the function changes a record only for a held key with no record (Windows has it down) or a
   `RepeatSuppressed` one; impl.md shows by a mutation run that the deletion changes no test except the function's own
   unit test (`src/engine/keys.rs:609-635`, removed) and the new F1 scenario. If the architect keeps a guarded
   function instead, the cut argues it.
5. `when:` after the match. A chord tests membership (`src/engine/chords.rs:81-83`) before its condition
   (`src/engine/mod.rs:891-894`). A sequence asks its condition only for a trigger key of step 0, or for any non-repeat
   key-down while its shape is pending (`src/engine/mod.rs:857-862`). The documented rule that a false `when` neither
   advances nor resets a pending entry (`book/src/config/kinds.md:82`, section 8.2 item 10) holds.
6. Known issue 19 (`book/src/config/known-issues.md:51`), under the one-winner rule of 1.6: one key-down feeds one
   shape once. A sequence entry started, advanced or completed by this key-down is not fed again by a later binding
   with the same steps, and a chord key set completed by this key-down is not re-armed by a later binding with the
   same keys (`src/engine/sequences.rs:84-133`, `src/engine/chords.rs:91-125`). The first binding in file order fires
   once. No allocation. Fix the module doc claim at `src/engine/sequences.rs:8-10` to match.
7. Bookkeeping: name the already ported chord repro test against the acceptance line
   (`engine::chords::tests::unsided_shift_chord_completes_with_either_shift`, chords.rs:309-335;
   `engine::scenarios::unsided_shift_in_a_chord_fires_for_either_side`, scenarios.rs:1336-1363); fix the rustdoc
   examples at `src/config/schema.rs:120` and :131 (`g, g` and `j, k` eat typed letters; use F13/F14, Workstream B
   results :29-30); update the engine module doc rules at `src/engine/mod.rs:202-204`, :215-217, :247-249.

Acceptance:
- Unit, red on master (`engine::sequences::tests`): ported and inverted `sequence_completes_on_rollover_of_unrelated_key`
  (`[F13,F13]`; with F13 held, A gives `NotMatching`) and `sequence_modifier_press_aborts_pending` (`[F13, Ctrl+S]`;
  `VK_LCONTROL` with {LCtrl} held waits; S with {LCtrl, S} held completes).
- Scenario, red on master: `["F13","Ctrl+S"]`: down F13 suppressed, up F13, down LCtrl passed, down S suppressed and
  fires, up S suppressed, up LCtrl passed; `check_nothing_stuck` and `check_press_symmetry` hold.
- Scenario, red on master: `["F13","F13"]` with F13 held, A passes and fires nothing; `["F13","F14"]` with F13 held, A
  passes and re-arms nothing (F14 then passes).
- Scenario: `["Ctrl+X","Ctrl+S"]` with Ctrl released and pressed again between the steps completes; Ctrl pressed while
  X is held starts nothing.
- Scenario (exact steps): `["F13","Ctrl+S"]`, Ctrl+Shift+S after F13 fires nothing and S passes; with
  `allow_extra_modifiers: true` it fires. Loader unit: the flag loads on `sequence` and is an error on `chord`.
- Scenario, red on master (F1): chord `["Shift","Space"]` (window 2000): down LShift, up LShift, `down_unseen` LShift,
  `hidden_input_noted`, down Space (fires), repeat LShift passes, up LShift passes.
- Unit, red on master (`ForegroundForbiddenEnv`, `src/engine/mod.rs:1056`): the key-down of A with a sequence and a
  chord bound under a title `when:` asks nothing of the foreground. Scenario: a pending sequence whose `when:` turns
  false at a key-down neither advances nor aborts.
- Scenario, red on master (known issue 19): two `[F13,F13]` bindings whose `when:` both hold fire the first binding's
  action once, on the second press; two `[F13,F14]` fire the first binding's action; chord `[J,K]` bound twice fires
  once for J, K and not again for a further J inside the window.
- Scenario and property: every existing sequence and chord scenario and both property configs
  (`src/engine/properties.rs:155-185`) stay green, except the deliberate rewrites of item 2; the 10,000-script net of
  p3-3.4-groundwork stays green. The tester re-runs the Sequence and Chord leak-form mutations of item 1.1
  (p1-1.1-repeat/results.md:103).
- Bench (counted apart): R19w p50 <= R19 p50 + 0.1 us on HEAD (rows of p3-3.5-budget-bench; the title read on
  unrelated keys is gone); R1 may move down; R2-R18 and the rest of the R19 family within the CLAUDE.md rules.
  Optional evidence row R20: `Engine::on_key` per event of a sequence start and completion (the dropped `Vec`).
- Manual (owner, `#[ignore = "manual: ..."]` stubs, never counted as passes): M1 in Notepad,
  `( sequence: (steps: ["ScrollLock", "Ctrl+S"]), action: SwitchLanguage )`: ScrollLock then Ctrl+S switches the
  layout, opens no Save dialog, and typing `a` afterwards gives `a`; M2 with `["ScrollLock","ScrollLock"]`, holding
  ScrollLock and typing `x` types `x` and fires nothing; M3 ScrollLock, then Ctrl+Shift+S opens Save As.
- Gate green.

Latency: per non-repeat key-down a `sequence` or `chord` binding first tests whether the pressed key is one of its
trigger keys (a scan of a few `KeySpec`s); its `when:`, `retain` and lookup run only for those keys or while its shape is
pending. Removed on unrelated keys: the live title read of a title condition (`GetWindowTextW` and its `String`; about
0.2 us at the median by 2.5's R10 - R9, p2-2.5-latency-trim/bench/H1.md:20-21; the scope report's 1.7 us is a pre-2.5
figure) and two `to_lowercase` `String`s; one `Vec` per completion. Added: nothing. No Win32 call, allocation, lock,
timer or wake-up is added.

Docs: CHANGELOG `[Unreleased]`: Fixed KS-15, KS-16, F1 of 2.3 (roadmap 3.2), known issue 19; Changed: **Breaking:**
sequence steps match modifiers exactly, `allow_extra_modifiers` accepted on `sequence` (owner decision, roadmap 3.2);
Changed: `when:` after the match for `sequence`/`chord` (roadmap 3.2/3.5). README.md (at 252daef :300-309, :346-349,
:380-421, :641); `src/config/default_config.ron` (:206-209); `book/src/config/kinds.md` (section 8.2 items 2, 5, 6, 10;
8.3; 8.4; 9.2 items 4 and 8; 9.3), `known-issues.md` rows 17-19, `triggers-conditions.md` section 12.4 (:225-226),
`matching.md` section 16.2 (:174, :181), `structure.md:151`, `errors.md` section 18.7; every RON example through
`cfgcheck`. Non-config pages (architecture.md :244-245, :416, :423-429, :440, :603; roadmap 3.2 and 3.5 status lines,
issue rows :894-895, the F1 row :991; audit.md status lines of KS-15/KS-16; gaps.md:165) go to the phase book sync,
listed in impl.md with file:line.
