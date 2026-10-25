# Cut: p2-2.4-watchdog — roadmap 2.4 Hook watchdog (KS-08), with the 2.3 leftovers

Tree: `D:/key-switch-rs/key-switch-rs`, `master` at `fa20b30` (merge of `chore/after-2.3`). Branch `feat/hook-watchdog`.
Every `path:line` below is on that tree unless it names `@7cb3521` or another commit.

## Verdict
CUT READY

## Spec re-located

| Spec reference | Current file:line | Status | Note |
|---|---|---|---|
| roadmap 2.4 "Why": `is_installed()` only checks the stored handle (`src/hook/keyboard_hook.rs:137-139` @7cb3521) | `src/hook/keyboard_hook.rs:223-226` | moved | still `!hook_handle.is_null()`; nothing detects a hook Windows removed |
| roadmap 2.4 "Files": `app.rs`, `keyboard_hook.rs`, `keyboard_hook_callback.rs`, `ipc_server.rs` | `src/core/app.rs:182-301`, `src/hook/keyboard_hook.rs:24-283`, `src/hook/keyboard_hook_callback.rs:341-385`, `src/hook/ipc_server.rs:45-52,176-196` | moved | plus new files (touch set) |
| roadmap 2.4 "RIDEV_INPUTSINK on the 2.3 window" | `src/hook/control_window.rs:74-199` (hidden top-level window, main thread) | new in 2.3 | **not used** for raw input: D1 puts the sink on a dedicated thread |
| audit KS-08 location `windows_actions.rs:123-124`, `tap_state.rs:83/148` @7cb3521 | `src/core/windows_actions.rs:187` (`as_fast`), `src/core/executor/` | gone | the action half was fixed by 2.1; only the watchdog half remains |
| audit KS-08 "The hook lives on the main thread, which pumps `GetMessageW`" (`app.rs:91,125-131` @7cb3521) | `src/hook/main_thread.rs:621-665` (`MainThread::run`), `:776-795` (`pump`), `src/core/app.rs:285` | moved | `MsgWaitForMultipleObjectsEx` loop at `THREAD_PRIORITY_HIGHEST` |
| keyboard callback gates | `src/hook/keyboard_hook_callback.rs:346` (`code < 0`), `:353-359` (switch gate), `:364` (timing start), `:368` (deref), `:369-377` | unchanged | the heartbeat goes between `:353` and `:354` |
| gaps.md:155 "No hook health check" (`keyboard_hook.rs:136-139` @7cb3521) | `src/hook/keyboard_hook.rs:223-226` | moved | closed by this lane |
| gaps.md:164 refusal counter (`keyboard_hook_callback.rs:58-82` @20af28d) | `src/hook/main_thread.rs:199-210` (`with_main`), `:177-182` (`REFUSED`, `#[cfg(test)]` only) | moved | today a refusal is counted only under test |
| gaps.md:232 / 1.7 cut Q4 (`D:/tmp/ks-orch/p1-1.7-logging/cut.md:423`) panic hook | no `set_hook` anywhere under `src` (grep); `src/logging/mod.rs:109-125` (`init`) | unchanged (open) | |
| spec: gauges `longest_callback_us`, `executor_unavailable`, `active_runs` | `src/logging/counters.rs:78-83,132-145`, `:38-40`; `src/core/executor/mod.rs:249-252` | unchanged | D7 says which one helps |
| spec: log writer, counters | `src/logging/writer.rs:41-57,99-117,132-142`; `src/logging/counters.rs:17-203` | unchanged | |
| spec: today's `swch status` | `src/hook/ipc_server.rs:186-190` (`OK: running (N bindings live)`); `src/ipc.rs:199-219`; `src/bin/swch.rs:77-94` (open's probe: any `Ok`), `:146-165`; `tests/ipc_roundtrip.rs:48-117` | unchanged | |
| spec: bench rows R1-R13 and rules | `src/bench.rs:20-58,709-821`; rules `D:/tmp/ks-orch/p2-2.2-timers/cut.md:194`, `D:/tmp/ks-orch/p2-2.3-control-window/cut.md:405-414`; CLAUDE.md:58-67 | unchanged | R14, R15 added (D11) |
| 2.3 P1 `MovingQuery` PIDs | `src/hook/main_thread.rs:1639-1690` (windows `0x1000`/`0x2000` at `:1667,:1671,:1674`; names at `:1677,:1686`) | unchanged | |
| 2.3 P2 stale doc | `src/engine/winner.rs:56-57` | unchanged | |
| 2.3 P3 README typo `when::` | `README.md:73` | unchanged | |
| 2.3 `WaitableTimer::not_a_timer` `.expect("an event is made")` | `src/platform/timers.rs:71` | unchanged | |
| `Win32::UI::Input` (raw input) feature | `Cargo.toml:46` enables `Win32_UI_Input_KeyboardAndMouse`, which enables `Win32_UI_Input` (`windows-0.62.2/Cargo.toml:692`); module gated at `windows-0.62.2/src/Windows/Win32/UI/mod.rs:11-12` | present (transitively) | D5 names it explicitly |

Nothing the spec relies on is already fixed or restructured away.

## Goal and non-goals

**Goal.** After this lane:
- A `hook-watchdog` thread counts keyboard Raw Input (`RIDEV_INPUTSINK`, its own message-only window) and compares it, four times a second while keys are typed, with a heartbeat the keyboard callback bumps on every call. If keys arrive and the callback stays silent for more than 1 s, it asks the main thread, which owns the hook (KS-01), to reinstall the keyboard hook. The main thread replaces the hook (new one first, then the old one off), resets the transient state the way `swch off`/`on` does (no macro is stopped), and logs it.
- The decision is a pure state machine of counts and instants (`hook::watchdog::decision`), pinned by fake-clock tests, with a stated anti-loop rule (D3).
- `swch status` → `OK: running (N bindings live; hook ok, reinstalls: 0)`; the reply still starts `OK: running (N bindings live`.
- `main_state_refused` and `raw_input_unread` counters; a panic hook that logs through the writer with a 250 ms bound; a debug-only callback hold for the manual acceptance; bench rows R14 and R15; the four 2.3 leftovers.
- Nothing injects input. The key path gains one atomic increment per event.

**Non-goals (later or never):**
- Reinstalling the foreground and desktop-switch WinEvent hooks: KS-08 is about the LL timeout; WinEvent out-of-context hooks have no documented timeout removal.
- Raw input as the "input left this desktop" signal (2.3 follow-up F5): the sink exists now; using it is a later lane.
- Replaying the keys typed while the hook was gone; stopping a Hold that fires while the hook is gone (residual R-6).
- Detecting a callback that is slow but alive: 2.1's `longest_callback_us` gauge does that.
- An uninstall guard for a main-thread panic outside a callback (follow-up FU-1).
- A machine-readable `swch status` (4.3), per-session pipe (4.1), tray indicator (4.6).

**Later items that depend on this one:** 3.5 (the hot-path budget includes the heartbeat), 4.6 (the tray shows `HookHealth`; any later raw input user in this process must share this registration, D5), 4.2 (background mode relies on the panic line reaching the file).

## Decisions

### D1: Raw input runs on a dedicated `hook-watchdog` thread, not on the 2.3 control window
**What.** A new thread `hook-watchdog` (normal priority) owns a message-only window of the system class `"Message"` (`HWND_MESSAGE`), the raw input registration for the keyboard collection (usage page `0x01`, usage `0x06`, `RIDEV_INPUTSINK`), a `platform::timers::WaitableTimer` and its own `MsgWaitForMultipleObjectsEx` loop. It reads only atomics of the hook and posts at most one reinstall request at a time to the main thread through the existing request queue (`main_thread.rs:415-427,673-676`). The main thread does nothing per key event for the watchdog.

**Why (Learn evidence; the latency owner decision).**
- "This hook is called in the context of the thread that installed it. The call is made by sending a message to the thread that installed the hook. Therefore, the thread that installed the hook must have a message loop." ([LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)) A hook call waits until the main thread is back in a message call.
- "When an application receives raw input, its message queue gets a WM_INPUT message and the queue status flag QS_RAWINPUT is set (QS_INPUT also includes this flag)." ([Raw Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input)) On the control window, every key event would add a posted `WM_INPUT` to the main queue. That wakes the main wait (`QS_ALLINPUT`, `main_thread.rs:643-650`) and costs a `PeekMessageW`, a dispatch, a `GetRawInputData` and a `DefWindowProcW` per key ("it must be read via GetRawInputData before calling DefWindowProc", [Using Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/using-raw-input)).
- `PeekMessageW` processes "Sent messages, Posted messages, Input (hardware) messages and system internal events, Sent messages (again)" ([PeekMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew)). A hook call that arrives while main dispatches a `WM_INPUT` waits for that dispatch. With raw input on main, the hook path gets slower in the interleaved case. On a dedicated thread it gets no slower.
- Microsoft's own advice in the same page: "it should run the hooks on a dedicated thread that passes the work off to a worker thread … In most cases … it should monitor raw input instead." The watcher should not be the thread it watches.
- Bench: R15 (D11) measures what the rejected option would add per key on the main thread (one posted message dispatched to a procedure that calls `GetRawInputData` and `DefWindowProcW`). R14 measures the chosen design's key path. **R15 cannot flip D1**: the dedicated thread costs the main thread zero per key by construction, and R15 only quantifies what was avoided (expected 1-3 us against R12's 4.8 us; the tester records the real number).
- Message-only sink: Microsoft's buffered-read sample does exactly this: `CreateWindowExW(0, L"Message", NULL, 0, 0, 0, 0, 0, HWND_MESSAGE, NULL, NULL, NULL)` with `{ 0x01, 0x06, RIDEV_INPUTSINK, hWnd }` ([Using Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/using-raw-input)). No class to register and no window procedure of ours.

**Alternatives rejected.**
- (a) The 2.3 control window (the roadmap text): adds per-key main-thread work (above).
- (b) `GetLastInputInfo` polling: it covers mouse input too, so it cannot tell keyboard silence apart.
- (c) A periodic `SendInput` ping: forbidden (roadmap "Deliberately out of scope"; it resets the idle timer).
- (d) A second LL hook as a probe: it dies the same way and adds a hook to every key.
- (e) A watchdog thread at raised priority: not needed. Its only cost when starved is a later detection, and D3's owed credit absorbs its lateness.

This does not move hook work off the main thread (CLAUDE.md principle 6): install, uninstall and reinstall stay on main.

**Cost.** One thread, one message-only window and one raw input registration. Per key on another core: one `WM_INPUT` and one `GetRawInputData`. While keys were typed in the last 2 s: at most 4 ticks a second. Idle: no wake-ups.

### D2: The heartbeat: one atomic increment per callback, read only at ticks
**What.** `hook::health::HealthState` (new file `src/hook/health.rs`) is a field of `KeyboardHook` (`keyboard_hook.rs:24-52`), reached by `KeyboardHook::health()`. Its `callbacks: AtomicU64` sits alone on a cache line (`#[repr(align(64))] struct Heartbeat(AtomicU64)`). `keyboard_hook_callback` calls `hook.health().callback_seen()` right after `let hook = get_hook();` (`keyboard_hook_callback.rs:353`) and before the switch gate (`:354`). The call is `fetch_add(1, AcqRel)`. It counts every call with `code >= 0`, injected events and remapping-off events included. The `code < 0` short-circuit (`:346-351`) stays untouched first.

**Why.**
- Counting before the gate keeps the watchdog working while remapping is off: a hook that dies under `swch off` is repaired before `swch on`.
- Counting injected events matches Raw Input, which also reports injected keyboard input (D6 has no injected filter).
- The watchdog reads the counter only at its ticks (D3), not per raw event. The main core keeps the line in Modified state, and a read for ownership happens at most 4 times a second.
- `AcqRel` is the house default for an RMW. The count publishes no data, and on x86-64 every RMW is one `lock xadd` (`counters.rs:118-121` gives the same reasoning).

**Alternatives rejected.**
- Reusing `record_callback`: it runs only after the gate and is reset every minute by `take()` (`counters.rs:150-157`).
- Publishing `KBDLLHOOKSTRUCT.time` from the callback: a second store per event and a comparison across two clocks (tick count against `Instant`) at 15.6 ms resolution, for precision D3 does not need.
- A split load/store: correct only for one writer, and the tests drive the global callback from many threads.

**Cost.** About 5-20 ns per event (R12/R14). No allocation, lock, Win32 call or wake-up.

### D3: The pure decision and the anti-loop rule
**What.** `src/hook/watchdog/decision.rs`, `#![forbid(unsafe_code)]`, uses no Win32, thread or clock: every instant comes in as an argument.

```rust
pub(crate) const SILENCE: Duration = Duration::from_secs(1);     // roadmap 2.4: "more than 1 s"
pub(crate) const TICK: Duration = Duration::from_millis(250);    // how often the counter is read while keys are typed
pub(crate) const ACTIVE_FOR: Duration = Duration::from_secs(2);  // ticks stop this long after the last raw key event
pub(crate) const MAX_BACKOFF_SHIFT: u32 = 6;                     // the silence doubles up to 64 s

pub(crate) struct Watchdog { /* counts and instants only:
    last_callbacks: u64, raw_events: u64, raw_at_last_tick: u64, owed: u64,
    evidence: u64, quiet_since: Instant, last_raw_at: Option<Instant>,
    ticking: bool, outstanding: bool, attempts: u32 */ }

pub(crate) enum Verdict { Idle, TickAt(Instant), Reinstall { request: ReinstallRequest, next_tick: Instant } }

impl Watchdog {
    pub(crate) fn new(now: Instant, callbacks: u64) -> Self;
    /// A raw key event the hook should see arrived. `Some(at)` asks the thread to arm a tick (first event after idle).
    pub(crate) fn raw_event(&mut self, now: Instant) -> Option<Instant>;
    pub(crate) fn tick(&mut self, now: Instant, callbacks: u64) -> Verdict;
    pub(crate) fn answered(&mut self, now: Instant, outcome: ReinstallOutcome);
    pub(crate) fn not_sent(&mut self);            // the queue was full or main is ending: retry at the next tick
    pub(crate) fn declined(&mut self, now: Instant); // keyboard input is not on this desktop: drop the evidence
}
```

`tick` semantics, in order:
1. `dc = callbacks - last_callbacks` (wrapping) and `dr = raw_events - raw_at_last_tick`; both bases move to now.
2. If `dc > 0`, the hook is alive: `quiet_since = now`, `evidence = 0`, `owed = dc`, `attempts = 0`.
3. Otherwise:
   - `explained = min(dr, owed)` and `owed = 0`: the credit lasts one interval. It covers raw events that trail their callbacks into the next interval, whatever the order of raw input and the hook, and a watchdog thread that ran late.
   - `decide = !outstanding && evidence >= 2 && now >= quiet_since + SILENCE * 2^min(attempts, 6)`.
   - Then `evidence += dr - explained`.
   - If `decide`: `outstanding = true` and return `Reinstall { request: ReinstallRequest { seen: callbacks, silent_for: now - quiet_since, attempt: attempts + 1 }, next_tick: now + TICK }`.
4. Return `TickAt(now + TICK)` while `outstanding`, while `evidence > 0`, or while `last_raw_at` is within `ACTIVE_FOR`. Otherwise `ticking = false` and return `Idle`.

`evidence` is checked **before** the current interval's raw events are added. Every raw event that counts is therefore at least one tick (250 ms) old without a callback, which absorbs the case where raw input arrives before its own hook call.

`answered(now, outcome)`:
- `outstanding = false`.
- `Reinstalled` and `Failed` add 1 to `attempts`. `Alive` and `NotInstalled` add nothing.
- `evidence = 0`, `owed = 0`, `quiet_since = now`.
- `last_callbacks` stays, so a callback of the new hook resets `attempts` at the next tick.

**The anti-loop rule (stated).**
1. A request needs three things: keys arrived without a callback (at least 2 unexplained raw key events, each at least 250 ms old); no callback start seen for `S = 1 s × 2^min(n, 6)` (`n` = reinstall attempts since the last callback seen); and keyboard input on the daemon's desktop at that moment (D5).
2. At most one request is outstanding. It waits in main's request queue while main is busy. No new evidence counts until main answers.
3. Main decides on fresh evidence (D4). It first dispatches every pending sent message, which includes pending hook calls. It then reinstalls only if the heartbeat has not moved since the watchdog's snapshot. A hook that answers once main is free gives `Alive`, and nothing changes.
4. After any answer, a new request needs a new full window `S` and new evidence after the answer.
5. Each reinstall or failed attempt with no callback in between doubles `S`: 1, 2, 4 … 64 s. The first callback seen resets it to 1 s.

Consequences:
- A busy main thread gives **at most one request per busy period**, because the request stays outstanding the whole time.
- That request becomes a reinstall only if no hook call reaches main once main is free. In that case main was unresponsive for at least `S ≥ 1 s` while keys arrived, which is at least the maximum `LowLevelHooksTimeout` ("The maximum timeout value the system allows is 1000 milliseconds", LowLevelKeyboardProc). Windows therefore timed out a hook call, and per Learn "on Windows 7 and later, the hook is silently removed". So the reinstall is not needless.
- A reinstall that does not help is spaced out exponentially, so there is no loop.

**Alternatives rejected.**
- Reading the counter at every raw event: one read for ownership on the hook's cache line per key.
- Measuring the silence from the first unexplained raw event: about 1 s later detection in the hold case, with no robustness gain once the evidence rule above holds.
- A probe round trip ("main answered a ping"): the request itself is the probe (rule 3).

**Cost.** About 120 lines of pure code. Detection latency while typing is at most `S + 2 × TICK` after the last callback.

### D4: The reinstall on the main thread: new hook first, then the old one off; never install what is not installed
**What.**
- `KeyboardHook::reinstall_keyboard_hook(&self) -> windows::core::Result<Reinstalled>` (main thread only):
  - If `hook_handle` is null, return `Ok(Reinstalled::NotInstalled)` and make no Win32 call.
  - Otherwise `SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook_callback), None, 0)?`. On `Err` the old handle stays and the error is returned.
  - Then `hook_handle.swap(new, AcqRel)`, then `UnhookWindowsHookEx(previous)`.
  - `Ok` gives `OldHook::StillInstalled`; `Err(e)` gives `OldHook::Gone(e.code())`, which is diagnostic only.
  - Returns `Ok(Reinstalled::Replaced { old })`.
- `Request::ReinstallHook { request: ReinstallRequest, reply: SyncSender<ReinstallOutcome> }` in `main_thread.rs:415-427`.
- `MainThreadHandle::request_reinstall(&self, request) -> Result<Receiver<ReinstallOutcome>, MainThreadError>`: `try_send` plus a wake, and **no wait**. The `Full`/`Disconnected` mapping is the one of `main_thread.rs:472-479`.
- `serve` (`:751-771`) gets a new arm that calls `reinstall_if_silent(hook, request, || hook.reinstall_keyboard_hook())` and sends the reply with `try_send`.
- `reinstall_if_silent(hook: &KeyboardHook, request: ReinstallRequest, replace: impl FnOnce() -> windows::core::Result<Reinstalled>) -> ReinstallOutcome` (the closure is the test seam):
  1. `dispatch_sent_messages()`: `PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE | PM_QS_SENDMESSAGE)`. It processes pending sent messages, hook calls among them, and removes nothing.
  2. If `hook.health().callbacks() != request.seen`: log `[INFO] the keyboard hook answered before its reinstall; it was kept` and return `Alive`.
  3. `replace()`:
     - `NotInstalled` gives `NotInstalled`, with no log (shutdown or `WM_ENDSESSION` came first).
     - `Replaced { old }` runs `with_main(MainState::hook_reinstalled)`, logs `[WARN] the keyboard hook got no key event for {ms} ms while keys were typed; reinstalled it ({old}; longest_callback_us={us})` and `[INFO] state reset: the keyboard hook was reinstalled`, records the reinstall in `HealthState` (from C18), and returns `Reinstalled`.
     - `Err(e)` logs `[ERROR] could not reinstall the keyboard hook ({e}); the old one is kept`, records a failed attempt (from C18) and returns `Failed`.
- `MainState::hook_reinstalled(&mut self)` runs `self.reset_engine(ResetReason::HookReinstalled)`. It cancels no action (D13).

**Why.**
- "The SetWindowsHookEx function always installs a hook procedure at the beginning of a hook chain" ([Hooks Overview](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks)).
- "If the hook procedure is not being called concurrently, the hook procedure is removed immediately before UnhookWindowsHookEx returns" ([UnhookWindowsHookEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex)).
- Hook calls reach main only inside a message call (D1 quote), and none happens between the two calls. So a key event sees exactly one live hook of ours: never two (no double processing through `CallNextHookEx`) and never none.
- If the install fails, the old handle is kept, so the daemon is never left with fewer hooks than before.
- Every path runs on main, in the drain (`main_thread.rs:670-689`), outside `with_main` and outside every callback. `install`/`uninstall`/`reinstall` are serialized on one thread, so a stale request after `uninstall()` finds a null handle and installs nothing.
- `PM_QS_SENDMESSAGE`: "Process all sent messages". "During this call, the system dispatches (DispatchMessage) pending, nonqueued messages" ([PeekMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew)). That it covers a pending LL hook call is **(unverified)**, Q7. If it does not, the check still sees every hook call that main dispatched before the drain. Correctness does not depend on it.

**Alternatives rejected.**
- Unhook first, then set: there is a gap with no hook, and a failed set leaves none.
- `install()` again: it refuses a non-null handle (`keyboard_hook.rs:142-147`) by design.
- A reinstall from the watchdog thread: KS-01, because hooks belong to the installing thread.

**Cost.** The reinstall is rare and runs in the drain. Its main-thread time is one peek, two user32 calls, 254 `GetAsyncKeyState` calls in the reset (D13) and two log lines.

### D5: The watchdog thread
**What.** `src/hook/watchdog/mod.rs`:

