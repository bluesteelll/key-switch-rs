# Cut: p0-A1-engine — Workstream A scope additions (pure decision core, Env, scenario tests); fixes KS-11, KS-25 (equal terms), KS-33 (partly)

## Verdict
CUT READY

Trees read (read-only, no builds): `master` = `73bbf40`; lane branch `refactor/engine` = `1d12c7b`
(commits `677ee50`, `e32169c`, `1d12c7b` on merge-base `da58f5f`). Between `da58f5f` and `73bbf40`
master changed only `.claude/`, `CLAUDE.md`, `docs/WORKFLOW.md`, `.gitattributes` (no `src/`).
Line numbers below: `M:` = master `73bbf40`, `B:` = branch `1d12c7b`, `@7cb` = the book's commit.

## Assessment of the existing branch (what is done, what remains)

| Spec item | State on `refactor/engine` @ `1d12c7b` | Remaining in this lane |
|---|---|---|
| `src/engine/` pure core: key state, Tap/Hold/DoubleTap, chord, sequence as plain data with deadlines | DONE: `B:src/engine/keys.rs:82-158`, `gestures.rs:27-169`, `chords.rs:26-107`, `sequences.rs:24-124`; no Win32 call, thread or lock (grep clean, see Acceptance) | add `#![forbid(unsafe_code)]` (D8); overdue-Hold rule (D6) |
| `Engine::on_key(event, &bindings, &dyn Env) -> Decision`, `Engine::on_timer(token) -> Decision` | DONE: `B:src/engine/mod.rs:223-233`, `:237-243`; also `on_foreground_changed` `:248-251` | none |
| `Decision { suppress, fire, timers(TimerRequest{token,due}), mask }`, executes nothing | DONE: `B:mod.rs:155-187`; `mask` reserved, never set | none |
| `Env`: async key state, foreground app/title, monotonic clock | DONE: `B:src/engine/env.rs:22-43`; `Condition::evaluate(&dyn ForegroundInfo)` `B:src/data/condition.rs:59`, trait `:83`, impl for `ForegroundContext` `:170` | Win32 impl `HookEnv` in the adapter (D7) |
| Hook callback becomes an ADAPTER | NOT DONE: `src/hook/` is byte-identical to master on the branch (`git diff master...refactor/engine -- src/hook` is empty) | D2-D5, commits C4-C5 |
| tap/chord/sequence state files deleted or shrunk | NOT DONE: still on the branch, logic duplicated in `src/engine` | C6 |
| `engine::testing::Scenario::from_ron` with `.down/.repeat/.up/.advance_ms` + assertions, fake Env and clock | DONE: `B:src/engine/testing.rs:106-297` (`FakeEnv` `:38-82`) | add `.advance_ms_stalled` (C1) |
| Scenario tests for Immediate, Tap, Hold, DoubleTap, Sequence, Chord, `when:`, `block_default: false` | DONE: `B:src/engine/scenarios.rs` (33 tests, list in Acceptance) | 3 more for D6 (C1) |
| KS-11 fixed by design + CHANGELOG + test | tests DONE (`B:scenarios.rs:282`, `:339`, `:430`; `B:gestures.rs:274`); CHANGELOG NOT DONE | CHANGELOG in C5 |
| Existing suite passes | the 21 hook-module tests are ported (mapping in D9); 198 tests reported passing at `1d12c7b` | keep green through C6 |
| Architecture in module docs | engine side DONE (`B:mod.rs:1-93`); adapter side NOT DONE | C5 |

## Spec re-located

| Spec reference (book @7cb3521) | Current file:line | Status | Note |
|---|---|---|---|
| roadmap Workstream A scope additions 1-4 (`book/src/roadmap.md:85-90`) | `B:src/engine/*` | moved (implemented on branch) | `arm` in the book = `timers` in code |
| `handle_key_down` @7cb `keyboard_hook_callback.rs#L183` | `M:src/hook/keyboard_hook_callback.rs:186-318` | moved | logic ported to `B:src/engine/mod.rs:253-377`; master copy deleted in C5 |
| `handle_key_up` @7cb `#L302` | `M:keyboard_hook_callback.rs:320-343` | moved | ported to `B:mod.rs:379-403` |
| callback entry, injected filter, sync | `M:keyboard_hook_callback.rs:127-164` (filter `:139-141`, sync `:149`) | unchanged | becomes the adapter (C5) |
| `foreground_changed` @7cb `#L176-L181` | `M:keyboard_hook_callback.rs:170-184` | moved | ported to `B:mod.rs:248-251` |
| `sync_modifiers`, `MODIFIERS`, `ActiveKeys`, blocked bitmap helpers | `M:keyboard_hook_callback.rs:34-125`, `:345-365` | moved | ported to `B:src/engine/keys.rs:12-158` |
| shared state fields `active_keys`, `blocked_keys`, `tap_state`, `chord_state`, `sequence_state` | `M:src/hook/keyboard_hook.rs:34-41`, init `:52-61` | unchanged | replaced by `engine: Mutex<Engine>` (C5) |
| `update_bindings` sort @7cb `keyboard_hook.rs#L72` | `M:src/hook/keyboard_hook.rs:69-74` | unchanged | C5 calls `engine::sort_by_specificity` (`B:mod.rs:193-195`), same stable sort |
| KS-11 tap-gc @7cb `tap_state.rs#L61-L66` | `M:src/hook/tap_state.rs:59-67` | moved | removed by deadlines (`B:gestures.rs:66-93`) |
| KS-11 hold-timer @7cb `#L103-L119` | `M:tap_state.rs:103-116` | moved | generation token `B:gestures.rs:99-143`; adapter thread in C5 |
| KS-11 double-tap-gc @7cb `#L156-L167` | `M:tap_state.rs:152-164` | moved | deadlines `B:gestures.rs:150-169` |
| KS-25 key-up resolves Tap and Hold independently @7cb `keyboard_hook_callback.rs#L314-L315` | `M:keyboard_hook_callback.rs:332-333`; `B:mod.rs:387-389` | moved | D6 adds the overdue rule |
| KS-33 `.expect("spawn ... thread")` @7cb `tap_state.rs#L61-L67`,`#L105-L119`,`#L158-L167` | `M:tap_state.rs:67`, `:116`, `:164` | moved | C5: only Hold spawns; error logged, no panic |
| chord @7cb `chord_state.rs#L61-L120` | `M:src/hook/chord_state.rs:61-123`, `mark_completed_keys_blocked` `:130-138` | moved | `B:chords.rs:58-106`; blocking via `KeyState::mark_blocked` `B:mod.rs:364-366` |
| sequence @7cb `sequence_state.rs#L55-L120` | `M:src/hook/sequence_state.rs:55-124` | moved | `B:sequences.rs:57-116` |
| `Condition::evaluate(&ForegroundContext)` @7cb `condition.rs` | `M:src/data/condition.rs:57`; capture `:93` | moved (branch) | `&dyn ForegroundInfo` `B:condition.rs:59` |
| foreground captured once per key-down when any `when:` @7cb `#L207-L221` | `M:keyboard_hook_callback.rs:210-214` | moved | lazy per event via `HookEnv` (D7); `Always` never asks (`M:condition.rs:59`) |
| message loop @7cb `app.rs#L125-L131` | `M:src/core/app.rs:125-131`; install `:91` | unchanged | not touched |
| doc refs to `tap_state` | `M:src/data/trigger.rs:7`, `M:src/data/binding.rs:63`, `README.md:47-49`, `:272-285` | unchanged | updated C5/C6 |

## Goal and non-goals

