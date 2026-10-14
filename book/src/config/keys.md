# Key names and combo syntax

<a id="s6"></a>
## 6. Key names and combo strings

Main files: [src/config/parsing.rs](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/parsing.rs), [src/config/loader.rs](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs), [src/data/vk_name.rs](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs), [src/data/key_combination.rs](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/key_combination.rs), [src/hook/keyboard_hook_callback.rs](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs). Key names and codes were checked with the validator, which runs the real `config::load` ([How the audit was done](../methodology.md#validation-harness)). The codes that Windows sends were cross-checked against the layout tables of `kbdus.dll` and `kbdru.dll` (through `KbdLayerDescriptor`), but were not tested on a live daemon.

<a id="s6-1"></a>
### 6.1 Where key names appear in the config

| Place | What is written | Parsed by | Code |
|---|---|---|---|
| `keys: "..."` | a combo string | `parse_combo` | [src/config/loader.rs:259-261](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L259-L261) |
| `sequence: (steps: [...])` | each step is a combo string | `parse_combo` for each step | [src/config/loader.rs:283](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L283) |
| `chord: (keys: [...])` | each element is **one** name, without `+` | `parse_vk`, duplicates rejected | [src/config/loader.rs:296-304](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L296-L304) |
| `action: PressKey("...")` | one name | `parse_vk` | [src/config/loader.rs:358-361](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L358-L361) |
| step `Key("...")` in `Sequence([...])` | one name | `parse_vk` | [src/config/loader.rs:428-431](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L428-L431) |
| step `Combo("...")` in `Sequence([...])` | a combo string | `parse_combo` | [src/config/loader.rs:434-440](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L434-L440) |

`Media(...)` (both the action and the step) does not take a key name. It takes a separate RON variant, written without quotes: `PlayPause`, `Stop`, `Next`, `Previous`, `VolumeUp`, `VolumeDown`, `VolumeMute` ([src/config/schema.rs:156-164](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L156-L164)). That is why `Media(Mute)` is a RON parse error, while `PressKey("Mute")` is valid (`VK_VOLUME_MUTE`, 0xAD).

Case does not matter only inside strings that hold key names. RON identifiers (actions, `Media` variants, steps, triggers) are case-sensitive and are written without quotes: `Media(playpause)`, `pressKey("A")` and `Media("PlayPause")` are parse errors.

<a id="s6-2"></a>
### 6.2 Combo string grammar

```text
combo   := segment ( "+" segment )*
segment := [whitespace] key-name [whitespace]
```

`parse_combo` ([src/config/parsing.rs:16-35](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/parsing.rs#L16-L35)) works as follows:

1. The string is trimmed at both ends. If it is empty or consists only of whitespace, the error is `combo is empty` ([lines 17-20](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/parsing.rs#L17-L20)).
2. The string is split at **every** `+` character ([line 23](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/parsing.rs#L23)). There is no escaping.
3. Each segment is trimmed at both ends. An empty segment gives the error `empty segment in combo "<string>" (stray '+'?)` ([lines 24-27](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/parsing.rs#L24-L27)). This rejects `"Ctrl++A"`, `"+A"`, `"A+"`, `"+"` and `"Ctrl+ +A"`.
4. Each segment is passed to `parse_vk`. If the name is unknown, the error is `unknown key "<segment>" in combo "<string>"` ([lines 28-30](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/parsing.rs#L28-L30)). The segment is printed already trimmed, the string as it is written in the config. Parsing stops at the first unknown segment: `"Ctrl+Plus+Foo"` reports only `unknown key "Plus"`.
5. **Duplicates are removed silently**, by VK code (`KeyCombination::from_keys`, [src/data/key_combination.rs:39-47](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/key_combination.rs#L39-L47)), so synonyms merge as well. `"Ctrl+Ctrl+A"` and `"Ctrl+Control+A"` give 2 keys, and so do `"Win+LWin+Super+E"` and `"Alt+Menu+F4"`. `"Shift+LShift"`, however, keeps two different codes (0x10 and 0xA0) and never fires ([§6.7](#s6-7)).
6. The written order is kept, but it does not affect matching: `"A+Ctrl"` is the same trigger as `"Ctrl+A"`. The order matters only for the `Combo(...)` step ([§6.8](#s6-8)).

<a id="s6-3"></a>
### 6.3 Name normalisation

`parse_vk` ([src/data/vk_name.rs:95-99](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L95-L99)) removes **all** Unicode whitespace characters from the name, including whitespace inside the name, tabs and U+00A0, and then lowercases the name using Unicode rules. This applies in every place that takes a name, including `PressKey`, `Key` and `chord`. Consequences:

- case does not matter: `ctrl`, `CTRL` and `Ctrl` are equivalent;
- spaces inside a name are allowed: `"Caps Lock"`, `"Page Up"`, `"Left Shift"`, `"Print Screen"`, `"Volume Up"`, `"F 1"`, `"Num 5"`, `PressKey("Volume Mute")`, even `"C T R L"`;
- a space does **not** separate keys: `"Ctrl Shift"` becomes the single name `ctrlshift` and gives `unknown key "Ctrl Shift"`;
- a curiosity: the Kelvin sign `\u{212A}` lowercases to the ASCII `k`, that is, to `VK_K`.

**Can a key be named `+`?** No. In a combo, `+` is always the separator. In the places that take a single name, `parse_vk("+")` finds nothing: `PressKey("+")` gives `PressKey: unknown key "+"`, the step `Key("+")` gives `step #0: Key: unknown key "+"`, and `chord: (keys: ["+", "k"])` gives `Chord key #0: unknown key name "+"`. There is no `Plus` word either. The `=`/`+` key of the main row is written as `"="` (`VK_OEM_PLUS`), and with Shift as `"Shift+="`. The numpad plus (`VK_ADD`) is not available at all.

**Numbers in F-keys and numpad keys.** The number after the `F`, `Num` or `Numpad` prefix is parsed with `u16::parse` ([src/data/vk_name.rs:114-129](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L114-L129)). Therefore:

- leading zeros are allowed: `"F01"` = F1, `"F007"` = F7, `"Numpad05"` = Num5;
- a `+` sign is allowed where the string is not split at `+`: `PressKey("F+1")` = F1, `PressKey("Num+5")` = Num5, `chord: (keys: ["F+13", "F+14"])` = F13 and F14. In `keys`, `sequence` and `Combo` the same string `"F+1"` is split into two keys: the letter F and the digit 1;
- `"F0"`, `"F25"`, `"F65537"`, `"Num10"`, `"Num-0"`, `"Num"` and `"Numpad"` are errors.

<a id="s6-4"></a>
### 6.4 Complete list of names

**Modifiers** ([src/data/vk_name.rs:133-143](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L133-L143)). The "In `keys`" and "In `chord`" columns are derived from the hook code ([§6.7](#s6-7)). **never** marks the defect [KS-07](../audit.md#ks-07): the binding loads but does not fire.

| Names | VK | Code | In `keys` / a `sequence` step | In `chord` |
|---|---|---|---|---|
| `Shift` | VK_SHIFT | 0x10 | yes, either Shift | **never** |
| `LShift`, `LeftShift` | VK_LSHIFT | 0xA0 | **never** | left only |
| `RShift`, `RightShift` | VK_RSHIFT | 0xA1 | **never** | right only |
| `Ctrl`, `Control` | VK_CONTROL | 0x11 | yes, either Ctrl | **never** |
| `LCtrl`, `LeftCtrl`, `LeftControl` | VK_LCONTROL | 0xA2 | **never** | left only |
| `RCtrl`, `RightCtrl`, `RightControl` | VK_RCONTROL | 0xA3 | **never** | right only |
| `Alt`, `Menu` | VK_MENU | 0x12 | yes, either Alt | **never** |
| `LAlt`, `LeftAlt` | VK_LMENU | 0xA4 | **never** | left only |
| `RAlt`, `RightAlt` | VK_RMENU | 0xA5 | **never** | right only |
| `Win`, `Super`, `LWin`, `LeftWin` | VK_LWIN | 0x5B | yes, **either** Win (the right one too) | left only |
| `RWin`, `RightWin` | VK_RWIN | 0x5C | **never** | right only |

These synonyms do not exist: `LControl`, `RControl`, `LMenu`, `RMenu`, `LSuper`, `RSuper`, `Windows`, `Cmd`, `Meta`, `AltGr`, `Fn`. `Menu` means Alt (in WinAPI it is VK_MENU), **not** the context-menu key.

**Letters, digits, F-keys, numpad** ([src/data/vk_name.rs:106-129](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L106-L129), [src/data/vk_name.rs:177-184](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L177-L184))

| Names | VK | Codes | Note |
|---|---|---|---|
| `A` … `Z` (any case) | VK_A … VK_Z | 0x41–0x5A | ASCII Latin letters only; `"Й"` is an error |
| `0` … `9` | VK_0 … VK_9 | 0x30–0x39 | top-row digits; `"10"` is an error |
| `F1` … `F24` | VK_F1 … VK_F24 | 0x70–0x87 | leading zeros allowed |
| `Num0` … `Num9`, `Numpad0` … `Numpad9` | VK_NUMPAD0 … VK_NUMPAD9 | 0x60–0x69 | digits only, no numpad operators |

Letters are identified by VK codes, not by characters. In the standard Russian layout the letter keys have the same VK codes as in US (the `kbdru.dll` and `kbdus.dll` tables match), so `"Ctrl+S"` also fires on Ctrl+`Ы` (`Ы` is the Russian letter on the S key). In QWERTZ and AZERTY layouts the letter VK follows the character, not the position: in the German `kbdgr.dll` the key in the US Z position sends `VK_Y`, and in the French `kbdfr.dll` the key in the US Q position sends `VK_A`.

**Named keys** ([src/data/vk_name.rs:144-163](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L144-L163))

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

**OEM punctuation.** The name is exactly one character ([src/data/vk_name.rs:185-196](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L185-L196)). It is the character the key produces in the **US layout**, which identifies its physical position. In the standard Russian layout these keys have the same VK codes as in US. Other layouts assign the OEM codes differently: for example, in the German `kbdgr.dll` the key in the US `` ` `` position sends `VK_OEM_5`, and in the French `kbdfr.dll` the key in the US `;` position sends `VK_M`.

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

**Media and volume** ([src/data/vk_name.rs:166-172](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L166-L172))

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

**How the daemon prints names.** At startup the daemon prints the list of bindings ([src/core/app.rs:62](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/core/app.rs#L62)) with the canonical `vk_name` names ([src/data/vk_name.rs:5-73](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L5-L73)), not the way they are written in the config: `LWin` and `Super` are printed as `Win`, `Menu` as `Alt`, `Return` as `Enter`, `PrintScreen` as `PrtSc`, `Numpad5` as `Num5`. The form `VK(n)` with a decimal `n` is the fallback output of `vk_name` for codes that have no name ([src/data/vk_name.rs:71](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L71)). Every code that can be set in the config has a name, so `VK(n)` does not appear in this list, and the form is not parsed back.

<a id="s6-5"></a>
### 6.5 What does not exist at all

Everything listed here gives `unknown key`:

- the context-menu key (`VK_APPS`, 0x5D). It has no name (`Apps`, `ContextMenu` and `Application` are not accepted), and `Menu` means Alt. It can be neither a trigger nor pressed through `PressKey`;
- the numpad operators (`VK_MULTIPLY` … `VK_DIVIDE`, 0x6A–0x6F: `NumpadAdd`, `Multiply`, `Add`, `Subtract`, `Decimal`, `Divide` …) and a separate numpad Enter (`NumpadEnter`);
- `VK_OEM_102` (0xE2), the extra `\` key between left Shift and Z on ISO keyboards;
- browser and launch keys (`VK_BROWSER_*`, `VK_LAUNCH_*`), as well as `Sleep`, `Clear`, `Cancel` and the IME keys (`Kana`, `Kanji`, `Convert` …). The OS does not see the `Fn` key at all;
- raw codes: neither `"0x5D"`, nor `"93"`, nor `"VK_APPS"`, nor `"VK(93)"` is accepted.

<a id="s6-6"></a>
### 6.6 Windows behaviour that affects key names

The hook takes only `vkCode` and `dwExtraInfo` from `KBDLLHOOKSTRUCT` (the latter to skip its own synthesized events). It reads neither the flags nor the scan code ([src/hook/keyboard_hook_callback.rs:134-140](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L134-L140)). The codes in the list and in the two warnings below are taken from the layout tables of `kbdus.dll` and `kbdru.dll`; this was not tested on a live daemon:

- the main Enter and the numpad Enter (scan code E0 1C) have the same code, `VK_RETURN`, so `"Enter"` catches both. The same goes for the navigation keys: `"Home"` catches both Home and Num7 with NumLock off;
- with NumLock off, the numpad sends navigation codes: Num1 → End, Num8 → Up, Num0 → Insert, `.` → Delete, Num5 → `VK_CLEAR` (0x0C). In that state `"Num1"` does not fire, and `"End"` fires on Num1 as well.

> [!WARNING]
> Ctrl+Pause arrives as scan code E0 46, which both layouts map to `VK_CANCEL` (0x03). `"Ctrl+Pause"` and `"Ctrl+Break"` therefore load without errors but apparently never fire.

> [!WARNING]
> **The Russian layout has AltGr.** `kbdru.dll` sets the `KLLF_ALTGR` flag (as the German and French layouts do); `kbdus.dll` does not. In a layout with AltGr, right Alt arrives as a fake left Ctrl plus right Alt, and after the sides are folded that is `Ctrl+Alt`. With the Russian layout active, right Alt+X therefore fires bindings on `"Ctrl+Alt+X"` and even on `"Ctrl+X"` (extra held keys do not prevent a match, [§7](kinds.md#s7)); with the English (US) layout it does not. The hook cannot tell the fake Ctrl from a real one, because it does not read the scan code.

<a id="s6-7"></a>
### 6.7 Left and right modifiers

How the hook sees modifiers:

- For physical keys the low-level hook reports the code of the specific side: LShift 0xA0, RShift 0xA1, LCtrl 0xA2, RCtrl 0xA3, LAlt 0xA4, RAlt 0xA5, LWin 0x5B, RWin 0x5C (`vkCode`, [src/hook/keyboard_hook_callback.rs:140](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L140)).
- For **`keys`** and **`sequence`**, the set of held keys passes through `collect_active_keys` ([src/hook/keyboard_hook_callback.rs:327-347](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L327-L347)) before matching, and that function folds the sides together ([lines 335-341](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L335-L341)):

  | Physically pressed | Goes into the set used for matching |
  |---|---|
  | LShift or RShift | `Shift` (0x10) |
  | LCtrl or RCtrl | `Ctrl` (0x11) |
  | LAlt or RAlt | `Alt` (0x12) |
  | LWin or RWin | `Win` (0x5B, the same code as `LWin`) |

- **`chord`** receives the raw `vk_code` without folding ([src/hook/keyboard_hook_callback.rs:275-280](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L275-L280), compared in [src/hook/chord_state.rs:73](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L73)).
- The loader stores the code exactly as `parse_vk` returned it ([src/data/vk_name.rs:134-143](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L134-L143)). Nothing normalises it afterwards: `update_bindings` only sorts the list ([src/hook/keyboard_hook.rs:71-74](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook.rs#L71-L74)).

> [!WARNING]
> Known issue ([KS-07](../audit.md#ks-07)): the names `LShift`, `RShift`, `LCtrl`, `RCtrl`, `LAlt`, `RAlt`, `RWin` and their `Left…`/`Right…` synonyms in `keys` or in a `sequence` step load without an error or a warning, but the binding **never fires**: the set used for matching never contains side-specific codes ([src/hook/keyboard_hook_callback.rs:335-341](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L335-L341); [src/data/vk_name.rs:134-143](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/vk_name.rs#L134-L143)). This holds for every trigger, including the README example `keys: "LShift", trigger: DoubleTap(250)` ([README.md:262](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/README.md?plain=1#L262)) and the commented-out example in [src/config/default_config.ron:200-205](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L200-L205). [README.md:173](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/README.md?plain=1#L173) also lists `RWin` as a modifier for `keys`. The exception is `LWin`/`LeftWin`: they have the same code as `Win` and work, for both Win keys.

> [!WARNING]
> Known issue ([KS-07](../audit.md#ks-07)): in `chord` it is the other way round. `Shift`, `Ctrl` and `Alt` (and the synonyms `Control` and `Menu`) **never complete a chord**, because the hook reports only side-specific codes ([src/hook/chord_state.rs:73](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L73); [src/hook/keyboard_hook_callback.rs:276](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L276)). The other keys of such a chord are still suppressed on every press: `chord: (keys: ["Shift", "F13"])` simply makes F13 dead. Side-specific names in a chord work, judging by the code, but only for their own side. `Win` (and `LWin`, `Super`) in a chord means the left Win key only.

What to write in practice:

| You need | In `keys` / `sequence` | In `chord` |
|---|---|---|
| either of the two Shift/Ctrl/Alt keys | `Shift` / `Ctrl` / `Alt` | cannot be expressed |
| the left one only or the right one only | cannot be expressed | `LShift` / `RShift` and so on |
| either Win key | `Win` (= `LWin` = `Super`) | cannot be expressed |
| the right Win key only | cannot be expressed | `RWin` |

> [!TIP]
> **It is better not to put modifiers into chords at all.** If you must, only side-specific names work (`LShift`, `RCtrl`, `LAlt` …), and that key is then suppressed on **every** press, because a chord key is always suppressed ([src/hook/chord_state.rs:12-17](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L12-L17), [§9](kinds.md#s9)). For example, `["LCtrl", "F13"]` breaks every combination with the left Ctrl, and even the working `chord: (keys: ["LShift", "RShift"])` disables both Shift keys for normal typing.

<a id="s6-8"></a>
### 6.8 Synthesized input: `PressKey`, `Key`, `Combo`

The code is passed to `SendInput` as is, with `wScan = 0` and without the `KEYEVENTF_EXTENDEDKEY` flag ([src/core/windows_actions.rs:149-188](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/core/windows_actions.rs#L149-L188), [src/data/sequence.rs:185-254](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/sequence.rs#L185-L254)). Side-specific names are allowed here and are sent with their own codes: `PressKey("RShift")` sends `VK_RSHIFT`, while `Shift`, `Ctrl` and `Alt` go out as the generic codes 0x10/0x11/0x12. The problem with side-specific names affects only triggers (`keys`, `sequence`, `chord`), not output.

- Which side the application sees is decided by Windows; this was not tested on a live system.
- `PressKey` and `Key` make a single press (down and up). A modifier is pressed and released at once, not held: `trigger: Hold(...)` with `PressKey("LShift")` does not give a held Shift. Holding a synthesized modifier is not implemented.
- `Combo("...")` first presses all the modifiers (any of the 11 modifier codes, [src/data/sequence.rs:256-263](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/sequence.rs#L256-L263)) in the written order, then the other keys in the written order, and releases them in reverse order ([src/data/sequence.rs:205-238](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/sequence.rs#L205-L238)). For example, `Combo("A+Ctrl")` sends Ctrl↓ A↓ A↑ Ctrl↑.

> [!WARNING]
> Because the extended-key flag is missing, programs that read the scan code (games, RDP and VM clients) may take right Ctrl/Alt for the left keys, and the arrows, Home/End, Insert/Delete and PageUp/PageDown for numpad keys (also not tested; [KS-36](../audit.md#ks-36)).

> [!WARNING]
> Known issue ([KS-19](../audit.md#ks-19)): synthesized presses do not release the modifiers that the user is physically holding ([src/core/windows_actions.rs:149](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/core/windows_actions.rs#L149); [src/data/sequence.rs:185](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/sequence.rs#L185), [src/data/sequence.rs:205](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/sequence.rs#L205)). With `keys: "Ctrl+Alt+L", action: PressKey("L")` the application receives Ctrl+Alt+L, not L ([§13.1](actions.md#s13-1)).
