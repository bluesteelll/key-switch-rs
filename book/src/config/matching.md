# Auto-blockers and matching order

<a id="s15"></a>
## 15. `block_original_combo`: auto-blockers

Code: `expand_with_auto_blockers` ([src/config/loader.rs:172-192](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L172-L192)); the registry is read in [src/system/registry.rs](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/system/registry.rs).

<a id="s15-1"></a>
### 15.1 What the flag does

After all bindings have been validated, the loader walks through them in file order. If a binding has `block_original_combo: true` and its action has a system hotkey, an auto-blocker is inserted **immediately before it**. The auto-blocker is a combination binding on the system combination with `action: DoNothing`, `block_default: true`, `trigger: Immediate` and `when: Always` ([src/data/binding.rs:89-99](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/binding.rs#L89-L99)). In the binding list printed at startup it appears as `[AUTO-BLOCK] … -> (blocked)`.

```ron
( bindings: [ ( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true ) ] )
```

This produces 2 bindings: `Alt+Shift → DoNothing` (the auto-blocker; this is the default combination unless the registry specifies otherwise) and `CapsLock → SwitchLanguage`.

- Only `SwitchLanguage` and `SwitchLanguageBackward` have a system combination. `ToggleCapsLock` and all other actions have none: the flag does nothing, and there is no error either.
- The binding kind does not matter: a blocker is also added for a `sequence` or a `chord` with these actions, and the blocker is still an ordinary combination.
- The blocker does not affect the daemon's own synthesized input ([§13.1](actions.md#s13-1)).

<a id="s15-2"></a>
### 15.2 How the system combination is determined

[src/system/registry.rs:21-91](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/system/registry.rs#L21-L91), [src/system/registry.rs:192-200](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/system/registry.rs#L192-L200):

- The key `HKCU\Keyboard Layout\Toggle` is read: first the value `Hotkey`, and, if it is absent or yields no combination, the value `Language Hotkey`. `REG_SZ` and `REG_EXPAND_SZ` strings are accepted (leading and trailing whitespace is trimmed from the value); a value of any other type is skipped with the warning `[WARN] registry value "Hotkey" has type …, expected REG_SZ/REG_EXPAND_SZ`. Values:
  - `"1"` → `Alt+Shift`;
  - `"2"` → `Ctrl+Shift`;
  - `"4"` → `` ` `` (`VK_OEM_3`; on the Russian layout this is the `Ё` key). A single-key blocker makes that key completely "dead";
  - `"3"` ("not assigned") and any other value yield no combination. If neither of the two values yields a combination, or the key does not exist, the **fallback `Alt+Shift`** is used.
- The console shows `[INFO] SwitchLanguage: registry combination [...]` (a list of key codes, for example `[VIRTUAL_KEY(18), VIRTUAL_KEY(16)]`) or `[INFO] SwitchLanguage: using default combination`. For `SwitchLanguageBackward` the line starts with that name instead. The line is printed for every binding that has the flag.
- The registry is read on every load of the file: at startup and on every reload.
- The value `Layout Hotkey` (switching layouts within one language) is not read, and Win+Space is not blocked ([src/system/registry.rs:27](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/system/registry.rs#L27)).

<a id="s15-3"></a>
### 15.3 Deduplication

A blocker is not added if the entries **already added** (bindings higher up in the file and blockers created earlier) include a combination with the same set of keys; key order does not matter ([src/config/loader.rs:181-183](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L181-L183)).

- Only `keys` bindings count (not `sequence` and not `chord`).
- The key set must match exactly: `Ctrl+Alt+Shift` does not cancel an `Alt+Shift` blocker.
- The `when`, `trigger`, `block_default` and action of that binding play no role.

So two bindings with the flag produce one blocker, and your own `Shift+Alt` binding **above** the flagged binding prevents the blocker from being created.

<a id="s15-4"></a>
### 15.4 Known auto-blocker issues

> [!WARNING]
> A combination counts as matched when all of its keys are held; extra keys are allowed. So the `Alt+Shift` blocker intercepts every `Alt+Shift+X` shortcut and AltGr+Shift typing, and the `Ctrl+Shift` blocker (value `"2"`) intercepts every `Ctrl+Shift+X`. The key-down of the second modifier is suppressed, and three-key shortcuts lose a modifier ([src/data/key_combination.rs:49](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/key_combination.rs#L49), [src/hook/keyboard_hook_callback.rs:234](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L234), [src/config/loader.rs:185](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L185)). See [KS-05](../audit.md#ks-05) and, for superset matching in general, [KS-04](../audit.md#ks-04).

> [!WARNING]
> The value `"3"` (hotkey disabled in Windows) still produces an `Alt+Shift` blocker ([src/system/registry.rs:196](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/system/registry.rs#L196)). See [KS-17](../audit.md#ks-17).

> [!WARNING]
> Deduplication looks only at the entries above, while the blocker sits before its owner and fires as `Immediate`. So your own **`Immediate`** binding on the same combination **below** the flagged binding never runs its action: on that key press an `Immediate` binding (the blocker) has already fired. The same applies to the owning binding itself if its `keys` equal the system combination: `( keys: "Alt+Shift", action: SwitchLanguage, block_original_combo: true )` never switches the language ([src/config/loader.rs:181](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L181)). Deferred bindings (`Tap`/`Hold`/`DoubleTap`) are not affected: they arm independently of `Immediate` ([§11.4](triggers-conditions.md#s11-4)), so, for example, an `Alt+Shift` binding with `Tap(…)` below the blocker runs its action on release. Workaround for `Immediate`: put your binding on this combination above the flagged binding. See [KS-18](../audit.md#ks-18).

> [!WARNING]
> Deduplication ignores the `when` and `trigger` of the binding above. If, for example, `( keys: "Shift+Alt", …, when: TitleContains("x") )` sits above, no blocker is created at all, and outside that window the system combination is not blocked ([src/config/loader.rs:181-183](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L181-L183)).

> [!WARNING]
> The blocker does not inherit the owner's `when` and `trigger`. It is active in every window, even if the owning binding is restricted to one window or uses `Tap(…)` ([src/data/binding.rs:95](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/binding.rs#L95)). See [KS-35](../audit.md#ks-35).

<a id="s16"></a>
## 16. Matching order and binding priority

<a id="s16-1"></a>
### 16.1 Sorting

1. The loader returns the bindings in file order; each auto-blocker sits before its owner.
2. `KeyboardHook::update_bindings` ([src/hook/keyboard_hook.rs:71-74](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook.rs#L71-L74)) is called at startup ([src/core/app.rs:90](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/core/app.rs#L90)) and on every reload. It sorts the list **stably** by "key count", from largest to smallest ([src/data/binding.rs:42-48](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/binding.rs#L42-L48)):
   - for `keys`, the number of distinct keys after duplicates are removed (`"Ctrl+Alt+L"` = 3);
   - for `sequence`, the number of **steps**;
   - for `chord`, the number of keys.

   Ties keep the file order.

<a id="s16-2"></a>
### 16.2 Matching on every key press

On every key-down (`handle_key_down`, [src/hook/keyboard_hook_callback.rs:216-292](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L216-L292)) **all** bindings are checked in this order. No binding takes the event away from the others.

- `when` is checked first. If the condition is false, the binding is skipped entirely. The active window is queried only if at least one binding has a `when` other than `Always` ([§12.4](triggers-conditions.md#s12-4)).
- `keys` + `Immediate`: **only the first** matching binding runs. The other matching `Immediate` bindings do not run, but if they have `block_default: true`, the key is still suppressed.
- `keys` + `Tap`/`Hold`/`DoubleTap`: these arm regardless of whether an `Immediate` binding has run. Of several matching `Tap` bindings (or several `Hold` bindings), only the first is armed ([§11.7](triggers-conditions.md#s11-7)). A `Tap` and a `Hold` on the same key are both armed.
- `sequence` and `chord` bindings are each processed independently, with no link to combinations.
- The key is suppressed if at least one binding requested it ([§10](kinds.md#s10)).

<a id="s16-3"></a>
### 16.3 Practical consequences and issues

- `"Shift+CapsLock"` (2 keys) is checked before `"CapsLock"` (1 key), so while Shift is held only the former runs.
- A binding with a condition gets no priority over a binding without one: at equal key count, file order decides. Put the binding with `when` **above** the general binding on the same combination. Then it runs in the intended window, and in all other windows its `when` is false and the general binding fires.
- Of two identical `Immediate` bindings without `when`, only the upper one works; there is no warning about this.

> [!WARNING]
> More specific bindings do not override less specific deferred ones ([src/hook/keyboard_hook_callback.rs:238](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L238)). For example, with `"Shift+F13" → Immediate` and `"F13" → Tap(200)`, pressing Shift+F13 runs the first binding and **arms** the second. The reverse also holds: if the more specific binding is deferred (`"Ctrl+F22"` with `Tap`) and the less specific one is `Immediate` (`"F22"`), pressing Ctrl+F22 runs `"F22"` at once. Make overlapping bindings on one key the same type: all `Immediate` or all `Tap`. See [KS-13](../audit.md#ks-13).

- The other matching issues are described with their binding kinds: firing on an unrelated key press and on auto-repeat ([§7](kinds.md#s7); [KS-04](../audit.md#ks-04), [KS-10](../audit.md#ks-10)), the auto-repeat leak ([§10](kinds.md#s10); [KS-06](../audit.md#ks-06)), `block_default: false` when another binding also matches ([§10](kinds.md#s10); [KS-13](../audit.md#ks-13)), and two `DoubleTap` bindings on one press ([§11.7](triggers-conditions.md#s11-7)).
