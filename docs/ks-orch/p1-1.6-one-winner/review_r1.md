# Code review: p1-1.6-one-winner @ f7d54ef
APPROVED (Critical 0, Important 0, Minor 3)

Branch `feat/one-winner` (6 commits, `master` = merge-base `9357cfa`). I read the code through `git show` and `git diff master...feat/one-winner`. I read all of `src/engine/winner.rs` and all of `on_key_down`/`on_key_up` at head. I also read, unchanged at head, `engine/{keys,gestures,matching}.rs`, `hook/keyboard_hook_callback.rs:60-270`, `data/{key_combination,condition}.rs` and `config/loader.rs:150-330`, plus every hunk of the diff (code, tests, README, CHANGELOG, template, the 7 book pages). Inputs: `pack_r1.md`, `cut.md` (incl. the developer addendum), `impl.md`, `critique.md`, `red1.log`, `red4.log`.

## Gate: fmt ✅ · clippy ✅

I ran both on a clean `git archive f7d54ef` export, not on the shared work tree. It sits in the session scratchpad, with its own `CARGO_TARGET_DIR`. Its file list is identical to `git ls-tree -r f7d54ef`.
- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0. I re-ran it after touching `src/lib.rs`, `src/main.rs` and `src/bin/swch.rs`, to force a real re-check: `Checking key-switch-rs v0.1.0 ... Finished`, no warnings.
- I did not run `cargo test` (the tester owns it).

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor

#### m1. `elect`'s doc describes a different winner than the code picks (R7)
**Where**: `src/engine/winner.rs:48-49`  **Confidence**: CONFIRMED (doc vs code)
**Problem**: "The winner is the first candidate in list order whose rank is strictly greater than that of every earlier candidate." Read literally, this is always the first candidate: nothing comes before it, so the condition holds trivially. The loop (`:60-80`) does something else. It replaces the winner only when a later candidate's rank is strictly greater (`rank(w.binding) >= rank(binding)` → skip), so it returns the *first candidate of the highest rank*.
**Failure**: `[(keys: "F13", action: SwitchLanguage), (keys: "F13", action: ToggleCapsLock, when: AppEquals("code.exe"))]` in VS Code. By the doc sentence the unconditional first candidate wins. The code (and W3) picks the conditional one. Someone who writes a test or a refactor from the doc gets the wrong rule.
**What to do**: say "the first candidate in list order with the highest rank". A later candidate replaces the winner only if its rank is strictly higher. cut.md D2 uses the same wording.

#### m2. `sort_by_specificity` doc says the auto-blockers stay after the file's bindings; across key counts they do not (R7, KS-18)
**Where**: `src/engine/mod.rs:283-285`  **Confidence**: CONFIRMED (by reading `expand_with_auto_blockers` + `sort_by_key(Reverse(key_count))`)
**Problem**: "...keep their config order, and the auto-blockers the loader appends stay after the file's bindings." The sort is by key count only. So a 2-key blocker moves ahead of every 1-key binding of the file.
**Failure**: take the default config (`CapsLock` with `block_original_combo: true`, then `Shift+CapsLock`) with the Alt+Shift hotkey. After the sort the list is `[Shift+CapsLock, Alt+Shift blocker, CapsLock]`, so the blocker is visited before the file's `CapsLock`. No code relies on the claim today: `elect` skips blockers (`winner.rs:64-66`), and the blocker branch only ORs into `suppress` (`mod.rs:405-422`). A later change that trusts the comment ("blockers are last, so a first-match rule is safe") would break KS-18.
**What to do**: limit the clause to equal key counts ("among bindings with the same key count, the auto-blockers stay after the file's"), or drop it. cut.md D2 makes the same claim ("the auto-blockers stay last").

#### m3. `parse_ron_str` is documented as the non-printing half, but it still prints
**Where**: `src/config/loader.rs:242-250` (docs), `:286-288` (`eprintln!("[WARN] config contains no bindings")`)  **Confidence**: CONFIRMED
**Problem**: `ParsedConfig` is "the warnings to show about them", and `parse_ron_str` is "[`from_ron_str_with_hotkeys`] without the printing". Yet the empty-config `[WARN]` is still printed inside `parse_ron_str`, and so are `expand_with_auto_blockers`' `[INFO]` lines. It never goes into `warnings`. That matches cut D10 (a pure refactor in commit 3), but the new doc says otherwise.
**Failure**: when 1.7 sends `parsed.warnings` to `log::warn!` per this seam, `config contains no bindings` stays an `eprintln!` and never reaches the log file. That warning matters most when the daemon runs from Task Scheduler with no console. A test that uses `warnings_of` on `bindings: []` gets an empty `Vec` and stray stderr output.
**What to do**: either move the empty-config warning into `warnings` (and fix the doc), or reword the doc ("without printing the duplicate warnings"). Then 1.7's cut can see the second path.

## Checklist walked (absent, no reachable failure)

