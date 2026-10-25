# Code review: p0-A1-engine @ d3131fc
APPROVED (Critical 0, Important 0, Minor 1)

Ref: `refactor/engine` = `d3131fc`, `master` = merge-base = `73bbf40`. Every touched file read in full through
`git show refactor/engine:<path>` and compared with `git show master:<path>` for the ported logic
(`keyboard_hook_callback.rs`, `tap_state.rs`, `chord_state.rs`, `sequence_state.rs`). Plan: `cut.md` D0-D11 plus the
developer addendum (D6', I2, M1-M3); `impl.md`; `critique.md`; context pack `pack_r1.md`.

## Gate: fmt OK · clippy OK

Run on a `git archive d3131fc` export in the reviewer's scratchpad (not the tester's `wt/mut`, not the shared tree):

- `cargo fmt --check`: exit 0.
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0 (re-run after touching `src/lib.rs`/`src/main.rs`, so the
  crate itself was re-checked: `Checking key-switch-rs v0.1.0 ... Finished`).
- `cargo test` not run (tester owns it).
- Spec grep `rg 'GetAsyncKeyState|GetForegroundWindow|SendInput|Mutex|thread::' src/engine`: one hit,
  `src/engine/mod.rs:73`, a `//!` line. Acceptance holds.

## Remarks

### Critical
None.

### Important
None.

### Minor

#### m1. A blank line splits the `### Fixed` list in CHANGELOG
**Where**: `CHANGELOG.md:68` (between the new KS-33 item ending at `:67` and the existing KS-38 item at `:69`).
**Confidence**: CONFIRMED.
**Problem**: the three new Fixed entries were inserted above KS-38 with a trailing empty line:
```
- KS-33 (partly; an unverified report): ... that Hold then does not fire.

- KS-38: `cargo test --release` ...
```
**Failure**: in CommonMark a blank line between items makes the whole `### Fixed` list loose, so every item there,
including KS-38 and KS-42, is wrapped in its own `<p>` with paragraph spacing. The other sections of `[Unreleased]`
stay tight.
**What to do**: drop the empty line at `:68`.

## Checklist walked (no reachable failure found)

**Plan conformance**
- D1 API: `Engine::{new,on_key,on_timer,forget_timer,on_foreground_changed}`, `KeyEvent`, opaque `TimerToken`,
  `TimerRequest{token,due}`, `#[must_use] Decision{suppress,fire,timers,mask}` (`src/engine/mod.rs:153-287`). The
  extra `forget_timer` is a justified deviation (addendum D5).
- D2: `engine: Mutex<Engine>` reached only through `with_engine` (`src/hook/keyboard_hook.rs:51,79-88`). Poison is
  recovered and `DecidingGuard` is declared after the `MutexGuard`, so the flag clears before the unlock. The re-entry
  test is bounded by `recv_timeout`. This is the Mutex the spec and cut Q1 allow, so it is not an R3 remark.
- D3: the sentinel check (`keyboard_hook_callback.rs:156`) runs before `key_event`, the `ArcSwap` load and the lock.
  The binding guard and `HookEnv` are locals of `decide_event` and are dropped before `carry_out` (`:231-233`).
- D4: the clock is read once per event inside the lock (`mod.rs:290,416`). `sleep_until` loops until
  `Instant::now() >= due` before locking (`:205-213`). I traced both lock orders for Tap(n)+Hold(n) and exactly one
  fires in each.
- D5 and D6': only `on_timer` fires a Hold (`mod.rs:266-272`), and the key-up `Decision::fire` holds at most the Tap
  action (`:424-429`). Hold actions therefore run only on `hold-timer`, as on master (`M:tap_state.rs:105-116`). A
  spawn error is logged with `eprintln!` and `forget_timer` is called, with no `expect` (`:196-199`).
- D8: `#![forbid(unsafe_code)]` at `mod.rs:122`. D9: the three files are deleted and their 17 tests are ported under
  the same names. I checked `chords` and `sequences` by name against master. D10: module docs exist for the engine,
  `env`, `gestures`, `testing` and the adapter.
- Acceptance: scenario tests cover Immediate combo, Tap, Hold, DoubleTap, Sequence, Chord, `when:` and
  `block_default: false` (`src/engine/scenarios.rs`). The CHANGELOG Fixed entries KS-11, KS-25 (equal terms) and KS-33
  (partly) each have tests. The README layout, module bullets and "How resolution works" match the code.
  `git grep tap_state|chord_state|sequence_state|tap-gc` outside `book/` and `.claude/` finds only `CLAUDE.md:21-22`
  (see Open questions).

