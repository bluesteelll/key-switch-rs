# Implementation: p0-A1-engine round rework (results.md P1, P2)

Verdict: DONE. Branch `refactor/engine`, HEAD `10df59a`, tree clean.

## Items
- P1 (tester round-1 gap tests, uncommitted) - committed in `ab0d0b9` with the assertions unchanged:
  - `engine::tests::forget_timer_makes_a_late_timer_fire_nothing` - `src/engine/mod.rs:443-489`. This is a new `#[cfg(test)] mod tests` at the bottom of the file, with its own `FixedEnv` (`:450`). It was `tester_tests::engine_forget_timer_makes_a_late_timer_fire_nothing`; the redundant `engine_` prefix is dropped inside `engine::tests`. Catches M4 (KS-33).
  - `engine::scenarios::tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release` - `src/engine/scenarios.rs:281-295`, in the Tap section after `tap_held_past_its_term_fires_nothing_and_swallows_the_release`. The why-comment says the repeat's own pass-through is KS-06 and is not pinned, which matches the file header's rule. Catches M15.
  - `engine::scenarios::chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key` - `src/engine/scenarios.rs:727-745`, at the end of the Chord section. Catches M16.
  - `hook::keyboard_hook_callback::tests::extern_callback_returns_one_for_a_suppressed_key_and_passes_an_unbound_one` - `src/hook/keyboard_hook_callback.rs:338-381`, in the existing `mod tests` (`tester_` prefix dropped). Catches M17.
  - `src/hook/keyboard_hook_callback.rs:271-276`: the test-module comment now says that no test installs a hook and that the `decide_event` tests use a local hook, while the extern test drives the process-wide one.
  - The mod.rs `mod tester_tests` and the "uncommitted" comments are gone (`grep -rn tester src`: no hits).
- Red check (delta, as results.md asks for M4 and M17). Each mutation was applied by Edit and reverted by Edit after the commit; `git status --short` was empty afterwards.
  - M4: `Engine::forget_timer` body replaced by `let _ = token;`. The test went red at `src\engine\mod.rs:483:9`, "a timer the caller could not start but that wakes anyway must not fire the forgotten Hold", with `left: Decision { suppress: false, fire: [PressKey(VIRTUAL_KEY(160))], .. }`. Green once reverted.
  - M17: the `if suppress { return LRESULT(1); }` in `keyboard_hook_callback` removed. The test went red at `src\hook\keyboard_hook_callback.rs:366:9`, "bound F13 down is swallowed", with `left: LRESULT(0)`. Green once reverted.
  - M15 and M16 were shown red in test_r1 and their bodies are unchanged; they were not re-run.
- P2 (review m1) - `CHANGELOG.md:68` blank line removed in `10df59a`, so the `### Fixed` list under `[Unreleased]` has no blank line between items. The work-tree CRLF is kept (Edit tool); `git diff` shows one deleted line.

## Remarks
- results.md P1 - RESOLVED in `ab0d0b9`, at the locations above. It is pinned by the four tests, and M4 and M17 were re-run red against the committed versions.
- results.md P1 "add why CallNextHookEx is harmless" (critique open question 1) - RESOLVED at `src/hook/keyboard_hook_callback.rs:354-360`. The SAFETY comment states:
  - code 0 is HC_ACTION;
  - lparam points to the stack `KBDLLHOOKSTRUCT`, which outlives the call;
  - no hook is installed and the thread is not inside a hook dispatch, so `CallNextHookEx` has no next procedure to call;
  - `hhk` is ignored (Microsoft Learn, CallNextHookEx), and nothing is injected or delivered.

  Observed evidence: under M17 the pass path returned `LRESULT(0)` from `CallNextHookEx` for the bound F13.
- results.md P2 / review m1 - RESOLVED at `CHANGELOG.md:67-68` (`10df59a`). Docs only, no test.
- results.md note "keep tests that drive `get_hook()` to one" - RESOLVED. The extern test is the only one (`rg get_hook src`: `app.rs:88`, the callback file's production uses, and this test), and its comment at `:340-343` says so.

## Deviations from the cut
- The M4 test is renamed from `engine_forget_timer_makes_a_late_timer_fire_nothing` to `forget_timer_makes_a_late_timer_fire_nothing`. Reason: its path is now `engine::tests::...`, and no other test repeats its module name (compare `gestures::tests::hold_timer_fires_its_own_entry_once`). Assertions are unchanged.
- Two commits instead of one: tests (`Engine:`) and the CHANGELOG fix (`Docs:`), as results.md P2 allows ("same follow-up, or its own `Docs:` commit").

## Unsafe / atomics added
- `src/hook/keyboard_hook_callback.rs:354-367`: one `unsafe` block, test-only, carried over from the tester patch with an extended SAFETY comment: "code 0 is HC_ACTION, and lparam points to `info`, a KBDLLHOOKSTRUCT on this stack frame that outlives the call: all the callback dereferences. No hook is installed and this thread is not inside a hook dispatch, so the pass path's CallNextHookEx has no next hook procedure to call; its hhk argument is ignored (Microsoft Learn, CallNextHookEx), and it neither injects input nor delivers the event to any window."
- No atomics added.

## Gates (HEAD `10df59a`, shared work tree, default target dir)
- `cargo fmt --check`: pass (exit 0).
- `cargo clippy --all-targets --locked -- -D warnings`: pass (exit 0).
- `cargo test --locked`: lib `running 193 tests` / `193 passed; 0 failed; 0 ignored`; swch and doc-tests 0 tests. The full run was requested by this brief.
- `cargo test --locked --lib -- --list` shows the four new names.
- The production-only purity checks from the cut addendum (`check src/engine '...|Instant::now|...'` and `check src/hook '...'`) give no output (grep exit 1 for both).
- `git diff --check d3131fc..HEAD`: clean.
- Manual ignores added: none.

## Docs
- CHANGELOG: blank line removed (P2). No other doc change; the book, README and `default_config.ron` are untouched (no behaviour change).

## Commits
- `ab0d0b9` Engine: keep the tester's round-1 gap tests
- `10df59a` Docs: keep the Unreleased Fixed list tight

## Open points / pre-existing warnings
- None new. The rustdoc warning at `src/data/vk_name.rs:4` predates the lane (results.md), and `cargo doc` was not re-run.
- Hand-overs H1-H3 in results.md are for the orchestrator and were not touched.
