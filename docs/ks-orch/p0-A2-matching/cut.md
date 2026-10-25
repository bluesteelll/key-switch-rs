# Cut: p0-A2-matching — Workstream A matching: trigger-key rule (KS-04), sided keys (KS-07), exact modifiers + `allow_extra_modifiers` (owner decision)

## Verdict
CUT READY

Tree: `master` at `e67e5cb` (work tree `D:/key-switch-rs/key-switch-rs`, clean). Branch to cut: `fix/matching`.
Book references are to `7cb3521`; every one is re-located below. The engine (`src/engine`, A1) is merged; KS-04 and KS-07
are **still present** on `e67e5cb` (A1 reproduced the old rules verbatim; `src/engine/mod.rs:116-120` lists both as open).

## Spec re-located

| Spec reference (book @7cb3521) | Current file:line (`e67e5cb`) | Status | Note |
|---|---|---|---|
| KS-04 subset test `key_combination.rs:49-51` | `src/data/key_combination.rs:49-51` (`matches`) | unchanged | callers: `src/engine/mod.rs:307` (combos), `src/engine/sequences.rs:76`, `:96` (steps) |
| KS-04 `test_extra_keys` `key_combination.rs:77-80` | `src/data/key_combination.rs:76-80` | unchanged | asserts superset; removed with `matches` (C2) |
| KS-04 combo loop `keyboard_hook_callback.rs:225` | `src/engine/mod.rs:306-309` | moved | `combo.matches(active.as_slice())`, no trigger-key check |
| KS-04 deferred arms on `vk_code` `:240/:245/:250` | `src/engine/mod.rs:324-329` (Tap), `:336-341` (Hold), `:350-355` (DoubleTap) | moved | armed under the event `vk`; defect: they arm for any key |
| KS-04 `immediate_fired` `:230`, `block_default` `:234-236` | `src/engine/mod.rs:311-319` | moved | unchanged by this lane (KS-13 is 1.6) |
| KS-04 blocked bit `:294-297`, key-up `:324` | `src/engine/mod.rs:409-411`, `:437` | moved | unchanged |
| KS-04 held set keeps CapsLock `:192` | `src/engine/keys.rs:115-120` (`press`), called `src/engine/mod.rs:291` | moved | unchanged |
| KS-07 names `vk_name.rs:134-143` | `src/data/vk_name.rs:133-144` | unchanged | `"win"|"super"|"lwin"|"leftwin"` all -> `VK_LWIN` at `:143` |
| KS-07 `parse_combo` `parsing.rs:28` | `src/config/parsing.rs:13-31` (`parse_vk` at `:25-26`) | moved (lines) | keeps sided codes |
| KS-07 side folding `keyboard_hook_callback.rs:335-341` | `src/engine/keys.rs:62-73` (`normalize`), `:129-140` (`active_keys`), buffer `:26-60` | moved | RWin folds to LWin |
| KS-07 chord raw vk `keyboard_hook_callback.rs:276`, `chord_state.rs:70-74` | `src/engine/mod.rs:391-394`, `src/engine/chords.rs:66-73` | moved | raw sided compare; doc admits it at `src/engine/chords.rs:16-18` |
| KS-07 chord parse `loader.rs:298` | `src/config/loader.rs:287-304` (dup check `:295-297`) | moved | `parse_vk`, VK-equality dedup |
| KS-07 example `default_config.ron:201-205` | `src/config/default_config.ron:200-205` | unchanged | `keys: "LShift"` + `DoubleTap(250)` |
| KS-07 README `:173`, `:181-183`, `:262` | `README.md:179`, `:187-189`, `:268` | moved | "the hook normalises left/right" is false today |
| roadmap A trigger-key rule `roadmap.md:79`; "Shift either side, LShift left" `:82`; acceptance `:98` | same lines | unchanged | brief adds exact modifiers (owner decision; roadmap Q2 `:578`, 1.6 `:205-213`, syntax table `:503`, `:507`) |
| Sequence step matching (KS-15/16 context) | `src/engine/sequences.rs:55-116`, `last_step_keys` `:119-124` | moved | sided semantics only in this lane (D6) |
| Engine rule docs | `src/engine/mod.rs:88-120` | — | rules 3-4 and the defect list change |
| Hook-adapter tests relying on "F13 matches whatever modifiers are held" | `src/hook/keyboard_hook_callback.rs:271-276`, `:290-293`, `:318-321`, `:345-348` | — | machine-state dependent under exact matching (D10) |
| Registry Win combos | `src/system/registry.rs:74-75`, `:199`; INFO print `:55-58` | — | `VK_LWIN` meant "either Win" under folding |
| Scenario harness key names | `src/engine/testing.rs:86-97` (`physical_key`), `FakeEnv::os_key_down` `:68-75` | unchanged | already sided; no harness change |

## Goal and non-goals

**After the lane**
- A `keys` binding fires (Immediate) or arms (Tap/Hold/DoubleTap) only on the key-down of its **trigger key**: one of its
  non-modifier keys, or for a binding made only of modifiers, any of its keys (D2). Deferred gestures stay keyed on the event's
  own sided code, which is the code its key-up carries.
- Modifiers match **exactly** at family level (Shift, Ctrl, Alt, Win): held families == the binding's families (D3).
  `allow_extra_modifiers: true` (per binding, `keys` only, default `false`) restores superset matching for modifiers.
  Non-modifier keys of the combo must still be held; extra non-modifier keys never block a match.
- `Shift`/`Ctrl`/`Alt`/`Win`(`Super`) mean either side; `LShift`/`RShift`/`LCtrl`/`RCtrl`/`LAlt`/`RAlt`/`LWin`/`RWin` (and
  `Left…`/`Right…`) mean that side only, in `keys`, `sequence` steps and `chord` keys alike (D1, D5, D6).
- README, `default_config.ron` comments and CHANGELOG describe the new meaning in the commit that introduces it.

**Non-goals (later items)**
- Sequence trigger-key rule and modifier-tolerant waiting (KS-15, KS-16), chord/sequence blocked-bit hygiene (KS-30, rest of KS-32): roadmap **3.2**.
- One winner per event / `block_default` across matches (KS-13): **1.6**. `block_default` for deferred triggers (usability of the LShift example): **1.3**.
- Repeat symmetry and `sync_modifiers` vs suppressed modifiers (KS-06, KS-14), per-binding `repeat` (KS-10): their own lanes / **1.1**.
- Mask key and auto-blocker v2 (KS-12, rest of KS-05): **1.2**. Auto-blocker dedup (KS-18): not here.
- AltGr fake-LCtrl detection (`scanCode 0x21D`): new feature 2. No change to `LLKHF_*`, `scanCode` or extended-key handling.
- Book pages (pinned to `7cb3521`): phase book sync (Q2).

## Decisions

### D1: `KeySpec`, one canonical trigger-key representation fixed at load time
- **What.** New `src/data/key_spec.rs`:
  - `pub enum Modifier { Shift, Ctrl, Alt, Win }` (`Clone, Copy, Debug, PartialEq, Eq`), with `pub const ALL: [Modifier; 4]`,
    `left()/right() -> VIRTUAL_KEY` (`VK_LSHIFT/VK_RSHIFT`, `VK_LCONTROL/VK_RCONTROL`, `VK_LMENU/VK_RMENU`, `VK_LWIN/VK_RWIN`),
    `generic() -> Option<VIRTUAL_KEY>` (`VK_SHIFT`, `VK_CONTROL`, `VK_MENU`; `None` for Win), `of(vk) -> Option<Modifier>`
    (sided and generic codes), `name() -> &'static str`, and `Display`.
  - `pub enum KeySpec { AnySide(Modifier), Key(VIRTUAL_KEY) }` (`Clone, Copy, Debug, PartialEq, Eq`):
    - `accepts(self, vk) -> bool`: `AnySide(m)` accepts `m.left()`, `m.right()` and `m.generic()`; `Key(k)` accepts `k` only.
    - `modifier(self) -> Option<Modifier>`: `AnySide(m)` -> `Some(m)`; `Key(vk)` -> `Modifier::of(vk)`.
    - `overlaps(self, other) -> bool`: some physical key is accepted by both (`Shift`~`LShift`, `Win`~`RWin`, `A`~`A`).
    - `parse(name: &str) -> Option<KeySpec>`: normalised `"win"`/`"super"` -> `AnySide(Win)`; otherwise `parse_vk(name).map(KeySpec::from)`.
    - `impl From<VIRTUAL_KEY>`: `VK_SHIFT`/`VK_CONTROL`/`VK_MENU` -> `AnySide`; every other code -> `Key(code)`. So `VK_LWIN`
      is the **left** key; "either Win" has no VK code and is written `KeySpec::AnySide(Modifier::Win)` (doc comment says so).
    - `Display`: `AnySide(m)` -> `m.name()`; `Key(VK_LWIN)` -> `"LWin"`; other `Key(vk)` -> `vk_name(vk)`.
  - `src/data/vk_name.rs`: extract the whitespace-strip + lowercase of `parse_vk` (`:96-100`) into
    `pub(crate) fn normalize_key_name(name: &str) -> String`; `parse_vk` and `KeySpec::parse` use it. `vk_name()` output is
    unchanged (`VK_LWIN` still renders `"Win"` for output keys, `:31`).
  - `KeyCombination` (`src/data/key_combination.rs:3-6`): `pub keys: Vec<KeySpec>`. Constructors keep their shapes, generic over
    the key: `new(key: impl Into<KeySpec>)`, `with(self, key: impl Into<KeySpec>)`,
    `from_keys<K: Into<KeySpec>>(keys: impl IntoIterator<Item = K>)` (dedup on `KeySpec` equality, order kept). Set-equality
    `PartialEq` unchanged. `Display` joins key names with `+`. `matches` and its three tests (`:59-80`) are removed: matching moves
    to the engine. C6 adds `modifiers(&self) -> ModifierSet`.
  - Trigger keys (combo keys, sequence-step keys, chord keys) are `KeySpec`. Output keys (`PressKey`, `Key`, `Combo` macro steps)
    stay `VIRTUAL_KEY` (D7).
