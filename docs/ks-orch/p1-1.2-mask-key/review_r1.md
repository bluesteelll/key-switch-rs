# Code review: p1-1.2-mask-key @ 0187cde
APPROVED (Critical 0, Important 0, Minor 2)

Read through git only: `git diff master...feat/mask-key` (master `ece1767`, merge-base `ece1767`, HEAD `0187cde`, 9 commits), full files at `feat/mask-key` for `src/engine/mod.rs`, `src/engine/keys.rs` (1-260 + new test), `src/hook/keyboard_hook_callback.rs` (whole file), `src/config/loader.rs:200-430, 480-612, 1105-1285`, `src/data/binding.rs` (diff + Display), `src/config/schema.rs` (diff), `src/core/constants.rs` (diff), `src/engine/testing.rs` and `properties.rs` (diff), `src/engine/scenarios.rs:1-60, 3890-4480`, `keyboard_hook.rs::set_enabled`, `key_combination.rs::is_modifier_only`, `condition.rs` (Or/is_always/evaluate). Docs: CHANGELOG, README, `default_config.ron`, and every `book/src/config/*.md` hunk.

## Gate: fmt OK · clippy OK

Run on a `git archive feat/mask-key` export (not the tester's tree, not `wt/mut`), in the session scratchpad with its own target dir:
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0, no warnings (`Finished dev profile`).
- `cargo test` not run (the tester owns it).

## Remarks

### Critical
None.

### Important
None.

### Minor

#### m1. The MaskOnly fallback warning says "blocked as with true" when the KS-18 rule adds no blocker at all (R7, KS-18)
**Where**: `src/config/loader.rs:353-360` (warning pushed while the requests are collected) vs `:373-379` (KS-18 skip, checked afterwards). **Confidence**: CONFIRMED (traced).
**Problem**: The fallback warning goes out for every `MaskOnly` request on a non-modifier hotkey before the loader knows if that hotkey will get a blocker. When a `keys` binding of the file uses the hotkey's keys, the KS-18 skip then drops the blocker.
**Failure**: Registry hotkey value "4" (`` ` ``), config `[(keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly), (keys: "`", action: DoNothing)]`. Startup prints ``[WARN] binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true`` and right after it `[INFO] no auto-blocker for `: binding #1 uses the same keys`. The WARN states something false: no blocker exists, and binding #1 decides about `` ` ``. The config is rare and nothing misbehaves at run time. Only the console text is wrong.
**What to do**: emit the fallback warning only for hotkeys that actually get a blocker, or word it so that it holds in both cases.

#### m2. Agent-facing docs still say `Decision::mask` is reserved, and they are not in the phase book-sync hand-off
**Where**: `CLAUDE.md:26` at `feat/mask-key` ("`fire`, `timers`; `mask` is reserved for roadmap 1.2"), `.claude/agents/architect.md:37` ("`Decision { suppress, fire, timers, mask }`; `mask` is reserved for roadmap 1.2"). **Confidence**: CONFIRMED (`git grep` at the lane ref).
**Problem**: impl.md "Open points" hands `architecture.md`, `build-and-run.md`, `glossary.md`, `roadmap.md:117`, the audit status lines and `methodology.md` to the phase book sync. It does not list these two files. Every later agent reads both before it plans.
**Failure**: an architect cutting item 3.1 or 2.2 from `CLAUDE.md` takes `mask` as an unused field and may plan to set it from scratch. It may also not keep `carry_out`'s mask-first order and the `cfg(test)` seam.
**What to do**: add both lines to the hand-off list, or let the orchestrator update them at merge. `CLAUDE.md` is not the developer's file.

## Checklist walked (no reachable failure found)

**Section 1: plan and roadmap conformance**
- D1 to D10 are implemented as cut.md says. The deviations listed in impl.md (transcript marker, the shared `with_alt_shift_blocker_of`, `mask_only` config appended last, a D1 helper that never clears the mask, doc extras) are harmless and justified.
- Roadmap 1.2 acceptance: automated half pinned (S1-S4, S8-S11 true; S5-S7, S12 false; property oracle `check_mask_rule`, `testing.rs:562-575`). v2: S16-S20, L1, L5. KS-35: S13, S14, L7-L9, E1. Manual half: M1-M9 are listed in cut.md, in the CHANGELOG (Added, Changed and Fixed entries) and in `matching.md` §15.5.
- The decision logic sits in `src/engine` (`mod.rs:646-654`, `keys.rs:229-241`). The adapter only carries it out.
- CHANGELOG: Added (`MaskOnly`), Changed **Breaking** (KS-35), Fixed (KS-12), all under `[Unreleased]`. README, `default_config.ron` and the config pages are updated. Code links are pinned to `ad7fe86`.

**Section 2: unsafe**
- Two new `unsafe` blocks. `keyboard_hook_callback.rs:246-250` (`SendInput` over a stack `[INPUT; 2]`, `cbsize = size_of::<INPUT>()`, a lossless `as i32` of 40) and test H1 `:554-556` (union read of `ki` on records built as `INPUT_KEYBOARD`). Both have concrete SAFETY comments that hold.
- No new pointer from Win32, no wide string, no handle.
- No new panic path in the `extern "system"` callback. `menu_mask_inputs` and `injected_sentinel` neither index nor unwrap.

**Section 3 / R4: atomics**
- None added. `mask_owed` is plain data in `Mutex<Engine>`, taken and set only inside `on_key` and `reset`. `MASKS_SENT` is a `cfg(test)` thread-local.

**Section 4 / R10: Win32 contract and thread affinity**
- No install, uninstall or new thread.
- `SendInput` runs in `carry_out` on the hook thread after `decide_event` returned, so the lock and the snapshot are already gone (`:191-195`, `:339-341`). It comes before `LRESULT(1)` (`:343`) and before `CallNextHookEx` (`:349`).
- The returned count is checked (`:258-262`).
- `carry_out` on `hold-timer` (`:293`) never masks: `on_timer` builds `Decision::default()` (`mod.rs:366-372`), and `testing.rs:415` asserts it.

**Section 5 / R1: hot path**
- No allocation on the new paths. `menu_mask_inputs` builds a stack array. The D1 check is a 5-entry scan. The MaskOnly branch only sets two bools.
- Nothing added under the engine lock.
- `eprintln!` runs only on a short count, the same pre-existing pattern as `press_vk`, which item 1.7 replaces.
- `code < 0` and remapping-off early returns are unchanged. Every path still ends in exactly one of `LRESULT(1)` or `CallNextHookEx`.
- D8: the blocker `when:` is evaluated only after `blocker_matches` (`mod.rs:449-451`, `&&` short-circuit), pinned by E1. A swallowed auto-repeat under Alt/Win masks again (at most the repeat rate). The cut accepted that (Q2).

**R2: sentinel**
- Both E8 records carry `constants::injected_sentinel()` (`:234`), pinned by H1. There is no new hard-coded constant (`MENU_MASK_VK` is a key code, not a sentinel).
- `decide_event` checks the sentinel at `:179` before it takes the snapshot or the lock, so the E8 events that `SendInput` re-enters with pass without touching state.

**R3: shared state**
- No `static mut`, `RefCell` or new `Mutex`. `with_engine` is unchanged and still refuses a nested call.

**R5: builders**
- `new_auto_blocker(combination, mode)` takes two positional arguments, and the condition goes on through `.with_condition`. `with_block_original_combo(Option<BlockerMode>)` consumes self. `is_auto_blocker()` stays `pub(crate)`.

**R6: Display**
- `BlockerMode` has a `Display` (`binding.rs:82-89`).
- The blocker arm matches both modes exhaustively (`:283-292`).
- `raw_blocker_to_mode` is exhaustive (`loader.rs:602-608`). No `_ =>` arm was added.

**R7: specificity order**
- `sort_by_specificity` is unchanged.
- Blockers are still appended after the file's bindings (`loader.rs:398-400`). Hotkeys and modes come in first-request order.
- For `true` with no `when:` the output equals master's: one blocker per hotkey, condition `Always`.

**R8: key identity**
- `MENU_KEYS` covers L/R Alt, the generic `VK_MENU` slot (resync-written; the fake models it at `testing.rs:101`) and L/R Win.
- Traced cases: Alt key-up in flight with stale `VK_MENU`, and AltGr's RAlt. The mask is right in both: only a suppressed event masks, and a suppressed Alt key-up implies a swallowed key-down.

**R9: registry**
- No registry change. `is_modifier_only` is applied to the hotkey that was read.

**Section 6: correctness**
- The suppression invariant is untouched: `mask` never feeds `suppress`. MaskOnly never sets `suppress`. A reload from `true` to `MaskOnly` mid-press stays symmetric through the repeat rule and `take_blocked`.
- The owed mask is consumed by the next engine event only, cleared by `reset` (S21), and harmless when stale.
- KS-35 (`any_of_conditions`) returns `Always` on any `Always` owner, deduplicates equal conditions and ORs the rest. Mode grouping applies after the MaskOnly fallback.

**Safety seam**
- The `cfg(test)` `send_menu_mask` (`:268-271`) only increments `MASKS_SENT`. It calls neither `inject_menu_mask` nor `SendInput`.
- `tests/` holds only `ipc_roundtrip.rs`, which reaches no `carry_out` or callback. Both are private or `pub(crate)`.
- H3 is `#[ignore = "manual: ..."]`.

**Tests that can fail**
- The red-first commits are reported red for the right reason (pack table).
- The guards S5, S6, S7 and S12 each kill a named mutation.
- H2 fails if `carry_out` ignores `mask` or always sends it.

**Section 7: style and commits**
- Source is ASCII. No new `#[allow]`, no new dependency, `Cargo.lock` unchanged.
- Every commit has the subject `<Area>: <what>` and ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

**Hand-off already recorded (not a remark)**
- `book/src/methodology.md:408-409` (cfgcheck) formats `b.block_original_combo` with `{}`, which no longer compiles for `Option<BlockerMode>`.
- `architecture.md`, `build-and-run.md`, `glossary.md`, `roadmap.md:117`/`:308` and the audit status lines are named in impl.md for the phase book sync.

## Positive
- The mask is computed from records the resync already keeps, with no new `Env` call. The property oracle reads the modelled OS, not the engine's records, so it is a real second opinion.
- `carry_out` destructures `Decision` without `..`, so a future field fails to compile until someone carries it out.
- The `cfg(test)` seam is narrow and leaves nothing to chance: `cargo test` cannot inject E8, whatever keys the owner holds.
- D8 heads off the KS-08 regression that KS-35 would otherwise have brought.
- `RawBlockOriginalCombo` keeps `true`/`false` loading. That rests on reading ron 0.10.1's source, and L1/L2 pin both values and the error text.

## Open questions
1. The mask in the Alt+Tab switcher with the default `true` blocker (Alt held, Tab tapped, Shift swallowed, E8 injected while the switcher is up): PLAUSIBLE, not decidable without live input. Manual check M3 settles it. The orchestrator has already made a failure a RETHINK at results.
2. Whether the MaskOnly hypothesis holds (M4/M5), and the injection order of a `SendInput` made from inside the LL hook: both are by design manual. The owed mask hedges the order.
