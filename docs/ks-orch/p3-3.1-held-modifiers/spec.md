# Lane spec: p3-3.1-held-modifiers - Output without the user's modifiers (roadmap item 3.1, part 2 of 2; KS-19)

Branch: fix/held-modifiers

Roadmap item 3.1 (book/src/roadmap.md 'Single input module', :674-684 at 252daef), the modifier half: KS-19, known
issues 30, 35, 68 and 75 (`book/src/config/known-issues.md:74`, :79, :136, :143), the Phase 1 follow-ups on row 66 and
on the mask per event (roadmap :961-963). It builds on p3-3.1-input-module. Binding: lowest key-to-action latency; no key
may ever stick; never prefer a stuck key over a lost one. The owner's answers to plan_draft.md section 5 Q1 (which held
modifiers are released), Q2 (`PressKey("Ctrl+S")`), Q3 (known issue 66) and Q9 (AltGr matching) are relayed in the
brief; items marked with them apply only as answered. Scope report: <orch>/phase-3/scope/3.1-input.md
(sections 2, 4, 5, 8, 9 lane B).

1. The engine adds to `Decision` (`src/engine/mod.rs:391-406`) the sided modifiers Windows has down when the decision's
   actions run, from its own records (`KeyState::alt_or_win_down_in_windows`, `src/engine/keys.rs:304-309`,
   generalised over `ModifierSet::SIDED`), with no OS read in the callback. The event's own key is left out at a
   key-down that passes and put in at the key-up of a passed key (scope T2). Timer decisions (`on_timer`, :576-590)
   fill it too. Computed under the borrow; `carry_out` destructures it (`src/hook/keyboard_hook_callback.rs:308-313`).
   No heap (p3-3.4-zero-alloc measures `on_key` with a counting allocator later).
2. Which of them are released around a key output (owner Q1):
   - (a) every held modifier the payload does not press (the roadmap text; AutoHotkey's `Send` default);
   - (b) for `PressKey`, only the modifiers of the binding's own trigger, as an AutoHotkey remap does (`a::b` sends
     `{Blind}`, and modifiers on the left side but not the right side are released), so extra modifiers that
     `allow_extra_modifiers` admitted stay in effect; macro steps release every held modifier, as (a);
   - (c) (a) plus a per-binding opt-out.
   The module's builder takes `(payload, release set)`: E8 before the release of an Alt or Win, the release, the
   payload, the restore of what it released, E8 after restoring an Alt or Win; one `SendInput`, the decision's mask
   merged into the leading E8. `Media` keys are not wrapped (scope T8: KS-12's verification found an injected media
   key under Alt works and masks Alt), unless the cut argues otherwise. `Combo` presses the modifiers it names and
   never adds held ones (known issue 35, `src/data/sequence.rs:176-212`).
3. `ToggleCapsLock` keeps today's output (no release, no restore) unless the cut shows, with the manual check under
   both 'turn off Caps Lock' settings, that a release is needed. The template's `Shift+CapsLock -> ToggleCapsLock`
   (`src/config/default_config.ron:243-246`) is the owner's daily binding and works at 252daef (owner feedback,
   2026-10-25); an injected Shift restore may turn caps off again under 'Press the SHIFT key' (scope T10, UNVERIFIED).
4. Macro key steps on the executor (`src/core/executor/mod.rs:126-153`, `src/data/sequence.rs:105-117`) take the held
   set from `GetAsyncKeyState` of the 8 sided modifiers right before each step, through a pure function with a key
   reader (a snapshot at submit goes stale after a `Delay`).
5. The restore guard (scope T9, the key risk): a restore of a modifier whose physical key-up the engine has already
   decided must not leave that modifier down in Windows, on every path (callback, drain, executor). The cut chooses the
   mechanism. A second marker next to the sentinel is allowed only if it is derived in `src/core/constants.rs`,
   checked through `is_injected_event`, and the cut states its R2 impact (one constant, one comparison) for the
   reviewer.
6. AltGr's fake left Ctrl: on a `KLLF_ALTGR` layout (`kbdru.dll`, the owner's Russian layout; keys.md section 6.6)
   right Alt arrives with a fake LCtrl (`KBDLLHOOKSTRUCT.scanCode` 0x21D). The release and restore must not leave
   Ctrl or Alt down when the user holds AltGr; the adapter may copy `scanCode` (one field, today it copies four,
   `src/hook/keyboard_hook_callback.rs:425-430`). Matching is unchanged unless Q9 is yes, in which case new feature 2's
   AltGr line joins: 'LCtrl (0x21D) down, RAlt down, Q down does not fire Ctrl+Alt+Q' (roadmap :836), **Breaking**.
7. The scenario harness (`src/engine/testing.rs:829-952`) applies each fired key action's batch to the modelled OS
   before the event itself, as Windows does (`src/hook/keyboard_hook_callback.rs:94-117`).
8. Per-press mask (scope T7, Phase 1 follow-up roadmap :963): the mask follows AutoHotkey's rule, only while Windows has
   seen no other unsuppressed key since that Alt or Win key-down (`src/engine/mod.rs:1011-1019`; flips
   `swallowed_repeats_and_key_up_while_win_is_held_are_masked`, scenarios.rs:5351). The evidence is the count of
   injected events per held press (no bench can run `SendInput`).
9. If Q3 is yes, known issue 66: a `Tap` on a passed lone Alt or Win asks for the mask at its passing key-up; a `Hold`
   on one asks for it when it fires (changes the rule 'a timer never masks', `src/engine/mod.rs:245`,
   `a_hold_timer_never_masks` scenarios.rs:5593, the harness assert at testing.rs:43-44); a `DoubleTap`'s first tap
   stays a real lone tap for Windows and is documented, with `block_default: true` as the workaround.
