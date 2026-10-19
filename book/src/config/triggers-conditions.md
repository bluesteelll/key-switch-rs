# Triggers and conditions

`trigger` decides when a `keys` binding fires: at once (`Immediate`), on a short press (`Tap`), on a long press (`Hold`) or on a double press (`DoubleTap`). `when` restricts a binding to the windows whose application or title matches. Facts describe commit `20af28d`, the tip of `master` after roadmap Phase 1; the run-time behaviour is derived from the code of the decision core and its scenario tests, not observed on a live daemon.

<a id="s11"></a>
## 11. Triggers (`trigger`)

<a id="s11-1"></a>
### 11.1 Values

| Value | RON | Fires |
|---|---|---|
| `Immediate` (default) | `trigger: Immediate`, or the field omitted | on key-down; on an auto-repeat again only with `repeat` ([§11.3](#s11-3)) |
| `Tap(ms)` | `trigger: Tap(200)` | on key-up, if the key is released less than `ms` after it was pressed |
| `Hold(ms)` | `trigger: Hold(300)` | `ms` after the press, if the key was not released before that |
| `DoubleTap(ms)` | `trigger: DoubleTap(250)` | on the second key-down of the same key less than `ms` after the first key-down |

<a id="s11-2"></a>
### 11.2 Restrictions and parse errors

[src/config/loader.rs:463-472](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L463-L472), [src/config/loader.rs:610-634](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L610-L634), [src/config/schema.rs:108-118](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/schema.rs#L108-L118):

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

[src/engine/mod.rs:475-482](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L475-L482). The action fires on key-down on the hook thread, after the decision: a fast action runs there, and `Launch`, `OpenUrl` and `Sequence` are queued for the action executor ([§13.1](actions.md#s13-1)).

- If several `keys` bindings match one press, one wins ([§16.2](matching.md#s16-2)): the one with the most keys, then one with `when` over one without, then the first in the file. An `Immediate` winner runs alone; the losers, whatever their triggers, neither run nor arm.
- The event is suppressed if the winner has `block_default: true`. A losing binding no longer suppresses it ([KS-13](../audit.md#ks-13), fixed on `feat/one-winner`), but a `sequence`, a `chord` or an auto-blocker that takes the key still can ([§10](kinds.md#s10)).
- An auto-blocker never wins: it fires nothing and only suppresses ([§15](matching.md#s15)).
- On an auto-repeat of the key the binding fires again only if it repeats: with `repeat: true`, or by default for `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)` (owner decision, [KS-10](../audit.md#ks-10)). Every other action fires once per press. The repeats are still suppressed when the key-down was, and passed when it passed ([§10](kinds.md#s10)), whether the action fires or not.
- The election runs on every auto-repeat, so the winner takes an auto-repeat even when it does not fire on it, and a lower binding on the same key never fires in its place. With `( keys: "F13", action: SwitchLanguage )` above `( keys: "F13", action: PressKey("Left") )`, holding F13 switches the layout once and sends no Left.
- If a modifier is released while the key is held, a repeat can elect another binding, which then fires as it says, while the repeat itself is still suppressed or passed like the key-down ([§19](known-issues.md#s19) row 73).
- `repeat: true` on a `Tap`, `Hold` or `DoubleTap` binding, a `sequence` or a `chord` is a load error, because those never fire on an auto-repeat: ``binding #0 [F13]: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger``. `repeat: false` is accepted there.
- A press that the daemon reads as an auto-repeat, because the key's previous release went to the lock screen or another desktop, fires only a binding that repeats; the next press works ([KS-29](../audit.md#ks-29), [§19](known-issues.md#s19) row 63).

```ron
( keys: "CapsLock", action: SwitchLanguage ),           // one switch per press, however long CapsLock is held
( keys: "F13", action: Media(Next), repeat: true ),     // skips tracks while F13 is held
```

<a id="s11-4"></a>
### 11.4 Common to `Tap`, `Hold` and `DoubleTap`

[src/engine/mod.rs:483-528](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L483-L528), [src/engine/gestures.rs](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs):

- A gesture is armed on a non-repeat key-down that fires the `keys` binding by the rules of [§7](kinds.md#s7): the key just pressed is one of the binding's trigger keys, all its keys are held and the modifiers are exactly its own. The gesture is bound to that key's code, and that key's key-up resolves it. For `keys: "Ctrl+F17"` with `Tap(150)`, press Ctrl, then F17: the time runs from the F17 press to its release. Pressing F17 first, then Ctrl arms nothing.
- A key pressed while the gesture is pending is not the binding's key: it passes and arms nothing. If the gesture's key passed (`block_default: false`, or a binding made only of modifiers), that key-down cancels the gesture, so that a shortcut such as Ctrl+C is not also a Ctrl `Tap` (owner decision, roadmap [item 2.2](../roadmap.md#22-timers-on-the-main-thread); [§11.8](#s11-8)). A gesture whose key was swallowed is not cancelled. A `Hold` whose term has passed is resolved and fires, and a `Tap` that waits for its DoubleTap window fires at once, ahead of the new key ([§11.6](#s11-6)).
- A deferred binding arms only if it wins the press or belongs to the winning group ([§16.2](matching.md#s16-2)): an `Immediate` binding that wins keeps it from arming, and so does a more specific binding of any trigger. Before `feat/one-winner` deferred bindings armed regardless of an `Immediate` binding that ran on the same press ([KS-13](../audit.md#ks-13), fixed).
- `block_default` decides whether the press, the auto-repeats and the release of the trigger key reach the application ([§10](kinds.md#s10)); the gesture is armed and fires either way. Without the field it is `false` for a combination made only of modifiers (`"LShift"`, `"Ctrl+Shift"`, `"Win"`) and `true` otherwise, so `"CapsLock"` with `Tap(200)` still swallows CapsLock and `"LShift"` with `DoubleTap(250)` lets the left Shift type capitals. Combo keys pressed earlier (for example Ctrl) do reach the application. Before `feat/deferred-block-default` (roadmap [item 1.3](../roadmap.md#13-block_default-for-tap-hold-and-doubletap)) the trigger key was always suppressed, whatever `block_default` said.
- Auto-repeat does not re-arm the gesture, so `Hold` fires once per press.
- `Tap` and `DoubleTap` store a deadline and start no thread; the next event of the key checks it. A `Hold` starts no thread either: it asks the daemon's main thread for a timer, and the main thread wakes for the earliest one on one high-resolution waitable timer, within about 1 ms of the term, not on the 15.6 ms system tick (roadmap [item 2.2](../roadmap.md#22-timers-on-the-main-thread), `src/hook/main_thread.rs`). If Windows refuses that timer, the daemon counts it in `hold_timer_not_started`, which the `[WARN] counters since the last report: ...` line reports, and the Hold is dropped ([KS-33](../audit.md#ks-33), fixed; [§18.6](errors.md#s18-6)).
- The `when` condition is checked only on key-down. `Tap` runs on release even if the window has changed in the meantime; `Hold` runs on its timer whatever window is active, and so does a `Tap` that waits for its DoubleTap window ([§11.6](#s11-6)).

<a id="s11-5"></a>
### 11.5 How `Tap`, `Hold` and `DoubleTap` each resolve

`src/engine/gestures.rs` describes the rules (roadmap item 2.2). A first key-down that a gesture group wins arms one press of the key, with every gesture of the group: the `Tap`, `Hold` and `DoubleTap` on the same keys with the same `when` ([§11.7](#s11-7)). The press resolves to exactly one of them ([§11.6](#s11-6)).

**`Tap(ms)`.** The press stores the deadline "press time + `ms`". At the key-up of the same key the action fires if the release came strictly before the deadline. The action fires on the hook thread ([§13.1](actions.md#s13-1)). If the key is released later, nothing happens; with `block_default: true` the keystroke is lost. With a `DoubleTap` in the group the Tap waits for the DoubleTap window ([§11.6](#s11-6)).

**`Hold(ms)`.** Arming gives the press a new number and asks for a timer. The main thread's loop takes the timer once the deadline has passed and asks the engine to fire the press with its token; a token whose press is gone, re-armed or from before a `swch off` fires nothing. A key-up before the deadline cancels the gesture. A key-up at or after the deadline does not: the key was held for the whole term, so the timer fires the Hold even if the loop takes it a few milliseconds after the release, and the press fires nothing else. `Hold` is the only trigger whose action fires outside a key event, in the main loop between two events, on the same thread as every other trigger: a fast action runs there, and `Launch`, `OpenUrl` and `Sequence` go to the action executor ([§13.1](actions.md#s13-1)). `Hold` is no longer the only safe trigger for `Launch` and `OpenUrl` ([KS-08](../audit.md#ks-08), fixed on `feat/action-executor`). It fires once. It does not hold a modifier down: `Hold` with `PressKey("LShift")` produces one short Shift press.

**`DoubleTap(ms)`.** The first press stores the window's end, "press time + `ms`"; nothing runs. If the key goes up before the window's end, the window opens, and if the same key is pressed again strictly before it, won by the same group (the same keys and `when`), the action fires on that key-down, on the hook thread ([§13.1](actions.md#s13-1)), and that press is spent: it arms nothing. A press of the key that another group wins, such as `Shift+CapsLock` after a `CapsLock` press, ends the window instead (known issue 74, fixed). A later press arms afresh, so a third press starts a new cycle. A single press does nothing; with `block_default: true` the keystroke is lost.

> [!NOTE]
> Until `16a0461` every armed press started its own sleeper thread with no generation number, so a timer from an earlier press removed or fired the entry of the next one ([KS-11](../audit.md#ks-11), fixed): two quick taps lost the second, and a Hold could fire early on the next press.

<a id="s11-6"></a>
### 11.6 `Tap` and `Hold` (or `DoubleTap`) on the same key

These are two separate bindings. With the same keys and the same `when` they win the press as one group ([§16.2](matching.md#s16-2)), and the press is one gesture, of which exactly one fires, whatever the terms ([KS-25](../audit.md#ks-25), fixed on `feat/main-thread-timers`). With different `when` conditions they compete like any two bindings, and where both conditions hold only the winner arms ([§19](known-issues.md#s19) row 70). The result for a group depends on how long the key is held, `t`:

| Duration `t` | Result |
|---|---|
| `t < Tap` and `t < Hold` | Tap |
| `t ≥ Hold` | Hold only: the Hold resolves the press at the `Hold` mark, and its release fires no Tap |
| `Tap ≤ t < Hold` (possible when `Tap < Hold`) | nothing: too long for the Tap, too short for the Hold |

With **equal** values, for example `Tap(200)` and `Hold(200)`, Tap fires if the key is released before 200 ms, Hold otherwise. The engine decides the key-up and the Hold timer on one clock, so a release near the boundary never fires both or neither, also when the timer runs late. Equal values leave no gap, so use them for a dual-role key:

```ron
( keys: "F13", action: Media(PlayPause), trigger: Tap(200) ),
( keys: "F13", action: Media(Next),      trigger: Hold(200) ),
```

**`Tap` and `DoubleTap` on the same key.** The Tap waits until the DoubleTap window closes, so that a double press fires only the DoubleTap ([open question 5](../roadmap.md#open-questions-for-the-owner), decided in roadmap item 2.2). A single quick press fires the Tap when the window closes, the DoubleTap term after the press, or at once when another key goes down first, ahead of that key; a press that another binding wins, or a press of the key after the window, fires it too, first. A double press within the window fires only the DoubleTap. A press held past the window, or a Tap released after the window closed, fires the Tap at the release, as without a DoubleTap. A press that its Hold resolved opens no window.

```ron
( keys: "F9", action: PressKey("A"), trigger: Tap(200) ),
( keys: "F9", action: PressKey("B"), trigger: DoubleTap(300) ),
```

Tap F9: `a` about 0.3 s after the press. Double-tap F9: only `b`. Tap F9 and type `x` at once: `ax`, in that order.

> [!NOTE]
> The waiting Tap costs latency, up to the DoubleTap term after the press, and only for a group that has both a `Tap` and a `DoubleTap`. It fires into the window that has the focus when it fires, without checking its `when` again, and with the modifiers held at that moment ([§19](known-issues.md#s19) row 75).

<a id="s11-7"></a>
### 11.7 Several deferred bindings of the same type on one press

Since `feat/one-winner` this happens only within one group: bindings on the same keys with the same `when` (written the same way), which win the press together ([§16.2](matching.md#s16-2)). Bindings with other keys or another `when` compete, and only the winner arms: with `"Shift+F13"` on `Tap` and `"F13"` on `Tap` with `allow_extra_modifiers: true`, pressing Shift+F13 runs only the first one, because it has more keys.

- Of several bindings with the same trigger in a group, only the first in the file acts; the others are skipped entirely: they arm nothing and do not ask for suppression ([src/engine/winner.rs:117-140](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/winner.rs#L117-L140)). The loader warns about such a binding, whatever its term ([§18.6](errors.md#s18-6)).
- A `Tap`, a `Hold` and a `DoubleTap` of one group arm together, as one press, and exactly one of them fires ([§11.6](#s11-6)).

> [!NOTE]
> Fixed on `feat/one-winner`: of two `DoubleTap` bindings on the same keys and `when`, only the first acts, and it fires on the second press. The second binding used to take the first binding's entry, so the first binding's action ran already on the **first** press ([src/engine/gestures.rs:200-219](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L200-L219)). Fixed on `feat/main-thread-timers`: a DoubleTap window belongs to its group, so a press of the key that another group wins no longer completes it ([§19](known-issues.md#s19) row 74).

<a id="s11-8"></a>
### 11.8 Defects

> [!NOTE]
> **Slow actions left the hook thread ([KS-08](../audit.md#ks-08), roadmap [item 2.1](../roadmap.md#21-action-executor), fixed on `feat/action-executor`).** `Tap` and `DoubleTap` actions, as well as `chord` and `sequence` completions, are still carried out in the hook callback after the decision, but only the fast actions run there; a `Launch`, `OpenUrl` or `Sequence` is queued for the action executor, so a slow one can no longer exceed `LowLevelHooksTimeout` and get the hook removed ([§13.1](actions.md#s13-1)).

> [!NOTE]
> **A deferred trigger on a modifier no longer swallows that modifier** (roadmap [item 1.3](../roadmap.md#13-block_default-for-tap-hold-and-doubletap), fixed on `feat/deferred-block-default`). `keys: "LShift", trigger: DoubleTap(250)` ([README.md:384](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/README.md?plain=1#L384), [src/config/default_config.ron:308-318](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/default_config.ron#L308-L318)) toggles CapsLock on a double press of the left Shift, and every left-Shift press still reaches the application, so it types capitals. Until then every left-Shift press was suppressed. `block_default: true` keeps the left Shift for the gesture alone.

> [!NOTE]
> **Another key cancels a gesture whose key passes** (owner decision, roadmap [item 2.2](../roadmap.md#22-timers-on-the-main-thread), fixed on `feat/main-thread-timers`; [§19](known-issues.md#s19) row 67). A key-down while a `Tap`, `Hold` or `DoubleTap` whose key passed is pending cancels it, as AutoHotkey does, which matters for a modifier you also type with. With `( keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250) )`, typing `<T>` with the left Shift pressed again quickly for the `T` types two capitals and leaves CapsLock alone. With `( keys: "LCtrl", action: PressKey("Escape"), trigger: Tap(200) )`, a Ctrl+C released within 200 ms sends no Escape; a lone quick Ctrl tap does. With `( keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000) )`, holding Ctrl+Shift for more than a second to select words with Right leaves the layout alone. Only a key-down cancels: the keyboard hook does not see the mouse, so a quick Ctrl+click still sends Escape. A `Hold` whose term has passed fires whatever comes after it.

> [!WARNING]
> **A deferred binding on the keys of the Windows hotkey lets Windows see the hotkey.** A `keys` binding on exactly the keys of the Windows language hotkey takes the hotkey over from the auto-blocker ([§15.3](matching.md#s15-3)), also when it sets `block_original_combo: true` itself, and its own `block_default` decides. `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200) )` is made only of modifiers, so Windows sees Alt+Shift and switches the layout forward while the binding switches it back. Write `block_default: true` on it.

> [!WARNING]
> **Lone Win or Alt ([KS-12](../audit.md#ks-12)).** For `Win+…` and `Alt+…` with a deferred trigger only the trigger key is suppressed, but since `feat/mask-key` it sends the menu mask key, so releasing Win or Alt no longer opens the Start menu or the menu bar ([§13.1](actions.md#s13-1)). A `Tap`, `Hold` or `DoubleTap` on `Win` or `Alt` alone still does: it lets the press reach Windows by default since `feat/deferred-block-default`, nothing is swallowed, so nothing is masked, and a tap also opens Start or activates the menu bar. With `block_default: true` Windows never sees that key while the binding matches it, so the key's own shortcuts, such as Win+E, stop working.

> [!WARNING]
> **Hot reload and armed gestures.** A hot reload does not reset gestures that are already armed ([src/hook/config_watcher.rs:162](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L162)). They complete with the old actions. `swch off` and `swch on` do drop them ([KS-31](../audit.md#ks-31)).

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

Every string comparison ignores case with Unicode lowercasing, in every alphabet ([src/data/condition.rs:65-96](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L65-L96); [KS-28](../audit.md#ks-28), fixed in `4177540`).

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

`AppEquals` reads the process of the foreground window with `QueryFullProcessImageNameW`, which needs only `PROCESS_QUERY_LIMITED_INFORMATION`, and compares the file name after the last `\` ([src/data/condition.rs:143-158](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L143-L158), [src/data/condition.rs:188-223](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L188-L223)). The code relies on Windows granting that right for elevated processes too ([src/data/condition.rs:190-193](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L190-L193)); the audit's probe read the image path of `explorer.exe` and of its own process through such a handle ([KS-03](../audit.md#ks-03)). An image path of 1024 or more UTF-16 units gives no name, so `AppEquals` is then false.

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

[src/engine/winner.rs:73-78](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/winner.rs#L73-L78), [src/engine/mod.rs:531-560](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L531-L560), [src/hook/keyboard_hook_callback.rs:114-155](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L114-L155), [src/data/condition.rs:107-182](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L107-L182):

- Window data is collected only on key-down, and only when a condition other than `Always` is evaluated (judged at the top level: `Not(Always)` or `And([])` already count as a condition). The window is found with `GetForegroundWindow()`: this is the top-level window, not the child control that has focus.
- The data is queried lazily and at most once per keystroke: the executable name and the title are read when first needed and cached until the keystroke has been handled. The title is read with `GetWindowTextW` into a buffer of 512 UTF-16 code units; anything longer than 511 is truncated.
- A `keys` binding's condition is checked only once its keys match the press ([§16.2](matching.md#s16-2)), and an auto-blocker's only once its hotkey matches (since `feat/mask-key`), so their conditions cost no Win32 query on other keys. A `sequence` or `chord` checks its condition before its keys, on every key-down that is not an auto-repeat, so with a `when` on one of them the Win32 queries run on almost every key press, on the hook thread (roadmap [item 3.5](../roadmap.md#35-hot-path-budget)).
- The condition is not re-checked on key-up or in timers ([§11.4](#s11-4)). For `sequence`, the condition is checked at every step; if it is false, the step is skipped, and the wait is not reset ([§8.2](kinds.md#s8-2)).

> [!NOTE]
> The auto-blocker created by `block_original_combo` acts only where the `when` of a binding that asked for it holds ([§15.3](matching.md#s15-3); [KS-35](../audit.md#ks-35), fixed on `feat/mask-key`); it used to act in every window. With `when: Not(AppEquals("game.exe"))` on the default config's CapsLock binding, CapsLock reaches the game as CapsLock and the Windows hotkey switches the layout there.
