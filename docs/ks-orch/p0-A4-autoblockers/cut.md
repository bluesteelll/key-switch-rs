# Cut: p0-A4-autoblockers — Workstream A "exact auto-blockers and dedup" (KS-18; KS-05 exact-match part)

Tree: `D:/key-switch-rs/key-switch-rs`, `master` @ `44f79ac85dee2a2cc27c3d1f5d34d88b4f44bc36` (clean). Branch to cut: `fix/auto-blockers`.
All `path:line` below are on that tree unless marked `@7cb3521`.

## Verdict

CUT READY

## Spec re-located

| Spec reference (book @7cb3521) | Current file:line | Status | Note |
|---|---|---|---|
| KS-05 loc. `src/config/loader.rs:172-191` (`expand_with_auto_blockers`) | `src/config/loader.rs:162-190` (doc 162-169, fn 170-190) | moved | logic unchanged since 7cb3521: backward-only dedup, blocker pushed before its requester |
| KS-05 loc. `src/hook/keyboard_hook_callback.rs:234` (blocker's `block_default` suppresses) | `src/engine/mod.rs:360-362` | moved | blocker goes through the generic Immediate arm `mod.rs:354-363` |
| KS-05 why-1 `src/config/default_config.ron:127-131` | `src/config/default_config.ron:146-150` | moved | `block_original_combo: true` still on `CapsLock` |
| KS-05 why-1 `src/data/binding.rs:89-99` (`new_auto_blocker`) | `src/data/binding.rs:94-105` | moved | DoNothing, `block_default: true`, `Always`, `Immediate`, `allow_extra_modifiers: false`, `is_auto_blocker: true` (`:71`, `pub(crate)`) |
| KS-05 why-1 `src/system/registry.rs:68-78`, `:192-199` | `src/system/registry.rs:67-79` (`get_default_combination`), `:181-189` (`parse_language_hotkey`), `:52-65` (`get_system_hotkey`) | moved | blocker keys are `KeySpec::AnySide` (`from_keys([VK_MENU, VK_SHIFT])`, `win_with` `:204-206`); "3" still falls back to Alt+Shift (KS-17, item 1.4, out of scope) |
| KS-05 why-2 / KS-04 `src/data/key_combination.rs:49-51` (subset `matches`) | `src/engine/matching.rs:58-67` (`combo_matches`), called at `src/engine/mod.rs:346-351` | gone (replaced by A2) | trigger-key rule (`:46-51`) and family-exact modifiers already apply to blockers; **non-modifier keys held never block a match** (`matching.rs:56-57`). That is the remaining KS-05 delta for this lane |
| KS-05 why-3 `keyboard_hook_callback.rs:86-97` (`sync_modifiers`) | `src/engine/keys.rs:88-96` | moved, KS-14 fixed (A3) | swallowed modifier stays held |
| KS-05 `README.md:353` (flag text) | `README.md:428` | moved | rewritten by A3 |
| KS-18 loc. `loader.rs:169-171` (doc promise) | `src/config/loader.rs:167-169` | moved | promise still false |
| KS-18 `loader.rs:181-183` (`already_present`) | `src/config/loader.rs:179-181` | moved | **defect live**: looks only at `result` built so far |
| KS-18 `loader.rs:185`, `:188` | `src/config/loader.rs:183`, `:186` | moved | blocker pushed before the requester |
| KS-18 `src/hook/keyboard_hook.rs:72` (stable sort) | `src/hook/keyboard_hook.rs:94-99` -> `src/engine/mod.rs:252-254` | moved | stable, unchanged |
| KS-18 `keyboard_hook_callback.rs:230` (`immediate_fired`) | `src/engine/mod.rs:335`, `:355-359` | moved | **defect live**: blocker's DoNothing takes the first-Immediate slot |
| roadmap 1.2 / KS-35 `src/data/binding.rs:95` | `src/data/binding.rs:100` | moved | blocker `when:` inheritance: out of scope (1.2) |
| `architecture.md:400` step 5, `config/matching.md` §15.3 `loader.rs:181-183` | `src/config/loader.rs:170-190`, `:179-181` | moved | book text is historical; phase book sync |
| roadmap A bullet `roadmap.md:80`, A warning `:100-101`, smoke 4 `:232`, 3.4 `os_guards` `:367`, syntax table `:503`, coverage `:521`, `:534` | book, current | unchanged | `:100-101` and `:232` say the modifier-first order needs 1.2 |
| Existing blocker pins (A2/A3) | `src/engine/scenarios.rs:27-40` (`with_alt_shift_blocker`), `:1168-1191`, `:1516-1557`, `:1648-1679`, `:1681-1702`, `:1879-1910`; `src/engine/properties.rs:19`, `:54-60`, `:193-197` | current | three of them assert `fired [DoNothing]` (`:1179`, `:1530`, `:1547`, `:1660`, `:1678`) and flip in commit 4 (D3) |

State on master: KS-05 is **partly** fixed (A2: trigger-key rule + exact families make the blocker ignore Ctrl+Alt+Shift and Shift+`); the blocker still matches with any non-modifier key held. KS-18 is fully live. Neither is restructured away, so the lane proceeds.

## Goal and non-goals

**After the lane**

1. An auto-blocker suppresses a key-down only when that key is one of the hotkey's keys, all hotkey keys are held, and every other key the engine holds is one Windows does not have down: Windows would see the hotkey and nothing else (D2). Shift pressed while Alt and Tab are held, Shift while Ctrl and T are held, and Alt+Shift during typing rollover reach Windows.
2. An auto-blocker only suppresses. It fires nothing and never takes the first-Immediate slot, so a user binding matching the same key-down always fires, whatever the file order or key count (D3).
3. The loader skips a blocker when any `keys` binding of the file (the requester included) has a `KeyCombination` equal to the hotkey, emits one blocker per distinct hotkey, appends blockers after the user bindings, and prints one `[INFO]` line per skipped hotkey (D4, D5).
4. Blockers stay distinguishable by the existing `Binding::is_auto_blocker` flag (D1).

**Non-goals (later items)**

- Second modifier still swallowed in the modifiers-first order (Alt, Shift, Tab -> Alt+Tab; Ctrl, Shift, T -> Ctrl+T), lone-Alt menu, mask key, blocker v2: **1.2** (KS-12, rest of KS-05).
- Blocker inheriting the requester's `when:`: **1.2** (KS-35).
- Hotkey "3" -> no blocker: **1.4** (KS-17).
- One winner per event / `block_default` of a losing binding: **1.6** (KS-13).
- Stale held keys after the secure desktop, resync of non-modifiers: **2.3** (KS-29, KS-31). D2 makes blockers immune; user bindings are unaffected by non-modifier extras anyway.
- Blockers in a separate `os_guards` list: **3.4**.
- `book/src/**`: phase book sync (Q5).

**Later items that build on this one:** 1.2 hooks its mask/v2 into the blocker branch (`mod.rs`, D1) and reuses `blocker_matches`; 1.4 changes what the hotkey lookup returns (the D6 seam keeps its tests registry-free); 3.4 moves the flag-selected blockers into `os_guards`.

## Decisions

### D1: The engine selects blockers by the existing `is_auto_blocker` flag

- **What.** Keep `pub(crate) is_auto_blocker: bool` (`binding.rs:71`) as the only marker. In `Engine::on_key_down`, the `BindingKind::Combo` arm (`mod.rs:345`) first checks the flag; a blocker takes its own branch (D2, D3) and `continue`s; user bindings keep `combo_matches` unchanged.
- **Why.** Spec bullet 3 allows a flag. The binding list type (`ArcSwap<Vec<Binding>>`, `keyboard_hook.rs:44`), `update_bindings` (`:94-99`) and `Engine::on_key(event, &[Binding], &dyn Env)` (`mod.rs:282`) stay as they are, so no publish path, sort or adapter changes (R3, R7). The engine is in the same crate, so the `pub(crate)` field is readable; reviewer rule "`is_auto_blocker` stays `pub(crate)`" (`code-reviewer.md:106`) holds. With D2 and D3 the blocker no longer "inherits subset matching and list order", which the critic's rule `architecture-critic.md:148-149` forbids.
- **Rejected.** A separate list (`Vec<Binding>` + `Vec<KeyCombination>` in the `ArcSwap`): changes `KeyboardHook`, the adapter, `Scenario` and IPC `status` count (`ipc_server.rs:133`); that is 3.4's `CompiledBindings`. A new `BindingKind::Blocker` variant: every `match` on `kind` (Display `binding.rs:186-203`, `key_count` `:40-46`, `combination()` `:110-115`) would need an arm and `combination()` would stop returning the blocker's keys.
- **Cost.** One branch per Combo binding per key-down (a bool read).

### D2: Exactness is "Windows sees the hotkey and nothing else"

- **What.** `matching::blocker_matches(combo, pressed, held, windows_has_down)` is true iff
  1. `is_trigger_key(combo, pressed)` (`matching.rs:46-51`; for the reachable blockers, all-modifier combos and the single grave key, that is "`pressed` is one of its keys"),
  2. `keys_down(combo, held)` (`:37-39`), and
  3. every code in the engine's held table is accepted by some key of `combo` (`KeySpec::accepts`, `key_spec.rs:134-139`), **or** `windows_has_down(code)` is false.
  The engine passes `|vk| env.os_key_down(vk)`. Condition 3 short-circuits: Windows is asked only about held codes outside the hotkey, never about the key being delivered (always accepted by 1) and never about the hotkey's own keys.
  `HeldKeys` (`matching.rs:22-34`) gains `fn all_held(&self, accept: impl FnMut(VIRTUAL_KEY) -> bool) -> bool`; `KeyState` implements it by scanning its 256-entry `active` table (`keys.rs:65`), generic slots included, no allocation.
- **Why.** The blocker exists to stop Windows from seeing its own hotkey; Windows judges by its own key state. A key Windows does not have down cannot cancel the hotkey for Windows:
  - a key the engine swallowed (CapsLock held by the default `SwitchLanguage` binding, Shift held by a `DoubleTap`): Windows never saw it, so Alt+Shift would reach Windows clean and switch the language;
  - a key whose key-up the hook never saw (L of Win+L, released on the secure desktop): the engine keeps it held until its next press (pre-existing, 2.3). With the literal engine-held reading the blocker would be off until the user types an `l` (fail-open, a regression against master, where non-modifier extras never mattered).
  Passed keys (Tab, T, the letter in rollover) are down in Windows, so the acceptance cases pass. A2's family exactness is kept for passed modifiers (the resync makes table = OS, `keys.rs:88-96`): Ctrl+Alt+Shift and Shift+` still pass.
  `GetAsyncKeyState` is valid for any key except the one being delivered: "the callback function is called before the asynchronous state of the key is updated" (LowLevelKeyboardProc, link below); D2 never asks about that key.
- **Rejected.** (a) Literal "engine-held set equals the combo": the two failure modes above (Q2 records it as the fallback: drop the `windows_has_down` clause). (b) `combo_matches` plus "no non-modifier key held": same failure modes, and swallowed modifiers of another family (KS-14 keeps them held) would also disable the blocker. (c) Scan the OS state of all 256 keys: 256 Win32 calls on the hook thread.
- **Cost.** At most one `GetAsyncKeyState` per held key outside the hotkey (typically 0-2), only on a key-down that is one of the blocker's keys while all its keys are held; under the existing engine lock, where the resync already makes 11 such calls per event (`keys.rs:22-34`, `:88-96`). Env contract text changes (`env.rs:32-41`), signature unchanged.

### D3: An auto-blocker only suppresses; it fires nothing

- **What.** In the blocker branch: if `blocker_matches` -> `decision.suppress = true`; nothing goes into `decision.fire`, `immediate_fired` (`mod.rs:335`) is not touched; then `continue`. Repeats need nothing new: rule 5 (`mod.rs:453-485`) already overrides a repeat's suppression by the key's record.
- **Why.** Spec: the blocker "never shadows a user binding". Dedup (D4) removes blockers on **equal** keys only. Since A2 (KS-07, exact families) user bindings can overlap a blocker without being equal: `LAlt+LShift` (same key count, so file order decided), or `Shift` with `allow_extra_modifiers: true` (fewer keys, always after the blocker). Today the blocker's DoNothing takes the Immediate slot (`mod.rs:355-359`) and those bindings never run on that press: KS-18 in a new form. With D3 the blocker's position in the list stops mattering, which also removes R7's "auto-blockers rely on load order among equal counts" (`code-reviewer.md:124`). `DoNothing` executes nothing (`windows_actions.rs:139`), so no observable action is lost.
- **Rejected.** Sorting blockers after user bindings only (audit's second suggestion, `audit.md:455`): does not help a user binding with fewer keys (`allow_extra_modifiers`). Dedup by overlap instead of equality: would drop blocking of RAlt+RShift whenever `LAlt+LShift` is bound.
- **Cost.** Five assertions in three pinned scenarios flip from `fired [DoNothing]` to nothing (commit 4). An overlapping user binding with `block_default: false` still has its key swallowed by the blocker (RK5).

### D4: Dedup over the whole user list, by key-set equality

- **What.** `expand_with_auto_blockers(user_bindings, system_combination)`:
  1. Walk the user bindings in file order; for each with `block_original_combo` whose `system_combination(&action)` is `Some(combo)`, record `combo` if not recorded yet (one entry per distinct hotkey, first-request order).
  2. For each recorded hotkey: if any user binding's `combination()` equals it (`KeyCombination::eq`, order-free, `key_combination.rs:16-30`; `Combo` kind only, any `trigger`, `when:`, `block_default`, action; the requester included), print once `[INFO] no auto-blocker for {combo}: binding #{index} uses the same keys` (index of the first such binding, counting from 0 like the load errors) and skip it; otherwise create `Binding::new_auto_blocker(combo)`.
  3. Return the user bindings unchanged, followed by the blockers (D5).
  Doc comments `loader.rs:122-127`, `:162-169` are rewritten to match.
- **Why.** Spec bullet 2, literally: whole list, requester included, skip if one exists. Equality (not overlap) because a sided or larger user combo does not cover the other sides of the hotkey (D3 handles their shadowing). Sequences and chords do not represent a held set (`combination()` returns `None`), and D3 already keeps them unshadowed (they do not use `immediate_fired`). A user binding on the exact hotkey with the default `block_default: true` swallows the hotkey's completing key itself, so the OS hotkey stays silent without the blocker.
- **Rejected.** Ignoring user bindings with a `when:` or a deferred trigger (keep the blocker for other windows): contradicts the spec's "if one exists, the blocker is skipped", and with `block_default: false` the kept blocker would override the user's choice. Recorded as Q4.
- **Cost.** O(requesters x bindings) at load; the registry is still read once per requester (`registry.rs:52-65`, unchanged, prints its own `[INFO]`).

### D5: Blockers follow the user bindings in the loaded list

- **What.** `from_ron_str` returns `[user bindings in file order..., blockers...]`. The startup banner (`app.rs:57-70`) therefore lists `[AUTO-BLOCK]` lines last. The stable sort (`mod.rs:252-254`) keeps blockers after user bindings of the same key count.
- **Why.** With D3 a blocker's position no longer changes any decision; a trailing group reads as "added by the loader" and keeps loaded indices equal to file indices. Belt and braces for any future code that does honour order.
- **Rejected.** Keep "immediately before the requester" (`loader.rs:183`): order would still matter to readers of the banner and to R7 without any benefit.
- **Cost.** Banner order changes (CHANGELOG line); book §15.1/§16.1 text (phase sync).

### D6: The loader takes the system hotkey lookup as a parameter

- **What.** `pub(crate) type SystemCombination = dyn Fn(&BindAction) -> Option<KeyCombination>;` `from_ron_str(text)` keeps its signature and delegates to `pub(crate) fn from_ron_str_with_hotkeys(text, &BindAction::get_system_combination)` (`windows_actions.rs:152-157`). `expand_with_auto_blockers` takes `&SystemCombination`.
  Test stand-ins return `Some(keys)` for `SwitchLanguage` and `SwitchLanguageBackward` only, `None` otherwise, like `registry::get_system_hotkey` (`registry.rs:17-50`, `:67-79`).
- **Why.** Every dedup test needs a known hotkey; today `from_ron_str` reads `HKCU\Keyboard Layout\Toggle` (`registry.rs:95-127`), so the outcome depends on the machine (Alt+Shift, Ctrl+Shift or grave). No registry access in any new test (R9).
- **Rejected.** Building blocker lists by hand only (`with_alt_shift_blocker`, `scenarios.rs:31-40`): cannot test the loader. A cargo feature or `cfg(test)` switch inside `registry.rs`: hidden global state.
- **Cost.** One alias, one function; `config/mod.rs` gains a `cfg(test)` re-export.

### D7: Tests

Loader unit tests (registry-free via D6), scenario tests on `Scenario` through `Engine::on_key` with `FakeEnv` (`testing.rs:47-93`, which models Windows' key state exactly as D2 needs: a key is down in Windows only if its key-down passed or was `down_unseen`), and unit tests of `blocker_matches` / `all_held`. Property tests (`properties.rs`) are unchanged: configs `default` and `mixed` already run the Alt+Shift blocker through 64 seeds x 250 steps in both modes, and D2/D3 do not touch the record logic they check.

### D8: Docs

README row `:428`, `CHANGELOG.md` Fixed (KS-05 partly, KS-18) and `default_config.ron:119-125` ride with the commit that changes the meaning (3 and 5). Engine and loader doc comments are updated in the same commits. `book/src/**`: phase book sync (Q5), list under "Docs impact".

### Type and signature sketch (no bodies)

```rust
// src/config/loader.rs
pub(crate) type SystemCombination = dyn Fn(&BindAction) -> Option<KeyCombination>;
pub(crate) fn from_ron_str(text: &str) -> Result<Vec<Binding>, ConfigError>; // unchanged signature
pub(crate) fn from_ron_str_with_hotkeys(
    text: &str,
    system_combination: &SystemCombination,
) -> Result<Vec<Binding>, ConfigError>;
fn expand_with_auto_blockers(
    user_bindings: Vec<Binding>,
    system_combination: &SystemCombination,
) -> Vec<Binding>;

// src/config/mod.rs
#[cfg(test)]
pub(crate) use loader::{from_ron_str, from_ron_str_with_hotkeys};

// src/engine/matching.rs
pub(super) trait HeldKeys {
    fn is_down(&self, key: KeySpec) -> bool;
    /// Whether `accept` holds for the code of every key held, generic
    /// modifier slots included; true when nothing is held.
    fn all_held(&self, accept: impl FnMut(VIRTUAL_KEY) -> bool) -> bool;
    fn modifiers(&self) -> ModifierSet; // existing default method, unchanged
}
pub(super) fn blocker_matches<H: HeldKeys + ?Sized>(
    combo: &KeyCombination,
    pressed: VIRTUAL_KEY,
    held: &H,
    windows_has_down: impl Fn(VIRTUAL_KEY) -> bool,
) -> bool;

// src/engine/testing.rs
impl Scenario {
    /// Like `from_ron`, with `keys` standing in for the Windows language
    /// hotkey that `block_original_combo` reads from the registry.
    pub(crate) fn from_ron_with_language_hotkey(text: &str, keys: &[VIRTUAL_KEY]) -> Self;
}
```

`HeldKeys` has no `dyn` use (only `H: HeldKeys + ?Sized` generics: `matching.rs:37`, `:58`, `sequences.rs:60`), so a generic method is fine. MSRV 1.88 (`Cargo.toml:8`) covers everything above.

## Threads

| Code path | Thread | Change |
|---|---|---|
| `config::load` -> `from_ron_str` -> `expand_with_auto_blockers` + registry read | main at startup (`main.rs:33`); `config-watcher` on reload (`config_watcher.rs:100-104`) | D4/D5/D6; read-only registry, unchanged |
| `Engine::on_key` blocker branch, `HookEnv::os_key_down` (`keyboard_hook_callback.rs:116-122`) | main thread, LL callback, inside `with_engine` (`keyboard_hook_callback.rs:162-164`) | D2/D3: a few table reads + bounded `GetAsyncKeyState`; no allocation, no lock, no action |
| `Engine::on_timer` | `hold-timer` threads | untouched (blockers arm nothing) |
| Startup banner | main (`app.rs:57-70`) | order only (D5) |

No hook, window, timer, COM apartment, atomic or `SendInput` is added or moved. Install/uninstall stay on the main thread (`keyboard_hook.rs:108-208`).

## Win32 API choices

No new API and no new Cargo feature.

| Purpose | `windows` path | Feature flag (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| D2: is a held key outside the hotkey down in Windows | `windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(i32) -> i16`, already wrapped by `HookEnv::os_key_down` (`keyboard_hook_callback.rs:116-122`, MSB test) | `Win32_UI_Input_KeyboardAndMouse` (yes, `Cargo.toml:44`) | main (hook callback) | "The return value is zero if the call fails. Reasons for failure include: The current desktop is not the active desktop. UI Privilege Isolation (UIPI) prevents the calling thread from accessing the foreground thread." Zero reads as up: the blocker then matches as on master (RK1) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate |
| Constraint: never ask about the delivered key; stay within the budget | `WH_KEYBOARD_LL` callback contract | n/a | main | "the callback function is called before the asynchronous state of the key is updated"; "If the hook procedure times out ... on Windows 7 and later, the hook is silently removed without being called." | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc |
| Hotkey lookup (unchanged) | `RegOpenKeyExW` / `RegQueryValueExW` (`registry.rs:106-154`) | `Win32_System_Registry` (yes, `:48`) | main / `config-watcher` | unchanged; read-only | https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regqueryvalueexw |

Not relevant here: `LLKHF_INJECTED`, the extended-key flag and `scanCode` (the lane adds no injection and reads none of them). Third-party injected keys and AltGr's fake LCtrl update Windows' key state like physical keys, so D2 counts them: AltGr+Shift never matches the Alt+Shift or Ctrl+Shift blocker (Ctrl is down in Windows), as with A2. Elevation/UIPI: RK1. Pipe DACL, token inheritance: n/a.

## Touch set

| File | Change | Why / commit |
|---|---|---|
| `src/config/loader.rs` | C1: `SystemCombination`, `from_ron_str_with_hotkeys`, parameterised `expand_with_auto_blockers` (old algorithm), `use crate::data::key_combination::KeyCombination`, test `auto_blocker_keys_come_from_the_hotkey_lookup`. C2: test helper `language_hotkey` + tests L1-L4. C3: D4/D5 algorithm, `[INFO]` line, doc comments `:122-127`, `:162-169`, comment in `parse_simple_binding` `:500-501` ("user binding first, then the auto-blocker") | D4, D5, D6 |
| `src/config/mod.rs` | C2: `cfg(test)` re-export of `from_ron_str_with_hotkeys` (`:17-20`) | first use is in C2; adding it in C1 would trip `unused_imports` under `-D warnings` |
| `src/engine/testing.rs` | C2: `Scenario::from_ron_with_language_hotkey`; `from_ron` doc `:139-141` points to it | D6 |
| `src/engine/scenarios.rs` | C2: T1. C3: `with_alt_shift_blocker` doc `:27-30` ("blocker first: the place where it used to shadow a binding on the same keys; the loader now appends it"). C4: const `DEFAULT_CONFIG` (the template's two bindings with the flag), T2-T10, flips F1-F3. C5: drop the KS-10 sentence in the comment `:1886-1888` | D7 |
| `src/engine/properties.rs` | C3: doc `:56-57` "in front, as the loader does" -> "in front of the bindings (`with_alt_shift_blocker`)" | stale after D5 |
| `src/engine/matching.rs` | C5: `HeldKeys::all_held`, `blocker_matches`, module doc `:1-14` (blockers: what Windows sees), tests U1, U2 | D2 |
| `src/engine/keys.rs` | C5: `all_held` for `KeyState` (scan `active` over `0..256`), module doc `:5-9`, test U3 | D2 |
| `src/engine/mod.rs` | C5: blocker branch in the `Combo` arm (`:345-353`); module docs rule 4 (`:106-118`) gains the blocker bullet, "first matching Immediate binding" -> "first matching Immediate user binding" | D1, D3 |
| `src/engine/env.rs` | C5: `os_key_down` doc `:32-41`: also asked, without overwriting anything, about held keys outside an auto-blocker's hotkey | D2 |
| `src/data/binding.rs` | C5: doc only: `allow_extra_modifiers` `:66-70` ("auto-blockers ignore it: they match only their hotkey on its own"), doc comment on `is_auto_blocker` `:71` (set only by `new_auto_blocker`; the engine matches it by what Windows sees held and lets it only suppress) | D1 |
| `README.md` | C3 + C5: `block_original_combo` row `:428` (texts below) | config meaning |
| `CHANGELOG.md` | C3: Fixed KS-18. C5: Fixed KS-05 (partly) + one sentence added to KS-18 | Keep a Changelog |
| `src/config/default_config.ron` | C3 + C5: flag comment `:119-125` (texts below) | config meaning |
| `src/hook/*`, `src/core/*`, `src/system/*`, `src/data/key_*`, `Cargo.toml` | none | adapter, registry, keys and features unchanged |
| `book/src/**` | none in this lane | phase book sync (Q5) |

## Commits, in order

Gate per commit: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Commits 2 and 4 are the declared red commits: fmt and clippy pass, exactly the listed tests fail with the predicted text, every other test passes. If any other existing test turns red, stop and report; do not edit its expectations. Each message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`; red commits quote the failing lines in the body.

Scenario failure texts come from `testing.rs:465-484` (`"{label}: expected the event to be {x}, it was {y}"`) and `:331-341` (`"{label}: expected to fire {expected:?}, fired {actual:?}"`); labels are `"{what} {key} at +{ms} ms"` (`testing.rs:383`).

**1. `Config: auto-blocker expansion takes the system hotkey lookup as a parameter`** (green, no behaviour change)
- D6 seam in `loader.rs`; the expansion keeps today's algorithm (backward-only check, blocker before its requester).
- Test `auto_blocker_keys_come_from_the_hotkey_lookup` (loader): `( bindings: [ (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true), (keys: "F13", action: ToggleCapsLock, block_original_combo: true) ] )` through `from_ron_str_with_hotkeys` with a local closure that answers `KeyCombination::from_keys([VK_CONTROL, VK_SHIFT])` for the two language actions and `None` otherwise; assert 3 bindings, exactly one with `is_auto_blocker`, whose `combination()` equals Ctrl+Shift (ToggleCapsLock gets none). Green before and after C3 (uses `filter`/`find`, not indices).

**2. `Tests: red-first: auto-blocker dedup over the whole binding list (KS-18)`** (RED)
- `loader.rs` tests: `fn language_hotkey(keys: &'static [VIRTUAL_KEY]) -> impl Fn(&BindAction) -> Option<KeyCombination>` (`Some` for `SwitchLanguage`/`SwitchLanguageBackward` only; it may replace C1's local closure); `fn blockers(b: &[Binding]) -> usize` counting `is_auto_blocker`. Kept local to `loader.rs` so `config` tests do not depend on `engine::testing` (no test-time cycle).
- `mod.rs` re-export; `testing.rs` `from_ron_with_language_hotkey` (builds the same stand-in from `keys`, loads with `from_ron_str_with_hotkeys`, then `with_bindings`).

| Test | Script / assertions (hotkey Alt+Shift unless stated) | PREDICTED RED on commit 1 |
|---|---|---|
| L1 `auto_blocker_is_skipped_when_a_binding_uses_the_hotkey_keys_in_either_order` (loader) | Loop, name in the message. X "flagged binding above": `[ (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true), (keys: "Alt+Shift", action: SwitchLanguageBackward) ]`; Y "flagged binding below": the two swapped; Z "conditional deferred binding below": X with the second entry `(keys: "Shift+Alt", action: SwitchLanguageBackward, trigger: Tap(200), when: AppEquals("code.exe"))`. Each: `assert_eq!(blockers(&b), 0, "{name}")`, `assert_eq!(b.len(), 2, "{name}")` | ``assertion `left == right` failed: flagged binding above`` / `left: 1` / `right: 0` |
| L2 `auto_blocker_is_skipped_when_the_flagged_binding_uses_the_hotkey_keys` (loader) | `[ (keys: "Alt+Shift", action: SwitchLanguageBackward, block_original_combo: true) ]`: `assert_eq!(blockers(&b), 0)`, `assert_eq!(b.len(), 1)` | `left: 1` / `right: 0` |
| L3 `auto_blockers_follow_the_bindings_of_the_file` (loader) | `[ (keys: "F13", action: SwitchLanguage, block_original_combo: true), (keys: "F14", action: DoNothing) ]`: `assert!(!b[0].is_auto_blocker, "the file's bindings come first")`; `b[0]` keys `[KeySpec::Key(VK_F13)]`, `b[1]` `[KeySpec::Key(VK_F14)]`; `b[2].is_auto_blocker` and `b[2].combination() == Some(&Alt+Shift)`; `b.len() == 3` | panic `the file's bindings come first` |
| L4 `auto_blocker_stays_when_a_binding_only_overlaps_the_hotkey` (loader; pin, **green**) | flagged CapsLock plus `(keys: "LAlt+LShift", ...)`, `(keys: "Ctrl+Alt+Shift", ...)`, `(chord: (keys: ["Alt", "Shift"]), ...)`, `(sequence: (steps: ["Alt+Shift", "F13"]), ...)`, all `action: SwitchLanguageBackward`: `assert_eq!(blockers(&b), 1)` | green on 1 and after 3 |
| T1 `user_binding_on_the_language_hotkey_fires_in_either_file_order` (scenarios; the audit's KS-18 repro) | For X, Y (L1) and W = L2's config: `Scenario::from_ron_with_language_hotkey(cfg, &[VK_MENU, VK_SHIFT]).with_context(name).check_nothing_stuck()`: down LAlt passed, nothing fired; down LShift suppressed, fired `[SwitchLanguageBackward]`; up LShift suppressed; up LAlt passed | `down LShift at +0 ms: expected to fire [SwitchLanguageBackward], fired [DoNothing]` with `context: flagged binding above` |

**3. `Config: an auto-blocker is skipped when any binding uses the hotkey's keys, and follows the file's bindings (KS-18)`** (green; makes L1-L3 and T1 green)
- D4 + D5 in `loader.rs`, docs per Touch set; `with_alt_shift_blocker` and `properties.rs` doc lines.
- README `:428` and template `:119-125` (C3 parts below); CHANGELOG Fixed KS-18 (C3 text below).
- Invariant: loader output = user bindings in file order, then distinct blockers; the engine is untouched.

**4. `Tests: red-first: auto-blockers act only when Windows sees exactly the hotkey, and fire nothing (KS-05, KS-18)`** (RED; predicted on commit 3)
- `scenarios.rs`: `const DEFAULT_CONFIG: &str` = `( bindings: [ (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true), (keys: "Shift+CapsLock", action: ToggleCapsLock) ] )` (the template's bindings, `default_config.ron:146-154`).
- Every new scenario starts with `.check_nothing_stuck()`; T2-T6, T9, T10 also `.check_press_symmetry()`.

| Test | Script | PREDICTED RED on commit 3 |
|---|---|---|
| T2 `shift_pressed_while_alt_and_tab_are_held_reaches_windows` (acc. 1) | `with_alt_shift_blocker(DEFAULT_PAIR)`: down LAlt passed; down Tab passed; down LShift passed; advance 500; repeat Tab passed; up Tab passed; down Tab passed; up Tab passed; up LShift passed; up LAlt passed; total `[]` | `down LShift at +0 ms: expected the event to be passed, it was suppressed` |
| T3 `shift_pressed_while_ctrl_and_t_are_held_reaches_windows` (acc. 1) | `from_ron_with_language_hotkey(DEFAULT_CONFIG, &[VK_CONTROL, VK_SHIFT])`: down LCtrl passed; down T passed; down LShift passed; up T passed; up LShift passed; up LCtrl passed; then the same with RCtrl, T, RShift; total `[]` | `down LShift at +0 ms: expected the event to be passed, it was suppressed` |
| T4 `alt_shift_pressed_while_a_letter_is_held_reaches_windows` | `with_alt_shift_blocker(DEFAULT_PAIR)`: down A passed; down LAlt passed; down LShift passed; up A passed; up LShift passed; up LAlt passed; total `[]` | same text as T3 |
| T5 `grave_blocker_acts_only_on_a_lone_grave` | `from_ron_with_language_hotkey(DEFAULT_CONFIG, &[VK_OEM_3])`: down A passed; down `` ` `` passed; up `` ` `` passed; up A passed; down LShift passed; down `` ` `` passed; up `` ` `` passed; up LShift passed (A2's Shift+` stays); down `` ` `` suppressed, nothing fired; up `` ` `` suppressed; total `[]` | ``down ` at +0 ms: expected the event to be passed, it was suppressed`` |
| T6 `ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing` (acc. 1 residue, 1.2) | `from_ron_with_language_hotkey(DEFAULT_CONFIG, &[VK_CONTROL, VK_SHIFT])`: down LCtrl passed; down LShift suppressed, nothing fired; down T passed (Windows gets Ctrl+T until item 1.2); up T passed; up LShift suppressed; up LCtrl passed; total `[]` | `down LShift at +0 ms: expected to fire [], fired [DoNothing]` |
| T7 `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order` (acc. 2, D3) | `USER = ( bindings: [ (keys: "Alt+Shift", action: SwitchLanguageBackward) ] )`. Run A: `with_alt_shift_blocker(USER)` (blocker first). Run B: `from_ron_str(USER)` then `push(Binding::new_auto_blocker(KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT])))` (blocker last). Each via `with_bindings`: down LAlt passed; down LShift suppressed, fired `[SwitchLanguageBackward]`; up LShift suppressed; up LAlt passed | run A: `down LShift at +0 ms: expected to fire [SwitchLanguageBackward], fired [DoNothing]` |
| T8 `sided_user_binding_fires_and_the_blocker_keeps_the_other_sides` (acc. 2, D3+D4) | `from_ron_with_language_hotkey` of `( bindings: [ (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true), (keys: "LAlt+LShift", action: SwitchLanguageBackward) ] )` with `&[VK_MENU, VK_SHIFT]`: down LAlt passed; down LShift suppressed, fired `[SwitchLanguageBackward]`; up LShift suppressed; up LAlt passed; down RAlt passed; down RShift suppressed, nothing fired; up RShift suppressed; up RAlt passed | `down RShift at +0 ms: expected to fire [], fired [DoNothing]` |
| T9 `key_whose_release_the_hook_missed_does_not_disable_the_blocker` (D2; Win+L) | `with_alt_shift_blocker(DEFAULT_PAIR)`: down L passed; `up_unseen("L")`; down LAlt passed; down LShift suppressed, nothing fired; up LShift suppressed; up LAlt passed | `down LShift at +0 ms: expected to fire [], fired [DoNothing]` (under the literal reading of Q2 it would fail on suppression after commit 5) |
| T10 `key_swallowed_by_a_binding_does_not_disable_the_blocker` (D2) | `with_alt_shift_blocker(DEFAULT_PAIR)`: down CapsLock suppressed, fired `[SwitchLanguage]`; down LAlt passed; down LShift suppressed, nothing fired; up LShift suppressed; up LAlt passed; up CapsLock suppressed | same text as T9 |
| F1 `auto_blocker_matches_its_modifiers_exactly` | `:1179` `.expect_fired(&[BindAction::DoNothing])` -> `.expect_nothing_fired()` | `down LShift at +0 ms: expected to fire [], fired [DoNothing]` |
| F2 `alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier` | `:1530`, `:1547` -> `.expect_nothing_fired()` | same text (run A) |
| F3 `shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press` | `:1660` -> `.expect_nothing_fired()`; `:1678` -> `.expect_total_fired(&[])` | same text |

Checked on commit 3 for each red: T2-T5 fail on suppression because `combo_matches` ignores non-modifier keys (`matching.rs:56-67`); T6-T10, F1-F3 fail on the fire list because the blocker's DoNothing is pushed (`mod.rs:355-359`). In T8 the user binding wins the LShift step on commit 3 (blocker appended after it), so the first failure is RShift.

**5. `Engine: an auto-blocker matches only the hotkey on its own and never takes a user binding's fire (KS-05, KS-18)`** (green; makes commit 4 green)
- `matching.rs`: `all_held` in `HeldKeys`, `blocker_matches` (D2), module doc. `keys.rs`: `all_held` impl. `mod.rs`: D1 branch (flag check first in the `Combo` arm; on match set `suppress`, never `fire`, never `immediate_fired`; `continue`), rule 4 docs. `env.rs`, `binding.rs` docs. Comment `scenarios.rs:1886-1888`.
- Unit tests (added here; they cannot compile before):
  - U1 `blocker_matches_only_what_windows_sees_as_its_hotkey` (`matching.rs`): combo `AnySide(Alt)+AnySide(Shift)`; `held(&[..])` = existing `KeyState` helper `:79-85`. True: `[LMENU, LSHIFT]` pressed LSHIFT, Windows says all down; `[RMENU, RSHIFT]` pressed RMENU; `[LMENU, LSHIFT, TAB]` with Windows saying TAB up. False: same with TAB down; `[LCONTROL, LMENU, LSHIFT]` with LCONTROL down; pressed `VK_A` with `[LMENU, LSHIFT, A]`; `[LMENU]` pressed LMENU (Shift not held). Grave combo `KeyCombination::new(VK_OEM_3)`: `[OEM_3]` true; `[A, OEM_3]` with A down false; `[LSHIFT, OEM_3]` with LSHIFT down false.
  - U2 `blocker_never_asks_windows_about_its_own_keys`: held `[LMENU, MENU, LSHIFT]` (generic slot set via `press(VK_MENU)`), pressed LSHIFT, `windows_has_down = |vk| panic!("asked about {vk:?}")`: returns true without panicking.
  - U3 `all_held_visits_every_held_code` (`keys.rs`): empty state -> true for any predicate; press LSHIFT and A, `sync_modifiers(&ShiftDown, None)` (writes `VK_SHIFT`, existing `ShiftDown` env `:342-357`): `all_held(|vk| [VK_LSHIFT, VK_SHIFT, VK_A].contains(&vk))` true; `all_held(|vk| vk != VK_A)` false.
- README `:428`, template, CHANGELOG (C5 parts below).
- Invariants: every event still ends in one outcome (adapter untouched); records/symmetry logic untouched (`mod.rs:453-485`, `:489-515`); no allocation (the scan iterates the table); table index bounded by `0..256`.

## Acceptance mapping

| Acceptance line | Covered by |
|---|---|
| Spec 1: blocker only when the held set exactly equals its combo and the pressed key belongs to it | T2, T3, T4, T5 (extra key down in Windows -> passes), T9, T10 (D2 reading: keys Windows does not have down do not count), U1, U2; existing `auto_blocker_matches_its_modifiers_exactly` (extra modifier) |
| Spec 2: dedup checks the whole list incl. the requester; never shadows a user binding on the same combo | L1 (above/below, any trigger/`when:`), L2 (requester), L4 (equality, not overlap), T1 (through the loader, fires in either file order), T7, T8 (D3: no shadowing even when a blocker is present) |
| Spec 3: blockers stay distinguishable | `is_auto_blocker` flag kept (`binding.rs:71`), read by the engine (C5) and asserted in L3, L4, C1 test; `[AUTO-BLOCK]` Display unchanged (`binding.rs:170-175`) |
| Acc. 1: Alt+Shift+Tab and Ctrl+Shift+T pass through with the blocker (scenario tests) | T2 (Alt, Tab held, Shift: every event passes), T3 (Ctrl, T held, Shift; both sides), T4 (rollover); third key in the modifiers-first order: T6 (T passes) and existing `alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier` (`scenarios.rs:1516`). The modifiers-first order still loses its second modifier until 1.2 (`roadmap.md:100-101`, `:232`; Q1). Manual M3, M4 |
| Acc. 2: a user binding on Alt+Shift still fires with the blocker present, in either file order | T7 (blocker literally present, both orders), T1 (loader: both file orders + the flagged binding itself), T8 (blocker present next to a sided binding); manual M2 |
| Acc. 3: tests for loader dedup | L1, L2, L3, L4, `auto_blocker_keys_come_from_the_hotkey_lookup`; existing `auto_blocker_deduped_when_two_bindings_want_the_same_combo` (`loader.rs:551-575`) |
| Roadmap A `roadmap.md:80` | all of the above |
| Checkpoint smoke 4 `roadmap.md:232` (A + 1.2) | M1 (bare hotkey blocked), M3, M4 (A's part); the rest is 1.2 |

## Manual checks (human only)

No automatable test needs a live hook, injection or a registry write; no `#[ignore]` stubs are added (as in A2, A3). Release daemon started with `swch open`; `osk.exe` visible to watch modifier state. Agents never change the hotkey: the owner sets it in Settings > Time & language > Typing > Advanced keyboard settings > Input language hot keys.

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | default config, hotkey Alt+Shift, Notepad | Alt down, Shift down, release both; Esc (a menu bar from the lone Alt is KS-12, 1.2); press CapsLock | layout unchanged by Alt+Shift; CapsLock switches it once; nothing stuck in osk | none (F1, T6 are the twins) |
| M2 | config: flagged `CapsLock` + `( keys: "Alt+Shift", action: SwitchLanguageBackward )` below it | start: read the banner; Alt+Shift x2; then swap the two entries and save (hot reload) and repeat | banner has no `[AUTO-BLOCK]` line and prints `[INFO] no auto-blocker for Alt+Shift: binding #1 uses the same keys` (`#0` after the swap, printed on reload); every Alt+Shift switches backward exactly once | none (T1, L1) |
| M3 | default config, hotkey Alt+Shift, three windows open | hold Alt, hold Tab until the switcher cycles, press Shift while Tab is still held, release Tab, tap Tab, release Shift, release Alt; then press Shift alone | the switcher steps backward after Shift; nothing stuck in osk; record whether the layout changed (input for the 1.2 spike, Q7) | none (T2) |
| M4 | default config, hotkey set to Ctrl+Shift by the owner, a browser | (a) Ctrl down, T down and held, Shift down, release all; (b) Ctrl, Shift, T in that order; (c) bare Ctrl+Shift | (a) every key reaches the browser (osk shows Shift lit), nothing stuck; (b) a new tab opens (Ctrl+T, the 1.2 residue), not a reopened one; (c) layout unchanged | none (T3, T6) |
| M5 | default config, hotkey Alt+Shift | Win+L, unlock, then immediately Alt down, Shift down, release (no `l` typed in between) | layout unchanged | none (T9) |

## Risks and edge cases

| Id | Risk | Handling |
|---|---|---|
| RK1 | `GetAsyncKeyState` returns 0 when the desktop is not active or under UIPI (non-elevated debug daemon, elevated foreground): every extra key reads up, so the blocker matches as on master (Shift swallowed even with Tab held) | fail-closed, debug builds only; the release daemon is elevated (`build.rs`) |
| RK2 | A non-modifier pressed while remapping was off is unknown to the engine, so D2 never asks about it: the blocker may swallow Shift while Windows has that key down | pre-existing class (A3 RK7); 2.3 owns a full resync |
| RK3 | Dedup ignores `when:`: with `( keys: "Alt+Shift", ..., when: AppEquals("code.exe") )` the hotkey is not blocked in other windows | spec-literal; README and CHANGELOG say it; Q4 |
| RK4 | Modifiers-first order still loses the second modifier: Alt, Shift, Tab -> Alt+Tab; Ctrl, Shift, T -> Ctrl+T; Ctrl, Shift, Esc -> Ctrl+Esc (Ctrl+Shift hotkey) | 1.2; T6 pins it; README keeps the 1.2 sentence |
| RK5 | An overlapping user binding with `block_default: false` (`LAlt+LShift`) still has its key swallowed by the blocker (D3 keeps blocker suppression) | workaround: bind the hotkey's exact keys (the blocker is then skipped, D4); 1.6/3.4 |
| RK6 | Grave hotkey ("4"): a grave typed while another key is down in Windows now reaches Windows, which may switch the layout | spec-literal; rare hotkey; Q2 notes it |
| RK7 | Sided blockers: D2 accepts a generic slot (`VK_SHIFT`) only through an `AnySide` key. All reachable blockers are `AnySide` or non-modifier (`registry.rs:70`, `:183-186`, `:205`); a future sided blocker would see the generic slot as down in Windows and never match | invariant stated in the `blocker_matches` doc; 1.4 and new-feature hotkeys must build `AnySide` |
| RK8 | Reload mid-press (a blocker appears or disappears while Shift is held and swallowed) | blockers are stateless; the key's record keeps its repeats and key-up symmetric (`mod.rs:453-485`) |
| RK9 | `swch off`/`on`, foreground change, shutdown during a pending gesture | blockers arm nothing; `on_foreground_changed` (`mod.rs:324-327`) and the off/on paths are untouched |
| RK10 | Hot-path cost | bounded scan of a 256-bool table + at most a few `GetAsyncKeyState` calls, only on the blocker's own key-downs with all its keys held; no allocation |
| RK11 | Repeats (KS-06, KS-10): a blocker matching a repeat changes nothing, the record decides; it no longer fires DoNothing on repeats | intended; KS-10 stays with 1.1 for user bindings |

**R1-R10 (code-reviewer areas).**
- R1: adapter untouched; engine still returns one `Decision` per event; new table access iterates `0..256` inside `KeyState`.
- R2: no `SendInput`.
- R3: no new shared state; `KeyState` stays inside `Mutex<Engine>`; the `Env` call happens under the existing lock and does not pump messages.
- R4: no atomics.
- R5: no new `Binding` option or setter; `new_auto_blocker` unchanged; harness constructor only.
- R6: no new enum variant; the `[INFO]` text uses `KeyCombination`'s `Display` (`key_combination.rs:71-81`).
- R7: `update_bindings` and its single publish are unchanged; the loader order changes (D5) and blockers stop depending on order (D3). The rule text "auto-blockers rely on load order among equal counts (KS-18)" goes stale (Q6).
- R8: D2 compares the table's sided and generic codes through `KeySpec::accepts`, the A2 canonical form.
- R9: registry read path unchanged and read-only; new tests never touch the registry (D6).
- R10: message loop and shutdown untouched.

## Docs impact

**This lane (developer).**

README `:428`, final text after C5. C3 changes "no-op binding" to "blocker" and adds the sentences "It is not added when ..." and "Auto-blockers are listed last at startup."; C5 adds "The blocker acts only when ..." and "The blocker never fires anything ...". The first two sentences and the closing 1.2 sentences are today's text.

> Auto-install a blocker for the OS's default hotkey for this action's underlying function. Use when remapping language switching so Windows' own `Alt+Shift`/`Ctrl+Shift` go silent. The blocker acts only when Windows would see the hotkey's keys and no other key held: Alt, then Shift is blocked, while Alt, Tab, then Shift (Tab still held) reaches Windows. It is not added when any `keys` binding in the file, the flagged one included, uses exactly the hotkey's keys: that binding gets the hotkey, and where its `when:` does not match, the hotkey is not blocked. The blocker never fires anything and never keeps a binding of yours that matches the same press from firing. Auto-blockers are listed last at startup. Until roadmap item 1.2 the blocker swallows the hotkey's second modifier (Shift pressed after Alt) for that key's whole press, auto-repeats and release included, even after Alt is released; it still counts as held, so `Alt+Shift+X` bindings fire.

`default_config.ron`, after `:125` (ASCII). C3:
```
//                         It is not added when a binding in this file
//                         uses the hotkey's keys itself.
```
C5:
```
//                         It acts only on the hotkey pressed on its own,
//                         so Alt+Tab, then Shift still reaches Windows.
```

`CHANGELOG.md` `## [Unreleased]` / Fixed:
- C3, after the KS-14 entry: "KS-18: `block_original_combo` no longer adds an auto-blocker when a `keys` binding anywhere in the file uses the same keys as the Windows hotkey, in any order and with any trigger or `when:`, the flagged binding itself included. That binding now runs its action wherever it sits in the file; a binding on `Alt+Shift` below the flagged one, or `( keys: "Alt+Shift", action: SwitchLanguageBackward, block_original_combo: true )` on its own, never did. The console says so at load (`[INFO] no auto-blocker for Alt+Shift: binding #1 uses the same keys`); with a `when:` on that binding the Windows hotkey is not blocked where the condition does not hold. Auto-blockers are listed after the file's bindings at startup."
- C5, appended to the KS-18 entry: "An auto-blocker also no longer runs in place of a binding of yours that matches the same press: with `LAlt+LShift` bound, that binding fires, and the `Alt+Shift` blocker still swallows Shift pressed with the right Alt."
- C5, after the KS-04 entry: "KS-05 (partly; roadmap item 1.2 fixes the rest): an auto-blocker swallows a key only when Windows would see the hotkey's keys and no other key held. Shift pressed while Alt and Tab are held (to step back in the Alt+Tab switcher), Shift pressed while Ctrl and T are held, and Alt+Shift pressed while a letter is still held in fast typing now reach Windows. A key Windows does not have down (one a binding swallowed, or one whose release went to the lock screen) does not count. Pressed in the order Alt, Shift, Tab, the hotkey's Shift is still swallowed and the application gets Alt+Tab (Ctrl+T for Ctrl, Shift, T with the `Ctrl+Shift` hotkey) until item 1.2."

**Phase book sync (doc-writer), not this lane:** `book/src/config/matching.md` §15.1 (placement, no firing, what Windows sees), §15.3 (whole list, requester, equality, `[INFO]` line), §15.4 (drop the KS-18 warning; dedup-ignores-`when:` becomes documented behaviour; first warning: only the modifiers-first residue remains), §16.1 item 1; `config/structure.md:44`, `:103`; `config/example.md:15`, `:41-42`, `:234`, `:245`; `config/triggers-conditions.md:43`; `config/known-issues.md` rows 41, 42 (§19.6); `config/errors.md:158` (new `[INFO]` line); `architecture.md:254`, `:277`, `:400`, `:527`; `audit.md` status lines of KS-18 and KS-05 (Q6); `roadmap.md:80` status.

## Questions for the orchestrator

1. **Acceptance 1 scope.** "Alt+Shift+Tab and Ctrl+Shift+T pass through" is proven for every order in which another key is down in Windows before the hotkey completes (T2-T4) and for the third key of the modifiers-first order (T6, `scenarios.rs:1516`). In the modifiers-first order the second modifier is still swallowed, as `roadmap.md:100-101` states and smoke 4 (`:232`) assigns to "A, 1.2". Recommendation: accept this mapping; 1.2 owns the rest. Not blocking.
2. **D2 reading of "held set".** The cut counts only keys Windows has down besides the hotkey's own, which keeps the blocker working with a swallowed key (T10) or a key whose release went to the lock screen (T9, M5); the literal engine-held reading would leave it off until that key is pressed again. Recommendation: accept D2. Fallback if vetoed: drop the `windows_has_down` clause (`blocker_matches` takes no closure, `env.rs` doc unchanged), drop T9/T10 or flip them to expect "passed", and add RK "stale key disables the blocker until 2.3". RK6 (grave typed during rollover now reaches Windows) is the same under both readings. Not blocking.
3. **D3 flips three pinned scenarios** (F1-F3: blocker `fired [DoNothing]` -> nothing). Recommendation: accept; it is what "never shadows a user binding" needs once A2's sided keys and `allow_extra_modifiers` let a user binding overlap a blocker without being equal. Not blocking.
4. **Dedup ignores `when:` and `trigger`** (spec-literal; RK3). Recommendation: keep the spec; README/CHANGELOG state the consequence; KS-35 (1.2) revisits blocker conditions. Not blocking.
5. **Book pages** are left to the phase book sync (list above), as in A2 (Q2) and A3 (Q4). Recommendation: yes. Not blocking.
6. **Status lines and rule texts after merge.** `audit.md`: "KS-18: Fixed in <sha> (fix/auto-blockers)"; "KS-05: Partly fixed in <sha> (fix/auto-blockers): exact auto-blocker; the swallowed second modifier remains for 1.2". `.claude/agents/code-reviewer.md:124` (R7) and `.claude/agents/project-analyst.md:60` say "auto-blockers rely on load order among equal counts (KS-18)"; after this lane they should read "auto-blockers (`is_auto_blocker`) only suppress, match only the hotkey Windows would see on its own, and do not depend on list order". `.claude/agents/architecture-critic.md:148-149` is satisfied as written. Recommendation: doc-writer for the audit, orchestrator for the agent files (agents do not edit them). Not blocking.
7. **Research for the 1.2 spike (not needed here).** Query for `researcher`: "Windows input-language hotkey (HKCU\Keyboard Layout\Toggle Hotkey=1/2): exactly when does it fire - on key-down or on release of the modifiers, and does a non-modifier key that is already down when the second modifier goes down, or pressed before release, cancel it? Primary sources or reproducible tests." M3 records one data point. Recommendation: run it at the start of 1.2. Not blocking.

## Cut readiness checklist

Spec
- [x] KS-05 (`audit.md:218-235`) and KS-18 (`audit.md:445-457`) read; KS-18 repro = T1/L1/L2, KS-05 three-key cases = T2/T3/T4 (+ residue T6), AltGr+Shift = existing A2 behaviour kept by D2 (Win32 section)
- [x] Every book reference re-located (table); KS-05 marked partly fixed on master, KS-18 live
- [x] Goal, non-goals, dependent items (1.2, 1.4, 1.6, 2.3, 3.4) stated

Design
- [x] Every decision has rationale and rejected alternatives (D1-D6)
- [x] Owner decisions: exact modifiers kept (blockers stay exact, D2; `allow_extra_modifiers` untouched for user bindings); repeat defaults untouched; RON unchanged; `println!` only (no logging change); licence n/a
- [x] Thread named for every path (Threads table); in the LL callback only table reads and bounded `GetAsyncKeyState`, no lock beyond `with_engine`, no allocation
- [x] N/A install/uninstall/timers: none added or moved
- [x] N/A `SendInput`: none added
- [x] Sided/unsided: D2 via `KeySpec::accepts` on sided and generic slots; chords and sequences untouched (dedup counts `Combo` only; D3 keeps them unshadowed)
- [x] Reload, `swch off`/`on`, foreground change, shutdown: RK8, RK9
- [x] No new shared state or atomics; no `static mut`
- [x] No new module cycle (config -> core/data and engine(test) -> config exist; loader test helper kept local); logic in `engine::matching`/`engine`, reached by scenarios

Win32
- [x] `GetAsyncKeyState` path, feature present (`Cargo.toml:44`), thread, error shape, doc link and quotes; no new API
- [x] Elevation/UIPI: RK1; pipe DACL and token inheritance n/a

Commits and tests
- [x] Red-first with exact predicted text (commits 2, 4), predicted on the commit before each
- [x] Tests run through `Engine::on_key`, `FakeEnv`, fake clock; no hook, injection or registry access in new tests
- [x] N/A `#[ignore]`: M1-M5 are human checks with automated twins
- [x] Every commit passes fmt, clippy, test except the declared red commits 2 and 4
- [x] Bisectable; README, template and CHANGELOG ride with commits 3 and 5

Acceptance
- [x] Every acceptance line mapped (table)
- [x] Every question has a recommendation; none blocks

## Developer addendum (implementation round 1, branch `fix/auto-blockers`)

The critique (`critique.md`) has no Critical or Important remark. Its Minor remarks and the open question on RK1 end as follows; details and test names in `impl.md`.

- **M1** (`VIRTUAL_KEY(raw)` in the engine): RESOLVED by stating the bound at the only construction, `src/engine/keys.rs:204-209` (`KeyState::all_held`): the code is the index of the engine's own 256-entry table (the inverse of `slot`), iterated as `0..256u16`, so it is always a code the hook or the resync stored. The cut's signature (`accept: impl FnMut(VIRTUAL_KEY)`) is kept because `KeySpec::accepts` and `Env::os_key_down` both take `VIRTUAL_KEY`; any other signature moves the same construction into `matching.rs`. The reviewer may still rule on CLAUDE.md principle 7.
- **M2** (`'static` trait object): RESOLVED. Both stand-ins own their `KeyCombination` (built from `keys` up front, then a `move` closure): `Scenario::from_ron_with_language_hotkey` (`src/engine/testing.rs:154`) and the loader helper `language_hotkey` (`src/config/loader.rs:632`). Commit 2 compiled; its reds were the predicted runtime assertions.
- **M3** (swallowed extra modifier not pinned): RESOLVED. U1 has the case `[LCONTROL, LMENU, LSHIFT]` with Windows reporting LCONTROL up -> true (`src/engine/matching.rs:262-265`) next to the LCONTROL-down -> false case, and the new scenario `modifier_swallowed_by_a_binding_does_not_disable_the_blocker` (a `Win` Tap swallows LWin; red on commit 3: "down LShift at +0 ms: expected the event to be suppressed, it was passed"). The passed-extra-modifier side stays pinned by the second half of `auto_blocker_matches_its_modifiers_exactly`.
- **M4** (switcher wording): RESOLVED. The CHANGELOG KS-05 entry names the usual switcher order (Alt held, Tab released, then Shift+Tab) as part of the 1.2 residue; the README row says the same in its 1.2 sentence; the template comment uses "Shift pressed while Alt and Tab are held". New pin `shift_pressed_after_tab_was_released_is_still_swallowed` (red on commit 3: "down LShift at +0 ms: expected to fire [], fired [DoNothing]"). Manual check M3 gains the expected-residue step (in `impl.md`).
- **M5** (Breaking under Fixed): RESOLVED. `CHANGELOG.md` `### Changed` has a **Breaking:** entry: a `when:`-restricted binding on the hotkey's exact keys now leaves the hotkey unblocked in other windows (KS-18), and a grave typed while another key is held now reaches Windows (KS-05, RK6).
- **Open question 3 (RK1 text)**: accepted, no code change. Corrected RK1: if every `GetAsyncKeyState` read fails (non-elevated debug daemon, elevated foreground window), `sync_modifiers` clears every passed modifier on each event, so `keys_down` fails and the blocker never matches: it fails open, as it did before this lane. Debug builds only; the release daemon is elevated.
- Open questions 1, 2, 4-7 were not answered by the orchestrator; the implementation follows the cut's recommendations (D2 kept, F1-F3 flipped, dedup ignores `when:`/`trigger`, book pages left to the phase sync).