**Behaviour equivalence (ported line by line against master)**
- Key-down binding loop (`mod.rs:289-413` vs `M:keyboard_hook_callback.rs:186-318`) is identical:
  - same order: `when:`, then kind;
  - same first-Immediate-fires rule and same `block_default` handling;
  - Tap/Hold/DoubleTap skip repeats and always suppress;
  - Sequence and Chord outcomes match;
  - `mark_blocked` for the event key on suppress, and for the chord keys and last-step keys on completion.
- Immediate, chord, sequence and DoubleTap actions now run in `carry_out` after the loop rather than inside it. They
  run on the same thread and in the same order. I checked every `BindAction::execute` arm
  (`core/windows_actions.rs`): none synchronously changes the engine's view (`SendInput` output comes back as
  sentinel-filtered events after the callback returns). Master's per-key-down `ForegroundContext` snapshot and the
  branch's lazy `OnceCell` capture give the same results.
- Key-up (`mod.rs:415-440` vs `M:handle_key_up`): suppression matches master for all of these cases:
  - Tap fired;
  - Tap expired: blocked flag;
  - Hold cancelled;
  - Hold already fired by its timer: `NotPending` -> blocked flag;
  - foreground change in between.
  The only new case is `Overdue`, which is suppressed. Its key-down was suppressed, because Hold always suppresses.
- Chords and sequences: `chords.rs:58-106` and `sequences.rs:57-116` match `M:chord_state.rs:61-123` and
  `M:sequence_state.rs:55-124`. Two differences, neither observable: the membership check now runs before the
  canonical copy, and `now` is read once per event instead of once per binding.
- `MODIFIERS`, `normalize`, `ActiveKeys` cap 16, and `sync_modifiers` with `skip = event.vk` are the same as
  `M:keyboard_hook_callback.rs:34-100,345-365`. Master also ran the resync for non-key `wParam` values, but a
  `WH_KEYBOARD_LL` hook only receives the four key messages, so nothing changes.
- Edge inputs:
  - `term = 0`: Tap never fires; Hold fires at once from its thread, as on master.
  - `term = u64::MAX`: `Instant + Duration` cannot overflow on Windows (secs ≈ 1.8e16), so there is no panic.
  - `vk >= 256`: every index goes through `slot()`, `keys.rs:76-79`.
  - Empty sequence returns `NotMatching`.
  - A 1-step sequence completes at once.
  - The `released` Hold entries are always removed by their timer or by `forget_timer`, so the table stays bounded.

**Suppression invariant**
- A key-up is suppressed only through a live Tap, a non-released Hold or a blocked flag. Each of those is created only
  by a suppressed key-down of the same key. A `released` entry is excluded from `pending_hold_index`, so it never
  answers a later key-up.
- Pre-existing and unchanged asymmetries:
  - KS-06 repeat leaks;
  - foreground change clearing blocked flags (pinned by `foreground_change_forgets_pending_key_up_suppression`; 2.3);
  - `swch off`/`on` stale `active` (KS-31);
  - re-entry refusal of a key-up (documented at `keyboard_hook_callback.rs:47-60`; no re-entry path exists today).

