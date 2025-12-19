# key-switch-rs

Native Windows application in Rust for key remapping via Windows API bindings.

## Features

Default bindings:

- **CapsLock** → switch input language
- **Shift + CapsLock** → toggle CapsLock on/off

## Project Architecture

Modular architecture:

```text
src/
├── core/
│   ├── app.rs              - main application with binding management
│   └── windows_actions.rs  - BindAction enum for binding actions
├── data/
│   ├── binding.rs          - Binding struct (key combo → action)
│   └── key_combination.rs  - key combinations (key + modifiers)
├── hook/
│   ├── keyboard_hook.rs    - Windows API hook wrapper
│   └── keyboard_hook_callback.rs - hook callback handler
├── system/
│   ├── registry.rs         - Windows registry hotkey reader
│   └── system_function.rs  - Windows system functions
└── main.rs                 - entry point, binding configuration
```

### Modules

#### `key_combination.rs`

`KeyCombination` struct - represents key combination:

- Key list (`Vec<VIRTUAL_KEY>`)
- `matches()` method checks if pressed keys match combination

#### `windows_actions.rs`

`BindAction` enum - actions executed on binding trigger:

- `SwitchLanguage` - switch to next keyboard layout
- `SwitchLanguageBackward` - switch to previous layout
- `ToggleCapsLock` - toggle CapsLock state
- `PressKey(VIRTUAL_KEY)` - emulate key press
- `PostMessage { ... }` - send message to active window
- `DoNothing` - no action (for auto-blockers)

#### `binding.rs`

`Binding` struct - links key combination with action:

- Key combination (`KeyCombination`)
- Action (`BindAction`)
- Block default behavior flag
- Auto-block original system combination flag

#### `keyboard_hook.rs`

Windows API wrapper:

- Install/remove low-level keyboard hook
- Store bindings list
- Track active keys via atomic operations
- Process keyboard events and execute actions

#### `app.rs`

Main application:

- Manage bindings via builder pattern
- Install keyboard hook
- Ctrl+C handler for correct shutdown
- Windows message loop

## Usage

### Adding Bindings

```rust
use key_combination::KeyCombination;
use windows_actions::BindAction;
use binding::Binding;

App::new()
    .add_binding(
        Binding::new(
            KeyCombination::new(VK_CAPITAL),
            BindAction::SwitchLanguage,
        )
        .with_block_original_combo(true)
    )
    .add_binding(
        Binding::new(
            KeyCombination::new(VK_CAPITAL).with(VK_SHIFT),
            BindAction::ToggleCapsLock,
        )
    )
    .run()
```

### Binding Examples

```rust
// Ctrl + Alt + Delete → send message
Binding::new(
    KeyCombination::new(VK_DELETE)
        .with(VK_CONTROL)
        .with(VK_MENU),
    BindAction::PostMessage {
        msg: WM_CLOSE,
        wparam: 0,
        lparam: 0
    },
)

// Win + L → emulate key press (lock system)
Binding::new(
    KeyCombination::new(VK_L).with(VK_LWIN),
    BindAction::PressKey(VK_L),
)
```

## Build

```bash
cargo build --release
```

Executable: `target/release/key-switch-rs.exe`

## Run

Simply run `key-switch-rs.exe`. The program will run in background and intercept keyboard events.

Press `Ctrl+C` in console to exit.

## Technologies

- Rust 2024 edition
- Windows API via official `windows` crate v0.62
- Low-level keyboard hook (WH_KEYBOARD_LL)
- Modular architecture with separation of concerns
- Builder pattern for convenient configuration
- Lock-free atomic operations for thread safety
- Minimal dependencies, native performance

## Implementation Features

- **Not macros**: application calls native Windows API functions, not emulating action sequences
- **Global hook**: uses `WH_KEYBOARD_LL` to intercept all system keyboard events
- **Modifier normalization**: left/right Shift treated as single modifier
- **Block default behavior**: configurable blocking of standard key actions
- **Thread-safe**: atomic operations (`AtomicBool`, `AtomicPtr`) for safe concurrent access
- **Priority bindings**: more specific combinations (more keys) checked first
- **Auto-blocking**: automatically blocks original system combinations when needed
