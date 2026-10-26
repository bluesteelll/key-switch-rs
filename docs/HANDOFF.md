# Hand-off: moving to a new machine (2026-10-25)

The owner stopped all work on 2026-10-25 to move to another machine. On 2026-10-26 the working project moved to the
external drive **E:** (KINGSTON, exFAT), and the owner works from that drive on the new machine. There the drive is
**D:** (NTFS), and the project folder is `<root>` of [CLAUDE.md](../CLAUDE.md) "Paths".
This page is the state of the programme and the steps to resume it. CLAUDE.md stays authoritative for the rules;
[WORKFLOW.md](WORKFLOW.md) for the lane pipeline.

## State

- **master** at the merges of `fix/clippy-rust-1-99` and `chore/work-from-d` (this page). Before them: `9c3b5f4`, the
  move to drive E:, and `1c1f3a0`, the book sync after roadmap 2.5. Roadmap Workstreams A and B and Phases 0, 1 and 2
  (items 2.1 to 2.5) are merged, and the book is synced.
- **The gate** is green on the new machine (rustc 1.99.1) on `fix/clippy-rust-1-99`: 862 library tests passed, 7
  ignored manual and bench tests, 7 `ipc_roundtrip` tests.
- **CI was red on master from `bf549c0` (2026-10-19) to `9c3b5f4`.** Rust 1.99 deprecates `AtomicU32::fetch_update`,
  and clippy's `-D warnings` failed on its one call in the test code of `src/hook/watchdog/mod.rs`; the old machine ran
  1.98.1, so the local gate stayed green. `da77aff` allows the deprecation on that statement (the MSRV 1.88 has no
  `try_update`). The real-clock test `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline` still
  flakes in full runs (2 of 3 on the new machine; alone it passed 6 of 6), so CI can still fail at `cargo test` until
  lane p3-3.4-groundwork fixes it.
- **Phase 3 is scoped, not started.** No Phase 3 branch exists. The plan has **not** been through Plan Mode yet, so
  the owner has not approved the lane list.
- **Stopped mid-run:** the uiAccess workflow (evidence plus plan revision) was stopped before it wrote anything.
  `D:/tmp/ks-orch/phase-3/uiaccess/` was empty.
- **The owner's daily build** (`key-switch-rs-252daef/`) worked without problems in daily use (CapsLock = layout,
  Shift+CapsLock = CapsLock). It is in `<root>/tmp/ks-release/` with the build before 2.5.