```rust
pub(crate) trait RequestReinstall: Send + 'static {
    fn request(&self, request: ReinstallRequest) -> Result<Receiver<ReinstallOutcome>, MainThreadError>;
}
impl RequestReinstall for MainThreadHandle { /* calls request_reinstall */ }   // here: no new module cycle

pub(crate) struct HookWatchdog { thread_id: u32, join: Option<JoinHandle<()>>, done: Receiver<()> }
impl HookWatchdog {
    pub(crate) const START_TIMEOUT: Duration = Duration::from_secs(5);
    pub(crate) const STOP_TIMEOUT: Duration = Duration::from_secs(1);
    pub(crate) fn start(health: &'static HealthState, requests: impl RequestReinstall) -> io::Result<Self>;
    #[cfg(test)] pub(crate) fn start_synthetic(health: &'static HealthState, requests: impl RequestReinstall) -> io::Result<Self>;
    #[cfg(test)] pub(crate) fn post_synthetic_raw(&self);   // a WM_APP + 2 thread message taken as one qualifying raw key event
    pub(crate) fn stop(self) -> bool;                        // true if joined within STOP_TIMEOUT
}
impl Drop for HookWatchdog { /* stop */ }
```

- Thread `hook-watchdog` (`thread::Builder`, so a failed spawn is an `io::Error` and never a panic). The start and stop pattern follows `Executor::start`/`stop_thread` (`executor/mod.rs:281-355`): a ready channel (`START_TIMEOUT`) and a done channel (`STOP_TIMEOUT`).
- On the thread:
  1. Make the queue (`PeekMessageW` `PM_NOREMOVE`, as `main_thread.rs:585-590`).
  2. Make the window: `CreateWindowExW(WINDOW_EX_STYLE(0), w!("Message"), None, WINDOW_STYLE(0), 0,0,0,0, Some(HWND_MESSAGE), None, None, None)`.
  3. Register raw input: `RegisterRawInputDevices(&[RAWINPUTDEVICE { usUsagePage: 0x01, usUsage: 0x06, dwFlags: RIDEV_INPUTSINK, hwndTarget: hwnd }], size_of::<RAWINPUTDEVICE>() as u32)`. `start_synthetic` skips this step, so **no test taps the owner's keyboard**.
  4. Make the `WaitableTimer`.
  5. Send ready, then loop.
- Guards drop in this order:
  - `RawInputRegistration` unregisters with `RIDEV_REMOVE` and `hwndTarget` null ("If RIDEV_REMOVE is set and the hwndTarget member is not set to NULL, then RegisterRawInputDevices function will fail", [RAWINPUTDEVICE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputdevice)).
  - `OwnedWindow` calls `DestroyWindow` on this thread.
  - `Watched` (from C18) runs `health.set_watched(false)`, also on unwind.
- Loop, in the house shape (`main_thread.rs:621-665`):
  1. `MsgWaitForMultipleObjectsEx(Some(&[timer]), INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)`.
  2. Pump with `PeekMessageW(PM_REMOVE)`:
     - `WM_QUIT` ends the loop.
     - `WM_INPUT` with `msg.hwnd == window`: `raw_key_event(msg.lParam)`; if true, `watchdog.raw_event(now)` and arm the returned tick; log the first one once. Then `DispatchMessageW(&msg)`, which reaches `DefWindowProcW` for the cleanup that `RIM_INPUT` asks for ("The application must call DefWindowProc so the system can perform cleanup", [WM_INPUT](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-input)).
     - Every other message: `DispatchMessageW`.
  3. Poll the outstanding reply with `try_recv`: `Ok(outcome)` calls `answered`; `Disconnected` calls `not_sent`.
  4. If the tick is due by `Instant::now()`, call `tick(now, health.callbacks())`:
     - `Idle`: cancel the timer.
     - `TickAt(t)`: set the timer.
     - `Reinstall { request, next_tick }`:
       - check `platform::desktop::input_is_on_this_desktop()` on this thread. `Ok(true)` sends the request through `requests.request(request)`: `Ok(rx)` keeps `rx`, `Err` calls `not_sent`. `Ok(false)` or `Err` calls `declined(now)`;
       - if `request.attempt >= 2`, log `[WARN] the keyboard hook still gets no key events after its reinstall; asking again after {s} s of silence`;
       - then set the timer for `next_tick`.
- `raw_key_event(lparam) -> bool` is the only function that touches key data:
  - `GetRawInputData(HRAWINPUT(lparam), RID_INPUT, Some(&mut raw as *mut RAWINPUT as *mut c_void), &mut size, size_of::<RAWINPUTHEADER>() as u32)` into a stack `RAWINPUT` (`size = size_of::<RAWINPUT>()`; it is 8-aligned, and the doc asks for DWORD alignment).
  - `u32::MAX` bumps `Counter::RawInputUnread` and returns false.
  - Else, if `raw.header.dwType == RIM_TYPEKEYBOARD.0`, return `qualifies(raw.data.keyboard.MakeCode, raw.data.keyboard.VKey)`. The union read is justified by the `dwType` check.
- `qualifies(make_code: u16, vkey: u16) -> bool { make_code != KEYBOARD_OVERRUN_MAKE_CODE && (1..=254).contains(&vkey) }`, with a local `const KEYBOARD_OVERRUN_MAKE_CODE: u16 = 0xFF`. The `windows` crate has it only under the absent `Win32_Devices_HumanInterfaceDevice` feature (`HumanInterfaceDevice/mod.rs:6992`). This follows Microsoft's sample "Ignore key overrun state and keys not mapped to any virtual key code" ([RAWKEYBOARD](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawkeyboard)) and the hook's documented `vkCode` range 1-254 ([KBDLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct)). A key with VK 255 (some Fn combinations) never reaches the LL hook and must not count as silence.
- The process may register one raw-input window per device class ("Only one window per raw input device class may be registered to receive raw input within a process", [RegisterRawInputDevices](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerrawinputdevices)). The module doc says that any later raw-input user in this process (4.6) must share this one.
- `App::run` (`app.rs`):
  - After `IPC server listening` (`:242`): `let watchdog = match HookWatchdog::start(hook.health(), main.handle()) { Ok(w) => Some(w), Err(error) => { log::warn!("the hook watchdog is off ({error}): a keyboard hook that Windows removes is not reinstalled until the daemon restarts"); None } };`. The watchdog is not fatal, and it starts only once the hooks and the pipe are live.
  - After `drop(raised)` (`:286`) and before `drop(main)` (`:289`): stop it. If `stop()` returns false, log `[WARN] the hook watchdog did not stop within 1 s; exiting without it`.
  - Every early return drops the guard. It is declared after `control_window` and `executor`, so it drops first.
- `Cargo.toml:41-57` names `"Win32_UI_Input"` explicitly. It is already on through `Win32_UI_Input_KeyboardAndMouse`, and `Cargo.lock` is unchanged.

**Why.** The executor's shape is proven (2.1). A guard per handle keeps every exit path clean (CLAUDE.md principle 7). The desktop check costs one `UOI_IO` read per decision, not per event. It guards against raw input that reaches the daemon while its desktop has no input (unverified either way, Q5).

**Alternatives rejected.**
- `SetTimer`: this would be the only `WM_TIMER` user. The waitable timer pattern is the house one (`platform/timers.rs:1-11`).
- Registering a class: `"Message"` needs none.
- `GetRawInputBuffer`: buffered reads are for high-rate devices ("useful for high-frequency devices such as mice at 1000Hz", Raw Input Overview). Keyboards are not.

### D6: Raw input filtering and "no key data kept"
Only `raw_key_event` reads `MakeCode`/`VKey`, into a stack local, and returns a `bool`. The decision state is counts and instants (D3 struct). Nothing logs a raw value: the only watchdog log lines are D5's and D4's, which carry durations and counts. A scan test pins this (C15 `no_key_data_leaves_the_raw_input_read`).

Injected input is not filtered (`hDevice == NULL` would mark it) because the heartbeat counts injected callbacks too (D2). The two sides stay symmetric.

