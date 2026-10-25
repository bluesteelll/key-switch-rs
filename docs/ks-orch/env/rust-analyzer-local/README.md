# rust-analyzer-local

Declares one LSP server (`rust-analyzer`) for `.rs`. It exists because the official
`rust-analyzer-lsp@claude-plugins-official` plugin is dark: it declares its server only on its
**marketplace entry**, and the LSP config loader never reads that source. The loader takes
`plugin.lspServers`, then `<plugin root>/.lsp.json`, then `plugin.json`'s `lspServers` — and the
cached plugin.json is a stub whose mere existence shadows the marketplace entry, silently. Hence a
local plugin, whose `.lsp.json` sits exactly where the loader looks.

`plugin init` also scaffolded a `SKILL.md`; it was deleted, and `plugin.json` carries no `skills`
key, so this plugin contributes an LSP server and nothing else (`~0 tok` per session).

## Working rule: linkedProjects follows the active work

`settings["rust-analyzer"].linkedProjects` must list **the main checkout plus the lanes currently
being worked on** — never every worktree that happens to exist.

- **Starting work on a lane** → add that worktree's `Cargo.toml`.
- **Finishing work on a lane** → remove it again.

This is a memory budget, not tidiness. One project costs **3.6–5.5 GB** RSS (a high-water mark that
grows with query volume through the LRU query cache) on a **15.4 GB** machine that was measured at
0.3–1.0 GB available with a 23 GB commit charge. All six worktrees would be ~21 GB — not an option.
The other side is just as real: without its project linked, a file in a worktree gets **syntax only**
— `documentSymbol` works, while `hover`, `findReferences`, `goToDefinition` and `workspaceSymbol`
all come back empty.

`cargo.allTargets` stays `true` by owner decision (2026-09-07): analysis of `tests/` and `benches/`
is worth more than the memory it costs, because that is where the campaign's gates live.

## Editing this file — four things that bite

1. **Never drop `D:/claude/BoykoEngine/Cargo.toml`.** `linkedProjects` *replaces* project
   auto-discovery rather than extending it, so an incomplete list breaks what already worked.
2. **Forward slashes, absolute paths.** A single backslash before `U`/`.`/space is an invalid JSON
   escape and the parser throws. Write this file with an editor, not a shell heredoc.
3. **`/reload-plugins` after every edit.** The server map is built once at initialization; the LSP
   manager does not reinitialize when this file changes.
4. **The server-config schema is strict and all-or-nothing.** One unrecognized key at the server
   level rejects the whole file and the plugin then contributes zero servers — announced only as an
   error line into `~/.claude/debug`, which is empty because the app runs the CLI without `--debug`.
   Only these thirteen keys are accepted per server: `command`, `args`, `extensionToLanguage`,
   `transport`, `env`, `initializationOptions`, `settings`, `workspaceFolder`, `startupTimeout`,
   `shutdownTimeout`, `restartOnCrash`, `maxRestarts`, `diagnostics`. Anything rust-analyzer-specific
   belongs *inside* `settings["rust-analyzer"]`, which is free-form and unchecked.

## Host toolchain: MSVC since 2026-09-17

The machine's build host moved from windows-gnu to MSVC on 2026-09-17, so the server is pinned to
MSVC in three places, none of which depends on `rustup default`:

- **`command`** names the MSVC toolchain's own binary, not the `~/.cargo/bin` proxy. The server then
  takes its proc-macro server from the same toolchain's `libexec`. A gnu server cannot load
  msvc-built proc-macro DLLs, and the reverse also fails.
- **`settings["rust-analyzer"].cargo.extraEnv`** sets `RUSTUP_TOOLCHAIN` for every `cargo` and
  `rustc` the server runs, including build scripts and sysroot discovery. The tree's
  `rust-toolchain.toml` says only `channel = "stable"`, and that resolves through the default host.
- **`cargo.targetDir`** is `D:/wt/_targets/ra-msvc`. The server's build-script and proc-macro build
  stays out of each tree's `target/` and off C:. Both linked projects share this directory; cargo's
  lock serialises their builds.

There is no top-level `env` key. The loader accepts one, but it is not established whether it
extends or replaces the inherited environment, and a replaced `PATH` or `USERPROFILE` would hide
`cargo` from the server.

The first start after a toolchain change rebuilds every build script and proc macro of both
projects in check mode, cold. Queries answer `has not finished indexing` until that is done.

## Verifying a change

Measure the capability, not a proxy. Run `hover` and `findReferences` on a file in the newly linked
lane **and** the same pair on a main-checkout file as a control — the control is what catches the
replace trap in rule 1. A flat CPU curve is not a readiness signal: at the plateau the server held
1.3 GB and had merely gone idle waiting for requests; real queries then took it to 5.5 GB. For the
first minutes after a reload, `has not finished indexing` is normal rather than a failure.

Ask for a symbol that exists **in the linked tree**: `ScopeBlock` lives only on
`feat/threadpool-ke16`, and its absence from the main checkout was once mistaken for a cold index.
Expect `workspaceSymbol` to return one hit per tree for the 28 identically-named packages; tell them
apart by path form — a lane answers with an absolute `d:/wt/...`, the main checkout with a relative
`crates/...`.

If memory gets tight: `Stop-Process -Name rust-analyzer`. The manager restarts it on the next LSP
request (`restartOnCrash`, `maxRestarts` 3).
