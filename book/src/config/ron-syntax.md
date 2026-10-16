# RON syntax essentials

<a id="s3"></a>
## 3. RON syntax essentials

The file is parsed by `ron 0.10.1` with one extension enabled, `IMPLICIT_SOME` ([src/config/loader.rs:242-243](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L242-L243)). The whole file is read at once with `fs::read_to_string` ([src/config/loader.rs:187-197](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L187-L197)). Facts describe commit `1dfd80a`, and those about `Some` and `None` in `repeat` and `block_default` the branches `feat/repeat-flag` and `feat/deferred-block-default`. Parsing stops at the first error; the message format is described in [§18](errors.md#s18).

<a id="s3-1"></a>
### 3.1 Encoding, whitespace and line breaks

**UTF-8 only, with or without a BOM.**
- A file in UTF-16 ("Unicode" in the old Notepad) gives `config I/O error (<path>): stream did not contain valid UTF-8`.
- A file in cp1251 gives the same error only if it contains non-ASCII bytes (for example, Cyrillic in comments). A cp1251 file made only of ASCII characters is byte-for-byte identical to UTF-8 and loads normally.

> [!NOTE]
> A byte-order mark at the start of the file is skipped ([src/config/loader.rs:233-236](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L233-L236)), so Notepad's "UTF-8 with BOM" and `Set-Content -Encoding UTF8` in Windows PowerShell 5.1 work. Until `1dfd80a` such a file failed with ``1:1: Expected opening `(` for struct `RawConfig` `` ([KS-24](../audit.md#ks-24), fixed). Only the leading mark is skipped: a U+FEFF inside a string stays part of the text.

**Whitespace.** Any number of spaces, tabs and line breaks may appear between tokens. The exception is the **non-breaking space U+00A0**: RON does not treat it as whitespace. The one-line file `( bindings: [ ( keys: "F13",<NBSP>action: DoNothing ) ] )` gives `1:29: Expected identifier`. This is a common trap when copying examples from a browser or from Word: in an editor the character looks like an ordinary space. U+2028 (line separator) between tokens, on the other hand, is accepted. **Inside strings** that hold key names and `WM_*` names, U+00A0 and all other whitespace characters are silently removed ([§6.3](keys.md#s6-3), [§13.4](actions.md#s13-4)), so `"Ctrl+<NBSP>F13"` loads as `Ctrl+F13`.

**Line breaks.** LF and CRLF behave the same. The difference shows only inside string literals, when the line break is written directly inside the string ([§3.9](#s3-9)).

<a id="s3-2"></a>
### 3.2 Comments

- `// to the end of the line` and `/* block */`. Block comments can nest: `/* a /* b */ c */`.
- An unclosed block comment (`/* …` running to the end of the file, and also `/* a /* b */`, where only the inner comment is closed) gives `Unclosed block comment`.
- The characters `#` and `;` do not start a comment. A line `# ...` at the start of the file gives ``Expected an `#![enable(...)]` attribute`` (elsewhere in the file, a different parse error), and `;` instead of a comma gives `Expected comma`.
- Comments are allowed anywhere between tokens, including inside step lists and between fields.

<a id="s3-3"></a>
### 3.3 Structs, lists, variants; commas and brackets

| Construct | Syntax | Example |
|---|---|---|
| Struct | `( field: value, ... )` | `( keys: "F13", action: DoNothing )` |
| List | `[ a, b, ... ]` | `steps: ["F13", "F14"]` |
| Variant without data | `Name` | `SwitchLanguage`, `Always`, `Immediate` |
| Variant with an argument | `Name(value)` | `PressKey("L")`, `Tap(200)`, `Not(TitleContains("x"))` |
| Variant with fields | `Name(field: value, ...)` | `PostMessage(msg: "WM_CLOSE")`, `Launch(exe: "wt.exe")` |
| String | `"..."`, `r"..."`, `r#"..."#` | [§3.9](#s3-9) |
| Integer | `500`, `1_500`, `0x1F4`, `0o764`, `0b101`, `+200` | [§3.7](#s3-7) |
| Boolean | `true` / `false` | `block_default: false` |

- **Commas.** A comma between elements is required; without it you get `Expected comma`. A trailing comma after the last list element, the last struct field or the last argument (`PressKey("L",)`, `Tap(200,)`, `Launch(exe: "x.exe",)`) is allowed everywhere but not required. Two commas in a row, or a comma with no element, are not allowed: in the binding list, `( bindings: [ , ] )` and `( … ),, ] )` give ``Expected opening `(` for struct `RawBinding` ``.
- **Field order** within a struct is free. A repeated field is an error: ``Unexpected duplicate field named `keys` in `RawBinding` `` (likewise inside `sequence`: ``… named `steps` in `SequenceSpec` ``, and in actions: ``… named `msg` in `PostMessage` ``).
- A **variant without data** is written without parentheses: `DoNothing()` gives `Expected comma`. A variant with an argument written without parentheses, for example `trigger: Tap`, gives ``Expected opening `(` ``; empty parentheses `Tap()` give `Expected integer`.
- **Curly braces** `{ }` are used neither for structs nor for variants with fields: `{ bindings: [...] }` gives ``Expected opening `(` for struct `RawConfig` ``, and `Launch { exe: "x.exe" }` gives ``Expected opening `(` ``.
- **Unclosed brackets.** The message depends on where the file ends, and `Unexpected end of RON` is rare:

  | File | Message |
  |---|---|
  | `( bindings: [ ( keys: "F13", action: DoNothing ) ]` (the final `)` is missing) | `1:51: Expected comma` |
  | `… PressKey("A" ) ] )` (the binding's parenthesis is not closed) | `Expected comma` |
  | `( bindings: [` (the file ends after `[`) | ``Expected opening `(` for struct `RawBinding` `` |
  | `(` (the file ends right after the opening parenthesis) | `Unexpected end of RON` |
  | `… Sequence([Text("a"), ` (the file ends after a dangling comma) | `Unexpected end of RON` |
  | `( bindings: [] ))` (an extra `)`) | `Non-whitespace trailing characters` |

<a id="s3-4"></a>
### 3.4 Case, struct names, raw identifiers

- Field and variant names are **case-sensitive**. `action: switchlanguage` gives ``Unexpected variant named `switchlanguage` in enum `RawAction`, expected one of …``. `Keys:` instead of `keys:` is an unknown field and a load error: ``Unexpected field named `Keys` in `RawBinding`, expected one of `keys`, …`` ([§5.3](structure.md#s5-3)). Key names **inside strings** are case-insensitive: `"ctrl+a"` = `"Ctrl+A"`.
- A struct's internal name may be written before its parentheses: `RawConfig( bindings: [ RawBinding( ... ) ] )`, `sequence: SequenceSpec(...)`, `chord: ChordSpec(...)`. This is not needed, but it is allowed. Any other name is an error: ``Expected struct `RawConfig` but found `Config` `` (for a binding, ``Expected struct `RawBinding` but found `Binding` ``; for a sequence, ``Expected struct `SequenceSpec` but found `Foo` ``).
- RON raw identifiers are allowed: `r#keys: "F13"` and `action: r#DoNothing` load the same as the plain forms.

<a id="s3-5"></a>
### 3.5 The `#![enable(...)]` attribute

At the very start of the file, before the root `(`, you can put a ron attribute that enables syntax extensions. It applies to the whole file:

| Attribute | Effect in this file |
|---|---|
| `#![enable(implicit_some)]` | Changes nothing: the loader already enables this extension |
| `#![enable(unwrap_newtypes)]`, `#![enable(unwrap_variant_newtypes)]` | Accepted, but have no effect on this schema |
| `#![enable(explicit_struct_names)]` | Makes the names of **all** structs mandatory. A plain `( bindings: … )` gives ``1:35: Expected the explicit struct name `RawConfig`, but none was found`` (with the attribute and the root `(` on one line, separated by a space), and `sequence: (…)` gives ``… `SequenceSpec` …``. A file in the form `RawConfig( bindings: [ RawBinding( …, sequence: SequenceSpec(…) ) ] )` passes. The requirement does not extend to variants with fields (`Launch(exe: …)`, `PostMessage(…)`) |
| `#![enable(foo)]` (unknown name) | ``No RON extension named `foo` `` |

Several extensions are separated by commas: `#![enable(implicit_some, unwrap_newtypes)]`. The attribute is allowed only at the start of the file: after the root `)` it gives `Non-whitespace trailing characters`. The attributes have no practical use in `config.ron`.

<a id="s3-6"></a>
### 3.6 `IMPLICIT_SOME`: `Some` and `None`

The fields `keys`, `sequence`, `chord`, `repeat` and `block_default` have the type `Option<…>`. Thanks to the extension, you do not need to wrap the value in `Some(...)`:
```ron
( keys: "F13", action: DoNothing ),                                   // usual form
( keys: Some("F14"), action: DoNothing ),                             // also allowed
( keys: None, sequence: (steps: ["F15", "F15"]), action: DoNothing ), // None = field not set
( keys: "F16", action: DoNothing, block_default: Some(false) ),      // same as block_default: false
( keys: "F17", action: DoNothing, block_default: None ),             // None = the default
```
`None` in `repeat` or `block_default` means the field's default, which depends on the binding ([§5.1](structure.md#s5-1)). The other fields are not of an `Option` type, so `Some(...)` and `None` are not allowed in them:
- `bindings: None` and `bindings: Some([])` give ``Expected opening `[` ``;
- `block_original_combo: Some(true)` gives `Expected boolean`;
- `max_gap_ms: Some(300)` gives `Expected integer`;
- `when: None` gives ``Unexpected variant named `None` in enum `RawCondition` ``, and `action: None` gives the same for `RawAction`;
- `trigger: Some(Tap(200))` gives ``Unexpected variant named `Some` in enum `RawTrigger` ``.

<a id="s3-7"></a>
### 3.7 Integers

Types of the numeric fields:

| Field | Type |
|---|---|
| `max_gap_ms`, `window_ms`, the arguments of `Tap`/`Hold`/`DoubleTap`, `Delay(…)`, `wparam` | `u64` (0 … 18446744073709551615) |
| `lparam` in `PostMessage` | `i64` (allows a minus sign) |
| a numeric `msg` in `PostMessage` | `u32` (0 … 4294967295), parsed through the untagged enum `MessageRef` ([§13.4](actions.md#s13-4)) |

Accepted forms:
- decimal `200`; `016` reads as decimal `16` (a leading zero does not mean octal);
- hexadecimal `0xC8`, binary `0b11001000`, octal `0o310`. The prefixes must be **lowercase**: `0X10` is a parse error. Hex digits may be in either case;
- a `+` sign is allowed on any integer field: `+200`. A minus sign is allowed only on `lparam` (including `-0x10`), with one exception: `-0` is accepted in unsigned fields too (`Tap(-0)` loads as `Tap(0)`, and `msg: -0` as 0). Any other negative value in an unsigned field gives `Integer is out of bounds` (for `msg`, the `MessageRef` error);
- underscores are allowed after the first digit, in any number and in any position: `1_000`, `0x0001_0000`, and also `200_` and `2__00` (both = 200). Only a leading underscore is rejected: `_200` and `0x_10` give `Unexpected leading underscore in a number`;
- a Rust-style type suffix is allowed, but in `u64`/`i64` fields it must match the field type **exactly**:
  - `Tap(200u64)`, `Delay(5u64)`, `wparam: 5u64` and `lparam: -5i64` are accepted;
  - `Tap(200u16)` gives `Expected an 64-bit unsigned integer but found 200u16 instead` (and likewise `Tap(200i32)`), and `lparam: -5i8` gives `Expected an 64-bit signed integer but found -5i8 instead`;
  - if the number does not fit the suffix's type, the error is different: `Tap(300u8)` → `Integer is out of bounds`;
  - the suffixes `usize`/`isize` and `u128`/`i128` are not supported: `Tap(200usize)` and `Tap(1_000u128)` → ``Expected closing `)` ``;
  - the `msg` field is parsed "by content" ([§13.4](actions.md#s13-4)), so any suffix from `u8` to `i64` works there as long as the value fits in `u32`: `msg: 16u8`, `msg: 16u32` and `msg: 16i64` all give 16.

Rejected forms:

| Form | Error |
|---|---|
| the value does not fit the type (`Tap(18446744073709551616)`, `Tap(-5)`) | `Integer is out of bounds` |
| a string instead of a number (`max_gap_ms: "500"`, `Hold("300")`) | `Expected integer` |
| empty parentheses `Tap()`, `Delay()`; `Tap(0x)`; `Tap(.5)`, `Tap(NaN)`, `Tap(inf)` | `Expected integer` |
| a fractional number as a positional argument: `Tap(50.0)`, `Delay(1.5)`, `Tap(5.)` | ``Expected closing `)` `` |
| a fractional number in a named field: `window_ms: 50.0`, `wparam: 1.0` | `Expected comma` |
| exponent notation: `Hold(1e3)`, `window_ms: 1e3` | `Invalid digit 'e' for base 10 integers` |
| the `f32` suffix: `Tap(200f32)` | `Invalid digit 'f' for base 10 integers` |
| an uppercase prefix: `window_ms: 0X10` / `Tap(0X10)` | `Expected comma` / ``Expected closing `)` `` |
| any invalid value in `msg` (`-1`, `16.0`, `1e3`, `0x1_0000_0000`) | `data did not match any variant of untagged enum MessageRef` |

`true`/`false` cannot be replaced with `1`/`0` or with the string `"true"`: `Expected boolean`.

<a id="s3-8"></a>
### 3.8 Time values

All times are given as an integer number of milliseconds. The `Tap`, `Hold` and `DoubleTap` terms, `max_gap_ms` and `window_ms` must be 1 to 10000; another value is a load error that names the binding ([src/config/loader.rs:34-37](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L34-L37); [KS-27](../audit.md#ks-27), fixed in `1dfd80a`). `Delay(…)` in a macro is not checked: `Delay(0)` and any value up to `u64::MAX` are accepted. Each field is described in ([§8](kinds.md#s8), [§9](kinds.md#s9), [§11](triggers-conditions.md#s11), [§14](actions.md#s14)).

<a id="s3-9"></a>
### 3.9 Strings and escapes

Strings are written in double quotes. Cyrillic, emoji and any other characters can be written as is. Escapes work in an ordinary string:

| Form | Meaning |
|---|---|
| `\\` | backslash |
| `\"` | double quote |
| `\'` | apostrophe |
| `\n`, `\r`, `\t` | LF, CR, TAB |
| `\0` | null character |
| `\x41` | one byte, exactly two hex digits (in either case). `\x00`–`\x7F` give an ASCII character. Several consecutive `\xHH` that together form valid UTF-8 combine into one character: `"\xD0\x9F"` (and `"\xd0\x9f"`) → `П`, `"\xC3\xA9"` → `é` |
| `\u{1F680}` | a Unicode character by code point, 1 to 6 hex digits; the braces are required |

Escape errors:

| Form | Error |
|---|---|
| `\x80` or `\xFF` with no continuation (the bytes do not form UTF-8) | `Not a valid byte-escaped Unicode character` |
| `\x4` (one digit) | `Non-hex digit found` |
| `\u0041` (no braces) | `Missing { in Unicode escape` |
| `\u{}` | `Expected 1-6 digits, got 0 digits in Unicode escape` |
| `\u{1234567}` | `No } at the end of Unicode escape` |
| `\u{D83D}` (a surrogate), `\u{110000}` | `Not a valid Unicode-escaped character` |
| `\a`, `\b`, `\f`, `\/`, `\` before a line break, `\` before any other character | `Unknown escape character` |

Other forms:
- `'a'` (single quotes) gives `Expected string`.
- A byte string `b"..."` in string fields gives `Expected string`. The only exception is the `msg` field in `PostMessage` ([§13.4](actions.md#s13-4)).
- A line break written directly inside `"..."` is allowed and becomes part of the string; if the file is saved with CRLF, it will be `\r\n`.
- An unclosed quote usually gives `Expected end of string`, but only if there is no other `"` anywhere later in the file. Otherwise the string "swallows" the text up to the next quote, and the error is a different one that points further down the file ([§6.4](keys.md#s6-4), the OEM key `\`).

**Raw strings** work: escapes are not processed in them, and `\` is written once.
- `r"C:\Windows"`: such a string cannot contain the `"` character;
- `r#"C:\path with "quotes"\a.txt"#`: when the string contains `"`;
- `r##"..."##`: when the string contains `"#`.

<a id="s3-10"></a>
### 3.10 Windows paths

In an ordinary string every backslash must be written twice:
```ron
action: Launch(exe: "C:\\Windows\\System32\\notepad.exe", args: ["D:\\proj dir\\file.txt"]),
```
- A single `\` before a character that has no escape is a parse error: `"C:\Windows\notepad.exe"` gives `Unknown escape character`, and the position points right after `\W`.
- More dangerous is a single `\` that happens to form a valid escape: `"C:\temp\new.exe"` parses **without an error** and turns into `C:<TAB>emp<LF>ew.exe`. The loader does not notice this.

Raw strings are safer; in them `\` is written once:
```ron
action: Launch(exe: r"C:\Windows\System32\notepad.exe", args: [r"D:\"]),
action: OpenUrl(r#"C:\path with "quotes"\a.txt"#),   // r#"..."# if the string contains "
```
The loader accepts paths with forward slashes (`"C:/Windows/System32/notepad.exe"`) as ordinary text, and Windows understands them.

<a id="s3-11"></a>
### 3.11 Nesting limit

ron has a recursion limit of 128 nesting levels. The root, the `bindings` list, the binding and each level of a condition consume this budget. In practice 60 nested `Not(` pass (61 no longer do), and 30 nested `And([` pass (31 no longer do). When the limit is exceeded:
```text
Exceeded recursion limit, try increasing `ron::Options::recursion_limit` and using `serde_stacker` to protect against a stack overflow
```
The limit cannot be configured from the config. Real configs never come close to it.
