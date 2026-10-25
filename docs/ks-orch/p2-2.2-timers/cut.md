# Cut: p2-2.2-timers — roadmap 2.2 Timers on the main thread (KS-11, KS-25, KS-33)

Branch `feat/main-thread-timers` from `master` @ `c9c1373`. Tree read: `D:/key-switch-rs/key-switch-rs` (clean).
All `path:line` below are on `c9c1373` unless marked `@7cb3521` / `@20af28d`. `windows` items were checked in
`%USERPROFILE%/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/windows-0.62.2/src/Windows/Win32/…`.

## Verdict
CUT READY

## Spec re-located

| Spec reference | Current file:line | Status | Note |
|---|---|---|---|
| roadmap 2.2 "SetTimer timers arrive through the GetMessageW loop" (app.rs:125-131 @7cb3521) | `src/core/app.rs:253-263` (plain `GetMessageW`/`TranslateMessage`/`DispatchMessageW`) | moved | Replaced by the main loop of D3 |
| KS-11 stale timers (tap_state.rs:61-66, 103-119, 156-167 @7cb3521) | gone; `src/engine/gestures.rs:94-121` (Tap deadline), `:128-154` (Hold arm + token), `:173-179` (`fire_hold`), `:200-219` (DoubleTap) | gone / already fixed | Fixed in `16a0461`; pinned by `two_quick_taps_fire_twice` (`scenarios.rs:370-389`), `stale_hold_timer_does_not_fire_the_next_press_early` (`:427-449`), `four_quick_presses_double_tap_twice` (`:643-667`) |
| KS-25 Tap/Hold independent (keyboard_hook_callback.rs:314-315 @7cb3521) | `src/engine/mod.rs:619-646` (`on_key_up`: `resolve_tap` :628, `release_hold` :630, independently); separate tables `gestures.rs:75-82` | moved, half fixed | Equal terms exclusive (`scenarios.rs:451-476`, `:502-532`); different terms still "both"/"neither" |
| KS-33 one thread per gesture (tap_state.rs:61-67/105-119/158-167 @7cb3521) | `src/hook/keyboard_hook_callback.rs:302-338` (`start_hold_timer` spawns `hold-timer`, `sleep_until`), called at `:222-224`; `use std::thread` `:91` | moved, Hold half open | Spawn failure no longer aborts (`:321-324`, counter `HoldTimerNotStarted` `src/logging/counters.rs:27-28`); Sequence half closed by 2.1 |
| roadmap 2.2 Files: "keep the engine behind a Mutex" | `src/hook/keyboard_hook.rs:62-68` (`engine: Mutex<Engine>`), `:90-112` (`with_engine`), `:20-40` (`DECIDING`, `DecidingGuard`) | unchanged | Removed by D1 |
| roadmap 2.2 Files: "new src/platform/timers.rs" | `src/platform/timers.rs:1-114` exists (2.1) | moved (exists) | Reused as is (D3); roadmap wording goes to book sync |
| spec: executor `MsgWaitForMultipleObjectsEx` + pump shape | `src/core/executor/mod.rs:586-673` (`run_loop`, wait `:647-654`), `:677-701` (`pump`), `:442-452` (`plan`) | unchanged | Shape copied, executor not touched (D3) |
| set_enabled must move to the engine's thread | `src/hook/keyboard_hook.rs:167-202` (doc `:185-188` says so) | unchanged | D2 |
| `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` | `src/hook/keyboard_hook_callback.rs:620-688` | unchanged | Successor in C6/C7 |
| `a_hold_timer_never_masks` | `src/engine/scenarios.rs:4239-4256`; harness assert `src/engine/testing.rs:414-415` | unchanged | Stays |
| Known issue 67 scenarios | `src/engine/scenarios.rs:3358-3391`, `:3393-3418` | unchanged | Flip in C10 |
| Known issue 74 (gestures.rs:197-219 @20af28d) | `src/engine/gestures.rs:197-219`; caller `src/engine/mod.rs:515-530` | unchanged | D5 |
| roadmap Follow-ups row "DoubleTap entries per binding or group" | `book/src/roadmap.md:805` | unchanged | Closed by D5 |
| Open question 5 (Tap waits for DoubleTap) / gaps "Tap versus DoubleTap" | `book/src/roadmap.md:787`; `book/src/gaps.md:30`, `:68-91` | open | D5 |
| Open question 10 (other key cancels) | `book/src/roadmap.md:792`; README `:411-414`, `:465-470`; `default_config.ron:314-319` | decided, not implemented | D6 |
| R-7 of 2.1 (hold-timer wake to a reused thread id) | `D:/tmp/ks-orch/p2-2.1-executor/cut.md:501`; source: `keyboard_hook_callback.rs:316-318` → `executor/mod.rs:256-262` | open | Hold half closes with D1; config-watcher half stays (Risks) |
| m-1 MWMO_INPUTAVAILABLE unpinned | `D:/tmp/ks-orch/p2-2.1-executor/results_v3.md:141-146`; `executor/mod.rs:652` | open | Pinned for the main loop (C6 test); executor half is a follow-up |
| 2.1 `delay_lateness` probe shape | `src/core/executor/mod.rs:897-939` | unchanged | Bench row R6 + acceptance test |
| gesture repro tests | `book/src/methodology.md:260-290` | unchanged | Ported in C2 |
| CLAUDE.md principle 5 exception | `CLAUDE.md` "Shared state" bullet (`Mutex<Engine>` until 2.2) | ends | Orchestrator edit (Q1) |
| `Env::now` doc (lock, timer threads) | `src/engine/env.rs:23-30` | unchanged | Rewritten in C7 |
| engine docs "Time", "Threads and locking" | `src/engine/mod.rs:60-99` | unchanged | Rewritten in C7/C9 |

## Goal and non-goals

**Goal.** After the lane:
- The engine and every gesture timer live on the main thread: a `thread_local!` main state (engine + timer queue + one high-resolution waitable timer), reached only by `with_main`. `Mutex<Engine>`, `with_engine`, `DECIDING`, `start_hold_timer`, `sleep_until` are gone; no thread is started on the gesture path.
- The main loop waits with `MsgWaitForMultipleObjectsEx([gesture timer], INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)`, drains control requests and due timers before every wait, and pumps with `PeekMessageW`. Gesture deadlines fire within about 1 ms (high-resolution timer, not the 15.6 ms tick).
- `swch on`/`off` are served on the main thread through a request queue and a wake (flip and reset atomic with respect to key events and timers).
- `TimerToken { generation, slot, seq }`; a stale token does nothing; `Scheduler` trait + pure `TimerQueue`; engine scenarios drive time through the fake.
- Tap, Hold and DoubleTap of one press form one exclusive machine (KS-25 for every term pair); Tap waits for the DoubleTap window only when its group has both; DoubleTap identity is the group (known issue 74).
- Another first key-down cancels a pending gesture whose key passed (owner decision; known issue 67 flips); swallowed gestures keep today's behaviour.
- Chord and Sequence entries expire on their own timers.
- The main thread pumps at `THREAD_PRIORITY_HIGHEST` (D9), kept only if bench row R8 does not contradict it.
- A latency bench (`bench::latency_bench`, ignored) runs on the parent and on HEAD.

