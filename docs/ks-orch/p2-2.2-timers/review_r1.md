# Code review: p2-2.2-timers @ d738632
APPROVED (Critical 0, Important 0, Minor 2)

Branch `feat/main-thread-timers`, 18 commits `611c83f..d738632` on `master` `c9c1373`. I read the code through
`git show`/`git diff` at `d738632`. The work tree `D:/key-switch-rs/key-switch-rs` was clean on `d738632`
before and after the gate run.

## Gate: fmt ✅ · clippy ✅

- `cargo fmt --check`: exit 0 (`D:/tmp/ks-orch/p2-2.2-timers/review_r1_fmt.txt`).
- `cargo clippy --all-targets --locked --target-dir D:/tmp/ks-orch/p2-2.2-timers/review-target -- -D warnings`:
  exit 0, `Finished dev profile` (`review_r1_clippy.txt`). I used a separate target dir so I did not take the tester's
  build lock. I did not run `cargo test`, which is the tester's job.
- Every commit ends with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. `git diff --check` is clean, and
  no added `src` line has a non-ASCII character.

## Remarks

### 🔴 Critical

None.

### 🟡 Important

None.

### 🟢 Minor

#### m1. The successor of the fast-gate test does not pin the keyboard callback's own gate (R1, spec "keep the remapping-off fast gate")
**Where**: `src/hook/keyboard_hook_callback.rs:326-331` (gate), `:196-198` (second check in `decide_event`),
`:553-556` (comment), test `:797-854`. **Confidence**: CONFIRMED (traced).
**Problem**: `decide_event` checks `hook.is_enabled()` itself before it calls `with_main`:
```rust
if !hook.is_enabled() {
    return None;
}
let bindings = hook.bindings();
...
let (decision, refused) = with_main(|main| { ... })?;
```
So with remapping off, `REFUSED` stays 0 whether or not the callback's early return at `:326-331` exists.
`remapping_off_callbacks_pass_without_asking_for_the_engine` therefore pins the `foreground_changed` gate
(`:377-379`) and the property "nothing asks for the engine while off". It does not pin the keyboard callback's gate.
The comment at `:553-556` says it does ("pins that gate (A5-m1)").
**Failure**: delete `keyboard_hook_callback.rs:326-331`. The test stays green. The callback then reads `lparam` and
calls `record_callback` for every event while `swch off` is in force, which goes against step 2 of the module docs
("without reading it"). Nothing fails functionally.
**What to do**: make the test tell the two checks apart, or say in the comment that the test pins the property and
not the line.

#### m2. "N pending gestures dropped" and `hold_timer_not_started` count timers whose entries are not lost
**Where**: `src/hook/main_thread.rs:227-240` and `:448-450`; `src/engine/gestures.rs:406-417` (`forget` keeps an
`AwaitSecond` press); `src/engine/mod.rs:489-496` (`forget_timer` ignores `Chord`/`Sequence`);
`src/hook/keyboard_hook_callback.rs:209-213` and `main_thread.rs:520-522` (one bump per refusal, whatever the slot);
`src/logging/counters.rs:27-30`; `book/src/config/errors.md:198` and `:208`. **Confidence**: CONFIRMED (traced).
**Problem**: when `SetWaitableTimer` fails in `plan_wake`, `dropped` is `lost.len()`, the count of every queued token.
That count includes the window timers of Taps that wait for a DoubleTap window, which `Gestures::forget` keeps (they
fire at the next key-down, ruling M6), and chord and sequence expiry timers, whose entries stay under the lazy deadline
check. The counter is also bumped for a refused chord or sequence expiry timer, and nothing is lost there. The errors
page says "the `Hold`s waiting for it were dropped", and the counter doc names only Holds and waiting Taps.
**Failure**: a pending `chord` plus an F9 Tap waiting for its window, then a failed `SetWaitableTimer`. The log reads
`could not set the gesture timer (...); 2 pending gestures dropped`, but the chord still completes inside its window
and the Tap fires at the next key-down. A user who reads the log or the counter concludes that gestures were lost.
**What to do**: count only the entries that `forget_timer` really dropped (for example, liveness before and after),
or reword the message, the counter doc and the errors.md rows. The trigger is rare, because `SetWaitableTimer`
practically never fails on a valid handle.

