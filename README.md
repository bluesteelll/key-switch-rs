# key-switch-rs

Native Windows application in Rust for key remapping via Windows API bindings.

## Features

Default bindings (from the auto-generated `config.ron`):

- **CapsLock** -> switch input language (suppresses the OS Alt+Shift/Ctrl+Shift)
- **Shift + CapsLock** -> toggle CapsLock on/off

All bindings live in `config.ron` next to the executable. See
[Configuration](#configuration).

## Project Architecture

Modular architecture:

```text
key-switch-rs/
├── build.rs                - emits /MANIFESTUAC linker arg for the daemon binary only
├── tests/ipc_roundtrip.rs  - pipe server and swch client over a real pipe, no keyboard hook
└── src/
    ├── lib.rs              - public crate surface (so swch binary can import shared modules)
    ├── bench.rs            - test-only latency bench (one ignored test, run by hand)
    ├── ipc.rs              - IPC protocol and the swch client (bounded connect, 5 s reply deadline)
    ├── bin/
    │   └── swch.rs         - swch CLI driver binary
    ├── config/
    │   ├── mod.rs              - public API: ConfigPath, load(), reload()
    │   ├── schema.rs           - serde structs for the on-disk shape
    │   ├── parsing.rs          - combo string and WM_* name parsers
    │   ├── loader.rs           - file I/O, default-config generation, conversion to Vec<Binding>
    │   ├── testing.rs          - test-only temporary folders for the loader and watcher tests
    │   └── default_config.ron  - embedded RON template, written on first run
    ├── core/
    │   ├── app.rs              - main application with binding management
    │   ├── constants.rs        - shared constants and per-process injected-event sentinel
    │   ├── executor/           - the action-executor thread: Launch, OpenUrl and Sequence off the hook
    │   │   ├── mod.rs          - queue, STA COM, message loop, Delay timer, epochs, Runner
    │   │   ├── runs.rs         - running macros: steps up to each Delay, deadlines, cancellation
    │   │   └── testing.rs      - test-only recording runner
    │   └── windows_actions.rs  - BindAction enum, FastAction (what runs on the hook thread)
    ├── data/
    │   ├── binding.rs          - Binding struct (key combo -> action)
    │   ├── condition.rs        - when: conditions, evaluated against a ForegroundInfo
    │   ├── key_combination.rs  - key combinations (key + modifiers)
    │   ├── key_spec.rs         - trigger keys: one key on its side, or a modifier on either side
    │   ├── sequence.rs         - macro engine: SequenceStep, type_text, press_combo, window ops
    │   ├── trigger.rs          - Immediate, Tap, Hold, DoubleTap
    │   └── vk_name.rs          - VIRTUAL_KEY <-> human-readable name (both directions)
    ├── engine/
    │   ├── mod.rs              - pure decision core: Engine::on_key / on_timer -> Decision
    │   ├── env.rs              - Env trait: clock, OS key state, foreground app and title
    │   ├── keys.rs             - held and blocked keys, modifier resync
    │   ├── matching.rs         - compares a binding's keys with the keys held
    │   ├── gestures.rs         - pending Tap / Hold / DoubleTap: deadlines and timer tokens
    │   ├── schedule.rs         - Scheduler trait and TimerQueue: where the engine's timers go
    │   ├── chords.rs           - pending simultaneous chords
    │   ├── sequences.rs        - pending leader-key sequences
    │   ├── testing.rs          - test-only scenario harness (RON + key script + fake clock)
    │   ├── scenarios.rs        - scenario tests for every binding kind and trigger
    │   └── properties.rs       - test-only property tests of the suppression invariant
    ├── hook/
    │   ├── keyboard_hook.rs    - Windows API hook wrapper, ArcSwap binding list, remapping switch
    │   ├── keyboard_hook_callback.rs - adapter: Win32 event -> engine -> Decision; foreground-change hook
    │   ├── control_window.rs   - hidden control window: session notifications, session end
    │   ├── main_thread.rs      - the main thread: engine, gesture timers, main loop, swch on/off, reloads, input signals, hook reinstall
    │   ├── health.rs           - the keyboard callback's heartbeat, what the watchdog asks the main thread
    │   ├── installer.rs        - the Win32 calls that install hooks and raw input, compiled only outside tests
    │   ├── watchdog/           - the hook watchdog: raw keyboard input against the heartbeat (mod.rs), the pure decision (decision.rs)
    │   ├── config_watcher.rs   - notify-driven hot-reload (debounce + atomic swap)
    │   └── ipc_server.rs       - named-pipe listener (explicit SDDL); on/off/exit/status commands
    ├── platform/
    │   ├── desktop.rs          - whether this desktop has keyboard input (UOI_IO)
    │   ├── foreground.rs       - the foreground window for when: executable cached per window, title read live
    │   ├── priority.rs         - THREAD_PRIORITY_HIGHEST for the main thread while it runs its loop
    │   └── timers.rs           - high-resolution waitable timer: the executor's Delay and the gesture timers
    ├── system/
    │   ├── registry.rs         - Windows registry hotkey reader
    │   ├── single_instance.rs  - one daemon per session: the Local\key-switch-rs mutex
    │   └── system_function.rs  - Windows system functions
    └── main.rs                 - entry point: single-instance guard, loads config, builds App
```

### Modules

#### `key_combination.rs`

`KeyCombination` struct - represents key combination:

- Key list (`Vec<KeySpec>`) with dedup: each key is one key on its side
  (`LShift`) or a modifier on either side (`Shift`), see `data/key_spec.rs`
- Matching against the keys held is the engine's job (`engine/matching.rs`)

#### `windows_actions.rs`

`BindAction` enum - actions executed on binding trigger:

- `SwitchLanguage` - switch to next keyboard layout
- `SwitchLanguageBackward` - switch to previous layout
- `ToggleCapsLock` - toggle CapsLock state
- `PressKey(VIRTUAL_KEY)` - emulate key press
- `PostMessage { ... }` - send message to active window
- `Sequence(Arc<Vec<SequenceStep>>)` - macro steps, run on the action executor
- `Launch { exe, args }` - start a process
- `OpenUrl(String)` - open a URL, file or program with its default handler
- `Media(MediaKey)` - emulate a media or volume key press
- `DoNothing` - no action (for auto-blockers)

#### `binding.rs`

`Binding` struct - links a trigger with an action:

- `kind` (`BindingKind`: `Combo` / `Sequence` / `Chord`)
- `action` (`BindAction`)
- `block_default`: block default behavior flag
- `block_original_combo`: the auto-blocker this binding asks for, for the original system combination (`Option<BlockerMode>`: `true` is `Swallow`, `MaskOnly` is `MaskOnly`, `false` is `None`)
- `condition` (`Condition`, the `when:` field)
- `trigger` (`Trigger`: `Immediate` / `Tap` / `Hold` / `DoubleTap`)
- `allow_extra_modifiers`
- `repeat`: whether an `Immediate` `keys` binding fires again on each auto-repeat
- crate-private `auto_blocker` (`Option<BlockerMode>`), set only on the auto-blockers the loader adds

#### `keyboard_hook.rs`

Windows API wrapper:

- Installs the low-level keyboard hook (`WH_KEYBOARD_LL`), the foreground-change hook (`SetWinEventHook` / `EVENT_SYSTEM_FOREGROUND`) and the desktop-switch hook (`EVENT_SYSTEM_DESKTOPSWITCH`) once, on the main thread, and removes them at shutdown
- Bindings list in `ArcSwap<Vec<Binding>>` — lock-free reads on the hot path, atomic swap for hot-reload
- Holds the remapping switch that `swch on` / `swch off` flip (`is_enabled`); only the main thread flips it, together with a reset of the decision core (see `main_thread.rs`)
- Holds the action executor's handle while the hooks are installed (`attach_executor`) and hands it `Launch`, `OpenUrl` and `Sequence` (`submit`); a reload stops every running macro too. The daemon swaps the list on its main thread, together with a reset of the decision core (see `main_thread.rs`)

#### `keyboard_hook_callback.rs`

- The adapter between Win32 and the decision core in `src/engine`: translates each `WH_KEYBOARD_LL` event into an `engine::KeyEvent`, asks `Engine::on_key` for a `Decision` on the main thread's state, hands the timers it asks for to the main thread's gesture timers, and carries it out (sends the menu mask key if asked, runs the fast actions and queues the slow ones, then swallows or passes the event). It decides nothing itself
- Runs only the fast actions itself, before it returns: `PressKey`, `Media`, `ToggleCapsLock`, `SwitchLanguage`, `SwitchLanguageBackward`, `PostMessage`, so their output stays in order with the keys typed after them. `Launch`, `OpenUrl` and `Sequence` go to the action executor without waiting (KS-08)
- Sends the menu mask key, virtual-key code 0xE8 down and up with the sentinel, before it returns when the decision asks for it: the event is swallowed while Windows has Win or Alt down, so the Win or Alt release opens neither Start nor the menu bar (KS-12)
- While remapping is off (`swch off`), passes every event straight to `CallNextHookEx` without reading it
- Answers the engine's `Env`: `GetAsyncKeyState` for the modifier re-sync on every event and on foreground change, to repair state lost while a higher-integrity window (Task Manager, UAC consent, lock screen, etc.) had focus, and for the key of a key-up it would swallow; the foreground window's executable for `when:` from the main thread's per-window cache, with no Win32 call; its title read live, only when a title condition asks, and never for a window of the daemon's own process
- Hands each foreground change to the main thread's cache, which reads the new window's executable name there, outside the keyboard callback (roadmap item 2.3)
- Skip injected events generated by this process (PID-mixed sentinel in `dwExtraInfo`) before the engine sees them
- Records how long each keyboard callback ran; the longest run since the last report ends the log's counters line, which it triggers on its own from 5 ms

#### `main_thread.rs`

The daemon's main thread (roadmap item 2.2):

- Owns the decision core (`engine::Engine`: held and blocked keys, pending gestures) and its gesture timers in a thread-local reached only through `with_main`; the hook callbacks run on this thread, so no lock guards the engine, and a callback re-entered while the state is borrowed passes its event instead of panicking or hanging
- Runs the main loop: it waits for its gesture timer and its messages at once (`MsgWaitForMultipleObjectsEx`), pumps with `PeekMessageW`, in which the hook callbacks run, and fires every gesture timer that is due. The timer is one high-resolution waitable timer, so a `Hold` fires within about 1 ms of its term; no thread is started per press (KS-33)
- A gesture timer fires on this thread: a fast `Hold` action runs here, and a slow one goes to the action executor
- Serves `swch on` / `swch off`: the IPC worker queues a request and waits at most 2 s for the answer; the main thread flips the switch and resets the decision core between two key events (pending gestures are dropped and the keys held are re-read from Windows; a key whose press was swallowed keeps its release swallowed) and stops every running macro and queued `Launch`/`OpenUrl` on the action executor
- Serves config reloads the same way: the watcher thread hands over the new list, and the main thread swaps it in and drops the pending gestures in one step (roadmap item 2.3)
- Answers the input signals that the control window (a lock, an unlock) and the desktop-switch hook (Ctrl+Alt+Del, a UAC prompt on the secure desktop, and back) note: when keyboard input leaves the desktop it drops the pending gestures, stops the running macros and releases every held key, so the next press of each key is a press; a signal that input came back only reads the foreground window again. Each signal is logged (roadmap item 2.3, KS-29)

#### `app.rs`

Main application:

- Manage bindings via builder pattern
- Install keyboard + foreground-change + desktop-switch hooks
- Creates the hidden control window (`hook/control_window.rs`) before it installs the hooks, and destroys it right after it removed them: it receives the session notifications (`WTSRegisterSessionNotification`) and the session end; at sign-out the daemon removes its hooks, writes the log and exits
- Creates the IPC pipe before it installs the hooks; a pipe it cannot create stops the daemon (KS-20)
- Starts the action executor before it installs the hooks, and stops it after it removed them, waiting at most 2 s for a step that still runs
- Sets up the main thread's engine and gesture timer before it installs the hooks; a gesture timer it cannot create stops the daemon
- Ctrl+C handler for correct shutdown
- Runs the main loop (`main_thread.rs`) on the thread that installed the hooks, at `THREAD_PRIORITY_HIGHEST` while it runs (every keystroke waits for that thread), and at normal priority again for the shutdown; a priority it cannot set is logged, and the daemon runs on

#### `executor/`

The `action-executor` thread (roadmap item 2.1, KS-08):

- Runs `Launch`, `OpenUrl` and `Sequence`, which the main thread queues without waiting, from the hook callback or, for a `Hold`, from its main loop; up to 32 wait, and a further one is dropped and counted (`executor_jobs_dropped`)
- Initialises COM as a single-threaded apartment, as `ShellExecuteW` asks, and pumps its own messages, waiting for them and for its `Delay` timer at once
- Runs macros side by side on its one thread: a macro goes up to its next `Delay`, which waits on one high-resolution timer, within about 1 ms as `thread::sleep` did, and holds up no other action or macro. A step that blocks inside Windows (a `Launch`, an `OpenUrl`, a dialog `ShellExecuteW` shows) holds up the others while it runs
- `swch off`, `swch on` (a real change), a reload and exit stop every running macro within 100 ms, also inside a `Delay`, and drop the actions still queued; a step already running finishes first
- An executor that stopped on its own (it logs one `[ERROR]` line) makes every later `Launch`, `OpenUrl` and `Sequence` count in `executor_unavailable` until the daemon restarts

## Configuration

Bindings are defined in `config.ron` ([RON — Rust Object Notation](https://github.com/ron-rs/ron)).
The format is enum-variant-literal-syntax — `Sequence([Window(Restore), Delay(100), Text("hi")])` —
that maps 1:1 to the program's internal Rust enums.

On first run the program writes a self-documenting default file next to the
executable, then loads it. **Bindings are hot-reloaded automatically** — edit
`config.ron` in any editor, save, and the new bindings go live within ~250 ms
without a restart. The daemon's main thread swaps them in between two key
events and drops, in the same step, every `Tap`, `Hold`, `DoubleTap`,
`chord` and `sequence` still pending: a `Tap` armed before the save fires
nothing at its release (roadmap item 2.3, KS-31). The keys held stay held,
and a key whose press the old bindings swallowed keeps its release
swallowed (known issue 77).

If the file does not load (a syntax error, an unknown or misspelled field,
an unknown key name, a timing value outside 1 to 10000 ms, ...) or is
deleted or renamed, the error is written to the console and to the
[log file](#log-file), and the **previously-running
bindings stay live** — a malformed save never
bricks the daemon. Fix the file and save again to apply. The file is UTF-8,
with or without a BOM.

### Where the file lives

Resolution order:

1. **Explicit path** as the first CLI argument:
   `key-switch-rs.exe C:\path\to\my-config.ron`. A relative path is resolved
   once at startup against the daemon's current folder (for a Scheduled
   Task, its "Start in" folder); the console shows the full path. The file
   must exist: a missing file stops the daemon with
   `config file <path> does not exist`, and nothing is created.
2. **`<exe_dir>\config.ron`** (portable default — drop the .exe anywhere and
   its config lives beside it)
3. **If `<exe_dir>\config.ron` is missing at startup**, the default template
   is written there and used.

A file deleted or renamed while the daemon runs is not recreated: the
bindings loaded last stay live, and putting the file back reloads it.

Hot reload watches the file's folder and reacts to the file's name in any
letter case.

### Schema

```ron
// The first two bindings are the auto-generated default.
(
    bindings: [
        (
            keys: "CapsLock",
            action: SwitchLanguage,
            block_original_combo: true,
        ),
        (
            keys: "Shift+CapsLock",
            action: ToggleCapsLock,
        ),
        (
            keys: "Ctrl+Alt+L",
            action: PressKey("L"),
        ),
        (
            keys: "Win+Q",
            action: PostMessage(msg: "WM_CLOSE"),
        ),
    ],
)
```

RON supports `//` and `/* ... */` comments and trailing commas, so the file
stays human-editable.

### Combo syntax (the `keys` field)

`+`-joined list of segments. Whitespace around segments is ignored and matching
is case-insensitive (`"Ctrl + shift + esc"` parses identically to `"Ctrl+Shift+Esc"`).

| Group       | Recognised names                                                                                       |
| ----------- | ------------------------------------------------------------------------------------------------------ |
| Modifiers   | Either side: `Shift`, `Ctrl`/`Control`, `Alt`/`Menu`, `Win`/`Super`. One side: `LShift`/`LeftShift`, `RShift`/`RightShift`, `LCtrl`/`LeftCtrl`/`LeftControl`, `RCtrl`/`RightCtrl`/`RightControl`, `LAlt`/`LeftAlt`, `RAlt`/`RightAlt`, `LWin`/`LeftWin`, `RWin`/`RightWin` |
| Letters     | `A`–`Z`                                                                                                |
| Digits      | `0`–`9`                                                                                                |
| Function    | `F1`–`F24`                                                                                             |
| Numpad      | `Num0`–`Num9` (also `Numpad0`–`Numpad9`)                                                               |
| Named       | `CapsLock`/`Caps`, `Tab`, `Esc`/`Escape`, `Enter`/`Return`, `Space`/`Spacebar`, `Backspace`/`Back`, `Delete`/`Del`, `Insert`/`Ins`, `Home`, `End`, `PageUp`/`PgUp`, `PageDown`/`PgDn`, `Up`/`UpArrow`, `Down`/`DownArrow`, `Left`/`LeftArrow`, `Right`/`RightArrow`, `NumLock`, `ScrollLock`/`Scroll`, `Pause`/`Break`, `PrintScreen`/`PrtSc`/`PrtScn` |
| Media       | `MediaPlayPause`/`PlayPause`, `MediaStop`, `MediaNext`/`NextTrack`, `MediaPrev`/`MediaPrevious`/`PrevTrack`, `VolumeUp`/`VolUp`, `VolumeDown`/`VolDown`, `VolumeMute`/`VolMute`/`Mute` |
| OEM punct   | `` ` ``, `-`, `=`, `,`, `.`, `;`, `/`, `[`, `]`, `\`, `'`                                                |

**Sided modifiers.** `Shift`, `Ctrl`, `Alt` and `Win` (`Super`) match either
key of the pair; `LShift`, `RShift`, `LCtrl`, `RCtrl`, `LAlt`, `RAlt`, `LWin`
and `RWin` match only that key. This holds wherever a key triggers a binding:
`keys`, `sequence` steps and `chord` keys. `"LShift+F13"` fires with the left
Shift held and not with the right one; `"Win+E"` fires with either Win key,
`"LWin+E"` only with the left one. Keys the daemon sends (`PressKey`, and
`Key` and `Combo` in a macro) are not affected: `Win` there sends the left Win
key.

**Which key fires a binding.** A `keys` binding fires, or arms its `Tap`,
`Hold` or `DoubleTap`, only on the key-down of one of its own keys: a
non-modifier key (`CapsLock` in `"Shift+CapsLock"`, `J` in `"Ctrl+J"`), or any
of its keys when the binding is made only of modifiers (`"Alt+Shift"`,
`"LShift"`). The modifiers go down first: Shift, then CapsLock fires
`"Shift+CapsLock"`, while CapsLock, then Shift does not, and J, then Ctrl does
not fire `"Ctrl+J"`. A key pressed while a binding's key is held is not that
binding's key, so with CapsLock held, typing `abc` delivers `abc`. The order of
the names in the string does not matter.

**Exact modifiers.** A `keys` binding fires only when the modifiers held
(Shift, Ctrl, Alt and Win; either key of a pair counts as that modifier) are
exactly the ones it names. `"CapsLock"` does not fire while Shift is held, and
`"Ctrl+J"` does not fire on Ctrl+Shift+J, so that shortcut reaches the
application. The binding's other keys must be held too; other non-modifier
keys held at the same time do not matter. `allow_extra_modifiers: true` lets a
binding fire while modifiers it does not name are held as well. It is valid
on `keys` bindings only and a load error on `sequence` and `chord`, whose
steps and keys match whatever modifiers are held.

```ron
// Either Shift with CapsLock, and no other modifier: not Ctrl+Shift+CapsLock.
( keys: "Shift+CapsLock", action: ToggleCapsLock ),
// The left Shift with F13; holding the right Shift as well still counts as Shift.
( keys: "LShift+F13", action: SwitchLanguage ),
// CapsLock alone, or with any modifiers held.
( keys: "CapsLock", action: SwitchLanguage, allow_extra_modifiers: true ),
```

On keyboard layouts with an AltGr key (German, Polish and others), the right
Alt key arrives together with a left Ctrl. A binding on `RAlt` or `Alt` then
does not fire on AltGr: write `Ctrl+RAlt` or set `allow_extra_modifiers: true`.

**Win and Alt.** When a binding swallows a key pressed while Win or Alt is
held, as `"Win+Q"` or `"Alt+J"` do, Windows has seen Win or Alt go down and
nothing after it, and releasing it would open the Start menu or activate the
menu bar. The daemon prevents that: before it swallows such a key, its
auto-repeats or its release, it sends the key code 0xE8 down and up, as
AutoHotkey's `#MenuMaskKey` does. Windows assigns 0xE8 to nothing, so it
types nothing, but other programs that watch the keyboard see it. A `Tap`,
`Hold` or `DoubleTap` on `Alt` or `Win` alone that lets the key through is
not covered (see the behaviour rules under [Triggers](#triggers-trigger-field)).

**Which binding wins.** When several `keys` bindings match one key press, one
of them wins, and only the winner runs its action and decides whether the key
is swallowed. A binding that loses neither fires, nor arms its `Tap`, `Hold`
or `DoubleTap`, nor swallows the key. The binding with more keys wins; with
equal key counts, one with a `when:` condition beats one without; then the one
higher in the file. `Tap`, `Hold` and `DoubleTap` bindings on the same keys
with the same `when:` (written the same way) win or lose together, so a `Tap`
and a `Hold` on CapsLock both arm (see [Triggers](#triggers-trigger-field));
of two such bindings with the same trigger, only the first acts. An
`Immediate` binding and a `Tap`, `Hold` or `DoubleTap` binding on the same
keys and `when:` never act on the same press: the one higher in the file wins
every press. The election runs again on every auto-repeat. Auto-blockers,
`sequence` and `chord` bindings keep their own rules next to the winner: a key
that is also a sequence step or a chord key still drives the sequence or the
chord, which swallows the key when it advances. The loader warns about a
binding that duplicates one above it (see [Errors](#errors)).

```ron
// Shift+CapsLock has more keys: it wins, and the Tap does not arm with it.
( keys: "Shift+CapsLock", action: ToggleCapsLock ),
( keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true ),
// In Spotify the binding with a when: wins, wherever it sits in the file.
( keys: "Pause", action: SwitchLanguage ),
( keys: "Pause", action: Media(PlayPause), when: AppEquals("Spotify.exe") ),
```

### Action variants

| Variant                              | Purpose                                                                  |
| ------------------------------------ | ------------------------------------------------------------------------ |
| `SwitchLanguage`                     | Switch to the next keyboard layout                                       |
| `SwitchLanguageBackward`             | Switch to the previous layout                                            |
| `ToggleCapsLock`                     | Toggle CapsLock state                                                    |
| `PressKey("<name>")`                 | Synthesize a key press via `SendInput`                                   |
| `PostMessage(msg: ..., wparam: ..., lparam: ...)` | `PostMessage` to the foreground window                      |
| `Launch(exe: "<path>", args: [...])` | Spawn a process. `args` defaults to `[]`. Until roadmap item 3.3 the program gets the daemon's administrator rights (release build) and, for a console program, its console (KS-09, KS-34). |
| `OpenUrl("<url-or-path>")`           | Hand to OS default handler via `ShellExecuteW`. Works for URLs, `mailto:`, file paths, even .exe. |
| `Media(<key>)`                       | Synthesize a media/volume key. `<key>` is one of `PlayPause`, `Stop`, `Next`, `Previous`, `VolumeUp`, `VolumeDown`, `VolumeMute`. |
| `Sequence([...])`                    | Chain multiple steps (text, key presses, delays, window ops) — see below |
| `DoNothing`                          | Match the combo and drop it (no side effects)                            |

`PostMessage` arguments:

- `msg` is either a symbolic name (string) like `"WM_CLOSE"` or a numeric code like `0x10`. Recognised names: `WM_CLOSE`, `WM_QUIT`, `WM_DESTROY`, `WM_COMMAND`, `WM_SYSCOMMAND`, `WM_INPUTLANGCHANGEREQUEST`, `WM_KEYDOWN`, `WM_KEYUP`, `WM_SYSKEYDOWN`, `WM_SYSKEYUP`, `WM_HOTKEY`. Anything else: pass a number.
- `wparam` and `lparam` are optional, both default to 0.

### Binding kinds: `keys`, `sequence`, `chord`

Each binding specifies **exactly one** of three trigger shapes:

| Field          | Shape                                                                                                  | Fires when                                                              |
| -------------- | ------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------- |
| `keys: "..."`  | Plain combo string (e.g. `"Ctrl+Alt+L"`).                                                              | All keys of the combo are held, on the key-down of one of its own keys (see "Which key fires a binding" above). |
| `sequence: (steps: [...], max_gap_ms: 500)` | Vim/Emacs-style ordered sequence. Each step is a combo string. `max_gap_ms` (default 500, 1 to 10000) is the maximum time between consecutive steps. | Steps pressed in order, each within `max_gap_ms` of the previous.       |
| `chord: (keys: [...], window_ms: 50)`       | Simultaneous chord. `window_ms` (default 50, 1 to 10000, typical QMK COMBO_TERM) is how close together the presses must be. | All listed keys go down within `window_ms` of each other, in any order. |

Examples:

```ron
// Classic combo
( keys: "Ctrl+Alt+L", action: PressKey("L") )

// Sequence: press F13, then F13 again within 500ms → Home
( sequence: (steps: ["F13", "F13"], max_gap_ms: 500),
  action: PressKey("Home") )

// Chord: press F13 + F14 within 50ms → Escape
( chord: (keys: ["F13", "F14"], window_ms: 50),
  action: PressKey("Escape") )

// Multi-step Emacs-style save (Ctrl held throughout)
( sequence: (steps: ["Ctrl+X", "Ctrl+S"], max_gap_ms: 1000),
  action: ToggleCapsLock )
```

**Important caveat — suppression-trap:**

Sequence and Chord bindings **suppress** their trigger keys from the foreground app while waiting for the sequence/chord to complete. If the sequence times out or the user presses an unexpected key, the suppressed presses are **lost** — there's no replay buffer. Bind sequences/chords to keys you don't use in normal typing (e.g. function keys F13–F24, or rarely-used punctuation), otherwise text input will randomly drop characters.

**Restrictions:**

- Sequence requires at least 2 steps. For a single combo use plain `keys: "..."` instead.
- Chord requires at least 2 keys. A single key is not a chord.
- Chord keys cannot repeat or overlap: `chord: (keys: ["j", "j"])` and
  `chord: (keys: ["Shift", "LShift"])` are rejected, because one press would
  count for both keys. A modifier in a chord is swallowed on every press,
  like every other chord key.
- Sequence and Chord **cannot** be paired with deferred `trigger: Tap/Hold/DoubleTap` — they have their own temporal semantics. Only `trigger: Immediate` (the default) is valid.

### Triggers (`trigger:` field)

A binding's `trigger:` decides *when* its action fires given the combo has
matched. Default is `Immediate` (fire on key-down — the existing behaviour
of every binding before triggers existed). Deferred triggers — `Tap`,
`Hold`, `DoubleTap` — defer to an asynchronous gesture state machine.

**The whole point**: two bindings on the same combo and `when:` with
different triggers coexist. They win a press together (see
[Which binding wins](#combo-syntax-the-keys-field)), Tap and Hold both arm
on key-down, as one gesture, and exactly one of them fires (see below).

```ron
// CapsLock — quick press switches language, holding sends one LShift press.
// Replace the default CapsLock binding with these two: an Immediate binding
// listed above them on the same key wins every press.
( keys: "CapsLock", action: SwitchLanguage,   trigger: Tap(200), block_original_combo: true ),
( keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200) ),

// LShift — double tap toggles CapsLock; the left Shift still types capitals
( keys: "LShift",   action: ToggleCapsLock,   trigger: DoubleTap(250) ),
```

A deferred binding made only of modifiers, such as the `LShift` one, lets
its key through by default (`block_default`, see the behaviour rules
below): every left-Shift press still reaches applications, and the double
tap still toggles CapsLock. The `CapsLock` pair, whose key is not a
modifier, swallows CapsLock. Another key pressed in between cancels the
double tap, so typing `<T>` with the left Shift pressed again quickly for
the `T` types two capitals and leaves CapsLock alone. Write
`block_default: true` to keep the left Shift for the gesture alone.

| Variant            | When the action fires                                                                  |
| ------------------ | -------------------------------------------------------------------------------------- |
| `Immediate`        | Synchronously on key-down (default — same as no `trigger:` field); on an auto-repeat again only if `repeat` is on (see [Per-binding flags](#per-binding-flags)). |
| `Tap(<ms>)`        | On key-up, but only if the key was released within `<ms>` of being pressed.            |
| `Hold(<ms>)`       | After the key has been held for `<ms>` without being released.                         |
| `DoubleTap(<ms>)`  | On the *second* key-down within `<ms>` of the first. Single press emits nothing.       |

`<ms>` is 1 to 10000; another value is a load error.

**How resolution works (under the hood):**

- Key-down of one of the binding's own keys (see "Which key fires a
  binding"): the winning group (see "Which binding wins") arms its gestures
  (`Tap`, `Hold`, `DoubleTap`), one per trigger, as one press in the
  decision core (`src/engine/gestures.rs`) under that key, and that key's
  key-up resolves it: exactly one of the gestures fires (KS-25). A key
  pressed while the gesture is pending passes and arms nothing.
- For `Tap`: the entry stores a deadline `<ms>` after the key-down; the
  key-up fires the action only before the deadline. No thread is involved.
- For `Hold`: arming asks the daemon's main thread for a timer that fires
  the action after `<ms>`, within about 1 ms (a high-resolution waitable
  timer, not the 15.6 ms system tick); no thread is started. A fast action
  runs on the main thread, and `Launch`, `OpenUrl` and `Sequence` go to
  the action executor. A key-up before `<ms>` cancels it; a key-up at or
  after `<ms>` leaves it to the timer, so a key held for the whole term
  fires its Hold even if the timer runs late. Each timer carries the
  number of the press that started it, so a timer from an earlier press
  never acts on a newer one, nor one from before a `swch off`.
- For `DoubleTap`: the first key-down arms the window with a deadline,
  and a key-up before it opens the window; a second key-down before the
  deadline that the same group (the same keys and `when:`) wins fires the
  action, and that press arms nothing else. A press that another group
  wins, such as `Shift+CapsLock` after a `CapsLock` press, ends the window.
- `Tap` and `Hold` of one press: the Hold resolves the press once its term
  has passed, so with `Tap(300)` + `Hold(200)` a release at 250 ms fires
  only the Hold. With equal terms (`Tap(200)` + `Hold(200)`): `Tap` if the
  key is released before 200 ms, `Hold` otherwise. With `Tap(100)` +
  `Hold(200)` a release between 100 and 200 ms fires nothing, so prefer
  equal terms.
- `Tap` and `DoubleTap` on the same keys: the `Tap` waits until the
  DoubleTap window closes, so a double press fires only the `DoubleTap`.
  A single press fires the `Tap` at the press plus the DoubleTap term, or
  at once when another key goes down first (ahead of that key). Only a
  group with both pays that latency.

**Behaviour rules:**

- `block_default` applies to deferred triggers too (roadmap item 1.3).
  With `true` the key's press, auto-repeats and release are swallowed while
  the gesture is detected; with `false` they all reach Windows, and the
  gesture still arms and fires. Without the field, a `Tap`, `Hold` or
  `DoubleTap` binding made only of modifiers (`"LShift"`, `"Ctrl+Shift"`,
  `"Win"`) lets its keys through, and every other binding, every
  `Immediate` one included, swallows them. When several `keys` bindings
  match one press, only the winner decides (see "Which binding wins"); in a
  group of `Tap`, `Hold` and `DoubleTap` bindings the key is swallowed if
  any of them swallows it. Once a modifier is released under a held key,
  its repeats may elect another binding, which fires as that binding says,
  while the repeats themselves still follow the key-down: with
  `( keys: "J", action: PressKey("Down"), allow_extra_modifiers: true )` and
  `( keys: "Ctrl+J", action: SwitchLanguage, trigger: Tap(200), block_default: false )`,
  holding Ctrl+J and releasing Ctrl sends Down per repeat and lets `j`
  repeat too.
- Another key pressed while a gesture whose key passes is pending cancels
  it (owner decision, roadmap item 2.2), as AutoHotkey does: Shift+A
  released within an `LShift` `Tap`'s term fires nothing, the left Shift
  pressed twice for two capitals (`<T>`) fires no `LShift` `DoubleTap`, a
  quick Ctrl+C fires no `LCtrl` `Tap`, and selecting words with
  Ctrl+Shift+Right fires no `Ctrl+Shift` `Hold`. Only a key-down cancels:
  a mouse click does not, because the keyboard hook does not see it, so a
  quick Ctrl+click still fires an `LCtrl` `Tap`. A `Hold` whose term has
  passed is not pending any more and fires. A gesture whose key is
  swallowed is not cancelled: a key pressed while CapsLock of the CapsLock
  pair is held passes and the pair resolves as usual. A key of the
  gesture's own combination does not cancel a `DoubleTap` that waits for
  its second press (roadmap item 2.3): the second press of a `Ctrl+Shift`
  `DoubleTap` may begin with Ctrl. A `Tap` waiting for that window then
  fires at the next key outside the combination, with that modifier down
  in Windows: a `Ctrl+Shift` `Tap` of `PressKey("Escape")` next to a
  `Ctrl+Shift` `DoubleTap`, tapped once and followed by Ctrl+C within the
  window, sends Ctrl+Esc (known issue 68).
- A deferred binding on exactly the keys of the Windows language hotkey,
  such as `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200) )`,
  takes the hotkey over from the auto-blocker, also when it sets
  `block_original_combo: true` itself. Made only of modifiers, it lets
  Windows see the hotkey by default, so Windows switches the layout forward
  while the binding switches it back. Write `block_default: true` on it.
- A `Tap`, `Hold` or `DoubleTap` on `Alt` or `Win` alone lets the press
  reach Windows by default, so a tap also activates the menu bar or opens
  Start. Nothing is swallowed, so the menu mask key (see "Win and Alt"
  under [Combo syntax](#combo-syntax-the-keys-field)) does not help here.
  With `block_default: true` Windows never sees that key while the binding
  matches it, so the key's own shortcuts (`Win+E`) stop working.
- Auto-repeat key-down events from a stuck/held key do not stack new
  pending entries; the press's gestures stay as armed until it resolves.
  A press that follows a key-up the hook never saw replaces the stale one.
- An `Immediate` binding fires on an auto-repeat only if its `repeat` is
  on: by default for `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)`.
  Every other action fires once per press. Its repeats are still swallowed
  when its key-down was. `Tap`, `Hold`, `DoubleTap`, `sequence` and `chord`
  never fire on an auto-repeat.
- An auto-repeat is swallowed exactly when its key's first key-down was,
  for every binding kind. A key held past the repeat delay therefore does
  not leak repeats after a swallowed press (which would leave it held in
  Windows once its release is swallowed), and a key whose press reached
  Windows keeps its repeats and its release, even when a `chord` or
  `sequence` completes while it is held. The resync of modifiers from
  the Windows key state leaves alone a modifier whose key-down was
  swallowed, because Windows never saw it go down: it still counts as held
  for matching, and its auto-repeats stay auto-repeats (KS-14). The
  release of a key whose key-down was swallowed is swallowed too, across a
  foreground change and `swch off`/`swch on`, so Windows never gets a
  key-up whose press it did not see (roadmap item 2.3). It reaches Windows
  only when Windows has that key down, so the key does not stick (known
  issue 78). A key released while remapping is off reaches Windows as a
  lone key-up (known issue 59).
- Hold is "fire once" semantics, not "press-and-hold synthesized
  modifier". Binding `Hold → PressKey("LShift")` sends one Shift press at
  the threshold moment, not a held Shift. For continuously-held synthesized
  modifiers, a dedicated action would be needed (not yet implemented).

### `Sequence(...)` — chained steps (macros)

A `Sequence` action runs a list of steps on the action executor thread. The
keyboard hook callback only queues it, so even long sequences with
multi-second delays never block input system-wide. `swch off`, a reload
(a saved `config.ron` that loads) and `swch exit` stop a running macro
within 100 ms, also inside a `Delay`; a step already running (a `Launch`,
an `OpenUrl`) finishes first. A save that does not load keeps the running
macros, as it keeps the bindings.

```ron
(
    keys: "Ctrl+Alt+N",
    action: Sequence([
        Window(Restore),
        Delay(100),
        Text("Note: groceries 🛒"),
        Key("Enter"),
        Text("Date: 2026-05-24"),
        Combo("Ctrl+S"),
        Delay(500),
        Window(Minimize),
    ]),
)
```

Eight step variants:

| Step                  | Effect                                                                                                       |
| --------------------- | ------------------------------------------------------------------------------------------------------------ |
| `Delay(<ms>)`         | Wait `<ms>` milliseconds on the action executor's high-resolution timer, which ends the wait within about 1 ms of its time and never before. Blocks neither the hook nor other actions and macros; `swch off`, a reload or exit cut it short. |
| `Text("<string>")`    | Type the string via `SendInput` with `KEYEVENTF_UNICODE`. Layout-independent; emoji and surrogate pairs work. |
| `Key("<name>")`       | Synthesize a single key press (down + up). Same name syntax as `keys`.                                       |
| `Combo("<combo>")`    | Synthesize a chord: modifiers down → payload keys → reverse-order release. `"Ctrl+S"`, `"Alt+F4"`, etc.      |
| `Window(<op>)`        | Apply `<op>` to the captured target window. `<op>` is one of `Minimize`, `Maximize`, `Restore`, `Close`. It does not wait for the window (`ShowWindowAsync`, a posted `WM_CLOSE`): put a `Delay` after it when the next step depends on the change. |
| `Launch(exe: "<path>", args: [...])` | Spawn a process (same as the top-level action).                                               |
| `OpenUrl("<url>")`    | OS default handler (same as the top-level action).                                                           |
| `Media(<key>)`        | Synthesize a media key (same as the top-level action).                                                       |

**Target window** for `Window(...)`-steps is `GetForegroundWindow()` at the
moment the combo matched — *captured once* and reused for every step. So
"restore … type … minimize" operates on the same HWND throughout, even if
focus drifts midway.

**Parallel sequences:** macros run side by side on the one executor
thread, without a thread each. Two fast presses run two macros that
interleave at their `Delay` steps, oldest first; the steps between two
`Delay`s run without a break. For deterministic behaviour, put a trailing
`Delay` long enough to cover the previous run. A step that blocks inside
Windows (a `Launch`, an `OpenUrl`, a dialog `ShellExecuteW` shows) holds up
the other macros and actions while it runs.

**Errors** in step shape (unknown variant name, wrong arity) are caught at
parse time. Per-step value errors (unknown `Key`/`Combo` names) are reported
as `binding #N [keys]: step #M: ...` so the failing step is unambiguous.

### Per-binding flags

| Field                  | Default  | Meaning                                                                                                                                                                              |
| ---------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `block_default`        | `true`; `false` for a `Tap`, `Hold` or `DoubleTap` binding made only of modifiers | Suppress the original OS key events (the press, its auto-repeats and its release) so the foreground app does not also see them. Set to `false` to let the keystroke pass through: an `Immediate` binding still fires, and a `Tap`, `Hold` or `DoubleTap` binding still detects its gesture. When several `keys` bindings match one press, only the binding that wins it, or its `Tap`/`Hold`/`DoubleTap` group, counts (see [Which binding wins](#combo-syntax-the-keys-field)). `Some(..)` and `None` (the default) load too. `sequence` and `chord` bindings ignore it. |
| `block_original_combo` | `false`  | `true`, `false` or `MaskOnly` (experimental). Auto-install a blocker for the OS's default hotkey for this action's underlying function. Use when remapping language switching so Windows' own `Alt+Shift`/`Ctrl+Shift` go silent. The hotkey comes from `HKCU\Keyboard Layout\Toggle` (the value `Hotkey`, then `Language Hotkey`). When Windows has no language hotkey ("Not assigned" in its settings, value `"3"`), no blocker is added; `Alt+Shift` is assumed only when neither value names a hotkey or says `"3"`. The blocker acts only when Windows would see the hotkey's keys and no other key held: Alt, then Shift is blocked, while Alt, Tab, then Shift (Tab still held) reaches Windows. It is not added when any `keys` binding in the file, the flagged one included, uses exactly the hotkey's keys: that binding gets the hotkey, and where its `when:` does not match, the hotkey is not blocked. An auto-blocker that is added acts only where the `when:` of a binding that asked for it holds (KS-35): with `when: Not(AppEquals("game.exe"))` added to the default config's CapsLock binding, the Windows hotkey (`Alt+Shift` by default) reaches the game and switches the layout there, while CapsLock reaches it as CapsLock. Bindings that ask for a blocker of the same hotkey share one, which acts where any of their `when:` holds, and everywhere if one of them has no `when:`; their triggers are not inherited. The blocker never fires anything and never keeps a binding of yours that matches the same press from firing. Auto-blockers are listed last at startup. With `true` the blocker swallows the hotkey's second modifier (Shift pressed after Alt with no other key held, also when stepping back in the Alt+Tab switcher after releasing Tab) for that key's whole press, auto-repeats and release included, even after Alt is released; it still counts as held, so `Alt+Shift+X` bindings fire, and with Alt held the menu mask key keeps Alt's release from activating the menu bar. **`MaskOnly`** swallows nothing: the hotkey reaches Windows with the menu mask key (0xE8, see "Win and Alt" under [Combo syntax](#combo-syntax-the-keys-field)) sent on the key-down that completes it and again on the next key event, so `Ctrl+Shift+T`, `Alt+Shift+Tab` and `Alt+Shift+X` reach applications. It is experimental: it assumes that Windows switches the layout only on a clean press of its hotkey, which no test can check. Check it by hand (a bare `Alt+Shift` or `Ctrl+Shift` pressed and released several times must not switch the layout) and go back to `true` if it does. Startup lists such a blocker as `(masked, experimental)`. On a hotkey that is not made only of modifiers (the `` ` `` hotkey) `MaskOnly` blocks as `true` does, with a warning. |
| `allow_extra_modifiers` | `false` | `keys` bindings only: also fire while modifiers the binding does not name are held. By default the held modifiers must be exactly the binding's own (see [Exact modifiers](#combo-syntax-the-keys-field)). A load error on `sequence` and `chord`. |
| `repeat`               | `true` for `PressKey`, `Media(VolumeUp)`, `Media(VolumeDown)`; `false` otherwise | `keys` bindings with the `Immediate` trigger: fire again on every auto-repeat while the key is held. With `false` the action fires once per press; the repeats and the release are still swallowed when the press was. `repeat: true` is a load error on `Tap`, `Hold`, `DoubleTap`, `sequence` and `chord` bindings, which never fire on an auto-repeat; `repeat: false` is accepted there. The startup list marks a value that differs from the default with `[repeat]` or `[no repeat]`. Each repeat of a `PressKey` sends a new press and release, so for a toggle key (`CapsLock`, `NumLock`, `ScrollLock`) or a modifier write `repeat: false`: otherwise holding the trigger toggles the key at the repeat rate, or taps the modifier, where the real key held down toggles once or stays down. |
| `when`                 | `Always` | Foreground-window guard. The binding fires only when the predicate matches. See [`Conditions`](#conditions-when-field) below.                                                        |

### Conditions (`when:` field)

Gate a binding on what window is focused. The executable name comes from a
per-window cache that the daemon fills when a window takes the foreground,
so a key press queries no process; the title is read on a key press only
when a title condition asks, and the common case "no `when:` anywhere" pays
nothing (roadmap item 2.3). A program that activates a window and sends a
key at once can have that key decided against the previous window (known
issue 80).
All string comparisons ignore case in every alphabet:
`TitleEquals("КАЛЬКУЛЯТОР")` matches a window titled "Калькулятор".

```ron
// Media-Pause only acts as play/pause in Spotify:
(
    keys: "Pause",
    action: Media(PlayPause),
    when: AppEquals("Spotify.exe"),
)

// L key remapped only in Chrome and only when *not* in Incognito:
(
    keys: "Ctrl+Alt+L",
    action: PressKey("L"),
    when: And([
        AppEquals("chrome.exe"),
        Not(TitleContains("Incognito")),
    ]),
)
```

Variants:

| Variant                       | Matches when                                                          |
| ----------------------------- | --------------------------------------------------------------------- |
| `Always` (default)            | Always — same as omitting `when:`                                     |
| `AppEquals("<exe>")`          | File name of the focused window's executable, with `.exe`, equals `<exe>` (case insensitive): `"code.exe"` matches `C:\...\Microsoft VS Code\Code.exe` |
| `TitleContains("<s>")`        | Window title contains `<s>` (case insensitive)                        |
| `TitleEquals("<s>")`          | Window title equals `<s>` exactly (case insensitive)                  |
| `Not(<cond>)`                 | Negation                                                              |
| `And([<c1>, <c2>, ...])`      | All sub-conditions hold                                               |
| `Or([<c1>, <c2>, ...])`       | At least one sub-condition holds                                      |

Windows of UWP apps such as Calculator and Settings belong to
`ApplicationFrameHost.exe`, so `AppEquals` cannot tell them apart; match
them with `TitleContains`.

### Errors

Invalid bindings are surfaced at startup, one error per entry, so you see
every problem in one pass. Common cases:

- `unknown key "Foo" in combo "Ctrl+Foo"` — typo in a key name; see the table above.
- `PressKey: unknown key "Foo"` — same, but inside a `PressKey` action.
- `PostMessage: unknown WM_* name "WM_FOO"` — not in the built-in table; use a numeric code.
- `config file <path> does not exist` — the file named on the command line,
  or the config at a reload, is missing; nothing is created.
- ``Unexpected field named `tigger` in `RawBinding`, expected one of `keys`, `sequence`, `chord`, `action`, `block_default`, `block_original_combo`, `when`, `trigger`, `allow_extra_modifiers`, or `repeat` instead``
  — an unknown or misspelled field, here in a binding; the message lists
  the valid names. Field names are case-sensitive; RON reports the first
  such error only.
- ``Unexpected missing field named `bindings` in `RawConfig` `` — the file
  has no `bindings: [...]` list; a misspelled one gives
  ``Unexpected field named `bindngs` in `RawConfig`, expected `bindings` instead``.
- `binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms`
  — a `Tap`, `Hold` or `DoubleTap` term, `max_gap_ms` or `window_ms`
  outside 1 to 10000 ms.
- ``binding #0 [F13]: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger``
  — `repeat: true` on a `Tap`, `Hold`, `DoubleTap`, `sequence` or `chord`
  binding.
- ``Unexpected variant named `FlyToMars` in enum `RawAction`, expected one of `SwitchLanguage`, …``
  — an action, trigger, condition or step name is misspelled; names are
  case-sensitive.
- ``Unexpected variant named `Maskonly` in enum `RawBlockOriginalCombo`, expected one of `true`, `false`, or `MaskOnly` instead``
  — a misspelled `block_original_combo` value; `Some(true)` gives the same
  message for `` `Some` ``.

Bindings and steps are numbered from 0 in file order: `binding #1` is the
second binding of the file.

A warning does not stop the file from loading; it is logged at startup and
on every reload:

- `[WARN] binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them`
  — the later binding repeats an earlier one: the same kind (`keys`,
  `sequence` or `chord`) and keys (in any order for `keys` and `chord`, in
  order for `sequence` steps), the same trigger whatever its term, the same
  `when:` written the same way and, for `keys`, the same
  `allow_extra_modifiers`. The action does not count. Of two such `keys`
  bindings only the first ever acts (see
  [Which binding wins](#combo-syntax-the-keys-field)). If the one you remove
  sets `block_original_combo: true`, move that flag to the one you keep:
  the auto-blocker comes from it. An `Immediate` binding next to a `Tap`,
  `Hold` or `DoubleTap` binding on the same keys is not warned about,
  although only one of them acts on a press.
- ``[WARN] binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true``
  — the Windows hotkey is `` ` `` (or another key that is not a modifier),
  so the experimental `MaskOnly` blocker would let it through and Windows
  would switch the layout; the blocker swallows it as with `true`.

## Build

Requires Rust 1.88 or newer.

```bash
cargo build --release
```

Executable: `target/release/key-switch-rs.exe`

The `requireAdministrator` manifest is embedded only by the MSVC linker
(`x86_64-pc-windows-msvc`, the default Windows toolchain). A release build for
a `*-windows-gnu` target links without it and prints a cargo warning; start
that daemon elevated yourself.

### Tests

```bash
cargo test            # or: cargo test --release
```

The daemon binary has no test or bench harness (`test = false` and
`bench = false` in `Cargo.toml`). Selecting targets explicitly brings it back:
`cargo test --release` or `cargo bench` with `--all-targets`, `--bins` or
`--bin key-switch-rs` builds a harness from `src/main.rs` that inherits the
`requireAdministrator` manifest, and running it in a non-elevated shell fails
with "The requested operation requires elevation" (os error 740). Use plain
`cargo test --release` and plain `cargo bench`. Debug builds embed no manifest,
so `cargo test --all-targets` works.

## Run

Release builds ship two binaries:

- **`key-switch-rs.exe`** — the daemon. Manifest-marked `requireAdministrator`,
  so launching it triggers a UAC consent prompt. After consent the process
  runs at high integrity, required for the low-level keyboard hook to observe
  events in elevated windows (Task Manager, regedit, anything "Run as
  administrator").

- **`swch.exe`** — CLI driver that talks to the running daemon via a named
  pipe. Runs as normal user — **no UAC prompt** — also against the elevated
  daemon: the pipe grants SYSTEM and administrators full control and
  interactive users read and write, with a medium integrity label.

### Typical session

```text
> swch open      # swch: daemon launched (UAC prompt on every launch)
> swch status    # running (3 bindings live) with the default config
> swch off       # disabled       (remapping off; every key reaches Windows)
> swch on        # enabled        (remapping on again)
> swch exit      # shutting down  (daemon process terminates)
```

### swch commands

| Command       | What it does                                                |
| ------------- | ----------------------------------------------------------- |
| `swch open`   | Launch the daemon (a UAC prompt on every launch of a release build). Does nothing if a daemon already answers, refuses this console, is busy, or does not answer within 5 s. |
| `swch on`     | Turn remapping back on (idempotent). |
| `swch off`    | Turn remapping off (idempotent): the hook stays installed and passes every key through; pending Tap/Hold/DoubleTap, chords and sequences are dropped, and a running macro stops within 100 ms (a step already running finishes first). Useful for games or temporarily passing the OS's defaults through. |
| `swch status` | Print "running" or "disabled" (remapping on or off) and the live binding count. |
| `swch exit`   | Cleanly shut down the daemon (`PostThreadMessageW(WM_QUIT)`). |
| `swch help`   | Print usage.                                                |

Aliases: `open` = `start` = `launch`; `exit` = `quit` = `shutdown` = `stop`;
`status` = `ping`; `help` = `--help` = `-h`.

Every command except `help` connects to the daemon's named pipe
(`\\.\pipe\key-switch-rs.sock`); `open` sends `status` to find out whether a
daemon runs. If the daemon isn't running, the other commands print a hint and
exit 1, and `open` starts it. If it does not answer within 5 s, `open`
treats it as running, and the other commands print
`swch: cannot reach daemon (the daemon did not answer within 5s).` and exit 1.
The pipe admits interactive users at medium integrity or
above, so a normal console can control an elevated daemon. That includes
every user signed in to the machine: under fast user switching or Remote
Desktop another user can switch your remapping on or off, stop your daemon,
or answer `swch` in its place, until roadmap item 4.1 gives each session its
own pipe.
Until then only one daemon on the machine can own the pipe: a daemon that
cannot create it, for example a second signed-in user's, stops with
`[FATAL] could not create the IPC pipe \\.\pipe\key-switch-rs.sock: ...`
before it installs any hook.

### Direct launch (without swch)

You can also launch the daemon directly — right-click → "Run as administrator"
or just double-click and accept the UAC prompt. `Ctrl+C` in the console exits
cleanly. `swch` commands continue to work against the directly-launched daemon.

Only one daemon runs per Windows session. A second start, by double-click,
from a console or from a Scheduled Task, prints
``[ERROR] already running in this session; use `swch status`, `swch off` or `swch exit` to control it``
and exits with code 2 before it reads the config, without waiting for Enter
and without writing to the running daemon's log file.
The check covers the Windows session the daemon starts in; a second
signed-in user's daemon stops at the pipe instead (see above).

### Log file

The daemon writes the lines it logs on its console also to
`%LOCALAPPDATA%\key-switch-rs\daemon.log`, so a daemon started by a
Scheduled Task, whose console nobody watches, still leaves a record. The
folder is the one of the account the daemon runs as: with UAC consent on
your own account, your own `%LOCALAPPDATA%`. It is created with the first
line, and again if you delete it. Every line of the file starts with the
time in UTC (ISO 8601, with milliseconds) and the level:

```text
2026-10-16T09:41:07.123Z [INFO] key-switch-rs 0.1.0 started (pid 4242), logging to C:\Users\you\AppData\Local\key-switch-rs\daemon.log
2026-10-16T09:52:30.517Z [INFO] loading config from C:\Tools\key-switch-rs\config.ron
2026-10-16T09:52:30.518Z [ERROR] config reload failed:
2026-10-16T09:52:30.518Z [ERROR] config has 1 invalid binding(s):
2026-10-16T09:52:30.518Z [ERROR]   - binding #0 [F13]: PressKey: unknown key "Foo"
2026-10-16T09:52:30.518Z [INFO] keeping previous bindings live
```

A message of several lines, such as the reload error above, repeats its
time and level on every line, so that a search for `[ERROR]` finds all of
it. The console shows the same lines without the time, on stderr, with the
level on the first line of a message only. Only the startup banner (the
list of bindings) goes to stdout, and it is never written to the file,
because a binding can hold the text of a macro. The log never records the
keys you press or the text you type; lines about the config quote the
config, key names included. What the keyboard hook must not log at every
key press, a `SendInput` that Windows cut short, a `PostMessage` or
language switch that Windows refused, a Hold whose timer could not be set,
a `Launch`, `OpenUrl` or `Sequence` that the action executor's full
queue dropped or that found the executor stopped, or a key event that found
the daemon's decision state in use and passed undecided
(`main_state_refused`), it counts. It also records how long its longest callback ran. While a count
is not 0, or that callback took 5 ms or more, one
`[WARN] counters since the last report: ...` line every minute and at exit
gives all of them, ending with `longest_callback_us=<microseconds>`.

At 1 MiB, `daemon.log` becomes `daemon.1.log`, the previous `daemon.1.log`
becomes `daemon.2.log`, and the oldest file is deleted: three files, about
3 MiB, are kept. Read the file with Notepad or
`Get-Content -Tail 50 "$env:LOCALAPPDATA\key-switch-rs\daemon.log"`. A
program that keeps a log file open without letting others delete it holds
up the rotation, and `daemon.log` then grows past 1 MiB until that program
closes it.

If the daemon cannot find the folder, the console says
`[WARN] no log file: ...` and the daemon logs to the console only. If the
file cannot be written, the console says `[WARN] log file <path>: ...` once,
and every later line tries again.

Lines reach the file and the console through one writer thread, the file
first. In a window of the classic console host (`conhost.exe`) with
QuickEdit on, a selection you hold in the daemon's console stops that
thread at its next console write, so the lines after it reach the file
only once you release the selection; the keyboard is not slowed down. At
exit the daemon waits at most 1 s for the lines still queued.

### Avoiding the UAC prompt at every boot

Register the daemon as a Scheduled Task running at logon with the
**Run with highest privileges** flag. The task starts the binary
elevated without a prompt; `swch` then talks to it normally.

## Technologies

- Rust 2024 edition
- Windows API via official `windows` crate v0.62 with feature flags:
  - `Win32_Foundation`
  - `Win32_Globalization` (`CompareStringOrdinal`: the config watcher compares file names the way the file system does)
  - `Win32_Graphics_Gdi` (`RegisterClassW` and `WNDCLASSW` for the control window)
  - `Win32_UI_WindowsAndMessaging`
  - `Win32_UI_Input` (`RegisterRawInputDevices`, `GetRawInputData`: the hook watchdog's raw keyboard input)
  - `Win32_UI_Input_KeyboardAndMouse`
  - `Win32_UI_Accessibility` (for `SetWinEventHook` foreground-change tracking)
  - `Win32_UI_Shell` (`ShellExecuteW` for `OpenUrl` and `swch open`; `SHGetKnownFolderPath` for the log folder)
  - `Win32_System_Com` (`CoTaskMemFree` for the path `SHGetKnownFolderPath` returns; `CoInitializeEx` for the action executor's single-threaded apartment)
  - `Win32_System_Console`
  - `Win32_System_LibraryLoader` (`GetModuleHandleW` for the control window's class)
  - `Win32_System_Registry`
  - `Win32_System_RemoteDesktop` (`WTSRegisterSessionNotification` for the lock and unlock notifications)
  - `Win32_System_StationsAndDesktops` (`GetThreadDesktop`, `GetUserObjectInformationW`: whether this desktop has keyboard input)
  - `Win32_System_Threading`
  - `Win32_Security` (`CreateMutexW` for the single-instance guard)
- `serde` + `ron` for `config.ron` parsing — native Rust enum-variant literal syntax
- `arc-swap` for lock-free, hot-reloadable binding list
- `notify` for filesystem-driven config reload
- `interprocess` for cross-platform local socket / Win32 named pipe IPC
- `log` facade with an in-house sink: one writer thread for the console and the rotating `daemon.log`
- Per-binary `requireAdministrator` manifest embedded via raw MSVC linker flag (`/MANIFESTUAC`) in `build.rs` — only the daemon binary gets it; `swch.exe` stays at default `asInvoker`
- Low-level keyboard hook (`WH_KEYBOARD_LL`)
- Foreground-change hook (`SetWinEventHook` / `EVENT_SYSTEM_FOREGROUND`)
- Modular architecture with separation of concerns
- Lock-free binding list on the hot path (atomic Arc swap for cold-path reloads); the decision state lives on the main thread, which runs the hooks and the gesture timers, with no lock
- Self-documenting RON config with auto-generation and hot-reload on first run
- Two-binary split: daemon (`key-switch-rs.exe`) + CLI driver (`swch.exe`)

## Implementation Features

- **Native APIs first**: language switching, window operations and messages use Win32 calls; key output (`ToggleCapsLock`, `PressKey`, `Media`, macro `Text`/`Key`/`Combo`) is synthesized with `SendInput`
- **Global hook**: uses `WH_KEYBOARD_LL` to intercept all system keyboard events
- **Exact, sided modifiers**: a binding fires only with exactly its own modifiers held (`allow_extra_modifiers: true` relaxes that per binding); `Shift` matches either Shift key and `LShift` only the left one, and the same for Ctrl, Alt and Win
- **Block default behavior**: configurable blocking of standard key actions, for every trigger; a `Tap`, `Hold` or `DoubleTap` on modifiers alone lets them through by default
- **Auto-repeat policy**: holding the key of a `PressKey` or volume binding sends one press per auto-repeat, at the keyboard's repeat rate; every other action fires once per press unless the binding sets `repeat: true`
- **Thread-safe**: `ArcSwap<Vec<Binding>>` for the binding list, atomics for the hook handles and the remapping switch, and the decision core owned by the main thread, which runs the hooks and every gesture timer, so no lock is taken on the key path; binding list can be hot-swapped atomically; a reload stops the macros still running, which belong to the old list
- **Timers on the main thread**: a `Hold` sets the main thread's one high-resolution waitable timer instead of starting a thread, and fires within about 1 ms of its term
- **Hook watchdog**: Windows removes a low-level keyboard hook whose callback runs past `LowLevelHooksTimeout`, without a word (KS-08). A `hook-watchdog` thread receives the raw input of every keyboard on a message-only window of its own and compares it with a heartbeat the keyboard callback bumps; when keys arrive and the callback stays silent for more than 1 s, the main thread reinstalls the hook (the new one first, then the old one off) and logs it. The main thread pays one atomic increment per key; Windows sends one `WM_INPUT` per key event to the watchdog thread, never to the main thread, which reads it on another core; idle, the watchdog does not wake up. It keeps no key data and never injects input
- **Hot-reload**: edit `config.ron` in any editor, save, and bindings reload automatically (~250ms debounce). A save that does not load (a syntax error, an unknown field, a bad value) or a deleted file keeps the previous config live.
- **One winner per key press**: among the `keys` bindings that match one press, one wins (more keys, then a `when:` condition, then file order), and only the winner, or its Tap/Hold/DoubleTap group, runs and decides whether the key is swallowed (KS-13)
- **Auto-blocking**: automatically blocks original system combinations when needed
- **Focus-transition repair**: re-syncs modifier state via `GetAsyncKeyState` whenever a higher-integrity window (Task Manager, UAC consent, lock screen) returns focus, so phantom-held modifiers do not leak across the transition; when keyboard input leaves the desktop (a lock, Ctrl+Alt+Del, a UAC prompt on the secure desktop) every held key is released, so nothing is stuck after Win+L
- **Per-process sentinel**: injected-event marker is mixed with the process ID so this daemon does not collide with other input-synthesizing apps (AHK, PowerToys)
- **Table-driven matching**: a binding's keys are looked up in the engine's per-key table of held keys; no list of held keys is built per key press
- **Independently blocked keys**: multiple keys can be blocked simultaneously; each key's release is tracked independently

## Known Limitations

Release builds embed a manifest requesting `requireAdministrator`, which puts
the process at high integrity and lets the keyboard hook observe events inside
Task Manager, regedit, and other elevated windows. This is the practical
workaround for UIPI (User Interface Privilege Isolation).

What still cannot be intercepted:

- The **Secure Desktop** (Ctrl+Alt+Del menu, UAC consent prompt itself, the
  lock screen, Windows logon). These run under Winlogon on an isolated
  desktop that no user-mode process — even SYSTEM — can hook. When
  keyboard input leaves for such a desktop (`WTS_SESSION_LOCK`, or
  `EVENT_SYSTEM_DESKTOPSWITCH` while another desktop has input), the daemon
  releases every held key, so the next press of each key is a press (roadmap
  item 2.3). A key held through the whole lock and still auto-repeating
  afterwards reads its first repeat as a press, so its binding fires once
  more (known issue 76). A Ctrl+Alt+Del or UAC round trip while the
  daemon's main thread is busy for all of it and a UAC prompt with the
  secure desktop switched off by policy are not answered, and a lock signal
  that arrives only after a very short unlock resets after the return
  (known issue 79).
- Anti-cheat-protected processes that explicitly block low-level hooks
  (some games' kernel-mode anti-cheat drivers).
- A key whose press the daemon swallowed, then released and pressed again
  where the hook cannot see it (an elevated window while the daemon is not
  elevated, the secure desktop), keeps its auto-repeats swallowed until it
  is released. Its release reaches Windows, because the hook finds the key
  down in Windows, so it does not stay stuck (roadmap item 2.3, known
  issue 78).

The hook watchdog (roadmap item 2.4) has limits of its own:

- The keys typed between the hook's loss and its reinstall, about 1 to 1.5
  s, reach Windows unremapped, and a `Hold` armed just before the loss can
  still fire on its timer.
- Another program's low-level hook installed after the daemon that
  swallows every key for more than 1 s can draw one reinstall, which puts
  the daemon's hook first again, if Windows reports raw input for the keys
  that hook swallows (not verified).
- A daemon that is not elevated gets no hook calls while an elevated
  window has the focus, but may get its raw input: it then asks for a
  reinstall after 1, 2, 4 ... up to 64 s of typing there.
- Raw keyboard input in the daemon's process belongs to the watchdog:
  Windows allows one registration per device class and process.

For a fully unattended/no-UAC-prompt setup, the alternative is
`uiAccess="true"` in the manifest — but that requires a binary signed with a
trusted code-signing certificate and installed under `C:\Program Files\`.
See the AutoHotkey / kanata projects for prior art.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