- **§1 Plan conformance.** I checked D1-D11 against the code:
  - D1: Combo user bindings only.
  - D2: `rank = (key_count, !is_always)`, `winner.rs:41-43`.
  - D3: `admits`, `:93-114`: combo `==`, condition `==`, own-flag `combo_matches`.
  - D4: `GroupArms::claim` has an exhaustive match, so a new `Trigger` variant forces an arm.
  - D5: OR inside the group only.
  - D6: re-election per repeat.
  - D7: rank, then keys, then `when:`.
  - D8: module placement.
  - D9/D10: predicates and seam.
  - The deviations are all declared in `impl.md`.

  All five acceptance lines are pinned by tests that were red on master (`red1.log:339-353`: 12 failures; `red4.log:106-109`: 2 failures), or by guards that kill named mutations. I traced W8, W9, W10, W13 and W14 against the corresponding mutation by hand. CHANGELOG entries sit under Added / Changed (Breaking) / Fixed in `[Unreleased]` and cite 1.6 / KS-13. README, template and book pages were updated in the commits that change meaning. Every commit carries the Opus trailer.
- **Group/winner soundness.** Can a group member sit before the winner, so that the winner loses its `GroupArms` claim? No. A member has the winner's rank (same combo → same `key_count`, same condition → same `is_always`). Its condition is true on the same event (`HookEnv` caches the foreground in a `OnceCell`, `keyboard_hook_callback.rs:103-118`). Its own match does not change during the loop: `mark_completed` touches only `last_down`, never `active` (`keys.rs:168-176`), and `HeldKeys` reads only `active`. So an earlier matching member would itself have been elected. The winner is always the group's first visit and always claims its kind.
- **§2 Unsafe.** No `unsafe` added or newly reachable. The adapter is not in the diff. The engine keeps `#![forbid(unsafe_code)]`.
- **§3/R4 Atomics.** None added.
- **§4 Win32 contract and thread affinity.** No new API, thread, timer, window or COM use. Install and uninstall are untouched. The duplicate warnings print on main (`config::load`) or `config-watcher`, never on the hook thread.
- **§5 Hot path.**
  - `elect` makes one extra pass over the list per key-down, with no allocation, lock or I/O. `Winner` is a borrow; `GroupArms` is three bools.
  - `admits` costs a few `KeySpec`/`Condition` comparisons and does no allocation.
  - Foreground lookups are strictly fewer than on master: master evaluated every binding's `when:` on every key-down. W16 pins this.
  - The only per-event clones (`action.clone()`, `arm_*`) predate the lane.
- **§6 Correctness.**
  - Suppression is monotone. Every key-down suppressed after the lane would also have been suppressed on master: the winner and its group matched there too, and blockers, sequences and chords are unchanged. The `LastDown` record (`mod.rs:536-569`) and `on_key_up` (`:572-598`) have no hunks.
  - So key-up iff key-down still holds by the record, and no new stuck key or lone-Win/Alt swallow is possible.
  - `vk >= 256` → `slot` returns `None` → no match.
  - Repeats follow the record. The M1 case is documented and pinned.
  - Exact modifiers and the trigger-key rule are reused unchanged (`combo_matches`).
  - Indices: `winner.index` and the act loop enumerate the same `bindings` slice.
- **§7 Style.** The new public query methods have docs. No `#[allow]` and no non-ASCII in the added lines. No new dependency. The new `_ => false` arms (`admits`, `same_keys`) are predicates, not `Display`/mapping arms.
- **R1.** Callback unchanged. Each path still ends in exactly one `LRESULT(1)` or `CallNextHookEx`.
- **R2.** No new `SendInput`. Fewer actions fire.
- **R3.** No new shared state. The engine lock is used only through `with_engine`.
- **R5.** No new field or setter. Constructors unchanged.
- **R6.** No new variant. `Condition` gains derives only.
- **R7.** The sort and the single store are unchanged. The plan replaces list-order precedence with `rank` explicitly. Ties still go by stable file order. Blockers are not candidates (W12, `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order`). Only the doc nit m2 remains.
- **R8.** Group and duplicate identity use `KeySpec` equality (`LShift` ≠ `Shift`; documented, known-issues row 71). Chord containment is sound because the loader rejects overlapping chord keys (`parsing.rs:514-518`).
- **R9.** No registry write. Loader tests use `&|_| None`.
- **R10.** Unchanged.
- **Loader.** `raw.bindings[k]` ↔ `user_bindings[k]` is guaranteed, because errors return before `duplicate_warnings` (`loader.rs:282-292`). `duplicate_warnings` is O(n²) at load time only.

## Positive

- The rank is explicit and independent of the sort. Sequence, chord and blocker visit order and their same-step behaviour stay exactly as before.
- Condition evaluation now runs only after the keys match (D7), and W16 proves it with an `Env` that panics if the foreground is read. That is a real cut in foreground lookups for the KS-08 budget.
- The mutation coverage is unusually complete. Each piece of the group rule has a killer test: keys W10, `when:` W8, own-flag match W13, `GroupArms` W9, tie direction W14 / `non_repeating_winner_...`, condition rank W3. Every scenario runs with `check_press_symmetry` and `check_nothing_stuck`, and the `winners` property config adds random scripts over competing bindings.
- The developer handled all three critique Minors, including the M1 repeat behaviour change, which is now pinned and documented in four places.

## Open questions

- `book/src/audit.md:36,394` and `book/src/roadmap.md:283-289,617` still say KS-13 / 1.6 are open. The cut defers them to the doc-writer's Phase-1 book sync, which is fine for this lane. The merge step or that sync must not forget them; fact that settles it: the sync's touch list.
