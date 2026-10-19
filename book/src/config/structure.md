# File location, reload and top-level structure

The daemon reads one file: the first command-line argument, or `config.ron` next to `key-switch-rs.exe`. It writes the default template only for the second, only at startup, and reloads the file about 250 ms after every save. A file that does not load keeps the previous bindings live.

<a id="s2"></a>
## 2. File location, creation and reload

Main files: [src/main.rs](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs), [src/config/loader.rs](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs), [src/hook/config_watcher.rs](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs). Facts describe commit `20af28d`, the tip of `master` after roadmap Phase 1.

<a id="s2-1"></a>
### 2.1 Where the file is

The path is determined once, when the daemon starts ([src/main.rs:67-85](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L67-L85), `ConfigPath::from_arg` at [src/config/loader.rs:120-136](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L120-L136)):

1. **The first command-line argument**, if one is given. A relative path is made absolute against the daemon's current folder, once, with `std::path::absolute`: `key-switch-rs.exe keys.ron` started in `D:\cfg` uses `D:\cfg\keys.ron`. The file must exist ([§2.2](#s2-2)).
2. Without an argument, **`<exe folder>\config.ron`** is used (`default_config_path`, [src/config/loader.rs:98-108](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L98-L108)). The folder comes from `std::env::current_exe()`.

The hot-reload watcher receives the same absolute path ([src/main.rs:79](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L79)), and the console prints it in full.

Details:

