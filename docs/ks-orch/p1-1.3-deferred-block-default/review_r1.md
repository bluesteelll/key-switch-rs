# Code review: p1-1.3-deferred-block-default @ 1562acd
APPROVED (Critical 0, Important 0, Minor 2)

Read through git only: `git diff master...feat/deferred-block-default` (master `1afa720`, lane `1562acd`, 4 commits), plus
`git show 1562acd:` of `src/engine/{mod,keys,gestures,testing,properties}.rs`, `src/config/{loader,schema}.rs`,
`src/data/{binding,key_combination}.rs`, `src/hook/keyboard_hook_callback.rs` (unchanged, read for the adapter contract),
`src/core/windows_actions.rs` (`press_vk`), `book/src/roadmap.md` 1.2/1.3, the CHANGELOG/README/template hunks and all ten book hunks.

## Gate: fmt pass, clippy pass
- Tree extracted with `git archive 1562acd` into the reviewer scratchpad (the work tree was not used).
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked --offline -- -D warnings`: exit 0, no warnings. The lib, the bins and the test targets were all checked.
- Extra: `mdbook build book` from the same tree builds. Every anchor the new text links to exists in the output (`s11-8`, `s11-4`, `s15-3`, `s3-6`, `s5-1`, `s10`, `s19`, roadmap `12-`, `13-`, `31-`).
- `cargo test` was not run (the tester owns it). The lane logs show red on `6950476`: `commit2_red.log:589`, 18 failed, the declared set. They show green on HEAD: `head_lib.log:418`, 411 passed.

## Remarks

### Critical
None.

### Important
None.

### Minor

#### m1. "Until roadmap item 1.2" names a fix that item 1.2, as written, does not deliver (KS-12)
**Where**: `CHANGELOG.md:148-149`, `README.md:406-408`, `book/src/config/known-issues.md:134` (row 66, "Open until roadmap item 1.2 for the menu bar and Start"). **Confidence**: CONFIRMED (against the roadmap text)
**Problem**: Item 1.2's mask fires only "When `decide()` suppresses an event while Win or Alt is held, and that modifier's key-down has already reached the OS" (`book/src/roadmap.md:238`).
**Failure**: Take `( keys: "LWin", action: ToggleCapsLock, trigger: Tap(200) )` with the new default `false`. Win down passes and Win up passes. No event is suppressed while Win is held, so 1.2's mask never triggers, and Start still opens after 1.2 ships. The same holds for a lone Alt and the menu bar. So the docs give row 66 an owner that does not cover it. The critic's open question 1 asks whether to merge 1.2 first, and it rests on the same premise.
- Smaller point: the CHANGELOG sub-bullet says only "A `Tap` on `Alt` or `Win`". README and the book say "`Tap`, `Hold` or `DoubleTap`".
**What to do**: Mark row 66 as having no roadmap owner yet, the way row 67 does, or have the orchestrator extend 1.2's scope (a mask when a gesture fires on a passed Win/Alt) and cite that. Align the CHANGELOG sub-bullet with README.

#### m2. The Breaking entry does not say that a modifier-only `Hold` now sends its injected output with that modifier down (KS-19)
**Where**: `CHANGELOG.md:132-151` (Breaking entry); `book/src/config/known-issues.md:136` (row 68 covers only the `ToggleCapsLock` of an `LShift` `DoubleTap`). The code path: `src/engine/mod.rs:438-453` (Hold arms, and now passes by default), `src/hook/keyboard_hook_callback.rs:204-211` (`hold-timer` -> `carry_out` while the key is still held), `src/core/windows_actions.rs:190-220` (`press_vk` sends down+up and releases no held modifier). **Confidence**: CONFIRMED for `Hold`; PLAUSIBLE for `DoubleTap` (it depends on how Windows orders the passed second key-down against the injected events).
**Failure**: `( keys: "LCtrl", action: PressKey("Escape"), trigger: Hold(300) )` with no `block_default`:
- Before the lane, LCtrl was swallowed and the timer sent a plain Escape.
- Now LCtrl reaches Windows, so the timer's Escape arrives as Ctrl+Esc and opens Start. `( keys: "LWin", action: PressKey("L"), trigger: Hold(..) )` becomes Win+L and locks the session.
- The config does not change, only the version.
- The book explains the mechanism in general (row 30, `actions.md:60`). Row 68 and the four Breaking sub-bullets name other consequences, but not this one.
**What to do**: Add one sub-bullet to the Breaking entry, and widen row 68 or point it to row 30. Say that the action of a modifier-only `Hold` (and likely `DoubleTap`) now runs with that modifier down in Windows, and that `block_default: true` restores the old output.

## Checklist walked (no reachable failure)
- **Plan conformance**: D1-D9 implemented as cut. The deviations (m3 stronger, two configs in k, the I1 scenarios, extra checks, the harness test placement) are justified in impl.md and only add strength. Commit 2 is declared red, with the failing lines in its body. Commits 1, 3 and 4 are green. README, the template and CHANGELOG change in commit 3.
- **Acceptance**: each of the four brief lines is pinned by a scenario that was red on `6950476` or is a guard that a wrong fix would break:
  - line 1: `lshift_double_tap_fires_and_every_lshift_event_passes` and the flipped `lshift_double_tap_fires_only_for_the_left_shift`;
  - line 2: `lshift_held_with_its_double_tap_bound_types_a_capital_a`, including `expect_down_in_windows("LShift")` at A's key-down;
  - line 3: `capslock_tap_with_the_default_block_default_still_swallows_capslock`;
  - line 4: `explicit_block_default_true_on_a_modifier_only_double_tap_swallows`.
  - Loader rules L1-L4 are pinned. `is_modifier_only` uses `all`; a version with `any` would fail L1's `Shift+CapsLock` row. A missing gate on any one trigger fails scenario a, b or c.
- **Suppression invariant**: the key-up rule `mod.rs:576-579` and the repeat rule `:537-543` are untouched. A passed key-down is recorded `Passed` (`:544-551`). No path turns `Passed` into `Suppressed` while the key is held:
  - `mark_completed` skips `Passed` (`keys.rs:166-172`);
  - `forget_blocked` keeps it (`:212-218`);
  - `reset_from_os` re-seeds held keys as `Passed`;
  - `mark_blocked` runs only on a fresh press.
  - The OR across bindings (scenario l) still yields one decision per press. The property configs now cover passing (F15 Tap; LShift and RShift by default; Ctrl+Shift in `mixed`) and swallowing variants.
- **R1 hot path**: the adapter is unchanged. The change adds one bool read per matching deferred binding. `blocks_by_default` runs only in the loader. No new allocation, lock, I/O or Win32 call. `code < 0` and the drop/pass outcomes are untouched.
- **R2 sentinel**: no new `SendInput` builder and no new constant.
- **R3/R4**: no new static, atomic, `Mutex`, `RefCell` or ordering. Engine state is still plain data under `Mutex<Engine>`.
- **R5**: `blocks_by_default` is an associated fn, not a setter. The constructors keep `true` and two arguments. `with_block_default` is unchanged.
- **R6**: no new variant. `Display for Binding` still does not show `block_default`, as the cut lists among its non-goals.
- **R7**: the sort and the single publish are untouched. Loader resolution happens before `update_bindings`, and every path (startup, reload, cfgcheck) goes through `raw_binding_to_binding`.
- **R8**: `is_modifier_only` uses `KeySpec::modifier()`, the same predicate `is_trigger_key` used inline. The refactor in `matching.rs:58-63` is equivalent. Sequences and chords ignore the flag. An empty combination yields `true` but is unreachable from the parser (`blocks_by_default`) and makes `any` false in `is_trigger_key`.
- **R9/R10**: not touched. Tests use `from_ron_with_language_hotkey` / `from_ron_str_with_hotkeys` and never the registry. No `#[ignore]` test was added, and none is needed.
- **unsafe / Win32 handles**: the diff adds no `unsafe` block and no handle. The only new `panic!` calls are in tests.
- **Other kinds and triggers**: Immediate resolves to `true`, as before. Sequence and chord resolve to `true` and ignore the flag. Deferred bindings with a payload key resolve to `true`. Builder-made bindings (hook and adapter tests) are unchanged. The only behaviour change is the spec's modifier-only default, which is declared Breaking.
- **Schema**: `Option<bool>` with `#[serde(default)]` under `IMPLICIT_SOME` (L3). `default_true` is removed with its last user.
- **Docs**: README, the template (example marked as working, plus the `<T>` caveat), the CHANGELOG (Changed/Breaking, Fixed, and the KS-07 edit, all under Unreleased) and ten config pages are updated. The KS-13 OR text now covers deferred bindings. Source lines added under `src/` are ASCII.

