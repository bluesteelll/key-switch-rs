# Actions and macro steps

`action` says what a binding does when it fires: switch the layout, press a key, post a window message, start a program, open a URL, press a media key, run a macro, or nothing. Facts describe commit `f4c1fb6`, the merge of roadmap item 2.5, except where a language request goes ([§13.2](#s13-2), [§13.4](#s13-4)): that describes, and links to, `6d67816`, the dialog-box fix (owner report 2026-10-30) and the last change of the program's behaviour on `master`; the `Text` action ([§13.10](#s13-10)) describes branch `feat/text-action`, and the `Paste` action and step ([§13.11](#s13-11), [§14.5](#s14-5)) describe branch `feat/clipboard-paste`. How actions behave in Windows is inferred from the code and was not observed on a live daemon.

<a id="s13"></a>
## 13. Actions (`action`)

`action` is a required binding field; without it the loader reports ``Unexpected missing field named `action` in `RawBinding` ``. The value is one of the 12 variants of `RawAction` (`src/config/schema.rs`). The loader turns it into a `BindAction` ([src/config/loader.rs:667-725](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/config/loader.rs#L667-L725)), and the hook adapter runs it where it fires or hands it to the action executor ([§13.1](#s13-1)).

| Variant | Syntax | What it does |
|---|---|---|
| `SwitchLanguage` | `SwitchLanguage` | Posts `WM_INPUTLANGCHANGEREQUEST` with a "forward" switch to the foreground window, or in a dialog box to its focused control |
| `SwitchLanguageBackward` | `SwitchLanguageBackward` | The same with a "backward" switch |
| `ToggleCapsLock` | `ToggleCapsLock` | Synthesizes a CapsLock press |
| `PressKey` | `PressKey("Esc")` | Synthesizes one press of one key |
| `PostMessage` | `PostMessage(msg: "WM_CLOSE", wparam: 0, lparam: 0)` | Calls `PostMessageW` on the foreground window (`WM_INPUTLANGCHANGEREQUEST` in a dialog box: on its focused control) |
| `Launch` | `Launch(exe: "code.exe", args: ["D:\\proj"])` | Starts a process |
| `OpenUrl` | `OpenUrl("https://github.com")` | Calls `ShellExecuteW` with the `"open"` verb |
| `Text` | `Text("Kind regards,")` | Types the text at once, as the macro step `Text` does; runs on the action executor ([§13.10](#s13-10)) |
| `Paste` | `Paste("Kind regards,")` | Puts the text on the clipboard, sends `Ctrl+V` and puts the previous clipboard content back; runs on the action executor ([§13.11](#s13-11)) |
| `Media` | `Media(PlayPause)` | Synthesizes a media key or a volume key |
| `Sequence` | `Sequence([Text("hi"), Key("Enter")])` | A macro; runs on the action executor ([§14](#s14)) |
| `DoNothing` | `DoNothing` | Does nothing (needed only to suppress a key) |

The code also has `LockWorkstation`, `ShowDesktop` and `TaskManager` ([src/system/system_function.rs:18-25](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/system/system_function.rs#L18-L25)), but `RawAction` has no variants for them, so they are not available from the config: `action: LockWorkstation` gives ``Unexpected variant named `LockWorkstation` in enum `RawAction`, expected one of `SwitchLanguage`, ... or `DoNothing` instead``. Other macro steps do not work outside `Sequence`: `action: Window(Minimize)` and `action: Key("A")` give an error of the same form. `Text` and `Paste` are both actions and steps.

**Variant forms** (names are case-sensitive, including the arguments of `Media(...)` and `Window(...)`):

| Form | Variants | Correct | Incorrect |
|---|---|---|---|
| No arguments | `SwitchLanguage`, `SwitchLanguageBackward`, `ToggleCapsLock`, `DoNothing` | `SwitchLanguage` | `SwitchLanguage()` or `DoNothing(1)` → `Expected comma`; `"SwitchLanguage"` → `Expected identifier` |
| One positional argument | `PressKey`, `OpenUrl`, `Text`, `Media`, `Sequence` | `PressKey("A")` | without parentheses → ``Expected opening `(` ``; `PressKey()` → `Expected string`; `Media()` → `Expected identifier`; `Sequence()` → ``Expected opening `[` ``; `PressKey("A", "B")` → ``Expected closing `)` `` |
| A string or named fields | `Paste` | `Paste("x")`, `Paste(text: "x", keys: "Shift+Insert")` | `Paste()` → ``Expected a string, or (text: "...", keys: "...", restore_delay: ...) but found a unit value instead``; `Paste(text: "x", delay: 5)` → ``Unexpected field named `delay` in `Paste`, expected one of `text`, `keys`, or `restore_delay` instead`` |
| Named fields | `PostMessage`, `Launch` | `Launch(exe: "x", args: [])` | `Launch("x")` → `Expected identifier`; `Launch { exe: "x" }` → ``Expected opening `(` ``; `Launch()` → ``Unexpected missing field named `exe` in `Launch` `` |

Named fields are written inside parentheses, in any order; a trailing comma is allowed, and a repeated field is an error. An unknown field is an error that lists the valid ones ([§5.3](structure.md#s5-3)): `Launch(exe: "notepad.exe", cwd: "C:/temp")` gives ``Unexpected field named `cwd` in `Launch`, expected either `exe` or `args` instead``.

<a id="s13-1"></a>
### 13.1 General execution rules

**Which thread runs the action, and when.** The decision core decides which actions fire; the hook adapter carries them out after the decision, outside the borrow of the main thread's state, on the main thread, which fired them (`carry_out` and `dispatch` in `src/hook/keyboard_hook_callback.rs`):

| How the binding fired | Thread that fires | Moment | Decided at |
|---|---|---|---|
| `keys` + `Immediate` | hook thread | key press; on auto-repeat only with `repeat` | [src/engine/mod.rs:844-851](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/engine/mod.rs#L844-L851) |
| `trigger: Tap(ms)` | hook thread; in its main loop when the Tap waits for a DoubleTap window that closes | key released before `ms`; with a `DoubleTap` on the same keys, when the DoubleTap window closes or another key goes down ([§11.6](triggers-conditions.md#s11-6)) | [src/engine/mod.rs:961-979](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/engine/mod.rs#L961-L979) |
| `trigger: Hold(ms)` | hook thread, in its main loop, between key events | `ms` after the press, within about 1 ms (the main thread's high-resolution gesture timer, roadmap item 2.2) | `src/hook/main_thread.rs` (`take_due_timer`) |
| `trigger: DoubleTap(ms)` | hook thread | second press | [src/engine/gestures.rs:179-199](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/engine/gestures.rs#L179-L199) |
| `chord` | hook thread | the last key of the chord is pressed | [src/engine/mod.rs:908-914](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/engine/mod.rs#L908-L914) |
| `sequence` | hook thread | last step | [src/engine/mod.rs:879-888](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/engine/mod.rs#L879-L888) |

The "hook thread" is the main thread with the message loop that services `WH_KEYBOARD_LL` and the gesture timers (`src/hook/main_thread.rs`, roadmap item 2.2); no action runs on a thread of its own. While it runs an action, keyboard input stalls system-wide, a `Hold` action included. Since roadmap [item 2.1](../roadmap.md#21-action-executor) (fixed on `feat/action-executor`, [KS-08](../audit.md#ks-08)) the thread that fires runs only the fast actions itself, before the hook returns: `PressKey`, `Media`, `ToggleCapsLock`, `SwitchLanguage`, `SwitchLanguageBackward`, `PostMessage` and `DoNothing`. Each of them only posts a message or sends one key press, and running it there keeps its output in order with the keys typed after it. `Launch`, `OpenUrl`, `Text`, `Paste` and `Sequence` are queued for the `action-executor` thread, and the hook returns at once. When one key-down fires several actions, the fast ones run in order before the hook returns, and the others run in order on the executor after it.

**The action executor** (`src/core/executor/`) is one thread with its own message loop, and it initialises COM as a single-threaded apartment, as `ShellExecuteW` asks. Up to 32 actions wait for it; a further one is dropped and counted in `executor_jobs_dropped` ([§18.6](errors.md#s18-6)). Macros run side by side on it ([§14.1](#s14-1)): a `Delay` holds up no other action. A `Launch` or `OpenUrl`, as an action or a macro step, holds up the other queued actions and macros while it runs inside Windows (a cold browser, a dialog that `ShellExecuteW` shows). `swch off`, `swch on` when remapping was off, a reload (a saved `config.ron` that loads) and `swch exit` drop the actions still queued and stop the running macros; a step already running finishes first. A save that does not load keeps them, as it keeps the bindings. If the executor stops on its own, it logs one `[ERROR]` line, and every later `Launch`, `OpenUrl`, `Text`, `Paste` and `Sequence` does nothing and counts in `executor_unavailable` until the daemon restarts. A `Paste` holds up the jobs fired after it while it waits ([§13.11](#s13-11)).

> [!NOTE]
> **Auto-repeat ([KS-10](../audit.md#ks-10), fixed in `feat/repeat-flag`).** While the trigger of a `keys` + `Immediate` binding is held, the keyboard repeats its key-down, about 30 times per second after the initial delay. By default only `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)` fire again on each repeat (owner decision); every other action fires once per press, so `trigger: Tap(…)` is no longer needed to make an action one-shot. `repeat: true` on a binding makes it fire on every repeat, and `repeat: false` once per press ([§11.3](triggers-conditions.md#s11-3)). Each repeat of a `PressKey` sends a new press and release: for a toggle key (`CapsLock`, `NumLock`, `ScrollLock`) or a modifier write `repeat: false`, or a held trigger toggles the key at the repeat rate or taps the modifier ([§13.5](#s13-5)).

One known issue and two fixes cut across several actions:

> [!NOTE]
> **Fixed on `feat/action-executor` ([KS-08](../audit.md#ks-08), roadmap [item 2.1](../roadmap.md#21-action-executor)).** `Launch` and `OpenUrl` no longer run inside the hook callback. `CreateProcessW` and `ShellExecuteW` can take hundreds of milliseconds, and a callback that runs longer than `LowLevelHooksTimeout` (at most 1000 ms on Windows 10 1709 and later) gets the hook silently removed; they now run on the action executor, and the hook returns at once. `Sequence([Launch(...)])` and `Sequence([OpenUrl(...)])` are no longer needed as a workaround. The fast actions still run in the callback, by design; noticing a hook that Windows removed is roadmap [item 2.4](../roadmap.md#24-hook-watchdog).

> [!WARNING]
> **Known issue ([KS-19](../audit.md#ks-19)).** Synthesized keys combine with physically held modifiers ([src/core/windows_actions.rs:257](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L257), [src/data/sequence.rs:179](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/sequence.rs#L179)). The trigger's modifier presses have already reached the OS; only the key that completed the combination is suppressed. Example: with `keys: "Ctrl+J", action: PressKey("Down")` the application receives Ctrl+Down. Workaround: triggers without modifiers (`F13`–`F24`, `CapsLock`, `Pause`), and in a macro a first step `Delay(150…300)` so that the user has time to release the keys. A `Paste` waits until the modifiers are released instead ([§13.11](#s13-11)).

> [!NOTE]
> **The menu mask key** (fixed on `feat/mask-key`, [KS-12](../audit.md#ks-12)). When the hook swallows a key event (a key-down, an auto-repeat or a key-up) while Windows has a Win or Alt key down whose key-down it saw, it first sends virtual-key code 0xE8 down and up, tagged as the daemon's own input ([src/engine/mod.rs:1011-1019](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/engine/mod.rs#L1011-L1019), [src/hook/keyboard_hook_callback.rs:298-378](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/hook/keyboard_hook_callback.rs#L298-L378)). Windows then sees a key between the Win or Alt press and its release, so `Win+X` or `Alt+X` with any action, `SwitchLanguage*`, `PostMessage`, `Launch`, `OpenUrl` and `DoNothing` included, no longer opens the Start menu or activates the menu bar on release. This is the technique of AutoHotkey's `#MenuMaskKey`. 0xE8 is unassigned by Microsoft and types nothing, but applications, games, key-logging tools and other keyboard hooks see it as injected input. An Alt or Win whose own key-down a binding swallowed is not masked for: Windows never saw it. A `Tap`, `Hold` or `DoubleTap` on `Win` or `Alt` alone that lets the key through is not covered: nothing is swallowed there ([§11.8](triggers-conditions.md#s11-8)).

Two further rules apply to every action:

- `SendInput` events are tagged in `dwExtraInfo` ([src/core/constants.rs:15-28](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/constants.rs#L15-L28)), and the daemon's own hook passes them through without deciding them; for a modifier among them it only updates the decision core's flag of that modifier ([src/hook/keyboard_hook_callback.rs:246-249](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/hook/keyboard_hook_callback.rs#L246-L249)). The output of one action therefore never triggers another binding: there is no chaining and no looping (`CapsLock → ToggleCapsLock` does not recurse), and the auto-blocker does not block the daemon's own synthesized input. To other programs this is ordinary injected input with the `LLKHF_INJECTED` flag.
- Process rights. The release build requests `requireAdministrator` when it is linked with the MSVC linker ([build.rs:24-43](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/build.rs#L24-L43)): the daemon runs elevated, so `PostMessage` and `SendInput` reach any window. The debug build runs with the rights of whoever started it; UIPI then blocks input to elevated windows: a refused `PostMessage` is counted in `post_message_failed` ([§18.6](errors.md#s18-6)), while `SendInput` is blocked silently.

<a id="s13-2"></a>
### 13.2 `SwitchLanguage` / `SwitchLanguageBackward`

```ron
( keys: "CapsLock",       action: SwitchLanguage ),
( keys: "Shift+CapsLock", action: SwitchLanguageBackward ),
```

No arguments. Code: [src/system/system_function.rs:44-64](https://github.com/bluesteelll/key-switch-rs/blob/6d67816/src/system/system_function.rs#L44-L64), [src/system/system_function.rs:145-197](https://github.com/bluesteelll/key-switch-rs/blob/6d67816/src/system/system_function.rs#L145-L197).

- The window is obtained with `GetForegroundWindow()`. If there is no window, nothing happens.
- If that window is a dialog box (the system dialog class `#32770`: Save As, Open, a message box, the Run dialog), the request goes to the window that has the keyboard focus in the dialog's thread (`GetGUIThreadInfo`), such as the File name box, the file list or the address bar; if there is none, or it cannot be read, it goes to the dialog. Every other window gets the request itself (`src/system/system_function.rs`, `src/platform/focus.rs`).
- Call: `PostMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST (0x0050), wParam, lParam = 0)`, where `wParam = INPUTLANGCHANGE_FORWARD (0x0002)` for `SwitchLanguage` and `INPUTLANGCHANGE_BACKWARD (0x0004)` for `SwitchLanguageBackward`.
- Posting is asynchronous. The action runs on the hook thread, so a posting error is only counted in `post_message_failed`, which the log writer reports in its counters line ([§18.6](errors.md#s18-6), [Log file](../build-and-run.md#log-file)); the error text is not shown (since `feat/action-executor`, roadmap item 2.1).
- Outside a dialog box the recipient is the top-level window, not the focused child control. Whether to switch the layout is up to the window that gets the request; usually `DefWindowProc` accepts it.
- No OS hotkey is pressed, so the action does not depend on what is configured in Windows (Alt+Shift and so on).
- Equivalent: `PostMessage(msg: "WM_INPUTLANGCHANGEREQUEST", wparam: 2)` (forward) or `wparam: 4` (backward).
- `block_original_combo: true` works only with these two actions ([§15](matching.md#s15)).

> [!NOTE]
> **Fixed on `fix/dialog-layout-switch` (owner report 2026-10-30).** A dialog box ignores the request posted to the dialog itself, so before this fix `SwitchLanguage` did nothing while Save As or Open was in front. The check costs one read of the window's class per firing, and two more reads in a dialog. Pickers that are not `#32770` dialogs keep the old target.

<a id="s13-3"></a>
### 13.3 `ToggleCapsLock`

```ron
( keys: "Shift+CapsLock", action: ToggleCapsLock ),
```

No arguments. Call: `SendInput` with a press and a release of `VK_CAPITAL (0x14)`, `wScan = 0` ([src/system/system_function.rs:119-159](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/system/system_function.rs#L119-L159)). The CapsLock state toggles as with a physical press. The hook passes the synthesized press through, so `keys: "CapsLock", action: ToggleCapsLock` does not loop. If the trigger is `Shift+CapsLock` (as in the template), the OS sees Shift+CapsLock, because Shift is physically held ([KS-19](../audit.md#ks-19)). `block_original_combo` has no effect here.

Windows behaviour, not tested in the project: with the "To turn off Caps Lock: Press the SHIFT key" setting, a synthesized CapsLock does not turn the mode off.

<a id="s13-4"></a>
### 13.4 `PostMessage`

```text
PostMessage(msg: <WM_* name or number>, wparam: <u64> = 0, lparam: <i64> = 0)
```

| Field | Type | Default | Allowed values |
|---|---|---|---|
| `msg` | name string or `u32` | required (``Unexpected missing field named `msg` in `PostMessage` ``) | a name from the table below, or a number `0..=4294967295` |
| `wparam` | `u64` | `0` | `0..=18446744073709551615`; a minus sign → `Integer is out of bounds` |
| `lparam` | `i64` | `0` | `-9223372036854775808..=9223372036854775807`; `-0x10` is allowed |

**A string `msg` is always a name** ([src/config/parsing.rs:57-78](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/config/parsing.rs#L57-L78)). All whitespace characters (Unicode ones included) are removed from the string, and it is then uppercased by Unicode rules ([src/config/parsing.rs:61](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/config/parsing.rs#L61)). So `"wm_close"`, `" WM_SysCommand "`, `"W M_CLO SE"` and even `"wm_clo\u{17F}e"` are recognised: the long s (`ſ`) uppercases to `S`. The built-in table:

| Name | Code |
|---|---|
| `WM_DESTROY` | `0x0002` |
| `WM_CLOSE` | `0x0010` |
| `WM_QUIT` | `0x0012` |
| `WM_INPUTLANGCHANGEREQUEST` | `0x0050` |
| `WM_KEYDOWN` | `0x0100` |
| `WM_KEYUP` | `0x0101` |
| `WM_SYSKEYDOWN` | `0x0104` |
| `WM_SYSKEYUP` | `0x0105` |
| `WM_COMMAND` | `0x0111` |
| `WM_SYSCOMMAND` | `0x0112` |
| `WM_HOTKEY` | `0x0312` |

For an unknown name the loader reports `PostMessage: unknown WM_* name "WM_PASTE" (use a numeric code if it is not in the built-in table)`. A number in quotes (`"0x10"`, `"16"`) and an empty string also count as names and give the same error.

**A numeric `msg`.** The field has the type of the untagged enum `MessageRef` ([src/config/schema.rs:224-229](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/config/schema.rs#L224-L229)), which is parsed "by content". Accepted forms are `16`, `+16`, `0x10`, `0b10000`, `0o20`, `0x0001_0000`, numbers with any suffix from `u8` to `i64` (`16u8`, `16u32`, `16i64`), and also `-0` (gives 0). The error ``data did not match any variant of untagged enum MessageRef`` is given by:

- a negative number (`-1`), a fractional one (`16.0`, `1e3`), or one above `u32::MAX` (`0x1_0000_0000`);
- an unquoted name (`msg: WM_CLOSE`);
- a character literal `'W'`, `true`, `Some(16)`, `(16)`, `[16]`.

Because of the parsing "by content", `msg` also accepts a byte string: `msg: b"WM_CLOSE"` gives 16, although in every other string field `b"..."` gives `Expected string`.

The values are passed as is: on x64, `wparam` becomes `usize` and `lparam` becomes `isize` without loss ([src/config/loader.rs:697-701](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/config/loader.rs#L697-L701)). To pass the bit pattern `0xFFFFFFFFFFFFFFFF` in `lparam`, write `-1`: the literal `0xFFFF...` does not fit in `i64` (`Integer is out of bounds`).

**Where the message goes** ([src/core/windows_actions.rs:154-158](https://github.com/bluesteelll/key-switch-rs/blob/6d67816/src/core/windows_actions.rs#L154-L158), [src/system/system_function.rs:145-197](https://github.com/bluesteelll/key-switch-rs/blob/6d67816/src/system/system_function.rs#L145-L197)):

- The recipient is `PostMessageW(GetForegroundWindow(), ...)` at the moment the binding fires: the top-level window of the active application, not the focused child control and not the window under the cursor. The one exception is `WM_INPUTLANGCHANGEREQUEST` (0x0050) in a dialog box: it goes where `SwitchLanguage` sends it ([§13.2](#s13-2)).
- If there is no foreground window, nothing happens.
- Posting is asynchronous, and the result of the processing is unknown. Only an error of the posting itself is noticed, for example when UIPI blocks it in a debug build: it is counted in `post_message_failed` ([§18.6](errors.md#s18-6)), without the error text, because the action runs on the hook thread.

In practice:

- `WM_CLOSE` is the same as the close button; the application may ask about saving.
- `WM_SYSCOMMAND` with an `SC_*` code in `wparam`: `SC_MINIMIZE 0xF020`, `SC_MAXIMIZE 0xF030`, `SC_RESTORE 0xF120`, `SC_CLOSE 0xF060`; `SC_MONITORPOWER 0xF170` with `lparam: 2` turns the monitor off. They are usually handled by `DefWindowProc`.
- `WM_KEYDOWN`, `WM_KEYUP` and `WM_SYSKEY*` sent to a top-level window usually have no effect: keyboard messages are handled by the focused control. Emulate key presses with `PressKey` or `Combo`.
- `WM_QUIT` and `WM_DESTROY` sent from outside are an abnormal way to close a window: the application may exit without asking about saving, or remain in an inconsistent state. Use `WM_CLOSE`.
- The identifiers for `WM_COMMAND` and `WM_HOTKEY` differ from application to application.
- If the desktop is active, the foreground window is a shell window, and `WM_CLOSE` may open the shutdown dialog. This is Windows behaviour, not tested in the project.

<a id="s13-5"></a>
### 13.5 `PressKey`

```ron
( keys: "CapsLock", action: PressKey("Esc") ),
( keys: "F13",      action: PressKey("  page down ") ),   // = PageDown
( keys: "F14",      action: PressKey("RShift") ),         // sends VK_RSHIFT 0xA1
```

- The argument is **one** key name, following the rules of [§6](keys.md#s6): case and all whitespace characters are ignored. An empty string is an error: `PressKey: unknown key ""`.
- Combinations are not accepted: `PressKey("Ctrl+C")` → `PressKey: unknown key "Ctrl+C"`. A combination is sent like this: `Sequence([Combo("Ctrl+C")])`.
- Code: [src/core/windows_actions.rs:257-291](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L257-L291). One `SendInput` call with two events: the press and the release of the VK; `wScan = 0`, and neither `KEYEVENTF_EXTENDEDKEY` nor `KEYEVENTF_SCANCODE` is set ([§6.8](keys.md#s6-8), [KS-36](../audit.md#ks-36)). If some of the events are lost, the press is counted in `key_press_short`, which the `[WARN] counters since the last report: ...` line reports; no line names the key.
- It is a **tap**, not a hold. With `Immediate`, a held trigger produces a series of taps at the auto-repeat rate, unless the binding sets `repeat: false`. `PressKey("LShift")` is one short Shift press, not a held Shift, and a held trigger of `PressKey("CapsLock")` toggles CapsLock on every repeat: write `repeat: false` for a toggle key or a modifier.
- Output goes by VK, so the character depends on the current layout and on the held modifiers. `PressKey("Q")` in the Russian layout types "й". For layout-independent text use `Text("...")` ([§13.10](#s13-10)).
- Names with a side (`LShift`, `RAlt`, `RWin` …) are allowed here and send their own VK.

<a id="s13-6"></a>
### 13.6 `Launch`

```text
Launch(exe: "<name or path>", args: ["arg1", "arg2"] = [])
```

| Field | Type | Default |
|---|---|---|
| `exe` | string | required (``Unexpected missing field named `exe` in `Launch` ``) |
| `args` | list of strings | `[]`. A string instead of a list → ``Expected opening `[` ``; a number in the list → `Expected string` |

Code: `std::process::Command::new(exe).args(args).spawn()` ([src/core/windows_actions.rs:312-326](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L312-L326)). The child process handle is dropped at once; nobody waits for the process to exit.

- **No shell.** `%VAR%` is not expanded, and there are no pipes or redirections. Each element of `args` is a separate argument; Rust quotes it itself when it contains spaces. Do not put several arguments into one string.
- **How `exe` is found** (Rust std rules for Windows):
  - if `exe` contains a path separator, it is a path; a relative path is resolved against the daemon's current directory; if `exe` does not end in `.exe`, the variant with `.exe` appended is tried first;
  - a bare name is searched for in this order: the directory of `key-switch-rs.exe`, then `System32`, then the Windows directory, then the daemon's `PATH` as it was when the daemon started. The current directory is not searched. If the name contains no dot, `.exe` is appended;
  - the App Paths registry key is **not** used. Start programs known only from there (often `chrome.exe`) by full path or through `OpenUrl("chrome.exe")`.
- std starts `.bat` and `.cmd` files through `cmd.exe`. Arguments that cannot be escaped safely for cmd (for example, ones containing a line break) are rejected by std with the error `batch file arguments are invalid`.
- The child process inherits the daemon's environment and current directory ([KS-34](../audit.md#ks-34)). There is no `cwd` field: `cwd:` is a load error. Roadmap [item 3.3](../roadmap.md#33-launch-and-openurl-without-elevation) plans `cwd:` and `elevated:`.
- Errors are logged to the daemon console and to `daemon.log`: `[ERROR] Launch "...": <text>`. The text is `program not found` when the program is not found, `program path has no file name` for `exe: ""`, and otherwise the OS message. The loader does not check that the file exists, so `Launch(exe: "")` loads.

> [!WARNING]
> **Known issue ([KS-09](../audit.md#ks-09)).** Processes are started elevated ([src/core/windows_actions.rs:318](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L318)). The release daemon runs with `requireAdministrator` ([build.rs:27-36](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/build.rs#L27-L36)), so every program it launches gets administrator rights without a UAC prompt. For example, dragging files into such a program from a normal Explorer window does not work. There is no de-elevation.

> [!WARNING]
> **Known issue ([KS-09](../audit.md#ks-09), [KS-34](../audit.md#ks-34)).** Console programs attach to the daemon's console ([src/core/windows_actions.rs:318](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L318); the doc comment at [src/core/windows_actions.rs:35-38](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L35-L38) says so since [`13ce323`](https://github.com/bluesteelll/key-switch-rs/commit/13ce323)). The daemon is a console application, and `spawn` is called without creation flags. So `cmd`, `powershell`, `python` and CLI utilities do not get a window of their own: they write to the daemon's console, share its input, and exit on Ctrl+C or when its console is closed. GUI programs are not affected. Workaround: `Launch(exe: "wt.exe", args: [...])`, or `Launch(exe: "cmd.exe", args: ["/c", "start", "", "powershell.exe"])` (`start` opens a new console window), or `OpenUrl("powershell.exe")` (a new console, but without arguments).

> [!NOTE]
> **Fixed on `feat/action-executor` ([KS-08](../audit.md#ks-08)).** It runs on the action executor, not on the hook thread ([§13.1](#s13-1)); `Sequence([Launch(...)])` and `trigger: Hold(…)` are no longer needed for that.

Roadmap [item 3.3](../roadmap.md#33-launch-and-openurl-without-elevation) starts programs without elevation.

<a id="s13-7"></a>
### 13.7 `OpenUrl`

```ron
( keys: "F13", action: OpenUrl("https://github.com") ),
( keys: "F14", action: OpenUrl("mailto:me@example.com") ),
( keys: "F15", action: OpenUrl("ms-settings:display") ),
( keys: "F16", action: OpenUrl(r"D:\key-switch-rs") ),
```

The argument is one string, and there are no other parameters: `OpenUrl()` → `Expected string`, `OpenUrl("a", "b")` → ``Expected closing `)` ``. Call: `ShellExecuteW(NULL, "open", <string>, NULL, NULL, SW_SHOWNORMAL)` ([src/core/windows_actions.rs:328-351](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L328-L351)).

- It accepts anything the shell can "open": URLs of any registered scheme (`https:`, `mailto:`, `ms-settings:` …), documents (opened with the associated program), folders (opened in Explorer), and program names, including those registered in App Paths.
- Command-line arguments cannot be passed (`lpParameters = NULL`); use `Launch` for that.
- Relative paths are resolved against the daemon's current directory.
- If the call returns ≤ 32, the console shows `[ERROR] OpenUrl "...": ShellExecuteW returned error code N`. The loader does not check the string, so `OpenUrl("")` loads.
- A console program opened through `OpenUrl` gets a console window of its own (the default `ShellExecute` behaviour).

> [!NOTE]
> **Fixed on `feat/action-executor` ([KS-08](../audit.md#ks-08)).** The call is still synchronous, but it runs on the action executor, which initialises COM as a single-threaded apartment, as `ShellExecuteW` asks; the hook returns at once ([§13.1](#s13-1)). While the call runs (a cold browser, a dialog), the other queued actions and macros wait.

> [!WARNING]
> **Known issue ([KS-09](../audit.md#ks-09)).** The new handler process inherits the daemon's elevated rights, as with `Launch` ([src/core/windows_actions.rs:335](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L335)).

<a id="s13-8"></a>
### 13.8 `Media`

```text
Media(PlayPause | Stop | Next | Previous | VolumeUp | VolumeDown | VolumeMute)
```

The argument is an identifier without quotes, and case matters. `Media("PlayPause")` and `Media()` → `Expected identifier`, `Media(Play)` and `Media(playpause)` → ``Unexpected variant named `Play` in enum `MediaKeyRef` ``, `Media` without parentheses → ``Expected opening `(` ``. How the variants map to VK codes and key names is shown in the media and volume table in [§6.4](keys.md#s6-4).

The mechanism is the same as for `PressKey`: one VK tap ([src/core/windows_actions.rs:67-79](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L67-L79), [src/core/windows_actions.rs:162](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L162)). Windows handles these keys globally: volume with the on-screen indicator, and control of the current media session. One tap of `VolumeUp` or `VolumeDown` is one system volume step. With `Immediate`, holding the trigger changes the volume continuously; the other media keys fire once per press unless the binding sets `repeat: true` ([§11.3](triggers-conditions.md#s11-3)). `PressKey("VolumeMute")` is equivalent to `Media(VolumeMute)`, while `PressKey("Stop")`, `PressKey("Next")` and `PressKey("Previous")` are `unknown key` errors.

<a id="s13-9"></a>
### 13.9 `DoNothing`

```ron
( keys: "Insert", action: DoNothing ),   // disable Insert
( keys: "Ctrl+W", action: DoNothing ),   // forbid Ctrl+W
```

It executes nothing ([src/core/windows_actions.rs:163](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/core/windows_actions.rs#L163)). It is used together with `block_default: true` (the default for every `Immediate` binding) to swallow a key or a combination. Auto-blockers use `DoNothing` too ([src/data/binding.rs:156-171](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/binding.rs#L156-L171)), although the engine never fires their action ([§15.1](matching.md#s15-1)).

- Only the key that completed the combination is suppressed. For `Ctrl+W` the application sees just a press and a release of Ctrl.

- Modifiers match exactly ([§7](kinds.md#s7)): `Ctrl+W → DoNothing` does not fire on Ctrl+Shift+W, so that shortcut still reaches the application. Add `allow_extra_modifiers: true` to swallow it as well. Until `1dfd80a` the binding matched any superset and also swallowed every key pressed while Ctrl+W were held ([KS-04](../audit.md#ks-04), fixed).

> [!NOTE]
> With a Win or Alt modifier the swallowed key sends the menu mask key, so the release opens neither the Start menu nor the menu bar ([§13.1](#s13-1); [KS-12](../audit.md#ks-12), fixed on `feat/mask-key`).

- With `block_default: false` the action visibly does nothing, but the binding still wins the press over less specific bindings (those with fewer keys) and, if it has a `when`, over a binding without one on the same keys, whatever their triggers; of two bindings with the same keys and both with or both without a `when`, the one higher in the file wins. The losers neither run, nor arm, nor swallow the key ([src/engine/winner.rs:44-88](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/engine/winner.rs#L44-L88); [§16.2](matching.md#s16-2)). With exact modifiers such a binding competes only with bindings that set `allow_extra_modifiers: true` or have the same keys.

> [!NOTE]
> Fixed on `feat/one-winner` ([KS-13](../audit.md#ks-13)): a losing binding with `block_default: true`, whatever its trigger, no longer swallows the key of a winner with `block_default: false` ([§10](kinds.md#s10)).

<a id="s13-10"></a>
### 13.10 `Text("...")`

```ron
( keys: "ScrollLock", action: Text("Kind regards,") ),
```

Types the text at once, as the macro step `Text` does ([§14.4](#s14-4)), without a `Sequence` around it (owner request 2026-10-30, roadmap [new feature 9](../roadmap.md#new-features-after-stabilisation); branch `feat/text-action`).

- The argument is one RON string, raw strings and escapes included ([§3.9](ron-syntax.md#s3-9)). `Text("")` is an error, `binding #0 [ScrollLock]: Text: empty`, while the macro step `Text("")` loads and does nothing. `Text(123)` and `Text()` → `Expected string`; `Text("a", "b")` → ``Expected closing `)` ``.
- It types exactly as the step (`type_text` in `src/data/sequence.rs`): the whole string goes out in one `SendInput` call, two `KEYEVENTF_UNICODE` events per UTF-16 code unit, so it does not depend on the layout, emoji go out as surrogate pairs, and no key you type lands inside it.
- It runs on the action executor, like `Launch` ([§13.1](#s13-1)). The hook only queues it and, unlike `Sequence([Text("...")])`, reads no window for it, so it is the cheaper of the two on the key path (bench rows R27 and R27l). The text goes to the window that has the keyboard focus when Windows inserts the batch, a fraction of a millisecond after the trigger.
- It fires once per press; `repeat: true` types it again on each auto-repeat, up to 32 waiting. `swch off`, a reload and `swch exit` drop a `Text` that still waits; one that Windows is inserting finishes.
- The startup list shows it as `text (N chars)`, N the number of characters, and no log line contains the text.
- **Order with the keys after it.** The hook only queues the text, so a key that goes down in the same moment reaches the application first. With `Immediate` that takes a key typed within that fraction of a millisecond. A `Text` that another key's key-down fires always comes after that key: a `Tap` that waits for its `DoubleTap` window fires when the next key goes down ([§11.6](triggers-conditions.md#s11-6)), and so does a `Hold` whose term has just passed ([§11.4](triggers-conditions.md#s11-4)). With `( keys: "F13", action: Text("Kind regards,"), trigger: Tap(200) )` next to a `DoubleTap` on F13, a tap of F13 and then Enter within the window sends a chat message first and types the phrase into the next one ([§19](known-issues.md#s19) row 92). Give a `Text` binding no `DoubleTap` on the same keys, or use `Immediate`. The same holds for `Launch`, `OpenUrl` and `Sequence`; a fast action runs before that key.
- `\n`, `\t` and `\r` are characters, not Enter or Tab ([§14.4](#s14-4); [§19](known-issues.md#s19) row 34).
- **A long text.** The batch has two events per UTF-16 code unit. While Windows inserts it, the keys you type and the daemon's due `Hold` timers wait behind it, and the daemon's own hook sees each of its events and passes it; a very long one may make the daemon count `hook_late` ([§18.6](errors.md#s18-6)). Some applications lose characters of a very long batch ([§19](known-issues.md#s19) row 91). Several `Text` steps of a `Sequence` with a `Delay` between them avoid that, but your keys can then land between the parts.

> [!WARNING]
> **Known issue ([KS-19](../audit.md#ks-19)).** The trigger's held modifiers stay down in Windows while the text arrives, as for the macro step: with Alt held an application may take the characters as menu shortcuts, with Ctrl as its own shortcuts. `Text` sends no modifier event, so nothing stays stuck. Use a trigger without modifiers (`ScrollLock`, `Pause`, `F13`–`F24`), or `Sequence([Delay(200), Text("...")])` to give yourself time to release them. Roadmap item 3.1 releases them.

<a id="s13-11"></a>
### 13.11 `Paste("...")`

```ron
( keys: "F15", action: Paste("Kind regards,\nIvan") ),
( keys: "F16", action: Paste(text: "git status", keys: "Shift+Insert", restore_delay: 800) ),
```

Puts the text on the clipboard, sends the paste keystroke and then puts the previous clipboard content back (owner request 2026-10-30, roadmap [new feature 9](../roadmap.md#new-features-after-stabilisation); branch `feat/clipboard-paste`). Where `Text` types the phrase key by key, `Paste` hands it over in one paste, as Ctrl+V does: an editor's auto-indent and auto-close treat it as pasted text, a long phrase loses no characters, and no key you type lands inside it.

| Field | Type | Default | Allowed values |
|---|---|---|---|
| `text` | string | required (``Unexpected missing field named `text` in `Paste` ``) | not empty, no U+0000 |
| `keys` | a combination string ([§6.2](keys.md#s6-2)) | `"Ctrl+V"` | at least one key that is not a modifier |
| `restore_delay` | integer, ms | `500` | `50` to `5000` |

- `Paste("...")` is the text with both defaults. `Paste(text: "...", keys: "...", restore_delay: N)` names the fields, in any order; `text` is required, a trailing comma is allowed, and a repeated or unknown field is an error. `keys` and `restore_delay` take a plain value: `keys: Some("Shift+Insert")` gives `Expected string`.
- Load errors ([§18.5](errors.md#s18-5) rules 20 to 24): `Paste: empty`; `Paste: the text contains the character U+0000, which ends a text on the clipboard`; `Paste: keys "Ctrl+Foo": unknown key "Foo" in combo "Ctrl+Foo"`; `Paste: keys "Ctrl+Shift" has no key besides modifiers`; `Paste: restore_delay: 10000 is out of range: 50 to 5000 ms`. No message quotes the text. The parse errors are in [§18.3](errors.md#s18-3).
- The text is converted once, at load: to UTF-16, every line break (`\n`, `\r` or `\r\n`) as CR-LF, one NUL at the end, the form `CF_UNICODETEXT` asks for. A `\n` is a line break of the pasted text, not an Enter key: what a pasted line break does is up to the application (a chat box may send on it or insert it).
- The startup list shows `paste (N chars, Ctrl+V)`, N the number of characters. No log line, debug output or error message contains the text or the clipboard content.
- It fires once per press; `repeat: true` fires it on each auto-repeat, but at most one `Paste` action waits at a time (below), so a held key pastes about once per restore.

**What happens.** The hook only queues the `Paste` for the action executor, as it queues a `Text` (bench row R28); everything else runs on the executor thread:

1. It reads the foreground window, the target.
2. If the restore of an earlier paste is pending, it waits for it, at most 1 s, and then goes ahead (`[INFO] paste: a paste went ahead after waiting 1 s for the previous one's restore`).
3. It waits until none of the eight Shift, Ctrl, Alt and Win keys is down, checking every 5 ms on its timer; after 3 s the paste is skipped (`[WARN] paste: a modifier key was still held after 3 s; the paste was skipped`). See the warning below.
4. It checks that remapping is still on and that the foreground window is the same; otherwise the paste is skipped.
5. If another program's window owns the clipboard, it sends that window `WM_NULL` and waits at most 100 ms for the answer, so that a hung owner cannot hold up the executor; without an answer the paste is skipped.
6. It opens the clipboard with a message-only window of its own. If another program has the clipboard open, it tries again after 5, 10, 20, 40 and 80 ms, 155 ms in all, and then skips the paste. Up to here nothing was changed.
7. While it has the clipboard open, it saves the previous content (below), empties the clipboard, puts the phrase on it with four markers, and closes it.
8. It checks remapping, the foreground window and the modifiers again: if one of them changed, it puts the previous content back at once and sends no keystroke.
9. It sends the keystroke in one `SendInput` batch, as a `Combo` step sends it (the modifiers down, the key down and up, the modifiers up), tagged as the daemon's own input. If Windows inserted none of it, the previous content goes back at once; if it cut the batch short, the missing key releases are sent, so no key stays down.
10. `restore_delay` after the keystroke it sends the target window `WM_NULL` and waits at most 20 ms. An answer puts the previous content back. No answer, or one that took 5 ms or more, means the window's thread is busy and may not have read the clipboard yet: it asks again every 250 ms, and once the window answers after that, it waits 100 ms more. 5 s after the keystroke the content goes back anyway (`[WARN] paste: the target window did not answer for 5 s; the clipboard was put back anyway`). A window that no longer exists counts as one that answered.
11. The restore opens the clipboard and checks, while it has it open, that its window still owns the clipboard and that no other program added a format to it. If another program copied something, or added a format, the clipboard stays as that program left it (`[INFO] paste: the clipboard changed after the paste; the previous content was not put back`). Otherwise it empties the clipboard and puts the saved formats back in their order, the four markers and the daemon's own marker `KeySwitchRsRestored`. A clipboard another program holds open is tried every 25 ms for 1 s.

| Path | Time |
|---|---|
| Hook callback of the key that fires it | as for `Text`: one queued job, no Win32 call (bench rows R27, R28) |
| Key to keystroke, usual case (no earlier paste pending, no modifier held, an owner that answers, a small clipboard) | the executor's wake-up plus one `WM_NULL` round trip, the save, five writes and the keystroke: a few milliseconds (from code, not measured) |
| Key to keystroke, worst case | about 4.3 s once this `Paste` is the first one waiting: 1 s for an earlier paste's restore, 3 s of held modifiers, 100 ms owner check, 155 ms of open retries, 50 ms of save budget; then it is sent or skipped. A `Paste` step of a macro that waits behind another waiting `Paste` adds that one's wait ([§19](known-issues.md#s19) row 95) |
| A format being rendered when the save budget ends | as long as the owner takes, up to Windows' own limit of 30 s per format, with the clipboard open ([§19](known-issues.md#s19) row 95) |
| Keystroke to restore | `restore_delay`; while the target does not answer, up to 5 s |
| Memory | up to 64 MiB of saved formats, wiped when dropped, and as much again for the copies at the restore |

**What is saved and put back.** Everything that can be copied back (owner decision): the text first, then the other formats in the order the clipboard lists them, until 64 MiB in all or 50 ms after the save began; one read already running when the budget ends finishes. The formats Windows makes from others (`CF_TEXT`, `CF_OEMTEXT`, `CF_BITMAP`, the second of `CF_DIB`/`CF_DIBV5`) are not saved, since Windows makes them again from the restored ones. Metafiles, GDI-only formats, owner-drawn ones, the private and the GDI object ranges, the OLE link formats and a format whose name cannot be read are never saved ([§19](known-issues.md#s19) row 94); a warning counts what is lost: `[WARN] paste: 1 of 4 clipboard formats will not be restored (too large, too slow or unreadable)`.

**Privacy.**

- The phrase carries the markers `ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory` (0), `CanUploadToCloudClipboard` (0) and `Clipboard Viewer Ignore`, so neither the Windows clipboard history (Win+V), nor the cloud clipboard, nor a clipboard manager that honours them records it ([§19](known-issues.md#s19) rows 101 and 102). The restored copy carries the same markers, so the history does not record the previous content a second time.
- Content that its program marked private (`ExcludeClipboardContentFromMonitorProcessing`, or `CanIncludeInClipboardHistory` set to 0, as password managers mark a copied password) is not saved: at the restore the clipboard is emptied instead (`[INFO] paste: the clipboard held private content; it will be cleared, not put back`; owner decision; row 100).
- A copy made between the paste and the restore is kept: the restore checks, with the clipboard open, that the daemon's window still owns it. The clipboard sequence number is only logged at debug level.

**One paste at a time.**

- The executor jobs fired after a `Paste` that waits (for a released modifier, for an earlier restore, for a busy clipboard) run after it, in firing order; a macro that started before it goes on meanwhile. The keys you type, and the fast actions their bindings fire, are not held: they can reach the application before the keystroke ([§19](known-issues.md#s19) row 97).
- A second `Paste` within the restore delay waits for the first one's restore, at most 1 s, then takes the clipboard over and keeps the first one's saved content, which its own restore puts back. With a `restore_delay` over 1 s the first window can paste the second phrase (row 99).
- A `Paste` action fired while another one waits is dropped and counted in `paste_dropped` ([§18.6](errors.md#s18-6)); a `Paste` step of a macro is never dropped.
- `swch off`, `swch on` when remapping was off, a reload and a lock drop a `Paste` that waits; a restore that is pending still runs at its time. `swch exit`, and an executor that stops, put a pending clipboard back at once, for at most 250 ms more ([§19](known-issues.md#s19) row 93).

> [!WARNING]
> **Held modifiers ([KS-19](../audit.md#ks-19)), until roadmap item 3.1.** A `Paste` waits until no Shift, Ctrl, Alt or Win key is held, so that the keystroke goes out with exactly its own modifiers and no modifier changes state: with `keys: "Ctrl+Shift+V"` it pastes once both keys are released, and a modifier held for 3 s skips it. A modifier that the binding itself swallows is not seen as held ([KS-14](../audit.md#ks-14)). Roadmap item 3.1 replaces the wait with a release of the held modifiers around the keystroke ([§19](known-issues.md#s19) row 98).

> [!WARNING]
> **A busy browser or Electron page.** The check of step 10 asks the target window's own thread. In Chrome, Edge and Electron or WebView2 applications (Slack, Discord, VS Code, Teams) that thread answers at once while the page, which handles the Ctrl+V, can still be busy: the restore then comes after `restore_delay` and a page busy for longer pastes the previous content, a copied password for example. Give such windows a longer delay with a `when:`, such as `( keys: "F15", action: Paste(text: "Thanks!", restore_delay: 1500), when: AppEquals("slack.exe") )` ([§19](known-issues.md#s19) row 105). Remote Desktop and virtual machine windows read the clipboard late too (row 96).

Further limits: [§19](known-issues.md#s19) rows 93 to 106 (a killed daemon, formats that are not put back, a slow owner, the order with typed keys, the scan code of the keystroke, `CF_TEXT` readers, a clipboard owner that never answers).

<a id="s14"></a>
## 14. The `Sequence([...])` macro and its steps

Do not confuse the macro action `Sequence([...])` with the binding field `sequence:` ([§8](kinds.md#s8)). The field is the "key sequence" trigger, while `Sequence([...])` is an action. The two can be combined.

```text
action: Sequence([ <step>, <step>, ... ])
```

- The argument is a list in square brackets. `Sequence(Text("x"))` and `Sequence()` → ``Expected opening `[` ``.
- An empty list `Sequence([])` is allowed: it is queued and does nothing. A trailing comma and comments inside the list are allowed.
- The steps are the 9 variants of `RawStep` (`src/config/schema.rs`): `Window`, `Delay`, `Text`, `Paste`, `Key`, `Combo`, `Launch`, `OpenUrl` and `Media`. An unknown step gives ``Unexpected variant named `Fly` in enum `RawStep`, expected one of `Window`, `Delay`, `Text`, `Paste`, `Key`, `Combo`, `Launch`, `OpenUrl`, or `Media` instead``. Actions that are not among the steps give the same error: `PressKey("A")` inside a macro (write `Key("A")`) and a nested `Sequence([...])`.
- All steps are validated at load time ([src/config/loader.rs:704-714](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/config/loader.rs#L704-L714), [src/config/loader.rs:739-774](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/config/loader.rs#L739-L774)). The error gives the number of the first invalid step, counting from 0: `binding #0 [F13]: step #2: Combo "Ctrl+Foo": unknown key "Foo" in combo "Ctrl+Foo"`.
- `Sequence` as an action works with any trigger on `keys`, including `Tap`, `Hold` and `DoubleTap`, and also with the `sequence` and `chord` kinds (their trigger is always `Immediate`).

<a id="s14-1"></a>
### 14.1 How it runs

`src/core/executor/` (`runs.rs` walks the steps) and `run_step` in `src/data/sequence.rs`:

1. When the action fires, the thread that fired it calls `GetForegroundWindow()` once and remembers the **target window (HWND)**. The firing moment follows the table in [§13.1](#s13-1): the press for `Immediate`, the release for `Tap`, the timer expiry for `Hold`.
2. The macro is queued for the action executor, and the hook returns at once. No thread is started for it.
3. The executor runs the steps one after another up to the next `Delay`. A `Delay` stores its deadline and lets the executor run other actions and macros meanwhile; once the deadline has passed, the macro goes on with the next step. There are no pauses between steps other than `Delay`. Each `Text`, `Key`, `Combo` and `Media` step is a separate `SendInput` call.

- `Window(...)` steps always act on the remembered HWND, even if the focus has already moved.
- `Text`, `Key` and `Combo` go to the window that has the keyboard focus **at the time of the step**: `SendInput` is not addressed to a window.
- `Launch`, `OpenUrl` and `Media` do not depend on a window.
- A `Paste` step pastes through the clipboard and waits until its keystroke is sent; the restore of the clipboard does not hold up the next step ([§14.5](#s14-5)).
- A step error does not abort the macro: it is only logged, to the console and to `daemon.log`.
- `Launch` and `OpenUrl` inside `Sequence` run on the action executor, as the actions of the same name do. While one of them runs inside Windows (a cold browser, a dialog), the other macros and actions wait.
- Several macros run side by side on the one executor, as they did on a thread each before: they interleave at their `Delay` steps, oldest first, and the steps between two `Delay`s of one macro run without a break. A held `Immediate` macro starts once per press; with `repeat: true` it starts once per auto-repeat ([KS-10](../audit.md#ks-10)), up to 32 waiting to start, a further one dropped and counted.
- A step that panics stops only its macro: `[ERROR] an action panicked on the action executor; it keeps running`.

> [!NOTE]
> **Fixed on `feat/action-executor`** (roadmap [item 2.1](../roadmap.md#21-action-executor)). `swch off`, `swch on` when remapping was off, a reload (a saved `config.ron` that loads) and `swch exit` stop every running macro within 100 ms, also inside a `Delay` of any length, and drop the actions still queued; a step already running (a `Launch`, an `OpenUrl`) finishes first. A repeated `swch on` stops nothing, and neither does a new key press. A macro no longer starts a thread ([KS-33](../audit.md#ks-33)). The number of macros running at once has no limit.

<a id="s14-2"></a>
### 14.2 `Window(<operation>)`

`Window(Minimize)`, `Window(Maximize)`, `Window(Restore)`, `Window(Close)`. Case matters: `Window(minimize)` and `Window(Explode)` → ``Unexpected variant named ... in enum `RawWindowKind` ``. `Window` without parentheses → ``Expected opening `(` ``, `Window()` → `Expected identifier`.

| Operation | Call (`apply_window_op` in `src/data/sequence.rs`) |
|---|---|
| `Minimize` | `ShowWindowAsync(target, SW_MINIMIZE)`: minimize and activate the next window |
| `Maximize` | `ShowWindowAsync(target, SW_MAXIMIZE)`: maximize |
| `Restore` | `ShowWindowAsync(target, SW_RESTORE)`: return from the minimized or maximized state to the normal one |
| `Close` | `PostMessageW(target, WM_CLOSE, 0, 0)`: asynchronous; the application may ask about saving |

- If there was no foreground window when the macro started, the step is skipped with the message `[WARN] window step skipped: no foreground HWND captured`. The other steps still run.
- No operation waits for the window. `ShowWindowAsync` posts the change to the window's own thread, and the window changes when that thread handles it, so a window that is Not Responding holds up neither the macro nor the other actions (fixed on `feat/action-executor`, roadmap item 2.1; `ShowWindow` used to wait for the window). A change that Windows does not accept logs `[WARN] window step failed: ShowWindowAsync did not start Minimize` (or `Maximize`, `Restore`); a `Close` that cannot be posted logs `[WARN] window close PostMessage failed: ...`.
- After `Window(Minimize)` the focus moves to another window once Windows has handled the minimize, and the following `Text` and `Key` steps go there. Put a `Delay` between them (for example `Delay(100)`), or the first keys can still reach the window being minimized. The same holds after `Window(Restore)`, before typing into the restored window.

<a id="s14-3"></a>
### 14.3 `Delay(<ms>)`

`Delay(250)`: the argument is a `u64` in milliseconds. Since roadmap [item 2.1](../roadmap.md#21-action-executor) the action executor keeps the deadline and goes on with the macro when its high-resolution waitable timer wakes it: within about 1 ms after the deadline, about 0.3 ms on average, as with the `thread::sleep` it replaces. A `Delay` never ends early, and short ones pace keys as written: 100 steps of `Delay(5)` take about 0.5 s. Windows 10 before version 1803 has no high-resolution timer; there the executor's timer, like `thread::sleep`, wakes on a tick of the system clock, about 15.6 ms apart.

- All integer forms work ([§3.7](ron-syntax.md#s3-7)): `Delay(0x10)`, `Delay(1_000)`, `Delay(+5)`.
- `Delay(0)` is allowed, and lets the other macros run first. There is no upper bound: `Delay(18446744073709551615)` loads and holds the macro until `swch off`, a reload or `swch exit` stops it.
- Errors: `Delay(-1)` and `Delay(18446744073709551616)` → `Integer is out of bounds`; `Delay("100")` and `Delay()` → `Expected integer`; `Delay(1.5)` → ``Expected closing `)` ``.
- It blocks neither the hook nor other actions and macros, and `swch off`, a reload or `swch exit` cut it short.
- Typical uses: give an application time after `Window(...)` or `Launch(...)` before typing; let the user release the trigger's modifiers.

<a id="s14-4"></a>
### 14.4 `Text("...")`

`Text("Привет, мир")`. It accepts any RON string, including raw strings and escapes ([§3.9](ron-syntax.md#s3-9)). `Text(123)`, `Text('a')` and `Text()` → `Expected string`. `Text("")` is a no-op. As an action, `Text("")` is a load error ([§13.10](#s13-10)).

Code: [src/data/sequence.rs:123-161](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/sequence.rs#L123-L161).

- The string is converted to UTF-16. For each code unit two `INPUT` events are built: a press and a release with `KEYEVENTF_UNICODE`, `wVk = 0`, and `wScan` = the code unit.
- The whole string goes out in **one** `SendInput` call: it does not interleave with physical input, but it is not slowed down either. If some of the events are lost, the console shows `[WARN] type_text: SendInput sent N of M inputs`.
- Windows delivers such input as `VK_PACKET` and then `WM_CHAR`. Consequences:
  - the input does not depend on the layout, CapsLock or Shift: `Text("Привет")` types Cyrillic in the English layout;
  - characters outside the BMP (emoji) go out as two surrogates, and Windows joins them itself;
  - a very long string goes out as one batch, and some applications lose characters. Split it into several `Text` steps with `Delay`;
  - programs that do not handle `VK_PACKET` (some games, RDP and VM clients) may not receive the text.

> [!WARNING]
> **Known issue.** `\n`, `\t` and `\r` do not turn into the Enter and Tab keys ([src/data/sequence.rs:132-135](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/sequence.rs#L132-L135)). The application receives the **character** U+000A, U+0009 or U+000D as `WM_CHAR`, not a `VK_RETURN` or `VK_TAB` key press. Simple input fields may insert a line break or a tab, but anything that reacts to the key itself (sending in a messenger, submitting a form, moving the focus with Tab, autocompletion, terminals) does not see it. The reliable way: `Text("line 1"), Key("Enter"), Text("line 2")`.

- A real line break inside the literal also becomes part of the text. In a file with CRLF line endings (the default for editors on Windows) that is `"\r\n"`, that is, two characters.

<a id="s14-5"></a>
### 14.5 `Paste("...")`

`Paste("Kind regards,")` and `Paste(text: "Kind regards,", keys: "Shift+Insert", restore_delay: 800)`: the same two forms, defaults, bounds and load errors as the action ([§13.11](#s13-11)), with the step number in the message: `binding #0 [F13]: step #1: Paste: empty`. Unlike the step `Text("")`, the step `Paste("")` is an error: an empty paste would still swap the clipboard.

```ron
( keys: "F13", action: Sequence([Key("Tab"), Paste("first field"), Key("Tab"), Paste("second field"), Key("Enter")]) ),
```

- The macro waits at the step until its paste is over: from the keystroke on, the next step runs at once, while the restore of the clipboard waits for its delay. The keystroke goes to the window that has the keyboard focus when it is sent, as a `Key` step does; the target whose answer the restore waits for is the foreground window when the step starts.
- A second `Paste` step within the delay waits for the first one's restore, at most 1 s ([§13.11](#s13-11)). A `Paste` step is never dropped, even while a `Paste` action waits.
- While the step waits (for a released modifier, for an earlier restore, for a busy clipboard), the jobs fired after the macro wait behind it, and the other macros go on.
- `swch off`, a reload and `swch exit` stop the macro at the step as at a `Delay`; a clipboard the step already changed is still put back.

<a id="s14-6"></a>
### 14.6 `Key("...")`

`Key("Enter")` is a single key name, as in `PressKey`. `Key("Ctrl+S")` → `step #0: Key: unknown key "Ctrl+S"`, `Key("Bogus")` → `Key: unknown key "Bogus"`, `Key()` → `Expected string`. The behaviour is the same as for `PressKey` ([src/data/sequence.rs:163-174](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/sequence.rs#L163-L174)): a VK tap, no modifiers are added, the result depends on the layout, and physically held modifiers combine with it ([KS-19](../audit.md#ks-19)). Lost events are printed as `[WARN] press_key: SendInput sent N of 2 ...`.

<a id="s14-7"></a>
### 14.7 `Combo("...")`

`Combo("Ctrl+Shift+T")` is a combination string following the rules of [§6.2](keys.md#s6-2): `+` is the separator, spaces are allowed, case does not matter, duplicates are removed, the written order is kept, and names with a side (`LCtrl` and so on) are allowed and send their own VK.

Errors: `Combo "": combo is empty` (the same for a string of only spaces); `Combo "Ctrl++A": empty segment in combo "Ctrl++A" (stray '+'?)` (the same for `"Ctrl+"`); `Combo "Ctrl+Foo": unknown key "Foo" in combo "Ctrl+Foo"`.

Press order ([src/data/sequence.rs:179-244](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/sequence.rs#L179-L244)). The keys are split into modifiers (`Shift`, `LShift`, `RShift`, `Ctrl`, `LCtrl`, `RCtrl`, `Alt`, `LAlt`, `RAlt`, `Win` = `LWin`, `RWin` and their synonyms) and main keys (all the others). Then one `SendInput` call sends:

1. the modifier presses, in written order;
2. the main key presses, in written order;
3. the main key releases, in **reverse** order;
4. the modifier releases, in **reverse** order.

Examples:

- `Combo("Ctrl+Shift+T")` → Ctrl↓ Shift↓ T↓ T↑ Shift↑ Ctrl↑.
- `Combo("A+Ctrl")` works the same as `Ctrl+A`: the modifiers always go first.
- `Combo("Ctrl+K+C")` → Ctrl↓ K↓ C↓ C↑ K↑ Ctrl↑. This is **not** the two-step Ctrl+K, Ctrl+C of VS Code; that one needs two steps: `Combo("Ctrl+K"), Combo("Ctrl+C")`.
- `Combo("Ctrl+Shift")`, made only of modifiers, is allowed. It can trigger the system language-switch hotkey, and the daemon's auto-blocker does not prevent that (the hook passes the daemon's own input through).
- Shortcuts such as Ctrl+S work in any layout, because applications check the VK.

> [!WARNING]
> **Known issue ([KS-19](../audit.md#ks-19)).** The modifier releases are sent even if the user is physically holding the same modifier. After that the OS considers the key released although it is down ([src/data/sequence.rs:200-202](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/sequence.rs#L200-L202)).

- Held modifiers that are not in the combination are added to it: with Shift held, `Combo("Ctrl+S")` turns into Ctrl+Shift+S ([src/data/sequence.rs:179](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/sequence.rs#L179); [§13.1](#s13-1), [KS-19](../audit.md#ks-19)).

<a id="s14-8"></a>
### 14.8 `Launch(...)`, `OpenUrl("...")` and `Media(...)` as steps

The syntax and semantics are the same as for the actions of the same name ([§13.6](#s13-6)–[§13.8](#s13-8), [src/data/sequence.rs:113-115](https://github.com/bluesteelll/key-switch-rs/blob/f4c1fb6/src/data/sequence.rs#L113-L115)), including the elevated rights and the console for `Launch` ([KS-09](../audit.md#ks-09)). They run on the action executor, as those actions do. `Launch` waits neither for the process to start nor for its window to appear, so put a `Delay(...)` before typing into a new window.