- **Why.** RC4 "canonicalise in one place" (`book/src/audit.md:70-74`). `VIRTUAL_KEY` cannot express "either Win" vs "left Win":
  both parse to `VK_LWIN` (`src/data/vk_name.rs:143`). A distinct type keeps a trigger-only notion away from `SendInput`. Roadmap
  3.4 already names `KeySpec` (`book/src/roadmap.md:367`). R8 allows "move normalisation to parse time consistently".
- **Rejected.** (a) Put sided + unsided codes into the held set (audit suggestion, `book/src/audit.md:294`): still cannot tell
  `Win` from `LWin`. (b) A pseudo-VK (>= 0x100) for any-Win inside `VIRTUAL_KEY`: `parse_combo` also feeds macro `Combo` steps
  (`src/config/loader.rs:427-433`), so the code could reach `SendInput`; breaks principle 7 (raw `VIRTUAL_KEY` only at the hook or
  in `vk_name.rs`). (c) A side bitmask beside `Vec<VIRTUAL_KEY>`: macro `Combo("LShift+A")` would lose its side; harder `Eq`.
  (d) Reject sided names at load (audit option): contradicts the owner spec.
- **Cost.** The type change touches about 12 test assertions (touch set) and `registry.rs`.

### D2: Trigger-key rule (KS-04)
- **What.** `engine::matching::is_trigger_key(combo, pressed: VIRTUAL_KEY) -> bool`: some key `k` of the combo with
  `k.accepts(pressed)` and (`k.modifier().is_none()` or every key of the combo is a modifier). A Combo binding of any trigger is
  considered only if this holds. Deferred gestures keep arming under the event's `vk` (`src/engine/mod.rs:324-355`), which is now
  always a key of the combo; its key-up carries the same sided code, so resolution is unchanged.
- **Why.** The brief's "fires or arms ONLY when the key just pressed belongs to its combo" is a necessary condition; limiting the
  trigger to the non-modifier keys also closes the second KS-04 symptom, "Pressing Shift while CapsLock is still held fires
  `Shift+CapsLock`: CapsLock turns on and Shift is eaten" (`book/src/audit.md:204`), which "belongs to the combo" leaves open, and
  it is the audit's own fix ("ideally its last non-modifier key", `:214`). It also stops the engine from suppressing modifier
  key-downs for combos that have a payload key (fewer KS-05/KS-12/KS-14 interactions). Modifier-only combos (`"Alt+Shift"`
  auto-blocker, `"LShift"` DoubleTap) need any key as trigger or they could never fire.
- **Rejected.** Literal "any key of the combo": leaves the Shift-while-CapsLock half of KS-04; fallback in Q1 (one predicate).
  "The last key pressed must be the last listed key": the written order is documented as irrelevant (`book/src/config/keys.md:39`).
- **Cost.** "Hold L, then press Ctrl" no longer fires `Ctrl+L` (CHANGELOG says so).

### D3: Exact modifiers at family level + `allow_extra_modifiers`
- **What.** `combo_matches(combo, pressed, held, allow_extra_modifiers)` = `is_trigger_key` && `keys_down(combo, held)` &&
  (`allow_extra_modifiers` || `held.modifiers() == combo.modifiers()`). `ModifierSet` (in `key_spec.rs`): `Clone, Copy, Debug,
  Default, PartialEq, Eq`, one bit per `Modifier`, `EMPTY`, `with(self, m) -> Self`, `contains(self, m) -> bool`. A family is
  held if its left or right slot, or its generic slot (`VK_SHIFT/VK_CONTROL/VK_MENU`, written by `sync_modifiers`,
  `src/engine/keys.rs:104-111`), is set. Side requirements are per key (`keys_down`): `"LShift+A"` needs the left Shift; with both
  Shifts held it still matches (one family). `RawBinding.allow_extra_modifiers: bool` (`#[serde(default)]`),
  `Binding.allow_extra_modifiers: bool` (`false` in `with_kind` and `new_auto_blocker`), builder `with_allow_extra_modifiers(self, bool)`.
  The loader rejects `allow_extra_modifiers: true` on `sequence`/`chord`, checked after the deferred-trigger rule
  (`src/config/loader.rs:220-226`): `` `allow_extra_modifiers` applies only to `keys` bindings ``. Auto-blockers stay exact.
- **Why.** Owner decision (CLAUDE.md "Owner decisions"); brief: "the held modifier set (Ctrl/Shift/Alt/Win, side-normalised)
  equals the combo's modifiers". Rejecting the flag where it has no effect beats a silently ignored field; relaxing later (3.2) is
  non-breaking, while giving meaning later to an accepted-and-ignored field would be a silent change.
- **Rejected.** Per-key exactness (an extra RShift breaks `"LShift+A"`): contradicts "side-normalised" families. Exactness on
  sequence steps now: changes semantics that 3.2 rewrites anyway (Q3).
- **Cost.** Breaking for configs relying on superset matching (default config: Ctrl/Alt/Win+CapsLock now reach Windows).

### D4: Held-key queries replace the normalised `ActiveKeys` list
- **What.** New `src/engine/matching.rs` with `pub(super) trait HeldKeys { fn is_down(&self, key: KeySpec) -> bool; }` (C6 adds
  the default method `fn modifiers(&self) -> ModifierSet`: `Modifier::ALL` filtered by `is_down(AnySide(m))`),
  `pub(super) fn keys_down<H: HeldKeys + ?Sized>(combo: &KeyCombination, held: &H) -> bool` (every key down on its side), and
  later `is_trigger_key`, `combo_matches`. `impl HeldKeys for KeyState` in `keys.rs` (`Key(vk)` -> `active[vk]` via `slot`, bounded
  `< 256`; `AnySide(m)` -> left || right || generic). `#[cfg(test)]` impls for `[VIRTUAL_KEY]` and `[VIRTUAL_KEY; N]` (the same
  rules over a list of held codes) let unit tests pass key lists. Remove `ActiveKeys`, `ACTIVE_KEYS_CAP`, `normalize`,
  `active_keys` (`src/engine/keys.rs:26-73`, `:129-140`) and their three tests (`:172-202`); `on_key_down` drops
  `let active = …` (`src/engine/mod.rs:292`).
- **Why.** Matching needs sides; a folded list loses them. Direct table lookups: no 256-slot scan, no 16-key cap, no allocation.
- **Rejected.** Keep `ActiveKeys` with sided + generic entries: two representations of one fact, and the cap stays.
- **Cost.** `Sequences::on_key_down` takes `held: &H` (`H: HeldKeys + ?Sized`) instead of `&[VIRTUAL_KEY]`.

