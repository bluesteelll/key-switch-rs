# Triggers and conditions

`trigger` decides when a `keys` binding fires: at once (`Immediate`), on a short press (`Tap`), on a long press (`Hold`) or on a double press (`DoubleTap`). `when` restricts a binding to the windows whose application or title matches. Facts describe commit `1dfd80a`; the run-time behaviour is derived from the code of the decision core and its scenario tests, not observed on a live daemon.

<a id="s11"></a>
## 11. Triggers (`trigger`)

<a id="s11-1"></a>
### 11.1 Values

| Value | RON | Fires |
|---|---|---|
| `Immediate` (default) | `trigger: Immediate`, or the field omitted | on key-down, including every auto-repeat |
| `Tap(ms)` | `trigger: Tap(200)` | on key-up, if the key is released less than `ms` after it was pressed |
| `Hold(ms)` | `trigger: Hold(300)` | `ms` after the press, if the key was not released before that |
| `DoubleTap(ms)` | `trigger: DoubleTap(250)` | on the second key-down of the same key less than `ms` after the first key-down |

<a id="s11-2"></a>
### 11.2 Restrictions and parse errors

[src/config/loader.rs:353-362](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L353-L362), [src/config/loader.rs:473-497](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L473-L497), [src/config/schema.rs:70-80](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/schema.rs#L70-L80):

- Deferred triggers (`Tap`, `Hold`, `DoubleTap`) work only with `keys`. For `sequence` and `chord` the loader reports ``Sequence/Chord cannot use a deferred trigger (Tap(200)); only `Immediate` (the default) is valid for these kinds``. The same holds for `Hold(…)` and `DoubleTap(…)`. An explicit `trigger: Immediate` is allowed for these kinds. This error is checked after the kind, the action and the range of the trigger: if one of those is invalid, only that error is reported. For example, `sequence: (steps: ["F13"])` with `Tap(100)` gives the error about the number of steps.
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

- The value must be 1 to 10000 ms ([KS-27](../audit.md#ks-27), fixed in `1dfd80a`). Outside that range the binding is rejected with a message that names it, counting from 0: `binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms`, or `Hold(10001) is out of range: …`.

<a id="s11-3"></a>
### 11.3 `Immediate`

[src/engine/mod.rs:404-412](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L404-L412). The action fires on key-down and runs on the hook thread, after the decision ([§13.1](actions.md#s13-1)).

- If several `Immediate` bindings match one press, only the first in check order runs: the one with the most keys, and on a tie the first in the file ([§16](matching.md#s16)).
- The event is suppressed if at least one of the matched `Immediate` bindings has `block_default: true`, even one that did not run ([KS-13](../audit.md#ks-13)).
- An auto-blocker never takes this slot: it fires nothing and only suppresses ([§15](matching.md#s15)).

> [!WARNING]
> **Known issue ([KS-10](../audit.md#ks-10)).** The action repeats on every auto-repeat while the binding matches ([src/engine/mod.rs:404-412](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L404-L412)). `SwitchLanguage`, `Launch` and similar actions repeat at the auto-repeat rate ([§13.1](actions.md#s13-1)). Roadmap [item 1.1](../roadmap.md#11-per-binding-repeat-flag) adds a `repeat` flag.

<a id="s11-4"></a>
### 11.4 Common to `Tap`, `Hold` and `DoubleTap`

[src/engine/mod.rs:413-452](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L413-L452), [src/engine/gestures.rs](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs):

- A gesture is armed on a non-repeat key-down that fires the `keys` binding by the rules of [§7](kinds.md#s7): the key just pressed is one of the binding's trigger keys, all its keys are held and the modifiers are exactly its own. The gesture is bound to that key's code, and that key's key-up resolves it. For `keys: "Ctrl+F17"` with `Tap(150)`, press Ctrl, then F17: the time runs from the F17 press to its release. Pressing F17 first, then Ctrl arms nothing.
- A key pressed while the gesture is pending is not the binding's key: it passes and arms nothing.
- Deferred bindings are armed **regardless** of whether an `Immediate` binding ran on the same press: the "`Immediate` already fired" flag does not affect them ([KS-13](../audit.md#ks-13)).
- Suppression is forced: the press, the auto-repeats and the release of the trigger key never reach the application, whatever `block_default` says ([§10](kinds.md#s10)). Combo keys pressed earlier (for example Ctrl) do reach the application. Roadmap [item 1.3](../roadmap.md#13-block_default-for-tap-hold-and-doubletap) makes `block_default` apply.
- Auto-repeat does not re-arm the gesture, so `Hold` fires once per press.
- `Tap` and `DoubleTap` store a deadline and start no thread; the next event of the key checks it. Each `Hold` starts one short-lived `hold-timer` thread ([src/hook/keyboard_hook_callback.rs:197-218](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L197-L218)). If that thread cannot be started, the daemon prints `[WARN] could not start a hold timer, that Hold will not fire: <error>` and the Hold is dropped ([KS-33](../audit.md#ks-33), partly fixed).
- The `when` condition is checked only on key-down. `Tap` runs on release even if the window has changed in the meantime; `Hold` runs on its timer whatever window is active.

<a id="s11-5"></a>
### 11.5 How `Tap`, `Hold` and `DoubleTap` each resolve

[src/engine/gestures.rs:1-27](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L1-L27) describes the rules; [src/engine/mod.rs:538-564](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L538-L564) applies them on key-up.

**`Tap(ms)`.** On key-down an entry with the deadline "press time + `ms`" is stored. While a live entry for this key exists, a new one is ignored ([src/engine/gestures.rs:91-112](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L91-L112)). At the key-up of the same key the entry is removed, and the action fires if the release came strictly before the deadline ([L114-L121](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L114-L121)). The action runs on the hook thread. If the key is released later, nothing happens, and the keystroke is lost.

**`Hold(ms)`.** Arming gives the entry a new generation number and asks for a timer ([L125-L154](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L125-L154)). The `hold-timer` thread sleeps until the deadline and then asks the engine to fire the entry with its token; a token whose entry is gone or re-armed fires nothing ([L171-L179](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L171-L179)). A key-up before the deadline cancels the gesture. A key-up at or after the deadline does not: the key was held for the whole term, so the timer fires the Hold even if its thread wakes a few milliseconds after the release ([L156-L169](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L156-L169)). `Hold` is the only trigger whose action runs outside the hook thread: it runs on the `hold-timer` thread, so `Launch` and `OpenUrl` can safely be used with `Hold` directly. It fires once. It does not hold a modifier down: `Hold` with `PressKey("LShift")` produces one short Shift press.

**`DoubleTap(ms)`.** The first press only stores an entry with the deadline "press time + `ms`"; nothing runs. If the same key is pressed again strictly before the deadline, the entry is consumed and the action fires on that key-down, on the hook thread ([L197-L219](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L197-L219)). A later press arms afresh, so a third press starts a new cycle. A single press does nothing, and the keystroke is lost.

> [!NOTE]
> Until `16a0461` every armed press started its own sleeper thread with no generation number, so a timer from an earlier press removed or fired the entry of the next one ([KS-11](../audit.md#ks-11), fixed): two quick taps lost the second, and a Hold could fire early on the next press.

<a id="s11-6"></a>
### 11.6 `Tap` and `Hold` (or `DoubleTap`) on the same key

These are two separate bindings. Both gestures are armed independently and resolve independently. The result depends on how long the key is held, `t`:

| Duration `t` | Result |
|---|---|
| `t < Tap` and `t < Hold` | Tap |
| `t ≥ Hold` and `t ≥ Tap` | Hold |
| `Hold ≤ t < Tap` (possible when `Tap > Hold`) | **both**: Hold at the `Hold` mark, Tap on release |
| `Tap ≤ t < Hold` (possible when `Tap < Hold`) | nothing |

With **equal** values, for example `Tap(200)` and `Hold(200)`, exactly one of the two fires: Tap if the key is released before 200 ms, Hold otherwise ([KS-25](../audit.md#ks-25), fixed for equal terms in `16a0461`). The engine decides the key-up and the Hold timer on one clock, so a release near the boundary never fires both or neither ([src/engine/mod.rs:68-74](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L68-L74)). Use equal values for a dual-role key:

```ron
( keys: "F13", action: Media(PlayPause), trigger: Tap(200) ),
( keys: "F13", action: Media(Next),      trigger: Hold(200) ),
```

> [!WARNING]
> With **different** values nothing arbitrates between the two gestures, so the rows "both" and "nothing" above can happen ([KS-25](../audit.md#ks-25), rest open until roadmap [item 2.2](../roadmap.md#22-timers-on-the-main-thread)).

**`Tap` and `DoubleTap` on the same key** are also allowed, but the ambiguity is not resolved in any way. A double press produces three actions: Tap (release 1), DoubleTap (press 2), Tap (release 2). A single press produces Tap ([Tap versus DoubleTap disambiguation](../gaps.md#tap-versus-doubletap-disambiguation)).

<a id="s11-7"></a>
### 11.7 Several deferred bindings of the same type on one press

This happens with one key and overlapping `when` conditions, or with a binding that sets `allow_extra_modifiers: true` next to a more specific one, such as `"F13"` with the flag and `"Shift+F13"`. The gesture tables are keyed by the key code only:

- `Tap` and `Hold`: only the first matching binding in check order ([§16](matching.md#s16)) is armed; the others are silently dropped ([src/engine/gestures.rs:91-106](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L91-L106), [L125-L137](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L125-L137)). This can be used on purpose: with `"Shift+F13"` on `Tap` and `"F13"` on `Tap` with `allow_extra_modifiers: true`, pressing Shift+F13 runs only the first one.
- `DoubleTap`: see the warning below.
- A related problem: a more specific binding does not override less specific deferred ones ([§16.3](matching.md#s16-3), [KS-13](../audit.md#ks-13)).

> [!WARNING]
> **Known issue: several `DoubleTap` bindings.** The second binding takes the first binding's entry, and the first binding's action runs already on the **first** press ([src/engine/gestures.rs:200-219](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L200-L219); inferred from the code).

<a id="s11-8"></a>
### 11.8 Defects

> [!WARNING]
> **Known issue ([KS-08](../audit.md#ks-08)): actions on the hook thread.** `Tap` and `DoubleTap` actions, as well as `chord` and `sequence` completions, run in the hook callback after the decision ([src/hook/keyboard_hook_callback.rs:185-195](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L185-L195)). A slow `Launch` or `OpenUrl` can exceed `LowLevelHooksTimeout`, and Windows then silently removes the hook ([§13.1](actions.md#s13-1); roadmap [item 2.1](../roadmap.md#21-action-executor)).

> [!WARNING]
> **A deferred trigger on a modifier swallows that modifier.** `keys: "LShift", trigger: DoubleTap(250)` ([README.md:340](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/README.md?plain=1#L340), [src/config/default_config.ron:251-258](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/default_config.ron#L251-L258)) toggles CapsLock on a double press of the left Shift, but every left-Shift press is suppressed, so it no longer types capitals. The right Shift is unaffected. Roadmap [item 1.3](../roadmap.md#13-block_default-for-tap-hold-and-doubletap) fixes this.

> [!WARNING]
> **Lone Win or Alt ([KS-12](../audit.md#ks-12)).** For `Win+…` and `Alt+…` with a deferred trigger, only the trigger key is suppressed, so releasing Win or Alt can open the Start menu or the menu bar ([§13.1](actions.md#s13-1); roadmap [item 1.2](../roadmap.md#12-mask-key-and-auto-blocker-v2)).

> [!WARNING]
> **Hot reload and armed gestures.** A hot reload does not reset gestures that are already armed ([src/hook/config_watcher.rs:162](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/config_watcher.rs#L162)). They complete with the old actions. `swch off` and `swch on` do drop them ([KS-31](../audit.md#ks-31)).

> [!NOTE]
> Fixed in Phase 0: a gesture no longer arms on an unrelated key pressed while its key is held ([KS-04](../audit.md#ks-04)); a key held past the repeat delay no longer leaks its auto-repeats and sticks ([KS-06](../audit.md#ks-06)); sided names such as `LShift` fire ([KS-07](../audit.md#ks-07)); and a modifier whose key-down was suppressed is no longer erased by the resync, so its auto-repeat no longer looks like a new press ([KS-14](../audit.md#ks-14)).

<a id="s12"></a>
## 12. Conditions (`when`)

<a id="s12-1"></a>
### 12.1 Variants and syntax

```text
when: Always                                    // default, same as omitting the field
when: AppEquals("code.exe")
when: TitleContains("Visual Studio Code")
when: TitleEquals("Калькулятор")
when: Not(TitleContains("Incognito"))
when: And([AppEquals("chrome.exe"), Not(TitleContains("Incognito"))])
when: Or([AppEquals("firefox.exe"), AppEquals("chrome.exe")])
```

Every string comparison ignores case with Unicode lowercasing, in every alphabet ([src/data/condition.rs:61-92](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/condition.rs#L61-L92); [KS-28](../audit.md#ks-28), fixed in `4177540`).

| Variant | True when |
|---|---|
| `Always` | always |
| `AppEquals("x.exe")` | the file name of the active window's executable, `.exe` included, equals the string: `"code.exe"` matches `C:\…\Microsoft VS Code\Code.exe` |
| `TitleContains("s")` | the title of the active window contains the substring |
| `TitleEquals("s")` | the title matches in full: `TitleEquals("КАЛЬКУЛЯТОР")` matches a window titled "Калькулятор" |
| `Not(c)` | `c` is false |
| `And([..])` | all are true; evaluation stops at the first false one; an empty list is true |
| `Or([..])` | at least one is true; evaluation stops at the first true one; an empty list is false |

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
### 12.3 `AppEquals`

`AppEquals` reads the process of the foreground window with `QueryFullProcessImageNameW`, which needs only `PROCESS_QUERY_LIMITED_INFORMATION`, and compares the file name after the last `\` ([src/data/condition.rs:139-154](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/condition.rs#L139-L154), [src/data/condition.rs:184-219](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/condition.rs#L184-L219)). The code relies on Windows granting that right for elevated processes too ([src/data/condition.rs:186-189](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/condition.rs#L186-L189)); the audit's probe read the image path of `explorer.exe` and of its own process through such a handle ([KS-03](../audit.md#ks-03)). An image path of 1024 or more UTF-16 units gives no name, so `AppEquals` is then false.

```ron
( keys: "F13", action: PressKey("F5"), when: AppEquals("code.exe") ),
( keys: "F14", action: Media(PlayPause),
  when: Or([AppEquals("chrome.exe"), AppEquals("firefox.exe")]) ),
( keys: "F15", action: DoNothing, when: Not(AppEquals("code.exe")) ),
```

The tests `data::condition::tests::app_equals_matches_the_current_process` and `data::condition::tests::process_image_name_resolves_the_current_process` evaluate it against the test's own process. Whether a binding fires in a focused application on a live daemon is a manual check that has not been run ([Manual checks after Phase 0](../roadmap.md#manual-checks-after-phase-0)).

> [!NOTE]
> Until `4177540` every `AppEquals` was false, so `Not(AppEquals(..))` was true even inside that application ([KS-03](../audit.md#ks-03), fixed). A config that worked around it with `TitleContains` keeps working.

The windows of UWP apps such as Calculator and Settings belong to the `ApplicationFrameHost.exe` process, so `AppEquals` cannot tell them apart: match them by title. A title is not unique either: a browser tab with the text "Visual Studio Code" matches `TitleContains("Visual Studio Code")`. Combine the two with `And([...])` if needed.

<a id="s12-4"></a>
### 12.4 When and how a condition is checked

[src/engine/mod.rs:373-378](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L373-L378), [src/hook/keyboard_hook_callback.rs:100-141](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L100-L141), [src/data/condition.rs:103-178](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/condition.rs#L103-L178):

- Window data is collected only on key-down, and only when a condition other than `Always` is evaluated (judged at the top level: `Not(Always)` or `And([])` already count as a condition). The window is found with `GetForegroundWindow()`: this is the top-level window, not the child control that has focus.
- The data is queried lazily and at most once per keystroke: the executable name and the title are read when first needed and cached until the keystroke has been handled. The title is read with `GetWindowTextW` into a buffer of 512 UTF-16 code units; anything longer than 511 is truncated.
- Each binding's condition is checked **before** its keys are compared. So if the config contains even one `when`, the Win32 queries run on every press of any key, auto-repeat included, on the hook thread (roadmap [item 3.5](../roadmap.md#35-hot-path-budget)).
- The condition is not re-checked on key-up or in timers ([§11.4](#s11-4)). For `sequence`, the condition is checked at every step; if it is false, the step is skipped, and the wait is not reset ([§8.2](kinds.md#s8-2)).

> [!WARNING]
> The auto-blocker created by `block_original_combo: true` does not inherit the `when` of its binding and acts in every window ([§15.4](matching.md#s15-4), [KS-35](../audit.md#ks-35), open until roadmap [item 1.2](../roadmap.md#12-mask-key-and-auto-blocker-v2)). With `when: Not(AppEquals("game.exe"))` on the default config's CapsLock binding, CapsLock reaches the game as CapsLock while the Windows hotkey stays blocked there, so neither switches the layout in the game.