**Non-goals.** Reset on reload / unlock and the control window (2.3). Watchdog (2.4). Mask on lone Alt/Win gestures (known issue 66, 3.1). Modifier-held output (known issue 68, 3.1). Winner locked per press (known issue 73, 3.4). Loader warning for different Tap/Hold terms (KS-25's suggested fix; 3.4 diagnostics). Executor wait/pump consolidation and the executor's own m-1 pin (3.4). Re-entry counter (3.4). `KBDLLHOOKSTRUCT.time` as the event clock (follow-up, Risks).

**Later items that depend on this.** 2.3 (adds `Request::Reset` to the same queue; `reset_transient_state` on the main thread), 2.4 (watchdog reinstall from the main loop), 3.2 (sequence matching on the seq'd entries), 3.4 (the thread-local engine is its prescribed form), 3.5 (bench budget), features 5/6 (chord expiry timer = replay trigger; mod-tap interrupt rules extend D6).

## Decisions

### D1: The engine lives in a thread-local main state on the main thread; `with_main` is the only door
**What.** New `src/hook/main_thread.rs`:
```rust
thread_local! { static MAIN: RefCell<Option<MainState>> = const { RefCell::new(None) }; }
pub(crate) struct MainState { pub(crate) engine: Engine, pub(crate) timers: GestureTimers }
pub(crate) fn with_main<R>(f: impl FnOnce(&mut MainState) -> R) -> Option<R>;   // try_borrow_mut only; None on re-entry or no state
pub(crate) struct MainThread { /* MainGuard (!Send), requests: Receiver<Request>, handle: MainThreadHandle */ }
impl MainThread {
    pub(crate) fn new() -> windows::core::Result<Self>;
    pub(crate) fn handle(&self) -> MainThreadHandle;
    pub(crate) fn run(&self, hook: &KeyboardHook) -> windows::core::Result<()>;
}
```
`MainThread::new` (the calling thread becomes the main thread): `PeekMessageW(PM_NOREMOVE)` to create the queue (as `executor/mod.rs:529-533`), `GestureTimers::new()?` (a `WaitableTimer`), `Engine::new()`, a `sync_channel::<Request>(8)`, then installs the state (`Err(E_UNEXPECTED)` if the thread already has one). Dropping `MainThread` clears the slot. The engine belongs to the thread; bindings and `enabled` stay in the shared `KeyboardHook`.
Under the borrow only: `Engine::on_key`/`on_timer`/`on_foreground_changed`/`reset`, `Engine::start_timers` (D4), `SetWaitableTimer` (D3). Never under it: actions, `SendInput`, menu mask, `submit`, pumping. `with_main` uses `try_borrow_mut` and never `borrow_mut`: a nested call returns `None` and the event passes (today's answer, `keyboard_hook.rs:90-112`), so a re-entered callback can neither panic nor deadlock. A `#[cfg(test)]` thread-local `REFUSED: Cell<u32>` counts refusals for the fast-gate successor test.
**Why.** After D2/D3 every caller of the engine runs on the main thread (LL hook callback, WinEvent callback, timer dispatch, control requests), so a lock only adds cost and a deadlock class. Roadmap 3.4 prescribes exactly this form (`roadmap.md:564`: "an `Engine` held in a `thread_local!` on the main thread. On re-entry, `try_borrow_mut` fails; the event is skipped"). No lock also means no priority inversion, which D9 relies on.
**Rejected.** Keep `Mutex<Engine>` (spec removes it; would invert priority under D9). `Cell<Option<Box<_>>>` take/put (same semantics, loses state on unwind). Engine in `KeyboardHook` behind `UnsafeCell` (needs `unsafe` and a false `Sync`).
**Cost.** TLS access plus one RefCell flag per event (ns; replaces an uncontended lock plus the `DECIDING` TLS). CLAUDE.md principle 5 / reviewer R3 forbid `RefCell` "for hook state"; this is not shared state (Q1).

### D2: `swch on`/`off` are requests served by the main thread; the IPC worker waits at most 2 s
**What.** `MainThreadHandle { thread_id: u32, requests: SyncSender<Request> }` (Clone + Send + Sync). `enum Request { SetEnabled { enabled: bool, reply: SyncSender<bool> } }`. `MainThreadHandle::set_enabled(&self, enabled) -> Result<bool, MainThreadError>`: `try_send` (Full → `Busy`, Disconnected → `ShuttingDown`), then `PostThreadMessageW(thread_id, MAIN_WAKE = WM_APP + 1)`, then `reply.recv_timeout(REPLY_TIMEOUT = 2 s)` (Timeout → `NoAnswer`, Disconnected → `ShuttingDown`). `MainThreadHandle::request_exit()` posts `WM_QUIT` (as `app.rs:90-100`). `MainThreadError` has one `Display` arm per variant: `the daemon's main thread did not answer within 2 s`, `the daemon is shutting down`, `the daemon is busy; try again`.
The loop serves it with `MainState::set_enabled(&mut self, hook: &KeyboardHook, enabled: bool, env: &dyn Env) -> bool`: `hook.switch_enabled(enabled)` (`enabled.swap(.., AcqRel)`, true if it changed) → if changed: `engine.reset(env)`, `timers.retain(live)` (empties the queue: D4 generation), `hook.cancel_actions()` (made `pub(crate)`). `KeyboardHook::set_enabled` is removed. `DaemonHandle` (`app.rs:69-101`) holds `hook` and a `MainThreadHandle`.
**Why.** Flip and reset must be atomic with respect to events (`keyboard_hook.rs:174-188`). `PeekMessageW` dispatches sent messages (hook calls) before posted ones, so a flip on the IPC worker followed by a posted reset would let a key-down be decided on stale state before the reset and lose it. On the main thread both happen between callbacks. 2 s is under the swch client's 5 s reply deadline (`build-and-run.md:264`). The receiver is dropped at exit, so pending requests fail fast.
**Rejected.** Flip on the worker and post a reset (race above). A reply pointer in `lparam` (unsafe; leaks if never dispatched). A blocking `send` on a full queue (the worker could hang).
**Cost.** One channel allocation per `swch on/off`, on the IPC worker. New swch error texts (book sync: `build-and-run.md:229-230`).

### D3: One main loop: drain → plan → `MsgWaitForMultipleObjectsEx` → pump
**What.** `MainThread::run(&self, hook)`, on the main thread:
1. **Drain until a whole pass did nothing:** every `Request` (`try_recv`), then every due timer: `with_main(|m| { let r = m.timers.take_due(Instant::now())?; let mut d = m.engine.on_timer(r.token); m.engine.start_timers(take(&mut d.timers), &mut m.timers); Some(d) })`, and after the borrow `carry_out(hook, d)`.
2. **Plan** (pure `fn plan(armed: Option<Instant>, next: Option<Instant>, now) -> Plan { Cancel, Due, Keep, Arm { after, at } }`, the executor's shape `executor/mod.rs:442-452`): `retain(live)` first; `Due` → back to 1; `Arm` → `WaitableTimer::set`; `Cancel` → `cancel`. On a set failure: `log::error!("could not set the gesture timer ({e}); {n} pending gestures dropped")` and every queued token is forgotten (D4).
3. **Wait:** `MsgWaitForMultipleObjectsEx(Some(&[timer]), INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)`. `WAIT_OBJECT_0` → `timers.fired()` (armed = None) → 1. `WAIT_FAILED` → return `Err(Error::from_thread())` (App::run uninstalls; `[FATAL] the main message loop failed: …`). Otherwise → 4.
4. **Pump:** `PeekMessageW(PM_REMOVE)` until empty: `WM_QUIT` → return `Ok(())`; thread message `MAIN_WAKE` (hwnd null) → back to 1; anything else → `TranslateMessage` + `DispatchMessageW`. Hook callbacks run inside these `PeekMessageW` calls.

`GestureTimers { queue: TimerQueue, wake: WaitableTimer, armed: Option<Instant> }` implements `Scheduler` (D4): `start(r)` pushes and, if `armed.is_none_or(|a| r.due < a)`, calls `wake.set(r.due.saturating_duration_since(Instant::now()))` and sets `armed`; on `Err` it removes `r` and returns `false`. Setting the timer inside `start` (that is, inside a callback) keeps correctness independent of whether a callback can run inside the kernel wait: the wait always holds the earliest live deadline. A stale or early signal causes one extra drain only (Instant decides; SetWaitableTimer remarks: a synchronization timer "remains in this state until a thread completes a wait operation"). `WaitableTimer::new()` falls back to a plain timer before Windows 10 1803; `MainThread::new` then logs `[WARN] gesture timers wake on the 15.6 ms system tick: no high-resolution timer before Windows 10 version 1803`.
**Why (latency, from Microsoft Learn).** LowLevelKeyboardProc: "The call is made by sending a message to the thread that installed the hook." GetQueueStatus: `QS_SENDMESSAGE` "A message sent by another thread or application is in the queue", and `QS_ALLINPUT` includes it. MsgWaitForMultipleObjectsEx returns `WAIT_OBJECT_0 + nCount` when "New input of the type specified in the dwWakeMask parameter is available". PeekMessageW: "During this call, the system dispatches (DispatchMessage) pending, nonqueued messages", first in its processing order ("Sent messages / Posted messages / …"). So a hook call wakes the wait and the next `PeekMessageW` dispatches it before any posted message: no timer, poll or extra message stands between the hook call and the callback. Whether `MsgWaitForMultipleObjectsEx` itself dispatches a sent message is not documented; D3 does not depend on it (the wake timer is set in `start`). `MWMO_INPUTAVAILABLE`: "The function returns if input exists for the queue, even if the input has been seen (but not removed)"; it is needed because `pump` returns at `MAIN_WAKE` leaving later messages, and `QS_POSTMESSAGE` "is cleared when you call GetMessage or PeekMessage". Microsoft's KB 2682659 recommends `MsgWaitForMultipleObjects` + `PeekMessage` instead of `GetMessage` on a thread with WinEvent callbacks ("GetMessage ignores a message that is received during the execution of the callback"), so the WinEvent hook is at least as well served as with today's `GetMessageW` loop. Tested without a hook in C6 (sent-message round trip, MWMO pin) and bench R7 (against master's `GetMessageW` loop).
**Rejected.** `SetTimer`/`SetCoalescableTimer` (tick-granular; the owner decided high resolution, open question 7). One waitable timer per gesture (n handles, `MAXIMUM_WAIT_OBJECTS`). Re-planning only in the loop (wrong if a callback runs inside the wait). One wait/pump helper shared with the executor (touches merged, measured 2.1 code and its scan test `executor/mod.rs:1479-1522`; left to 3.4).
**Cost.** Per wake: one drain pass (empty `try_recv`, empty `take_due`; ns) plus `plan`. Per timer-arming event: at most one `SetWaitableTimer` (about 1 µs) instead of a thread spawn (about 50-100 µs). Hold fast actions now run on the main thread (the same budget as every Tap/Immediate fast action, `actions.md:49`).

### D4: Tokens, `Scheduler`, `TimerQueue`, liveness
**What.** `src/engine/mod.rs`:
```rust
pub struct TimerToken { generation: u32, slot: TimerSlot, seq: u64 }      // private fields; Copy, Eq, Debug
enum TimerSlot { Gesture(u16 /* vk */), Chord, Sequence }                   // private; Copy, Eq, Debug
struct Ids { generation: u32, last_seq: u64 }                               // field of Engine
impl Engine {
    pub fn is_live(&self, token: TimerToken) -> bool;   // generation == current && an entry with (slot, seq) waits for this timer
    pub fn start_timers(&mut self, timers: Vec<TimerRequest>, scheduler: &mut dyn Scheduler) -> u32; // retain(live), start each; a refused one is forgotten; returns how many were refused
    pub fn forget_timer(&mut self, token: TimerToken);  // generalised (D5, D8)
}
```
`generation` counts `Engine::reset` calls (wrapping); `seq` is an engine-wide arming counter (wrapping, never reset). `src/engine/schedule.rs` (new, pure):
```rust
pub trait Scheduler {
    fn start(&mut self, request: TimerRequest) -> bool;
    fn retain(&mut self, live: &dyn Fn(TimerToken) -> bool);
}
pub struct TimerQueue { pending: Vec<TimerRequest> }   // Vec::with_capacity(16)
impl TimerQueue {
    pub fn new() -> Self; pub fn len(&self) -> usize; pub fn next_due(&self) -> Option<Instant>;
    pub fn take_due(&mut self, now: Instant) -> Option<TimerRequest>;  // earliest due <= now; ties in start order (Vec::remove)
    pub fn clear(&mut self);
}
impl Scheduler for TimerQueue { /* start: push, true; retain: Vec::retain */ }
```
The adapter, the loop and `engine::testing::Scenario` all go through `Engine::start_timers`; a refused start bumps `Counter::HoldTimerNotStarted` in the adapter (name kept for log-line compatibility; doc: "a gesture timer could not be set"). `Scenario.timers` becomes a `TimerQueue` (`testing.rs:152`, `:400-423`, `:439-446`); `advance_ms` pops with `take_due`; `remapping_off`/`remapping_on` also `retain(live)`.
**Why.** The spec's token shape. Pruning by `is_live` before each start keeps the queue at "live timers + 1" without a cancel list in `Decision` (no allocation on ordinary key-ups), and "a stale token does nothing" holds twice (pruned, and `on_timer` checks again). One code path for production and scenarios.
**Rejected.** `Decision.cancel: Vec<TimerToken>` (allocates on common key-ups). An engine-computed `next_deadline` without tokens (drops the spec's token and the Scheduler seam). Passing `&mut dyn Scheduler` into `on_key` (churns every call site).
**Cost.** `retain` is O(queue × entries), both tiny, and runs only on timer-arming events. The engine reaches Win32 only through the `Env` and `Scheduler` trait objects (documented in the module docs).

### D5: One exclusive machine per press; Tap waits for the DoubleTap window; DoubleTap identity is the group
**What.** `gestures.rs` is rewritten around `Press { vk: u16, seq: u64, phase }` (at most one per vk) plus `due_holds: Vec<DueHold { vk, seq, action }>`:
- `Phase::Held { tap: Option<(Instant, BindAction)>, hold: Option<(Instant, BindAction)>, double: Option<(Instant, BindAction, GroupKey)> }`
- `Phase::AwaitSecond { due: Instant, double: BindAction, group: GroupKey, deferred_tap: Option<BindAction>, passed: bool }`
- `GroupKey { keys: KeyCombination, condition: Condition }` (cloned only when the group has a DoubleTap; compared with the order-insensitive `KeyCombination ==` (`key_combination.rs:16-29`) and `Condition ==`).

`winner.rs` gains `pub(super) struct GroupArming<'a> { keys, condition, tap, hold, double_tap: Option<(Duration, &'a BindAction)>, block: bool }` and `Winner::deferred_group(&self, vk, bindings, held) -> Option<GroupArming>` (None for an Immediate winner): it walks `bindings[winner.index..]` with `admits` and `GroupArms::claim`; `block` is the OR of the claimed members' `block_default` (today's rule, `mod.rs:485-530`). `on_key_down`, on a first key-down only, computes it right after `winner::elect`, calls `gestures.press(..)` before the binding loop and sets `suppress` from `block`; the loop keeps blockers, the Immediate winner, sequences and chords, and skips deferred members.

| Moment | Rule |
|---|---|
| first key-down of k: `press(k, group)` | If k has an `AwaitSecond` A: if `group` has a DoubleTap, `group.keys`/`condition` equal `A.group` and `now < A.due` → fire A's DoubleTap action (press 1's, as today), drop A, arm nothing (press consumed). Otherwise drop A, firing its `deferred_tap` if any (flush). Then, if `group` is Some, arm `Held` (a Hold → `TimerRequest { Gesture(k), seq }` at `now + H`). |
| Hold timer (live) | fire the Hold; drop the press (Tap and DoubleTap consumed: KS-25 for every term pair) |
| key-up of k at t, press `Held` | Hold present and `t ≥ hold.due` → `DueHold` stub (same seq) left to its timer (today `gestures.rs:156-169`). Else the Hold is cancelled; Tap live (`t < tap.due`) and window open (`t < double.due`) → `AwaitSecond { deferred_tap: Some }` with a **new seq** and a timer at `double.due`; Tap live, window closed → fire the Tap now; Tap dead, window open → `AwaitSecond { deferred_tap: None }`, new seq, no timer (lazy expiry); else drop. `passed` = the key's record before the key-up (`was_passed`, `mod.rs:640`). |
| window timer (live) | fire the deferred Tap; drop |
| DueHold timer (live) | fire the Hold; drop the stub |

`on_key_up` suppression keeps today's rule, with `gesture_bound` = (Tap fired or deferred) || a Hold was found (`mod.rs:640-643`).
**Why.** Spec: "one mutually exclusive state machine"; "Tap waits until the DoubleTap window closes (only then)". The new seq at `AwaitSecond` makes the press's Hold token stale, so it cannot fire the waiting Tap early. A DoubleTap needs two consecutive presses of its key won by its group, so at most one entry per key exists and a press won by another group ends it: the known-issue-74 press no longer fires the first binding's action. Identity by keys + `when:` is the group's own definition (`winner.rs:84-114`) and survives a reload (`reload_during_a_passing_double_tap_keeps_each_press_whole`, `scenarios.rs:3317-3356`, stays green); a list index would not.
**Result (Tap T, Hold H, release at t).** `t < T` and `t < H` → Tap (at `press + D` if the group has DoubleTap D); `t ≥ H` → Hold only (fixes "both" for `H ≤ t < T`); `T ≤ t < H` → nothing (too long for a Tap, too short for a Hold; pinned and documented). A double press within D fires only the DoubleTap; a press resolved by its Hold opens no window.
**Latency cost (declared).** Only for a group with both Tap and DoubleTap: the Tap fires at `max(release, press + D)` instead of at the release, at most D after the press; another key's key-down fires it at once (D6). Every other gesture timing is unchanged.
**Rejected.** Three tables plus cross-checks (today; the defect). Entries keyed by binding index (wrong after a reload). Firing the Tap at release and cancelling it on the second press (an action that ran cannot be cancelled). Firing an overdue Hold from the key-up (reorders a passing modifier's key-up and the Hold output: known issue 68).
**Cost.** One more pass over the bindings for a deferred winner; one `KeyCombination` + `Condition` clone per DoubleTap arm (next to today's action clone, `gestures.rs:213-217`).

### D6: Another first key-down cancels a pending gesture whose key passed; swallowed gestures keep today's behaviour
**What.** `Gestures::other_key_down(vk, keys: &KeyState, fire: &mut Vec<BindAction>)`, called at the top of `on_key_down` for a first key-down (`!was_repeat`), for every press of another key (`p.vk != vk`):

| Press state | key-down passed | key-down swallowed |
|---|---|---|
| `Held` (Tap/Hold/DoubleTap pending) | cancel (drop; its timer goes stale): known issue 67 | keep (today; `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing`, `scenarios.rs:1057-1080`) |
| `AwaitSecond` with `deferred_tap` | fire the Tap now, before the new key, and drop (window closed) | same |
| `AwaitSecond`, no deferred Tap | cancel | keep (today) |
| `DueHold` | keep (already resolved as a Hold) | keep |

"Passed" is `keys.is_passed(p.vk)` for `Held` and the stored `passed` for `AwaitSecond`. Auto-repeats and key-ups of other keys change nothing. Injected events never reach the engine.
**Why.** Owner decision (CLAUDE.md; roadmap open question 10): another key-down cancels a pending gesture whose key passes, as AutoHotkey does, and the two known-issue-67 scenarios flip. Swallowed: no evidence for a change, and the cited scenario pins today's behaviour. The waiting Tap is a different case: it completed with no key in between, and with a Tap-only config it would already have fired at its release, before the new key. Firing it at the next key-down keeps its output before that key (roadmap 2.1: fast actions run in the callback "so that their output stays in order with the keys that follow", `roadmap.md:447`); closing the window then is forced by exclusivity (a later DoubleTap would make one pair of presses fire both). Repeats are excluded: a key held since before the gesture is not a key pressed during it, and a key pressed during it already cancelled with its first key-down.
**Rejected.** Cancelling the waiting Tap of a passing press (Tap+DoubleTap would then behave differently from Tap alone). Cancelling swallowed gestures (pre-empts mod-tap's interrupt rules, new feature 6). Counting repeats.
**Cost.** One scan over the pending presses (0-2 entries) per first key-down.

### D7: Hold, waiting-Tap and expiry timers fire on the main thread; `on_timer` never masks
**What.** Timer decisions are carried out by `carry_out` (now `pub(super)`) on the main thread, outside any callback: fast actions run there, slow ones are queued for the executor (`dispatch`, `keyboard_hook_callback.rs:234-242`). `Decision::mask` stays `false` from `on_timer` (no code path sets it; the harness assert stays).
**Why.** Spec. `TargetWindow::for_action` reads the foreground on the main thread, as it did on `hold-timer`.
**Cost.** A fast Hold action now holds up hook delivery while it runs (µs; as Tap actions do today).

### D8: Chord and Sequence entries expire on their own timers
**What.** `PendingChord` and `PendingSequence` get a `seq`; a new chord entry and every sequence start or advance push `TimerRequest { token(Chord | Sequence, seq), due: expires_at }` (an advance takes a new seq, so the previous token goes stale). `on_timer` removes the entry with that seq and fires nothing. The lazy `retain(expires_at > now)` (`chords.rs:78`, `sequences.rs:74`) stays as the authoritative check, so matching behaviour is unchanged, and a refused timer is harmless (`forget_timer` ignores these slots).
**Why.** Roadmap 2.2 What-line; frees stale entries on time and gives a replay buffer (new feature 5) its trigger.
**Cost.** One timer arm per chord start and per sequence step.

### D9: The main thread pumps at `THREAD_PRIORITY_HIGHEST`, kept only if bench row R8 does not contradict it
**What.** `src/platform/priority.rs` (new): `pub struct RaisedPriority` (`!Send`): `raise() -> windows::core::Result<Self>` = `SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST)`; `Drop` restores `THREAD_PRIORITY_NORMAL`. `App::run` raises right before `main.run(hook)` and drops it right after (before `uninstall`, the executor stop and the log flush). A failure → `log::warn!("could not raise the main thread's priority ({e}); it runs at normal priority")`, and the daemon continues. **Rule:** C15/C16 land iff R8 (normal-priority busy threads on every core) gives p99(HIGHEST) ≤ p99(NORMAL); otherwise they are skipped and impl.md records R8 as the reason.
**Evidence.** Scheduling Priorities: "A typical strategy is to use THREAD_PRIORITY_ABOVE_NORMAL or THREAD_PRIORITY_HIGHEST for the process's input thread, to ensure that the application is responsive to the user"; "a high-priority thread should execute for a brief time, and only when it has time-critical work to perform"; "If a higher-priority thread becomes available to run, the system ceases to execute the lower-priority thread". The main thread blocks except for µs per event, every keystroke on the desktop waits for it, and after D1 it holds no lock (no inversion). Priority Boosts: a woken normal thread is boosted only temporarily, and boosted normal threads of other processes can outrank a base-8 hook thread. SetThreadPriority: "A thread with a base priority level above 11 interferes with the normal operation of the operating system", hence HIGHEST (base 10 in `NORMAL_PRIORITY_CLASS`) and not the `TIME_CRITICAL` (15) of AutoHotkey's hook thread (`AutoHotkey/source/hook.cpp`: `SetThreadPriority(sThreadHandle, THREAD_PRIORITY_TIME_CRITICAL)`). No starvation: the loop has no spin path (each `Due` iteration removes a timer), and other input is not hooked by us; it only waits on our callback.
**Rejected.** TIME_CRITICAL (above 11; Microsoft warns). Raising the process priority class (affects the executor and child processes).
**Cost.** None per event. A runaway loop bug would hurt more at 10 than at 8 (no such path; review item).

### D10: Latency bench, in-crate, ignored, file output
**What.** `src/bench.rs` (`#[cfg(test)] mod bench;` in `lib.rs`), one test:
`#[test] #[ignore = "bench: KS_BENCH_OUT=<file> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1"] fn latency_bench()`. It fails with a message if `KS_BENCH_OUT` is unset, and writes a Markdown table (label from `KS_BENCH_LABEL`) with n, mean, p50, p99 and max per row. No print macros (scan test `logging/mod.rs:320-382`), no new dependency, not in the default run.

| Row | What | n | master (C1) | HEAD (C7+) |
|---|---|---|---|---|
| R1 | `Engine::on_key`, typing script (letters, Shift, Space; unbound) on the template + CapsLock Tap/Hold + LShift DoubleTap + a chord + a sequence; bench-local `Env` with a `Cell<Instant>` clock | 10 000 events × 5 | same code | same |
| R2 | `Engine::on_key`, CapsLock Tap(200)+Hold(200) presses (down, +80 ms, up) and F13 Hold presses | 10 000 events | same | same |
| R3 | `Engine::on_timer` for armed Holds | 10 000 | same | same |
| R4 | adapter key path: `keyboard_hook_callback(0, ..)` on `get_hook()` with `F13 → DoNothing` + unbound `A`, down/up | 10 000 events | as is | + `MainThread::new()` on the bench thread |
| R5 | adapter Hold press: `F13 Hold(50) → DoNothing`, key-down callback time | 200 | spawns `hold-timer` | `SetWaitableTimer` |
| R6 | gesture deadline lateness: `F13 Hold(d) → Sequence([Key("F13")])`, FakeRunner executor attached to `get_hook()`, `record.at − t_down − d`, d ∈ {5, 20, 50, 200} | 10 per d | hold-timer thread | main loop on the bench thread until the record (a helper posts `WM_QUIT`) |
| R7 | sent-message round trip: `SendMessageTimeoutW(WM_NULL)` from the bench thread to a `STATIC` message-only window owned by a helper thread running the loop | 1 000 | verbatim `app.rs:257-262` `GetMessageW` loop | `MainThread::run` |
| R8 (only with `KS_BENCH_LOAD=1`, C14) | R7 while `available_parallelism()` busy threads at normal priority run; the loop thread at NORMAL, then HIGHEST | 300 each | n/a | both |

Run each tree 3 times and report the median of the three p50s (and the largest p99). Master numbers = commit C1 (master + bench, no product change); HEAD = lane HEAD; both go into `impl.md` (developer) and `test_r1.md` (tester, own worktree). **Comparison rule (a breach is a defect unless impl.md shows a re-run within bounds):** R1, R2, R3: HEAD p50 ≤ 1.25 × master p50 + 100 ns. R4: HEAD p50 ≤ 1.25 × master + 0.5 µs, p99 ≤ 1.5 × master + 2 µs. R5: HEAD p50 ≤ master p50 (expected at least 10× lower). R6: HEAD p50 ≤ 1 ms and ≤ master p50 + 0.5 ms. R7: HEAD p50 ≤ master p50 + 20 µs, p99 ≤ 1.5 × master p99 + 50 µs. Justified extra work: R2/R4 the D5/D6 scans; R5 one `SetWaitableTimer`.
**Why.** `benches/` and `examples/` see only the public API, while `from_ron_str` (`loader.rs:222`) and the callback are `pub(crate)`. A file keeps the default run silent. R8 is opt-in because it loads every core for about 3 s on the owner's machine (normal priority, like a build).
**Cost.** One non-manual ignored test (Q5).

## Win32 API choices

| Purpose | `windows` path | Feature (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| gesture wake timer (reused) | `windows::Win32::System::Threading::{CreateWaitableTimerExW, SetWaitableTimer, CancelWaitableTimer}` through `platform::timers::WaitableTimer` | `Win32_System_Threading` (yes) | main; `WaitableTimer` is `!Send` | `new` Err → startup error before install; `set` Err → `start` returns false / plan drops + `[ERROR]` | https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer, https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createwaitabletimerexw |
| wait for timer + messages | `windows::Win32::UI::WindowsAndMessaging::MsgWaitForMultipleObjectsEx(Some(&[HANDLE]), INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE) -> WAIT_EVENT` (`WindowsAndMessaging/mod.rs:1795`) | `Win32_UI_WindowsAndMessaging` (yes) | main | `WAIT_FAILED` → `Err(Error::from_thread())`, fatal | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex |
| pump, create the queue | `PeekMessageW`, `TranslateMessage` (`:2402`), `DispatchMessageW` (`:575`) | same (yes) | main | none | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew |
| wake / quit from IPC | `PostThreadMessageW(u32, u32, WPARAM, LPARAM) -> Result<()>` (`:1872`) | same (yes) | IPC workers → main | wake error ignored (the request is drained at the next wake of any kind; the worker times out); quit error returned to swch | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew |
| LL hook delivery argument | — | — | — | — | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getqueuestatus |
| WinEvent hook with a MsgWait loop | — | — | — | — | https://learn.microsoft.com/en-us/troubleshoot/windows/win32/getmessage-ignores-posted-messages, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwineventhook ("For out-of-context events, the event is delivered on the same thread that called SetWinEventHook") |
| thread priority | `windows::Win32::System::Threading::{GetCurrentThread() -> HANDLE (:704), SetThreadPriority(HANDLE, THREAD_PRIORITY) -> Result<()> (:1706), GetThreadPriority(HANDLE) -> i32 (:961), THREAD_PRIORITY_HIGHEST (:3455), THREAD_PRIORITY_NORMAL (:3459)}` | `Win32_System_Threading` (yes) | main (pseudo handle, never closed) | Err → `[WARN]`, run at normal | https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setthreadpriority, https://learn.microsoft.com/en-us/windows/win32/procthread/scheduling-priorities, https://learn.microsoft.com/en-us/windows/win32/procthread/priority-boosts |
| tests and bench only: message-only window | `CreateWindowExW(.., w!("STATIC"), .., Some(HWND_MESSAGE), None, None, None) -> Result<HWND>` (`:430`), `DestroyWindow` (`:534`), `IsWindow(Option<HWND>) -> BOOL` (`:1335`), `PostMessageW(Some(hwnd), WM_CLOSE, ..)` (`:1857`), `SendMessageTimeoutW(hwnd, WM_NULL, .., SMTO_ABORTIFHUNG, 1000, None) -> LRESULT` (`:2039`) | `Win32_UI_WindowsAndMessaging` (yes) | test threads | asserted | https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features#message-only-windows, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendmessagetimeoutw |

No `Cargo.toml` change. No new `SendInput` path (Hold fast actions use the existing builders, sentinel included; `LLKHF_INJECTED`, the extended flag and `scanCode` play no part in this lane). No COM on the main thread. Elevation, UIPI and the pipe DACL are unchanged; requests travel in-process.

## Touch set

| File | Change | Why |
|---|---|---|
| `src/bench.rs` (new) | latency bench (D10) | spec |
| `src/lib.rs` | `#[cfg(test)] mod bench;` | D10 |
| `src/engine/schedule.rs` (new) | `Scheduler`, `TimerQueue` + tests | D4 |
| `src/engine/mod.rs` | `TimerToken { generation, slot, seq }`, `TimerSlot`, `Ids`, `is_live`, `start_timers`, `forget_timer`; `reset` bumps the generation; `on_key_down` flow (D5/D6); `on_timer` by slot; module docs (Time, Threads, Decision rules); unit tests | D4-D8 |
| `src/engine/gestures.rs` | rewritten: `Press`, phases, `DueHold`, `press`/`release`/`other_key_down`/`on_timer`/`is_live`/`forget`; unit tests rewritten (same cases where they still apply) | D5, D6 |
| `src/engine/winner.rs` | `GroupArming`, `Winner::deferred_group` + tests | D5 |
| `src/engine/chords.rs`, `src/engine/sequences.rs` | `seq`, expiry `TimerRequest`, removal by seq | D8 |
| `src/engine/testing.rs` | `TimerQueue` through `start_timers`; reset prunes; docs (no `hold-timer`) | D4 |
| `src/engine/scenarios.rs` | ports (C2), KS-25 / known issue 74 / Tap-wait (C8), known issue 67 flips (C10) | acceptance |
| `src/engine/properties.rs` | new last config `tap_double_tap` (CapsLock Tap(200)+Hold(400)+DoubleTap(300); LShift Tap(200)+DoubleTap(300), passing); doc list | D5 coverage; seeds of the earlier configs kept |
| `src/engine/env.rs` | `now` doc: main thread, no lock | D1 |
| `src/hook/main_thread.rs` (new) | D1-D3 + tests | spec |
| `src/hook/mod.rs` | `pub(crate) mod main_thread;` | D1 |
| `src/hook/keyboard_hook.rs` | remove `engine`, `with_engine`, `DECIDING`, `DecidingGuard`, `set_enabled`; add `switch_enabled` (`pub(crate)`, main thread only); `cancel_actions` `pub(crate)`; docs; tests migrated or removed (C7) | D1, D2 |
| `src/hook/keyboard_hook_callback.rs` | `decide_event` through `with_main` + `start_timers`, `pub(super)` (main-thread tests drive it with a local hook); `carry_out` `pub(super)` and takes `timers: _` (already started under the borrow; comment says so); remove `start_hold_timer`, `sleep_until`, `use std::thread`; `foreground_changed` through `with_main`; module docs (Threads, Re-entry); tests migrated + new | D1, D3 |
| `src/core/app.rs` | `MainThread::new()` before `install`; `DaemonHandle { hook, main: MainThreadHandle }`; loop → `main.run(hook)`; `RaisedPriority` (C16); tests migrated | D2, D3, D9 |
| `src/platform/priority.rs` (new), `src/platform/mod.rs` | `RaisedPriority`; module doc names both users of `timers` | D9 |
| `src/logging/counters.rs` | `HoldTimerNotStarted` doc only | D4 |
| `src/logging/mod.rs` | `the_functions_on_the_hook_path_log_nothing` list: `src/hook/main_thread.rs` `fn with_main(`, `fn start(`, `fn retain(` (one `fn start(` in that file: the startup function is `MainThread::new`) | hook path stays log-free |
| `README.md` | tree (`main_thread.rs`, `schedule.rs`, `priority.rs`, `timers.rs` text, `bench.rs`), `keyboard_hook.rs` bullets :121-122, callback :127/:133, app.rs :140-145, executor :151, triggers :391-414/:425-445/:465-470/:483-484, log :799, Technologies :856, Features :867 | meaning changes |
| `src/config/default_config.ron` | comments :123-128, :298-299, :314-319 (new text ASCII) | meaning changes |
| `CHANGELOG.md` | Unreleased (below) | Keep a Changelog |
| `book/src/config/triggers-conditions.md` | §11.4 :62-67, §11.5 :73-82, §11.6 :85-106, §11.7 :111-117, §11.8 :128-129 | config semantics |
| `book/src/config/known-issues.md` | rows 23 :61, 24 :62, 67 :135, 74 :142; new row 75 (By design: Tap waits for the DoubleTap window) | spec |
| `book/src/config/actions.md` | §13.1 :38 ("outside the engine lock"), :44 Hold row, :49 | thread facts |
| `book/src/config/errors.md` | §18.6 new rows (`[FATAL] could not create the gesture timer: …`, `[FATAL] the main message loop failed: …`, `[ERROR] could not set the gesture timer …`, `[WARN] gesture timers wake on the 15.6 ms system tick …`, `[WARN] could not raise the main thread's priority …`); counter text :203 | messages |
| `book/src/config/kinds.md` | :79, :143 (expiry timers) | D8 |

Not touched: `Cargo.toml`, `Cargo.lock`, `build.rs`, `src/core/executor/*`, `src/platform/timers.rs`, `src/hook/ipc_server.rs`, `src/hook/config_watcher.rs`, `tests/ipc_roundtrip.rs`, `CLAUDE.md` and the agent files (Q1, orchestrator).

**CHANGELOG `[Unreleased]`.** Changed: edit `:196-203` ("Hold actions still run on their own `hold-timer` thread" → gesture timers on the main thread); add "Gesture timers run on the main thread: no thread per Hold; deadlines are met within about 1 ms (high-resolution waitable timer)"; "`swch on` and `swch off` are carried out on the main thread; new error replies …"; "The main thread runs at `THREAD_PRIORITY_HIGHEST` while it pumps" (C16); "Chord and sequence entries expire on their own timers"; **Breaking:** "another key pressed while a `Tap`, `Hold` or `DoubleTap` whose key passes is pending cancels it (owner decision, roadmap item 2.2, known issue 67) …"; **Breaking:** "`Tap`, `Hold` and `DoubleTap` on one key are one gesture: `Tap(300)`+`Hold(200)` released at 250 ms fires only the Hold; a double press fires only the `DoubleTap`; with a `DoubleTap` on the same keys the `Tap` waits for its window (it fires at the press plus the term, or when another key goes down)". Fixed: edit KS-25 `:547-552` (every term pair) and KS-33 `:652-656` (a Hold no longer starts a thread; KS-33 closed); add known issue 74 (a `DoubleTap` entry no longer fires the action of another binding). Added: the latency bench.

## Commits, in order

Gate per commit: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` green, except the declared red commits (R). Trailer `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

1. **`Bench: a latency bench for the key path, the gesture timers and the message loop`** — `src/bench.rs`, `src/lib.rs`; rows R1-R7 on master's API. Run 3× → `D:/tmp/ks-orch/p2-2.2-timers/bench-master.md`. Green.
2. **`Tests: the gesture repro's stale-timer tests, ported with their assertions inverted`** — `engine::scenarios::stale_tap_gc_eats_second_quick_tap` (Tap(200) on F13, `DoNothing`: down 0, up 20 fires, down 120, advance to 230, up → `expect_fired(&[DoNothing])`, total 2) and `engine::scenarios::stale_hold_timer_fires_new_press_early` (Hold(200) on F14: down 0, up 20, down 120, advance to 230 → nothing, to 319 → nothing, to 320 → fires, up). Doc comment: ported from `methodology.md#gesture-repro-crate`; the name is the defect guarded against. Green on master (KS-11 fixed in `16a0461`). **Red only at `7cb3521`:** the tester replays the repro crate of `methodology.md:236-290` against a `7cb3521` worktree with the asserts inverted to `assert!(fired)` / `assert!(still_pending)`; PREDICTED: both panic at those asserts (`second tap fired = false`, `press#2 hold still pending at +110ms = false`). Safe: `TapState`'s threads only run `DoNothing`.
3. **(R) `Tests: timer tokens, the timer queue and the Scheduler seam`** — `engine::schedule::tests::the_queue_hands_out_due_timers_earliest_first_and_ties_in_start_order`, `engine::schedule::tests::retain_keeps_only_live_timers`, `engine::tests::a_token_from_before_a_reset_is_stale`, `engine::tests::a_cancelled_hold_leaves_its_token_stale`, `engine::tests::a_timer_the_scheduler_refuses_is_forgotten` (a test-local refusing `Scheduler`). PREDICTED RED: compile, `E0432 unresolved import` (`super::schedule`, `crate::engine::{Scheduler, TimerQueue}`), `E0599 no method named is_live` / `start_timers` on `Engine`.
4. **`Engine: timer tokens carry a generation, a slot and a sequence number; a Scheduler trait and a timer queue`** — `engine/mod.rs`, `engine/schedule.rs`, `engine/gestures.rs` (Hold `generation` → `seq`; token shape only), `engine/testing.rs` (TimerQueue). Behaviour-neutral: every existing test green; commit 3 green. `keyboard_hook_callback.rs` untouched (the `TimerRequest` shape is kept).
5. **(R) `Tests: no thread is started on the gesture path`** — `hook::keyboard_hook_callback::tests::the_gesture_path_starts_no_thread`: the non-test lines (up to the `#[cfg(test)]` + `mod tests {` pair; the whole file if it has none) of every `src/hook/*.rs` except `ipc_server.rs` and `config_watcher.rs`, and of every `src/engine/*.rs`, for `thread::spawn`, `thread::Builder` and `use std::thread`, the tokens assembled with `concat!`. PREDICTED RED (assertion): `threads started on the gesture path:` listing `src/hook/keyboard_hook_callback.rs:91: use std::thread;` and `src/hook/keyboard_hook_callback.rs:312: let spawned = thread::Builder::new()`.
6. **(R) `Tests: the main thread owns the engine, waits on its gesture timer and its messages at once, and answers swch on and off`** — new `src/hook/main_thread.rs` holding only its `#[cfg(test)] mod tests`, plus the `hook/mod.rs` line:
   - `with_main_refuses_reentry_on_the_same_thread` (nested → `None`), `with_main_is_none_without_a_main_state`;
   - `plan_sets_the_timer_only_for_another_deadline` (pure; mirror of `executor/mod.rs:1017-1049`);
   - `a_timer_started_for_an_earlier_deadline_resets_the_wake` (`GestureTimers::start` updates `armed`; a later one leaves it);
   - `a_message_posted_behind_a_wake_is_dispatched_without_more_input` (m-1): a scratch thread creates a `STATIC` message-only window and a `MainThread`, posts `MAIN_WAKE` to itself and `WM_CLOSE` to the window, then runs; the test thread polls `IsWindow` for up to 1 s, then posts `WM_QUIT`; asserts the window closed before the watchdog. Without `MWMO_INPUTAVAILABLE` the wait blocks: `pump` took `MAIN_WAKE` and `WM_CLOSE` is no longer new input;
   - `a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer`: the loop thread holds a gesture timer 10 s out; 100 × `SendMessageTimeoutW(WM_NULL, SMTO_ABORTIFHUNG, 1000)` all succeed, median < 2 ms (a wait blind to sent messages would time out at 1 s);
   - `a_200_ms_hold_fires_within_a_few_ms_of_its_deadline`: local hook `F13 Hold(200) → Sequence([Key("F13")])` with `allow_extra_modifiers`, FakeRunner executor; 5 presses through `decide_event`, the loop until the record (a helper posts `WM_QUIT`), then the key-up; every lateness ≥ 0, median < 5 ms (reason as `executor/mod.rs:922-928`: a tick wake is about 7.8 ms late on average, the high-resolution timer under 1 ms, the median absorbs one scheduler hiccup);
   - `remapping_switches_through_the_main_thread_and_drops_pending_timers` (handle → `Ok(true)`, repeat → `Ok(false)`; a Hold armed before `off` never fires; `is_enabled` follows);
   - `a_switch_request_to_an_ended_main_thread_fails_at_once` (`ShuttingDown` within 100 ms);
   - in `keyboard_hook_callback.rs` tests: `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued` (extern callback, `GLOBAL_HOOK`, F13 Hold(10 000) down/up × 1000; queue length ≤ 1, no pending press); `remapping_off_callbacks_pass_without_asking_for_the_engine` (successor of `:620-688`: with the main state borrowed and remapping off, both callbacks pass and `REFUSED` is unchanged; with remapping on, the keyboard callback raises `REFUSED` by 1, the control); `a_hold_runs_its_slow_action_on_the_executor_from_the_main_loop` (successor of `:846-878`);
   - `core::app::tests::daemon_handle_switches_remapping_through_the_main_thread` (successor of `app.rs:341-363`).
   PREDICTED RED: compile, `E0425`/`E0433`/`E0412` for `with_main`, `MainThread`, `GestureTimers`, `plan`, `MAIN_WAKE`, `REFUSED`; `E0599 no method named switch_enabled`.
7. **`Hook: the engine and its gesture timers live on the main thread`** — `main_thread.rs` (D1-D3), `keyboard_hook.rs`, `keyboard_hook_callback.rs`, `app.rs`, `env.rs`, `engine/mod.rs` docs, `logging/mod.rs` scan list, `counters.rs` doc, `bench.rs` (R4-R7 HEAD setup), README (structure and thread bullets, Technologies, Features, log :799), CHANGELOG (edit :196-203, main-thread timers, swch on/off; Fixed KS-33), `actions.md` §13.1, `errors.md` §18.6, `triggers-conditions.md` §11.4 :67 and §11.5 :77 (thread facts only). Test migrations (no assertion weakened): `keyboard_hook.rs:435-456` → the `main_thread` re-entry tests; `:458-472`, `:493-529`, `:580-590` → `MainState::set_enabled` on the test thread; `:623-638` removed (no nested path exists: `set_enabled` takes the state by `&mut`); `keyboard_hook_callback.rs:451-457` (`RemappingOnAtEnd` through `with_main`), `:543-563`, `:565-618`, `:690-708` install `MainThread::new()` on the test thread; `:620-688` and `:846-878` replaced by their successors. Makes commits 5 and 6 green. Invariants: sentinel check before `with_main`; `code < 0` and remapping-off gates unchanged; every path ends in `LRESULT(1)` or `CallNextHookEx`.
8. **(R) `Tests: Tap, Hold and DoubleTap on one key resolve as one press, and a DoubleTap entry belongs to its group`** (`engine::scenarios`):
   - `tap_300_and_hold_200_released_at_250_fire_only_the_hold` → PREDICTED `up CapsLock at +250 ms: expected to fire [], fired [SwitchLanguage]`;
   - `tap_100_and_hold_200_released_at_150_fire_nothing` (green; pins the gap row);
   - `a_hold_closes_the_double_tap_window` (F13 Hold(200)+DoubleTap(300): down 0, Hold at 200, up 250, down 280) → `down F13 at +280 ms: expected to fire [], fired [ToggleCapsLock]`;
   - `a_double_tap_press_arms_no_hold` (DoubleTap(300)+Hold(200): down 0, up 50, down 100 fires, advance to 400) → `advance 300 ms to +400 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(160))]`;
   - `tap_and_double_tap_on_one_key_wait_for_the_double_tap_window_before_the_tap` (Tap(200)+DoubleTap(300): down 0, up 80 nothing, 299 nothing, 300 fires) → `up F13 at +80 ms: expected to fire [], fired [SwitchLanguage]`;
   - `a_double_press_of_a_tap_and_double_tap_key_fires_only_the_double_tap` → the same first failure;
   - `another_key_fires_a_tap_that_waits_for_the_double_tap_window` (A down at 120 fires the Tap on that step and passes; F13 again at 200 is a new press, its Tap fires at 500) → the same first failure;
   - `a_stale_hold_timer_does_not_fire_a_tap_waiting_for_the_double_tap_window` (Tap(100)+Hold(200)+DoubleTap(300): up 50, nothing at 200, Tap at 300) → `up F13 at +50 ms: expected to fire [], fired [SwitchLanguage]`;
   - `a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins` (known issue 74: CapsLock DoubleTap(300) SwitchLanguage + Shift+CapsLock DoubleTap(300) ToggleCapsLock; CapsLock down 0 / up 40, LShift down 60, CapsLock down 80 → nothing; up 120; down 160 → ToggleCapsLock) → `down CapsLock at +80 ms: expected to fire [], fired [SwitchLanguage]`;
   - `a_tap_released_after_the_double_tap_window_closed_fires_at_its_key_up` (green).
9. **`Engine: one gesture machine per press; Tap waits for the DoubleTap window; DoubleTap entries belong to their group (KS-25, known issue 74)`** — `gestures.rs`, `winner.rs`, `engine/mod.rs`, `properties.rs` (new config last), unit tests; docs: README :391-405, :425-445, :483-484; `default_config.ron` :123-128 and :298-299; CHANGELOG (Breaking exclusive machine; Fixed KS-25 edit, known issue 74); `triggers-conditions.md` §11.5-§11.7; `known-issues.md` rows 24 and 74, new row 75. Green; commit 8 green; `tap_and_hold_with_equal_terms_fire_exactly_one`, `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one`, `hold_left_to_a_late_timer_does_not_block_the_next_press`, `reload_during_a_passing_double_tap_keeps_each_press_whole`, `four_quick_presses_double_tap_twice` and `two_double_taps_on_the_same_keys_fire_only_the_first_on_the_second_press` unchanged and green.
10. **(R) `Tests: another key-down cancels a pending gesture whose key passes (known issue 67)`**:
    - `a_key_typed_between_two_passing_lshift_presses_still_completes_the_double_tap` renamed `…_cancels_the_double_tap`; the second LShift down `expect_nothing_fired`, total `[]` → PREDICTED `down LShift at +150 ms: expected to fire [], fired [ToggleCapsLock]`;
    - `a_quick_ctrl_shortcut_still_fires_a_passing_ctrl_tap` renamed `a_quick_ctrl_shortcut_cancels_a_passing_ctrl_tap`; up LCtrl `expect_nothing_fired` → `up LCtrl at +90 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(27))]`;
    - `selecting_words_with_ctrl_shift_held_cancels_a_passing_ctrl_shift_hold` (Ctrl+Shift Hold(1000): LCtrl, LShift, Right down/up, advance past the term → nothing) → the `expect_nothing_fired` after the term fails with `fired [SwitchLanguage]`;
    - `a_lone_passing_ctrl_tap_still_fires`, `another_key_does_not_cancel_a_swallowed_double_tap` (green).
11. **`Engine: another key-down cancels a pending gesture whose key passes`** — `gestures.rs::other_key_down`, `engine/mod.rs`; docs: README :407-414 and :465-470; `default_config.ron` :314-319; CHANGELOG (Breaking); `triggers-conditions.md` §11.4 :63 and §11.8 :128-129 (warning → fixed note); `known-issues.md` row 67. Green; commit 10 green; `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing`, `lshift_held_with_its_double_tap_bound_types_a_capital_a` and `modifier_only_hold_and_tap_pass_their_keys_by_default` green.
12. **(R) `Tests: chord and sequence entries ask for their own expiry timers`** — `engine::tests::a_chord_start_asks_for_a_timer_at_the_end_of_its_window`, `engine::tests::a_chord_expiry_timer_drops_the_pending_chord`, `engine::tests::each_sequence_step_asks_for_a_new_expiry_timer_and_the_old_one_goes_stale`, each starting `let [timer] = decision.timers[..] else { panic!("one expiry timer, got {:?}", decision.timers) };`. PREDICTED: `one expiry timer, got []`.
13. **`Engine: chord and sequence entries expire on their own timers`** — `chords.rs`, `sequences.rs`, `engine/mod.rs`; `kinds.md` :79 and :143; CHANGELOG line. Green.
14. **`Bench: sent-message round trips under CPU load at normal and highest priority`** — R8 in `bench.rs` (opt-in; the bench calls `SetThreadPriority` itself). Run once with `KS_BENCH_LOAD=1` → `bench-head-load.md`; apply D9's rule. Green.
15. **(R) `Tests: the priority guard raises the thread to THREAD_PRIORITY_HIGHEST and restores it`** — `platform::priority::tests::raise_sets_highest_and_drop_restores_normal` (scratch thread; `GetThreadPriority` = 2 inside, 0 after the drop). PREDICTED: compile, `E0432 unresolved import crate::platform::priority`. Skipped together with 16 if R8 contradicts D9.
16. **`Core: the main thread pumps at THREAD_PRIORITY_HIGHEST`** — `platform/priority.rs`, `platform/mod.rs`, `app.rs`; README app.rs bullet; CHANGELOG; `errors.md` row. Green.

After 16 (or 14): run the bench 3× at HEAD → `bench-head.md`; numbers and the D10 comparison go into impl.md, with the non-config book pages listed below.

## Acceptance mapping

| Roadmap / spec acceptance | Covered by |
|---|---|
| repro tests ported with inverted assertions pass; red at 7cb3521 | `engine::scenarios::stale_tap_gc_eats_second_quick_tap`, `…::stale_hold_timer_fires_new_press_early` (C2, green from C2 on); red = the tester's 7cb3521 replay (C2 note) |
| Tap(200)+Hold(200): release at 199 → only Tap, at 201 → only Hold | `engine::scenarios::tap_and_hold_with_equal_terms_fire_exactly_one` (`scenarios.rs:451-476`) and `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` (`:502-532`), green at every commit; different terms: the C8 tests |
| no thread per press over 1000 presses | `the_gesture_path_starts_no_thread` (C5, red on master) + `a_thousand_hold_presses_through_the_adapter_keep_at_most_one_timer_queued` (C6) |
| real-timer 200 ms deadline within a few ms, CI-safe bound | `hook::main_thread::tests::a_200_ms_hold_fires_within_a_few_ms_of_its_deadline`: never early, median of 5 < 5 ms (reason in C6) |
| another key-down cancels a pending pass-through gesture; the two known-issue-67 scenarios flip | the C10 tests; swallowed kept: `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing`, `another_key_does_not_cancel_a_swallowed_double_tap` |
| the known-issue-74 case fires no first-binding action | `a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins` |
| bench runs; master and HEAD numbers in impl.md and test_r1.md; no unjustified hook-path regression | `bench::latency_bench` at C1 and at HEAD; D10 comparison rule |
| timers on the main thread in the same queue as key events; tokens; a stale token does nothing | C3/C4 tests, `a_cancelled_hold_leaves_its_token_stale`, `a_token_from_before_a_reset_is_stale`, `a_stale_hold_timer_does_not_fire_a_tap_waiting_for_the_double_tap_window` |
| deadlines not rounded to 15.6 ms; hook callbacks delivered without added delay | `a_200_ms_hold_fires…`, `a_message_sent_from_another_thread_is_handled_while_the_loop_waits_on_its_timer`, bench R6/R7; `platform::timers::tests::a_timer_signals_once_after_its_due_time` (high resolution present) |
| drain before every wait; `MWMO_INPUTAVAILABLE` pinned (m-1) | loop structure (D3, review); `a_message_posted_behind_a_wake_is_dispatched_without_more_input` |
| `Mutex<Engine>` gone; fast gate kept; safe re-entry | `rg "Mutex<Engine>\|with_engine" src` empty (review); `remapping_off_callbacks_pass_without_asking_for_the_engine`; `with_main_refuses_reentry_on_the_same_thread` |
| Tap waits for DoubleTap only when both; latency declared | the C8 Tap-wait tests; D5 cost; README, §11.6, known-issues row 75 |
| Chord and Sequence expiry timers | the C12 tests |
| Scheduler trait; scenarios drive time through the fake | `engine::schedule::tests::*`; `Scenario` uses `TimerQueue` + `start_timers` |
| `on_timer` never masks | `a_hold_timer_never_masks` + the harness assert (`testing.rs:415`), now also for waiting-Tap and expiry timers |
| thread priority evaluated with evidence | D9 + bench R8 (C14) + `raise_sets_highest_and_drop_restores_normal` (C15), or the impl.md reason |
| R-7 (hold-timer half) closes | C5 scan (no thread on the gesture path), D1 |

## Manual checks (human only)

Release daemon started with `swch open` from a normal console; Notepad focused; `osk.exe` open to see held modifiers. None of these has an ignored test; they stand alone.

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M1 | The CapsLock binding replaced by the README `Tap(200)` + `Hold(200)` pair | 20 quick CapsLock taps; then hold 1 s | 20 switches; one LShift press after 200 ms with no input stall; nothing stuck (Checkpoint 2 item 1) | none |
| M2 | The template's `LShift` `DoubleTap(250)` uncommented | Type `<T>` with the left Shift for both; then double-tap the left Shift alone | `<T>` typed and CapsLock unchanged (known issue 67 fixed); the lone double tap toggles CapsLock | none |
| M3 | `( keys: "LCtrl", action: PressKey("A"), trigger: Tap(200) )` | Select text, quick Ctrl+C; then a lone quick Ctrl tap | Ctrl+C copies and types nothing; the lone tap types `a` | none |
| M4 | `( keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000) )` | Hold Ctrl+Shift and press Right 5 times over 2 s; then hold Ctrl+Shift alone for 1.5 s | Words selected, layout unchanged; the lone hold switches once | none |
| M5 | `( keys: "F9", action: PressKey("A"), trigger: Tap(200) )` + `( keys: "F9", action: PressKey("B"), trigger: DoubleTap(300) )` | Tap F9; double-tap F9; tap F9 then type `x` at once | `a` about 0.3 s after the press; only `b`; `ax` in that order | none |
| M6 | `( keys: "CapsLock", action: PressKey("A"), trigger: DoubleTap(300) )` + `( keys: "Shift+CapsLock", action: PressKey("B"), trigger: DoubleTap(300) )` | CapsLock, then Shift+CapsLock quickly; then Shift+CapsLock twice | Nothing typed; then `B` (known issue 74) | none |
| M7 | Default config, daemon elevated | `swch off`, CapsLock, `swch on`, CapsLock, `swch status`, `swch exit` | Every command answers within 1 s; CapsLock reaches Windows only while off; the window closes without `[FATAL]` | none |
| M8 | `( keys: "F9", action: PressKey("A"), trigger: Hold(5000) )` | Hold F9; press Ctrl+C in the daemon console within 2 s | The daemon exits at once, no `[FATAL]`, nothing typed | none |
| M9 | Default config | 30 s of typing with frequent Alt+Tab and one Win+L and unlock | No stuck modifier in `osk.exe`; no `[ERROR]` in `daemon.log` (WinEvent delivery through the new loop) | none |
| M10 | Default config, a `cargo build --release` of any crate running (all cores busy) | Type in Notepad and tap CapsLock; Process Explorer → key-switch-rs.exe → Threads | Typing as responsive as when idle; the main thread shows base priority 10 (C16) | none |

The 5 existing `manual:` ignored tests are unchanged. The new ignored test is `bench::latency_bench` (`bench:` reason): not a manual check and never counted as a pass.

## Risks and edge cases

- **R1 hook callback.** `with_main` replaces the lock; the sentinel check stays before it (`keyboard_hook_callback.rs:187-189`); the `code < 0` and remapping-off gates are unchanged; no `expect` or indexing panic is added (`try_borrow_mut`). New per-event work: TLS + RefCell flag, the D6 scan, the D5 pre-pass for a deferred winner; per timer-arming event `retain` and at most one `SetWaitableTimer`. New allocation: only the `GroupKey` clone on a DoubleTap arm, and `TimerQueue` growth past 16 live timers.
- **R2 sentinel.** No new `SendInput` builder. Hold fast actions call `SendInput` on the main thread outside a callback; the system calls our hook for those events while the thread is inside `SendInput` (`keyboard_hook_callback.rs:65-71`); they carry the sentinel and pass before `with_main`, and `carry_out` runs after the borrow, so a physical event dispatched meanwhile finds the state free. M1 checks that there is no stall.
- **R3 shared state.** `Mutex<Engine>` removed; `MAIN` is a thread-local `RefCell`, not shared (Q1). `KeyboardHook` keeps `ArcSwap`, atomics and `ArcSwapOption`.
- **R4 orderings.** `enabled`: `swap(AcqRel)` only on the main thread (`switch_enabled`), `load(Acquire)` in the callbacks and in IPC `status`; it publishes no data (the engine state is thread-owned). No new atomics, no `Relaxed`, no `SeqCst`, no `static mut`. Requests and replies travel through `mpsc`, which synchronises.
- **R5 builders.** No new `Binding` option; `MainThread::new`, no setters.
- **R6 Display.** `MainThreadError` has one arm per variant; `TimerSlot` is `Debug` only and never displayed.
- **R7 specificity.** `update_bindings` unchanged; `deferred_group` walks `bindings[winner.index..]` in list order (`GroupArms` keeps the first of each kind).
- **R8 key identity.** Presses are keyed by the event's sided vk; "another key" is a different vk (LShift and RShift are different keys); group identity uses `KeyCombination ==` as written (sided `LShift` is not `Shift`, as in the group rule).
- **R9 registry.** None.
- **R10 loop and shutdown.** `WM_QUIT` (Ctrl+C, `swch exit`) is taken by `PeekMessageW` ("PeekMessage always retrieves WM_QUIT messages") and ends `run`; then the priority guard drops, `uninstall` runs on the main thread, the executor stops, and `MainThread` drops (pending requests → `ShuttingDown`). `WAIT_FAILED` → hooks removed, `[FATAL]`. Startup: `MainThread::new` (timer, queue) before `install`, so a failure leaves nothing to undo (KS-20 order kept). No spin path: each `Plan::Due` iteration removes a due timer.
- **Callback clock.** The engine's `now` is the callback time (`HookEnv::now`, `keyboard_hook_callback.rs:143-145`), not `KBDLLHOOKSTRUCT.time` (tick-granular). A key-up delayed past a Hold's term by a busy main thread counts as a hold, before and after this lane; exclusivity holds either way, and D9 shortens the window. Follow-up.
- **Timer and input ready together.** Microsoft documents the handle order of the wait only for `dwFlags` 0; with `MWMO_INPUTAVAILABLE` either may be reported first. Both orders are correct: a due Hold fires in the drain, a key-up at or after the term leaves it to its timer (D5), and the engine decides by `Instant`.
- **Reload, `swch off`/`on`, foreground change, shutdown mid-gesture.** A reload keeps pending presses with their actions (unchanged; 2.3). `off`/`on` reset and bump the generation, so every token goes stale (`remapping_off_drops_a_pending_hold` and its siblings stay). Foreground change: unchanged semantics. Shutdown: nothing fires (M8).
- **Repeats.** Repeats never reach `press` or `other_key_down`; the `check_repeats_fire_only` property stays valid, and the new last config adds Tap+DoubleTap coverage.
- **Pre-1803 Windows.** Plain timer fallback, one `[WARN]`.
- **CI timing.** New real-time tests: about 1 s (200 ms × 5), at most 1 s (MWMO watchdog), 100 round trips. Bounds as stated; a flake goes to the orchestrator, with no loosening without a ruling (the 2.1 rule).
- **Residual R-7.** The config-watcher's `update_bindings → cancel_actions` can still post `WAKE` to an ended executor at exit (2.1 `cut.md:501`); the hold-timer half is gone.
- **Neighbouring defects (follow-ups).** `config_watcher.rs:126` `.expect("spawn config-watcher thread")` (startup, KS-33-like); the executor's m-1 pin (3.4); a re-entry refusal counter (3.4); the KS-25 loader warning for different terms (3.4).

## Non-config book pages for the Phase-2 book sync (the developer copies this into impl.md)

`architecture.md:121`, `:126`, `:148-150`, `:163-166`, `:181`, `:187`, `:191`, `:193-200`, `:215`, `:225`, `:249`, `:271`, `:306-307`, `:357-389`, `:397`, `:405-409`, `:426`, `:563`, `:588`; `overview.md:68`, `:89`, `:223`; `glossary.md` entries Counters `:121-125`, Decision `:143-145`, Enabled flag `:169-171`, Engine `:173-175`, Generation token `:201-203`, Gesture group `:209-213`, Hold `:223-225`, Hook thread `:231-233`, KeyboardHook singleton `:275-277`, Message loop `:331-333`, Tap, Hold and DoubleTap `:457-459`, Thread affinity `:465-467`; `build-and-run.md:141` (counter text), `:229-230` (new `on`/`off` reasons), `:253-265`; `roadmap.md:28`, `:122`, `:404` (Phase-1 manual check 9: `<T>` no longer toggles), `:417-428`, `:458-477` (2.2 status; "Files" reuses `platform/timers.rs`; R-7 and m-1 notes), `:787`, `:792`, `:805`, the issue-coverage rows for KS-11, KS-25 and KS-33; `audit.md:364` (KS-11), `:574` (KS-25 → Fixed), `:656` (KS-33 → Fixed); `gaps.md:30`, `:68-91`.

## Questions for the orchestrator

1. CLAUDE.md principle 5 and code-reviewer R3 ban `RefCell` for hook state; D1 puts the engine in a thread-local `RefCell` reached only through `try_borrow_mut`, the form roadmap 3.4 prescribes. Recommendation: accept; tell the critic and the reviewer that `hook::main_thread::MAIN` is not shared state (D1 governs), and update CLAUDE.md principle 5, R3/R4 and the agent files after the merge, as after 2.1.
2. D6 makes another key-down fire a Tap that waits for the DoubleTap window and close that window, for swallowed groups too. Recommendation: keep it (output order and exclusivity force it, and it reproduces today's Tap-before-the-next-key order). The owner can veto later with a change confined to `other_key_down`.
3. D9 picks `THREAD_PRIORITY_HIGHEST`, not AutoHotkey's `TIME_CRITICAL`. Recommendation: HIGHEST (Microsoft: a base priority "above 11 interferes with the normal operation of the operating system"); C15/C16 land only if R8 does not contradict it.
4. The `Tap(100)+Hold(200)` gap (a release between the terms fires nothing) stays, documented. Recommendation: a loader warning for unequal terms in 3.4's diagnostics, not here.
5. `bench::latency_bench` is an ignored test with a `bench:` reason. Recommendation: the tester and merge reports count it apart from `manual:` tests and never as a pass; the ks-lane and tester briefs name the D10 command.
6. The ported KS-11 tests are green on master; their red is the 7cb3521 repro replay. Recommendation: accept this as the acceptance line's "where a red on master needs a seam master lacks" case.
7. The executor's own `MWMO_INPUTAVAILABLE` stays unpinned (m-1). Recommendation: 3.4 moves both loops' wait and pump into `platform` with one pinning test.

## Readiness checklist

- [x] KS-11, KS-25, KS-33 read (`audit.md:361-373`, `:571-583`, `:654-659`); scenarios reproduced (C2, C8, C5/C6).
- [x] Book references re-located (table above).
- [x] Goal, non-goals and dependants stated.
- [x] Every decision has a rationale and rejected alternatives.
- [x] Owner decisions: exact modifiers untouched; repeat defaults untouched (D6 ignores repeats); RON unchanged; `log` + sink (the loop logs, the hook path does not); licence N/A; latency first (D3, D9, D10); open question 10 (D6).
- [x] Thread named for every path (main: callbacks, loop, timers, requests; IPC workers: requests only; executor unchanged); nothing heavy in the callback.
- [x] Install, uninstall and timers on the main thread.
- [x] No new `SendInput` path; sentinel unchanged; repeats and key-ups paired as before.
- [x] Sided and unsided keys consistent (R8).
- [x] Reload, `swch off`/`on`, foreground change and shutdown during a pending gesture covered (Risks, M8).
- [x] Orderings stated; no `static mut`.
- [x] No new module cycle (`hook → engine/platform/core` edges exist; `engine::schedule` is pure).
- [x] Win32 table with paths checked in the `windows-0.62.2` sources, features present, links.
- [x] Elevation, UIPI, DACL and token: unchanged, noted.
- [x] Red-first with predicted failures; nothing installs a hook or injects input.
- [x] Gates per commit; red commits declared; docs in the commits that change the meaning.
- [x] Every acceptance line mapped; every question has a recommendation.

## Developer addendum

Implemented on `feat/main-thread-timers` (18 commits, `611c83f..d738632`), as cut and amended by
`rulings.md`. Where this addendum and the cut differ, the addendum states what the code does.

### Rulings, as implemented

- **I1 (a Hold past its term is resolved by the clock).** `Gestures::other_key_down`
  (`src/engine/gestures.rs`) cancels a held press of a passing key only if
  `!hold.is_some_and(|(due, _)| now >= due)`; a press whose Hold is due by the clock stays for its
  timer, as a `DueHold` does. Red first: the cancel rule without the clock check turned
  `engine::scenarios::a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down`
  red ("advance 0 ms to +1001 ms: expected to fire [SwitchLanguage], fired []"); with it, green
  (the `advance_ms(1001)` script and the `advance_ms_stalled(1001)` script both fire the Hold once).
  Every other timer-driven state that a late timer could flip, checked:
  1. Held press, Hold due, key-up before the timer runs: `release` leaves a `DueHold` (D5, as cut).
  2. Held press, Hold due, another key-down before the timer runs: kept (the I1 rule above).
  3. Held press of a key whose key-up was lost, Hold due by the clock, next press of that key:
     `press` keeps it as a `DueHold` for its timer (ruling M6(b), below).
  4. A Tap waiting for a DoubleTap window that has closed by the clock, timer not run yet: its own
     key-down is no DoubleTap (`now < due` decides) and fires the Tap first; another key-down fires
     the Tap ahead of that key. Either way the outcome is the timer's, only earlier.
  5. A DoubleTap window without a Tap, closed by the clock: its own key-down is no DoubleTap; another
     key-down cancels or keeps an entry that is dead anyway. No timer exists for it.
  6. Chord and sequence expiry timers: the lazy `expires_at > now` check stays the authority (D8), so
     a late expiry timer changes no match.
- **M1.** Every test that reached the engine, `set_enabled` or `DaemonHandle` is migrated or replaced,
  with no assertion weakened:
  - `keyboard_hook.rs`: `with_engine_refuses_reentry_on_the_same_thread` and
    `with_engine_works_again_after_a_call` are replaced by
    `hook::main_thread::tests::with_main_refuses_reentry_on_the_same_thread` and
    `with_main_works_again_after_a_call`; `toggling_reports_changes_and_never_installs_a_hook`,
    `turning_remapping_off_drops_a_pending_hold_and_a_repeated_on_keeps_it`,
    `swch_off_stops_a_running_macro_within_100_ms` and `a_repeated_swch_on_keeps_the_running_macro`
    keep their names and go through `MainState::set_enabled` on a `MainThread` of the test thread;
    `set_enabled_refuses_a_nested_call` keeps its name and asserts that a switch inside `with_main`
    is refused (`Some(None)`) and leaves remapping on.
  - `keyboard_hook_callback.rs`: `RemappingOnAtEnd` goes through `with_main`;
    `physical_key_events_reach_the_engine`, `injected_and_unknown_events_bypass_the_engine`,
    `disabled_hook_passes_every_event_without_asking_the_engine`,
    `extern_callback_swallows_a_bound_key_only_while_remapping_is_on` and
    `the_keyboard_callback_records_how_long_it_ran` keep their names, with `MainThread::new()` on the
    test thread; `remapping_off_callbacks_pass_without_waiting_for_the_engine_lock` is replaced by
    `remapping_off_callbacks_pass_without_asking_for_the_engine`; `a_hold_runs_its_slow_action_on_the_executor`
    by `a_hold_runs_its_slow_action_on_the_executor_from_the_main_loop`.
  - `app.rs`: `daemon_handle_status_follows_the_hook_flag` is replaced by
    `daemon_handle_switches_remapping_through_the_main_thread`;
    `daemon_handle_refuses_exit_without_a_main_thread` keeps its name, with a test-only
    `MainThreadHandle::without_a_thread()` (thread id 0, closed queue), and also asserts that `on`
    and `off` then answer "the daemon is shutting down".
  - `gestures.rs` unit tests are rewritten for the press machine, the same cases where they still
    apply (mapping table in `impl.md`).
- **M2.** Declared in `known-issues.md` row 75, in `triggers-conditions.md` §11.6 (the note after the
  F9 example) and in the CHANGELOG Breaking line: a waiting Tap fires into the window that has the
  focus when it fires, without checking `when:` again, with the modifiers held at that moment.
- **M3.** `known-issues.md` row 67 ("Fixed for key-downs"; remaining: a mouse click does not cancel),
  the `triggers-conditions.md` §11.8 note, the README behaviour rules and the CHANGELOG.
- **M4.** (a) The reply channel is a `sync_channel(1)`, and `serve` answers with `try_send`, ignoring a
  failure: the main thread never blocks on it. (b) A request that timed out at the worker still runs
  when the main thread reaches it; declared on `MainThreadError::NoAnswer` and in the CHANGELOG reply
  text. (c) `App::run` drops the `MainThread` right after `main.run(hook)` returns, before the hooks are
  removed and the executor stops. The request queue closes with it, so a later `on` or `off` fails at
  once with "the daemon is shutting down" (`a_switch_request_to_an_ended_main_thread_fails_at_once`,
  under 100 ms).
- **M5.** Bench row R8 measures one-way delivery at the receiver: the window procedure of a STATIC
  message-only window, replaced with `SetWindowLongPtrW`, stamps the probe on entry, against a stamp
  taken before the send. 300 sends 1 ms apart per priority, run 3 times. Margin: HIGHEST counts as worse
  iff `p99(HIGHEST) > 1.25 x p99(NORMAL) + 20 us`. Result: 0 of 3 runs (numbers in `impl.md`), so
  commits 15 and 16 landed.
- **M6.** (a) A refused window timer keeps its press: `Gestures::forget` drops only a Hold. The waiting
  Tap fires at the next first key-down of any key (`other_key_down`, or the key's own `press`), or with
  the next gesture timer handed back whose due time is at or after the window's end
  (`Gestures::on_timer` fires every closed window first). Pinned by
  `engine::scenarios::a_tap_whose_window_timer_is_lost_fires_at_the_next_key_down` and
  `engine::tests::a_tap_whose_window_timer_is_refused_fires_when_a_later_timer_wakes`. Key-ups and
  auto-repeats of other keys do not fire it (D6: they change nothing; the property
  `check_repeats_fire_only` stays valid). The refusal is counted in `hold_timer_not_started`. (b)
  `press(k)` with a stale held press of `k` (a lost key-up) replaces it; its Hold, if due by the clock,
  stays as a `DueHold` (I1). Pinned by
  `a_press_after_a_lost_key_up_replaces_the_stale_press_and_keeps_a_hold_due_by_the_clock`.
- **M7.** Commit `7cb6868`: under `cfg(test)`, `FastAction::run` records into a thread-local
  (`windows_actions::fast_actions_run`) and reaches no Win32 call; `press_vk` and
  `post_message_to_foreground` carry `cfg_attr(test, allow(dead_code))` with the reason. Pinned by
  `under_test_a_fast_action_is_only_recorded`, whose action is `PostMessage(WM_NULL)`, harmless if it
  ever reached Windows. Not run red first: before the change that test posts WM_NULL to the foreground
  window.
- **M8.** `a_200_ms_hold_fires_within_a_few_ms_of_its_deadline` and bench R6 read the press time
  before the key-down reaches the engine; the test asserts every lateness is >= 0.
- **OQ1.** Left to the tester (QS_ALLEVENTS). Done here for the MWMO pin only: with
  `MWMO_INPUTAVAILABLE` replaced by 0, `a_message_posted_behind_a_wake_is_dispatched_without_more_input`
  failed ("the WM_CLOSE posted behind a MAIN_WAKE was not dispatched within 1 s"); restored.
- **OQ2 / Q2.** D6 as read by the cut: a waiting Tap fires on another key-down, for swallowed groups too.
- **OQ4.** `the_gesture_path_starts_no_thread` scans the non-test part of `src/hook/*.rs` (except
  `ipc_server.rs` and `config_watcher.rs`), `src/engine/*.rs`, `core/windows_actions.rs`,
  `system/system_function.rs` and `platform/timers.rs` for `thread::spawn`, `thread::Builder`,
  `thread::scope` and `use std::thread`.
- **OQ5.** Feasible: `KeyboardHook::switch_enabled(&self, enabled, _main: &mut MainState) -> bool`. A
  `&mut MainState` exists only inside `with_main` on the main thread (its fields are `pub(super)`, so
  only the `hook` module can build one), and `MainState::set_enabled` is its only caller.
- **Q1.** `MAIN` is a `thread_local! RefCell<Option<MainState>>`, reached only by `with_main`
  (`try_with` and `try_borrow_mut`, never a panic). **Q3** HIGHEST. **Q4** the `Tap(100)+Hold(200)` gap is
  pinned (`tap_100_and_hold_200_released_at_150_fire_nothing`) and documented. **Q5** bench rules as
  ruled. **Q6** the commit-2 tests are green on master. **Q7** the executor's m-1 pin is left to 3.4.

### Deviations from the cut

1. An extra commit `7cb6868` (ruling M7) before commit 6, and a comment-only follow-up `d738632`.
2. The log scan entry for `with_main` is `fn with_main<` (the function is generic).
3. `GestureTimers::plan_wake` holds the plan step and its failure path, which collects the dropped
   tokens through `retain` (no new `TimerQueue` API).
4. Gesture API: `press(vk, group, now, seq, fire) -> Option<Instant>`,
   `release(vk, now, passed, seq) -> Released`, `other_key_down(vk, keys, now, fire)`,
   `on_timer(vk, seq, fire)`, `is_live`, `forget`. `on_timer` also fires closed windows (M6(a)).
   `Phase::AwaitSecond::passed` arrives in commit 11, where it is first read; in commit 9 it would
   have been a dead field.
5. The D8 mechanism: `ChordOutcome::Started { due }` and `SequenceOutcome::Advanced { due }`; the chord
   and sequence unit tests take the seq and expect these outcomes.
6. `decide_event` checks `is_enabled` once, before `with_main`. The switch flips only on the main
   thread between callbacks, inside `with_main`, which refuses a nested callback, so a second check
   under the borrow could not see another value.
7. `App::run` drops the `MainThread` before `uninstall` (M4(c)); R10 of the cut had it after the
   executor stop.
8. Bench R8 replaces a STATIC window's procedure (`SetWindowLongPtrW`): `RegisterClassW` needs the
   `Win32_Graphics_Gdi` feature, which is not enabled, and `Cargo.toml` stays untouched. R4, R5 and R6
   share one `MainThread` of the bench thread.
9. Config pages beyond the cut's line list, whose text the change made false: `kinds.md` (`:177`,
   `:198`, "another key does not cancel") and `structure.md:63` ("`<T>` toggles CapsLock").
10. Tests beyond the cut: `with_main_works_again_after_a_call`,
    `a_second_main_thread_on_one_thread_is_refused`, the two M6 scenarios and the M6 engine test,
    `under_test_a_fast_action_is_only_recorded`, and the rewritten gesture unit tests.
11. `src/core/executor/mod.rs:6` still says that a Hold's `hold-timer` thread submits; the cut keeps the
    executor untouched, so the line is listed for the sync in `impl.md`.