- **No manual check has been run**: roadmap "Manual checks after Phase 1" 1-18 and "Manual checks after Phase 2" 1-59
  (49-59 are lane 2.5's M1-M11). The owner will run them later as a batch; do not ask again during Phase 3 unless a lane
  needs one decided.

## The archive: `docs/ks-orch/`

`docs/ks-orch/<lane>/` mirrors `D:/tmp/ks-orch/<lane>/` of the old machine: every lane report (`spec.md`, `cut.md`,
`critique*.md`, `rulings.md`, `impl.md`, `pack_rN.md`, `test_rN.md`, `review_rN.md`, `triage_rN.md`, `fix_rN.md`,
`results*.md`, `merge.md`), the phase book syncs (`phase-N/book.md`, `book-2.5/`), bench summaries (`*/bench/*.md`) and
the small top-level `.txt` notes. `C:\Users\<name>` paths were rewritten to `%USERPROFILE%`.

Not archived (scratch, regenerable or large): cargo target directories, the testers' git worktrees, mdbook build
output, cfgcheck crates, `*.log` gate and mutation logs, `.rs` snapshots, and the post-A1 copies of the agent files.
Reports cite some logs by path; those citations do not resolve in the archive.

The repository's `docs/ks-orch/` is the source. `<orch>` is the working copy that the workflows, agent files and
CLAUDE.md expect, and new lanes write there; on this machine it is restored. To restore it again from the repository,
from `<repo>` (the paths are `<orch>` written out, as CLAUDE.md "Paths" lists):

```powershell
New-Item -ItemType Directory -Force D:/projects/key-switch-rs/tmp/ks-orch | Out-Null
Copy-Item -Recurse -Force docs/ks-orch/* D:/projects/key-switch-rs/tmp/ks-orch/
```

The archived reports cite the earlier machines' paths: read `D:/tmp/ks-orch/...` and `E:/tmp/ks-orch/...` as
`<orch>/...`, and `D:/key-switch-rs/key-switch-rs` and `E:/key-switch-rs/key-switch-rs` as `<repo>`. The Phase 3 lane
specs, `phase-3/plan_draft.md` and `phase-3/scope/` already say `<orch>` and `<repo>`.

## Working from the project folder

The layout on the drive (CLAUDE.md "Paths" names the first three and gives their paths):

| Path | What |
|---|---|
| `<root>/` | The project folder on the owner's KINGSTON drive, D: (NTFS) on this machine |
| `<root>/repo/` (`<repo>`) | The working checkout, with `.git`, on `master`, the same commit as `origin/master` |
| `<root>/tmp/ks-orch/` (`<orch>`) | Lane reports (restored from `docs/ks-orch/`); new lanes write here |
| `<root>/tmp/ks-release/` | The owner's release builds (`key-switch-rs-252daef/`, `key-switch-rs-3cde1d0/`) |
| `<root>/tmp/key-switch-rs/` | The owner's Russian translations: `roadmap.ru.md`, `config-reference.ru.md` |
| `<root>/claude/projects/E--key-switch-rs-key-switch-rs/` | The old machine's Claude Code project folder: its auto-memory and session transcripts, kept as an archive |
| `<root>/claude/skills/rust-analyzer-local/` | The old machine's installed copy of the LSP plugin, with that machine's paths; install from `docs/ks-orch/env/rust-analyzer-local/` instead |
| `<root>/README-move.txt` | The same steps, short, in Russian |

Live paths in the repository are written with the names of CLAUDE.md "Paths". When the drive letter or the folder
changes, edit the literal copies that section lists; `git grep -n -F "<the old folder>"` must then list nothing. The
earlier machines' paths are history: `git grep -n -i -E "\b[DE]:[/\\](tmp|wt|claude|key-switch-rs)"` lists this page's
history lines, the mapping sentence of CLAUDE.md, one scope note, the archived reports, `book/src/methodology.md`
(which records how the past runs were made) and an example value in `book/src/config/actions.md`.

Steps on a new machine:

1. Find the project folder on the drive. If it is not `<root>`, update the paths first (above). The owner gives the
   drive its letter; agents never change system settings.
2. Git: on NTFS there is nothing to do. On exFAT, which records no file owner, Git refuses the checkout as "dubious
   ownership" until it is trusted, once per machine: `git config --global --add safe.directory <the checkout>`.
3. Rust with the MSVC host (`stable-x86_64-pc-windows-msvc`; this machine has rustc 1.99.1, the old one 1.98.1;
   `rust-version` is 1.88), the Visual Studio Build Tools, `mdbook` 0.5.3 (CI's version) and `mdbook-mermaid` (0.17.0 on
   the old machine), `git`, `gh`, ripgrep. Sign in to `gh`/Git for the push. On 2026-10-26 `gh`, `mdbook`,
   `mdbook-mermaid` and ripgrep were not installed on this machine yet (`rg` worked only inside Claude Code's Bash tool,
   which brings its own).
4. Claude Code: open sessions in `<repo>`, not in `<root>`. Only there are CLAUDE.md, `.claude/settings.json` (the deny
   rules and the clarify hook), the nine agents and the `ks-lane`/`ks-phase` workflows loaded, and the auto-memory
   belongs to that folder's key (`D--projects-key-switch-rs-repo`). The old machine's memory was moved there on
   2026-10-26 with its paths updated. LSP: the official `rust-analyzer-lsp` plugin answers on this machine
   ([WORKFLOW.md "Code navigation"](WORKFLOW.md#code-navigation)); the template `docs/ks-orch/env/rust-analyzer-local/`
   is one user-level file for every Rust project on the machine, so it is installed only with all their paths (its
   README, rule 1).
5. Check the gate on master: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
   `cargo test --locked`. The real-clock test `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline`
   flakes (State above); lane p3-3.4-groundwork fixes it.
6. The owner's daemon: `cargo build --release --locked`, start it with `swch open` (today it asks for UAC, the
   `requireAdministrator` manifest). Its config and `daemon.log` live as the book's "Building and running" page says.

Build notes: builds on a USB drive are slower than on an internal SSD. If that hurts, point `CARGO_TARGET_DIR` at an
internal disk; the lanes' `target-wt/` directories under `<orch>` grow by about 1 GB per tester round. While the
official LSP plugin checks the tree, a cargo command in `<repo>` can wait on `Blocking waiting for file lock on build
directory`. Eject the drive only when no cargo, git or Claude Code process writes to it.

## Orchestrator memory from the old machine

The auto-memory was moved to the key of `<repo>` (step 4 above). What it holds, in case it is lost:

- The owner writes in **Russian**; answer in Russian. Every repository artifact is in English.
- Git flow: one branch per feature from `master`, `git merge --no-ff` into `master` (no pull requests), then push
  `master`. Only a lane's merge step (or a chore the owner asked for) pushes.
- Owner decisions: CLAUDE.md "Owner decisions", including the Phase 3 answers of 2026-10-25.
- History: the owner paused the programme on 2026-10-20 after lane 2.5 to use a release build, resumed on 2026-10-25
  ("Продолжай"), and stopped again the same day for this machine move. On 2026-10-26 the working project moved to the
  external drive E:, which is D: on the new machine.

## Phase 3: what exists

- `docs/ks-orch/phase-3/plan_draft.md`: completeness map of every open point, overlaps, the lane order, the bench row
  allocation (R19-R26), the owner questions with recommendations, Checkpoint 3 and 29 manual checks.
- `docs/ks-orch/phase-3/scope/{3.1-input,3.2-seq-chord,3.3-launch,3.4-core,3.5-budget}.md`: one scope report per
  roadmap item, re-located on `252daef`.
- Eight lane specs, in order:

| # | Lane | Branch | Size |
|---|---|---|---|
| 1 | `p3-3.5-budget-bench` | `perf/budget-bench` | S |
| 2 | `p3-3.4-groundwork` | `refactor/core-groundwork` | M |
| 3 | `p3-3.2-seq-chord-matching` | `fix/sequence-chord-matching` | M |
| 4 | `p3-3.1-input-module` | `fix/input-module` | M |
| 5 | `p3-3.1-held-modifiers` | `fix/held-modifiers` | L |
| 6 | `p3-3.3-launch-de-elevate` | `fix/launch-de-elevate` | M, risk L |
| 7 | `p3-3.4-compiled-bindings` | `feat/compiled-bindings` | L |
| 8 | `p3-3.4-zero-alloc` | `perf/zero-alloc-engine` | L |

  Each spec is `docs/ks-orch/<lane>/spec.md`. Specs written before the owner's answers name them as "Q1".."Q9" of
  `plan_draft.md` section 5 and say "relayed in the brief"; the answers below settle them.
- **Owner answers (2026-10-25)** to `plan_draft.md` section 5, also in CLAUDE.md:
  - Q1 (b): `PressKey` releases only its trigger's own modifiers; macro steps release all.
  - Q2 yes: `PressKey("Ctrl+S")`.
  - Q3: known issue 66 fixed for `Tap` and `Hold`; `DoubleTap` documented.
  - Q4: **move to uiAccess** (against the recommendation to keep `requireAdministrator`).
  - Q5: `elevated: true` shows a UAC prompt. Q6: `elevated:` on `Launch` only. Q7: default `cwd` is the profile folder.
  - Q8 (a): lock the winner of a key-down for its repeats.
  - Q9 **yes**: AltGr matching changes in Phase 3 (Breaking; against the recommendation). The held-modifiers spec item 6
    and its acceptance line "with Q9 yes" now apply.
  - Q10: no v0.2.0; v0.3.0 after Checkpoint 3.
  - Run mode: all lanes back to back, merged and pushed on ACCEPT.
- **Known gaps of the plan:**
  - The uiAccess decision is not folded in. It likely changes lane 6 (KS-09 may be solved by the deployment), adds a
    deployment lane (manifest, build, signing, install), and pulls roadmap item 4.4 (config location) forward, because
    a standard user cannot write `config.ron` next to an executable under Program Files. Owner-only steps follow:
    creating or importing a certificate, installing under Program Files.
  - The item 3.3 research was never written: on the first try a safety classifier stopped the researcher's report,
    which covered token and parent-process launch techniques. Keep the brief on documented deployment facts
    (Microsoft Learn's uiAccess requirements, `ShellExecuteEx` and `runas`, signing routes).
  - Three Minor notes from the 2.5 book sync's re-check stay open for the Phase 3 book sync:
    `docs/ks-orch/book-2.5/recheck.md` (N1 glossary.md:67, N2 glossary.md:497, N3 roadmap.md:691 check 57).

## Next steps, in order

1. Set up the machine ("Working from the project folder" above). Confirm `git log -1` on master, the gate and CI.
2. Re-run the uiAccess evidence and plan revision: `docs/ks-orch/phase-3/uiaccess-scope.workflow.js`, the repository's
   copy (a Workflow script, LF; it already carries every owner answer). It writes
   `<orch>/phase-3/uiaccess/{research,impact,plan_revision}.md` and rewrites the affected specs. If the researcher is
   stopped again, narrow its brief to question 1, 2 and 4.
3. Enter **Plan Mode** with the Phase 3 plan: `plan_draft.md` plus `plan_revision.md`, the final lane list, the owner's
   manual steps for the deployment, and any new owner question (for example the signing route: self-signed on the
   owner's machine, a commercial certificate, Microsoft's cloud signing, SignPath for open source). Get approval.
4. The orchestrator's chore before the lanes, if the plan changes rules: CLAUDE.md layout and bench rows.
5. Run `ks-phase` with the approved lanes in order (each `spec` argument points at its `<orch>/<lane>/spec.md`),
   `bookSync: true`. Then the Phase 3 book sync and Checkpoint 3 for the owner.
