# Cut: p2-2.5-latency-trim — roadmap item 2.5 Latency trim (open question 13; FU-4; the witness follow-up)

## Verdict

CUT READY

Tree read: `master` at `50b56a8` (clean). Branch to create: `perf/latency-trim` from `master`.

Questions Q1-Q9 at the end are non-blocking, each with a recommendation. Two need the orchestrator before the
review starts:

- Q2: the declared residuals, set against "no key may ever stick".
- Q3: the R16 bound is provisional until the developer's probe.

## Spec re-located

Line numbers are on `50b56a8`. "Unchanged" means the code the spec or the book cites is where they say.

| Spec reference | Current file:line | Status | Note |
|---|---|---|---|
| The witness "src/engine/mod.rs around :885" (spec item 1; follow-up `roadmap.md:996` cites `mod.rs:878-896` @bf549c0) | `src/engine/mod.rs:878-896` (`on_key_up`), read at `:896` | unchanged | `decision.suppress = swallow && !env.os_key_down(vk)` |
| Bench row R11 (+0.6 µs) | `src/bench.rs:405-420` (`r11_swallowed_key_up`), row `:1098-1102`, doc `:51-52` | unchanged | 2.3: C11 (no witness) 4.8 µs, HEAD 5.4 µs (`D:/tmp/ks-orch/p2-2.3-control-window/bench/summary2.txt:5,11`) |
| Bench row R12 (ordinary key) | `src/bench.rs:422-436`, row `:1103-1107` | unchanged | 4.7 µs at 2.4 HEAD (`p2-2.4-watchdog/bench-r2-head-3.md:19`) |
| "The D14 rule" | `D:/tmp/ks-orch/p2-2.3-control-window/cut.md:405-414`; `CLAUDE.md:69-73` | unchanged | R12 rule: p50 ≤ 1.05 × base + 100 ns, p99 ≤ 1.5 × base p99 + 2 µs |
| The critic's unseen re-press script | `D:/tmp/ks-orch/p2-2.3-control-window/critique.md:18-24`; test `src/engine/scenarios.rs:2030-2045` | unchanged | must stay green |
| 2.3 stuck-key and lone-key-up tests | `src/engine/scenarios.rs:2011-2084` (witness), `:2091-2338` (lock, desktop switch), `:2342-2406` (reinstall); `src/engine/mod.rs:1241-1268`, `:1289-1316`; `src/engine/properties.rs:360-388` (six properties) | unchanged | listed in the acceptance mapping |
| The witness rule in the engine docs | `src/engine/mod.rs:197-210`, `:41-61`; `src/engine/env.rs:34-52` | unchanged | docs to update |
| Swallowed-record store and consume | `src/engine/keys.rs:62-85` (`LastDown`, `KeyState`), `:174-192` (`mark_blocked`, `mark_completed`), `:212-221` (`take_blocked`) | unchanged | |
| Engine resets and foreground change | `src/engine/mod.rs:613-615` (`on_foreground_changed`), `:651-666` (`reset`) | unchanged | the internal bumps go here |
| The adapter's `Env` | `src/hook/keyboard_hook_callback.rs:159-195` (`HookEnv`), `:178-184` (`GetAsyncKeyState`) | unchanged | |
| Input-signal notes | `src/hook/main_thread.rs:149-161` (`note_input_signal`), `:165-172` (`note_unknown_desktop_switch`); callers `src/hook/control_window.rs:242-248`, `src/hook/keyboard_hook_callback.rs:466-469` | unchanged | E3 bumps here |
| Refused re-entry | `src/hook/main_thread.rs:216-229` (`with_main`); `src/hook/keyboard_hook_callback.rs:236-247` (`decide_event`), `:424-425` (foreground callback) | unchanged | E8 and E1 bump here |
| The keyboard callback's timing | `src/hook/keyboard_hook_callback.rs:375-393` (`started`; debug hold `:383-384`; `record_callback` `:393`) | unchanged | E4 goes here |
| The main loop | `src/hook/main_thread.rs:676-720` (`MainThread::run`), wait `:698-705` | unchanged | E5 goes here |
| The 2.4 heartbeat | `src/hook/health.rs:54-61`; bumped at `src/hook/keyboard_hook_callback.rs:367` | unchanged | |
| `HealthState` | `src/hook/health.rs:19-52` | unchanged | gains `unseen_input` |
| Watchdog decision | `src/hook/watchdog/decision.rs:138-149` (`raw_batch`), `:171-222` (`tick`) | unchanged | |
| Watchdog loop, first pass after idle (FU-4 cites `watchdog/mod.rs:23-42` @bf549c0) | `src/hook/watchdog/mod.rs:23-42` (Cost doc), `:233-269` (`Ticker::pass`), `:324-349` (`arm`), `:694-750` (`run_loop`), `:756-762` (`wake_mask`), `:786-809` (`pump`), `:1014-1069` (`drain`), `:1076-1094` (`next_wait`) | unchanged | the deferral goes into `run_loop` |
| Bench row R16 (about 50 µs) | `src/bench.rs:579-609`, row `:1128-1132`, doc `:75-82`; 2.4 r3 p50 46.1-52.7 µs (`p2-2.4-watchdog/results.md:70`) | unchanged | |
| Q6 of item 2.4 | `D:/tmp/ks-orch/p2-2.4-watchdog/cut.md:691`; `book/src/roadmap.md:1005`; manual checks 43 and 44 at `roadmap.md:653-654` | open | settled as far as the evidence goes (D7) |
| 2.4 R16 diagnostics | `D:/tmp/ks-orch/p2-2.4-watchdog/bench-dev-diag.md:5-30` | evidence | D-B (no `raw_batch`, no timer) 53.6 µs; D-C (call 200 µs after the post) 16.5 µs; D-D (warm thread) 79.7 µs; R16/high (bench thread at HIGHEST) 58.5 µs |
| CHANGELOG "on its timer thread" (spec item 3) | `CHANGELOG.md:743-745` | unchanged | `:115-116` is stale the same way (`hold_timer_not_started`, "a Hold whose timer thread could not be started") |
| `src/platform/timers.rs:2-3` "later" (spec item 3) | `src/platform/timers.rs:1-5` | unchanged | |
| README latency notes | `README.md:149` (Env), `:564-567` (witness), `:865-874` (counters), `:948` (watchdog cost) | unchanged | |
| Config book pages that state a cost or a behaviour | `book/src/config/errors.md:214-215` (watchdog lines), `:231` (counters line); `book/src/config/known-issues.md:144` (row 76), `:146` (78), `:151` (83) | unchanged | |
| Roadmap 2.5 and its acceptance | `book/src/roadmap.md:586-595`; open question 13 `:952`; follow-ups `:996`, `:999`, `:1005` | unchanged | non-config pages: next book sync |
| `Cargo.toml` features | `Cargo.toml:41-58` | unchanged | `Win32_System_SystemInformation` is absent |

## Goal and non-goals

**After the lane:**

1. At a key-up the engine would swallow, it asks Windows about the key (`GetAsyncKeyState`) only when the
   **hiding count** has moved since Windows last had that key up for sure (D1-D4). A key pressed and released
   again with nothing in between that can hide key events pays no read. R11 equals R12 within the D14 rule.
2. The hook watchdog's first wake after idle makes no user32 read. It only starts its ticks; the raw input waits
   for the first tick, a quarter of a second later (D7). R16 is gated (D8).
3. The two leftovers of the Phase 2 book sync are fixed (D9).

**Non-goals:**

- **The modifier resync.** It makes 11 `GetAsyncKeyState` calls on every key event (`src/engine/keys.rs:102-110`,
  called at `src/engine/mod.rs:522`). That is about 4 µs of R12's 4.7 µs; R1, the engine alone, is 0.1 µs. It is
  the largest cost on the key path but outside 2.5's text. New row R17a measures one call. See Q5.
- **Settling Q6 for good.** Manual checks 43 and 44 stay the authority; D7 states the evidence.
- **A timer-only idle poll for the watchdog.** It is the fallback only if the probe trips its stop rule (Q3).
- Known issue 79's existing cases, the title read (owner Q12), and `KBDLLHOOKSTRUCT.time` as the engine's clock
  (a 2.2 follow-up). This lane uses the time stamp only to detect late events.

**Later items that depend on this:**

- 3.5 (Hot-path budget) can reuse the hiding count to gate the modifier resync (Q5).
- 3.2 (Sequence and Chord matching) must keep the `mark_completed` rule of D2.
- 4.6 (Tray) must deal with the same-process raw-input effect noted in D7 (Q8).

## Decisions

### D1: The hiding count, owned by the engine and fed by the adapter

**What.**

- The engine keeps `hiding: u64`, which starts at 1, and `notes_seen: u64`, which starts at 0.
- `Env` gains one required method:

  ```rust
  /// How often the adapter has noticed something that can hide key events from the hook and that
  /// the engine is not told about itself (roadmap item 2.5). Only grows. The engine compares it with
  /// the value it saw last, so any different value, a smaller one included, counts as news.
  fn hidden_input_notes(&self) -> u64;
  ```

- `Engine::hiding_now(&mut self, env: &dyn Env) -> u64` folds the notes in. If `env.hidden_input_notes() !=
  self.notes_seen`, it stores the value and adds 1 to `hiding` (`wrapping_add`). It returns `hiding`.
- The engine bumps `hiding` itself on what it is told directly:
  - `on_foreground_changed` (`mod.rs:613`);
  - every `reset` (`mod.rs:651`), for all four `ResetReason`s, `Reloaded` included.
- `hiding_now` is called only at a key-up the engine would swallow. No key-down pays it.

**Why.**

- The engine stays pure: the count is data. The engine is told about foreground changes and resets; the adapter
  counts what only it sees (signals before the drain, late callbacks, refusals, watchdog gaps). The rule lives
  where scenario tests pin it.