### D7: `HookHealth` in `swch status`
**What.**
- `hook::health` (`pub mod health`):
  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)]
  pub struct HookHealth { pub watched: bool, pub reinstalls: u64, pub last_reinstall: Option<SystemTime>, pub last_attempt_failed: bool }
  impl fmt::Display for HookHealth // "hook ok" | "hook not watched" | "hook reinstall failed", then ", reinstalls: N", then ", last <UTC>" if any
  ```
  The state word is `not watched` if `!watched`, else `reinstall failed` if `last_attempt_failed`, else `ok`. The time comes from `logging::utc_timestamp(SystemTime) -> String`, a new `pub(crate)` wrapper of `sink::write_timestamp` (`sink.rs:120-133`), in the log's format `2026-10-19T09:41:07.123Z`.
- `HealthState` gains:
  - `watched: AtomicBool`, written by the watchdog thread;
  - `reinstalls: AtomicU64`, `last_reinstall_ms: AtomicU64` (milliseconds since `UNIX_EPOCH`, 0 = none) and `last_attempt_failed: AtomicBool`, written by main;
  - `snapshot()`.
- Orderings:
  - `record_reinstall(at)` stores `last_reinstall_ms` (`Release`), then `reinstalls.fetch_add(1, AcqRel)`, then `last_attempt_failed.store(false, Release)`.
  - `snapshot` loads `reinstalls` (`Acquire`), then `last_reinstall_ms` (`Acquire`). A reader that sees count `n` sees the time of reinstall `n` or a later one.
- `DaemonStatus { enabled, bindings, hook: HookHealth }` (`ipc_server.rs:46-52`). `dispatch` (`:186-190`) gives `format_ok(&format!("{state} ({bindings} bindings live; {hook})"))`. `DaemonHandle::status` (`app.rs:86-91`) adds `hook: self.hook.health().snapshot()`.
- Examples:
  - `OK: running (12 bindings live; hook ok, reinstalls: 0)`
  - `OK: running (12 bindings live; hook ok, reinstalls: 1, last 2026-10-19T09:41:07.123Z)`
  - `OK: disabled (12 bindings live; hook not watched, reinstalls: 0)`

**Why.**
- The reply keeps the prefix `OK: running (N bindings live`. `swch` prints the message unchanged (`swch.rs:148-151`), and `swch open`'s probe accepts any `Ok` (`swch.rs:77-81`).
- `reinstalls: N` is the acceptance's own token.
- Gauges: `longest_callback_us` is the evidence of what killed the hook, so it goes into the reinstall log line (D4), read with `COUNTERS.longest_callback()` without a take. `executor_unavailable` and `active_runs` describe the action executor, which since 2.1 runs off the hook thread. They say nothing about the keyboard hook, so `HookHealth` leaves them out.

**Alternatives rejected.**
- A second reply line: the protocol is one line (`ipc.rs:1-8`).
- A new `swch health` command: the spec says `swch status`.

### D8: The re-entry refusal counter
`Counter::MainStateRefused` (`main_state_refused`) is appended to `Counter::ALL` (`counters.rs:48-56`), with a `Display` arm and a slot, `SLOTS` 7 → 8. In `with_main`'s refusal branch (`main_thread.rs:201-205`), `counters::bump(Counter::MainStateRefused)` comes next to the test-only `REFUSED` (kept: the tests need a per-thread count). A thread without a main state is not a refusal and is not counted.

`#[cfg(test)] pub fn get(&self, counter: Counter) -> u64` (`Acquire` load) lets a test read one slot without `take`.

C16 adds `Counter::RawInputUnread` (`raw_input_unread`) the same way (`SLOTS` 9).

### D9: The panic hook and its bound
**What.** `src/logging/panic.rs`:

```rust
pub(crate) const REPORT_BUDGET: Duration = Duration::from_millis(250);
const EPISODE: Duration = Duration::from_secs(1);
pub(crate) fn install();                                   // std::panic::set_hook(Box::new(report)); called by logging::init when it installed the logger
pub(crate) fn panic_text(thread: Option<&str>, location: Option<(&str, u32, u32)>, payload: &(dyn Any + Send)) -> String;
pub(crate) fn budget_end(now: Instant, previous: Option<Instant>) -> Instant; // previous end if now < previous + EPISODE, else now + REPORT_BUDGET
pub(crate) fn report_on(tx: &SyncSender<WriterMsg>, line: LogLine, deadline: Instant) -> bool;
#[cfg(test)] pub(crate) fn install_with(writer: fn() -> Option<&'static SyncSender<WriterMsg>>);
```

- `report(info)` builds `thread '<name>' panicked at <file>:<line>:<col>: <message>`:
  - The message is the payload as `&str` or `String`, else `Box<dyn Any>`. A thread without a name is `<unnamed>` and a missing location is `<unknown>`, as Rust's default hook writes them.
  - It becomes a `LogLine` at `Error` with target `key_switch_rs::panic` (an own target, so the line reads `[ERROR] thread 'main' panicked at …`).
  - The deadline comes from a thread-local `Cell<Option<Instant>>` through `budget_end`.
  - The line goes out with `writer::send_by(tx, Line, deadline)`, which retries while the queue is full; then `writer::flush_by(tx, deadline)`.
- `send_by` (`writer.rs:99-117`) becomes `pub(crate)`. `flush_on` (`:132-142`) is refactored over a new `pub(crate) fn flush_by(tx, deadline) -> bool`.
- On the `log-writer` thread itself, or with no writer, the hook writes `console_text()` to stderr and does not wait. Waiting there would wait for itself.
- The hook builds one line with `write!` into a `String`. It never formats with `format!` on a value that can panic, never calls `log::`, and does not chain to the default hook. The default hook writes stderr from the panicking thread with no bound; a console in a QuickEdit selection can block it.

**The bound.**
- A panic holds its thread in the hook for at most `REPORT_BUDGET` = 250 ms, plus the time to build one line.
- A second panic on the same thread within 1 s of that budget's end shares it. That is std's "panic in a function that cannot unwind", if std runs the hook for it; that is **(unverified)**, Q8, and the bound holds either way.
- In the keyboard callback the panic then aborts the process: "As of 1.81, the non-unwind ABIs (e.g., "C") will now abort on uncaught unwinds" ([Rust 1.81.0](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/)), and `extern "system"` is a non-unwind ABI. So keyboard input waits at most about 250 ms for the dying daemon, under the 1000 ms maximum of `LowLevelHooksTimeout`.
- The writer writes the file before the console (`writer.rs:182-192`), so the line reaches `daemon.log` even when the console blocks.

**Alternatives rejected.**
- A synchronous file write from the hook (gaps.md:232's wording): it bypasses the writer that owns the files (CLAUDE.md principle 6) and races its rotation.
- `[FATAL]`: a panic the executor catches is not fatal (`errors.md:217`).
- Chaining to the default hook: a duplicate line on the console and an unbounded stderr write.

### D10: The debug hold exists only in debug builds
**What.** `src/hook/debug_hold.rs`, declared `#[cfg(debug_assertions)] pub(crate) mod debug_hold;` in `hook/mod.rs`:
- `ENV_VAR = "KS_DEBUG_HOOK_HOLD"` and `HOLD = 1500 ms`.
- `static ARMED: AtomicU16` (0 = off): one `Release` store by main at startup, `Acquire` loads in the callback.
- `pub(crate) fn arm_from_env()`, called by `App::run` before `hook.install()` (`app.rs:221`) under `#[cfg(debug_assertions)]`:
  - It parses the variable with `data::vk_name::parse_vk`.
  - If it names a key: `[WARN] debug build: the keyboard callback holds 1.5 s at each <key> key-down (KS_DEBUG_HOOK_HOLD)`.
  - If it names none: `[WARN] KS_DEBUG_HOOK_HOLD names no key ("…"); no hold`.
- `pub(crate) fn should_hold(armed: u16, wparam: WPARAM, vk_code: u32) -> bool` (pure: key-down or sys-key-down of the armed key).
- `pub(crate) fn hold_if_asked(wparam, vk_code)` sleeps `HOLD` when `should_hold`. The callback calls it under `#[cfg(debug_assertions)]` right after the deref (`keyboard_hook_callback.rs:368`), so only with remapping on.

**Why.**
- Release builds have no `debug_assertions`, so the code, the variable and the per-event load do not exist in the shipped (elevated, manifest) daemon.
- A debug build needs both the variable at startup and the named key to hold.
- `cargo test` never runs `App::run`, so `ARMED` stays 0 under test.
- The CI release builds (`ci.yml:40-41,57-78`) compile without it. Its tests are inside the `cfg`ed module, so `cargo test --release` (the bench) still compiles.

**Alternatives rejected.**
- A Cargo feature: an extra build matrix, and an elevated release daemon could carry it by mistake.
- A `swch` command: it would exist in release.

**Cost.** None in release builds. In debug builds, one atomic load per enabled event.

### D11: Bench rows R14 and R15, and the comparison rule for this lane
- **R14**: `keyboard_hook_callback`, unbound A down and up (as R12), with `HookWatchdog::start_synthetic(get_hook().health(), recorder)` running. Before each timed call, outside the timing, the bench posts one synthetic raw event, so that the watchdog thread works next to the callback. 2 000 samples.
- **R15** (D1 evidence): a bench-only window class `ks-bench-r15` whose procedure, on `WM_APP + 0x15`, calls `GetRawInputData(HRAWINPUT(null), RID_INPUT, …)` (fails fast: an invalid handle) and then `DefWindowProcW`. Timed on the bench thread: `PostMessageW` + `PeekMessageW(PM_REMOVE)` + `DispatchMessageW`. 2 000 samples. This is the in-thread lower bound of what raw input on the control window would add per key: it has no wake-up and no kernel copy. The class is unregistered after the row.

Rules (3 runs each on master and HEAD, median of p50s, largest p99; as CLAUDE.md:65-67):

| Rows | Against | Bound | Justified extra work |
|---|---|---|---|
| R1-R3, R6-R8 | master | 2.2 D10 (`p2-2.2-timers/cut.md:194`) | none |
| R4, R5 | master | 2.2 D10 | one heartbeat RMW |
| R9-R12 | master | HEAD p50 ≤ 1.05 × master p50 + 100 ns; p99 ≤ 1.5 × master p99 + 2 µs | one heartbeat RMW |
| R11 | master (≈ 5.4 µs, CLAUDE.md:66) | as R9-R12 | one heartbeat RMW |
| R13 | master | 2.3 D14 (HEAD R13 ≤ HEAD R7 + 20 µs) | none |
| R14 | HEAD R12 | p50 ≤ R12 p50 + 300 ns; p99 ≤ R12 p99 + 2 µs | the watchdog thread on another core |
| R15 | — | reported in results.md; no bound | evidence row |

### D12: The 2.3 leftovers, as four small commits
- **P1**: `main_thread.rs:1667,1671,1674` become `0x1001`/`0x2001`, and the expected names at `:1677,:1686` become `p4097.exe`/`p8193.exe`. The doc of `MovingQuery` (`:1636-1638`) says why: a real PID is a multiple of 4, and `foreground.rs:132-137` would take the window for this process's own.
- **P2**: `winner.rs:56-57` becomes "…its `when:` is evaluated only once its keys match: the executable comes from the main thread's foreground cache and a title condition reads the window's title (roadmap item 2.3)".
- **P3**: `README.md:73`: `when::` becomes `when:`.
- **`not_a_timer`**: `timers.rs:71` becomes `.expect("invariant: CreateEventW with no name and no security attributes makes an event")`.

### D13: The reset after a reinstall
**What.**
- `ResetReason::HookReinstalled` (`engine/mod.rs:379-401`), with `Display` "the keyboard hook was reinstalled".
- `Engine::reset` (`:642-655`) handles it with `self.keys.resync_from_os(env)`, the arm of `RemappingSwitched`. The doc list (`:619-638`) and `KeyState::resync_from_os`'s doc (`keys.rs:112-125`) name it.
- The scenario harness gets `pub(crate) fn hook_reinstalled(mut self) -> Self`: `engine.reset(&env, HookReinstalled)`, `drop_stale_timers()` (`testing.rs:751`), and a step label.

**Why.**
- While the hook was gone the engine saw no event and Windows saw every one. That is exactly the remapping-off case `resync_from_os` was written for (`keys.rs:112-118`).
- A swallowed key's record is kept, and the key is released when Windows has it up. So the owner decision "the next press of a swallowed, non-repeating trigger key fires" (CLAUDE.md:154) holds after a hook loss too.
- Pending gestures are dropped and the generation moves on (KS-11). A Hold armed before the loss cannot fire after the repair.
- Macros are not cancelled: they run on the executor, their keys carry the sentinel and pass whatever the hook state, and the user did not ask to stop them.
- The foreground cache is not resynced: the foreground WinEvent hook was not affected (`main_thread.rs:250-254` keeps that resync for `RemappingSwitched`).

**Alternatives rejected.**
- `release_all` (`InputLeft`): it reads no OS state because the OS is no witness on another desktop. Here the OS saw everything on this desktop.
- No reset: a key released while the hook was gone stays held for the engine, and its next press reads as a repeat and is swallowed without firing.

## Win32 API choices

| Purpose | `windows` path (0.62.2 file:line) | Feature (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| register / unregister the keyboard sink | `Win32::UI::Input::RegisterRawInputDevices(&[RAWINPUTDEVICE], u32) -> Result<()>` (`UI/Input/mod.rs:61`); `RAWINPUTDEVICE` (`:145`), `RIDEV_INPUTSINK` (`:254`), `RIDEV_REMOVE` (`:258`) | `Win32_UI_Input` (yes, through `Win32_UI_Input_KeyboardAndMouse`; C16 names it) | hook-watchdog | register `Err`: watchdog off, warn; unregister `Err`: ignored at exit | [RegisterRawInputDevices](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerrawinputdevices), [RAWINPUTDEVICE](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputdevice), [Raw Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input) |
| read one event | `GetRawInputData(HRAWINPUT, RAW_INPUT_DATA_COMMAND_FLAGS, Option<*mut c_void>, *mut u32, u32) -> u32` (`:36`); `RAWINPUT` (`:122`), `RAWINPUTHEADER` (`:195`), `RAWKEYBOARD` (`:203`), `RID_INPUT` (`:317`), `RIM_TYPEKEYBOARD` (`:319`), `HRAWINPUT` (`:67`) | `Win32_UI_Input` (yes) | hook-watchdog | `u32::MAX`: counted in `raw_input_unread` | [GetRawInputData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getrawinputdata), [RAWKEYBOARD](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawkeyboard), [WM_INPUT](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-input) |
| message-only sink window | `CreateWindowExW` (`UI/WindowsAndMessaging/mod.rs:430`), `HWND_MESSAGE` (`:4060`), `DestroyWindow` (`:534`), `WM_INPUT` (`:6990`) | `Win32_UI_WindowsAndMessaging` (yes) | hook-watchdog | create `Err`: watchdog off; destroy `Err`: ignored at exit | [Using Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/using-raw-input) |
| watchdog loop and wake | `MsgWaitForMultipleObjectsEx` (`:1795`), `PeekMessageW` (`:1842`), `DispatchMessageW` (`:575`), `PostThreadMessageW` (`:1872`); `platform::timers::WaitableTimer` | same, `Win32_System_Threading` (yes) | hook-watchdog; stop posted by main | wait `WAIT_FAILED`: the loop ends, watched false, logged | [MsgWaitForMultipleObjectsEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex) |
| replace the hook | `SetWindowsHookExW` (`:2316`), `UnhookWindowsHookEx` (`:2412`) | `Win32_UI_WindowsAndMessaging` (yes) | main (drain) | set `Err`: old kept, `Failed`; unhook `Err`: `OldHook::Gone`, logged | [SetWindowsHookExW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowshookexw), [UnhookWindowsHookEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-unhookwindowshookex), [Hooks Overview](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks), [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc) |
| dispatch pending hook calls before the check | `PeekMessageW(.., PM_NOREMOVE \| PM_QS_SENDMESSAGE)` (`PM_QS_SENDMESSAGE` `:5492`, `PM_NOREMOVE` `:5487`, `BitOr` `:5439`) | same (yes) | main (drain) | none | [PeekMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew) (LL hook calls covered: **unverified**, Q7) |
| is input on this desktop | `platform::desktop::input_is_on_this_desktop` (`platform/desktop.rs:30-51`) | `Win32_System_StationsAndDesktops` (yes) | hook-watchdog | `Err`: declined | [GetUserObjectInformationW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getuserobjectinformationw) |
| panic hook | `std::panic::set_hook`, `PanicHookInfo` | std (1.81+, MSRV 1.88) | any thread | never panics | [set_hook](https://doc.rust-lang.org/std/panic/fn.set_hook.html), [Rust 1.81.0](https://blog.rust-lang.org/2024/09/05/Rust-1.81.0/) |
| R15 only | `RegisterClassW` (`:1921`), `UnregisterClassW` (`:2425`), `DefWindowProcW` (`:475`), `PostMessageW` (`:1857`) | `Win32_Graphics_Gdi`, `Win32_System_LibraryLoader` (yes) | bench thread | asserted | — |

**Injection.** No new `SendInput` path, no sentinel change. `LLKHF_INJECTED`, the extended flag and `scanCode` play no part: the heartbeat counts every call, and raw input is filtered by `MakeCode`/`VKey` only (D5).

**Elevation and UIPI.**
- The release daemon is elevated, and its sink gets raw input whatever the foreground's integrity.
- A non-elevated (debug) daemon's LL hook is not called for an elevated foreground (README:761-765). If its sink still gets raw input there, D3's backoff limits the reinstalls to 1, 2, 4 … 64 s apart (residual R-5).
- Pipe DACL and token inheritance: unchanged.

## Thread map after the lane

| Thread | Owns | New work |
|---|---|---|
| main (`THREAD_PRIORITY_HIGHEST`) | three hooks, control window, engine, gesture timer | heartbeat RMW per callback; in the drain, `Request::ReinstallHook` (peek, replace, reset, log, record) |
| `hook-watchdog` (new, normal) | `"Message"` window, raw input registration, `WaitableTimer`, `Watchdog` state | `WM_INPUT` read per key event; ticks while active; at most one request outstanding |
| `log-writer` | console, files | panic lines |
| IPC workers | — | `status` reads `HealthState` atomics |
| any thread | — | the panic hook (≤ 250 ms) |

## Touch set

| File | Change | Why |
|---|---|---|
| `src/hook/health.rs` (new) | `HealthState` (heartbeat on its own line; watched, reinstalls, last time, last attempt failed), `HookHealth` + `Display`, `ReinstallRequest`, `ReinstallOutcome` + `Display`; tests | D2, D4, D7 |
| `src/hook/watchdog/mod.rs` (new) | `HookWatchdog`, `RequestReinstall` + impl for `MainThreadHandle`, window/raw input/timer guards, loop, `raw_key_event`, `qualifies`; tests incl. two scans | D1, D5, D6 |
| `src/hook/watchdog/decision.rs` (new) | pure `Watchdog`, `Verdict`, constants; fake-clock tests | D3 |
| `src/hook/debug_hold.rs` (new, debug only) | `arm_from_env`, `should_hold`, `hold_if_asked`, `parse`; tests | D10 |
| `src/hook/mod.rs` | `pub mod health; pub(crate) mod watchdog; #[cfg(debug_assertions)] pub(crate) mod debug_hold;` | modules |
| `src/hook/keyboard_hook.rs` | field `health`, `health()`, `reinstall_keyboard_hook`, `Reinstalled`, `OldHook`; `is_installed` doc (`:223`); test | D2, D4 |
| `src/hook/keyboard_hook_callback.rs` | heartbeat at `:353`; debug-hold call after `:368`; module docs (`:4-34`, Threads); tests (heartbeat, debug-hold scan) | D2, D10 |
| `src/hook/main_thread.rs` | `Request::ReinstallHook`, `request_reinstall`, `serve` arm, `reinstall_if_silent`, `dispatch_sent_messages`, `MainState::hook_reinstalled`, refusal bump in `with_main`; module docs (`:1-53`); P1 test fix; tests | D4, D8, D12 |
| `src/hook/ipc_server.rs` | `DaemonStatus.hook`, `dispatch` text, `FakeControl`, tests | D7 |
| `src/core/app.rs` | `DaemonHandle::status`; watchdog start/stop; `arm_from_env` (debug); `run` docs (`:166-181`); test update | D5, D7, D10 |
| `src/engine/mod.rs` | `ResetReason::HookReinstalled`, `Display`, reset arm, docs | D13 |
| `src/engine/keys.rs` | `resync_from_os` doc names the reinstall (`:112-125`) | D13 |
| `src/engine/testing.rs` | `hook_reinstalled()` step | D13 |
| `src/engine/scenarios.rs` | 4 scenarios | D13 |
| `src/engine/winner.rs` | doc `:56-57` | D12 P2 |
| `src/logging/counters.rs` | `MainStateRefused`, `RawInputUnread`, `SLOTS` 9, `#[cfg(test)] get`; tests | D8 |
| `src/logging/mod.rs` | `pub(crate) mod panic`; `init` installs it; `utc_timestamp`; hook-path scan list (`:393-449`) adds `src/hook/health.rs` `fn callback_seen(` and `src/hook/debug_hold.rs` `fn hold_if_asked(` | D7, D9 |
| `src/logging/panic.rs` (new) | panic hook; tests incl. child process | D9 |
| `src/logging/writer.rs` | `send_by` `pub(crate)`; `flush_by`; `flush_on` over it | D9 |
| `src/logging/sink.rs` | `write_timestamp` `pub(crate)` | D7 |
| `src/platform/timers.rs` | `:71` expect text | D12 |
| `src/bench.rs` | R14, R15, doc list | D11 |
| `tests/ipc_roundtrip.rs` | `TestControl` carries a `HookHealth`; status assertions with the new text; new test | D7 |
| `Cargo.toml` | `"Win32_UI_Input"` in the feature list | D5 |
| `README.md` | `:73` typo (C4). C6: Log file counters sentence (`:854-857`). C16: tree (`:64-70`); Implementation Features (watchdog bullet, `:920-935`); Technologies (`Win32_UI_Input`, `:891-906`); Known Limitations (R-1, R-5, R-6, R-7). C18: Typical session (`:776`), `swch status` row (`:789`). C20: panic sentence in Log file. C22: debug hold note | user-visible changes |
| `CHANGELOG.md` | `[Unreleased]`: Added (`:14`), Changed (`:174`), Fixed (`:539`), per commit | Keep a Changelog |
| `book/src/config/errors.md` | §18.6 rows (`:199-227`): watchdog, reinstall, state reset, panic, debug hold; counters row (`:220`) gains `main_state_refused`, `raw_input_unread` | new log lines |
| `src/config/default_config.ron` | **no change**: no config meaning changes | — |
| other `book/src/config/*.md` | **no change** | no config meaning changes |

**Phase-2 book sync** (listed by the developer in `impl.md` with file:line; not edited in this lane):
- `architecture.md`: thread table (`hook-watchdog`), KS-08 root-cause row `:577`, key-event path (heartbeat).
- `roadmap.md` 2.4: status block; the "on the 2.3 window" wording becomes "on its own message-only window, on the `hook-watchdog` thread" (D1); the acceptance reading (Q1). Also the issue row `:730` and follow-ups FU-1/FU-2.
- `audit.md` KS-08 status line `:31,:312`.
- `gaps.md:155` (closed), `:164` (counter added), `:232` (closed by D9).
- `build-and-run.md:270` (`status` now tests the hook), `:461` (troubleshooting row).
- `glossary.md` (Raw Input, watchdog, heartbeat).
- `overview.md`.
- README tree has no `logging/` entry (pre-existing gap, FU-3).

## Commits, in order

Rules for every commit:
- The gate passes on every commit except the declared red ones: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`.
- An item that a later commit first uses outside tests carries `#[cfg_attr(not(test), expect(dead_code, reason = "first used by <subject of that commit>"))]`. That commit deletes it: an unfulfilled expectation warns, and `-D warnings` then fails, so none can be forgotten.
- Subjects are `<Area>: <what>`. Trailer: `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

1. **`Tests: MovingQuery's fake windows are no multiple of 4 (2.3 P1)`**
   - Touches `main_thread.rs:1636-1690`.
   - Not a red-first commit: it fixes a spurious red. Gate green.
   - Re-check: `a_return_signal_reads_the_foreground_window_again` is green, and still red when `input_returned` is removed from `answer_input_signals` (2.3's M13 mutation, `main_thread.rs:732-734`).
2. **`Engine: the winner's doc names the foreground cache (2.3 P2)`**. Doc only (`winner.rs:56-57`).
3. **`Platform: not_a_timer states its invariant`**. `timers.rs:71`.
4. **`README: the foreground.rs line of the tree (2.3 P3)`**. `README.md:73`.
5. **`Tests: a refused re-entry of the main state is counted (red)`**
   - `hook::main_thread::tests::a_refused_reentry_is_counted_in_main_state_refused`: on its own thread, `before = COUNTERS.get(MainStateRefused)`, then `with_main(|_| with_main(|_| ()))`, then assert `after - before >= 1` (`>=`, because other tests may refuse too).
   - `logging::counters::tests::counter_names_are_snake_case` lists 8 names.
   - PREDICTED RED: `error[E0599]: no variant or associated item named MainStateRefused found for enum Counter`, and `no method named get found for struct Counters`.
6. **`Hook: count the refused re-entries of the main state`**
   - D8: `counters.rs`, the `with_main` bump at `main_thread.rs:201-205`, counter tests updated (`[_; 8]` arrays, summary strings).
   - Docs: `errors.md:220` counters row; README Log file sentence; CHANGELOG Added.
   - Makes 5 green.
7. **`Tests: the keyboard callback counts every call (red)`**
   - `hook::keyboard_hook_callback::tests::the_keyboard_callback_counts_every_call_also_while_remapping_is_off`, under `GLOBAL_HOOK` (so the delta is exact): `before = get_hook().health().callbacks()`; F13 down/up with remapping on; one event carrying `injected_sentinel()`; remapping off; F13 down/up; then assert `after - before == 5`.
   - PREDICTED RED: `error[E0599]: no method named health found for reference &'static KeyboardHook`.
   - Mutation for the tester: move the bump after the switch gate, which gives `left: 3, right: 5`.
8. **`Hook: a heartbeat of the keyboard callback`**
   - D2: `health.rs` (`HealthState { callbacks }`, `callback_seen`, `callbacks`; `callbacks` carries the dead-code expect until 12); `keyboard_hook.rs` field and accessor; the callback bump; `hook/mod.rs`; the scan list entry; callback module docs.
   - Unit test `health::tests::the_heartbeat_counts_each_call`.
9. **`Tests: a reinstalled hook resets the engine (red)`**. Scenarios in `engine::scenarios`:
   - `a_swallowed_key_released_while_the_hook_was_gone_fires_at_its_next_press_after_the_reinstall`: `(keys: "F13", action: SwitchLanguage)`; down F13; `up_unseen` F13; `hook_reinstalled`; down F13; `expect_fired([SwitchLanguage])`.
   - `a_passed_key_held_through_a_reinstall_keeps_its_repeats_and_key_up_passing`: `LShift` down passes; `hook_reinstalled`; repeat and up pass; `check_no_lone_key_up`.
   - `a_hold_armed_before_the_hook_was_gone_fires_nothing_after_the_reinstall`: F13 `Hold(200)` down at 0; `hook_reinstalled` at +50; `advance_ms(300)`; nothing fired.
   - `a_key_pressed_while_the_hook_was_gone_passes_its_key_up_after_the_reinstall`: `down_unseen` A; `hook_reinstalled`; up A passes.
   - PREDICTED RED: `error[E0599]: no method named hook_reinstalled found for struct Scenario`.
   - Behavioural red: with `HookReinstalled => {}` (no key handling), the first scenario fails with `down F13 at +…: expected [SwitchLanguage] fired, got []`. This is the tester's mutation.
10. **`Engine: reset after a hook reinstall`**. D13: `engine/mod.rs`, `keys.rs` doc, `testing.rs` `hook_reinstalled`. Makes 9 green.
11. **`Tests: the main thread reinstalls only a silent, installed hook (red)`**
    - `hook::keyboard_hook::tests::a_reinstall_of_a_hook_that_was_never_installed_installs_nothing`: `Ok(Reinstalled::NotInstalled)` and `!is_installed()`.
    - `hook::main_thread::tests`, with a leaked local hook, a loop on its own thread and requests from the test thread:
      - `a_reinstall_request_finds_a_hook_that_answered_and_changes_nothing`: the heartbeat is bumped once after `seen = 0`, then `Alive`;
      - `a_reinstall_request_for_a_hook_that_is_not_installed_installs_nothing`: `NotInstalled` and `!is_installed()`;
      - `a_reinstall_request_to_an_ended_main_thread_fails_at_once`: `Err(ShuttingDown)` in under 100 ms.
    - Direct calls of `reinstall_if_silent` on a `MainThread::new()` thread, with a fake `replace`:
      - `a_reinstall_resets_the_engine_and_keeps_the_macros`: `replace` returns `Ok(Replaced { old: StillInstalled })`. Expect `Reinstalled`; the pending Hold no longer live; a swallowed F13 released unseen fires at its next press; a macro on a `FakeRunner` executor attached to the local hook still runs (`active_runs() == 1`); log lines `reinstalled it (the old hook was still installed; longest_callback_us=` and `state reset: the keyboard hook was reinstalled` (`logging::testing::capture`).
      - `a_failed_reinstall_keeps_the_old_hook_and_resets_nothing`: `replace` returns `Err(E_ACCESSDENIED)`. Expect `Failed`, the Hold still live, and the `[ERROR] could not reinstall the keyboard hook` line.
      - `a_reinstall_that_finds_no_hook_resets_nothing`: `replace` returns `Ok(Reinstalled::NotInstalled)`. Expect `NotInstalled`, a pending F13 `Hold(10000)` still live, and no log line.
    - PREDICTED RED: `error[E0599]: no method named reinstall_keyboard_hook found for struct KeyboardHook`; `no method named request_reinstall found for struct MainThreadHandle`; `error[E0425]: cannot find function reinstall_if_silent in this scope`; `error[E0433]: failed to resolve: use of undeclared type ReinstallRequest`.
12. **`Hook: reinstall the keyboard hook on the main thread`**
    - D4: `health.rs` (`ReinstallRequest`, `ReinstallOutcome`); `keyboard_hook.rs` (`reinstall_keyboard_hook`, `Reinstalled`, `OldHook`, `is_installed` doc); `main_thread.rs` (variant, `request_reinstall`, `serve` arm, `reinstall_if_silent`, `dispatch_sent_messages`, `hook_reinstalled`, module docs).
    - The expect from commit 8 is removed. `request_reinstall` and `Request::ReinstallHook` carry the expect until 16.
    - Docs: `errors.md` rows for "reinstalled it", "state reset: the keyboard hook was reinstalled", "answered before its reinstall", "could not reinstall".
    - Makes 11 green.
13. **`Tests: the watchdog's decision (red)`**
    - `src/hook/watchdog/mod.rs` with `mod decision;` only; `hook/mod.rs` `pub(crate) mod watchdog;`; `decision.rs` holds only `#[cfg(test)] mod tests` with a fake-clock feeder (fake counter, 10 ms steps, scripted raws, callbacks and answers; it ticks whenever the last `TickAt` is due).
    - The scenarios are in the table below.
    - PREDICTED RED: `error[E0412]: cannot find type Watchdog in this scope` / `E0425 cannot find value SILENCE`.
14. **`Hook: the watchdog's decision`**
    - D3: `decision.rs`, with `#![cfg_attr(not(test), expect(dead_code, reason = "first used by Hook: the hook watchdog thread"))]`.
    - Makes 13 green.
15. **`Tests: the hook watchdog thread (red)`**. In `hook::watchdog::tests`:
    - `only_keyboard_events_the_hook_can_see_count`: a `qualifies` table: `(0x1E, VK_A)` → true; `(0xFF, VK_A)` → false; `(0x1E, 0)` → false; `(0x1E, 255)` → false; `(0, VK_VOLUME_UP)` → true.
    - `the_watchdog_thread_starts_and_stops_without_raw_input`: `start_synthetic` on a leaked `HealthState`; `stop()` returns true within 1 s.
    - `synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request`: real clock; a raw every 20 ms for 3 s; the recording requester never answers. Exactly one request, `seen == 0`, `silent_for >= 1 s`, arriving 1.0-2.0 s after the first raw.
    - `synthetic_raw_events_with_a_live_heartbeat_send_no_request`: the test thread bumps the heartbeat right before each post, 3 s, no request.
    - `the_watchdog_injects_no_input` (scan): the non-test lines of `src/hook/watchdog/mod.rs`, `decision.rs`, `src/hook/health.rs`, and of `fn reinstall_keyboard_hook(`, `fn reinstall_if_silent(` and `fn dispatch_sent_messages(`, contain none of `SendInput`, `keybd_event`, `mouse_event`, `KEYBDINPUT`, `INPUT_KEYBOARD`. Each named file and function must be found, so the scan is never vacuous.
    - `no_key_data_leaves_the_raw_input_read` (scan): `VKey` and `MakeCode` occur only inside `fn raw_key_event(` and `fn qualifies(`; no `log::` line in the file names them.
    - `counter_names_are_snake_case` lists 9 names.
    - PREDICTED RED: `error[E0425]: cannot find function qualifies in this scope`; `E0433: use of undeclared type HookWatchdog`.
16. **`Hook: the hook watchdog thread`**
    - D5/D6: `watchdog/mod.rs`; `Counter::RawInputUnread`; `app.rs` start/stop; `Cargo.toml` feature. The expects of 12 and 14 are removed.
    - Docs: README (tree, Implementation Features bullet, Technologies, Known Limitations R-1, R-5, R-6, R-7); CHANGELOG Added (watchdog, roadmap 2.4, KS-08) and Fixed (KS-08 detection half); `errors.md` rows (watchdog off, raw input arrives, asking again, did not stop) and the counters row.
    - Makes 15 green.
17. **`Tests: swch status shows the hook health (red)`**
    - `hook::health::tests`:
      - `the_hook_health_reads_ok_not_watched_or_failed_with_its_reinstalls`: a Display table;
      - `a_reinstall_is_recorded_with_its_time`: `record_reinstall(UNIX_EPOCH + 1_760_866_867_123 ms)` gives `hook ok, reinstalls: 1, last 2025-10-19T09:41:07.123Z`;
      - `a_failed_attempt_reads_reinstall_failed_until_the_next_reinstall`.
    - `hook::ipc_server::tests::dispatch_status_appends_the_hook_health`.
    - `tests/ipc_roundtrip.rs`: `status_off_on_status_round_trip_without_a_hook` expects `running (2 bindings live; hook ok, reinstalls: 0)`, and the new `status_reports_the_hook_health_over_the_pipe` (reinstalls 1 with a time; the message starts `running (2 bindings live`).
    - `core::app::tests::daemon_handle_switches_remapping_through_the_main_thread` expects `hook: HookHealth { watched: false, reinstalls: 0, last_reinstall: None, last_attempt_failed: false }`.
    - PREDICTED RED: `error[E0432]: unresolved import key_switch_rs::hook::health::HookHealth` and `error[E0560]: struct DaemonStatus has no field named hook`.
18. **`IPC: swch status shows the hook health`**
    - D7: `health.rs`, plus `reinstall_if_silent` recording and the watchdog's `Watched` guard; `ipc_server.rs`; `app.rs`; `logging::utc_timestamp`; `sink.rs`.
    - Docs: README `:776`, `:789`; CHANGELOG Added (status) and Changed (reply text).
    - Makes 17 green.
19. **`Tests: a panic is logged within a bounded wait (red)`**. In `logging::panic::tests`:
    - `a_panic_line_names_the_thread_the_place_and_the_message`: `&str`, `String` and other payloads.
    - `a_nested_panic_on_one_thread_shares_the_budget`: `budget_end`.
    - `a_stuck_writer_holds_a_panic_report_at_most_its_budget`: an unread `sync_channel(1)` already full. `report_on` returns false in ≤ 250 ms + 100 ms.
    - `a_panic_report_reaches_the_log_file_before_it_returns`: `writer::spawn` with a capture console, `attach_on` a `TempDir`. `report_on` returns true and `daemon.log` holds `[ERROR] thread`.
    - `a_panicking_thread_of_a_child_process_is_logged_through_the_installed_hook`: runs `current_exe()` with `--exact logging::panic::tests::panic_child --test-threads=1` and `KS_PANIC_CHILD=<TempDir>`. The child installs the hook with `install_with` on its own writer and panics on a spawned thread (an unwinding panic: no abort, no Windows Error Reporting prompt). The parent waits up to 10 s, then asserts that `daemon.log` has one line holding `[ERROR] thread '<unnamed>' panicked at `, `panic.rs:` and `KS panic child`. The path separator in `Location::file()` is not asserted.
    - `panic_child`: returns at once unless `KS_PANIC_CHILD` is set.
    - PREDICTED RED: `error[E0433]: failed to resolve: could not find panic in logging`.
20. **`Logging: a panic hook writes the message through the log writer`**
    - D9: `panic.rs`, `mod.rs` (`init` installs it after `set_max_level`, `:116`), `writer.rs`.
    - Docs: `errors.md` panic row; README Log file sentence; CHANGELOG Added and Changed (Rust's default panic message replaced).
    - Makes 19 green.
21. **`Tests: the debug hold exists only in debug builds (red)`**
    - `hook::debug_hold::tests::the_hold_key_is_named_like_a_config_key`: `parse("ScrollLock") == Some(VK_SCROLL)`, `parse("nope") == None`, `parse("") == None`.
    - `hook::debug_hold::tests::only_a_key_down_of_the_armed_key_holds`: `should_hold` table; 0 never holds.
    - `hook::keyboard_hook_callback::tests::the_debug_hold_is_compiled_only_into_debug_builds` (scan): `hook/mod.rs` has `#[cfg(debug_assertions)]` on the line before `mod debug_hold;`. In `keyboard_hook_callback.rs` and `core/app.rs`, every non-test line naming `debug_hold::` has `#[cfg(debug_assertions)]` on the line before. At least one call site is found in each file.
    - PREDICTED RED: `error[E0433]: failed to resolve: could not find debug_hold in hook`, and the scan fails with `no call site of debug_hold:: in src/hook/keyboard_hook_callback.rs`.
22. **`Hook: a debug-build hold of the keyboard callback for the watchdog's manual check`**
    - D10.
    - Docs: `errors.md` rows (debug builds only); README note under the watchdog bullet; CHANGELOG Added (debug builds only).
    - Makes 21 green.
23. **`Bench: R14 key path with the watchdog running, R15 main-thread cost of a raw input dispatch`**
    - D11: `bench.rs`; CHANGELOG Added (bench rows).
    - No red: the bench is ignored. The developer runs it once and puts the numbers in `impl.md`. The tester runs the official set.

**Decision scenarios (commit 13), `hook::watchdog::decision::tests`:**

| Test | Script (fake clock) | Expected |
|---|---|---|
| `key_events_with_a_silent_callback_for_more_than_1_s_ask_for_exactly_one_reinstall` | raw every 30 ms, 0-5000 ms; no callback; no answer | exactly 1 `Reinstall`, at t ∈ [1000, 1500] ms, `seen` = start count, `silent_for ≥ 1 s`, `attempt 1` |
| `a_reinstall_that_brings_the_callbacks_back_asks_for_nothing_more` | as above; answer `Reinstalled` 50 ms after the request; then a callback 1 ms after each raw, to 10 s | exactly 1 |
| `a_callback_that_keeps_up_asks_for_nothing` | 10 s, raw every 30 ms, in three variants: callback 1 ms before each raw; callback 1 ms after; callbacks at 230-249 ms of each tick interval and their raws at 251-270 ms | 0 |
| `idle_asks_for_nothing_and_stops_ticking` | one raw + callback at 0; then 600 s of nothing | 0; `Idle` by 2250 ms; no `TickAt` after |
| `a_busy_main_thread_gives_no_reinstall_loop` | raw every 30 ms, no callback for 0-10 s, request unanswered; at 10 s answer `Alive` and the counter +50; callbacks keep up to 20 s | exactly 1 in 0-20 s |
| `reinstalls_that_bring_no_callback_back_are_spaced_out` | raw every 30 ms, 120 s, never a callback; every request answered `Reinstalled` 10 ms later | 6 requests, at ≈1, 3, 7, 15, 31, 63 s; gaps doubling, never under 1 s |
| `the_silence_resets_to_1_s_once_a_callback_is_seen` | as the row above until 3 requests; one callback; silence again | the next request ≥ 1 s and < 2 s after the callback's tick |
| `a_single_raw_event_without_a_callback_asks_for_nothing` | one raw at 0; ticks to 10 s | 0 |
| `raw_events_of_callbacks_seen_at_the_last_tick_are_explained` | 20 callbacks at 230-249 ms, their 20 raws at 251-270 ms, then nothing | 0 |
| `a_declined_request_needs_fresh_evidence` | silent; `declined(now)` at the verdict; raws go on | next `Reinstall` ≥ 1 s after the decline |
| `a_request_that_could_not_be_sent_is_asked_again_at_the_next_tick` | silent; `not_sent()` at the verdict | next `Reinstall` at the next tick |

## Acceptance mapping

| Roadmap / spec acceptance line | Covered by |
|---|---|
| Pure decision: silent callback > 1 s gives exactly one reinstall | `decision::tests::key_events_with_a_silent_callback_for_more_than_1_s_ask_for_exactly_one_reinstall`, `…a_reinstall_that_brings_the_callbacks_back_asks_for_nothing_more` |
| A callback that keeps up gives none | `a_callback_that_keeps_up_asks_for_nothing` (both orders and the tick boundary), `raw_events_of_callbacks_seen_at_the_last_tick_are_explained` |
| Idle gives none | `idle_asks_for_nothing_and_stops_ticking` |
| A busy main thread gives no reinstall loop | `a_busy_main_thread_gives_no_reinstall_loop`, `reinstalls_that_bring_no_callback_back_are_spaced_out`, `main_thread::tests::a_reinstall_request_finds_a_hook_that_answered_and_changes_nothing` |
| Debug hold 1.5 s gives `reinstalls: 1` within 2 s | manual M1 (Q1 states the reading). Below the hook: `synthetic_raw_events_with_a_silent_heartbeat_send_one_reinstall_request`, `a_reinstall_resets_the_engine_and_keeps_the_macros`, `the_debug_hold_is_compiled_only_into_debug_builds` |
| 10 min idle produce no injected input | scan `the_watchdog_injects_no_input`; `idle_asks_for_nothing_and_stops_ticking` (no tick, so no work); manual M2 |
| `swch status` shows HookHealth; `ipc_roundtrip` covers it | `tests/ipc_roundtrip.rs::status_reports_the_hook_health_over_the_pipe`, `::status_off_on_status_round_trip_without_a_hook`, `ipc_server::tests::dispatch_status_appends_the_hook_health`, `health::tests::*`; manual M7 |
| Latency bench: no unjustified regression with the watchdog active | R14 vs R12, R1-R13 vs master (D11); tester section 4b |
| Reinstall never double-installs and never leaves none | D4 argument; `a_reinstall_of_a_hook_that_was_never_installed_installs_nothing`, `a_reinstall_request_for_a_hook_that_is_not_installed_installs_nothing`, `a_failed_reinstall_keeps_the_old_hook_and_resets_nothing`, `a_reinstall_that_finds_no_hook_resets_nothing`; manual M1 (counters line shows `main_state_refused=0`) |
| No key data kept or logged | scan `no_key_data_leaves_the_raw_input_read`; the `Watchdog` fields (D3) |
| Reuses the 1.7 writer and counters; gauges where they help | `raw_input_unread`; `longest_callback_us` in the reinstall line (asserted in `a_reinstall_resets_the_engine_and_keeps_the_macros`) |
| Re-entry refusal counter | `a_refused_reentry_is_counted_in_main_state_refused` |
| Panic hook with a stated bound | `a_stuck_writer_holds_a_panic_report_at_most_its_budget`, `a_nested_panic_on_one_thread_shares_the_budget`, `a_panicking_thread_of_a_child_process_is_logged_through_the_installed_hook`, `a_panic_report_reaches_the_log_file_before_it_returns` |
| Debug flag absent from release | `the_debug_hold_is_compiled_only_into_debug_builds`; CI release builds (`ci.yml:40-41,57-78`) compile without it |
| 2.3 leftovers | commits 1-4 (P1 re-check as stated) |
| KS-08 audit scenario ("Every binding then stops working while `swch status` still reports `running`") | M1 (status shows the reinstall and remapping works after it) |

## Manual checks (human only)

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M1 | `cargo build --locked` (debug, not elevated). Read and record `reg query "HKCU\Control Panel\Desktop" /v LowLevelHooksTimeout` (read-only; absent is a valid answer). In a console: `$env:KS_DEBUG_HOOK_HOLD='ScrollLock'; .\target\debug\key-switch-rs.exe`. A second console polls `while(1){ "$(Get-Date -f HH:mm:ss.fff) $(.\target\debug\swch.exe status)"; Start-Sleep -m 250 }` | Open Notepad, hold `a` (auto-repeat) for 1 s, and while still holding it press ScrollLock once. Keep holding `a` 3 s, then release. Press CapsLock | Startup shows `[WARN] debug build: the keyboard callback holds 1.5 s at each ScrollLock key-down`. Within 2 s of ScrollLock the poll shows `hook ok, reinstalls: 1, last …`. `daemon.log` has `the keyboard hook got no key event for … ms while keys were typed; reinstalled it` and `state reset: the keyboard hook was reinstalled`. CapsLock switches the language afterwards (the template binding). No `main_state_refused` count. If instead `reinstalls: 0` and CapsLock still works, Windows kept the hook: report it as acceptance evidence against the premise, not as a defect. If remapping stops and `reinstalls` stays 0, that is a defect | none (needs a live hook) |
| M2 | Release daemon (`swch open`). Record `powercfg /requests` (read-only) | Leave the machine untouched 10 min | `powercfg /requests` lists nothing for key-switch-rs; the display turns off per the power plan if its timeout is ≤ 10 min; `daemon.log` has no watchdog line for those 10 min | none |
| M3 | Release daemon | Type 5 min across apps, including an elevated one (Task Manager) | `swch status` stays `hook ok, reinstalls: 0`; `daemon.log` has exactly one `hook watchdog: raw keyboard input arrives` line | none |
| M4 | Release daemon | Win+L, type the PIN or password, unlock, type | no reinstall line; `reinstalls` unchanged | none |
| M5 | Release daemon; `PromptOnSecureDesktop = 1` (read-only check, as 2.3 M5) | Open a UAC prompt and type Tab/Esc in it; open the Ctrl+Alt+Del screen and type there | no reinstall line | none |
| M6 (amended by ruling L2: it settles R-1 and Q6) | **A:** release daemon, then an AutoHotkey script started after it with `CapsLock::return`. **B:** quit the script; precondition M7 (`swch exit`, `swch status` says not running); `cargo build --locked` (debug, not elevated); note `reg query "HKCU\Control Panel\Desktop" /v LowLevelHooksTimeout` (read only; absent is an answer); a key the config does not bind, here F9 | **A:** hold CapsLock alone 3 s, then type normally 10 s. **B:** 1. console B: `while(1){ $k=[Console]::ReadKey($true); "$((Get-Date).ToUniversalTime().ToString('HH:mm:ss.fff')) $($k.Key)" }`. 2. console A: `$env:KS_DEBUG_HOOK_HOLD='F9'; Start-Sleep -s 3; .\target\debug\key-switch-rs.exe`, Enter, then hands off the keyboard (the sleep lets Enter's release pass before the daemon starts, so F9 is the first key event it counts). 3. after the `[WARN] debug build: ... F9 key-down` line, click console B with the mouse and wait 3 s. 4. press F9 once, wait 5 s. 5. read console B's F9 line (t_key, UTC) and, read only, `Select-String 'raw keyboard input arrives' "$env:LOCALAPPDATA\key-switch-rs\daemon.log" \| Select-Object -Last 1` (t_raw, UTC). 6. stop console B's loop, `swch exit`, `swch open` | **A:** at most one reinstall line (residual R-1), none after. No reinstall line: keys a later hook swallows give no raw input, so raw input comes after the hook chain (Q6 = after; B optional). One line within about 1.5 s of CapsLock: raw input exists for swallowed keys, before the chain or after it whatever its verdict; B decides. **B:** the hold keeps F9's callback 1.5 s, so console B gets F9 only after the hook timeout (at most 1 s). t_raw earlier than t_key by about that timeout (100 ms or more): the watchdog read F9's raw input while its hook call ran, so raw input comes **before** the hook call (Q6 = before): the first key of each burst pays R16 (about 50 µs at the median); FU-4 (roadmap 3.5) applies. t_raw within about 20 ms of t_key, either side: raw input comes **after** the hook chain (Q6 = after): a key's own callback never meets the wake; FU-4 lapses. Anything else, or no `raw keyboard input arrives` line: report both times and the timeout value | none (needs a live hook) |
| M7 | Release daemon running | `swch status`; `swch open` | `running (N bindings live; hook ok, reinstalls: 0)`; `swch: daemon is already running` | none |
| M8 | As M1 | `swch exit` | the process ends within 3 s; no `hook watchdog did not stop` line | none |

## Risks and edge cases (R1-R10 of the code-reviewer)

- **R1 Hook callback.**
  - The only additions are one RMW after `code < 0` and, in debug builds, one atomic load plus an optional sleep (debug, armed key only).
  - Every path still ends in one `LRESULT(1)` or `CallNextHookEx`, and the sentinel check stays before any state change (`:211`).
  - No allocation, lock, `log::` or Win32 call.
  - Watch for: the bump moved after the gate (the commit 7 mutation), or placed before `code < 0`.
- **R2 Sentinel.** No new `SendInput`. The scan of commit 15 keeps the watchdog and the reinstall functions free of injection.
- **R3 Shared state.**
  - `HealthState` is atomics inside `KeyboardHook`; no `Mutex`/`RefCell` is shared.
  - `reinstall_keyboard_hook` is `pub(crate)` and called only from `serve` on main; the watchdog thread holds no `&KeyboardHook`, only `&'static HealthState`.
  - Nothing runs under `with_main` except `hook_reinstalled`'s reset.
  - `dispatch_sent_messages` runs outside the borrow, as `pump` does.
- **R4 Orderings.**
  - Heartbeat: `fetch_add` `AcqRel` / `load` `Acquire`.
  - `watched`, `last_attempt_failed`: `Release` / `Acquire`. `reinstalls` and `last_reinstall_ms`: D7.
  - `hook_handle.swap(AcqRel)` as in `uninstall`. Debug `ARMED`: `Release` / `Acquire`.
  - No `Relaxed`, no `SeqCst`, no `static mut`.
- **R5 Builders.** No config type changes. `DaemonStatus` gains a public field: the test stand-ins construct it, and nothing outside the crate does.
- **R6 Display.** New arms: `ResetReason::HookReinstalled`; `Counter::{MainStateRefused, RawInputUnread}`; `ReinstallOutcome`; `OldHook`; `HookHealth`. There is no `_ =>` arm.
- **R7 Specificity.** Untouched; the reinstall does not call `update_bindings`.
- **R8 Key identity.** Raw `VKey` is used only for the 1-254 range test, never matched against bindings.
- **R9 Registry/process.** The code reads nothing new. M1 reads `LowLevelHooksTimeout` with `reg query` by hand.
- **R10 Message loop and shutdown.**
  - The new thread has its own pump and is stopped after `main.run` and before `drop(main)` and `uninstall()`.
  - Its window and registration drop on its own thread.
  - A request after `uninstall()` or `WM_ENDSESSION` finds a null handle and returns `NotInstalled`.
  - The stop bound is 1 s, and this thread never runs foreign code (unlike the executor), so it cannot hang.
  - A main-thread panic outside a callback unwinds through `App::run` with the hooks installed, and the executor guard's stop then waits up to 2 s (pre-existing; FU-1).

**Residuals (README Known Limitations, book sync):**
- **R-1.** Another program's LL hook, installed after the daemon, that swallows every key for over 1 s can draw one reinstall, which puts the daemon's hook first. This happens only if raw input precedes the hook chain, which is **(unverified)**, Q6.
- **R-2.** The keys typed between the hook's loss and its reinstall (about 1-1.5 s) reach Windows unremapped.
- **R-3.** If Windows does not remove a hook after one timeout, the hold check shows `reinstalls: 0` with working remapping (M1 says how to report it).
- **R-4.** Raw input delivered while another desktop has input is **(unverified)**, Q5. The decision-time desktop check limits it.
- **R-5.** A non-elevated daemon with an elevated window focused may reinstall at 1, 2, 4 … 64 s while the user types there.
- **R-6.** A Hold armed just before the loss fires on its timer while the hook is gone. The reset only stops later effects.
- **R-7.** Raw input in this process belongs to the watchdog: one registration per class (D5).

**Other edges:**
- Reload mid-episode: no effect (the heartbeat goes on).
- `swch off`: counted, so the watchdog keeps working.
- Foreground change: no effect.
- Sleep/resume: `Instant` may jump forward, but only evidence counts, and evidence needs raw events after the resume.
- IME/layouts: no effect (the VK range only).
- The watchdog thread starved for several seconds: the timer signals once, the backlog of raw events is pumped before the tick reads the counter, so `dc` covers their callbacks.
- CI: commit 15's real-clock tests allow up to 2 s for the request, and "exactly one" holds by construction, because the request stays outstanding.

## Questions for the orchestrator

1. **Reading of "reinstalls: 1 within 2 s".** Recommendation: measure from the ScrollLock key-down, with keys typed before, during and after the hold, as M1 prescribes. Then the bound is ≤ T + 1.5 s (the hold) + one tick ≈ T + 1.75 s, whatever the order of raw input and the hook, and with any `LowLevelHooksTimeout` ≤ 1000 ms. With no key typed in the 2 s before the hold, the bound is T + timeout + 1 s + one tick (≤ T + 2.25 s), still within 0.75 s of the hold's end. Non-blocking.
2. **Reset on reinstall (D13): `resync_from_os`, macros kept.** Recommendation: as decided. It is the remapping-off case (Windows saw everything) and keeps the owner decision on lost key-ups. Non-blocking.
3. **Debug hold through `cfg(debug_assertions)` and `KS_DEBUG_HOOK_HOLD`, not a Cargo feature (D10).** Recommendation: as decided. M1 runs on a debug (non-elevated) build, which is enough for Notepad. Non-blocking.
4. **Follow-ups to record in the roadmap at the book sync.**
   - FU-1: an uninstall guard in `App::run`, so that a main-thread panic removes the hooks before the bounded stops (R10 above).
   - FU-2: raw input as the "input left" signal (2.3 F5) now that the sink exists.
   - FU-3: the README tree lacks `src/logging/`.
   - Recommendation: add all three under "Follow-ups from Phase 2". Non-blocking.
5. **Unverified OS behaviour (no blocking: each has a fallback).**
   - Q5: raw input across desktops. Covered by the desktop check, M4 and M5.
   - Q6: whether raw input is generated before or after the LL hook chain, and whether a blocking hook suppresses it. The design is order-independent (D3 owed credit and one-tick age); only R-1 depends on it. Researcher query if wanted: "WH_KEYBOARD_LL hook returns 1: is WM_INPUT still generated for RIDEV_INPUTSINK, and in which order relative to the hook call, Windows 10/11". *(Ruling L2: Q6 goes to the owner as manual check M6, part A for a swallowed key and part B, the debug build's timestamps, for the order; the order decides whether a real key ever pays R16.)*
   - Q7: `PM_QS_SENDMESSAGE` and LL hook calls. Defense only.
   - Q8: whether std runs the panic hook a second time for the nounwind abort. The shared budget makes the bound hold either way.
   - Recommendation: proceed; M1, M4 and M5 give the owner the evidence.
6. **CLAUDE.md and agent files after the merge (orchestrator's chore lane).**
   - Layout: `hook/health.rs`, `hook/watchdog/`, `hook/debug_hold.rs`, `logging/panic.rs`.
   - Principle 1: the heartbeat is the prescribed hook-path addition.
   - Principle 5: `HealthState`.
   - Principle 6: a fourth helper thread, `hook-watchdog`, and that only main reinstalls.
   - Bench: R14, R15 and D11's rule.
   - Tester section 4b: R14 vs R12.
   - Recommendation: do it in the post-lane chore, as after 2.3. Non-blocking.

## Developer addendum

Branch `feat/hook-watchdog`, 28 commits on `fa20b30` (list and red outputs in `impl.md`). Every ruling of `rulings.md`, as carried out:

### Safety (S1-S4)
- **S1** `src/hook/installer.rs`: trait `Installer` (keyboard hook set/unhook, WinEvent hook set/unhook, raw keyboard register/unregister) and the daemon's `installer::SYSTEM`. The only Win32 calls of all three kinds sit in `mod win32` (`installer.rs:60-159`), declared `#[cfg(not(test))]`; under test `impl Installer for System` (`installer.rs:170`) is `unreachable!` at every call, before any Win32 call. `KeyboardHook::install`/`uninstall`/`reinstall_keyboard_hook` delegate to `install_with`/`uninstall_with`/`reinstall_keyboard_hook_with` (`keyboard_hook.rs:159,227,276`) on `SYSTEM`; the watchdog's `RawInputRegistration` registers through `SYSTEM` too. Tests use `installer::FakeInstaller` (`installer.rs:236`), which models numbered handles and records each call and the number of keyboard hooks installed after it. `the_system_installer_panics_at_every_call_under_test` pins the panics.
- **S2** Order: `29060a1` (red scan) and `2904d04` (seam) land before `a8aa0f7`, the first commit with a reinstall test.
- **S3** One test per rule, all on the fake: `a_reinstall_installs_the_new_hook_before_it_removes_the_old_one` (calls `[InstallKeyboard(4), RemoveKeyboard(1)]`, keyboard counts `[2, 1]`: never none; the hook keeps handle 4), `a_failed_reinstall_keeps_the_old_hook`, `a_reinstall_of_a_hook_that_is_not_installed_installs_nothing` (never installed, and removed), plus `a_reinstall_reports_an_old_hook_that_windows_removed_as_gone`. Mutations run by the developer, each caught: unhook first; the handle nulled before the install; no null guard (also caught by `the_daemons_reinstall_of_a_hook_never_installed_makes_no_call` through the system installer's panic, so no hook was installed); the old hook left installed.
- **S4** `hook::installer::tests::the_hook_installers_are_compiled_out_of_the_tests`: every `.rs` under `src` (recursively, `src/bin` and `main.rs` included), the code of each line before its tests module with comments and string literals stripped, may name `SetWindowsHookExW`, `UnhookWindowsHookEx`, `SetWinEventHook`, `UnhookWinEvent` or `RegisterRawInputDevices` only inside the `#[cfg(not(test))] mod win32 {` block, which must call each of them; no file under `tests/` may name one of them or `core::app` (integration tests link the non-test library, where `App::run` is the one public way to the hooks).
  - Fix round 1 (triage R-I1, `ee9ef4d` red, `0e7165b` green): the scan reads every line of every file, test modules included, plus `build.rs`, and matches `SetWindowsHook` as a prefix (`SetWindowsHookExA` and the obsolete `SetWindowsHookW`/`SetWindowsHookA` count). Tests `an_installer_call_in_a_tests_module_is_found`, `every_spelling_of_the_hook_installer_is_found`, `the_build_script_is_scanned`; see `fix_r1.md`.

### Important
- **I1** `Watchdog::next_tick` (`decision.rs`) is the one source of the pending tick. `Ticker::pass` (`watchdog/mod.rs:208`) runs after every wake: it takes the answer, ticks if due by the clock (an early or stale signal changes nothing), and re-arms the timer from `next_tick` through `plan()` (`arm`, `:278`). A failed `SetWaitableTimer` is counted (`watchdog_timer_not_set`), sets `HealthState::timer_failing` (status `hook watchdog timer failing` until a set succeeds), and the loop's wait then times out at the tick (`Ticker::wake_by`, `:197`), so the retry comes at the tick even with no raw event. Red-first tests on a fake timer and a fake clock, with ticks reachable only through the timer: `an_early_timer_signal_does_not_lose_the_tick` (every signal 10 ms early) and `a_timer_that_refuses_once_is_set_again_at_the_next_pass` (the first set after the burst refused); both end in one request. Mutations caught: arming only after a tick (both tests), no timeout fallback (the refusal test).
- **I3** `App::run` (`app.rs:220`) starts the watchdog after `MainThread::new` and the executor guard, before `ControlWindow::create` and `install`, and stops it (`stop_watchdog`, `app.rs:321`) after `uninstall` and `drop(control_window)`, before the executor's stop; the order is stated next to the executor's start. On early returns the guard drops after the control window. The stated rule for raw input before `install`: a raw key event counts only while the keyboard hook is installed (`HealthState::hook_installed`, set in `install_with` once the keyboard handle is stored, cleared in `uninstall_with`; `Ticker::raw_event`, `:176`). Tests: `raw_input_before_the_hook_is_installed_is_no_evidence`, and `core::app::tests::the_hook_watchdog_starts_before_the_hooks_and_stops_after_them` (an order scan of `App::run`).

### Minor
- **M1** `Watchdog::keeps_ticking` (`decision.rs:188`): a request outstanding, a raw event within `ACTIVE_FOR`, or at least 2 pieces of evidence younger than their window plus a tick. One raw event stops ticking at 2 s (`a_single_raw_event_without_a_callback_asks_for_nothing` asserts `Idle` and no tick after); `evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops` pins the window clause.
- **M2** `Counter::WatchdogDesktopCheckFailed` (`watchdog_desktop_check_failed`, with a Display arm); a failed check still declines, as the ruling words it (`a_failed_desktop_check_declines_and_is_counted`).
- **M3** `HealthState::hook_answered` clears a failed attempt at a tick that sees the heartbeat move; `record_reinstall` clears it too (`a_failed_attempt_clears_once_the_hook_answers`, `a_failed_reinstall_is_no_longer_reported_once_the_hook_answers`).
- **M4** The refusal test holds `GLOBAL_HOOK` and takes a saturating difference; the watchdog takes a counters reference (`&COUNTERS` in the daemon, leaked counters in every test); every writer the panic tests start counts in leaked counters.
- **M5** Row `evidence_is_checked_before_this_intervals_raw_events_are_added` (raw events at 1248 and 1249 ms, callbacks at 1251 and 1252 ms, after a second of quiet): 0 requests; red with the order swapped.
- **M6** The per-event system cost is stated in `hook::watchdog`'s module docs and the README's Hook watchdog bullet: one `WM_INPUT` per key event to the watchdog thread, never to main, read there with `GetRawInputData`. R14 adds the watchdog thread's per-key work next to the callback; see the open point below. The key path end to end, from system input to the callback, cannot be measured without a hook.
- **M7** The precondition is in M1 of `impl.md`'s manual checks: `swch exit` the release daemon first; M2-M8 start it again.

### Questions
- **Q1** is the reading of manual check M1. **Q2** is implemented (`ResetReason::HookReinstalled` takes the keys from the OS and keeps the macros; `a_reinstall_resets_the_engine_and_keeps_the_macros`). **Q3** is implemented (the `#[cfg(debug_assertions)]` module and call sites; `the_debug_hold_is_compiled_only_into_debug_builds`; the release daemon and the release test harness build). **Q4**: FU-1, FU-2 and FU-3 are in `impl.md` under "Follow-ups from Phase 2". **Q5-Q8**: the manual checks in `impl.md`.

### Deviations from the cut
1. New file `src/hook/installer.rs` and the `*_with` methods (S1); the cut's touch set had neither.
2. `HookHealth` has a fifth field, `timer_failing`, and a fourth state word, `watchdog timer failing` (I1).
3. Decision: the first raw event after idle also sets `quiet_since`, so that the anti-loop consequence "keys arrived for all of `S` without a callback" holds after idle too. Without it a main thread busy for 300 ms when typing resumes would draw a request after 500 ms (`a_key_after_idle_waits_a_full_second_of_silence` is red without it). `raw_event` returns nothing: the thread arms from `next_tick`. `answered` and `declined` also drop the raw events before the answer (rule 4; `raw_events_before_an_answer_are_no_evidence_after_it`).
4. The watchdog loop is `Ticker` + `TickTimer` + `plan` + `wake_by` (I1); the wait times out at the tick when the timer cannot be set.
5. `HealthState::hook_installed` (the I3 rule).
6. Counters `watchdog_timer_not_set` (I1) and `watchdog_desktop_check_failed` (M2) beside `raw_input_unread`: `SLOTS` is 11, not 9.
7. The `asking again` warning is logged only when the request is sent.
8. `Watched` drops first on the thread (before the timer, the registration and the window): the status reads `not watched` as soon as the loop ends.
9. The fake installer's call names (`InstallKeyboard`, `RemoveKeyboard`, ...) were renamed in the green commit of the reinstall: the red commit's names spelled the Win32 calls, which the installer scan counts.
10. Bench row `R14/settled` (evidence, no bound) beside R14 and R15.
11. Decision row `reinstalls_that_bring_no_callback_back_are_spaced_out` runs 250 s and checks 8 requests, the 64 s cap included (the cut: 120 s and 6 requests).
12. `OldHook::Gone` reads `Windows had removed the old hook: <error>`.
13. The panic hook names the thread with `std::thread::current()`, which panics, and so aborts, only after std's own thread cleanup, when no user code runs.

### Open point that needs a ruling (latency owner decision)
D11's R14 bound (`p50 <= R12 p50 + 300 ns`) is not met as R14 is built. Developer runs on a quiet machine (`D:/tmp/ks-orch/p2-2.4-watchdog/bench-dev.md`, HEAD runs 4 and 5, master run 2): R12 p50 4.7-4.8 us (master 4.8 us, so the heartbeat RMW does not show), **R14 p50 22.2-22.8 us**, R15 3.9 us; R14/settled, with the watchdog done with each event before the call, stays within 0.5 us of R12. So a callback that runs while the watchdog thread handles a raw event waits about 18 us for it, plausibly on win32k serializing the two threads' user32 calls. Whether that overlap happens in the daemon depends on Q6: if Windows generates a key's raw input only after the low-level hook chain returns (keys a low-level hook swallows are reported to reach no raw input reader), the watchdog's work for a key never overlaps that key's callback, but it can still overlap the foreground program's read of the key. Options: (a) accept on that argument; (b) take raw input in batches at the ticks (while ticking, wait without `QS_RAWINPUT` and drain the `WM_INPUT` messages right before each tick), so that the watchdog does no work per key; (c) raw input on the main thread after all (R15: about 3.9 us per key there, in sequence with the callback). Not decided by the developer.

Ruled 2026-10-19: option (b), ruling L1 (`rulings.md` round 2), with the round-3 rulings on the delta below. Carried out in fix round 1, next subsection.

### Fix round 1: the L1 delta with the round-3 rulings (developer)

Follow-up commits on `0e7165b`, in this order (red outputs in `fix_r1.md`):
`65c4d84` F1, `4c711ed` F2 (red), `0478a2c` F3, `f58c95c` F4 (red), `f222c5c` F5, `e712a2b` F6 (red), `76551be` F7, `ea8f19d` G1 (red, V3-I1 decision), `30ac79d` G2 (V3-I1 decision), `c605d0f` F8c (red, V3-I1 drain, V3-M3, OQ3), `0ac825a` F8d, `384ddf5` F8a (red), `d668a13` F8b (red), `a0aaf7c` F9, `e69ce7f` F10, `e75def6` (the README and module docs quote R16 as measured), `a90b38b` (OQ2: a filtered peek leaves a posted `WM_QUIT`), `959fce1` (the SAFETY comments of the two queue characterisations).

- **L1** as L1.2-L1.5: the ticking wait leaves `QS_RAWINPUT` out (`wake_mask`), the pump takes every message but `WM_INPUT` (`pump_ranges`, no window filter), the drain reads the queue (`GetRawInputBuffer` on 64-bit, the standard read after a failed read and on 32-bit), the pass hands the count to `Watchdog::raw_batch` before the answer, the stop line `[INFO] hook watchdog stopped after N wakes for M raw key events` reports the thread's counts. `raw_event` is `#[cfg(test)]` (= `raw_batch(1, ..)`).
- **V3-I1 (no false reinstall from a backlog)**:
  - The drain's bound is a time budget, `DRAIN_BUDGET` = 5 ms (`drain`), not a record cap: the cut's `MAX_BUFFERED_READS`/`MAX_STANDARD_READS` are not built. A drain that spends it returns `Drained { pending: true }`, and `next_wait` gives the next wait a timeout of 0, so the loop drains again before it waits (`the_next_wait_never_waits_while_raw_input_is_left`, `a_drain_that_spends_its_budget_leaves_the_rest_for_the_next`).
  - The decision (`decision.rs`, `tick(now, callbacks, backlog)`) matches totals: the callbacks no raw event has matched yet are `credit`. At a tick, the credit explains the interval's raw events first; at a tick that sees the callback alive, the evidence goes (the baseline) and the callbacks beyond the interval's unexplained raw events become credit; the rest of an older credit is kept only while the drain before the tick left raw input queued (`backlog`), and lapses at the first tick whose drain emptied the queue. The rule for events one side legitimately does not see: raw events the hook does not see (another desktop, UIPI) are dropped at every live tick and never add up across a live hook's ticks; callbacks raw input does not see (keys the daemon swallows) are credit only until a drain empties the queue, so they never hide a hook that dies later.
  - Red first: `a_burst_read_over_several_ticks_is_matched_by_the_callbacks_counted_before_it` (the critic's trace: 10 000 callbacks at one tick, raw read over three ticks, then silence; red `left: [1250]`), `a_hook_that_dies_during_a_burst_asks_for_exactly_one_reinstall` (red `left: 0`); pinned green for the other half: `a_hook_that_dies_after_a_burst_asks_for_exactly_one_reinstall`, `long_drift_with_a_live_hook_asks_for_nothing` (raw sees more, the hook sees more, the hook sees one key in eight; 10 minutes each), `a_hook_that_dies_after_long_drift_is_found_within_its_window`. Loop rows on the batched `Drive` with a per-pass budget: `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request`, `a_hook_that_dies_during_a_flood_is_reinstalled_once`. Mutations caught: no lapse, the whole interval as credit, the backlog ignored (decision and loop), the evidence kept at a live tick.
- **V3-I2**: bench row R14c (`r14c_paced_control`): R14's 1 ms pacing and the same `SYNTHETIC_RAW` post before each timed call, to a parked thread that never reads its queue, no watchdog. The bound is R14 p50 <= R14c p50 + 300 ns, p99 <= R14c p99 + 2 us; R12 is reported for context. *(Ruling L2: the paced rows gate on p50 only; p99 is reported as the largest of 3 against R14c's largest of 3. See fix round 2 below.)*
- **V3-M1**: M9 has one pass rule, **PASS iff K >= 400 and W <= K / 2**, stated with its margin in `fix_r1.md` (Manual checks). The ruling's example `W <= K / 10 + 20` does not fit M9's own script: the 20 s hold alone ticks about 22.25 s, which is about 90 wakes (4 a second plus the first), above 600 / 10 + 20 = 80, so a correct build would fail it. Expected for a correct build: about 90 wakes for the hold and about 75 for the setup's own bursts (`swch open`'s Enter, opening Notepad, `swch status`, `swch exit`, a window switch), W of about 165 for K of about 640 at Windows' default repeat rate, half the bound; one wake per key event gives W >= K, twice the bound. K < 400 (a slow repeat rate) makes the run void: hold `a` longer. The amended M2 has its own rule for idle, W <= 100 (two typed commands and the start, about 45; a watchdog that ticked through 10 idle minutes would show about 2 400).
- **V3-M2**: R16 (`r16_first_key_of_a_burst(1_000)`) sets `hook_installed` as the daemon does after `install`, so its first pass counts the event and sets the tick timer; n = 1000. The README and the module docs give R16's measured number from the developer's run (about 50 us at the median, `e75def6`), never the planned one.
- **Open point for a ruling (bench, not code)**: in the developer's run R16 is p50 53.5 us, p99 168.4 us against R12 4.7 / 5.9 us, above the sanity bound of L1.6 (p50 <= R12 + 25 us, p99 <= R12 p99 + 100 us). The bound came from the pre-L1 R14, whose posts came back to back to a busy thread. R16's construction on the pre-L1 tree `0e7165b` (a wake from idle, the timed call at once) measures p50 35.6 us (R12 4.8 us), so the first key after idle broke that bound before L1 too, and pre-L1 every key of human typing woke the idle thread that way. The bench thread at `THREAD_PRIORITY_HIGHEST`, as the daemon's main thread runs, changes nothing (R16 58.5 us); a call 200 us after the post still pays +12 us at the median. Numbers and the diagnostic rows: `fix_r1.md`, Bench. *(Ruled 2026-10-19: ruling L2, R16 becomes a declared evidence row; carried out in fix round 2 below.)*
- **V3-M3 / critic OQ1**: in `drain`, a read that finds nothing while `GetQueueStatus(QS_RAWINPUT)` still reports raw input falls back to one standard read; if the bit is still set after it, the drain counts one `raw_input_unread` and returns `stuck`, and `next_wait` leaves raw input out of the next wait with a timeout of one `TICK`: at most four such passes a second, never a spin. Fake-source tests: `raw_input_that_no_buffered_read_returns_is_taken_by_one_standard_read`, `raw_input_that_no_read_returns_is_counted_and_the_wait_backs_off_for_a_tick`. The mutation table's reasons: "the drain only at due ticks" on the synthetic source shows as one wake per post (F8a `wakes`), not as a spin, because a peek clears `QS_POSTMESSAGE` (critique_v3 M3); the keyboard-only spin is covered by the guard's tests and by M2/M9 (a spin would show millions of wakes). `SyntheticReads` documents the difference.
- **Critic OQ2**: the pump peeks with no window filter (`pump`), so `WM_QUIT` is always seen; only the standard read filters by the sink window. Cut E7's "a filtered peek still returns `WM_QUIT`" does not hold for the stop's `PostThreadMessageW(WM_QUIT)`, a queued message: a peek filtered to another message leaves it, and only the quit state that `PostQuitMessage` sets passes any filter (`a_range_filtered_peek_leaves_a_posted_wm_quit_for_the_pump`). The pump's ranges hold `WM_QUIT`, so the design stands; the drains' `Quit` branches only catch a `PostQuitMessage` quit.
- **Critic OQ3**: `the_buffered_read_of_a_thread_without_raw_input_reads_nothing` accepts both documented outcomes of `GetRawInputBuffer` (0 records; `(UINT)-1`, then counted) and names them; on the developer's Windows 11 (build 26200) it returned 0.
- **Critic OQ4**: recorded as unverified (whether pending `HRAWINPUT` objects count against the USER object quota). The drain until empty bounds the backlog; M10 and M9 observe it.
- **Delta Q1-Q6**: as recommended: the once-per-run `[INFO]` stop line; R16's sanity bound (withdrawn by ruling L2, fix round 2); R14/settled removed; M9 settles E4 on the live daemon (a failure means a new delta for the kernel-wait fallback); triage m1 folded into F9 (`Counter::WatchdogTimerNotSet` docs); 32-bit builds use the standard read (`BUFFERED`).

Deviations from the delta:
1. `Ticker::pass(now, drained: Drained, context)`, not `pass(now, raw_keys, context)`: the drain's `pending` reaches the tick inside `pass`, so the fake-clock `Drive` covers that plumbing (the mutation "backlog not passed to the tick" is caught by the flood row).
2. The drain sits behind a `RawReads` seam (`Read`, `Drained { keys, pending, stuck, quit }`, `drain`, `KeyboardReads`, `SyntheticReads`) so that the time budget and the V3-M3 guard are tested on a fake; `next_wait` is a pure function for the wait's mask and timeout.
3. `SyntheticReads` reads one `SYNTHETIC_RAW` per read and reports no pending queue bit; `SYNTHETIC_RAW` is `pub(crate)` under `cfg(test)` for R14c.
4. A keyboard record too short for its key fields is counted unread (`a_keyboard_record_too_short_for_its_keys_is_unread`); the cut skipped it.
5. `a_single_batched_raw_event_stops_ticking_after_two_seconds` and `the_first_raw_key_event_after_idle_wakes_the_watchdog` assert the idle pass with nothing armed and no pass in [2250, 5000), not a `Cancel` call: the timer has signalled for the idle tick, so `plan` has nothing to cancel. The cut's `Cancel` expectation does not match `plan` (unchanged since round 1).
6. F1 asserts the masked wait lasted >= 80 ms, not >= 90 ms: one system-clock tick of slack on a 100 ms timeout; the `WAIT_TIMEOUT` result is the rule.
7. The stuck queue is counted in `raw_input_unread`, not in a new counter: it is raw input that could not be read; `SLOTS` stays 11.
8. The pump loops over its two ranges until a round takes nothing.
9. Commit order: the decision's V3-I1 pair (G1/G2) after F7, and the drain pair (F8c/F8d, which also holds the OQ3 smoke test of F8b) before F8a/F8b.
10. Bench helpers `bind_f13_to_do_nothing`, `spin_for`, `paced_calls` and `ParkedQueue` serve R14, R14c and R16; R12 is unchanged.

### Fix round 2: ruling L2, R-m1, R-m2 (developer)

Follow-up commits on `f51dc7d`, in this order (outputs in `fix_r2.md`): `62cf30a` (the `RawBuffer::take` seam), `3117b0a` (R-m2, red), `fef60f9` (R-m2), `1db14ca` (R-m1), `4d6b228` (L2 docs).

- **L2 (R16 a declared evidence row)**: R16 is measured and reported, not gated. L1.6's R16 row states the measured figure (about 50 µs at p50 above R12, once per burst, the worst order of Q6; the tester's runs and the developer's) and keeps the withdrawn sanity bound as "as first written"; L1.13's planned "18-25 µs", the touch set's planned README "20 µs" and L1.11's "sanity bound" now name the measured figure, and L1.14 Q2 says that L2 withdrew it. The README's Hook watchdog bullet (`README.md:939`), the watchdog's "# Cost" docs (`src/hook/watchdog/mod.rs:33-42`) and the bench's R16 docs (`src/bench.rs:75-82`) say the same, with no planned number.
- **L2 (p99 of the 1 ms-paced rows)**: L1.6's R14 row gates on p50 against R14c (V3-I2) and reports p99 as the largest of 3 runs against R14c's largest of 3; an R14c row is added; the V3-I2 line above carries the note.
- **L2 (Q6 to the owner)**: manual check M6 (Manual checks table) is amended to settle Q6. Part A (the AutoHotkey swallow, release daemon) settles it when no reinstall line comes (raw input after the hook chain); part B, on the debug build with `KS_DEBUG_HOOK_HOLD=F9`, compares the console's UTC time of F9 with `daemon.log`'s `[INFO] hook watchdog: raw keyboard input arrives`: raw input read about one hook timeout before the key reaches the console means before the hook call, within about 20 ms means after. No code was added for it: the first-arrival line and its millisecond timestamp exist in every build.
- **L2 (3.5 follow-up)**: `impl.md`, "Follow-ups from Phase 2", FU-4: if M6 shows raw input before the hook call, roadmap item 3.5 trims the user32 work of the watchdog's first pass after idle.
- **R-m1**: `book/src/config/errors.md:215`, the stop line's row: about 10 wakes per isolated shortcut and about 4 a second of longer typing, so N above M is normal for isolated bursts (100 Ctrl+C: M = 400, N about 1000); the per-key symptom is named for sustained typing only (a 20 s hold adds about 600 to M and about 90 to N). Doc only, no test.
- **R-m2**: `RawBuffer::take` (`src/hook/watchdog/mod.rs:864-882`) maps the buffered read's return value and clears the whole buffer after any read that may have written to it (records or a failed read; an empty read wrote nothing). Red first: `a_batchs_key_codes_do_not_outlive_its_count` ("bytes of a counted batch are left", left: 12). The module's "No key data" docs say that no key code outlives the count of its batch; the standard read's record stays a local of its call, as before L1.

Deviations from the rulings:
1. The seam commit `62cf30a` (behaviour-neutral) precedes R-m2's red test, so that the test is red on its assertion rather than on a missing function.
2. R-m2's clear also covers a failed buffered read: Learn does not say what a failed `GetRawInputBuffer` leaves in the buffer.

## Delta after ruling L1 (raw input read in batches at the ticks)

Written 2026-10-19 after `rulings.md` round 2 (ruling L1, option (b)). Everything above this heading stands except where this section replaces it; where they differ, this section wins. Tree: `feat/hook-watchdog` at `0e7165b` (fix round 1). This role has no shell for `git show`: the checkout's `.git/HEAD` is `ref: refs/heads/feat/hook-watchdog`, that ref is `0e7165b04943eb1bcb1326261869a50c34762d4f`, and `fix_r1.md:5` reports the tree clean, so the files were read from the checkout. Every `path:line` in this section is on `0e7165b` unless it names `windows-0.62.2` (the registry copy `index.crates.io-1949cf8c6b5b557f/windows-0.62.2`).

### L1.0 Verdict for the delta
CUT READY. L1 can be met with the batched-read pattern Microsoft documents. While the watchdog ticks, the wake mask of `MsgWaitForMultipleObjectsEx` leaves out `QS_RAWINPUT`. The pump never removes `WM_INPUT`. `GetRawInputBuffer` drains the queue at every pass. One OS behaviour is not stated in so many words on Learn: that a pending `WM_INPUT` outside the mask leaves an `MWMO_INPUTAVAILABLE` wait asleep (E4). Test F1 pins the same rule in-process for posted messages. Manual check M9 settles it for raw input on the live release daemon. L1.5 names the fallback if M9 fails. No owner question blocks code.

### L1.1 Spec re-located (0e7165b)

| Reference | 0e7165b file:line | Status | Note |
|---|---|---|---|
| loop `run_loop`: wait with `QS_ALLINPUT`; pump only on a message wake | `src/hook/watchdog/mod.rs:585-622` (mask `:603`, pump branch `:617`) | replaced by L1.5 | every `WM_INPUT` ends the wait: one wake per key event (R14 22.7 µs, `test_r1.md:92`) |
| `pump`: `PeekMessageW(0, 0)` takes every message, reads each `WM_INPUT` | `:636-662` | replaced | it removes `WM_INPUT`, so a later `GetRawInputBuffer` would not see it (E7) |
| `raw_key_event` (`GetRawInputData`) | `:671-698` | kept | used by the standard read only (L1.5) |
| `qualifies` | `:706-708` | unchanged | |
| `Ticker::raw_event` (the I3 gate `hook_installed`, the first-arrival line) | `:176-185` | folded into `Ticker::pass` | gate and line unchanged |
| `Ticker::pass` / `take_answer` / `arm` / `wake_by` | `:208-223` / `:225-241` / `:278-303` / `:197-202` | `pass` gains the drained count; the rest unchanged | |
| `Source`, `is_synthetic_raw`, `SYNTHETIC_RAW = WM_APP + 2` | `:308-344` | extended; `is_synthetic_raw` goes | |
| `HookWatchdog::spawn` / `stop_thread` / `done: Receiver<()>` | `:386-409` / `:426-442` / `:352` | the done channel carries `Option<LoopStats>` | |
| `run_watchdog` | `:532-580` | returns `Option<LoopStats>`; owns the `RawBuffer`; logs the stop line | |
| test `Drive` (loop on a fake timer and clock) | `:837-926` | rewritten to the batched model (L1.7) | today it passes at every raw event (`:910-913`) |
| `a_reply_of_the_main_thread_is_taken_at_the_next_pass` | `:977-987` | renamed, same bounds | |
| `raw_input_before_the_hook_is_installed_is_no_evidence` | `:1006-1023` | replaced (L1.7) | |
| `no_key_data_leaves_the_raw_input_read` (allowed: `raw_key_event`, `qualifies`) | `:1288-1315` | allowed list gains `count_raw_keys` | |
| decision `raw_event` (returns nothing, deviation 3) | `src/hook/watchdog/decision.rs:120-128` | becomes `raw_batch`; `raw_event` stays for tests | |
| decision `tick` / `keeps_ticking` / `answered` / `start_over` | `:141-183` / `:188-195` / `:202-211` / `:226-231` | unchanged | |
| bench R14, R14/settled, R15; `Unanswered`; row list; docs | `src/bench.rs:448-480`, `:482-587`, `:430-446`, `:984-998`, `:59-74` | R14 redefined, R14/settled removed, R16 added (L1.6) | R14 posts right before each timed call (`:468-474`) |
| `Counter::RawInputUnread`, `WatchdogTimerNotSet` docs | `src/logging/counters.rs:48-50`, `:51-54` | doc text | m1: "the next raw key event brings" is wrong already (`triage_r1.md:54`) and stays wrong under L1 |
| raw input registration | `src/hook/installer.rs:130-142` (`dwFlags: RIDEV_INPUTSINK`, `:134`) | unchanged | no `RIDEV_DEVNOTIFY`, which Microsoft's batched pattern forbids (E10) |
| README watchdog bullet, Technologies, watchdog limits | `README.md:939`, `:905`, `:979-992` | text | |
| CHANGELOG watchdog entry, bench entry | `CHANGELOG.md:179-215`, `:244-249` | text (unreleased entries, edited in place) | |
| errors.md watchdog rows, counters row | `book/src/config/errors.md:210-219`, `:230` | one row added, one reworded | |
| `windows` items | `GetRawInputBuffer(pdata: Option<*mut RAWINPUT>, pcbsize: *mut u32, cbsizeheader: u32) -> u32` `windows-0.62.2/src/Windows/Win32/UI/Input/mod.rs:31`; `DefRawInputProc` `:16`; `RAWHID` `:110`; `RAWINPUT` `:122`; `RAWINPUTHEADER` `:195`; `RAWKEYBOARD` `:203`; `RIM_TYPEHID`/`RIM_TYPEKEYBOARD`/`RIM_TYPEMOUSE` `:318-320`; `QS_ALLINPUT` = `0x4FF` `UI/WindowsAndMessaging/mod.rs:5534`; `QS_POSTMESSAGE` `:5544`; `QS_RAWINPUT` = `0x400` `:5545`; `QUEUE_STATUS_FLAGS` with `BitAnd` and `Not` `:5550-5584`; `MsgWaitForMultipleObjectsEx` `:1795`; `PeekMessageW` `:1842`; `MWMO_INPUTAVAILABLE` `:5201`; `WM_INPUT` = `0xFF` `:6990` | present | all under `Win32_UI_Input` / `Win32_UI_WindowsAndMessaging`, already in `Cargo.toml`; no feature or lock change |

### L1.2 The mechanism

The `hook-watchdog` thread has two states. It is **ticking** while `Ticker::is_ticking()`, that is while `Watchdog::next_tick()` is `Some`, and **idle** otherwise.

| | idle | ticking |
|---|---|---|
| wake mask (keyboard source) | `QS_ALLINPUT` (contains `QS_RAWINPUT`) | `QS_ALLINPUT & !QS_RAWINPUT` (= `0xFF`) |
| wake mask (synthetic source, tests and bench) | `QS_ALLINPUT` | `QS_ALLINPUT & !QS_POSTMESSAGE` |
| flags | `MWMO_INPUTAVAILABLE` | `MWMO_INPUTAVAILABLE` |
| what ends the wait | the first raw key event of a burst; `WM_QUIT`; a spurious return (E5) | the tick timer; the wait timeout when the timer could not be set (I1, `wake_by`); `WM_QUIT` (keyboard source only, see below); a spurious return |
| per key event | one wake (the first key of a burst) | **none**: Windows queues the `WM_INPUT` and the thread sleeps on |

After every return of the wait except `WAIT_FAILED`, all on `hook-watchdog`, in this order:
1. `WAIT_OBJECT_0` → `ticker.fired()` (unchanged).
2. **Pump**, on every wake, the timer wake included. It loops `PeekMessageW(msg, None, 0, B - 1, PM_REMOVE) || PeekMessageW(msg, None, B + 1, 0xFFFF, PM_REMOVE)` until both are empty, where `B = source.batched_message()` (`WM_INPUT` for the keyboard, `SYNTHETIC_RAW` for the synthetic source). The ranges come from `pump_ranges(B)`. `WM_QUIT` goes to step 6, and every other message goes to `DispatchMessageW`. The pump never removes `B`.
3. **Drain** with `source.drain(window, &mut buffer, counters, msg) -> Drained { keys, quit }`. `quit` → step 6.
4. **Pass**: `ticker.pass(Instant::now(), drained.keys, context)`. The order is: stats; `raw_batch` if the hook is installed; `take_answer`; the tick if due by the clock; `arm`.
5. Loop.
6. **End**: on `WM_QUIT` the loop drains once more and adds those keys to the stats only, not to the decision. Then it returns. `run_watchdog` logs `[INFO] hook watchdog stopped after {wakes} wakes for {raw_keys} raw key events` and returns `Some(stats)`, and the spawn closure sends it on `done`.

**The keyboard drain**, `drain_keyboard`:
- **Buffered read** in 64-bit builds (`const BUFFERED: bool = cfg!(target_pointer_width = "64")`). Up to `MAX_BUFFERED_READS = 64` calls per pass: `size = RAW_BUFFER_BYTES as u32; n = GetRawInputBuffer(Some(buffer.0.as_mut_ptr().cast::<RAWINPUT>()), &mut size, size_of::<RAWINPUTHEADER>() as u32)`. `size` is reset before each call, as Microsoft's sample does.
  - `n == 0`: the queue is empty, done.
  - `n == u32::MAX`: one `raw_input_unread`, then the standard read for the rest of this pass.
  - otherwise `count_raw_keys(&buffer.0, n)`: `keys +=` the count, plus one `raw_input_unread` per unread record.
- **Standard read**, used after a failed buffered read and as the only read in 32-bit builds. Up to `MAX_STANDARD_READS = 4096` calls per pass of `PeekMessageW(msg, Some(window), WM_INPUT, WM_INPUT, PM_REMOVE)`. `WM_QUIT` sets `quit`. Every other message goes to `raw_key_event(msg.lParam, counters)` (unchanged) and then `DispatchMessageW(msg)`, which reaches `DefWindowProcW` for the `RIM_INPUT` cleanup (E9).
- `RawBuffer`: `#[repr(C, align(8))] struct RawBuffer([u8; RAW_BUFFER_BYTES])`, with `RAW_BUFFER_BYTES = 64 * size_of::<RAWINPUT>()` (3 072 bytes on x64). It is plain bytes, always initialised, so no padding byte of a typed `RAWINPUT` is ever read. `run_watchdog` makes it once, on the thread's stack. Nothing is allocated per pass.
- `count_raw_keys(bytes: &[u8], count: u32) -> BlockCount { keys: u64, unread: u64 }` is safe code. It reads `RAWINPUTHEADER.dwType`/`dwSize` and `RAWKEYBOARD.MakeCode`/`VKey` with `u32::from_ne_bytes` and `u16::from_ne_bytes` on `bytes.get(..)`, at `core::mem::offset_of!` offsets (`offset_of!(RAWINPUT, data)` for the record body). The walk, record by record:
  - A record whose header does not fit, whose `dwSize < size_of::<RAWINPUTHEADER>()` (a zero size included) or whose `dwSize` runs past the buffer ends the walk. The rest of `count` is counted unread.
  - A `RIM_TYPEKEYBOARD` record with `dwSize >= offset_of!(RAWINPUT, data) + size_of::<RAWKEYBOARD>()` counts if `qualifies(MakeCode, VKey)`.
  - Any other record is skipped by its `dwSize`.
  - The next record starts at `align_up(offset + dwSize, RAWINPUT_ALIGN)`, with `RAWINPUT_ALIGN = size_of::<usize>()` (E11).
- No `DefRawInputProc` and no `DefWindowProcW` for records the buffered read took (E9).

**The synthetic drain** (`#[cfg(test)]`) loops `PeekMessageW(msg, None, SYNTHETIC_RAW, SYNTHETIC_RAW, PM_REMOVE)` and counts one key per message. `WM_QUIT` sets `quit`, because PeekMessage "always retrieves WM_QUIT messages, no matter which values you specify" (E7).

**Why the synthetic source mirrors the keyboard one.** A test can set no `QS_RAWINPUT` without a raw input registration, which ruling S1 forbids. `SYNTHETIC_RAW` thread messages set `QS_POSTMESSAGE` instead. The synthetic ticking mask leaves that bit out, the synthetic pump leaves those messages in the queue, and the drain reads them. The tests and the bench therefore run the same wait, pump, drain and pass sequence as the daemon, with one queue bit swapped. There is a cost in the tests and the bench: while a synthetic watchdog ticks, its `WM_QUIT` (also a posted message) is seen at the next tick's pump, so `stop()` takes up to one `TICK`, which is within `STOP_TIMEOUT`. The keyboard source keeps `QS_POSTMESSAGE` in both masks, so the daemon's stop is immediate in both states (I3, M8 unchanged).

**Work per key event after L1:**
- While ticking: nothing on any thread of ours. Windows still builds and queues one `WM_INPUT` per key event for the watchdog's queue, never for main's (M6). This system-side cost is unchanged by L1 and cannot be measured without a registration.
- Per tick (4 Hz while keys are typed): 2 empty `PeekMessageW`, one `GetRawInputBuffer` per 76 keyboard records (40 bytes each on x64), one final call that returns 0, the existing tick and `SetWaitableTimer`.
- The first key event after the thread went idle, that is at least `ACTIVE_FOR` without keys: one wake and one pass (R16, declared).
- Two cases wake an idle thread once per event and never start ticking: raw events the hook cannot see (VK 255, keyboard overrun) and raw events before `install` or after `uninstall` (the I3 gate). The second case lasts only for the startup and shutdown windows.

### L1.3 Microsoft Learn evidence

| # | Claim the design relies on | Learn text (verbatim) | Status |
|---|---|---|---|
| E1 | Raw input that is not read stays in the thread's queue as `WM_INPUT`. The wake mask only selects what ends the wait. | "When an application receives raw input, its message queue gets a WM_INPUT message and the queue status flag QS_RAWINPUT is set." ([GetRawInputBuffer](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getrawinputbuffer), Remarks; [Raw Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input)). The batched sample: "This keeps all raw input events in the queue where GetRawInputBuffer can drain them all at once on each timer tick." ([Using Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/using-raw-input), "Performing a Buffered Read of Raw Input"). "dwWakeMask: The input types for which an input event object handle will be added to the array of object handles." ([MsgWaitForMultipleObjectsEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex)) | documented: **kept** |
| E2 | Events are not merged. | `GetRawInputBuffer` returns "the number of RAWINPUT structures written to pData". No Learn page says that `WM_INPUT` is coalesced. | **(unverified)**. A merge would only lower the raw count, which means less evidence and no false reinstall. M9 shows the count. |
| E3 | Queue limits. | "There is a limit of 10,000 posted messages per message queue. … GetLastError returns ERROR_NOT_ENOUGH_QUOTA when the limit is hit. The minimum acceptable value is 4000." ([PostMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postmessagew)). No Learn page gives a limit for the raw input queue. | **(unverified)** whether `WM_INPUT` counts against the 10 000. The backlog is at most one tick: human typing and auto-repeat are ≤ ~50 events/s, so ≤ 13 per tick, and an injected `type_text` burst is a few hundred, more than 10× under the 4 000 floor. A dropped event lowers the evidence (the safe direction). No manual check is needed for acceptance; M9 shows the counts. |
| E4 | A pending `WM_INPUT` does not end a wait whose mask lacks `QS_RAWINPUT`, also with `MWMO_INPUTAVAILABLE`. | `MWMO_INPUTAVAILABLE`: "The function returns if input exists for the queue, even if the input has been seen (but not removed) using a call to another function, such as PeekMessage." Return `WAIT_OBJECT_0 + nCount`: "New input of the type specified in the dwWakeMask parameter is available in the thread's input queue." Remarks: "MsgWaitForMultipleObjectsEx does not return if there is unread input of the specified type in the message queue after the thread has called a function to check the queue, unless you use the MWMO_INPUTAVAILABLE flag." | Every condition is phrased as input "of the specified type". That a raw input bit outside the mask is ignored is **(unverified)** in words. F1 pins it for `QS_POSTMESSAGE` on a test thread, and M9 settles it for `QS_RAWINPUT`. If it fails, every pass still drains, so the thread falls back to one wake per key event (the pre-L1 cost), never to a spin. Fallback design: L1.5. |
| E5 | Spurious returns are harmless. | "This value is also returned upon the occurrence of a system event that requires the thread's action, such as foreground activation. Therefore, MsgWaitForMultipleObjectsEx can return even though no appropriate input is available and even if dwWakeMask is set to 0." | documented. Every return is a pass, the clock gates the tick (I1), and the drain takes what is there. |
| E6 | `GetRawInputBuffer` reads and removes the queued `WM_INPUT`, in parts if needed, and clears `QS_RAWINPUT`. | "Performs a buffered read of the raw input messages data found in the calling thread's message queue." "Using GetRawInputBuffer, the raw input data is read in the array of variable size RAWINPUT structures and corresponding WM_INPUT messages are removed from the calling thread's message queue. You can call this method several times with buffer that cannot fit all message's data until all raw input messages have been read." "If all raw input messages have been successfully read from message queue then QS_RAWINPUT flag is cleared from the calling thread's message queue status." Return: "If an error occurs, the return value is (UINT)-1." | documented. It runs on the calling thread's queue, which is `hook-watchdog`. |
| E7 | The pump must not take `WM_INPUT`, and a filtered peek still returns `WM_QUIT`. | "GetMessage removes the current WM_INPUT from the raw input queue before returning. As a result, GetRawInputBuffer will not see the current event" (Raw Input Overview, Important). The batched sample uses "only PeekMessage with explicit message range filters …, skipping WM_INPUT entirely" (Using Raw Input). "Note that PeekMessage always retrieves WM_QUIT messages, no matter which values you specify for wMsgFilterMin and wMsgFilterMax." ([PeekMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew)) | documented |
| E8 | Buffered read rather than a `WM_INPUT`-filtered `PeekMessageW` | "Standard method reads one RAWINPUT structure at a time and is adequate for most devices." "Buffered method reads an array of RAWINPUT structures at a time and is useful for high-frequency devices such as mice at 1000Hz, where multiple events may accumulate between message loop iterations. The application calls GetRawInputBuffer to drain all accumulated events in a single batch." (Raw Input Overview) | documented. The design accumulates on purpose, so the buffered read fits. The standard read costs three user32 calls per event (`PeekMessageW`, `GetRawInputData`, `DispatchMessageW`→`DefWindowProcW`) and is kept as the fallback. |
| E9 | Cleanup | [DefRawInputProc](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-defrawinputproc): "Unlike DefWindowProcA and DefWindowProcW, this function doesn't do any processing. DefRawInputProc only checks whether cbSizeHeader's value corresponds to the expected size of RAWINPUTHEADER." The batched sample: "WM_INPUT messages are intentionally never dispatched through DispatchMessage", and it calls no `DefWindowProc` for them. [WM_INPUT](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-input): "RIM_INPUT … The application must call DefWindowProc so the system can perform cleanup." | `DefRawInputProc` is not called. The standard read dispatches every `WM_INPUT` it takes. Learn does not say in words that a `RIM_INPUT` record removed by `GetRawInputBuffer` needs no `DefWindowProc` **(unverified)**; Microsoft's own sample relies on it. M10 checks for a leak. |
| E10 | No device notifications, or the pattern spins | "Don't use RIDEV_DEVNOTIFY with this pattern. Because WM_INPUT_DEVICE_CHANGE is delivered through the raw input queue, the range-filtered PeekMessage calls used here can't retrieve it, which causes the thread to spin." (Using Raw Input) | The registration sets `RIDEV_INPUTSINK` only (`installer.rs:134`). Scan test F4 pins it. |
| E11 | Alignment | `pData`: "Pointer should be aligned on a DWORD (32-bit) boundary." WOW64: "To get the correct size of the raw input buffer, do not use \*pcbSize, use \*pcbSize \* 8 instead. To ensure GetRawInputBuffer behaves properly on WOW64, you must align the RAWINPUT structure by 8 bytes." ([GetRawInputBuffer](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getrawinputbuffer)). [NEXTRAWINPUTBLOCK](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-nextrawinputblock): "Retrieves the location of the next structure in an array of RAWINPUT structures." No formula is given. | `RawBuffer` is `align(8)`. 32-bit builds take the standard read, so the WOW64 caveat does not apply (CI builds only x86_64, `ci.yml:99-117`). The step `RAWINPUT_ALIGN = size_of::<usize>()` follows winuser.h's `RAWINPUT_ALIGN` (QWORD on `_WIN64`), which is **(unverified)**: it is not on Learn. It cannot matter for keyboard records: a record is `size_of::<RAWINPUTHEADER>() + size_of::<RAWKEYBOARD>()` = 24 + 16 = 40 bytes on x64, a multiple of 8. Tests pin both facts, and a wrong step would surface as `raw_input_unread`. |
| E12 | Why not Microsoft's sample loop as it stands | "Blocks thread execution until the thread needs to process a new message. The new message could be an input message, a queued message, or a non-queued message." ([WaitMessage](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-waitmessage)) | The sample's `WaitMessage` wakes for every new `WM_INPUT` and only defers the read. L1 forbids that per-key wake, so the mask is ours. |

### L1.4 D3, as changed

**Replaces** in D3's sketch `pub(crate) fn raw_event(&mut self, now: Instant) -> Option<Instant>;` (on the branch already `fn raw_event(&mut self, now: Instant)`, `decision.rs:120-128`) with:

```rust
/// `count` raw key events the hook should see were read at `now`, by one drain.
/// The first batch after idle starts the ticking and the silence (rule 1).
pub(crate) fn raw_batch(&mut self, count: u64, now: Instant);
/// One raw key event at `now`: `raw_batch(1, now)`. The decision rows keep their per-event scripts.
#[cfg(test)] pub(crate) fn raw_event(&mut self, now: Instant);
```

`raw_batch(0, _)` changes nothing. Otherwise `raw_events = raw_events.saturating_add(count)` and `last_raw_at = Some(now)`. If `next_tick` is `None`, it also sets `next_tick = now + TICK`, `quiet_since = now` and `owed = 0`. It equals `count` calls of `raw_event(now)` (test `a_batch_counts_as_that_many_raw_events`).

**Replaces** the paragraph "`evidence` is checked **before** the current interval's raw events are added. Every raw event that counts is therefore at least one tick (250 ms) old without a callback, …" with:

> `evidence` is checked **before** the current interval's raw events are added. The current interval's raw events are the ones the drains of the interval read. That is the drain of the tick's own pass, which runs right before the heartbeat is read, and the drain of any earlier wake in the interval (an early timer signal, a spurious return). A raw event counted at the pass of tick `T` arrived at or before `T`, and it is first checked at `T + TICK`. So every raw event that counts is at least one tick (250 ms) old without a callback, as with per-event reads. This absorbs the case where raw input arrives before its own hook call.

**Adds** to D3:
- `last_raw_at` is the instant of the drain that read the newest raw key event, at most one `TICK` after the event itself. Ticking stops at the first tick at or after `last_raw_at + ACTIVE_FOR`. That tick is at most the last key + `ACTIVE_FOR` + `TICK`, the same bound as with per-event reads (`decision.rs:188-195` unchanged).
- The main thread's answer is taken at the next pass, and while keys are typed that is the next tick. So rule 4's new window `S` starts at most one `TICK` after the answer arrived. Before L1 it started at most one raw-event gap after it. A batch read at that pass is counted **before** the answer is taken, as the per-event pump counted before `take_answer` (`mod.rs:617-620`, `:209`). So raw events read together with the answer are no evidence after it (rule 4).
- The detection latency is unchanged: at most `S + 2 × TICK` after the last callback.
- The anti-loop rule, its consequences, `tick`, `answered`, `not_sent` and `declined` are unchanged.

### L1.5 D5, as changed

**Replaces** D5's "Loop, in the house shape" bullets (from `MsgWaitForMultipleObjectsEx(Some(&[timer]), INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)` to "then set the timer for `next_tick`") and the `raw_key_event` bullets' claim "the only function that touches key data" with L1.2. Sketch:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LoopStats { pub(crate) wakes: u64, pub(crate) raw_keys: u64 }   // per thread life; wakes = passes
struct Drained { keys: u64, quit: bool }
#[repr(C, align(8))] struct RawBuffer([u8; RAW_BUFFER_BYTES]);
#[derive(Debug, PartialEq, Eq)] struct BlockCount { keys: u64, unread: u64 }
const RAW_BUFFER_BYTES: usize = 64 * size_of::<RAWINPUT>();
const MAX_BUFFERED_READS: usize = 64;
const MAX_STANDARD_READS: usize = 4096;
const RAWINPUT_ALIGN: usize = size_of::<usize>();
const BUFFERED: bool = cfg!(target_pointer_width = "64");

fn wake_mask(batched: QUEUE_STATUS_FLAGS, ticking: bool) -> QUEUE_STATUS_FLAGS; // idle: QS_ALLINPUT; ticking: QS_ALLINPUT & !batched
fn pump_ranges(skip: u32) -> [(u32, u32); 2];                                     // [(0, skip - 1), (skip + 1, 0xFFFF)]
fn count_raw_keys(bytes: &[u8], count: u32) -> BlockCount;
fn drain_keyboard(window: HWND, buffer: &mut RawBuffer, counters: &Counters, msg: &mut MSG) -> Drained;
impl Source {
    fn batched_wake(self) -> QUEUE_STATUS_FLAGS;   // Keyboard: QS_RAWINPUT; Synthetic: QS_POSTMESSAGE
    fn batched_message(self) -> u32;               // Keyboard: WM_INPUT;   Synthetic: SYNTHETIC_RAW
    fn drain(self, window: HWND, buffer: &mut RawBuffer, counters: &Counters, msg: &mut MSG) -> Drained;
}
impl<T: TickTimer> Ticker<T> {
    fn is_ticking(&self) -> bool;                                                   // watchdog.next_tick().is_some()
    fn pass<R: RequestReinstall>(&mut self, now: Instant, raw_keys: u64, context: &Context<'_, R>); // was pass(now, context)
    fn stats(&self) -> LoopStats;
}
impl HookWatchdog { #[cfg(test)] pub(crate) fn stop_counting(self) -> Option<LoopStats>; } // as stop, returning the thread's stats
```

- `Ticker::pass(now, raw_keys, ..)`, in order:
  1. `stats.wakes += 1` and `stats.raw_keys += raw_keys`.
  2. If `raw_keys > 0 && health.hook_installed()`: the first-arrival line once (`mod.rs:180-183`, unchanged), then `watchdog.raw_batch(raw_keys, now)`.
  3. `take_answer`.
  4. Tick if due, unchanged body (`:210-221`).
  5. `arm`.
- `Source::is_synthetic_raw` is removed. `post_synthetic_raw` is unchanged.
- `HookWatchdog::done` becomes `Receiver<Option<LoopStats>>`. `stop()` still returns `bool`, and `stop_counting` returns `Some(stats)` once the thread is joined within `STOP_TIMEOUT`.
- `wake_mask`, `pump_ranges`, `Source::{batched_wake, batched_message}`, `RawBuffer`, `count_raw_keys` and `BlockCount` arrive one or two commits before their first non-test use, with `#[cfg_attr(not(test), expect(dead_code, reason = "first used by Hook: the hook watchdog reads raw input in batches at its ticks"))]`.
- **Why it is correct:**
  - Hook-thread budget: nothing changes on main or in the callback.
  - Thread affinity: every read runs on the thread whose queue it reads (E6).
  - No spin: every pass removes all the raw input it finds, or 64 × 76 / 4 096 records of it, so a wait that pending raw input ends finds less each time (E4, E5, E10).
  - No lost first key: `MWMO_INPUTAVAILABLE` ends the idle wait for raw input that arrived between the last drain and the wait (E4, first quote).
- **Alternatives rejected** (this replaces D5's "`GetRawInputBuffer`: buffered reads are for high-rate devices …", which L1 reverses):
  - The standard read as the main read: three user32 calls per event at each tick, against one `GetRawInputBuffer` per 76 records. Kept only as the fallback and for 32-bit builds.
  - Microsoft's `SetTimer` + `WaitMessage` loop: one wake per key (E12).
  - Draining only at due ticks: an idle wake that raw input ended would leave that input unread. With `MWMO_INPUTAVAILABLE` the next wait would then return at once, which is a spin.
  - A per-burst log line in debug builds instead of the stop line: the release daemon that M9 checks would not have it, and a line per burst is log noise.
  - A wake counter in `COUNTERS`: the counters line is written every minute once any count is non-zero (`counters.rs:204-208`), so normal typing would add a line per minute, against the 1.7 rule that normal typing adds none.
  - **Fallback if M9 fails (not built now):** while ticking, wait with `WaitForMultipleObjectsEx` on the timer and a stop event, with no queue wake at all. This needs a stop event shared by `HookWatchdog` and the thread, a `Send` handle wrapper, and the rule that the handle outlives every wait ("If one of these handles is closed while the wait is still pending, the function's behavior is undefined", MsgWaitForMultipleObjectsEx). That would be a new delta.

**Consequential edits elsewhere above:**
- **D1 "Cost"** "Per key on another core: one `WM_INPUT` and one `GetRawInputData`." becomes "Per key event Windows queues one `WM_INPUT` for the watchdog thread (system work, M6). The thread reads the queue in one `GetRawInputBuffer` pass at the next tick, and the first key event after it went idle wakes it once (R16)."
- **Thread map**, `hook-watchdog` row: "`WM_INPUT` read per key event; ticks while active" becomes "one wake per burst; while ticking, no work per key event and one buffered read per tick (4 Hz)".
- **D6**: the key fields are read by `raw_key_event`, `count_raw_keys` and `qualifies`, and only there. `no_key_data_leaves_the_raw_input_read` allows those three.

### L1.6 D11, as changed: bench rows R14, R16, R15

**Replaces** D11's R14 bullet, removes `R14/settled` (developer deviation 10) and adds R16. All rows run on the bench thread, which is the main thread for the adapter rows (`bench.rs:902-904`). None registers raw input, installs a hook or injects input.

- **R14, steady typing with the watchdog ticking** (`r14_steady_typing_with_the_watchdog(2_000, 1 ms)`):
  - Setup as R12 (F13 `DoNothing`, A unbound). `health.set_hook_installed(true)`, and `start_synthetic` with `Unanswered`.
  - Outside the timing, one `post_synthetic_raw` (the first key of the burst, which R16 times) and a 5 ms spin, so that the watchdog ticks.
  - Then, for each of 2 000 samples, outside the timing: spin until 1 ms after the previous sample began, then `post_synthetic_raw`. Inside the timing: `call_keyboard_callback(VK_A, down/up)`.
  - The posts do not wake the ticking watchdog (synthetic mask). The callbacks move the heartbeat, so no request is made. Then `stop`, which takes at most one tick, and `set_hook_installed(false)`.
  - The row spans about 2 s, so 8 ticks drain about 250 queued events each. That is 20-50× the per-tick backlog of human typing. Each synthetic event costs one `PeekMessageW`, against one `GetRawInputBuffer` per 76 real records. So R14 **overstates** the share of samples a drain lands next to.
- **R16, the first key of a burst** (`r16_first_key_of_a_burst(100)`): 100 samples. Each sample:
  - `start_synthetic` (idle), then a 2 ms spin so that the thread is in its idle wait.
  - `post_synthetic_raw`, then at once a timed `call_keyboard_callback(VK_A, WM_KEYDOWN)`. The watchdog's one wake and first pass run next to the callback, the pessimistic order of Q6.
  - An untimed key-up, then `stop`, which takes at most one tick because the synthetic watchdog now ticks.
  - The row takes about 26 s. Thread start and stop stay outside the timing.
- **R15**: unchanged.
- **What no row contains:** Windows' own per-key work to build and queue a `WM_INPUT` (M6, which cannot be measured without a registration, and is unchanged by L1), and `GetRawInputBuffer`'s own cost, which is lower than the synthetic drain's.
- **Rules** (these replace D11's R14 row; 3 runs each on master and HEAD; R12 on both; R14, R15 and R16 exist on HEAD only):

| Row | Against | Bound | Justified extra work |
|---|---|---|---|
| R14 | HEAD R14c (ruling V3-I2); HEAD R12 reported for context | p50 ≤ R14c p50 + 300 ns. **L1's acceptance line.** p99 is not gated (ruling L2, 2026-10-19): the 1 ms-paced control alone swings 9-107 µs between runs, so R14's p99 is reported as the largest of 3 runs against R14c's largest of 3. *(As first written: against R12, p99 ≤ R12 p99 + 2 µs.)* | the watchdog ticking on another core, with its drains at 4 Hz |
| R14c | — | the control of R14: reported; its p99 is reported, not gated (ruling L2) | none (no watchdog) |
| R16 | HEAD R12 | **declared evidence row: measured and reported, not gated** (ruling L2, 2026-10-19). Measured: about 50 µs at p50 above R12, once per burst of keys (after at least `ACTIVE_FOR` = 2 s without keys), in the worst order of Q6 (the wake runs next to the key's own callback). Tester's three HEAD runs at `f51dc7d` (`bench-r2-head-{1,2,3}.md`): p50 54.6 / 52.7 / 46.1 µs (median 52.7), p99 107.6 / 101.2 / 121.9 µs, against R12 p50 4.7 µs; developer's run 53.5 / 168.4 µs. Whether a real key ever pays it is settled by manual check M6 (Q6). *(As first written: a sanity bound p50 ≤ R12 p50 + 25 µs, p99 ≤ R12 p99 + 100 µs; withdrawn by L2, since a wake from idle measured above it before L1 too, 35.6 µs on `0e7165b`.)* | the one watchdog wake per burst, which L1 keeps and declares |
| R15 | — | evidence, no bound | unchanged |

### L1.7 Rulings I1, M1, M5, rule 4 and the decision table on batched counts

| Rule | Why it holds after L1 | Pinned by |
|---|---|---|
| I1: the tick is never lost | While ticking, the tick timer and the `wake_by` timeout are now the only tick sources. Raw events no longer bring passes. `pass` → `arm` runs after every wake as before (`mod.rs:208-223,278-303`), so an early signal re-arms the same tick and a refused set falls back to the wait's timeout. | `an_early_timer_signal_does_not_lose_the_tick`, `a_timer_that_refuses_once_is_set_again_at_the_next_pass` on the batched `Drive`, with the same assertions (1 request in [1000, 1500] ms; the refused set is the one at 500 ms) |
| M1: one raw event keeps nothing ticking | `keeps_ticking` is unchanged. `last_raw_at` is at most event + `TICK`, so the idle tick is at most the last key + `ACTIVE_FOR` + `TICK`, the same bound as before. | `a_single_batched_raw_event_stops_ticking_after_two_seconds`; the decision rows `a_single_raw_event_without_a_callback_asks_for_nothing` and `evidence_keeps_the_watchdog_ticking_until_its_window_and_then_stops` are unchanged |
| M5: evidence before this interval's raw events | The batch of tick `T` is read before the heartbeat and added after `T`'s decide check (L1.4). | `a_batch_is_evidence_only_from_the_next_tick` (loop); `evidence_is_checked_before_this_intervals_raw_events_are_added` (decision, unchanged) |
| Rule 4: new evidence after an answer | The batch is counted before `take_answer`, so `start_over` drops it. | `raw_events_read_with_an_answer_are_no_evidence_after_it` (loop). It is red under the order mutation: a second request at 3 250 ms. |
| The D3 decision table | `tick` reads only per-interval counts (`decision.rs:142-145`), and a per-event delivery is a run of batches of one. `raw_batch(n, t)` equals `n` × `raw_event(t)`. Only `last_raw_at` moves, by less than `TICK`, and M1 above covers that. | `a_batch_counts_as_that_many_raw_events`; all decision rows unchanged; the loop replicas below |

**Batched `Drive` (test model of the real wait), replacing `mod.rs:837-926`:**
- A raw key event adds 1 to `queued`. It brings a pass only if `wake_mask(Source::Keyboard.batched_wake(), ticker.is_ticking()).contains(QS_RAWINPUT)`, and those passes are counted in `raw_wakes`.
- Every pass hands over `take(&mut queued)` as `raw_keys`.
- The timer and timeout handling is unchanged.
- The helpers `run_script(from, to, |ms, d| ..)`, `d.raw(ms)`, `d.health.callback_seen()` and `d.answer(outcome)` (which sends on the Recorder's newest waiting reply) let the scripts mix raw events, callbacks and answers.

| Loop test (`hook::watchdog::tests`) | Script (fake clock, ms) | Expected |
|---|---|---|
| `while_ticking_raw_key_events_wake_nothing` | raw every 30 ms, 0-3000; a callback 1 ms after each raw | `raw_wakes == 1`; passes ≤ 22; `stats().raw_keys == 101`; 0 requests |
| `the_first_raw_key_event_after_idle_wakes_the_watchdog` | raw + callback at 0; raw + callback at 5000 | `raw_wakes == 2`; `Cancel` before 5000; a `Set(250 ms)` at 5000 |
| `a_batch_read_at_a_tick_counts_all_its_events` | raw every 30 ms, 0-5000; no callback; unanswered | exactly 1 request, at 1000 ms; `seen == 0`; `silent_for ≥ 1 s` |
| `batched_reads_of_a_callback_that_keeps_up_ask_for_nothing` | the three variants of `a_callback_that_keeps_up_asks_for_nothing`, 10 s | 0 requests in each |
| `a_batch_is_evidence_only_from_the_next_tick` | raw + callback at 0; raws at 1248 and 1249; callbacks at 1251 and 1252 | the pass at 1250 reads 2; 0 requests |
| `a_single_batched_raw_event_stops_ticking_after_two_seconds` | one raw at 0, no callback, to 10 000 | 0 requests; the last timer call is `Cancel`, at ≤ 2250; no pass after it |
| `a_busy_main_thread_gives_one_request_with_batched_reads` | silent typing 0-20 000; at 10 000 the heartbeat +50 and `answer(Alive)`; then a callback 1 ms after each raw | exactly 1 request |
| `batched_reinstalls_without_callbacks_are_spaced_out` | silent typing 0-40 000; `Recorder::answering(Some(Reinstalled))` | 5 requests, at 1000, 3250, 7500, 15 750 and 32 000; gap `k` ∈ [2^(k+1) s, 2^(k+1) s + 500 ms) |
| `raw_events_read_with_an_answer_are_no_evidence_after_it` | typing 0-990; the request at 1000 unanswered; `answer(Reinstalled)` at 1001; raws at 1001 and 1003; then nothing to 10 000 | requests == [1000] |
| `raw_input_before_the_hook_is_installed_is_read_and_not_counted` (replaces `…_is_no_evidence`) | not installed: raw every 30 ms, 0-600; installed at 601; raw every 30 ms from 630 | `raw_wakes` in 0-600 == 21 (the thread stays idle and wakes per event); no `Set` before 630; 1 request at ≥ 1630; `stats().raw_keys` counts every raw read |
| `a_reply_is_taken_at_the_next_tick` (renames `a_reply_of_the_main_thread_is_taken_at_the_next_pass`) | as today, 0-6000 | 2 requests, gap ∈ [2000, 2600) (now 2250) |
| `the_loop_counts_its_wakes_and_raw_key_events` | raw every 30 ms, 0-300, then to 3000 | `stats().wakes` == the passes the `Drive` made; `raw_keys == 11` |
| the existing `a_failed_reinstall_is_no_longer_reported_once_the_hook_answers`, `a_failed_desktop_check_declines_and_is_counted`, `a_request_is_not_sent_while_input_is_on_another_desktop`, `a_request_the_main_threads_queue_refused_is_asked_again` | ported to `run_script`, same scripts | same assertions |

### L1.8 Win32 API choices (delta)

| Purpose | `windows` path (0.62.2 file:line) | Feature (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| batched read | `Win32::UI::Input::GetRawInputBuffer(pdata: Option<*mut RAWINPUT>, pcbsize: *mut u32, cbsizeheader: u32) -> u32` (`UI/Input/mod.rs:31`) | `Win32_UI_Input` (yes, named in `Cargo.toml` since cut commit 16) | hook-watchdog: it reads the calling thread's queue | `u32::MAX`: `raw_input_unread` + 1, then the standard read for the rest of the pass; `0`: the queue is empty | [GetRawInputBuffer](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getrawinputbuffer), [Raw Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input), [Using Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/using-raw-input) |
| wait mask | `QS_ALLINPUT & !QS_RAWINPUT` (`QUEUE_STATUS_FLAGS: BitAnd + Not`, `UI/WindowsAndMessaging/mod.rs:5563-5584`; `QS_RAWINPUT` `:5545`; `QS_POSTMESSAGE` `:5544` for the synthetic source) into `MsgWaitForMultipleObjectsEx(Option<&[HANDLE]>, u32, QUEUE_STATUS_FLAGS, MSG_WAIT_FOR_MULTIPLE_OBJECTS_EX_FLAGS) -> WAIT_EVENT` (`:1795`) with `MWMO_INPUTAVAILABLE` (`:5201`) | `Win32_UI_WindowsAndMessaging` (yes) | hook-watchdog | `WAIT_FAILED`: as today, the loop ends and the error is logged; spurious returns: a harmless pass | [MsgWaitForMultipleObjectsEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex), [GetQueueStatus](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getqueuestatus) |
| pump without `WM_INPUT` | `PeekMessageW(*mut MSG, Option<HWND>, u32, u32, PEEK_MESSAGE_REMOVE_TYPE) -> BOOL` (`:1842`), ranges `[0, B-1]` and `[B+1, 0xFFFF]`; `WM_INPUT` = `0xFF` (`:6990`) | yes | hook-watchdog | none | [PeekMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-peekmessagew), [Using Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/using-raw-input) |
| standard read (fallback; 32-bit builds) | `PeekMessageW(msg, Some(window), WM_INPUT, WM_INPUT, PM_REMOVE)` + `GetRawInputData` (`UI/Input/mod.rs:36`) + `DispatchMessageW` (`:575`) | yes | hook-watchdog | as today: `raw_input_unread` | [WM_INPUT](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-input), [GetRawInputData](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getrawinputdata) |
| not used | `DefRawInputProc` (`UI/Input/mod.rs:16`) | — | — | — | [DefRawInputProc](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-defrawinputproc) ("doesn't do any processing") |
| tests and bench | `PostThreadMessageW` (`:1872`, existing), `GetCurrentThreadId` (existing) | yes | test threads, bench thread | asserted or ignored | — |

**Injection, sentinel, `LLKHF_INJECTED`, scan codes:** unchanged. **Elevation and UIPI:** unchanged. The buffered read sees exactly what the standard read saw.

### L1.9 Touch set (delta)

| File | Change | Why |
|---|---|---|
| `src/hook/watchdog/mod.rs` | `LoopStats`, `Drained`, `RawBuffer`, `BlockCount`, `wake_mask`, `pump_ranges`, `count_raw_keys`, `drain_keyboard`, `Source::{batched_wake, batched_message, drain}`, `Ticker::{is_ticking, pass(now, raw_keys, ..), stats}`, `run_loop`/`pump`/`run_watchdog` (always pump; drain; quit drain), the stop line, the `done` type, `stop_counting`; `is_synthetic_raw` removed; module docs "# Cost" (`:22-35`), the loop doc (`:582-584`) and the pump doc (`:631-635`); tests: batched `Drive` and L1.7 rows, F1, F4, F6, F8a, F8b; the allowed list of `no_key_data_leaves_the_raw_input_read` | L1.2, L1.5 |
| `src/hook/watchdog/decision.rs` | `raw_batch`; `raw_event` `#[cfg(test)]`; docs (`:6-12` evidence, `:90-91` `last_raw_at`); `a_batch_counts_as_that_many_raw_events` | L1.4 |
| `src/logging/counters.rs` | docs: `RawInputUnread` (`:48-50`: also a failed buffered read and a record that cannot be read) and `WatchdogTimerNotSet` (`:51-54`: retried at the next tick through the wait's timeout; triage m1) | L1.5 |
| `src/bench.rs` | R14 redefined, R16 added, R14/settled removed; docs `:59-74` | L1.6 |
| `README.md` | `:939` watchdog bullet: the cost sentence (M6) becomes "Windows queues one `WM_INPUT` per key event for the watchdog thread, never for the main thread; while keys are typed the thread reads them four times a second in one batch, and the first key after 2 s without keys wakes it once, which makes a keyboard callback that runs at that moment about 20 µs slower; idle, it does not wake up". `:905` Technologies adds `GetRawInputBuffer`. *(Ruling V3-M2 and L2: the README quotes R16's measured figure, about 50 µs at the median, once per burst and in the worst order, never the planned 20 µs; `e75def6`, `4d6b228`.)* | user-visible cost statement (ruling M6) |
| `CHANGELOG.md` | `[Unreleased]` Added, the watchdog entry `:179-215`: "compares it, four times a second while keys are typed" gains "reading the raw input queued since the last tick in one batch (`GetRawInputBuffer`), so it does not wake per key"; "New lines" gains the stop line. Bench entry `:244-249`: R14 (steady typing with the watchdog ticking), R15, R16 (the first key of a burst); R14/settled removed | Keep a Changelog; the entries are unreleased, so they are edited in place |
| `book/src/config/errors.md` | new row `[INFO] hook watchdog stopped after N wakes for M raw key events` (at exit; M9 reads it); the `raw_input_unread` wording in the counters row `:230` | new log line |
| `src/config/default_config.ron`, other `book/src/config/*.md`, `Cargo.toml`, `Cargo.lock` | **no change** | no config meaning, no feature |
| `D:/tmp/ks-orch/p2-2.4-watchdog/cut.md` | the Developer addendum records this round (the developer) | traceability |
| Phase-2 book sync (`impl.md` list) | `architecture.md` thread table: `hook-watchdog` reads raw input in batches at 4 Hz and wakes once per burst | book owner |

### L1.10 Commits, in order (follow-ups on `0e7165b`; Opus trailer; no amend)

| # | Subject | Content | Gate |
|---|---|---|---|
| F1 | `Tests: a message wait leaves out a pending message its wake mask does not name` | `hook::watchdog::tests::a_message_wait_leaves_out_a_pending_message_its_wake_mask_does_not_name`. On a spawned thread of its own: make the queue (`PeekMessageW` `PM_NOREMOVE`), then `PostThreadMessageW(GetCurrentThreadId(), WM_APP + 0x30, ..)`. (1) `MsgWaitForMultipleObjectsEx(None, 100, QS_ALLINPUT & !QS_POSTMESSAGE, MWMO_INPUTAVAILABLE) == WAIT_TIMEOUT`, after ≥ 90 ms. (2) The same wait with `QS_ALLINPUT` returns `WAIT_OBJECT_0` (input, `nCount` 0) in < 50 ms. (3) Exactly one `WM_APP + 0x30` is then removed. Real Win32 on the thread's own queue: no hook, no raw input, no injection. | **green**, declared: it characterises Windows (E4), not our code |
| F2 | `Tests: the watchdog reports its wakes and raw key events when it stops (red)` | `the_watchdog_thread_reports_its_wakes_and_raw_key_events_when_it_stops`: `start_synthetic`, 5 posts 20 ms apart, then at once `stop_counting()` == `Some(LoopStats { raw_keys: 5, wakes })` with `wakes ≥ 1`. `the_loop_counts_its_wakes_and_raw_key_events` on today's `Drive`. | **red**. PREDICTED: `error[E0599]: no method named stop_counting found for struct HookWatchdog in the current scope`; `error[E0599]: no method named stats found for struct Ticker<FakeTimer>`; `error[E0412]: cannot find type LoopStats in this scope` |
| F3 | `Hook: the watchdog counts its wakes and logs them when it stops` | `LoopStats` in `Ticker` on today's per-event loop (`wakes` per pass, `raw_keys` per qualifying raw event, before the I3 gate); the stop line; `done: Receiver<Option<LoopStats>>`; `stop_counting`. Docs: errors.md row; CHANGELOG "New lines". Makes F2 green. | green |
| F4 | `Tests: the watchdog's wait and pump leave the batched raw input in the queue (red)` | `the_wait_leaves_the_batched_input_out_only_while_ticking`: `wake_mask(QS_RAWINPUT, false) == QS_ALLINPUT`; `wake_mask(QS_RAWINPUT, true) == QUEUE_STATUS_FLAGS(0xFF)`, which lacks `QS_RAWINPUT` and contains `QS_POSTMESSAGE \| QS_SENDMESSAGE \| QS_TIMER`; `Source::Keyboard.batched_wake() == QS_RAWINPUT`, `Source::Synthetic.batched_wake() == QS_POSTMESSAGE`; `batched_message` is `WM_INPUT` and `SYNTHETIC_RAW`. `the_pump_leaves_only_the_batched_message_in_the_queue`: for `skip` in [`WM_INPUT`, `SYNTHETIC_RAW`], every `m` in `0..=0xFFFF` lies in a range of `pump_ranges(skip)` iff `m != skip`. `the_raw_input_registration_asks_for_no_device_notifications`: a scan of `installer.rs`, which names `RIDEV_INPUTSINK` and not `RIDEV_DEVNOTIFY` (token built with `concat!`); green on arrival. | **red**. PREDICTED: `error[E0425]: cannot find function wake_mask in this scope`; `error[E0425]: cannot find function pump_ranges in this scope`; `error[E0599]: no method named batched_wake found for enum Source`; `no method named batched_message found for enum Source` |
| F5 | `Hook: the wait mask and the pump ranges of a batched read` | `wake_mask`, `pump_ranges`, `Source::{batched_wake, batched_message}` with the dead-code expect. Makes F4 green. No behaviour change. | green |
| F6 | `Tests: raw key events in a buffered read are counted in one pass (red)` | Test records are built from `to_ne_bytes` at `offset_of!` offsets, with no `unsafe`. `a_buffered_read_counts_the_key_events_the_hook_sees`: 6 keyboard records of 40 bytes, (0x1E, VK_A) ×2, (0xFF, VK_A), (0x1E, 0), (0x1E, 255), (0, VK_VOLUME_UP) → `BlockCount { keys: 3, unread: 0 }`. `the_walk_steps_by_each_records_size_rounded_up_to_a_pointer` (64-bit): keyboard at 0, a `RIM_TYPEHID` record with `dwSize` 36 at 40 (4 padding bytes), keyboard at 80 → keys 2, unread 0. `a_record_that_does_not_fit_ends_the_walk_as_unread`: record 2 has `dwSize` 4000 → keys 1, unread 2. `a_record_shorter_than_its_header_ends_the_walk`: record 2 has `dwSize` 0 → keys 1, unread 2, and the walk ends. `the_buffer_holds_64_records_aligned_for_the_buffered_read`: `align_of::<RawBuffer>() >= 4`; `size_of::<RawBuffer>() == 64 * size_of::<RAWINPUT>()`; on 64-bit, `size_of::<RAWINPUTHEADER>() + size_of::<RAWKEYBOARD>() == 40`. | **red**. PREDICTED: `error[E0425]: cannot find function count_raw_keys in this scope`; `error[E0422]: cannot find struct, variant or union type BlockCount in this scope`; `error[E0433]: failed to resolve: use of undeclared type RawBuffer` |
| F7 | `Hook: count the raw key events of a buffered read` | `RawBuffer`, `count_raw_keys`, `BlockCount` with the dead-code expect; `no_key_data_leaves_the_raw_input_read` allows `fn count_raw_keys(`. Makes F6 green. | green |
| F8a | `Tests: a ticking watchdog thread is woken by every synthetic raw event (red)` | `a_ticking_watchdog_thread_is_not_woken_by_synthetic_raw_events`: `start_synthetic` with `leaked_health`; one post; a 50 ms sleep; then for 1 s `callback_seen` + post + a 5 ms sleep (`n` posts); `stop_counting()` → `raw_keys == n + 1` and `wakes <= 12`. It compiles on F7's tree, which has `stop_counting` since F3. | **red by assertion**. PREDICTED: `a ticking watchdog woke W times for n + 1 raw key events (at most 12)`, with `W ≥ n + 1` (one wake per post; `n` ≈ 65-200 depending on the sleep's resolution) |
| F8b | `Tests: the watchdog reads raw input in batches at its ticks (red)` | `decision::tests::a_batch_counts_as_that_many_raw_events`: for n in [1, 2, 7, 250], `raw_batch(n, t)` against n × `raw_event(t)` gives an equal `Debug` and equal verdicts over two ticks; `raw_batch(0, t)` changes nothing. The batched `Drive` and every L1.7 loop row. `the_buffered_read_of_a_thread_without_raw_input_reads_nothing`: on a spawned thread, a `SinkWindow` with **no** registration and a `RawBuffer`; `drain_keyboard(..) == Drained { keys: 0, quit: false }`, and `RawInputUnread` is 0 (an FFI smoke test of the sizes; it reads only that thread's empty queue). | **red**. PREDICTED: `error[E0599]: no method named raw_batch found for struct Watchdog`; `error[E0599]: no method named is_ticking found for struct Ticker<FakeTimer>`; `error[E0061]: this method takes 2 arguments but 3 arguments were supplied` (`Ticker::pass`); `error[E0425]: cannot find function drain_keyboard in this scope` |
| F9 | `Hook: the hook watchdog reads raw input in batches at its ticks` | L1.2/L1.4/L1.5 in full; every expect of F5/F7 removed; module docs; counters docs (m1); README `:939`, `:905`; CHANGELOG watchdog entry; errors.md `:230`. Makes F8a and F8b green. | green |
| F10 | `Bench: R14 times steady typing with the watchdog ticking, R16 the first key of a burst` | L1.6; CHANGELOG bench entry. No red (the bench is ignored); the developer runs it once into `impl.md`. | green |

Tester mutations, each caught:

| Mutation | Caught by |
|---|---|
| `wake_mask` ignores `ticking` | F8a, `while_ticking_raw_key_events_wake_nothing` |
| the pump without range filters (it takes the batched message) | F8a `raw_keys` |
| the drain only at due ticks | F8a: the idle spin makes `wakes` explode |
| the batch counted after `take_answer` | `raw_events_read_with_an_answer_are_no_evidence_after_it` |
| no rounding, or rounding to 16, in the walk | `the_walk_steps_by_each_records_size_rounded_up_to_a_pointer` |
| no quit drain | F2 under batching (`raw_keys` < 5) |
| `RIDEV_DEVNOTIFY` added to the registration | the F4 scan |

### L1.11 Acceptance mapping (delta)

| L1 / acceptance line | Covered by |
|---|---|
| R14 within D11's bound in steady typing (`rulings.md:94-95`) | R14 vs HEAD R12, 3 runs each on master and HEAD (tester) |
| Idle: the wait includes raw input, so the first key of a burst wakes the thread and starts the ticks | `the_first_raw_key_event_after_idle_wakes_the_watchdog`, `the_wait_leaves_the_batched_input_out_only_while_ticking`; M9 |
| That one wake per burst is declared | R16 (declared evidence row, measured, not gated: ruling L2); README `:939` cost sentence; manual check M6 (Q6) |
| Ticking: the wait leaves out `QS_RAWINPUT`; each tick drains and counts in one pass | `the_wait_leaves_the_batched_input_out_only_while_ticking`, `while_ticking_raw_key_events_wake_nothing`, `a_ticking_watchdog_thread_is_not_woken_by_synthetic_raw_events`, `a_buffered_read_counts_the_key_events_the_hook_sees`, `a_batch_read_at_a_tick_counts_all_its_events`; F1 (the Windows rule for posted messages); M9 (for raw input) |
| Learn evidence: kept, lost or merged; limits; `DefRawInputProc` or cleanup; alignment | L1.3 E1-E12; M9 (E2, E3, E4), M10 (E9), F6 (E11) |
| I1, M1, M5 and the decision table keep working on batched counts | L1.7 table and loop rows; `a_batch_counts_as_that_many_raw_events` |
| No wake-ups at idle (Latency ruling, results analyst) | `a_single_batched_raw_event_stops_ticking_after_two_seconds`; M2 (amended) through the stop line |
| Ruling M6: the system-side cost is stated | README `:939`; L1.2 "Work per key event" |

### L1.12 Manual checks (delta; human only; to the owner from the start)

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| M9 (new): batching on the live daemon | `cargo build --release --locked`. Precondition (M7 of the rulings): `swch exit` any running daemon, then start the release daemon fresh (`swch open`, elevated as usual), so that its stop line covers this run only. Open Notepad. | 1. Leave the machine untouched for 60 s. 2. In Notepad hold `a` for 20 s, then release. 3. Wait 5 s. 4. Run `swch status` and record the output. 5. Run `swch exit`. 6. `Select-String 'hook watchdog stopped' "$env:LOCALAPPDATA\key-switch-rs\daemon.log" \| Select-Object -Last 1` (read-only). | Step 4 shows `hook ok, reinstalls: 0`. Step 6 shows `[INFO] hook watchdog stopped after W wakes for K raw key events` with **K ≥ 150** (about 600 at Windows' default repeat rate) and **W ≤ 130**. Expected: about 90 wakes for the 22.25 s of ticking around the hold, ≤ 40 for typing the two `swch` commands, and 0 for the idle minute. **FAIL** if W ≥ K, which is one wake per key: the wait does not honour the mask for `QS_RAWINPUT` (E4). Report it, and the orchestrator rules on the L1.5 fallback. Also no `raw_input_unread` in a counters line. | none (needs real raw input) |
| M10 (new): no leak from buffered reads (E9) | Release daemon running. Task Manager → Details, with the columns Handles, USER objects, GDI objects and Memory (private working set). | Record the daemon's four values. In Notepad hold `a` 60 s, release, wait 10 s, and record again. Repeat the hold once and record a third time. | Handles, USER objects and GDI objects are unchanged (±2) between the second and third records. The private working set grows < 1 MB between them. | none |
| M2 (amended) | as M2 | as M2, then `swch exit` and read the stop line as in M9 step 6 | as M2, and the stop line shows W ≤ 40 (only the bursts of typing the commands). A watchdog that woke while idle would show about 2 400 for the 10 minutes. | none |
| M1 (re-run) | as M1, on the batched debug build | as M1 | as M1: `reinstalls: 1` within 2 s of ScrollLock. Batching does not change the detection bound (L1.4). | none |

### L1.13 Risks and edge cases (delta)

- **Hook-thread budget.**
  - The callback and main are untouched.
  - The latency added to the key path is: steady typing about 0 (R14, bound R14c + 300 ns at p50); the first key of a burst about 50 µs at p50 above R12, measured (R16, a declared evidence row, ruling L2), in the worst order of Q6 only; and the key events that follow the first key within its pass (for example the injected down and up of a `PressKey` binding fired by that key) can meet the same pass. That is R16's case too, and the only one if raw input comes after the hook call (manual check M6).
- **Mask semantics (E4) unverified for raw input.**
  - If Windows ends the wait for pending raw input anyway, every pass drains, so the cost reverts to one wake per key event (the pre-L1 R14, about 22 µs), never to a spin.
  - M9 shows it, and L1.5 gives the fallback.
- **Spin guards.** No `RIDEV_DEVNOTIFY` (E10, scan). Every pass empties the raw input queue or reads 64 × 76 / 4 096 records of it. A failed buffered read switches to the standard read, which removes each message.
- **Backlog and quota (E3).** A starved thread reads what accumulated at its next pass, in parts. A message dropped at a quota lowers the evidence, the safe direction; the only cost is a possibly later detection during such a flood.
- **Answers.** An answer is taken at most one tick later, and rule 4's next window starts at most one tick later. The spacing grows slightly, which is the safe direction.
- **I3 gate.** Raw events before `install` and after `uninstall` are read and discarded, and each wakes the idle thread once. The quit drain reads raw input after `uninstall`, into the stats only.
- **Synthetic stop.** At most one tick while ticking, within `STOP_TIMEOUT`. This affects tests and the bench only; the daemon's `WM_QUIT` keeps `QS_POSTMESSAGE` in both masks.
- **32-bit builds** use the standard read (E11, WOW64). They are not built in CI, but the standard read is compiled and type-checked in 64-bit builds as the fallback.
- **Shared state and orderings.** No new shared state. `LoopStats` stays on the watchdog thread and crosses only through the `done` channel. No atomics, no `static mut`.
- **Reload, `swch off`/`on`, foreground changes, sided modifiers, IME and layouts:** unchanged. The drain counts, and the decision reads counts only.
- **No new module cycle.** The watchdog uses `logging::counters` and `platform::timers`, as today. The decision stays pure (`#![forbid(unsafe_code)]`), and `count_raw_keys` is safe code.

### L1.14 Questions for the orchestrator (non-blocking)

1. **The stop line is a new `[INFO]` line in every build, written once per run.** Recommendation: accept. It is the only release-build evidence of batching (M9) and of no idle wake-ups (M2), and it adds nothing to the per-minute counters line.
2. **R16's sanity bound** (p50 ≤ R12 + 25 µs, p99 ≤ R12 + 100 µs, n = 100). Recommendation: accept. It is the pre-L1 per-key wake cost with margin, and it catches a first-key pass that grew, for example a drain loop. *(Accepted in round 3, then withdrawn by ruling L2: R16 is a declared evidence row, measured and not gated; see L1.6.)*
3. **R14/settled is removed.** Recommendation: accept. R16 now carries the collision evidence, and R14 itself shows the steady state.
4. **If M9 fails.** Recommendation: proceed now, because E4's wording and F1 support the mask. On a failure, a new delta builds the L1.5 kernel-wait fallback. Researcher query if wanted: "MsgWaitForMultipleObjectsEx with MWMO_INPUTAVAILABLE and a dwWakeMask without QS_RAWINPUT: does a pending WM_INPUT end the wait (Windows 10/11)".
5. **Triage m1** (the `WatchdogTimerNotSet` doc) is folded into F9, because L1 rewrites the same sentence. m2-m4 and the T-note stay NOTED, outside this delta. Recommendation: accept.
6. **32-bit builds use the standard read** (E11). Recommendation: accept. This avoids the WOW64 layout of `GetRawInputBuffer` without dropping 32-bit support, which `core/constants.rs:19-26` keeps.

### L1.15 Readiness checklist (delta)

- [x] **Spec.**
  - Every reference the delta relies on is re-located on `0e7165b` (L1.1).
  - L1's text (`rulings.md:70-96`), R14's data (`test_r1.md:92-96`), I2 (`review_r1.md:40-60`), the triage and `fix_r1.md` have been read.
  - Goal: R14 within D11's bound in steady typing. Non-goal: the system-side `WM_INPUT` cost (M6).
- [x] **Decisions.** Each has its reasons and its rejected alternatives (L1.5), and the fallback is named. The owner decisions hold: lowest latency (R14 bound unchanged, R16 declared), RON, `log` with the sink, the licence. No config change.
- [x] **Threads.** Every new path is named: `hook-watchdog` (wait, pump, drain, stop line), the bench thread (R14, R16), the test threads (F1, F8a, F8b). Nothing runs in the LL callback.
- [x] **Install, uninstall and timers.** Unchanged and on main. The tick timer is on the watchdog thread, as before.
- [x] N/A **SendInput.** No new `SendInput` path.
- [x] **Modifiers.** Sided and unsided modifiers: N/A, because the drain counts and matches nothing.
- [x] **Gestures and lifecycle.** Reload, `swch off`/`on`, foreground and shutdown: L1.13. Shutdown: the quit drain and the stop latency.
- [x] **Shared state.** No new shared state and no new orderings; `LoopStats` goes through a channel.
- [x] **Module structure.** No new cycle. The decision logic stays in the pure `decision.rs`, and the loop rules sit in the `Ticker`, which the fake-clock `Drive` reaches.
- [x] **Win32.**
  - Each API has its `windows` path, feature check, thread, errors and Learn link (L1.8).
  - The unverified items are marked: E2, E3, E4, E9, E11. Their checks are M9, M10 and F6.
  - Elevation and UIPI: unchanged.
- [x] **Red-first.**
  - Each fix has a preceding red with its exact predicted failure: F2, F4, F6, F8a, F8b.
  - F1 and the F4 scan are declared green characterisations.
  - No test installs a hook, registers raw input or injects input. F1 and F8b call Win32 on their own thread's queue only.
- [x] **Gate.** Every commit except F2, F4, F6, F8a and F8b passes the gate. The dead-code expects bridge F5/F7 to F9. Docs land with the commit that changes the meaning (F3, F9, F10).
- [x] **Acceptance and questions.** Every L1 acceptance line maps to a test or a manual check (L1.11), and every question has a recommendation (L1.14).
