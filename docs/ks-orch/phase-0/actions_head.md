# Actions and macro steps

`action` says what a binding does when it fires: switch the layout, press a key, post a window message, start a program, open a URL, press a media key, run a macro, or nothing. Facts describe commit `1dfd80a`; how actions behave in Windows is inferred from the code and was not observed on a live daemon.

<a id="s13"></a>
## 13. Actions (`action`)

`action` is a required binding field; without it the loader reports ``Unexpected missing field named `action` in `RawBinding` ``. The value is one of the 10 variants of `RawAction` ([src/config/schema.rs:133-167](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/schema.rs#L133-L167)). The loader turns it into a `BindAction` ([src/config/loader.rs:515-573](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L515-L573)), and `BindAction::execute` runs it ([src/core/windows_actions.rs:118-142](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L118-L142)).

| Variant | Syntax | What it does |
|---|---|---|
| `SwitchLanguage` | `SwitchLanguage` | Posts `WM_INPUTLANGCHANGEREQUEST` with a "forward" switch to the foreground window |
| `SwitchLanguageBackward` | `SwitchLanguageBackward` | The same with a "backward" switch |
| `ToggleCapsLock` | `ToggleCapsLock` | Synthesizes a CapsLock press |
| `PressKey` | `PressKey("Esc")` | Synthesizes one press of one key |
| `PostMessage` | `PostMessage(msg: "WM_CLOSE", wparam: 0, lparam: 0)` | Calls `PostMessageW` on the foreground window |
| `Launch` | `Launch(exe: "code.exe", args: ["D:\\proj"])` | Starts a process |
| `OpenUrl` | `OpenUrl("https://github.com")` | Calls `ShellExecuteW` with the `"open"` verb |
| `Media` | `Media(PlayPause)` | Synthesizes a media key or a volume key |
| `Sequence` | `Sequence([Text("hi"), Key("Enter")])` | A macro; runs on a separate thread ([§14](#s14)) |
| `DoNothing` | `DoNothing` | Does nothing (needed only to suppress a key) |

The code also has `LockWorkstation`, `ShowDesktop` and `TaskManager` ([src/system/system_function.rs:17-24](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/system_function.rs#L17-L24)), but `RawAction` has no variants for them, so they are not available from the config: `action: LockWorkstation` gives ``Unexpected variant named `LockWorkstation` in enum `RawAction`, expected one of `SwitchLanguage`, ... or `DoNothing` instead``. Macro steps do not work outside `Sequence` either: `action: Text("hi")` and `action: Window(Minimize)` give an error of the same form.

**Variant forms** (names are case-sensitive, including the arguments of `Media(...)` and `Window(...)`):

| Form | Variants | Correct | Incorrect |
|---|---|---|---|
| No arguments | `SwitchLanguage`, `SwitchLanguageBackward`, `ToggleCapsLock`, `DoNothing` | `SwitchLanguage` | `SwitchLanguage()` or `DoNothing(1)` → `Expected comma`; `"SwitchLanguage"` → `Expected identifier` |
| One positional argument | `PressKey`, `OpenUrl`, `Media`, `Sequence` | `PressKey("A")` | without parentheses → ``Expected opening `(` ``; `PressKey()` → `Expected string`; `Media()` → `Expected identifier`; `Sequence()` → ``Expected opening `[` ``; `PressKey("A", "B")` → ``Expected closing `)` `` |
| Named fields | `PostMessage`, `Launch` | `Launch(exe: "x", args: [])` | `Launch("x")` → `Expected identifier`; `Launch { exe: "x" }` → ``Expected opening `(` ``; `Launch()` → ``Unexpected missing field named `exe` in `Launch` `` |

Named fields are written inside parentheses, in any order; a trailing comma is allowed, and a repeated field is an error. An unknown field is an error that lists the valid ones ([§5.3](structure.md#s5-3)): `Launch(exe: "notepad.exe", cwd: "C:/temp")` gives ``Unexpected field named `cwd` in `Launch`, expected either `exe` or `args` instead``.

<a id="s13-1"></a>
### 13.1 General execution rules

**Which thread runs the action, and when.** The decision core decides which actions fire; the hook adapter runs them after the decision, outside the engine lock ([src/hook/keyboard_hook_callback.rs:185-195](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L185-L195)):

| How the binding fired | Thread | Moment | Decided at |
|---|---|---|---|
| `keys` + `Immediate` | hook thread | key press, **including every auto-repeat** | [src/engine/mod.rs:404-412](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L404-L412) |
| `trigger: Tap(ms)` | hook thread | key released before `ms` | [src/engine/mod.rs:547-552](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L547-L552) |
| `trigger: Hold(ms)` | separate `hold-timer` thread | `ms` after the press, when the timer thread wakes | [src/hook/keyboard_hook_callback.rs:204-218](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L204-L218) |
| `trigger: DoubleTap(ms)` | hook thread | second press | [src/engine/mod.rs:439-452](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L439-L452) |
| `chord` | hook thread | the last key of the chord is pressed | [src/engine/mod.rs:490-496](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L490-L496) |
| `sequence` | hook thread | last step | [src/engine/mod.rs:468-477](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L468-L477) |

The "hook thread" is the main thread with the message loop that services `WH_KEYBOARD_LL` ([src/core/app.rs:121-192](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/app.rs#L121-L192)). While an action runs, keyboard input stalls system-wide. Only `Sequence` moves its work to a separate thread; all other actions run synchronously on the thread that invoked them. When one key-down fires several actions, they run in order.

Four known issues cut across several actions:

> [!WARNING]
> **Known issue ([KS-08](../audit.md#ks-08)).** `Launch` and `OpenUrl` run synchronously inside the hook callback ([src/core/windows_actions.rs:137-138](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L137-L138)). The exception is `trigger: Hold(ms)`, where the action is called from the `hold-timer` thread. `CreateProcessW` and `ShellExecuteW` can take hundreds of milliseconds. If the callback runs longer than `LowLevelHooksTimeout` (at most 1000 ms on Windows 10 1709 and later), Windows silently removes the hook, while the daemon goes on considering itself running. **Workaround:** `Sequence([Launch(...)])` or `Sequence([OpenUrl(...)])`: macro steps run on a worker thread. Roadmap [item 2.1](../roadmap.md#21-action-executor) moves actions off the hook thread.

> [!WARNING]
> **Known issue ([KS-10](../audit.md#ks-10)).** `Immediate` actions repeat on auto-repeat ([src/engine/mod.rs:404-412](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L404-L412)). While the trigger is held, the action fires at the auto-repeat rate, about 30 times per second after the initial delay. For `PressKey` this is what you want: the remap "holds" like an ordinary key. But `SwitchLanguage` will cycle through the layouts, `ToggleCapsLock` will flicker, `Launch` and `OpenUrl` will open dozens of windows, and `Sequence` will start dozens of parallel macros. Deferred triggers, as well as `chord` and `sequence`, skip auto-repeat, so `trigger: Tap(…)` is convenient for "one-shot" actions. Roadmap [item 1.1](../roadmap.md#11-per-binding-repeat-flag) will keep repeat on by default only for `PressKey`, `VolumeUp` and `VolumeDown` (owner decision).

> [!WARNING]
> **Known issue ([KS-19](../audit.md#ks-19)).** Synthesized keys combine with physically held modifiers ([src/core/windows_actions.rs:163](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L163), [src/data/sequence.rs:202](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/sequence.rs#L202)). The trigger's modifier presses have already reached the OS; only the key that completed the combination is suppressed. Example: with `keys: "Ctrl+J", action: PressKey("Down")` the application receives Ctrl+Down. Workaround: triggers without modifiers (`F13`–`F24`, `CapsLock`, `Pause`), and in a macro a first step `Delay(150…300)` so that the user has time to release the keys.

> [!WARNING]
> **Known issue ([KS-12](../audit.md#ks-12)).** If `Win+X` or `Alt+X` runs an action that injects no keys (`SwitchLanguage`, `SwitchLanguageBackward`, `PostMessage`, `Launch`, `OpenUrl`, `DoNothing`), the OS sees a lone Win or Alt press and, on release, opens the Start menu or activates the menu bar. No mask key is injected: the decision's `mask` field is reserved for roadmap [item 1.2](../roadmap.md#12-mask-key-and-auto-blocker-v2) and never set ([src/engine/mod.rs:255-258](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L255-L258)). `Sequence` produces its output asynchronously and may not get there before the release. `SwitchLanguage*` belong here too: they send the window a message, not a keystroke ([src/system/system_function.rs:51](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/system_function.rs#L51), [src/system/system_function.rs:70](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/system_function.rs#L70)).

Two further rules apply to every action:

- `SendInput` events are tagged in `dwExtraInfo` ([src/core/constants.rs:15-28](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/constants.rs#L15-L28)), and the daemon's own hook passes them through before the decision core sees them ([src/hook/keyboard_hook_callback.rs:166-168](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L166-L168)). The output of one action therefore never triggers another binding: there is no chaining and no looping (`CapsLock → ToggleCapsLock` does not recurse), and the auto-blocker does not block the daemon's own synthesized input. To other programs this is ordinary injected input with the `LLKHF_INJECTED` flag.
- Process rights. The release build requests `requireAdministrator` when it is linked with the MSVC linker ([build.rs:24-43](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/build.rs#L24-L43)): the daemon runs elevated, so `PostMessage` and `SendInput` reach any window. The debug build runs with the rights of whoever started it; UIPI then blocks input to elevated windows: the daemon prints the `PostMessage` error to the console, while `SendInput` is blocked silently.