**Unsafe**
- Four `unsafe` blocks in the adapter, each with a concrete `// SAFETY:` comment:
  - `keyboard_hook_callback.rs:120` (`GetAsyncKeyState`, by-value argument);
  - `:224` and `:241` (`CallNextHookEx` with the system's arguments);
  - `:230` (`KBDLLHOOKSTRUCT` deref for `HC_ACTION`, two `Copy` fields read).
- The `keyboard_hook.rs` unsafe blocks are unchanged. The engine has none (`forbid`).
- No panic path is reachable from either `extern "system"` fn:
  - `key_event` uses `try_from(...).ok()?`;
  - every engine index is guarded (`chords.rs:81`, `sequences.rs:75-77`, `gestures.rs:102,162,166` index by a
    `position()` result);
  - the lock uses `unwrap_or_else(into_inner)`.

**Atomics, R4**
- No new atomics. The `active_keys`/`blocked_keys` atomics are gone.
- `DECIDING` is a const-initialised `thread_local!` `Cell<bool>` with no destructor, so `with` cannot panic on thread
  exit.
- The remaining orderings are on `hook_handle`/`foreground_hook` and are unchanged.

**Win32 and threads**
- Install and uninstall did not move.
- No new handle, window, COM or `SetTimer`. `hold-timer` threads use only `get_hook()` and the action, as on master.
- The lock is uncontended unless Hold bindings exist. The `hold-timer` critical section is `fire_hold` only, which is
  shorter than master's hold/tap/double-tap mutex sections that the hook thread also took.
- Nothing under the lock retrieves messages:
  - `ForegroundContext::capture` uses `GetForegroundWindow`, `GetWindowThreadProcessId`, `OpenProcess` and
    `GetModuleBaseNameW`;
  - `GetWindowTextW` targets a window the daemon does not own.

**Hot path**
- Actions run outside the lock, and the `ArcSwap` guard is released before them, which is better than master (master
  held the guard across `execute`).
- New allocations per Immediate fire: `binding.action.clone()` plus the `Vec` push at `mod.rs:313`. Cut D11 prescribes
  them and leaves zero allocation to 3.4. The README line `README.md:528` "Hot-path allocations: none" was already
  false on master and is left to 3.4 by the cut (Docs impact). Not a remark.
- `"hold-timer".into()` is the same as master. The spawn-failure `eprintln!` is a failure path, and it replaces the
  former abort (KS-33).

**R2 sentinel**
- No new `SendInput` builder.
- The injected-event filter precedes all state, and the test `injected_and_unknown_events_bypass_the_engine` fails if
  it moves after the engine.

**R3, R5-R10**
- No `static mut`/`RefCell` in shared state. No new builders or variants, so no `Display` gaps.
- `update_bindings` keeps the stable sort through `engine::sort_by_specificity` (`mod.rs:222-224`), and
  `equal_specificity_keeps_config_order` stays.
- Folding is unchanged; chords still use raw codes (KS-07, out of scope).
- No registry or process-read change.
- Shutdown: a late `hold-timer` firing after uninstall is unchanged (KS-31, 2.3).

**Tests that can fail**
- `physical_key_events_reach_the_engine` fails if any of the four messages is unmapped.
- `injected_and_unknown_events_bypass_the_engine` fails if the sentinel is checked after the engine.
- `with_engine_refuses_reentry_on_the_same_thread` reports a broken guard as `Timeout`.
- The D6' scenarios use `advance_ms_stalled`, and `hold_released_before_its_term_with_a_late_timer_is_cancelled`
  catches an "always fire at release" mutation.
- No test installs a hook, runs `carry_out`, calls `.execute()` or injects input. `HookEnv` only reads
  `GetAsyncKeyState`, and the F13/A assertions do not depend on the owner's held modifiers.
- No `#[ignore]` is needed.

**Principle 7 (VIRTUAL_KEY construction)**
- `keys.rs:136` builds `VIRTUAL_KEY(code as u16)` from the bounded `0..256` table index inside the engine (moved from
  the master callback). The value is lossless and cannot fail.

**Not verified by this review**
- The adapter's spawn-failure branch (`keyboard_hook_callback.rs:196-199`) and the `carry_out` order (timers, then
  actions) are proven by reading only. No automated fault injection exists, as cut D5 and KS-33 accepted.
- `LRESULT(1)` iff `suppress` at the `extern` level is review-only (critique open question 1).

## Positive

- D6' is the right answer to critique I1. Hold resolution is a pure function of one clock read under one lock:
  key-up before `due` cancels, at or after `due` leaves the Hold to its timer, and only `on_timer` fires. That keeps
  Hold actions off the hook thread and makes equal-term Tap/Hold exclusive. The `released` flag also avoids blocking
  the next press (`pending_hold_index`), and two tests pin that.
- The generation token plus per-entry deadlines removes the KS-11 class by construction, not by timing.
- `with_engine` plus the thread-local guard turns any future message-pumping re-entry into a pass instead of a
  deadlock. The module docs state the residual key-up asymmetry honestly.
- The scenario harness models the OS key state correctly: only passed events change it, and the generic modifier is
  derived from its sides. Failure messages carry a full transcript, and the harness self-tests prove its assertions
  fire.
- The adapter is small and every return path is visible. `carry_out` is shared by the hook and timer threads.

## Open questions

1. `CLAUDE.md:21-26` (and `.claude/agents/architect.md`, per pack Fact 45) still list `tap_state.rs`,
   `chord_state.rs` and `sequence_state.rs` and say `src/engine/` does not exist. Principle 5 still forbids a
   "hook-wide Mutex" without the 2.2 exception. Cut Q5/Q1 assign this to the orchestrator after the merge. Not counted
   as a remark. It should land before the next Workstream A lane's architect reads `CLAUDE.md`, or that cut will plan
   against the old layout.
2. `foreground_change_forgets_pending_key_up_suppression` (`scenarios.rs:171-180`) pins a key-up that passes after its
   key-down was suppressed (audit line 251, item 2.3). The module doc at `scenarios.rs:3-5` says known defects are not
   pinned. The cut and critique accepted this as today's deliberate repair. Settled if 2.3's cut records that it must
   rewrite this scenario.