- The fold is lazy. A key event pays the adapter's two reads (a thread-local `Cell` and one `Acquire` load) only
  at a swallowed key-up. They replace the `GetAsyncKeyState`, about 0.5 µs (R11 − R12), with about 2-5 ns.
- The fold compares with `!=`, not `>`. The test harness and the local `KeyboardHook`s in tests can present a
  smaller sum, and any difference only adds a read.

**Alternatives rejected.**

- An adapter-only epoch passed with each event. The engine's own bumps would be untestable in scenarios, and
  every caller of `reset` would have to remember to bump.
- An eager fold at every key event: two reads on every key-down and key-up for nothing.
- A bump from the drain only: it leaves a gap between a WinEvent's note and its drain, in which a key event can be
  decided with the old count.

**Cost.** Two `u64` in the engine. Per would-swallow key-up, one TLS read, one atomic load and a compare. Per
foreground change or reset, one add.

### D2: Each key is stamped at its last release, not at its swallowed key-down

**What.**

- `KeyState` gains `up_since: [u64; 256]`: "the hiding count at which Windows last had this key up for sure".
  `NEVER_KNOWN_UP = 0` at start.
- At every key-up the engine decides, passed or swallowed, `on_key_up` ends with
  `self.keys.note_up(vk, self.hiding)`.
- The witness becomes:

  ```rust
  let was_blocked = self.keys.take_blocked(vk.0);
  let swallow = was_blocked || (released.gesture_bound && !was_passed);
  // roadmap item 2.5: ask Windows only if something that can hide key events happened
  // since Windows last had this key up for sure
  let ask = swallow && { let now = self.hiding_now(env); self.keys.up_since(vk) != now };
  decision.suppress = swallow && !(ask && env.os_key_down(vk));
  self.keys.note_up(vk, self.hiding);
  ```

- `KeyState::mark_completed(&mut self, vk: u16, pressed: u16)` gains the code of the key-down being decided.
  When it turns a held key with no record (`LastDown::Unknown`) into `Suppressed`, and that key is not `pressed`,
  it also sets `up_since[vk] = NEVER_KNOWN_UP`, so that key's release always asks. `pressed` is exempt: its record
  is written right after the binding loop (`mod.rs:846-853`), and its `up_since` from its last release is valid.
  The callers at `mod.rs:788-790` and `:814-816` pass `vk.0`.
- Resets do not raise `up_since`. Their bump makes every older stamp ask once.

**Why: the spec's suggestion, a stamp at the swallowed key-down, leaves a key stuck in a documented case that is
checked by hand.** Manual check 36 (`roadmap.md:646`) and known issue 76 (`known-issues.md:144`):

1. CapsLock is held and swallowed.
2. Ctrl+Alt+Del. Input leaves, the `InputLeft` reset releases every key, and the count moves.
3. On that screen CapsLock is released and pressed again, unseen. Windows has it down.
4. Esc. CapsLock is still held, and its next auto-repeat reaches the hook.
5. The engine has CapsLock released, so it reads the repeat as a first key-down, swallows it, and fires once more
   (known issue 76).
6. A key-down stamp would carry the count after the bumps. The key-up would skip the read and swallow the release,
   and CapsLock would stay down in Windows for good.

The hidden press happened *before* the key-down the engine saw. Only "since Windows last had the key up" covers
it.

The `Unknown` rule of `mark_completed` covers a key that Windows may hold without the engine having seen its
key-down. Today only modifiers that the resync found held fit that case, and sequences leave modifiers out
(`src/engine/sequences.rs:156-166`), so the rule is defence for 3.2.

**The price.** The first swallowed key-up of each key after each bump reads once: one read per key per Alt+Tab.
R11, presses in one window, is unaffected after the first. See Q1.

**Alternatives rejected.**

- A stamp at the key-down: it leaves a key stuck (above).
- A read at the key-down instead of the key-up: the same cost.
- A time window ("skip only for a press shorter than T"): a hidden release and re-press fit inside any T that a
  fast tap allows.

**Cost.** One store per key-up into a 2 KiB table, its index bounded by `slot()` (`keys.rs:56-59`).

### D3: The engine bumps on every foreground change and every reset

**What.** `on_foreground_changed` and `reset` add 1 to `hiding` before anything else.

**Why.**

