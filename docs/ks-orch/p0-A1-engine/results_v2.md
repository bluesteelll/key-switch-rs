# Results analysis v2: p0-A1-engine / Workstream A scope additions (pure decision core, Env, scenario tests), re-check after the rework

## VERDICT: ACCEPT

**Summary.** Both blockers from `results.md` are resolved on `refactor/engine` HEAD `10df59a`, and the work tree is clean.

- **P1 (tests left uncommitted).** The four tester tests are in `ab0d0b9`, in the right modules, with house-style names and the tester's original assertions. I re-ran M4 and M17 myself on a scratch copy: each turns its test red, and both pass again once reverted.
- **P2 (blank line in the changelog).** The blank line inside the Unreleased `### Fixed` list is gone (`10df59a`).

My own gate on a clean `git archive 10df59a` export is green: 193 passed, 0 ignored. The production-only engine purity check prints nothing. Nothing else changed since the first analysis (`d3131fc`), so all of its acceptance, KS-ID and invariant findings still hold.

## Scope of this re-check

`git diff --name-status d3131fc..HEAD` lists four files: `CHANGELOG.md`, `src/engine/mod.rs`, `src/engine/scenarios.rs` and `src/hook/keyboard_hook_callback.rs`. All four changes are additions (tests and one comment) or a deletion (one blank line). No production code changed. The full acceptance mapping, the KS-ID table and the invariant review from `results.md` (done at `d3131fc`) therefore still apply. Below I re-verify only what the rework touched, plus the full gate.

## Quality gate (own run)

Run on a fresh `git archive 10df59a` export in my scratchpad, with its own `CARGO_TARGET_DIR`. The export's file set is identical to `git ls-tree -r 10df59a` (checked with `diff`). The shared tree was not built.

| Check | Result |
|---|---|
| Work tree `git status` | `nothing to commit, working tree clean`. `--ignored` shows only `.claude/settings.local.json`, `book/book/` and `target/` |
| `cargo fmt --check` | OK (exit 0) |
| `cargo clippy --all-targets --locked -- -D warnings` | OK (exit 0) |
| `cargo test --locked` | exit 0. lib: `running 193 tests` / `193 passed; 0 failed; 0 ignored`. swch and doc-tests: 0 tests. The output is saved in `D:/tmp/ks-orch/p0-A1-engine/analyst-test-v2.txt`. For comparison, master `73bbf40` has 130 tests (120 at audit commit `7cb3521`) |
| `cargo test --locked -- --list --ignored` | `0 tests, 0 benchmarks` in all 3 targets |
| `cargo doc --no-deps` | 1 warning, `src/data/vk_name.rs:4` "unclosed HTML tag `raw`". It predates the lane: the file is not in `git diff --stat master...refactor/engine` |
| Production-only engine purity (cut addendum M1) | `check src/engine 'GetAsyncKeyState\|GetForegroundWindow\|SendInput\|Mutex\|thread::\|Instant::now\|ForegroundContext\|\.execute\(\|unsafe *(\{\|fn\|impl)'` prints nothing (grep exit 1) |
| Production-only hook decision-logic check (cut I2) | `check src/hook 'Trigger::\|BindingKind::\|...\|SequenceState'` prints nothing (grep exit 1) |
| Spec grep `GetAsyncKeyState\|GetForegroundWindow\|SendInput\|Mutex\|thread::` over `src/engine` | one hit, `src/engine/mod.rs:73`, which is a `//!` doc line. 0 code lines |
| `git diff --check master...refactor/engine` | clean |
| Commit trailer | 14 of 14 commits in `master..refactor/engine` carry `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` |
| Mergeability | `master` = merge-base = `73bbf404`. `git merge-tree --write-tree master refactor/engine` exits 0. A `--no-ff` merge has no conflicts |

## P1 re-check: the four tester tests

| Test (new path) | Location | Mutation it catches | Assertions vs `tester_tests.patch` | Evidence it is red under the mutation |
|---|---|---|---|---|
| `engine::tests::forget_timer_makes_a_late_timer_fire_nothing` | `src/engine/mod.rs:443-489`, a new `#[cfg(test)] mod tests` at the bottom of the file, with a local `FixedEnv` | M4: `Engine::forget_timer` becomes a no-op (KS-33) | identical. One KS-33 comment was added | **My run**: body replaced by `let _ = token;` gives `FAILED`, `panicked at src\engine\mod.rs:483:9`, `left: Decision { suppress: false, fire: [PressKey(VIRTUAL_KEY(160))], .. }`. It passes again once reverted |
| `engine::scenarios::tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` | `src/engine/scenarios.rs:281-295`, in the Tap section | M15: the Tap arm stops skipping auto-repeats | identical (`expect_suppressed` + `expect_nothing_fired` on the release). A comment explains why the repeat's own pass is not pinned (KS-06) | test_r1. The body is unchanged, so it was not re-run |
| `engine::scenarios::chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` | `src/engine/scenarios.rs:727-745`, at the end of the Chord section | M16: a completed chord stops re-marking its keys as blocked | identical (4 suppression checks + `expect_fired(SwitchLanguage)`) | test_r1. The body is unchanged |
| `hook::keyboard_hook_callback::tests::extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one` | `src/hook/keyboard_hook_callback.rs:338-381`, in the existing `mod tests` | M17: the extern callback stops returning `LRESULT(1)` on suppress | identical (F13 down and up give `LRESULT(1)`; `A` down and up do not) | **My run**: the `if suppress { return LRESULT(1); }` block removed gives `FAILED` at `keyboard_hook_callback.rs:367:9`, `left: LRESULT(0)`. It passes again once reverted |

