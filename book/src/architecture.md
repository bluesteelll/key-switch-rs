# Architecture

The key-switch-rs daemon is a single console process, elevated in release builds. Its main thread owns a `WH_KEYBOARD_LL` hook, a foreground WinEvent hook and the message loop that services both. Every keyboard event goes through a thin Win32 adapter into a pure decision core, `src/engine`, which says whether to swallow the event, which actions to run, which timers to start and whether to send the menu mask key. The IPC server, hot reload, Hold timers and macro workers run on helper threads that share one process-wide `KeyboardHook` value, and one `log-writer` thread writes every log line. This chapter describes that structure as it exists in the code, and shows which defects listed in the [audit](audit.md) the structure explains and which ones roadmap Phases 0 and 1 removed.

> [!NOTE]
> Facts describe commit [`20af28d`](https://github.com/bluesteelll/key-switch-rs/tree/20af28d) (2026-10-18), the tip of `master` after roadmap Phase 1. Every code reference is a permalink to that commit. The audit described commit `7cb3521`, before the decision core existed; the [audit](audit.md) keeps that description, and each KS entry says what changed. Issue IDs such as [KS-04](audit.md#ks-04) link to their entries in the audit chapter. Run-time behaviour is derived from the code and the engine's scenario tests; the daemon was not run against live keyboard input.

## Crate layout

One Cargo package (edition 2024, version 0.1.0, `rust-version = "1.88"`) builds a library crate and two binaries. The library exists so that the CLI can share the IPC protocol with the daemon ([src/lib.rs:1-20](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/lib.rs#L1-L20)).

| Target | Source | Role |
|---|---|---|
| `key_switch_rs` (library) | `src/lib.rs` | Declares the eight modules below. Both binaries import from it. |
| `key-switch-rs.exe` | `src/main.rs` | The daemon. Starts the logger, takes the single-instance guard, resolves the config path, loads it and runs `App`. Built with `test = false` and `bench = false` ([KS-38](audit.md#ks-38)). |
| `swch.exe` | `src/bin/swch.rs` | CLI driver. From the library it uses only the `ipc` module, which also holds the pipe client; it calls `ShellExecuteW` itself for `open`. |
| Build script | `build.rs` | Embeds a `requireAdministrator` manifest into the release daemon only, and only with the MSVC linker. |

The runtime dependencies are `windows` 0.62.2, `serde` 1.0.228, `ron` 0.10.1, `arc-swap` 1.9.1, `notify` 8.2.0, `interprocess` 2.4.2 and `log` 0.4.29. The versions come from `Cargo.lock`, whose only change since the audit is `log` as a direct dependency, added by roadmap [item 1.7](roadmap.md#17-file-logging).

| Module | Files | Responsibility |
|---|---|---|
| `config` | `loader.rs`, `parsing.rs`, `schema.rs`, `default_config.ron` | Reads `config.ron`, deserializes it into `Raw*` serde types, validates and converts each entry into a `Binding`, resolves the defaults of `block_default` and `repeat`, warns about a binding that duplicates an earlier one, adds auto-blockers, and writes the bundled default file when `<exe_dir>\config.ron` is missing at startup. It exports `ConfigPath`, `load` and `reload` ([src/config/mod.rs:17](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/mod.rs#L17)). |
| `core` | `app.rs`, `constants.rs`, `windows_actions.rs` | The `App` builder and process lifecycle, the daemon's side of the IPC commands (`DaemonHandle`), the injected-event sentinel and the menu mask key code, and `BindAction` with its execution code. |
| `data` | `binding.rs`, `key_spec.rs`, `key_combination.rs`, `trigger.rs`, `condition.rs`, `sequence.rs`, `vk_name.rs` | The runtime model: bindings, trigger keys (`KeySpec`), kinds, triggers, `when:` conditions, macro steps and key-name tables. `sequence.rs` also runs macros. |
| `engine` | `mod.rs`, `env.rs`, `keys.rs`, `matching.rs`, `winner.rs`, `gestures.rs`, `chords.rs`, `sequences.rs`; test-only `testing.rs`, `scenarios.rs`, `properties.rs` | The decision core: `Engine::on_key`, `Engine::on_timer`, the `Env` trait and `Decision`. Plain Rust with `#![forbid(unsafe_code)]`: no Win32 calls, threads, locks or clock of its own ([src/engine/mod.rs:1-198](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L1-L198)). |
| `hook` | `keyboard_hook.rs`, `keyboard_hook_callback.rs`, `config_watcher.rs`, `ipc_server.rs` | Hook installation and shared state, the Win32 adapter that feeds the engine and sends the menu mask key, and the two background services (file watcher and pipe server). |
| `ipc` | `ipc.rs` | Pipe name, the `Command` enum, the line-based wire format, and the client `swch` uses (`request`, with a 5 s deadline). |
| `logging` | `mod.rs`, `sink.rs`, `writer.rs`, `counters.rs`; test-only `testing.rs` | The logger behind the `log` facade, the `log-writer` thread that owns the console output and `daemon.log`, the rotating file sink, and the counters for what the hook callback must not log per event ([src/logging/mod.rs:1-18](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/logging/mod.rs#L1-L18)). |
| `system` | `system_function.rs`, `registry.rs`, `single_instance.rs`, `known_folder.rs` | The OS functions behind `SwitchLanguage` and `ToggleCapsLock`, the registry lookup of the Windows language hotkey, the single-instance mutex, and the lookup of the `LocalAppData` folder for the log. |

The `hook` module holds more than the hook: the pipe server and the file watcher live there too. The pipe server no longer knows `KeyboardHook`: it talks to a `DaemonControl` trait, which the daemon implements in `core::app` and the integration tests implement with a stand-in ([src/hook/ipc_server.rs:31-43](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/ipc_server.rs#L31-L43)).

```mermaid
flowchart LR
    subgraph BINS["Binaries"]
        MAINRS["main.rs, key-switch-rs.exe"]
        SWCH["bin/swch.rs, swch.exe"]
    end
    subgraph LIB["Library crate key_switch_rs"]
        APP["core::app"]
        ACT["core::windows_actions"]
        CONST["core::constants"]
        CFG["config: loader, parsing, schema"]
        IPCP["ipc"]
        IPCS["hook::ipc_server"]
        WATCH["hook::config_watcher"]
        KH["hook::keyboard_hook"]
        CB["hook::keyboard_hook_callback"]
        ENG["engine"]
        DATA["data: binding, key_spec, key_combination, trigger, condition, vk_name"]
        SEQ["data::sequence"]
        SYS["system: system_function, registry, single_instance, known_folder"]
        LOGM["logging"]
    end
    MAINRS --> CFG
    MAINRS --> APP
    MAINRS --> SYS
    MAINRS --> LOGM
    SWCH --> IPCP
    APP --> KH
    APP --> CB
    APP --> IPCS
    APP --> WATCH
    APP --> IPCP
    APP --> DATA
    APP --> LOGM
    IPCS --> IPCP
    WATCH --> CFG
    WATCH --> KH
    KH --> ENG
    KH --> CB
    KH --> DATA
    CB --> KH
    CB --> ENG
    CB --> CONST
    CB --> DATA
    CB --> LOGM
    ENG --> DATA
    ENG --> ACT
    CFG --> DATA
    CFG --> ACT
    CFG --> SEQ
    DATA --> ACT
    ACT --> DATA
    ACT --> SEQ
    ACT --> LOGM
    SEQ --> ACT
    ACT --> SYS
    ACT --> CONST
    SEQ --> CONST
    SYS --> CONST
    SYS --> DATA
    SYS --> LOGM
```

Every module logs through the `log` macros; the edges to `logging` show the modules that call it directly, for the counters or to flush the queue.

The modules are still not fully layered. `core::windows_actions` and `data::sequence` import each other, `core` and `data` import each other, and so do `core` and `system`. The engine uses `BindAction` from `core` only as data to fire, and never executes it; moving `BindAction` into `data` is a follow-up of roadmap [item 3.4](roadmap.md#34-core-consolidation).

## Startup sequence

1. `main` starts the logger first. `logging::init` installs the logger behind the `log` facade and starts the `log-writer` thread, with the console (stderr) as its only output ([src/main.rs:14-19](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L14-L19), [src/logging/mod.rs:104-122](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/logging/mod.rs#L104-L122)).
2. `main` takes the single-instance guard before any config is read and before any hook or pipe exists: `InstanceGuard::acquire` creates the named mutex `Local\key-switch-rs` ([src/main.rs:20-36](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L20-L36), [src/system/single_instance.rs:35-65](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/system/single_instance.rs#L35-L65)). If another daemon of the same Windows session holds it, `main` logs `` [ERROR] already running in this session; use `swch status`, `swch off` or `swch exit` to control it ``, flushes the log and exits with code 2, without the Enter prompt. A mutex this process may not open, such as one an elevated daemon created, counts as held ([KS-20](audit.md#ks-20) fixed, roadmap [item 1.5](roadmap.md#15-single-instance)).
3. `run` attaches `%LOCALAPPDATA%\key-switch-rs\daemon.log` to the logger and logs the start line into it, so a refused second start never writes to, or rotates, the running daemon's file. Without a `LocalAppData` folder or a log writer the daemon logs to the console only ([src/main.rs:87-108](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L87-L108)).
4. `run` resolves the config path once with `ConfigPath::from_arg`: the first argument, made absolute against the current folder, or `<exe_dir>\config.ron` ([src/main.rs:70-75](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L70-L75), [src/config/loader.rs:110-149](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L110-L149); [KS-21](audit.md#ks-21) fixed).
5. `config::load` reads, parses, validates and expands the bindings ([src/main.rs:77](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L77)). It writes the bundled default only when `<exe_dir>\config.ron` is missing; a missing command-line path is an error. See [Config loading and hot reload](#config-loading-and-hot-reload).
6. The `App` builder collects the bindings and the watcher path: `App::new().with_config_watcher(path)`, then `add_binding` for each binding ([src/main.rs:79-83](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L79-L83), [src/core/app.rs:76-98](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L76-L98)).
7. `App::run` stores the main thread ID for the Ctrl+C handler and creates the pipe with `ipc_server::bind`, with its explicit security descriptor, before any hook is installed ([src/core/app.rs:137-146](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L137-L146), [src/hook/ipc_server.rs:105-117](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/ipc_server.rs#L105-L117)). A pipe that cannot be created is an error that stops the daemon with nothing to undo; for a taken name the message names its likely owners ([src/core/app.rs:220-239](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L220-L239)).
8. It flushes the log, so that the loader's lines come first, prints the binding banner on stdout and registers the console control handler ([src/core/app.rs:100-127](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L100-L127), [L148-L149](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L148-L149)).
9. `get_hook()` initialises the singleton. `update_bindings` sorts and publishes the list, and `install()` registers the `WH_KEYBOARD_LL` hook and the `EVENT_SYSTEM_FOREGROUND` WinEvent hook on the main thread ([src/core/app.rs:151-158](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L151-L158), [src/hook/keyboard_hook.rs:157-214](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L157-L214)). If the second hook fails, the first is removed again. A second `install` on an installed hook is an error.
10. `serve` starts the `ipc-server` thread on the pipe, with a `DaemonHandle` that carries the hook and the main thread ID ([src/core/app.rs:160-175](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L160-L175), [src/hook/ipc_server.rs:119-145](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/ipc_server.rs#L119-L145)). It runs only once the hooks are installed, so `swch status` never answers for a daemon whose install failed. If the thread cannot start, the main thread removes the hooks again and the error stops the daemon.
11. `config_watcher::spawn_watcher` starts the file watcher. A failure only warns ([src/core/app.rs:177-199](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L177-L199)).
12. The main thread pumps `GetMessageW`/`DispatchMessageW` until `WM_QUIT` ([src/core/app.rs:201-211](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L201-L211)). This loop is what delivers both hooks' callbacks.
13. After `WM_QUIT`, `hook.uninstall()?` removes the hooks on the main thread ([src/core/app.rs:213-214](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L213-L214)). A foreground hook that cannot be removed is logged; a failed keyboard unhook is returned ([src/hook/keyboard_hook.rs:221-250](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L221-L250)). The watcher is dropped when `run` returns, which closes its channel and ends the debounce thread, and the single-instance guard is released with it. `main` then flushes the log ([src/main.rs:37-43](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L37-L43)). The detached `ipc-server` and `log-writer` threads end with the process.

Any error that `run` returns becomes a `[FATAL] ...` record. `main` flushes it, writes `Press Enter to exit...` and waits on stdin, so that a double-clicked console window stays open ([src/main.rs:46-62](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L46-L62)). The guard is already released then, so a corrected restart is not refused.

## Process and thread model

The main thread does the time-critical work. Windows delivers a low-level keyboard hook callback as a message to the thread that installed the hook, and that thread must pump messages. `WINEVENT_OUTOFCONTEXT` WinEvent callbacks use the same mechanism. Only the main thread installs and removes the hooks ([src/hook/keyboard_hook.rs:157-161](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L157-L161), `install` and `uninstall` are `pub(crate)`). Every other thread is either created by Windows (the console control handler) or a plain `std::thread` without a message loop.

```mermaid
flowchart TD
    OS["Windows input stack"]
    HOOK["KeyboardHook singleton: bindings, enabled flag, Mutex of Engine"]
    subgraph MAIN["Main thread, App::run"]
        LOOP["GetMessageW and DispatchMessageW loop"]
        KCB["keyboard_hook_callback, the adapter"]
        FG["foreground_changed"]
        ACTS["menu mask key, Immediate, Tap, DoubleTap, Chord and Sequence-kind actions"]
    end
    subgraph CTRL["Console control thread, created by Windows"]
        CCH["console_ctrl_handler"]
    end
    subgraph IPCT["ipc-server thread"]
        ACC["listener.incoming loop"]
    end
    subgraph CONN["Connection worker, one per swch call"]
        DISP["handle_connection and dispatch"]
    end
    subgraph NTF["notify backend thread"]
        FILT["event filter closure"]
    end
    subgraph DEB["config-watcher thread"]
        DL["debounce_loop, config::reload, update_bindings"]
    end
    subgraph TMR["hold-timer threads, one per armed Hold"]
        HT["sleep until due, Engine::on_timer, run the Hold action"]
    end
    subgraph SQW["Sequence workers, one per macro run"]
        STEPS["execute_step loop"]
    end
    subgraph LWT["log-writer thread"]
        LW["console on stderr and daemon.log"]
    end
    OS -->|"LL hook and WinEvent messages"| LOOP
    LOOP --> KCB
    LOOP --> FG
    KCB -->|"Engine::on_key under the lock"| HOOK
    FG -->|"Engine::on_foreground_changed under the lock"| HOOK
    KCB -->|"after the lock"| ACTS
    KCB -->|"Decision timers"| HT
    HT -->|"Engine::on_timer under the lock"| HOOK
    ACTS -->|"BindAction::Sequence"| STEPS
    HT -->|"BindAction::Sequence"| STEPS
    CCH -->|"PostThreadMessageW WM_QUIT"| LOOP
    ACC -->|"thread::spawn"| DISP
    DISP -->|"on and off: set_enabled resets the engine"| HOOK
    DISP -->|"exit posts WM_QUIT"| LOOP
    FILT -->|"mpsc unit message"| DL
    DL -->|"ArcSwap store"| HOOK
    DL -->|"log records, try_send"| LW
    ACTS -->|"log records, try_send"| LW
```

Every thread that logs queues its line for the `log-writer` thread; the diagram shows two of them. The hook callback itself never logs: it only updates counters, which the writer reports.

| Thread | Created by | Runs | Lifetime |
|---|---|---|---|
| Main | Process start, then `App::run` | Startup, config load, pipe creation, hook install, the message loop, `keyboard_hook_callback`, `foreground_changed`, condition evaluation, the menu mask key, every Immediate action, Tap actions (on key-up), DoubleTap actions (on the second key-down), Chord and Sequence-kind completion actions, final uninstall | Process |
| Console control | Windows, on Ctrl+C or console close ([src/core/app.rs:119-127](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L119-L127)) | `console_ctrl_handler`, which posts `WM_QUIT` to the main thread and flushes the log, waiting at most 1 s ([src/core/app.rs:241-262](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L241-L262)) | One signal |
| `ipc-server` | `IpcListener::serve` ([src/hook/ipc_server.rs:119-145](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/ipc_server.rs#L119-L145)) | The accept loop | Process (detached) |
| Connection worker (unnamed) | One `thread::spawn` per accepted connection ([src/hook/ipc_server.rs:135](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/ipc_server.rs#L135)) | Reads one line, then `dispatch`: `on`/`off` flip the enabled flag and reset the engine under its lock, `status`, `exit` | One request, or forever if the client never sends a line ([KS-26](audit.md#ks-26), open) |
| notify backend | `notify::recommended_watcher` | The filter closure ([src/hook/config_watcher.rs:107-115](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L107-L115)) | While the watcher value lives |
| `config-watcher` | `spawn_watcher` ([src/hook/config_watcher.rs:123-126](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L123-L126)) | Debounce, `config::reload` (including the registry read for auto-blockers), `update_bindings` | Until the channel closes |
| `hold-timer` | One per `TimerRequest` that a Hold arms ([src/hook/keyboard_hook_callback.rs:281-304](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L281-L304)) | Sleeps until the due time, asks `Engine::on_timer`, runs the Hold action if one fires | One term; a cancelled Hold's timer wakes and finds nothing |
| Sequence worker (unnamed) | `spawn_sequence` for every `BindAction::Sequence` run ([src/data/sequence.rs:111-122](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/sequence.rs#L111-L122)) | All steps, including `Delay` sleeps | Until the last step; no cancellation or limit |
| `log-writer` | `logging::init`, the first statement of `main` ([src/logging/writer.rs:38-57](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/logging/writer.rs#L38-L57)) | Every log record: the console line on stderr and, once attached, `daemon.log` with its rotation, the file first; the counter report every 60 s and at each flush. Every other thread only formats its line and queues it with `try_send` on a bounded queue (1024 lines), which takes no lock of its own beyond the channel's, held for O(1) list work when it wakes the parked writer; a full queue drops the line and counts it. No Win32 UI API, no message loop | Process (detached) |

Tap, DoubleTap, Chord and Sequence state needs no thread: each pending entry stores a deadline, and the next event compares its time with it. Until `16a0461` every armed Tap, Hold and DoubleTap started its own `tap-gc`, `hold-timer` or `double-tap-gc` thread ([KS-33](audit.md#ks-33), partly fixed).

### Threads and locking

The engine is single-threaded data, but three kinds of thread reach it: the main thread for every hook event and foreground change, a `hold-timer` thread when its Hold is due, and an IPC worker for `swch on` and `swch off`. Until roadmap [item 2.2](roadmap.md#22-timers-on-the-main-thread) moves the Hold timers onto the main thread, the engine therefore sits behind one `Mutex` in `KeyboardHook` ([src/hook/keyboard_hook.rs:40-67](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L40-L67)). The rules that keep this lock off the critical path:

- Only `KeyboardHook::with_engine` takes the lock, and only while the engine decides or resets ([src/hook/keyboard_hook.rs:82-104](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L82-L104)). The adapter runs every action, `SendInput` and thread spawn after it has released the lock and the binding snapshot ([src/hook/keyboard_hook_callback.rs:169-221](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L169-L221)).
- `with_engine` refuses a nested call on the same thread instead of deadlocking: a thread-local flag marks the thread as deciding, and a nested call returns `None`, so the event passes ([src/hook/keyboard_hook.rs:18-38](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L18-L38)). Nothing under the lock pumps messages today, so this path is not taken. If it ever were, a refused key-up whose key-down was swallowed would pass and leave the key held in the engine ([src/hook/keyboard_hook_callback.rs:58-82](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L58-L82)). Anything that adds a window to the daemon, such as roadmap items 2.3 and 4.6, has to deal with this first.
- The engine reads the clock once per event, under the lock, and a `hold-timer` takes the lock only once `Instant::now()` has reached its due time ([src/hook/keyboard_hook_callback.rs:306-317](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L306-L317)). A key-up and the Hold timer of the same press are therefore decided in the order of one clock, which makes `Tap` and `Hold` with equal terms exclusive ([KS-25](audit.md#ks-25), partly fixed).
- The enabled flag is an `AtomicBool`. The callback reads it before the lock, to pass every event untouched while remapping is off, and again under the lock, where `set_enabled` flips it ([src/hook/keyboard_hook.rs:50-59](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L50-L59), [src/hook/keyboard_hook_callback.rs:188-196](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L188-L196)).

<a id="why-swch-on-kills-the-hook-ks-01"></a>
### How `swch off` and `swch on` work (KS-01)

At the audit, `swch on` reinstalled both hooks on the short-lived IPC worker, whose exit removed them again, and `status` kept reporting `running` ([KS-01](audit.md#ks-01)). Since `a1b5a27` the hooks stay installed for the whole life of the daemon, and `on` and `off` only flip the enabled flag through `DaemonControl::set_enabled` ([src/core/app.rs:33-59](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L33-L59), [src/hook/keyboard_hook.rs:129-155](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L129-L155)):

- While remapping is off, `keyboard_hook_callback` passes every event to `CallNextHookEx` without reading it, and `foreground_changed` does nothing ([src/hook/keyboard_hook_callback.rs:331-337](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L331-L337), [src/hook/keyboard_hook_callback.rs:373-376](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L373-L376)).
- A real switch, in either direction, resets the engine under its lock: every pending Tap, Hold, DoubleTap, chord and sequence is dropped, the Hold generation keeps counting, and the held keys are re-read from Windows as already pressed ([src/engine/mod.rs:398-420](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L398-L420), [src/engine/keys.rs:112-135](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L112-L135); [KS-31](audit.md#ks-31)). A repeated `on` or `off` resets nothing.
- `status` reports the flag, not the hook handle ([src/core/app.rs:41-46](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L41-L46)).

What it does not cover: a running macro keeps running after `swch off` ([item 2.1](roadmap.md#21-action-executor)), and a key swallowed before `off` and released while remapping is off reaches Windows as a lone key-up.

### Why actions can stall all keyboard input (KS-08)

Windows does not pass a keystroke on to any application until the low-level hook callback returns. If the callback takes longer than `LowLevelHooksTimeout`, Windows skips it, and on Windows 7 and later it can remove the hook silently. The adapter runs the decided actions on the hook thread before it returns ([src/hook/keyboard_hook_callback.rs:343-349](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L343-L349)). Only `BindAction::Sequence` moves its work to another thread ([src/core/windows_actions.rs:134-137](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L134-L137)). `Launch` (`CreateProcessW` through `Command::spawn`) and `OpenUrl` (`ShellExecuteW`, with COM never initialised) run inside the callback ([src/core/windows_actions.rs:138-139](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L138-L139)), and so do Tap and DoubleTap actions and Chord and Sequence-kind completions. Only Hold actions run elsewhere, on their `hold-timer` thread. Nothing watches whether the hook is still installed. See [KS-08](audit.md#ks-08), open until roadmap items [2.1](roadmap.md#21-action-executor) and [2.4](roadmap.md#24-hook-watchdog).

## The key-event path

`keyboard_hook_callback` ([src/hook/keyboard_hook_callback.rs:319-354](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L319-L354)) runs on the main thread for every keyboard event on the desktop. It either returns `LRESULT(1)`, which drops the event, or calls `CallNextHookEx`, which passes it on. The adapter translates and carries out; the engine decides ([src/hook/keyboard_hook_callback.rs:1-32](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L1-L32)).

1. **Negative code.** If `code < 0`, the event is passed on unchanged ([L324-L329](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L324-L329)).
2. **Remapping off.** If `swch off` cleared the enabled flag, the event is passed on without being read ([L331-L337](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L331-L337)).
3. **Sentinel check.** If `KBDLLHOOKSTRUCT::dwExtraInfo` equals the per-process sentinel, the event is the daemon's own `SendInput` output, the menu mask key included, and is passed on before the engine sees it, so actions never trigger bindings ([L180-L182](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L180-L182)). Only `vkCode` and `dwExtraInfo` are read ([L339-L343](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L339-L343)). `scanCode` and `flags` (including `LLKHF_INJECTED` and the extended-key bit) are ignored, so input injected by other programs is matched like physical input, and the fake LCtrl that AltGr sends looks like a real Ctrl.
4. **Translation.** `WM_KEYDOWN`/`WM_SYSKEYDOWN` become a key-down event and `WM_KEYUP`/`WM_SYSKEYUP` a key-up event; any other message passes ([L157-L167](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L157-L167)).
5. **Decision.** The adapter takes one snapshot of the binding list, creates a `HookEnv`, and asks `Engine::on_key` under the engine lock, if remapping is still on there ([L169-L197](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L169-L197)). The list is sorted by key count, most keys first ([src/engine/mod.rs:303-311](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L303-L311)).
6. **Modifier resync.** The engine reads the 11 modifier codes (generic and sided Shift, Ctrl and Alt, plus LWin and RWin) from `GetAsyncKeyState` through the `Env`. It skips the key being delivered, whose OS state lags until the hook returns, and every modifier whose key-down it swallowed, because Windows never saw it go down ([src/engine/keys.rs:95-110](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L95-L110); [KS-14](audit.md#ks-14) fixed).
7. **Repeat detection.** On a key-down the key is marked held; if it already was, the event is an auto-repeat ([src/engine/keys.rs:146-153](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L146-L153)).
8. **One winner.** Among the `keys` bindings that are not auto-blockers, the engine elects one winner for this key-down, auto-repeats included ([src/engine/mod.rs:427-431](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L427-L431), [src/engine/winner.rs:45-82](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/winner.rs#L45-L82); [KS-13](audit.md#ks-13) fixed, roadmap [item 1.6](roadmap.md#16-one-winner-per-event)). A candidate matches the press: the key just pressed is one of its trigger keys, all of its keys are held on their sides, and the modifier families held are exactly its own unless it sets `allow_extra_modifiers` ([src/engine/matching.rs:65-79](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/matching.rs#L65-L79); [KS-04](audit.md#ks-04), [KS-07](audit.md#ks-07) fixed). Its `when:` must hold, and is asked only once its keys match. The rank is the key count, then whether it has a `when:`, then the earlier place in the list, which is file order ([src/engine/winner.rs:39-43](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/winner.rs#L39-L43)).
9. **Binding loop.** Each binding is then visited in list order ([src/engine/mod.rs:436-578](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L436-L578)):
   - An **auto-blocker** has its own rule. It matches when the key just pressed is one of its hotkey's, all of them are held, and Windows would see the hotkey and nothing else ([src/engine/matching.rs:81-102](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/matching.rs#L81-L102)). Its `when:`, the `when:` of the bindings that asked for it OR'ed, is asked only once the hotkey matches ([KS-35](audit.md#ks-35) fixed). A `true` blocker then asks for suppression. A `MaskOnly` blocker, experimental, lets the key-down pass and, on a first key-down, asks for the menu mask key on this event and on the next key event. A blocker fires nothing and never keeps a user binding from winning ([L439-L467](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L439-L467); [KS-05](audit.md#ks-05) partly fixed, [KS-18](audit.md#ks-18) fixed).
   - A **`keys` binding** acts only if it is the winner, or a member of the winner's gesture group: a `Tap`, `Hold` or `DoubleTap` binding next to a deferred winner, on the same keys in any order, with the same `when:` as written, that matches the press by its own `allow_extra_modifiers` ([src/engine/winner.rs:84-115](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/winner.rs#L84-L115)). A losing binding neither fires, arms nor asks for suppression.
     - `Immediate`: it fires on a first key-down, and on an auto-repeat only if it sets `repeat`, which by default only `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)` do ([L475-L482](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L475-L482); [KS-10](audit.md#ks-10) fixed). It asks for suppression if it sets `block_default`.
     - `Tap`/`Hold`/`DoubleTap`: repeats are skipped, and only the first binding of each trigger kind in the group acts ([src/engine/winner.rs:117-140](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/winner.rs#L117-L140)). The gesture is armed under the key just pressed, which is a key of the combo, and a Hold also asks for a timer. It asks for suppression only if it sets `block_default`, which the loader resolves to `false` for a combo made only of modifiers ([L483-L528](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L483-L528); roadmap [item 1.3](roadmap.md#13-block_default-for-tap-hold-and-doubletap)).
   - **Sequence.** The `when:` is checked first, and repeats are skipped. A step matches when its keys are held, by a subset test ([KS-15](audit.md#ks-15), [KS-16](audit.md#ks-16), open). `Advanced` and `Completed` ask for suppression, whichever `keys` binding won the key-down ([L531-L557](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L531-L557)).
   - **Chord.** The `when:` is checked first, and repeats are skipped. The pressed code counts for a chord key that accepts it. Any key of the chord asks for suppression ([L558-L576](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L558-L576)).
10. **Suppression record.** A first key-down is suppressed if anything asked, and the engine records the key as blocked; a first key-down that passes is recorded as passed. An auto-repeat follows that record, whatever the bindings asked this time. A completed chord or sequence also records the keys whose key-ups it swallows, but only keys still held whose key-down did not reach Windows ([L580-L612](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L580-L612); [KS-06](audit.md#ks-06), [KS-30](audit.md#ks-30), [KS-32](audit.md#ks-32) fixed).
11. **Key-up.** The key is released, a pending Tap fires if its deadline has not passed, and a pending Hold is cancelled before its deadline or left to its timer from the deadline on. The key-up of a key recorded as passed always passes. Otherwise it is suppressed if the key was recorded as blocked or a Tap or Hold of the press was pending ([src/engine/mod.rs:617-644](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L617-L644)). `when:` conditions are not evaluated on key-up.
12. **Menu mask.** Once the record is updated, the decision asks for the menu mask key if the event is suppressed while Windows has an Alt or Win key down whose key-down it saw: one recorded as passed, or one the resync found held ([src/engine/mod.rs:646-654](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L646-L654), [src/engine/keys.rs:230-241](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L230-L241); [KS-12](audit.md#ks-12) fixed, roadmap [item 1.2](roadmap.md#12-mask-key-and-auto-blocker-v2)). An Alt or Win whose key-down was swallowed does not count. A timer never asks for the mask.
13. **Carry-out.** After the lock and the snapshot are released, the adapter first sends the menu mask key if the decision asks for it: vk 0xE8 down and up in one `SendInput`, tagged with the sentinel. Then it starts a `hold-timer` thread for each requested timer and runs the actions in order, on the hook thread ([src/hook/keyboard_hook_callback.rs:199-221](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L199-L221)). A `SendInput` that inserts fewer than the two mask events is counted in `menu_mask_short` ([src/hook/keyboard_hook_callback.rs:254-265](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L254-L265)). Finally the callback returns `LRESULT(1)` if the decision suppresses the event, and `CallNextHookEx` otherwise ([L343-L353](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L343-L353)).

The first diagram shows the checks every event passes and the key-down path; the second shows the key-up path.

```mermaid
flowchart TD
    EV["WH_KEYBOARD_LL event on the main thread"] --> C0{"code below zero or remapping off?"}
    C0 -->|"yes"| PASS["CallNextHookEx, the OS sees the event"]
    C0 -->|"no"| SEN{"dwExtraInfo equals own sentinel?"}
    SEN -->|"yes, own injection"| PASS
    SEN -->|"no"| LOCK["Engine::on_key under the engine lock"]
    LOCK --> SYNC["resync modifiers from the OS, skipping swallowed ones"]
    SYNC --> DIR{"key-down or key-up?"}
    DIR -->|"key-up"| UPREF["key-up path, second diagram"]
    DIR -->|"key-down"| REP["mark held, an already held key is an auto-repeat"]
    REP --> WIN["elect one keys binding: most keys, then a when, then file order"]
    WIN --> NEXT{"next binding?"}
    NEXT -->|"yes"| KND{"binding kind"}
    KND -->|"auto-blocker"| BLK["hotkey alone in Windows and its when holds? suppress, or mask for MaskOnly"]
    KND -->|"keys"| MT{"the winner or its gesture group?"}
    MT -->|"no"| NEXT
    MT -->|"Immediate"| IMM["fire, on a repeat only with repeat, suppress if block_default"]
    MT -->|"Tap, Hold or DoubleTap"| DEF["skip repeats, arm under the key, suppress if block_default"]
    KND -->|"Sequence"| SQ["when, skip repeats, step held? subset test"]
    KND -->|"Chord"| CH["when, skip repeats, pressed code in the chord?"]
    BLK --> NEXT
    IMM --> NEXT
    DEF --> NEXT
    SQ --> NEXT
    CH --> NEXT
    NEXT -->|"none left"| REC["record: repeats follow their key-down"]
    REC --> MSK["suppressed while Alt or Win is down in Windows? ask for the mask"]
    MSK --> OUT["after the lock: mask key, Hold timers, actions"]
    OUT --> SUP{"suppress?"}
    SUP -->|"yes"| DROP["return 1"]
    SUP -->|"no"| PASS
```

```mermaid
flowchart TD
    UP["key-up after the same entry checks: release the key"]
    UP --> TAP["pending Tap fires if before its deadline, pending Hold cancelled or left to its timer"]
    TAP --> PQ{"key-down recorded as passed?"}
    PQ -->|"yes"| PASS["CallNextHookEx, the OS sees the event"]
    PQ -->|"no"| BQ{"recorded as blocked, or a Tap or Hold was pending?"}
    BQ -->|"yes"| SW["swallow the key-up, after the mask key if Alt or Win is down in Windows"]
    BQ -->|"no"| PASS
```

> [!NOTE]
> **Foreground changes.** `foreground_changed` hands every `EVENT_SYSTEM_FOREGROUND` to `Engine::on_foreground_changed`: the engine resyncs the modifiers, again skipping swallowed ones, and forgets which key-ups were due to be swallowed ([src/engine/mod.rs:381-396](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L381-L396)). A held key's repeats stay swallowed, but its release then reaches Windows as a lone key-up, unless a Tap or Hold of the press is still pending. Roadmap [item 2.3](roadmap.md#23-control-window-and-state-reset) revisits this.

> [!NOTE]
> **Lone Win or Alt ([KS-12](audit.md#ks-12), fixed).** For a combo such as `Win+Q`, only the `Q` events are dropped, and Windows has Win down. The engine therefore asks for the menu mask key, and Windows sees 0xE8 between Win down and Win up, so the release opens no Start menu; a lone Alt likewise activates no menu bar. 0xE8 types nothing, but applications and other hooks see the injected events. What Windows does with them is a manual check of item 1.2. Not covered: a `Tap`, `Hold` or `DoubleTap` on `Alt` or `Win` alone whose key passes, because nothing is swallowed there ([known issue 66](config/known-issues.md#s19-9)).

> [!WARNING]
> **Auto-blocker ([KS-05](audit.md#ks-05), partly fixed).** The blocker for the language hotkey acts only when Windows would see the hotkey and nothing else, so Alt+Shift+Tab with Tab held reaches Windows. Pressed modifiers first, Alt, Shift, X, the template's `block_original_combo: true` still swallows Shift, and the application gets Alt+X; the swallowed Shift is masked, so no menu opens. `block_original_combo: MaskOnly` swallows nothing and sends the mask key inside the hotkey instead. Whether that keeps Windows from switching the layout is an unverified hypothesis, so `MaskOnly` is marked experimental ([§15.5](config/matching.md#s15-5)).

## Core data structures

All hot state lives in one `KeyboardHook` value ([src/hook/keyboard_hook.rs:40-67](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L40-L67)), held in a process-wide `LazyLock` static ([src/hook/keyboard_hook_callback.rs:100-112](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L100-L112)). A `LazyLock` is used instead of a plain `static` because creating the `ArcSwap` allocates, so `KeyboardHook::new` cannot be `const`. `get_hook()` returns `&'static KeyboardHook`, which is what `spawn_watcher` and the daemon's `DaemonHandle` take. The reference is shared rather than `&'static mut`, so a re-entered callback stays memory-safe: all mutable state sits behind atomics, `ArcSwap` or the engine lock.

| Field or static | Type | Written by | Read by |
|---|---|---|---|
| `KEYBOARD_HOOK` | `LazyLock<KeyboardHook>` | First `get_hook()` call | Everything |
| `bindings` | `ArcSwap<Vec<Binding>>` | `update_bindings`: main thread at startup, `config-watcher` on reload | The callback (one snapshot per event), IPC `status` |
| `hook_handle`, `foreground_hook` | `AtomicPtr<c_void>` | `install`, `uninstall` (`swap` to null), main thread only | `is_installed` |
| `enabled` | `AtomicBool` | `set_enabled`, under the engine lock | Both callbacks, before and under the lock; IPC `status` |
| `engine` | `Mutex<Engine>` | `with_engine` only | `with_engine` only |
| `DECIDING` | `thread_local! Cell<bool>` ([src/hook/keyboard_hook.rs:18-38](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L18-L38)) | `with_engine` | `with_engine`, to refuse re-entry |
| `SENTINEL` | `OnceLock<usize>` ([src/core/constants.rs:5](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/constants.rs#L5)) | First `injected_sentinel()` call | Every `SendInput` builder, the callback |
| `MAIN_THREAD_ID` | `AtomicU32` ([src/core/app.rs:21](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L21)) | `App::run` | Ctrl+C handler; the IPC `exit` command uses the copy in `DaemonHandle` |
| `LOGGER` | `DaemonLogger` holding a `OnceLock` of the writer's queue ([src/logging/mod.rs:55-57](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/logging/mod.rs#L55-L57)) | `logging::init`, once | Every `log` call, on any thread |
| `COUNTERS` | four `AtomicU64` counts ([src/logging/counters.rs:103-104](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/logging/counters.rs#L103-L104)) | `counters::bump`, the hook thread among others; `AcqRel` | The `log-writer` thread, which takes and resets them for its report |

The single-instance guard is a local of `main`, not a static: it lives until `run` returns ([src/main.rs:64-67](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/main.rs#L64-L67)).

<a id="gesture-state-maps"></a>
The engine's own state ([src/engine/mod.rs:313-322](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L313-L322)) is plain data:

| Part | Type | Holds |
|---|---|---|
| `keys` | `KeyState { active: [bool; 256], last_down: [LastDown; 256] }` ([src/engine/keys.rs:56-85](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L56-L85)) | The keys the engine believes are held, by the sided code the hook reports plus the generic modifier codes the resync writes; and for each key whether its latest key-down was suppressed, passed, unknown, or suppressed and then forgotten by a foreground change |
| `gestures` | three `Vec`s of pending entries and a generation counter ([src/engine/gestures.rs:36-82](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L36-L82)) | Pending Taps and DoubleTaps with deadlines, pending Holds with deadlines and generations |
| `mask_owed` | `bool` | A `MaskOnly` auto-blocker matched the last key-down, so the next key event asks for the menu mask key too |
| `chords`, `sequences` | `Vec`s of pending entries ([src/engine/chords.rs:30-43](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L30-L43), [src/engine/sequences.rs:26-36](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L26-L36)) | The keys seen so far or the next step, the action and the deadline |

<a id="arcswapvecbinding"></a>
### The binding list

Reads are lock-free: the callback takes one `Guard` per event and iterates it. `update_bindings` sorts the new list by `key_count()` descending with a stable sort, then publishes it with one atomic store ([src/hook/keyboard_hook.rs:106-115](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L106-L115)). The stable sort keeps load order among bindings with the same key count, which is the last tie-break of the winner election; the loader appends auto-blockers after the file's bindings, so they come last among equals. `key_count` is the number of keys for a Combo or Chord and the number of steps for a Sequence ([src/data/binding.rs:34-47](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/binding.rs#L34-L47)). The list is not engine state: the engine gets the snapshot on every call, so a reload never touches the engine, and pending gestures keep the clones of the actions they were armed with. A `BindAction::Sequence` holds its steps in an `Arc`, so a macro that is running keeps its steps after a reload.

<a id="atomicbool-256-active-keys-table"></a>
<a id="atomicu64-4-blocked-key-bitmap"></a>
### The per-key table

The engine's `KeyState` replaces the audit-era `[AtomicBool; 256]` active-keys table and `[AtomicU64; 4]` blocked-key bitmap. It has one writer, the engine, under its lock. Its two halves are the held keys, which also detect auto-repeat, and the suppression record that keeps a key's repeats and key-up in line with its key-down ([src/engine/keys.rs:1-12](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L1-L12)):

| Record | Set when | Repeats | Key-up |
|---|---|---|---|
| `Unknown` | the key is up, or the resync found a modifier held whose key-down the engine never saw | follow what the bindings ask; once one passes, the key is recorded as passed | passes unless a Tap or Hold was pending |
| `Passed` | the key-down reached Windows | pass | passes |
| `Suppressed` | the key-down was swallowed | swallowed | swallowed |
| `Forgotten` | a foreground change while the key-down was recorded as suppressed | swallowed | passes unless a Tap or Hold was pending |

Only the modifier codes are resynced from the OS. A missed key-up of any other key leaves a phantom held key until its next press, which then reads as an auto-repeat. With the trigger-key rule such a key no longer makes combos fire on unrelated key-downs, and `swch off`/`on` rebuild the table from the OS ([KS-29](audit.md#ks-29), not independently verified; partly addressed).

### The PID-mixed sentinel

Every synthesized event carries `injected_sentinel()` in `dwExtraInfo`. On 64-bit targets it is `(pid << 32) | 0xDEADBEEF` ([src/core/constants.rs:15-28](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/constants.rs#L15-L28)). The process ID makes collisions with other injectors (AutoHotkey, PowerToys), which tend to use small constants, very unlikely. `is_injected_event` is an exact comparison ([src/core/constants.rs:30-32](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/constants.rs#L30-L32)).

### Decisions, timers and tokens

`Decision` carries `suppress`, the actions to `fire` in order, the `timers` to start, and `mask`, which asks the adapter to send the menu mask key before the hook returns (roadmap item 1.2; [src/engine/mod.rs:282-301](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L282-L301)). The adapter destructures the whole `Decision`, so a new field does not compile until it is carried out. A `TimerRequest` pairs a due `Instant` with an opaque `TimerToken` of key code and generation ([src/engine/mod.rs:266-280](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L266-L280)). The `Env` trait is the engine's only window on the world: the clock, the OS key state and the foreground app and title ([src/engine/env.rs:9-53](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/env.rs#L9-L53)). The adapter answers it with live Win32 calls in `HookEnv`; the test harness answers it with a fake clock and a modelled key state.

## Gesture state machines

<a id="tap-hold-and-doubletap-tap_staters"></a>
### Tap, Hold and DoubleTap (`engine/gestures.rs`)

Each gesture kind has its own table of pending entries, keyed by the code of the key-down that armed it, so Tap and Hold on one key are armed and resolved independently ([src/engine/gestures.rs:1-27](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L1-L27)).

| Trigger | Key-down | Later | Key-up | Action runs on |
|---|---|---|---|---|
| `Tap(ms)` | Store a deadline, unless a live Tap is pending on the key ([L91-L112](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L91-L112)) | Nothing; an entry past its deadline counts as absent | Remove the entry; fire if the key-up came strictly before the deadline ([L114-L121](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L114-L121)) | Main thread |
| `Hold(ms)` | Store a deadline and a new generation, unless a Hold is pending; return a timer request ([L125-L154](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L125-L154)) | The `hold-timer` hands its token back; the entry with that key and generation fires ([L171-L179](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L171-L179)) | Before the deadline: cancel. From the deadline on: leave the entry to its timer ([L156-L169](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L156-L169)) | `hold-timer` thread |
| `DoubleTap(ms)` | If an entry is pending and its deadline has not passed, consume it and fire; otherwise store a new deadline ([L197-L219](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/gestures.rs#L197-L219)) | Nothing | Nothing in the gesture table; the suppression record swallows the key-up | Main thread |

Only the winning binding or its gesture group arms, and only the first binding of each trigger kind in the group, so two `DoubleTap` bindings on one key no longer share an entry on the first press (roadmap item 1.6). The DoubleTap table is still keyed by the key code across presses: an entry that one binding armed is consumed by the next press of that key that another `DoubleTap` binding wins, such as `Shift+CapsLock` after `CapsLock`, and the first binding's action fires ([known issue 74](config/known-issues.md#s19-9); a follow-up for roadmap [item 2.2](roadmap.md#22-timers-on-the-main-thread)).

The generation token is what fixed [KS-11](audit.md#ks-11): a timer from an earlier press finds a different generation and does nothing. Leaving an overdue Hold to its timer is what makes equal Tap and Hold terms exclusive ([KS-25](audit.md#ks-25)):

```mermaid
sequenceDiagram
    participant U as User
    participant CB as Hook callback
    participant E as Engine
    participant T1 as hold-timer of press 1
    U->>CB: F13 down, press 1
    CB->>E: on_key arms Tap and Hold, generation 1
    CB->>T1: start, sleep until the deadline
    U->>CB: F13 up before the deadline
    CB->>E: on_key fires the Tap and cancels Hold 1
    U->>CB: F13 down, press 2
    CB->>E: on_key arms Hold, generation 2
    T1->>E: on_timer with token 1
    E-->>T1: no entry with generation 1, nothing fires
    Note over E: press 2 keeps its own Hold and Tap
```

If a `hold-timer` thread cannot be started, the adapter counts the failure (`hold_timer_not_started`, which the log writer reports; [src/hook/keyboard_hook_callback.rs:281-304](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L281-L304)) and tells the engine to forget that Hold, which then never fires ([KS-33](audit.md#ks-33), partly fixed). Tap and DoubleTap on the same key are still not disambiguated ([Tap versus DoubleTap disambiguation](gaps.md#tap-versus-doubletap-disambiguation)).

<a id="chord-chord_staters"></a>
### Chord (`engine/chords.rs`)

A chord fires when all of its keys go down within `window` of the first one, in any order ([src/engine/chords.rs:58-116](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L58-L116)). For each Chord binding on each non-repeat key-down:

1. If no chord key accepts the pressed code, the result is `NotInChord` ([L72-L74](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L72-L74)). Chord keys are `KeySpec`s, so `"Shift"` accepts either Shift key and `"LShift"` only the left one ([KS-07](audit.md#ks-07) fixed). No list is allocated for the comparison.
2. Expired entries are dropped (lazy expiry, no timer thread), and the entry with the same key set is found or created with `expires_at = now + window` ([L76-L114](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L76-L114)). The window is not extended by later keys.
3. When every chord key accepts a code seen so far, the chord completes: the action fires, and the seen keys that are still held and whose key-down did not pass are recorded for a swallowed key-up ([L92-L102](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L92-L102); [KS-30](audit.md#ks-30) fixed).

Every key-down of a chord key is suppressed, whether or not the chord completes. Keys of a failed chord are lost; there is no replay buffer ([src/engine/chords.rs:10-16](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L10-L16)). The loader rejects chord keys that overlap, so one press never counts twice.

<a id="sequence-sequence_staters"></a>
### Sequence (`engine/sequences.rs`)

A sequence is a list of combo steps pressed in order, with at most `max_gap` between steps ([src/engine/sequences.rs:53-120](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L53-L120)). Pending entries are identified by the step list, so bindings with identical steps share one slot. For each Sequence binding on each non-repeat key-down:

1. Expired entries are dropped ([L72-L74](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L72-L74)).
2. If an entry is pending and the next step's keys are held, the entry advances and its deadline moves to `now + max_gap`. On the last step `Completed` returns the action and the last step's non-modifier keys for a swallowed key-up ([L76-L92](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L76-L92), [L122-L142](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L122-L142)).
3. Any other key-down removes the pending entry, and the same key-down is then checked as step 0 ([L93-L116](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L93-L116)).

The state machine still sees only the held keys, not which key was just pressed. Rollover therefore completes a step ([KS-15](audit.md#ks-15)), and a step that adds a modifier fails, because the modifier's own key-down does not cover the full step and aborts the sequence ([KS-16](audit.md#ks-16)). Both are open until roadmap [item 3.2](roadmap.md#32-sequence-and-chord-matching). Completion no longer swallows key-ups whose key-down reached the application ([KS-32](audit.md#ks-32), fixed).

## Conditions

`Condition` is a small expression tree: `Always`, `AppEquals`, `TitleContains`, `TitleEquals`, `Not`, `And` and `Or` ([src/data/condition.rs:28-55](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L28-L55)). Its equality is the condition as written, which the gesture group and the duplicate warning use. It is evaluated against the `ForegroundInfo` trait, so the engine can evaluate it against a fake foreground in tests ([src/data/condition.rs:65-105](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L65-L105), [src/engine/env.rs:55-68](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/env.rs#L55-L68)). In the daemon the answers come from a `ForegroundContext` that `HookEnv` captures at most once per event, on the first question ([src/hook/keyboard_hook_callback.rs:114-155](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L114-L155)):

- `capture()` makes one call, `GetForegroundWindow` ([src/data/condition.rs:117-129](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L117-L129)).
- `app()` and `title()` are `OnceCell`s. The Win32 calls run on first use and are cached for the rest of that keystroke ([src/data/condition.rs:131-141](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L131-L141)).
- `app()` opens the process with `PROCESS_QUERY_LIMITED_INFORMATION` and reads its image path with `QueryFullProcessImageNameW` in the native form; the file name is the part after the last `\` ([src/data/condition.rs:143-158](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L143-L158), [src/data/condition.rs:184-231](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L184-L231); [KS-03](audit.md#ks-03) fixed). The handle is closed by `Owned` on every path.
- All comparisons fold case with Unicode lowercasing; two ASCII strings compare without allocating ([src/data/condition.rs:87-96](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/condition.rs#L87-L96); [KS-28](audit.md#ks-28) fixed).

`Always` never asks the foreground, so a config without `when:` pays nothing. A `keys` binding's condition is evaluated only once its keys match the press ([src/engine/winner.rs:73-78](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/winner.rs#L73-L78), roadmap item 1.6), and an auto-blocker's only once its hotkey matches ([src/engine/mod.rs:449-451](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L449-L451), item 1.2), so a conditional `keys` binding no longer costs a lookup on the key-downs of other keys. `sequence` and `chord` bindings still evaluate theirs first, on every key-down that is not a repeat ([src/engine/mod.rs:531-560](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L531-L560)). The lookups run on the hook thread and under the engine lock. Roadmap [item 3.5](roadmap.md#35-hot-path-budget) caches the names per window. Conditions are not evaluated on key-up or in timers: a Tap or Hold armed under a condition fires later without checking it again. An auto-blocker carries the `when:` of the bindings that asked for it, OR'ed, and `Always` if one of them has none ([src/config/loader.rs:403-422](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L403-L422); [KS-35](audit.md#ks-35) fixed).

## Actions and input synthesis

`BindAction` has ten variants ([src/core/windows_actions.rs:17-51](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L17-L51)). `execute` dispatches them synchronously on the calling thread ([L118-L143](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L118-L143)). For Immediate, Tap, DoubleTap, Chord and Sequence-kind bindings that thread is the main thread, and for Hold it is the `hold-timer` thread.

| Action | Implementation | Blocking work on the calling thread |
|---|---|---|
| `SwitchLanguage`, `SwitchLanguageBackward` | `SystemFunction`: `PostMessageW(GetForegroundWindow(), WM_INPUTLANGCHANGEREQUEST, INPUTLANGCHANGE_FORWARD or _BACKWARD)` ([src/system/system_function.rs:43-80](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/system/system_function.rs#L43-L80)) | None (posted) |
| `ToggleCapsLock` | `SendInput` of `VK_CAPITAL` down and up ([src/system/system_function.rs:118-158](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/system/system_function.rs#L118-L158)) | `SendInput` |
| `PressKey(vk)`, `Media(key)` | `press_vk`: `SendInput` down and up ([src/core/windows_actions.rs:189-226](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L189-L226)) | `SendInput` |
| `PostMessage` | `PostMessageW` to the foreground window ([L228-L241](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L228-L241)) | None |
| `Launch` | `std::process::Command::spawn` ([L243-L257](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L243-L257)) | `CreateProcessW` ([KS-08](audit.md#ks-08)) |
| `OpenUrl` | `ShellExecuteW` with the `open` verb ([L259-L282](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L259-L282)) | Shell handler start-up, no COM initialisation ([KS-08](audit.md#ks-08)) |
| `Sequence` | `spawn_sequence` ([src/data/sequence.rs:102-122](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/sequence.rs#L102-L122)) | Only the thread spawn |
| `DoNothing` | Nothing; used by auto-blockers, whose action the engine never fires | None |

An action that fails logs one `[ERROR]` line, for example `[ERROR] Launch "no-such-program.exe": program not found`, which also goes into `daemon.log`. A key press that `SendInput` cuts short is not logged on the hook thread: it is counted in `key_press_short` ([L222-L225](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L222-L225)).

`SystemFunction` also implements `LockWorkstation`, `ShowDesktop` and `TaskManager` ([src/system/system_function.rs:82-116](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/system/system_function.rs#L82-L116)), but `to_system_function` maps only the three language and CapsLock actions ([src/core/windows_actions.rs:145-152](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L145-L152)) and the schema has no variant for them, so they cannot be reached from a config. `get_system_combination` routes through `registry::get_system_hotkey`, which provides the auto-blocker combo ([src/system/registry.rs:65-78](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/system/registry.rs#L65-L78)). It reads `HKCU\Keyboard Layout\Toggle` (`Hotkey`, then `Language Hotkey`), only reads, and its modifiers mean either side. A pure function resolves the values: the first that names a hotkey (`"1"` Alt+Shift, `"2"` Ctrl+Shift, `"4"` the grave accent) decides, and so does `"3"`, "Not assigned", which gives no hotkey and therefore no auto-blocker. Only missing, unreadable or unknown values fall back to `Alt+Shift` ([src/system/registry.rs:80-114](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/system/registry.rs#L80-L114), [src/system/registry.rs:223-231](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/system/registry.rs#L223-L231); [KS-17](audit.md#ks-17) fixed, roadmap [item 1.4](roadmap.md#14-tri-state-language-hotkey)).

### Macros (`data/sequence.rs`)

`spawn_sequence` captures `GetForegroundWindow()` when the action fires, passes the `HWND` to a new thread as an integer, and runs the steps in order ([src/data/sequence.rs:111-135](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/sequence.rs#L111-L135)). `Window(...)` steps act on that captured window, not on whatever has focus later ([L264-L291](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/sequence.rs#L264-L291)). `Delay` sleeps in the worker, never in the callback. Workers have no limit and no cancellation, so `swch off`, a reload or a new trigger cannot stop a running macro. Since item 1.1 a held Immediate macro binding starts one worker per press, unless it sets `repeat: true` ([KS-10](audit.md#ks-10) fixed). `spawn_sequence` uses `thread::spawn`, which panics if the thread cannot be started, on the hook thread for an Immediate binding ([Other functional gaps](gaps.md#other-functional-gaps)).

### Input synthesis

There are four separate `SendInput` builders: `press_vk` in `windows_actions.rs`, `type_text`/`press_key`/`press_combo` in `sequence.rs` ([L137-L245](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/sequence.rs#L137-L245)), `SystemFunction::press_key`, and `menu_mask_inputs` in the hook adapter, which item 1.2 added for the menu mask key ([src/hook/keyboard_hook_callback.rs:223-240](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L223-L240)). All of them set `dwExtraInfo` to the sentinel. The virtual-key builders set `wScan` to 0 and never `KEYEVENTF_EXTENDEDKEY`; `type_text` puts the UTF-16 code unit in `wScan` (not independently verified as a defect: [KS-36](audit.md#ks-36)). `type_text` sends each UTF-16 code unit as a `KEYEVENTF_UNICODE` down/up pair, so text does not depend on the keyboard layout. `press_combo` splits the keys into modifiers and payload, presses modifiers, then payload, and releases them in reverse order. None of the builders releases modifiers the user is physically holding. The trigger's own modifiers (for example the `Ctrl+Alt` of `Ctrl+Alt+L`) reached the OS before the payload key was suppressed, so synthesized keys combine with them ([KS-19](audit.md#ks-19), [src/core/windows_actions.rs:192](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L192)). Roadmap [item 3.1](roadmap.md#31-single-input-module) merges the builders.

## Config loading and hot reload

`config::load` runs at startup and `config::reload` on every reload ([src/config/loader.rs:151-182](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L151-L182)). Both log through `log`: `[INFO] loading config from <path>`, or `[INFO] generated default config at <path>` for a written template. They share one pipeline:

1. Read the file. A missing file, or a missing folder on its path, is `ConfigError::Missing` ([L184-L197](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L184-L197)). Only `load` with `<exe_dir>\config.ron` then writes the bundled `default_config.ron` and reads it again ([L160-L165](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L160-L165)); `reload` never writes ([KS-23](audit.md#ks-23) fixed).
2. Skip one leading UTF-8 byte-order mark ([L255-L258](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L255-L258); [KS-24](audit.md#ks-24) fixed).
3. Deserialize `RawConfig` with `IMPLICIT_SOME` ([L260-L270](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L260-L270)). Every struct of the schema has `deny_unknown_fields`, and `bindings` is required ([src/config/schema.rs:18-28](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/schema.rs#L18-L28); [KS-22](audit.md#ks-22) fixed).
4. Convert each `RawBinding`. Errors from all entries are collected, and any error rejects the whole file ([L272-L284](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L272-L284)). Trigger keys become `KeySpec`s; timing values must be 1 to 10000 ms ([L621-L634](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L621-L634); [KS-27](audit.md#ks-27) fixed). A missing `block_default` resolves to `false` for a `Tap`, `Hold` or `DoubleTap` combo made only of modifiers and to `true` otherwise, and `repeat: true` on a binding that never fires on a repeat is an error ([L484-L508](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L484-L508)). An empty list only logs a warning ([L286-L288](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L286-L288)).
5. Every binding that duplicates an earlier one gets a warning, such as `[WARN] binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them`. The file still loads ([L297-L319](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L297-L319); roadmap item 1.6).
6. `expand_with_auto_blockers` appends one blocker per distinct system hotkey and mode of the flagged bindings, unless a `keys` binding of the file uses exactly the hotkey's keys ([L321-L401](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L321-L401); [KS-18](audit.md#ks-18) fixed). The blocker takes the `when:` of the bindings that asked for it. `MaskOnly` on a hotkey that is not made only of modifiers gives a blocker that swallows, as `true` does, and a warning. The system hotkey comes from a function parameter, which tests replace so that they do not depend on the machine's registry ([L211-L234](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L211-L234)). The warnings are logged on the thread that loads: the main thread at startup, `config-watcher` on a reload.

The watcher ([src/hook/config_watcher.rs:93-170](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L93-L170)) watches the parent directory without recursion, so editors that save through a temporary file and a rename keep working ([L117-L121](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L117-L121)). The filter closure forwards only `Modify`, `Create` and `Remove` events whose file name equals the config's, compared with `CompareStringOrdinal` ignoring case ([L63-L91](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L63-L91); [KS-21](audit.md#ks-21) fixed). The `config-watcher` thread waits until 250 ms pass without a new event ([L131-L154](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L131-L154)), then reloads.

```mermaid
sequenceDiagram
    participant ED as Editor
    participant NB as notify backend thread
    participant CW as config-watcher thread
    participant LD as config loader
    participant KH as KeyboardHook
    participant MT as Main thread
    ED->>NB: save writes or renames files in the config directory
    NB->>NB: drop events for other file names, ignoring case
    NB->>CW: unit message for Modify, Create or Remove
    loop until 250 ms pass with no new message
        NB->>CW: later messages restart the wait
    end
    CW->>LD: reload the absolute config path
    LD->>LD: read, skip a BOM, parse RON, convert bindings, add auto-blockers
    alt load succeeded
        LD-->>CW: Vec of Binding
        CW->>KH: update_bindings sorts and stores a new Arc
        MT->>KH: next event takes the new snapshot
    else file missing or invalid
        LD-->>CW: ConfigError, nothing written
        CW->>CW: log the error and keep the previous list
    end
```

The design keeps a broken save from taking the daemon down: on error the old list stays live ([L156-L170](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/config_watcher.rs#L156-L170)). The three paths around that guarantee that the audit found are closed: a relative path, a deleted file and a misspelled `bindings` ([KS-21](audit.md#ks-21), [KS-23](audit.md#ks-23), [KS-22](audit.md#ks-22)). Reload results go to the daemon console and to `daemon.log`, every line of an error with its timestamp and `[ERROR]` (roadmap [item 1.7](roadmap.md#17-file-logging)). Pending gesture entries from the old config are not cleared ([item 2.3](roadmap.md#23-control-window-and-state-reset)).

## IPC and the swch client

The transport is the `interprocess` crate's local socket, which on Windows is the named pipe `\\.\pipe\key-switch-rs.sock` ([src/ipc.rs:31-43](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/ipc.rs#L31-L43)). Each connection carries exactly one request line and one response line, with no length prefix or other framing ([src/ipc.rs:1-17](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/ipc.rs#L1-L17)).

| `swch` subcommand | Request line | Daemon side | Response line |
|---|---|---|---|
| `on` | `on` | `DaemonControl::set_enabled(true)` on the connection worker | `OK: enabled` |
| `off` | `off` | `set_enabled(false)` | `OK: disabled` |
| `status`, `ping` | `status` | The enabled flag and the binding count | `OK: running (N bindings live)`, with `disabled` in place of `running` while remapping is off |
| `exit`, `quit`, `shutdown`, `stop` | `exit` | `PostThreadMessageW(main_tid, WM_QUIT)` ([src/core/app.rs:48-58](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/app.rs#L48-L58)) | `OK: shutting down` |
| `open`, `start`, `launch` | `status`, as a probe | Answers like `status`; if no daemon is listening, `swch` starts one with `ShellExecuteW("runas")` ([src/bin/swch.rs:71-142](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/bin/swch.rs#L71-L142)) | none printed from the daemon |
| `help`, `--help`, `-h` | none | Not involved; `swch` prints its usage and exits with 0 | none |
| anything else | none | Not involved; `swch` prints `swch: unknown command "..."` and its usage, and exits with 1 ([src/bin/swch.rs:49-53](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/bin/swch.rs#L49-L53)) | none |

The daemon parses requests case-insensitively after trimming and accepts `quit`/`shutdown` and `ping` as aliases ([src/ipc.rs:173-184](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/ipc.rs#L173-L184)); `stop` is an alias only in `swch`. Another client that sends an unknown line gets `ERR: unknown command "..."`. A command the daemon cannot carry out gets `ERR: could not …`, for example `ERR: could not shut down: main thread id unknown, cannot post WM_QUIT` ([src/hook/ipc_server.rs:175-196](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/ipc_server.rs#L175-L196)). `swch` turns `OK` and `ERR` into exit code 0 or 1, and treats a reply without a prefix as success ([src/ipc.rs:207-219](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/ipc.rs#L207-L219)).

The client side is `ipc::request`, which `swch` calls for every command ([src/ipc.rs:87-155](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/ipc.rs#L87-L155)). It runs the exchange on its own `ipc-request` thread and waits at most 5 s for the connection and the reply together, because interprocess pipe streams take no I/O timeout. A connect error is sorted by what it says about the daemon: a missing pipe means not running, while access denied, a busy pipe and a timeout mean a daemon runs.

```mermaid
sequenceDiagram
    participant SW as swch.exe
    participant RQ as ipc-request thread in swch
    participant PS as ipc-server thread
    participant WK as Connection worker
    participant KH as KeyboardHook
    participant MT as Main thread
    SW->>RQ: request, deadline 5 s
    RQ->>PS: connect to the key-switch-rs.sock pipe
    PS->>WK: spawn a thread for the connection
    RQ->>WK: one command line, for example off
    alt on or off
        WK->>KH: set_enabled flips the flag and resets the engine under its lock
        WK-->>RQ: OK enabled or OK disabled
    else status
        WK->>KH: enabled flag and binding count
        WK-->>RQ: OK running or disabled, with the count
    else exit
        WK->>MT: PostThreadMessageW WM_QUIT
        WK-->>RQ: OK shutting down
        MT->>KH: uninstall after the message loop returns
    end
    RQ-->>SW: reply, or an error after 5 s
    SW->>SW: print the reply, exit code 0 or 1
```

The listener is created by `ipc_server::bind` before the hooks are installed, with an explicit security descriptor, `PIPE_SDDL` ([src/hook/ipc_server.rs:54-103](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/ipc_server.rs#L54-L103); [KS-02](audit.md#ks-02) fixed): a protected DACL with full control for SYSTEM and Administrators, read and write for interactive users, and a medium mandatory label. A non-elevated `swch` can therefore reach an elevated daemon. Every interactive user of the machine can, too: under fast user switching or Remote Desktop another signed-in user can switch remapping, stop the daemon, or create a pipe instance that answers `swch` in the daemon's place, until roadmap [item 4.1](roadmap.md#41-ipc-hardening) gives each session its own pipe ([KS-40](audit.md#ks-40), open). A client that connects and never sends a newline still parks its worker in `read_line` forever ([src/hook/ipc_server.rs:155-160](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/ipc_server.rs#L155-L160); [KS-26](audit.md#ks-26), open). The `exit` reply is still written after `WM_QUIT` is posted ([KS-37](audit.md#ks-37), not independently verified).

## Elevation and integrity levels

`build.rs` adds two linker arguments, for the `key-switch-rs` binary only, only in release builds for a Windows target, and only when the target environment is `msvc`: `/MANIFEST:EMBED` and `/MANIFESTUAC:level='requireAdministrator' uiAccess='false'` ([build.rs:21-47](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/build.rs#L21-L47)). Other Windows toolchains get a cargo warning that no UAC manifest is embedded ([KS-42](audit.md#ks-42) fixed). `swch.exe` gets no elevation request and runs `asInvoker`, and debug builds of the daemon run at the caller's level. The rationale is UIPI: a low-level hook in a medium-integrity process does not see keystrokes aimed at higher-integrity windows. `app.manifest` in the repository root is still not passed to the linker, so its `supportedOS` block never ships.

Elevation does not cover the Secure Desktop (UAC consent, Ctrl+Alt+Del, the lock screen). While it has focus, the hook misses key-ups. Two repairs exist for this:

- The modifier resync on every event, described in [the key-event path](#the-key-event-path).
- A WinEvent hook on `EVENT_SYSTEM_FOREGROUND` with `WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS` ([src/hook/keyboard_hook.rs:183-211](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook.rs#L183-L211)). Its callback `foreground_changed` makes the engine resync the modifiers and forget which key-ups are due to be swallowed ([src/hook/keyboard_hook_callback.rs:356-385](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/hook/keyboard_hook_callback.rs#L356-L385)).

Both repairs resync only the modifier codes, and skip a modifier whose key-down the engine swallowed. Gesture state and the held state of the other keys are left as they are.

Running elevated has costs beyond the UAC prompt on every launch:

- Every `Launch` and `OpenUrl` child inherits the administrator token, and console children also share the daemon's console ([src/core/windows_actions.rs:243-257](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/core/windows_actions.rs#L243-L257), [KS-09](audit.md#ks-09), open until roadmap [item 3.3](roadmap.md#33-launch-and-openurl-without-elevation)).
- The pipe has to admit medium-integrity clients, so it admits every interactive user (above).
- In debug builds both binaries run at the same level, which hides elevation problems during development.

## Design strengths and architectural root causes

### Strengths

- **A pure, tested decision core.** `src/engine` has no Win32 calls, threads, locks or clock of its own, and `#![forbid(unsafe_code)]`. Scenario tests drive it from a RON config, a script of key events and a fake clock, and property tests check the suppression invariant over generated key scripts. The audit's scenarios are now regression tests: at `20af28d`, `cargo test --locked -- --list` shows 537 library tests, 178 of them in `engine::scenarios`.
- **One representation of trigger keys.** `KeySpec` is fixed at load time and used by combos, steps and chords alike, so sided and unsided names mean the same everywhere.
- **Lock-free, reload-safe binding list.** One `ArcSwap` load per event, and a reload is a single atomic store. Running macros keep their own `Arc`.
- **Sound shared state.** `&'static KeyboardHook` with interior mutability only. The single engine lock is held only while deciding, never across an action, and refuses re-entry instead of deadlocking.
- **Loop-proof injection.** The PID-mixed sentinel stops the daemon from reacting to its own `SendInput`.
- **Clear thread ownership.** Only the main thread installs and removes hooks; `on` and `off` are a flag. Only the `log-writer` thread writes log lines to the console and to `daemon.log`, so a slow console or disk never holds up the hook thread, which only counts what it would report.
- **One daemon per session.** The instance mutex is taken before anything else is touched, and the pipe is created before the hooks, so a daemon either owns its session and its pipe or installs nothing.
- **Config pipeline.** Serde types are separate from the runtime types, unknown fields are rejected, all binding errors are collected in one pass, a failed reload keeps the old list, and watching the parent directory with a 250 ms debounce handles editors that save through a rename.
- **Macros off the hook thread**, with the target window captured once.

### Root causes

Most verified defects came from a few structural decisions rather than from isolated slips. The table lists the causes the audit found at `7cb3521` and where each stands now; the [roadmap](roadmap.md) orders the rest.

| Root cause at the audit | Status at `20af28d` | Issues |
|---|---|---|
| Thread affinity is implicit: hooks belong to the thread that installs them, but `install`/`uninstall` could be called from any thread | **Removed**: only the main thread installs, and `on`/`off` are a flag | [KS-01](audit.md#ks-01) |
| Actions run on whichever thread resolves a gesture, mostly the hook thread | Open: actions still run on the hook thread, except Hold actions ([item 2.1](roadmap.md#21-action-executor)) | [KS-08](audit.md#ks-08) |
| Matching tests a snapshot of held keys instead of the event: no trigger key, subset matching | **Removed for combos and blockers**: trigger-key rule, exact modifiers and one winner per key press. Sequence steps still use a subset test ([item 3.2](roadmap.md#32-sequence-and-chord-matching)) | [KS-04](audit.md#ks-04), [KS-05](audit.md#ks-05), [KS-13](audit.md#ks-13), [KS-15](audit.md#ks-15), [KS-16](audit.md#ks-16) |
| Suppression is decided per event, with no model of the key's whole press | **Mostly removed**: repeats and key-ups follow the key-down, an `Immediate` action fires on a repeat only with `repeat`, and a key swallowed while Windows has Alt or Win down is masked. Modifiers that already passed still combine with synthesized output ([item 3.1](roadmap.md#31-single-input-module)) | [KS-06](audit.md#ks-06), [KS-10](audit.md#ks-10), [KS-12](audit.md#ks-12), [KS-19](audit.md#ks-19) |
| Two sources of truth for key state: the hook's own tracking and `GetAsyncKeyState` | **Removed for swallowed modifiers**: the resync skips them | [KS-14](audit.md#ks-14) |
| Key identity is normalised only at match time, and differently for chords | **Removed**: `KeySpec` | [KS-07](audit.md#ks-07) |
| Time is handled by detached sleeper threads with no generation token | **Mostly removed**: deadlines and generation tokens; Hold still has a thread per press ([item 2.2](roadmap.md#22-timers-on-the-main-thread)) | [KS-11](audit.md#ks-11), [KS-25](audit.md#ks-25) |
| The auto-blocker is modelled as an ordinary binding | **Removed**: it has its own matching rule, fires nothing, follows the file's bindings and takes their `when:` | [KS-05](audit.md#ks-05), [KS-18](audit.md#ks-18), [KS-35](audit.md#ks-35); [KS-17](audit.md#ks-17), a registry-parsing defect, is fixed too |
| The process boundary uses Win32 defaults: the default pipe DACL, the inherited admin token for children, no instance guard | **Mostly removed**: an explicit pipe SDDL and a single-instance guard per Windows session. Children still inherit the token, and one pipe name serves every session ([KS-40](audit.md#ks-40)) | [KS-02](audit.md#ks-02), [KS-09](audit.md#ks-09), [KS-20](audit.md#ks-20), [KS-26](audit.md#ks-26) |
| Lenient, startup-oriented config handling is reused unchanged for hot reload | **Removed**: `load` and `reload` differ, and the loader is strict | [KS-21](audit.md#ks-21), [KS-22](audit.md#ks-22), [KS-23](audit.md#ks-23), [KS-24](audit.md#ks-24) |
| The decision logic is tied to Win32 calls and cannot be unit-tested | **Removed**: the engine | Allowed all of the above to ship |

The remaining structural step is the one the audit pointed at: the hook only decides, and a single worker executes. Roadmap items [2.1](roadmap.md#21-action-executor) and [2.2](roadmap.md#22-timers-on-the-main-thread) move actions to an executor and timers to the main thread, which also removes the engine lock.