10. If Q2 is yes: `PressKey("Ctrl+S")`, a combination on the fast path (`src/config/loader.rs:674-677`,
    `src/config/parsing.rs:18-32`), with a `Display` arm (R6).

Acceptance:
- Scenario: the Decision's held/release set (per Q1) is checked for `Alt+J -> PressKey("Left")`; an `LCtrl` `Hold(300)`
  -> Escape at the timer; an `LCtrl` `Tap(200)` -> Escape at the key-up; an `LShift` `DoubleTap` -> ToggleCapsLock at
  the second key-down; the Ctrl+Shift waiting Tap at the C key-down (flips scenarios.rs:4654); a modifier with
  `block_default: true` (not in the set); a modifier found held by the main thread's read (in the set); under Q1 (b), an
  `allow_extra_modifiers` binding with Shift held keeps Shift.
- Harness, over every scenario that fires a key action: each payload key-down meets no modifier down in the modelled
  Windows that the payload does not name and Q1 does not keep; after the batch Windows has the same modifiers down as
  before; `check_nothing_stuck` holds.
- Unit: `(Left, {LAlt})` builds `E8 down, E8 up, LAlt up, Left down (ext), Left up (ext), LAlt down, E8 down, E8 up`
  (refines the roadmap's line, which lacks the leading E8: a timer decision never masks, so without it the release of
  a passing LAlt Hold would be a lone Alt tap).
- Unit: `Combo("Ctrl+S")` with {LCtrl, LAlt} held never has Alt down at the S key-down; `Combo` with Shift held is
  Ctrl+S; `ToggleCapsLock` and `Media` follow items 2 and 3.
- Scenario: a physical LCtrl key-up decided before the daemon's restore of LCtrl leaves LCtrl up in Windows (item 5).
- Scenario: the AltGr fake LCtrl (scan 0x21D) with RAlt held and a `PressKey` fired leaves the modelled Windows with
  the same modifiers after the batch (item 6); with Q9 yes, the feature 2 line above.
- Unit: the executor's held-set function, with a fake key reader.
- Scenario: a held Win with three swallowed repeats asks for one mask (item 8). With Q3 yes: the `LWin` `Tap` and
  `LAlt` `Hold` cases ask for the mask; a `DoubleTap`'s first tap does not.
- Unit (Q2 yes): `PressKey("Ctrl+S")` loads and builds its batch; `PressKey("Ctrl+Bogus")` fails with the loader's text.
- Scenario and property: all engine tests and the 10,000-script net stay green, apart from the named flips.
- Bench (counted apart): R1-R18, the R19 family and R21/R22 within the CLAUDE.md rules; R1, R2 and R3 must not move
  (decisions that fire no key action pay nothing). New rows: R22m, R22 with held {LAlt}; R23, the callback of one
  daemon-injected modifier event (the path the batches multiply).
- Manual (owner, later batch): `Alt+J -> PressKey("Left")` moves the cursor in Notepad and no menu bar opens after Alt
  goes up; `Ctrl+Alt+L -> PressKey("L")` types `l`; `LCtrl` `Hold(300)` -> Escape opens no Start; roadmap Phase 1
  manual check 10 opens no Start (if Q3); the template `Shift+CapsLock` toggles caps under both 'turn off Caps Lock'
  settings (set by hand; agents never write it); Ctrl+J held 2 s -> Down leaves nothing held in `osk.exe`; `RAlt+J`
  with AltGr held on the Russian layout leaves nothing stuck; a `Ctrl+Alt+N` macro with `Key("Enter")`, modifiers
  released mid-macro, leaves nothing stuck; `longest_callback_us` and the over-budget counter after Alt+J held 2 s,
  read before and after the lane.
- Gate green.

Latency: unchanged for events that fire no key action. A key action with k modifiers released: +2k injected events and
up to +4 for masks, in one `SendInput` per decision; Q1 (b) releases fewer than (a). Daemon work per extra event: about
0.1 us on the sentinel path (R23); the OS cost is unmeasured and taken from the manual `longest_callback_us`. No OS read
in the callback (the set comes from the engine's records); about 4.4 us of `GetAsyncKeyState` per macro key step on the
executor (8 x R17a). No allocation, lock, timer or wake-up added on the hook path. The per-press mask removes a
`SendInput` per swallowed repeat under Alt or Win.

Docs: CHANGELOG `[Unreleased]`: Changed, **Breaking:** key output no longer combines with the modifiers that Q1
releases (KS-19, roadmap 3.1), with examples, and `PressKey("Ctrl+S")` if Q2; Changed: the menu mask once per press;
Fixed: known issues 30, 35 and 68 (and 66 if Q3); rewrite the 1.3 entry at CHANGELOG :499-503 ('sends Ctrl+Esc ...').
README (:324-332, :533-535, the `PressKey` and `Combo` rows :367, :612, Known Limitations); `default_config.ron` (:27,
:250-253); `actions.md` section 13.1 (:62) and the `Combo` note (:374-376); `keys.md` section 6.8 (:229);
`triggers-conditions.md` sections 11.6 and 11.8 (:141); `known-issues.md` rows 30, 35, 68, 75 (and 66), plus a row for
each declared residual (AltGr if Q9 is no, the scan code under the daemon thread's layout, CapsLock with the Shift
setting if not closed, the restore race if not closed); `errors.md` if a counter changes. Book-sync list in impl.md with
file:line: roadmap 3.1 status, the Follow-ups rows, 'Known issues until Phases 2 and 3' (:423, :425), Phase 1 manual
check 10's expected result, a 3.1 row in 'Planned changes to the config.ron syntax'; audit KS-19 status and KS-12's
'not covered' sentence (if Q3); architecture.md :449-450, :604, :613. The orchestrator updates CLAUDE.md principle 3.
