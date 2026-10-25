# Architecture critique: p1-1.1-repeat / roadmap 1.1 "Per-binding repeat flag" (KS-10) (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 4)
Plan: D:/tmp/ks-orch/p1-1.1-repeat/cut.md   Code read at: master `dbd55ca` (work tree D:/key-switch-rs/key-switch-rs)

I traced the engine path (`src/engine/mod.rs:365-564`, `keys.rs`), the harness (`testing.rs`), the properties
(`properties.rs`), the loader/schema/binding, the hook adapter, the watcher and the template. The design holds:
I found no defect in the code-level cut. The only must-fix is a factual premise in Q1 and in the docs that
rely on it.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. Q1 and the KS-29 docs say roadmap item 2.3 removes the lost-key-up regression. For a swallowed key it does not
Where: cut §"Questions for the orchestrator" Q1 (cut.md:459-462), §Risks "Lost key-up (KS-29)" (cut.md:445),
docs impact: CHANGELOG Fixed (cut.md:350), known-issues new row 63 (cut.md:377); `book/src/roadmap.md:377`;
`src/engine/keys.rs:97-105`, `:134-139`; `src/engine/scenarios.rs:1511-1527`.

Problem: Q1 says "Item 2.3 fixes the root by resyncing all 256 keys on unlock and desktop switch". Roadmap 2.3
says something different: `reset_transient_state()` "resyncs all 256 keys, **except blocked ones**"
(roadmap.md:377). It has to, for the same reason `sync_modifiers` skips swallowed keys (keys.rs:97-105,
KS-14): Windows never saw a swallowed key-down, so `GetAsyncKeyState` reports the key up whether it is held
or not. The engine counts both `Suppressed` and `Forgotten` as swallowed (`key_down_swallowed`,
keys.rs:134-139). The usual trigger key of an `Immediate` hotkey is exactly such a key: CapsLock, or the T
of `Ctrl+Alt+T`. So is the case the cut pins, the replaced test at scenarios.rs:1511-1527 (F13 down
suppressed, `up_unseen`, focus change). As specified, 2.3 repairs only keys whose key-down passed (L after
Win+L). For a swallowed key, no roadmap item repairs the lost key-up.

Consequence: take `( keys: "Pause", action: Launch(exe: "rundll32.exe", args: ["user32.dll,LockWorkStation"]) )`,
or any `Immediate` hotkey whose key is still held when the secure desktop takes input (lock, UAC prompt,
Ctrl+Alt+Del). The Pause key-down is swallowed and the action fires. The session locks while Pause is still
down, so the key-up goes to Winlogon. After unlock the engine still has Pause held, on record as `Forgotten`.
The first Pause press is then read as an auto-repeat (`press` returns true, mod.rs:367) and suppressed
(`is_blocked`, mod.rs:520). With this lane it fires nothing. Its release passes as a lone key-up
(`take_blocked` is false for `Forgotten`, mod.rs:560-562), and only the second press works. On `master` the
first press fired.

This keeps happening after 2.3 merges as written. The owner would accept Q1 believing the regression is
temporary. Meanwhile the published book (row 63, "Open until item 2.3") and the CHANGELOG ("(KS-29, roadmap
item 2.3)") promise a fix the roadmap does not contain.

Aside, PLAUSIBLE, not verified here: in a non-elevated debug build, UIPI hides from the hook the key-up of a
hotkey whose action brings an elevated window to the front while the key is held. That would make every
second press of such a hotkey do nothing.

Confidence: CONFIRMED (roadmap.md:377 wording; keys.rs and mod.rs paths traced; the cut's own replaced test
is the swallowed case).
Breaks: accuracy of an owner escalation (CLAUDE.md: value questions go to the owner); the docs rule (the
config reference must not publish false facts on push to `master`); KS-29 / RC2 context.
What is needed:
- Restate Q1 with two cases:
  - keys whose key-down passed: 2.3's resync repairs them;
  - keys whose key-down was swallowed: 2.3 explicitly excludes them, and no item repairs them today.
- Let the owner decide with those facts.
- Word row 63 and the CHANGELOG Fixed sentence so they do not promise 2.3 for swallowed keys: either "open,
  no roadmap item", or a note that the orchestrator asks to extend 2.3's acceptance to swallowed keys on
  unlock and desktop switch. That extension is a roadmap/owner call, not code in this lane.

## Minor (improvements, not blockers)

### M1. Manual check #5 expects output that a hot reload never prints
Where: cut.md:419; `src/hook/config_watcher.rs:158-169`; `src/core/app.rs:99-105`.
Problem: the check expects that "the listed line ends with `[repeat]`" after a save. A reload prints only
`[INFO] config reloaded (N bindings live)` (config_watcher.rs:163). The binding list is printed only at
startup by `print_welcome` (app.rs:103-105).
Consequence: the human doing manual #5 finds no listed line after the save, so the check reads as failed or
gets skipped. The `[repeat]` marker (D5) is then not checked by hand at all.
Confidence: CONFIRMED.
What is needed: check the marker after a restart, or drop that expectation from #5. D5 is already pinned by
`display_of_a_loaded_binding_marks_a_repeat_flag_that_differs_from_its_action_default`.

