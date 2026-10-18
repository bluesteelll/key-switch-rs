# Error messages

A config that does not load produces one of four kinds of error: the file is missing, it cannot be read, it is not valid RON for the schema, or a binding in it is invalid. At startup each is fatal; on hot reload the previous bindings stay live. Facts describe commit `20af28d`, the tip of `master` after roadmap Phase 1, and every message on this page was produced by the validator, which runs the real loader ([How the audit was done](../methodology.md#validation-harness)), unless the row says otherwise.

<a id="s18"></a>
## 18. Error messages

<a id="s18-1"></a>
### 18.1 Four kinds of error and where they appear

| Kind | Format | When |
|---|---|---|
| File missing | `config file <path> does not exist` | the file, or a folder on its path, does not exist: a command-line path at startup, or any path at a reload ([src/config/loader.rs:184-197](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L184-L197)) |
| File read error | `config I/O error (<path>): <OS message>` | the file cannot be read, or the template cannot be written |
| RON parse error | `config parse error (<path>): <line>:<column>: <message>` | RON syntax or the schema is violated, including an unknown field; parsing stops at the **first** error |
| Binding validation errors | `config has <K> invalid binding(s):` followed by one line `  - binding #<N> [<label>]: <message>` per error | the RON is valid, but bindings are semantically invalid; errors are collected across **all** bindings |

If a file has both a parse error and validation errors, only the parse error is printed. Any error rejects the whole file.

**At startup** the error is fatal ([src/main.rs:47-62](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L47-L62)):
```text
[FATAL] config has 3 invalid binding(s):
  - binding #1 [Ctrl+Foo]: unknown key "Foo" in combo "Ctrl+Foo"
  - binding #2 [<unset>]: binding has none of `keys` / `sequence` / `chord` set — must specify exactly one
  - binding #3 [F14]: PressKey: unknown key "Baz"

Press Enter to exit...
```
The log drops the line break at the end of the validation-error text, and one empty line is printed before `Press Enter to exit...`, so every kind of error has one blank line here. The `[FATAL]` record also goes into `daemon.log` ([Log file](../build-and-run.md#log-file)), where each of its lines starts with the timestamp and `[FATAL]`.

**On hot reload** the same text is logged below `[ERROR] config reload failed:`, followed by `[INFO] keeping previous bindings live` with no blank line between them, and the previous bindings keep working ([src/hook/config_watcher.rs:165-168](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L165-L168)). In `daemon.log` every line of the error starts with the timestamp and `[ERROR]`:
```text
2026-10-16T09:52:30.518Z [ERROR] config reload failed:
2026-10-16T09:52:30.518Z [ERROR] config has 1 invalid binding(s):
2026-10-16T09:52:30.518Z [ERROR]   - binding #0 [F13]: PressKey: unknown key "Foo"
2026-10-16T09:52:30.518Z [INFO] keeping previous bindings live
```

<a id="s18-2"></a>
### 18.2 File missing and file read errors

| Cause | Message |
|---|---|
| A path given on the command line does not exist, or a folder on it does not | `config file <path> does not exist`; nothing is created ([§2.2](structure.md#s2-2)) |
| `config.ron` was deleted or renamed while the daemon runs | `config file <path> does not exist` after `[ERROR] config reload failed:`; the previous bindings stay live |

`config I/O error (<path>): <message>`. The message text comes from Windows and may depend on the system language; the English wordings are shown below.

| Cause | Message |
|---|---|
| The file is not UTF-8: UTF-16, or cp1251 with non-ASCII bytes | `stream did not contain valid UTF-8` |
| The path points to a folder, or access is denied | `Access is denied. (os error 5)` |
| The file is locked by another process (on reload) | `… (os error 32)`; there is no retry ([§2.3](structure.md#s2-3)) |
| `<exe folder>\config.ron` does not exist and its folder could not be created | `config I/O error (<folder>): …` |
| `<exe folder>\config.ron` does not exist and the template could not be written | `config I/O error (<path>): …` |
| Default path: the path to the exe could not be determined | `config I/O error (<current_exe>): …` |
| Default path: the path to the exe has no parent folder | `config error: executable path … has no parent directory` |
| A command-line path that cannot be made absolute | `config error: config path … cannot be used: …` ([src/config/loader.rs:127-136](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L127-L136)) |

The rows about locks, folders, the exe path and paths that cannot be made absolute come from reading the code; the validator cannot produce them.

<a id="s18-3"></a>
### 18.3 RON parse errors

Lines and columns are numbered from 1, and the column is counted in characters (a Cyrillic letter is 1 character). The position marks the place where parsing stopped: for a missing field, the closing `)` of the struct; for an unknown field, the character right after its name; for an invalid escape, the position right after it; for an unclosed quote, the start of the string (the first character after the opening `"`).

| Cause | Message |
|---|---|
| An unknown or misspelled field, at any level | `` Unexpected field named `tigger` in `RawBinding`, expected one of `keys`, … `` (likewise `RawConfig`, `SequenceSpec`, `ChordSpec`, `Launch`, `PostMessage`); the message lists the valid names ([§5.3](structure.md#s5-3)) |
| A missing comma between elements or fields; `;` instead of `,`; parentheses on a variant without data (`DoNothing()`); `Immediate(0)` | `Expected comma` |
| An unclosed bracket (usually) | `Expected comma` or `` Expected opening `(` for struct `RawBinding` `` ([§3.3](ron-syntax.md#s3-3)) |
| The file ends right after `(` or after a dangling comma in a list of steps | `Unexpected end of RON` |
| An extra comma in the bindings list (`[ , ]`, `),,`) | `` Expected opening `(` for struct `RawBinding` `` |
| Text after the root `)`, an extra `)`, an `#![…]` attribute at the end of the file | `Non-whitespace trailing characters` |
| An unclosed quote (with no further `"` in the file) | `Expected end of string` |
| An unclosed block comment | `Unclosed block comment` |
| `#` at the start of the file | ``Expected an `#![enable(...)]` attribute`` |
| An unknown extension in `#![enable(…)]` | `` No RON extension named `foo` `` |
| `#![enable(explicit_struct_names)]` without the struct name | ``Expected the explicit struct name `RawConfig`, but none was found`` |
| A no-break space U+00A0 between tokens | `Expected identifier` (or another message, depending on where it is) |
| An empty file, a file with only comments, `{…}` instead of `(…)` | `` Expected opening `(` for struct `RawConfig` `` |
| No outer parentheses | `` Expected struct `RawConfig` but found `bindings` `` |
| A foreign struct name | `` Expected struct `RawConfig` but found `Config` `` (likewise `RawBinding`, `SequenceSpec`, `ChordSpec`) |
| `sequence`/`chord` written as a list | `` Expected opening `(` for struct `SequenceSpec` `` |
| A required field is missing | `` Unexpected missing field named `action` in `RawBinding` `` (likewise `bindings` in `RawConfig`, `steps` in `SequenceSpec`, `keys` in `ChordSpec`, `exe` in `Launch`, `msg` in `PostMessage`) |
| A field is repeated | `` Unexpected duplicate field named `keys` in `RawBinding` `` |
| An unknown variant, or a case mistake | `` Unexpected variant named `SwitchLang` in enum `RawAction`, expected one of `SwitchLanguage`, … `` (likewise `RawTrigger`, `RawCondition`, `RawStep`, `MediaKeyRef`, `RawWindowKind`) |
| `Some(…)`/`None` in a field that is not an `Option` | `Expected boolean`, `Expected integer`, `` Expected opening `[` ``, ``Unexpected variant named `None` …``, ``Unexpected variant named `Some` …`` ([§3.6](ron-syntax.md#s3-6)) |
| A `block_original_combo` value other than `true`, `false` or `MaskOnly` (`Maskonly`, `yes`, `Some(true)`) | ``Unexpected variant named `Maskonly` in enum `RawBlockOriginalCombo`, expected one of `true`, `false`, or `MaskOnly` instead`` ([§15](matching.md#s15)) |
| A variant that takes an argument written without parentheses; `Launch { … }` | `` Expected opening `(` `` |
| An extra argument (`PressKey("A", "B")`, `Not(a, b)`) | `` Expected closing `)` `` |
| A list instead of a single value, or the reverse (`And(x)`, `args: "x"`, `Sequence(Text("x"))`, `bindings: (…)`) | `` Expected opening `[` `` |
| A string instead of an identifier (`action: "SwitchLanguage"`, `Media("PlayPause")`, `when: "x"`); a positional argument to a variant with fields (`Launch("x")`); `Media()`, `Window()`, `Not([…])` | `Expected identifier` |
| Not a string where a string is required (`Text(123)`, `'a'`, `b"…"`, `keys: [F13]`, `PressKey()`) | `Expected string` |
| Not a number where a number is required (`"500"`, `Tap()`, `Tap(.5)`, `Tap(NaN)`, `Tap(0x)`) | `Expected integer` |
| Not `true`/`false` in a `true`/`false` field | `Expected boolean` |
| A negative number in an unsigned field (except `-0`); a value outside the type's range; a value that does not fit the suffix type (`Tap(300u8)`, `msg: 300u8`) | `Integer is out of bounds` |
| A fractional number as a positional argument (`Tap(50.0)`, `Tap(5.)`); `Tap(200usize)`, `Tap(1_000u128)`; `Tap(0X10)` | `` Expected closing `)` `` |
| A fractional number in a named field (`window_ms: 50.0`); `window_ms: 0X10`; `msg: 16usize` | `Expected comma` |
| Exponential notation (`Hold(1e3)`, `window_ms: 1e3`) | `Invalid digit 'e' for base 10 integers` |
| An `f32`/`f64` suffix | `Invalid digit 'f' for base 10 integers` |
| A suffix that does not match the field type (`Tap(200u16)`, `Tap(200i32)`, `lparam: -5i8`) | `Expected an 64-bit unsigned integer but found 200u16 instead` / `Expected an 64-bit signed integer but found -5i8 instead` (the message quotes the literal as written) |
| A leading underscore (`_200`, `0x_10`) | `Unexpected leading underscore in a number` |
| An invalid `msg` in `PostMessage` (negative, fractional, larger than `u32`, a name without quotes, `'W'`, `true`, `Some(16)`, `(16)`, `[16]`) | `data did not match any variant of untagged enum MessageRef` |
| An invalid `\` escape | `Unknown escape character` |
| `\u0041` without braces | `Missing { in Unicode escape` |
| `\u{}` / `\u{1234567}` | `Expected 1-6 digits, got 0 digits in Unicode escape` / `No } at the end of Unicode escape` |
| `\u{D83D}`, `\u{110000}` | `Not a valid Unicode-escaped character` |
| `\x80`, `\xFF` (the bytes do not form UTF-8) | `Not a valid byte-escaped Unicode character` |
| `\x4` (one digit) | `Non-hex digit found` |
| Nesting too deep (61 `Not(`, 31 `And([`) | ``Exceeded recursion limit, try increasing `ron::Options::recursion_limit` and using `serde_stacker` to protect against a stack overflow`` |

A UTF-8 byte-order mark at the start of the file is no longer an error ([KS-24](../audit.md#ks-24), fixed in `1dfd80a`). Most rows of this table were produced by the audit's validator at `7cb3521`. The parser has not changed since (`ron` 0.10.1, the same `Cargo.lock` entry), and the rows about unknown, missing and repeated fields were checked again at `1dfd80a`.

<a id="s18-4"></a>
### 18.4 Internal type names in messages

| Name in the message | What it is in the file |
|---|---|
| `RawConfig` | the root struct |
| `RawBinding` | an element of the `bindings` list |
| `SequenceSpec` | the value of `sequence: (…)` |
| `ChordSpec` | the value of `chord: (…)` |
| `RawAction` | the value of `action` |
| `RawTrigger` | the value of `trigger` |
| `RawCondition` | the value of `when` |
| `RawBlockOriginalCombo` | the value of `block_original_combo` |
| `RawStep` | a step inside `Sequence([…])` |
| `MediaKeyRef` | the argument of `Media(…)` |
| `RawWindowKind` | the argument of `Window(…)` |
| `MessageRef` | the value of `msg` in `PostMessage` |
| `Launch`, `PostMessage` | the fields of these actions |

<a id="s18-5"></a>
### 18.5 Binding validation errors

Code: [src/config/loader.rs:272-284](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L272-L284), [src/config/loader.rs:451-634](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L451-L634), [src/config/loader.rs:652-759](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L652-L759).

```text
config has <K> invalid binding(s):
  - binding #<N> [<label>]: <message>
```
- **All** bindings are checked, and the errors are collected into one list. For each binding only the **first** error is printed, in this order: the kind (`keys`/`sequence`/`chord`, including the step count, the chord keys and the `max_gap_ms` or `window_ms` range), the action, the trigger and its range, a deferred trigger on `sequence`/`chord`, `allow_extra_modifiers` on `sequence`/`chord`, and `repeat: true` outside `keys` + `Immediate` ([src/config/loader.rs:451-511](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L451-L511)). Within a kind, a combo or a macro, checking stops at the first bad segment or step. For example, `( keys: "Foo", action: PressKey("Bar") )` gives only `unknown key "Foo" in combo "Foo"`.
- `N` is the index in the `bindings` list, **counting from 0**, in file order. Commented-out bindings and auto-blockers are not counted. The numbers in `step #M`, `Sequence step #M` and `Chord key #M` also count from 0.
- `<label>` is the value of `keys` as written (spaces included), `Sequence[F13 → F14]` for a sequence, `Chord[F15+F16]` for a chord, or `<unset>`. If several kind fields are set, the first of `keys`, `sequence`, `chord` is used. A chord label is built by simply joining the original strings with `+` ([src/config/loader.rs:589-600](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L589-L600)), so with empty or `+` names it is confusing: `["", "k"]` → `[Chord[+k]]`, `["+", "k"]` → `[Chord[++k]]`, `keys: []` → `[Chord[]]`.
- Strings after the colon are printed in Rust's Debug format: in quotes, with `\` as `\\`, `"` as `\"`, a tab as `\t` and U+00A0 as `\u{a0}`. For example, `keys: "Ctrl+\\+Foo"` gives `binding #0 [Ctrl+\+Foo]: unknown key "Foo" in combo "Ctrl+\\+Foo"`.

| # | Rule | Message after `binding #N [...]: ` |
|---|---|---|
| 1 | Exactly one of `keys`/`sequence`/`chord` is set | ``binding has none of `keys` / `sequence` / `chord` set — must specify exactly one`` / ``binding has multiple of `keys` / `sequence` / `chord` set — must be exactly one`` |
| 2 | `keys` is not an empty string and not whitespace only | `combo is empty` |
| 3 | No empty segments (`Ctrl++A`, `+A`, `A+`, `+`, `Ctrl+ +A`) | `empty segment in combo "Ctrl++A" (stray '+'?)` |
| 4 | Every key name in `keys` is known | `unknown key "Foo" in combo "Ctrl+Foo"` |
| 5 | `sequence` has at least 2 steps | `` Sequence must have at least 2 steps; for a single combo use plain `keys: "..."` `` |
| 6 | Every `sequence` step is a valid combo | `Sequence step #1: unknown key "Foo" in combo "Ctrl+Foo"`, `Sequence step #0: combo is empty`, `Sequence step #1: empty segment in combo "Ctrl++S" (stray '+'?)` |
| 7 | `max_gap_ms` and `window_ms` are 1 to 10000 | `max_gap_ms: 0 is out of range: timings are 1 to 10000 ms`, `window_ms: 20000 is out of range: timings are 1 to 10000 ms` |
| 8 | `chord` has at least 2 keys | `Chord must have at least 2 keys; a single key is not a chord` |
| 9 | Every chord key is a known single name (`"Ctrl+A"`, `""`, `"+"` are errors) | `Chord key #1: unknown key name "Foo"` |
| 10 | No two chord keys overlap (repeats, synonyms, and a sided key with the unsided one of its family: `"F13"`/`"f13"`, `"Ctrl"`/`"Control"`, `"Shift"`/`"LShift"`, `"Win"`/`"LWin"`) | `Chord key #1: duplicate "LShift" (overlaps key #0)` |
| 11 | `PressKey`: the key is known | `PressKey: unknown key "Foo"` |
| 12 | `PostMessage`: the `WM_*` name is known | `PostMessage: unknown WM_* name "WM_FOO" (use a numeric code if it is not in the built-in table)` |
| 13 | `Sequence([...])`: the key in a `Key` step is known | `step #2: Key: unknown key "Foo"` |
| 14 | `Sequence([...])`: a `Combo` step is a valid combo | `step #0: Combo "": combo is empty`, `step #0: Combo "Ctrl+": empty segment in combo "Ctrl+" (stray '+'?)`, `step #2: Combo "Ctrl+Foo": unknown key "Foo" in combo "Ctrl+Foo"` |
| 15 | A `Tap`, `Hold` or `DoubleTap` term is 1 to 10000 | `Tap(0) is out of range: timings are 1 to 10000 ms` |
| 16 | A deferred trigger only together with `keys` | ``Sequence/Chord cannot use a deferred trigger (Tap(200)); only `Immediate` (the default) is valid for these kinds`` |
| 17 | `allow_extra_modifiers: true` only together with `keys` | `` `allow_extra_modifiers` applies only to `keys` bindings `` |
| 18 | `repeat: true` only together with `keys` and `Immediate` | `` `repeat: true` applies only to `keys` bindings with the `Immediate` trigger `` |

> [!NOTE]
> The code also has a `Combo {:?}: empty` branch ([src/config/loader.rs:737-739](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L737-L739)), but it is unreachable: the combo parser never returns an empty list without an error, so a user never sees this message.

<a id="s18-6"></a>
### 18.6 Messages at load time and at run time

These messages do not reject the file. The loader logs its messages to the daemon console and to `daemon.log` ([Log file](../build-and-run.md#log-file)); the others appear there when actions are executed or the log file fails.

| Message | Cause |
|---|---|
| `[INFO] generated default config at <path>` | `<exe folder>\config.ron` did not exist at startup, so the template was written ([§2.2](structure.md#s2-2)) |
| `[INFO] loading config from <path>` | the file was read (at startup and on every reload) |
| `[WARN] config contains no bindings` | `bindings: []` ([§4](structure.md#s4)) |
| `[WARN] binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them` | the later binding repeats an earlier one: the same kind and keys (in any order for `keys` and `chord`, in order for `sequence` steps), the same trigger whatever its term, the same `when` as written and, for `keys`, the same `allow_extra_modifiers`; the action does not count ([src/config/loader.rs:297-319](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L297-L319)). Of two such `keys` bindings only the first acts ([§11.7](triggers-conditions.md#s11-7), [§16.3](matching.md#s16-3)). If the one you remove sets `block_original_combo: true`, move the flag to the one you keep, or its auto-blocker goes ([§15](matching.md#s15)). Printed at startup and on every reload |
| `[INFO] SwitchLanguage: registry combination Alt+Shift` / `[INFO] SwitchLanguage: using default combination` / `[INFO] SwitchLanguage: hotkey not assigned in Windows, no auto-blocker` | a `block_original_combo` binding asked for its hotkey ([§15.2](matching.md#s15-2)). The last one: the Windows hotkey is set to Not assigned (`"3"`), so no blocker is added |
| `[INFO] no auto-blocker for Alt+Shift: binding #1 uses the same keys` | a binding of the file uses the hotkey's keys, so no blocker was added ([§15.3](matching.md#s15-3)) |
| ``[WARN] binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true`` | `MaskOnly` on a binding whose Windows hotkey is not made only of modifiers (the `` ` `` hotkey, registry value `"4"`): the blocker swallows the hotkey as with `true` ([src/config/loader.rs:346-362](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L346-L362), [§15.5](matching.md#s15-5)). Not printed when a `keys` binding of the file takes the hotkey over (KS-18): then there is no blocker and no warning. Printed at startup and on every reload |
| `[WARN] registry value "Hotkey" has type …, expected REG_SZ/REG_EXPAND_SZ` | the registry value is not of a string type |
| `[INFO] config reloaded (N bindings live)` | a reload was applied |
| `[ERROR] config reload failed:` + `[INFO] keeping previous bindings live` | a reload was rejected ([§18.1](#s18-1)) |
| `[WARN] could not start config file watcher for <path>: …` + `[INFO] continuing without hot-reload` | the watcher did not start ([§2.3](structure.md#s2-3)) |
| `[ERROR] Language switch error: …` / `[ERROR] Language switch backward error: …` | `WM_INPUTLANGCHANGEREQUEST` could not be posted |
| `[ERROR] Post message error 0x10: …` | `PostMessageW` returned an error (for example, UIPI in a debug build) |
| `[ERROR] Launch "…": program not found` | the program was not found ([§13.6](actions.md#s13-6)) |
| `[ERROR] Launch "": program path has no file name` | `Launch(exe: "")` |
| `[ERROR] Launch "…": batch file arguments are invalid` | arguments to a `.bat`/`.cmd` file that cannot be passed to cmd safely |
| `[ERROR] OpenUrl "…": ShellExecuteW returned error code N` | `ShellExecuteW` returned a code ≤ 32 |
| `[WARN] counters since the last report: menu_mask_short=N key_press_short=N hold_timer_not_started=N log_lines_dropped=N` | every 60 s and at exit, while a count is not 0 ([src/logging/counters.rs](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/logging/counters.rs)). The keyboard hook counts these instead of logging them per event: `menu_mask_short`, the menu mask key ([§13.1](actions.md#s13-1)) was not inserted whole, for example because another thread blocked input (a mask that UIPI keeps from an elevated window is lost without being counted); `key_press_short`, a `PressKey`, `Media` or `ToggleCapsLock` press was not sent whole; `hold_timer_not_started`, the `hold-timer` thread of a `Hold` could not be started, so that Hold did not fire ([§11.4](triggers-conditions.md#s11-4)); `log_lines_dropped`, log lines were lost to a full queue |
| `[WARN] type_text: SendInput sent N of M inputs` / `[WARN] press_key: …` / `[WARN] press_combo: …` | `Text`, `Key`, `Combo` steps: some events were not sent |
| `[WARN] window step skipped: no foreground HWND captured` | a `Window(…)` step ran with no foreground window |
| `[WARN] window close PostMessage failed: …` | a `Window(Close)` step could not post `WM_CLOSE` |
| `[WARN] window step failed: ShowWindowAsync did not start Minimize` (or `Maximize`, `Restore`) | Windows did not accept the change of a `Window(…)` step, for example because the window was closed meanwhile ([§14.2](actions.md#s14-2)) |
| `[WARN] no log file: the LocalAppData folder is unknown: …` | at startup the shell did not return `%LOCALAPPDATA%`; the daemon logs to the console only |
| `[WARN] log file <path>: could not write: …` / `[WARN] log file <path>: could not rotate, so daemon.log grows past its limit: …` | on the console only, once per run of failures: `daemon.log` or its folder cannot be written, or another program holds a log file open without delete sharing. Every later line tries again |
| `[INFO] log file <path> works again` | the first line written after such a run of failures |

The run-time messages come from reading the code; the validator shows only the load-time ones.

<a id="s18-7"></a>
### 18.7 What is not validated

The following load without errors or warnings, although they are often mistakes in intent:
- `Delay(0)` and any `Delay` up to `u64::MAX` (the 1 to 10000 ms range covers triggers, `max_gap_ms` and `window_ms` only);
- the numeric `msg` code in `PostMessage` (any `u32` is accepted);
- `Launch(exe: "")`, `OpenUrl("")`, `Sequence([])`, `Text("")`, `And([])`, `Or([])`, `TitleContains("")`;
- an `Immediate` binding and a `Tap`, `Hold` or `DoubleTap` binding on the same keys and `when`, of which only the higher one ever acts ([§19](known-issues.md#s19) row 69); exact duplicates are warned about since `feat/one-winner` ([§18.6](#s18-6));
- `block_original_combo` on actions that have no system combo, and on `sequence`/`chord` (the blocker is added);
- `block_default` where it is ignored (`sequence`, `chord`);
- strings with accidentally valid escapes such as `"C:\temp"`.

Unknown fields and timing values out of range used to load silently; since `1dfd80a` they are errors ([KS-22](../audit.md#ks-22), [KS-27](../audit.md#ks-27)).
