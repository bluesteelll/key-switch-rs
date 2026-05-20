# Agents for `key-switch-rs`

Three specialist agents that collaborate on feature work in this codebase. Each one is intentionally narrow — together they form a research → implement → review pipeline that mirrors a small engineering team.

## The three agents

All three agents inherit the model of the orchestrating session — no `model:` field is set in their frontmatter. When the main session runs on Opus, every subagent runs on Opus too.

| Agent | Role | Reads | Writes |
|---|---|---|---|
| [`key-switch-researcher`](key-switch-researcher.md) | Investigates *how* to build something. Surveys Win32 APIs, the `windows` crate, the existing code, and known footguns. Produces a written report with API choices, integration sketch, edge cases, and citations. | repo + web | nothing (research report only) |
| [`key-switch-coder`](key-switch-coder.md) | Implements a specific, planned change. Knows the project's module layout, unsafe-Win32 conventions, atomic patterns, builder API, and the injected-event sentinel mechanism. Compiles, tests, and lints what it writes. | repo + cargo | source files |
| [`key-switch-reviewer`](key-switch-reviewer.md) | Critiques a diff against ten project-specific risk areas (hook-callback hot path, sentinel discipline, atomic orderings, builder consistency, registry I/O, etc.). Produces a severity-ranked findings list. | repo + cargo | nothing (review report only) |

## Recommended workflow

```
┌────────────────────────┐     ┌─────────────────────┐     ┌─────────────────────┐
│  key-switch-researcher │ →   │  key-switch-coder   │ →   │ key-switch-reviewer │
│  "how do we do X?"     │     │ "implement X per    │     │ "review the diff    │
│  report                │     │  the report"        │     │  on branch foo"     │
└────────────────────────┘     └─────────────────────┘     └─────────────────────┘
```

### When to skip stages

- **Skip the researcher** when the change is mechanical and well-understood — adding a `Display` impl, renaming a private field, fixing a typo. Go straight to the coder.
- **Skip the coder** and stay in the main agent when the change is a one-line obvious edit. Spawning a subagent for a typo wastes context.
- **Never skip the reviewer** for changes that touch `hook/keyboard_hook_callback.rs`, any `unsafe` block, or any new atomic operation. Those are the highest-risk surfaces in the codebase.

## How to invoke

From the main Claude Code session:

```
@key-switch-researcher how should I implement double-tap detection for Shift?
```

```
@key-switch-coder implement the double-tap approach from the research report above.
Key fields: 300ms window, single binding type DoubleTap(VIRTUAL_KEY, BindAction).
```

```
@key-switch-reviewer review the diff between origin/main and HEAD
```

Or programmatically via the `Agent` tool with `subagent_type: "key-switch-coder"` etc.

## Why three agents and not one

A single agent that tries to research, implement, and review tends to:

1. **Skip research** under time pressure and guess at Win32 semantics — which produces subtle bugs in `unsafe` code that compile fine.
2. **Skip review** because it just wrote the code and feels confident — which means no second pair of eyes catches the missed `INJECTED_EXTRA_INFO` sentinel.
3. **Bloat its context** with API documentation when it should be writing focused code.

Splitting them:

- The researcher gets a heavy WebFetch/WebSearch budget to nail down the API — its output is plain text, not code, so its context can be discarded after the report lands.
- The coder stays narrow — it gets the research report as input and produces a diff, nothing else, keeping its context window focused.
- The reviewer comes in fresh, without the coder's "I just wrote this and it looks right to me" bias, and applies the R1–R10 checklist mechanically.

All three inherit the model of the orchestrating session, so the team scales up or down with the master agent's capability.

## Maintenance

Each agent file embeds project-specific knowledge:

- Module layout
- Layering rules
- The 10 risk areas (in the reviewer) / mandatory conventions (in the coder)
- Win32 idioms used in this codebase

When the architecture changes meaningfully — a new module, a new shared-state primitive, a switch from `static mut` to something else — update the corresponding sections in `key-switch-coder.md` (conventions) and `key-switch-reviewer.md` (risk areas). The researcher's project-context section is shorter and changes less often.

If you add a fourth agent (e.g. a `key-switch-benchmarker` or `key-switch-release-manager`), document it here in the same table.