- **Names and placement.** No `tester_` prefix and no "uncommitted" comment remain (`grep -rn "tester\|uncommitted" src` finds nothing). Every test module sits at the bottom of its file as `mod tests` (CLAUDE.md principle 9). Because of that, the purity check's `^mod tests {` cut-off now leaves out the test's `Instant::now()` at `mod.rs:472`, which is why that check is empty.
- **Safety of the extern test.** I checked it in the code:
  - The binding is `Binding::new(F13, DoNothing)`, which means `Trigger::Immediate` (`src/data/binding.rs:85`), so no timer is requested.
  - `BindAction::DoNothing => {}` (`src/core/windows_actions.rs:139`), so `carry_out` sends nothing.
  - No hook is installed.
  - The pass path's `CallNextHookEx(None, ..)` has no next procedure to call, and it injects nothing.
  - The test's `// SAFETY:` comment (`:354-360`) states all of this, which answers critique open question 1.
  - It is the only test that touches `get_hook()` (`rg get_hook src`: `app.rs:88`, the callback's production uses, and this test). Parallel tests therefore never share the singleton.
- **Commit.** `ab0d0b9` "Engine: keep the tester's round-1 gap tests" names each test and its mutation in the body and carries the trailer.

**P1: RESOLVED.**

## P2 re-check

`10df59a` deletes exactly one line (`CHANGELOG.md`, `@@ -65,7 +65,6`). The Unreleased `### Fixed` list now runs KS-11 (`:55`), KS-25 (`:59`), KS-33 (`:65`), KS-38 (`:68`), KS-42 (`:77`), with no blank line between any of them. The index stores the file with LF line endings (`git ls-files --eol`: `i/lf`), and the diff changes no other line.

**P2 (review m1): RESOLVED.**

## Acceptance criteria, KS-IDs and invariants

These are unchanged since `results.md`; every row there was MET or CLOSED, citing `file:line`. The rework improves two rows:

- **Criterion 3a** (the hook callback calls the engine; `LRESULT(1)` iff `suppress`). The extern-level mapping now has committed automated evidence: the M17 test above. Before, only code review covered it.
- **KS-33 (partly).** The `forget_timer` contract is now pinned by a committed regression test, the M4 test above. Before, only the uncommitted copy pinned it.

Nothing new in the hook path. The rework adds no production code, no new `unsafe` outside tests and no atomics. Its one new `unsafe` block is test-only, and its `// SAFETY:` comment names concrete invariants.

## Review status

- `review_r1.md`: APPROVED (Critical 0, Important 0). Its only Minor, m1, is now resolved (P2).
- `test_r1.md`: GREEN. Its "must keep" tests are now committed (P1).
- The rework commits touch only tests and one changelog line, so they need no new review round.
- No CONFIRMED Critical or Important item is open.

## Docs and hygiene

- CHANGELOG has the Keep a Changelog headings, and the Fixed list is now tight.
- The README, `default_config.ron` and the book are unchanged by the rework, and config meaning is unchanged.
- No new dependencies. `--locked` passes.

## Technical debt

No new `TODO` or `FIXME`. The debt list from `results.md` still applies unchanged: the re-entry key-up residual risk, moving `BindAction` into `data` (roadmap 3.4), the scenario that pins focus-change clearing (roadmap 2.3), the `Mutex<Engine>` against CLAUDE.md principle 5, and the remaining part of KS-33. Its "where recorded" gaps are carried as hand-overs H1-H3.

## Problems

None blocking.

## Hand-overs (unchanged from `results.md`, still open, for the orchestrator)

- **H1, right after the merge and before the next Workstream A cut.** `CLAUDE.md` still says "`src/engine/` does not exist yet" (`:24`), lists `tap_state.rs` and its siblings (`:21`), and principle 5 forbids a "hook-wide `Mutex`" with no item-2.2 exception (`:65`). The same wording is in `.claude/agents/architect.md:37` and code-reviewer rule R3.
- **H2, book sync** by the doc-writer phase. This covers `config/triggers-conditions.md` §11.4-11.8 and the KS-08 note, `config/known-issues.md` rows 23-24, the `architecture.md` thread table and key-event path, the roadmap 2.2 "Files" list, and the `audit.md` status of KS-11 (fixed) and KS-25/KS-33 (partly). It also adds manual checks M1-M9 to a smoke checklist.
- **H3, record during the book sync.** Record two items:
  - the re-entry key-up residual risk, as a precondition on roadmap items 2.3 and 4.6, or in `gaps.md`;
  - "move `BindAction` into `data`" as a 3.4 follow-up.

## Manual checks for the owner

M1-M9 in `results.md` still apply. They need a live hook. They do not block acceptance: every acceptance line has automated evidence.

## Positives

- The rework did exactly what was asked and nothing more:
  - two small commits;
  - assertions kept byte-for-byte;
  - each test moved to the module whose contract it pins;
  - the SAFETY comment extended with the one fact a reviewer would ask about (why `CallNextHookEx` is harmless with no hook installed).
- The developer's red checks for M4 and M17 reproduce exactly in an independent copy. The panic locations and `left:` values match the report.
