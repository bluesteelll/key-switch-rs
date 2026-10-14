# Triggers and conditions

<a id="s11"></a>
## 11. Triggers (`trigger`)

<a id="s11-1"></a>
### 11.1 Values

| Value | RON | Fires |
|---|---|---|
| `Immediate` (default) | `trigger: Immediate`, or the field omitted | on key-down, including every auto-repeat |
| `Tap(ms)` | `trigger: Tap(200)` | on key-up, if the key is released less than `ms` after it was pressed |
| `Hold(ms)` | `trigger: Hold(300)` | `ms` after the press, if the key is still held |
| `DoubleTap(ms)` | `trigger: DoubleTap(250)` | on the second key-down of the same key within `ms` of the first key-down |

<a id="s11-2"></a>
### 11.2 Restrictions and parse errors

[src/config/loader.rs:224-230](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L224-L230), [src/config/schema.rs:61-68](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L61-L68):

- Deferred triggers (`Tap`, `Hold`, `DoubleTap`) work only with `keys`. For `sequence` and `chord` the loader reports ``Sequence/Chord cannot use a deferred trigger (Tap(200)); only `Immediate` (the default) is valid for these kinds``. The same holds for `Hold(…)`, `DoubleTap(…)` and even `Tap(0)`. An explicit `trigger: Immediate` is allowed for these kinds. This error is checked last: if the kind or the action of the same binding is invalid, only that error is reported. For example, `sequence: (steps: ["F13"])` with `Tap(100)` gives the error about the number of steps.
- The argument is required. It is a `u64` integer in milliseconds (for the accepted notations, see [§3.7](ron-syntax.md#s3-7)):

  | Written as | Error |
  |---|---|
  | `Tap` without parentheses (including `Tap 200`) | ``Expected opening `(` `` |
  | `Tap()`, `Tap(ms: 200)`, `Hold("300")` | `Expected integer` |
  | `Tap(-5)` | `Integer is out of bounds` |
  | `Tap(50.0)` | ``Expected closing `)` `` |
  | `Hold(1e3)` | `Invalid digit 'e' for base 10 integers` |
  | `Immediate(0)` | `Expected comma` |
  | `tap(200)` or any other unknown variant (the message names the variant as written; shown here for `LongPress(200)`) | ``Unexpected variant named `LongPress` in enum `RawTrigger`, expected one of `Immediate`, `Tap`, `Hold`, or `DoubleTap` instead`` |

- The range is not checked: `0` and any large value are accepted ([KS-27](../audit.md#ks-27)). With `0` the key is still swallowed, and the gestures behave as follows: `Tap(0)` almost never fires (the timer gets there before the release); `Hold(0)` fires almost immediately after the press; `DoubleTap(0)` practically never fires (the timer removes the entry before the second press).

<a id="s11-3"></a>
### 11.3 `Immediate`

[src/hook/keyboard_hook_callback.rs:229-237](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L229-L237). The action runs synchronously on key-down, on the hook thread.

- If several `Immediate` bindings match one press, only the first in check order runs: the one with the most keys, and on a tie the first in the file ([§16](matching.md#s16)).
- The event is suppressed if at least one of the matched `Immediate` bindings has `block_default: true`, even one that did not run.
- The auto-blocker is inserted before the binding that requested it, so it wins over `Immediate` bindings on the same combo that come later in the file ([§15.4](matching.md#s15-4), [KS-18](../audit.md#ks-18)).

> [!WARNING]
> **Known issue ([KS-10](../audit.md#ks-10)).** The action repeats on every auto-repeat ([src/hook/keyboard_hook_callback.rs:229](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L229)). `SwitchLanguage`, `Launch` and similar actions repeat at the auto-repeat rate ([§13.1](actions.md#s13-1)).

<a id="s11-4"></a>
### 11.4 Common to `Tap`, `Hold` and `DoubleTap`

[src/hook/keyboard_hook_callback.rs:238-252](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L238-L252), [src/hook/tap_state.rs](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs):

- A gesture is armed on a non-repeat key-down on which the `keys` combo matched. It is bound to the code of the key **just pressed** (`vk_code`). For `keys: "Ctrl+F17"` with `Tap(150)` this is F17 if Ctrl was pressed first, and the time runs from the F17 press to its release. If F17 is pressed first, the gesture is armed on Ctrl (code `LCtrl` or `RCtrl`): Ctrl is suppressed, and F17 goes through to the application.
- Deferred bindings are armed **regardless** of whether an `Immediate` binding ran on the same press: the "`Immediate` already fired" flag does not affect them.
- Suppression is forced: the press and the release of the trigger key never reach the application, whatever `block_default` says ([src/hook/keyboard_hook_callback.rs:241](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L241), [L246](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L246), [L251](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L251), [L317-L321](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L317-L321)). Combo keys pressed earlier (for example Ctrl) do reach the application.
- Auto-repeat does not re-arm the gesture, so `Hold` fires once per press.
- Each new arming starts a separate timer thread ([src/hook/tap_state.rs:61](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L61), [L105](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L105), [L158](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L158); see [KS-33](../audit.md#ks-33)).
- The `when` condition is checked only on key-down. `Tap` runs on release even if the window has changed in the meantime; `Hold` runs on its timer whatever window is active.

<a id="s11-5"></a>
### 11.5 How `Tap`, `Hold` and `DoubleTap` each resolve

**`Tap(ms)`.** On key-down an entry is put into the table. If an entry for this key already exists, the new one is ignored ([src/hook/tap_state.rs:51-53](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L51-L53)). A timer thread removes the entry after `ms` ([L65](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L65)). If the entry is still there at the key-up of the same key, the action runs synchronously on the hook thread ([L83](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L83)). If the key is released later, nothing happens, and the keystroke is lost.

**`Hold(ms)`.** After `ms` the timer thread removes the entry and runs the action ([src/hook/tap_state.rs:108-117](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L108-L117)). This is the only trigger whose action runs outside the hook thread, so `Launch` and `OpenUrl` can safely be used with `Hold` directly. If the key is released earlier, the gesture is cancelled ([L125-L131](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L125-L131)). It fires once. It does not hold a modifier down: `Hold` with `PressKey("LShift")` produces one short Shift press.

**`DoubleTap(ms)`.** The first press only arms the gesture; nothing runs. The timer thread removes the entry `ms` after the first press ([L162-L165](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L162-L165)). If the same key is pressed again before that, the action runs synchronously on key-down, while the mutex is held ([L146-L149](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L146-L149)). A third press starts a new cycle. A single press does nothing, and the keystroke is lost.

<a id="s11-6"></a>
### 11.6 `Tap` and `Hold` (or `DoubleTap`) on the same key

These are two separate bindings. Both gestures are armed independently, and nothing arbitrates between them. The result depends on how long the key is held, `t`:

| Duration `t` | Result |
|---|---|
| `t < Tap` and `t < Hold` | Tap |
| `t ≥ Hold` and `t ≥ Tap` | Hold |
| `Hold ≤ t < Tap` (possible when `Tap > Hold`) | **both**: Hold while the key is held, Tap on release |
| `Tap ≤ t < Hold` (possible when `Tap < Hold`) | nothing |

> [!WARNING]
> **Known issue ([KS-25](../audit.md#ks-25)).** With equal values, for example `Tap(200)` and `Hold(200)`, the thread scheduler decides the outcome of a release near the boundary ([src/hook/tap_state.rs:105](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L105)). Tap, Hold, both actions at once, or neither are all possible. Do not make `Tap` larger than `Hold`, or a window appears in which both fire. It is more reliable to make `Tap` slightly smaller than `Hold`, for example `Tap(180)` and `Hold(200)`: a short window with no action is better than a double action.

```ron
( keys: "F13", action: Media(PlayPause), trigger: Tap(180) ),
( keys: "F13", action: Media(Next),      trigger: Hold(200) ),
```

**`Tap` and `DoubleTap` on the same key** are also allowed, but the ambiguity is not resolved in any way. A double press produces three actions: Tap (release 1), DoubleTap (press 2), Tap (release 2). A single press produces Tap.

<a id="s11-7"></a>
### 11.7 Several deferred bindings of the same type on one press

This happens with one key and overlapping `when` conditions, or with `"Shift+F13"` and `"F13"` while Shift is held. The gesture table is keyed by the key code only:

- `Tap` and `Hold`: only the first matching binding in check order ([§16](matching.md#s16)) is armed; the others are silently dropped ([src/hook/tap_state.rs:51](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L51), [L97](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L97)). This can be used on purpose: with `"Shift+F13"` on `Tap` and `"F13"` on `Tap`, pressing Shift+F13 runs only the first one.
- `DoubleTap`: see the warning below.
- A related problem: a more specific binding does not override less specific deferred ones ([§16.3](matching.md#s16-3), [KS-13](../audit.md#ks-13)).

> [!WARNING]
> **Known issue: several `DoubleTap` bindings.** The second binding takes the first binding's entry, and the first binding's action runs already on the **first** press ([src/hook/tap_state.rs:146-152](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L146-L152); inferred from the code).

<a id="s11-8"></a>
### 11.8 Defects

> [!WARNING]
> **Known issue ([KS-04](../audit.md#ks-04)): a gesture arms on any key pressed while the trigger is held** ([src/hook/keyboard_hook_callback.rs:240](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L240), also [L245](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L245) and [L250](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L250)). The match is a subset test, and the gesture is armed on `vk_code`. Example: `keys: "CapsLock", trigger: Tap(200)`. If you hold CapsLock and quickly press and release `A`, the Tap action runs and `A` is lost.

> [!WARNING]
> **Known issue ([KS-11](../audit.md#ks-11)): stale timers** ([src/hook/tap_state.rs:65](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L65), [L108-L117](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L108-L117), [L162-L165](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L162-L165)). A timer carries no generation number, so the thread of an earlier press removes or runs the entry of the next press. Example: CapsLock with `Tap(200)` and `Hold(200)`. Press at 0 ms, release at 80 ms: Tap fires. New press at 150 ms. At 200 ms the old Hold timer runs the Hold action only 50 ms after the second press, and the old Tap timer removes the second press's entry, so its Tap is lost. With `DoubleTap`, in a fast series the second double press in a row may not fire.

> [!WARNING]
> **Known issue ([KS-06](../audit.md#ks-06)): stuck keys.** While the key is held, auto-repeat passes through to the application while the release is suppressed, and the key gets stuck ([src/hook/keyboard_hook_callback.rs:239](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L239), [L244](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L244), [L249](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L249) together with [L324](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L324)). It is especially noticeable with `Hold`.

> [!WARNING]
> **Known issue ([KS-08](../audit.md#ks-08)): actions on the hook thread.** `Tap` and `DoubleTap` actions, as well as `chord` and `sequence` completions, run directly in the hook handler ([src/hook/tap_state.rs:83](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L83), [L148](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/tap_state.rs#L148)). A slow `Launch` or `OpenUrl` can exceed `LowLevelHooksTimeout`, and Windows then silently removes the hook ([§13.1](actions.md#s13-1)).

> [!WARNING]
> **Deferred triggers on modifiers are unreliable.** Sided names such as `LShift` in `keys` never match ([§6.7](keys.md#s6-7), [KS-07](../audit.md#ks-07)), so the example `keys: "LShift", trigger: DoubleTap(250)` ([README.md:262](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/README.md?plain=1#L262), [src/config/default_config.ron:200-205](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L200-L205)) loads but does not work. `Shift`, `Ctrl` and `Alt` have a different problem. Known issue ([KS-14](../audit.md#ks-14)): `sync_modifiers` clears a suppressed modifier on the next event of another key ([src/hook/keyboard_hook_callback.rs:95](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L95)), and the modifier's auto-repeat then looks like a new press.

> [!WARNING]
> **Lone Win or Alt ([KS-12](../audit.md#ks-12)).** For `Win+…` and `Alt+…` with a deferred trigger, only the trigger key is suppressed, so releasing Win or Alt can open the Start menu or the menu bar ([src/hook/keyboard_hook_callback.rs:294](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L294), [§13.1](actions.md#s13-1)).

> [!WARNING]
> **Hot reload and armed gestures.** A hot reload does not reset gestures that are already armed ([src/hook/config_watcher.rs:103](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L103)). They complete with the old actions.

<a id="s12"></a>
## 12. Conditions (`when`)

<a id="s12-1"></a>
### 12.1 Variants and syntax

```text
when: Always                                    // default, same as omitting the field
when: TitleContains("Visual Studio Code")
when: TitleEquals("Калькулятор")
when: Not(TitleContains("Incognito"))
when: And([TitleContains("Chrome"), Not(TitleContains("Incognito"))])
when: Or([TitleContains("Firefox"), TitleContains("Chrome")])
when: AppEquals("code.exe")                     // currently never matches (§12.3)
```

| Variant | True when | Case handling ([src/data/condition.rs:57-78](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L57-L78)) |
|---|---|---|
| `Always` | always | not applicable |
| `TitleContains("s")` | the title of the active window contains the substring | Unicode `to_lowercase` on both sides ([L67](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L67)): case-insensitive matching of Cyrillic works |
| `TitleEquals("s")` | the title matches in full | case-insensitive for ASCII only ([L71](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L71), [KS-28](../audit.md#ks-28)): `"ПРИВЕТ"` ≠ `"Привет"` |
| `AppEquals("x.exe")` | the base name of the executable of the active window's process equals the string (including `.exe`) | case-insensitive for ASCII only ([L62](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L62), [KS-28](../audit.md#ks-28)) |
| `Not(c)` | `c` is false | not applicable |
| `And([..])` | all are true; evaluation stops at the first false one; an empty list is true | not applicable |
| `Or([..])` | at least one is true; evaluation stops at the first true one; an empty list is false | not applicable |

- `Not` takes one condition; `And` and `Or` take a list in `[ ]`. Conditions can be nested inside each other, but not without limit: ron has a nesting limit ([§3.11](ron-syntax.md#s3-11)), which in practice means 60 levels of `Not(` or 30 levels of `And([`.
- The string is written in double quotes; raw strings `r"..."` work too. Whitespace is not trimmed. There are no wildcard patterns and no regular expressions.
- There are no semantic checks: `And([])`, `Or([])` and `TitleContains("")` are accepted without warnings.
- If the data is unavailable (no active window, an empty title, or a failed query), each of the three simple conditions is false, and `Not(...)` of it is true. `TitleEquals("")` is never true. `TitleContains("")` is true for any window with a non-empty title.

<a id="s12-2"></a>
### 12.2 Parse errors

| Written as | Error |
|---|---|
| `And(TitleContains("a"))` | ``Expected opening `[` `` |
| `Not(a, b)` | ``Expected closing `)` `` |
| `Not([...])` | `Expected identifier` |
| `when: "chrome.exe"` | `Expected identifier` |
| `AppEquals`, `Not` or `And` without parentheses | ``Expected opening `(` `` |
| `Always()` | `Expected comma` |
| `TitleContains('x')` (single quotes) | `Expected string` |
| `when: None` | ``Unexpected variant named `None` in enum `RawCondition` `` |
| an unknown variant, including a known one in a different case (`titleContains`) | ``Unexpected variant named `AppContains` in enum `RawCondition`, expected one of `Always`, `AppEquals`, `TitleContains`, `TitleEquals`, `Not`, `And`, or `Or` instead`` |
| nesting too deep | ``Exceeded recursion limit, try increasing `ron::Options::recursion_limit` …`` |

<a id="s12-3"></a>
### 12.3 `AppEquals` does not work

> [!WARNING]
> **Known issue ([KS-03](../audit.md#ks-03)): `AppEquals` currently never matches** ([src/data/condition.rs:139-144](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L139-L144)). The process is opened with the `PROCESS_QUERY_LIMITED_INFORMATION` right ([L139](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L139)), but `GetModuleBaseNameW` ([L144](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L144)) needs `PROCESS_QUERY_INFORMATION | PROCESS_VM_READ`. It returns 0 (ACCESS_DENIED), and the application name always stays unknown. Consequences:
>
> - `AppEquals("x")` is always false;
> - `Not(AppEquals("x"))` is always true, including inside `x` itself;
> - `And([AppEquals(..), …])` is always false, and `Or([AppEquals(..), C])` behaves like plain `C`.

Until the bug is fixed, use **`TitleContains`** with a distinctive part of the window title:

```ron
( keys: "F13", action: PressKey("F5"), when: TitleContains("Visual Studio Code") ),
( keys: "F14", action: Media(PlayPause),
  when: Or([TitleContains("Google Chrome"), TitleContains("Mozilla Firefox")]) ),
( keys: "F15", action: DoNothing, when: Not(TitleContains("Visual Studio Code")) ),
```

A title is not unique: a browser tab with the text "Visual Studio Code" matches as well. If needed, narrow the condition with `And([...])`. For UWP apps (Calculator, Settings) the active window belongs to the `ApplicationFrameHost.exe` process, so for them the title is more reliable even after `AppEquals` is fixed. A possible fix is `QueryFullProcessImageNameW`, which works with the `PROCESS_QUERY_LIMITED_INFORMATION` right.

<a id="s12-4"></a>
### 12.4 When and how a condition is checked

[src/hook/keyboard_hook_callback.rs:204-221](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L204-L221), [src/data/condition.rs:81-163](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L81-L163):

- Window data is collected only on key-down, and only if at least one binding has a condition other than `Always` (judged at the top level: `Not(Always)` or `And([])` already count as a condition). The window is found with `GetForegroundWindow()`: this is the top-level window, not the child control that has focus.
- The data is queried lazily and at most once per keystroke: the executable name and the title are read when first needed and cached until the keystroke has been handled. The title is read with `GetWindowTextW` into a buffer of 512 UTF-16 code units; anything longer than 511 is truncated.
- Each binding's condition is checked **before** its keys are compared. So if the config contains even one `when`, the Win32 queries run on every press of any key, auto-repeat included.
- The condition is not re-checked on key-up or in timers ([§11.4](#s11-4)). For `sequence`, the condition is checked at every step; if it is false, the step is skipped, and the wait is not reset ([§8.2](kinds.md#s8-2)).

> [!WARNING]
> The auto-blocker created by `block_original_combo: true` does not inherit the `when` of its binding and always gets `Always` ([src/data/binding.rs:95](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/binding.rs#L95), [§15.4](matching.md#s15-4), [KS-35](../audit.md#ks-35)).
