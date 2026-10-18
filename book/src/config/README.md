# Configuration reference

`config.ron` is the only settings file of the `key-switch-rs.exe` daemon, a keyboard remapper for Windows. It lists the bindings: which key, combination, sequence or chord runs which action, on which gesture (`Tap`, `Hold`, `DoubleTap`) and in which window (`when`). The file is written in RON (Rusty Object Notation). The daemon creates it on first run and re-reads it every time it is saved.

This reference describes what the code actually does at commit [`20af28d`](https://github.com/bluesteelll/key-switch-rs/commit/20af28d) (2026-10-18), the tip of `master` after roadmap Phase 1, not what the [README](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/README.md) and the comments in the [default config template](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/default_config.ron) promise: where they disagree, the code is right. Every example in a `ron` block was run through the real loader ([`config::reload`](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L175-L182), which parses a file without writing anything), and the complete example in [§17](example.md#s17) loads without errors. Statements about how the hook behaves at run time are derived from reading the code and the engine's scenario tests, not from a live daemon, and the text says so. Code references are permalinks to that commit: paths under `src/` are source files, and `README.md` and `build.rs` sit at the repository root. The reference was first written for the audit commit `7cb3521`; the [audit](../audit.md) keeps the defects of that version.

Known problems are flagged as warnings: open defects from the audit, traps, and places where the documentation disagrees with the code. All of them are collected in [§19](known-issues.md#s19), together with the pitfalls that Phase 0 fixed, and issue IDs such as [KS-13](../audit.md#ks-13) link to the [audit results](../audit.md).

## Pages in this reference

- [File location, reload and top-level structure](structure.md): where the file lives, how it is created on first run and hot-reloaded ([§2](structure.md#s2)), the top level of the file ([§4](structure.md#s4)) and the fields of a binding ([§5](structure.md#s5)).
- [RON syntax essentials](ron-syntax.md): the RON syntax the file accepts, from encoding, comments and brackets to integers, strings, Windows paths and the nesting limit ([§3](ron-syntax.md#s3)).
- [Key names and combo syntax](keys.md): every accepted key name, the grammar of a combo string, and left and right modifiers ([§6](keys.md#s6)).
- [Binding kinds and key suppression](kinds.md): the `keys` combo ([§7](kinds.md#s7)), `sequence` ([§8](kinds.md#s8)) and `chord` ([§9](kinds.md#s9)) kinds, and which keys are suppressed, including `block_default` ([§10](kinds.md#s10)).
- [Triggers and conditions](triggers-conditions.md): the `trigger` values `Immediate`, `Tap`, `Hold` and `DoubleTap` ([§11](triggers-conditions.md#s11)) and the `when` window conditions ([§12](triggers-conditions.md#s12)).
- [Actions and macro steps](actions.md): every `action` variant ([§13](actions.md#s13)) and the `Sequence([...])` macro with its steps ([§14](actions.md#s14)).
- [Auto-blockers and matching order](matching.md): the auto-blockers created by `block_original_combo` ([§15](matching.md#s15)) and the order and priority in which bindings are checked ([§16](matching.md#s16)).
- [Complete example](example.md): a full `config.ron` that loads without errors ([§17](example.md#s17)).
- [Error messages](errors.md): read, parse and validation errors, where they appear and what the loader does not check ([§18](errors.md#s18)).
- [Known configuration pitfalls](known-issues.md): all known problems at `7cb3521` in one place ([§19](known-issues.md#s19)).

<a id="s1"></a>
## 1. Cheat sheet

The sketch below is not a ready-made file but an overview of the whole shape on one screen. `|` means "one of", and `= x` is the default value. A real file that loads without errors is in [§17](example.md#s17).

```text
// config.ron: UTF-8, with or without a BOM; comments // … and /* … */; trailing commas allowed everywhere
(
    bindings: [                                   // required; [] → 0 bindings and a warning (§4)
        (
            // KIND: exactly one of the three fields (§5.2)
            keys:     "Ctrl+Alt+L",                                // combo (§7)
            sequence: (steps: ["F13", "Ctrl+F14"], max_gap_ms: 500), // ≥ 2 steps; max_gap_ms = 500, 1…10000 (§8)
            chord:    (keys: ["F15", "F16"], window_ms: 50),         // ≥ 2 keys, none overlapping; window_ms = 50, 1…10000 (§9)

            action: ACTION,                        // required (§13)

            trigger: Immediate | Tap(ms) | Hold(ms) | DoubleTap(ms),  // = Immediate; ms 1…10000; deferred ones only with keys (§11)
            when: COND,                            // = Always (§12)
            block_default: true | false,           // = true, but false for Tap/Hold/DoubleTap on modifiers only; keys only (§10)
            block_original_combo: true | false | MaskOnly,  // = false; only for SwitchLanguage*; MaskOnly is experimental (§15)
            allow_extra_modifiers: true | false,   // = false; keys only: fire with other modifiers held too (§7)
            repeat: true | false,                  // = true for PressKey, Media(VolumeUp/VolumeDown), else false; keys + Immediate only (§11.3)
        ),
    ],
)

ACTION := SwitchLanguage | SwitchLanguageBackward | ToggleCapsLock | DoNothing
        | PressKey("key")
        | PostMessage(msg: "WM_NAME" | u32, wparam: u64 = 0, lparam: i64 = 0)
        | Launch(exe: "name or path", args: ["arg", …] = [])
        | OpenUrl("URL, file, folder or program")
        | Media(PlayPause | Stop | Next | Previous | VolumeUp | VolumeDown | VolumeMute)
        | Sequence([STEP, …])                     // macro on a separate thread (§14)

STEP   := Window(Minimize | Maximize | Restore | Close) | Delay(ms)
        | Text("any text") | Key("key") | Combo("combo")
        | Launch(exe: …, args: […]) | OpenUrl("…") | Media(…)

COND   := Always | TitleContains("…") | TitleEquals("…") | AppEquals("x.exe")
        | Not(COND) | And([COND, …]) | Or([COND, …])

combo   := key ( "+" key )*                  // case and spaces do not matter; order does not matter
key     := A…Z | 0…9 | F1…F24 | Num0…Num9 | Shift | Ctrl | Alt | Win | CapsLock | Esc
         | Enter | Space | Tab | Up | PageUp | ` - = , . ; / [ ] \ ' | VolumeUp | …   (§6.4)
         // Shift, Ctrl, Alt, Win = either side; LShift, RCtrl, RWin … = that side only (§6.7)
         // modifiers match exactly: "CapsLock" does not fire while Shift is held (§7)
ms, u64 := 200 | 0xC8 | 0b11001000 | 0o310 | 1_000                                (§3.7)
string  := "…" (with escapes \\ \" \n \u{…}) | r"…" | r#"…"#                    (§3.9)
```

Sections referenced in the sketch: [§3.7](ron-syntax.md#s3-7) integers, [§3.9](ron-syntax.md#s3-9) strings and escapes, [§4](structure.md#s4) top level, [§5.2](structure.md#s5-2) exactly one kind, [§6.4](keys.md#s6-4) full list of key names, [§6.7](keys.md#s6-7) left and right modifiers, [§7](kinds.md#s7) `keys`, [§8](kinds.md#s8) `sequence`, [§9](kinds.md#s9) `chord`, [§10](kinds.md#s10) suppression and `block_default`, [§11](triggers-conditions.md#s11) triggers, [§12](triggers-conditions.md#s12) conditions, [§13](actions.md#s13) actions, [§14](actions.md#s14) the `Sequence` macro, [§15](matching.md#s15) auto-blockers.

> [!NOTE]
> Phase 0 removed the traps the audit version of this sketch marked: `AppEquals` matches ([KS-03](../audit.md#ks-03)), sided names such as `LShift` and `RWin` work in `keys`, `sequence` and `chord` ([KS-07](../audit.md#ks-07)), and a file saved with a BOM loads ([KS-24](../audit.md#ks-24)). [§19](known-issues.md#s19) lists what is still open.

The rules that are broken most often:

- RON field and variant names (`keys`, `action`, `SwitchLanguage`, `Tap`, `TitleContains`) are case-sensitive. Key names inside strings are not.
- An unknown or misspelled field is a load error that names the field ([§5.3](structure.md#s5-3)).
- Modifiers match exactly: `"CapsLock"` does not fire while Shift, Ctrl, Alt or Win is held, and `"Ctrl+J"` does not fire on Ctrl+Shift+J. Add `allow_extra_modifiers: true` to relax this for one binding ([§7](kinds.md#s7)).
- A binding fires only on the key-down of one of its own keys: Shift, then CapsLock fires `"Shift+CapsLock"`, CapsLock, then Shift does not ([§7](kinds.md#s7)).
- A single `\` in a normal string starts an escape: write `"C:\\dir"` or `r"C:\dir"` ([§3.10](ron-syntax.md#s3-10)).
- One invalid binding rejects the whole file. At startup this is fatal; on reload the previous bindings stay live ([§2.3](structure.md#s2-3)).
- Keys that arm `Tap`/`Hold`/`DoubleTap`, and the keys of a `sequence` or `chord`, are suppressed and do not reach applications. There is no replay buffer ([§10](kinds.md#s10)).
