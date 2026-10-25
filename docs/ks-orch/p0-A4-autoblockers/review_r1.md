# Code review: p0-A4-autoblockers @ 3d65ac9
APPROVED (Critical 0, Important 0, Minor 1)

Reviewed `git diff master...fix/auto-blockers` (merge-base `44f79ac`). The branch moved during the
review: the pack is on `f5595aa`, and `3d65ac9` landed at 21:51 (test-only, `src/engine/keys.rs` +14).
Both are covered here. All code was read through `git show`/`git diff` on the refs. Read in full:
`engine/mod.rs` on_key/on_key_down/on_key_up, `engine/matching.rs`, `engine/keys.rs`, the loader
expansion and its tests, `data/binding.rs`, `data/key_combination.rs`, `data/key_spec.rs`
(`accepts`), the scenario diff, the `testing.rs` diff, the `HookEnv` adapter
(`hook/keyboard_hook_callback.rs:94-165`), `system/registry.rs` lookups, `core/app.rs` banner and the docs diff.

## Gate: fmt OK, clippy OK
Run on a `git archive` export of the lane outside the repo, with its own target dir:
- `f5595aa`: `cargo fmt --check` exit 0. `cargo clippy --all-targets --locked -- -D warnings` exit 0
  (re-run after touching the crate roots: `Checking key-switch-rs ... Finished`, no warnings).
- `3d65ac9` (export updated with its `keys.rs`, the only file that differs): fmt exit 0, clippy `Finished`, no warnings.
- `cargo test` not run (the tester owns it).

## Remarks

### Critical
None.

### Important
None.

### Minor

