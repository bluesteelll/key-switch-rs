# Architecture critique: p0-B-config / Workstream B (KS-21, KS-22, KS-23, KS-24, KS-27, docs drift) (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 2)
Plan: D:/tmp/ks-orch/p0-B-config/cut.md   Code read at: master 4177540

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. `config::load` changes signature, but the project's RON validation harness calls it with a path
Where: cut D1 (`pub fn load(config: &ConfigPath)`, `default_config_path` leaves the `pub use`), C2, touch set ("not touched": `book/`), Docs impact (book-sync list); `book/src/methodology.md:376-428` (cfgcheck `src/main.rs`, published "verbatim", calls `key_switch_rs::config::load(&path)` with `path: PathBuf` at `:403`); `.claude/agents/doc-writer.md:80-84`; CLAUDE.md "Docs and the book" ("Every published RON example is validated against the real loader first (the `cfgcheck` harness ...)").

Problem: The cut treats `config::load(&Path)` as internal, but it has an external caller that the process depends on: the `cfgcheck` crate that every doc-writer builds from the methodology page. Neither D1's rejected alternatives nor the Docs impact mention it, and the book-sync list names no change to `methodology.md#validation-harness` or `doc-writer.md`.

Consequence: After C2 is merged, the phase-0 book sync (WORKFLOW.md:105-107, on by default, runs after this phase's lanes) follows doc-writer.md:82-84. It creates cfgcheck from the verbatim source, and `cargo run --manifest-path D:/tmp/ks-orch/<lane>/cfgcheck/Cargo.toml -- <file>` fails with E0308 (expected `&ConfigPath`, found `&PathBuf`). This lane changes what loads (KS-22, KS-23, KS-27), so the pages that most need validated RON are `config/errors.md`, `structure.md`, `ron-syntax.md` and `known-issues.md`, all on the cut's own book-sync list. The sync then either stalls or publishes them unvalidated, and "nothing gates the doc-writer" (WORKFLOW.md:177). The published methodology page also goes on documenting a harness that no longer compiles. doc-writer.md:83's reason for the existence check ("`config::load` writes the default config to any missing path") becomes false.

Confidence: CONFIRMED (cut D1 and C2; methodology.md:395-403; doc-writer.md:80-84; `src/config/mod.rs:15`).

Breaks: the CLAUDE.md validation rule (a public-API break nobody declared, hitting the project's own tooling).

What is needed: The cut names cfgcheck's entry point after the lane. It adds methodology.md:376-428 (source, existence check, exit-code table) and doc-writer.md:80-84 to the docs impact: the book-sync list for the page, and an orchestrator item like Q2 for `.claude/`. `.claude/agents/code-reviewer.md:25` ("then `config::load` and `update_bindings`") becomes stale the same way.

Options: (a) cfgcheck calls `config::reload(&Path)`, which takes a path and never writes, so the existence pre-check and exit code 2 can map to `ConfigError::Missing` (recommended, because it needs no shim); (b) keep a path-taking public loader next to `load(&ConfigPath)`.

## Minor (improvements, not blockers)

### M1. The README tree and architecture.md still name `default_config_path` as the public API after C2
Where: `README.md:30` ("mod.rs - public API: load(), default_config_path()"); `book/src/architecture.md:23` ("Only `load` and `default_config_path` are exported"); cut C1 tree edits (stray `:28`, `condition.rs`, `trigger.rs`, `properties.rs`), C2 (re-exports become `ConfigPath`, `load`, `reload`), C3 (`testing.rs` line). The book-sync list for architecture.md (`:93-94`, `:165-166`, `:392-439`, `:529`) leaves out `:23`.

Consequence: After the merge, a contributor (or a cfgcheck author, see I1) who reads the README module tree looks for `config::default_config_path`, which is gone. The tree does not mention `ConfigPath` or `reload`. This lane exists to remove this exact kind of drift, and here it adds one.

Confidence: CONFIRMED.

What is needed: C2 updates README:30 to the new exports, and architecture.md:23 joins the book-sync list.

### M2. C1's replacement for README "Not macros" lists ToggleCapsLock as a native Win32 call
Where: cut C1, README `:601` text ("language switching, CapsLock, window operations and messages use Win32 calls; key output (`PressKey`, `Media`, macro `Text`/`Key`/`Combo`) is synthesized with `SendInput`"); `src/system/system_function.rs:117-151` (`toggle_caps_lock` -> `press_key(VK_CAPITAL)` -> `SendInput` with `injected_sentinel()`); `book/src/gaps.md:175` (its Reality column lists `ToggleCapsLock` among the `SendInput` outputs).

Consequence: The README would say that CapsLock is toggled by a native API, but it is a synthesized VK_CAPITAL press. For example, another LL hook (AutoHotkey) sees an injected CapsLock press, and in a non-elevated debug build the toggle does not reach an elevated foreground window (UIPI blocks injected input there). The drift row would be only half fixed.

Confidence: CONFIRMED.

What is needed: Move `ToggleCapsLock` into the `SendInput` list of the new line.

## Strengths to keep

- **Seams first (D7).** I traced every red test against its parent: T1-T4 and T7 on C2, T9-T11, T13 and T14 on C4, T15-T18 on C6, T19 and T20 on C8, T21 and T22 on C10. Each compiles there and fails on its own assertion for the right reason. T14 proves the watcher is live before the delete, so it cannot pass vacuously.
- **D3** maps `NotFound` instead of pre-checking `exists()`. That also covers the missing-folder typo, because std maps ERROR_PATH_NOT_FOUND to `NotFound`. `create_new` closes the TOCTOU when the template is written. Reload never writes.
- **D4 covers struct variants too.** serde_derive 1.0.228 passes the container attributes into struct variants (`de/enum_externally.rs:153-158` -> `de/struct_.rs:285`, `:580`, `:682`), so `Launch(cwd: ..)` and `PostMessage(wparm: ..)` fail as T16 predicts.
- **RON message shapes check out in ron 0.10.1.** `error.rs:116-119` formats `SpannedError` as `{position}: {code}`. `:238-265` gives "Unexpected field named `f` in `S`, expected ..." and "Unexpected missing field named `f` in `S`". The T15, T16 and T17 substrings hold, and T19's `1:51:` matches `ron-syntax.md:52`.
- **The Win32 and notify facts hold.** `CompareStringOrdinal(&[u16], &[u16], bool)` and `CSTR_EQUAL` are at `windows-0.62.2/src/Windows/Win32/Globalization/mod.rs:20-23` and `:7242`. In notify, an event path is the watched folder plus the reported name (`notify-8.2.0/src/windows.rs:395-398`), and a relative watch is joined to the current folder (`:531-535`).
- **D1 keeps the logic testable.** One library call from an untestable `main` (`Cargo.toml:30-34`, `test = false`) means T1 and T2 test it directly.
- **No hook-path change, no ownership change, and no production lock or atomic.**

## Open questions for the architect

1. **Q1 against CLAUDE.md.** CLAUDE.md says "A developer whose change alters config semantics updates the matching `book/src/config/*.md` page ... in the same lane." The cut defers to the phase book sync (WORKFLOW.md:105-107, as in A2-A6). Between this merge and the sync, `config/structure.md:27` (a missing file is generated) and the KS-22 text in `errors.md` describe the old behaviour. Those pages state that they describe `7cb3521` (`known-issues.md:114`), which limits the harm. The orchestrator should confirm the deferral for this phase as an explicit exception to the CLAUDE.md rule. It is not a cut defect.
2. **Q5:** I agree that T7 and T14 run by default. ReadDirectoryChangesW on a folder in `%TEMP%` is not a hook, injects no input and changes no setting.
3. **A cleanup race on red parents only.** notify's `Drop` only sends `Stop` (`notify-8.2.0/src/windows.rs:590-596`). After T7 or T14 panics on C2 or C4, a late debounce could still run `load_or_generate` and recreate the TempDir folder with a template. The worst case is a stray `key-switch-rs-test-*` folder in `%TEMP%` during the tester's replay. From C6 on, reload never writes. No change needed unless the tester reports litter.

## Absent topics (walked, not applicable)

- **1. Hook-thread budget:** No code on the LL or WinEvent callback path changes (touch set; `src/engine/**` and `src/hook/keyboard_hook*.rs` are untouched). The new range check caps a Hold's `hold-timer` sleep at 10 s, which is a side benefit.
- **2. Thread affinity:** No install or uninstall moves. `from_arg` runs on main before `App::run`. `wants_reload` and `CompareStringOrdinal` run on notify's backend thread (no COM and no window), and `reload_into` runs on `config-watcher`.
- **3. Matching, 4. suppression lifetime, 5. gestures:** Untouched (RK10, RK12). A rejected or missing file no longer swaps the list, so fewer swaps reach pending gestures.
- **6. Pure core:** The engine is untouched. Validation stays in the loader, where the binding number is known, and has direct unit tests.
- **7. Reload leniency:** A missing file is no longer rewritten (D3). The path is made absolute once (D1). Unknown fields are denied (D4) and the BOM is stripped (D5). An explicit `bindings: []` still replaces the live list; that is the spec's first option, and D4's symmetry argument holds.
- **8. Shared state:** One test-only `Relaxed` counter with a written pairing argument. `update_bindings` is unchanged.
- **9. Process boundary:** No pipe, registry or child-process change. The elevated daemon now writes only beside its own exe, which is an improvement.
- **10. Input synthesis:** No `SendInput`.
- **11. Fit:** `ConfigPath` is a plain enum and the `App` builder is unchanged. `ConfigError::Missing` gets its own `Display` arm. The CHANGELOG Breaking entries cover KS-22, KS-27 and the command-line path, and each commit's README, template and CHANGELOG changes ride with the code that changes the meaning.