## Checklist walked

These are the items I checked and found nothing reachable that fails.

**Rulings** (`rulings.md`)
- **I1**: `gestures.rs:317-320` keeps a passing `Held` press whose Hold is due by the clock. Both scripts of
  `a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down` fire the Hold. Every
  other state the addendum lists (DueHold on key-up, the `press` M6(b) case, a closed window decided by `now < due`,
  chord and sequence lazy checks) is decided by `Instant`.
- **M1**: every removed test has a named successor:
  - `with_engine_*` → `main_thread` re-entry tests;
  - `daemon_handle_status_follows_the_hook_flag` → `daemon_handle_switches_remapping_through_the_main_thread`;
  - the fast-gate test and the Hold-on-executor test → their successors.

  The C10 renames carry flipped assertions, and the master tests in the ruling's ranges keep their names with
  `MainThread::new()`.
- **M2, M3**: `known-issues.md` row 75 and row 67, §11.6 and §11.8, CHANGELOG.
- **M4**:
  - (a) `sync_channel(1)` plus `try_send` in `serve` (`main_thread.rs:528-539`);
  - (b) the `NoAnswer` doc and the CHANGELOG;
  - (c) `drop(main)` right after `run` (`app.rs:270-274`). A request that is queued or arrives later sees its reply
    sender dropped and gets `ShuttingDown` at once.
- **M6**: (a) and (b) are implemented and pinned (`a_tap_whose_window_timer_is_lost_fires_at_the_next_key_down`,
  `a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes`,
  `a_press_after_a_lost_key_up_replaces_the_stale_press_and_keeps_a_hold_due_by_the_clock`). The ruling says "the
  next event". Key-ups and repeats do not fire the Tap, and the addendum justifies that with D6.
- **M7**: under `cfg(test)`, `FastAction::run` only records (`windows_actions.rs:148-165`), and `send_menu_mask` only
  counts. Each `cfg_attr(test, allow(dead_code))` has a reason.
- **M8**: `pressed` is read before `decide_event`, and `held >= 200 ms` is asserted (`main_thread.rs:915-929`).
- **OQ4**: the scan covers the three extra files and the four tokens.
- **OQ5**: `switch_enabled(&self, _, &mut MainState)`.
- **Q1**: `MAIN` is a thread-local reached only through `with_main` (`try_with` plus `try_borrow_mut`).
- **Q3**: HIGHEST.
- **Q4**: the gap is pinned and documented.
- **Q7**: only the main loop's MWMO pin is in this lane.

**R1, hook callback**
- Every path ends in exactly one outcome: `code < 0` → `CallNextHookEx` (`:318-323`), off →
  `CallNextHookEx` (`:326-331`), `LRESULT(1)` iff `suppress` (`:350-352`), otherwise `CallNextHookEx` (`:356`).
- The sentinel check (`:188`) comes before any state.
- The callback file has no `log::` call.
- New per-event work: TLS plus the RefCell flag, `other_key_down` over 0-2 presses, and the `deferred_group` pass.
- Under the borrow, `start` makes at most one `SetWaitableTimer` call, with no completion routine and no pump.
- No new panic path: `try_borrow_mut`; indices come from `position`; `Instant + term` has terms ≤ 10 s
  (`loader.rs:37`); the `due_time` invariant holds.
- Allocations are those the cut prescribes: the `GroupKey` clone on a DoubleTap arm (`KeyCombination.keys` is a
  `Vec`), the `Decision.timers` push of the D5 and D8 timers, and `TimerQueue` growth past 16.

