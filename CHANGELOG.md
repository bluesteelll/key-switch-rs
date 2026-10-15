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

### Fixed

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