### D5: Chords on `KeySpec`
- **What.** `BindingKind::Chord { keys: Vec<KeySpec>, window }`. `Chords::on_key_down(vk, action, chord_keys: &[KeySpec], window, now)`:
  `NotInChord` unless some chord key `accepts(vk)`; a pending entry is found by set-equality of `chord_keys` (no per-event
  `canonical` Vec; today's allocation at `src/engine/chords.rs:72-73` goes); `PendingChord { chord_keys: Vec<KeySpec>, seen: Vec<VIRTUAL_KEY>, action, expires_at }`,
  `seen` = sided codes pressed (press order, no duplicates); complete when every chord key accepts some `seen` code;
  `Completed { keys: seen codes as u16 }` (each of those key-downs was suppressed as `Pending`, so marking them keeps the
  suppression invariant). Loader: chord keys via `KeySpec::parse`; any two keys that `overlap` give
  `Chord key #<i>: duplicate "<name>" (overlaps key #<j>)` (keeps "duplicate" for `src/config/loader.rs:1212-1229`).
- **Why.** Brief: "the chord path compares normalised keys, so 'Shift' in a chord works". Overlap must be rejected: with
  `["Win","RWin"]` one RWin press would satisfy both keys and fire.
- **Rejected.** Sorting chord keys at load: changes the banner's key order for no gain.
- **Cost.** Breaking: `["Shift","LShift"]`, `["Win","RWin"]` used to load; now a load error (CHANGELOG).

### D6: Sequences get sided semantics only
- **What.** Steps are `KeyCombination` of `KeySpec`; a step matches when `keys_down(step, held)` (subset, as today; no trigger-key
  or modifier rule: KS-15/16 are 3.2). `last_step_keys` (`src/engine/sequences.rs:119-124`) returns only the codes of the last
  step's **non-modifier** `Key` specs.
- **Why.** Brief: sided names apply to sequence steps. A modifier in a step is never suppressed by the sequence unless it is the
  event key, which the event path (`src/engine/mod.rs:409-411`) already marks; marking it from `last_step_keys` swallows a key-up
  whose key-down reached the OS (today for `Win`, which marks `VK_LWIN`; with sided names it would hit `LShift` too). The generic
  codes 0x10-0x12 marked today never match a physical key-up.
- **Rejected.** Map `AnySide(Win)` back to `VK_LWIN` "to keep behaviour": keeps a stuck-Win path. Exact modifiers on steps: Q3.
- **Cost.** Behaviour change for `Win` in a last step (CHANGELOG Fixed, KS-32 partly).

### D7: Output keys unchanged
- `PressKey` and `Key` keep `parse_vk` (`src/config/loader.rs:351-354`, `:422-425`). Macro `Combo` uses the new
  `parsing::parse_key_list(s) -> Result<Vec<VIRTUAL_KEY>, String>` (today's split/parse/dedup, same error texts) instead of
  `parse_combo(..).keys` (`src/config/loader.rs:427-433`). `"Win"` still sends `VK_LWIN`, `"Shift"` still `VK_SHIFT`.
  `parse_combo` and `parse_key_list` share one private segment splitter (errors as `src/config/parsing.rs:14-27`).

### D8: Display and console
- `Binding` Display (`src/data/binding.rs:140-169`) appends `  [allow_extra_modifiers]` when set (C6); `format_kind`
  (`:171-197`) renders keys through `KeySpec` Display (separators unchanged). The registry INFO line (`src/system/registry.rs:55-58`)
  prints `{combo}` (`Alt+Shift`) instead of `{:?}` of key codes (C2; `{:?}` would now print `[AnySide(Alt), …]`).

### D9: Registry Win combos are "either Win"
- `src/system/registry.rs:74-75` and `:199` build `KeySpec::AnySide(Modifier::Win)` + key: "either Win" under folding, kept.
  Language hotkeys (`:183-191`) keep `VK_MENU/VK_CONTROL/VK_SHIFT` -> `AnySide` via `From`. Reads only.

### D10: Hook-adapter tests opt out of exactness
- The three tests in `src/hook/keyboard_hook_callback.rs` (`:290-293`, `:318-321`, `:345-348`) run `HookEnv`, which reads the live
  `GetAsyncKeyState`. Under exact matching a modifier the owner holds during `cargo test` would stop F13 from matching (flake).
  They get `.with_allow_extra_modifiers(true)`; the comment at `:271-276` says why. No production change in `src/hook`.

### D11: Commit staging
- Three red/fix pairs in dependency order (sided -> trigger key -> exact), each red commit compile-clean (assertion reds only),
  each fix commit carrying its README / `default_config.ron` / CHANGELOG edits. Every scenario is checked against the semantics of
  the commit where it must be green (status table under Commits).

### D12: Book pages
- Not edited in this lane; exact list for the phase book sync in Docs impact (Q2).

## Win32 API choices

No new Win32 call, feature, hook, window, timer or COM apartment. The design relies on these existing facts:

| Purpose | `windows` path | Feature (present?) | Thread | Errors | Doc |
|---|---|---|---|---|---|
| Modifier resync, unchanged (`HookEnv::os_key_down`, `src/hook/keyboard_hook_callback.rs:116-122`) | `windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState` | `Win32_UI_Input_KeyboardAndMouse` (yes, `Cargo.toml:44`) | hook thread (main), inside `with_engine` | returns 0 on failure ("The current desktop is not the active desktop", UIPI): reads as up | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate : "You can use the virtual-key code constants VK_SHIFT, VK_CONTROL, and VK_MENU … without distinguishing between left and right"; sided constants "are only available when you call … GetAsyncKeyState …" |
| Event code range (bounds for `slot`) | `KBDLLHOOKSTRUCT::vkCode` | same | hook thread | — | https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct : "The code must be a value in the range 1 to 254." |
| Key constants | `VK_LSHIFT … VK_RWIN`, `VK_SHIFT/VK_CONTROL/VK_MENU` | same | any | — | same pages |

- Physical modifiers arrive sided (`VK_LSHIFT` 0xA0 …) in `WH_KEYBOARD_LL`: empirical, confirmed by five KS-07 verifiers and a test
  program (`book/src/audit.md:296`); not stated on Learn. A **generic** code (`VK_SHIFT`) arriving from third-party `SendInput` is
  **(unverified)**; handled defensively: `AnySide` accepts the generic code and the generic slot counts for its family.
- `LLKHF_INJECTED`, `scanCode`, extended flag: not read, unchanged. AltGr layouts deliver a fake LCtrl + RAlt; with exact matching
  `Alt+X`/`RAlt+X` no longer fire on AltGr+X and `Ctrl+RAlt+X` does (README note; detection is new feature 2).
- Elevation/UIPI/pipe/token: unchanged. Under UIPI or the secure desktop `GetAsyncKeyState` returns 0, so exact matching sees no
  OS-held modifiers there, as superset matching did.

## Threading and ownership

| Code path | Thread | Shared state | Note |
|---|---|---|---|
| `KeySpec::parse`, `parse_combo`, `parse_key_list`, loader validation | main (startup), `config-watcher` (reload) | none (returns `Vec<Binding>`) | pure; the registry read for auto-blockers is unchanged and read-only |
| `combo_matches`, `keys_down`, `is_trigger_key`, `HeldKeys for KeyState` | hook thread (main), inside `KeyboardHook::with_engine` | `Engine` behind the existing `Mutex` | no allocation, lock or Win32; O(keys) per binding |
| `Chords::on_key_down`, `Sequences::on_key_down` | same | same | chord path allocates only on arm / `seen.push`, as today |
| `Engine::on_timer` | `hold-timer` threads | same | untouched (no matching on timers) |
| `Engine::on_foreground_changed` (WinEvent callback) | main | same | untouched |
| Binding publish | `update_bindings` (main / `config-watcher`) | one `ArcSwap` store after a stable sort (`src/hook/keyboard_hook.rs:94-99`) | unchanged |

No new atomics, no `static mut`, no new `Mutex`.

## Config / RON changes and compatibility

| Change | RON | Compatibility |
|---|---|---|
| `allow_extra_modifiers` | `( keys: "CapsLock", action: SwitchLanguage, allow_extra_modifiers: true )`; `bool`, default `false`; `keys` only | new optional field; on `sequence`/`chord` a load error. Older binaries ignore it (no `deny_unknown_fields` until Workstream B, `src/config/schema.rs:26`); B's `deny_unknown_fields` accepts it because it is a `RawBinding` field |
| Exact modifiers | — | **Breaking** in behaviour (CHANGELOG Changed) |
| Sided names | `LWin`/`LeftWin` now left only; `Win`/`Super` either side, also in `chord` | **Breaking** in behaviour |
| Overlapping chord keys | `["Shift","LShift"]`, `["Win","RWin"]`, `["Ctrl","RCtrl"]` | **Breaking**: load error |
| Trigger-key rule | — | behaviour (Fixed KS-04, described explicitly) |

## Touch set

| File | Change | Commit | Why |
|---|---|---|---|
| `src/data/key_spec.rs` (new) | `Modifier`, `KeySpec` (parse/accepts/modifier/overlaps/From/Display), unit tests; C6 adds `ModifierSet` | C2, C6 | D1, D3 |
| `src/data/mod.rs` | `pub mod key_spec;` | C2 | register |
| `src/data/vk_name.rs` | extract `normalize_key_name` (`:96-100`); `parse_vk` uses it | C2 | D1 |
| `src/data/key_combination.rs` | `keys: Vec<KeySpec>`, generic ctors, `Display`, remove `matches` + tests `:59-80`, new tests; C6 `modifiers()` | C2, C6 | D1, D3 |
| `src/data/binding.rs` | `Chord.keys: Vec<KeySpec>` (`:30-33`), `format_kind` (`:171-197`); C6 field, `with_allow_extra_modifiers`, Display suffix, `#[cfg(test)] mod tests` | C2, C6 | D5, D3, D8 |
| `src/config/parsing.rs` | `parse_combo` -> `KeySpec`, new `parse_key_list`, shared splitter; tests `:65-96` updated + new | C2 | D1, D7 |
| `src/config/schema.rs` | `RawBinding.allow_extra_modifiers` (`#[serde(default)]`, doc comment) | C6 | D3 |
| `src/config/loader.rs` | chord `KeySpec` + overlap error (`:287-304`); macro `Combo` via `parse_key_list` (`:427-433`); C6 map + reject flag (`:206-235`); tests: red-firsts (C1, C5), assertions `:487`, `:1081-1082`, `:1103-1106`, `:1127-1128`, new tests | C1, C2, C5, C6 | D3, D5, D7 |
| `src/config/default_config.ron` | combo-syntax block `:27-44` (sided names C2, trigger rule C4, exact + flag C6), flags block `:100-110` (C6), LShift example `:200-205` (warning, C2) | C2, C4, C6 | config meaning |
| `src/system/registry.rs` | Win combos `AnySide(Win)` (`:74-75`, `:199`); INFO `{combo}` (`:55-58`); tests `:259-287` | C2 | D9, D8 |
| `src/engine/matching.rs` (new) | `HeldKeys`, `keys_down` (C2); `is_trigger_key`, `combo_matches` (C4); modifier rule, flag, `HeldKeys::modifiers` (C6); unit tests | C2, C4, C6 | D2-D4 |
| `src/engine/mod.rs` | `mod matching;`; Combo branch (`:306-309`), Sequence call (`:366-372`), drop `active` (`:292`); docs `:88-120` at each step | C2, C4, C6 | D2-D4 |
| `src/engine/keys.rs` | remove `ActiveKeys`/`normalize`/`active_keys` (`:26-73`, `:129-140`, tests `:172-202`); `impl HeldKeys for KeyState`; module doc; new tests | C2 | D4 |
| `src/engine/chords.rs` | `KeySpec` keys, set-equality, `seen` codes; doc `:16-18`; tests updated + new | C2 | D5 |
| `src/engine/sequences.rs` | `HeldKeys` param, `keys_down`, `last_step_keys` non-modifier only; doc `:5-9`; `ctrl_x_ctrl_s_sequence` (`:224-225`) updated + new test | C2 | D6 |
| `src/engine/scenarios.rs` | new sections "Sided keys (KS-07)", "Trigger key (KS-04)", "Exact modifiers" | C1, C3, C5 | red-first + guards |
| `src/hook/keyboard_hook.rs` | tests only: `:291-295` (compare `KeySpec`), `:313` | C2 | type change |
| `src/hook/keyboard_hook_callback.rs` | tests only: `.with_allow_extra_modifiers(true)` at `:290`, `:319`, `:345`; comment `:271-276` | C6 | D10 |
| `README.md` | tree `:38-51` (+`key_spec.rs`, +`matching.rs`, `keys.rs` text), module `:65-70`, combo syntax `:172-189`, kinds table `:217`, chord restriction `:248`, triggers `:262-269`/`:280-282`, flags table `:357-363`, features `:520`, `:528` | C2, C4, C6 | config meaning |
| `CHANGELOG.md` | Unreleased entries (texts in Docs impact) | C2, C4, C6 | Keep a Changelog |
| `src/engine/testing.rs` | none | — | names already sided (`:86-97`) |
| `book/src/**` | none (Q2); list in Docs impact | — | doc-writer / book sync |

## Commits, in order

Subjects follow `<Area>: <what>`; red-commit bodies quote the failing line; every message ends with the Co-Authored-By trailer.
Gates per commit for the developer: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, and the tests the
commit touches (`cargo test --locked engine::`, `config::`, `data::`, `system::`, `hook::`); the tester owns the full suite. Red
commits compile and fail **only** the listed tests, by assertion. Scenario messages follow the harness format
(`src/engine/testing.rs:325`, `:362-367`, `:286-289`); the panic text starts `scenario: <label>: …`.

**C1. `Engine: red-first: sided keys in keys, sequences and chords (KS-07)`** (declared red). `src/engine/scenarios.rs` (new section)
and one loader test; all compile on `e67e5cb`.
- T1 `lshift_in_keys_fires_only_for_the_left_shift`: `(keys: "LShift+F13", action: SwitchLanguage)`. down LShift (passed); down F13
  (suppressed, `[SwitchLanguage]`); up F13 (suppressed); up LShift (passed); down RShift (passed); down F13 (passed, nothing);
  up F13 (passed); up RShift; total `[SwitchLanguage]`.
  PREDICTED RED: `down F13 at +0 ms: expected the event to be suppressed, it was passed`.
- T2 `rwin_and_lwin_in_keys_fire_only_for_their_side`: `(keys: "RWin+F13", action: SwitchLanguage)`, `(keys: "LWin+F14", action: ToggleCapsLock)`.
  RWin+F13 fires `[SwitchLanguage]` (suppressed); LWin+F13 passes, nothing; LWin+F14 fires `[ToggleCapsLock]`; RWin+F14 passes,
  nothing (keys released between pairs).
  PREDICTED RED: `down F13 at +0 ms: expected the event to be suppressed, it was passed`.
- T3 guard `unsided_shift_and_win_in_keys_fire_for_either_side`: `Shift+F13` fires with LShift and with RShift; `Win+F14` fires with
  LWin and with RWin. GREEN on master (folding); must stay green.
- T4 `lshift_double_tap_fires_only_for_the_left_shift`: `(keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250))`. down
  LShift (suppressed, nothing); up (suppressed); +100; down LShift (suppressed, `[ToggleCapsLock]`); up; +500; RShift down, up,
  +100, down: passed, nothing; total `[ToggleCapsLock]`.
  PREDICTED RED: `down LShift at +0 ms: expected the event to be suppressed, it was passed`.
- T5 `unsided_shift_in_a_chord_fires_for_either_side`: `(chord: (keys: ["Shift", "Space"], window_ms: 50), action: SwitchLanguage)`.
  down LShift (suppressed); +20 down Space (suppressed, `[SwitchLanguage]`); up Space, up LShift (suppressed); +100; RShift, +20,
  Space (fires again); total two.
  PREDICTED RED: `down LShift at +0 ms: expected the event to be suppressed, it was passed`.
- T6 guard `sided_shift_in_a_chord_is_only_its_side`: chord `["LShift", "F13"]`: down RShift passed; down F13 suppressed, nothing.
  GREEN on master.
- T7 `sided_shift_in_a_sequence_step_matches_only_its_side`: `(sequence: (steps: ["F13", "LShift+F14"]), action: SwitchLanguage)`.
  Run 1: down LShift (passed); F13 down/up (suppressed); down F14 (suppressed, `[SwitchLanguage]`); up F14 (suppressed); up LShift
  (passed). Run 2 (fresh scenario): RShift held; F13 down/up (suppressed); down F14 (passed, nothing).
  PREDICTED RED (run 1): `down F14 at +0 ms: expected the event to be suppressed, it was passed`.
- T8 `win_in_the_last_sequence_step_does_not_swallow_its_key_up`: `(sequence: (steps: ["F13", "Win+F14"]), action: SwitchLanguage)`.
  down Win (passed); F13 down/up (suppressed); down F14 (suppressed, fired); up F14 (suppressed); up Win -> **passed**.
  PREDICTED RED: `up Win at +0 ms: expected the event to be passed, it was suppressed` (`last_step_keys` marks `VK_LWIN`).
- T9 `config::loader::tests::chord_keys_that_overlap_are_rejected`: for `["Shift", "LShift"]`, `["Win", "RWin"]`, `["Ctrl", "RCtrl"]`:
  `from_ron_str(..).expect_err("overlapping chord keys must be rejected")`, `ConfigError::Bindings(errs)`, `errs[0].contains("duplicate")`.
  PREDICTED RED: panic `overlapping chord keys must be rejected: [Binding { kind: Chord { keys: [VIRTUAL_KEY(16), VIRTUAL_KEY(160)]` …

**C2. `Matching: sided and unsided key specs in every trigger (KS-07)`**: makes T1, T2, T4, T5, T7, T8, T9 green; T3, T6 stay green.
D1, D4 (`HeldKeys::is_down`, `keys_down`), D5, D6, D7, D8 (registry line, `format_kind`), D9. At this commit combos still match as a
superset without the trigger-key rule (`keys_down` alone in `src/engine/mod.rs:306-309`). New unit tests:
- `data::key_spec::tests`: `unsided_names_parse_to_either_side` (Shift, Ctrl, Control, Alt, Menu, Win, Super),
  `sided_names_parse_to_one_side` (LShift, RShift, LCtrl, RCtrl, LAlt, RAlt, LWin, LeftWin, RWin, RightWin),
  `other_names_parse_to_their_key` (A, CapsLock, F13; `"Foo"` -> `None`), `any_side_accepts_both_sides_and_the_generic_code`,
  `sided_key_accepts_only_its_side`, `win_has_no_generic_code`, `overlaps_table`, `display_names` (`Win`, `LWin`, `RWin`, `Shift`, `LShift`).
- `data::key_combination::tests`: `from_keys_dedups_and_keeps_order`, `equality_ignores_order`,
  `generic_modifier_codes_become_either_side`, `display_joins_with_plus`.
- `config::parsing::tests`: `parse_combo_keeps_sides` (`"LShift+RWin+A"`), `parse_combo_win_is_either_side_lwin_is_left`,
  `parse_key_list_keeps_output_codes` (`"Win+Shift+S"` -> `[VK_LWIN, VK_SHIFT, VK_S]`); existing tests compare `KeySpec`.
- `config::loader::tests`: `chord_with_an_unsided_modifier_parses_to_either_side`; `parse_sequence_all_step_kinds` unchanged (output `PressCombo` stays VK).
- `engine::keys::tests`: `any_side_is_down_for_either_side_or_the_generic_slot`, `sided_key_is_down_only_on_its_side`.
- `engine::matching::tests`: `keys_down_needs_every_key_on_its_side`.
- `engine::chords::tests`: existing tests take `&[KeySpec]` (helper `specs(&[VK_J, VK_K])`); new
  `unsided_shift_chord_completes_with_either_shift`, `completed_reports_the_pressed_codes` (`[VK_RSHIFT.0, VK_SPACE.0]`).
- `engine::sequences::tests`: `ctrl_x_ctrl_s_sequence` asserts `last_step_keys == vec![VK_S.0]` (deliberate, D6); new `sided_step_needs_its_side`.
- `system::registry::tests`: Win combos contain `KeySpec::AnySide(Modifier::Win)`; the language default contains `AnySide(Alt)` and `AnySide(Shift)`.
- `hook::keyboard_hook::tests`: `:291-295` compares with `SINGLE_KEYS.map(KeySpec::from)`, `:313` with `vec![KeySpec::from(VK_F13)]`.
Docs in C2: README tree, `key_combination.rs` module bullets, combo-syntax modifier row and a "Sided modifiers" paragraph (replaces
`:187-189`, prose only), chord restriction "keys cannot repeat or overlap", LShift example warning, features `:520`, `:528`;
`default_config.ron` modifier list (`:32`) and LShift example warning (`:200`); CHANGELOG C2 texts.
Invariants: suppression symmetry (chord `Completed` marks only suppressed codes; sequences skip modifiers); every per-key index
`< 256`; `#![forbid(unsafe_code)]` kept; no new allocation per key-down.

**C3. `Engine: red-first: a combo fires only on its own key (KS-04)`** (declared red against C2; same first failing line on `e67e5cb`).
- T10 `key_pressed_while_an_immediate_binding_key_is_held_passes`: `(keys: "CapsLock", action: SwitchLanguage)`. down CapsLock
  (suppressed, fired); +30 down A (passed, nothing); up A (passed); up CapsLock (suppressed); total `[SwitchLanguage]`.
  PREDICTED RED: `down A at +30 ms: expected the event to be passed, it was suppressed`.
- T11 `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing`: README pair `Tap(200)` SwitchLanguage + `Hold(200)`
  `PressKey("LShift")` on CapsLock. down CapsLock; +50 down A (passed, nothing); up A (passed, nothing); up CapsLock (suppressed,
  `[SwitchLanguage]`); +500 nothing; total `[SwitchLanguage]`.
  PREDICTED RED: `down A at +50 ms: expected the event to be passed, it was suppressed`.
- T12 `modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock`: `CapsLock` -> SwitchLanguage, `Shift+CapsLock` ->
  ToggleCapsLock. down CapsLock (`[SwitchLanguage]`); down LShift (passed, nothing); up LShift (passed); up CapsLock (suppressed);
  total `[SwitchLanguage]`.
  PREDICTED RED: `down LShift at +0 ms: expected the event to be passed, it was suppressed`.
- T13 `modifier_pressed_last_does_not_fire_a_combo`: `(keys: "Ctrl+J", action: SwitchLanguage)`. down J (passed); down LCtrl (passed,
  nothing); ups pass; total `[]`.
  PREDICTED RED: `down LCtrl at +0 ms: expected the event to be passed, it was suppressed`.
- T14 `deferred_combo_arms_only_on_its_trigger_key`: `(keys: "Shift+CapsLock", action: ToggleCapsLock, trigger: Tap(200))`. down LShift
  (passed); down CapsLock (suppressed, nothing); +50 up CapsLock (suppressed, `[ToggleCapsLock]`); up LShift (passed); +300; down
  CapsLock (passed); down LShift (passed, nothing); +50 up LShift (passed, nothing); up CapsLock (passed); total `[ToggleCapsLock]`.
  PREDICTED RED: `down LShift at +350 ms: expected the event to be passed, it was suppressed`.

**C4. `Matching: trigger-key rule for combos and deferred triggers (KS-04)`**: makes T10-T14 green. `is_trigger_key`,
`combo_matches(combo, vk, &self.keys)` in the Combo branch; engine docs (rule 4; the defect list drops KS-04). Unit tests in
`engine::matching::tests`: `trigger_key_is_a_non_modifier_key_of_the_combo` (`Ctrl+J`: J yes, LCtrl no, A no),
`modifier_only_combo_triggers_on_any_of_its_keys` (`Alt+Shift`: LAlt, RShift yes; A no), `sided_trigger_needs_its_side`
(`"LShift"`: RShift no). Docs: README kinds table "Fires when" (`:217`), triggers "Key-down" bullet (`:280-282`), a "Which key fires a
binding" paragraph in combo syntax (prose); `default_config.ron` combo block; CHANGELOG Fixed KS-04. Invariant: repeats of the
trigger key still match (`press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed` stays green).

**C5. `Engine: red-first: exact modifier matching`** (declared red against C4).
- T15 `capslock_binding_does_not_fire_while_shift_is_held`: `(keys: "CapsLock", action: SwitchLanguage)`. down LShift (passed); down
  CapsLock (passed, nothing); up CapsLock (passed); up LShift (passed); down CapsLock (suppressed, `[SwitchLanguage]`).
  PREDICTED RED: `down CapsLock at +0 ms: expected the event to be passed, it was suppressed`.
- T16 guard `allow_extra_modifiers_lets_capslock_fire_while_shift_is_held`: same config plus `allow_extra_modifiers: true`; LShift, CapsLock
  -> suppressed, `[SwitchLanguage]`. GREEN at C4 (field ignored, superset).
- T17 `ctrl_j_does_not_fire_on_ctrl_shift_j`: `(keys: "Ctrl+J", action: SwitchLanguage)`. down LCtrl, down LShift, down J (passed,
  nothing); up J; up LShift; down J (suppressed, `[SwitchLanguage]`).
  PREDICTED RED: `down J at +0 ms: expected the event to be passed, it was suppressed`.
- T18 guard `allow_extra_modifiers_lets_ctrl_j_fire_on_ctrl_shift_j`. GREEN at C4.
- T19 guard `either_shift_counts_as_the_same_modifier`: `(keys: "LShift+F13")` with LShift and RShift held: F13 fires. GREEN at C4.
- T20 `auto_blocker_matches_its_modifiers_exactly`: `Scenario::with_bindings(vec![Binding::new_auto_blocker(KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT]))])`.
  LAlt (passed), LShift (suppressed, `[DoNothing]`), ups; then LCtrl, LAlt, LShift -> LShift passed, nothing.
  PREDICTED RED: the second `down LShift at +0 ms: expected the event to be passed, it was suppressed`.
- T21 `config::loader::tests::allow_extra_modifiers_is_only_for_keys_bindings`: a sequence `["F13","F14"]` and a chord `["F13","F14"]`,
  each with `allow_extra_modifiers: true`, -> `expect_err("allow_extra_modifiers on a sequence/chord must be rejected")`; the
  message contains `allow_extra_modifiers`.
  PREDICTED RED: panic `allow_extra_modifiers on a sequence/chord must be rejected: [Binding {` … (unknown field ignored at C4).
- T22 guard `shift_capslock_fires_toggle_capslock_with_either_shift`: the default pair without `block_original_combo`: LShift+CapsLock
  and RShift+CapsLock each `[ToggleCapsLock]`, suppressed; Shift key-ups pass. GREEN everywhere.

**C6. `Matching: exact modifiers with per-binding allow_extra_modifiers`**: makes T15, T17, T20, T21 green; guards stay green. D3, D8
(`Binding` suffix), D10. Unit tests: `data::key_combination::tests::modifiers_collects_families`;
`data::binding::tests::display_marks_allow_extra_modifiers`; `engine::matching::tests`: `exact_modifiers_reject_an_extra_family`,
`either_side_of_a_family_counts_once`, `allow_extra_modifiers_accepts_a_superset`, `modifier_only_combo_is_exact_too`;
`config::loader::tests`: `allow_extra_modifiers_defaults_to_false`, `allow_extra_modifiers_true_is_parsed`,
`readme_matching_examples_load` (the README RON blocks of this lane, copied verbatim, through `from_ron_str`; README RON blocks are
added only in this commit, C2/C4 README edits are prose). Hook tests per D10. Docs: README "Exact modifiers" paragraph with the RON
block and the AltGr note, flags-table row, features `:520`; `default_config.ron` combo block and flags block; CHANGELOG Added and
Breaking. Engine docs: rule 4 complete, defect list.

| Test | master | C1 | C2 | C3 | C4 | C5 | C6 |
|---|---|---|---|---|---|---|---|
| T1 T2 T4 T5 T7 T8 T9 | — | red | green | green | green | green | green |
| T10-T14 | — | — | — | red | green | green | green |
| T15 T17 T20 T21 | — | — | — | — | — | red | green |
| T3 T6 T16 T18 T19 T22 | — | green | green | green | green | green | green |
| the 40 existing scenarios (`src/engine/scenarios.rs`) | green | green | green | green | green | green | green |

Checked by hand against each stage: `right_hand_modifiers_match_unsided_combo_keys` (`AnySide` accepts RCtrl/RAlt),
`more_specific_immediate_binding_wins` (C4: Shift is not a trigger of `Shift+CapsLock`; C6: `CapsLock` alone is not exact with
Shift), `sequence_with_a_modifier_held_through_both_steps` (Ctrl key-up still passes: Ctrl is no longer marked, and before it was
the generic 0x11, a no-op), `modifier_lost_by_the_os_is_resynced_before_matching` (resync clears LShift, `CapsLock` matches exactly).

## Acceptance mapping

| Acceptance line (brief / roadmap) | Covered by |
|---|---|
| with CapsLock held, `a` passes through and nothing fires | T10 (Immediate), T11 (Tap/Hold); M1 |
| the CapsLock binding does NOT fire while Shift is held, and does with `allow_extra_modifiers: true` | T15, T16, `allow_extra_modifiers_true_is_parsed`; M2 |
| Ctrl+J does not fire on Ctrl+Shift+J | T17 (+T18); M3 |
| LShift and RWin in keys fire only for that side, and Shift fires for both | T1, T2, T3, T4; M4, M5 |
| Shift in a chord fires | T5 (+T6); M6 |
| Shift+CapsLock fires ToggleCapsLock | T22, existing `more_specific_immediate_binding_wins` (`src/engine/scenarios.rs:97-118`); M2 |
| the sided rule applies to sequence steps | T7, T8 |
| deferred arms are keyed on the trigger key | T14, T11 |
| the trigger-key rule holds for Immediate and deferred bindings | T10-T14 |
| roadmap A: "`decide()` tests … `LShift` and `RWin` in `keys`, and `Shift` in `chord`, fire" (`roadmap.md:98`) | T1, T2, T4, T5 |
| roadmap smoke 3 (CapsLock held, `abc` typed) and 6 (double LShift) (`roadmap.md:231`, `:234`) | M1, M4 |
| README / default_config / CHANGELOG updated | review of C2/C4/C6; `default_config_parses`, `readme_matching_examples_load` |

## Manual checks (human only)

Release build, `swch open`, Notepad in front.

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M1 | default `config.ron` | press CapsLock and, still holding it, type `abc` quickly (< 0.4 s) | `abc` appears; the language switched once | none (needs a live hook) |
| M2 | default `config.ron` | Shift+CapsLock with each Shift; CapsLock, then Shift while CapsLock is held; Ctrl+CapsLock | CapsLock toggles; second: the language switches once, CapsLock state unchanged, Shift+A then types "A"; Ctrl+CapsLock reaches Windows | none |
| M3 | add `( keys: "Ctrl+J", action: DoNothing )`, Chrome or Edge | Ctrl+J; Ctrl+Shift+J | Ctrl+J swallowed (no downloads page); Ctrl+Shift+J opens the DevTools console | none |
| M4 | uncomment the `keys: "LShift"` `DoubleTap(250)` example | double-tap left Shift; double-tap right Shift | left toggles CapsLock, right does not (left Shift no longer types capitals until item 1.3: expected) | none |
| M5 | keyboard with a right Win key; `( keys: "RWin+E", action: DoNothing )` | RWin+E; LWin+E | RWin+E swallowed; LWin+E opens Explorer (Start may open on RWin release: KS-12) | none |
| M6 | `( chord: (keys: ["Shift", "Space"], window_ms: 50), action: ToggleCapsLock )` | LShift+Space together; RShift+Space together | CapsLock toggles each time; remove the binding afterwards (both Shifts are swallowed while it exists) | none |
| M7 | Russian layout (AltGr), `( keys: "Alt+X", action: DoNothing )` | LAlt+X; RAlt+X | LAlt+X swallowed; RAlt+X passes (the fake LCtrl is an extra modifier) | none |

## Risks and edge cases (R1-R10)

- **R1 hot path.** Per key-down per Combo binding: at most keys x 3 table reads; `modifiers()` only after the trigger and keys checks
  pass. No allocation (the 256-slot scan of `active_keys` and the chord `canonical` Vec go). No panics: no `unwrap`, indexing only
  through `slot()`; `Modifier::left/right` constants are < 256. Adapter paths (`code < 0`, sentinel, `LRESULT(1)`/`CallNextHookEx`) untouched.
- **R2 sentinel.** No new `SendInput`. Macro `Combo` output codes unchanged (`parse_key_list`); the sentinel builders are untouched.
- **R3 shared state.** Nothing new is shared; matching runs inside `with_engine`; bindings come from the one `ArcSwap` store.
- **R4 atomics.** None added.
- **R5 builders.** `with_allow_extra_modifiers(self, bool)`; `KeyCombination::new/with/from_keys` keep consume-self, <= 2-argument shapes.
- **R6 Display.** `Modifier`, `KeySpec`, `KeyCombination` get `Display`; `Binding` shows the flag; no `_ =>` arm hides a variant
  (`KeySpec::Key(vk)` falls back to `vk_name`, which already prints `VK(n)`).
- **R7 specificity.** Sort unchanged (`key_count` = number of `KeySpec`s). `"Shift+LShift+A"` counts 3 (edge, harmless).
- **R8 key identity.** Canonicalisation moves to parse time (`KeySpec`), and combo, step and chord paths all compare the event's
  sided code against it. R8's current text ("folds … to the generic code", "chords see the raw sided vk", "'Any Windows key' is
  `VK_LWIN`") becomes stale: the reviewer applies R8's second clause; Q4 updates the text after the merge.
- **R9 registry.** Only combo construction and the INFO line change; reads only.
- **R10 loop/shutdown.** Unchanged.
- **New KS-06 path (follow-up for the KS-06/KS-14 lane).** With exact matching, pressing an extra modifier while an Immediate key is
  held makes the binding stop matching mid-hold; its repeats then leak while its key-up stays blocked. That is the existing KS-06
  mechanism (`book/src/audit.md:253`) with one more trigger; KS-06's symmetric-repeat fix closes it. Not pinned here (known defect).
- **KS-14 interplay.** A modifier whose key-down was suppressed is erased by the next resync and then counts as "not held" for
  exactness (after the Alt+Shift blocker swallows Shift, `Alt+F` matches). Same as with superset matching; the KS-14 lane changes it.
- **Phantom OS modifiers.** If Windows itself believes a modifier is down (lost key-up), exact bindings stop firing until the resync
  sees it released; the user's own typing is affected the same way. UIPI/secure desktop: modifiers read as up (Win32 table).
- **Our own injected modifiers.** A macro `Combo("Ctrl+S")` on a worker thread holds Ctrl in the OS for a moment; a physical hotkey
  pressed in that instant sees an extra Ctrl. Rare; superset matching hid it.
- **AltGr.** `RAlt`/`Alt` bindings need `Ctrl+RAlt` or the flag on AltGr layouts (README note); `Ctrl+X` no longer fires on AltGr+X (improvement).
- **Default config.** Ctrl/Alt/Win+CapsLock now reach Windows (CHANGELOG says so). The `` ` `` blocker (registry "4") lets Shift+` through.
- **LShift DoubleTap example.** Now fires but swallows every left-Shift press (deferred triggers always suppress until 1.3); README
  and template warn.
- **DoubleTap on `"Shift"`.** Keyed by the sided code: LShift then RShift is not a double tap (as today).
- **Reload mid-gesture, `swch off`/`on`, foreground change, shutdown.** No new state: pending entries keep cloned actions; pending
  sequences compare `KeyCombination` by set-equality of `KeySpec`; resync and blocked-clearing are unchanged.
- **Repeats.** The trigger rule holds for repeats of the trigger key; repeats of other keys never match (correct).

## Docs impact

**CHANGELOG.md `## [Unreleased]`** (texts for the developer; Breaking entries sit under Changed and start with `**Breaking:**`):
- C2, Changed: "**Breaking:** key names keep their side in every trigger: `keys`, `sequence` steps and `chord` keys (KS-07).
  `Shift`, `Ctrl`, `Alt` and `Win` (`Super`) match either key; `LShift`, `RShift`, `LCtrl`, `RCtrl`, `LAlt`, `RAlt`, `LWin` and
  `RWin` (and their `Left…`/`Right…` forms) match only that key. `LWin` used to match both Win keys in `keys` and `sequence`, and
  `Win` meant the left Win key only in `chord`. A `chord` whose keys overlap, such as `["Shift", "LShift"]` or `["Win", "RWin"]`,
  is now a load error. Output keys (`PressKey`, `Key`, `Combo`) are unchanged: `Win` still sends the left Win key." Also: "The
  auto-blocker line printed at load names its keys (`SwitchLanguage: registry combination Alt+Shift`) instead of key codes."
- C2, Fixed: "KS-07: sided names in `keys` and `sequence` steps fire, and `Shift`, `Ctrl` and `Alt` complete a `chord`. The
  `keys: "LShift"` + `DoubleTap(250)` example works; until roadmap item 1.3 it swallows every left-Shift press." Also: "KS-32
  (partly; an unverified report): when a `sequence` completes, the key-up of a modifier held in its last step (for example `Win`
  in `"Win+F14"`) is no longer swallowed."
- C4, Fixed: "KS-04: a `keys` binding fires, or arms its `Tap`/`Hold`/`DoubleTap`, only on the key-down of one of its own keys: a
  non-modifier key, or any of its keys for a binding made only of modifiers (`"Alt+Shift"`, `"LShift"`). Typing while CapsLock is
  held no longer re-fires the CapsLock binding or swallows the typed keys, Shift pressed while CapsLock is held no longer fires
  `Shift+CapsLock`, and pressing a combo's modifier after its other key (J, then Ctrl) no longer fires `Ctrl+J`."
- C6, Added: "Per-binding `allow_extra_modifiers: true` for `keys` bindings: the binding also fires while modifiers it does not name
  are held. It is a load error on `sequence` and `chord` bindings."
- C6, Changed: "**Breaking:** modifiers match exactly (owner decision; roadmap open question 2). A `keys` binding fires only when the
  held modifiers (Shift, Ctrl, Alt, Win; either key of a pair counts as that modifier) are exactly the ones it names: `"CapsLock"` no
  longer fires while Shift is held, and `"Ctrl+J"` no longer fires on Ctrl+Shift+J. With the default config, CapsLock pressed with
  Ctrl, Alt or Win held now reaches Windows instead of switching the language. Auto-blockers match exactly too: the `Alt+Shift`
  blocker ignores Ctrl+Alt+Shift, and the `` ` `` blocker lets Shift+` through. Add `allow_extra_modifiers: true` to a binding for the
  old behaviour. On layouts with AltGr, right Alt arrives together with a left Ctrl, so a binding on `RAlt` needs `Ctrl+RAlt` or
  `allow_extra_modifiers: true`."

**README.md:** as in the touch set. The new text covers: the modifier row (either-side names vs that-side names, all aliases from
`src/data/vk_name.rs:134-144`); "Sided modifiers" (replaces "the hook normalises left/right"); "Which key fires a binding" (trigger
key; deferred gestures armed on it); "Exact modifiers" with one RON block (`Shift+CapsLock`, `LShift+F13`, `CapsLock` with
`allow_extra_modifiers: true`) and the AltGr note; flags-table row `allow_extra_modifiers | false | keys bindings only …`; chord
restriction "keys cannot repeat or overlap (`["Shift", "LShift"]` is rejected); a modifier in a chord is swallowed on every press";
the LShift example warning (1.3); features line 520 rewritten to sided + exact matching; line 528 rewritten without the removed
"stack-allocated active-keys buffer" (the engine matches against its per-key table and builds no key list per press).

**`src/config/default_config.ron`:** modifier list `:32` (either-side vs that-side names), a short "which key fires / exact
modifiers / `allow_extra_modifiers`" paragraph after `:41`, a flags entry after `:110`, the LShift example warning at `:200`.

**Book (phase book sync, doc-writer; not this lane):** `config/keys.md` §6.2 item 5 (dedup now by `KeySpec`; `"Win+LWin+Super+E"`
gives 3 keys), §6.4 table (both columns) and "how the daemon prints names" (`LWin`), §6.6 AltGr warning, §6.7 whole;
`config/kinds.md` §7 (bullets + KS-04 warning), §8.2 item 2, §8.4 KS-07 warning, §9.1 duplicates/overlap, §9.2 item 1, §9.3 KS-07
warning; `config/matching.md` §15.2 INFO line, §15.4 first warning (exact blockers), §16.2-§16.3; `config/structure.md` §2.2 LShift
bullet, §5.1/§5.2 tables (new field); `config/errors.md` §18.5 rule 9 (overlap) and a new rule (`allow_extra_modifiers` on
sequence/chord), §18.6 INFO line, §18.7 sided-names bullet; `config/triggers-conditions.md` §11.8 (arming on any key);
`config/known-issues.md` rows 7, 12, 13, 15, 22, 37, 39, 52; `config/example.md` header bullets and `:245-249`;
`architecture.md` key-event path steps 6 and 9 and flowchart nodes `COL`, `MT`; `audit.md` status lines for KS-04 and KS-07
("Fixed in <sha> (fix/matching)"); `roadmap.md` A bullets `:79`, `:82`, 1.6 `:207`, Q2 `:578`, syntax table `:503`, `:507`.

## Questions for the orchestrator

1. **Trigger key = a non-modifier key (D2) vs the brief's literal "belongs to its combo".** Recommendation: keep D2. It satisfies
   the brief's "ONLY when" condition, closes the "Shift pressed while CapsLock is held fires Shift+CapsLock" half of KS-04
   (`book/src/audit.md:204`) and is the audit's suggested fix. Fallback if vetoed: `is_trigger_key` becomes "some key accepts
   `pressed`"; T12, T13 and the second half of T14 are dropped; the README/CHANGELOG KS-04 text loses the modifier clause. Not blocking.
2. **Book pages.** CLAUDE.md asks the developer to update `book/src/config/*.md` with a semantic change, but every page is pinned to
   `7cb3521` and validated with `cfgcheck` by the doc-writer. Recommendation: the phase book sync applies the Docs-impact list (as
   A1 did); this lane does README, template and CHANGELOG. Not blocking.
3. **Sequences.** Recommendation: steps get sided semantics only; no exact modifiers or trigger-key rule until 3.2 (KS-15/16), and
   `allow_extra_modifiers` is a load error on `sequence`/`chord`. Not blocking.
4. **Rule texts after the merge.** `.claude/agents/code-reviewer.md` R8 and CLAUDE.md "Layout" (line 27, engine file list) describe
   folding and `VK_LWIN` as "any Win". Recommendation: the orchestrator updates them after the merge (R8: "triggers are `KeySpec`
   (`data/key_spec.rs`); the engine compares the event's sided code against them on every path; either Win is
   `KeySpec::AnySide(Modifier::Win)`"). Not blocking.
5. **Overlapping chord keys become a load error** (breaking for configs that loaded). Recommendation: accept; `["Win","RWin"]` is
   rewritten `["LWin","RWin"]`; the CHANGELOG Breaking entry says so. Not blocking.
6. **KS-05 / KS-18.** Exact auto-blockers (roadmap A bullet `:80`) arrive here as a side effect; dedup (KS-18) and the swallowed
   second modifier (1.2) do not. Recommendation: no KS-05/KS-18 status change from this lane. Not blocking.

## Cut readiness checklist

- Spec
  - [x] KS-04 and KS-07 read (`book/src/audit.md:198-216`, `:278-296`); scenarios: T10-T12 (KS-04 Immediate, deferred, Shift), T4 (LShift example), T1/T2/T7 (sided keys and steps), T5 (`["Shift","Space"]`)
  - [x] Every book reference re-located (table)
  - [x] Goal, non-goals and dependent items (1.1, 1.2, 1.3, 1.6, 3.2, the KS-06/14 lane, feature 2, B) stated
- Design
  - [x] Every decision has a rationale and rejected alternatives (D1-D12)
  - [x] Owner decisions: exact modifiers + `allow_extra_modifiers`; repeat defaults untouched; RON kept; `println!`/`eprintln!` only; licence untouched
  - [x] Thread named for every path; nothing heavy in the LL callback (table reads only)
  - [x] Install, uninstall and timers untouched
  - [x] N/A new `SendInput`: none; output codes unchanged
  - [x] Sided/unsided consistent in combo, chord and sequence (one `KeySpec`)
  - [x] Reload, `swch off`/`on`, foreground change and shutdown during a pending gesture covered (Risks)
  - [x] No new shared state or atomics; no `static mut`
  - [x] No new module cycle (`engine -> data`, `config -> data`, `data::key_spec -> data::vk_name`); logic in `engine::matching`, reached by scenarios
- Win32
  - [x] APIs relied on listed with path, feature (present), thread, errors and doc link; generic-VK injection marked (unverified)
  - [x] Elevation/UIPI considered (`GetAsyncKeyState` 0 -> modifiers up); pipe/token N/A
- Commits and tests
  - [x] Red-first with exact predicted reds (C1, C3, C5; assertion reds, compile-clean)
  - [x] Tests drive `Engine::on_key` through `Scenario`/`FakeEnv` or are pure unit tests; hook tests only read `GetAsyncKeyState`
  - [x] N/A ignored tests: nothing automatable needs a live hook; M1-M7 are human checks
  - [x] Every non-red commit passes the gate; red commits fail only the listed tests
  - [x] README/template/CHANGELOG in the commit that changes the meaning (C2, C4, C6)
- Acceptance
  - [x] Every acceptance line mapped
  - [x] Every question has a recommendation

## Developer addendum (implementation, round 1)

Branch `fix/matching`, 10 commits on `e67e5cb` (head `c609a69`). C1-C6 are implemented as cut. C7-C9 were added to resolve
critique I1 and I2 in code (critique option I2(a)); they change no decision of D1-D12.

### Critique remarks

| Remark | State | Where | Pinned by |
|---|---|---|---|
| I1 (modifier triggers go live while KS-06 is open; M4/M6 lead to a stuck Shift) | RESOLVED in code | `src/engine/mod.rs:427-443` (a repeat is suppressed iff its key is marked blocked) | `double_tap_lshift_held_past_the_repeat_delay_swallows_its_repeats_too`, `chord_keys_held_past_the_repeat_delay_swallow_their_repeats_too` (`src/engine/scenarios.rs:1231`, `:1252`) |
| I2 (exact matching opens pass-then-suppress on auto-repeat) | RESOLVED in code | same rule; plus a passing first key-down clears its blocked bit (`src/engine/mod.rs:437-443`), so the bit the repeat rule reads always describes the key's latest key-down | `repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (`:1204`, the critic's scenario), `shift_held_after_the_auto_blocker_and_alt_released_is_not_left_stuck` (`:1338`) |
| M1 (M2 manual check has no timing bound) | RESOLVED | impl.md manual checks: M2 bounded to < 0.4 s like M1 | manual |
| M2 (test-only `HeldKeys` impls duplicate the production rule) | RESOLVED | no test impls; `engine::matching` and `engine::sequences` tests build a real `KeyState` (`held()` helpers) | `keys_down_needs_every_key_on_its_side`, `sided_step_needs_its_side`, the matching tests |
| M3 (README test proves a copy) | RESOLVED | `src/config/loader.rs:1307-1336`: `include_str!("../../README.md")`, every RON block of the "Combo syntax" section is loaded | `readme_matching_examples_load`; mutation checked (a broken `F99` example in README.md fails it) |
| M4 (README:528 keeps a false "no allocations" claim) | RESOLVED | README features: "Table-driven matching ... no list of held keys is built per key press"; the "Hot-path allocations: none" heading is gone | review |

Open questions of the critique: (1) book pages are not edited (D12, list in Docs impact); (2) the optional trigger check before
`condition.evaluate` (roadmap 3.5) is not done; (3) lane order: KS-06 is fixed in this lane (C7-C9), so master never carries
exact matching or live modifier triggers without symmetric repeats. KS-14 (the resync erases a swallowed modifier) stays with its
own lane; C9 only stops it from leaving a key stuck.

### Added commits

- **C7 `Engine: red-first: an auto-repeat follows its key-down (KS-06)`** (`f4b2582`): five scenarios, red against C6.
- **C8 `Engine: red-first: a key-down that reaches the OS clears its key-up block (KS-06)`** (`faf3be1`): one scenario, red
  against C7 (`up LShift at +533 ms: expected the event to be passed, it was suppressed`).
- **C9 `Engine: an auto-repeat follows its key-down's suppression (KS-06)`** (`20672b7`): `Engine::on_key_down` ends with
  repeat -> `suppress = is_blocked(vk)`; first key-down suppressed -> `mark_blocked`; first key-down passed -> `take_blocked`.
  Actions still fire on repeats as before (KS-10 stays with roadmap 1.1). With the repeat rule alone the C8 scenario still
  failed at `repeat LShift at +533 ms` (stale bit); the clearing clause makes it green.

Why in this lane and not deferred: I2 is created by exact matching itself, and I1 by sided keys; both break the suppression
invariant (CLAUDE.md principle 2) on paths this lane opens. The rule is the audit's KS-06 suggested fix
(`book/src/audit.md` KS-06 "Suggested fix": suppress a repeat whose bit is set; clear the bit when a key-down passes) extended by
I2's requirement that a repeat never suppresses when its key-down passed. CHANGELOG lists it under Fixed KS-06.

### Cut questions as implemented

Q1: D2 kept (non-modifier trigger key). Q2: book not edited. Q3: sequences get sided semantics only; `allow_extra_modifiers` is a
load error on `sequence`/`chord`. Q4: CLAUDE.md "Layout" and the reviewer's R8 text are not edited (orchestrator). Q5: overlapping
chord keys are a load error. Q6: no KS-05/KS-18 status claim.
