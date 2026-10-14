# Agents for `key-switch-rs`

Nine specialist roles run the fix programme and feature work. The main Claude session is the
**orchestrator**. It writes briefs, launches the `ks-lane` / `ks-phase` workflows, reads verdicts
and asks the owner. Subagents cannot ask the user, so on an ambiguity they stop and return `BLOCKED`.
The full manual is in [docs/WORKFLOW.md](../../docs/WORKFLOW.md), and the binding rules are in [CLAUDE.md](../../CLAUDE.md).

## The roles

| Agent | Purpose | Model | Effort |
|---|---|---|---|
| [`architect`](architect.md) | Writes THE CUT for a lane: re-located items, touch set, design (threads, Win32 APIs, RON compatibility), commits with red-first tests, acceptance proofs. No product code. | opus | xhigh |
| [`architecture-critic`](architecture-critic.md) | Critiques the cut before code: tests that cannot fail, unproven criteria, hot-path cost, hook/window/timer/COM ownership, sentinel, config breaks. Does not design. | opus | xhigh |
| [`developer`](developer.md) | Implements an approved cut red-first on the lane branch, resolves or refutes every remark, owns fmt/check/clippy, runs fix rounds and the merge step. | opus | xhigh |
| [`code-reviewer`](code-reviewer.md) | Reviews the lane diff against the cut and risk areas R1-R10. In TRIAGE mode it tries to refute every finding before a fix round. Does not fix. | opus | xhigh |
| [`tester`](tester.md) | Independent gate: full fmt/clippy/test, red-first replay on the parent in a scratch worktree, at least 2 mutations, acceptance mapping. No commits. | **sonnet** | high |
| [`results-analyst`](results-analyst.md) | The final gate before merge: ACCEPT, REWORK (and the stage to return to) or RETHINK, from every report plus the actual diff. | opus | xhigh |
| [`project-analyst`](project-analyst.md) | Read-only analysis and Q&A on the codebase. In a lane it is the context-pack scout (`pack_rN.md`, at `high`). | opus | xhigh |
| [`researcher`](researcher.md) | Facts the book lacks (Win32 behaviour, crate APIs, prior art), as a cited report. | opus | high |
| [`doc-writer`](doc-writer.md) | Owns the English mdBook `book/src`, `README.md` and the `default_config.ron` comments. Validates every RON example against the loader. | opus | high |

**Routing** (owner, 2026-10-14, mirroring boyko-engine): every role runs on Opus except the tester,
which runs on Sonnet. Six roles run `xhigh` (architect, architecture-critic, developer, code-reviewer,
results-analyst, project-analyst) and three run `high` (tester, researcher, doc-writer). Nothing runs below `high`.
The routing is set in each file's `model:` / `effort:` frontmatter and mirrored by the `MODEL` / `EFFORT` maps in
`.claude/workflows/ks-lane.js` (and the doc-writer call in `ks-phase.js`), so a workflow never inherits the
session's settings. One per-call override: `project-analyst` as the context-pack scout runs `high`. The
orchestrator stays on the session model. Rationale:
[docs/WORKFLOW.md#model-routing](../../docs/WORKFLOW.md#model-routing).

Every role carries the same **Safety** rules (some roles add role-specific lines). No role runs the daemon, installs
hooks, injects input or writes the registry, and only a merge step pushes.

## The lane

```
architect (cut) -> architecture-critic (critique; one revision if Critical)
  -> developer (implement, red-first)
  -> project-analyst (context pack) -> tester || code-reviewer
  -> code-reviewer TRIAGE -> developer (fix) -> delta re-verify   [at most 3 rounds]
  -> results-analyst -> developer MERGE (--no-ff into master, gate, push)
```

Run one item with `workflow('ks-lane', {name, branch, title, spec, base?, wt?, merge?, stopAfter?, testerModel?, resumeNote?})`.
Run a phase with `workflow('ks-phase', {phase, items: [...], bookSync?, continueOnFailure?})`, which runs the lanes in order and then a
`doc-writer` book sync. Reports go to `D:/tmp/ks-orch/<lane>/`.

Ad hoc use outside a lane works the same way. Pass the agent a brief with file paths and a report path, for example
`project-analyst`: "where is the suppression decision made for chords?", or `researcher`:
"how does `WH_KEYBOARD_LL` behave under UIPI for elevated windows?".

## Maintenance

- **Keep every copy in sync** when the routing changes: the frontmatter here, the `MODEL`/`EFFORT`
  maps in `ks-lane.js` (and the doc-writer call in `ks-phase.js`), the role files' own "You run on" lines, and the
  routing text in this README, `CLAUDE.md` and `docs/WORKFLOW.md#model-routing`. Do not downgrade a role without a measured before/after over
  at least ~25 items (see WORKFLOW.md).
- **Project knowledge embedded in role files** includes the module layout, conventions, the risk areas R1-R10
  (code-reviewer), the sentinel, thread ownership and owner decisions. When the architecture moves, update the
  developer's conventions and the code-reviewer's R1-R10 first, then the others' project-context sections. Examples of a move:
  `src/engine` arriving in Phase 0 and taking over matching, the gesture mutexes being removed, the `log` crate landing.
- **The Safety rules** are the same in every role, in the lane header of `ks-lane.js` and in the book-sync brief of
  `ks-phase.js` (a role may add lines of its own). Change a rule everywhere or nowhere.
- **Superseded:** `key-switch-coder`, `key-switch-researcher` and `key-switch-reviewer` were the
  earlier three-agent setup. They are replaced by `developer`, `researcher` and `code-reviewer`.
- A new role gets a row in the table above, in `CLAUDE.md` and in `docs/WORKFLOW.md#roles`, plus an
  entry in the lane templates' `MODEL`/`EFFORT` maps.