#### m1. Commit 3d65ac9 was made by the tester role, with a non-standard trailer (process)
**Where**: commit `3d65ac9` "Tests: all_held visits the highest key code the table holds (KS-05)",
`src/engine/keys.rs:275-287`. **Confidence**: CONFIRMED (`git reflog fix/auto-blockers`: `commit` at
21:51:31, after the pack; the main work tree's HEAD moved to it).
**Problem**: `docs/WORKFLOW.md:23` gives the `tester` "scratch worktree under the lane dir; no
commits". The commit ends with `Co-Authored-By: Claude Sonnet 5.5`, but CLAUDE.md's git flow requires
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` on every commit. The content is fine: the test
pins the upper bound of `0..256u16` in `KeyState::all_held` (it fails with `0..255`), and fmt and clippy are green.
**Failure**: none at runtime. The lane history contains a commit outside the role split, and the
reviewed HEAD changed under the parallel review arm. The pack and this review started on
`f5595aa`.
**What to do**: the orchestrator decides whether to keep it or revert it with a follow-up commit
(amending is forbidden). Check that the tester's brief keeps it off the lane branch.

## Checklist walked (items absent with no reachable failure)

**1. Plan conformance.** D1 is the flag check first in the `Combo` arm (`engine/mod.rs:353-365`).
D2 is `blocker_matches` (`matching.rs:92-101`) with `HeldKeys::all_held` (`matching.rs:36`,
`keys.rs:204-210`). D3: on a match, only `suppress` is set. `fire` and `immediate_fired` are not touched, then
`continue`. D4/D5: `expand_with_auto_blockers` (`loader.rs:187-219`) gives distinct hotkeys in
first-request order, a whole-list `position` including the requester (`:205-207`), the `[INFO]` line
(`:210`), and blockers appended (`:216-218`). D6 is the `SystemCombination` seam (`:125`, `:134-145`).
The deviations in impl.md:36-44 are justified. Spec 3 (distinguishable) is the existing `pub(crate)
is_auto_blocker` (`binding.rs:75`), which stays `pub(crate)`.

**Acceptance.**
- (1) T2/T3/T4 (`scenarios.rs` new block) prove pass-through when the third key is down in Windows
  first. The modifiers-first residue is pinned by T6 and the M4 pin, and is carved out by `roadmap.md:100-101`. See Open questions.
- (2) T7: blocker first and last, red on run A before the engine commit. T1 goes through the
  loader, and T8 adds a sided user binding next to the blocker.
- (3) Loader tests L1-L4 and C1 are registry-free.
- The tests can fail: dropping `is_trigger_key` fails U1 (`pressed VK_A` case). Dropping the
  `continue`, or firing DoNothing, fails F1-F3. Setting `immediate_fired` fails T7 run A. The literal
  engine-held reading fails T9/T10. Asking Windows about the hotkey's own keys panics in U2. Overlap
  dedup fails L4.

**2. Unsafe.** None added. `engine` stays `#![forbid(unsafe_code)]`. The one reachable `unsafe` is the
existing `GetAsyncKeyState` in `HookEnv::os_key_down` (`keyboard_hook_callback.rs:116-122`). Its
SAFETY note still holds for the new callers, whose codes all come from `0..256`.

**3. Atomics.** None touched.

**4. Win32 / threads.** No hook, window, timer, COM, `SendInput` or install path changed. The new
`GetAsyncKeyState` calls run on the hook thread inside `with_engine`, like the existing resync.
`Engine::on_timer` never reaches them.

**5. Hot path.** The blocker branch allocates nothing. `all_held` is a filtered iterator over the 256-bool table,
and the closure is `combo.keys.iter().any(..)`. The scan runs only when the pressed key is one of the hotkey's
and all of its keys are held. `os_key_down` is asked only about held keys that no hotkey key accepts (the `||` short-circuits).
That is typically 0-2 calls, on top of the existing 11 resync calls. No I/O, sleep or spawn, and no action under the lock.

**6. Correctness.**
- Suppression symmetry: the record logic is untouched (`mod.rs:473-506`, `on_key_up` `:509-535`). A
  blocker-suppressed first key-down goes through `mark_blocked`, and a passed one through `mark_passed`.
  Repeats follow the record, and key-up suppression equals the record. Hot reload mid-press is covered by the record (RK8).
- Pressed-key lag: `blocker_matches` never asks about `pressed`. `is_trigger_key` succeeded first, so
  some hotkey key accepts it.
- Generic slots: the resync writes `VK_SHIFT`/`VK_CONTROL`/`VK_MENU` from the OS. A hotkey family accepts them
  through `AnySide`. A foreign family is asked about, and `GetAsyncKeyState` answers for either side,
  matching `FakeEnv` (`testing.rs:78-85`).
- AltGr+Shift: the fake LCtrl is down in Windows, so it is rejected, as in A2.
- Swallowed or forgotten keys read as up, so they are accepted. That is correct per D2.
- RK1 (every read fails): the resync clears the passed hotkey modifiers, so `keys_down` fails and the
  blocker fails open, as on master.
- Regressions for other kinds and triggers: only `Combo` bindings carry the flag. The `continue` skips
  only the blocker's own arm. Sequences, chords, Tap, Hold and DoubleTap are unchanged. User Immediate
  bindings that overlap a blocker now fire (D3, intended). RK5 (`block_default: false` overlap still
  swallowed) is documented.
- Dedup equality: `KeyCombination::eq` ignores order. The registry's `from_keys([VK_MENU, VK_SHIFT])` maps to
  `AnySide`, the same as the parsed `"Alt+Shift"`/`"Shift+Alt"`. The grave hotkey is `Key(VK_OEM_3)`, the same as the parsed `` "`" ``.
- Index in `[INFO]`: the loader is all-or-nothing on binding errors (`loader.rs:163-169`), so the index
  is the file index.
- Reachable blockers: only `SwitchLanguage`/`SwitchLanguageBackward` map to a hotkey
  (`windows_actions.rs:143-150`), so every blocker is all-modifier or the single grave key. The docs'
  phrase "the key just pressed is one of the hotkey's" is therefore exact for every reachable blocker.

**7. Style / build.**
- Doc comments are present on the new items. No commented-out code, no new `#[allow]`, no dependency.
- `VIRTUAL_KEY(code)` at `keys.rs:209` (critique M1). Ruling: not a principle-7 breach. The code is
  the inverse of `slot` for a table index that only ever held a code that already entered through the hook
  boundary or the resync constants. It is not a raw literal, and there is no failure.
- Test-only `_ => None` arms are in the hotkey stand-ins only.

**R1-R10.**
- R1: the adapter is untouched, so every event still ends in one outcome. Table indices are bounded `0..256`.
- R2: no new `SendInput` or sentinel change.
- R3: no new shared state. `KeyState` stays inside `Mutex<Engine>`, and `Env` is called under the
  existing lock with no pump.
- R4: no atomics.
- R5: no setter. `new_auto_blocker` is unchanged.
- R6: no new variant. The `[AUTO-BLOCK]` arm is intact. `[INFO]` uses `KeyCombination`'s `Display`.
- R7: `update_bindings` sort and single store are unchanged. Blocker position no longer affects any decision
  (D3), and the loader order change is intentional (D5). The rule text in `code-reviewer.md:124` is stale (Open questions).
- R8: `KeySpec::accepts` is applied to sided and generic codes in one form.
- R9: the read path is unchanged and read-only. New tests are registry-free.
- R10: no change.

**Docs.**
- `CHANGELOG.md`: Changed **Breaking** (:77-85), Fixed KS-05 partly (:96-106) and KS-18 (:156-169) are
  in Keep a Changelog sections and cite IDs.
- README `:428` and `default_config.ron:126-130` agree with the code. "Listed last at startup" is true:
  `print_welcome` prints the pre-sort list (`app.rs:56-62`).
- `book/src` is deferred to the phase book sync, which `docs/WORKFLOW.md:105` provides.

## Positive
- D2 is the right reading of "exact". It is decided by what Windows will see, it is robust to a later
  hook swallowing a passed key, and it is immune to stale engine-held keys (T9) and swallowed keys (T10/M3 pin).
  It needs only a few extra `GetAsyncKeyState` calls, all after the cheap checks have passed.
- D3 removes the order dependence at the source instead of patching the sort. The flips F1-F3 make
  the change visible in pinned scenarios.
- The D6 seam makes every new loader and scenario test machine-independent.
- The tests pin the boundary cases precisely: a generic slot (U3, U2), the last table slot (3d65ac9),
  and both file orders (T1, T7).

## Open questions
1. **Acceptance 1 and cut Q1 are still unanswered by the orchestrator.** "Ctrl+Shift+T passes through"
   holds only when T is down before Shift (T3). In the usual order Ctrl, Shift, T the browser gets
   Ctrl+T (T6), and the usual Alt+Tab-back gesture still moves forward (M4 pin). Exact matching
   decides at Shift's key-down, before T exists, so it cannot do better. `roadmap.md:100-101` assigns the
   rest to 1.2. This settles it if the orchestrator records acceptance with that carve-out at merge.
2. **Cut Q2 (D2 vs. the literal "held set exactly equals").** I endorse D2 (see Positive). The orchestrator
   should record the ruling, since the spec text reads literally otherwise.
3. **impl.md:82 is inaccurate, but no action is needed.** With registry hotkey "3", `parse_language_hotkey` returns `None`,
   `read_from_registry` tries the next value name, and then `get_system_hotkey` falls back to Alt+Shift
   (`registry.rs:52-71`, `:118-124`). The configs of `parse_simple_binding` and
   `auto_blocker_deduped_when_two_bindings_want_the_same_combo` use CapsLock/F13, which no hotkey equals. So
   these registry-reading tests cannot fail on any of the values 1-4. They do read HKCU (allowed), so
   moving them to `from_ron_str_with_hotkeys` stays optional hygiene.
4. **Stale agent rule texts** (`.claude/agents/code-reviewer.md:124`, `project-analyst.md:60`: "auto-blockers rely on load
   order among equal counts") belong to the orchestrator after merge (cut Q6), together with the audit
   status lines for KS-05 (partly) and KS-18.
5. **Pre-existing, 1.2.** After the blocker swallowed Shift and Alt was released, pressing Alt again with
   Shift still held is swallowed too, although Windows has only Alt down. The hotkey's own keys are
   counted from the engine table, not from Windows, as on master. This is critic open question 2 and not
   a regression.
