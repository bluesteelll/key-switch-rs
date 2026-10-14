# Introduction

This book documents key-switch-rs as it stands at commit `7cb3521`: how it is built, how it is configured, what is broken and what to fix first.

key-switch-rs is a Windows-only keyboard remapper written in Rust. An elevated daemon, `key-switch-rs.exe`, intercepts key events with a low-level `WH_KEYBOARD_LL` hook, matches them against bindings from a hot-reloaded `config.ron` and runs actions such as switching the input language, pressing keys or launching programs. A small CLI, `swch.exe`, controls the daemon over a named pipe.

## Who this book is for

The book is written for the project owner and for future contributors who need to know how the code fits together, what the configuration format accepts and which defects are open. Readers who only want to write a `config.ron` can start at the [configuration reference](config/index.md).

## Current state

The code builds cleanly: `cargo build --release` gives no warnings, all 120 unit tests pass, and clippy reports one style lint. The audit nevertheless verified 26 issues, and six high-severity defects block daily use: two break the default configuration during normal typing, two break the `swch` control path, one disables every `AppEquals` condition, and one leaves keys stuck down with the README's Tap/Hold example. They are [KS-01 to KS-06](audit.md#summary) in [Audit results](audit.md).

> [!WARNING]
> Do not rely on the daemon for daily use until KS-01 to KS-06 are fixed. Workarounds for the defects you are most likely to hit are in [Troubleshooting](build-and-run.md#troubleshooting).

## How this book is organised

- [Overview and current state](overview.md): what the program does, its features and default bindings, and its overall health.
- [Architecture](architecture.md): processes, threads, hooks, shared state and the key-down path, and where the structure causes defects.
- [Building, running and controlling the daemon](build-and-run.md): build, elevation, `swch` commands, autostart, tests and workarounds.
- [Configuration reference](config/index.md): entry point to the `config.ron` format.
  - [File location, reload and top-level structure](config/structure.md): where the file lives, how it is created on first run and hot-reloaded, the top-level layout of the file and the fields of a binding.
  - [RON syntax essentials](config/ron-syntax.md): the RON syntax the file accepts, from encoding, comments and brackets to integers, strings, Windows paths and the nesting limit.
  - [Key names and combo syntax](config/keys.md): accepted key names and how combo strings are parsed.
  - [Binding kinds and key suppression](config/kinds.md): `keys`, `sequence` and `chord` bindings, and which keys are suppressed, including `block_default`.
  - [Triggers and conditions](config/triggers-conditions.md): Immediate and Tap/Hold/DoubleTap triggers and `when:` conditions.
  - [Actions and macro steps](config/actions.md): every action and every step of the `Sequence` macro action.
  - [Auto-blockers and matching order](config/matching.md): the auto-blockers that `block_original_combo` adds and the order and priority in which bindings are checked.
  - [Complete example](config/example.md): a full `config.ron` that loads without errors.
  - [Error messages](config/errors.md): what the loader reports and what causes each message.
  - [Known configuration pitfalls](config/known-issues.md): settings that load but misbehave because of verified defects.
- [Audit results](audit.md): the 26 verified issues KS-01 to KS-26, their root causes, and the unverified low-severity reports.
- [Gaps and documentation drift](gaps.md): missing features, README claims the code does not meet, missing infrastructure and untested code.
- [Roadmap](roadmap.md): the proposed order of fixes and features.
- [How the audit was done](methodology.md): the build check, the multi-agent review, the probes and the limits of each.
- [Glossary](glossary.md): Win32, Rust and project terms used in this book.

## Dates and scope

> [!NOTE]
> Facts describe commit [`7cb3521`](https://github.com/bluesteelll/key-switch-rs/commit/7cb3521) (2026-05-20), the tip of `master` on the audit date. Build, test and audit runs were done on 2026-10-14. Code references are permalinks to that commit, so their line numbers can differ on later commits. The daemon was not run against live keyboard input; findings come from code reading, Win32 documentation, probe programs and a scratch test crate, as described in [How the audit was done](methodology.md).

## Reporting problems

Report errors in this book or defects in key-switch-rs on the [GitHub issues page](https://github.com/bluesteelll/key-switch-rs/issues). For a defect already listed here, quote its ID, for example [KS-04](audit.md#ks-04). The edit icon at the top of each page opens its source file on GitHub.

## Building this book locally

1. Install the tools that CI pins: `cargo install mdbook --version 0.5.3 --locked` and `cargo install mdbook-mermaid --version 0.17.0 --locked`.
2. From the repository root, run `mdbook serve book`.
3. Open `http://localhost:3000`. The site rebuilds on every save.

`mdbook-mermaid` must be on `PATH`, or the build stops. [Building this book](build-and-run.md#building-this-book) covers `mdbook build` and the GitHub Pages workflow.
