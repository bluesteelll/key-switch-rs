# Key names and combo syntax

A trigger key is written by name, case and spaces ignored: `"CapsLock"`, `"Ctrl+Alt+L"`, `"LShift"`. `Shift`, `Ctrl`, `Alt` and `Win` mean either key of the pair, and `LShift`, `RWin` and the other sided names mean that key only, in `keys`, `sequence` steps and `chord` keys alike.

<a id="s6"></a>
## 6. Key names and combo strings

Main files: [src/config/parsing.rs](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/parsing.rs), [src/config/loader.rs](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/loader.rs), [src/data/vk_name.rs](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs), [src/data/key_spec.rs](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/key_spec.rs), [src/engine/matching.rs](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/engine/matching.rs). Facts describe commit `bf549c0`, the tip of `master` after roadmap Phase 2. Key names and codes were checked with the validator, which runs the real loader ([How the audit was done](../methodology.md#validation-harness)). The codes that Windows sends were cross-checked against the layout tables of `kbdus.dll` and `kbdru.dll` (through `KbdLayerDescriptor`) during the audit, but were not tested on a live daemon.

<a id="s6-1"></a>
### 6.1 Where key names appear in the config

| Place | What is written | Parsed by | Code |
|---|---|---|---|
| `keys: "..."` | a combo string | `parse_combo`, into trigger keys | [src/config/loader.rs:545-547](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/loader.rs#L545-L547) |
| `sequence: (steps: [...])` | each step is a combo string | `parse_combo` for each step | [src/config/loader.rs:568-571](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/loader.rs#L568-L571) |
| `chord: (keys: [...])` | each element is **one** name, without `+` | `KeySpec::parse`; overlapping keys rejected | [src/config/loader.rs:583-596](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/loader.rs#L583-L596) |
| `action: PressKey("...")` | one name | `parse_vk`, into a key to send | [src/config/loader.rs:674-677](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/loader.rs#L674-L677) |
| step `Key("...")` in `Sequence([...])` | one name | `parse_vk` | [src/config/loader.rs:745-748](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/loader.rs#L745-L748) |
| step `Combo("...")` in `Sequence([...])` | a combo string | `parse_key_list`, into keys to send | [src/config/loader.rs:750-756](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/loader.rs#L750-L756) |

Trigger keys (`keys`, steps, chord keys) and keys the daemon sends (`PressKey`, `Key`, `Combo`) use the same names but are stored differently. A trigger key is a `KeySpec`: either one key on its side, or a modifier on either side ([src/data/key_spec.rs:116-129](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/key_spec.rs#L116-L129)). A key to send is a single virtual-key code ([§6.8](#s6-8)).

`Media(...)` (both the action and the step) does not take a key name. It takes a separate RON variant, written without quotes: `PlayPause`, `Stop`, `Next`, `Previous`, `VolumeUp`, `VolumeDown`, `VolumeMute` ([src/config/schema.rs:207-219](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/schema.rs#L207-L219)). That is why `Media(Mute)` is a RON parse error, while `PressKey("Mute")` is valid (`VK_VOLUME_MUTE`, 0xAD).

Case does not matter only inside strings that hold key names. RON identifiers (actions, `Media` variants, steps, triggers) are case-sensitive and are written without quotes: `Media(playpause)`, `pressKey("A")` and `Media("PlayPause")` are parse errors.

<a id="s6-2"></a>
### 6.2 Combo string grammar

```text
combo   := segment ( "+" segment )*
segment := [whitespace] key-name [whitespace]
```

`parse_combo` and `parse_key_list` share one splitter ([src/config/parsing.rs:15-52](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/parsing.rs#L15-L52)). It works as follows:

1. The string is trimmed at both ends. If it is empty or consists only of whitespace, the error is `combo is empty` ([lines 36-39](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/parsing.rs#L36-L39)).
2. The string is split at **every** `+` character ([line 42](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/parsing.rs#L42)). There is no escaping.
3. Each segment is trimmed at both ends. An empty segment gives the error `empty segment in combo "<string>" (stray '+'?)` ([lines 43-46](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/parsing.rs#L43-L46)). This rejects `"Ctrl++A"`, `"+A"`, `"A+"`, `"+"` and `"Ctrl+ +A"`.
4. Each segment is parsed as a key name. If the name is unknown, the error is `unknown key "<segment>" in combo "<string>"` ([lines 47-48](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/parsing.rs#L47-L48)). The segment is printed already trimmed, the string as it is written in the config. Parsing stops at the first unknown segment: `"Ctrl+Plus+Foo"` reports only `unknown key "Plus"`.
5. **Duplicates are removed silently** (`KeyCombination::from_keys`, [src/data/key_combination.rs:50-59](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/key_combination.rs#L50-L59)). Two names are duplicates when they mean the same trigger key, so synonyms merge: `"Ctrl+Ctrl+A"` and `"Ctrl+Control+A"` give 2 keys, and so does `"Alt+Menu+F4"`. `Win` and `LWin` are different trigger keys (either Win key, the left one), so `"Win+LWin+Super+E"` keeps 3 keys: `Win`, `LWin`, `E`. `"Shift+LShift"` keeps both too; it is held only when the left Shift is, so it behaves like `"LShift"` (inferred from the code).
6. The written order is kept, but it does not affect matching: `"A+Ctrl"` is the same trigger as `"Ctrl+A"`. The order matters only for the `Combo(...)` step ([§6.8](#s6-8)).

<a id="s6-3"></a>
### 6.3 Name normalisation

`parse_vk` and `KeySpec::parse` first normalise the name ([src/data/vk_name.rs:175-183](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L175-L183)): **all** Unicode whitespace characters are removed, including whitespace inside the name, tabs and U+00A0, and the name is lowercased with Unicode rules. This applies in every place that takes a name, including `PressKey`, `Key` and `chord`. Consequences:

- case does not matter: `ctrl`, `CTRL` and `Ctrl` are equivalent;
- spaces inside a name are allowed: `"Caps Lock"`, `"Page Up"`, `"Left Shift"`, `"Print Screen"`, `"Volume Up"`, `"F 1"`, `"Num 5"`, `PressKey("Volume Mute")`, even `"C T R L"`;
- a space does **not** separate keys: `"Ctrl Shift"` becomes the single name `ctrlshift` and gives `unknown key "Ctrl Shift"`;
- a curiosity: the Kelvin sign `\u{212A}` lowercases to the ASCII `k`, that is, to `VK_K`.

**Can a key be named `+`?** No. In a combo, `+` is always the separator. In the places that take a single name, `"+"` is not a key name either: `PressKey("+")` gives `PressKey: unknown key "+"`, the step `Key("+")` gives `step #0: Key: unknown key "+"`, and `chord: (keys: ["+", "k"])` gives `Chord key #0: unknown key name "+"`. There is no `Plus` word either. The `=`/`+` key of the main row is written as `"="` (`VK_OEM_PLUS`), and with Shift as `"Shift+="`. The numpad plus (`VK_ADD`) is not available at all.

**Numbers in F-keys and numpad keys.** The number after the `F`, `Num` or `Numpad` prefix is parsed with `u16::parse` ([src/data/vk_name.rs:111-127](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L111-L127)). Therefore:

- leading zeros are allowed: `"F01"` = F1, `"F007"` = F7, `"Numpad05"` = Num5;
- a `+` sign is allowed where the string is not split at `+`: `PressKey("F+1")` = F1, `PressKey("Num+5")` = Num5, `chord: (keys: ["F+13", "F+14"])` = F13 and F14. In `keys`, `sequence` and `Combo` the same string `"F+1"` is split into two keys: the letter F and the digit 1;
- `"F0"`, `"F25"`, `"F65537"`, `"Num10"`, `"Num-0"`, `"Num"` and `"Numpad"` are errors.

<a id="s6-4"></a>
### 6.4 Complete list of names

**Modifiers** ([src/data/vk_name.rs:131-141](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L131-L141), [src/data/key_spec.rs:162-170](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/key_spec.rs#L162-L170)). The "As a trigger" column applies to `keys`, `sequence` steps and `chord` keys ([§6.7](#s6-7)); the "As output" column to `PressKey`, `Key` and `Combo` ([§6.8](#s6-8)).

| Names | As a trigger | As output |
|---|---|---|
| `Shift` | either Shift key | VK_SHIFT, 0x10 |
| `LShift`, `LeftShift` | the left Shift only | VK_LSHIFT, 0xA0 |
| `RShift`, `RightShift` | the right Shift only | VK_RSHIFT, 0xA1 |
| `Ctrl`, `Control` | either Ctrl key | VK_CONTROL, 0x11 |
| `LCtrl`, `LeftCtrl`, `LeftControl` | the left Ctrl only | VK_LCONTROL, 0xA2 |
| `RCtrl`, `RightCtrl`, `RightControl` | the right Ctrl only | VK_RCONTROL, 0xA3 |
| `Alt`, `Menu` | either Alt key | VK_MENU, 0x12 |
| `LAlt`, `LeftAlt` | the left Alt only | VK_LMENU, 0xA4 |
| `RAlt`, `RightAlt` | the right Alt only | VK_RMENU, 0xA5 |
| `Win`, `Super` | either Win key | VK_LWIN, 0x5B (the left Win key) |
| `LWin`, `LeftWin` | the left Win only | VK_LWIN, 0x5B |
| `RWin`, `RightWin` | the right Win only | VK_RWIN, 0x5C |

These synonyms do not exist: `LControl`, `RControl`, `LMenu`, `RMenu`, `LSuper`, `RSuper`, `Windows`, `Cmd`, `Meta`, `AltGr`, `Fn`. `Menu` means Alt (in WinAPI it is VK_MENU), **not** the context-menu key.

**Letters, digits, F-keys, numpad** ([src/data/vk_name.rs:103-127](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L103-L127), [src/data/vk_name.rs:185-192](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L185-L192))

| Names | VK | Codes | Note |
|---|---|---|---|
| `A` … `Z` (any case) | VK_A … VK_Z | 0x41–0x5A | ASCII Latin letters only; `"Й"` is an error |
| `0` … `9` | VK_0 … VK_9 | 0x30–0x39 | top-row digits; `"10"` is an error |
| `F1` … `F24` | VK_F1 … VK_F24 | 0x70–0x87 | leading zeros allowed |
| `Num0` … `Num9`, `Numpad0` … `Numpad9` | VK_NUMPAD0 … VK_NUMPAD9 | 0x60–0x69 | digits only, no numpad operators |

Letters are identified by VK codes, not by characters. In the standard Russian layout the letter keys have the same VK codes as in US (the `kbdru.dll` and `kbdus.dll` tables match), so `"Ctrl+S"` also fires on Ctrl+`Ы` (`Ы` is the Russian letter on the S key). In QWERTZ and AZERTY layouts the letter VK follows the character, not the position: in the German `kbdgr.dll` the key in the US Z position sends `VK_Y`, and in the French `kbdfr.dll` the key in the US Q position sends `VK_A`.

**Named keys** ([src/data/vk_name.rs:142-161](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L142-L161))

| Names | VK | Code |
|---|---|---|
| `CapsLock`, `Caps` | VK_CAPITAL | 0x14 |
| `Tab` | VK_TAB | 0x09 |
| `Esc`, `Escape` | VK_ESCAPE | 0x1B |
| `Enter`, `Return` | VK_RETURN | 0x0D |
| `Space`, `Spacebar` | VK_SPACE | 0x20 |
| `Backspace`, `Back` | VK_BACK | 0x08 |
| `Del`, `Delete` | VK_DELETE | 0x2E |
| `Ins`, `Insert` | VK_INSERT | 0x2D |
| `Home` | VK_HOME | 0x24 |
| `End` | VK_END | 0x23 |
| `PageUp`, `PgUp` | VK_PRIOR | 0x21 |
| `PageDown`, `PgDn` | VK_NEXT | 0x22 |
| `Up`, `UpArrow` | VK_UP | 0x26 |
| `Down`, `DownArrow` | VK_DOWN | 0x28 |
| `Left`, `LeftArrow` | VK_LEFT | 0x25 |
| `Right`, `RightArrow` | VK_RIGHT | 0x27 |
| `NumLock` | VK_NUMLOCK | 0x90 |
| `ScrollLock`, `Scroll` | VK_SCROLL | 0x91 |
| `Pause`, `Break` | VK_PAUSE | 0x13 |
| `PrtSc`, `PrintScreen`, `PrtScn` | VK_SNAPSHOT | 0x2C |

These common spellings are **not** accepted: `PrtScr`, `SysRq`, `ScrLk`, `PgDown`, `PageDn`, `ArrowUp`, `Bksp`, `Caps-Lock`.

**OEM punctuation.** The name is exactly one character ([src/data/vk_name.rs:193-206](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L193-L206)). It is the character the key produces in the **US layout**, which identifies its physical position. In the standard Russian layout these keys have the same VK codes as in US. Other layouts assign the OEM codes differently: for example, in the German `kbdgr.dll` the key in the US `` ` `` position sends `VK_OEM_5`, and in the French `kbdfr.dll` the key in the US `;` position sends `VK_M`.

| Name | VK | Code | Same key on the Russian `ЙЦУКЕН` layout |
|---|---|---|---|
| `` ` `` | VK_OEM_3 | 0xC0 | `Ё` |
| `-` | VK_OEM_MINUS | 0xBD | `-` |
| `=` | VK_OEM_PLUS | 0xBB | `=` |
| `,` | VK_OEM_COMMA | 0xBC | `Б` |
| `.` | VK_OEM_PERIOD | 0xBE | `Ю` |
| `;` | VK_OEM_1 | 0xBA | `Ж` |
| `/` | VK_OEM_2 | 0xBF | `.` |
| `[` | VK_OEM_4 | 0xDB | `Х` |
| `]` | VK_OEM_6 | 0xDD | `Ъ` |
| `\` | VK_OEM_5 | 0xDC | `\` |
| `'` | VK_OEM_7 | 0xDE | `Э` |

In a normal RON string the `\` key is written as `\\`: `"Ctrl+\\"`, `PressKey("\\")`. A raw string also works: `r"Ctrl+Alt+\"` or `r#"Ctrl+Shift+\"#`. Without the escape, parsing of the whole file fails, and the message depends on what follows:

- `"Ctrl+\+A"` gives `Unknown escape character`;
- `"Ctrl+\"` turns `\"` into a quote inside the string, and the string continues up to the next `"` in the file. If there is none, the error is `Expected end of string`. If there is one (and in a real file there almost always is), the error is a different one and points further into the file: `( keys: "Ctrl+\", action: PressKey("A") )`, as the only binding of a one-line `( bindings: [ ... ] )` file, gives `1:51: Expected comma`.

The `"` key is written as `"Shift+'"`; `"Ctrl+\""` gives `unknown key "\""`. There are no word names (`Plus`, `Minus`, `Comma`, `Grave`, `Tilde` …).

**Media and volume** ([src/data/vk_name.rs:162-170](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L162-L170))

| Names | VK | Code | `Media(…)` variant |
|---|---|---|---|
| `MediaPlayPause`, `PlayPause` | VK_MEDIA_PLAY_PAUSE | 0xB3 | `PlayPause` |
| `MediaStop` | VK_MEDIA_STOP | 0xB2 | `Stop` |
| `MediaNext`, `NextTrack` | VK_MEDIA_NEXT_TRACK | 0xB0 | `Next` |
| `MediaPrev`, `MediaPrevious`, `PrevTrack` | VK_MEDIA_PREV_TRACK | 0xB1 | `Previous` |
| `VolumeUp`, `VolUp` | VK_VOLUME_UP | 0xAF | `VolumeUp` |
| `VolumeDown`, `VolDown` | VK_VOLUME_DOWN | 0xAE | `VolumeDown` |
| `VolumeMute`, `VolMute`, `Mute` | VK_VOLUME_MUTE | 0xAD | `VolumeMute` |

Bare `Stop`, `Next` and `Previous` are not accepted as key names: they are `Media(...)` variants, not names.

**How the daemon prints names.** At startup the daemon prints the list of bindings ([src/core/app.rs:139-156](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/core/app.rs#L139-L156)) with canonical names, not the way they are written in the config, and joins the keys of a combo with ` + `. A trigger key prints as `Shift`, `Ctrl`, `Alt` or `Win` when it means either side, and `LWin` for the left Win key ([src/data/key_spec.rs:184-194](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/key_spec.rs#L184-L194)). Every other key prints with its `vk_name` ([src/data/vk_name.rs:5-73](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L5-L73)): `Super` is printed as `Win`, `Menu` as `Alt`, `LeftShift` as `LShift`, `Return` as `Enter`, `PrintScreen` as `PrtSc`, `Numpad5` as `Num5`. A key that a `PressKey` action sends prints with `vk_name`, so `PressKey("LWin")` prints as `press key Win`. The form `VK(n)` with a decimal `n` is the fallback output of `vk_name` for codes that have no name ([src/data/vk_name.rs:71](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/vk_name.rs#L71)). Every code that can be set in the config has a name, so `VK(n)` does not appear in this list, and the form is not parsed back.

<a id="s6-5"></a>
### 6.5 What does not exist at all

Everything listed here gives `unknown key`:

- the context-menu key (`VK_APPS`, 0x5D). It has no name (`Apps`, `ContextMenu` and `Application` are not accepted), and `Menu` means Alt. It can be neither a trigger nor pressed through `PressKey`;
- the numpad operators (`VK_MULTIPLY` … `VK_DIVIDE`, 0x6A–0x6F: `NumpadAdd`, `Multiply`, `Add`, `Subtract`, `Decimal`, `Divide` …) and a separate numpad Enter (`NumpadEnter`);
- `VK_OEM_102` (0xE2), the extra `\` key between left Shift and Z on ISO keyboards;
- browser and launch keys (`VK_BROWSER_*`, `VK_LAUNCH_*`), as well as `Sleep`, `Clear`, `Cancel` and the IME keys (`Kana`, `Kanji`, `Convert` …). The OS does not see the `Fn` key at all;
- raw codes: neither `"0x5D"`, nor `"93"`, nor `"VK_APPS"`, nor `"VK(93)"` is accepted.

New key names are planned as [new feature 2](../roadmap.md#new-features-after-stabilisation) in the roadmap.

<a id="s6-6"></a>
### 6.6 Windows behaviour that affects key names

The hook adapter takes only `vkCode` and `dwExtraInfo` from `KBDLLHOOKSTRUCT` (the latter to skip its own synthesized events). It reads neither the flags nor the scan code ([src/hook/keyboard_hook_callback.rs:382-385](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/hook/keyboard_hook_callback.rs#L382-L385)). The codes in the list and in the two warnings below are taken from the layout tables of `kbdus.dll` and `kbdru.dll`; this was not tested on a live daemon:

- the main Enter and the numpad Enter (scan code E0 1C) have the same code, `VK_RETURN`, so `"Enter"` catches both. The same goes for the navigation keys: `"Home"` catches both Home and Num7 with NumLock off;
- with NumLock off, the numpad sends navigation codes: Num1 → End, Num8 → Up, Num0 → Insert, `.` → Delete, Num5 → `VK_CLEAR` (0x0C). In that state `"Num1"` does not fire, and `"End"` fires on Num1 as well.

> [!WARNING]
> Ctrl+Pause arrives as scan code E0 46, which both layouts map to `VK_CANCEL` (0x03). `"Ctrl+Pause"` and `"Ctrl+Break"` therefore load without errors but apparently never fire.

> [!WARNING]
> **Layouts with AltGr.** `kbdru.dll` sets the `KLLF_ALTGR` flag (as the German, Polish and French layouts do); `kbdus.dll` does not. In a layout with AltGr, right Alt arrives as a fake left Ctrl plus right Alt, so the hook sees Ctrl and Alt held. With modifiers matched exactly ([§7](kinds.md#s7)), right Alt+X then fires `"Ctrl+Alt+X"` but not `"Ctrl+X"`, `"Alt+X"` or `"RAlt+X"`. Write `"Ctrl+RAlt+X"` or set `allow_extra_modifiers: true` for an AltGr binding. The hook cannot tell the fake Ctrl from a real one, because it does not read the scan code ([new feature 2](../roadmap.md#new-features-after-stabilisation)).

<a id="s6-7"></a>
### 6.7 Left and right modifiers

How the daemon compares modifiers ([KS-07](../audit.md#ks-07), fixed in `1dfd80a`; the sided handling came with Workstream A):

- For physical keys the low-level hook reports the code of the specific side: LShift 0xA0, RShift 0xA1, LCtrl 0xA2, RCtrl 0xA3, LAlt 0xA4, RAlt 0xA5, LWin 0x5B, RWin 0x5C. The engine keeps these codes in its table of held keys. The generic codes Shift 0x10, Ctrl 0x11 and Alt 0x12 are held only between the key-down and the key-up a program sends under them, and stop counting once a key-down, or the main thread's read between key events, finds neither key of the pair down in Windows; the daemon never takes them from the Windows key state (roadmap item 2.5, [src/engine/modifiers.rs](https://github.com/bluesteelll/key-switch-rs/blob/045b8b4/src/engine/modifiers.rs)).
- Every trigger key is a `KeySpec`, fixed when the config loads. `Shift`, `Ctrl`, `Alt` and `Win` (`Super`) accept either side, and the generic code too; every other name accepts its own code only ([src/data/key_spec.rs:131-139](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/key_spec.rs#L131-L139)).
- A key of a combo or of a sequence step counts as held if a key it accepts is held ([src/engine/keys.rs:242-252](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/engine/keys.rs#L242-L252)). A chord key counts as pressed if it accepts the code of the key-down ([src/engine/chords.rs:81-83](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/engine/chords.rs#L81-L83)).
- Exact modifier matching compares families, not sides: the held modifiers are Shift, Ctrl, Alt and Win, each held if either of its keys is ([src/engine/matching.rs:38-45](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/engine/matching.rs#L38-L45)). `"LShift+F13"` fires with the left Shift held, also when the right Shift is held too, and does not fire with only the right Shift.

What to write in practice, the same in `keys`, `sequence` and `chord`:

| You need | Write |
|---|---|
| either of the two Shift/Ctrl/Alt keys | `Shift` / `Ctrl` / `Alt` |
| the left one only or the right one only | `LShift` / `RShift`, `LCtrl` / `RCtrl`, `LAlt` / `RAlt` |
| either Win key | `Win` or `Super` |
| the left or the right Win key only | `LWin` / `RWin` |

> [!NOTE]
> Before `1dfd80a` the names behaved differently ([KS-07](../audit.md#ks-07)). Sided names never fired in `keys` and `sequence`, `LWin` meant either Win key there, and in a `chord` the unsided `Shift`, `Ctrl` and `Alt` never matched while `Win` meant the left key. A config written for that version may need its names checked: `LWin` now means the left Win key only.

A chord's keys must not overlap: `["Shift", "LShift"]`, `["Win", "RWin"]` and `["Ctrl", "Control"]` are load errors, because one press would count for two keys ([§9.1](kinds.md#s9-1)).

> [!TIP]
> **It is better not to put modifiers into chords at all.** A chord key is suppressed on **every** press ([src/engine/chords.rs:12-18](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/engine/chords.rs#L12-L18), [§9](kinds.md#s9)). For example, `["LCtrl", "F13"]` breaks every combination with the left Ctrl, and `chord: (keys: ["Shift", "Space"])` swallows both Shift keys and the space bar for normal typing.

<a id="s6-8"></a>
### 6.8 Synthesized input: `PressKey`, `Key`, `Combo`

The code is passed to `SendInput` as is, with `wScan = 0` and without the `KEYEVENTF_EXTENDEDKEY` flag ([src/core/windows_actions.rs:257-291](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/core/windows_actions.rs#L257-L291), [src/data/sequence.rs:163-227](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/sequence.rs#L163-L227)). Side-specific names are allowed here and are sent with their own codes: `PressKey("RShift")` sends `VK_RSHIFT`, while `Shift`, `Ctrl` and `Alt` go out as the generic codes 0x10/0x11/0x12, and `Win` as the left Win key, 0x5B. The trigger-key rules of [§6.7](#s6-7) do not apply to output.

- Which side the application sees is decided by Windows; this was not tested on a live system.
- `PressKey` and `Key` make a single press (down and up). A modifier is pressed and released at once, not held: `trigger: Hold(...)` with `PressKey("LShift")` does not give a held Shift. Holding a synthesized modifier is not implemented ([Hold that keeps a synthesized modifier held](../gaps.md#hold-that-keeps-a-synthesized-modifier-held)).
- `Combo("...")` first presses all the modifiers (any of the 11 modifier codes, [src/data/sequence.rs:229-244](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/sequence.rs#L229-L244)) in the written order, then the other keys in the written order, and releases them in reverse order ([src/data/sequence.rs:179-212](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/sequence.rs#L179-L212)). For example, `Combo("A+Ctrl")` sends Ctrl↓ A↓ A↑ Ctrl↑. Repeated names are dropped by code ([src/config/parsing.rs:19-32](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/config/parsing.rs#L19-L32)).

> [!WARNING]
> Because the extended-key flag is missing, programs that read the scan code (games, RDP and VM clients) may take right Ctrl/Alt for the left keys, and the arrows, Home/End, Insert/Delete and PageUp/PageDown for numpad keys (not tested; [KS-36](../audit.md#ks-36), open until roadmap [item 3.1](../roadmap.md#31-single-input-module)).

> [!WARNING]
> Known issue ([KS-19](../audit.md#ks-19)): synthesized presses do not release the modifiers that the user is physically holding ([src/core/windows_actions.rs:257](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/core/windows_actions.rs#L257); [src/data/sequence.rs:163](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/sequence.rs#L163), [src/data/sequence.rs:179](https://github.com/bluesteelll/key-switch-rs/blob/bf549c0/src/data/sequence.rs#L179)). With `keys: "Ctrl+Alt+L", action: PressKey("L")` the application receives Ctrl+Alt+L, not L ([§13.1](actions.md#s13-1)).
