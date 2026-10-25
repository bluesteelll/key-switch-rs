# Cut: p2-2.5-latency-trim — roadmap item 2.5 Latency trim (open question 13; FU-4; ruling W2: the modifier resync)

## Verdict

CUT READY

Tree read: `master` at `50b56a8` (clean; `.git/refs/heads/master`). Branch to create: `perf/latency-trim` from
`master`. Binding input: `rulings.md` (W1-W4, M1-M4, Q1-Q5), `critique.md` (I1, I2, M1-M4). The questions at the end
are non-blocking; each has a recommendation.

## Revision after rulings

| Ruling | What changed in this cut | Why |
|---|---|---|
| W1 | The witness trim leaves the lane: the hiding gate at the witness, the per-key `up_since` stamp, the `mark_completed(vk, pressed)` rule, the old witness tests (old commits 4-5, old S1-S7), mutations M1-M6, bench row R11f, the old residuals R-E1..R-E4 as stuck-key risks, the old Q1/Q2 trade and the known-issues row 78 rewording are gone. The witness reads at every swallowed key-up, as on master. R11 keeps its witness term (D13) | The skip could not be proven safe; "an unknown case keeps the read" |
| W2 | New D1-D7, D10, D11, D15; commits 3-7; scenarios S1-S9; a new property mode `Unnoticed`; known-issues row 88; bench gates for R12/R1/R4 set from probe D-R12n; new row R18 (the resync on main) | The modifier resync (about 4 µs of R12's 4.7 µs) leaves the key path; the hiding count of the old cut is its trigger |
| W3 | D12 (old D7) adopted; its Cost, the module Cost doc, `errors.md:215` and manual checks M6/M7 now say "one more wake per burst" (critique M3); R16 bound provisional, probed (D-R16d) and checked right after commit 11, with a stop rule | Owner's latency priority; M3 confirmed |
| W4 | `hook_late` is a resync trigger only (D8, D9): one exit check per callback; the stamp of an `LLKHF_INJECTED` event is ignored; a stamp ahead of the clock is age 0; a counters line whose only non-zero count is `hook_late` is written at `[INFO]`; manual check M8; the stamp/`GetTickCount` base is marked (unverified) with what Microsoft Learn says | A false positive costs one resync between events, never correctness |
| M1 | Residual R-M1 declared (row 88); E5 now loops straight back to the drain, so nothing is dispatched between its note and the resync | The resync never runs inside a callback (W2) |
| M2 | W-table rows W8 (a short `SendInput` count) and W9 (injected generic modifier codes); tracked injected modifiers (D2) and generic slots from events only (D3), each tested; feature 6 `HoldModifier` among the later items | Critique M2 |
| M3 | See W3 | |
| M4 | The mutation table names one mutation for every compile-red test | `tester.md:83`, `:170` |
| Q1-Q5 | Q1 moot; Q2 answered by row 88 (matching-only); Q3 = W3; Q4, Q6-Q9 accepted (Q9 content updated); Q5 = W2 | Rulings |

## Spec re-located

Line numbers are on `50b56a8`. "Unchanged": the code is where the spec, the book or the rulings say.

| Reference | Current file:line | Status | Note |
|---|---|---|---|
| Ruling W2: `KeyState::sync_modifiers` | `src/engine/keys.rs:95-110`; called at `src/engine/mod.rs:518-522` (every event, key-ups included) and `:613-615` (`on_foreground_changed`) | unchanged | 11 codes `MODIFIERS` `keys.rs:25-42`, generic ones included |
| Ruling W2: `resync_from_os` | `src/engine/keys.rs:112-142` | unchanged | reads codes 1..=254, generic 0x10-0x12 included |
| Ruling W2: `key_down_swallowed`, the KS-14 rule | `src/engine/keys.rs:153-156`; skip at `:105` | unchanged | kept by every new read |
| Records `LastDown` | `src/engine/keys.rs:61-71`; `mark_blocked` `:174-179`, `mark_completed` `:181-192`, `mark_passed` `:194-199`, `take_blocked` `:212-221` | unchanged | D10 adds a variant |
| Repeat rule 5 (no-record repeat) | `src/engine/mod.rs:823-845` (code), `:185-195` (docs) | unchanged | D10 |
| The witness (W1: stays) | `src/engine/mod.rs:878-896` | unchanged | `swallow && !env.os_key_down(vk)` |
| Auto-blocker OS read | `src/engine/mod.rs:721-723`; `src/engine/matching.rs:81-102` | unchanged | stays (Q10) |
| Mask rule | `src/engine/keys.rs:228-239`; `src/engine/mod.rs:901-909` | unchanged | reads flags (D11) |
| `Env::os_key_down` docs | `src/engine/env.rs:34-52` | unchanged | docs to update |
| `Engine::reset` | `src/engine/mod.rs:617-666` | unchanged | gains the resync bookkeeping |
| `HookEnv` (real `GetAsyncKeyState`) | `src/hook/keyboard_hook_callback.rs:154-195`, read `:178-184` | unchanged | used on the key path and, new, in the drain |
| Sentinel early return | `src/hook/keyboard_hook_callback.rs:222-224` (`decide_event` `:209-254`) | unchanged | D2 tracks modifiers there |
| Callback timing (`started`, `record_callback`) | `src/hook/keyboard_hook_callback.rs:375-393` | unchanged | E4 goes after `:393` |
| Foreground callback | `src/hook/keyboard_hook_callback.rs:403-426`; `MainState::foreground_changed` `src/hook/main_thread.rs:279-286` | unchanged | engine bump only, then a wake |
| `with_main` refusal branch | `src/hook/main_thread.rs:216-229` (`:218-224`) | unchanged | E8 note goes here |
| Signals | `note_input_signal` `src/hook/main_thread.rs:145-161`; `note_unknown_desktop_switch` `:163-172`; `answer_input_signals` `:760-790`; `MainState::input_returned` `:288-293` | unchanged | D6 |
| Main loop and drain | `MainThread::run` `src/hook/main_thread.rs:673-720` (wait `:698-705`); `drain` `:722-744`; `wake_this_thread` `:182-190`; `MainThreadHandle::wake` `:574-583` | unchanged | D4, E5 |
| `HealthState` | `src/hook/health.rs:17-61` | unchanged | gains `unseen_input` (E6) |
| Watchdog | trait `RequestReinstall` `src/hook/watchdog/mod.rs:90-107`; Cost doc `:23-42`; `Ticker::pass` `:226-269`; `run_loop` `:694-750`; `wake_mask` `:756`; `pump` `:786`; `drain` `:1014`; `next_wait` `:1076`; `Recorder` `:1249`, its impl `:1276`; `Drive` `:1354-1519`; `decision.rs` `raw_batch` `:135-149`, `tick` `:168-222`, `keeps_ticking` `:224-234` | unchanged | E6, D12 |
| Counters | `src/logging/counters.rs:17` (`SLOTS`), `:19-62`, `:64-79` (`ALL`), `:81-97` (`Display`), `:128-145` (`slot`), `:203-222` (`summary`); `src/logging/writer.rs:228-234` (`[WARN]` fixed) | unchanged | D9 |
| Bench | docs `src/bench.rs:20-82`; `BenchEnv` `:153-204`; R1 `:256-274`; `call_keyboard_callback` `:325-344` (`time` 0); R4 `:346-366`; R11 `:405-420`; R12 `:422-436`; `Unanswered` `:438-454` | unchanged | D13 |
| 2.4 HEAD bench | `D:/tmp/ks-orch/p2-2.4-watchdog/bench-r2-head-3.md:6,9,18,19,24` | evidence | R1 0.100, R4 4.700, R11 5.300, R12 4.700, R16 46.1 µs p50 |
| 2.3 witness cost | `D:/tmp/ks-orch/p2-2.3-control-window/bench/summary2.txt:5,11` | evidence | R11 4.8 → 5.4 µs with the witness |
| The daemon's injections | `src/core/windows_actions.rs:252-291`; `src/data/sequence.rs:163-212` (a macro `Combo("Ctrl+S")` sends the generic `VK_CONTROL`: `src/config/parsing.rs:19-22`); `src/system/system_function.rs:123-159`; mask `src/hook/keyboard_hook_callback.rs:297-323` | unchanged | every one pairs down and up in one `SendInput` |
| Harness | `src/engine/testing.rs:1-71` (docs), `:89-135` (`FakeEnv`), `:346-380` (unseen), `:382-395` (foreground), `:427-446` (signals), `:630-668` (`key_event`) | unchanged | D15 |
| Properties | `src/engine/properties.rs:1-55`, `:207-236`, `:250-303`, `:308-350`, `:360-388` | unchanged | new mode |
| Scenarios that lean on the per-event sync | `src/engine/scenarios.rs:189-203`, `:1011-1058`, `:3203-3233`, `:3526-3549`, `:5154-5167`, `:5543-5557` | unchanged | stay green under D15 (comments change) |
| Phase 2 leftovers | `CHANGELOG.md:116`, `:744-745`; `src/platform/timers.rs:1-3` | unchanged | D14 |
| README notes | `README.md:149`, `:553-567`, `:865-874`, `:948`, `:952` | unchanged | |
| Config book pages | `book/src/config/keys.md:194` (generic codes "from the Windows key state"); `errors.md:214-215`, `:231`; `known-issues.md:131` (63), `:144` (76), `:146` (78), `:147` (79) | unchanged | `kinds.md:186`, `triggers-conditions.md:147` are history: untouched |
| Roadmap 2.5 | `book/src/roadmap.md:586-595`; 3.5 status `:739`; FU-4 `:999`; `:1005`; checks 35-47 `:645-657` | unchanged | non-config pages: next book sync |
| `Cargo.toml` features | `Cargo.toml:41-58` | unchanged | `Win32_System_SystemInformation` absent |
| windows 0.62.2 items | `GetTickCount` `src/Windows/Win32/System/SystemInformation/mod.rs:194`; `KBDLLHOOKSTRUCT` `src/Windows/Win32/UI/WindowsAndMessaging/mod.rs:4251-4255`; `KBDLLHOOKSTRUCT_FLAGS::contains` `:4262`; `LLKHF_INJECTED` `:4409` | verified in the registry copy | |

## Goal and non-goals

**After the lane:**

1. The keyboard callback reads no modifier state. The engine follows Shift, Ctrl, Alt and Win from the key events it
   sees, the daemon's own sentinel-tagged modifier events included (D1, D2). R12 drops by about the 11 reads.
2. The main thread reads the eight sided modifiers from Windows between events, in its drain, when the hiding count
   has moved since its last read, and once a second while a modifier is held (D4-D7). No key event pays for it.
3. The witness is unchanged (W1). No key sticks and no key-up reaches Windows alone; stale flags affect matching only
   (D11), declared as known-issues row 88.
4. The hook watchdog's first wake after idle makes no user32 read (D12); R16 is gated.
5. The Phase 2 book-sync leftovers are fixed (D14).

**Non-goals:** the witness trim (W1); the auto-blocker's conditional OS read (KS-05, Q10); settling Q6 of item 2.4 for
good (manual checks 43 and 44 stay the authority); a timer-only idle poll (fallback only, Q3); known issue 79's cases;
`KBDLLHOOKSTRUCT.time` as the engine's clock.

**Later items that depend on this:**

- 3.1 (single input module): every new injection path keeps the sentinel, so D2 tracks its modifiers.
- 3.2 (Sequence and Chord matching): `mark_completed` reads flags that may be stale until the next resync (D11).
- 3.5 (Hot-path budget): its follow-up "skip the witness read" lapses (W1); FU-4 is done here (D12).
- 4.6 (Tray): the same-process raw-input effect (Q8).
- New feature 6 (`HoldModifier`): its held modifier is a sentinel event, tracked by D2, so matching sees it; its
  release on `off`, reload and Win+L must be injected the same way.

## Type sketch

Signatures only. `pub` items of `engine` are reachable from `lib.rs:8`; no dead-code lint applies to them.

```rust
// src/engine/resync.rs (new, pure, under the engine's #![forbid(unsafe_code)])
pub(super) const RESYNC_PERIOD: Duration = Duration::from_secs(1);

/// The eight sided modifiers the resync reads, one bit each (LShift, RShift, LCtrl, RCtrl, LAlt, RAlt, LWin, RWin).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ModifierBits(u8);
impl ModifierBits {
    pub(super) const NONE: ModifierBits;
    pub(super) const ALL: ModifierBits;
    /// Its own bit for a sided code, both bits of the pair for VK_SHIFT/VK_CONTROL/VK_MENU, NONE otherwise.
    pub(super) fn of_event(vk: VIRTUAL_KEY) -> ModifierBits;
    pub(super) fn keys(self) -> impl Iterator<Item = VIRTUAL_KEY>;
    pub(super) fn union(self, other: ModifierBits) -> ModifierBits;
    pub(super) fn without(self, other: ModifierBits) -> ModifierBits;
    pub(super) fn is_empty(self) -> bool;
}

/// When the main thread reads the modifiers from the OS (roadmap item 2.5).
pub(super) struct Resync {
    hiding: u64,                 // starts at 1: the first drain reads
    resynced: u64,               // `hiding` at the last full read; starts at 0
    notes_seen: u64,             // the adapter's notes at the last fold
    fresh: ModifierBits,         // touched by an event since the last plan
    owed: ModifierBits,          // left out of a read because fresh
    input_away: bool,            // after an InputLeft reset, until a return or a decided event
    held_since: Option<Instant>, // first plan that saw an unswallowed sided modifier held
    last_read_at: Option<Instant>,
}
impl Resync {
    pub(super) fn new() -> Self;
    pub(super) fn bump(&mut self);
    pub(super) fn event(&mut self, vk: VIRTUAL_KEY, decided: bool); // fresh |= of_event(vk); decided => input_away = false
    pub(super) fn input_left(&mut self);                            // input_away = true; bump
    pub(super) fn input_returned(&mut self);                        // input_away = false; bump
    pub(super) fn read_everything(&mut self, now: Instant);         // after resync_from_os
    /// The drain's plan: which modifiers to read now. Folds `notes`; reads the clock only when needed.
    pub(super) fn plan(&mut self, notes: u64, held: bool, now: &dyn Fn() -> Instant) -> ModifierBits;
}

// src/engine/keys.rs
enum LastDown { Unknown, Passed, Suppressed, RepeatSuppressed } // D10
impl KeyState {
    pub(super) fn resync_modifiers(&mut self, env: &dyn Env, which: ModifierBits); // KS-14 skip; never a generic code
    pub(super) fn track_injected(&mut self, event: KeyEvent);  // MODIFIERS only; KS-14 skip; `active` only
    pub(super) fn unswallowed_modifier_held(&self) -> bool;    // a sided modifier active whose key-down was not swallowed
    pub(super) fn mark_repeat_suppressed(&mut self, vk: u16);  // D10
    // sync_modifiers: deleted in commit 7
}

// src/engine/mod.rs
pub fn is_tracked_modifier(vk: VIRTUAL_KEY) -> bool;           // the 11 MODIFIERS codes
impl Engine {
    pub fn resync_if_due(&mut self, env: &dyn Env, notes: u64) -> bool; // true if it asked the OS
    pub fn input_returned(&mut self);
    pub fn track_injected(&mut self, event: KeyEvent);
    pub fn on_foreground_changed(&mut self);                    // was (&mut self, env: &dyn Env)
}

// src/hook/main_thread.rs
thread_local! { static HIDDEN_INPUT: Cell<u64> = const { Cell::new(0) }; }
pub(crate) fn note_hidden_input();          // main only; TLS bump, no post, never panics (try_with)
pub(crate) fn hidden_input_notes() -> u64;  // 0 on a TLS that is gone
pub(crate) fn wake_this_thread();           // was private
impl MainState { pub(crate) fn resync_if_due(&mut self, hook: &KeyboardHook) -> bool; }
pub(crate) const HOOK_LATE_MS: u32 = 17;    // commit 8
pub(crate) const HOOK_LATE: Duration;       // commit 8
fn note_if_awake_too_long(since: Instant, now: Instant) -> bool; // commit 8
impl MainThreadHandle { pub(crate) fn wake_for_hidden_input(&self); } // commit 9

// src/hook/keyboard_hook_callback.rs (commit 8)
fn event_age_ms(now_ms: u32, stamp_ms: u32) -> u32; // wrapping; a stamp ahead of now (difference >= 2^31) is 0
fn late_callback(age_ms: u32, ran: Duration) -> bool; // ran >= HOOK_LATE || age_ms >= HOOK_LATE_MS

// src/hook/health.rs (commit 9)
unseen_input: AtomicU64; fn note_unseen_input(&self); fn unseen_input(&self) -> u64;

// src/hook/watchdog/mod.rs (commit 9)
trait RequestReinstall { fn request(..) -> ..; fn hidden_input(&self) {} } // default no-op; MainThreadHandle overrides

// src/hook/watchdog/decision.rs
unseen_at_last_tick: u64; pub(crate) fn unseen_at_last_tick(&self) -> u64; // commit 9
pub(crate) fn raw_pending(&mut self, now: Instant);                          // commit 11

// src/logging/counters.rs (commit 8)
Counter::HookLate; impl Counter { pub fn warns(self) -> bool }  // every arm spelled out
pub fn summary(snapshot: &Snapshot) -> Option<(log::Level, String)>;
```

## Decisions

### D1: The modifier flags follow the key events; the key path reads no modifier state

**What.**

- `Engine::on_key` (`mod.rs:509-530`) no longer calls `sync_modifiers` (`:518-522`). It calls
  `self.resync.event(event.vk, true)`: one OR into `fresh` and one store of `input_away = false`.
- `press`/`release` (`keys.rs:158-172`) already set and clear the flag of every decided event. Nothing else changes
  a flag on the key path.
- `on_foreground_changed(&mut self)` only bumps the hiding count. `MainState::foreground_changed` keeps its cache read
  (`main_thread.rs:283`); the callback (`keyboard_hook_callback.rs:425`) then calls `wake_this_thread()` outside the
  borrow, so the drain runs right after the current message.
- `KeyState::sync_modifiers` is deleted (commit 7); `resync_modifiers` (D4) replaces it, off the key path.

**Why.** The 11 `GetAsyncKeyState` calls are about 4 µs of R12's 4.7 µs (ruling W2). Every key event the hook sees
already updates its own flag; the reads only repaired flags made stale by events the hook did not see. Those are
rare, and each has a source that moves the hiding count (D4) or is caught by the periodic read (D7).

**Alternatives rejected.**

- Gating the per-event sync on the hiding count inside the callback: the read would still run on a key event (ruling
  W2: "never inside a callback").
- Reading only the modifiers a matching binding names: still reads on the key path, and exact matching needs every
  family (owner decision).

**Cost on the key path.** About −4 µs per event (R12, R4, R5, R9, R10, R11 all lose it); +1-2 ns (`fresh`, the store).

### D2: The daemon's own injected modifier events are tracked

**What.** In `decide_event` (`keyboard_hook_callback.rs:222-224`), a sentinel event whose code is one of the 11
`MODIFIERS` (`engine::is_tracked_modifier`) calls `with_main(|m| m.engine.track_injected(event))` and still returns
`None` (passes, nothing decided, nothing fired). `KeyState::track_injected` sets or clears `active` only, leaves every
record alone, and skips a modifier whose key-down was swallowed (KS-14, `keys.rs:105`): the user holds that key, and
its auto-repeats must stay auto-repeats. `Resync::event(vk, false)` marks it fresh and does not clear `input_away`.

**Why.**

- Ruling W2: the engine tracks "the sentinel-tagged injected ones included where they change Windows' state".
- Every injection of the daemon pairs each key-down with its key-up in one `SendInput`
  (`windows_actions.rs:258-287`, `sequence.rs:163-212`, `system_function.rs:125-155`), and "These events are not
  interspersed with other keyboard or mouse input events" ([SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)).
  A whole pair is net zero for the flags.
- A short count is not: "The function returns the number of events that it successfully inserted" (same page). The
  inserted events go through the hook, so tracking them leaves the engine with what Windows has (W8).
- Feature 6 (`HoldModifier`) will hold an injected modifier by design.

**Alternatives rejected.** A note of hidden input at each short count (critique M2's option): it reads the OS later
for what the hook already saw. Tracking every injected event: `TypeText` sends one pair per code unit, and only
modifiers feed matching.

**Cost.** Only on sentinel modifier events: a code compare, one `with_main`, one store, about 10-20 ns. Never on a
physical event. CLAUDE.md principle 3 ("passes those events ... before it mutates any state") is amended by this
decision (Q9): the event still passes and decides nothing, so no feedback loop is possible.

### D3: The generic modifier slots come only from events

**What.** Neither the drain resync (`resync_modifiers`, sided codes only) nor `resync_from_os` (`keys.rs:127-142`,
commit 7: codes 0x10-0x12 read as `held = false`, records as for any key the OS has up) takes `VK_SHIFT`,
`VK_CONTROL` or `VK_MENU` from Windows. Those slots are set and cleared only by events under a generic code: another
program's injection, or the daemon's own macro `Combo("Ctrl+S")`, whose `"Ctrl"` is the generic code
(`config/parsing.rs:19-22`).

**Why.** `GetAsyncKeyState(VK_SHIFT)` "gives the state of the SHIFT, CTRL, or ALT keys without distinguishing between
left and right" ([GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate)).
Today the per-event sync clears the generic slot at the next event. Without it, a read that found LShift down would
leave `VK_SHIFT` held after LShift's key-up until the next read: a phantom Shift family on every key. The sided
slots carry the same information.

**Behaviour change (reviewer R8).** An injector's generic code now counts for matching only between its own key-down
and key-up (W9). Known issue 78 ("Such an Alt or Win does not count for the menu mask key") now holds for Alt too:
master's sync could set the generic `VK_MENU` slot for an Alt pressed again unseen. Pinned by
`engine::keys::tests::a_resync_never_takes_a_generic_modifier_from_the_os` and
`engine::tests::an_injected_generic_ctrl_counts_for_matching_between_its_key_down_and_key_up`.

**Cost.** None on the key path.

### D4: The hiding count, its sources, and the drain step

**What.**

- `Engine` owns `resync: Resync` (type sketch). `Engine::resync_if_due(env, notes)`:
  1. `held = keys.unswallowed_modifier_held()` (8 slot reads);
  2. `which = resync.plan(notes, held, &|| env.now())`;
  3. if `which` is not empty, `keys.resync_modifiers(env, which)` and return true.
- `Resync::plan`: fold `notes` (a different value bumps `hiding`; `!=`, so a smaller value counts too); if
  `input_away`, read nothing (D6); if `hiding != resynced`, plan `ALL`, set `resynced = hiding` and `last_read_at`;
  else the periodic rule (D7); else plan `owed`. Then leave out `fresh` (D5): `owed' = (owed ∪ planned) − read`,
  `fresh = NONE`.
- `MainState::resync_if_due(hook)`: false while `!hook.is_enabled()`; else `notes = hidden_input_notes()` (commit 9:
  `.wrapping_add(hook.health().unseen_input())`) and `self.engine.resync_if_due(&HookEnv::new(&self.foreground), notes)`.
- `MainThread::drain` (`main_thread.rs:725-744`): after its loop ends (requests, signals and due timers served), one
  `with_main(|m| m.resync_if_due(hook))`. Under the borrow: the reads only, no action, no post, no pump (principle 5).

**The sources** (the E-numbers name tests and mutations).

| # | Source | Where it moves the count | Thread | How the drain comes |
|---|---|---|---|---|
| E0 | Daemon start | `Resync::new()` (`hiding` 1, `resynced` 0) | main | first drain of `run` |
| E1 | Foreground change | `Engine::on_foreground_changed` | main (WinEvent) | `wake_this_thread()` after `with_main` |
| E2 | `swch off`/`on`, hook reinstall | `reset` bumps; `resync_from_os` reads every code; `read_everything` | main (drain) | the reset is the read |
| E3 | Input left / came back | `reset(InputLeft)` → `input_left`; `answer_input_signals` → `MainState::input_returned` → `Engine::input_returned`; or the next decided event (D6) | main (drain, callback) | the drain that serves the signal; the drain after the event |
| E3u | Desktop switch to an unknown destination | `note_unknown_desktop_switch` (`main_thread.rs:165-172`) calls `note_hidden_input()` | main (WinEvent) | its existing post |
| E4 | Late keyboard callback | `note_late()` at the callback's exit (D8) | main (callback) | the drain after the pump |
| E5 | Awake stretch ≥ 17 ms | `note_if_awake_too_long` before the wait, then `continue` (D8) | main (loop) | at once |
| E6 | Raw key events no callback explains | `HealthState::note_unseen_input` + `RequestReinstall::hidden_input` (D8) | hook-watchdog | `MAIN_WAKE` |
| E7 | A modifier held ≥ 1 s | `Resync::plan` periodic rule (D7) | main (drain) | the drain after any message |
| E8 | A callback refused by `with_main` (key event, injected-modifier tracking, foreground change) | `note_hidden_input()` in the refusal branch (`main_thread.rs:218-224`) | main | the drain after the pump |

`ResetReason::Reloaded` moves nothing: a reload hides no event, and its reset deliberately reads no OS state
(`mod.rs:637-641`).

**Every way Windows' modifier state can change without the engine seeing the event** (critique M2 included):

| # | Way | Evidence | What makes the main thread read | Left open (row 88) |
|---|---|---|---|---|
| W1 | Focus in a higher-integrity window (UIPI) | "a non-elevated program cannot spy on input intended for an elevated program" ([AutoHotkey FAQ](https://www.autohotkey.com/docs/v2/FAQ.htm)); `GetAsyncKeyState` returns zero when "UI Privilege Isolation (UIPI) prevents the calling thread from accessing the foreground thread" (GetAsyncKeyState) | E1 on entering (reads zeros; the hook sees no key there) and on leaving (reads the truth) | R-E4: focus moves to a child window of a higher-integrity process with no foreground change |
| W2 | Another desktop (lock, Ctrl+Alt+Del, UAC) | "The current desktop is not the active desktop" makes `GetAsyncKeyState` return zero (GetAsyncKeyState); `EVENT_SYSTEM_DESKTOPSWITCH` ([Event Constants](https://learn.microsoft.com/en-us/windows/win32/winauto/event-constants)) | E3 (no read while away, D6), E3u | R-E3: an episode with no signal at all |
| W3 | Remapping off | `keyboard_hook_callback.rs:368-373` | E2 at `swch on` | none |
| W4 | Hook removed | "on Windows 7 and later, the hook is silently removed without being called" ([LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)) | E2 at the reinstall | none |
| W5 | A call that timed out, hook kept | "If the hook procedure times out, the system passes the message to the next hook" (LowLevelKeyboardProc) | E4 (age + run ≥ 17 ms) | R-E2: `LowLevelHooksTimeout` below about 34 ms |
| W6 | Calls dropped while main was not pumping | from code; not documented | E4 (the next event waited), E5, E6 | R-E1: a stall outside every measured span; R-M1 (D8) |
| W7 | A refused callback | `main_thread.rs:216-229` | E8 | none |
| W8 | The daemon's own injection, a short `SendInput` count included | `SendInput` return value (above); the inserted events reach the hook with the sentinel | Not needed: D2 tracks what was inserted | none |
| W9 | Another program's injection; a generic code (`VK_SHIFT`) that Windows applies to a side | "Testing LLKHF_INJECTED (bit 4) will tell you whether the event was injected" ([KBDLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct)); known issue 78 | Not needed: decided and tracked like a physical event; the generic slot holds the family while it is held (D3) | R-G: a read while the generic code is held finds its side held, and that side stays held for the engine after the generic key-up until E7 or the next count change |
| W10 | A later hook swallows the event | "it may return a nonzero value to prevent the system from passing the message to the rest of the hook chain" (LowLevelKeyboardProc) | Not needed: Windows does not apply it either | none |
| W11 | The async state of a key just decided lags | "the callback function is called before the asynchronous state of the key is updated" (LowLevelKeyboardProc) | D5: a key touched since the last plan is not read now but at the next drain | R-L: a lag longer than one pump-and-drain cycle (a slow later hook); the next event is then late (E4) |
| W12 | A modifier held when the daemon starts | — | E0 | none |

**Alternatives rejected.** An adapter-only epoch passed with each event (the engine's own sources would be untestable
in scenarios). An `Env` method for the notes (ten `impl Env` to touch; the drain passes the value). `GetLastInputInfo`
as a universal detector (moves with every mouse event; secure-desktop coverage unverified).

**Cost.** Key path: none. Main, per drain: one TLS read (+ one `Acquire` load from commit 9), 8 slot reads, a
compare. Per read: up to 8 `GetAsyncKeyState`, about 3 µs (R18), between events.

### D5: A modifier touched since the last plan is read at the next drain

**What.** `Resync::event` ORs `ModifierBits::of_event(vk)` into `fresh` for every decided or tracked event (both bits
for a generic code). A plan leaves `fresh` out and records it in `owed`; the next plan reads `owed` minus that plan's
`fresh`. Every plan clears `fresh`.

**Why.** "the callback function is called before the asynchronous state of the key is updated" (LowLevelKeyboardProc).
A drain can run microseconds after a callback returned, before Windows updated that key; and the daemon's own
`SendInput` inside `carry_out` re-enters the hook (`keyboard_hook_callback.rs:85-96`) so several keys can be
touched in one stretch. The flag of a key touched since the last plan comes from its own latest event, the freshest
information; reading it now could take a stale state. The owed read catches a hidden event of that key that came
later. No claim about the order in which Windows finishes events is needed **(unverified; not relied on)**.

**Alternatives rejected.** A settle time (a timer per note and a bound nobody documents). Skipping only the latest
event's key (misses the re-entrant case). Reading it anyway (a lagging LShift key-down read as up turns its next
repeat into a press).

**Cost.** One OR per event on the key path; at most one more partial read at the next drain.

### D6: No read while input is on another desktop

**What.** `reset(InputLeft)` keeps `release_all` (`keys.rs:144-151`, no OS read) and calls `Resync::input_left`
(`input_away`, bump). `plan` reads nothing while `input_away`. `Engine::input_returned` (called from
`MainState::input_returned`, `main_thread.rs:288-293`, for `SessionUnlocked` and `DesktopSwitchedBack`) and the next
decided key event clear it; the next drain then reads.

**Why.**

- CLAUDE.md principle 6: "a lost-input reset reads no OS state". `GetAsyncKeyState` fails "The current desktop is
  not the active desktop", and a read before the switch completes takes keys about to lose their key-ups as held.
- A key event the hook sees proves that input is back: "once input is on another desktop the hook sees nothing"
  (`mod.rs:413-415`, 2.3 ruling R2) **(unverified; manual checks M2, M3)**. So a missing return signal (known issue
  79) delays the read by one decided event, not forever.

**Alternatives rejected.** Bumping at the away signal and reading at once (principle 6). Reading only at the return
signal (a missing signal would block every later read).

**Cost.** One store per decided event (in D1's cost).

### D7: A slow read while a modifier is held: once a second, on the drains, no timer

**What.** In `plan`, when nothing else is due and `held` (an unswallowed sided modifier is active): `held_since` is
set at the first such plan; a full read is planned when `now >= max(held_since, last_read_at) + RESYNC_PERIOD`
(1 s). When nothing is held, `held_since = None` and the clock is not read.

**Why (ruling W2: "the architect decides").** A modifier key-up the hook missed with no source noticing it leaves a
phantom-held modifier, the one stale state nothing else repairs while the user does not touch that key: every key then
matches with an extra family, so `CapsLock` stops switching the layout under exact matching (owner decision). The
condition "a modifier held for the engine" is exactly the phantom's symptom. A phantom-released modifier (Windows has
it down, the engine does not) heals at its own key-up or auto-repeat, which the hook sees.

**Why on the drains.** The drain follows every pump (`main_thread.rs:684-719`), so the rule adds no wake: idle costs
nothing; a phantom costs at most the first key event after the period, which is decided before the drain that
heals it. A timer would heal before that event but would set and cancel `SetWaitableTimer` around every shortcut.

**Alternatives rejected.** No slow read (a phantom lasts until the next foreground change). A timer at 1 s while held
(two kernel calls per shortcut, wakes while Shift is held). A shorter period (more reads during long Shift holds for no
gain: shortcuts last well under 1 s).

**Cost.** While a modifier is held ≥ 1 s: one read per second (about 3 µs) in a drain right after a key event; a key
event that arrives in the same few microseconds waits for it. Idle: nothing.

### D8: The adapter's notes: E4, E5, E6, E8, E3u

**What.**

- `HIDDEN_INPUT: Cell<u64>` next to `PENDING_SIGNALS` (`main_thread.rs:103-112`); `note_hidden_input()` is
  `try_with(|n| n.set(n.get().wrapping_add(1)))`, no post; `hidden_input_notes()`.
- **E8** (commit 5): `with_main`'s refusal branch (`:218-224`) calls `note_hidden_input()` after its counter bump.
  One place covers a refused key event, a refused injected-modifier tracking and a refused foreground change.
- **E3u** (commit 5): `note_unknown_desktop_switch` bumps the note before its existing post.
- **E4** (commit 8), in `keyboard_hook_callback` (`:375-401`):
  - at entry, after `event` (`:382`): copy `event.time` and `event.flags.contains(LLKHF_INJECTED)`;
  - after `carry_out`: `let ran = started.elapsed(); counters::record_callback(ran);` (no extra clock read);
  - then `let age = if injected { 0 } else { event_age_ms(GetTickCount(), stamp) };` with
    `// SAFETY: GetTickCount takes no argument, reads the system tick count and cannot fail`;
  - `if late_callback(age, ran) { note_late() }`, where `note_late()` is `note_hidden_input();
    counters::bump(Counter::HookLate);`. No `log::`, no post (principle 1).
  - `event_age_ms`: `now.wrapping_sub(stamp)`, and 0 if the difference is 2^31 or more (a stamp ahead of the clock).
- **E5** (commit 8), in `MainThread::run`: `let mut awake_since = Instant::now()` before the loop;
  `awake_since = Instant::now()` right after `MsgWaitForMultipleObjectsEx` returns; right before it,
  `let now = Instant::now(); if note_if_awake_too_long(awake_since, now) { awake_since = now; continue; }`.
  `note_if_awake_too_long` notes and bumps `HookLate` when `now − since >= HOOK_LATE`. The `continue` goes back to
  the drain, so nothing is dispatched between the note and the read (critique M1).
- **E6** (commit 9): `Watchdog::tick` (`decision.rs:171-222`) records
  `unseen_at_last_tick = arrived.saturating_sub(credit_before.saturating_add(answered))` (`credit_before` taken at
  entry). `Ticker::pass` (`watchdog/mod.rs:262-266`) after a tick with a value above 0 calls
  `context.health.note_unseen_input()` (`fetch_add(1, AcqRel)`) and `context.requests.hidden_input()`;
  `MainThreadHandle` implements it with `wake_for_hidden_input()`, which posts `MAIN_WAKE` (`:574-583`).
  `unseen_input()` is a `load(Acquire)` on main; the counter publishes no other data (principle 5).

**Why 17 ms.** "The resolution of the GetTickCount function is limited to the resolution of the system timer, which is
typically in the range of 10 milliseconds to 16 milliseconds" ([GetTickCount](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount)):
one tick boundary shows as 15-16 ms, so 17 ms needs two, and a prompt event never counts. Detection holds for any
timeout of about 34 ms or more; the default is 300 ms per AutoHotkey ([#HotIfTimeout](https://www.autohotkey.com/docs/v2/lib/_HotIfTimeout.htm)),
and "The maximum timeout value the system allows is 1000 milliseconds" (LowLevelKeyboardProc).

**Why one exit check.** The read runs in the drain after the callback anyway (D4), so an entry note decides nothing
earlier. The age at exit covers the queue wait and the run.

**The stamp's base (ruling W4): (unverified).** `KBDLLHOOKSTRUCT.time` is "The time stamp for this message,
equivalent to what GetMessageTime would return for this message" (KBDLLHOOKSTRUCT); `GetMessageTime` gives "the
elapsed time, in milliseconds, from the time the system was started to the time the message was created"
([GetMessageTime](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagetime)), whose
remarks prescribe subtracting "ignoring overflow"; `GetTickCount` returns "the number of milliseconds that have
elapsed since the system was started". That both read the same counter is not stated; manual check M8 settles it.

**An injector's own time cannot flood the trigger.** "If this parameter is zero, the system will provide its own time
stamp" ([KEYBDINPUT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput)): a nonzero
`time` is the injector's. An `LLKHF_INJECTED` event's age is 0; only its run time counts (QPC). Remote-input tools
inject, so they are covered too.

**A false positive** costs one read between events and one `hook_late` count, never a decision (ruling W4). Hooks
installed after the daemon's run first and can make events arrive late: the same.

**Alternatives rejected.** `started.elapsed()` alone (misses the queue wait). Reading `LowLevelHooksTimeout` (R-E2 now
risks a miss, not a stuck key). A post per note from the callback (a Win32 call on the key path; the drain follows the
pump).

**Cost on the key path.** Per callback while remapping is on: two field copies, and for a physical event one
`GetTickCount` (about 2-5 ns, a user-mode read **(unverified)**, R17t; stop rule Q4) and two compares. Late events
only: a TLS add and one RMW. Main loop: two `Instant` reads per pass (about 40-60 ns), outside callbacks.

### D9: `hook_late` and the counters line level

**What.** `Counter::HookLate`, `hook_late`: `SLOTS` 11 → 12 (`counters.rs:17`), `ALL` after
`WatchdogDesktopCheckFailed` (`:66-78`), `Display` (`:82-96`), slot 11 (`:130-145`). Doc: "A keyboard callback ended 17
ms or more after its key event, or the main thread spent 17 ms or more away from its messages: Windows may have
applied a key event without the daemon, so the main thread reads the modifiers again (roadmap item 2.5)."
`Counter::warns(self) -> bool`, every arm spelled out, `false` only for `HookLate`. `summary` returns
`Option<(log::Level, String)>`: `None` as today; `Warn` if a warning counter is not 0 or the slow-callback gauge fired;
else `Info`. `writer.rs:229-234` writes at that level. The line keeps one format, all 12 counters.

**Why.** Ruling W4: "Count it without making the minute line a warning by itself". One line keeps log readers' greps.

**Cost.** None on the key path beyond the bump in D8.

### D10: A swallowed no-record repeat keeps its key-up from Windows unless Windows has the key down

**What.** `LastDown::RepeatSuppressed`: the engine never saw this press's key-down, and it swallowed one of its
auto-repeats. In the repeat branch (`mod.rs:839-845`), a repeat with no record that the bindings suppress calls
`mark_repeat_suppressed`. The state:

- consumed at the key-up like `Suppressed`: `take_blocked` returns true, so the key-up is swallowed unless Windows has
  the key down (the witness, `mod.rs:896`);
- kept like `Suppressed` by `resync_from_os` (`keys.rs:138`) and `release_all`;
- otherwise like `Unknown`: later repeats are decided by the bindings, a passing one marks `Passed`; `is_blocked` and
  `key_down_swallowed` are false (KS-14 does not skip it; it counts for the menu mask as a key Windows may have down).

**Why.** D11's "no lone key-up" needs it. A sided modifier can be held for the engine with no record (a read found it
down). If its key-up then goes unseen with nothing noticed, the next physical press reads as a repeat with no record;
a binding that swallows it (the `Alt+Shift` blocker) leaves no record on master, so its key-up passes while Windows has
the key up: a lone key-up. The window exists on master too: its per-event sync skips the event's own key
(`mod.rs:522`); commit 3's scenario reproduces it on `50b56a8`. Marking it `Suppressed` instead would be wrong when
Windows does have the key down (a read found it held): KS-14 would then freeze its flag and the mask would ignore it.

**Alternatives rejected.** `mark_blocked` (above). Reading the OS at that repeat (a read of the event's own key on the
key path, beyond the witness).

**Cost.** None: one store on that rare path.

### D11: The failure mode, traced (ruling W2)

The modifier flags are `KeyState::active` (`keys.rs:79`). They are read only by:

1. matching: `HeldKeys::is_down`, `modifiers`, `all_held` (`keys.rs:242-261` → `matching.rs:29-102`; the winner, the
   sequences, the auto-blocker);
2. repeat detection: `press` returns the old flag (`keys.rs:160-165`);
3. the menu mask: `alt_or_win_down_in_windows` (`keys.rs:234-239`);
4. `mark_completed`'s "held" test (`keys.rs:185-192`).

Suppression of a key-up reads no flag: `on_key_up` (`mod.rs:858-899`) uses the record (`take_blocked`,
`is_passed`), the gesture's `gesture_bound` and the witness.

**Lemma 1: no key sticks.** A seen key-up the engine would swallow passes when Windows has the key down
(`mod.rs:896`, kept by W1); an unseen key-up reaches Windows. So once the key is physically up, Windows has it up. No
flag enters.

**Lemma 2: no key-up reaches Windows alone.** A seen key-up passes only if (i) its record is `Passed`, (ii) it has no
record and no gesture, or (iii) Windows has it down. (iii) is not lone. (i): `Passed` comes from a key-down that
reached Windows (`mod.rs:846-853`), a passing no-record repeat (`:843-845`) or a key `resync_from_os` found down
(`keys.rs:136-137`). A hidden key-up after it can only be followed, before this key-up, by a key-down: seen (a
"repeat" for a stale flag, `is_passed` → passes → Windows down) or unseen (Windows down). (ii): after D10 every seen
key-down leaves a record, so no record at a key-up means no seen key-down since the last record was consumed or
reset; then the key-down Windows applied was unseen, and Windows has the key down. The drain's reads write flags only,
never records (`resync_modifiers`). No flag enters.

**The KS-14 rule.** Every new read (`resync_modifiers`, `track_injected`) skips a modifier whose key-down was
swallowed (`key_down_swallowed`, `keys.rs:153-156`), as `sync_modifiers` did (`:105`). Such a key's flag changes only
by its own events and `release_all`; master's behaviour for it is unchanged.

**What a stale flag can do until the next read** (row 88; all bounded by E0-E8 or D7):

- a binding misses or fires on the wrong modifier family (exact matching);
- a key-down of that modifier counts as an auto-repeat, so its own first-press bindings do not act (known issue 63's
  kind), or a repeat counts as a press (known issue 76's kind);
- the menu mask key is sent when not needed (harmless) or not sent when Windows has Alt or Win down (Windows may see a
  lone Alt or Win press: the menu bar or Start);
- a completed chord or sequence marks a phantom-held key, so its next press is swallowed whole (no lone key-up: the
  record and the witness hold).

**Residuals** (row 88): R-E1, R-E2, R-E3, R-E4 (W-table), R-M1 (D8: events dispatched in the same awake stretch after
a stall are decided before the drain), R-G (W9), R-L (W11).

### D12: The watchdog's first wake after idle starts the ticks only (ruling W3)

**Q6 of item 2.4, as far as the evidence goes.** Microsoft does not document whether raw input is produced before or
after the LL hook chain. LowLevelKeyboardProc: "The system calls this function every time a new keyboard input event
is about to be posted into a thread input queue"; the [Raw Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input)
is silent on hooks. Prior art:

| Source | Claim | How read |
|---|---|---|
| [input-leap #1458](https://github.com/input-leap/input-leap/discussions/1458) | "WH_KEYBOARD_LL -> Raw Input -> [OS Update] -> WH_KEYBOARD" | fetched |
| [GameDev.net 2009](https://gamedev.net/forums/topic/550264-dinput-and-global-key-hooks-vistawin7/) | "WH_KEYBOARD_LL does indeed block Raw Input / WM_INPUT messages"; disputed in the thread | search summary (403) |
| [PureBasic forum](https://www.purebasic.fr/english/viewtopic.php?t=66655) | "Raw Input callback will never receive any messages" with an LL hook that blocks | search summary |
| [AutoHotkey t=6073](https://www.autohotkey.com/boards/viewtopic.php?t=6073) | "if you block keystrokes via a keyboard hook, you won't get the WM_INPUT messages" | search summary |
| [NotEnoughHotkeys](https://github.com/VollRahm/NotEnoughHotkeys) | Win, Alt+Tab and lock keys "are sent before RawInput gets anything" | fetched |
| [Muratori, "Five-Second Stall"](https://caseymuratori.com/blog_0006) | the process's own LL hook "never got called" while its raw-input window had focus; `RIDEV_NOHOTKEYS` fixed it | fetched; a same-process, focused-window effect |
| [AltSnap PR #773](https://github.com/RamonUnch/AltSnap/pull/773) | the hook stops "as soon as the Menu has focus" if raw input is registered | fetched; same effect |

**Verdict (unverified; probable):** raw input after the chain, none for a swallowed key. The burst's first raw event
can still meet the next event of the same HID report: the class driver hands the system a range of packets at once
(`KeyboardClassServiceCallback(..., InputDataStart, InputDataEnd, ...)`,
[MS Learn](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/kbdmou/nc-kbdmou-pservice_callback_routine)),
**(unverified)** timing. So R16 keeps its construction and is gated. The 2.4 diagnostics (`bench-dev-diag.md:5-30`)
put the cost in the woken thread's first pass: not the decision (D-B 53.6 µs), not priority (R16/high 58.5 µs), not a
cold thread (D-D 79.7 µs), lasting past 200 µs (D-C 16.5 µs); its user32 calls are `pump` (`watchdog/mod.rs:786`),
`drain` (`:1014`) and the next wait.

**What.**

- `Watchdog::raw_pending(now)` (pure, `decision.rs`): when not ticking, `next_tick = now + TICK`, `quiet_since = now`,
  `credit = 0`; `last_raw_at` is left for the first tick's drain.
- `Ticker::woke_idle(now, context)`: `stats.wakes += 1`, `raw_pending(now)`, `arm(now, context)`.
- `run_loop` (`:703-749`) keeps the wait's mask and takes the deferred path when `woke == WAIT_OBJECT_0 + 1`, the mask
  contains `source.batched_wake()` and `context.health.hook_installed()`: `woke_idle`, `drained =
  Drained::default()`, `continue`, with no pump and no drain. "WAIT_OBJECT_0 + nCount — New input of the type
  specified in the dwWakeMask parameter is available in the thread's input queue"
  ([MsgWaitForMultipleObjectsEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex)).
- No spin: the next wait is a ticking wait without the batched bit; with `MWMO_INPUTAVAILABLE` a `WM_QUIT`, a sent
  message or a system event ends it, and that pass pumps. Two deferred wakes in a row are impossible. The raw input
  stays queued for the first tick (2.4 assumption, pinned by
  `a_message_wait_leaves_out_a_pending_message_its_wake_mask_does_not_name`, `:2526`; manual check 47).
- The synthetic source's batched bit is `QS_POSTMESSAGE`, which `WM_QUIT` shares: an idle synthetic watchdog stops at
  its first tick (≤ 250 ms, inside `STOP_TIMEOUT`, `:430`). The keyboard source stops at once.
- 2.4 rules kept: I3 (deferred only with the hook installed); the silence starts at the wake; the first count lands in
  the first tick's interval and becomes evidence from the second tick.
- Loop-model expectations that move by one tick (meaning unchanged, Q6): `a_batch_read_at_a_tick_counts_all_its_events`
  (`:1631`) `(250, 8)` → `(250, 9)`; `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request` (`:1813`) last
  read 749 → 750; `the_first_raw_key_event_after_idle_wakes_the_watchdog` (`:1607`) "no pass in `2250..5000`" →
  `2500..5000`. `Drive` (`:1354-1519`) gains `wake(ms)`: `woke_idle`, a counted pass, the timer calls, no read.

**Alternatives rejected.** A timer-only idle poll at 1 s (600 wakes per 10 idle minutes against check 39's W ≤ 100;
Q3 fallback only). A stop event in the handle array (changes 2.4's shutdown protocol). Priority changes (R16/high =
R16). Raw input on main (2.4 D1, R15).

**Cost (critique M3).** At the first key after idle the watchdog makes one wait return, one `SetWaitableTimer` and one
wait entry, and reads nothing until the tick. `last_raw_at` is now set one tick later, so ticking ends one tick later:
**one more wake per burst**, about 11 for an isolated shortcut instead of 10 (`errors.md:215`), about 1100 for 100
isolated Ctrl+C. The module Cost doc (`watchdog/mod.rs:23-42`) says so. No idle wake is added.

### D13: Bench rows and comparison rules

| Row | What | n | Rule |
|---|---|---|---|
| R12 | unchanged construction | 2000 | **New bound from probe D-R12n:** HEAD R12 p50 ≤ D-R12n p50 + R17t p50 + 100 ns; HEAD p99 ≤ 1.5 × D-R12n p99 + 2 µs |
| R4 | unchanged | 10000 | HEAD R4 p50 ≤ D-R4n p50 + R17t p50 + 100 ns (same probe run) |
| R1 | unchanged (engine with `BenchEnv`, whose reads are a `RefCell` lookup) | 50000 | HEAD R1 mean ≤ parent R1 mean + 0.005 µs; the mean (3 decimals) shows the 11 lookups gone, p50 sits at the 0.1 µs resolution |
| R11 | unchanged; W1 | 2000 | "unchanged" = the witness term: HEAD (R11 − R12) p50 ≤ parent (R11 − R12) p50 + 0.2 µs; R11 itself falls with R12 |
| R16 | unchanged construction; doc: "the event right after the burst's first raw event: the key's own callback if raw input comes before the hook call, the next event of the same report if after" | 1000 | **Provisional:** HEAD R16 p50 ≤ min(HEAD R12 p50 + 20 µs, 0.5 × parent R16 p50); p99 reported |
| R17t (new) | `GetTickCount()`: 1000 calls / 1000 per sample | 2000 | evidence; stop rule Q4 |
| R17a (new) | `GetAsyncKeyState(VK_F13)` (read only), the same way | 2000 | evidence: 11 × R17a ≈ the removed cost |
| R18 (new) | the main thread's read: untimed `note_hidden_input()`, timed `with_main(\|m\| m.resync_if_due(get_hook()))` on the bench thread as main | 2000 | evidence; sanity bound p50 ≤ 8 × R17a p50 + 0.5 µs |
| R2, R3, R5-R10, R13-R15 | unchanged | | existing rules (2.2 D10, 2.3 D14, 2.4; `CLAUDE.md:69-73`); R5, R9, R10 fall with R12; R7, R13 carry E5's two `Instant` reads per pass and the drain's check |

**Probes after commit 2** (work tree only, uncommitted, as 2.4 did), into
`D:/tmp/ks-orch/p2-2.5-latency-trim/bench-dev-diag.md`:

1. **D-R12n.** Remove the line `self.keys.sync_modifiers(env, Some(event.vk));` (`src/engine/mod.rs:522`), run the
   bench once in release with the line and once without, back to back; record R1, R4, R11, R12 of both (the run
   without the line gives D-R1n, D-R4n, D-R11n, D-R12n); restore the line. **Stop rule:** if D-R12n p50 > 0.5 × R12
   p50, stop after commit 2 and report: the reads are not the cost the ruling names. Planning estimate: 0.5-1.0 µs
   (R1 0.1 µs; 4.7 − 11 × about 0.37 µs).
2. **D-R16d.** A bench-local helper thread: its queue and a `WaitableTimer`; it waits with
   `MsgWaitForMultipleObjectsEx(&[timer], INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)`; on input it reads
   `Instant::now()`, sets the timer for 5 ms and waits again with the posted bit left out; at the timer it pumps
   everything and goes idle. Each sample posts one message to it, times `call_keyboard_callback(VK_A, WM_KEYDOWN)` at
   once, calls the key-up untimed, spins 10 ms; n = 1000. **Stop rule:** if D-R16d p50 > R12 p50 + 20 µs (same run),
   stop after commit 9 and report (Q3).
3. **After commit 11** the developer runs R16 at HEAD; if it misses the provisional bound, stop and report the numbers
   (ruling W3).

**Runs.** The developer: once at commit 2, once at HEAD (`impl.md`). The tester: three runs on `master` and on HEAD
(`tester.md` §4b): median of three p50s, largest p99. **The results analyst states** (rulings, Latency): R12, R1 and R4
before and after; R11's witness term unchanged; R16 before and after; R18 and how often it runs (per count change,
and at most once a second while a modifier is held ≥ 1 s).

### D14: The Phase 2 book-sync leftovers

- `CHANGELOG.md:744-745`: "a `Hold` runs them on its timer thread as before and queues the others" → "a `Hold`, and a
  `Tap` that waits for its DoubleTap window, runs them on the main thread when its gesture timer fires (roadmap item
  2.2), and queues the others".
- `CHANGELOG.md:116`: "(a Hold whose timer thread could not be started, so that it never fired)" → "(a Hold whose
  gesture timer could not be set, so that it never fired)".
- `src/platform/timers.rs:1-3`: "the action executor's `Delay` steps today, the gesture timers of roadmap item 2.2
  later" → "the action executor's `Delay` steps (roadmap item 2.1), the main thread's gesture timers (item 2.2) and the
  hook watchdog's ticks (item 2.4)".

### D15: The scenario harness models the drain, noticed and unnoticed hidden events

**What** (`src/engine/testing.rs`; commit 5, adjusted in 7):

- `Scenario` gains `notes: u64` and `fn drain(&mut self)`: when remapping is on,
  `self.engine.resync_if_due(&self.env, self.notes)`. It runs after every step at which the daemon's main thread would
  reach its drain: a key event while on, `foreground_changed`, every input signal, `remapping_on`, `hook_reinstalled`,
  `reload`, `advance_ms` that fired a timer, `hidden_input_noted`, and the noticed unseen steps. Not after
  `up_unnoticed`, `down_unnoticed`, `advance_ms_stalled`, `lose_timers`, `remapping_off`.
- `up_unseen`/`down_unseen` (`:346-380`) add a note and drain: "an event the hook did not see, which a source noticed
  (roadmap item 2.5)".
- New `up_unnoticed`/`down_unnoticed`: the same OS change, no note, no drain.
- New `hidden_input_noted()`: `notes += 1`, drain, a step. It stands for E3u, E4, E5, E6, E8.
- `foreground_changed` (`:382-395`) calls `on_foreground_changed()` and drains. `input_signal` (`:427-446`) resets as
  today, calls `engine.input_returned()` for a return signal (as `answer_input_signals` does), drains.
- Module docs (`:1-71`) say all of this.
- `properties.rs` (commit 7): new `Mode::Unnoticed`, the Unlock steps with `down_unnoticed`/`up_unnoticed` in place of
  the unseen steps, checking `check_nothing_stuck` and `check_no_lone_key_up` (D11). `presses_unseen`, `loses_key_ups`
  and the signal branch (`:292`) include it; the existing modes draw exactly the same numbers, so their seeds keep
  their scripts. New test
  `no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed`.

**Why.** The daemon notices almost every hidden event before the next key (W-table), so the existing scenarios and
properties keep their meaning with noticed unseen steps (an eager read, like master's sync). Unnoticed steps pin what
row 88 declares, and the new mode tests D11's lemmas on random scripts.

**Alternatives rejected.** Generating a source step before each unseen step (changes every Unseen and Unlock seed).

## Win32 API choices

| Purpose | `windows` path | Feature (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| Event age (E4) | `windows::Win32::System::SystemInformation::GetTickCount() -> u32` (`pub unsafe fn`, windows-0.62.2 `SystemInformation/mod.rs:194`) | `Win32_System_SystemInformation`: **absent, add** to `Cargo.toml:41-58` (same crate; `Cargo.lock` unchanged) | main (keyboard callback, physical events only); bench | none; wraps after 49.7 days: `wrapping_sub`, signed rule | https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount |
| The event's stamp and injected flag | `KBDLLHOOKSTRUCT { flags: KBDLLHOOKSTRUCT_FLAGS, time: u32 }` (`WindowsAndMessaging/mod.rs:4251-4255`), `LLKHF_INJECTED` (`:4409`), `KBDLLHOOKSTRUCT_FLAGS::contains` (`:4262`) | `Win32_UI_WindowsAndMessaging` (yes) | main | — | https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagetime, https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput |
| The drain's read | `windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState` (existing, `HookEnv::os_key_down`, `keyboard_hook_callback.rs:178-184`) | `Win32_UI_Input_KeyboardAndMouse` (yes) | main, in the drain under `with_main` (a read, no pump) | "The return value is zero if the call fails": not the active desktop, UIPI, no hook/journal access; zero reads as up (D6, W1) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate |
| Wake after a foreground change; the watchdog's wake | `PostThreadMessageW` (existing: `wake_this_thread` `main_thread.rs:182-190`, `MainThreadHandle::wake` `:574-583`) | `Win32_UI_WindowsAndMessaging` (yes) | main (WinEvent, outside the borrow); `hook-watchdog` → main | ignored as today: a lost wake loses no note, the next drain folds the count; "a limit of 10,000 posted messages per message queue": one post per foreground change or gap tick | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew |
| Watchdog's wait and "input" result | `MsgWaitForMultipleObjectsEx` (existing, `watchdog/mod.rs:714-721`), `WAIT_OBJECT_0 + 1` | yes | `hook-watchdog` | `WAIT_FAILED` as today | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex |
| First tick after a deferred wake | `platform::timers::WaitableTimer::set` → `SetWaitableTimer` (existing) | `Win32_System_Threading` (yes) | `hook-watchdog` | counted and retried as today (`watchdog/mod.rs:334-347`) | https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer |
| W8 evidence (no new call) | `SendInput` | yes | — | short count | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput |

**Unverified, and what settles it:**

| # | Claim | Settled by |
|---|---|---|
| U1 | `KBDLLHOOKSTRUCT.time` and `GetTickCount` share a base | M8 (`hook_late` stays 0 in normal typing); M4 (it counts a held callback) |
| U2 | `GetTickCount` is a user-mode read | R17t; stop rule Q4 |
| U3 | The LL hook sees no events while input is on another desktop | M2, M3 (roadmap 36, 28) |
| U4 | An injected generic `VK_SHIFT` reaches the hook under `VK_SHIFT` and is applied to a side | known issue 78 (2.3); not needed for no-stuck (D11) |
| U5 | Raw input comes after the hook chain (Q6) | M5 (roadmap 43, 44) |
| U6 | The keys of one HID report are handled back to back | not needed: R16 keeps its construction |

**Elevation and UIPI:** W1 and W2; `GetAsyncKeyState` returns zero under UIPI and on an inactive desktop, read as up.
No pipe, token, registry or hook-registration change; nothing injects input.

## Thread map

| Code path | Thread | Change |
|---|---|---|
| `keyboard_hook_callback`, physical event | main (inside `PeekMessageW`) | − 11 reads; + two copies, one `GetTickCount` at exit, a note when late |
| `keyboard_hook_callback`, sentinel modifier event | main | + `with_main`, one flag store (D2) |
| `foreground_changed` WinEvent | main | engine bump instead of 11 reads; + `wake_this_thread()` outside the borrow |
| `desktop_switched` WinEvent | main | an unknown destination also notes (E3u) |
| `with_main` refusal | main | + TLS note (E8) |
| `MainThread::drain` | main | + the read step after requests, signals and timers |
| `answer_input_signals` | main (drain) | + `engine.input_returned()` via `MainState::input_returned` |
| `MainThread::run` | main | + E5 (two `Instant` reads per pass; note + `continue`) |
| `Ticker::pass` | `hook-watchdog` | + E6 RMW and post |
| `run_loop` | `hook-watchdog` | D12 deferral |
| `report_counters` | `log-writer` | the line's level |
| Bench R17t, R17a, R18 | bench thread (as main for R18) | reads only |

No install, uninstall or timer moves; every hook stays on main; no new thread, lock, `static mut` or `Relaxed`.

## Touch set

| File | Change | Commit |
|---|---|---|
| `src/platform/timers.rs` | doc `:1-3` (D14) | 1 |
| `CHANGELOG.md` | `:116`, `:744-745` (1); Added R17t/R17a (2); Changed D10 (4); Added R18 (5); Changed modifier resync + generic slots (7, list extended in 8 and 9); Added `hook_late`, Changed line level, example line `:120` (8); Changed watchdog first wake, R16 gated (11) | 1-11 |
| `Cargo.toml` | feature `Win32_System_SystemInformation` | 2 |
| `src/bench.rs` | `call_keyboard_callback` `time: GetTickCount()` (2); R17t, R17a (2); R18 (5); `Unanswered` unchanged (default `hidden_input`); docs `:20-82` | 2, 5, 11 |
| `src/engine/keys.rs` | `RepeatSuppressed`, `mark_repeat_suppressed`, `take_blocked` (4); `SIDED` codes, `resync_modifiers`, `unswallowed_modifier_held` (5); delete `sync_modifiers`, generic codes out of `resync_from_os`, `track_injected`, module docs `:1-17`, `:25-29`, `:44-47`; tests that call `sync_modifiers` (`:286`, `:316`, `:484-492`, `:521`, `:568`, `:681-705`, commit 4's mask test) move to `resync_modifiers` with sided expectations; `:600-608` drops the `VK_SHIFT` expectation (7) | 4, 5, 7 |
| `src/engine/resync.rs` (new) | D4-D7 state and plan; unit tests | 5 |
| `src/engine/mod.rs` | rule 5 code `:839-845` and docs `:185-195` (4); `resync` field, `resync_if_due`, `input_returned`, `on_foreground_changed()`, `reset` bookkeeping, `mod resync` (5); `on_key` without the sync, `track_injected`, `is_tracked_modifier`, docs `:39-61`, `:113-121`, `:197-220` (7); unit tests | 4, 5, 7 |
| `src/engine/env.rs` | `os_key_down` docs `:34-52` | 7 |
| `src/engine/matching.rs` | docs `:3-8`, `:33-36`, `:88-92` (generic codes from events) | 7 |
| `src/engine/testing.rs` | D15 | 5, 7 |
| `src/engine/scenarios.rs` | commit 3's scenario (3; `up_unnoticed` in 5); S1-S9 (7); comments of `:189-203`, `:1011-1016`, `:3203-3211`, `:5154-5157`, `:5543-5550` ("the next event reads the OS" → "the main thread's read after the noticed event") (5) | 3, 5, 7 |
| `src/engine/properties.rs` | `Mode::Unnoticed`, the new test, docs `:1-55` | 7 |
| `src/hook/main_thread.rs` | `HIDDEN_INPUT`, `note_hidden_input`, `hidden_input_notes`, refusal note, unknown-switch note, `wake_this_thread` `pub(crate)`, `MainState::resync_if_due`, `input_returned` → engine, drain step, module docs (5); `HOOK_LATE(_MS)`, `note_if_awake_too_long`, E5 in `run` (8); notes sum, `wake_for_hidden_input` (9); tests | 5, 8, 9 |
| `src/hook/keyboard_hook_callback.rs` | foreground callback wake, docs `:48-52`, `:403-410` (5); `track_injected` in `decide_event`, docs `:13-15`, `:209-215`, `:494-506` (7); E4, `event_age_ms`, `late_callback`, test helper and `:871-875` time stamps (8); tests | 5, 7, 8 |
| `src/hook/health.rs` | `unseen_input` (9) | 9 |
| `src/hook/watchdog/decision.rs` | `unseen_at_last_tick` (9); `raw_pending` (11); tests, docs | 9, 11 |
| `src/hook/watchdog/mod.rs` | `RequestReinstall::hidden_input`, impl, `Ticker::pass` E6, `Recorder` counts (9); deferral, `woke_idle`, `Drive::wake`, three expectations, Cost doc `:23-42` (11) | 9, 11 |
| `src/logging/counters.rs` | `HookLate`, `warns`, `summary` level; tests | 8 |
| `src/logging/writer.rs` | level from `summary`; expected lines gain `hook_late=0` | 8 |
| `README.md` | `:553-567` (D10) (4); `:149`, `:558-561`, `:952` (7); `:865-874` (8); `:948` with R16's measured figure (11) | 4, 7, 8, 11 |
| `book/src/config/keys.md` | `:194`: "The generic codes Shift 0x10, Ctrl 0x11 and Alt 0x12 count as held only between the key-down and the key-up that a program sends under them; the main thread's modifier resync, between key events, reads the sided codes only (roadmap item 2.5)" | 7 |
| `book/src/config/known-issues.md` | new row 88 (text below) | 7 |
| `book/src/config/errors.md` | `:231` (`hook_late`, `[INFO]` rule) (8); `:214` "at the first tick that counts a key event, a quarter of a second after it", `:215` "about 11 wakes", "N of about 1100" (11) | 8, 11 |
| `src/config/default_config.ron` | **none**: no config meaning changes | — |
| Non-config book pages | **not touched**; listed in `impl.md` for the next book sync: `architecture.md:237`, `:248`, `:251`, `:265`, `:294`, `:331`, `:350`, `:362`, `:566-567`, `:605`; `glossary.md:67`, `:377`, `:569`, `:577` (add "Hiding count"); `overview.md:16`; `gaps.md:167`, `:300`; `roadmap.md:584`, `:586-595` (2.5 status: W1, W2), `:739` (3.5: the witness follow-up lapses, FU-4 done), `:999`, `:1005`, check 44 `:654` (amended, M5), the re-runs M1-M8 | — |

**Known-issues row 88** (commit 7; commits 8 and 9 add their triggers; the developer pins the link to the merge
commit, as the other rows pin `bf549c0`):

> | 88 | A modifier (Shift, Ctrl, Alt, Win) pressed or released where the daemon cannot see it, with nothing the daemon
> notices after it, keeps its old state for the daemon until the main thread next reads the modifiers from Windows:
> after a foreground change, `swch on`, a hook reinstall, the return from a lock or the secure desktop (or the first key
> after it), a refused or late keyboard callback, a gap the hook watchdog saw, or within a second while the daemon
> believes a modifier is held. Until then a binding can miss or fire with the wrong modifiers, a press of that modifier
> can count as an auto-repeat, and the menu mask key can be sent when not needed or not sent when Windows has Alt or
> Win down. No key sticks and no key-up reaches Windows without its key-down. Cases: hook calls Windows drops while the
> main thread stalls outside every span it measures; `LowLevelHooksTimeout` below about 34 ms; a secure-desktop
> episode with no signal at all; focus in a child window of a higher-integrity process; key events decided in the same
> burst of messages right after a stall; a generic `VK_SHIFT`, `VK_CONTROL` or `VK_MENU` another program holds while
> the daemon reads the modifiers (from code) | By design (roadmap [item 2.5](../roadmap.md#25-latency-trim)) |
> [src/engine/resync.rs](https://github.com/bluesteelll/key-switch-rs/blob/<merge-sha>/src/engine/resync.rs) |
> [§7](kinds.md#s7) | none |

## Commits, in order

Subjects `<Area>: <what>`; each message ends with a blank line and
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Every commit passes the gate except the declared red
commits 3, 6 and 10.

### 1. `Docs: a Hold's fast actions run on the main thread, and timers.rs serves three threads`

D14. No test. Green.

### 2. `Bench: rows R17t and R17a; the callback's events carry a time stamp`

`Cargo.toml` feature; `bench.rs` `call_keyboard_callback` sets `time: GetTickCount()` (its SAFETY line); rows R17t,
R17a; docs; CHANGELOG Added. Green (the bench is ignored). The developer then runs the bench once into `impl.md`,
probes D-R12n and D-R16d (D13), and applies the stop rules of D13 and Q4.

### 3. `Engine: a swallowed repeat of a modifier with no record keeps its release from Windows (red)`

`engine::scenarios::a_swallowed_repeat_of_a_modifier_with_no_record_keeps_its_release_from_windows`:

```rust
Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
    .check_nothing_stuck()
    .check_no_lone_key_up()
    .down("LAlt").expect_passed()
    .down_unseen("LShift")
    .down("A").expect_passed()   // master's sync finds LShift held, no record
    .up("A").expect_passed()
    .up_unseen("LShift")         // Windows has LShift up; the next event is LShift's own, which master's sync skips
    .down("LShift").expect_suppressed() // a repeat to the engine, no record; the Alt+Shift blocker swallows it
    .up("LShift").expect_suppressed()
    .up("LAlt").expect_passed();
```

**PREDICTED RED:** panic `scenario: up LShift at +0 ms: Windows got a lone key-up of LShift` (the lone-key-up check,
`testing.rs:644-657`), before `expect_suppressed`.

### 4. `Engine: a swallowed repeat with no record is recorded, so its key-up asks Windows`

D10; makes 3 green. `keys.rs` (`RepeatSuppressed`, `mark_repeat_suppressed`, `take_blocked`, `resync_from_os` keeps
it); `mod.rs:839-845` and docs `:185-195`; README `:553-567`; CHANGELOG Changed. Unit tests:

- `engine::keys::tests::a_repeat_suppressed_record_is_consumed_like_a_swallowed_one_and_kept_by_a_reset`;
- `engine::keys::tests::a_repeat_suppressed_key_is_not_skipped_by_the_modifier_resync_and_counts_for_the_mask`
  (calls `sync_modifiers` here; moves to `resync_modifiers` in commit 7);
- `engine::tests::a_repeat_with_no_record_that_passes_after_a_swallowed_one_is_recorded_as_passed`.

Invariant kept: a key-up is suppressed iff its key-down was, unless Windows has the key down.

### 5. `Main thread: read the modifiers between events when something could have hidden them`

Additive: `on_key` still syncs; the drain reads as well. Contents: `engine/resync.rs`; `Engine::resync_if_due`,
`input_returned`, `on_foreground_changed()` (bump only), `reset` bookkeeping (E2, E3); `KeyState::resync_modifiers`,
`unswallowed_modifier_held`; `HIDDEN_INPUT`, `note_hidden_input`, `hidden_input_notes`, E8 in `with_main`, E3u;
`MainState::resync_if_due`, the drain step, `input_returned` → engine; the foreground callback's wake; the harness
(D15, without the property mode); commit 3's scenario switches `up_unseen("LShift")` to `up_unnoticed("LShift")` (a
noticed key-up would now heal LShift before the press, and the scenario would stop exercising D10); bench R18;
CHANGELOG Added R18. Compile-red on master (new API), carried by mutations X1-X12. Tests:

- `engine::resync::tests::`: `a_new_resync_plans_a_full_read_at_its_first_plan`; `a_bump_plans_one_full_read`;
  `a_changed_note_count_plans_a_full_read_a_smaller_one_included`;
  `a_modifier_touched_since_the_last_plan_is_left_out_and_read_at_the_next`;
  `nothing_is_read_while_input_is_away_until_it_returns_or_a_key_event_is_decided`;
  `a_modifier_held_for_a_second_plans_a_full_read_once_a_second`; `with_nothing_held_and_nothing_due_the_clock_is_not_read`
  (its `now` closure panics).
- `engine::tests::` (asks counted from the `resync_if_due` call only, since `on_key` still syncs in this commit):
  `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers` (foreground change, `InputLeft` then
  `input_returned`, `InputLeft` then a decided event, notes; `CountingEnv` sees the 8 sided codes);
  `swch_on_and_a_reinstall_read_everything_themselves_so_the_next_drain_reads_nothing`; `a_reload_makes_no_read`;
  `a_drain_read_asks_only_the_sided_modifiers_and_skips_a_swallowed_one`.
- `hook::main_thread::tests::`: `the_drain_reads_the_modifiers_after_a_note_and_not_again`;
  `a_refused_call_notes_hidden_input`; `a_desktop_switch_to_an_unknown_destination_notes_hidden_input`;
  `a_return_signal_makes_the_next_drain_read_the_modifiers`. These read the live key state (read only); assertions are
  on whether a read happened, not on its result.
- `hook::keyboard_hook_callback::tests::a_foreground_change_queues_a_wake_for_the_drain` (`PeekMessageW` with
  `PM_REMOVE` filtered to `MAIN_WAKE` on the test thread's own queue).

### 6. `Engine: a key event asks Windows about no modifier (red)`

`engine::tests::a_key_event_asks_the_os_about_no_modifier`: `CountingEnv` (`mod.rs:1214-1239`), `F13` bound to
`DoNothing`; `A` down, `A` up, `F13` down, `F13` up; the number of asks whose code is one of the 11 `MODIFIERS`.
**PREDICTED RED:** `assertion `left == right` failed: a key event reads no modifier from the OS (roadmap item 2.5)`
with `left: 44`, `right: 0` (11 per event, none skipped: neither key is a modifier, nothing is swallowed).

### 7. `Engine: the key path reads no modifier state`

D1, D2, D3; makes 6 green. `on_key` without the sync; `sync_modifiers` deleted; generic codes out of
`resync_from_os`; `track_injected` and `is_tracked_modifier`; `decide_event` tracks sentinel modifiers; keys.rs tests
moved (touch set); `Mode::Unnoticed`; docs (`env.rs`, `matching.rs`, `mod.rs`, `keys.rs`, callback module docs);
README `:149`, `:558-561`, `:952`; `keys.md:194`; known-issues row 88; CHANGELOG Changed. Tests:

- Scenarios on `DEFAULT_PAIR` (S9 on its own config), each under `check_nothing_stuck` and `check_no_lone_key_up`,
  each starting with `.down("A").up("A")` so the first drain's full read is behind them:
  - **S1** `a_hidden_modifier_release_nobody_noticed_misfires_one_binding_until_the_next_note`: `down("LShift")`,
    `up_unnoticed("LShift")`, `down("CapsLock")` fires `ToggleCapsLock` (row 88), `up("CapsLock")`,
    `hidden_input_noted()`, `down("CapsLock")` fires `SwitchLanguage`.
  - **S2** `a_hidden_modifier_release_nobody_noticed_heals_within_a_second_of_holding`: `down("LShift")`,
    `up_unnoticed("LShift")`, `advance_ms(1000)`, `down("CapsLock")` fires `ToggleCapsLock` and its drain reads
    (D7), `up("CapsLock")`, `down("CapsLock")` fires `SwitchLanguage`.
  - **S3** foreground change: `down("LShift")`, `up_unnoticed("LShift")`, `foreground_changed()`, `down("CapsLock")`
    fires `SwitchLanguage`.
  - **S4** lost input, return signal: `session_locked()`, `down_unnoticed("LShift")`, `session_unlocked()`,
    `down("CapsLock")` fires `ToggleCapsLock`.
  - **S5** lost input, no return signal: `session_locked()`, `down_unnoticed("LShift")`, `down("F13")` (decided with
    every key released; its drain reads), `down("CapsLock")` fires `ToggleCapsLock`.
  - **S6** remapping switch: `remapping_off()`, `down("LShift")`, `remapping_on()`, `down("CapsLock")` fires
    `ToggleCapsLock`.
  - **S7** reinstall: `down_unnoticed("LShift")`, `hook_reinstalled()`, `down("CapsLock")` fires `ToggleCapsLock`.
  - **S8** an adapter note (refusal, late callback, watchdog gap): `down_unnoticed("LShift")`, `hidden_input_noted()`,
    `down("CapsLock")` fires `ToggleCapsLock`.
  - **S9** `a_swallowed_modifier_released_unnoticed_stays_held_and_its_next_press_stays_swallowed` (KS-14 through a
    read): `( keys: "LShift", action: DoNothing, block_default: true )`, `down("LShift")` suppressed,
    `up_unnoticed("LShift")`, `hidden_input_noted()`, `down("LShift")` suppressed (a repeat), `up("LShift")`
    suppressed.
- `engine::tests::`: `an_injected_generic_ctrl_counts_for_matching_between_its_key_down_and_key_up`;
  `an_injected_key_up_of_a_held_swallowed_modifier_leaves_it_held`;
  `an_injected_event_of_a_key_that_is_not_a_modifier_changes_nothing`.
- `engine::keys::tests::a_resync_never_takes_a_generic_modifier_from_the_os`.
- `hook::keyboard_hook_callback::tests::the_daemons_own_injected_modifiers_pass_and_are_tracked`: `decide_event` with
  the sentinel, `LCtrl` down → `None`; a physical `J` with `( keys: "Ctrl+J", action: DoNothing )` → fires; the
  sentinel `LCtrl` up; `J` again → nothing fires. Deterministic: the key path reads no live modifier state now.
- `engine::properties::no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed`.
- Every 2.3 test and property, unchanged (acceptance).

S1, S2, S4 and S5 fail on commit 6's tree (master's sync heals at `CapsLock`'s own event): reverting commit 7's
`on_key` change is mutation X13.

### 8. `Hook: a late callback or a long stretch away from messages makes the main thread read the modifiers`

E4, E5, D9; README `:865-874`; `errors.md:231`; CHANGELOG Added `hook_late`, Changed (line level; the row-88 trigger
list), example line `:120`; test helpers' time stamps. Compile-red (new API), carried by X17-X20. Tests:

- `hook::keyboard_hook_callback::tests::`: `lateness_counts_age_and_run_with_a_17_ms_boundary` (pure: 16 false, 17
  true; `ran` 17 ms true; wraparound at `u32::MAX`; a stamp 5 ms ahead of `now` is age 0);
  `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late` (extern callback under `GLOBAL_HOOK`,
  `time = GetTickCount() - 1000`); `a_callback_for_a_fresh_event_notes_nothing` (one retry of the whole check);
  `an_injected_events_own_stamp_is_not_its_age` (`LLKHF_INJECTED`, `time` 1).
- `hook::main_thread::tests::`: `an_awake_stretch_of_17_ms_notes_hidden_input` (pure, 20 ms notes, 5 ms does not);
  `the_main_loop_checks_its_awake_stretch_before_every_wait` (scan of `fn run(`: the check before
  `MsgWaitForMultipleObjectsEx`, `awake_since` set right after it).
- `logging::counters::tests::hook_late_alone_is_reported_at_info_and_with_another_count_at_warn`; the existing counters
  and writer tests updated for 12 counters.

### 9. `Watchdog: raw key events no callback explains make the main thread read the modifiers`

E6; `RequestReinstall::hidden_input`; the notes sum in `MainState::resync_if_due`; CHANGELOG and row 88 trigger lists.
Compile-red, carried by X21-X24. Tests: `hook::health::tests::unseen_input_counts_each_note`;
`hook::watchdog::decision::tests::a_tick_reports_raw_key_events_no_callback_explains` (raw every 30 ms with no callback
→ above 0; callbacks keeping up → 0 at every tick);
`hook::watchdog::tests::a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread` (`Drive`, `Recorder`
counts `hidden_input`); `hook::main_thread::tests::the_drain_counts_the_watchdogs_notes`.

### 10. `Watchdog: the first raw key event after idle is read at the first tick (red)`

`hook::watchdog::tests::the_first_raw_key_event_after_idle_is_read_at_the_first_tick`: on `Drive`, one raw event and
one callback at 0 ms, run to 600; asserts `drive.reads.first() == Some(&(250, 1))` and `drive.raw_wakes == 1`.
**PREDICTED RED:** `left: Some((0, 1))`, `right: Some((250, 1))`.

### 11. `Watchdog: an idle wake only starts the ticks`

D12; makes 10 green; only if D-R16d passed. `raw_pending`, `woke_idle`, the deferral, `Drive::wake`, the three
expectations, Cost doc; R16 doc and gate; README `:948` with R16's measured HEAD figure, never a planned one (2.4 V3-M2);
`errors.md:214-215`; CHANGELOG Changed. Tests: `hook::watchdog::decision::tests::a_deferred_first_read_starts_the_silence_at_the_wake`;
`hook::watchdog::tests::an_idle_wake_with_no_hook_installed_still_reads_at_once`;
`…::a_deferred_wake_is_followed_by_a_wait_that_leaves_only_the_batched_input_out`. Then R16 at HEAD (D13 step 3).
After it the developer records in `impl.md` the HEAD bench, the non-config book pages and the manual checks.

**CHANGELOG texts** (developer copies; the trigger list grows in 8 and 9):

- Added (2): "Bench rows R17t, one `GetTickCount`, and R17a, one `GetAsyncKeyState` (roadmap item 2.5)."
- Changed (4): "An auto-repeat of a key whose key-down the daemon never saw (a modifier Windows had down when the
  daemon read it), which a binding swallows, now keeps the key's release from Windows unless Windows has the key down,
  as for any swallowed press. Before, the release always passed, and Windows could get a key-up without its key-down
  (roadmap item 2.5)."
- Added (5): "Bench row R18: the main thread's read of the modifiers between key events (roadmap item 2.5)."
- Changed (7): "The keyboard hook no longer asks Windows for the state of eleven modifier codes at every key event,
  about 4 µs of the 4.7 µs of an ordinary key event (bench row R12; roadmap item 2.5). The engine follows Shift, Ctrl,
  Alt and Win from the key events it sees, the daemon's own injected ones included, and the main thread reads the eight
  sided modifier keys from Windows between key events: at startup, after a foreground change, `swch on` and a hook
  reinstall, when keyboard input comes back from a lock or the secure desktop, after a callback that found the daemon
  busy, and once a second while a modifier is held. A modifier pressed or released where the daemon cannot see it,
  with none of these after it, can make a binding miss or fire until then; no key sticks (known issue 88). The generic
  codes `VK_SHIFT`, `VK_CONTROL` and `VK_MENU` count as held only between the key-down and the key-up a program sends
  under them."
- Added (8): "Counter `hook_late` (roadmap item 2.5): a keyboard callback that ended 17 ms or more after its key event,
  or a main thread 17 ms or more away from its messages; the main thread then reads the modifiers again. The stamp of
  an injected event does not count." Changed (8): "A counters line whose only non-zero count is `hook_late` is written
  at `[INFO]`, not `[WARN]`."
- Changed (11): "The hook watchdog's first wake after idle makes no user32 call: it only starts its ticks, and the key
  events wait for the first tick, a quarter of a second later (roadmap item 2.5, follow-up FU-4). A keyboard callback
  at that moment is no longer slowed by about 50 µs (bench row R16, now N µs at the median, gated). A burst of keys
  costs the watchdog one more wake: about 11 for an isolated shortcut."

## Acceptance mapping

| Acceptance line (roadmap `:595`, spec `:15-19`, rulings) | Covered by |
|---|---|
| R11 within the D14 rule of R12 | Superseded by W1 (the witness stays). R11's witness term unchanged: D13 R11 rule; `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key` (unchanged) |
| A new row shows the witness reading after a foreground change | Moot under W1: the witness reads at every swallowed key-up (R11) |
| W2: R12 and R1/R4 show the reads gone; the new R12 bound from a diagnostic row; a row for the resync on main | D13: R12/R4 from D-R12n, R1 mean, R18; `engine::tests::a_key_event_asks_the_os_about_no_modifier` |
| Every stuck-key and lone-key-up test and property of 2.3 stays green, the unseen re-press after a focus change included | `engine::scenarios` (unchanged, incl. `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows`, `scenarios.rs:2030-2045`), `:2011-2084`, `:2091-2338`, `:2342-2406`; `engine::tests` `mod.rs:1241-1316`; the six `engine::properties` tests |
| W2: a hidden modifier key-up followed by a binding press heals at the next resync | S1, S2 |
| W2 / spec: each source triggers a resync (foreground change, lost-input reset, remapping switch, reinstall, refused event, watchdog gap, late callback) | S3 (E1), S4, S5 (E3), S6 (E2), S7 (E2), S8 (E8/E4/E6 in the engine); adapter: commit 5 `a_refused_call_notes_hidden_input`, `a_desktop_switch_to_an_unknown_destination_notes_hidden_input`; commit 8 `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late`, `an_awake_stretch_of_17_ms_notes_hidden_input`; commit 9 `a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread`, `the_drain_counts_the_watchdogs_notes`; engine `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers` |
| W2: no stuck key and no lone key-up across the property modes of 2.3 | the six properties (unchanged), the new `Unnoticed` property, commit 3's scenario, S9 |
| W2: residual matching misses in known issues | row 88 (commit 7) |
| R16 meets a bound the cut sets from the evidence | D12, D13 (provisional bound, D-R16d, HEAD check); `the_first_raw_key_event_after_idle_is_read_at_the_first_tick` |
| No other bench row regresses | tester §4b with the D13 table |
| The 2.4 watchdog acceptance stays green | `hook::watchdog::decision::tests::*` unchanged; `roadmap.md:578-581` rows; three loop-model expectations move by one tick (D12, Q6) |
| W4: `hook_late` no WARN in normal typing; manual check; unverified base marked; injector stamps | D8, D9; `hook_late_alone_is_reported_at_info_and_with_another_count_at_warn`; `an_injected_events_own_stamp_is_not_its_age`; M8; U1 |
| Book-sync leftovers | commit 1 |
| CHANGELOG (citing 2.5), README latency notes, config pages where a cost or behaviour changes | commits 2, 4, 5, 7, 8, 9, 11 (touch set) |

## Manual checks (human only)

None can be a test: each needs a live hook, the secure desktop or a debug daemon. No test is ignored for them.

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | Roadmap check 35 (`roadmap.md:645`): non-elevated debug daemon after `swch exit`, default config, Notepad, elevated Task Manager | (a) as check 35; (b) hold LShift in Notepad, click into Task Manager, release LShift there, click back into Notepad, press CapsLock | (a) as check 35 (W1: unchanged); (b) the layout switches, caps lock does not toggle: the foreground change made the main thread read LShift as up (E1) | none |
| M2 | Roadmap check 36 (`:646`), release daemon, default config | (a) as check 36; (b) Ctrl+Alt+Del, there press and hold LShift, press Esc, on the desktop press CapsLock with LShift still held, release both | (a) as check 36; (b) caps lock toggles (`Shift+CapsLock`), the layout does not switch: the return read LShift held (E3, D6) | none |
| M3 | M2 with a UAC prompt; `PromptOnSecureDesktop` = 1 (read only, as check 28) | M2 (b) at the prompt instead of Ctrl+Alt+Del, Cancel instead of Esc | as M2 (b) | none |
| M4 | 1. `swch exit`; `swch status` says not running. 2. `cargo build --locked`. 3. default config, `osk.exe` open. 4. Read only: `reg query "HKCU\Control Panel\Desktop" /v LowLevelHooksTimeout` | 1. `$env:KS_DEBUG_HOOK_HOLD='CapsLock'; .\target\debug\key-switch-rs.exe`. 2. Press and release CapsLock once (1.5 s hold); during the hold press and release LShift. 3. Press CapsLock again. 4. `swch status`; `swch exit`. 5. Read `daemon.log`. 6. `swch open` | CapsLock is not shown held in `osk.exe`; step 3 switches the layout (no phantom Shift); an `[INFO] counters ...` line shows `hook_late=` of at least 1. Report `reinstalls` (1: Windows removed the hook; 0 with remapping working: known issue 83) and whether `hook_late` is above 2 (keys typed during the hold arrived late: Windows delivered timed-out calls, critique open question 1) | none |
| M5 | Roadmap check 44 (`:654`), amended | as check 44 | `raw keyboard input arrives` now comes one tick (250 ms) after the raw input. Compare t_raw − 250 ms with t_key: earlier by about the hook timeout means before the hook call; within about 20 ms means after it (U5) | none |
| M6 | Roadmap check 39 (`:649`) | as check 39 | W ≤ 100; the start and the two typed commands now give about 48 (one more wake per burst, D12) | none |
| M7 | Roadmap check 47 (`:657`) | as check 47 | `W ≤ K/2`; `hook ok, reinstalls: 0`; W is about one higher than before for the same hold | none |
| M8 | Release daemon via `swch open` (ruling W4) | Type normally for 5 minutes across programs, an elevated one included; `swch exit`; read only: `Select-String 'counters since the last report' "$env:LOCALAPPDATA\key-switch-rs\daemon.log" \| Select-Object -Last 3` | No line in those minutes, or lines with `hook_late=0`. Any `hook_late` above 0 while typing normally: report the lines; the stamp base differs (U1), and the fallback is the run time alone (Q4) | none |

## Risks and edge cases (code-reviewer R1-R10)

- **R1 Hook callback.** Removed: the per-event modifier reads. Added on every callback while remapping is on: two field
  copies, one `GetTickCount` at exit for a physical event, two compares; when late, a TLS add and one RMW; for a
  sentinel modifier, one `with_main` and a store. No allocation, lock, `log::`, post or blocking call. `code < 0` and
  the remapping gate are first, unchanged; the sentinel test still precedes every decision; one outcome per path.
- **R2 Sentinel.** No new `SendInput`. Sentinel modifier events are tracked, never decided (D2).
- **R3 Shared state.** `HIDDEN_INPUT` is a main-thread TLS `Cell`, like `PENDING_SIGNALS`; `unseen_input` an atomic
  in `HealthState`. Under the borrow only pure engine work and `GetAsyncKeyState`.
- **R4 Orderings.** `unseen_input`: `fetch_add(AcqRel)`, `load(Acquire)`, no published data. No `Relaxed`.
- **R5 Builders.** None.
- **R6 Display.** `Counter::HookLate`: `Display` arm, `ALL`, slot and `warns` arm in commit 8; no `_ =>`.
- **R7 Specificity.** Untouched.
- **R8 Key identity.** Generic slots only from events (D3); sentinel modifiers tracked (D2); `ModifierBits::of_event`
  maps a generic code to both sides for `fresh`; the witness still reads the event's own code
  (`the_witness_of_a_generic_shift_key_up_reads_the_generic_state` unchanged).
- **R9 Registry and process.** No new read.
- **R10 Loop and shutdown.** E5's `continue` cannot spin: it resets `awake_since`, and the next check sees a short
  stretch. The drain's read stops with the loop. D12 cannot spin (above).

**Other edges.**

- Reload mid-gesture: no read (`Reloaded` moves nothing). `swch off`: no reads while off; `swch on` reads everything.
- Foreground change mid-press: bump, read at the drain; the pressed modifier is fresh if touched in the same stretch.
- AltGr: the layout's LCtrl and RAlt reach the hook as events and are tracked like any other.
- Auto-repeat: sets the flag again; no read.
- A modifier physically held while a higher-integrity window has the foreground: a read then sees zero (UIPI); the hook
  sees no key there; the read at the return gives the truth.
- Sleep and resume: `GetTickCount` counts since start, so an event stamped before sleep reads as late: one read.
- Tests that drain read the live key state (read only); they assert only whether a read happened. Tests that do not
  drain no longer depend on the keys the user holds.
- IME and layouts: no effect.

## Questions for the orchestrator

1. **(old Q1) Moot** under W1.
2. **(old Q2)** The residuals are matching-only (D11) and declared as row 88; the owner sees them in the lane report
   (ruling). Recommendation: none needed.
3. **(old Q3) R16's bound is provisional** (D13). Recommendation: as ruled (W3): probe D-R16d after commit 2, check
   HEAD after commit 11, stop and report on a miss. If the cost is in the wake itself, keep R16 declared (2.4 ruling
   L2) rather than buy the 1 s idle poll; that poll is an idle-cost choice for the owner.
4. **(old Q4) E4: 17 ms and one `GetTickCount` per physical callback.** Accepted. Stop rule: if R17t p50 > 100 ns (a
   system call), E4 uses the run time alone (`ran >= HOOK_LATE`), dropping the age part; W6 is then covered by E5 and
   E6 only. The same fallback applies if M8 fails.
5. **(old Q5) Folded** (W2).
6. **(old Q6) Three loop-model expectations move by one tick** (D12). Accepted.
7. **(old Q7) `hook_late`.** Accepted with W4 (D9).
8. **(old Q8) 4.6 follow-up** (a raw-input registration plus a focused daemon window stops the process's LL hook;
   `RIDEV_NOHOTKEYS`). Accepted: "Follow-ups from Phase 2" at the book sync.
9. **(old Q9) Chore after the merge** (CLAUDE.md, `tester.md` §4b, `code-reviewer.md`): bench rules (R12/R4 bound from
   D-R12n, R1 mean, R11 witness term, R16 gated, R17t, R17a, R18); principle 1 (the callback reads no modifier state;
   E4's `GetTickCount` and late note are sanctioned); principle 3 (sentinel modifier events update a flag, D2);
   principle 5 (`HIDDEN_INPUT`, `unseen_input`); principle 6 (the drain's read; the foreground callback's wake); the
   layout (new `src/engine/resync.rs`). Recommendation: as after 2.3 and 2.4.
10. **(new) The auto-blocker's OS read stays on the key path** (`mod.rs:721-723`, KS-05): it asks only about held keys
    outside a blocker's hotkey, only once that hotkey is fully held, overwrites nothing, and is the KS-05 witness, not
    a resync. Recommendation: keep; answering it from flags would change KS-05 and belongs to 3.5.
11. **(new) `RESYNC_PERIOD` = 1 s** (D7). Recommendation: accept; shortcuts end well within it, and a phantom costs at
    most one key event after it.
12. **(new) The `Unnoticed` property mode** tests D11 on random scripts. If a seed fails, the proof is wrong:
    recommendation, the developer stops and reports the seed and transcript, with no workaround.
13. **(new) D10 changes a suppression rule in a corner case** (README, CHANGELOG). Recommendation: accept; without it
    D11's "no lone key-up" fails for an unnoticed hidden key-up, and the hole already exists on master (commit 3).

## Cut readiness checklist

### Spec
- [x] Item 2.5 cites no KS issue (`roadmap.md:439`); KS-14, KS-05, KS-06 (status) and KS-12 rules are traced (D3, D10,
  D11) and their scenarios kept or added (commit 3, S1-S9, M1-M8).
- [x] Every reference re-located; none gone.
- [x] Goal, non-goals and later items stated.

### Design
- [x] Every decision has a rationale and rejected alternatives (D1-D15).
- [x] Owner decisions: exact modifiers and repeat defaults unchanged; RON unchanged; `log` and the sink (the hook path
  only counts); licence n/a; latency first (every addition costed, D13); no key sticks (D11, W1).
- [x] Threads named (thread map); nothing heavy in the LL callback (it loses 11 reads, gains one tick read).
- [x] Install, uninstall and timers unchanged; the watchdog's timer stays on its thread.
- [x] No new `SendInput`; sentinel events tracked, never decided.
- [x] Sided and unsided: D3, R8; chords and sequences read the same flags (D11 item 4).
- [x] Reload, `swch off`/`on`, foreground change, shutdown during a pending gesture: D4 table, edges.
- [x] Orderings stated (R4); no `static mut`.
- [x] No new module cycle: `engine::resync` is a child of `engine`; `hook` already depends on `engine`. Logic in
  `engine`, reachable by scenarios.

### Win32
- [x] Paths, feature check (one to add), threads, errors, links; unverified items U1-U6 with their checks.
- [x] Elevation and UIPI (W1, W2, GetAsyncKeyState's zero); pipe and tokens n/a.

### Commits and tests
- [x] Red-first: 3 → 4, 6 → 7, 10 → 11 by assertion; 5, 8, 9 compile-red, carried by mutations (below).
- [x] Tests go through `Engine::on_key`/`resync_if_due`, `Env` and the fake clock; adapter tests drive the extern
  callback with no hook installed; nothing hooks, registers raw input or injects.
- [x] Real Win32 in tests: reads only (`GetTickCount`, `GetAsyncKeyState`, `PeekMessageW` on the own queue).
- [x] Every commit passes the gate except 3, 6, 10; docs go with the commit that changes the meaning.

**Mutations for the tester** (one per compile-red test, critique M4, plus the design pins):

| # | Mutation | Caught by |
|---|---|---|
| X1 | `Resync::new` with `hiding` 0 | `a_new_resync_plans_a_full_read_at_its_first_plan` |
| X2 | `bump` does nothing | `a_bump_plans_one_full_read` |
| X3 | `plan` does not fold the notes (or folds with `>`) | `a_changed_note_count_plans_a_full_read_a_smaller_one_included` |
| X4 | `plan` reads fresh modifiers (or drops `owed`) | `a_modifier_touched_since_the_last_plan_is_left_out_and_read_at_the_next` |
| X5 | `input_away` ignored, or not cleared by a decided event | `nothing_is_read_while_input_is_away_until_it_returns_or_a_key_event_is_decided`; S5 |
| X6 | the periodic rule never fires | `a_modifier_held_for_a_second_plans_a_full_read_once_a_second`; S2 |
| X7 | `plan` always reads the clock | `with_nothing_held_and_nothing_due_the_clock_is_not_read` |
| X8 | `on_foreground_changed` does not bump | `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers`; S3 |
| X9 | `Reloaded` bumps | `a_reload_makes_no_read` |
| X10 | `resync_modifiers` reads a generic code or ignores KS-14 | `a_drain_read_asks_only_the_sided_modifiers_and_skips_a_swallowed_one`; S9 |
| X11 | the drain does not call `resync_if_due`; `MainState::input_returned` does not call the engine | `the_drain_reads_the_modifiers_after_a_note_and_not_again`; `a_return_signal_makes_the_next_drain_read_the_modifiers` |
| X12 | no note in `with_main`'s refusal; none for an unknown switch; no wake after a foreground change | `a_refused_call_notes_hidden_input`; `a_desktop_switch_to_an_unknown_destination_notes_hidden_input`; `a_foreground_change_queues_a_wake_for_the_drain` |
| X13 | `on_key` syncs again | `a_key_event_asks_the_os_about_no_modifier`; S1, S2, S4, S5 |
| X14 | `resync_from_os` reads 0x10-0x12 | `a_resync_never_takes_a_generic_modifier_from_the_os` |
| X15 | `track_injected` does nothing / ignores KS-14 / `decide_event` does not call it | `an_injected_generic_ctrl_counts_for_matching_between_its_key_down_and_key_up`; `an_injected_key_up_of_a_held_swallowed_modifier_leaves_it_held`; `the_daemons_own_injected_modifiers_pass_and_are_tracked` |
| X16 | D10 reverted | commit 3's scenario; the `Unnoticed` property |
| X17 | `late_callback` uses `>`; the signed-age rule removed | `lateness_counts_age_and_run_with_a_17_ms_boundary` |
| X18 | no note at the callback's exit / a note always | `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late` / `a_callback_for_a_fresh_event_notes_nothing` |
| X19 | an injected event's stamp counts | `an_injected_events_own_stamp_is_not_its_age` |
| X20 | E5 never notes / its call removed / `HookLate` warns | `an_awake_stretch_of_17_ms_notes_hidden_input` / `the_main_loop_checks_its_awake_stretch_before_every_wait` / `hook_late_alone_is_reported_at_info_and_with_another_count_at_warn` |
| X21 | `note_unseen_input` does nothing | `unseen_input_counts_each_note` |
| X22 | `unseen_at_last_tick` always 0 | `a_tick_reports_raw_key_events_no_callback_explains` |
| X23 | `Ticker::pass` notes but does not call `hidden_input` (or the reverse) | `a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread` |
| X24 | the notes sum drops the watchdog's count | `the_drain_counts_the_watchdogs_notes` |
| X25 | the deferral reads anyway | `the_first_raw_key_event_after_idle_is_read_at_the_first_tick` |
| X26 | the deferral ignores `hook_installed` | `raw_input_before_the_hook_is_installed_is_read_and_not_counted`; `an_idle_wake_with_no_hook_installed_still_reads_at_once` |

### Acceptance
- [x] Every acceptance line mapped, the superseded ones with their ruling.
- [x] Every question has a recommendation.

## Critique response

| Item | Disposition | Where |
|---|---|---|
| **I1** residuals against "an unknown case keeps the read"; Q2 routed to the orchestrator | **Resolved by ruling W1.** The witness keeps reading at every swallowed key-up; the hiding gate, the `up_since` stamp and the old residuals as stuck-key risks are removed (Revision table). The orchestrator put the trade-off to the owner (`rulings.md:8-9`). R-E2's registry read is no longer needed: no read is skipped on a stuck-key path. R-E1..R-E4 survive only as matching residuals of the new resync, proven harmless for stuck keys and lone key-ups in D11 and declared in row 88. "What happens to the trim if M1-M3 fail": no stuck key in any case (Lemma 1 holds without them); a failure of M1-M3 means a slower heal (the next count change or D7), reported, with no code fallback needed | D11, row 88, M1-M3 |
| **I2** the clock base, injected stamps, slow later hooks, no normal-typing check | **Resolved by ruling W4.** `hook_late` only triggers a read between events (a false positive costs about 3 µs on main, never a decision); an `LLKHF_INJECTED` event's age is 0 (KEYBDINPUT quote); a stamp ahead of the clock is age 0; later hooks only add reads; the line is `[INFO]` when `hook_late` is alone; M8 asks the owner for `hook_late=0` in normal typing; U1 marks the base with the Microsoft Learn quotes; fallback in Q4 | D8, D9, M8, U1, Q4 |
| **M1** E5 notes after events of the same stretch | **Both options taken.** E5's note now `continue`s straight to the drain, so nothing is dispatched between it and the read; events dispatched earlier in the stretch are decided before the read by construction (W2 forbids reading in a callback): residual R-M1, matching only | D8, D11, row 88 |
| **M2** short `SendInput` count; injected generic codes; R8 test; feature 6 | **Resolved.** W8 (the daemon's own inserted modifier events are tracked, D2, so a short count leaves the engine with what Windows has); W9 and D3 (generic slots from events only; R-G declared, healed by D7); tests `an_injected_generic_ctrl_counts_for_matching_between_its_key_down_and_key_up`, `a_resync_never_takes_a_generic_modifier_from_the_os`, `the_daemons_own_injected_modifiers_pass_and_are_tracked`. The old R8 behaviour change (the generic witness skip) left with W1; the new one is D3's. Feature 6 is listed under later items | D2, D3, W-table, Goal |
| **M3** the deferral adds one tick per burst | **Fixed.** D12 Cost, the module Cost doc (touch set), `errors.md:215` (about 11, about 1100) and M6/M7 now say one more wake per burst; "the deferral adds no wake" is gone | D12, M6, M7 |
| **M4** compile-red tests without mutations | **Fixed.** X1-X26 name a mutation for every compile-red test of commits 5, 8 and 9, and for the design pins | Mutation table |
| **Open question 1** (does Windows deliver a timed-out call later?) | No longer decides a stuck key (W1). M4 records it as evidence (`hook_late` above 2 after keys typed during the hold) | M4 |
| **Open question 2** (order of WinEvent and hook calls) | Matters only for matching now; within R-M1 | D11 |
| Strengths noted (D7 evidence, re-location accuracy) | Kept: D12 carries the Q6 table and the D-R16d probe unchanged; the re-location table is re-checked at `50b56a8`. The strengths about D1/D2/E3 of the old cut lapse with W1 | D12, Spec re-located |