**R2, sentinel**: no new `SendInput` builder and no hard-coded constant. Hold fast actions use the existing builders,
now from `drain` → `carry_out`, outside any borrow.

**R3, shared state**
- `Mutex<Engine>`, `with_engine`, `DECIDING`, `start_hold_timer` and `sleep_until` are gone.
- `MAIN` is thread-local (Q1).
- No borrow site pumps or injects:
  - `decide_event`, `foreground_changed`, `run`/`plan_wake`/`fired`, `take_due_timer`;
  - `serve` → `set_enabled`, which calls `cancel_actions` → `PostThreadMessageW`, which does not block.

  So the "no such re-entry" claim at `keyboard_hook_callback.rs:83` holds.

**R4, orderings**: `enabled` uses `swap(AcqRel)` and `load(Acquire)` and publishes no data. No `Relaxed`, `SeqCst`
or `static mut` was added. The bench's `PROBED_AT` Release/Acquire pair is ordered by `SendMessageTimeoutW`. The
`MAIN_THREAD_ID` static (Ctrl+C) and `MainThreadHandle.thread_id` both come from `GetCurrentThreadId` on the same
thread.

**R5-R9**
- R5: no new `Binding` option.
- R6: `MainThreadError` has one `Display` arm per variant; `TimerSlot`, `Plan`, `Wake` and `Request` are never shown.
- R7: `update_bindings` is unchanged, and `deferred_group` walks `skip(self.index)` in list order.
- R8: presses are keyed by the sided event vk, and the group identity is the order-insensitive `KeyCombination ==`
  plus `Condition ==`.
- R9: this lane has no registry or process reads.

**R10, loop and shutdown**
- Startup: `MainThread::new` comes before `install`, and `serve` after it (KS-20 order kept).
- `WM_QUIT` → `pump` true → priority restored → `drop(main)` → `uninstall` → executor stop. Nothing between `run`
  and `uninstall` blocks.
- `WAIT_FAILED` → `Err` → hooks removed → `[FATAL]`.
- There is no spin path:
  - the timer is a synchronization timer (`timers.rs:27-29`, no `MANUAL_RESET`);
  - `Plan::Due` iterations each remove a timer;
  - a stale or early signal costs one drain;
  - the input and the timer cannot starve each other, because `drain` reads `Instant::now()` after every pump.
- Lost wakeups: `try_send` happens before `PostThreadMessageW`, and `drain` reads the channel directly, so a lost
  `MAIN_WAKE` only delays a request until the next wake.
- `armed` bookkeeping holds under every interleaving I traced: callbacks inside `pump`, `take_due` taking the armed
  request, and `retain` dropping it.
- `run`'s SAFETY claim about the handle holds: `MainThread` is `!Send`/`!Sync`, and only its `Drop` takes the state.

**Engine correctness**
- Suppression invariant: the key-up rule is unchanged (`mod.rs:759-760`). Cancel, spend and wait change no
  `keys` record. A spent second DoubleTap press is suppressed through `group.block` → `mark_blocked`.
- Repeats never reach `press` or `other_key_down`.
- Tokens: a new seq on `AwaitSecond` makes the Hold token stale, and `reset` bumps the generation. Tokens are checked
  twice, by `retain(is_live)` and in `on_timer`.
- `on_timer` never sets `mask` or `suppress`.
- Chord and sequence expiry timers fire at the same condition as the lazy `expires_at > now`.

**Win32 and unsafe**: every new `unsafe` block in `main_thread.rs`, `priority.rs` and `bench.rs` has a concrete
`// SAFETY:`. The timer handle is owned and closed by `WaitableTimer::drop`. The pseudo thread handle is not closed.
The test and bench windows are message-only `STATIC` windows, destroyed by their owning thread, with the original
WNDPROC restored.

