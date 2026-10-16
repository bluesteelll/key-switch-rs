# Auto-blockers and matching order

`block_original_combo: true` silences the Windows hotkey that does the same job as a binding's action, such as Alt+Shift for `SwitchLanguage`. The second half of this page gives the order in which the daemon checks bindings and what it means when several match. Facts describe commit `1dfd80a`, those about the `repeat` field the branch `feat/repeat-flag`, those about `block_default` on `Tap`, `Hold` and `DoubleTap` the branch `feat/deferred-block-default`, and those about the menu mask key, `MaskOnly` and the blocker's `when` the branch `feat/mask-key` (roadmap [item 1.2](../roadmap.md#12-mask-key-and-auto-blocker-v2)); the run-time behaviour is derived from the code of the decision core and its scenario tests, not observed on a live daemon.

<a id="s15"></a>
## 15. `block_original_combo`: auto-blockers

Code: `expand_with_auto_blockers` ([src/config/loader.rs:321-422](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/config/loader.rs#L321-L422)) builds the blockers, and the engine matches them ([src/engine/mod.rs:439-467](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/engine/mod.rs#L439-L467), [src/engine/matching.rs:81-102](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/engine/matching.rs#L81-L102)). The registry is read in [src/system/registry.rs](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs).

<a id="s15-1"></a>
### 15.1 What the flag does

The flag takes `true`, `false` (the default) or `MaskOnly` (experimental, [§15.5](#s15-5)). After all bindings have been validated, the loader collects the system hotkeys of the bindings that have `block_original_combo: true` or `MaskOnly`, one blocker per distinct hotkey and mode. For each it **appends** an auto-blocker after all of the file's bindings, unless a binding of the file already uses the hotkey's keys ([§15.3](#s15-3)). The auto-blocker is a combination binding on the system hotkey with `action: DoNothing`, `trigger: Immediate`, the `when` of the bindings that asked for it ([§15.3](#s15-3)) and its mode, marked as a blocker ([src/data/binding.rs:156-171](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/data/binding.rs#L156-L171)). In the binding list printed at startup it appears last, as `[AUTO-BLOCK] … -> (blocked)` for `true` and `[AUTO-BLOCK] … -> (masked, experimental)` for `MaskOnly`, followed by its `when` if it has one.

```ron
( bindings: [ ( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true ) ] )
```

This produces 2 bindings, `CapsLock → SwitchLanguage` and then the auto-blocker `Alt+Shift` (the default combination unless the registry specifies otherwise). The validator prints:

```text
[INFO] SwitchLanguage: using default combination
OK: 2 bindings (after auto-blocker expansion)
  #0: CapsLock                           -> switch language
  #1: [AUTO-BLOCK] Alt + Shift              -> (blocked)
```

An auto-blocker is not an ordinary binding at run time ([src/engine/mod.rs:439-467](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/engine/mod.rs#L439-L467)):

- **It acts only on the hotkey pressed on its own.** It acts on a key-down when the key just pressed is one of the hotkey's keys, all of them are held, and every other key held is one that Windows does not have down ([src/engine/matching.rs:81-102](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/engine/matching.rs#L81-L102)). A key "Windows does not have down" is one whose key-down a binding swallowed, or one whose key-up the hook missed. Alt, then Shift matches. Shift pressed while Alt and Tab are held, Shift pressed while Ctrl and T are held, and Alt+Shift pressed while a letter is still held in fast typing reach Windows. Ctrl+Alt+Shift does not match the `Alt+Shift` blocker.
- **It acts only where its `when` holds**, and that is asked only once the hotkey matches, so a blocker with a `when` costs the hook no window lookup on other keys ([§12.4](triggers-conditions.md#s12-4)).
- **It fires nothing** and never takes the place of a binding of yours: a binding that matches the same key-down fires as if the blocker were not there.
- **With `true` it suppresses the whole press.** The swallowed key's auto-repeats and release are swallowed too, even after the other hotkey key is released ([§10](kinds.md#s10)). The swallowed key still counts as held for matching, so `Alt+Shift+X` bindings fire. When Alt is down in Windows, as with Alt, then Shift, every swallowed event also sends the menu mask key, so releasing Alt does not activate the menu bar ([§13.1](actions.md#s13-1)).
- **With `MaskOnly` it suppresses nothing** and sends the menu mask key instead ([§15.5](#s15-5)).

Further points:

- Only `SwitchLanguage` and `SwitchLanguageBackward` have a system hotkey. `ToggleCapsLock` and all other actions have none: the flag does nothing, and there is no error either.
- The binding kind does not matter: a blocker is also added for a `sequence` or a `chord` with these actions, and the blocker is still a combination.
- The blocker takes the `when` of the bindings that asked for it, not their `trigger`: it acts on the hotkey's own press, whatever gesture its binding uses.
- The blocker does not affect the daemon's own synthesized input ([§13.1](actions.md#s13-1)).

<a id="s15-2"></a>
### 15.2 How the system combination is determined

[src/system/registry.rs:17-79](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs#L17-L79), [src/system/registry.rs:181-189](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs#L181-L189):

- The key `HKCU\Keyboard Layout\Toggle` is read: first the value `Hotkey`, and, if it is missing, unreadable or holds an unknown value, the value `Language Hotkey`. A value that names a combination or says `"3"` stops the lookup. `REG_SZ` and `REG_EXPAND_SZ` strings are accepted (leading and trailing whitespace is trimmed from the value); a value of any other type is skipped with the warning `[WARN] registry value "Hotkey" has type …, expected REG_SZ/REG_EXPAND_SZ`. Values:
  - `"1"` → `Alt+Shift`;
  - `"2"` → `Ctrl+Shift`;
  - `"4"` → `` ` `` (`VK_OEM_3`; on the Russian layout this is the `Ё` key). The blocker swallows that key when it is pressed with no other key down in Windows, so `` ` `` typed on its own does nothing, while Shift+`` ` `` and a `` ` `` typed while another key is held reach Windows (and may switch the layout);
  - `"3"` ("Not assigned" in Windows' settings) means that Windows has no language hotkey: no blocker is added, and there is no fallback. The first value that names a combination or says `"3"` decides, so this holds for `"3"` in `Hotkey`, and for `"3"` in `Language Hotkey` when `Hotkey` is missing or unknown;
  - any other text, a missing value and a value of another type are skipped. If no value decides, or the key does not exist, the **fallback `Alt+Shift`** is used.
- The modifiers of these hotkeys mean either side: the `Alt+Shift` blocker covers both Alt keys and both Shift keys.
- The console shows `[INFO] SwitchLanguage: registry combination Alt+Shift` (the keys by name), `[INFO] SwitchLanguage: hotkey not assigned in Windows, no auto-blocker` or `[INFO] SwitchLanguage: using default combination`. For `SwitchLanguageBackward` the line starts with that name instead. The line is printed for every binding that has the flag.
- The registry is read on every load of the file: at startup and on every reload.
- The value `Layout Hotkey` (switching layouts within one language) is not read, and Win+Space is not blocked ([`Layout Hotkey` and Win+Space](../gaps.md#layout-hotkey-and-winspace)).

<a id="s15-3"></a>
### 15.3 Deduplication

A blocker is not added if **any** `keys` binding of the file, the flagged binding included, has exactly the hotkey's keys, in any order ([src/config/loader.rs:371-379](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/config/loader.rs#L371-L379); [KS-18](../audit.md#ks-18), fixed in `1dfd80a`). The console then says so:

```ron
( bindings: [
    ( keys: "CapsLock",  action: SwitchLanguage, block_original_combo: true ),
    ( keys: "Alt+Shift", action: SwitchLanguageBackward ),
] )
```

```text
[INFO] SwitchLanguage: using default combination
[INFO] no auto-blocker for Alt+Shift: binding #1 uses the same keys
OK: 2 bindings (after auto-blocker expansion)
```

- `#N` is the first such binding in the file, counting from 0.
- Only `keys` bindings count (not `sequence` and not `chord`).
- The key set must match exactly, side for side: `Ctrl+Alt+Shift` or `LAlt+LShift` does not cancel an `Alt+Shift` blocker. With `LAlt+LShift` bound, that binding fires on the left keys, and the blocker still covers the presses that use a right Alt or Shift.
- The `when`, `trigger`, `block_default` and action of that binding play no role in the decision. That binding takes the hotkey over, and its own `when` and `block_default` decide whether Windows sees the hotkey: where its `when` is false, the hotkey reaches Windows, and so it does with `block_default: false`. Since `feat/deferred-block-default` that is the default of a `Tap`, `Hold` or `DoubleTap` binding made only of modifiers, as a binding on a modifier hotkey is: `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200) )` lets Windows switch the layout forward while it switches it back, also when it sets `block_original_combo: true` itself. Write `block_default: true` on it to keep the hotkey from Windows.
- Flagged bindings with the same hotkey and mode produce one blocker. Its `when` is the `when` of those bindings OR'ed, each distinct condition once, in file order (written the same way counts as the same); if one of them has no `when`, the blocker has none either and acts in every window ([src/config/loader.rs:380-422](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/config/loader.rs#L380-L422); [KS-35](../audit.md#ks-35), fixed on `feat/mask-key`). A hotkey asked for with `true` by one binding and with `MaskOnly` by another gets two blockers, one of each mode; where both act, the hotkey is swallowed, and the extra mask key does no harm.

```ron
( bindings: [
    ( keys: "CapsLock", action: SwitchLanguage,         block_original_combo: true, when: AppEquals("a.exe") ),
    ( keys: "F13",      action: SwitchLanguageBackward, block_original_combo: true, when: AppEquals("b.exe") ),
] )
```

```text
[INFO] SwitchLanguage: using default combination
[INFO] SwitchLanguageBackward: using default combination
OK: 3 bindings (after auto-blocker expansion)
  #0: CapsLock                           -> switch language  [AppEquals("a.exe")]
  #1: F13                                -> switch language backward  [AppEquals("b.exe")]
  #2: [AUTO-BLOCK] Alt + Shift              -> (blocked)  [Or([AppEquals("a.exe"), AppEquals("b.exe")])]
```

The blocker acts in `a.exe` and in `b.exe`; in every other window Windows gets its hotkey.

`( keys: "Alt+Shift", action: SwitchLanguageBackward, block_original_combo: true )` on its own therefore gets no blocker either, and its action runs.

<a id="s15-4"></a>
### 15.4 Known auto-blocker issues

> [!WARNING]
> **With `true` the second modifier is swallowed.** Pressed modifiers first, as in Alt, Shift, X, the blocker matches on Shift, so the application gets Alt+X; in the order Shift, Alt, X it swallows Alt, and the application gets Shift+X. With the `Ctrl+Shift` hotkey, Ctrl, Shift, T arrives as Ctrl+T. That includes the usual way back in the Alt+Tab switcher, Alt held, Tab released, then Shift+Tab, which still moves forward. [KS-05](../audit.md#ks-05) is partly fixed with `true`; the experimental `MaskOnly` ([§15.5](#s15-5)) lets the second modifier through. A lone Alt no longer activates the menu bar: the swallowed Shift sends the menu mask key ([KS-12](../audit.md#ks-12), fixed on `feat/mask-key`).

> [!WARNING]
> **The mask key in the Alt+Tab switcher.** With `true`, Alt held, Tab tapped, then Shift: Shift is swallowed and the menu mask key reaches Windows while the switcher is on screen. Whether the switcher reacts to that key is not verified; it is one of the manual checks of roadmap [item 1.2](../roadmap.md#12-mask-key-and-auto-blocker-v2).

> [!NOTE]
> Fixed on `fix/language-hotkey-tristate` ([KS-17](../audit.md#ks-17), roadmap [item 1.4](../roadmap.md#14-tri-state-language-hotkey)): the value `"3"` (hotkey disabled in Windows) gives no blocker ([§15.2](#s15-2)); it used to give an `Alt+Shift` blocker.

> [!NOTE]
> Fixed on `feat/mask-key` ([KS-35](../audit.md#ks-35)): the blocker takes the `when` of the bindings that asked for it ([§15.3](#s15-3)); it used to act in every window. It still does not take their `trigger`, by design: it acts on the hotkey's own press whatever gesture its binding uses. With `when: Not(AppEquals("game.exe"))` on the default config's CapsLock binding, the Windows hotkey now reaches the game and switches the layout there, while CapsLock reaches it as CapsLock.

<a id="s15-5"></a>
### 15.5 `MaskOnly` (experimental)

`block_original_combo: MaskOnly` adds a blocker that swallows nothing ([src/engine/mod.rs:439-467](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/engine/mod.rs#L439-L467), [src/engine/mod.rs:343-362](https://github.com/bluesteelll/key-switch-rs/blob/ad7fe86/src/engine/mod.rs#L343-L362)). It matches exactly as the `true` blocker does ([§15.1](#s15-1)): the hotkey on its own, where its `when` holds. Then:

- the key-down that completes the hotkey reaches Windows, and the daemon sends the menu mask key (virtual-key code 0xE8, [§13.1](actions.md#s13-1)) before passing it on;
- the next key event, whatever key it is (usually the release of Shift or Alt, or the T of Ctrl+Shift+T), also reaches Windows as the bindings decide, and sends the mask key once more;
- an auto-repeat of the hotkey's key sends no new mask.

The second mask is there because the order of a key injected from inside the keyboard hook against the event in the hook is not documented by Microsoft. If the first mask key reached Windows before Shift, Windows could still see a clean Alt+Shift; the second one lands after Shift's key-down whatever the order. `swch off` and `swch on` drop a second mask that is still due.

So `Ctrl+Shift+T`, `Alt+Shift+X` and `Alt+Shift+Tab` reach applications, where `true` swallows the second modifier ([§15.4](#s15-4)). Whether Windows then leaves the layout alone is the hypothesis this mode rests on: that Windows switches the layout only on a clean press and release of its hotkey, with no other key in between. No test can check it, because tests do not send input to Windows.

```ron
( bindings: [ ( keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly ) ] )
```

```text
[INFO] SwitchLanguage: using default combination
OK: 2 bindings (after auto-blocker expansion)
  #0: CapsLock                           -> switch language
  #1: [AUTO-BLOCK] Alt + Shift              -> (masked, experimental)
```

Check it by hand before relying on it, with the release build started by `swch open` and `osk.exe` open to watch the modifiers:

1. With the `Ctrl+Shift` hotkey: in a browser, close a tab and press Ctrl+Shift+T; the tab reopens. Then press and release a bare Ctrl+Shift five times; the layout never switches. CapsLock still switches it.
2. With the `Alt+Shift` hotkey: press and release a bare Alt+Shift five times; the layout never switches. With three windows open, Alt+Shift+Tab steps back in the switcher. In Notepad, Alt, Shift, release activates no menu.
3. Afterwards `osk.exe` shows no modifier held.

If the layout still switches, the hypothesis or the injection order does not hold: go back to `true`.

`MaskOnly` needs a hotkey made only of modifiers. With the `` ` `` hotkey (registry value `"4"`) letting a lone `` ` `` through would type it and switch the layout, so the loader adds a blocker that swallows, as `true` does, and warns: ``[WARN] binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true`` ([§18.6](errors.md#s18-6)).

> [!NOTE]
> Before `1dfd80a` the blocker matched any superset of its keys and swallowed every Alt+Shift+X ([KS-05](../audit.md#ks-05)), and it was inserted before its owner as an ordinary `Immediate` binding, so an `Immediate` binding on the same keys below it never ran ([KS-18](../audit.md#ks-18)).

<a id="s16"></a>
## 16. Matching order and binding priority

<a id="s16-1"></a>
### 16.1 Sorting

1. The loader returns the bindings in file order, followed by the auto-blockers.
2. `KeyboardHook::update_bindings` ([src/hook/keyboard_hook.rs:106-115](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook.rs#L106-L115)) is called at startup ([src/core/app.rs:128-131](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/app.rs#L128-L131)) and on every reload. It sorts the list **stably** by "key count", from largest to smallest ([src/engine/mod.rs:261-267](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L261-L267), [src/data/binding.rs:34-47](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/binding.rs#L34-L47)):
   - for `keys`, the number of distinct keys after duplicates are removed (`"Ctrl+Alt+L"` = 3);
   - for `sequence`, the number of **steps**;
   - for `chord`, the number of keys.

   Ties keep the file order, so an auto-blocker comes after the file's bindings with the same key count.

<a id="s16-2"></a>
### 16.2 Matching on every key press

On every key-down ([src/engine/mod.rs:388-570](https://github.com/bluesteelll/key-switch-rs/blob/ed623c5/src/engine/mod.rs#L388-L570)) one `keys` binding wins the press, and the other bindings are visited next to it ([src/engine/winner.rs:44-82](https://github.com/bluesteelll/key-switch-rs/blob/ed623c5/src/engine/winner.rs#L44-L82)). This holds since `feat/one-winner` (roadmap [item 1.6](../roadmap.md#16-one-winner-per-event), [KS-13](../audit.md#ks-13)); before it every matching binding acted, and only `Immediate` bindings competed.

- The candidates are the `keys` bindings that match the press by the rules of [§7](kinds.md#s7) (the trigger key, all keys held, exact modifiers or `allow_extra_modifiers: true`) and whose `when` holds. Auto-blockers, `sequence` and `chord` bindings are not candidates.
- The winner is the candidate with the most keys; with equal key counts, a binding with `when` beats one without; then the one higher in the file. `when: Always` written out counts as no condition.
- A candidate's `when` is evaluated only once its keys match the press and only if it could still beat the winner so far, so the active window is queried only for such a condition ([§12.4](triggers-conditions.md#s12-4)).
- Only the winner acts. An `Immediate` winner fires (on an auto-repeat only if it repeats, [§11.3](triggers-conditions.md#s11-3)). A `Tap`, `Hold` or `DoubleTap` winner arms together with every deferred binding on the same keys (in any order) with the same `when` (written the same way) that matches the press: one group. Of several bindings with the same trigger in the group only the first acts ([§11.7](triggers-conditions.md#s11-7); [src/engine/winner.rs:84-140](https://github.com/bluesteelll/key-switch-rs/blob/ed623c5/src/engine/winner.rs#L84-L140)).
- A losing binding neither fires, nor arms, nor asks for suppression. The winner, or a member of its group, with `block_default: true` asks for it.
- The election runs again on every auto-repeat ([§11.3](triggers-conditions.md#s11-3)).
- An auto-blocker only suppresses (`true`) or sends the menu mask key (`MaskOnly`), by its own rule ([§15.1](#s15-1), [§15.5](#s15-5)).
- `sequence` and `chord` bindings check their `when` first and are each processed independently, whichever `keys` binding wins. A key that is both a `keys` trigger and a sequence step or chord key drives both, and a sequence or chord that starts, advances or completes on it swallows it: a winner with `block_default: false` lets the key through only when no `sequence` or `chord` takes it.
- The key is suppressed if the winner or its group, an auto-blocker with `true`, or a `sequence` or `chord` asked for it ([§10](kinds.md#s10)). A suppressed key sends the menu mask key while Windows has Win or Alt down ([§13.1](actions.md#s13-1)).

<a id="s16-3"></a>
### 16.3 Practical consequences and issues

- With exact modifiers, bindings on the same key with different modifiers do not compete: `"Shift+CapsLock"` matches only with Shift held, and `"CapsLock"` only without it. Bindings compete when they have the same keys or when one sets `allow_extra_modifiers: true`.
- A binding with a condition wins over a binding without one on the same keys, wherever it sits in the file. It runs in the intended window, and in all other windows its `when` is false and the general binding fires. Before `feat/one-winner` it had to sit **above** the general binding.
- Of two identical `Immediate` bindings without `when`, only the upper one works; the loader warns about it ([§18.6](errors.md#s18-6)).
- A sided key does not count as more specific than its family: `"LShift+F13"` and `"Shift+F13"` have the same key count, so with the left Shift held the one higher in the file wins ([§19](known-issues.md#s19) row 71).

> [!NOTE]
> Fixed on `feat/one-winner` ([KS-13](../audit.md#ks-13), roadmap [item 1.6](../roadmap.md#16-one-winner-per-event)): a more specific binding overrides less specific ones whatever their triggers. With `"Shift+F13"` as `Immediate` and `"F13"` as `Tap(200)` with `allow_extra_modifiers: true`, pressing Shift+F13 runs the first binding and no longer arms the second; with `"Ctrl+F22"` on `Tap` and `"F22"` as `Immediate` with `allow_extra_modifiers: true`, pressing Ctrl+F22 no longer runs `"F22"`. Two consequences:
> - An `Immediate` binding and a `Tap`, `Hold` or `DoubleTap` binding on the same keys and `when` never act on the same press: the one higher in the file wins every press, and the loader does not warn about it ([§19](known-issues.md#s19) row 69).
> - A `Tap` and a `Hold` on the same keys with different `when` conditions do not arm together where both conditions hold ([§11.6](triggers-conditions.md#s11-6), row 70).

- The other matching rules and issues are described with their binding kinds: which bindings fire on an auto-repeat, the `repeat` flag ([§11.3](triggers-conditions.md#s11-3)), `block_default` when several bindings match ([§10](kinds.md#s10)), and two `DoubleTap` bindings on one press ([§11.7](triggers-conditions.md#s11-7)).