### M2. "Repeats like a real key" is wrong for `PressKey` of a toggle key or a modifier
Where: cut.md:324 (README "Auto-repeat policy"), cut.md:318 and :321 (README rows), template comment cut.md:332-340.
Problem: with the owner's default (`PressKey` repeats), every auto-repeat sends a fresh down/up pair
(`press_vk`, windows_actions.rs:163-202). Each pair is a new transition for Windows. A real CapsLock or
NumLock held down toggles once per press (KS-10's own Shift+CapsLock point). A real held Shift is not a
series of taps either (actions.md:167 already says this for `PressKey("LShift")`).
Consequence: hold F13 for 1 s with `( keys: "F13", action: PressKey("CapsLock") )`. CapsLock toggles about
15 times, and the final state depends on the repeat count. That is the KS-10 flicker, but for a `PressKey`
target. The docs as planned say it repeats "like a real key".
Confidence: CONFIRMED (code path); the Windows toggle-once behaviour is as KS-10 states it.
What is needed: do not touch the owner's default. Qualify the README/template sentence, and add a pitfall:
for `PressKey` of a toggle key or a modifier, write `repeat: false` (known-issues §19.9 or actions.md §13).

### M3. One red-first prediction holds only on commit 2, but the tester replays on `master`
Where: cut.md:265 ("The parent is commit 2") and :273; `docs/WORKFLOW.md:69` (replay "on the parent
(`master` in round 1)").
Problem: `repeat_false_on_press_key_fires_once_per_press` loads `repeat: false` in its initial config. On
`master` it fails with ``scenario: config does not load: … Unexpected field named `repeat` …``, not with the
predicted ``repeat F13 at +500 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(37))]``. The other
commit-3 reds put `repeat:` only in a reload that comes after the first failing repeat, or not at all, so
their messages hold on `master` too.
Consequence: during triage, the replay on `master` shows a red with a message other than the predicted one,
which costs a refutation round.
Confidence: CONFIRMED.
What is needed: give the `master`-replay message for that test as well, or state that it must be replayed
on commit 2.

### M4. The book's complete example would opt a `Sequence` macro into repeating
Where: cut.md:387 (example.md:120-129, VS Code `F24` -> `Sequence([ Combo("Ctrl+Shift+F5") ])` + `repeat: true`).
Problem: `Sequence` spawns one uncancellable worker per firing (`spawn_sequence`, data/sequence.rs:111-121;
the `thread::spawn` runs on the hook thread). Roadmap 1.1's own *Why* names that as the harm. In VS Code,
Ctrl+Shift+F5 is "Restart debugging".
Consequence: a user who copies the reference config and holds F24 for 2 s in VS Code gets about 60 macro
threads and about 60 debugger restarts.
Confidence: CONFIRMED for the thread per repeat. The VS Code shortcut meaning is its documented default.
What is needed: either let the VS Code F24 fire once per press and reword the comment, or show `repeat: true`
in the example on an action where repeating is cheap and sensible, for example `Media(Next)`, which the cut
already uses elsewhere.

## Strengths to keep

- **D3 slot rule.** A non-repeating winner still takes the Immediate slot on a repeat, so the "skip" fall-through
  (F13: SwitchLanguage then PressKey(Left) at 30 Hz) cannot happen. It is pinned by
  `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat`, which names the wrong-fix message.
- **Suppression untouched.** The A3 block (mod.rs:502-534) and `on_key_up` stay byte-identical, so "every
  event is suppressed" holds by construction.
- **Hot path.** On a non-repeating repeat there is one `bool` read and one clone and one `Vec` push fewer.
  No Win32 call, lock, thread or atomic is added.
- **Defaults.** The default is resolved once at load (`Option<bool>` plus `Binding::with_kind`), through an
  exhaustive match with no `_` arm (D2). It matches the owner decision exactly: `PressKey`, `Media(VolumeUp)`,
  `Media(VolumeDown)`. Builders keep their two positional arguments (R5).
- **Property argument.** I checked the claim that a harness `repeat` step is always an engine repeat in the
  property (cut.md:286). The generator never calls `down_unseen` or `remapping_*` (properties.rs:146-187),
  and the `FakeEnv` keeps a passed modifier down in the OS, so the resync cannot clear `active` while the
  script holds the key.
- **Red predictions.** I traced the commit-3 reds against the parent engine, which equals `master` (e.g. the
  `+533 ms` red of the off/on scenario through `reset_from_os`, and the `+533 ms` red of the edited
  `:1306-1331` test). They are correct.
- **Docs in-lane.** The `book/src/config` pages are updated in the lane (Q2), and the sha-bearing status lines
  are left to the book sync. That matches CLAUDE.md and WORKFLOW.md:105-107.

## Open questions for the architect

1. **Passed keys and the OS key state.** For a key whose key-down passed, the OS key state can tell a
   lost-key-up press from a real repeat: on a real repeat Windows already has the key down, and after a lost
   key-up it has it up. This is KS-29/2.3 scope and is not asked of this lane. Was it considered when Q1 was
   framed? It shrinks the regression to swallowed keys only, which I1 makes the case the owner must weigh.
2. **Per-event slot after a focus change.** With a `when:`-gated pair on one key (example.md F24: VS Code
   `Sequence`, then `PressKey("F5")`), a focus change mid-hold hands the repeats to the repeating fallback.
   That is the same as `master`, so it is not a regression. It is worth one line in item 1.6's notes: should
   the winner of the key-down own the press's repeats? Check: a scenario with `foreground("Code.exe", ..)`,
   `down F24`, `foreground_changed` to another app, `repeat F24`.

## Absent topics (walked, not applicable)

Hook install/uninstall and thread affinity (§2), the IPC pipe, DACL and single instance (§9), registry
writes (`block_original_combo` is replaced by the stand-in hotkey in tests), new `SendInput` builders (§10),
timer generations and gesture clearing (§5: no timer added; the reload, `swch off`/`on` and focus-change
cases have scenarios), `Mutex<Engine>` scope (§8: `with_engine` unchanged), logging (no new output on the
hot path) and module cycles (§6: no new edge; `data -> core` and `engine -> data` already exist). I walked
each and found no consequence for this cut.
