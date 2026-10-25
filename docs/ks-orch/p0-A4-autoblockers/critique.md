# Architecture critique: p0-A4-autoblockers / Workstream A "exact auto-blockers and dedup" (round 1)
Verdict: APPROVED (Critical 0, Important 0, Minor 5)
Plan: D:/tmp/ks-orch/p0-A4-autoblockers/cut.md   Code read at: master 44f79ac (clean)

No Critical or Important findings. I walked all 11 sections. Every `path:line` in the "Spec re-located" table and in D1-D8 was checked against the tree, and each one matches. I traced every predicted red (L1-L3, T1 on commit 1; T2-T10 and F1-F3 on commit 3) through `Engine::on_key_down` (`src/engine/mod.rs:329-487`), `combo_matches` (`src/engine/matching.rs:58-67`), the stable sort (`mod.rs:252-254`) and the harness (`testing.rs:371-390`, `:331-341`, `:465-484`). Each one fails on the stated step with the stated text, and turns green after commit 3 or 5 as the cut says.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

None.

## Minor (improvements, not blockers)

### M1. `KeyState::all_held` has to build `VIRTUAL_KEY(raw)` inside the engine
Where: cut, D2 and the type sketch (`fn all_held(&self, accept: impl FnMut(VIRTUAL_KEY) -> bool)`); `src/engine/keys.rs:65` (`active: [bool; 256]`, indexed by `usize`); CLAUDE.md coding principle 7.
Problem: the predicate takes a `VIRTUAL_KEY`, and the table is indexed by `usize`. The implementation therefore has to write `VIRTUAL_KEY(index as u16)`. Principle 7 allows that only at the hook boundary and in `vk_name.rs`. Today `src/engine` builds one only in tests (`keys.rs:262-268`).
Consequence: the code-reviewer will flag the construction against principle 7, which costs one fix round. Nothing changes on the desktop: the index comes from the engine's own table and is below 256.
Confidence: CONFIRMED (cut type sketch; `keys.rs:65`; `rg "VIRTUAL_KEY\("` over `src`).
Breaks: house rule (CLAUDE.md §7); no R-code.
What is needed: the cut should either name this as a sanctioned exception and give its bound (index < 256, taken from `KeyState`'s own table), or pick a signature that keeps the raw construction where the rule allows it.

### M2. The `SystemCombination` alias is `'static`, so a stand-in that borrows `keys` does not compile
Where: cut D6 (`pub(crate) type SystemCombination = dyn Fn(&BindAction) -> Option<KeyCombination>;`) and commit 2 (`Scenario::from_ron_with_language_hotkey(text: &str, keys: &[VIRTUAL_KEY])` "builds the same stand-in from `keys`").
Problem: a trait object in a type alias gets the default object lifetime bound `'static`. Rust Reference, "Default trait object lifetime bounds": "If the trait has no lifetime bounds, then the lifetime is inferred in expressions and is `'static` outside of expressions". The same page says the innermost object sets the bound, so `&SystemCombination` means `&(dyn Fn(..) + 'static)`. The loader helper `language_hotkey(keys: &'static [VIRTUAL_KEY])` is fine. The harness constructor takes a non-`'static` `&[VIRTUAL_KEY]`, though, and a closure that captures it by reference will not coerce.
Consequence: commit 2 does not compile until the developer rewrites the stand-in. This is cheap to fix, but it is a compile error inside a declared red commit whose predicted failures are runtime assertions.
Confidence: CONFIRMED by the language rule (https://doc.rust-lang.org/reference/lifetime-elision.html#default-trait-object-lifetime-bounds); not compiled.
Breaks: none (commit hygiene).
What is needed: either say that the stand-in owns its `KeyCombination` (a move closure built from `keys` up front), or give the alias a lifetime parameter.

### M3. No test pins D2's change for a swallowed modifier of another family
Where: cut D2 ("A2's family exactness is kept for passed modifiers"), U1, T9, T10; the second half of F1 (`scenarios.rs:1184-1190`, passed LCtrl).
Problem: D2 widens what the blocker matches for modifiers. Take an extra modifier of another family that the engine holds but Windows does not, for example LCtrl or LWin swallowed by a `Tap`/`DoubleTap`/`Hold` binding (`keys.rs:88-96` keeps it held, KS-14). Master refuses the match through `held.modifiers() == combo.modifiers()` (`matching.rs:66`). D2 now blocks. That is the right behaviour, because Windows sees a bare Alt+Shift, and the C5 CHANGELOG sentence claims it ("A key Windows does not have down (one a binding swallowed ...) does not count"). Only non-modifier keys prove it, though: T10 uses CapsLock and T9 uses L. U1 has "`[LCONTROL, LMENU, LSHIFT]` with LCONTROL down -> false" and no case with LCONTROL held but up in Windows.
Consequence: a later edit that adds the family check back to blockers would pass every listed test. With `( keys: "Win", ..., trigger: Tap(200) )` plus the default config, Win, Alt, Shift would then let Windows switch the language again.
Confidence: CONFIRMED (gap in the test list of cut D7/C4/C5).
Breaks: R8 (the canonical form of an "extra" modifier); test coverage for a stated behaviour.
What is needed: one pin for "swallowed extra modifier -> blocker acts" and one for "passed extra modifier -> blocker passes". A U1 case is enough, or a scenario with a deferred binding on `Ctrl` or `Win`.

### M4. The CHANGELOG overstates what this lane fixes for Alt+Shift+Tab
Where: cut "Docs impact", C5 KS-05 entry: "Shift pressed while Alt and Tab are held (to step back in the Alt+Tab switcher) ... now reach Windows"; manual check M3.
Problem: people usually step back in the switcher by holding Alt, tapping Tab and releasing it, then pressing Shift and Tab. When Shift goes down, Tab is up, so the engine holds only {Alt, Shift}. D2 then matches (`blocker_matches` conditions 1-3 all hold), Shift is swallowed, and Tab reaches Windows as Alt+Tab. The entry names the residue only as "Pressed in the order Alt, Shift, Tab", and a reader will not connect that to the switcher. M3 checks only the order where Tab is still held.
Consequence: the owner reads "to step back in the Alt+Tab switcher", steps back the usual way, and the switcher moves forward. That gets reported as a regression of this lane.
Confidence: CONFIRMED (engine trace: `mod.rs:337-363` plus D2). The Windows half is the standard Shift+Tab behaviour of the switcher.
Breaks: traceability (CHANGELOG accuracy); roadmap.md:100-101 already assigns this residue to 1.2.
What is needed: name the usual switcher order as part of the 1.2 residue in the CHANGELOG, and in the README sentence if that one gives the switcher as an example. Add that order to M3 as an expected-residue step. An optional scenario pin, the Alt+Shift counterpart of T6, would help too.

### M5. A behaviour change sits under Fixed, while the CHANGELOG's own rule asks for **Breaking:**
Where: cut "Docs impact" (C3 KS-18 entry under Fixed); `CHANGELOG.md:7-9` ("a breaking change to the `config.ron` syntax or behaviour ... sit under **Changed** and start with **Breaking:**"); `book/src/roadmap.md:503` (A: "an auto-blocker fires only on an exact match" is "Breaking in behaviour").
Problem: take a config with a flagged requester plus `( keys: "Alt+Shift", ..., when: AppEquals("code.exe") )`. Today Alt+Shift is silenced in every window. After D4 it is silenced nowhere outside code.exe (RK3). The cut documents this, but only inside a Fixed entry. The same goes for the D2 narrowing: Alt+Shift pressed while a passed non-modifier is held now reaches Windows, which may switch the layout (RK6, M3).
Consequence: a user who checks the **Breaking:** entries before upgrading misses that a `when:`-restricted Alt+Shift binding now lets the Windows hotkey through in every other window.
Confidence: CONFIRMED.
Breaks: Keep a Changelog convention as the project defines it.
What is needed: add a short **Breaking:** line under Changed for these consequences, or have the orchestrator record that this lane's changes count as fixes (Q4 is the place).

## Strengths to keep

- **D3 is the right cut.** Once a blocker can only suppress, list order stops mattering for blockers. That closes KS-18 for overlapping bindings too (`LAlt+LShift`, `allow_extra_modifiers`), which dedup by equality alone cannot reach. It also clears R7's "auto-blockers rely on load order" dependency without touching `update_bindings` or the sort.
- **D2 is well founded.** The literal reading ("the engine-held set equals the combo") would be a fail-open regression against master for every non-modifier whose key-up the hook missed: L after Win+L, or Enter released on the UAC secure desktop. The blocker would stay off until that key is typed again. The Win32 premise is quoted correctly. Microsoft Learn's LowLevelKeyboardProc page says "the callback function is called before the asynchronous state of the key is updated" (https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc), and condition 3 never asks about the delivered key, because condition 1 guarantees that the combo accepts it. FakeEnv (`testing.rs:72-84`, `:378-381`) models exactly this. The cut also quotes the GetAsyncKeyState failure list verbatim (https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate).
- **Hot path stays bounded.** The new code path does one bool read per Combo binding. It scans the 256-entry table only after `is_trigger_key` and `keys_down` pass, and makes one `GetAsyncKeyState` per held key outside the hotkey, under the existing `with_engine` lock. It adds no allocation, no lock and no new Win32 API, and the adapter is untouched (`keyboard_hook_callback.rs:150-242`).
- **D6 takes the registry out of the tests.** All new loader and scenario tests are machine-independent, and the red-first predictions stay deterministic.
- **The commit plan is bisectable.** The two red commits are declared, and the F1-F3 flips are listed explicitly instead of discovered. L4 is honestly labelled a pin. Every acceptance line is mapped, and the one place the spec's acceptance cannot be met inside its own bullet 1 (the modifiers-first order) is raised as Q1 rather than hidden.

## Open questions for the architect / orchestrator

1. **Q2 must be settled before commit 4.** T9/T10, the `blocker_matches` signature and the `env.rs` contract all depend on it. My view: D2 should stay. The fallback is a regression against master (see Strengths).
2. **D2 applies "what Windows sees" only to keys outside the hotkey.** The hotkey's other keys still count from the engine table. Two examples: Alt swallowed by the blocker itself in the Shift-first order, or Alt held by a `LAlt` DoubleTap. In both, the blocker still swallows the completing Shift, though Windows cannot see Alt+Shift, and the application loses Shift. This happens on master too and follows the spec's "held set", so it is not a finding here. It belongs to blocker v2 (1.2).
3. **The RK1 text is inaccurate.** If every `GetAsyncKeyState` read fails (a non-elevated debug daemon with an elevated window in front), `sync_modifiers` (`keys.rs:88-96`) clears the passed Alt on every event. The blocker then never matches: it fails open, not "as on master". This is debug-only and already true before this lane; please correct the risk text.
4. **Book pages.** CLAUDE.md ("Docs and the book") says a developer who changes config semantics updates the matching `book/src/config/*.md` pages in the same lane. The cut defers them to the phase book sync (WORKFLOW.md:105-107), as A2 and A3 did. `config/matching.md` §15.3/§15.4 already describe 7cb3521, so the drift exists today. The orchestrator should confirm this exception (Q5). The page list in "Docs impact" is complete as far as I checked (§15.1, §15.3, §15.4, §16.1).

## Absent topics (walked, not applicable)

- Hook install/uninstall, WinEvent hook, message loop and shutdown (sections 2, 8): untouched (`keyboard_hook.rs:108-208` and the adapter are unchanged).
- `SendInput`, sentinel, mask key (section 10): no synthesis added; `Decision::mask` stays reserved for 1.2.
- Timers, gesture state, generation tokens (section 5): blockers arm nothing. Reload mid-press is covered by records (RK8). Foreground change and `swch off`/`on` are untouched (RK9).
- Pipe, DACL, token inheritance (section 9): n/a. Registry access stays read-only and goes through the unchanged `HKeyGuard` path (`registry.rs:83-127`).
- New shared state, atomics, `static mut` (section 8): none. `KeyState` stays inside `Mutex<Engine>`, and the new `Env` calls happen under the existing lock and pump no messages.
- Config schema and RON compatibility (section 7): no new field. The semantic changes are documented in the README, the template and the CHANGELOG (see M5 for the section they go in). Reload strictness (KS-21..24) belongs to Workstream B.
- Module cycles (section 6): `engine` gains no dependency on `hook`, `system` or `core::windows_actions` beyond the existing `BindAction`-as-data. The loader's test helper is kept local, so `config` tests do not import `engine::testing`.