**Tests**
- No test installs a hook or calls `logging::init`/`attach_file`, and none injects input under `cfg(test)`.
- `SetThreadPriority` runs only on scratch or bench helper threads.
- Red commits: each declared red commit is in `impl.md`, and the I1 test was red under the documented mutation. The
  C2 ports are green on master by Q6.
- `a_thousand_hold_presses…` would fail without `retain` in `start_timers`.
- `remapping_switches…` would fail without the reset.

**Docs**: README, `default_config.ron`, `triggers-conditions.md`, `known-issues.md` (rows 23, 24, 67, 74, 75),
`actions.md`, `errors.md`, `kinds.md`, `structure.md` and the CHANGELOG (Added, Changed with **Breaking:**, Fixed
KS-25/KS-33/KI-74) match the code, apart from m2. The non-config pages are listed in `impl.md` for the Phase-2 book
sync.

## Positive

- The production and scenario harnesses share one path (`Engine::start_timers` through a `Scheduler`), so the
  scenarios exercise the same pruning and refusal logic as the main loop.
- `with_main` uses `try_with` plus `try_borrow_mut`, so thread teardown and re-entry cannot panic.
- `MainThread` is `!Send` and enforces one state per thread.
- `plan` is pure and pinned by its own test.
- Setting the wake inside `Scheduler::start` makes the loop correct whether or not a sent message is dispatched
  inside the kernel wait, so the design does not depend on undocumented behaviour.
- M7 closes a real safety gap: before it, a test reaching `carry_out` with a fast action could type into the owner's
  focused window.

## Open questions

1. **KB 2682659 and the new `MAIN_WAKE` dependency.** Microsoft's article (the cut's evidence for the loop) says a
   message posted while a WinEvent callback runs may not wake the loop until the next message. It recommends
   `MsgWaitForMultipleObjects` + `PeekMessage`, as built, plus an event that the WinEvent handler sets. The lane's
   loop has no such event. `swch on`/`off` now depend on `MAIN_WAKE`: a wake posted during `foreground_changed` could
   wait for the next key event, and the worker would answer `NoAnswer` after 2 s while the switch still takes effect
   later. `WM_QUIT` had the same exposure under master's `GetMessageW`, so this is not a regression there. What would
   settle it: whether `MWMO_INPUTAVAILABLE` (not used in the KB's loop) covers this case on Windows 10/11. Settling it
   needs a WinEvent hook, so it is a manual check (an M7/M9 variant: `swch off` during rapid Alt+Tab). The KB's own
   example loop uses `QS_ALLEVENTS | QS_ALLPOSTMESSAGE` and still receives WinEvents, so out-of-context WinEvents do
   wake a `QS_ALLINPUT` wait. That part is not in doubt.
2. **Two-modifier DoubleTap after D6.** For a passing `( keys: "Ctrl+Shift", ..., trigger: DoubleTap(300) )`, the
   press arms under the modifier pressed last. If the user releases both and presses Ctrl again, that is "another
   key-down", and it cancels the passing window (`gestures.rs:312`). The gesture now fires only if one modifier stays
   held across both presses; on master the Ctrl-first order completed it. This is the literal owner decision, but no
   test pins it and no page states it. What would settle it: the owner's intent (a values question for the
   orchestrator).
3. **`GestureTimers::start` keeps `armed` after a failed `SetWaitableTimer`** (`main_thread.rs:256-263`), while
   `plan_wake` clears it before setting (`:221`, "Setting the timer replaces the due time it had"). If a failed set
   could stop the earlier timer, `plan` would return `Keep` (`:175`), and that Hold would wait for the next message.
   What would settle it: whether `SetWaitableTimer` changes the timer's state when it fails. With a valid handle it
   practically never fails. If it can, clear `armed` on that path.

Housekeeping: `D:/tmp/ks-orch/p2-2.2-timers/review-target/` is my clippy target directory, which the orchestrator can
delete.
