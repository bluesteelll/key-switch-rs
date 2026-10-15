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

### Fixed

- KS-04: a `keys` binding fires, or arms its `Tap`/`Hold`/`DoubleTap`, only
  on the key-down of one of its own keys: a non-modifier key, or any of its
  keys for a binding made only of modifiers (`"Alt+Shift"`, `"LShift"`).
  Typing while CapsLock is held no longer re-fires the CapsLock binding or
  swallows the typed keys, Shift pressed while CapsLock is held no longer
  fires `Shift+CapsLock`, and pressing a combo's modifier after its other key
  (J, then Ctrl) no longer fires `Ctrl+J`.
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
  A foreground change while a key is held still forgets that its key-down
  was swallowed (roadmap item 2.3): its repeats are then swallowed only
  while an `Immediate` binding matches them, as before, and its release
  reaches Windows; a key whose key-down reached Windows keeps its repeats
  and its release across the change. A modifier whose `Tap` or `Hold` is
  still pending can still be left held when the combo stops matching
  mid-hold (`"Ctrl+Shift"` with `Hold(1000)`, Ctrl released first; KS-14).
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
- KS-32 (partly; an unverified report): when a `sequence` completes, the
  key-up of a modifier held in its last step (for example `Win` in
  `"Win+F14"`) is no longer swallowed.
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
