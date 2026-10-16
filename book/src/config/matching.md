# Auto-blockers and matching order

`block_original_combo: true` silences the Windows hotkey that does the same job as a binding's action, such as Alt+Shift for `SwitchLanguage`. The second half of this page gives the order in which the daemon checks bindings and what it means when several match. Facts describe commit `1dfd80a`, those about the `repeat` field the branch `feat/repeat-flag`, and those about `block_default` on `Tap`, `Hold` and `DoubleTap` the branch `feat/deferred-block-default`; the run-time behaviour is derived from the code of the decision core and its scenario tests, not observed on a live daemon.

<a id="s15"></a>
## 15. `block_original_combo`: auto-blockers

Code: `expand_with_auto_blockers` ([src/config/loader.rs:271-312](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L271-L312)) builds the blockers, and the engine matches them ([src/engine/mod.rs:382-394](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L382-L394), [src/engine/matching.rs:81-101](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/matching.rs#L81-L101)). The registry is read in [src/system/registry.rs](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs).

<a id="s15-1"></a>
### 15.1 What the flag does

After all bindings have been validated, the loader collects the system hotkeys of the bindings that have `block_original_combo: true`, one per distinct hotkey. For each hotkey it **appends** an auto-blocker after all of the file's bindings, unless a binding of the file already uses the hotkey's keys ([§15.3](#s15-3)). The auto-blocker is a combination binding on the system hotkey with `action: DoNothing`, `trigger: Immediate` and `when: Always`, marked as a blocker ([src/data/binding.rs:98-109](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/binding.rs#L98-L109)). In the binding list printed at startup it appears last, as `[AUTO-BLOCK] … -> (blocked)`.

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

An auto-blocker is not an ordinary binding at run time ([src/engine/mod.rs:382-394](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L382-L394)):

- **It acts only on the hotkey pressed on its own.** It suppresses a key-down when the key just pressed is one of the hotkey's keys, all of them are held, and every other key held is one that Windows does not have down ([src/engine/matching.rs:81-101](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/matching.rs#L81-L101)). A key "Windows does not have down" is one whose key-down a binding swallowed, or one whose key-up the hook missed. Alt, then Shift is blocked. Shift pressed while Alt and Tab are held, Shift pressed while Ctrl and T are held, and Alt+Shift pressed while a letter is still held in fast typing reach Windows. Ctrl+Alt+Shift does not match the `Alt+Shift` blocker.
- **It fires nothing** and never takes the place of a binding of yours: a binding that matches the same key-down fires as if the blocker were not there.
- **It suppresses the whole press.** The swallowed key's auto-repeats and release are swallowed too, even after the other hotkey key is released ([§10](kinds.md#s10)). The swallowed key still counts as held for matching, so `Alt+Shift+X` bindings fire.

Further points:

- Only `SwitchLanguage` and `SwitchLanguageBackward` have a system hotkey. `ToggleCapsLock` and all other actions have none: the flag does nothing, and there is no error either.
- The binding kind does not matter: a blocker is also added for a `sequence` or a `chord` with these actions, and the blocker is still a combination.
- The blocker does not affect the daemon's own synthesized input ([§13.1](actions.md#s13-1)).

<a id="s15-2"></a>
### 15.2 How the system combination is determined

[src/system/registry.rs:17-79](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs#L17-L79), [src/system/registry.rs:181-189](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs#L181-L189):

- The key `HKCU\Keyboard Layout\Toggle` is read: first the value `Hotkey`, and, if it is absent or yields no combination, the value `Language Hotkey`. `REG_SZ` and `REG_EXPAND_SZ` strings are accepted (leading and trailing whitespace is trimmed from the value); a value of any other type is skipped with the warning `[WARN] registry value "Hotkey" has type …, expected REG_SZ/REG_EXPAND_SZ`. Values:
  - `"1"` → `Alt+Shift`;
  - `"2"` → `Ctrl+Shift`;
  - `"4"` → `` ` `` (`VK_OEM_3`; on the Russian layout this is the `Ё` key). The blocker swallows that key when it is pressed with no other key down in Windows, so `` ` `` typed on its own does nothing, while Shift+`` ` `` and a `` ` `` typed while another key is held reach Windows (and may switch the layout);
  - `"3"` ("not assigned") and any other value yield no combination. If neither of the two values yields a combination, or the key does not exist, the **fallback `Alt+Shift`** is used.
- The modifiers of these hotkeys mean either side: the `Alt+Shift` blocker covers both Alt keys and both Shift keys.
- The console shows `[INFO] SwitchLanguage: registry combination Alt+Shift` (the keys by name) or `[INFO] SwitchLanguage: using default combination`. For `SwitchLanguageBackward` the line starts with that name instead. The line is printed for every binding that has the flag.
- The registry is read on every load of the file: at startup and on every reload.
- The value `Layout Hotkey` (switching layouts within one language) is not read, and Win+Space is not blocked ([`Layout Hotkey` and Win+Space](../gaps.md#layout-hotkey-and-winspace)).

<a id="s15-3"></a>
### 15.3 Deduplication

A blocker is not added if **any** `keys` binding of the file, the flagged binding included, has exactly the hotkey's keys, in any order ([src/config/loader.rs:294-307](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L294-L307); [KS-18](../audit.md#ks-18), fixed in `1dfd80a`). The console then says so:

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
- Two flagged bindings with the same hotkey produce one blocker.

`( keys: "Alt+Shift", action: SwitchLanguageBackward, block_original_combo: true )` on its own therefore gets no blocker either, and its action runs.

<a id="s15-4"></a>
### 15.4 Known auto-blocker issues

> [!WARNING]
> **The second modifier is swallowed.** Pressed modifiers first, as in Alt, Shift, X, the blocker matches on Shift, so the application gets Alt+X; in the order Shift, Alt, X it swallows Alt, and the application gets Shift+X. With the `Ctrl+Shift` hotkey, Ctrl, Shift, T arrives as Ctrl+T. That includes the usual way back in the Alt+Tab switcher, Alt held, Tab released, then Shift+Tab, which still moves forward. A lone Alt also reaches Windows and can activate the menu bar ([KS-12](../audit.md#ks-12)). [KS-05](../audit.md#ks-05) is partly fixed; roadmap [item 1.2](../roadmap.md#12-mask-key-and-auto-blocker-v2) fixes the rest.

> [!WARNING]
> The value `"3"` (hotkey disabled in Windows) still produces an `Alt+Shift` blocker ([src/system/registry.rs:185](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs#L185), [src/system/registry.rs:60-64](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs#L60-L64)). See [KS-17](../audit.md#ks-17), open until roadmap [item 1.4](../roadmap.md#14-tri-state-language-hotkey).

> [!WARNING]
> The blocker does not inherit the owner's `when` and `trigger`. It is active in every window, even if the owning binding is restricted to one window or uses `Tap(…)` ([src/data/binding.rs:98-109](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/binding.rs#L98-L109)). See [KS-35](../audit.md#ks-35), open until roadmap [item 1.2](../roadmap.md#12-mask-key-and-auto-blocker-v2).

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

On every key-down ([src/engine/mod.rs:365-500](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L365-L500)) **all** bindings are checked in this order. No binding takes the event away from the others.

- `when` is checked first. If the condition is false, the binding is skipped entirely. The active window is queried only when a condition other than `Always` is evaluated ([§12.4](triggers-conditions.md#s12-4)).
- `keys` + `Immediate`: a binding matches by the rules of [§7](kinds.md#s7). **Only the first** matching binding runs. The other matching `Immediate` bindings do not run, but if they have `block_default: true`, the key is still suppressed.
- `keys` + `Tap`/`Hold`/`DoubleTap`: these arm regardless of whether an `Immediate` binding has run, and ask for suppression if they have `block_default: true` (always before `feat/deferred-block-default`; [§11.4](triggers-conditions.md#s11-4)). Of several matching `Tap` bindings (or several `Hold` bindings), only the first is armed ([§11.7](triggers-conditions.md#s11-7)). A `Tap` and a `Hold` on the same key are both armed.
- An auto-blocker only suppresses, by its own rule ([§15.1](#s15-1)).
- `sequence` and `chord` bindings are each processed independently, with no link to combinations.
- The key is suppressed if at least one binding requested it ([§10](kinds.md#s10)).

<a id="s16-3"></a>
### 16.3 Practical consequences and issues

- With exact modifiers, bindings on the same key with different modifiers do not compete: `"Shift+CapsLock"` matches only with Shift held, and `"CapsLock"` only without it. Order matters when bindings have the same keys or when one sets `allow_extra_modifiers: true`.
- A binding with a condition gets no priority over a binding without one: at equal key count, file order decides. Put the binding with `when` **above** the general binding on the same combination. Then it runs in the intended window, and in all other windows its `when` is false and the general binding fires.
- Of two identical `Immediate` bindings without `when`, only the upper one works; there is no warning about this.

> [!WARNING]
> More specific bindings do not override less specific deferred ones ([src/engine/mod.rs:403-453](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L403-L453)). For example, with `"Shift+F13"` as `Immediate` and `"F13"` as `Tap(200)` with `allow_extra_modifiers: true`, pressing Shift+F13 runs the first binding and **arms** the second. The reverse also holds: if the more specific binding is deferred (`"Ctrl+F22"` with `Tap`) and the less specific one is `Immediate` with `allow_extra_modifiers: true` (`"F22"`), pressing Ctrl+F22 runs `"F22"` at once. Make overlapping bindings on one key the same type: all `Immediate` or all `Tap`. See [KS-13](../audit.md#ks-13), open until roadmap [item 1.6](../roadmap.md#16-one-winner-per-event).

- The other matching rules and issues are described with their binding kinds: which bindings fire on an auto-repeat, the `repeat` flag ([§11.3](triggers-conditions.md#s11-3)), `block_default: false` when another binding also matches ([§10](kinds.md#s10); [KS-13](../audit.md#ks-13)), and two `DoubleTap` bindings on one press ([§11.7](triggers-conditions.md#s11-7)).
