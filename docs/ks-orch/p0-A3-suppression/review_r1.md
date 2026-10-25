# Code review: p0-A3-suppression @ 5d0fada
APPROVED (Critical 0, Important 0, Minor 2)

Range reviewed: `master` `82f542f` ... `fix/suppression-symmetry`. The context pack was written at `15e901d`. While this review
ran, the branch gained `5d0fada` ("Tests: a suppressed press spends its record, a later key-up of an unseen press passes
(KS-06)", tester, test-only, +23 lines in `src/engine/scenarios.rs`). It is reviewed here too. All code was read through
`git show` / `git diff` at those refs, never from the work tree.

## Gate: fmt ✅ · clippy ✅

- Both runs were on `git archive` exports of `15e901d` and of `5d0fada` into the session scratchpad, with a separate
  `CARGO_TARGET_DIR`.
- `cargo fmt --check`: exit 0 at both refs.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0 at both refs, no warnings. At `15e901d` it was re-run after
  touching `src/lib.rs` and `src/engine/mod.rs`, and printed a fresh `Checking key-switch-rs`.
- `cargo test` was not run: the tester owns it.

## Remarks

### 🔴 Critical
None.

### 🟡 Important
None.

### 🟢 Minor

#### m1. README still promises a full focus-transition repair for modifiers (KS-14 trade-off RK1)
**Where**: `README.md:593` ("Focus-transition repair ... so phantom-held modifiers do not leak across the transition") and
`README.md:609-612` (Known Limitations, secure desktop: "resyncs modifier state when control returns from the secure desktop,
so phantom-held modifiers do not leak across the transition"). Code: `src/engine/keys.rs:91` (skip), `src/engine/mod.rs:325-326`.
**Confidence**: CONFIRMED (traced).

**Problem**: D2 deliberately stops resyncing a modifier whose key-down the engine swallowed, also after a foreground change.
The cut's RK1/Q3 accepts this, and `Engine::on_foreground_changed` documents it (`mod.rs:319-323`). The two README claims
were not qualified, and the CHANGELOG KS-14 entry (`CHANGELOG.md:123-135`) does not mention the cost either.

**Failure** (default config: blocker + `DEFAULT_PAIR`):
1. LAlt down passes (record Passed).
2. LShift down: the blocker matches and swallows it (record Suppressed).
3. A secure desktop (UAC consent, Ctrl+Alt+Del) takes focus. LShift and LAlt are released there, so both key-ups are lost.
4. Focus returns. `on_foreground_changed` resyncs LAlt to up, skips LShift (`keys.rs:91`) and turns it into Forgotten.
5. CapsLock down: LShift still counts as held, so `"CapsLock"` fails exact matching and `"Shift+CapsLock"` fires
   `ToggleCapsLock` instead of `SwitchLanguage`.

This goes on until Shift is pressed and released once, and that press is swallowed: it reads as a repeat (`is_blocked`
counts Forgotten). On `master`, step 4 cleared LShift and CapsLock switched the language. The route is rare: a swallowed
modifier has to be held at the moment a secure desktop appears.

**What to do**: add one sentence to the README limitation and the feature bullet (or the KS-14 CHANGELOG entry). It should
say that a modifier whose key-down was swallowed is not repaired, and that roadmap item 2.3 owns the repair. Optionally,
pin the accepted behaviour with a scenario next to `modifier_lost_by_the_os_is_resynced_before_matching`
(`scenarios.rs:178`), which pins only the passed-modifier repair. Then 2.3 changes the behaviour on purpose, not
unnoticed.

#### m2. Two touched test names now state the policy this lane reversed
**Where**: `src/engine/scenarios.rs:1474` `press_after_a_lost_key_up_and_a_focus_change_follows_its_binding`, and
`src/engine/keys.rs:298` `forget_blocked_resets_every_flag`.
**Confidence**: CONFIRMED (read).

**Problem**:
- The lane rewrote the first test's comment: the press is now swallowed because its record says so, not because the binding
  matches (I1 b). The name still says "follows its binding".
- After `forget_blocked`, `is_blocked` is still true for a Forgotten key (`keys.rs:154-156`). The second name says the
  flags are reset, which is now true only for the key-up side (`take_blocked`).

**Failure**: none in behaviour. A reader working on 2.3 who goes by these names infers master's focus-change policy, not
this lane's.

**What to do**: rename both to the rule they now pin, for example "is swallowed by its record" and "forgets the key-up but
not the swallowed key-down".

## Invariant trace (why no 🔴/🟡 on the suppression invariant)

Per key, `last_down` is written by four kinds of code:
- `mark_blocked` / `mark_passed`: only for the event key, at the end of `on_key_down` (`mod.rs:471-485`);
- `mark_completed`: for other keys, never over `Passed`, only while held (`keys.rs:134-141`);
- `take_blocked`: at its own key-up, always back to `Unknown` (`keys.rs:164-172`);
- `forget_blocked`: on a focus change only.

**Without a focus change or a lost event:**
- A first key-down records Suppressed or Passed.
- No other path can turn Passed into Suppressed. A record changes from Suppressed only through `take_blocked`, at the key's
  own key-up, or through `forget_blocked`.
- Repeats then follow the record (`mod.rs:471-474`). The key-up suppresses on Suppressed and passes on Passed
  (`mod.rs:510-513`: `was_blocked` is false for Passed, and `!was_passed` masks the gesture override).
- So "key-up suppressed iff key-down suppressed" holds for every kind and trigger. A fresh Unknown repeat of a non-modifier
  is unreachable: `press` sets `active` and the tail always records.

**The D2 guard keeps "Suppressed/Forgotten implies held":**
- `release` is always paired with `take_blocked` (`mod.rs:491`, `:511`).
- The resync never writes `active` for those two records (`keys.rs:91`).
- `mark_completed` marks only held keys.
- So the skip can never leave a released key frozen as held.

**Checked against the code:**
- I1(b): a Forgotten repeat is swallowed (`keys.rs:154-156`).
- D3: an Unknown repeat that passes records Passed (`mod.rs:475-476`).
- The tester's `5d0fada` scenario: the swallowed key-up spends the record, so an unseen later press's key-up passes.

## Checklist walked (absent items, no reachable failure)

**Section 1, conformance:**
- D1-D7 are implemented. D1 is amended by I1(b), with the developer addendum (`cut.md:417-459`).
- Pinned expectations are unchanged except T5, which replaces the deleted A2 test as the cut planned. The other three
  scenario hunks are comment-only.
- The red-first commits are test-only (`20d4844`, `72ea8d6`, `afdb60c`).
- All three acceptance lines map to tests:
  - T5 `scenarios.rs:1649` with `check_nothing_stuck`;
  - `5d0fada` `key_up_of_a_suppressed_press_is_suppressed_only_once`;
  - `properties.rs:229`, `:234`.
- The CHANGELOG entries sit under Unreleased/Fixed with KS IDs. The book is deferred to the phase sync (cut D8/Q4).

**Section 2, unsafe:**
- No `unsafe` added. `#![forbid(unsafe_code)]` stays (`mod.rs:148`).
- The two hook-file edits are comments. The `// SAFETY:` block at `keyboard_hook.rs:134-136` is intact and still sits
  directly above `SetWinEventHook`.

**Section 3, atomics:** no atomic, lock or `ArcSwap` use added. `grep` of the added lines finds no `Ordering::`.

**Section 4, Win32 and threads:** no new API, hook, window, timer, COM or thread. `GetAsyncKeyState` is now called less
often: it is skipped for swallowed modifiers.

**Section 5, hot path:**
- `sync_modifiers` adds one array read per modifier and no allocation. `mark_completed` allocates nothing.
- Every added `Vec`/`format!`/`collect` is in `#[cfg(test)]` code (`testing.rs`, `properties.rs`, `scenarios.rs`).

**Section 6, correctness:**
- Indexing for `vk >= 256` goes through `slot()` (`keys.rs:37-40`). `MODIFIERS` are all below 256.
- Chord and sequence completion marks are guarded (KS-30, KS-32).
- Records survive a reload mid-press (RK9; intended).
- `term_ms`, empty sequence and one-key chord paths are unchanged.

**Section 7, style:**
- Docs are consistent with the code (`mod.rs:95-141`, `keys.rs`, `env.rs:32-41`, `chords.rs`, `sequences.rs`).
- `LastDown` stays private. No `#[allow]` and no new dependency.
- Let-chains and exclusive range patterns are fine at MSRV 1.88 on edition 2024.
- Commit trailers are present on all 10 commits (`5d0fada` carries the tester's Sonnet trailer).

**R1-R10:**
- R1: the adapter is unchanged. `code < 0` and the sentinel check come before the engine call
  (`keyboard_hook_callback.rs:220`, `:156`), and every path still ends in `LRESULT(1)` or `CallNextHookEx`.
- R2: no `SendInput` builder added.
- R3: `KeyState` stays inside `Mutex<Engine>`. No static, `Cell` or `RefCell` added.
- R4: no atomics.
- R5: harness methods consume `self`. No `Binding` API change.
- R6: no public variant. `LastDown` has no `Display` and needs none.
- R7: `with_alt_shift_blocker` puts the blocker first, like the loader, and `Scenario::with_bindings` sorts. No second
  publish path.
- R8: records are kept per sided code. Generic slots are resynced from the OS. Chords mark the sided `done.seen` codes,
  sequences the non-modifier codes.
- R9: tests build the blocker directly. There is no registry access.
- R10: the message loop and shutdown are untouched.

## Positive

- The red-first discipline is real: each fix commit is preceded by a test-only commit with predicted failure text.
- The harness gains whole-press symmetry and "nothing stuck" oracles, each with its own failing self-test (M3 of the
  critique).
- The property tests are deterministic (SplitMix64, fixed seeds). They cover six configs across every kind and trigger, and
  the reported mutation runs show they can fail.
- D2 makes the hook path cheaper, not dearer.
- The D4 rule is defensive (`was_passed` is read before `take_blocked`). It is honestly documented as unreachable today
  and pinned by a direct unit test.

## Open questions

1. **Roadmap item KS-01 / 2.3 interaction.** Workstream A's "on/off as an `AtomicBool`" keeps the hook installed but hides
   events from the engine while off. A swallowed modifier held across `swch off` / `swch on` would then stay phantom-held,
   because D2 no longer repairs it. This is the m1 route via off/on instead of the secure desktop. What settles it: the
   KS-01 or 2.3 cut must reset or resync the engine on `on` (`reset_transient_state`).
2. **CHANGELOG section for RK2** (Shift after the Alt+Shift blocker now stays swallowed for its whole press). It sits under
   Fixed/KS-14 (`CHANGELOG.md:131-135`). The cut's Q5 marks this as the orchestrator's call, set against the CHANGELOG's
   "Breaking under Changed" convention (`CHANGELOG.md:7-9`). What settles it: whether a released version ever had Shift's
   repeats reach Windows without the key getting stuck (pre-engine master stuck it, KS-06).
3. **Deviation 6 / RK7 class** (an `Unknown` held modifier marked by a chord completion; a non-modifier held since before
   the hook). Both are reachable only with a lost key-down and are deferred to 2.3. No action in this lane.