- UIPI hides key events behind a foreground change. "The system posts keyboard messages to the message queue of
  the foreground thread that created the window with the keyboard focus"
  ([Keyboard Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input)).
  `EVENT_SYSTEM_FOREGROUND`: "The foreground window has changed. The system sends this event even if the
  foreground window has changed to another window in the same thread"
  ([Event Constants](https://learn.microsoft.com/en-us/windows/win32/winauto/event-constants)).
- Every reset marks a span the engine did not see:
  - `swch off` and `on`: the callback passes everything unread (`keyboard_hook_callback.rs:368-373`);
  - a reinstall: the hook was gone;
  - input leaving: another desktop has it;
  - a reload: it loses no event, but bumping keeps the rule "every reset" free of exceptions, at most one read per
    key after it.

**Alternatives rejected.** Leaving out the reload saves nothing measurable and makes `reset` depend on its reason.

**Cost.** None on the key path.

### D4: The adapter's notes: every way a key event can hide, and what moves the count

**What.**

- **Main-thread notes.** `hook::main_thread` gains the thread-local `HIDDEN_INPUT: Cell<u64> = const {
  Cell::new(0) }`, next to `PENDING_SIGNALS` (`main_thread.rs:103-112`), with two functions:
  - `pub(crate) fn note_hidden_input()`: `try_with`, `wrapping_add(1)`, never panics;
  - `pub(crate) fn hidden_input_notes() -> u64`: `try_with`, 0 on a TLS that is gone.
- **Watchdog notes.** `HealthState` gains `unseen_input: AtomicU64`.
  - `note_unseen_input()` is a `fetch_add(1, AcqRel)` on the `hook-watchdog` thread.
  - `unseen_input()` is a `load(Acquire)` on the main thread.
  - The counter publishes no other data; these are the house orderings (CLAUDE.md principle 5).
- **`HookEnv`.** `HookEnv::new(foreground: &'a ForegroundCache, health: &'a HealthState)` answers
  `hidden_input_notes()` with `main_thread::hidden_input_notes().wrapping_add(self.health.unseen_input())`. Both
  counters only grow, so the sum moves whenever either does.
- **Signatures**, so that every `HookEnv` has a health:
  - `MainState::reset_engine(&mut self, hook: &KeyboardHook, reason)` (`main_thread.rs:268`);
  - `MainState::foreground_changed(&mut self, hook: &KeyboardHook, window)` (`:282`);
  - `MainState::hook_reinstalled(&mut self, hook: &KeyboardHook)` (`:300`); its caller at `:861` becomes
    `with_main(|main| main.hook_reinstalled(hook))`;
  - `decide_event` passes `hook.health()` (`keyboard_hook_callback.rs:242`), and the foreground callback passes
    `get_hook()` (`:425`).
  - Tests that call these change with them (`keyboard_hook_callback.rs:639`, `main_thread.rs:1815`), and so does
    bench row R11f.

**The sources.** The E-numbers are used in tests and mutations.

| # | Source | Where, thread | Moves |
|---|---|---|---|
| E1 | A foreground change | `Engine::on_foreground_changed`, main. If `with_main` refuses the foreground callback, it calls `note_hidden_input()` (`keyboard_hook_callback.rs:424-425`) | engine `hiding`, or a note |
| E2 | Every reset | `Engine::reset`, main | engine `hiding` |
| E3 | Every input signal, at the moment it is noted: lock, unlock, switch away, switch back, and a switch to an unknown destination | `note_input_signal` and `note_unknown_desktop_switch` (`main_thread.rs:149-172`), called from the control window's procedure and the desktop-switch WinEvent callback, main | a note |
| E4 | A late keyboard callback: the event's `KBDLLHOOKSTRUCT.time` is at least 17 ms old when the callback starts, or its age plus the callback's run time reaches 17 ms when it ends | `keyboard_hook_callback` (D5), main | a note at entry (before deciding) and at exit (after carrying out); counter `hook_late` |
| E5 | A long awake stretch of the main loop: at least 17 ms from a wait's return to the next wait | `MainThread::run` (D5), main | a note; counter `hook_late` |
| E6 | Raw key events that no callback explains at a watchdog tick | `Ticker::pass` after `Watchdog::tick` (D6), `hook-watchdog` | `HealthState::unseen_input` |
| E8 | A key event refused because the main state was borrowed | `decide_event` when `with_main` returns `None` (`keyboard_hook_callback.rs:236-247`), main | a note |

**Why E3 notes in the callback and not in the drain.** The note is one `Cell` write, next to the existing
`PENDING_SIGNALS` write. No key event can be decided between a WinEvent and its note. The drain's reset (E2) comes
on top.

**Every way Windows can apply a key-down of key k that the engine does not see, after Windows last had k up for
sure** (the spec's "argue every way"):

| # | Way | Evidence | What moves the count before k's next key-up is decided | Left open |
|---|---|---|---|---|
| W1 | Keyboard focus in a higher-integrity window (UIPI). This hides keys from a non-elevated daemon (debug builds), and from the elevated daemon only for windows above High integrity | Keyboard Input Overview (above); AutoHotkey: "a non-elevated program cannot spy on input intended for an elevated program" ([FAQ](https://www.autohotkey.com/docs/v2/FAQ.htm)) | E1 when that window takes the foreground, and again when focus comes back. The window with the focus "is either the active window, or a child window of the active window" (Keyboard Input Overview) | That `EVENT_SYSTEM_FOREGROUND` reaches a lower-integrity out-of-context hook is **(unverified)**; manual check M1 (roadmap 35). A child window of a higher-integrity process inside a lower one's top-level window: residual R-E4 |
| W2 | Input on another desktop: Winlogon (lock, Ctrl+Alt+Del) or the secure desktop (UAC) | `EVENT_SYSTEM_DESKTOPSWITCH`: "The active desktop has been switched" (Event Constants); `WTS_SESSION_LOCK` (item 2.3) | E3 for every signal, including a late one that reads "back" and an unknown destination; E2 for the `InputLeft` reset | Windows delivering no switch, lock or foreground event at all: **(unverified)**; manual checks M2 and M3 (roadmap 36 and 28); residual R-E3 |
| W3 | Remapping off: the callback passes everything unread (`keyboard_hook_callback.rs:368-373`) | code | E2 at `swch on`, which also reads the keys from the OS | none |
| W4 | Windows removed the hook: "on Windows 7 and later, the hook is silently removed without being called" ([LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)) | docs | No hook call, so k's key-up reaches Windows (nothing to swallow); E2 at the reinstall | none |
| W5 | The callback for k's swallowed key-down timed out and Windows kept the hook. "If the hook procedure times out, the system passes the message to the next hook" (LowLevelKeyboardProc), and AutoHotkey reports: "The hook may time out up to 10 times, but is silently removed if it times out an 11th time" ([#HotIfTimeout](https://www.autohotkey.com/docs/v2/lib/_HotIfTimeout.htm)). Windows applies k's key-down although the engine swallowed it | docs, prior art | E4 at the exit of that callback: its age plus run time is at least the timeout, so at least 17 ms | `LowLevelHooksTimeout` below about 34 ms (D5): residual R-E2 |
| W6 | Hook calls dropped while the main thread was not pumping (timed out before dispatch), with the hook kept | from code; how Windows handles a timed-out sent message is not documented | E4 at the exit of the callback that held the thread; E4 at the entry of the next event that waited; E5 when the thread was busy outside a callback; E6, best effort | A stall outside every measured span (pre-emption by real-time threads, hard page faults) that drops k's release and re-press, after which k is released again before the watchdog's next tick: residual R-E1 |
| W7 | A refused callback (re-entry) passes k's events without the engine | `main_thread.rs:216-229` | E8 | none |
| W8 | The daemon's own injection of k, which carries the sentinel and is skipped (`keyboard_hook_callback.rs:222-224`) | code | Not needed. Every injection pairs a key-down with its key-up in one `SendInput` (`core/windows_actions.rs:287`, `data/sequence.rs:139,169,205`, `system/system_function.rs:155`, `keyboard_hook_callback.rs:322`), and "These events are not interspersed with other keyboard or mouse input events" ([SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)) | none |
| W9 | Another program's injected k (no sentinel) | LL hooks see injected events: `LLKHF_INJECTED` "Specifies whether the event was injected" ([KBDLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct)) | Not needed. The engine decides it like a physical event; a repeat of a held, swallowed k is swallowed | none |
| W10 | A low-level hook installed later swallows k | "it may return a nonzero value to prevent the system from passing the message to the rest of the hook chain or the target window procedure" (LowLevelKeyboardProc) | Not needed: Windows does not apply the event either | none |

Every residual needs Windows to behave outside its documentation, or a pathological stall (Q2).

**Alternatives rejected.**

- `GetLastInputInfo` as a universal detector of "input the hook did not see". It moves on every mouse event, so a
  read would follow any mouse use. Whether it covers the secure desktop is **(unverified)**.
- Reading `LowLevelHooksTimeout` from the registry to size E4. That adds a startup read and a code path for a
  misconfiguration (R-E2).
- Bumping in the drain only (D1).

**Cost on the key path.**

| Addition | Where | Expected | Bench |
|---|---|---|---|
| `GetTickCount`, subtract and compare at entry; add and compare at exit | every callback while remapping is on | about 2-5 ns (a user-mode read of the tick count, **(unverified)**; R17t measures it) | R12, R17t |
| The `note_up` store | every key-up | about 1 ns | R12 |
| `hidden_input_notes()` in place of `GetAsyncKeyState` | a would-swallow key-up with nothing hidden | about −550 ns | R11 |
| The same, plus the read | a would-swallow key-up after a bump | about +5 ns over today | R11f |
| `note_hidden_input()` and `counters::bump` | late events and refusals only | about 5 ns, rare | — |
| One `Instant::now()` per loop pass | the main loop, outside the callback | about 20 ns per pass | R7, R13 |

### D5: Lateness from `KBDLLHOOKSTRUCT.time`, `GetTickCount` and 17 ms

**What.**

- Constants in `hook::main_thread`: `pub(crate) const HOOK_LATE_MS: u32 = 17;` and
  `pub(crate) const HOOK_LATE: Duration = Duration::from_millis(HOOK_LATE_MS as u64);`.
- `keyboard_hook_callback` (`keyboard_hook_callback.rs:375-393`):
  - Right after `started`, it reads `event.time` and computes `waited = GetTickCount().wrapping_sub(event.time)`,
    with one `// SAFETY:` (takes no argument, reads the system tick count, cannot fail). If
    `late_at_entry(waited)`, it calls `note_late()`.
  - After `carry_out`: `let ran = started.elapsed(); counters::record_callback(ran);`, then `note_late()` if
    `late_at_exit(waited, ran)`.
  - `note_late()` is `main_thread::note_hidden_input(); counters::bump(Counter::HookLate);`, with no `log::`
    (principle 1).
  - The pure helpers have their own tests:
    - `fn late_at_entry(waited_ms: u32) -> bool` returns `waited_ms >= HOOK_LATE_MS`;
    - `fn late_at_exit(waited_ms: u32, ran: Duration) -> bool` returns
      `ran >= HOOK_LATE || waited_ms.saturating_add(ms_of(ran)) >= HOOK_LATE_MS`, where `ms_of` saturates to
      `u32`.
- `MainThread::run` (`main_thread.rs:676-720`):
  - `let mut awake_since = Instant::now();` before the loop;
  - right before every `MsgWaitForMultipleObjectsEx`, `note_if_awake_too_long(awake_since, Instant::now())`,
    which notes hidden input and bumps `HookLate` when `now - since >= HOOK_LATE`;
  - `awake_since = Instant::now()` right after the wait returns, before its result is looked at.
  - `Wake::Due => continue` skips the wait, so the stretch keeps adding up.
- New counter `Counter::HookLate`, `hook_late`:
  - `SLOTS` goes from 11 to 12 (`src/logging/counters.rs:17`). It joins `ALL` after `WatchdogDesktopCheckFailed`
    (`:66-78`), with a `Display` arm (`:82-96`) and slot index 11 (`:130-145`).
  - Doc: "A keyboard callback started or ended 17 ms or more after its key event, or the main thread spent 17 ms
    or more away from its messages. Windows may have passed a key event on without waiting for the daemon, so the
    next swallowed key-up of every key asks Windows whether it has the key down (roadmap item 2.5)."
- The tests' and the bench's `call_keyboard_callback` (`bench.rs:327-344`, `keyboard_hook_callback.rs:538-557`) and
  the injected struct at `keyboard_hook_callback.rs:871-875` set `time: GetTickCount()`. Without it every call is
  "late" (`time` 0 against the uptime), and R11 would read every time.

**Why 17 ms.**

- `GetTickCount`: "The resolution of the GetTickCount function is limited to the resolution of the system timer,
  which is typically in the range of 10 milliseconds to 16 milliseconds"
  ([GetTickCount](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount)).
- One tick boundary between the stamp and the read shows as 15 or 16 ms. 17 ms means at least two boundaries (or
  17 real milliseconds at a 1 ms tick), so a prompt event never counts.
- Detection is guaranteed for any timeout of about 17 + 15.6 + 1 ≈ 34 ms or more.
- AutoHotkey's research: "If LowLevelHooksTimeout is not defined, the default timeout is 300 ms" (#HotIfTimeout).
  Microsoft: "The maximum timeout value the system allows is 1000 milliseconds" (LowLevelKeyboardProc).
- Below 34 ms is residual R-E2.

**Why the event's own time stamp.**

- `KBDLLHOOKSTRUCT.time` is "The time stamp for this message, equivalent to what GetMessageTime would return for
  this message" (KBDLLHOOKSTRUCT). `GetMessageTime` gives "the elapsed time, in milliseconds, from the time the
  system was started to the time the message was created"
  ([GetMessageTime](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagetime)).
  `GetTickCount` returns "the number of milliseconds that have elapsed since the system was started". That both
  read the same counter is **(unverified)**.
- Subtracting while ignoring overflow is what `GetMessageTime` prescribes.
- An injector's odd `time` can only make an event look late, which adds a read.

**Why both entry and exit.**

- Exit catches W5: the verdict came after Windows had given up.
- Entry catches W6: a key-up that waited while WinEvents or dropped events were pending. It moves the count before
  that key-up is decided.
- A late key-down is noted at its entry before its release can be stamped, and noted again at its exit.

**Alternatives rejected.**

- `started.elapsed()` alone: it does not see the queue wait.
- A second `GetTickCount` at exit: the QPC run time already covers it.
- No E5: events dropped during a long drain would go unnoticed when no late event follows.

**Cost.** One `GetTickCount` per callback while remapping is on. Probe row R17t; stop rule in Q4.

### D6: The watchdog's gap note (E6)

**What.**

- `Watchdog::tick` (`decision.rs:171-222`) keeps `credit_before = self.credit` at entry and records
  `self.unseen_at_last_tick = arrived.saturating_sub(credit_before.saturating_add(answered))`: raw key events that
  neither earlier callbacks nor this interval's callbacks explain. Accessor:
  `pub(crate) fn unseen_at_last_tick(&self) -> u64`.
- `Ticker::pass` (`watchdog/mod.rs:262-266`) calls `context.health.note_unseen_input()` after a tick whose value is
  above 0.

**Why.**

- The acceptance names "a watchdog-seen gap".
- It is best effort:
  - Under Q6 = after, swallowed keys produce no raw input, so their callbacks can explain hidden events in the
    same interval.
  - A raw event read at a tick whose callback comes just after it gives a harmless extra note.
- It is not the only defence for W4-W6 (D4).

**Alternatives rejected.** The `evidence` value (it needs a silent hook for a whole second). A check per pass (the
passes come at the ticks anyway).

**Cost.** One RMW per tick that finds a gap, on the watchdog thread.

### D7: Q6, as far as the evidence settles it, and the deferred first read after idle

**Q6, question 1: does Windows produce raw input before or after the LL hook chain?**

Microsoft does not document it.

- LowLevelKeyboardProc: "The system calls this function every time a new keyboard input event is about to be
  posted into a thread input queue", and "the callback function is called before the asynchronous state of the key
  is updated".
- The [Raw Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input) is silent on
  hooks.

Prior art indicates *after*, and *none for a key a hook swallows*:

| Source | Claim | How read |
|---|---|---|
| [input-leap discussion #1458](https://github.com/input-leap/input-leap/discussions/1458) | "with the order WH_KEYBOARD_LL -> Raw Input -> [OS Update] -> WH_KEYBOARD"; "According to Microsofts documentation and observed behavior, GetAsyncKeyState updates after Raw Input has been processed" (no link given) | fetched |
| [GameDev.net 2009](https://gamedev.net/forums/topic/550264-dinput-and-global-key-hooks-vistawin7/) | a test program: "WH_KEYBOARD_LL does indeed block Raw Input / WM_INPUT messages" on Vista/7; disputed in the same thread | search summary; the page refused the fetch (403) |
| [PureBasic forum](https://www.purebasic.fr/english/viewtopic.php?t=66655) | "if set up keyboard_LL hook, Raw Input callback will never receive any messages" | search summary |
| [AutoHotkey forum t=6073](https://www.autohotkey.com/boards/viewtopic.php?t=6073) | "if you block keystrokes via a keyboard hook, you won't get the WM_INPUT messages" | search summary |
| [NotEnoughHotkeys](https://github.com/VollRahm/NotEnoughHotkeys) | per-keyboard blocking cannot block "Windows Key, Alt+Tab and State Keys (Caps Lock, Num Lock) because those are sent before RawInput gets anything"; consistent with the input-leap order, since its hook runs after raw input and after the OS update | fetched |
| [Casey Muratori, "Finding and Fixing a Five-Second Stall"](https://caseymuratori.com/blog_0006) | with raw input registered by the game's own window, that process's LL hook "never got called when The Witness window had focus"; `RIDEV_NOHOTKEYS` fixed it; he infers "Raw Input processing happens before low-level keyboard hook processing" | fetched. This is a same-process effect with a focused window. The daemon's windows never have the focus: the control window is hidden and the sink is message-only |
| [AltSnap PR #773](https://github.com/RamonUnch/AltSnap/pull/773) | premise "WM_INPUT is delivered independently of the low level hook chain"; the hook stops "as soon as the Menu has focus", and only "if the RegisterKeyboardRawInput() has been called" | fetched. The same same-process effect |

**Verdict:** **(unverified; probable)** raw input comes after the chain, and none comes for a swallowed key.
Manual checks 43 and 44 remain the authority; M5 amends the reading of 44 for the deferral.

**Q6, question 2: can the wake still meet a real callback under "after"?** Yes.

- The keyboard class driver hands the system a range of input packets at once
  (`KeyboardClassServiceCallback(..., InputDataStart, InputDataEnd, ...)`,
  [MS Learn](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/kbdmou/nc-kbdmou-pservice_callback_routine)).
  So the keys that change in one HID report are processed one after another (**(unverified)** timing).
- The burst's first raw event then wakes the watchdog while the next event of the same report is in its callback.
- R16's construction (post the raw event, then call at once) times that callback under both orders: under "before"
  it is the key's own callback, under "after" the next event's. **R16 keeps its construction and is gated (D8). It
  is not redefined.**

**The 2.4 diagnostics place the cost** (`p2-2.4-watchdog/bench-dev-diag.md`):

- It is not the decision or the timer (D-B 53.6 µs).
- It is not the main thread's priority (R16/high 58.5 µs).
- It is not a cold thread (D-D, warm, 79.7 µs).
- It lasts beyond 200 µs after the post (D-C 16.5 µs).

That leaves the woken thread's first pass and its user32 calls, on a core that has just woken, next to the
callback's 12 user32 calls:

- at least two `PeekMessageW` in `pump` (`watchdog/mod.rs:786-809`);
- two `GetRawInputBuffer` and one `GetQueueStatus` in `drain` (`:1014-1069`, `:1148-1154`);
- the next `MsgWaitForMultipleObjectsEx`.

**What.**

- `Watchdog::raw_pending(&mut self, now: Instant)`, pure, in `decision.rs`. When not ticking it sets
  `next_tick = now + TICK`, `quiet_since = now` and `credit = 0`. This is `raw_batch`'s first-batch branch without
  the count. `last_raw_at` is left alone: the first tick's drain sets it, "at most a tick after the event itself"
  (`decision.rs:106-108`).
- `Ticker::woke_idle(&mut self, now, context)`: `stats.wakes += 1`, `watchdog.raw_pending(now)`, then
  `self.arm(now, context)`.
- `run_loop` (`watchdog/mod.rs:703-749`) keeps the `mask` of the wait it makes. After the wait it takes the
  deferred path only when all three hold:
  - `woke == WAIT_OBJECT_0 + 1`, that is, input;
  - `mask.contains(source.batched_wake())`: the watchdog was idle, not backing off;
  - `context.health.hook_installed()`.

  The deferred path calls `ticker.woke_idle(Instant::now(), context)`, sets `drained = Drained::default()` and
  `continue`s, with no pump and no drain. Otherwise the loop runs as today.
- `MsgWaitForMultipleObjectsEx`: "WAIT_OBJECT_0 + nCount — New input of the type specified in the dwWakeMask
  parameter is available in the thread's input queue"
  ([MS Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex)).

**No spin and nothing lost.**

- The next wait is a ticking wait. It leaves out only the batched input. With `MWMO_INPUTAVAILABLE` ("The function
  returns if input exists for the queue"), a `WM_QUIT`, a sent message, or the "system event that requires the
  thread's action, such as foreground activation" that the same page says can end a wait, ends it at once.
- That wait then goes through the full pump ("call GetMessage or PeekMessage to process the system event before
  trying the call ... again").
- Two deferred wakes in a row are impossible, because the second wait has no batched bit.
- The raw input stays queued for the first tick's drain. That a wait leaves pending raw input out is 2.4's
  assumption E4, already in force while ticking. It is pinned for posted messages by
  `hook::watchdog::tests::a_message_wait_leaves_out_a_pending_message_its_wake_mask_does_not_name`
  (`watchdog/mod.rs:2525-2585`), and checked by manual check 47.

**The synthetic source (tests and bench).** Its batched bit is `QS_POSTMESSAGE`, which `WM_QUIT` shares, so an idle
synthetic watchdog stops at its first tick: at most 250 ms. That is what a ticking synthetic watchdog does today
(`bench.rs:514`), and it is within `STOP_TIMEOUT` (`watchdog/mod.rs:430`). The keyboard source keeps its immediate
stop.

**The 2.4 rules hold.**

- I3: the deferred path needs `hook_installed`; otherwise the full pass reads and discards, as today.
- The silence starts at the wake (rule 1).
- The first raw event's count lands in the first tick's interval, as before, so it becomes evidence from the second
  tick, as before.
- No idle wake-up is added.

**Loop-model tests whose timing moves with the deferred read.** Their meaning is unchanged. The developer adjusts
these and reports any other:

- `hook::watchdog::tests::a_batch_read_at_a_tick_counts_all_its_events` (`mod.rs:1630-1639`): `(250, 8)` becomes
  `(250, 9)`.
- `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request` (`:1812-1823`): the last read moves from 749
  to 750.
- `the_first_raw_key_event_after_idle_wakes_the_watchdog` (`:1606-1628`): "no pass in `2250..5000`" becomes
  `2500..5000`, because ticking ends 2 s after the first tick's read.

The `Drive` harness (`:1354-1519`) models the deferred wake. In `raw()`, idle with the hook installed, it calls a
new `Drive::wake(ms)`, which runs `ticker.woke_idle`, counts a pass and records the timer calls, with no read.

**Alternatives rejected.**

- A timer-only idle poll at 1 s: 600 wakes per 10 idle minutes. It breaks 2.4's ruling M1 and manual check 39's
  W ≤ 100. Its only gain over the deferral is the residual the probe measures (Q3 fallback).
- A stop event in the handle array, so that the idle mask can be `QS_RAWINPUT` alone with no classification. It
  changes 2.4's shutdown protocol to save a call the deferral already avoids.
- Raising or lowering the watchdog's priority: R16/high equals R16.
- Raw input on the main thread: rejected by 2.4 (D1, R15).

**Cost.** At the first key event after idle, the watchdog thread makes one wait return, one `SetWaitableTimer`
(kernel) and one wait entry, and reads nothing until the tick. Its first counted batch, and the
`raw keyboard input arrives` line, come one tick later.

### D8: Bench rows and comparison rules

**New and changed rows** (`src/bench.rs`; docs `:20-82`):

| Row | What | n | Gate |
|---|---|---|---|
| R11 | unchanged construction (`:405-420`), now timing the skipped read | 2000 | HEAD R11 p50 ≤ 1.05 × HEAD R12 p50 + 100 ns: the acceptance, R11 within the D14 rule of R12 |
| R11f (new) | R11 with `with_main(\|m\| m.foreground_changed(get_hook(), window))` between the untimed key-down and the timed key-up, outside the timing, where `window = GetForegroundWindow()` (read only). The witness still reads after a foreground change | 2000 | HEAD R11f p50 ≤ parent R11 p50 + 0.3 µs |
| R12 | unchanged | 2000 | D14: p50 ≤ 1.05 × parent + 100 ns; p99 ≤ 1.5 × parent p99 + 2 µs |
| R16 | unchanged construction; new doc: "the event right after the burst's first raw event: the key's own callback if raw input comes before the hook call, the next event of the same report if after (roadmap item 2.5)"; now gated | 1000 | HEAD R16 p50 ≤ min(HEAD R12 p50 + 20 µs, 0.5 × parent R16 p50); p99 reported. **Provisional until the probe (Q3)** |
| R17t (new) | `GetTickCount()`: the time of 1000 calls divided by 1000, per sample | 2000 | evidence; stop rule in Q4 |
| R17a (new) | `GetAsyncKeyState(VK_F13)` (read only), measured the same way | 2000 | evidence for Q5 |
| R1-R10, R13, R14/R14c, R15 | unchanged | | Their existing rules (2.2 D10, 2.3 D14, 2.4; `CLAUDE.md:69-73`). R4 has one swallowed F13 key-up in four events and may get cheaper. R7 and R13 carry E5's one `Instant` read per loop pass |

**The probe before commit 6.** The developer runs it after commit 2 and keeps it temporary, uncommitted, as 2.4
did with `bench-dev-diag.md`:

1. Add a diagnostic row D-R16d to `src/bench.rs` in the work tree only. A bench-local helper thread does exactly
   the work of the deferred wake:
   - it makes its queue and a `WaitableTimer`;
   - it waits with `MsgWaitForMultipleObjectsEx(&[timer], INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)`;
   - on input it reads `Instant::now()`, sets the timer for 5 ms and waits again with the posted bit left out;
   - when the timer fires it pumps everything and goes idle.
2. Each sample posts one message to the helper, makes the timed `call_keyboard_callback(VK_A, WM_KEYDOWN)` at once,
   then the untimed key-up, then spins 10 ms. n = 1000.
3. Run it with R12 and R16 in one release run, save the result to
   `D:/tmp/ks-orch/p2-2.5-latency-trim/bench-dev-diag.md`, and remove the row.
4. **Stop rule:** if D-R16d p50 > R12 p50 + 20 µs, stop after commit 5 and report to the orchestrator (Q3). The
   deferral would not remove the cost.

**Runs.**

- The developer runs the bench once at commit 2 and once at HEAD, into `impl.md`.
- The tester runs it three times on the parent (`master`) and three times on HEAD, per `tester.md` §4b, and reports
  the median of three p50s and the largest p99.

### D9: The Phase 2 book-sync leftovers

- `CHANGELOG.md:743-745`: "a `Hold` runs them on its timer thread as before and queues the others" becomes "a
  `Hold`, and a `Tap` that waits for its DoubleTap window, runs them on the main thread when its gesture timer fires
  (roadmap item 2.2), and queues the others".
- `CHANGELOG.md:115-116`, the same staleness: "(a Hold whose timer thread could not be started, so that it never
  fired)" becomes "(a Hold whose gesture timer could not be set, so that it never fired)".
- `src/platform/timers.rs:1-3`: "the action executor's `Delay` steps today, the gesture timers of roadmap item 2.2
  later" becomes "the action executor's `Delay` steps (roadmap item 2.1), the main thread's gesture timers (item
  2.2) and the hook watchdog's ticks (item 2.4)". The watchdog uses the timer at `watchdog/mod.rs:116-124`.

### D10: The scenario harness models noticed and unnoticed hidden events

**What** (`src/engine/testing.rs`):

- `FakeEnv` (`:91-135`) gains `hidden_notes: u64` and returns it from `hidden_input_notes()`.
- `up_unseen` (`:349-360`) and `down_unseen` (`:367-380`) also add 1 to `hidden_notes`. They now mean "an event the
  hook did not see, after which the daemon noticed something that can hide key events (roadmap item 2.5)".
- New `up_unnoticed` and `down_unnoticed` do the same with no note. They model what D4 argues cannot happen, so
  that tests can show what each source repairs.
- New `hidden_input_noted()` adds 1 and pushes a step. It stands for E3-E6 and E8.
- `input_signal` (`:432-446`) notes every signal (E3) before its reset.
- The module docs (`:1-71`) say so.

**Why.**

- The property tests (`properties.rs`) generate `down_unseen` and `up_unseen` with no foreground change between
  them. Under D2 these would leave keys stuck that, in the daemon, a source would have covered.
- With noticed unseen steps the properties go on checking the engine's logic: stamps, folds, resets and records.
- The sources are pinned one by one (S1-S7 and the adapter tests).
- `properties.rs` is unchanged, so its seeds keep their scripts.

**Alternatives rejected.** Generating an explicit source step before each unseen step changes the random stream of
every Unseen and Unlock script.

## Win32 API choices

| Purpose | `windows` path | Feature flag (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| How long a key event waited | `windows::Win32::System::SystemInformation::GetTickCount() -> u32` (`pub unsafe fn`; windows-0.62.2 `src/Windows/Win32/System/SystemInformation/mod.rs:194`) | `Win32_System_SystemInformation`: **absent; add** to `Cargo.toml:41-58` (same crate, `Cargo.lock` unchanged) | main (keyboard callback); bench and test threads | none. Wraps after 49.7 days, so use `wrapping_sub` | https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount |
| The event's time stamp | `KBDLLHOOKSTRUCT.time` (existing struct) | `Win32_UI_WindowsAndMessaging` (yes) | main | — | https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagetime |
| The watchdog's wait and its "input" result | `windows::Win32::UI::WindowsAndMessaging::MsgWaitForMultipleObjectsEx` (existing, `watchdog/mod.rs:714-721`); `WAIT_OBJECT_0 + 1` with one handle | `Win32_UI_WindowsAndMessaging` (yes) | `hook-watchdog` | `WAIT_FAILED` as today | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex |
| The first tick after a deferred wake | `platform::timers::WaitableTimer::set` → `SetWaitableTimer` (existing) | `Win32_System_Threading` (yes) | `hook-watchdog` | a failed set is counted and retried as today (`watchdog/mod.rs:334-347`) | https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer |
| R11f's window for the foreground change | `windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow` (read only) | yes | bench thread | a null window is fine | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getforegroundwindow |
| R17a | `windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState` (existing) | `Win32_UI_Input_KeyboardAndMouse` (yes) | bench thread | none | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate |

Elevation and UIPI: W1 and W2. No pipe, token or registry change. Nothing installs a hook, registers raw input or
injects input.

## Touch set

| File | Change | Why |
|---|---|---|
| `src/engine/env.rs` | `Env::hidden_input_notes`; docs of `os_key_down` (`:34-52`) | D1 |
| `src/engine/keys.rs` | the `up_since` table, `NEVER_KNOWN_UP`, `note_up`, `up_since()`; `mark_completed(vk, pressed)`; module doc; its test `Env`s gain the method; unit tests | D2 |
| `src/engine/mod.rs` | `hiding`, `notes_seen`, `hiding_now`; the witness gate and `note_up` in `on_key_up`; bumps in `on_foreground_changed` and `reset`; the `mark_completed` callers; module docs (`:39-61`, `:197-210`); its test `Env`s gain the method; unit tests | D1-D3 |
| `src/engine/testing.rs` | D10 | D10 |
| `src/engine/scenarios.rs` | S1-S7 | acceptance |
| `src/hook/main_thread.rs` | `HIDDEN_INPUT`, `note_hidden_input`, `hidden_input_notes`; notes in `note_input_signal` and `note_unknown_desktop_switch`; `HOOK_LATE(_MS)`; `note_if_awake_too_long` and its call in `run`; `MainState` methods take `&KeyboardHook`; test `QuietEnv`; tests | D4, D5 |
| `src/hook/keyboard_hook_callback.rs` | `HookEnv` with health and `hidden_input_notes`; the late check; notes on refusal (`decide_event`, foreground callback); test helper time stamps; module docs (`:1-110`); tests | D4, D5 |
| `src/hook/health.rs` | `unseen_input`, `note_unseen_input`, `unseen_input()`; test | D4, D6 |
| `src/hook/keyboard_hook.rs` | test `QuietEnv` (`:490-505`) gains the method | trait |
| `src/hook/watchdog/decision.rs` | `unseen_at_last_tick`, `raw_pending`; tests; docs | D6, D7 |
| `src/hook/watchdog/mod.rs` | `Ticker::pass` notes gaps; `Ticker::woke_idle`; the `run_loop` deferral; `Drive` models it; three expectations adjusted; Cost doc `:23-42` | D6, D7 |
| `src/logging/counters.rs` | `Counter::HookLate`; its tests' expected lines | D5 |
| `src/bench.rs` | `BenchEnv` gains the method; `call_keyboard_callback` time stamp; R11f, R17t, R17a; R16 doc; module doc rows | D8 |
| `src/platform/timers.rs` | module doc `:1-3` | D9 |
| `Cargo.toml` | feature `Win32_System_SystemInformation` | D5 |
| `CHANGELOG.md` | `:743-745` and `:115-116` (commit 1). Added: bench rows R11f, R17t, R17a (commit 2); `hook_late`, plus the counters example line `:120` (commit 3). Changed: the witness (commit 5); the watchdog's first wake and R16 gated (commit 7) | Keep a Changelog; roadmap 2.5 |
| `README.md` | `:149` and `:564-567` (commit 5); `:865-874` (commit 3); `:948` with R16's measured figure (commit 7) | latency notes |
| `book/src/config/errors.md` | `:231`, the counters line with `hook_late` (commit 3); `:214`, "at the first tick that counts a key event, a quarter of a second after it" (commit 7) | config page with a stated behaviour or cost |
| `book/src/config/known-issues.md` | row 78 (`:146`) reworded, and a new row 88 for residuals R-E1 to R-E4 (commit 5) | config page, behaviour |
| `src/config/default_config.ron` | **none** | no config change |
| Non-config book pages | **not touched.** The developer lists them in `impl.md` for the next book sync:<br>- `architecture.md:247`, `:251`, `:289`, `:352`, `:569`, `:605`;<br>- `glossary.md:187`; `:189-191` ("Epoch" belongs to the executor; add a new entry "Hiding count"); `:435`; `:575`;<br>- `roadmap.md`: the 2.5 status block (`:586-595`), Q13 (`:952`), follow-ups `:996` (closed), `:999` (FU-4 done) and `:1005`, manual check 44 (`:654`, the amended reading of M5), and the re-runs M1-M7;<br>- `overview.md` | spec: next book sync |

## Commits, in order

Each subject is `<Area>: <what>`. Each message ends with a blank line and
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Every commit passes the gate except the declared red
commits 4 and 6.

### Commit 1. `Docs: a Hold's fast actions run on the main thread, and timers.rs serves three threads`

- Implements D9: `CHANGELOG.md:115-116` and `:743-745`; `src/platform/timers.rs:1-3`.
- No test. Green.

### Commit 2. `Bench: rows R11f, R17t and R17a; the callback's events carry a time stamp`

- Implements D8:
  - `Cargo.toml`: the feature.
  - `bench.rs`: `call_keyboard_callback` sets `time: GetTickCount()`; rows R11f (with today's
    `MainState::foreground_changed(window)`), R17t and R17a; module doc rows.
  - `CHANGELOG.md` Added: "Bench rows R11f, R17t and R17a (roadmap item 2.5) ...".
- Green: the bench is ignored.
- The developer then:
  - runs the bench once into `impl.md`;
  - runs the probe of D8 into `bench-dev-diag.md`;
  - applies the stop rules of D8 and Q4.

### Commit 3. `Hook: count what can hide key events from the hook`

Implements D4, D5 and D6.

**No separate red commit.** The tests need the new API, so on master they are compile-red (`cannot find function
hidden_input_notes in module main_thread`, `no variant named HookLate`). Mutations M9-M14 carry them.

**Changes.**

- `Env::hidden_input_notes`, with every implementation:
  - `HookEnv` returns the sum;
  - `FakeEnv`, `BenchEnv` and the test `Env`s return 0: `keys.rs:462-475, 497-512, 575-590, 629-644`;
    `mod.rs:926-960, 1214-1239, 1272-1287`; `keyboard_hook.rs:490-505`; `main_thread.rs:946-961`. Commit 5 gives
    `FakeEnv` its field.
  - The engine does not call the method yet. The trait is public (`lib.rs:8`, `engine/mod.rs:256`), so there is no
    dead-code lint.
- `HookEnv` with the health; the `MainState` signatures (this also updates the bench's R11f call to
  `foreground_changed(get_hook(), window)`); `HIDDEN_INPUT` and its two functions.
- The E1, E3 and E8 notes; E4; E5; `Counter::HookLate`; `HealthState::unseen_input`; E6 in `decision.rs` and
  `Ticker::pass`; every test helper's time stamp.
- Every new `pub(crate)` function has a non-test caller in this commit.
- Docs: `errors.md:231`; `README.md:865-874`; `CHANGELOG.md` Added (`hook_late`) and the example line `:120`.

**Tests.**

- `hook::keyboard_hook_callback::tests`:
  - `a_callback_for_an_event_17_ms_old_notes_hidden_input_before_and_after_it_decides`: under `GLOBAL_HOOK`, with
    `time = GetTickCount() - 1000`, the notes grow by 2 and the `COUNTERS` `hook_late` by at least 2.
  - `a_callback_for_a_fresh_event_notes_nothing`: the notes do not grow. One retry of the whole check covers a
    CI stall.
  - `lateness_is_measured_with_wraparound_and_the_17_ms_boundary`: the pure `late_at_entry` and `late_at_exit`
    (16 gives false, 17 true; wraparound at `u32::MAX`; a run time of 17 ms or more gives true).
  - `a_refused_key_event_notes_hidden_input`: under `with_main(|_| decide_event(...))`.
  - `a_refused_foreground_change_notes_hidden_input`.
  - `the_hook_env_counts_the_main_threads_notes_and_the_watchdogs`.
- `hook::main_thread::tests`:
  - `every_input_signal_and_an_unknown_switch_note_hidden_input`: `ALL` plus `Unknown`, so +5.
  - `an_awake_stretch_of_17_ms_notes_hidden_input`: the pure `note_if_awake_too_long` with an `Instant` 20 ms old
    notes; one 5 ms old does not.
  - `the_main_loop_checks_its_awake_stretch_before_every_wait`: a scan of `fn run(` shows the call before
    `MsgWaitForMultipleObjectsEx` and `awake_since` reset right after it.
- `hook::health::tests::unseen_input_counts_each_note`.
- `hook::watchdog::decision::tests::a_tick_reports_raw_key_events_no_callback_explains`: raw events every 30 ms
  with no callback give `unseen_at_last_tick() > 0`; with callbacks keeping up it is 0 at every tick.
- `hook::watchdog::tests::a_tick_with_unexplained_raw_input_tells_the_main_thread`: on `Drive`,
  `health.unseen_input()` is above 0.
- The `logging::counters` tests, updated for 12 counters.

### Commit 4. `Engine: a key released again with nothing hidden asks Windows once (red)`

- Red-first test `engine::tests::a_swallowed_key_released_three_times_with_nothing_hidden_asks_the_os_once`: F13
  bound to `DoNothing`, `CountingEnv` (`mod.rs:1214-1239`; `hidden_input_notes` is 0), three down/up pairs at one
  instant.
- **PREDICTED RED:** `assertion `left == right` failed: a press released again with nothing hidden asks the OS only
  at its first release`, with `left: 3`, `right: 1`.

### Commit 5. `Engine: the witness reads only when something could have hidden the key's events`

Implements D1, D2, D3 and D10, and makes commit 4 green.

**Changes.**

- Engine: `hiding`, `hiding_now`, `up_since`, `note_up`, the gate, the bumps, `mark_completed(vk, pressed)`; docs.
- The harness (D10).
- Docs: `README.md:149` and `:564-567`; `CHANGELOG.md` Changed (the witness); `known-issues.md` row 78 and the new
  row 88.

**Unit tests.**

- `engine::tests::a_foreground_change_since_the_last_release_makes_the_next_swallowed_key_up_ask_again`: asked 2.
- `…::every_reset_reason_makes_the_next_swallowed_key_up_ask_again`: all four reasons.
- `…::a_change_of_the_adapters_notes_makes_the_next_swallowed_key_up_ask_again`: an `Env` with a `Cell` of notes,
  a smaller value included.
- `…::a_key_up_that_passes_records_the_release_so_the_next_swallowed_press_asks_nothing`:
  - binding `( keys: "F14", action: DoNothing, when: AppEquals("x.exe") )`;
  - a test `Env` whose foreground app is a `Cell`;
  - one F14 press while the app is `"y.exe"` passes, and the next, while it is `"x.exe"`, is swallowed;
  - `asked(VK_F14)` is 0.
- `engine::keys::tests::completion_of_a_held_key_with_no_record_other_than_the_key_pressed_forgets_when_it_was_up`.
- `engine::keys::tests::the_key_just_pressed_keeps_its_up_since_through_a_completion`.

**Scenarios.** These are green on master too, and sensitive to the mutations. Each runs on `DEFAULT_PAIR` under
`check_nothing_stuck`. Each starts with a baseline press and release of CapsLock, then a `down` that is swallowed.

- **S1** `a_foreground_change_since_the_last_release_lets_a_key_pressed_again_unnoticed_be_released_in_windows`:
  `foreground_changed`, `up_unnoticed`, `down_unnoticed`, then `up` passes.
- **S2** `a_lock_since_the_last_release_lets_a_key_held_again_on_the_lock_screen_be_released_in_windows`:
  `session_locked`, `up_unnoticed`, `down_unnoticed`, `session_unlocked`, `repeat` (suppressed; it fires
  SwitchLanguage, known issue 76), then `up` passes.
- **S3** `a_reload_since_the_last_release_lets_a_key_pressed_again_unnoticed_be_released_in_windows`:
  `reload(DEFAULT_PAIR)`, `up_unnoticed`, `down_unnoticed`, `repeat` (suppressed), then `up` passes.
- **S4** `a_remapping_switch_since_the_last_release_lets_a_key_pressed_again_unnoticed_be_released_in_windows`:
  `remapping_off`, `remapping_on`, `up_unnoticed`, `down_unnoticed`, `repeat` (suppressed), then `up` passes.
- **S5** `a_hook_reinstall_since_the_last_release_lets_a_key_pressed_again_unnoticed_be_released_in_windows`: S4
  with `hook_reinstalled`.
- **S6** `a_note_of_hidden_input_since_the_last_release_lets_a_key_pressed_again_unnoticed_be_released_in_windows`
  (a watchdog gap, a late callback, a refusal): `up_unnoticed`, `down_unnoticed`, `hidden_input_noted`, then `up`
  passes.
- **S7** `a_key_pressed_again_on_the_secure_desktop_whose_repeats_reach_the_hook_after_the_return_is_released_in_windows`
  (manual check 36; D2's case): `desktop_switched_away`, `up_unnoticed`, `down_unnoticed`,
  `desktop_switched_back`, `repeat` (suppressed; it fires SwitchLanguage), then `up` passes.

**Invariant kept.** A key-up is suppressed if and only if its key-down was, unless Windows has the key down. The
engine asks whenever something may have hidden that key since Windows last had it up.

### Commit 6. `Watchdog: the first raw key event after idle is read at the first tick (red)`

- Red-first test `hook::watchdog::tests::the_first_raw_key_event_after_idle_is_read_at_the_first_tick`: on `Drive`,
  one raw event and one callback at 0 ms, run to 600. It asserts `drive.reads.first() == Some(&(250, 1))` and
  `drive.raw_wakes == 1`.
- **PREDICTED RED:** `left: Some((0, 1))`, `right: Some((250, 1))`.

### Commit 7. `Watchdog: an idle wake only starts the ticks`

Implements D7, and makes commit 6 green. Only if the probe passed after commit 2.

**Changes.**

- `raw_pending`, `woke_idle`, the `run_loop` deferral, the `Drive` model, the three adjusted expectations, the Cost
  doc.
- Bench: the R16 doc and its gate note.
- Docs:
  - `README.md:948`, with R16's measured figure from the developer's HEAD run, never a planned one (2.4 rule
    V3-M2);
  - `errors.md:214`;
  - `CHANGELOG.md` Changed (the watchdog; R16 gated).

**Tests.**

- `hook::watchdog::decision::tests::a_deferred_first_read_starts_the_silence_at_the_wake`: after `raw_pending(0)`,
  with raw events counted from 250 and no callback, there is one request, with `silent_for ≥ SILENCE`, at 1000-1500.
- `hook::watchdog::tests::an_idle_wake_with_no_hook_installed_still_reads_at_once` (I3): `raw_wakes` per event, as
  in `raw_input_before_the_hook_is_installed_is_read_and_not_counted`.
- `…::a_deferred_wake_is_followed_by_a_wait_that_leaves_only_the_batched_input_out`: pure. `next_wait` after
  `woke_idle` gives the ticking mask, `QS_POSTMESSAGE | QS_SENDMESSAGE` included for the keyboard source.

After commit 7, the developer records in `impl.md` the HEAD bench, the list of non-config book pages and the manual
checks.

## Acceptance mapping

| Acceptance line (roadmap `:595`, spec `:15-19`) | Covered by |
|---|---|
| R11 (a key pressed and released in one window) is within the D14 rule of R12 | bench R11 against R12 (the D8 rule; tester §4b); `engine::tests::a_swallowed_key_released_three_times_with_nothing_hidden_asks_the_os_once` |
| A new row shows the witness still reading after a foreground change | bench R11f (≈ parent R11); `engine::tests::a_foreground_change_since_the_last_release_makes_the_next_swallowed_key_up_ask_again` |
| Every stuck-key and lone-key-up test and property of 2.3 stays green, the unseen re-press after a focus change included | `engine::scenarios::*` (unchanged): `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_is_released_in_windows`, `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows` (the critic's script), `a_swallowed_key_released_while_remapping_is_off_and_pressed_again_unseen_is_released_in_windows`, `a_swallowed_lshift_key_up_stays_swallowed_while_the_right_shift_is_down_in_windows`, `a_swallowed_key_held_through_a_lock_keeps_its_key_up_swallowed`, `a_key_swallowed_before_remapping_off_and_released_after_remapping_on_keeps_its_key_up_swallowed`, `nothing_is_stuck_after_win_l`, `after_a_lock_the_next_press_of_a_swallowed_non_repeating_key_fires`, `after_input_leaves_for_the_secure_desktop_the_next_press_of_a_swallowed_key_fires`, `a_swallowed_key_pressed_again_on_the_lock_screen_is_released_in_windows`, `a_swallowed_key_released_while_the_hook_was_gone_fires_at_its_next_press_after_the_reinstall`, `a_key_pressed_while_the_hook_was_gone_passes_its_key_up_after_the_reinstall`; `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key`, `the_witness_of_a_generic_shift_key_up_reads_the_generic_state`; the six `engine::properties` tests |
| New scenario tests pin each source: a foreground change, a reset, a remapping switch, a reinstall, a watchdog-seen gap | S1 (foreground change), S2 and S7 (lost-input reset), S3 (reload: every reset), S4 (remapping switch), S5 (reinstall), S6 (an adapter note: a watchdog gap, a late callback, a refusal); the adapter's sources: commit 3's tests, one per E-number |
| R16 meets a bound the cut sets from the evidence, or the evidence shows that a real key never meets the wake | D7: under the probable order a key's own callback never meets its wake, but the next event of the same report does, so R16 keeps its construction and is gated by the D8 rule (provisional until the probe, Q3); `hook::watchdog::tests::the_first_raw_key_event_after_idle_is_read_at_the_first_tick` |
| No other bench row regresses | tester §4b on parent and HEAD, with the D8 table |
| The 2.4 watchdog acceptance (the fake-clock unit rows) stays green | `hook::watchdog::decision::tests::*` unchanged and green, and the 2.4 acceptance rows at `roadmap.md:578-581` unchanged. Three loop-model expectations move by the deferred read, with their meaning unchanged (D7; Q6) |
| The book-sync leftovers | commit 1 (D9) |
| CHANGELOG (Changed, citing 2.5), the README latency notes, the config pages where a cost changes | commits 3, 5 and 7 (touch set) |

## Manual checks (human only)

None of these can run here: they need a live hook, the secure desktop or a debug daemon. No test is ignored for
them, because none of them can be a test.

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | Roadmap check 35 (`roadmap.md:645`): a non-elevated debug daemon, default config, Notepad and an elevated Task Manager | as check 35 | CapsLock is not stuck in `osk.exe`, and the next CapsLock switches the layout once. This now relies on E1 reaching the hook (W1) | none |
| M2 | Roadmap check 36 (`:646`): release daemon, default config | Hold CapsLock; press Ctrl+Alt+Del; there release CapsLock, press it again and keep it held; press Esc; release it on the desktop | Not stuck; the next press switches once. Relies on E3 (W2); this is D2's case | none |
| M3 | M2 with a UAC prompt; `PromptOnSecureDesktop` = 1 (read only, as in check 28) | Hold CapsLock; start an elevated program; at the prompt release CapsLock, then press and hold it; Cancel; release | Not stuck | none |
| M4 | 1. Stop the release daemon (`swch exit`; `swch status` says it is not running).<br>2. `cargo build --locked` (debug).<br>3. Default config, `osk.exe` open.<br>4. Read only: `reg query "HKCU\Control Panel\Desktop" /v LowLevelHooksTimeout` | 1. `$env:KS_DEBUG_HOOK_HOLD='CapsLock'; .\target\debug\key-switch-rs.exe`.<br>2. Press and release CapsLock once (the callback holds 1.5 s).<br>3. Look at `osk.exe`; type a few letters.<br>4. `swch status`; `swch exit`.<br>5. Read `daemon.log`.<br>6. `swch open` | CapsLock is not shown held in `osk.exe` after its release (the CapsLock light may toggle once, because Windows took the key-down while the callback held). A counters line shows `hook_late=` of at least 1. `reinstalls: 1` means that Windows removed the hook. `reinstalls: 0` with remapping working means that Windows kept it (known issue 83): this is W5, which E4 repairs. Report which | none |
| M5 | Roadmap check 44 (`:654`), amended | as check 44 | The `raw keyboard input arrives` line now comes at the watchdog's first tick, one tick (250 ms) after the raw input. Compare t_raw − 250 ms with t_key: earlier by about the hook timeout means before the hook call; within about 20 ms means after it (Q6) | none |
| M6 | Roadmap check 39 (`:649`), run again | as check 39 | W ≤ 100 as before: the deferral adds no wake | none |
| M7 | Roadmap check 47 (`:657`), run again | as check 47 | `W ≤ K/2`; `hook ok, reinstalls: 0` | none |

## Risks and edge cases (code-reviewer R1-R10)

- **R1 Hook callback.**
  - Added on every callback while remapping is on: one `GetTickCount` (with its SAFETY comment) and two compares.
    Only when late: a TLS increment and one `counters::bump`.
  - Added on key-ups: one table store. On a would-swallow key-up: a TLS read and an `Acquire` load in place of the
    `GetAsyncKeyState`.
  - No allocation, lock, `log::` or blocking call.
  - `code < 0` still short-circuits first. The late check sits after the remapping gate (`:368-373`) and before
    `decide_event`.
  - The check touches no engine state, so its place relative to the sentinel test (`decide_event`, `:222`) does
    not matter. That test still runs before any engine state is touched. A late event the daemon injected itself
    also notes, which only adds a read.
  - Every path still ends in one `LRESULT(1)` or one `CallNextHookEx`.
  - `up_since` is indexed through `slot()` (< 256).
- **R2 Sentinel.** No new `SendInput`. W8 argues the pairing.
- **R3 Shared state.**
  - `HIDDEN_INPUT` is a thread-local `Cell` of the main thread, like `PENDING_SIGNALS` (`main_thread.rs:111`). It
    is not shared state.
  - `unseen_input` is an atomic in `HealthState`.
  - There is no `Mutex` and no cross-thread `RefCell`, and nothing new runs under the borrow except pure engine
    work.
- **R4 Orderings.** `unseen_input` uses `fetch_add(AcqRel)` and `load(Acquire)`, and publishes no data. No
  `Relaxed`, no `SeqCst`.
- **R5 Builders.** Nothing changed.
- **R6 Display.** `Counter::HookLate` gets its arm, its `ALL` entry and its slot in the same commit. No `_ =>` arm.
- **R7 Specificity.** Untouched.
- **R8 Key identity.**
  - `up_since` is keyed like `last_down`, by the code the hook reported: sided, with injected generic codes in
    their own slots.
  - The witness still reads the event's own code
    (`a_swallowed_lshift_key_up_stays_swallowed_while_the_right_shift_is_down_in_windows`).
  - **Behaviour change:** the release of a generic `VK_SHIFT` that an injector sent, after a swallowed press, is
    now swallowed when nothing was hidden since its last release (rule L). Before, it passed whenever either Shift
    was down in Windows, which gave Windows a key-up of a press it never saw. Known issue 78 is reworded (commit 5).
- **R9 Registry and process.** No new read (D4, alternative b).
- **R10 Message loop and shutdown.**
  - E5 adds one `Instant` read per main-loop pass.
  - The watchdog's deferral cannot spin (D7).
  - An idle keyboard-source watchdog stops at once. An idle synthetic-source one stops within one tick, inside
    `STOP_TIMEOUT`.
  - The deferral needs the hook to be installed, so before `install` and after `uninstall` raw input is read and
    discarded at once (I3).

**Other edges.**

- A reload mid-gesture: the reset bumps, so the next swallowed key-up of each key reads once.
- `swch off` and `on`: bump and resync.
- A foreground change mid-press: bump (S1).
- Shutdown: nothing new.
- Auto-repeat: no read on repeats. D2 covers a repeat that reads as a first press (S7).
- Sided modifiers: the event's own code only.
- IME and layouts: no effect.
- The first press of each key after the daemon starts: it reads (`NEVER_KNOWN_UP`).
- Sleep and resume: `GetTickCount` counts through sleep, so an event stamped before the sleep and decided after it
  reads as late, which only adds a read.

**Residuals** (known-issues row 88; Q2):

- **R-E1.** A main-thread stall of at least twice `LowLevelHooksTimeout`, outside every span the daemon measures
  (pre-emption by real-time threads, hard page faults). Windows keeps the hook through it, the user releases and
  presses a swallowed key again during the stall, then releases it after the stall but before the watchdog's next
  tick. The key can stay down in Windows.
- **R-E2.** `LowLevelHooksTimeout` set below about 34 ms: a timeout of a swallowed key-down can go unnoticed.
- **R-E3.** Windows delivers no desktop-switch, lock or foreground event for a secure-desktop or UIPI episode.
  M1-M3 check this.
- **R-E4.** Keyboard focus moves to a child window of a higher-integrity process inside a lower one's top-level
  window, with no foreground change. Theoretical: that UIPI blocks cross-integrity `SetParent` and
  `AttachThreadInput` is **(unverified)**.

## Questions for the orchestrator

1. **The first swallowed key-up of each key after each bump reads once (D2).** The roadmap says "a key pressed and
   released in one window pays no read". The literal design leaves CapsLock stuck in manual check 36 and known
   issue 76 (S7). R11, repeated presses in one window, is unaffected.
   Recommendation: accept. Record in the 2.5 status block that the first press of each key after a foreground
   change, a reset or a signal reads once.

2. **Residuals R-E1 to R-E4 against "no key may ever stick".** The read stays on for every documented way a key can
   be hidden, and for the one known undocumented way, a timeout after which Windows keeps the hook (E4). Each
   residual needs Windows to behave outside its documentation, or a stall of hundreds of milliseconds outside every
   measured span with a release and re-press of the same key inside it.
   Recommendation:
   - Accept, declare known-issues row 88, and run M1-M4 after the merge.
   - If M2 or M3 fails because no desktop-switch event comes, the follow-up candidate is a note of hidden input on
     every foreground event whose window is null.
   - If the owner wants no residual at all, the trim of item 1 cannot be made. Then keep only commits 1-3, 6 and 7,
     and close the witness trim as "not possible without a residual".
   - That last case needs the owner only if the orchestrator rejects the recommendation.

3. **R16's bound is provisional: `p50 ≤ min(R12 + 20 µs, 0.5 × parent R16)`.** I could not probe it, because this
   role runs no builds. The probe (D8) runs after commit 2 and before commit 6.
   Recommendation:
   - If D-R16d p50 ≤ R12 p50 + 20 µs, the bound stands and commits 6 and 7 go ahead.
   - If not, the developer stops after commit 5: the cost is then in the wake itself, which the deferral cannot
     remove.
   - The only remaining option would be a timer-only idle poll. While idle, the wait leaves raw input out and times
     out every 1 s, and a poll that counts a batch starts the ticks. It costs 1 wake per second while idle (about
     600 per 10 minutes, against manual check 39's ≤ 100), and detection after idle is up to 1 s slower. It saves
     about 50 µs for one key event per burst that has two events in its first report.
   - In that case I recommend that the orchestrator keep R16 declared (2.4 ruling L2) rather than buy the idle
     wake-ups. If the orchestrator wants the poll, the owner decides it, because it is a choice about idle cost.

4. **E4's 17 ms threshold, and a `GetTickCount` on every callback.**
   Recommendation: accept. Stop rule for the developer: if R17t p50 is above 100 ns (which would mean a system
   call), stop after commit 2 and report. E4 would then read the tick count at exit only, and use the QPC run time
   for the entry part. That gives up E4's entry note, so W6 is covered only in part.

5. **The modifier resync is the key path's dominant cost** (a non-goal).
   - It makes 11 `GetAsyncKeyState` calls per event. R12 is 4.7 µs against the engine's 0.1 µs (R1), and R17a will
     give the cost per call.
   - This lane's hiding count can gate the resync the same way.
   - It needs its own cut: KS-14, the generic slots, exact matching, and modifiers held by the daemon's own macros.
   Recommendation: open it as a lane right after 2.5 and before Phase 3 (the owner's priority is latency first), or
   as the first part of 3.5. Record it in the follow-ups at the book sync.

6. **Three 2.4 loop-model tests change their timing expectations by one tick (D7).**
   Recommendation: accept. They are not 2.4's acceptance rows, their meaning is unchanged, and the deferral moves
   the first read by exactly one tick.

7. **The new counter `hook_late`**, a new token in the log.
   Recommendation: accept. It is the only trace of E4 and E5, and it is the evidence for known issue 83 (M4).

8. **A follow-up for 4.6 (tray).** A window of the daemon's own process that takes the focus while the process has a
   raw input registration stops the process's LL hook (Muratori; AltSnap #773). `RIDEV_NOHOTKEYS` cured it for
   Muratori. A tray menu would hit it, because `TrackPopupMenu` needs the daemon's window in the foreground.
   Recommendation: add it to "Follow-ups from Phase 2" at the book sync, for 4.6.

9. **CLAUDE.md, `tester.md` §4b and `code-reviewer.md` after the merge** (the orchestrator's chore lane):
   - the bench rules: R11 against R12, R11f, R16 gated, R17t and R17a;
   - principle 1: the late check is a sanctioned addition to the callback;
   - principle 2: the witness asks only after a bump;
   - principle 5: `HIDDEN_INPUT` and `unseen_input`;
   - the layout needs no change (no new files).
   Recommendation: do it in the chore after the lane, as after 2.3 and 2.4.

## Cut readiness checklist

### Spec

- [x] **Every cited issue read.** Item 2.5 cites no KS issue (roadmap `:439`). Its scenarios are reproduced as
  tests (S1-S7) or as manual checks (M1-M5): open question 13, the 2.3 critic's script, manual check 36 and known
  issue 76.
- [x] **Every reference re-located** (table above). None is gone.
- [x] **Goal, non-goals and the later items that depend on this are stated.**

### Design

- [x] **Every decision has a rationale and rejected alternatives** (D1-D10).
- [x] **Owner decisions respected.**
  - Exact modifiers and the repeat defaults are untouched.
  - RON is unchanged; there is no config change.
  - `log` and the sink: the hook path only counts.
  - The licence does not apply.
  - Latency first: every addition is costed and benched (the D4 table, D8).
  - "No key may stick": W1-W10, with the residuals put to Q2.
- [x] **Threads named.**
  - The main thread: E1-E5 and E8, the engine's fold, and the bench rows.
  - `hook-watchdog`: E6 and the deferral.
  - Nothing heavy runs in the LL callback; it gains one `GetTickCount`.
- [x] **Install, uninstall and timers.** Unchanged. The watchdog's timer stays on its own thread.
- [x] **`SendInput`.** No new call (W8).
- [x] **Sided and unsided keys.** `up_since` is keyed like `last_down`. Chord and sequence marks follow D2.
- [x] **Reload, `swch off`/`on`, foreground change and shutdown during a pending gesture.** Covered (D3, edges).
- [x] **Orderings stated** (R4). No `static mut`.
- [x] **No new module cycle.** `engine` gains only a trait method, and `hook` already depends on `engine` and
  `logging`. The logic sits in `engine`, where scenarios reach it.

### Win32

- [x] **Each API has its path, feature check, thread, errors and link.** One feature is to be added.
- [x] **Elevation and UIPI considered** (W1, W2). Pipe and tokens do not apply.

### Commits and tests

- [x] **Red-first.** Commit 4 → 5 and commit 6 → 7 are red by assertion. Commit 3 is compile-red on master and is
  carried by mutations. Mutations for the tester:

  | # | Mutation | Caught by |
  |---|---|---|
  | M1 | no bump in `on_foreground_changed` | S1 |
  | M2 | no bump in `reset` | S3, S4, S5 |
  | M3 | stamp at the swallowed key-down instead of the release | S7, S2 |
  | M4 | the gate ignores `up_since` and always skips | S1-S7 |
  | M5 | no `note_up` at a key-up that passes | `a_key_up_that_passes_records_the_release_so_the_next_swallowed_press_asks_nothing` |
  | M6 | `mark_completed` keeps `up_since` for an `Unknown` key | the keys unit test |
  | M7 | the deferral reads anyway | commit 6's test |
  | M8 | the deferral ignores `hook_installed` | `raw_input_before_the_hook_is_installed_is_read_and_not_counted` |
  | M9 | no E3 note | the signal test |
  | M10 | no E4 entry note | the 17 ms test (+1, not +2) |
  | M11 | no E8 note | the refusal test |
  | M12 | `HookEnv` drops the watchdog part | the `HookEnv` test |
  | M13 | no E6 note | the `Drive` gap test |
  | M14 | `late_at_entry` uses `>` | the boundary test |

- [x] **Tests go through `Engine::on_key`, `Env` and the fake clock.** The adapter tests drive the extern callback
  with no hook installed. Nothing hooks, registers raw input or injects input.
- [x] **Real Win32.** Only reads: `GetTickCount`, `GetAsyncKeyState`, `GetForegroundWindow`. The live checks are
  M1-M7.
- [x] **Every commit passes the gate except 4 and 6.** In commit 3, every new `pub(crate)` function has a non-test
  caller, and the trait is public. README, CHANGELOG and the config pages go in the commits that change the meaning.

### Acceptance

- [x] **Every acceptance line mapped** (table above).
- [x] **Every question has a recommendation** (Q1-Q9).
