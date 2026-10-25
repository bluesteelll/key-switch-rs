# Hand-off: moving to a new machine (2026-10-25)

The owner stopped all work on 2026-10-25 to move to another machine. This page is the state of the programme and the
steps to resume it. [CLAUDE.md](../CLAUDE.md) stays authoritative for the rules; [WORKFLOW.md](WORKFLOW.md) for the
lane pipeline.

## State

- **master** at the merge of `chore/handoff-machine-move` (this page). Before it: `1c1f3a0`, the book sync after roadmap
  2.5. Roadmap Workstreams A and B and Phases 0, 1 and 2 (items 2.1 to 2.5) are merged, the book is synced, and the gate
  was green on `1c1f3a0` (862 library tests passed, 7 ignored manual and bench tests, 7 `swch` tests).
- **Phase 3 is scoped, not started.** No Phase 3 branch exists. The plan has **not** been through Plan Mode yet, so
  the owner has not approved the lane list.
- **Stopped mid-run:** the uiAccess workflow (evidence plus plan revision) was stopped before it wrote anything.
  `D:/tmp/ks-orch/phase-3/uiaccess/` was empty.
- **The owner's daily build** (`D:/tmp/ks-release/key-switch-rs-252daef/`, old machine) worked without problems in daily
  use (CapsLock = layout, Shift+CapsLock = CapsLock). It is not archived; rebuild it on the new machine.
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

Restore it where the workflows, agent files and CLAUDE.md expect it:

```powershell
New-Item -ItemType Directory -Force D:/tmp/ks-orch | Out-Null
Copy-Item -Recurse -Force docs/ks-orch/* D:/tmp/ks-orch/
```

If the new machine has no `D:` drive, change the paths instead. They are hard-coded in `.claude/workflows/ks-lane.js`,
`.claude/workflows/ks-phase.js`, the ten files of `.claude/agents/`, `CLAUDE.md`, `docs/WORKFLOW.md` and
`book/src/methodology.md`: `D:/tmp/ks-orch` (reports), `D:/key-switch-rs/key-switch-rs` (the main checkout) and
`D:/wt/_targets/ra-msvc` (rust-analyzer's target directory). `git grep -n -E "D:/tmp/ks-orch|D:/key-switch-rs|D:/wt/"`
lists every site.

## Setting up the new machine

1. Clone `https://github.com/bluesteelll/key-switch-rs` to `D:/key-switch-rs/key-switch-rs` (the path the scripts use).
2. Rust with the MSVC host (`stable-x86_64-pc-windows-msvc`; the old machine had rustc 1.98.1; `rust-version` is 1.88),
   the Visual Studio Build Tools, `mdbook` 0.5.3 (CI's version; the old machine had 0.5.2) and `mdbook-mermaid`
   (0.17.0 there), `git`, `gh`, ripgrep.
3. The LSP for agents: the user-level plugin `rust-analyzer-local`, whose template is in
   [docs/ks-orch/env/rust-analyzer-local/](ks-orch/env/rust-analyzer-local/). Copy it to
   `%USERPROFILE%/.claude/skills/rust-analyzer-local/`, set `command` to the MSVC toolchain's `rust-analyzer.exe`, keep
   `linkedProjects` on this checkout's `Cargo.toml` (add other projects the machine works on), then `/reload-plugins`.
   Its README explains the four traps (strict schema, forward slashes, `linkedProjects` replaces discovery, reload).
4. Check the gate on master: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
   `cargo test --locked`. The real-clock test `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline`
   flakes (it failed once in 2.5's final gate run); lane p3-3.4-groundwork fixes it.
5. The owner's daemon: `cargo build --release --locked`, start it with `swch open` (today it asks for UAC, the
   `requireAdministrator` manifest). Its config and `daemon.log` live as the book's "Building and running" page says.

## Orchestrator memory from the old machine

The old machine's auto-memory does not travel. What it held, still true:

- The owner writes in **Russian**; answer in Russian. Every repository artifact is in English.
- Git flow: one branch per feature from `master`, `git merge --no-ff` into `master` (no pull requests), then push
  `master`. Only a lane's merge step (or a chore the owner asked for) pushes.
- Owner decisions: CLAUDE.md "Owner decisions", including the Phase 3 answers of 2026-10-25.
- History: the owner paused the programme on 2026-10-20 after lane 2.5 to use a release build, resumed on 2026-10-25
  ("Продолжай"), and stopped again the same day for this machine move.

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

1. Set up the machine and restore the archive (above). Confirm `git log -1` on master and the gate.
2. Re-run the uiAccess evidence and plan revision: `docs/ks-orch/phase-3/uiaccess-scope.workflow.js` (a Workflow script;
   it already carries every owner answer). It writes `D:/tmp/ks-orch/phase-3/uiaccess/{research,impact,plan_revision}.md`
   and rewrites the affected specs. If the researcher is stopped again, narrow its brief to question 1, 2 and 4.
3. Enter **Plan Mode** with the Phase 3 plan: `plan_draft.md` plus `plan_revision.md`, the final lane list, the owner's
   manual steps for the deployment, and any new owner question (for example the signing route: self-signed on the
   owner's machine, a commercial certificate, Microsoft's cloud signing, SignPath for open source). Get approval.
4. The orchestrator's chore before the lanes, if the plan changes rules: CLAUDE.md layout and bench rows.
5. Run `ks-phase` with the approved lanes in order (each `spec` argument points at its `D:/tmp/ks-orch/<lane>/spec.md`),
   `bookSync: true`. Then the Phase 3 book sync and Checkpoint 3 for the owner.
