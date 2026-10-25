# Cut: p1-1.2-mask-key — roadmap 1.2 (KS-12, rest of KS-05, KS-35)

Tree read: `D:/key-switch-rs/key-switch-rs`, `master` = `ece1767` (merge of `feat/one-winner`; `.git/refs/heads/master`). Items 1.1, 1.3 and 1.6 are merged; the book's line numbers (`7cb3521`, `1dfd80a`) are stale and re-located below. Branch to cut: `feat/mask-key` from `master`.

## Verdict
CUT READY

## Spec re-located
| Spec reference (book @7cb3521 unless noted) | Current file:line @ece1767 | Status | Note |
|---|---|---|---|
| Roadmap 1.2 / KS-35: `src/data/binding.rs:95`, `new_auto_blocker` hard-codes `Condition::Always` | `src/data/binding.rs:131-143` (`Condition::Always` at :137) | moved | Defect present |
| KS-35: `src/config/loader.rs:185`, blocker pushed without the owner's condition | `src/config/loader.rs:357`, inside `expand_with_auto_blockers` :323-364 | moved | Dedup is whole-file since KS-18 (:350-358) |
| KS-12: `src/hook/keyboard_hook_callback.rs:294-297`, only the payload key is suppressed | decision `src/engine/mod.rs:561-568` (`mark_blocked`/`mark_passed`); execution `src/hook/keyboard_hook_callback.rs:257-262` (`LRESULT(1)`) | moved (restructured into the engine) | The modifier's key-down still passes (`engine/mod.rs:563-567`) |
| KS-12: "the code has no mask key anywhere" | `Decision::mask` reserved and never set: `src/engine/mod.rs:276-279`; `carry_out` drops it with `..`: `src/hook/keyboard_hook_callback.rs:187-195` (:188); adapter doc :20-23 | moved | Defect present |
| KS-12: `PostMessage` at `src/core/windows_actions.rs:190-203` | `src/core/windows_actions.rs:231-244` | moved | Injects nothing |
| KS-12: `SwitchLanguage*` at `src/system/system_function.rs:45-82` | `src/system/system_function.rs:42-79` | moved | Posts `WM_INPUTLANGCHANGEREQUEST`, injects nothing |
| KS-12: Win+Q example, `default_config.ron:143-146`, `README.md:155-158` | `src/config/default_config.ron:222-225`, `README.md:192-195` | moved | |
| KS-05: `src/config/loader.rs:172-191` | `src/config/loader.rs:323-364` | moved | Exact (A) |
| KS-05: `keyboard_hook_callback.rs:234`, the blocker swallows Shift | `src/engine/mod.rs:405-423` (`decision.suppress = true` at :420); `src/engine/matching.rs:92-101` (`blocker_matches`) | moved | Rest of KS-05 pinned as current behaviour by `engine::scenarios::ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing` (`scenarios.rs:2579-2600`) and `shift_pressed_after_tab_was_released_is_still_swallowed` (:2603-2629) |
| KS-05: the default flag at `default_config.ron:127-131` | `src/config/default_config.ron:206-210`; flag comment :160-173 (KS-35 note :172-173) | moved | |
| KS-05: `src/system/registry.rs:68-78`, `:192-199` | `src/system/registry.rs:60-79` (fallback :70), `:181-189` (:183-184) | moved | KS-17 (value "3") stays for 1.4 |
| Blocker `when:` evaluated before its match | `src/engine/mod.rs:409` | new since A | Moves after the match (D8) |
| Blockers kept out of the election | `src/engine/winner.rs:64-66` | new since 1.6 | |
| Schema field `block_original_combo: bool` | `src/config/schema.rs:52-53` | unchanged | |
| Loader mapping | `src/config/loader.rs:442` (`.with_block_original_combo(raw.block_original_combo)`) | — | |
| `Binding.block_original_combo`, `is_auto_blocker`, builder, Display | `src/data/binding.rs:78`, `:104-107`, `:188-191`, `:250-255` | — | |
| Sentinel | `src/core/constants.rs:15-32` | unchanged | |
| `SendInput` builder pattern (`press_vk`) | `src/core/windows_actions.rs:190-229` (call :220, count check :221-228) | moved | Template for the mask builder |
| Adapter re-entry claim ("re-entered only while waiting in a message-retrieving call") | `src/hook/keyboard_hook_callback.rs:56-69` | unchanged | Wrong for `SendInput` on this thread (D10) |
| Book `matching.md` §15.4 warnings | `book/src/config/matching.md:84-91` | — | |
| Book known-issues rows 31, 39, 43, 66 | `book/src/config/known-issues.md:75, 89, 93, 134` | — | |
| Book `architecture.md` "mask never set" | `book/src/architecture.md:268, 271, 544` | — | Book sync, not this lane |
| Roadmap open question 6 (check the hypothesis by hand before v2) | `book/src/roadmap.md:670` | superseded by the brief | v2 ships opt-in and experimental, so that the owner can run the check |

No part of the spec is already fixed. KS-12 and KS-35 are open, and KS-05 is partly fixed.

## Goal and non-goals
**After the lane:**
1. When the hook swallows an event while a Win or Alt key whose key-down reached Windows is held, the hook injects vk 0xE8 down and up, tagged with the sentinel, before it returns. A release of Win or Alt after Win+Q, Alt+Shift (default blocker), `Alt+J -> DoNothing` and similar no longer opens Start or activates the menu bar (KS-12).
2. `block_original_combo: true | false` keep their meaning. `block_original_combo: MaskOnly` (experimental) is an auto-blocker that never swallows and injects the mask on an exact match. `Ctrl+Shift+T`, `Alt+Shift+X` and `Alt+Shift+Tab` reach applications. Whether Windows then skips the layout switch is the owner's manual check (KS-05 v2).
3. An auto-blocker acts only where the `when:` of a binding that asked for it holds (KS-35).

**Non-goals (left to later items, noted as follow-ups):**
- KS-19 (`Alt+J -> PressKey("Left")` arrives as Alt+Left): 3.1. Item 3.1's `build_inputs` absorbs `menu_mask_inputs`.
- Known-issues row 66 / CHANGELOG lines 157-158: a passing `Tap`/`Hold`/`DoubleTap` on Alt or Win alone still opens the menu or Start. Nothing is swallowed, so the mask rule does not apply, and a mask cannot be placed before a key-up that passes without re-injecting that key-up. Follow-up with no roadmap item yet; candidate 3.1.
- The default stays `true`, so with the default config Alt, Shift, X still reaches the application as Alt+X. Only `MaskOnly` fixes that (the rest of KS-05, opt-in).
- KS-17 (registry value "3"): 1.4. `Layout Hotkey` and Win+Space blockers: new feature 7, which builds on `BlockerMode`. Scan codes and `KEYEVENTF_EXTENDEDKEY` (KS-36): 3.1.
- Masking once per Win/Alt press instead of per swallowed event (Q2). Inheriting the owner's `trigger` (by design, see D7).
- `architecture.md`, `audit.md` status lines and the roadmap status block go to the phase book sync (doc-writer).

**Dependents:** 3.1 (single input module: the mask builder moves there), feature 7 (more blocker hotkeys reuse `BlockerMode`), Checkpoint-1 smoke items 4 and 5.

