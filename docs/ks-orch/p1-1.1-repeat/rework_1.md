# Tester rework 1: p1-1.1-repeat (P1, Hold pin too short)

HEAD 5879bda (previous 10bcf94). Test-only commit `Tests: a held Hold binding fires once across many repeats (KS-10)`, Sonnet trailer. No production change.

## Change
New test `engine::scenarios::held_hold_binding_with_repeating_action_fires_once_across_many_repeats` (src/engine/scenarios.rs, before `a_repeat_of_a_sequence_step_does_not_count_as_the_next_step`): `Hold(200)` + `PressKey("Backspace")`, down, +500 ms (one Backspace), ten repeats 33 ms apart (last at +797), release after +830, every event suppressed, `expect_total_fired` exactly one Backspace. The existing test was left untouched.

## Red-first (mutation, scratch worktree D:/tmp/ks-orch/p1-1.1-repeat/wt/rw1, own target dir, removed after)
Mutation: src/engine/mod.rs Hold arm `if was_repeat {` -> `if was_repeat && !binding.repeat {`.
- Mutant: `test result: FAILED. 386 passed; 1 failed; 1 ignored` (388 tests); the only failure is the new test:
  `scenario: expected the scenario to fire [PressKey(VIRTUAL_KEY(8))] in total, it fired [PressKey(VIRTUAL_KEY(8)), PressKey(VIRTUAL_KEY(8))]`
  (before this commit the same mutant passed 386/386, per results.md).
- HEAD: `running 1 test ... ok` (targeted), then full run below.

## Gate on HEAD 5879bda
- cargo fmt --check: OK
- cargo clippy --all-targets --locked -- -D warnings: OK (sources touched first; `Checking key-switch-rs ... Finished` 1.58 s)
- cargo test --locked --no-fail-fast: lib `running 388 tests`, 387 passed, 0 failed, 1 ignored (pre-existing manual: foreground_context_names_the_focused_notepad); ipc_roundtrip `running 6 tests`, 6 passed; swch and doc 0 tests. Lib count 387 >= 387 required (386 + 1).
- Release build not rerun: no production or manifest change.

## Hygiene
Tree clean, `git worktree list` shows only the lane tree. Rulings (Q1 accepted; critic notes and CHANGELOG:242 reflow for the book sync) require nothing from this rework.