## Positive
- The default is resolved once, in the loader. The engine holds no policy, and builder order cannot overwrite an explicit value.
- The invariant follows from the per-key record, not from the binding. Reload in both directions, a foreground change and `swch off`/`on` each have a pass-path scenario (m1-m3).
- D9 keeps the KS-06/KS-14 swallow-path regressions explicit (`block_default: true`) and adds pass-path twins (h, i), so neither path loses coverage.
- `expect_down_in_windows` makes acceptance 2 literal, and a `should_panic` self-test pins it.

## Open questions
1. The I1 owner question is still open (impl.md:133): should a pass-through gesture be cancelled by another key? Two scenarios pin the current behaviour as a known issue (`scenarios.rs:3341`, `:3376`). The developer chose option (b), but the critic asked for an owner answer before implementation. The orchestrator should confirm that the owner accepts shipping the `<T>` / quick Ctrl+C misfires on `master`.
2. Merge order against 1.2 (critic's open question 1): in light of m1, merging 1.2 first would not remove the lone Alt/Win tap regression, so that question needs a different premise.
3. The DoubleTap half of m2 (and the KS-19 row 68 claim itself) depends on whether Windows processes events injected by `SendInput` from inside the LL hook procedure after the event being decided. Manual check M1 settles it for `ToggleCapsLock`.