**Goal.** After the lane, `src/hook/keyboard_hook_callback.rs` only translates Win32 to an `engine::KeyEvent`,
asks `Engine::on_key`, and carries out the `Decision` on the same thread at the same point as today. The hook
module holds no decision logic, and `tap_state.rs`, `chord_state.rs`, `sequence_state.rs` are gone. Behaviour is
unchanged except three fixes that the plain-data design makes impossible to keep: KS-11 (stale timers),
KS-25 for equal Tap/Hold terms (exactly one fires at the boundary), KS-33 partly (no Tap/DoubleTap threads,
no abort on a failed Hold spawn).

**Non-goals (later items, do not touch):** KS-01/02 IPC (other A lanes), KS-03 `AppEquals` rights, KS-04 trigger-key
rule, KS-05/18 exact blockers, KS-06 repeat symmetry, KS-07 sides, KS-14 sync of blocked keys, KS-10 repeat policy (1.1),
mask key (1.2, `Decision::mask` stays reserved), KS-13 one winner (1.6), executor (2.1), timers on the main
thread and the full KS-25 state machine (2.2), reset on reload/off (2.3, KS-31), Sequence/Chord matching (3.2),
`thread_local!` engine and zero-allocation `on_key` (3.4), foreground cache (3.5).
**Depends on this lane:** every other Workstream A lane and items 1.1, 1.2, 1.3, 1.6, 2.2, 3.2, 3.4 put their logic
and scenario tests into `src/engine`.

## Decisions

### D0: Bring master into the lane branch first
What: first action on the branch is `git merge --no-ff master` (message `Merge branch 'master' into refactor/engine`
+ trailer). No rebase, no force.
Why: the branch forks at `da58f5f`, before `CLAUDE.md`, `.claude/settings.json` (deny rules), `.claude/agents/*.md`,
`.claude/workflows/*.js` and `docs/WORKFLOW.md` existed. Checking it out in the shared work tree
`D:/key-switch-rs/key-switch-rs` deletes those files from the tree, and every later brief of this lane reads
`.claude/agents/<role>.md` from it. The merge is conflict-free (disjoint paths).
Rejected: fresh branch + cherry-pick (the brief says continue on the existing branch); leaving it (role files
vanish mid-lane). Cost: one merge commit.

### D1: Keep the branch's engine API as is
What: `Engine::{new, on_key, on_timer, on_foreground_changed}`, `KeyEvent{vk, direction}`, `KeyDirection{Down, Up}`,
`TimerToken{vk, generation}` (opaque), `TimerRequest{token, due: Instant}`, `Decision{suppress, fire: Vec<BindAction>,
timers: Vec<TimerRequest>, mask: bool}` `#[must_use]`, `trait Env{now, os_key_down, foreground_app, foreground_title}`,
`sort_by_specificity`. The only engine change is D6.
Why: matches the spec signatures; already reviewed by its own 68 tests; no reason to churn.
Rejected: passing `now` into `on_timer` (not needed once D4 holds). Cost: none.

### D2: The engine sits in `KeyboardHook` behind one `Mutex`, reached only through `with_engine`
What (`src/hook/keyboard_hook.rs`):
```rust
pub struct KeyboardHook { bindings: ArcSwap<Vec<Binding>>, pub hook_handle: AtomicPtr<c_void>,
                          pub foreground_hook: AtomicPtr<c_void>, engine: Mutex<Engine> }
thread_local! { static DECIDING: Cell<bool> = const { Cell::new(false) }; }
impl KeyboardHook {
    /// Runs `f` on the engine under its lock. Returns `None` without locking when this
    /// thread is already inside `with_engine` (re-entry), so a re-entered hook passes the
    /// event instead of deadlocking.
    pub(crate) fn with_engine<R>(&self, f: impl FnOnce(&mut Engine) -> R) -> Option<R>;
}
```
- `lock()` poisoning is recovered with `PoisonError::into_inner` (no `expect` inside an `extern "system"` path).
- `DECIDING` is set after the lock is taken and reset by a guard's `Drop` before the `MutexGuard` drops.
- Removed: `blocked_keys`, `active_keys`, `tap_state`, `chord_state`, `sequence_state`, `ATOMIC_FALSE`.

Why: the spec allows a Mutex until 2.2; Hold timers still run on their own threads and must reach `on_timer`.
The lock covers only the decision state, not the hook (`bindings` stays lock-free `ArcSwap`, handles stay atomics).
It is held only while the engine decides: no action, no `SendInput`, no thread spawn under it. The LL hook is
re-entered only "by sending a message to the thread that installed the hook" (LowLevelKeyboardProc remarks), i.e.
while that thread waits in a message-retrieving call; nothing under the lock does that, and the `DECIDING` guard turns
any future violation into a pass instead of a deadlock. `std::sync::Mutex::lock` on a re-entrant call "will not
return on the second call (it might panic or deadlock)", which is exactly what the guard prevents. The flag is a
per-thread `thread_local!` `Cell`, not shared state, so no atomics and no ordering argument are needed.
Rejected: (a) engine on the main thread with timers posting tokens there (`PostThreadMessageW`): the Hold action would
then run on the hook thread (it runs on its timer thread today; KS-08 regression) — that is 2.1/2.2. (b) `try_lock` and pass
on contention: a contended key-up would pass while its key-down was swallowed (breaks the suppression invariant).
(c) re-entry detection by `GetCurrentThreadId` in an `AtomicU32`: needs `unsafe` and a Relaxed argument for no gain.
Cost: one uncontended lock per key event (replaces up to five gesture mutexes taken today); hold-timer threads contend
for microseconds. Conflicts with the wording of CLAUDE.md principle 5 and code-reviewer R3 ("hook-wide Mutex"): see Q1.

### D3: Adapter shape
What (`src/hook/keyboard_hook_callback.rs`, all on the main thread unless noted):
```rust
struct HookEnv { foreground: OnceCell<ForegroundContext> }            // D7
fn key_event(wparam: WPARAM, vk_code: u32) -> Option<KeyEvent>;        // KEYDOWN|SYSKEYDOWN -> Down, KEYUP|SYSKEYUP -> Up, else None
fn decide_event(hook: &KeyboardHook, wparam: WPARAM, vk_code: u32, extra_info: usize) -> Option<Decision>;
fn carry_out(decision: Decision);                                      // timers first, then fire in order; mask ignored (item 1.2)
fn start_hold_timer(request: TimerRequest);                            // D5
pub(crate) unsafe extern "system" fn keyboard_hook_callback(code, wparam, lparam) -> LRESULT;
pub(crate) unsafe extern "system" fn foreground_changed(...);
```
- `keyboard_hook_callback`: `code < 0` -> `CallNextHookEx`; `// SAFETY:` deref of `lparam` as `*const KBDLLHOOKSTRUCT`
  (valid for `HC_ACTION`); `decide_event(get_hook(), wparam, kb.vkCode, kb.dwExtraInfo)`; on `Some(d)`: remember
  `d.suppress`, `carry_out(d)`, return `LRESULT(1)` if suppressed; every other path returns `CallNextHookEx`.
- `decide_event`: returns `None` for `is_injected_event(extra_info)` (before any state is touched, CLAUDE.md principle 3),
  for a wparam that is not one of the four key messages, and when `with_engine` refuses re-entry. Otherwise it loads the
  bindings guard, builds `HookEnv`, runs `with_engine(|e| e.on_key(event, &bindings, &env))`, and drops the guard before
  returning, so no `ArcSwap` guard and no lock is held while actions run. `VIRTUAL_KEY(vk_code as u16)` is built only in
  `key_event` (hook boundary, CLAUDE.md principle 7).
