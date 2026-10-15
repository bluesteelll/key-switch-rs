# Changelog

All notable changes to key-switch-rs are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Until 1.0, a breaking change to the `config.ron` syntax or behaviour bumps the
minor version; such entries sit under **Changed** and start with
"**Breaking:**". Issue ids such as KS-38 refer to the
[audit](https://bluesteelll.github.io/key-switch-rs/audit.html).

## [Unreleased]

### Added

- CI on `windows-latest` for every push and pull request
  (`.github/workflows/ci.yml`):
  - `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
    `cargo test --locked` and `cargo build --release --locked`;
  - a check that the release `key-switch-rs.exe`, and not `swch.exe`, embeds
    the `requireAdministrator` manifest;
  - a check of cargo's JSON messages that `cargo test --release --no-run` and
    `cargo bench --no-run` build no libtest harness for the daemon binary
    (KS-38);
  - a release build for `x86_64-pc-windows-gnu` with the GNU toolchain, which
    must link and warn that no UAC manifest is embedded (KS-42);
  - `cargo check` with Rust 1.88, the minimum supported version.
- Documentation book (mdBook): overview, architecture, building and running,
  the `config.ron` reference, the audit (KS-01 to KS-42), gaps and
  documentation drift, and the roadmap. `.github/workflows/book.yml` publishes
  it to [GitHub Pages](https://bluesteelll.github.io/key-switch-rs/) with the
  Node 24 releases of its actions: `actions/checkout@v7`,
  `actions/configure-pages@v6`, `actions/upload-pages-artifact@v5` and
  `actions/deploy-pages@v5`.
- Licence: `MIT OR Apache-2.0`, with `LICENSE-MIT` and `LICENSE-APACHE`.
- Package metadata in `Cargo.toml`: description, licence, repository,
  homepage, readme, keywords and categories, and `rust-version = "1.88"` as
  the minimum supported Rust version.
- This changelog.
- Per-binding `allow_extra_modifiers: true` for `keys` bindings: the binding
  also fires while modifiers it does not name are held. It is a load error
  on `sequence` and `chord` bindings.
- `tests/ipc_roundtrip.rs`: the `swch` client against test pipes with
  unique names: a server that never answers, a missing pipe, a pipe that
  refuses the client, a busy pipe and a server that answers, and the
  daemon's pipe server answering `status`, `off`, `on`, `status` through a
  stand-in for the daemon, with no keyboard hook installed.

### Changed

- The source is formatted with `cargo fmt`.
- The keyboard decisions moved into a pure decision core, `src/engine`
  (roadmap Workstream A, scope additions). The hook callback translates each
  event, asks `Engine::on_key` for a `Decision` and carries it out on the
  same thread as before; Hold actions still run on their own `hold-timer`
  thread. Tap and DoubleTap expire by deadline instead of a cleanup thread
  per press, so only Hold still starts a timer thread. Scenario tests
  (`engine::testing::Scenario`: a RON config, a script of key events and a
  fake clock) cover every binding kind and trigger.
- **Breaking:** key names keep their side in every trigger: `keys`,
  `sequence` steps and `chord` keys (KS-07). `Shift`, `Ctrl`, `Alt` and
  `Win` (`Super`) match either key; `LShift`, `RShift`, `LCtrl`, `RCtrl`,
  `LAlt`, `RAlt`, `LWin` and `RWin` (and the long forms such as `LeftShift`
  and `RightWin`) match only that key. `LWin` used to match both Win keys in
  `keys` and `sequence`, and `Win` meant the left Win key only in `chord`. A
  `chord` whose keys overlap, such as `["Shift", "LShift"]` or
  `["Win", "RWin"]`, is now a load error. Output keys (`PressKey`, `Key`,
  `Combo`) are unchanged: `Win` still sends the left Win key.
- The auto-blocker line printed at load names its keys
  (`SwitchLanguage: registry combination Alt+Shift`) instead of key codes.
- **Breaking:** modifiers match exactly (owner decision; roadmap open
  question 2). A `keys` binding fires only when the held modifiers (Shift,
  Ctrl, Alt, Win; either key of a pair counts as that modifier) are exactly
  the ones it names: `"CapsLock"` no longer fires while Shift is held, and
  `"Ctrl+J"` no longer fires on Ctrl+Shift+J. With the default config,
  CapsLock pressed with Ctrl, Alt or Win held now reaches Windows instead of
  switching the language. Auto-blockers match exactly too: the `Alt+Shift`
  blocker ignores Ctrl+Alt+Shift, and the `` ` `` blocker lets Shift+` through.
  Add `allow_extra_modifiers: true` to a binding for the old behaviour. On
  layouts with AltGr, right Alt arrives together with a left Ctrl, so a
  binding on `RAlt` needs `Ctrl+RAlt` or `allow_extra_modifiers: true`.
- **Breaking:** a `keys` binding on exactly the keys of the Windows hotkey
  now replaces the `block_original_combo` auto-blocker for that hotkey
  (KS-18). With a `when:` on that binding, such as
  `( keys: "Alt+Shift", action: SwitchLanguageBackward, when: AppEquals("code.exe") )`,
  the Windows hotkey now reaches Windows in every other window; it used to
  be blocked everywhere. The auto-blocker also lets the hotkey through
  while another key is down in Windows (KS-05): with the `` ` `` hotkey
  (registry value 4), a `` ` `` typed while another key is held now reaches
  Windows and may switch the layout.

### Fixed

- KS-01: `swch off` followed by `swch on` no longer leaves remapping dead.
  The keyboard and foreground hooks are installed once, on the daemon's
  main thread, and stay installed until it exits; they used to be removed
  and reinstalled on a short-lived IPC thread, whose exit took the new hooks
  with it, and `swch exit` then stopped at a `[FATAL]` prompt. `swch off`
  now makes the hook pass every key on untouched and `swch on` turns
  remapping back on; `swch status` answers `disabled` while remapping is
  off (it used to say `running` whenever a hook handle was stored). Each
  switch drops pending `Tap`, `Hold`, `DoubleTap`, `chord` and `sequence`
  state, so a `Hold` armed just before `swch off` no longer fires (KS-31,
  an unverified report), and a key still held at `swch on` counts as
  already pressed. A key swallowed by a binding and released while
  remapping is off reaches Windows as a lone key-up. A running macro keeps
  running after `swch off` (roadmap item 2.1).
- KS-02: the daemon's pipe has an explicit security descriptor: full
  control for SYSTEM and Administrators, read and write for interactive
  users, and a medium integrity label. `swch` in a normal console now
  reaches a daemon started elevated by `swch open`, a double-click or a
  Scheduled Task; it used to fail with "Access is denied. (os error 5)".
  Everyone and anonymous logons no longer get read access, and processes
  below medium integrity cannot open the pipe. Read and write go to every
  interactive user of the machine, though: under fast user switching or
  Remote Desktop another signed-in user can switch remapping on or off,
  stop the daemon, or create a pipe instance that answers `swch` in the
  daemon's place, until roadmap item 4.1 gives each session its own pipe.
- `swch` no longer waits forever for a daemon that accepts the connection
  but does not answer, or whose pipe stays busy: after 5 s it prints
  `swch: cannot reach daemon (the daemon did not answer within 5s).` and
  exits with 1. Without a daemon it prints
  ``swch: the daemon is not running. Use `swch open` to start it.``.
  `swch open` treats a pipe that refuses this console, a busy pipe and a
  daemon that does not answer as a running daemon and starts no second one
  (KS-20, partly; roadmap item 1.5 adds the single-instance guard). Its
  check is now a `status` request, so it no longer leaves an
  `IPC read error` or `IPC write error` warning in the daemon console
  (KS-39, an unverified report).
- KS-04: a `keys` binding fires, or arms its `Tap`/`Hold`/`DoubleTap`, only
  on the key-down of one of its own keys: a non-modifier key, or any of its
  keys for a binding made only of modifiers (`"Alt+Shift"`, `"LShift"`).
  Typing while CapsLock is held no longer re-fires the CapsLock binding or
  swallows the typed keys, Shift pressed while CapsLock is held no longer
  fires `Shift+CapsLock`, and pressing a combo's modifier after its other key
  (J, then Ctrl) no longer fires `Ctrl+J`.
- KS-05 (partly; roadmap item 1.2 fixes the rest): an auto-blocker swallows
  a key only when Windows would see the hotkey's keys and no other key
  held. Shift pressed while Alt and Tab are both held, Shift pressed while
  Ctrl and T are held, and Alt+Shift pressed while a letter is still held in
  fast typing now reach Windows. A key Windows does not have down (one a
  binding swallowed, or one whose release went to the lock screen) does not
  count. Pressed in the order Alt, Shift, Tab, the hotkey's Shift is still
  swallowed and the application gets Alt+Tab (Ctrl+T for Ctrl, Shift, T
  with the `Ctrl+Shift` hotkey) until item 1.2. That includes the usual way
  back in the Alt+Tab switcher, Alt held, Tab released, then Shift+Tab,
  which still moves forward.
- KS-06: an auto-repeat is suppressed exactly when its key's first key-down
  was, and a key-down that reaches Windows clears any suppression left for
  its key-up. Holding a key past the repeat delay no longer lets its
  repeats through and then swallows its release, which left the key held
  in Windows: CapsLock with the README `Tap`/`Hold` pair, a `chord` key, a
  `keys: "LShift"` `DoubleTap` key, the key of a combo whose modifiers are
  released first, or Shift after the `Alt+Shift` auto-blocker once Alt is
  released. A key whose key-down reached Windows (CapsLock pressed with
  Ctrl, which no longer matches `"CapsLock"`) keeps its repeats and its
  release when the binding starts matching after Ctrl is released; such a
  repeat still fires an `Immediate` action (KS-10, roadmap item 1.1).
  A foreground change while a key is held forgets only that its release is
  due to be swallowed (roadmap item 2.3): the release reaches Windows unless
  a `Tap` or `Hold` of the press is still pending, and its repeats stay
  swallowed for every binding kind. They used to be swallowed only while an
  `Immediate` binding matched them, so a key held while the window its
  binding opened took the foreground (a `chord` key, CapsLock with a
  `Hold`, the key of a combo whose modifiers were released) typed into
  that window. A key whose key-down reached Windows keeps its repeats and
  its release across the change. So does a modifier pressed while
  remapping was off: once one of its auto-repeats reaches Windows, the rest
  of the press does too, even when a binding starts matching it.
- KS-07: sided names in `keys` and `sequence` steps fire, and `Shift`,
  `Ctrl` and `Alt` complete a `chord`. The `keys: "LShift"` +
  `DoubleTap(250)` example works; until roadmap item 1.3 it swallows every
  left-Shift press.
- KS-11: a timer left over from an earlier press no longer acts on a newer
  press of the same key. Two quick taps fire twice, a Hold no longer fires
  early on the next press, and a quick series of DoubleTaps no longer loses
  fires.
- KS-25 (equal terms): with `Tap(n)` and `Hold(n)` on one key, exactly one
  fires: `Tap` when the key is released before `n` ms, `Hold` when it is
  held for `n` ms or more. A Hold whose key is released after its term but
  before its timer thread wakes now fires when the timer wakes, still on
  that thread, instead of being cancelled. Different terms behave as before
  until roadmap item 2.2.
- KS-14: the resync of modifiers from the Windows key state leaves alone a
  modifier whose key-down was swallowed, also after a foreground change.
  Windows never saw that key go down, so the next event of any other key
  used to read it as released. It now stays held for matching and its
  auto-repeats stay auto-repeats: with the `Alt+Shift` auto-blocker,
  `Alt+Shift+X` bindings fire in either key order; Shift held by a
  `DoubleTap` binding still counts for `Shift+CapsLock`; one long Shift
  press no longer fires its `DoubleTap(600)`; and `"Ctrl+Shift"` with
  `Hold(1000)`, Ctrl released first, no longer leaves Shift held in Windows.
  Shift pressed after Alt with the default config's auto-blocker now stays
  swallowed for its whole press, auto-repeats and release included, even
  after Alt is released; its repeats used to reach Windows from the first
  one on (roadmap item 1.2 stops the blocker swallowing it).
- KS-18: `block_original_combo` no longer adds an auto-blocker when a
  `keys` binding anywhere in the file uses the same keys as the Windows
  hotkey, in any order and with any trigger or `when:`, the flagged binding
  itself included. That binding now runs its action wherever it sits in the
  file; a binding on `Alt+Shift` below the flagged one, or
  `( keys: "Alt+Shift", action: SwitchLanguageBackward, block_original_combo: true )`
  on its own, never did. The console says so at load
  (`[INFO] no auto-blocker for Alt+Shift: binding #1 uses the same keys`);
  with a `when:` on that binding the Windows hotkey is not blocked where the
  condition does not hold. Auto-blockers are listed after the file's
  bindings at startup. An auto-blocker also no longer runs in place of a
  binding of yours that matches the same press: with `LAlt+LShift` bound,
  that binding fires, and the `Alt+Shift` blocker still swallows Shift
  pressed with the right Alt.
- KS-30 (an unverified report): a completed `chord` no longer marks keys
  already released for a swallowed key-up, and no longer swallows the
  key-up of a chord key whose key-down reached Windows (J pressed again
  while the chord's `when:` did not hold, before K completed it); that key
  was left held in Windows.
- KS-32 (an unverified report): a completed `sequence` no longer swallows
  the key-up of a key whose key-down reached Windows: a modifier held in
  its last step (for example `Win` in `"Win+F14"`), or F14 pressed first
  for `["F14+F15", "F14+F16"]`, which was left held in Windows.
- KS-33 (partly; an unverified report): Tap and DoubleTap no longer start a
  thread per press, and a Hold timer thread that cannot be started is
  logged instead of aborting the daemon; that Hold then does not fire.
- KS-38: `cargo test --release` and `cargo bench` no longer build a libtest
  harness from `src/main.rs` (`test = false` and `bench = false` on the
  daemon binary). That harness inherited the daemon's `requireAdministrator`
  manifest and failed with "The requested operation requires elevation"
  (os error 740) in a non-elevated shell; `cargo bench` uses the bench
  profile, which inherits release, so it failed on every run. An explicit
  target selection still builds the harness: `cargo test --release` or
  `cargo bench` together with `--all-targets`, `--bins` or
  `--bin key-switch-rs` hits the same error, so run them without those flags.
- KS-42: `build.rs` passes the MSVC linker switches `/MANIFEST:EMBED` and
  `/MANIFESTUAC` only when the target environment is `msvc`. A release build
  for `x86_64-pc-windows-gnu` now links, and warns that no UAC manifest is
  embedded.

## [0.1.0] - 2026-05-20

Baseline: the code at commit
[`7cb3521`](https://github.com/bluesteelll/key-switch-rs/commit/7cb3521),
before this changelog existed. It was never tagged or released. The
[audit](https://bluesteelll.github.io/key-switch-rs/audit.html) lists its
known issues, KS-01 to KS-42.

[Unreleased]: https://github.com/bluesteelll/key-switch-rs/compare/7cb3521...HEAD
[0.1.0]: https://github.com/bluesteelll/key-switch-rs/tree/7cb3521