- The file name and extension can be anything: the format is determined by the content alone.
- The daemon has no flags, so any first argument is taken as a path. `key-switch-rs.exe --help` stops with `[FATAL] config file <current folder>\--help does not exist`, unless a file of that name exists ([KS-41](../audit.md#ks-41), not independently verified; partly fixed in `1dfd80a`).
- A typo in an explicit path stops the daemon with `[FATAL] config file <path> does not exist`. Nothing is created, not even missing folders.
- A Scheduled Task with a relative argument resolves it against the task's "Start in" folder.
- `swch open` starts `key-switch-rs.exe` from the folder of `swch.exe`, elevated (UAC) and without arguments ([src/bin/swch.rs:96-103](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/bin/swch.rs#L96-L103), [src/bin/swch.rs:122-131](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/bin/swch.rs#L122-L131)). A daemon started this way therefore always uses `<exe folder>\config.ron`.

<a id="s2-2"></a>
### 2.2 First run: the default file

`config::load` ([src/config/loader.rs:151-173](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L151-L173)) reads the file. What happens when it is missing depends on where the path came from:

| Path | File missing at startup |
|---|---|
| `<exe folder>\config.ron` | The loader creates the folder if needed, writes the built-in template, prints `[INFO] generated default config at <path>` and loads the file ([src/config/loader.rs:424-449](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L424-L449)) |
| A command-line argument | `[FATAL] config file <path> does not exist`; nothing is written |

If the file exists, `[INFO] loading config from <path>` is logged, on the console and in `daemon.log` ([Log file](../build-and-run.md#log-file)). The template is `src/config/default_config.ron`, embedded into the exe at compile time (`include_str!`, [src/config/loader.rs:30-32](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L30-L32)), so it always matches the program version. The template is written only by this startup load. A hot reload never writes a file ([§2.3](#s2-3)).

The template is embedded byte for byte. The repository stores `default_config.ron` with LF line endings (`git ls-files --eol`: `i/lf`); CRLF appears only in a working copy checked out with `core.autocrlf=true`. The generated file therefore has CRLF or LF line endings depending on which working copy the exe was built from. It makes no difference to the loader.

The template has two active bindings ([src/config/default_config.ron:226-236](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/default_config.ron#L226-L236)); everything else is comments and commented-out examples:

```ron
( keys: "CapsLock",       action: SwitchLanguage, block_original_combo: true ),
( keys: "Shift+CapsLock", action: ToggleCapsLock ),
```

With the auto-blocker this gives 3 bindings: a blocker for the system language hotkey is appended after the file's bindings. It is `Alt+Shift` unless the registry sets `Ctrl+Shift` or `` ` `` ([§15](matching.md#s15)). The validator prints, on a machine where the registry gives no hotkey:

```text
[INFO] SwitchLanguage: using default combination
OK: 3 bindings (after auto-blocker expansion)
  #0: CapsLock                           -> switch language
  #1: Shift + CapsLock                   -> toggle CapsLock
  #2: [AUTO-BLOCK] Alt + Shift              -> (blocked)
```

The template's header describes hot reload, field checking and the BOM correctly ([src/config/default_config.ron:1-12](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/default_config.ron#L1-L12)). The documentation drift that the audit found in it was fixed in [`13ce323`](https://github.com/bluesteelll/key-switch-rs/commit/13ce323) and Workstream B ([Documentation drift](../gaps.md#documentation-drift)). Two commented-out examples still need care:

> [!WARNING]
> - The `Ctrl+Pause` example ([src/config/default_config.ron:268](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/default_config.ron#L268)) most likely does not fire: for Ctrl+Pause, Windows sends the code `VK_CANCEL` ([§6.6](keys.md#s6-6)).
> - The `keys: "LShift"` example with `DoubleTap(250)` ([src/config/default_config.ron:308-318](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/default_config.ron#L308-L318)) fires, and since `feat/deferred-block-default` (roadmap [item 1.3](../roadmap.md#13-block_default-for-tap-hold-and-doubletap)) every left-Shift press still reaches applications, so the left Shift keeps typing capitals. Pressing the left Shift twice within 250 ms for two capitals, as in `<T>`, no longer toggles CapsLock: the key typed in between cancels the double tap (roadmap item 2.2, [§11.8](triggers-conditions.md#s11-8)). The template says both.

<a id="s2-3"></a>
### 2.3 Hot reload

Code: [src/hook/config_watcher.rs](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs). The watcher starts after the hook is installed and runs for as long as the daemon runs.

How it works:

1. `notify` watches the file's **parent folder**, non-recursively ([L117-L121](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L117-L121)). This way the watch survives editors that save through a temporary file and a rename.
2. An event passes the filter if its kind is `Modify(*)`, `Create(*)` or `Remove(*)` and one of its paths has the config's **file name** ([L63-L75](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L63-L75)). The names are compared with `CompareStringOrdinal` ignoring case, the way the file system compares them ([L78-L91](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L78-L91)). The folder part is not compared: the watch is non-recursive, so the name alone identifies the file. `Modify` covers writes, renames and attribute changes. Events for other files in the folder are ignored.
3. Debounce: `DEBOUNCE_MS = 250` ([L35-L40](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L35-L40), loop at [L131-L154](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L131-L154)). The first event wakes the thread; it then waits until 250 ms pass **with no new events**, and every new event resets the timer. The reload happens about 250 ms after the *last* event of a burst.
4. `config::reload` runs ([L156-L170](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L156-L170), [src/config/loader.rs:175-182](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L175-L182)). It reads the file in full, prints `[INFO] loading config from <path>`, parses and validates it, and rebuilds the auto-blockers (the registry is read again). It never writes a file.
5. On success, the watcher thread hands the list to the daemon's main thread and waits at most 2 s for it (roadmap [item 2.3](../roadmap.md#23-control-window-and-state-reset), `MainThreadHandle::reload`). Between two key events the main thread calls `hook.update_bindings(...)`, which sorts the new list ([§16](matching.md#s16)), swaps it in atomically through `ArcSwap` and stops the running macros, and resets the engine in the same step: pending gestures, chords and sequences are dropped, the keys held stay held. The console shows `[INFO] config reloaded (N bindings live)`; `N` includes the auto-blockers. If the main thread does not answer within 2 s, the watcher logs `[WARN] config reload: the daemon's main thread did not answer within 2 s; the new bindings go live when it does`, and the main thread logs `[INFO] config reload carried out late: N bindings live` when it does ([§18.6](errors.md#s18-6)).
6. On a parse, validation or read error:

   ```text
   [ERROR] config reload failed:
   <error text>
   [INFO] keeping previous bindings live
   ```

   For binding validation errors ([§18.5](errors.md#s18-5)) there is also an empty line before `[INFO] …`, because the text of such an error itself ends with a line break. The previous binding list **keeps working**.

| Situation | What happens |
|---|---|
| A valid file is saved | The new bindings take effect about 250 ms after the last event |
| Syntax error, unknown key, unknown or misspelled field, timing value out of range, file not in UTF-8, empty file | Error in the console and in `daemon.log`; the previous bindings keep working |
| The file is temporarily locked by the editor or an antivirus (sharing violation, `os error 32`) | **Known issue:** the read fails, `[ERROR] config reload failed` is logged, and **there is no retry**. The previous bindings stay live; the new ones take effect only on the next save ([src/hook/config_watcher.rs:156-170](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L156-L170)) |
| `bindings` is misspelled or missing | An error that names the field ([§4](#s4)); the previous bindings keep working |
| `bindings: []` | Loads: 0 bindings are applied, remapping does nothing, and the console shows `[WARN] config contains no bindings` |
| The file is deleted or renamed | `[ERROR] config reload failed:` with `config file <path> does not exist`; the previous bindings stay live and no file is written. Putting the file back (or renaming it back) reloads it |
| The editor saves through "delete + create" or "temporary file + rename" | Works: the reload runs 250 ms after the last event of the save |
| The config path was relative on the command line | Works: the path was made absolute at startup ([§2.1](#s2-1)) |
| The letter case of the given path differs from the name on disk (`CONFIG.RON` versus `config.ron`) | Works: file names are compared ignoring case |
| The watcher could not be started | `[WARN] could not start config file watcher for <path>: <error>` and `[INFO] continuing without hot-reload` are printed; the daemon runs without reload ([src/core/app.rs:177-199](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L177-L199)) |

These rows come from the code and the watcher's tests, which run the real `notify` watcher on a temporary folder ([src/hook/config_watcher.rs](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs), for example `a_spawned_watcher_keeps_the_bindings_when_the_file_is_deleted`). They were not observed on a live daemon.

Further points:

- **At startup**, any config error is fatal: `[FATAL] <error>` is logged, on the console and in `daemon.log`, then `Press Enter to exit...` is printed ([src/main.rs:46-62](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L46-L62)). There is no partial load: one invalid binding rejects the whole file ([src/config/loader.rs:272-284](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L272-L284)).
- `swch` can neither trigger nor check a reload: it has only the commands `on`, `off`, `exit`, `status` and `open`, and no `reload` or `check` command ([src/ipc.rs:157-171](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/ipc.rs#L157-L171); [roadmap item 4.3](../roadmap.md#43-ipc-v2-commands)). Errors are visible in the daemon's console, which is a separate, elevated console window if the daemon was started with `swch open`, and in `%LOCALAPPDATA%\key-switch-rs\daemon.log` ([Log file](../build-and-run.md#log-file)).
- A reload drops the gestures that have already started: armed `Tap`/`Hold`/`DoubleTap` gestures and unfinished `chord`/`sequence` gestures, so none fires its old action (roadmap [item 2.3](../roadmap.md#23-control-window-and-state-reset), fixed on `feat/control-window`; [KS-31](../audit.md#ks-31)). It does not re-read the keys from Windows, because a reload loses no key event: a key held keeps its press, and a key whose press the old bindings swallowed keeps its release swallowed ([§19](known-issues.md#s19) row 77). Until then a reload left those gestures armed with the old actions.
- The Windows hotkey value in the registry is re-read on every reload. Changing this setting in Windows does not trigger a reload by itself: re-save `config.ron` or restart the daemon.
- At startup the daemon prints the binding list in load order: the file's bindings, then the auto-blockers, marked `[AUTO-BLOCK] … -> (blocked)` ([src/core/app.rs:100-117](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L100-L117)). That is not the order in which bindings are checked ([§16](matching.md#s16)). Keys in this list are printed with their canonical names ([§6.4](keys.md#s6-4)). On reload the list is not printed.

<a id="s4"></a>
## 4. Top level of the file

The file contains exactly one value: an unnamed struct with a single field, `bindings`, which holds the list of bindings (`RawConfig`, [src/config/schema.rs:21-28](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/schema.rs#L21-L28)):

```ron
(
    bindings: [
        ( keys: "CapsLock", action: SwitchLanguage ),
        // ...
    ],
)
```

- `bindings` is required. `()` gives ``Unexpected missing field named `bindings` in `RawConfig` ``, and the typo `( bindngs: [] )` gives ``Unexpected field named `bindngs` in `RawConfig`, expected `bindings` instead``.
- `bindings: []` loads with 0 bindings and prints `[WARN] config contains no bindings` ([src/config/loader.rs:286-288](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L286-L288)). This lets you comment out every binding.
- Any other field at the top level is an error that names it: `( bindings: [], bad: ) )` gives ``Unexpected field named `bad` in `RawConfig`, expected `bindings` instead``.
- After the closing `)` only whitespace and comments are allowed; anything else gives `Non-whitespace trailing characters`.
- An empty file, and a file that holds only comments, give ``Expected opening `(` for struct `RawConfig` ``.
- Without the outer parentheses (`bindings: [...]`) you get ``Expected struct `RawConfig` but found `bindings` ``.
- `bindings` must be a list `[...]`, not a tuple `(...)`; otherwise you get ``Expected opening `[` ``.
- Each list element is a struct `( … )`. A stray comma, or the end of the file, right after `[` gives ``Expected opening `(` for struct `RawBinding` `` ([§3.3](ron-syntax.md#s3-3)).

<a id="s5"></a>
## 5. Binding fields

<a id="s5-1"></a>
### 5.1 Field table

Each element of `bindings` is a struct `( … )` (`RawBinding`, [src/config/schema.rs:30-85](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/schema.rs#L30-L85)). Fields can appear in any order.

| Field | Type and RON form | Default | Meaning |
|---|---|---|---|
| `keys` | combo string: `"Ctrl+Shift+A"` | not set | Combo: fires on the key-down of one of its keys while all of its keys are held and the modifiers held are exactly its own ([§7](kinds.md#s7)). String syntax: [§6.2](keys.md#s6-2) |
| `sequence` | `(steps: ["F13", "Ctrl+F14"], max_gap_ms: 500)` | not set; `max_gap_ms` = 500 | Sequence of steps; each step is a combo string ([§8](kinds.md#s8)) |
| `chord` | `(keys: ["F15", "F16"], window_ms: 50)` | not set; `window_ms` = 50 | Chord: all keys pressed within the window, in any order ([§9](kinds.md#s9)) |
| `action` | a `RawAction` variant: `SwitchLanguage`, `PressKey("L")`, `Launch(exe: "…")`, … | **required** | What to do when the binding fires ([§13](actions.md#s13), [§14](actions.md#s14)) |
| `trigger` | `Immediate`, `Tap(ms)`, `Hold(ms)`, `DoubleTap(ms)` | `Immediate` | When the action fires ([§11](triggers-conditions.md#s11)). Deferred triggers are allowed only together with `keys` |
| `when` | a `RawCondition` variant: `Always`, `TitleContains("…")`, `TitleEquals("…")`, `AppEquals("x.exe")`, `Not(…)`, `And([…])`, `Or([…])` | `Always` | Condition on the active window ([§12](triggers-conditions.md#s12)). If the condition is false, the binding is skipped entirely: it does not fire and suppresses nothing |
| `block_default` | `true` / `false` (`Some(..)` and `None` too, [§3.6](ron-syntax.md#s3-6)) | `true`; `false` for a `Tap`, `Hold` or `DoubleTap` binding whose combination is made only of modifiers (`"LShift"`, `"Ctrl+Shift"`, `"Win"`) | Whether to hide the pressed key from the OS: its press, its auto-repeats and its release. Honoured for `keys` with every trigger, for `Tap`, `Hold` and `DoubleTap` since `feat/deferred-block-default`; a gesture is detected either way. `sequence` and `chord` ignore it and always hide their keys ([§10](kinds.md#s10)) |
| `block_original_combo` | `true` / `false` / `MaskOnly` | `false` | Add an auto-blocker for the system hotkey that does the same thing as `action` ([§15](matching.md#s15)): `true` swallows the hotkey pressed on its own, `MaskOnly` (experimental) lets it through with the menu mask key ([§15.5](matching.md#s15-5)). The blocker acts only where this binding's `when` holds. Works only for `SwitchLanguage` and `SwitchLanguageBackward`; for other actions the flag is silently ignored |
| `allow_extra_modifiers` | `true` / `false` | `false` | `keys` only: also fire while modifiers the binding does not name are held ([§7](kinds.md#s7)). `true` on a `sequence` or `chord` is a load error |
| `repeat` | `true` / `false` | `true` for `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)`; `false` for every other action | `keys` with `trigger: Immediate` only: fire again on every auto-repeat while the key is held ([§11.3](triggers-conditions.md#s11-3)). With `false` the action fires once per press; the repeats and the release are still hidden when the press was. `true` on a `Tap`, `Hold` or `DoubleTap` trigger, a `sequence` or a `chord` is a load error; `false` is accepted there. The startup list marks a value that differs from the default with `[repeat]` or `[no repeat]` |

Example with all fields:

```ron
(
    keys: "Ctrl+Alt+L",
    action: PressKey("L"),
    block_default: false,
    block_original_combo: false,
    when: TitleContains("Visual Studio Code"),
    trigger: Immediate,
    allow_extra_modifiers: false,
    repeat: false,
),
```

<a id="s5-2"></a>
### 5.2 Exactly one of `keys` / `sequence` / `chord`

`raw_binding_to_kind` ([src/config/loader.rs:513-543](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L513-L543)) counts how many of the three fields are set (`None` counts as not set):

- none: ``binding has none of `keys` / `sequence` / `chord` set — must specify exactly one``;
- two or three: ``binding has multiple of `keys` / `sequence` / `chord` set — must be exactly one``.

The chosen kind determines which other fields have an effect:

| | `keys` | `sequence` | `chord` |
|---|---|---|---|
| `trigger` | any | only `Immediate` (anything else is an error) | only `Immediate` (anything else is an error) |
| `block_default` | honoured with every trigger | ignored: the keys are always hidden | ignored: the keys are always hidden |
| `block_original_combo` | works | works (the auto-blocker is still a combo) | works |
| `when` | works | works | works |
| `allow_extra_modifiers` | works | `true` is an error | `true` is an error |
| `repeat` | honoured with `Immediate`; `true` is an error with `Tap`/`Hold`/`DoubleTap` | `true` is an error | `true` is an error |

<a id="s5-3"></a>
### 5.3 Unknown fields and typos

Every struct of the schema rejects fields it does not know: the root, a binding, `sequence: (…)`, `chord: (…)`, and the actions and steps with named fields, `PostMessage(…)` and `Launch(…)` ([src/config/schema.rs:18-19](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/schema.rs#L18-L19), [KS-22](../audit.md#ks-22), fixed in `1dfd80a`). The error is a parse error with the line and column, and it lists the valid names. RON reports only the first such error. The validator gives:

| Written | Error |
|---|---|
| `tigger: Tap(200)` in a binding | ``Unexpected field named `tigger` in `RawBinding`, expected one of `keys`, `sequence`, `chord`, `action`, `block_default`, `block_original_combo`, `when`, `trigger`, `allow_extra_modifiers`, or `repeat` instead`` |
| `Keys:` (different case) | the same message for `` `Keys` `` |
| `acton: DoNothing` | the same message for `` `acton` ``; the misspelling is reported before the missing `action` |
| `max_gap: 300` in `sequence` | ``Unexpected field named `max_gap` in `SequenceSpec`, expected either `steps` or `max_gap_ms` instead`` |
| `cwd: "C:/temp"` in `Launch`, as an action or a macro step | ``Unexpected field named `cwd` in `Launch`, expected either `exe` or `args` instead`` |
| `lparm: 5` in `PostMessage` | ``Unexpected field named `lparm` in `PostMessage`, expected one of `msg`, `wparam`, or `lparam` instead`` |

At startup such a file is fatal. Saved while the daemon runs, it is rejected and the previous bindings stay live ([§2.3](#s2-3)).

A missing required field is reported as missing: `step:` in place of `steps:` in `sequence` is first an unknown field, while a `sequence: ()` without `steps` gives ``Unexpected missing field named `steps` in `SequenceSpec` ``. A binding without any kind field gives ``binding has none of `keys` / `sequence` / `chord` set …``.

Repeating the same field (`keys: "F13", keys: "F14"`) gives ``Unexpected duplicate field named `keys` in `RawBinding` ``. If two different kind fields are set (`keys` and `sequence`), the error is `binding has multiple of …`.
