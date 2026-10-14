# File location, reload and top-level structure

<a id="s2"></a>
## 2. File location, creation and reload

<a id="s2-1"></a>
### 2.1 Where the file is

The path is determined once, when the daemon starts ([src/main.rs:28-31](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/main.rs#L28-L31)):

1. **The first command-line argument**, if one is given. The path is taken as is, without normalisation; a relative path is resolved against the process's current directory: `key-switch-rs.exe D:\cfg\keys.ron`.
2. Without an argument, **`<exe folder>\config.ron`** is used (`default_config_path`, [src/config/loader.rs:83-95](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L83-L95)). The folder comes from `std::env::current_exe()`.

The hot-reload watcher receives the same path ([src/main.rs:35](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/main.rs#L35)).

Details:

- The file name and extension can be anything: the format is determined by the content alone.
- The daemon has no flags, so any first argument is taken as a path. `key-switch-rs.exe --help` creates a file named `--help` containing the template in the current folder and starts with it ([src/main.rs:29](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/main.rs#L29), [src/config/loader.rs:102](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L102); [KS-41](../audit.md#ks-41), not independently verified).
- A typo in an explicit path causes no error: the file and all missing folders are created from the template ([src/config/loader.rs:194-206](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L194-L206); [KS-41](../audit.md#ks-41)).
- Pass an **absolute** path: with a relative path, hot reload does not work ([§2.3](#s2-3), [KS-21](../audit.md#ks-21)).
- `swch open` starts `key-switch-rs.exe` from the folder of `swch.exe`, elevated (UAC) and without arguments ([src/bin/swch.rs:81-84](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/bin/swch.rs#L81-L84), [src/bin/swch.rs:110](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/bin/swch.rs#L110)). A daemon started this way therefore always uses `<exe folder>\config.ron`.

<a id="s2-2"></a>
### 2.2 First run: the default file

`config::load` ([src/config/loader.rs:101-122](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L101-L122)) first checks `path.exists()`. If the file does not exist, the loader:

1. creates the parent folders (`fs::create_dir_all`), if the path contains any;
2. writes the built-in template. The file `src/config/default_config.ron` is embedded into the exe at compile time (`include_str!`, [src/config/loader.rs:28](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L28)), so the template always matches the program version;
3. prints `[INFO] generated default config at <path>` and immediately loads that file like any other.

If the file exists, `[INFO] loading config from <path>` is printed.

The template is embedded byte for byte. The repository stores `default_config.ron` with LF line endings (`git ls-files --eol`: `i/lf w/crlf`); CRLF appears only in a working copy checked out with `core.autocrlf=true`. The generated file therefore has CRLF or LF line endings depending on which working copy the exe was built from. It makes no difference to the loader.

The template has two active bindings ([src/config/default_config.ron:125-135](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L125-L135)); everything else is comments and commented-out examples:

```ron
( keys: "CapsLock",       action: SwitchLanguage, block_original_combo: true ),
( keys: "Shift+CapsLock", action: ToggleCapsLock ),
```

With the auto-blocker this gives 3 bindings: a blocker for the system language-switch combination, with the action `DoNothing`, is inserted before `CapsLock`. It is `Alt+Shift` unless the registry sets `Ctrl+Shift` or `` ` `` ([§15](matching.md#s15)).

The template is generated on **every** load that finds no file, including a hot reload ([§2.3](#s2-3)).

The template's comments disagree with the code in places (see also [Documentation drift](../gaps.md#documentation-drift)):

> [!WARNING]
> Known issues in the template comments:
>
> - The header says "Reload requires restarting the program.", although hot reload exists ([src/config/default_config.ron:3](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L3)).
> - The list of macro steps ([src/config/default_config.ron:87-98](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L87-L98)) names only 5 of the 8 variants: `Launch`, `OpenUrl` and `Media` are missing ([§14](actions.md#s14)).
> - [Line 115](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L115) promises "case-insensitive string comparisons" for all `when` conditions, but `AppEquals` and `TitleEquals` ignore case only for Latin (ASCII) letters ([src/data/condition.rs:62](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L62), [src/data/condition.rs:71](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/condition.rs#L71); [§12](triggers-conditions.md#s12); [KS-28](../audit.md#ks-28), not independently verified).
> - [Lines 78-81](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L78-L81) ("resolves whichever wins") and the example at [lines 186-198](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L186-L198), with equal `Tap(200)` and `Hold(200)` on one key, describe a situation in which the thread scheduler decides the outcome ([§11.6](triggers-conditions.md#s11-6), [KS-25](../audit.md#ks-25)). In addition, `Hold` with `PressKey("LShift")` from this example produces one short Shift press, not a held Shift.
> - The `AppEquals` examples ([src/config/default_config.ron:173](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L173), [src/config/default_config.ron:181](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L181)) never fire ([§12.3](triggers-conditions.md#s12-3), [KS-03](../audit.md#ks-03)).
> - The `Ctrl+Pause` example ([src/config/default_config.ron:163](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L163)) most likely does not fire either: for Ctrl+Pause, Windows sends the code `VK_CANCEL` ([§6.6](keys.md#s6-6)).
> - The example `keys: "LShift"` with `DoubleTap(250)` ([src/config/default_config.ron:200-205](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L200-L205)) never fires: sided names in `keys` do not match ([§6.7](keys.md#s6-7), [KS-07](../audit.md#ks-07)).

The `["g","g"]` and `["j","k"]` examples at [lines 17-21](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L17-L21) work, but with them every typed g, j and k is lost ([§10](kinds.md#s10)). The template itself warns about this at [lines 23-25](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L23-L25).

<a id="s2-3"></a>
### 2.3 Hot reload

Code: [src/hook/config_watcher.rs](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs). The watcher starts after the hook is installed and runs for as long as the daemon runs.

How it works:

1. `notify` watches the file's **parent folder**, non-recursively ([L65-L69](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L65-L69)). This way the watch survives editors that save through a temporary file and a rename.
2. An event passes the filter if its path equals the config path ([L47](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L47)) and its kind is `Modify(*)`, `Create(*)` or `Remove(*)` ([L51-L54](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L51-L54)). `Modify` covers writes, renames and attribute changes. Events for other files in the folder are ignored. The comparison uses `Path::eq`, which compares path components: `/` and `\`, repeated separators and intermediate `.` are treated as equal, but **letter case and `..` are not**. In addition, `notify` reports absolute paths.
3. Debounce: `DEBOUNCE_MS = 250` ([L29](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L29), loop at [L92-L98](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L92-L98)). The first event wakes the thread; it then waits until 250 ms pass **with no new events**, and every new event resets the timer. The reload happens about 250 ms after the *last* event of a burst.
4. The same `config::load` as at startup is called ([L100](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L100)): `[INFO] loading config from <path>` is printed again, the file is read in full, parsed and validated, and the auto-blockers are rebuilt (the registry is read again).
5. On success, `hook.update_bindings(...)` sorts the new list ([§16](matching.md#s16)) and swaps it in atomically through `ArcSwap` ([L103](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L103)). The console shows `[INFO] config reloaded (N bindings live)`; `N` includes the auto-blockers.
6. On a parse, validation or read error ([L106-L108](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L106-L108)):

   ```text
   [ERROR] config reload failed:
   <error text>
   [INFO] keeping previous bindings live
   ```

   For binding validation errors ([§18.5](errors.md#s18-5)) there is also an empty line before `[INFO] …`, because the text of such an error itself ends with a line break. The previous binding list **keeps working**.

| Situation | What happens |
|---|---|
| A valid file is saved | The new bindings take effect about 250 ms after the last event |
| Syntax error, unknown key, BOM ([KS-24](../audit.md#ks-24)), file not in UTF-8, empty file | Error in the console; the previous bindings keep working |
| The file is temporarily locked by the editor or an antivirus (sharing violation, `os error 32`) | **Known issue:** `read_to_string` fails, `[ERROR] config reload failed` is printed, and **there is no retry**. The previous bindings stay live; the new ones take effect only on the next save ([src/hook/config_watcher.rs:100-109](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L100-L109)) |
| The `bindings` field is deleted, empty (`[]`) or misspelled | **This is not an error.** 0 bindings are applied, and all remapping is turned off. The console shows only `[WARN] config contains no bindings` and `[INFO] config reloaded (0 bindings live)`. **Known issue** ([KS-22](../audit.md#ks-22)): unknown fields are silently ignored, so a typo in `bindings` empties the config ([src/config/schema.rs:18-23](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L18-L23), [src/config/loader.rs:157-159](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L157-L159)) |
| The file is deleted or renamed | **Known issue** ([KS-23](../audit.md#ks-23)): after 250 ms `load()` does not find the file, writes the default template and **applies it** instead of your bindings; the deleted file reappears with the template's content ([src/hook/config_watcher.rs:53](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L53), [src/config/loader.rs:102](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L102)) |
| The editor saves through "delete + create" or "temporary file + rename" | Usually works: if the events fit into the 250 ms window, the file is back in place by the time it is read |
| The config path is relative (`key-switch-rs.exe config.ron`) | **Known issue** ([KS-21](../audit.md#ks-21)): reload **never** fires. `notify` resolves the relative folder to an absolute one and reports absolute paths, while the comparison is made against the relative path, even though the console prints `✓ Watching config.ron for changes` ([src/hook/config_watcher.rs:47](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L47), [src/main.rs:29](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/main.rs#L29)). Workaround: pass an absolute path |
| The letter case of the given path differs from the name on disk (`CONFIG.RON` versus `config.ron`) | Reload does not fire either: paths are compared case-sensitively ([src/hook/config_watcher.rs:47](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L47); see [KS-21](../audit.md#ks-21)) |
| The watcher could not be started | `[WARN] could not start config file watcher for <path>: <error>` and `[INFO] continuing without hot-reload` are printed; the daemon runs without reload ([src/core/app.rs:112-119](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/core/app.rs#L112-L119)) |

Further points:

- **At startup**, any config error is fatal: `[FATAL] <error>` is printed, then `Press Enter to exit...` ([src/main.rs:11-15](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/main.rs#L11-L15), [src/main.rs:33](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/main.rs#L33)). There is no partial load: one invalid binding rejects the whole file ([src/config/loader.rs:153-155](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L153-L155)).
- `swch` can neither trigger nor check a reload: it has only the commands `on`, `off`, `exit`, `status` and `open`, and no `reload` or `check` command ([src/ipc.rs:37-43](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/ipc.rs#L37-L43)). Errors are visible only in the daemon's console. If the daemon was started with `swch open`, that is a separate, elevated console window.
- A reload does not reset gestures that have already started: armed `Tap`/`Hold`/`DoubleTap` gestures and unfinished `chord`/`sequence` gestures hold a copy of the action and run with the old actions ([src/hook/config_watcher.rs:103](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/config_watcher.rs#L103) calls only `update_bindings`).
- The Windows hotkey value in the registry is re-read on every reload. Changing this setting in Windows does not trigger a reload by itself: re-save `config.ron` or restart the daemon.
- At startup the daemon prints the binding list in file order (with auto-blockers marked `[AUTO-BLOCK] … -> (blocked)`), not in the order in which bindings are checked ([§16](matching.md#s16)). Keys in this list are printed with their canonical names ([§6.4](keys.md#s6-4)). On reload the list is not printed.

<a id="s4"></a>
## 4. Top level of the file

The file contains exactly one value: an unnamed struct with a single field, `bindings`, which holds the list of bindings (`RawConfig`, [src/config/schema.rs:18-24](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L18-L24)):

```ron
(
    bindings: [
        ( keys: "CapsLock", action: SwitchLanguage ),
        // ...
    ],
)
```

- `bindings: []`, `()` (field missing) and `( bindnigs: [...] )` (typo) load **successfully** and give 0 bindings. The only sign is `[WARN] config contains no bindings` ([src/config/schema.rs:22](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L22), [src/config/loader.rs:157-159](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L157-L159)).
- The value of an unknown field must still be valid RON: `bad: )` gives `Unexpected char ')'`.
- After the closing `)` only whitespace and comments are allowed; anything else gives `Non-whitespace trailing characters`.
- An empty file, and a file that holds only comments, give ``Expected opening `(` for struct `RawConfig` ``.
- Without the outer parentheses (`bindings: [...]`) you get ``Expected struct `RawConfig` but found `bindings` ``.
- `bindings` must be a list `[...]`, not a tuple `(...)`; otherwise you get ``Expected opening `[` ``.
- Each list element is a struct `( … )`. A stray comma, or the end of the file, right after `[` gives ``Expected opening `(` for struct `RawBinding` `` ([§3.3](ron-syntax.md#s3-3)).

> [!WARNING]
> Known issue ([KS-22](../audit.md#ks-22)): on hot reload, an empty, missing or misspelled `bindings` silently turns off all remapping ([src/config/schema.rs:18](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L18)).

<a id="s5"></a>
## 5. Binding fields

<a id="s5-1"></a>
### 5.1 Field table

Each element of `bindings` is a struct `( … )` (`RawBinding`, [src/config/schema.rs:26-56](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L26-L56)). Fields can appear in any order.

| Field | Type and RON form | Default | Meaning |
|---|---|---|---|
| `keys` | combo string: `"Ctrl+Shift+A"` | not set | Combo: fires when all of its keys are held at the same time ([§7](kinds.md#s7)). String syntax: [§6.2](keys.md#s6-2) |
| `sequence` | `(steps: ["F13", "Ctrl+F14"], max_gap_ms: 500)` | not set; `max_gap_ms` = 500 | Sequence of steps; each step is a combo string ([§8](kinds.md#s8)) |
| `chord` | `(keys: ["F15", "F16"], window_ms: 50)` | not set; `window_ms` = 50 | Chord: all keys pressed within the window, in any order ([§9](kinds.md#s9)) |
| `action` | a `RawAction` variant: `SwitchLanguage`, `PressKey("L")`, `Launch(exe: "…")`, … | **required** | What to do when the binding fires ([§13](actions.md#s13), [§14](actions.md#s14)) |
| `trigger` | `Immediate`, `Tap(ms)`, `Hold(ms)`, `DoubleTap(ms)` | `Immediate` | When the action fires ([§11](triggers-conditions.md#s11)). Deferred triggers are allowed only together with `keys` |
| `when` | a `RawCondition` variant: `Always`, `TitleContains("…")`, `TitleEquals("…")`, `AppEquals("x.exe")`, `Not(…)`, `And([…])`, `Or([…])` | `Always` | Condition on the active window ([§12](triggers-conditions.md#s12)). If the condition is false, the binding is skipped entirely: it does not fire and suppresses nothing. **Known issue:** `AppEquals` never matches ([KS-03](../audit.md#ks-03)) |
| `block_default` | `true` / `false` | `true` | Whether to hide the pressed key from the OS. Honoured **only** for `keys` with `trigger: Immediate`; in all other cases the key is always hidden ([§10](kinds.md#s10)) |
| `block_original_combo` | `true` / `false` | `false` | Add an auto-blocker for the system hotkey that does the same thing as `action` ([§15](matching.md#s15)). Works only for `SwitchLanguage` and `SwitchLanguageBackward`; for other actions the flag is silently ignored |

Example with all fields:

```ron
(
    keys: "Ctrl+Alt+L",
    action: PressKey("L"),
    block_default: false,
    block_original_combo: false,
    when: TitleContains("Visual Studio Code"),
    trigger: Immediate,
),
```

<a id="s5-2"></a>
### 5.2 Exactly one of `keys` / `sequence` / `chord`

`raw_binding_to_kind` ([src/config/loader.rs:241-272](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L241-L272)) counts how many of the three fields are set (`None` counts as not set):

- none: ``binding has none of `keys` / `sequence` / `chord` set — must specify exactly one``;
- two or three: ``binding has multiple of `keys` / `sequence` / `chord` set — must be exactly one``.

The chosen kind determines which other fields have an effect:

| | `keys` | `sequence` | `chord` |
|---|---|---|---|
| `trigger` | any | only `Immediate` (anything else is an error) | only `Immediate` (anything else is an error) |
| `block_default` | honoured with `Immediate` | ignored: the keys are always hidden | ignored: the keys are always hidden |
| `block_original_combo` | works | works (the auto-blocker is still a combo) | works |
| `when` | works | works | works |

<a id="s5-3"></a>
### 5.3 Unknown fields and typos

> [!WARNING]
> Known issue ([KS-22](../audit.md#ks-22)): unknown fields are ignored **at every level**: at the root, in a binding, inside `sequence: (…)` and `chord: (…)`, and in the fields of the actions `Launch(…)` and `PostMessage(…)`. The `#[serde(deny_unknown_fields)]` attribute appears nowhere in the schema ([src/config/schema.rs:18](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L18), [L26](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L26), [L72](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L72), [L81](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L81), [L129-L143](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L129-L143)).

A typo in an optional field silently leaves the default value in place:

| Written | Result |
|---|---|
| `blok_default: false`, `blockdefault: false` | `block_default` stays `true` |
| `max_gap: 300` | `max_gap_ms` stays 500 |
| `window: 200` | `window_ms` stays 50 |
| `trigerr: Tap(200)` | `trigger` stays `Immediate` |
| `whn: …`, `whne: …` | `when` stays `Always` |
| `argz: [...]`, `cwd: "C:/temp"` in `Launch` | `args` is empty; there is no `cwd` field |
| `lparm: 5`, `wParam: 0xF020` in `PostMessage` | `lparam` = 0, `wparam` = 0 (field names are lowercase) |
| `Keys:`, `Sequence:` (different case) | the field is not recognised |

A typo in a required field does give an error, but the error is about a missing field:

- `acton:` → ``Unexpected missing field named `action` in `RawBinding` ``;
- `step:` in `sequence` → ``Unexpected missing field named `steps` in `SequenceSpec` ``;
- a typo in `keys`/`sequence`/`chord` (or a different case) → ``binding has none of `keys` / `sequence` / `chord` set …``, if the binding has no other kind field.

Repeating the same field (`keys: "F13", keys: "F14"`) gives ``Unexpected duplicate field named `keys` in `RawBinding` ``. If two different kind fields are set (`keys` and `sequence`), the error is `binding has multiple of …`.