## Decisions

### D1: The mask decision lives in the engine and keeps no state for v1
**What.** `Decision::mask = true` for a key event (key-down, auto-repeat or key-up) when `decision.suppress` is true and `KeyState::alt_or_win_down_in_windows()` holds. That method is true when some key of `[VK_LMENU, VK_RMENU, VK_MENU, VK_LWIN, VK_RWIN]` is `active` and its `last_down` is not `Suppressed`/`Forgotten`. It is computed **after** the event's record update: `engine/mod.rs:536-568` on a key-down, after `release`/`take_blocked` on a key-up (`:574-596`). An Alt or Win key-down that is itself swallowed is therefore already `Suppressed` and does not count, which is the spec's "false when the modifier's own key-down was suppressed". `on_timer` never sets it.
**Why.** The spec defines it this way, and the acceptance line says "exactly when". `sync_modifiers` (`keys.rs:97-105`) rewrites `active` from `GetAsyncKeyState` for every non-swallowed modifier at the start of every event, so for those keys `active` equals Windows' view. The engine therefore needs no new `Env` call and stays pure. A modifier the resync found held with no record (pressed before the daemon started, or held through `swch on`, `reset_from_os` `keys.rs:117-130`) counts, which is correct because Windows has it down. The generic `VK_MENU` slot is included because the resync writes it from the OS (`keys.rs:32`).
**Rejected.** (a) Asking `env.os_key_down` for the four keys: equivalent after the resync, but it adds Win32 calls on the hook thread. (b) AutoHotkey's stateful "disguise at the Win/Alt key-up": needs per-press state across events, reload, `off`/`on` and foreground changes, and the spec asks for injection at the suppressed event. (c) Excluding the event's own key by code: this mis-handles a modifier held before the hook saw it whose unmarked repeat a binding swallows (engine rule 5, `mod.rs:549-560`). Windows has that key down, so the event must mask; the record-based rule does.
**Cost.** A swallowed auto-repeat while Win or Alt is held masks again, up to the repeat rate (~30/s). See Q2.

### D2: The adapter injects the mask first in `carry_out`, outside the lock, and never in tests
**What.** In `src/hook/keyboard_hook_callback.rs`:
- `fn menu_mask_inputs() -> [INPUT; 2]` builds two `INPUT_KEYBOARD` records: `wVk: VIRTUAL_KEY(constants::MENU_MASK_VK)`, `wScan: 0`, `dwFlags: KEYBD_EVENT_FLAGS(0)` and then `KEYEVENTF_KEYUP`, `time: 0`, `dwExtraInfo: injected_sentinel()`.
- `fn inject_menu_mask() -> u32` is the real `SendInput(&menu_mask_inputs(), size_of::<INPUT>() as i32)` with a `// SAFETY:` comment. It is always compiled.
- `fn send_menu_mask()`. `#[cfg(not(test))]`: calls `inject_menu_mask()`, and on a count other than 2 logs `eprintln!("[WARN] SendInput inserted {sent} of 2 menu mask key events")`. `#[cfg(test)]`: only increments a test-only `thread_local! static MASKS_SENT: Cell<u32>`.
- `carry_out` destructures `let Decision { suppress: _, fire, timers, mask } = decision;` (no `..`, so a future field fails to compile), then calls `if mask { send_menu_mask(); }` **first**, then the timers, then the actions.