- `foreground_changed`: `get_hook().with_engine(|e| e.on_foreground_changed(&HookEnv::new()))`, result ignored.
- `carry_out` starts timers before running actions, matching today, where the hold thread is spawned during the
  binding loop before later actions run; actions run in `Decision::fire` order, which is the binding visit order as today.

Why: `decide_event` is safe Rust and testable on a local `KeyboardHook` without a hook and without executing any action.
"Every key event" = every `HC_ACTION` event that is not the daemon's own injection; those two filters are the Win32
contract and the feedback-loop guard, not decisions about bindings.
Rejected: moving the sentinel check into the engine (adds a lock round-trip per injected event and puts a Win32
`dwExtraInfo` notion into the core). Cost: none beyond D2.

### D4: Clock = `Instant::now()`, read inside the engine call; timers wait until `Instant::now() >= due`
What: `HookEnv::now()` returns `Instant::now()`. The engine calls it once per event inside `on_key`, i.e. under the lock.
`start_hold_timer` sleeps in a loop until `Instant::now() >= request.due` before locking.
Why: Tap fires iff `now < due`, Hold's timer fires only at `>= due`, and both decisions are serialised by the lock with
the clock read inside it, so with equal terms Tap and Hold can never both fire (invariant I-TH, with D6 also never
neither). `thread::sleep` "will never sleep less" (std docs); the loop makes the guarantee hold against the same clock
regardless. `Instant` is monotonic (QPC on Windows).
Rejected: `KBDLLHOOKSTRUCT::time` (a different, 32-bit millisecond tick that wraps after 49.7 days; timers would mix
clocks, and the event time can precede a timer that already fired, re-opening the both-fire race). Cost: none.

### D5: Timers keep today's mechanism for Hold only
What: one thread named `hold-timer` per `TimerRequest` (only Hold arms produce them): sleep (D4) ->
`get_hook().with_engine(|e| e.on_timer(token))` -> `carry_out` on that thread, so the Hold action runs on its timer thread as
today. `thread::Builder::spawn` errors are logged with `eprintln!("[WARN] ...")` and not unwrapped; the Hold then fires at
its key-up through D6. `tap-gc` and `double-tap-gc` threads are gone (deadlines). The generation token makes a late
timer a no-op (KS-11).
Why: the spec says map timer requests onto the current mechanism; 2.2 moves them to `SetTimer` on the main thread.
A spawn `.expect` inside the LL callback aborts the daemon (KS-33); re-writing it would re-create a known bug.
Cost: Tap/DoubleTap no longer spawn a thread per press (fewer threads; KS-33 partly).

### D6: A Hold that is overdue at its key-up fires instead of being cancelled
What (`src/engine/gestures.rs`, `src/engine/mod.rs`):
```rust
struct PendingHold { vk: u16, action: BindAction, generation: u64, due: Instant }
pub(super) enum HoldRelease { NotPending, Cancelled, Overdue(BindAction) }
pub(super) fn release_hold(&mut self, vk: VIRTUAL_KEY, now: Instant) -> HoldRelease; // replaces cancel_hold
```
`now < due` -> `Cancelled`; `now >= due` -> `Overdue(action)` (entry removed, its timer later finds nothing).
`on_key_up`: fire order `[overdue hold, tap]` (chronological); the key-up is gesture-bound (suppressed, blocked flag taken)
when a tap resolved or the hold was `Cancelled` or `Overdue`.
Why: with Tap deadlines (branch) and real timer threads, a release in `[due, timer wake-up)` would fire neither
action — a new "neither" window that the branch's harness cannot show, because its timers fire exactly at `due`.
The rule makes the documented semantics exact ("fire after key held for N ms", `src/config/default_config.ron:74`,
`src/data/trigger.rs:24-26`): held for at least N ms means Hold, released before N ms means Tap. It closes the
equal-terms half of KS-25; unequal terms still behave as today (2.2). It also makes a failed Hold spawn (D5) degrade to
"fires at release".
Rejected: keeping the cancel (introduces the neither-window); firing Tap in the window (contradicts the Tap deadline).
Cost: one `Instant` per pending Hold.

### D7: `HookEnv`
What: `os_key_down(vk)` = `(unsafe { GetAsyncKeyState(vk.0 as i32) } as u16) & 0x8000 != 0` with a SAFETY comment
(as `M:keyboard_hook_callback.rs:94-97`); `foreground_app/title` = `self.foreground.get_or_init(ForegroundContext::capture).app()/title()`.
Created on the stack per event (`OnceCell` inline, no allocation).
Why: identical Win32 calls to today; capture becomes lazy (only when a non-`Always` condition is evaluated) instead of
"once per key-down when any binding has `when:`" — same results, never more calls. KS-03 is untouched.
Cost: the foreground lookups (`OpenProcess`, `GetModuleBaseNameW`, `GetWindowTextW`, unchanged code in
`M:src/data/condition.rs:93-157`) now run while the engine lock is held; a hold-timer thread may wait for them.

### D8: `#![forbid(unsafe_code)]` in `src/engine/mod.rs`
Why: every `windows` crate Win32 call is `unsafe`, so the attribute makes "no Win32 calls in the engine" a compile-time
property, ahead of 3.4's acceptance. Cost: none (the engine has no `unsafe` today).

### D9: Delete the three gesture files; tests are already ported
Ported (master -> branch): `tap_state` 8 -> `engine::gestures::tests::{arm_tap_inserts_then_resolve_removes,
resolve_tap_with_no_pending_returns_none, arm_hold_inserts_then_cancel_removes, cancel_hold_with_no_pending_returns_false,
double_tap_first_press_arms_second_press_fires, double_tap_third_press_arms_fresh_pending,
tap_and_hold_pending_simultaneously_on_same_key, auto_repeat_keydown_does_not_stack}`; `chord_state` 5 ->
`engine::chords::tests` (same names); `sequence_state` 4 -> `engine::sequences::tests` (same names);
`keyboard_hook_callback` 4 -> `engine::keys::tests::{active_keys_dedup, active_keys_capacity_bound,
blocked_flags_are_independent, clear_all_resets_every_flag}`. After D6 the gestures tests that call `cancel_hold` call
`release_hold(vk, t0 + ms(10))` with unchanged assertions (`Cancelled` = old `true`, `NotPending` = old `false`).

