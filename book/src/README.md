# Introduction

This book documents key-switch-rs: how it is built, how it is configured, what is broken and what to fix next. It started as the audit of commit `7cb3521` and now describes `master` at commit `f4c1fb6`, after roadmap Phases 0, 1 and 2, while keeping the audit's findings as the baseline.

key-switch-rs is a Windows-only keyboard remapper written in Rust. An elevated daemon, `key-switch-rs.exe`, intercepts key events with a low-level `WH_KEYBOARD_LL` hook, lets a pure decision core match them against bindings from a hot-reloaded `config.ron`, and runs actions such as switching the input language, pressing keys or launching programs. A small CLI, `swch.exe`, controls the daemon over a named pipe.

## Who this book is for

The book is written for the project owner and for future contributors who need to know how the code fits together, what the configuration format accepts and which defects are open. Readers who only want to write a `config.ron` can start at the [configuration reference](config/index.md).

## Current state

The audit of 2026-10-14 verified 26 issues at `7cb3521`, six of them high severity: two broke the default configuration during normal typing, two broke the `swch` control path, one disabled every `AppEquals` condition, and one left keys stuck down. Roadmap Phase 0 then added CI, a pure decision core with scenario tests, and fixes for 13 of the 26 issues, all of the high-severity ones except the rest of [KS-05](audit.md#ks-05). Phase 1 added a per-binding `repeat` flag, so that a held key no longer repeats one-shot actions, the menu mask key, one winning binding per key press, a single daemon per session and a log file, and fixed 5 more issues, 18 of the 26 in all. Phase 2 made the hook callback only decide: slow actions run on an action executor, the engine and its gesture timers live on the main thread, the daemon resets its state when keyboard input leaves the desktop or the config reloads, and a watchdog has the hook reinstalled if Windows removes it; it fixed 2 more issues, 20 of the 26 in all. Its last item, 2.5, took the reads of the modifier keys off the key path: a key typed with no modifier held asks Windows nothing, and the main thread reads the modifiers between key events. Its [Checkpoint 1](roadmap.md#checkpoint-1-smoke-checklist), which decides whether the default configuration is fit for daily use, and [Checkpoint 2](roadmap.md#checkpoint-2-v020) still need a manual run. At `f4c1fb6` the quality gate passes: `cargo fmt --check`, clippy with `-D warnings`, and 862 passing library tests with 7 ignored, 6 manual tests and the latency bench, plus 7 integration tests ([Overview and current state](overview.md#build-test-and-lint-status)). [Audit results](audit.md#summary) gives the status of every issue.

> [!WARNING]
> The fixes of Phases 0, 1 and 2 are pinned by tests, but nobody has yet run the daemon against live keyboard input to confirm them; [Manual checks after Phase 0](roadmap.md#manual-checks-after-phase-0), [after Phase 1](roadmap.md#manual-checks-after-phase-1) and [after Phase 2](roadmap.md#manual-checks-after-phase-2) list what to try. The phases changed how bindings match and how gestures resolve, which can break an existing `config.ron`: see [Known configuration pitfalls](config/known-issues.md#s19-9). Workarounds for the problems you are most likely to hit are in [Troubleshooting](build-and-run.md#troubleshooting).

## How this book is organised

- [Overview and current state](overview.md): what the program does, its features and default bindings, what changed since the audit, and the audit's snapshot.
- [Architecture](architecture.md): processes, threads, hooks, the decision core, shared state and the key-event path, and which defects the structure explains.
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
  - [Known configuration pitfalls](config/known-issues.md): settings that load but misbehave, with the status of each.
- [Audit results](audit.md): the 26 verified issues KS-01 to KS-26, their root causes, the unverified low-severity reports, and the status of each.
- [Gaps and documentation drift](gaps.md): missing features, README claims the code does not meet, missing infrastructure and untested code.
- [Roadmap](roadmap.md): the order of fixes and features, with the status of each item.
- [How the audit was done](methodology.md): the build check, the multi-agent review, the probes and the limits of each.
- [Glossary](glossary.md): Win32, Rust and project terms used in this book.

## Dates and scope

> [!NOTE]
> The pages that describe the program as it is (the overview, architecture, building and running, the configuration reference and the glossary) describe commit [`f4c1fb6`](https://github.com/bluesteelll/key-switch-rs/commit/f4c1fb6) (2026-10-20), the merge of roadmap item 2.5, and their code references are permalinks to it. The findings pages (the audit, gaps and drift, the roadmap and the methodology) describe the audit commit [`7cb3521`](https://github.com/bluesteelll/key-switch-rs/commit/7cb3521) (2026-05-20), the tip of `master` on the audit date of 2026-10-14, and carry status notes for `bf549c0` or, where item 2.5 changed something, for `f4c1fb6`; status notes written after Phase 0 or Phase 1 keep the commit `1dfd80a` or `20af28d` they describe. Each page says which commit it describes. The daemon was not run against live keyboard input; findings come from code reading, Win32 documentation, probe programs, a scratch test crate and the project's own tests, as described in [How the audit was done](methodology.md).

## Reporting problems

Report errors in this book or defects in key-switch-rs on the [GitHub issues page](https://github.com/bluesteelll/key-switch-rs/issues). For a defect already listed here, quote its ID, for example [KS-04](audit.md#ks-04). The edit icon at the top of each page opens its source file on GitHub.

## Building this book locally

1. Install the tools that CI pins: `cargo install mdbook --version 0.5.3 --locked` and `cargo install mdbook-mermaid --version 0.17.0 --locked`.
2. From the repository root, run `mdbook serve book`.
3. Open `http://localhost:3000`. The site rebuilds on every save.

`mdbook-mermaid` must be on `PATH`, or the build stops. [Building this book](build-and-run.md#building-this-book) covers `mdbook build` and the GitHub Pages workflow.