**Why.** "The SendInput call stays outside the engine lock": `carry_out` runs after `decide_event` returned (`:257-259`), and the lock and the bindings snapshot are dropped by then (`:170-182`). The mask goes first because it belongs to the event being decided, and every action's injected keys must come after it. The `cfg(test)` seam is required for safety: `extern_callback_swallows_a_bound_key_only_while_remapping_is_on` (`:401-461`) drives `carry_out` with the live `HookEnv`. If the owner holds Win or Alt while `cargo test` runs, F13's swallowed key-down would mask, and an unseamed `carry_out` would inject input on the owner's machine. The seam makes that impossible whatever the owner holds. The short-count log copies `press_vk` (`windows_actions.rs:221-228`) and the code-reviewer's R2 rule ("checks the returned count").
**Rejected.** Sending from the engine (it is `forbid(unsafe)`, does no Win32 and holds the lock). Sending after the actions (an action's own keys would precede the mask). Sending in `keyboard_hook_callback` outside `carry_out` (a decision field consumed in two places). Silently ignoring the count (R2).
**Cost.** One `SendInput` of two events on the hook thread per masked event (see Risks: third-party hooks).

### D3: The mask key is vk 0xE8 and its constant lives in `core/constants.rs`
`pub const MENU_MASK_VK: u16 = 0xE8;` goes next to the sentinel, with a doc comment: unassigned per Microsoft's table, AutoHotkey recommends `vkE8`, and why the sentinel tags it. It is a `u16`, not a `VIRTUAL_KEY`, because CLAUDE.md principle 7 builds `VIRTUAL_KEY(raw)` only at the hook boundary, which is `menu_mask_inputs`. The roadmap's 1.2 file list names `src/core/constants.rs`. Rejected: Ctrl (AutoHotkey v2's default `vk11sc01D`), which changes modifier state for apps and our own matching, and `VK_PACKET` 0xE7, which carries Unicode text.

### D4: The config surface is `block_original_combo: true | false | MaskOnly`
**What.** `src/config/schema.rs`:
```rust
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
pub(crate) enum RawBlockOriginalCombo {
    #[serde(rename = "true")] On,
    #[default] #[serde(rename = "false")] Off,
    MaskOnly,
}
// RawBinding: #[serde(default)] pub block_original_combo: RawBlockOriginalCombo,
```
The declaration order `true`, `false`, `MaskOnly` fixes the error text. A misspelling gives ``Unexpected variant named `Maskonly` in enum `RawBlockOriginalCombo`, expected one of `true`, `false`, or `MaskOnly` instead``, and `Some(true)` gives the same message for `` `Some` `` (it used to be `Expected boolean`).
**Why it parses (verified by reading ron 0.10.1, the locked version, `Cargo.lock:185-186`).** serde's derived enum calls `deserialize_enum` → `Enum::variant_seed` → `deserialize_identifier` (`ron-0.10.1/src/de/mod.rs:731-766, 912-925`). That calls `Parser::identifier` (`src/parse.rs:883-953`), which accepts `true` and `false` as ordinary identifiers, with no keyword check. The error message comes from `error.rs:212-237` and `OneOf` at :443-465; `Identifier` prints `` `true` `` without `r#`, because it is a valid identifier (:469-482). Red-first tests in C5 pin all three values and both messages.
**Rejected.**
- `#[serde(untagged)] enum { Flag(bool), Mode(..) }`: it buffers into serde's `Content`. ron's `deserialize_any` passes a bare identifier as a string only when the visitor's type name equals `"serde::__private::de::content::Content"` (`de/mod.rs:26-27, 173-192`). With the locked serde 1.0.228, `Content` lives in `serde_core` (`serde-1.0.228/src/private/de.rs:220`), so the canary misses, `MaskOnly` arrives as unit, and the identifier is lost: any word would load.
- A separate field (`blocker_mode:`): allows meaningless combinations with `false` and is less discoverable. The brief prefers a value of `block_original_combo`.

### D5: `MaskOnly` lets the hotkey through, masks on the exact match, and owes one more mask
**What.**
- A `MaskOnly` blocker matches by the unchanged `blocker_matches` (`matching.rs:92-101`), with its condition asked after the match (D8). It never sets `suppress`.
- On a matching **first** key-down (`!was_repeat`) it sets `decision.mask = true` and `self.mask_owed = true`.
- `Engine::on_key` takes `mask_owed` at entry (`std::mem::take`) and ORs it into the decision of that next key event, whatever its key or direction. A repeat therefore never masks through `MaskOnly` itself, but the first event after the match does: usually the release of Shift or Alt, or T in Ctrl+Shift+T.
- `Engine::reset` clears `mask_owed`. `on_foreground_changed` and `on_timer` leave it.

**Why the second mask.** The order of events injected from inside the hook against the event in the hook is not documented by Microsoft (`(unverified)`). AutoHotkey's `hook.cpp` `AllowIt()` relies on the opposite of "after": *"Since the below call to KeyEvent() calls the keybd hook recursively, a quick down-and-up is all that is necessary to disguise the key. This is because the OS will see that the keystroke occurred while ALT or WIN is still down because we haven't done CallNextHookEx() yet."* Under that model, a mask sent while Shift's key-down is in the hook lands **before** Shift. Windows then sees Alt, E8, Shift, release, which may still be a clean Alt+Shift. The owed mask, sent during the next event, lands after Shift's key-down and before the release under either model. The match-time mask covers the opposite model and meets the acceptance line "sets mask on an exact match". For v1 the order does not matter, because the swallowed event never reaches Windows.
**Non-modifier hotkey.** `MaskOnly` on a hotkey that is not `is_modifier_only()` (the `` ` `` hotkey, registry value "4", `key_combination.rs:75-77`) falls back to `Swallow`. The loader warns: `binding #N: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true`. Reason: the clean-release hypothesis is about modifier chords, and letting a lone `` ` `` through would both type it and very likely switch the layout.
**Rejected.**
- Suppressing Shift and re-injecting Shift followed by E8 (AutoHotkey-style replacement): the modifier becomes injected input (`LLKHF_INJECTED` seen by games and other hooks), and the engine would need a "suppressed but down in Windows" record.
- Only the owed mask: under the "after the current event" model it lands after the release, which is too late.
- Masking on every matching repeat: redundant injections.

**Cost.** One `bool` in `Engine` and at most two masks per hotkey press.

### D6: Data model: `BlockerMode`, `Option<BlockerMode>` twice, no positional bloat
In `src/data/binding.rs`:
```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockerMode { Swallow, MaskOnly }        // Display: "swallow", "mask only"
pub block_original_combo: Option<BlockerMode>,    // was bool: what this binding asks for
pub(crate) auto_blocker: Option<BlockerMode>,     // replaces is_auto_blocker: bool
pub(crate) fn is_auto_blocker(&self) -> bool
pub(crate) fn new_auto_blocker(combination: KeyCombination, mode: BlockerMode) -> Self  // condition Always
pub fn with_block_original_combo(self, mode: Option<BlockerMode>) -> Self
```
- Display of a blocker: `[AUTO-BLOCK] {keys:<24} -> (blocked){cond}{trigger}{extra}` for `Swallow` (unchanged text) and `-> (masked, experimental){cond}...` for `MaskOnly`. `cond` already shows a non-`Always` condition (`binding.rs:224-228`).
- `duplicates` (`:174-181`) uses `is_auto_blocker()`.
- Removing the field makes the compiler find all 15 reads: `winner.rs:64`, `engine/mod.rs:405`, `binding.rs:175, 176, 250`, and `loader.rs` tests 835, 844, 860, 879, 905, 919, 945, 1010, 1021, 1601.
- The engine uses `if let Some(mode) = binding.auto_blocker`.

Why: R5 allows at most two positional constructor arguments, so the condition goes on with `.with_condition`. R6 requires a `Display` arm for every new variant. `is_auto_blocker` stays crate-private (`code-reviewer` §7). Rejected: keeping `is_auto_blocker: bool` plus a separate mode field, which adds a field that means nothing when the flag is false, and storing the blocker's mode in its own `block_original_combo`, which confuses "asks for" with "is".

### D7: `when:` inheritance: one blocker per (hotkey, mode), with the owners' conditions OR'ed
**What.** `expand_with_auto_blockers(user_bindings, system_combination, warnings: &mut Vec<String>) -> Vec<Binding>` (the warnings are threaded from `parse_ron_str` `:292-296`).
1. Requests, in file order: for each binding with `block_original_combo = Some(requested)` and `system_combination(&action) = Some(hotkey)`, the mode is `requested`, except `MaskOnly` on a hotkey that is not modifier-only, which becomes `Swallow` plus a warning (D5).
2. Hotkeys in first-request order. For each hotkey, the KS-18 check runs once (`:350-356`, unchanged). If it hits, one `[INFO]` line and no blocker. Otherwise, for each mode in first-request order: the condition is `Condition::Always` if any request's condition `is_always()`; else the distinct conditions in file order (`PartialEq`, `condition.rs:32`), one of them as it is, several as `Condition::Or(..)`. Push `Binding::new_auto_blocker(hotkey, mode).with_condition(cond)`.

The owner's `trigger` is not inherited: the blocker acts on the hotkey's own press, whatever the owner's gesture.
**Why.** Blockers suppress or mask by OR across the list, so a union of conditions is the right semantics for several owners. One blocker per hotkey and mode keeps the startup listing short and evaluates one condition per match. A hotkey asked for with both `true` and `MaskOnly` gets two blockers; where both conditions hold the hotkey is swallowed, and the extra mask is harmless (Q6).
**Rejected.** One blocker per owner: duplicate listings and duplicate evaluations. Dropping conditional owners: violates KS-35.

### D8: A blocker's condition is evaluated only after its hotkey matches
`engine/mod.rs:405-423` becomes `if matching::blocker_matches(..) && binding.condition.evaluate(&foreground) { act by mode }`. Why: with inherited conditions, evaluating first (`:409`) would capture the foreground (`GetForegroundWindow`, `OpenProcess`, `QueryFullProcessImageNameW`, `GetWindowTextW`) on every key-down of every key, a KS-08 regression of the rule pinned by `a_when_condition_is_asked_only_for_a_binding_whose_keys_match` (`engine/mod.rs:644-666`). `blocker_matches` reads only `KeyState` plus `env.os_key_down` for non-hotkey held keys. Pinned by test E1. Lands in commit 8, so that E1 is red in commit 7.

### D9: Harness and property extensions
In `src/engine/testing.rs`:
- `Step.masked: Option<bool>`.
- `expect_masked()` fails with `"{label}: expected the menu mask, got none"`. `expect_unmasked()` fails with `"{label}: expected no menu mask, got one"`. On a non-key step both fail with `"{label}: not a key event, nothing to mask"`.
- `check_mask_rule()`: from then on, every key event must satisfy `masked == suppressed && FakeEnv has any of VK_LMENU, VK_RMENU, VK_LWIN, VK_RWIN down`. The value is read after the decision; a suppressed event leaves the modelled OS unchanged, so before and after are the same. On failure: `"{label}: mask {got}, the rule says {want} (suppressed {s}, Alt or Win down in Windows {d})"`. The doc says not to combine it with a `MaskOnly` blocker.
- `advance_ms` adds `assert!(!decision.mask, "a timer has no event to mask")` next to `:389`.

In `src/engine/properties.rs`, the strict and focus runs turn `check_mask_rule()` on. The oracle reads the modelled OS, so it is independent of the engine's records.

### D10: The adapter docs state that `SendInput` re-enters the callback
Update `keyboard_hook_callback.rs:56-69`. `SendInput` on this thread re-enters the hook for the events it injects (AutoHotkey's `hook.cpp` relies on this). Those events carry the sentinel and pass at `decide_event` (`:166-168`) before any lock. The mask and every `press_vk` action are sent after the lock is released, so the re-entry cannot reach the engine. Update step 6 (`:20-23`) to read: mask first, then timers, then actions.

## Win32 API choices
| Purpose | `windows` path | Feature flag (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| Inject vk 0xE8 down and up | `windows::Win32::UI::Input::KeyboardAndMouse::SendInput(pinputs: &[INPUT], cbsize: i32) -> u32` (`windows-0.62.2/.../KeyboardAndMouse/mod.rs:166-169`) | `Win32_UI_Input_KeyboardAndMouse` (yes, `Cargo.toml:45`) | Main thread, inside the LL hook callback, after `with_engine` returned. Never under the engine lock; never on `hold-timer` (timer decisions never mask) | Returns the number inserted; 0 = "the input was already blocked by another thread"; UIPI failure "neither GetLastError nor the return value will indicate". A short count is logged (D2); nothing panics | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput: "This function is subject to UIPI. Applications are permitted to inject input only into applications that are at an equal or lesser integrity level." "The SendInput function inserts the events in the INPUT structures serially into the keyboard or mouse input stream." |
| `INPUT`, `INPUT_0`, `KEYBDINPUT`, `INPUT_KEYBOARD`, `KEYEVENTF_KEYUP`, `KEYBD_EVENT_FLAGS`, `VIRTUAL_KEY` | same module (`INPUT_KEYBOARD` :402, `KEYEVENTF_KEYUP` :560) | yes | Built on the stack in `menu_mask_inputs`; no allocation | — | https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput |
| The key code | vk 0xE8 | — | — | — | https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes: row "0xE8 — Unassigned"; AutoHotkey `#MenuMaskKey` (https://www.autohotkey.com/docs/v2/lib/_MenuMaskKey.htm): "Good candidates are virtual key codes which generally have no effect, such as vkE8", and "Microsoft can assign an effect to an unassigned key code at any time." |
| Sentinel | `crate::core::constants::injected_sentinel` (`GetCurrentProcessId`) | `Win32_System_Threading` (yes) | any | — | existing |
| Hook budget | (existing `WH_KEYBOARD_LL`) | — | main | — | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc: "on Windows 7 and later, the hook is silently removed without being called"; "The maximum timeout value the system allows is 1000 milliseconds" |

Flags. `LLKHF_INJECTED`: our E8 events carry it; we recognise our own by `dwExtraInfo`, not by the flag. The extended-key flag and `scanCode`: 0xE8 is not extended, and `wScan = 0` as in `press_vk` (KS-36 is 3.1). AltGr's fake LCtrl (scan 0x21D): irrelevant, because the mask rule looks at Alt and Win only. AltGr's RAlt counts as Alt, so a key a binding swallows under AltGr masks too, which is harmless because E8 maps to no character. No new Cargo feature, no new dependency.

## Touch set
| File | Change | Why |
|---|---|---|
| `src/engine/mod.rs` | `Decision::mask` doc; mask after the record update in `on_key_down`/`on_key_up` (commit 2, D1); `Engine.mask_owed`, `on_key` takes and ORs it, blocker branch by mode keeping the condition first (commit 6, D5); condition moved after the match (commit 8, D8); `reset` clears `mask_owed`; module docs (State list, rule 4 auto-blocker bullet, new rule 6 "Mask"); test E1 | Decision logic belongs in the engine |
| `src/engine/keys.rs` | `const MENU_KEYS: [VIRTUAL_KEY; 5]`; `pub(super) fn alt_or_win_down_in_windows(&self) -> bool`; unit test | D1 |
| `src/engine/testing.rs` | `Step.masked`, `expect_masked`, `expect_unmasked`, `check_mask_rule`, timer assert, module doc | D9 |
| `src/engine/scenarios.rs` | New sections "Menu mask key (KS-12)", "MaskOnly (KS-05)", "Blocker `when:` (KS-35)"; `with_alt_shift_blocker` → `new_auto_blocker(.., BlockerMode::Swallow)`; new `with_alt_shift_mask_only_blocker`; call sites :1212, :2637; comments at :2579-2582 and :2603-2607 ("until roadmap item 1.2" → "with `true`; `MaskOnly` lets it through"), plus `.expect_masked()` on Shift in the Alt case and `.expect_unmasked()` in the Ctrl case | Tests |
| `src/engine/properties.rs` | `check_mask_rule()` on in both modes; `Config.blocker: Option<BlockerMode>`; new config `mask_only` (the `default` bindings with a `MaskOnly` Alt+Shift blocker; mask rule off for it); module doc | D9, D5 |
| `src/engine/winner.rs` | `binding.is_auto_blocker()` at :64 | D6 |
| `src/engine/matching.rs` | Doc of `blocker_matches` (:81-91): "the blocker for it acts (swallows or masks)" instead of "swallows" | D5 |
| `src/data/binding.rs` | `BlockerMode` + `Display`; fields; `new_auto_blocker(combo, mode)`; `with_block_original_combo(Option<BlockerMode>)`; `is_auto_blocker()`; blocker Display; tests (:319, :447 call sites; Display tests) | D6 |
| `src/config/schema.rs` | `RawBlockOriginalCombo` (D4); field type and doc | D4 |
| `src/config/loader.rs` | Map `RawBlockOriginalCombo` → `Option<BlockerMode>` at :442; `expand_with_auto_blockers` with modes and fallback warning (commit 6) and conditions (commit 8), `warnings` parameter (D7); doc comments :211-221, :323-331; tests (L1-L9) and `is_auto_blocker()` sites; `parse_simple_binding` :841 → `Some(BlockerMode::Swallow)` | D4, D5, D7 |
| `src/core/constants.rs` | `pub const MENU_MASK_VK: u16 = 0xE8` + doc | D3 |
| `src/hook/keyboard_hook_callback.rs` | `menu_mask_inputs`, `inject_menu_mask`, `send_menu_mask` (cfg seam), `MASKS_SENT` (test only), `carry_out`; module docs step 6 and Re-entry (D10); tests H1-H3 | D2 |
| `README.md` | :101 and :106 (binding fields); :117-123 (adapter sends the menu mask key); new "**Win and Alt.**" paragraph in "### Combo syntax" after the exact-modifiers text (no ```` ```ron ```` block there unless it loads: `readme_matching_examples_load` parses that section); :442-446 (a passing Tap on lone Alt/Win: the mask does not cover it); :527 `block_original_combo` row (`true`/`false`/`MaskOnly`, `when:` inherited, mask, experimental, its manual check) | Config meaning changes |
| `src/config/default_config.ron` | Combo-syntax comment: the mask sentence; flag comment :160-173 rewritten (`true | false | MaskOnly`, inherits `when:`, MaskOnly experimental); Win+Q example comment ("Start stays closed"). No new example binding: `the_template_and_its_examples_have_no_duplicate_bindings` (`loader.rs:2268-2271`) would flag an F13 or CapsLock duplicate | Config meaning changes |
| `CHANGELOG.md` | Added (`MaskOnly`), Changed **Breaking:** (`when:` inheritance), Fixed (KS-12); edit stale Unreleased text: :157-158, :244-249, :262-272, :344-347 | Keep a Changelog |
| `book/src/config/matching.md` | §15.1 (`when:` inherited, mode, listing text), §15.3 (one blocker per hotkey and mode, `when:` OR'ed), §15.4 (KS-35 warning removed; KS-05 rest "with `true`"; lone Alt now masked), new §15.5 "`MaskOnly` (experimental)" with its manual check | Config reference |
| `book/src/config/structure.md` | :150 row (`true` / `false` / `MaskOnly`) | |
| `book/src/config/README.md` | :42 quick reference | |
| `book/src/config/ron-syntax.md` | :93 `Some(true)` message | D4 |
| `book/src/config/errors.md` | Parse-error table: the misspelled-value message; §18.6: the MaskOnly fallback warning and `[WARN] SendInput inserted N of 2 menu mask key events` | |
| `book/src/config/actions.md` | :63 KS-12 warning → fixed note (the mask, 0xE8 visible to key-logging tools); :248-255 DoNothing note | |
| `book/src/config/triggers-conditions.md` | :222 KS-35 warning → fixed; §12.4 bullet about blockers (asked only once the hotkey matches) | |
| `book/src/config/known-issues.md` | Row 31 Fixed; row 39 (`MaskOnly`, experimental); row 43 Fixed for `when` (trigger by design); row 66 re-labelled "Open, not covered by the mask (nothing is swallowed)" | |
| `book/src/config/example.md` | :234 KS-12 bullet | |
| Not touched | `Cargo.toml`, `Cargo.lock`, `book/src/{architecture,audit,roadmap,overview}.md` (phase book sync), `src/hook/keyboard_hook.rs`, `src/system/*`, `tests/` (no file there reaches `carry_out`) | |

## Commits, in order
Every subject is `<Area>: <what>`; every message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Gate = `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`.

1. **`Engine: red-first: the menu mask decision (1.2, KS-12)`**. Harness (D9) plus scenarios plus `check_mask_rule` in properties. Compiles; declared RED.
   - S1 `engine::scenarios::win_q_post_message_masks_the_swallowed_q_and_its_release`. `(keys: "Win+Q", action: PostMessage(msg: "WM_CLOSE"))`: LWin down passed/unmasked; Q down suppressed, fires, masked; Q up suppressed, masked; LWin up passed/unmasked. PREDICTED RED: `scenario: down Q at +0 ms: expected the menu mask, got none`.
   - S2 `alt_then_shift_with_the_template_blocker_masks_the_swallowed_shift`. `template()` (`scenarios.rs:1632`): LAlt down passed/unmasked; LShift down suppressed/masked; repeat LShift suppressed/masked; LShift up suppressed/masked; LAlt up passed/unmasked; `check_press_symmetry`, `check_nothing_stuck`. RED at "down LShift": same message.
   - S3 `right_alt_and_right_win_mask_a_swallowed_key`. `(keys: "Alt+J", action: DoNothing)`, `(keys: "Win+J", action: DoNothing)` with RAlt and RWin. RED at the first `expect_masked`.
   - S4 `swallowed_repeats_and_key_up_while_win_is_held_are_masked`. `(keys: "Q", action: DoNothing)`: Q down suppressed/unmasked (no Win); LWin down passed; repeat Q suppressed/masked; Q up suppressed/masked; LWin up passed/unmasked. RED at "repeat Q".
   - S5 guard `a_swallowed_event_without_alt_or_win_in_windows_is_not_masked`: default pair CapsLock; Ctrl+Shift blocker (Ctrl, Shift suppressed → unmasked); Shift+CapsLock. GREEN on master by construction; it kills the mutation `mask = suppress`.
   - S6 guard `a_swallowed_key_is_not_masked_when_the_alt_or_win_key_down_was_swallowed`: Alt+Shift blocker pressed Shift, then Alt (Alt suppressed/unmasked, Alt up suppressed/unmasked); `(keys: "Win", action: ToggleCapsLock, trigger: Tap(200), block_default: true)` with `(keys: "Q", action: DoNothing, allow_extra_modifiers: true)`: LWin down suppressed, Q down suppressed/unmasked. GREEN on master; it kills the mutation "count Win/Alt whatever its record".
   - S7 guard `a_passed_event_is_not_masked`: `(keys: "Ctrl+Alt+L", action: DoNothing, block_default: false)` → L passed/unmasked; unbound A with LAlt held passed/unmasked. GREEN.
   - S8 `alt_held_before_the_hook_saw_it_masks_a_swallowed_key`: `down_unseen("LAlt")`, `(keys: "Alt+J", action: DoNothing)`, J down suppressed/masked. RED.
   - S9 `win_held_through_remapping_on_masks_a_swallowed_key`: `remapping_off`, LWin down, `remapping_on`, Q down (`Win+Q`) suppressed/masked. RED.
   - S10 `win_held_across_a_foreground_change_masks_a_swallowed_key`. RED.
   - S11 `sequence_and_chord_keys_swallowed_while_win_is_held_are_masked`: `sequence: (steps: ["Win+F13", "Win+F14"])` and `chord: (keys: ["F13", "F14"])`, both with LWin held. RED.
   - S12 guard `a_hold_timer_never_masks`: `(keys: "Win+F13", action: DoNothing, trigger: Hold(200))`; LWin down, F13 down (its mask is not asserted here), `advance_ms(250)` fires the Hold while the new timer assert holds. GREEN on master; it kills a mutation that sets `mask` in `on_timer`.
   - Properties `every_press_is_suppressed_or_passed_as_a_whole` and `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups`. PREDICTED RED: `scenario: <step>: mask false, the rule says true (suppressed true, Alt or Win down in Windows true)` with context `config default` or `immediate`, at the first seed whose script swallows a key while LAlt is held.
2. **`Engine: ask for the menu mask after a swallowed event while Win or Alt is down in Windows (1.2, KS-12)`**. `keys.rs` method plus unit test `alt_or_win_down_in_windows_counts_held_keys_whose_key_down_was_not_swallowed` (Passed, Unknown-held via resync, Suppressed, Forgotten, released, generic `VK_MENU`); `mod.rs` D1 and docs. No `MaskOnly` and no `mask_owed` yet. Makes 1 green. Invariant: no change to `suppress`, `fire` or `timers` (the whole existing suite stays green). Gate green. No user-visible change yet (the adapter still drops the mask), so no docs.
3. **`Hook: red-first: the menu mask key's inputs and carry-out (1.2, KS-12)`**. Tests in `keyboard_hook_callback.rs`. Declared RED (compile).
   - H1 `menu_mask_inputs_are_vk_e8_down_then_up_tagged_with_the_sentinel`: reads `inputs[i].Anonymous.ki` with `// SAFETY: every element is built with r#type INPUT_KEYBOARD, so ki is the initialised field`; asserts type, `wVk == 0xE8`, flags `0` then `KEYEVENTF_KEYUP`, `wScan == 0`, `dwExtraInfo == injected_sentinel()`.
   - H2 `carry_out_sends_the_menu_mask_only_when_the_decision_asks`: `carry_out(Decision::default())` leaves `MASKS_SENT` at 0; `carry_out(Decision { mask: true, ..Decision::default() })` makes it 1.
   - H3 `#[ignore = "manual: injects vk 0xE8 down and up into the live input stream; run it with --ignored while Notepad has focus and check that it passes and nothing is typed and no menu opens"]` `the_menu_mask_key_is_inserted_into_the_input_stream`: `assert_eq!(inject_menu_mask(), 2)`.
   - PREDICTED RED: `error[E0425]: cannot find function 'menu_mask_inputs' in this scope` (and for `inject_menu_mask`; `MASKS_SENT` unresolved). Clippy and test fail to compile; the lib builds.
4. **`Hook: send the menu mask key before the hook returns (1.2, KS-12)`**. `constants.rs` D3; adapter D2 and D10. Makes 3 compile and H1-H2 green; H3 stays ignored. Same commit: CHANGELOG **Fixed** KS-12 entry (Win+Q, Alt then Shift, `Alt+X -> DoNothing`; applications and key-logging tools see an injected 0xE8; known-issues row 66 not covered); README (adapter section, "Win and Alt" paragraph, :442-446); template (combo-syntax sentence, Win+Q comment). Gate green.
5. **`Config: red-first: block_original_combo: MaskOnly (1.2, KS-05)`**. RON-level tests only, so it compiles on 4. Declared RED.
   - L1 `config::loader::tests::block_original_combo_takes_true_false_and_mask_only`: `true` → the blocker's Display ends `(blocked)`; `MaskOnly` → `(masked, experimental)`; `false` and missing → no blocker. PREDICTED RED: panic containing `Expected boolean`.
   - L2 `a_misspelled_block_original_combo_value_names_the_valid_ones`: `Maskonly` and `Some(true)`; asserts ``expected one of `true`, `false`, or `MaskOnly` instead``. RED: actual text `Expected boolean`.
   - L4 `mask_only_on_a_hotkey_with_a_payload_key_blocks_it_as_true_and_warns` (`parse_ron_str` with hotkey `[VK_OEM_3]`): the warning text from D5 and `(blocked)`. RED: load panic `Expected boolean`.
   - L5 `true_and_mask_only_on_one_hotkey_give_one_blocker_each`. RED likewise.
   - Scenarios, all via `from_ron_with_language_hotkey(..)`:
     - S16 `mask_only_blocker_lets_alt_and_shift_through_and_masks_the_exact_hotkey`: LAlt down passed/unmasked; LShift down passed/masked; LShift up passed/masked (owed); LAlt up passed/unmasked; nothing fired; symmetry and stuck checks.
     - S17 `mask_only_blocker_lets_ctrl_shift_t_reach_the_application`: Ctrl+Shift hotkey; T down passed/masked (owed); T up passed/unmasked.
     - S18 `mask_only_blocker_does_not_mask_a_hotkey_pressed_with_another_key_held`: Alt, Tab, Shift → Shift passed/unmasked.
     - S19 `mask_only_blocker_masks_only_the_first_event_after_the_match`: LAlt, LShift masked, repeat LShift masked (owed), repeat LShift unmasked, LShift up unmasked.
     - S20 `alt_shift_x_fires_next_to_a_mask_only_blocker`: X down suppressed, fires, masked.
     - S21 `remapping_off_drops_an_owed_mask`: LAlt, LShift, `remapping_off`, `remapping_on`, F13 down unmasked.
   - PREDICTED RED for S16-S21: `scenario: config does not load: config parse error (<in-memory>): <line>:<col>: Expected boolean`.
6. **`Config: block_original_combo: MaskOnly lets the hotkey through and masks it (1.2, KS-05)`**. Schema D4, `binding.rs` D6 (+ tests `display_marks_a_mask_only_blocker`, `blocker_mode_display`), loader D5 fallback (one blocker per hotkey and mode, condition still `Always`), engine `MaskOnly` branch and `mask_owed`, harness `with_alt_shift_mask_only_blocker`, properties `Config.blocker` and the `mask_only` config, call-site updates. **The blocker branch keeps asking its condition before `blocker_matches`, as at `mod.rs:409`; commit 8 moves it (D8), so E1 is red in commit 7.** Makes 5 green. Same commit: CHANGELOG **Added** (`MaskOnly`, experimental, manual check, KS-05) and edits of :262-272 and :344-347; README :101, :106, :527; template :160-173 (MaskOnly part). Gate green.
7. **`Config: red-first: an auto-blocker follows its binding's when (1.2, KS-35)`**. Declared RED.
   - L7 `the_blocker_takes_the_when_of_the_binding_that_asked_for_it`: `assert_eq!(blocker.condition, Condition::AppEquals("x.exe".into()))`. PREDICTED RED: `left: Always`, `right: AppEquals("x.exe")`.
   - L8 `blockers_of_one_hotkey_combine_the_when_of_their_owners`: owners `AppEquals("a.exe")` and `AppEquals("b.exe")` → `Or([a, b])`; an `Always` owner → `Always`; an equal condition twice → once. RED `left: Always`.
   - L9 `a_hotkey_asked_for_by_true_and_mask_only_keeps_each_owners_when`. RED.
   - S13 `a_blocker_asked_for_by_a_binding_with_when_app_equals_is_inactive_in_other_apps`: `(keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("x.exe"))` with the Alt+Shift hotkey. `.foreground("y.exe", "Y")`: LAlt, LShift passed, nothing stuck. `.foreground("x.exe", "X")`: LShift suppressed/masked. PREDICTED RED at the first LShift key-down in y.exe: `scenario: down LShift at +<t> ms: expected the event to be passed, it was suppressed`.
   - S14 `blockers_act_where_any_owner_when_holds`: owners for a.exe and b.exe; suppressed in both, passed in c.exe. RED in c.exe, same message.
   - E1 `engine::tests::a_blocker_when_condition_is_asked_only_once_its_hotkey_matches`: `Binding::new_auto_blocker(KeyCombination::from_keys([VK_MENU, VK_SHIFT]), BlockerMode::Swallow).with_condition(Condition::AppEquals("x.exe".into()))` plus F14 bound; F14 down with `ForegroundForbiddenEnv`. PREDICTED RED: panic `the foreground was asked for although no binding with a when: matches this key`.
8. **`Config: an auto-blocker follows the when of the bindings that ask for it (1.2, KS-35)`**. Loader D7, engine D8, doc comments. Makes 7 green. Same commit: CHANGELOG **Changed** "**Breaking:** the auto-blocker acts only where the `when:` of a binding that asked for it holds (KS-35)", with the `Not(AppEquals("game.exe"))` example, and the edit of :244-249; README :527 (`when:` part); template :172-173. Gate green.
9. **`Book: the menu mask key, MaskOnly and the blocker's when in the config reference (1.2)`**. Every config page in the touch set. Each new RON block is validated through the real loader (cfgcheck, `book/src/methodology.md#validation-harness`). `mdbook build book -d D:/tmp/ks-orch/p1-1.2-mask-key/book` with a link check. Gate green (no code).

Bisect: 2, 4, 6 and 8 are green and each adds one behaviour; 1, 3, 5 and 7 are the declared red commits.

## Acceptance mapping
| Roadmap / brief acceptance line | Covered by |
|---|---|
| Scenario tests show `mask = true` exactly when an event is suppressed after Alt or Win reached the OS, and false otherwise (default modes) | S1-S4, S8-S11 (true); S5-S7, S12 (false); properties' `check_mask_rule` oracle in the strict and focus runs, against the modelled OS |
| The opt-in v2 mode passes the modifier key-downs and sets mask on an exact match | S16-S20, L1, L5; properties config `mask_only` (symmetry and nothing stuck). The extra owed mask on the next event (D5) is pinned by S16, S17, S19 and S21 |
| A blocker requested by a binding with `when: AppEquals("x.exe")` is inactive in other apps | S13 (+ S14, L7-L9, E1) |
| Manual: Alt then Shift in Notepad does not open the menu | M1 |
| Manual: `Win+Q -> PostMessage` does not open Start | M2 |
| Manual (v2): `Ctrl+Shift+T` works and a bare Alt+Shift does not switch the language | M4, M5 |
| The injection itself only in `#[ignore = "manual: ..."]` tests | H3 (+ H1 and H2, which inject nothing) |
| Update README, `default_config.ron`, the book config pages and CHANGELOG | Commits 4, 6, 8, 9 |

## Manual checks (human only)
Release build started with `swch open` from a normal console; `osk.exe` open to watch the modifiers. Registry hotkeys are changed by the owner in Windows Settings (Typing → Advanced keyboard settings → Input language hot keys), never by an agent.

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M1 | Default config, hotkey Alt+Shift | In Notepad: Alt down, Shift down, Shift up, Alt up; then type `fx` | No menu bar; `fx` is typed; layout unchanged | none (needs the daemon) |
| M2 | Add `( keys: "Win+Q", action: PostMessage(msg: "WM_CLOSE") )` | Focus a Notepad window; Win+Q; release Win | Notepad closes; the Start menu does not open | none |
| M3 | Default config, three windows | Hold Alt, tap Tab, press Shift, tap Tab, release all | The switcher stays open and steps forward (v1 limit); no menu bar in the target window | none |
| M4 | CapsLock binding with `block_original_combo: MaskOnly`, hotkey Ctrl+Shift | Browser: close a tab, then Ctrl+Shift+T; then a bare Ctrl+Shift press and release, five times | The tab reopens; the layout never switches; CapsLock still switches it | none |
| M5 | Same with hotkey Alt+Shift | Bare Alt+Shift five times; Alt+Shift+Tab with three windows; Notepad Alt, Shift, release | No layout switch; the switcher steps back; no menu bar | none |
| M6 | `( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("notepad.exe") )` | Alt+Shift in Notepad, then in another app | Notepad: no switch, no menu; elsewhere: Windows switches the layout | none |
| M7 | Hotkey `` ` `` (value "4") with `MaskOnly` | Start the daemon | `[WARN] binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; ...`; a lone `` ` `` is swallowed | none |
| M8 | Debug build, Notepad focused | `cargo test --lib -- --ignored the_menu_mask_key_is_inserted_into_the_input_stream` | Passes; nothing typed, no menu | `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream` |
| M9 | After M1-M6 | Look at `osk.exe` | No modifier left down | none |

If M4 or M5 fails (the layout still switches), the hypothesis or the injection order is wrong. Record it in the audit status and keep the default `true`.

## Risks and edge cases
- **R1, hot path.** One `SendInput` of two stack-built events per masked event, outside the lock; no allocation. Under the recursive model the call runs every LL hook on the machine for E8 down and up inside our callback, so a slow third-party hook lengthens our callback and can approach `LowLevelHooksTimeout` (KS-08; the 2.4 watchdog). A swallowed auto-repeat masks again (≤ ~30/s, Q2). The short-count `eprintln!` can block on a console in QuickEdit selection, the same pre-existing risk as `press_vk`; item 1.7 replaces it with counters.
- **R1, re-entry.** `SendInput` re-enters the callback on the main thread (D10). The injected events pass on the sentinel at `decide_event:166` before `with_engine`, and `carry_out` runs after the lock and the snapshot are released, so no deadlock and no engine mutation. Exactly one outcome per path is unchanged (`:257-267`).
- **R2, sentinel.** Both E8 events carry `injected_sentinel()`, pinned by H1. No hard-coded constant.
- **R3/R4, shared state.** No new shared state: `mask_owed` is plain data inside `Mutex<Engine>`, and the bindings carry their modes through the existing `ArcSwap`. `MASKS_SENT` is a `cfg(test)` thread-local. No new atomics or orderings.
- **R5/R6.** `new_auto_blocker(combo, mode)` has two positional arguments; the condition goes on with `.with_condition`. `BlockerMode` has `Display`, and the blocker's Display arm covers both modes. No `_ =>` arm.
- **R7.** Blockers are still appended after the file's bindings; two blockers for one hotkey keep first-request order; the stable sort is unchanged.
- **R8.** The mask looks at L/R Alt, the generic `VK_MENU` slot and L/R Win; blocker matching is unchanged (`AnySide` hotkeys). Sequences and chords mask through the same rule (S11).
- **R9.** Registry reads only. `MaskOnly` uses `is_modifier_only` of the hotkey read.
- **R10.** Shutdown and the message loop are unchanged; nothing is installed or uninstalled.
- **Safety seam.** Unit tests compile the lib with `cfg(test)`, so `carry_out` only counts masks. Integration tests in `tests/` link the lib without `cfg(test)`; none reaches `carry_out` or `keyboard_hook_callback` today (`tests/ipc_roundtrip.rs` drives only the pipe), and a future one must not. The reviewer should check this.
- **Reload mid-gesture.** A `true` → `MaskOnly` reload while a swallowed Shift is held: its repeats and key-up follow the swallowed key-down, so nothing sticks. The reverse case passes symmetrically. A reload changing an owner's `when:` takes effect on the next hotkey key-down. A pending owed mask survives a reload (one harmless E8).
- **`swch off` / `on`.** `reset` clears `mask_owed`. A Win held through `on` counts as passed (S9).
- **Foreground change.** `Forgotten` records still count as swallowed, so no mask. Passed records keep counting (S10). The owed mask is kept.
- **Lost key-up (secure desktop).** The resync repairs Alt and Win `active` from the OS on the next event, so a stale Alt does not mask. An owed mask is consumed by the next event, late but harmless.
- **UIPI and elevation.** The release daemon is elevated, so `SendInput` reaches every window at ≤ high integrity. A debug build cannot inject into elevated windows; the mask is then silently lost, which is no worse than today. Secure Desktop: the hook sees nothing there. No pipe or token change.
- **IME, layouts, other tools.** E8 is unassigned and maps to no character (AltGr+E8 types nothing). Microsoft may assign it later (AutoHotkey's warning). Applications, games, key overlays and an AutoHotkey running alongside see an injected 0xE8; an AutoHotkey E8 that reaches our engine is an unbound key and may consume an owed mask. RDP/VM clients get E8 with scan code 0 (KS-36).
- **Injection order (unverified).** The double mask in `MaskOnly` (D5) hedges it. v1 does not depend on it. M4 and M5 settle it.
- **Merge conflicts.** Parallel Phase-1 lanes (1.4, 1.5, 1.7) touch `loader.rs`, the README and the CHANGELOG. Keep both sides; `expand_with_auto_blockers` may meet 1.4's tri-state hotkey, and a disabled hotkey simply yields no request.

## Questions for the orchestrator
1. **The second (owed) mask in `MaskOnly` goes beyond the literal "on an exact match only injects the mask".** Recommendation: keep it (D5). AutoHotkey's source says injected keys are processed before the event in the hook, which would put a match-time mask before Shift and could leave Alt+Shift clean. The literal mask still happens, so the acceptance line holds. Dropping the owed mask removes one `bool`.
2. **Mask on every swallowed auto-repeat while Win or Alt is held, or once per press?** Recommendation: every swallowed event, as the acceptance says "exactly when"; the cost is bounded by the repeat rate. Revisit in 3.1 with measurements.
3. **Known-issues row 66 (a passing Tap or Hold on Alt or Win alone opens the menu or Start), which the book and CHANGELOG say 1.2 fixes, is not fixed by the specified rule.** Recommendation: re-label it in this lane and open a follow-up (re-inject the modifier's key-up after a mask, near 3.1). Not blocking.
4. **Research to commission only if M4 or M5 fails.** Query for the `researcher`: "Windows 10/11: are keyboard events injected with SendInput from inside a WH_KEYBOARD_LL callback processed before or after the event currently in the hook? How does win32k decide the Alt+Shift / Ctrl+Shift input-language toggle (which intervening keys reset it)? Primary sources and the AutoHotkey hook.cpp AllowIt comment." Recommendation: not needed before implementation.
5. **`when:` inheritance is a visible behaviour change.** Recommendation: CHANGELOG **Changed**, "**Breaking:**", like the KS-18 entry.
6. **A hotkey asked for with both `true` and `MaskOnly`.** Recommendation: two blockers, swallow wins where both conditions hold, no warning (documented in §15.3).
7. **Naming: `MaskOnly` (as the brief suggests), `BlockerMode::{Swallow, MaskOnly}`.** Recommendation: keep.

## Readiness checklist
- [x] Every cited KS issue read (KS-05, KS-12, KS-35, and the neighbours KS-14, KS-17, KS-18, KS-19, KS-36); scenarios reproduced (S1, S2, S13; KS-05 v2 as S16-S17 plus M4-M5).
- [x] Every book reference re-located, or marked superseded (open question 6).
- [x] Goal, non-goals and dependents stated.
- [x] Every decision has a rationale and rejected alternatives.
- [x] Owner decisions respected: exact modifiers untouched (blocker matching unchanged); repeat defaults untouched; RON kept (D4); `eprintln!` until 1.7; licence N/A.
- [x] Threads named: mask on the main thread in the callback; loader on the main thread or `config-watcher`; nothing heavy in the callback.
- [x] Install, uninstall and timers: none added.
- [x] The new `SendInput` path carries the sentinel; down and up are paired in one call.
- [x] Sided and unsided: the mask covers L/R and the generic slot; combo, chord and sequence alike.
- [x] Reload, `off`/`on`, foreground change and shutdown covered (Risks; S9, S10, S21).
- [x] Atomics: none new; no `static mut`.
- [x] No new module cycle (engine → `data::binding`, hook → `core::constants`, both existing); logic in the engine.
- [x] Each API: path, feature, thread, errors, doc link.
- [x] Elevation and UIPI considered; pipe and token N/A.
- [x] Red-first with exact predicted reds (commits 1, 3, 5, 7).
- [x] Tests go through `on_key`/`on_timer`/`Env`/the fake clock; the `cfg(test)` seam makes `carry_out` inject nothing.
- [x] Real-Win32 test H3 is `#[ignore = "manual: ..."]` and listed as M8.
- [x] Each non-red commit passes the gate; docs go with the meaning-changing commits 4, 6 and 8.
- [x] Every acceptance line is mapped.
- [x] Every question has a recommendation.

## Developer addendum (implementation round 1)

The critique has no Critical and no Important remark. State of every remark and open question:

- **M1 (config and book pages still stating KS-12 as current): RESOLVED.** Commit `0187cde` updates `book/src/config/triggers-conditions.md` §11.8 (line 135: the mask covers `Win+…`/`Alt+…` with a deferred trigger; a lone Alt/Win Tap/Hold/DoubleTap is still not covered, row 66) and `book/src/config/example.md` lines 6, 33-39 and 234-235 (the "Deliberately NOT used" comment now names the lone Alt/Win gesture instead of Win+X/Alt+X). Handed to the phase book sync, not edited here (outside `book/src/config`): `book/src/build-and-run.md:415` and `:457-465` ("no mask key is injected"), `book/src/glossary.md:141` and `:313` ("mask reserved", "no mask key yet"), `book/src/architecture.md:268, 271, 544`, the `audit.md` status lines of KS-05/KS-12/KS-35 and the roadmap status block.
- **M2 (Checkpoint-1 smoke item 4): RESOLVED by statement.** Checkpoint-1 item 4 ("`Ctrl+Shift+T` and `Alt+Shift+Tab` work; a bare Alt+Shift does not switch the language") is met only with `block_original_combo: MaskOnly` opted in (manual checks M4/M5). With the template's `true`, Alt, Shift, Tab still swallows Shift (`engine::scenarios::shift_pressed_after_tab_was_released_is_still_swallowed`, now also `.expect_masked()`), and Ctrl, Shift, T still arrives as Ctrl+T (`ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing`). `results.md` must say so, and the phase book sync rewords `book/src/roadmap.md:308` to "with `MaskOnly`" until M4/M5 decide the default.
- **Open question 1 (mask while the Alt+Tab switcher is open):** documented, not decided here. `book/src/config/matching.md` §15.4 carries a new warning, and M3 is the manual check. A failure of M3 is a RETHINK at results (the developer does not choose a narrower default rule).
- **Open question 2 (D10 wording): RESOLVED** at `src/hook/keyboard_hook_callback.rs:58-82`: "no event of ours reaches the engine. Microsoft does not document whether another callback, such as `foreground_changed`, can run on this thread while `SendInput` waits; one that does finds the lock free and does what any foreground change does."
- **Open question 3 (fail-safety of H2):** the seam is `#[cfg(not(test))] fn send_menu_mask` / `#[cfg(test)] fn send_menu_mask` at `src/hook/keyboard_hook_callback.rs:253-271`; `MASKS_SENT` exists only under `cfg(test)` (`:273-277`). Review checklist line for the code-reviewer: the `cfg(test)` variant of `send_menu_mask` must not call `inject_menu_mask` or `SendInput`, and no file under `tests/` may reach `carry_out` or `keyboard_hook_callback` (true today: only `tests/ipc_roundtrip.rs`).
- **Open question 4 (Q1, Q2 of the cut):** implemented as recommended: the owed mask (D5) and a mask per swallowed event (D1).