### D10: Docs
Module docs: `src/engine/mod.rs` (Time: overdue Hold; Threads and locking: `with_engine`, re-entry guard, clock read under
the lock, timer loop; Decision rules: key-up resolution), `src/engine/env.rs` (`now` is read under the adapter's lock),
new `//!` header in `src/hook/keyboard_hook_callback.rs` (the adapter: filters, translation, lock scope, carry-out
order, threads). README, CHANGELOG per Docs impact. The book is left to the phase book sync (Q4).

### D11: Allocation budget
Prescribed: `Decision::fire`/`timers` are `Vec`s that allocate only when something fires or a Hold arms; an Immediate
fire now clones its `BindAction` (today it ran by reference). Key events that fire nothing allocate nothing new; chord
key-downs allocate less than today (`B:chords.rs:66` checks membership before the sorted copy). Zero allocation in
`on_key` is 3.4's acceptance, not this lane's.

## Win32 API choices

No new API, no new Cargo feature; `Cargo.toml` and `Cargo.lock` are untouched.

| Purpose | `windows` path | Feature flag (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| OS modifier state for the resync (`HookEnv::os_key_down`) | `windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(vkey: i32) -> i16` | `Win32_UI_Input_KeyboardAndMouse` (yes) | main (hook + WinEvent callbacks), under the engine lock | returns 0 "if the current desktop is not the active desktop" or UIPI blocks: reads as up, same as today | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate |
| Constraint relied on: the delivered key's async state lags | (contract) | - | main | the engine skips `event.vk` (`B:keys.rs:104-111`) | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc ("called before the asynchronous state of the key is updated") |
| Foreground window for `when:` (unchanged code) | `windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow() -> HWND` via `ForegroundContext::capture` | `Win32_UI_WindowsAndMessaging` (yes) | main, lazily under the lock | null HWND -> `None` | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow |
| App name / title (unchanged code, KS-03 untouched) | `Threading::OpenProcess`, `ProcessStatus::GetModuleBaseNameW`, `WindowsAndMessaging::GetWindowTextW` | `Win32_System_Threading`, `Win32_System_ProcessStatus`, `Win32_UI_WindowsAndMessaging` (yes) | main, under the lock | `None` | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowtextw |
| Pass an event on | `windows::Win32::UI::WindowsAndMessaging::CallNextHookEx` | yes | main | - | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-callnexthookex |
| Event data | `windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT` (`vkCode`, `dwExtraInfo`) | yes | main | - | https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct |
| Hook budget and re-entry mechanism (contract) | - | - | main | timeout -> hook silently removed (Win7+), max 1000 ms (Win10 1709+) | https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc |
| Monotonic clock | `std::time::Instant` (QPC) | - | any | - | https://doc.rust-lang.org/std/time/struct.Instant.html |
| Timer wait | `std::thread::sleep` ("never sleep less") + loop | - | hold-timer | - | https://doc.rust-lang.org/std/thread/fn.sleep.html |
| Timer thread start | `std::thread::Builder::spawn -> io::Result<JoinHandle>` | - | main | `Err` logged | https://doc.rust-lang.org/std/thread/struct.Builder.html#method.spawn |
| Engine lock | `std::sync::Mutex` (re-entrant lock unspecified; `PoisonError::into_inner`) | - | main, hold-timer | poison recovered | https://doc.rust-lang.org/std/sync/struct.Mutex.html#method.lock |

**Threading after the lane.**

| Thread | Owns / runs | Change |
|---|---|---|
| main | installs/uninstalls `WH_KEYBOARD_LL` and the `EVENT_SYSTEM_FOREGROUND` WinEvent hook (`M:src/hook/keyboard_hook.rs:83-134`, `:159-182`), pumps `GetMessageW` (`M:src/core/app.rs:125-131`), runs `keyboard_hook_callback` and `foreground_changed`, executes Immediate/Tap/DoubleTap/Chord/Sequence-kind actions after deciding | engine lock taken per event, released before any action |
| `hold-timer` (one per armed Hold) | sleep until due, `on_timer` under the lock, Hold action without the lock | no longer removes map entries itself |
| `tap-gc`, `double-tap-gc` | - | removed |
| `config-watcher`, notify backend, IPC workers, macro workers | unchanged; never touch the engine (`update_bindings` is an `ArcSwap` store) | none |

No COM apartment, window or `SetTimer` is involved. Install/uninstall do not move. Injection: no new `SendInput`
path; injected events (macro workers, `PressKey`) are filtered by the sentinel before the lock; `LLKHF_INJECTED`,
extended-key flag and `scanCode` are not used by this lane (as today).

## Touch set

| File | Change | Why |
|---|---|---|
| (merge) `CLAUDE.md`, `.claude/**`, `docs/WORKFLOW.md`, `.gitattributes` | arrive via D0 merge, no edits | D0 |
| `src/engine/testing.rs` | `Scenario::advance_ms_stalled(ms)`: moves the clock, delivers no timer; the next `advance_ms` delivers overdue timers (existing `max` at `B:testing.rs:222` already handles late due times) | C1 tests |
| `src/engine/scenarios.rs` | 3 scenarios (C1) | D6 |
| `src/engine/gestures.rs` | `PendingHold::due`, `HoldRelease`, `release_hold` replaces `cancel_hold`; 1 new unit test; adjust 5 tests' calls (D9) | D6 |
| `src/engine/mod.rs` | `on_key_up` uses `release_hold`; `#![forbid(unsafe_code)]`; module docs | D6, D8, D10 |
| `src/engine/env.rs` | doc: `now` read under the adapter lock | D4, D10 |
| `src/hook/keyboard_hook.rs` | engine field + `with_engine` + `DECIDING`; remove 5 fields and `ATOMIC_FALSE`; `update_bindings` -> `engine::sort_by_specificity`; 2 tests | D2 |
| `src/hook/keyboard_hook_callback.rs` | adapter (D3-D5, D7), module doc; remove `MODIFIERS`, `ACTIVE_KEYS_CAP`, `ActiveKeys`, `sync_modifiers`, `mark_blocked`, `take_blocked`, `clear_all_blocked`, `handle_key_down`, `handle_key_up`, `collect_active_keys` and their 4 tests (ported); 2 new tests | adapter |
| `src/hook/tap_state.rs`, `chord_state.rs`, `sequence_state.rs` | delete | D9 |
| `src/hook/mod.rs` | drop the three `pub mod` lines (`M:src/hook/mod.rs:1,6,7`) | D9 |
| `src/data/trigger.rs:7`, `src/data/binding.rs:63` | doc refs `tap_state` -> `crate::engine` | stale refs |
| `CHANGELOG.md` | Unreleased: Changed + 3 Fixed entries (Docs impact) | user-visible fixes |
| `README.md` | layout tree, `keyboard_hook.rs`/`keyboard_hook_callback.rs` module bullets, "How resolution works (under the hood)", two "lock-free/atomic" feature lines | describes code this lane changes |
| `src/config/default_config.ron` | no change | config meaning unchanged |
| `Cargo.toml`, `Cargo.lock`, `src/core/*`, `src/config/*`, `src/lib.rs` | no change (`pub mod engine` already on the branch) | - |
| `book/src/**` | no change in this lane (Q4) | doc-writer owns the book |

## Commits, in order

Base for the reds below is the branch head before the remainder (`1d12c7b`, after C0 the merge commit). Every engine/adapter
test is a compile error on `master` (no `src/engine`, no `decide_event`); that is the expected red when the tester replays on
`master`. Gates per commit for the developer: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
the tests the commit touches (`cargo test --locked engine::` / `hook::`); the tester owns the full suite.

0. `Merge branch 'master' into refactor/engine` (D0). No source change; gate unchanged from `1d12c7b`.

1. `Engine: red-first: a Hold overdue at its key-up must fire` (declared red). Adds `Scenario::advance_ms_stalled` and:
   - `engine::scenarios::hold_overdue_at_its_release_fires_on_the_key_up` — `(keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200))`:
     `.down("CapsLock").expect_suppressed().expect_nothing_fired().advance_ms_stalled(250).expect_nothing_fired()
     .up("CapsLock").expect_suppressed().expect_fired(&[PressKey(VK_LSHIFT)]).advance_ms(10).expect_nothing_fired()
     .expect_total_fired(&[PressKey(VK_LSHIFT)])`.
     PREDICTED RED: panic `scenario: up CapsLock at +250 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []`.
   - `engine::scenarios::equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` — the README pair (`Tap(200)` SwitchLanguage +
     `Hold(200)` PressKey("LShift") on CapsLock). Run A: `.down.advance_ms_stalled(200).up` -> `expect_suppressed`,
     `expect_fired(&[PressKey(VK_LSHIFT)])`, `.advance_ms(50)` -> nothing, total `[PressKey(VK_LSHIFT)]`. Run B (fresh scenario):
     `.down.advance_ms_stalled(199).up` -> `expect_fired(&[SwitchLanguage])`, `.advance_ms(50)` -> nothing, total `[SwitchLanguage]`.
     PREDICTED RED (run A): panic `scenario: up CapsLock at +200 ms: expected to fire [PressKey(VIRTUAL_KEY(160))], fired []`. Run B is green on `1d12c7b`.
   - `engine::scenarios::hold_released_before_its_term_with_a_late_timer_is_cancelled` — Hold(200): `.down.advance_ms_stalled(199).up`
     -> suppressed, nothing; `.advance_ms(50)` -> nothing; total `[]`. Guard, GREEN on `1d12c7b` (kills "always fire at release").
   Only harness and scenario code: compiles on `1d12c7b`, fails by assertion.

2. `Engine: fire a Hold that is overdue at its key-up (KS-25)` (D6). Makes 1 green. Touches `gestures.rs`, `mod.rs` (on_key_up +
   "Time"/"Decision rules" docs). New unit test `engine::gestures::tests::hold_released_at_or_after_its_due_fires_once`
   (arm t0 term 200; `release_hold(t0+200)` = `Overdue(action)`, then `fire_hold(token)` = `None`; a second key: `release_hold(t0+199)` =
   `Cancelled`, `fire_hold(token)` = `None`). Invariant: every other scenario unchanged (timers in the harness fire exactly at due).

3. `Engine: forbid unsafe code` (D8). `#![forbid(unsafe_code)]` at the top of `src/engine/mod.rs`. No red-first (a lint attribute);
   proven by the gate and the acceptance grep.

4. `Hook: red-first: the adapter routes physical events through the engine` (declared red, compile error).
   - `hook::keyboard_hook_callback::tests::physical_key_events_reach_the_engine` — local `KeyboardHook::new()` with
     `update_bindings(vec![Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing)])`;
     `decide_event(&hook, WPARAM(WM_KEYDOWN as usize), VK_F13.0 as u32, 0)` -> `Some(d)`, `d.suppress`, `d.fire == [DoNothing]`;
     same with `WM_SYSKEYDOWN` after a `WM_KEYUP` (the up returns `Some`, suppressed); `WM_KEYDOWN` for `VK_A` -> `Some`, not suppressed.
   - `hook::keyboard_hook_callback::tests::injected_and_unknown_events_bypass_the_engine` — local hook with
     `Binding::new(KeyCombination::new(VK_F13), BindAction::DoNothing).with_trigger(Trigger::Tap(10_000))`:
     `decide_event(.., WM_KEYDOWN, F13, injected_sentinel())` -> `None`; then the physical F13 `WM_KEYDOWN` -> `Some(d)` with
     `d.suppress` (had the injected down marked F13 held, this down would be a repeat, which a Tap binding skips and passes today);
     `WPARAM(WM_CHAR as usize)` -> `None`.
   - `hook::keyboard_hook::tests::with_engine_refuses_reentry_on_the_same_thread` — on a spawned thread, `hook.with_engine(|_| hook.with_engine(|_| ()))`
     must return `Some(None)`; the result comes back through an `mpsc` channel with `recv_timeout(5 s)`, so a broken guard fails
     (`re-entry deadlocked`) instead of hanging the suite.
   - `hook::keyboard_hook::tests::with_engine_works_again_after_a_call` — two sequential calls both return `Some`.
   PREDICTED RED: `error[E0425]: cannot find function 'decide_event' in this scope` and
   `error[E0599]: no method named 'with_engine' found for struct 'KeyboardHook'`.
   Safety: these tests never call `carry_out` or the `extern "system"` functions, so no action executes and no timer starts;
   `HookEnv` only reads `GetAsyncKeyState` (the F13 single-key binding matches regardless of the result).

5. `Hook: the callback becomes an adapter over the engine` (D2-D5, D7, D10, D11). Makes 4 green. Touches
   `keyboard_hook.rs`, `keyboard_hook_callback.rs`, `src/engine/mod.rs` and `env.rs` (docs), `CHANGELOG.md`, `README.md`
   (all README edits except the three layout lines). Removes the callback's 4 helper tests (ported, D9). The gesture
   files stay compiled but unused (their items are `pub`, so no dead-code lint). Invariants: every path returns
   `LRESULT(1)` or `CallNextHookEx`; the sentinel check precedes any state; no lock, guard or `ArcSwap` guard is held while an
   action runs; no `expect`/`unwrap` reachable from the two `extern "system"` functions.

6. `Hook: remove tap_state, chord_state and sequence_state` (D9). Deletes the three files and their `mod` lines, updates
   `src/data/trigger.rs:7`, `src/data/binding.rs:63` and README layout lines 47-49. Their 17 tests are ported (D9). Gate green.

## Acceptance mapping

| Acceptance line | Covered by |
|---|---|
| the quality gate passes | tester: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked` on HEAD; check `test result:` names, not counts (expected 198 - 21 removed + 3 (C1) + 1 (C2) + 4 (C4) = 185) |
| `rg 'GetAsyncKeyState\|GetForegroundWindow\|SendInput\|Mutex\|thread::' src/engine` finds nothing outside comments | tester runs it; every hit must be a `//`/`//!` line (today: none at all, `git grep` on `1d12c7b`). Reinforced by `#![forbid(unsafe_code)]` (C3) and `rg "\.execute\(" src/engine` = no hits (the engine executes nothing) |
| the hook callback calls the engine for every key event | `physical_key_events_reach_the_engine`, `injected_and_unknown_events_bypass_the_engine`; review of `keyboard_hook_callback` (single path `decide_event` -> `with_engine` -> `on_key`) |
| no decision logic is left in `src/hook` | `rg -n "Trigger::\|BindingKind::\|\.matches\(\|was_repeat\|block_default\|sync_modifiers\|mark_blocked\|take_blocked\|ActiveKeys\|TapState\|ChordState\|SequenceState" src/hook` = no hits; `src/hook/{tap,chord,sequence}_state.rs` absent |
| scenario tests: Immediate combo | `immediate_single_key_fires_on_down_and_swallows_both_edges`, `immediate_combo_fires_on_its_last_key_and_lets_the_modifiers_through`, `right_hand_modifiers_match_unsided_combo_keys`, `combo_does_not_fire_without_all_of_its_keys`, `more_specific_immediate_binding_wins`, `press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed`, `unbound_key_passes_through` |
| ... Tap | `tap_fires_on_a_release_within_its_term`, `tap_held_past_its_term_fires_nothing_and_swallows_the_release`, `two_quick_taps_fire_twice` |
| ... Hold | `hold_fires_once_its_term_passes_while_held`, `hold_released_early_is_cancelled`, `stale_hold_timer_does_not_fire_the_next_press_early`, `tap_and_hold_with_equal_terms_fire_exactly_one`, + C1's three |
| ... DoubleTap | `double_tap_fires_on_the_second_press_within_its_term`, `double_tap_second_press_after_the_term_arms_again`, `four_quick_presses_double_tap_twice` |
| ... Sequence | `sequence_fires_when_its_steps_follow_within_max_gap`, `sequence_times_out_after_max_gap`, `sequence_broken_by_another_key_does_not_fire`, `sequence_with_a_modifier_held_through_both_steps` |
| ... Chord | `chord_fires_when_all_keys_go_down_within_its_window`, `chord_keys_can_go_down_in_any_order`, `chord_does_not_fire_once_its_window_has_passed`, `key_outside_the_chord_passes` |
| ... when-condition | `when_app_equals_matches_the_foreground_app_case_insensitively`, `when_title_contains_and_not_conditions`, `first_binding_whose_condition_holds_wins_on_a_shared_key` (all `engine::scenarios::`) |
| ... block_default false | `block_default_false_fires_and_lets_both_edges_through` |
| the architecture is documented in module docs | reviewer reads `src/engine/mod.rs` `//!` (Architecture, State, Time, Threads and locking, Decision rules), `env.rs`, `gestures.rs`, and the new `//!` of `keyboard_hook_callback.rs` |
| behaviour unchanged except design-made fixes, each in CHANGELOG Fixed with a test | KS-11: `two_quick_taps_fire_twice`, `stale_hold_timer_does_not_fire_the_next_press_early`, `four_quick_presses_double_tap_twice`, `gestures::tests::stale_hold_timer_does_not_fire_a_newer_press`; KS-25 (equal terms): C1 tests; KS-33 (partly): review of `start_hold_timer` (no automated fault injection) |
| the whole existing suite keeps passing | tester: the 21 removed tests map 1:1 to ported names (D9); all other master test names present and passing |
| Workstream A scope additions 1-4 (roadmap) | 1: `src/engine`; 2: `Decision` + `rg "\.execute\(" src/engine` empty; 3: Env + grep + forbid; 4: `Scenario::from_ron` |

**KS-11 evidence on master (optional, tester, scratch worktree only, never committed):** in `src/hook/tap_state.rs` tests of a
`master` worktree: `arm_tap(VK_F13, DoNothing, 100)`; sleep 40 ms; `assert!(resolve_tap_on_keyup(VK_F13))`; sleep 30 ms;
`arm_tap(VK_F13, DoNothing, 100)`; sleep 60 ms; `assert!(resolve_tap_on_keyup(VK_F13), "KS-11")` -> fails on master (press 1's
`tap-gc` removed press 2's entry at ~100 ms). Uses `DoNothing`, so nothing is injected.

## Manual checks (human only)

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M1 | release build, default `config.ron` | CapsLock; Shift+CapsLock; type normally for a minute | language switches; CapsLock toggles; no stuck or lost keys; same as before the lane | none (needs a live hook) |
| M2 | add the README pair `Tap(200)` SwitchLanguage + `Hold(200)` PressKey("LShift") on CapsLock | tap CapsLock 20 times quickly (about 150 ms apart); then hold it 1 s three times | 20 switches, no stray Shift (KS-11); each hold sends one Shift and no switch | none |
| M3 | `( keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(200) )` (or any spare key) | four presses at a brisk even pace, twice | CapsLock toggles twice per four presses (KS-11) | none |
| M4 | chord `["J","K"]` window 50 and sequence `["F13","F14"]` | press J+K together; F13 then F14 | each fires once; keys swallowed as before | none |
| M5 | `when: TitleContains("Notepad")` on F13 (not `AppEquals`, KS-03 is open) | F13 in Notepad and elsewhere | fires only in Notepad | none |
| M6 | `Hold(300)` with `Launch(exe: "notepad.exe")` | hold the key 0.5 s | Notepad starts, typing does not stall | none |
| M7 | Task Manager / Process Explorer thread view | press a `Tap(200)` key 50 times | thread count does not grow (no `tap-gc`); a Hold key adds at most one short-lived `hold-timer` per press | none |
| M8 | `PressKey("Left")` on F13 | hold F13 | cursor moves repeatedly, no feedback loop (sentinel path) | none |

## Risks and edge cases

- **R1 hot path.** Per event: 11 `GetAsyncKeyState` (as today), one uncontended lock, lazy foreground lookups (as today but under
  the lock). No action, spawn or `ArcSwap` guard under the lock. `extern "system"` paths contain no `expect`/`unwrap`: poison is
  recovered, spawn errors logged. A panic inside `on_key` (an engine bug) still aborts the process, as any panic in the callback
  does today; it is not caught (CLAUDE.md principle 1).
- **Re-entrancy.** Only a message-retrieving wait under the lock could re-enter; none exists (`GetWindowTextW` on another process's
  window reads the caption without sending `WM_GETTEXT`; the daemon owns no top-level window today). A future own-process window
  in a non-main thread (tray, 4.6) would make `GetWindowTextW` send a message: the guard then passes that one event. A passed
  key-down was never recorded, so its key-up also passes: suppression symmetry holds.
- **R3 shared state.** The `Mutex` is not around the hook: bindings and handles stay lock-free. See Q1 for the rule wording.
- **R4 atomics.** No new atomic. `active_keys`/`blocked_keys` atomics disappear (plain arrays under the lock). `DECIDING` is
  thread-local.
- **R7 order.** `update_bindings` keeps the stable sort via `engine::sort_by_specificity`; `equal_specificity_keeps_config_order` stays.
- **R8 sides.** Unchanged (KS-07 is another lane); the engine reproduces today's folding (`B:keys.rs:65-73`) and raw chord codes.
- **R10 shutdown.** A `hold-timer` that wakes after `WM_QUIT`/`uninstall` still fires its action, as today (KS-31, 2.3).
- **Behaviour deltas, all intended:** (1) Tap/DoubleTap expiry by deadline instead of a GC thread: identical up to thread-wakeup
  jitter; (2) KS-11 gone; (3) equal Tap/Hold terms: exactly one at the boundary (D6); unequal terms unchanged (both or neither
  between the terms, KS-25 rest, 2.2); (4) Immediate actions run after the binding loop instead of inside it — same callback, same
  order; none of today's actions changes matching state synchronously (`SendInput` output is delivered after the hook returns);
  (5) a DoubleTap action no longer runs while a mutex is held; (6) a Hold's due is measured from the decision, not from its thread's
  start (earlier by spawn latency); (7) a failed Hold spawn is logged and the Hold fires at release.
- **Repeats.** Unchanged, KS-06/KS-10 stay (other lanes). No scenario pins the leaking repeats.
- **Reload mid-gesture.** Pending entries keep their cloned actions and fire them after a reload, as today (2.3). Bindings are
  passed per call, so a reload never touches the engine.
- **`swch off`/`on`.** Engine state survives, as today (KS-31). Foreground change: modifiers resynced, blocked flags cleared,
  gestures kept, as today. Note for the critic: `foreground_change_forgets_pending_key_up_suppression` pins today's clearing, which
  item 2.3 deliberately narrows; 2.3 must revise that scenario.
- **Clock under load.** The event time is processing time, as today (master's chord/sequence also used `Instant::now()` at
  processing). Secure desktop / UIPI: `GetAsyncKeyState` returns 0, as today.
- **Lock held during foreground lookups.** A hold-timer thread can wait for `OpenProcess`/`GetModuleBaseNameW` (sub-millisecond);
  its Hold fires correspondingly later. Acceptable until 3.5 caches the foreground.
- **Branch hygiene.** Without D0 the role files and deny rules vanish from the shared work tree while the branch is checked out.

## Docs impact

- **CHANGELOG.md, Unreleased** (in C5):
  - Changed: "The keyboard decisions moved into a pure decision core, `src/engine` (Workstream A scope additions). The hook callback
    translates each event, asks `Engine::on_key` for a `Decision` and carries it out. Tap and DoubleTap expire by deadline instead of a
    cleanup thread per press; only Hold still starts a timer thread. Scenario tests (`engine::testing::Scenario`) cover every binding
    kind and trigger."
  - Fixed: "KS-11: a timer left over from an earlier press no longer acts on a newer press of the same key: two quick taps fire twice,
    a Hold no longer fires early on the next press, and a quick series of DoubleTaps no longer loses fires."
  - Fixed: "KS-25 (equal terms): with `Tap(n)` and `Hold(n)` on one key, exactly one fires: Tap when released before `n` ms, Hold from
    `n` ms on, even when the hold timer wakes late. Different terms behave as before until roadmap item 2.2."
  - Fixed: "KS-33 (partly, unverified issue): Tap and DoubleTap no longer start a thread per press, and a Hold timer that cannot be
    started is logged instead of aborting the daemon; the Hold then fires at release."
- **README.md:** layout tree (add `engine/` with its 8 files in C5; drop `hook/tap_state.rs`, `chord_state.rs`, `sequence_state.rs`,
  lines 47-49, in C6); `keyboard_hook.rs` bullets (lines 93-94: active/blocked keys now live in the engine behind one short lock);
  `keyboard_hook_callback.rs` bullets (lines 98-100: adapter); "How resolution works (under the hood)" (lines 272-285: deadlines,
  generation tokens, overdue Hold); line 503 "Lock-free atomic operations on the hot path" and line 513 "Thread-safe: atomic
  operations ..." (binding list lock-free, decision state behind one lock never held across an action). Leave line 519 "Hot-path
  allocations: none" to 3.4 (already false today, tracked in `gaps.md`).
- **default_config.ron:** none (the trigger semantics at lines 69-81 are unchanged; "resolves whichever wins" becomes exact for equal terms).
- **Book (phase book sync, doc-writer):** `architecture.md` (crate layout row `hook`, module graph, threads table row
  `tap-gc`/`hold-timer`/`double-tap-gc`, key-event path, core data structures, "Gesture state machines"), `config/triggers-conditions.md`
  §11.4 (thread per arming), §11.5 (Tap/DoubleTap resolution, DoubleTap no longer under a mutex), §11.6 (KS-25 warning: equal terms
  now deterministic), §11.8 (KS-11 warning), `config/actions.md` table rows citing `tap_state`/`chord_state`/`sequence_state`,
  `config/known-issues.md` rows 23-24, `config/example.md:236`, `overview.md` gestures row and test table, `gaps.md:257`,
  `audit.md` status lines for KS-11 (fixed), KS-25 (partly), KS-33 (partly), roadmap Workstream A note.
- **CLAUDE.md / `.claude/agents/architect.md`:** the "src/engine does not exist yet" lines become stale after the merge (Q5).

## Questions for the orchestrator

1. **`Mutex<Engine>` vs CLAUDE.md principle 5 / code-reviewer R3 ("hook-wide Mutex").** The spec allows a Mutex around the engine
   until 2.2. Recommendation: accept it as designed in D2 (engine state only, held only while deciding, same-thread re-entry passes,
   no action under it) and tell the critic and reviewer that D2 is the sanctioned exception; after the merge, reword principle 5 and
   R3 to "until item 2.2, the decision core sits behind one `Mutex` in `KeyboardHook`, held only while deciding". Not blocking:
   the brief already decided it.
2. **D0, merging `master` into `refactor/engine`.** Recommendation: yes, as the first commit; it is the only way to keep the role
   files and `settings.json` in the shared work tree without rewriting the branch. Not blocking.
3. **D6, the overdue-Hold rule (KS-25 equal terms).** It is a visible fix beyond KS-11. Recommendation: keep it; without it the
   refactor opens a "neither fires" window at the term boundary. If vetoed, drop C1-C2, the KS-25 CHANGELOG line and the D5 note
   on failed spawns. Not blocking.
4. **Book pages.** Config meaning does not change, only known-issue notes become obsolete. Recommendation: leave all book edits
   to the phase book sync (list in Docs impact) rather than the developer. Not blocking.
5. **CLAUDE.md "Layout" bullet and `architect.md` line 37** say `src/engine` does not exist; principle 9 already expects engine
   scenario tests. Recommendation: the orchestrator updates both after the merge (not in this lane's product commits). Not blocking.

## Cut readiness checklist

- Spec
  - [x] KS-11, KS-25, KS-33 read in `audit.md`; KS-11 scenarios are tests (`two_quick_taps_fire_twice`, `stale_hold_timer_does_not_fire_the_next_press_early`, `four_quick_presses_double_tap_twice`), KS-25 equal-terms is C1, KS-33 is a review item + M7
  - [x] Book references re-located (table above)
  - [x] Goal, non-goals, dependent items stated
- Design
  - [x] Every decision has a rationale and rejected alternatives
  - [x] Owner decisions: no matching, repeat, RON, logging or licence change (`eprintln!` until 1.7)
  - [x] Thread named for every code path; only deciding happens under the lock in the callback
  - [x] Install/uninstall unchanged on the main thread; timers stay threads by spec (2.2 moves them)
  - [x] No new `SendInput` path; sentinel filter before state
  - [x] Sided/unsided handling unchanged and consistent with today (KS-07 elsewhere)
  - [x] Reload, `swch off`/`on`, foreground change, shutdown during a pending gesture covered (Risks)
  - [x] No new atomics; no `static mut`; thread-local flag only
  - [x] No new module cycle (`hook -> engine -> {core, data}`; `engine -> config` only under `cfg(test)`)
- Win32
  - [x] Each API with path, feature (all present), thread, errors, link; no new API
  - [x] Elevation/UIPI: unchanged; `GetAsyncKeyState` returns 0 across desktops as today; no pipe/token change
- Commits and tests
  - [x] Red-first with exact predicted red (C1 assertion, C4 compile error)
  - [x] Tests go through `on_key`/`Env`/fake clock or `decide_event` on a local hook; nothing installs a hook or injects input
  - [x] N/A ignored tests: no automated test needs a live hook; human checks M1-M8
  - [x] Each commit passes the gate except the declared reds C1 and C4
  - [x] CHANGELOG/README in C5, the commit that makes the fixes reach users; README layout in C6 with the deletion
- Acceptance
  - [x] Every acceptance line mapped
  - [x] Every question has a recommendation

## Developer addendum (critique round 1 responses)

Written by the developer after implementing on `refactor/engine` (HEAD `d3131fc`). Line references are to that
HEAD. Every remark of `critique.md` is RESOLVED or REFUTED here, the Minor ones included.

### I1 (Important): an overdue Hold must not run on the hook thread - RESOLVED in code (option (a), without an API change)

D6 is replaced by **D6'**: a key-up at or after a Hold's due time neither cancels the Hold nor fires it. The
entry stays armed, marked `released`, and its own timer fires it whenever that timer runs. Only
`Engine::on_timer` ever fires a Hold, so the Hold action always runs on its `hold-timer` thread, as on master.

- `src/engine/gestures.rs:57` `HoldRelease { NotPending, Cancelled, Overdue }` (`Overdue` carries no action);
  `:158` `release_hold(vk, now)`: `now < due` -> removed, `Cancelled`; else `released = true`, `Overdue`.
  A released entry is no longer "the key's pending Hold" (`pending_hold_index`, `:191`): it neither blocks the
  next press from arming (`arm_hold`, `:128`) nor answers the next key-up. `fire_hold(token)` still fires it.
- `src/engine/mod.rs:415-440` `on_key_up` uses `release_hold`; the key-up is gesture-bound (suppressed, blocked
  flag taken) for `Cancelled` and `Overdue`, as D6 prescribed. `Decision::fire` of a key-up holds at most the
  Tap action.
- Failed spawn (D5): `Engine::forget_timer(token)` (`src/engine/mod.rs:277`, `gestures.rs:185`) drops the Hold.
  `start_hold_timer` (`src/hook/keyboard_hook_callback.rs:186-200`) logs `[WARN] could not start a hold timer,
  that Hold will not fire: ...` and calls it. The Hold never fires (it may not run on the hook thread) and no
  entry lingers. This replaces D5's "the Hold then fires at its key-up through D6".
- Invariant I-TH holds: the Tap needs `now_up < due`; the Hold fires iff the key is still held when its timer
  locks (a timer locks only once `Instant::now() >= due`) or the key-up read `now_up >= due`. Both reads happen
  under the engine lock on one clock, so with equal terms exactly one fires. Only exception: a failed spawn plus
  a release at or after the term fires neither (logged).
- Tests (red on the branch head before the fix, see impl.md):
  `engine::scenarios::hold_released_after_its_term_fires_when_its_late_timer_wakes` (asserts that the key-up fires
  **nothing** and the late timer fires the Hold, which pins the thread rule at engine level),
  `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one`,
  `hold_released_before_its_term_with_a_late_timer_is_cancelled` (guard),
  `hold_left_to_a_late_timer_does_not_block_the_next_press` (kills "keep the entry and let it block the re-arm"),
  `hold_whose_timer_cannot_start_never_fires`; unit tests
  `engine::gestures::tests::{hold_released_at_or_after_its_due_is_left_to_its_timer, hold_left_to_its_timer_does_not_block_a_new_arm, forgotten_hold_never_fires_and_frees_its_key}`.
- Threading table ("Threading after the lane"): unchanged and now exact: Hold actions run only on `hold-timer`.
- Behaviour deltas, replacing (3) and (7): (3') equal Tap/Hold terms: exactly one fires; a Hold whose key is
  released at or after its term fires when its (late) timer wakes, which can be a few ms after the release,
  still on its timer thread. (7') a failed Hold spawn is logged and that Hold does not fire.
- CHANGELOG (`CHANGELOG.md:59-68`) says the same. Docs impact for the book sync: `config/triggers-conditions.md`
  §11.5 keeps "the only trigger whose action runs outside the hook thread" (still true) and needs: Tap and
  DoubleTap resolve by deadline (no GC thread; DoubleTap no longer runs under a mutex); Hold: "a key-up before
  `ms` cancels; at or after `ms` the Hold fires from its timer, even if the timer wakes after the release".
  §11.6: equal terms are exact.
- New manual check **M9**: config `( keys: "F13", action: Launch(exe: "notepad.exe"), trigger: Hold(300) )`.
  With the CPU loaded (for example a release build running), hold F13 for about 300-320 ms and release, 10 times.
  Expected: Notepad starts once per press held for at least 300 ms, never for a shorter press, and typing never stalls.
- Q3 rationale, corrected: master already has a narrow both-or-neither window at equal terms (two racing
  GC/timer threads, KS-25). The deadline design does not create a new window; without D6' it would widen
  "neither" from the gap between two equally late threads to the whole lateness of the Hold timer. D6' closes
  that window without moving the Hold action.

### I2 (Important): the `src/hook` decision-logic proof fails on the C4 test - RESOLVED (the check is redefined; no test bent)

The acceptance line "no decision logic is left in `src/hook`" is proven on **production code only**: each
`src/hook/*.rs` file up to its `mod tests {` line, with comment lines (`//`, `///`, `//!`) dropped. Test modules
sit at the bottom of each file (CLAUDE.md principle 9). Command (Git Bash, repo root):

```bash
check() { for f in "$1"/*.rs; do case "$(basename "$f")" in testing.rs|scenarios.rs) continue;; esac;
  awk '/^mod tests \{/{exit} !/^[[:space:]]*\/\//{print FILENAME":"FNR": "$0}' "$f"; done | grep -E "$2"; }
check src/hook 'Trigger::|BindingKind::|\.matches\(|was_repeat|block_default|sync_modifiers|mark_blocked|take_blocked|ActiveKeys|TapState|ChordState|SequenceState'
```

Expected: no output (grep exit status 1). Result at `d3131fc`: no output. The raw `rg` over `src/hook` has exactly
one hit, `src/hook/keyboard_hook_callback.rs:318` (`.with_trigger(Trigger::Tap(10_000))` inside `mod tests`),
which is test setup, not decision logic. `src/hook/{tap,chord,sequence}_state.rs` are deleted.

### M1 (Minor): purity claims and the `core::windows_actions` dependency - RESOLVED (docs and a stricter check)

- `src/engine/mod.rs:26-32` says what `#![forbid(unsafe_code)]` enforces (no direct FFI) and what it does not
  (safe wrappers such as `ForegroundContext::capture`, `BindAction::execute`, `Instant::now()`), and that the
  engine uses `BindAction` only as data. That declares the `engine -> core::windows_actions` dependency. Moving
  `BindAction` into `data` is recorded as a follow-up candidate for 3.4 (not done: no module moves beyond the cut).
- Stricter engine purity check, with the `check` function above (it skips `testing.rs`, `scenarios.rs` and `mod tests`):
  `check src/engine 'GetAsyncKeyState|GetForegroundWindow|SendInput|Mutex|thread::|Instant::now|ForegroundContext|\.execute\(|unsafe *(\{|fn|impl)'`
  gives no output at `d3131fc`. The spec's own grep
  (`rg 'GetAsyncKeyState|GetForegroundWindow|SendInput|Mutex|thread::' src/engine`) has one hit, a `//!` comment at
  `src/engine/mod.rs:74`.
- `#![forbid(unsafe_code)]` was probed: a temporary `GetAsyncKeyState` call in `engine::keys` fails to compile with
  "error: usage of an `unsafe` block".

### M2 (Minor): `WM_SYSKEYUP` untested - RESOLVED

`hook::keyboard_hook_callback::tests::physical_key_events_reach_the_engine` (`src/hook/keyboard_hook_callback.rs:287`)
loops over `(WM_KEYDOWN, WM_KEYUP)` and `(WM_SYSKEYDOWN, WM_SYSKEYUP)` on a bound F13. Each up follows a suppressed
down and must return `Some` with `suppress == true`.

### M3 (Minor): re-entry symmetry holds only for key-downs - RESOLVED (risk text corrected, residual risk recorded and handed over)

The corrected statement is in the adapter module docs (`src/hook/keyboard_hook_callback.rs`, section "Re-entry"):
a refused key-down keeps the suppression invariant (its key-up then passes too); a refused key-up of a suppressed
key-down does not (the key-up passes, the engine keeps the key held and blocked, the key's next press counts as a
repeat and its release is swallowed). No re-entry path exists today: nothing under the lock pumps messages.
Hand-over: a change that adds a window to the daemon (for example the roadmap 4.6 tray) or any message wait
under the lock must deal with it first; items 2.2/3.4 remove the lock. No counter or log line was added: a log
line would be I/O on the hook thread, and a counter would be a new atomic that the cut does not prescribe.

### Other deviations from the cut

- C1/C2 test names and expectations follow D6', not D6: the key-up fires nothing; the next `advance_ms` delivers
  the late timer, which fires the Hold. One extra scenario (`hold_left_to_a_late_timer_does_not_block_the_next_press`)
  and one extra unit test (`hold_left_to_its_timer_does_not_block_a_new_arm`).
- An extra commit between C2 and C3: `Engine: forget a Hold whose timer cannot be started (KS-33)`
  (`Engine::forget_timer`, harness step `Scenario::lose_timers`), which D6' needs for D5.
- A follow-up doc commit (`Hook: adapter docs link no private items`) removes the rustdoc warnings the new docs caused.
- README: "Bitmap-blocked keys" is renamed "Independently blocked keys" (the bitmap is gone).
- Expected test count after the lane: 189 (198 - 21 removed + 4 C1 + 2 C2 + 2 forget_timer + 4 C4), not 185.
