# Cut: p2-2.5-latency-trim — roadmap item 2.5 Latency trim (open question 13; FU-4; rulings W2, W2a: the modifier resync)

## Verdict

CUT READY

Tree read: `master` at `3cde1d0`. Master moved from `50b56a8` by `merge chore/rust-analyzer-lsp`. Grep finds its
rust-analyzer references only in `.claude/agents/*.md` and `docs/WORKFLOW.md`.

- **Re-read for this revision and unchanged:** every line this revision relies on in the engine, the adapter, the main
  thread, the harness, the properties, the bench, `Cargo.toml`, `README.md`, `CHANGELOG.md`, `book/src/config/` and
  the roadmap.
- **Carried over from the round-1 re-location on `50b56a8`:** the watchdog, counters and non-config book lines.

Branch to create: `perf/latency-trim` from `master`.

Binding input:

- `rulings.md`, round 1: W1-W4, M1-M4, Q1-Q5;
- `rulings.md`, round 2: W2a, M1-M5, OQ1-OQ3, Q4, Q9-Q13;
- `critique.md`;
- `critique_v2.md`.

The questions at the end are non-blocking; each has a recommendation.

## Revision after rulings (round 1)

Round 1 turned the witness trim into the modifier resync. Each row below says what round 1 set and where it stands now.

| Ruling | Round-1 change | Now |
|---|---|---|
| W1 | The witness trim left the lane: the hiding gate, `up_since`, the old S1-S7, M1-M6, R11f and the old residuals | stands; R11 keeps its witness term (D13) |
| W2 | Drain read on the hiding count; D1-D7, D10, D11, D15; `Mode::Unnoticed`; row 88; R18; bounds from D-R12n | stands, amended by W2a (Revision 2) |
| W3 | D12 adopted; R16 bound provisional and probed | stands; its cost text was corrected (v2 M3) |
| W4 | `hook_late` is a trigger only; `[INFO]` line; M8; U1 | stands; E5's clock changed (R2-15) |
| M1 | E5 `continue`s to the drain; R-M1 declared | stands |
| M2 | W8, W9, D2, D3 | stand; D3 amended (generic slots cleared, never set) |
| M3 | "one more wake per burst" | corrected by v2 M3 |
| M4 | mutations for commits 5, 8 and 9 | extended to every compile-red test (v2 M4) |
| Q1-Q5 | Q1, Q2, Q5 folded; Q3 = W3; Q4 accepted | Q11 (`RESYNC_PERIOD`) is moot: D7 is dropped |

## Revision 2 after rulings

| # | Ruling | What changed (section) | Why |
|---|---|---|---|
| R2-1 | W2a | **D16**: at a decided key event, before any reader of a flag, a clear-only check of the modifiers held for the engine. **D11** rewritten in two directions. Commits **8** (red) and **9**. Tests R-A..R-H and C-1..C-7; mutations X25-X37 | I2 is a regression on every press wherever a later hook swallows a modifier; I1 leaves a generic phantom. Both are repaired at the event that would read the stale flag |
| R2-2 | W2a (`RepeatSuppressed`) | D10, D16: the check reads it as a key with no record, as master's sync did (`keys.rs:105` skips only `Suppressed`) | the ruling asks the rule |
| R2-3 | W2a (re-entrant decided call) | **D17**: a passed modifier key-down stays *unapplied* until its callback returns (TLS `UNAPPLIED`, `Env::unapplied`); U7 | not documented either way; the skip costs nothing on ordinary typing |
| R2-4 | W2a (D7, Q11) | **D7 dropped**, with `RESYNC_PERIOD`, `held_since`, `last_read_at`, `unswallowed_modifier_held`, the clock argument of `plan`, two resync tests, the old S2 and the old X6/X7 | every reader of a set flag runs at a decided event that the check precedes |
| R2-5 | W2a (the drain's direction) | D3, D4: the drain read sets and clears the sided slots; it clears a generic slot only when that code reads up, and never sets one. W-table re-derived per direction; row 88 now covers the phantom-released direction only | ruling |
| R2-6 | W2a (W10) | W10 corrected: a hook *after* the daemon's in the chain. New row W14: a hook *before* it, where the round-1 argument holds | Hooks Overview |
| R2-7 | W2a (bench) | New rows R12m and R12m2 (commit 2); R1 and R4 restated (D13) | ruling |
| R2-8 | v2 M1 | Goal 4 no longer says "matching only"; D11 has an owner summary; row 88 names the four visible effects | ruling M1 |
| R2-9 | v2 M2 | Mutation table re-derived. S9 asserts `expect_nothing_fired()` on the second LShift press. S3 is now phantom-released; the old S1 became R-E. No scenario is listed against a mutation it passes under | ruling M2 |
| R2-10 | v2 M3 | D12 Cost, the module Cost doc, `errors.md:215` (numbers kept), M6, M7 and CHANGELOG (13): one more tick only for a burst of a single raw key event | ruling M3 |
| R2-11 | v2 M4 | A mutation for every compile-red test of every commit (X1-X3, X12, X16, X18, X23, X47-X49 among them) | ruling M4 |
| R2-12 | v2 M5 | D13: the tester runs P (commit 2), N (commit 2 with one line removed) and H (HEAD) in one session. R12, R12m, R12m2 and R4 gate against N; R1 gates against N with 0.5 Δ, so it fails if the removal is reverted | ruling M5 |
| R2-13 | OQ1-OQ3 | U4 recorded, no check; W13 and manual check M9 (StickyKeys); M8 notes Remote Desktop server stamps | rulings |
| R2-14 | Q4, Q9-Q13 | Q4's stop rule also covers E5. Q9 content updated. Q10 kept (the check precedes the blocker read). Q11 moot. Q12: the later-hook case joins the `Unnoticed` property. Q13: CHANGELOG **Fixed** | rulings |
| R2-15 | architect | **Commit order**: the check lands two commits after the removal, so commit 8's tests are red on a real tree (commit 7; Q9 here). **Harness**: one drain at construction (E0), so scenarios need no prelude. **E5 clock**: `GetTickCount` instead of `Instant`, which removes one QPC read between the main thread's wake and its pump; fallback in Q2. **M10** added (AutoHotkey remap) | latency first; literal red-first |

## Spec re-located

Line numbers are on `3cde1d0`. "Unchanged" means the code is where the spec, the book or an earlier round said. The
watchdog, counters and non-config book rows carry over from `50b56a8` (see the Verdict).

| Reference | Current file:line | Status | Note |
|---|---|---|---|
| `KeyState::sync_modifiers` (W2) | `src/engine/keys.rs:95-110`. Called at `src/engine/mod.rs:518-522` (every event, key-ups too) and `:613-615` (`on_foreground_changed`) | unchanged | the 11 `MODIFIERS` codes, `keys.rs:25-42`, generic ones included |
| `resync_from_os` | `keys.rs:112-142` | unchanged | reads codes 1..=254, generic ones included |
| `key_down_swallowed`, the KS-14 rule | `keys.rs:153-156`; the skip at `:105` | unchanged | every new read keeps it |
| `MENU_KEYS`, `alt_or_win_down_in_windows` | `keys.rs:44-47`, `:228-239` | unchanged | becomes `ModifierSet::MENU` |
| `LastDown`, the records | `keys.rs:61-71`; `mark_blocked` `:174-179`, `mark_completed` `:181-192`, `mark_passed` `:194-199`, `take_blocked` `:212-221` | unchanged | D10 adds a variant |
| `HeldKeys` for `KeyState` | `keys.rs:242-261` | unchanged | `AnySide` counts the generic slot (`:246-250`) |
| Readers of a flag at a key-down | `mod.rs:668-856`: `winner::elect` `:677`; `other_key_down` `:686` (records only, `gestures.rs:306-349`); `deferred_group` `:693`; `blocker_matches` `:721-723`; `admits` `:742`; `sequences.on_key_down` `:767-774`; `mark_completed` `:789`, `:815`; rule 5 `:823-853`; mask `:854` | unchanged | D16 runs before all of them |
| Readers at a key-up | `mod.rs:858-899`. Only `mask_if_swallowed_under_alt_or_win` (`:897`) reads flags; `gestures.release` (`:869`) takes no `KeyState` | unchanged | D16's key-up check |
| Rule 5 (repeat with no record) | `mod.rs:823-845` (code), `:185-195` (docs) | unchanged | D10 |
| The witness (W1: stays) | `mod.rs:878-896` | unchanged | `swallow && !env.os_key_down(vk)` |
| Decision rule 1 docs | `mod.rs:118-121` | unchanged | rewritten (commits 7 and 9) |
| `Engine::reset` | `mod.rs:617-666` (fn `:651`) | unchanged | gains the resync bookkeeping |
| `Env::os_key_down` docs | `src/engine/env.rs:34-52`; trait `:24-60` | unchanged | gains `unapplied` (D17) |
| `HookEnv` (real `GetAsyncKeyState`) | `src/hook/keyboard_hook_callback.rs:154-195`, read `:178-184` | unchanged | used by D16, the drain and D17 |
| `decide_event` | `keyboard_hook_callback.rs:209-254`: sentinel `:222-224`, `with_main` `:236-247` | unchanged | D2 |
| `carry_out` and the callback | `:256-278`; callback `:352-401`: `started` `:378`, decision `:385-392`, `record_callback` `:393`, returns `:394-400` | unchanged | E4, D17 |
| Re-entry docs | `keyboard_hook_callback.rs:83-109` | unchanged | D17 cites them |
| Foreground callback | `:403-426`; `MainState::foreground_changed` `src/hook/main_thread.rs:279-286` | unchanged | E1 |
| `with_main` refusal | `main_thread.rs:216-229` (`:218-224`) | unchanged | E8 |
| Signals | `note_input_signal` `:145-161`; `note_unknown_desktop_switch` `:163-172`; `wake_this_thread` `:182-190`; `answer_input_signals` `:760-790`; `MainState::input_returned` `:288-293` | unchanged | D6, E3u |
| Main loop and drain | `MainThread::run` `:673-720` (wait `:698-705`, pump `:716`); `drain` `:722-744`; `MainThreadHandle::wake` `:574-583` | unchanged | D4, E5 |
| `HealthState`; watchdog | `src/hook/health.rs:17-61`. `src/hook/watchdog/mod.rs`: Cost doc `:23-42` (re-read), `RequestReinstall` `:90-107`, `Ticker::pass` `:226-269`, `run_loop` `:694-750`, `Drive` `:1354-1519`. `decision.rs`: `raw_batch` `:135-149`, `tick` `:168-222`, `keeps_ticking` `:224-234` | carried over | E6, D12 |
| Counters | `src/logging/counters.rs:17`, `:19-62`, `:64-79`, `:81-97`, `:128-145`, `:203-222`; `src/logging/writer.rs:228-234` | carried over | D9 |
| Bench | `src/bench.rs`: docs `:20-82`; `BenchEnv` `:153-204`; R1 `:256-274`; `call_keyboard_callback` `:325-344` (`time` 0); R4 `:346-366`; R11 `:405-420`; R12 `:422-436`; `latency_bench` `:1023-1155` | unchanged | D13 |
| Harness | `src/engine/testing.rs`: docs `:1-71`; `FakeEnv` `:89-135` (generic read `:119-126`); `physical_key` `:137-148`; `with_bindings` `:222-243`; unseen steps `:346-380`; `foreground_changed` `:382-395`; `input_signal` `:427-446`; `key_event` `:630-668` (lone check `:644`) | unchanged | D15 |
| Properties | `src/engine/properties.rs`: `:1-55`, `:207-236`, `:250-303`, `:308-350`, `:360-388` | unchanged | `Mode::Unnoticed` |
| Scenarios that lean on the per-event sync | `src/engine/scenarios.rs`: `:189-203`, `:1011-1058`, `:2121-2148`, `:3005-3025`, `:3203-3233`, `:3526-3549`, `:5154-5167`, `:5543-5557`; `DEFAULT_PAIR` `:22-25`; `with_alt_shift_blocker` `:32-34` | unchanged | they stay green (D15); comments change |
| The daemon's injections | `src/core/windows_actions.rs:252-291`; `src/data/sequence.rs:163-212` (a macro `Combo("Ctrl+S")` sends the generic `VK_CONTROL`, `src/config/parsing.rs:19-22`); `src/system/system_function.rs:123-159`; mask `keyboard_hook_callback.rs:297-323` | unchanged | each pairs down and up in one `SendInput` |
| Phase 2 leftovers | `CHANGELOG.md:116`, `:744-745`; `src/platform/timers.rs:1-3` | unchanged | D14 |
| README | `README.md:149`, `:553-567`, `:865-874`, `:948`, `:952` | unchanged | |
| Config book pages | `book/src/config/keys.md:194`; `errors.md:214-215`, `:231`; `known-issues.md:131` (63), `:144` (76), `:146` (78), `:147` (79) | unchanged | |
| Roadmap 2.5 | `book/src/roadmap.md:586-595`; checks 35-47 `:645-657` (re-read); 3.5 `:739`; FU-4 `:999`; `:1005` | unchanged / carried over | non-config pages wait for the next book sync |
| `Cargo.toml` features | `Cargo.toml:41-58` | unchanged | `Win32_System_SystemInformation` is absent |
| windows 0.62.2 items | `GetTickCount` `src/Windows/Win32/System/SystemInformation/mod.rs:194`; `KBDLLHOOKSTRUCT` `src/Windows/Win32/UI/WindowsAndMessaging/mod.rs:4251-4255`; `KBDLLHOOKSTRUCT_FLAGS::contains` `:4262`; `LLKHF_INJECTED` `:4409` | verified in the registry copy (round 1) | |
| Tester procedure | `.claude/agents/tester.md:78-93` (replay on the parent), `:99-101` (§4b bench) | read | Q9 (replay commit), Q10 (parent runs) |

## Goal and non-goals

**After the lane:**

1. **No resync on every event.** The keyboard callback no longer reads every modifier at every event. The engine
   follows Shift, Ctrl, Alt and Win from the key events it sees, the daemon's own sentinel-tagged modifier events
   included (D1, D2).
2. **The check.** At a decided key event, before anything reads a flag, the engine asks Windows only about the
   modifiers held for it whose key-down it did not swallow. It drops those Windows has up and never sets one (D16, D17).
   Ordinary typing reads none; Ctrl+J reads one; master reads eleven at every event.
3. **The drain read.** Between events, the main thread reads the eight sided modifiers when the hiding count has moved
   (D4-D6). It also clears a generic slot whose code reads up (D3).
4. **What is left.** A modifier pressed where the daemon cannot see it, with nothing the daemon notices afterwards,
   counts as up until the next drain read or that key's own next event. The visible effects are named in D11 and in
   known-issues row 88. A modifier released unseen, or one that a later hook swallows, stops counting at the next key
   event. No key sticks and no key-up reaches Windows alone (D11). The witness is unchanged (W1).
5. **The watchdog.** Its first wake after idle makes no user32 read (D12), and R16 is gated.
6. **Book-sync leftovers.** The Phase 2 leftovers are fixed (D14).

**Non-goals:**

- the witness trim (W1);
- the auto-blocker's conditional OS read (KS-05, Q6);
- settling Q6 of item 2.4 for good (manual checks 43 and 44 stay the authority);
- a timer-only idle poll (fallback only);
- the cases of known issue 79;
- `KBDLLHOOKSTRUCT.time` as the engine's clock;
- reading the event's own key in the check (Q11, a follow-up).

**Later items that depend on this:**

- **3.1 (single input module).** Every new injection path keeps the sentinel, so D2 tracks its modifiers.
- **3.2 (Sequence and Chord matching).** `mark_completed` reads flags only after the check (D11).
- **3.5 (Hot-path budget).** Its follow-up "skip the witness read" lapses (W1). FU-4 is done here (D12). The own-key
  read (Q11) and a StickyKeys finding (M9) are candidates.
- **4.6 (Tray).** The same-process raw-input effect (Q4).
- **New feature 6 (`HoldModifier`).** Its held modifier is a sentinel event, which D2 tracks; Windows has the key down,
  so the check keeps it. Its release on `off`, on reload and at Win+L must be injected the same way.

## Type sketch

Signatures only. The `pub` items of `engine` are reachable from `lib.rs:8`, so no dead-code lint applies to them.

```rust
// src/engine/modifiers.rs (new, commit 5; pure, under the engine's #![forbid(unsafe_code)])
/// The eleven codes the engine keeps modifier flags for (moved from keys.rs:30-42).
pub(super) const MODIFIERS: [VIRTUAL_KEY; 11];
/// A set of those codes, one bit each, in MODIFIERS order (roadmap item 2.5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModifierSet(u16);
impl ModifierSet {
    pub const EMPTY: ModifierSet;
    pub const ALL: ModifierSet;    // the 11 codes
    pub const SIDED: ModifierSet;  // LShift RShift LCtrl RCtrl LAlt RAlt LWin RWin
    pub const MENU: ModifierSet;   // LAlt RAlt VK_MENU LWin RWin (replaces keys.rs MENU_KEYS, commit 9)
    /// The codes whose OS state an event of `vk` changes: a sided Shift, Ctrl or Alt and its family's
    /// generic code; a generic code and both sides of its family; LWin or RWin alone; EMPTY for any other key.
    pub fn touched_by(vk: VIRTUAL_KEY) -> ModifierSet;
    pub fn contains(self, vk: VIRTUAL_KEY) -> bool;
    pub fn union(self, other: ModifierSet) -> ModifierSet;
    pub fn without(self, other: ModifierSet) -> ModifierSet;
    pub fn is_empty(self) -> bool;
    pub fn codes(self) -> impl Iterator<Item = VIRTUAL_KEY>;
}

// src/engine/resync.rs (new, commit 5; pure)
/// When the main thread reads the modifiers from the OS between events (D4-D6).
pub(super) struct Resync {
    hiding: u64,         // starts at 1: the first drain reads (E0)
    resynced: u64,       // `hiding` at the last full read; starts at 0
    notes_seen: u64,     // the adapter's note count at the last fold
    fresh: ModifierSet,  // touched by an event since the last plan (D5)
    owed: ModifierSet,   // left out of a planned read because fresh
    input_away: bool,    // after an InputLeft reset, until a return signal or a decided event (D6)
}
impl Resync {
    pub(super) fn new() -> Self;
    pub(super) fn bump(&mut self);
    pub(super) fn event(&mut self, vk: VIRTUAL_KEY, decided: bool); // fresh |= touched_by(vk); decided => !input_away
    pub(super) fn input_left(&mut self);                            // input_away = true; bump
    pub(super) fn input_returned(&mut self);                        // input_away = false; bump
    pub(super) fn read_everything(&mut self);                       // after resync_from_os: resynced = hiding, fresh = owed = EMPTY
    /// The drain's plan: which codes to read now. Folds `notes`; reads no clock.
    pub(super) fn plan(&mut self, notes: u64) -> ModifierSet;
}

// src/engine/env.rs (commit 9)
pub trait Env {
    // now, os_key_down, foreground_app, foreground_title as today
    /// The modifier codes of key events whose hook calls have not returned yet on this thread (D17).
    fn unapplied(&self) -> ModifierSet { ModifierSet::EMPTY }
}

// src/engine/keys.rs
enum LastDown { Unknown, Passed, Suppressed, RepeatSuppressed }                       // D10, commit 4
impl KeyState {
    pub(super) fn mark_repeat_suppressed(&mut self, vk: u16);                        // commit 4
    pub(super) fn resync_modifiers(&mut self, env: &dyn Env, which: ModifierSet);    // commit 5 (sided); commit 9 (+ generic clear-only)
    pub(super) fn track_injected(&mut self, event: KeyEvent);                        // commit 7: MODIFIERS only; KS-14 skip; `active` only
    pub(super) fn clear_released_modifiers(&mut self, env: &dyn Env, own: VIRTUAL_KEY, among: ModifierSet); // commit 9: D16
    // sync_modifiers: deleted in commit 7
}

// src/engine/mod.rs
pub use modifiers::ModifierSet;
impl Engine {
    pub fn resync_if_due(&mut self, env: &dyn Env, notes: u64) -> bool; // commit 5; true if it asked the OS
    pub fn input_returned(&mut self);                                    // commit 5
    pub fn on_foreground_changed(&mut self);                             // commit 5; was (&mut self, env: &dyn Env)
    pub fn track_injected(&mut self, event: KeyEvent);                   // commit 7
    #[cfg(test)] pub(crate) fn holds(&self, vk: VIRTUAL_KEY) -> bool;    // commit 7; KeyState::is_active
}

// src/hook/main_thread.rs
thread_local! { static HIDDEN_INPUT: Cell<u64> = const { Cell::new(0) }; }   // commit 5
pub(crate) fn note_hidden_input();          // main only; a TLS add, no post, never panics (try_with)
pub(crate) fn hidden_input_notes() -> u64;  // 0 on a TLS that is gone
pub(crate) fn wake_this_thread();           // was private
impl MainState { pub(crate) fn resync_if_due(&mut self, hook: &KeyboardHook) -> bool; } // commit 5
pub(crate) fn tick_count() -> u32;          // commit 10: GetTickCount, its SAFETY line
pub(crate) const HOOK_LATE_MS: u32 = 17;    // commit 10
pub(crate) const HOOK_LATE: Duration;       // commit 10: Duration::from_millis(17)
fn note_if_awake_too_long(since_ms: u32, now_ms: u32) -> bool; // commit 10
impl MainThreadHandle { pub(crate) fn wake_for_hidden_input(&self); } // commit 11

// src/hook/keyboard_hook_callback.rs
thread_local! { static UNAPPLIED: Cell<ModifierSet> = const { Cell::new(ModifierSet::EMPTY) }; } // commit 9
pub(super) struct Unapplied(Option<ModifierSet>); // the previous set; Drop restores it (commit 9)
impl Unapplied { pub(super) fn mark(set: ModifierSet) -> Unapplied; } // EMPTY touches no TLS
fn unapplied_while_running(wparam: WPARAM, vk: VIRTUAL_KEY, decision: &Decision) -> ModifierSet; // commit 9; pure
impl Env for HookEnv<'_> { fn unapplied(&self) -> ModifierSet; }        // commit 9; reads UNAPPLIED
fn event_age_ms(now_ms: u32, stamp_ms: u32) -> u32; // commit 10; wrapping; 0 when the stamp is ahead (difference >= 2^31)
fn late_callback(age_ms: u32, ran: Duration) -> bool; // commit 10; ran >= HOOK_LATE || age_ms >= HOOK_LATE_MS

// src/hook/health.rs (commit 11)
unseen_input: AtomicU64; fn note_unseen_input(&self); fn unseen_input(&self) -> u64;

// src/hook/watchdog/mod.rs (commit 11)
trait RequestReinstall { fn request(..) -> ..; fn hidden_input(&self) {} } // default no-op; MainThreadHandle overrides

// src/hook/watchdog/decision.rs
unseen_at_last_tick: u64; pub(crate) fn unseen_at_last_tick(&self) -> u64; // commit 11
pub(crate) fn raw_pending(&mut self, now: Instant);                          // commit 13

// src/logging/counters.rs (commit 10)
Counter::HookLate; impl Counter { pub fn warns(self) -> bool }  // every arm spelled out
pub fn summary(snapshot: &Snapshot) -> Option<(log::Level, String)>;
```

## Decisions

### D1: The per-event resync leaves the key path

**What.**

- `Engine::on_key` (`mod.rs:509-530`) no longer calls `sync_modifiers` (`:518-522`). It calls
  `self.resync.event(event.vk, true)` instead: one OR into `fresh` and one store of `input_away = false`.
- `press`/`release` (`keys.rs:158-172`) already set and clear the flag of every decided event.
- The only modifier reads left on the key path belong to the check (D16).
- `on_foreground_changed(&mut self)` only bumps the hiding count. `MainState::foreground_changed` keeps its cache read
  (`main_thread.rs:283`). The callback (`keyboard_hook_callback.rs:425`) then calls `wake_this_thread()` outside the
  borrow, so the drain runs right after the current message.
- `KeyState::sync_modifiers` is deleted in commit 7.

**Why.** The 11 `GetAsyncKeyState` calls are about 4 µs of R12's 4.7 µs (ruling W2). Every key event the hook sees
already updates its own flag. The reads only repaired flags that events the hook did not see had made stale. D16
repairs the phantom-held direction at the next decided event; D4 repairs the phantom-released direction.

**Alternatives rejected.**

- Gating the per-event sync on the hiding count inside the callback: a key event would still pay for a full read
  (W2).
- Reading only the modifiers that a matching binding names: exact matching needs every family (owner decision).

**Cost on the key path.** About −4 µs per event at the 11 reads. R12, R4, R5, R9, R10, R11, R12m and R12m2 all lose
it. The OR and the store add about 1-2 ns.

### D2: The daemon's own injected modifier events are tracked

**What.**

- **Where.** In `decide_event` (`keyboard_hook_callback.rs:222-224`), a sentinel key event whose code is in
  `ModifierSet::ALL` calls `with_main(|m| m.engine.track_injected(event))`. It still returns `None`: the event passes,
  nothing is decided, nothing fires, and no check runs.
- **What it changes.** `KeyState::track_injected` sets or clears `active` only and leaves every record alone.
- **KS-14.** It skips a modifier whose key-down was swallowed (`keys.rs:105`): the user holds that key, and its
  auto-repeats must stay auto-repeats.
- **Freshness.** `Resync::event(vk, false)` marks the codes fresh and does not clear `input_away`.

**Why.**

- Ruling W2: the engine tracks "the sentinel-tagged injected ones included where they change Windows' state".
- Every injection of the daemon pairs each key-down with its key-up in one `SendInput` (`windows_actions.rs:258-287`,
  `sequence.rs:163-212`, `system_function.rs:125-155`), and "These events are not interspersed with other keyboard or
  mouse input events" ([SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)).
  A whole pair is net zero for the flags.
- A short count is not net zero: "The function returns the number of events that it successfully inserted" (same
  page). The inserted events go through the hook, so tracking them leaves the engine with what Windows has (W8).
- Feature 6 (`HoldModifier`) will hold an injected modifier by design. Windows has it down, so the check keeps it.

**Alternatives rejected.**

- A note of hidden input at each short count: it reads the OS later for something the hook already saw.
- Tracking every injected event: `TypeText` sends one pair per code unit, and only modifiers feed matching.

**Cost.** Only on sentinel modifier events: a code compare, one `with_main` and one store, about 10-20 ns. Never on a
physical event.

CLAUDE.md principle 3 says the callback "passes those events ... before it mutates any state". This decision amends
it (Q5): the event still passes and decides nothing, so no feedback loop is possible.

### D3: The generic modifier slots are set only by events (amended by W2a)

**What.**

- `VK_SHIFT`, `VK_CONTROL` and `VK_MENU` are **set** only by an event under the generic code: another program's
  injection, or the daemon's own macro `Combo("Ctrl+S")`, whose `"Ctrl"` is the generic code
  (`config/parsing.rs:19-22`).
- A generic slot is **cleared** in four ways:
  - by such an event's key-up;
  - by the check (D16), when the generic code reads up;
  - by the drain read (`resync_modifiers`, commit 9), when the generic code reads up;
  - by `release_all`.
- `resync_from_os` (commit 7) takes codes 0x10-0x12 as `held = false`, with records as for any key the OS has up.

**Why.**

- `GetAsyncKeyState(VK_SHIFT)` "gives the state of the SHIFT, CTRL, or ALT keys without distinguishing between left
  and right" ([GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate)).
- So "up" means neither side is down, and a clear-only read is always right. A read that *set* the slot would leave
  `VK_SHIFT` held after LShift's key-up until the next read: a phantom Shift family on every key. The sided slots
  carry the same information.
- Critique v2 I1: without a clear, a generic key-up the engine did not decide left the slot set through every drain
  read.

**Behaviour change (reviewer R8).**

- An injector's generic code counts for matching from its own key-down until its key-up, or until a key event finds
  neither side down in Windows.
- Known issue 78 ("Such an Alt or Win does not count for the menu mask key") now holds for Alt too: master's sync
  could set the generic `VK_MENU` slot for an Alt pressed again unseen.
- Pinned by `a_reset_never_takes_a_generic_modifier_from_the_os`, G1, R-C, R-D, R-G and C-2.

**Cost.** None on the key path beyond D16.

### D4: The hiding count, its sources and the drain step

**What.**

- `Engine` owns `resync: Resync`.
- **`Engine::resync_if_due(env, notes)`.** `which = resync.plan(notes)`. If `which` is not empty, it calls
  `keys.resync_modifiers(env, which)` and returns true.
- **`Resync::plan(notes)`.**
  1. Fold `notes`: a value different from `notes_seen` bumps `hiding`. The test is `!=`, so a smaller value counts too.
  2. If `input_away`, read nothing and change nothing else (D6).
  3. `planned` is `ALL` if `hiding != resynced` (then `resynced = hiding`), else `owed`.
  4. `read = planned − fresh`, `owed = planned − read`, `fresh = EMPTY`. Return `read`.
- **`KeyState::resync_modifiers(env, which)`.** For each code in `which` whose key-down was not swallowed (KS-14):
  - a sided code takes the OS state;
  - a generic code (commit 9) is cleared when its slot is set and the code reads up, and is never set (D3).
  - It never touches a record.
- **`MainState::resync_if_due(hook)`.** It returns false while `!hook.is_enabled()`. Otherwise it reads
  `notes = hidden_input_notes()` (from commit 11, `.wrapping_add(hook.health().unseen_input())`) and calls
  `self.engine.resync_if_due(&HookEnv::new(&self.foreground), notes)`.
- **`MainThread::drain`** (`main_thread.rs:725-744`). After its loop ends (requests, signals and due timers served),
  it makes one `with_main(|m| m.resync_if_due(hook))`. Under the borrow there are only the reads: no action, no post,
  no pump (principle 5).

**The sources.** The E-numbers name tests and mutations.

| # | Source | Where it moves the count | Thread | How the drain comes |
|---|---|---|---|---|
| E0 | Daemon start | `Resync::new()` (`hiding` 1, `resynced` 0) | main | first drain of `run` |
| E1 | Foreground change | `Engine::on_foreground_changed` | main (WinEvent) | `wake_this_thread()` after `with_main` |
| E2 | `swch on`, hook reinstall | `reset`: `resync_from_os` reads every code; `read_everything` records it | main (drain) | the reset is the read |
| E3 | Input left, then came back | `reset(InputLeft)` → `input_left`. Back: `answer_input_signals` → `MainState::input_returned` → `Engine::input_returned`, or the next decided event (D6) | main (drain, callback) | the drain that serves the signal, or the drain after the event |
| E3u | Desktop switch to an unknown destination | `note_unknown_desktop_switch` (`main_thread.rs:165-172`) calls `note_hidden_input()` | main (WinEvent) | its existing post |
| E4 | Late keyboard callback | `note_late()` at the callback's exit (D8) | main (callback) | the drain after the pump |
| E5 | Awake stretch ≥ 17 ms | `note_if_awake_too_long` before the wait, then `continue` (D8) | main (loop) | at once |
| E6 | Raw key events that no callback explains | `HealthState::note_unseen_input` + `RequestReinstall::hidden_input` (D8) | hook-watchdog | `MAIN_WAKE` |
| E7 | — | dropped with D7 (W2a) | | |
| E8 | A callback refused by `with_main` (a key event, injected-modifier tracking, a foreground change) | `note_hidden_input()` in the refusal branch (`main_thread.rs:218-224`) | main | the drain after the pump |

`ResetReason::Reloaded` moves nothing: a reload hides no event, and its reset deliberately reads no OS state
(`mod.rs:637-641`).

**Every way Windows' modifier state can change without the engine deciding the event**, re-derived per direction:

- **phantom-held**: the engine has the flag, Windows has the key up;
- **phantom-released**: Windows has the key down, the engine does not.

| # | Way | Evidence | Phantom-held repaired by | Phantom-released repaired by | Left open (row 88) |
|---|---|---|---|---|---|
| W1 | Focus in a higher-integrity window (UIPI) | "a non-elevated program cannot spy on input intended for an elevated program" ([AutoHotkey FAQ](https://www.autohotkey.com/docs/v2/FAQ.htm)); `GetAsyncKeyState` returns zero when "UI Privilege Isolation (UIPI) prevents the calling thread from accessing the foreground thread" (GetAsyncKeyState) | D16 at the next decided event; E1 | E1 on leaving the window (the read on entering it sees zeros, which only clear) | R-E4: focus in a child window of a higher-integrity process with no foreground change |
| W2 | Another desktop (lock, Ctrl+Alt+Del, UAC) | "The current desktop is not the active desktop" makes `GetAsyncKeyState` return zero (GetAsyncKeyState); `EVENT_SYSTEM_DESKTOPSWITCH` ([Event Constants](https://learn.microsoft.com/en-us/windows/win32/winauto/event-constants)) | `release_all`; D16 | E3 (no read while away, D6), E3u | R-E3: an episode with no signal at all |
| W3 | Remapping off | `keyboard_hook_callback.rs:368-373` | E2 | E2 | none |
| W4 | Hook removed | "on Windows 7 and later, the hook is silently removed without being called" ([LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)) | E2 | E2 | none |
| W5 | A call that timed out, hook kept | "If the hook procedure times out, the system passes the message to the next hook" (LowLevelKeyboardProc) | D16 | E4 (age + run ≥ 17 ms) | R-E2: `LowLevelHooksTimeout` below about 34 ms |
| W6 | Calls dropped while main was not pumping | from code; not documented | D16 | E4 (the next event waited), E5, E6 | R-E1: a stall outside every measured span; R-M1 (D8) |
| W7 | A refused callback | `main_thread.rs:216-229` | D16 | E8 | none |
| W8 | The daemon's own injection, a short `SendInput` count included | `SendInput` return value (D2); the inserted events reach the hook with the sentinel | tracked (D2) | tracked (D2) | none |
| W9 | Another program's injection, a generic code included | "Testing LLKHF_INJECTED (bit 4) will tell you whether the event was injected" ([KBDLLHOOKSTRUCT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct)); known issue 78 | decided as an event; D16 reads a generic slot through its generic code (D3) | decided as an event | none |
| W10 (corrected) | A hook *after* the daemon's swallows an event the daemon passed | "The SetWindowsHookEx function always installs a hook procedure at the beginning of a hook chain ... Each hook procedure in the chain determines whether to pass the event to the next procedure" ([Hooks Overview](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks)). So a hook installed before the daemon's, or before its last 2.4 reinstall, runs after it | a swallowed key-down (I2: an AutoHotkey remap, a Win-key blocker, a full-screen Remote Desktop client): D16 at the next decided event, on every press, auto-repeats included | a swallowed key-up the daemon passed leaves Windows with that key down, the other hook's doing: the drain read at the next count change, or the key's next event | the key-up variant |
| W11 | A key's async state lags its event | "the callback function is called before the asynchronous state of the key is updated" (LowLevelKeyboardProc) | D16 never reads the event's own key; D17 covers a callback still on the stack | D5 for the drain | R-L: a lag longer than one pump-and-drain cycle (a slow later hook); the next event is then late (E4) |
| W12 | A modifier held when the daemon starts | — | E0 | E0 | none |
| W13 (new, OQ2) | Windows changes a modifier's state with no LL hook event (StickyKeys) (unverified, U9) | "Pressing a modifier key once *latches* the key down until the user presses a non-modifier key or clicks a mouse button" ([STICKYKEYS](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-stickykeys)) | D16 | the next count change, or the key's next event | M9 settles it; a failure is a 3.5 follow-up |
| W14 (new) | A hook *before* the daemon's (installed later) swallows an event | Hooks Overview (above): the event never reaches the daemon's hook | not needed: Windows does not apply it either | not needed | none |

**Alternatives rejected.**

- An adapter-only epoch passed with each event: the engine's own sources could not be tested in scenarios.
- An `Env` method for the notes: ten `impl Env` to touch, and the drain passes the value anyway.
- `GetLastInputInfo` as a universal detector: it moves with every mouse event, and its secure-desktop coverage is
  unverified.

**Cost.**

- Key path: none.
- Main, per drain: one TLS read (plus one `Acquire` load from commit 11), one `with_main` and a compare, about 10 ns.
- Per read: up to 8 `GetAsyncKeyState` plus one per set generic slot, about 3 µs (R18), between events.

### D5: A modifier touched since the last plan is read at the next drain

**What.**

- `Resync::event` ORs `ModifierSet::touched_by(vk)` into `fresh` for every decided or tracked event: a sided code
  with its family's generic code, or a generic code with both sides.
- A plan leaves `fresh` out of what it planned and records that part in `owed`.
- The next plan reads `owed` minus that plan's `fresh`.
- Every plan clears `fresh`.
- Only a planned read (a count change, or `owed`) can leave a key out. A drain with nothing planned owes nothing.

**Why.**

- "the callback function is called before the asynchronous state of the key is updated" (LowLevelKeyboardProc). A
  drain can run microseconds after a callback returned, before Windows updated that key.
- The daemon's own `SendInput` inside `carry_out` re-enters the hook (`keyboard_hook_callback.rs:85-96`), so several
  keys can be touched in one stretch.
- The flag of a key touched since the last plan comes from its own latest event, the freshest information. Reading it
  now could take a stale state that the check never repairs in the phantom-released direction.
- No claim about the order in which Windows finishes events is needed **(unverified; not relied on)**.

The check (D16) does not use `fresh` (ruling W2a).

**Alternatives rejected.**

- A settle time: a timer per note, and a bound nobody documents.
- Skipping only the latest event's key: it misses the re-entrant case.
- Reading the key anyway: a lagging LShift key-down read as up turns its next repeat into a press.

**Cost.** One OR per event on the key path; at most one more partial read at the next drain.

### D6: No drain read while input is on another desktop

**What.**

- `reset(InputLeft)` keeps `release_all` (`keys.rs:144-151`, no OS read) and calls `Resync::input_left`, which sets
  `input_away` and bumps.
- `plan` reads nothing while `input_away`.
- Two things clear it:
  - `Engine::input_returned`, called from `MainState::input_returned` (`main_thread.rs:288-293`) for
    `SessionUnlocked` and `DesktopSwitchedBack`;
  - the next decided key event.
- The next drain then reads.

**Why.**

- CLAUDE.md principle 6 says "a lost-input reset reads no OS state". `GetAsyncKeyState` fails when "The current
  desktop is not the active desktop".
- A key event the hook sees proves that input is back: "once input is on another desktop the hook sees nothing"
  (`mod.rs:413-415`, 2.3 ruling R2) **(unverified; M2, M3)**. So a missing return signal (known issue 79) delays the
  read by one decided event, not forever.
- Under W2a, a read while away could only produce phantom-held flags, which D16 repairs before any reader. D6 is kept
  for principle 6 and because such a read would be wasted.

**Alternatives rejected.**

- Reading at once at the away signal: violates principle 6.
- Reading only at the return signal: a missing signal would block every later read.

**Cost.** One store per decided event (counted in D1).

### D7: Dropped (ruling W2a): no slow read while a modifier is held

Round 1's D7 read the modifiers once a second while the engine believed an unswallowed modifier held, to heal a
phantom-held modifier. Under W2a nothing is left for it:

- **Its trigger is repaired by D16.** "A modifier held for the engine" is the phantom-held direction, and D16 repairs
  that direction at the next decided key event.
- **Every reader runs after the check.** Matching, the key-down's mask and `mark_completed` read flags inside a
  decided key-down after the key-down check; the key-up's mask reads them after the key-up check (D11). The one read
  left is repeat detection of the event's own key (`press`, `keys.rs:160-165`). D7 could not change that key before
  its own event: the drain after the previous event ran before the period was up, or no drain ran at all while idle.
  Master's sync never read that key either.
- **It never fired for the other direction.** A phantom-released modifier is never "held for the engine".

**Removed with it:**

- `RESYNC_PERIOD`, so Q11 is moot;
- `held_since`, `last_read_at` and `KeyState::unswallowed_modifier_held`;
- the clock argument of `plan`;
- the tests `a_modifier_held_for_a_second_plans_a_full_read_once_a_second` and
  `with_nothing_held_and_nothing_due_the_clock_is_not_read`;
- the old scenario S2 and the old mutations X6 and X7.

### D8: The adapter's notes: E4, E5, E6, E8, E3u

**What.**

- **The note.** `HIDDEN_INPUT: Cell<u64>` sits next to `PENDING_SIGNALS` (`main_thread.rs:103-112`).
  `note_hidden_input()` is `try_with(|n| n.set(n.get().wrapping_add(1)))`, with no post; `hidden_input_notes()` reads
  it.
- **E8** (commit 5). `with_main`'s refusal branch (`:218-224`) calls `note_hidden_input()` after its counter bump. That
  one place covers a refused key event, a refused injected-modifier tracking and a refused foreground change.
- **E3u** (commit 5). `note_unknown_desktop_switch` bumps the note before its existing post.
- **E4** (commit 10), in `keyboard_hook_callback` (`:375-401`):
  - At entry, after `event` (`:382`), copy `event.time` and `event.flags.contains(LLKHF_INJECTED)`.
  - After the decision: `let ran = started.elapsed(); counters::record_callback(ran);`. No extra QPC read.
  - Then `let age = if injected { 0 } else { event_age_ms(tick_count(), stamp) };`.
  - `if late_callback(age, ran) { note_late() }`, where `note_late()` is `note_hidden_input();
    counters::bump(Counter::HookLate);`. No `log::` and no post (principle 1).
  - `event_age_ms` is `now.wrapping_sub(stamp)`, and 0 if the difference is 2^31 or more (a stamp ahead of the
    clock).
- **E5** (commit 10), in `MainThread::run`:
  - `let mut awake_since = tick_count()` before the loop;
  - `awake_since = tick_count()` right after `MsgWaitForMultipleObjectsEx` returns;
  - right before the wait: `let now = tick_count(); if note_if_awake_too_long(awake_since, now) { awake_since = now;
    continue; }`.
  - `note_if_awake_too_long` notes and bumps `HookLate` when `event_age_ms(now, since) >= HOOK_LATE_MS`. The
    `continue` goes back to the drain, so nothing is dispatched between the note and the read (critique M1).
  - **Changed in revision 2 (R2-15):** the clock is `GetTickCount`, not `Instant`. The read after the wait sits
    between the wake and the pump that delivers a hook call. One tick read (R17t) costs less than a QPC read there,
    and E5's 17 ms threshold needs no finer clock (the resolution argument below). Fallback: Q2.
- **E6** (commit 11):
  - `Watchdog::tick` (`decision.rs:171-222`) records
    `unseen_at_last_tick = arrived.saturating_sub(credit_before.saturating_add(answered))`, with `credit_before` taken
    at entry.
  - `Ticker::pass` (`watchdog/mod.rs:262-266`), after a tick whose value is above 0, calls
    `context.health.note_unseen_input()` (`fetch_add(1, AcqRel)`) and `context.requests.hidden_input()`.
    `MainThreadHandle` implements it with `wake_for_hidden_input()`, which posts `MAIN_WAKE` (`:574-583`).
  - `unseen_input()` is a `load(Acquire)` on main. The counter publishes no other data (principle 5).

**Why 17 ms.** "The resolution of the GetTickCount function is limited to the resolution of the system timer, which is
typically in the range of 10 milliseconds to 16 milliseconds"
([GetTickCount](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount)).

- One tick boundary shows as 15-16 ms, so 17 ms needs two, and a prompt event never counts.
- Detection holds for any timeout or stall of about 34 ms or more.
- The default timeout is 300 ms per AutoHotkey
  ([#HotIfTimeout](https://www.autohotkey.com/docs/v2/lib/_HotIfTimeout.htm)), and "The maximum timeout value the
  system allows is 1000 milliseconds" (LowLevelKeyboardProc).

**Why one exit check.** The read runs in the drain after the callback anyway (D4), so a note at entry would decide
nothing earlier. The age at exit covers both the queue wait and the run.

**The stamp's base (ruling W4): (unverified, U1).**

- `KBDLLHOOKSTRUCT.time` is "The time stamp for this message, equivalent to what GetMessageTime would return for this
  message" (KBDLLHOOKSTRUCT).
- `GetMessageTime` gives "the elapsed time, in milliseconds, from the time the system was started to the time the
  message was created" ([GetMessageTime](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagetime)),
  and its remarks prescribe subtracting "ignoring overflow".
- `GetTickCount` returns "the number of milliseconds that have elapsed since the system was started".
- Microsoft does not state that both read the same counter. Manual check M8 settles it.

**An injector's own time cannot flood the trigger.** "If this parameter is zero, the system will provide its own time
stamp" ([KEYBDINPUT](https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput)), so a nonzero
`time` is the injector's own. An `LLKHF_INJECTED` event's age is 0; only its run time counts (QPC). Remote-input tools
inject, so they are covered too.

**A false positive** costs one read between events and one `hook_late` count, never a decision (ruling W4). Hooks
installed after the daemon's run first and can make events arrive late; the cost is the same.

**Alternatives rejected.**

- `started.elapsed()` alone: it misses the queue wait.
- Reading `LowLevelHooksTimeout`: R-E2 now risks a missed read, not a stuck key.
- A post per note from the callback: a Win32 call on the key path, and the drain follows the pump anyway.

**Cost.**

- Per callback while remapping is on: two field copies; for a physical event, one `GetTickCount` (about 2-5 ns, a
  user-mode read **(unverified, U2)**, R17t) and two compares. Stop rule: Q2.
- Late events only: a TLS add and one RMW.
- Main loop, per pass: two `GetTickCount` reads. One of them lies between the wait's return and the pump.

### D9: `hook_late` and the counters line level

**What.**

- **The counter.** `Counter::HookLate`, written `hook_late`:
  - `SLOTS` goes from 11 to 12 (`counters.rs:17`);
  - it goes into `ALL` after `WatchdogDesktopCheckFailed` (`:66-78`), gets a `Display` arm (`:82-96`) and slot 11
    (`:130-145`).
- **Its doc.** "A keyboard callback ended 17 ms or more after its key event, or the main thread spent 17 ms or more
  away from its messages: Windows may have applied a key event without the daemon, so the main thread reads the
  modifiers again (roadmap item 2.5)."
- **The level.**
  - `Counter::warns(self) -> bool`, with every arm spelled out, is `false` only for `HookLate`.
  - `summary` returns `Option<(log::Level, String)>`: `None` as today; `Warn` if a warning counter is not 0 or the
    slow-callback gauge fired; otherwise `Info`.
  - `writer.rs:229-234` writes at that level. The line keeps one format with all 12 counters.

**Why.** Ruling W4: "Count it without making the minute line a warning by itself". One line format keeps log readers'
greps working.

**Cost.** None on the key path beyond the bump in D8.

### D10: A swallowed repeat with no record keeps its key-up from Windows unless Windows has the key down

**What.** `LastDown::RepeatSuppressed` means: the engine never saw this press's key-down, and it swallowed one of its
auto-repeats. In the repeat branch (`mod.rs:839-845`), a repeat with no record that the bindings suppress calls
`mark_repeat_suppressed`. The record then behaves as follows:

- **At the key-up**, it is consumed like `Suppressed`: `take_blocked` returns true, so the key-up is swallowed unless
  Windows has the key down (the witness, `mod.rs:896`).
- **Across resets**, it is kept like `Suppressed` by `resync_from_os` (`keys.rs:138`) and by `release_all`.
- **Otherwise** it is like `Unknown`:
  - later repeats are decided by the bindings, and a passing one marks `Passed`;
  - `is_blocked` and `key_down_swallowed` are false, so the KS-14 skips do not apply to it;
  - it counts for the menu mask as a key Windows may have down;
  - **the check (D16) reads it** like a key with no record, as master's sync did (`keys.rs:105` skips only
    `Suppressed`). So when Windows has it up, its next auto-repeat is decided as a press, as on master.

**Why.**

- **No lone key-up needs it.** A sided modifier can be held for the engine with no record (a read found it down). If
  its key-up then goes unseen with nothing noticed, the next physical press reads as a repeat with no record, because
  the check skips the event's own key (ruling W2a). A binding that swallows that repeat, such as the `Alt+Shift`
  blocker, leaves no record on master, so its key-up passes while Windows has the key up: a lone key-up.
- **The hole exists on master too.** Master's sync skips the event's own key (`mod.rs:522`), and commit 3's scenario
  reproduces it on `3cde1d0`.
- **`Suppressed` would be wrong** when Windows does have the key down: KS-14 would freeze its flag, and the mask would
  ignore it.
- **Why the check reads it.** The engine does not know whether Windows has such a key down, and Windows answers that.

**Alternatives rejected.**

- `mark_blocked`: wrong for the reason above.
- Reading the OS at that repeat: a read of the event's own key on the key path, beyond the witness.
- Skipping `RepeatSuppressed` in the check: a hidden release then leaves a frozen phantom that no read repairs.

**Cost.** None: one store on that rare path.

### D11: The failure mode, traced (rulings W2 and W2a)

**The modifier flags** are `KeyState::active` (`keys.rs:79`). Four things read them:

1. **Matching** at a key-down: the winner, the gesture group, the auto-blocker and the sequences (`mod.rs:677-774` →
   `matching.rs:29-102`).
2. **Repeat detection** of the event's own key: `press` returns the old flag (`keys.rs:160-165`).
3. **The menu mask**, after the record update (`keys.rs:234-239`; `mod.rs:854`, `:897`).
4. **`mark_completed`'s "held" test** (`keys.rs:185-192`; `mod.rs:789`, `:815`).

Every one runs inside a decided key event:

- (1), (4) and the key-down's (3) run after the key-down check (D16);
- the key-up's (3) runs after the key-up check;
- (2) reads the event's own key, which neither the check nor master's sync reads.

Suppression of a key-up reads no flag. `on_key_up` (`mod.rs:858-899`) uses the record (`take_blocked`, `is_passed`),
the gesture's `gesture_bound` and the witness.

**Lemma 1: no key sticks.**

- A seen key-up that the engine would swallow passes when Windows has the key down (`mod.rs:896`, kept by W1).
- An unseen key-up reaches Windows.
- So once the key is physically up, Windows has it up. No flag enters this argument.

**Lemma 2: no key-up reaches Windows alone.** A seen key-up passes only in three cases:

- **(i) its record is `Passed`.** `Passed` comes from a key-down that reached Windows (`mod.rs:846-853`), from a
  passing repeat with no record (`:843-845`), or from a key that `resync_from_os` found down (`keys.rs:136-137`). A
  hidden key-up after it can only be followed, before this key-up, by a key-down. A seen one is a "repeat" for a stale
  flag; `is_passed` makes it pass, so Windows has the key down. An unseen one means Windows has it down.
- **(ii) it has no record and no gesture.** After D10 every seen key-down leaves a record. So no record at a key-up
  means no seen key-down since the last record was consumed or reset. The key-down that Windows applied was then
  unseen, and Windows has the key down.
- **(iii) Windows has the key down.** Then the key-up is not lone.

The check, the drain read and `track_injected` write flags only, never records. No flag enters this argument.

**The check is a subset of master.**

- At a key-down, each write of the check sets `active[m] = false` where `active[m]` is set and Windows has `m` up. It
  skips the event's own key, swallowed modifiers and unapplied codes.
- Master's sync wrote `active[m] = os(m)` for every `m` except the own key and swallowed ones, at the same event.
- So every write of the check is a write master made at that event, with the same value. At a key-up, the check
  writes a subset of master's key-up writes.
- The check therefore creates no state that master could not reach at the same event. The drain read writes what
  master's `on_foreground_changed` sync wrote (`mod.rs:613-615`), at more moments, minus the generic sets.

**The KS-14 rule.** The check, `resync_modifiers` and `track_injected` all skip `key_down_swallowed`
(`keys.rs:153-156`), as `sync_modifiers` did (`:105`). Such a key's flag changes only by its own events and
`release_all`, so master's behaviour for it is unchanged. `RepeatSuppressed` is read by all three (D10).

**Direction A, phantom-held** (the engine has the flag, Windows has the key up: a key-up the engine did not decide, or
a passed key-down that Windows did not apply, W10). The check repairs it at the next decided key event, before any
reader, with three exceptions:

- **the event's own key**, as on master: a modifier released unseen and pressed again with no decided event in between
  reads as an auto-repeat (row 63's kind). D10 keeps its release correct.
- **a swallowed modifier** (KS-14, as on master; row 78).
- **a key whose callback is still on the stack** (D17): its flag is the state Windows is about to have.

The drain read also clears sided flags at a count change.

**Direction B, phantom-released** (Windows has the key down, the engine does not: a key-down the engine did not
decide). Two things repair it: the drain read at the next count change (sided slots only; a generic slot is never
set), or the key's own next event (its auto-repeat, a first press for the engine, or its key-up). Until then (row 88)
the user sees four effects:

1. A binding with that modifier misses, and the binding without it acts. With that Shift, CapsLock switches the
   layout instead of toggling caps lock.
2. The `Alt+Shift` blocker misses when the unseen key is its partner, so Windows sees Alt+Shift and switches the layout.
3. While that Alt or Win is down in Windows, a key the daemon swallows gets no menu mask key. At its release the menu
   bar activates or the Start menu opens (KS-12's symptom).
4. The key's next auto-repeat counts as a new press, so a binding on that key alone acts once more (row 76's kind).

No key sticks and no key-up reaches Windows alone (Lemmas 1 and 2).

**Residual cases of direction B** (row 88; the W-table):

- R-E1: a stall outside every measured span;
- R-E2: `LowLevelHooksTimeout` below about 34 ms;
- R-E3: a secure-desktop episode with no signal;
- R-E4: focus in a child window of a higher-integrity process;
- R-M1: events dispatched in the same awake stretch after a stall, decided before the drain (D8);
- R-L: a lag longer than one pump-and-drain cycle;
- W10's key-up variant;
- W13 (StickyKeys, unverified).

**Owner summary** (the results analyst copies it into `results.md`): After this lane, a modifier can be wrong for the
daemon in only one direction: pressed where the daemon cannot see it, with nothing the daemon notices afterwards.
Until the next foreground change, lock return, `swch on`, refused or late callback, watchdog gap, or that key's next
auto-repeat:

- Shift+CapsLock can switch the layout instead of toggling caps lock;
- Alt+Shift can switch the layout although the blocker should stop it;
- the menu bar or Start can open at an Alt or Win release;
- a binding on that key can act once more.

A modifier released where the daemon cannot see it, or one that another program's hook swallows, is corrected at the
next key. No key sticks.

### D12: The watchdog's first wake after idle starts the ticks only (ruling W3)

**Q6 of item 2.4, as far as the evidence goes.** Microsoft does not document whether raw input is produced before or
after the LL hook chain. LowLevelKeyboardProc says "The system calls this function every time a new keyboard input
event is about to be posted into a thread input queue"; the
[Raw Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input) is silent on hooks.
Prior art:

| Source | Claim | How read |
|---|---|---|
| [input-leap #1458](https://github.com/input-leap/input-leap/discussions/1458) | "WH_KEYBOARD_LL -> Raw Input -> [OS Update] -> WH_KEYBOARD" | fetched |
| [GameDev.net 2009](https://gamedev.net/forums/topic/550264-dinput-and-global-key-hooks-vistawin7/) | "WH_KEYBOARD_LL does indeed block Raw Input / WM_INPUT messages"; disputed in the thread | search summary (403) |
| [PureBasic forum](https://www.purebasic.fr/english/viewtopic.php?t=66655) | "Raw Input callback will never receive any messages" with an LL hook that blocks | search summary |
| [AutoHotkey t=6073](https://www.autohotkey.com/boards/viewtopic.php?t=6073) | "if you block keystrokes via a keyboard hook, you won't get the WM_INPUT messages" | search summary |
| [NotEnoughHotkeys](https://github.com/VollRahm/NotEnoughHotkeys) | Win, Alt+Tab and lock keys "are sent before RawInput gets anything" | fetched |
| [Muratori, "Five-Second Stall"](https://caseymuratori.com/blog_0006) | the process's own LL hook "never got called" while its raw-input window had focus; `RIDEV_NOHOTKEYS` fixed it | fetched; a same-process, focused-window effect |
| [AltSnap PR #773](https://github.com/RamonUnch/AltSnap/pull/773) | the hook stops "as soon as the Menu has focus" if raw input is registered | fetched; same effect |

**Verdict (unverified; probable):** raw input comes after the chain, and there is none for a swallowed key.

- The burst's first raw event can still meet the next event of the same HID report. The class driver hands the system
  a range of packets at once (`KeyboardClassServiceCallback(..., InputDataStart, InputDataEnd, ...)`,
  [MS Learn](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/kbdmou/nc-kbdmou-pservice_callback_routine));
  the timing is **(unverified)**. So R16 keeps its construction and is gated.
- The 2.4 diagnostics (`bench-dev-diag.md:5-30`) put the cost in the woken thread's first pass. It is not the
  decision (D-B 53.6 µs), not priority (R16/high 58.5 µs), not a cold thread (D-D 79.7 µs), and it lasts past 200 µs
  (D-C 16.5 µs).
- That pass's user32 calls are `pump` (`watchdog/mod.rs:786`), `drain` (`:1014`) and the next wait.

**What.**

- **`Watchdog::raw_pending(now)`** (pure, `decision.rs`). When not ticking: `next_tick = now + TICK`,
  `quiet_since = now`, `credit = 0`. `last_raw_at` is left for the first tick's drain.
- **`Ticker::woke_idle(now, context)`**: `stats.wakes += 1`, `raw_pending(now)`, `arm(now, context)`.
- **`run_loop`** (`:703-749`) keeps the wait's mask. It takes the deferred path when `woke == WAIT_OBJECT_0 + 1`, the
  mask contains `source.batched_wake()` and `context.health.hook_installed()`: `woke_idle`,
  `drained = Drained::default()`, `continue`, with no pump and no drain. "WAIT_OBJECT_0 + nCount — New input of the
  type specified in the dwWakeMask parameter is available in the thread's input queue"
  ([MsgWaitForMultipleObjectsEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex)).
- **No spin.** The next wait is a ticking wait without the batched bit. With `MWMO_INPUTAVAILABLE`, a `WM_QUIT`, a sent
  message or a system event ends it, and that pass pumps. Two deferred wakes in a row are impossible.
- **The raw input stays queued** for the first tick. That is the 2.4 assumption, pinned by
  `a_message_wait_leaves_out_a_pending_message_its_wake_mask_does_not_name` (`:2526`) and manual check 47.
- **The synthetic source's batched bit** is `QS_POSTMESSAGE`, which `WM_QUIT` shares. An idle synthetic watchdog
  therefore stops at its first tick (≤ 250 ms, inside `STOP_TIMEOUT`, `:430`). The keyboard source stops at once.
- **The 2.4 rules are kept:** I3 (deferral only with the hook installed); the silence starts at the wake; the first
  count lands in the first tick's interval and becomes evidence from the second tick.
- **Loop-model expectations that move by one tick** (meaning unchanged, Q3):
  - `a_batch_read_at_a_tick_counts_all_its_events` (`:1631`): `(250, 8)` becomes `(250, 9)`;
  - `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request` (`:1813`): the last read moves from 749 to 750;
  - `the_first_raw_key_event_after_idle_wakes_the_watchdog` (`:1607`): "no pass in `2250..5000`" becomes
    `2500..5000`.
- **`Drive`** (`:1354-1519`) gains `wake(ms)`: `woke_idle`, a counted pass, the timer calls and no read.

**Alternatives rejected.**

- A timer-only idle poll at 1 s: 600 wakes per 10 idle minutes against check 39's W ≤ 100 (fallback only, Q1).
- A stop event in the handle array: it changes 2.4's shutdown protocol.
- Priority changes: R16/high equals R16.
- Raw input on main: rejected by 2.4 D1 (R15).

**Cost (critique v2 M3).**

- At the first key after idle the watchdog makes one wait return, one `SetWaitableTimer` and one wait entry, and
  reads nothing until the tick.
- `last_raw_at` is the time of the drain that read the newest raw event (`decision.rs:142-143`). For a key press or a
  shortcut that drain is a tick in both designs, so the end of ticking (`decision.rs:227-234`) and the wake count stay
  the same.
- **Only a burst made of a single raw key event gets one more tick**, such as a key-up whose key-down came 2 s or more
  earlier, because its one read moves from the wake to the first tick.
- `errors.md:215` keeps "about 10 wakes for an isolated shortcut" and "N of about 1000". The module Cost doc
  (`watchdog/mod.rs:23-42`) says the same. No idle wake is added.

### D13: Bench rows and comparison rules (ruling M5: one session per comparison)

**Trees** (the tester builds each in its own scratch worktree with its own `CARGO_TARGET_DIR`):

- **P**: commit 2 of the lane. It is `master` plus the bench rows. `git diff --stat master <commit 2> -- src` lists
  only `src/bench.rs` and `src/platform/timers.rs` (a doc line), so P runs master's engine and adapter byte for byte
  (Q10).
- **N**: commit 2 with the one line `self.keys.sync_modifiers(env, Some(event.vk));` (`src/engine/mod.rs:522`)
  removed, uncommitted. It is the diagnostic probe tree.
- **H**: the lane's HEAD.

**Runs.**

- The tester makes nine runs in one session, interleaved P1 N1 H1 P2 N2 H2 P3 N3 H3, all in release (`tester.md` §4b
  command).
- Per row it takes the median of the three p50s (of the three means for R1) and the largest p99.
- **Every gate below compares numbers from these nine runs only.** The developer's runs (P and N once at commit 2, H
  once at the end) set the expected values and the stop rules.

| Row | What | n | Gate |
|---|---|---|---|
| R12 | unchanged construction | 2000 | H p50 ≤ N p50 + R17t p50 + 100 ns; H p99 ≤ 1.5 × N p99 + 2 µs |
| R12m (new) | the key-down callback of the unbound A with LCtrl held for the engine. Before each sample, an untimed callback of the daemon's own injected LCtrl key-down (sentinel, D2); after it, untimed A up and injected LCtrl up | 2000 | H p50 ≤ N p50 + 1 × R17a p50 + R17t p50 + 100 ns |
| R12m2 (new) | the same with LCtrl and LShift injected | 2000 | H p50 ≤ N p50 + 2 × R17a p50 + R17t p50 + 100 ns |
| R4 | unchanged | 10000 | H p50 ≤ N p50 + R17t p50 + 100 ns |
| R1 | unchanged (the engine with `BenchEnv`, whose reads are a `RefCell` lookup) | 50000 | Δ = P mean − N mean; **H mean ≤ N mean + 0.5 × Δ**. A revert of the removal on the engine path puts H at about P and fails. If the tester's Δ is below 0.010 µs, R1 is reported as unable to tell a revert apart, not as a breach (stop rule (b) makes this unlikely) |
| R11 | unchanged (W1) | 2000 | "unchanged" means the witness term: H (R11 − R12) p50 ≤ P (R11 − R12) p50 + 0.2 µs. R11 itself falls with R12 |
| R16 | unchanged construction. Doc: "the event right after the burst's first raw event: the key's own callback if raw input comes before the hook call, the next event of the same report if after" | 1000 | **Provisional:** H p50 ≤ min(H R12 p50 + 20 µs, 0.5 × P R16 p50); p99 reported |
| R17t (new) | `GetTickCount()`: 1000 calls per sample | 2000 | evidence; stop rule (c) and Q2 |
| R17a (new) | `GetAsyncKeyState(VK_F13)` (read only): 1000 calls per sample | 2000 | evidence; stop rule (d) |
| R18 (new, commit 5) | the main thread's read: an untimed `note_hidden_input()`, then a timed `with_main(\|m\| m.resync_if_due(get_hook()))` on the bench thread as main | 2000 | evidence; sanity bound p50 ≤ 8 × R17a p50 + 0.5 µs (same H run) |
| R2, R3, R5-R10, R13-R15 | unchanged | | existing rules (2.2 D10, 2.3 D14, 2.4; `CLAUDE.md:69-73`). R5, R9 and R10 fall with R12. R7 and R13 carry E5's two `GetTickCount` reads per pass and the drain's check |

On P and N, the injected modifiers of R12m and R12m2 pass untouched. On P, the per-event sync reads all eleven codes,
so R12m there equals R12's key-down. On H, D2 tracks them, and the timed A key-down pays one read (R12m) or two
(R12m2). The bench machine's real keyboard reads up, so the check clears them, and each sample sets them again.

**Developer probes after commit 2** (work tree only, uncommitted, into
`D:/tmp/ks-orch/p2-2.5-latency-trim/bench-dev-diag.md`):

1. **P and N, once each, back to back.** Record R1 (mean), R4, R11, R12, R12m, R12m2, R17t and R17a for both, then
   restore the line. Planning values: N R12 0.5-1.0 µs; R17a about 0.36 µs; Δ about 0.02-0.04 µs. Stop rules:
   - (a) N R12 p50 > 0.5 × P R12 p50: stop after commit 2 and report; the reads are not the cost the ruling names.
   - (b) Δ < 0.010 µs: stop and report; the R1 gate cannot tell a revert apart.
   - (c) R17t p50 > 100 ns: no stop; apply Q2's fallback.
   - (d) R17a p50 > 0.6 µs: stop and report; a held modifier would cost more than the ruling accepted.
2. **D-R16d.** A bench-local helper thread with its queue and a `WaitableTimer`:
   - it waits with `MsgWaitForMultipleObjectsEx(&[timer], INFINITE, QS_ALLINPUT, MWMO_INPUTAVAILABLE)`;
   - on input it reads `Instant::now()`, sets the timer for 5 ms and waits again with the posted bit left out;
   - at the timer it pumps everything and goes idle.

   Each sample posts one message to it, times `call_keyboard_callback(VK_A, WM_KEYDOWN)` at once, calls the key-up
   untimed and spins 10 ms; n = 1000. **Stop rule:** if D-R16d p50 > R12 p50 + 20 µs (same run), stop after commit 11
   and report (Q1).
3. **After commit 13**, the developer runs R16 at HEAD. If it misses the provisional bound, stop and report the
   numbers (ruling W3).

**The results analyst states** in `results.md` (rulings, Latency, round 2):

- R12, R12m and R12m2: P and H, each against N;
- R1 and R4: P and H;
- R11's witness term: unchanged;
- R16: P and H;
- R18, and how often it runs: once per hiding-count change, plus at most one partial read of the codes left out as
  fresh;
- the W2a trade in one line: zero reads with nothing held, one per held modifier at a key-down;
- D11's owner summary.

### D14: The Phase 2 book-sync leftovers

- `CHANGELOG.md:744-745`: "a `Hold` runs them on its timer thread as before and queues the others" becomes "a `Hold`,
  and a `Tap` that waits for its DoubleTap window, runs them on the main thread when its gesture timer fires (roadmap
  item 2.2), and queues the others".
- `CHANGELOG.md:116`: "(a Hold whose timer thread could not be started, so that it never fired)" becomes "(a Hold
  whose gesture timer could not be set, so that it never fired)".
- `src/platform/timers.rs:1-3`: "the action executor's `Delay` steps today, the gesture timers of roadmap item 2.2
  later" becomes "the action executor's `Delay` steps (roadmap item 2.1), the main thread's gesture timers (item 2.2)
  and the hook watchdog's ticks (item 2.4)".

### D15: The scenario harness models the drain, noticed and unnoticed hidden events, later hooks and generic codes

**What** (`src/engine/testing.rs`):

- **Commit 5: the drain.**
  - `Scenario` gains `notes: u64` and `fn drain(&mut self)`, which calls
    `self.engine.resync_if_due(&self.env, self.notes)` when remapping is on.
  - `with_bindings` drains once at construction: the daemon's first drain, E0, runs before any key event. So no
    scenario needs a prelude.
  - It drains after every step at which the daemon's main thread would reach its drain: a key event while on,
    `foreground_changed`, every input signal, `remapping_on`, `hook_reinstalled`, `reload`, an `advance_ms` that fired
    a timer, `hidden_input_noted`, the noticed unseen steps and `daemon_injects_*`.
  - It does not drain after `*_unnoticed`, `advance_ms_stalled`, `lose_timers` or `remapping_off`.
- **Commit 5: the steps.**
  - `up_unseen`/`down_unseen` (`:346-380`) add a note and drain: "an event the hook did not see, which a source
    noticed (roadmap item 2.5)".
  - New `up_unnoticed`/`down_unnoticed`: the same OS change, with no note and no drain.
  - New `hidden_input_noted()`: `notes += 1`, a drain and a step. It stands for E3u, E4, E5, E6 and E8.
  - `foreground_changed` (`:382-395`) calls `on_foreground_changed()` and drains.
  - `input_signal` (`:427-446`) resets as today, calls `engine.input_returned()` for a return signal (as
    `answer_input_signals` does) and drains.
- **Commit 7: generic and injected steps.**
  - New `down_generic(name)`, `up_generic(name)` and `up_generic_unnoticed(name)`. The event carries the generic code
    (`"Shift"` → `VK_SHIFT`, `"Ctrl"` → `VK_CONTROL`, `"Alt"` → `VK_MENU`). The modelled OS applies a passed one to
    the left key, since which side does not matter to any scenario (U4). `held` and the lone-key-up check follow the
    left key.
  - New `daemon_injects_down(key)` and `daemon_injects_up(key)` model the daemon's own sentinel event: they call
    `engine.track_injected`, the OS applies the event, `held` follows it, there is no lone or symmetry check, and the
    step drains.
- **Commit 8: later-hook steps.** New `down_swallowed_later(key)`, `repeat_swallowed_later(key)` and
  `up_swallowed_later(key)`. The engine decides the event as `down`/`repeat`/`up` do. When it passes, a hook after the
  daemon's swallows it: the modelled OS does not change and the lone-key-up check does not apply. When the engine
  swallows it, the step behaves as `down`/`repeat`/`up`. `held` follows the physical key.
- **Module docs** (`:1-71`) describe all of this.
- **`properties.rs`.**
  - Commit 7 adds `Mode::Unnoticed`: the Unlock steps, with `down_unnoticed`/`up_unnoticed` in place of the unseen
    steps, checking `check_nothing_stuck` and `check_no_lone_key_up` (D11).
  - `presses_unseen`, `loses_key_ups` and the signal branch (`:292`) include it. The existing modes draw exactly the
    same numbers, so their seeds keep their scripts.
  - Commit 9 gives the mode the later-hook case (Q7): 1 press in 10 is swallowed by a later hook, and its down, repeats
    and up use the `_swallowed_later` steps.
  - New test: `no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed`.

**Why.**

- The daemon notices almost every hidden event before the next key (the W-table). So the existing scenarios and
  properties keep their meaning with noticed unseen steps: the read is eager, like master's sync.
- Unnoticed steps pin what row 88 declares.
- The later-hook and generic steps reproduce critique v2's I2 and I1.
- The new mode tests D11's lemmas on random scripts.

**Alternatives rejected.**

- Generating a source step before each unseen step: it changes every Unseen and Unlock seed.
- A per-test prelude instead of the construction drain: the old cut's choice, which models E0 less faithfully.

### D16: The check at a decided key event (ruling W2a)

**What.**

- **`KeyState::clear_released_modifiers(env, own, among)`.** For each code `m` in `among` such that `active[m]` is set,
  `m != own`, `!key_down_swallowed(m)` and `m` is not in `env.unapplied()` (D17; read once, only if a code is left):
  if `!env.os_key_down(m)`, it sets `active[m] = false`.
  - A generic code reads its generic state.
  - Nothing is set, and no record is touched.
- **At a key-down**, `Engine::on_key` calls `clear_released_modifiers(env, vk, ModifierSet::ALL)` before
  `on_key_down`, which holds every reader of a flag at a key-down (`mod.rs:677-854`).
- **At a key-up**, `on_key_up` calls it with `ModifierSet::MENU` only when `decision.suppress`, right before
  `mask_if_swallowed_under_alt_or_win`. That is the only reader at a key-up (`mod.rs:897`; `gestures.release` at
  `:869` takes no `KeyState`).
- **The rule for `RepeatSuppressed`**: the check reads it (D10).

**Why.**

- **What it repairs.** It repairs four cases at the event that would read the stale flag, before matching:
  - I2: a passed modifier that a later hook swallowed (W10), for as long as it is held;
  - I1: a generic phantom (D3);
  - round 1's D7 case: a phantom-held modifier after an unnoticed key-up;
  - M1(b): the blocker swallowing a whole Alt or Shift press on a phantom-held partner.
- **The reads are safe there.** Master's per-event sync read every modifier except the event's own key at every event
  (`keys.rs:102-110` called at `mod.rs:522`). If an earlier event's key were not yet applied at the next decided
  event, every held-modifier shortcut would have missed on master since `7cb3521`. The one exception is a decided
  event that runs while an earlier event's callback is still on the stack; D17 skips those keys. A modifier is never
  skipped only because it is fresh in D5's sense (ruling W2a).
- **Clear-only on a generic code is right.** "This gives the state of the SHIFT, CTRL, or ALT keys without
  distinguishing between left and right" (GetAsyncKeyState), so "up" means neither side is down.
- **Own key and KS-14 skips.** These are the ruling's rules, as on master (`keys.rs:96-97`, `:105`).
- **The key-up check covers MENU codes, and only for a swallowed key-up**, because the mask is the only reader there
  and reads only those codes after a swallowed event. So Ctrl+J pays one read, not two.

**Alternatives rejected.**

- Reading every modifier: master's eleven reads per event.
- Skipping fresh modifiers: this leaves I2 open whenever no drain ran between the modifier and the shortcut key
  (ruling).
- Setting from the read: D3's phantom.
- Checking at every key-up: it doubles a shortcut's cost for nothing.
- A drain-only repair: the drain runs after the shortcut key is decided.

**Cost** (accepted under W2a).

- Per decided key-down: an 11-slot scan (about 3 ns), plus one `GetAsyncKeyState` (R17a, about 0.36 µs) per modifier
  held for the engine whose key-down was not swallowed. That is 0 in ordinary typing, 1 for Ctrl+J and 2 for
  Ctrl+Shift+X, against 11 on master.
- Per swallowed key-up: a 5-slot scan and one read per held Alt or Win.
- A passing key-up: nothing.
- No allocation and no lock.

### D17: A key decided inside another key's callback does not clear that key (W2a, re-entrant decided call)

**Does such an event exist? It is not documented either way (U7).**

- The daemon's hook passes an event by calling the next procedure: "A hook procedure passes an event to the next
  procedure by calling the CallNextHookEx function" (Hooks Overview).
- That procedure belongs to another process. "This hook is called in the context of the thread that installed it. The
  call is made by sending a message to the thread that installed the hook" (LowLevelKeyboardProc).
- For injected input, "the context switches back to the process that installed the hook and it is called in its
  original context. Then the context switches back to the application that generated the event" (same page).
- `carry_out`'s `SendInput` re-enters the hook (`keyboard_hook_callback.rs:85-96`).
- So while the main thread waits inside `CallNextHookEx` or `SendInput` for another process's hook, it may be handed a
  *decided* event: another program's injection, or input that another hook injects from its own callback.
- That inner event would run before Windows applied the outer event, because "the callback function is called before
  the asynchronous state of the key is updated".

**What.**

- From the decision of a **passed key-down of a tracked modifier** until its callback returns (its `carry_out` and its
  `CallNextHookEx`), the adapter marks `ModifierSet::touched_by(vk)` in `UNAPPLIED`.
- The mark is a guard, `Unapplied::mark`. It replaces the TLS value with its union and restores the previous value on
  drop, so nested marks work.
- The guard is bound to a named local declared before the `match` on the decision, so it drops after
  `CallNextHookEx` on the pass path and at `return LRESULT(1)` on the other.
- `unapplied_while_running(wparam, vk, &decision)` is the pure rule: `touched_by(vk)` for a key-down that is not
  suppressed, otherwise EMPTY.
- `HookEnv::unapplied()` reads the TLS. `Env::unapplied` defaults to EMPTY. The check (D16) skips those codes.
- Only a passed modifier key-down needs the mark. A swallowed one is skipped by KS-14; a key-up's flag is already
  clear, and the check only clears; a non-modifier is never checked.

**Why.**

- Ruling W2a: "skips those keys, as D5 does".
- Without the mark, such an inner event would read the outer modifier as up and clear it while Windows is about to
  have it down. Master does the same: its sync overwrote the flag with the same stale "up". So D17 is strictly better
  than master.

**Alternatives rejected.**

- Skipping the previous decided event's key: it fails the repeat case of I2 (an LAlt repeat, then J).
- An engine-side nesting count: it needs a second `with_main` after every callback.
- Answering "down" from `HookEnv::os_key_down` for unapplied keys: it would change the witness and the KS-05 read too.

**Cost.**

- For a passed modifier key-down: two TLS accesses, about 1-2 ns.
- The check reads the TLS only when a held modifier is left to read.
- Ordinary typing: nothing.

## Win32 API choices

| Purpose | `windows` path | Feature (present?) | Thread | Errors | Doc link |
|---|---|---|---|---|---|
| Event age (E4) and awake stretch (E5) | `windows::Win32::System::SystemInformation::GetTickCount() -> u32` (`pub unsafe fn`, windows-0.62.2 `SystemInformation/mod.rs:194`), wrapped by `main_thread::tick_count` | `Win32_System_SystemInformation`: **absent; add** to `Cargo.toml:41-58` (same crate; `Cargo.lock` unchanged) | main (keyboard callback, physical events only; the main loop); bench | none; it wraps after 49.7 days, handled by `wrapping_sub` and the signed rule | https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount |
| The event's stamp and injected flag | `KBDLLHOOKSTRUCT { flags: KBDLLHOOKSTRUCT_FLAGS, time: u32 }` (`WindowsAndMessaging/mod.rs:4251-4255`), `LLKHF_INJECTED` (`:4409`), `KBDLLHOOKSTRUCT_FLAGS::contains` (`:4262`) | `Win32_UI_WindowsAndMessaging` (yes) | main | — | https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct, https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getmessagetime, https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput |
| The check (D16) and the drain read (D4) | `windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState` (existing, `HookEnv::os_key_down`, `keyboard_hook_callback.rs:178-184`) | `Win32_UI_Input_KeyboardAndMouse` (yes) | main: in the keyboard callback (D16) and in the drain under `with_main` (a read, no pump) | "The return value is zero if the call fails": not the active desktop, UIPI, no `DESKTOP_HOOKCONTROL`/`DESKTOP_JOURNALRECORD` access. Zero reads as up, which only clears (D16) or matches the zeros master read (drain) | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate |
| Passing an event on (D17 evidence; no new call) | `CallNextHookEx` (existing, `keyboard_hook_callback.rs:400`) | yes | main | as today | https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks, https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc |
| Wake after a foreground change; the watchdog's wake | `PostThreadMessageW` (existing: `wake_this_thread` `main_thread.rs:182-190`, `MainThreadHandle::wake` `:574-583`) | `Win32_UI_WindowsAndMessaging` (yes) | main (WinEvent, outside the borrow); `hook-watchdog` → main | ignored as today: a lost wake loses no note, and the next drain folds the count. "a limit of 10,000 posted messages per message queue": one post per foreground change or gap tick | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postthreadmessagew |
| The watchdog's wait and its "input" result | `MsgWaitForMultipleObjectsEx` (existing, `watchdog/mod.rs:714-721`), `WAIT_OBJECT_0 + 1` | yes | `hook-watchdog` | `WAIT_FAILED` as today | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex |
| The first tick after a deferred wake | `platform::timers::WaitableTimer::set` → `SetWaitableTimer` (existing) | `Win32_System_Threading` (yes) | `hook-watchdog` | counted and retried as today (`watchdog/mod.rs:334-347`) | https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimer |
| W8 evidence (no new call) | `SendInput` | yes | — | short count | https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput |
| W13 evidence (no call) | `STICKYKEYS` | — | — | — | https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-stickykeys |

**Unverified claims, and what settles them:**

| # | Claim | Settled by |
|---|---|---|
| U1 | `KBDLLHOOKSTRUCT.time` and `GetTickCount` share a base | M8 (`hook_late` stays 0 in normal typing); M4 (a held callback counts) |
| U2 | `GetTickCount` is a user-mode read | R17t; stop rule (c), Q2 |
| U3 | The LL hook sees no events while input is on another desktop | M2, M3 (roadmap checks 36 and 28) |
| U4 | An injected generic `VK_SHIFT` reaches the hook under `VK_SHIFT` and is applied to a side (OQ1) | not checked (ruling): it sets only how likely I1's trigger is. D3 and D16 read the generic code, which is right either way |
| U5 | Raw input comes after the hook chain (Q6 of 2.4) | M5 (roadmap checks 43 and 44) |
| U6 | The keys of one HID report are handled back to back | not needed: R16 keeps its construction |
| U7 | A decided event can run on the main thread while another event's callback is still on its stack (inside `CallNextHookEx` or `SendInput`) | no practical manual check. D17 makes the engine right whether or not it happens, at no cost to ordinary typing |
| U8 | Windows applies an event's key state before it calls the hooks for the next event | relied on by master's per-event sync since `7cb3521`: a lag would make every held-modifier shortcut miss. M10 exercises it, as does every shortcut in daily use |
| U9 | StickyKeys changes a modifier's state with no LL hook event (OQ2) | M9; a failure is a 3.5 follow-up in `impl.md` |
| U10 | Input that reaches this machine as a Remote Desktop server carries stamps on another tick base (OQ3) | M8's note: a wrong stamp costs reads between events and an `[INFO]` line, nothing more |

**Elevation and UIPI.**

- The release daemon is elevated, so UIPI does not limit its reads.
- A debug daemon that is not elevated reads zeros while an elevated window has the foreground. No decided event comes
  from there (W1), and the drain's zeros only clear.
- No pipe, token, registry or hook-registration change. Nothing injects input.

## Thread map

| Code path | Thread | Change |
|---|---|---|
| `keyboard_hook_callback`, physical event | main (inside `PeekMessageW`) | −11 reads. + D16: one read per held unswallowed modifier at a key-down, per held Alt/Win at a swallowed key-up. + D17 mark for a passed modifier key-down. + E4: two copies, one `GetTickCount` at exit, a note when late |
| `keyboard_hook_callback`, sentinel modifier event | main | + `with_main`, one flag store (D2) |
| `foreground_changed` WinEvent | main | an engine bump instead of 11 reads; + `wake_this_thread()` outside the borrow |
| `desktop_switched` WinEvent | main | an unknown destination also notes (E3u) |
| `with_main` refusal | main | + TLS note (E8) |
| `MainThread::drain` | main | + the read step after requests, signals and timers |
| `answer_input_signals` | main (drain) | + `engine.input_returned()` via `MainState::input_returned` |
| `MainThread::run` | main | + E5: two `GetTickCount` per pass; a note and `continue` |
| `Ticker::pass` | `hook-watchdog` | + E6: one RMW and one post |
| `run_loop` | `hook-watchdog` | D12 deferral |
| `report_counters` | `log-writer` | the line's level |
| Bench R12m, R12m2, R17t, R17a, R18 | bench thread (as main for R12m, R12m2, R18) | reads and the extern callback only |

No install, uninstall or timer moves. Every hook stays on main. No new thread, lock, `static mut` or `Relaxed`.
`UNAPPLIED` and `HIDDEN_INPUT` are main-thread TLS `Cell`s, like `PENDING_SIGNALS`.

**Effect on the key-to-action path** (owner decision of 2026-10-19):

| Event | Master | After |
|---|---|---|
| Ordinary key (nothing held), either direction | 11 `GetAsyncKeyState` | 0. + 11-slot scan at a key-down (about 3 ns), one OR and one store, two copies, one `GetTickCount` and two compares |
| Key-down with k unswallowed modifiers held | 11 | k |
| Swallowed key-up | 11 + witness | witness + one read per held Alt/Win |
| Passed modifier key-down | 11 | k for the others held, + two TLS accesses (D17) |
| Sentinel modifier event | 0 | `with_main` + one store |
| Allocations, locks, posts on the path | 0 | 0 |
| Main loop, wake to pump | one wait return | + one `GetTickCount` |

## Touch set

| File | Change | Commit |
|---|---|---|
| `src/platform/timers.rs` | doc `:1-3` (D14) | 1 |
| `CHANGELOG.md` | `:116`, `:744-745` (1); Added R12m/R12m2/R17t/R17a (2); **Fixed** D10 (4); Added R18 (5); Changed: the modifier resync and generic slots (7, extended in 9, 10 and 11); Added `hook_late` and Changed line level, with example line `:120` (10); Changed: the watchdog's first wake, R16 gated (13) | 1-13 |
| `Cargo.toml` | feature `Win32_System_SystemInformation` | 2 |
| `src/bench.rs` | `call_keyboard_callback` sets `time: GetTickCount()` (its SAFETY line); new `call_injected_callback` with the sentinel; rows R12m, R12m2, R17t, R17a; docs `:20-82` (2); R18 (5); R16 doc (13) | 2, 5, 13 |
| `src/engine/modifiers.rs` (new) | `MODIFIERS` (moved), `ModifierSet`; tests | 5 |
| `src/engine/resync.rs` (new) | D4-D6 state and plan; tests | 5 |
| `src/engine/keys.rs` | `RepeatSuppressed`, `mark_repeat_suppressed`, `take_blocked` (4). `resync_modifiers` (sided) (5). `sync_modifiers` deleted; generic codes out of `resync_from_os`; `track_injected`; module docs `:1-17`, `:25-29`; tests that call `sync_modifiers` (`:286`, `:316`, `:484-492`, `:521`, `:568`, `:681-705`, commit 4's test) move to `resync_modifiers` with sided expectations; `:600-608` drops the `VK_SHIFT` expectation (7). R-G (8). `clear_released_modifiers`; generic clear-only in `resync_modifiers`; `MENU_KEYS` → `ModifierSet::MENU`; docs `:44-47`, `:228-239`; C-1..C-3 (9) | 4, 5, 7, 8, 9 |
| `src/engine/mod.rs` | rule 5 code `:839-845`, docs `:185-195`, a test (4). `mod modifiers`, `mod resync`, `pub use ModifierSet`; `resync` field, `resync_if_due`, `input_returned`, `on_foreground_changed()`, `reset` bookkeeping, `resync.event` in `on_key` (5). c6 test (6). `on_key` without the sync; `track_injected`; `holds`; docs `:39-61`, `:113-121`, `:197-220`; tests I-2, I-3 (7). R-F (8). The check in `on_key` and `on_key_up`; rule 1 and key-up docs; C-4 (9) | 4-9 |
| `src/engine/env.rs` | `os_key_down` docs `:34-52` (7); `unapplied` with its docs (9) | 7, 9 |
| `src/engine/matching.rs` | docs `:3-8`, `:33-36`, `:88-92`: generic codes come from events, and the check precedes the blocker read | 7, 9 |
| `src/engine/testing.rs` | D15 | 5, 7, 8 |
| `src/engine/scenarios.rs` | commit 3's scenario (3; `up_unnoticed` in 5); comments of `:189-203`, `:1011-1016`, `:3203-3211`, `:5154-5157`, `:5543-5550` ("the next event reads the OS" → "the main thread's read after the noticed event") (5); S3-S9, G1, I-1 (7); R-A..R-E, R-H (8) | 3, 5, 7, 8 |
| `src/engine/properties.rs` | `Mode::Unnoticed`, the new test, docs `:1-55` (7); its later-hook case (9) | 7, 9 |
| `src/hook/main_thread.rs` | `HIDDEN_INPUT`, `note_hidden_input`, `hidden_input_notes`, refusal note, unknown-switch note, `wake_this_thread` made `pub(crate)`, `MainState::resync_if_due`, `input_returned` → engine, drain step, module docs (5). `tick_count`, `HOOK_LATE(_MS)`, `note_if_awake_too_long`, E5 in `run` (10). Notes sum, `wake_for_hidden_input` (11). Tests | 5, 10, 11 |
| `src/hook/keyboard_hook_callback.rs` | foreground callback wake, docs `:48-52`, `:403-410` (5). `track_injected` in `decide_event`, docs `:13-15`, `:209-215`, `:494-506` (7). `UNAPPLIED`, `Unapplied`, `unapplied_while_running`, `HookEnv::unapplied`, Re-entry docs `:83-109` (9). E4, `event_age_ms`, `late_callback`, test helper and `:871-875` time stamps (10). Tests | 5, 7, 9, 10 |
| `src/hook/health.rs` | `unseen_input` | 11 |
| `src/hook/watchdog/decision.rs` | `unseen_at_last_tick` (11); `raw_pending` (13); tests, docs | 11, 13 |
| `src/hook/watchdog/mod.rs` | `RequestReinstall::hidden_input`, its impl, `Ticker::pass` E6, `Recorder` counts (11). The deferral, `woke_idle`, `Drive::wake`, three expectations, Cost doc `:23-42` (13) | 11, 13 |
| `src/logging/counters.rs` | `HookLate`, `warns`, `summary` level; tests | 10 |
| `src/logging/writer.rs` | level from `summary`; expected lines gain `hook_late=0` | 10 |
| `README.md` | `:553-567` (D10) (4). `:149`: "GetAsyncKeyState for the main thread's read of the modifiers between key events (roadmap item 2.5)" (7). `:149` adds ", and at a key event for each modifier held for the daemon"; `:558-561`: "Neither the main thread's read of the modifiers nor the check of a key event touches a modifier whose key-down was swallowed"; `:952` describes the two reads (9). `:865-874`: `hook_late`, and the line at `[INFO]` when it is alone (10). `:948` with R16's measured HEAD figure (13) | 4, 7, 9, 10, 13 |
| `book/src/config/keys.md` | `:194`: "The generic codes Shift 0x10, Ctrl 0x11 and Alt 0x12 are held only between the key-down and the key-up a program sends under them, and stop counting once a key event finds neither key of the pair down in Windows; the daemon never takes them from the Windows key state (roadmap item 2.5)" | 9 |
| `book/src/config/known-issues.md` | new row 88 (9); its triggers grow in 10 and 11 | 9, 10, 11 |
| `book/src/config/errors.md` | `:231`: `hook_late`, the `[INFO]` rule (10). `:214`: "at the first tick that counts a key event, a quarter of a second after it". `:215`: "The first key after 2 s without keys wakes it once to start its ticks, and from a quarter of a second later it reads the keys in batches four times a second until 2 s after the last key", with **the numbers unchanged** ("about 10 wakes", "N of about 1000"), plus "A burst of one key event, such as a key-up whose key-down came 2 s or more earlier, costs one more wake" (13) | 10, 13 |
| `src/config/default_config.ron` | **none**: no config meaning changes | — |
| Non-config book pages | **not touched**. Listed in `impl.md` for the next book sync: `architecture.md:237`, `:248`, `:251`, `:265`, `:294`, `:331`, `:350`, `:362`, `:566-567`, `:605`; `glossary.md:67`, `:377`, `:569`, `:577` (add "Hiding count", "Unapplied key"); `overview.md:16`; `gaps.md:167`, `:300`; `roadmap.md:584`, `:586-595` (2.5 status: W1, W2, W2a), `:739` (3.5: the witness follow-up lapses, FU-4 done, Q11 and M9 candidates), `:999`, `:1005`, check 44 `:654` (amended, M5), the re-runs M1-M10 | — |

**Known-issues row 88.** It lands in commit 9; commits 10 and 11 add their triggers. The developer pins the link to the
merge commit, as the other rows pin `bf549c0`. The final text:

> | 88 | A modifier (Shift, Ctrl, Alt, Win) pressed where the daemon cannot see it, with nothing that the daemon notices
> after it, counts as up for the daemon until the main thread next reads the modifiers from Windows (after a
> foreground change, `swch on`, a hook reinstall, the return from a lock or the secure desktop or the first key after
> it, a keyboard callback that found the daemon busy or ended late, the main thread spending 17 ms or more away from
> its messages, or a gap the hook watchdog saw) or until that key's next auto-repeat or release reaches the daemon.
> Meanwhile a binding with that modifier misses and the binding without it acts (with that Shift, CapsLock switches the
> layout instead of toggling caps lock); Alt pressed with that Shift, or Shift with that Alt, is not blocked, so Windows
> switches the layout; with that Alt or Win, a key the daemon swallows gets no menu mask key, so the menu bar activates
> or the Start menu opens when it is released; and the key's next auto-repeat counts as a new press, so a binding on
> that key alone acts once more. A modifier released where the daemon cannot see it, or one that another program's
> hook swallows after the daemon, stops counting at the next key event. No key sticks and no key-up reaches Windows
> without its key-down. Cases: hook calls that Windows drops while the daemon's main thread stalls outside what it
> measures; `LowLevelHooksTimeout` below about 34 ms; a secure-desktop episode with no signal at all; a key pressed in
> a child window of a higher-integrity process; key events decided in the same burst of messages right after a stall;
> a key-up that another program's hook swallows after the daemon passed it (from code) | By design (roadmap
> [item 2.5](../roadmap.md#25-latency-trim)) |
> [src/engine/resync.rs](https://github.com/bluesteelll/key-switch-rs/blob/<merge-sha>/src/engine/resync.rs) |
> [§7](kinds.md#s7) | none |

At commit 9 the trigger list ends at "a keyboard callback that found the daemon busy". Commit 10 adds "or ended late,
the main thread spending 17 ms or more away from its messages". Commit 11 adds "or a gap the hook watchdog saw".

## Commits, in order

Subjects follow `<Area>: <what>`. Each message ends with a blank line and
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Every commit passes the gate except the declared red
commits 3, 6, 8 and 12.

| Red-first test(s) | Red by assertion on | Green on |
|---|---|---|
| commit 3's scenario | `master` (`3cde1d0`) and commit 2 | commit 4, HEAD |
| commit 6's test | `master`, commit 5 | commit 7, HEAD |
| commit 8's R-A..R-H | **commit 7**: the tester replays them there (Q9) | commit 9, HEAD |
| commit 12's test | `master` | commit 13, HEAD |

On `master` most of commit 8's tests cannot build: their harness steps arrive in commits 5, 7 and 8, and
`resync_modifiers` arrives in 5. R-F builds there and is red by its count (11 asks at the first step). In behaviour,
master passes the scenario ones, because its per-event read repairs both cases. That is why commit 7, not master, is
their replay tree.

Compile-red tests (commits 4, 5, 7, 9, 10, 11 and 13) are carried by the mutations in the table at the end.

### 1. `Docs: a Hold's fast actions run on the main thread, and timers.rs serves three threads`

D14. No test. Green.

### 2. `Bench: rows R12m, R12m2, R17t and R17a; the callback's events carry a time stamp`

- The `Cargo.toml` feature.
- `bench.rs`: `call_keyboard_callback` sets `time: GetTickCount()`; new `call_injected_callback` (sentinel in
  `dwExtraInfo`, stamped); rows R12m, R12m2, R17t, R17a (D13); docs.
- CHANGELOG Added.

Green: the bench is ignored. No non-bench line of `src/` changes, so this tree is P (D13). The developer then runs the
probes of D13 into `bench-dev-diag.md` and applies stop rules (a)-(d).

### 3. `Engine: a swallowed repeat of a modifier with no record keeps its release from Windows (red)`

`engine::scenarios::a_swallowed_repeat_of_a_modifier_with_no_record_keeps_its_release_from_windows`:

```rust
Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
    .check_nothing_stuck()
    .check_no_lone_key_up()
    .down("LAlt").expect_passed()
    .down_unseen("LShift")
    .down("A").expect_passed()   // master's sync finds LShift held, with no record
    .up("A").expect_passed()
    .up_unseen("LShift")         // Windows has LShift up; the next event is LShift's own, which every read skips
    .down("LShift").expect_suppressed() // a repeat to the engine, no record; the Alt+Shift blocker swallows it
    .up("LShift").expect_suppressed()
    .up("LAlt").expect_passed();
```

**PREDICTED RED:** panic `scenario: up LShift at +0 ms: Windows got a lone key-up of LShift` (the lone-key-up check,
`testing.rs:644-657`), before `expect_suppressed`.

### 4. `Engine: a swallowed repeat with no record is recorded, so its key-up asks Windows`

D10; makes 3 green.

- `keys.rs`: `RepeatSuppressed`, `mark_repeat_suppressed`, `take_blocked`; `resync_from_os` keeps it.
- `mod.rs:839-845` and docs `:185-195`.
- README `:553-567`.
- CHANGELOG **Fixed** (Q13).

Unit tests, compile-red, carried by X1-X3:

- `engine::keys::tests::a_repeat_suppressed_record_is_consumed_like_a_swallowed_one_and_kept_by_a_reset`;
- `engine::keys::tests::a_repeat_suppressed_key_is_not_skipped_by_the_modifier_resync_and_counts_for_the_mask`. It
  calls `sync_modifiers` here, moves to `resync_modifiers` in commit 7, and gains `clear_released_modifiers` in commit
  9;
- `engine::tests::a_repeat_with_no_record_that_passes_after_a_swallowed_one_is_recorded_as_passed`. A repeat that the
  blocker swallows, then one that passes, then one the blocker would swallow, which passes on its `Passed` record.

Invariant kept: a key-up is suppressed if and only if its key-down was, unless Windows has the key down.

### 5. `Main thread: read the modifiers between events when something could have hidden them`

This commit is additive: `on_key` still syncs, and the drain reads as well. Contents:

- **Engine:** `engine/modifiers.rs`, `engine/resync.rs`; `Engine::resync_if_due`, `input_returned`,
  `on_foreground_changed()` (bump only), the `reset` bookkeeping (E2, E3), `resync.event` in `on_key`;
  `KeyState::resync_modifiers` (sided codes).
- **Main thread:** `HIDDEN_INPUT`, `note_hidden_input`, `hidden_input_notes`, E8 in `with_main`, E3u;
  `MainState::resync_if_due`, the drain step, `input_returned` → engine; the foreground callback's wake.
- **Harness:** D15's commit-5 part. Commit 3's scenario switches `up_unseen("LShift")` to `up_unnoticed("LShift")`: a
  noticed key-up would now heal LShift before the press, and the scenario would stop exercising D10.
- **Scenario comments** as in the touch set.
- **Bench and CHANGELOG:** R18; CHANGELOG Added R18.

Compile-red on master (new API), carried by X5-X20. Tests:

- `engine::modifiers::tests::a_sided_modifier_touches_its_generic_code_and_a_generic_one_both_sides` (it also checks
  the members of `ALL`, `SIDED` and `MENU`).
- `engine::resync::tests::`:
  - `a_new_resync_plans_a_full_read_at_its_first_plan`;
  - `a_bump_plans_one_full_read`;
  - `a_changed_note_count_plans_a_full_read_a_smaller_one_included`;
  - `a_modifier_touched_since_the_last_plan_is_left_out_and_read_at_the_next`;
  - `nothing_is_read_while_input_is_away_until_it_returns_or_a_key_event_is_decided`;
  - `an_injected_event_marks_its_modifier_fresh_and_does_not_bring_input_back`.
- `engine::tests::`. Asks are counted from the `resync_if_due` call only, since `on_key` still syncs in this commit;
  the decided events use F13.
  - `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers`: a foreground change, `InputLeft` then
    `input_returned`, `InputLeft` then a decided F13, and notes. `CountingEnv` sees the 8 sided codes.
  - `swch_on_and_a_reinstall_read_everything_themselves_so_the_next_drain_reads_nothing`.
  - `a_reload_makes_no_read`.
  - `a_drain_read_asks_the_eight_sided_modifiers_and_skips_a_swallowed_one`.
- `hook::main_thread::tests::`. These read the live key state (read only); the assertions are on whether a read
  happened, not on its result.
  - `the_drain_reads_the_modifiers_after_a_note_and_not_again`;
  - `a_refused_call_notes_hidden_input`;
  - `a_desktop_switch_to_an_unknown_destination_notes_hidden_input`;
  - `a_return_signal_makes_the_next_drain_read_the_modifiers`.
- `hook::keyboard_hook_callback::tests::a_foreground_change_queues_a_wake_for_the_drain` (`PeekMessageW` with
  `PM_REMOVE`, filtered to `MAIN_WAKE`, on the test thread's own queue).

### 6. `Engine: a key event with no modifier held asks Windows about no modifier (red)`

`engine::tests::a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier`. It uses `CountingEnv`
(`mod.rs:1214-1239`, everything up) and F13 bound to `DoNothing`. The script is `A` down, `A` up, `F13` down, `F13`
up, and it counts the asks whose code is one of the 11 `MODIFIERS`.

**PREDICTED RED:** `assertion `left == right` failed: a key event with no modifier held asks Windows about no modifier
(roadmap item 2.5)`, with `left: 44` and `right: 0`. That is 11 per event, none skipped: neither key is a modifier and
nothing is swallowed.

### 7. `Engine: the key path no longer reads every modifier from Windows`

D1, D2 and D3 (without the generic clear of the drain, which comes in commit 9); makes 6 green.

- **Engine:** `on_key` without the sync; `sync_modifiers` deleted; generic codes out of `resync_from_os`;
  `track_injected`; `Engine::holds`; the keys.rs tests moved (touch set).
- **Adapter:** `decide_event` tracks sentinel modifiers.
- **Harness and properties:** D15's commit-7 part; `Mode::Unnoticed`.
- **Docs:** rustdoc in `env.rs`, `matching.rs`, `mod.rs`, `keys.rs` and the callback module docs; README `:149`;
  CHANGELOG Changed (first part).
- **Not yet:** row 88, `keys.md:194` and README `:558-561`, `:952` land in commit 9. This commit is the first half of
  one meaning change: its wider residual (I1, I2) lives for two commits, and commit 8 pins it red (Q9).

Tests. The S-scenarios run on `DEFAULT_PAIR` (S9 on its own config), each under `check_nothing_stuck` and
`check_no_lone_key_up`. The construction drain (E0) is behind each.

| # | Test (`engine::scenarios::`) | Script | Expect |
|---|---|---|---|
| S3 | `a_modifier_pressed_where_the_hook_cannot_see_it_counts_after_a_foreground_change` | `down_unnoticed("LShift")`, `foreground_changed()`, `down("CapsLock")` | fires `ToggleCapsLock` |
| S4 | `a_modifier_pressed_on_the_lock_screen_counts_after_the_unlock_signal` | `session_locked()`, `down_unnoticed("LShift")`, `session_unlocked()`, `down("CapsLock")` | `ToggleCapsLock` |
| S5 | `a_modifier_pressed_on_the_lock_screen_counts_after_the_first_key_when_no_return_signal_comes` | `session_locked()`, `down_unnoticed("LShift")`, `down("F13")` (decided with every key released; its drain reads), `up("F13")`, `down("CapsLock")` | `ToggleCapsLock` |
| S6 | `a_modifier_pressed_while_remapping_is_off_counts_after_swch_on` | `remapping_off()`, `down("LShift")`, `remapping_on()`, `down("CapsLock")` | `ToggleCapsLock` |
| S7 | `a_modifier_pressed_while_the_hook_was_gone_counts_after_the_reinstall` | `down_unnoticed("LShift")`, `hook_reinstalled()`, `down("CapsLock")` | `ToggleCapsLock` |
| S8 | `a_modifier_pressed_where_the_hook_cannot_see_it_counts_after_an_adapter_note` (refusal, late callback, watchdog gap) | `down_unnoticed("LShift")`, `hidden_input_noted()`, `down("CapsLock")` | `ToggleCapsLock` |
| S9 | `a_swallowed_modifier_stays_held_through_both_reads_and_its_next_press_stays_a_repeat` (KS-14), config `( bindings: [ ( keys: "LShift", action: DoNothing, block_default: true ) ] )` | `down("LShift")` (suppressed, fires `DoNothing`), `down("A")` (passed), `up("A")`, `up_unnoticed("LShift")`, `hidden_input_noted()`, `down("LShift")` | suppressed, **`expect_nothing_fired()`**; then `up("LShift")` suppressed |
| G1 | `another_programs_generic_ctrl_counts_for_matching_between_its_key_down_and_key_up`, config `Ctrl+J` → `DoNothing` | `down_generic("Ctrl")`, `down("J")`, `up("J")`, `up_generic("Ctrl")`, `down("J")` | the first J fires `DoNothing`; the second fires nothing |
| I-1 | `the_daemons_own_injected_modifier_counts_for_matching_until_its_injected_key_up`, config `Ctrl+J` → `DoNothing` | `daemon_injects_down("LCtrl")`, `down("J")`, `up("J")`, `daemon_injects_up("LCtrl")`, `down("J")` | fires, then nothing |

Each phantom-released S-scenario fires `SwitchLanguage` when its source does not read.

More tests:

- `engine::tests::an_injected_key_up_of_a_held_swallowed_modifier_leaves_it_held` (direct). A swallowed LShift down,
  then `track_injected` LShift down and up, then LShift down: a repeat, suppressed, nothing fired.
- `engine::tests::an_injected_event_of_a_key_that_is_not_a_modifier_changes_nothing` (direct). `A` → `DoNothing`;
  `track_injected(A down)`, then a physical A down fires `DoNothing`.
- `engine::keys::tests::a_reset_never_takes_a_generic_modifier_from_the_os`.
- `hook::keyboard_hook_callback::tests::the_daemons_own_injected_modifiers_pass_and_are_tracked`. `decide_event` with
  the sentinel and LCtrl down returns `None`, and `with_main(|m| m.engine.holds(VK_LCONTROL))` is true; LCtrl up makes
  it false; a sentinel A down leaves `holds(VK_A)` false. It is deterministic: no live read is asserted.
- `engine::properties::no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed`.
- Every 2.3 test and property, unchanged (acceptance).

### 8. `Engine: a modifier Windows has up does not count for the next key event (red)`

This commit is test-only: D15's later-hook steps and eight tests. All use API that exists on commit 7. **Replay on
commit 7** (Q9).

| # | Test | Script / assertion | PREDICTED RED on commit 7 |
|---|---|---|---|
| R-A | `engine::scenarios::a_modifier_that_a_later_hook_swallows_does_not_count_for_matching` (critique v2 I2, AutoHotkey `LAlt::LCtrl` started before the daemon) | `Ctrl+J` → `DoNothing`; `check_nothing_stuck`, `check_no_lone_key_up`. `down_swallowed_later("LAlt")` passed; `down("LCtrl")` passed (AutoHotkey's injection); `down("J")` → `expect_fired(&[DoNothing])`, `expect_suppressed()`; `up("J")`; `advance_ms(500)`; `repeat_swallowed_later("LAlt")`; `repeat("LCtrl")`; `down("J")` fires; `up("J")`; `advance_ms(33)`; `repeat_swallowed_later("LAlt")`; `down("J")` fires (right after an LAlt repeat, before AutoHotkey's); `up("J")`; `up_swallowed_later("LAlt")`; `up("LCtrl")` | `scenario: down J at +0 ms: expected to fire [DoNothing], fired []` |
| R-B | `engine::scenarios::a_shift_that_a_later_hook_swallows_does_not_make_alt_the_language_hotkey` | `with_alt_shift_blocker(DEFAULT_PAIR)`; `down_swallowed_later("LShift")`; `down("LAlt")` → `expect_passed()`; `down("Tab")` passed, `expect_down_in_windows("LAlt")`; `up("Tab")`; `up("LAlt")` passed; `up_swallowed_later("LShift")` | `scenario: down LAlt at +0 ms: expected the event to be passed, it was suppressed` |
| R-C | `engine::scenarios::another_programs_generic_shift_released_unnoticed_stops_counting_after_a_note` (critique v2 I1, the ruling's script) | `DEFAULT_PAIR`; `down_generic("Shift")` passed; `up_generic_unnoticed("Shift")`; `hidden_input_noted()`; `down("CapsLock")` → `expect_fired(&[SwitchLanguage])` | `scenario: down CapsLock at +0 ms: expected to fire [SwitchLanguage], fired [ToggleCapsLock]` |
| R-D | `engine::scenarios::another_programs_generic_shift_released_unnoticed_stops_counting_at_the_next_key` | as R-C without `hidden_input_noted()` | the same message |
| R-E | `engine::scenarios::a_modifier_released_where_the_hook_cannot_see_it_stops_counting_at_the_next_key` (W2: "a hidden modifier key-up followed by a binding press heals") | `DEFAULT_PAIR`; `down("LShift")`; `up_unnoticed("LShift")`; `down("CapsLock")` → `expect_fired(&[SwitchLanguage])`; `up("CapsLock")`; `down("CapsLock")` fires `SwitchLanguage` | the same message |
| R-F | `engine::tests::a_key_event_asks_windows_once_for_each_modifier_held_for_it_and_never_for_its_own_key_or_a_swallowed_one` (direct engine) | Bindings `LWin` → `DoNothing`, `F13` → `DoNothing`. A counting `Env` in which Windows has LCtrl, LShift and LAlt (and their generic codes) down and everything else up. It counts asks of a `MODIFIERS` code other than the event's own. Per step: A down/up 0/0; LCtrl down 0; LCtrl repeat 0; A down 1; A up 0; LShift down 1; A down 2; A up 0; LWin down (swallowed) 2; A down 2 (not LWin); A up 0; LWin up 0; LAlt down 2; F13 down (swallowed) 3; F13 up (swallowed) 1 (LAlt only); LAlt, LShift, LCtrl up 0 each | `assertion `left == right` failed: A down with LCtrl held: a key event asks Windows once about each modifier held for it, never about its own key or a swallowed one (roadmap item 2.5)`, `left: 0`, `right: 1` |
| R-G | `engine::keys::tests::the_drain_read_clears_a_generic_slot_only_when_its_code_reads_up_and_never_sets_one` | `press(VK_SHIFT)`. `resync_modifiers` with `Down(&[VK_RSHIFT, VK_SHIFT])` keeps it; with `Down(&[VK_LSHIFT, VK_SHIFT])` keeps it; with `Down(&[])` clears it. A new state with `Down(&[VK_LSHIFT, VK_SHIFT])` sets `VK_LSHIFT` and not `VK_SHIFT` | panic `Windows has neither Shift down: the drain read clears the generic slot (roadmap item 2.5)` |
| R-H | `engine::scenarios::a_swallowed_key_up_asks_no_mask_for_an_alt_that_windows_has_up` | `F13` → `DoNothing`; `check_mask_rule()`; `down("F13")` suppressed; `down_swallowed_later("LAlt")` passed; `up("F13")` → `expect_suppressed()`, `expect_unmasked()`; `up_swallowed_later("LAlt")` | `scenario: up F13 at +0 ms: mask true, the rule says false (suppressed true, Alt or Win down in Windows false)` |

### 9. `Engine: a key event asks Windows only about the modifiers held for it and drops those Windows has up`

D16 and D17, and D3's generic clear in the drain; makes 8 green.

- **Engine:** `KeyState::clear_released_modifiers`; the check in `on_key` and `on_key_up`; `resync_modifiers` gains
  the generic clear-only; `MENU_KEYS` → `ModifierSet::MENU`; `Env::unapplied`.
- **Adapter:** `UNAPPLIED`, `Unapplied`, `unapplied_while_running`, `HookEnv::unapplied`, the guard in the callback.
- **Properties:** the later-hook case in `Mode::Unnoticed` (Q7).
- **Docs:** rustdoc (`mod.rs` rule 1 and key-up rules, `env.rs`, `keys.rs`, `matching.rs`, callback Re-entry);
  README `:149`, `:558-561`, `:952`; `keys.md:194`; known-issues row 88; CHANGELOG Changed (second part).

Compile-red tests, carried by X25-X37:

| # | Test | What it pins |
|---|---|---|
| C-1 | `engine::keys::tests::the_check_reads_only_held_modifiers_skips_its_own_key_and_a_swallowed_one_and_never_sets_a_flag` | LCtrl (passed), LShift (swallowed), RAlt (own) set; LWin clear. A counting env with everything up except LWin. `clear_released_modifiers(env, VK_RMENU, ALL)` asks exactly `[VK_LCONTROL]`; LCtrl cleared, LShift and RAlt kept, LWin still clear |
| C-2 | `engine::keys::tests::the_check_asks_a_generic_slot_its_generic_code_and_clears_it_only_when_it_reads_up` | `VK_SHIFT` set. With `[VK_RSHIFT, VK_SHIFT]` down it asks `[VK_SHIFT]` and keeps it; with nothing down it asks `[VK_SHIFT]` and clears it |
| C-3 | `engine::keys::tests::the_check_leaves_a_key_that_windows_has_not_applied_yet` | LCtrl set; an env with everything up and `unapplied() = touched_by(VK_LCONTROL)`: no ask, LCtrl kept |
| C-4 | `engine::tests::a_key_decided_inside_another_keys_callback_keeps_that_keys_modifier` | `Ctrl+J`. LCtrl down, then J with `unapplied {LCtrl}`: it fires. J up; J again with `unapplied` empty: nothing fires (Windows has LCtrl up) |
| C-5 | `hook::keyboard_hook_callback::tests::a_passed_modifier_key_down_stays_unapplied_until_its_callback_returns` | `unapplied_while_running`: a passed LCtrl key-down → `{LCtrl, VK_CONTROL}`; a swallowed one, a key-up, a passed A → EMPTY |
| C-6 | `hook::keyboard_hook_callback::tests::the_callback_keeps_its_key_unapplied_until_it_returns` | a scan of `fn keyboard_hook_callback(`: a named `Unapplied` local is declared before `decide_event(` and assigned between `decide_event(` and `carry_out(`; no `let _ = Unapplied` |
| C-7 | `hook::keyboard_hook_callback::tests::a_key_decided_while_another_key_is_unapplied_keeps_its_modifier` | process-wide hook under `GLOBAL_HOOK`; `Ctrl+J` with `allow_extra_modifiers`. `decide_event` LCtrl down; `Unapplied::mark(touched_by(VK_LCONTROL))`; `decide_event` J fires. Without the mark the live read clears LCtrl whenever the user does not hold it; with LCtrl held by the user the test passes either way |

### 10. `Hook: a late callback or a long stretch away from messages makes the main thread read the modifiers`

- E4, E5 (on `GetTickCount`), D9.
- README `:865-874`; `errors.md:231`.
- CHANGELOG: Added `hook_late`; Changed: the line level and row 88's triggers; example line `:120`.
- The test helpers' time stamps.

Compile-red, carried by X38-X41. Tests:

- `hook::keyboard_hook_callback::tests::`:
  - `lateness_counts_age_and_run_with_a_17_ms_boundary` (pure: 16 is false, 17 is true; `ran` of 17 ms is true;
    wraparound at `u32::MAX`; a stamp 5 ms ahead of `now` has age 0);
  - `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late` (the extern callback under
    `GLOBAL_HOOK`, `time = GetTickCount() - 1000`);
  - `a_callback_for_a_fresh_event_notes_nothing` (one retry of the whole check);
  - `an_injected_events_own_stamp_is_not_its_age` (`LLKHF_INJECTED`, `time` 1).
- `hook::main_thread::tests::`:
  - `an_awake_stretch_of_17_ms_notes_hidden_input` (pure, in ticks: 20 ms notes, 5 ms does not, wraparound);
  - `the_main_loop_checks_its_awake_stretch_before_every_wait` (a scan of `fn run(`: the check comes before
    `MsgWaitForMultipleObjectsEx`, and `awake_since` is set right after it).
- `logging::counters::tests::hook_late_alone_is_reported_at_info_and_with_another_count_at_warn`; the existing counters
  and writer tests are updated for 12 counters.

### 11. `Watchdog: raw key events no callback explains make the main thread read the modifiers`

- E6; `RequestReinstall::hidden_input`; the notes sum in `MainState::resync_if_due`.
- CHANGELOG and the row 88 trigger lists.

Compile-red, carried by X42-X45. Tests:

- `hook::health::tests::unseen_input_counts_each_note`;
- `hook::watchdog::decision::tests::a_tick_reports_raw_key_events_no_callback_explains` (raw input every 30 ms with no
  callback gives a value above 0; callbacks that keep up give 0 at every tick);
- `hook::watchdog::tests::a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread` (`Drive`; `Recorder`
  counts `hidden_input`);
- `hook::main_thread::tests::the_drain_counts_the_watchdogs_notes`.

### 12. `Watchdog: the first raw key event after idle is read at the first tick (red)`

`hook::watchdog::tests::the_first_raw_key_event_after_idle_is_read_at_the_first_tick`: on `Drive`, one raw event and
one callback at 0 ms, run to 600. It asserts `drive.reads.first() == Some(&(250, 1))` and `drive.raw_wakes == 1`.

**PREDICTED RED:** `left: Some((0, 1))`, `right: Some((250, 1))`.

### 13. `Watchdog: an idle wake only starts the ticks`

D12; makes 12 green; only if D-R16d passed.

- `raw_pending`, `woke_idle`, the deferral, `Drive::wake`, the three expectations, the Cost doc.
- The R16 doc and gate; README `:948` with R16's measured HEAD figure, never a planned one (2.4 V3-M2);
  `errors.md:214-215`.
- CHANGELOG Changed.

Compile-red, carried by X46-X49. Tests:

- `hook::watchdog::decision::tests::a_deferred_first_read_starts_the_silence_at_the_wake`;
- `hook::watchdog::tests::an_idle_wake_with_no_hook_installed_still_reads_at_once`;
- `hook::watchdog::tests::a_deferred_wake_is_followed_by_a_wait_that_leaves_only_the_batched_input_out`.

Then the developer runs R16 at HEAD (D13, probe 3). Afterwards they record in `impl.md`: the HEAD bench, the commit-7
sha for the replay, the non-config book pages and the manual checks.

**CHANGELOG texts** (the developer copies them):

- **Added (2):** "Bench rows R12m and R12m2, a key-down with one and with two modifiers held, R17t, one `GetTickCount`,
  and R17a, one `GetAsyncKeyState` (roadmap item 2.5)."
- **Fixed (4):** "An auto-repeat of a modifier whose key-down the daemon never saw (one Windows had down when the
  daemon read it), which a binding swallows, now keeps the modifier's release from Windows unless Windows has it down,
  as for any swallowed press. Before, the release always passed, and Windows could get a key-up without its key-down
  (roadmap item 2.5)."
- **Added (5):** "Bench row R18: the main thread's read of the modifiers between key events (roadmap item 2.5)."
- **Changed (7, then extended in 9):**
  - Commit 7's text: "The keyboard hook no longer asks Windows for the state of eleven modifier codes at every key
    event, about 4 µs of the 4.7 µs of an ordinary key event (bench row R12; roadmap item 2.5). The engine follows
    Shift, Ctrl, Alt and Win from the key events it sees, the daemon's own injected ones included. The generic codes
    `VK_SHIFT`, `VK_CONTROL` and `VK_MENU` count as held only between the key-down and the key-up a program sends
    under them."
  - Commit 9 appends: "A key event asks Windows only about the modifiers held for the daemon, one read each, and stops
    counting those Windows has up: ordinary typing reads none, Ctrl+J one. A generic code also stops counting once
    Windows has neither key of its pair down. The main thread reads the eight sided modifiers between key events when
    something could have hidden one: at startup, after a foreground change, `swch on` and a hook reinstall, when
    keyboard input comes back from a lock or the secure desktop, and after a callback that found the daemon busy. A
    modifier pressed where the daemon cannot see it, with none of these after it, counts as up until then or until its
    next auto-repeat (known issue 88)."
  - Commit 10 adds ", a late callback, a stretch of 17 ms or more away from its messages" to the list, and commit 11
    adds "or a gap the hook watchdog saw".
- **Added (10):** "Counter `hook_late` (roadmap item 2.5): a keyboard callback that ended 17 ms or more after its key
  event, or a main thread 17 ms or more away from its messages; the main thread then reads the modifiers again. The
  stamp of an injected event does not count."
- **Changed (10):** "A counters line whose only non-zero count is `hook_late` is written at `[INFO]`, not `[WARN]`."
- **Changed (13):** "The hook watchdog's first wake after idle makes no user32 call: it only starts its ticks, and the
  key events wait for the first tick, a quarter of a second later (roadmap item 2.5, follow-up FU-4). A keyboard
  callback at that moment is no longer slowed by about 50 µs (bench row R16, now N µs at the median, gated). A key
  press or a shortcut costs the watchdog no more wakes than before. A burst of a single raw key event, such as a
  key-up whose key-down came 2 s or more earlier, costs one more."

## Acceptance mapping

| Acceptance line (roadmap `:595`, spec `:15-19`, rulings) | Covered by |
|---|---|
| R11 within the D14 rule of R12 | Superseded by W1 (the witness stays). R11's witness term is unchanged: the D13 R11 rule; `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key` (unchanged) |
| A new row shows the witness reading after a foreground change | Moot under W1: the witness reads at every swallowed key-up (R11) |
| W2: R12 and R1/R4 show the reads gone; the new R12 bound from a diagnostic row; a row for the resync on main | D13: R12 and R4 against the same-session N; R1 with 0.5 Δ; R18. Test: `a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier` |
| W2a: R12m and R12m2, each bounded against its probe; the trade in one line | D13; R-F |
| Every stuck-key and lone-key-up test and property of 2.3 stays green, the unseen re-press after a focus change included | `engine::scenarios` (unchanged, including `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows`, `scenarios.rs:2030-2045`), `:2011-2084`, `:2091-2338`, `:2342-2406`; `engine::tests` `mod.rs:1241-1316`; the six `engine::properties` tests |
| W2: a hidden modifier key-up followed by a binding press heals at the next resync | R-E and R-D: under W2a they heal at the press itself, before matching |
| W2 / spec: each source triggers a resync (foreground change, lost-input reset, remapping switch, reinstall, refused event, watchdog gap, late callback) | Engine: S3 (E1), S4, S5 (E3), S6, S7 (E2), S8 (E8, E4, E6 in the engine); `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers`. Adapter: commit 5 `a_refused_call_notes_hidden_input`, `a_desktop_switch_to_an_unknown_destination_notes_hidden_input`; commit 10 `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late`, `an_awake_stretch_of_17_ms_notes_hidden_input`; commit 11 `a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread`, `the_drain_counts_the_watchdogs_notes` |
| W2: no stuck key and no lone key-up across the property modes of 2.3 | the six properties (unchanged); the new `Unnoticed` property with its later-hook case; commit 3's scenario; S9; R-A, R-B (both under `check_nothing_stuck` and `check_no_lone_key_up`) |
| W2 / round-2 M1: residuals declared in known issues, with what the user sees | row 88 (commit 9); D11's owner summary in `results.md` |
| W2a pins: the AutoHotkey trace, the blocker trace, I1's scenario, a read count; the named mutations | R-A, R-B, R-C (+R-D), R-F; X26 (check removed), X27 (allowed to set), X28 (KS-14 skip removed), X29 (generic read replaced by a sided one) |
| W2a: W10 corrected | D4's W-table, W10 and W14 |
| W2a: the drain read sets and clears the sided slots and clears a generic slot only when it reads up | D3, D4; R-G; X33, X34 |
| R16 meets a bound the cut sets from the evidence | D12, D13 (provisional bound, D-R16d, the HEAD check); `the_first_raw_key_event_after_idle_is_read_at_the_first_tick` |
| No other bench row regresses | the tester's §4b runs with the D13 table |
| The 2.4 watchdog acceptance stays green | `hook::watchdog::decision::tests::*` unchanged; the `roadmap.md:578-581` rows; three loop-model expectations move by one tick (D12, Q3) |
| W4: no `[WARN]` from `hook_late` in normal typing; a manual check; the unverified base marked; injector stamps | D8, D9; `hook_late_alone_is_reported_at_info_and_with_another_count_at_warn`; `an_injected_events_own_stamp_is_not_its_age`; M8; U1 |
| OQ2: a modifier state Windows changes with no LL event | W13; M9 |
| Book-sync leftovers | commit 1 |
| CHANGELOG (citing 2.5), README latency notes, config pages where a cost or behaviour changes | commits 2, 4, 5, 7, 9, 10, 11, 13 (touch set) |

## Manual checks (human only)

None of these can be a test: each needs a live hook, the secure desktop, a debug daemon or another program's hook. No
test is ignored for them.

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | Roadmap check 35 (`roadmap.md:645`): a non-elevated debug daemon after `swch exit`, the default config, Notepad, an elevated Task Manager | (a) As check 35. (b) Hold LShift in Notepad, click into Task Manager, release LShift there, click back into Notepad, press CapsLock. (c) In Task Manager press and hold LShift, click into Notepad still holding it, press CapsLock, release both | (a) as check 35 (W1: unchanged). (b) The layout switches and caps lock does not toggle: the check at CapsLock or the foreground change's read found LShift up. (c) Caps lock toggles (`Shift+CapsLock`) and the layout does not switch: the foreground change made the main thread read LShift down (E1) | none |
| M2 | Roadmap check 36 (`:646`), release daemon, default config | (a) As check 36. (b) Ctrl+Alt+Del; there press and hold LShift; press Esc; on the desktop press CapsLock with LShift still held; release both | (a) as check 36. (b) Caps lock toggles (`Shift+CapsLock`) and the layout does not switch: the return read found LShift held (E3, D6) | none |
| M3 | M2 with a UAC prompt; `PromptOnSecureDesktop` = 1 (read only, as check 28) | M2 (b) at the prompt instead of Ctrl+Alt+Del, Cancel instead of Esc | as M2 (b) | none |
| M4 | 1. `swch exit`; `swch status` says not running. 2. `cargo build --locked`. 3. The default config, `osk.exe` open. 4. Read only: `reg query "HKCU\Control Panel\Desktop" /v LowLevelHooksTimeout` | 1. `$env:KS_DEBUG_HOOK_HOLD='CapsLock'; .\target\debug\key-switch-rs.exe`. 2. Press and release CapsLock once (a 1.5 s hold); during the hold press and release LShift. 3. Press CapsLock again. 4. `swch status`; `swch exit`. 5. Read `daemon.log`. 6. `swch open` | CapsLock is not shown held in `osk.exe`. Step 3 switches the layout (no phantom Shift). An `[INFO] counters ...` line shows `hook_late=` of at least 1. Report `reinstalls` (1 means Windows removed the hook; 0 with remapping working is known issue 83) and whether `hook_late` is above 2 (keys typed during the hold arrived late: Windows delivered timed-out calls) | none |
| M5 | Roadmap check 44 (`:654`), amended | as check 44 | `raw keyboard input arrives` now comes one tick (250 ms) after the raw input. Compare t_raw − 250 ms with t_key: earlier by about the hook timeout means before the hook call; within about 20 ms means after it (U5) | none |
| M6 | Roadmap check 39 (`:649`) | as check 39 | W ≤ 100. The start and the two typed commands still give about 45: the deferral adds a tick only to a burst of one raw key event (D12) | none |
| M7 | Roadmap check 47 (`:657`) | as check 47 | `W ≤ K/2`; `hook ok, reinstalls: 0`. W is the same as before for the same hold: a 20 s hold is one burst of many raw events (D12) | none |
| M8 | Release daemon via `swch open` (ruling W4) | Type normally for 5 minutes across programs, an elevated one included; `swch exit`; read only: `Select-String 'counters since the last report' "$env:LOCALAPPDATA\key-switch-rs\daemon.log" \| Select-Object -Last 3` | No line in those minutes, or lines with `hook_late=0`. If `hook_late` is above 0 in normal typing, report the lines: the stamp base differs (U1), and the fallback is the run time alone (Q2). Note (OQ3, U10): input typed into this machine from a Remote Desktop client may carry stamps on another base, so `hook_late` there is expected; it costs only reads and an `[INFO]` line. Report it; it is not a defect | none |
| M9 (new, OQ2) | Release daemon, default config, Notepad. **The owner** turns on Settings → Accessibility → Keyboard → Sticky keys, and turns it off afterwards | Tap LShift once, type `a`, then press CapsLock. Then tap LShift once and click into Notepad with the mouse, then press CapsLock | `A` is typed; each CapsLock switches the layout and does not toggle caps lock. If one toggles caps lock instead, report it: W13 then holds, and a follow-up for 3.5 goes in `impl.md` | none |
| M10 (new) | `swch exit`; an AutoHotkey v2 script `LAlt::LCtrl` started first; then `swch open`. A temporary binding `( keys: "Ctrl+J", action: SwitchLanguage )` in the owner's config, removed afterwards; Notepad | Hold the left Alt key; press J three times; release. Then hold it for 2 s (auto-repeat) and press J once more | The layout switches at every J (four times); no `j` and no menu bar. Without the check (commit 7's state) the binding missed whenever no drain ran between Alt and J (critique v2 I2); master's per-event read repairs it too | none |

## Risks and edge cases (code-reviewer R1-R10)

- **R1 Hook callback.**
  - Removed: the per-event modifier reads.
  - Added at a decided event: the 11-slot scan, plus one `GetAsyncKeyState` per held unswallowed modifier at a
    key-down or per held Alt/Win at a swallowed key-up (D16).
  - Added for a passed modifier key-down: the `Unapplied` guard (two TLS accesses, D17).
  - Added on every callback while remapping is on: two copies, one `GetTickCount` at exit for a physical event and
    two compares; when late, a TLS add and one RMW.
  - Added for a sentinel modifier: one `with_main` and a store.
  - No allocation, lock, `log::`, post or blocking call. `code < 0` and the remapping gate stay first; the sentinel
    test still precedes every decision; every path ends in one outcome.
  - The guard's restore runs on both return paths (D17, C-6). A panic in an `extern "system"` callback aborts, so no
    unwind leaves the TLS marked.
- **R2 Sentinel.** No new `SendInput`. Sentinel modifier events are tracked, never decided (D2), and no check runs at
  them.
- **R3 Shared state.** `HIDDEN_INPUT` and `UNAPPLIED` are main-thread TLS `Cell`s, like `PENDING_SIGNALS`.
  `unseen_input` is an atomic in `HealthState`. Under the borrow there is only pure engine work and `GetAsyncKeyState`.
- **R4 Orderings.** `unseen_input`: `fetch_add(AcqRel)`, `load(Acquire)`, with no published data. No `Relaxed`.
- **R5 Builders.** None.
- **R6 Display.** `Counter::HookLate` gets its `Display` arm, its `ALL` entry, its slot and its `warns` arm in
  commit 10, with no `_ =>`. `LastDown` has no `Display`.
- **R7 Specificity.** Untouched.
- **R8 Key identity.**
  - Generic slots: set by events only; cleared by events, by the check and by the drain when the generic code reads up
    (D3).
  - Sentinel modifiers are tracked (D2).
  - `ModifierSet::touched_by` maps a generic code to both sides for `fresh` and for `unapplied`.
  - The witness still reads the event's own code (`the_witness_of_a_generic_shift_key_up_reads_the_generic_state` is
    unchanged).
  - `VIRTUAL_KEY(raw)` is built only at the hook boundary (the callback's D17 call).
- **R9 Registry and process.** No new read.
- **R10 Loop and shutdown.** E5's `continue` cannot spin: it resets `awake_since`, so the next check sees a short
  stretch. The drain's read stops with the loop. D12 cannot spin (above).

**Other edges.**

- **Reload mid-gesture:** no read (`Reloaded` moves nothing); the check works as at any event.
- **`swch off`:** no reads while off. `swch on` reads everything.
- **Foreground change mid-press:** a bump, then a read at the drain; the pressed modifier is fresh if it was touched
  in the same stretch.
- **AltGr:** the layout's LCtrl and RAlt reach the hook as events and are tracked like any other. Both are applied
  before the next event, so the check keeps them.
- **Auto-repeat:** it sets the flag again, and the check skips the own key.
- **A phantom-held key cleared by the check while physically held** (W10, a later hook): its next auto-repeat is
  decided as a first press, as on master (master's sync cleared it the same way).
- **A nested decided event during the daemon's own half-processed `SendInput`:** it may read a just-tracked injected
  modifier as up and clear it; the injected key-up clears it anyway. This equals master, whose sync read it the same
  way.
- **A modifier physically held while a higher-integrity window has the foreground:** the drain's read then sees zero
  (UIPI); the hook sees no key there; the read at the return gives the truth.
- **Sleep and resume:** `GetTickCount` counts since start, so an event stamped before sleep reads as late: one read.
- **Tests:** tests that drain read the live key state (read only) and assert only whether a read happened. Tests that
  do not drain no longer depend on the keys the user holds. C-7 discriminates only while the user does not hold LCtrl,
  and passes otherwise.
- **IME and layouts:** no effect.

## Questions for the orchestrator

1. **(old Q3) R16's bound is provisional** (D13). Recommendation: as ruled (W3). Probe D-R16d after commit 2, check
   HEAD after commit 13, and stop and report on a miss. If the cost is in the wake itself, keep R16 declared (2.4
   ruling L2) rather than buy the 1 s idle poll, which is an idle-cost choice for the owner.
2. **(old Q4) E4: 17 ms and one `GetTickCount` per physical callback; E5 now on `GetTickCount` too.** Accepted.
   Stop rule: if R17t p50 > 100 ns (a system call), E4 uses the run time alone (`ran >= HOOK_LATE`) and drops the age
   part, and E5 uses `Instant` (two reads per pass, as round 1 had it). W6 is then covered by E5 and E6 only. The same
   E4 fallback applies if M8 fails. Recommendation: accept the E5 clock change. It removes a QPC read between the
   main thread's wake and the pump that delivers the hook call, and keeps E5's detection bound (stretches of about
   34 ms or more).
3. **(old Q6) Three loop-model expectations move by one tick** (D12). Accepted.
4. **(old Q8) The 4.6 follow-up** (a raw-input registration plus a focused daemon window stops the process's LL hook;
   `RIDEV_NOHOTKEYS`). Accepted: it goes to "Follow-ups from Phase 2" at the book sync.
5. **(old Q9) A chore after the merge** (CLAUDE.md, `tester.md` §4b, `code-reviewer.md`), content updated for W2a:
   - **Bench rules:** P on commit 2, N, H in one session; R12, R12m, R12m2 and R4 against N; R1 with 0.5 Δ; R11's
     witness term; R16 gated; R17t, R17a, R18.
   - **Principle 1:** the callback's sanctioned reads become the witness, the KS-05 blocker read and the check (D16);
     E4's `GetTickCount` and late note and D17's guard are sanctioned too.
   - **Principle 3:** sentinel modifier events update a flag (D2).
   - **Principle 5:** `HIDDEN_INPUT`, `UNAPPLIED`, `unseen_input`.
   - **Principle 6:** the drain's read; the foreground callback's wake.
   - **Layout:** the new `src/engine/modifiers.rs` and `src/engine/resync.rs`.
   - **Tester:** `tester.md` §3 learns that a lane may name a branch commit as the replay tree of a red test (Q9 here).

   Recommendation: do it as after 2.3 and 2.4.
6. **(old Q10) The auto-blocker's OS read stays on the key path** (`mod.rs:721-723`, KS-05). Accepted. The check now
   runs before it, so its hotkey's flags are checked first.
7. **(old Q12) The `Unnoticed` property mode, with its later-hook case.** If a seed fails, the proof is wrong.
   Recommendation (accepted): the developer stops and reports the seed and the transcript, with no workaround.
8. **(old Q13) D10 changes a suppression rule in a corner case.** Accepted: CHANGELOG **Fixed**, citing 2.5.
9. **(new) Commit order and the replay commit.** Commits 7 and 8 carry the I1/I2 regression for two commits, so that
   commit 8's tests are red by assertion on a real tree. The tester replays them on commit 7, not on master. On master
   most of them cannot build (their harness steps arrive in commits 5, 7 and 8), and in behaviour master passes the
   scenario ones. `tester.md:90-92` would read a green parent as "does not reproduce". The developer names commit 7's
   sha in `impl.md`. Recommendation: accept. Putting the check in the same commit as the removal would leave those
   tests red only under a mutation; first-parent `master` never sees commit 7's state.
10. **(new) The tester's parent runs use commit 2** (P in D13), because R12m, R12m2, R17t and R17a do not exist on
    master. Commit 2 changes no non-bench source line (the tester checks `git diff --stat master <commit 2> -- src`).
    Recommendation: accept.
11. **(new, follow-up, no action) The own-key skip** (ruling W2a) keeps row 63's modifier case: a modifier released
    unseen and pressed again, with no key event in between, reads as an auto-repeat, as on master. Reading the event's
    own key in the check would make that press a press. The cost is one read per auto-repeat of a held modifier, plus
    the double-fire risk of D10's `RepeatSuppressed`. Recommendation: no change in this lane; a candidate for 3.5.
12. **Settled:** old Q1 (moot, W1), old Q2 (superseded by round-2 M1: row 88 names the visible effects), old Q5
    (folded, W2), old Q7 (W4), Q11 `RESYNC_PERIOD` (moot: D7 dropped).

## Cut readiness checklist

### Spec

- [x] Item 2.5 cites no KS issue (`roadmap.md:439`). The KS-14, KS-05, KS-06 (status) and KS-12 rules are traced (D3,
  D10, D11, D16), and their scenarios are kept or added (commit 3, S3-S9, R-A..R-H, M1-M10).
- [x] Every reference is re-located on `3cde1d0` or carried over from `50b56a8` (Verdict); none is gone.
- [x] Goal, non-goals and later items are stated.

### Design

- [x] Every decision has a rationale and rejected alternatives (D1-D17; D7 is dropped with its argument).
- [x] Owner decisions: exact modifiers and repeat defaults unchanged; RON unchanged; `log` and the sink (the hook path
  only counts); licence n/a; latency first (every addition costed, key-path table, D13); no key sticks (D11, W1).
- [x] Threads are named (thread map). Nothing heavy runs in the LL callback: it loses 11 reads, and gains a read per
  held modifier and one tick read.
- [x] Install, uninstall and timers are unchanged; the watchdog's timer stays on its thread.
- [x] No new `SendInput`; sentinel events are tracked, never decided.
- [x] Sided and unsided: D3, D16, R8. Chords and sequences read the same flags, after the check (D11 item 4).
- [x] Reload, `swch off`/`on`, foreground change and shutdown during a pending gesture: the D4 table and the edges.
- [x] Orderings stated (R4); no `static mut`.
- [x] No new module cycle: `engine::modifiers` and `engine::resync` are children of `engine`, and `hook` already
  depends on `engine`. The logic is in `engine` and reachable by scenarios.

### Win32

- [x] Paths, the feature check (one to add), threads, errors and links; the unverified items U1-U10 with their checks.
- [x] Elevation and UIPI (GetAsyncKeyState's zero); pipe and tokens n/a.

### Commits and tests

- [x] Red-first: 3 → 4, 6 → 7, 8 → 9 (replayed on commit 7), 12 → 13, by assertion. The compile-red tests of 4, 5, 7,
  9, 10, 11 and 13 are carried by mutations.
- [x] Tests go through `Engine::on_key`, `resync_if_due`, `Env` and the fake clock; adapter tests drive the extern
  callback with no hook installed; nothing hooks, registers raw input or injects.
- [x] Real Win32 in tests is read-only: `GetTickCount`, `GetAsyncKeyState`, `PeekMessageW` on the test thread's own
  queue.
- [x] Every commit passes the gate except 3, 6, 8 and 12. Docs go with the commit that changes the meaning; commit 7's
  user-facing docs complete in 9 (Q9).

### Acceptance

- [x] Every acceptance line is mapped; the superseded ones name their ruling.
- [x] Every question has a recommendation.

**Mutations for the tester.** One for every compile-red test, plus the design pins. No test is listed against a
mutation it passes under.

| # | Mutation | Caught by |
|---|---|---|
| X1 | `resync_from_os` turns `RepeatSuppressed` into `Unknown`, or `take_blocked` ignores it | `a_repeat_suppressed_record_is_consumed_like_a_swallowed_one_and_kept_by_a_reset` |
| X2 | `key_down_swallowed` counts `RepeatSuppressed` | `a_repeat_suppressed_key_is_not_skipped_by_the_modifier_resync_and_counts_for_the_mask` |
| X3 | rule 5's passing branch records `Passed` only over `Unknown` | `a_repeat_with_no_record_that_passes_after_a_swallowed_one_is_recorded_as_passed` |
| X4 | D10 reverted (no `mark_repeat_suppressed`) | commit 3's scenario |
| X5 | `Resync::new` with `hiding` 0 | `a_new_resync_plans_a_full_read_at_its_first_plan` |
| X6 | `bump` does nothing | `a_bump_plans_one_full_read`; S3 |
| X7 | `plan` does not fold the notes / folds with `>` | `a_changed_note_count_plans_a_full_read_a_smaller_one_included` (both); S8 (the first) |
| X8 | `plan` reads fresh codes, or drops `owed` | `a_modifier_touched_since_the_last_plan_is_left_out_and_read_at_the_next` |
| X9 | `plan` ignores `input_away` | `nothing_is_read_while_input_is_away_until_it_returns_or_a_key_event_is_decided` |
| X10 | a decided event does not clear `input_away` | the same test; S5 |
| X11 | `Resync::input_left` does not bump | S5 |
| X12 | a tracked injected event clears `input_away` | `an_injected_event_marks_its_modifier_fresh_and_does_not_bring_input_back` |
| X13 | `Engine::input_returned` does nothing | `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers`; S4 |
| X14 | `on_foreground_changed` does not bump | `every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers`; S3 |
| X15 | `Reloaded` bumps | `a_reload_makes_no_read` |
| X16 | `reset` skips `read_everything` | `swch_on_and_a_reinstall_read_everything_themselves_so_the_next_drain_reads_nothing` |
| X17 | `resync_modifiers` asks a generic code whose slot is not set, or ignores KS-14 | `a_drain_read_asks_the_eight_sided_modifiers_and_skips_a_swallowed_one` (both); S9 (the second) |
| X18 | `touched_by` gives a sided code no generic bit, or a generic code fewer than both sides | `a_sided_modifier_touches_its_generic_code_and_a_generic_one_both_sides` |
| X19 | `MainThread::drain` does not call `resync_if_due` / `MainState::input_returned` does not call the engine | `the_drain_reads_the_modifiers_after_a_note_and_not_again` / `a_return_signal_makes_the_next_drain_read_the_modifiers` |
| X20 | no note in `with_main`'s refusal / none for an unknown switch / no wake after a foreground change | `a_refused_call_notes_hidden_input` / `a_desktop_switch_to_an_unknown_destination_notes_hidden_input` / `a_foreground_change_queues_a_wake_for_the_drain` |
| X21 | `on_key` reads every modifier again (`sync_modifiers` restored) | `a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier`; R-F |
| X22 | `resync_from_os` reads 0x10-0x12 | `a_reset_never_takes_a_generic_modifier_from_the_os` |
| X23 | `track_injected` does nothing / ignores KS-14 / tracks every code / `decide_event` does not call it | I-1 and `the_daemons_own_injected_modifiers_pass_and_are_tracked` / `an_injected_key_up_of_a_held_swallowed_modifier_leaves_it_held` / `an_injected_event_of_a_key_that_is_not_a_modifier_changes_nothing` / `the_daemons_own_injected_modifiers_pass_and_are_tracked` |
| X24 | `reset` keeps the held keys for `RemappingSwitched` and `HookReinstalled`, as for `Reloaded` | S6, S7 |
| X25 | the check clears a held generic slot whatever Windows says | G1 |
| X26 | the check removed (both call sites) | R-A, R-B, R-D, R-E, R-F, R-H |
| X27 | the check allowed to set: it visits every modifier slot and writes Windows' state | R-F (step 1 asks 11); C-1 |
| X28 | the check's KS-14 skip removed | R-F (the A down after LWin asks 3); S9; C-1 |
| X29 | the check's generic read replaced by a sided one | C-2 |
| X30 | the check does not skip the event's own key | R-F (the LCtrl repeat asks 1); C-1 |
| X31 | the key-up check runs over every modifier, or at a key-up that passes | R-F (A up with LCtrl held asks 1; F13 up asks 3) |
| X32 | the key-up check removed | R-H |
| X33 | the drain read leaves a generic slot alone, or sets one | R-G |
| X34 | the drain's generic read replaced by a sided one (either side) | R-G (one of its two "kept" cases) |
| X35 | the unapplied skip removed from the check | C-3, C-4 |
| X36 | `HookEnv::unapplied` answers EMPTY / the callback marks nothing, or the guard drops before `CallNextHookEx` | C-7 / C-6 |
| X37 | `unapplied_while_running` marks a swallowed key-down, a key-up or a non-modifier | C-5 |
| X26 + X33 | both repairs removed (commit 7's state) | R-C (red only under both) |
| X38 | `late_callback` uses `>`; the signed-age rule removed | `lateness_counts_age_and_run_with_a_17_ms_boundary` |
| X39 | no note at the callback's exit / a note always | `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late` / `a_callback_for_a_fresh_event_notes_nothing` |
| X40 | an injected event's stamp counts | `an_injected_events_own_stamp_is_not_its_age` |
| X41 | E5 never notes / its check is removed from `run` / `HookLate` warns | `an_awake_stretch_of_17_ms_notes_hidden_input` / `the_main_loop_checks_its_awake_stretch_before_every_wait` / `hook_late_alone_is_reported_at_info_and_with_another_count_at_warn` |
| X42 | `note_unseen_input` does nothing | `unseen_input_counts_each_note` |
| X43 | `unseen_at_last_tick` always 0 | `a_tick_reports_raw_key_events_no_callback_explains` |
| X44 | `Ticker::pass` notes but does not call `hidden_input` (or the reverse) | `a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread` |
| X45 | the notes sum drops the watchdog's count | `the_drain_counts_the_watchdogs_notes` |
| X46 | the deferral reads anyway | `the_first_raw_key_event_after_idle_is_read_at_the_first_tick` |
| X47 | the deferral ignores `hook_installed` | `raw_input_before_the_hook_is_installed_is_read_and_not_counted`; `an_idle_wake_with_no_hook_installed_still_reads_at_once` |
| X48 | `raw_pending` keeps the old `quiet_since` | `a_deferred_first_read_starts_the_silence_at_the_wake` |
| X49 | the deferred wait keeps the batched bit | `a_deferred_wake_is_followed_by_a_wait_that_leaves_only_the_batched_input_out` |

## Critique response (round 1)

Kept as history; the current text supersedes it where marked.

| Item | Disposition (round 1) | Now |
|---|---|---|
| **I1** residuals against "an unknown case keeps the read" | Resolved by ruling W1: the witness reads at every swallowed key-up | stands |
| **I2** the clock base, injected stamps, slow later hooks, no normal-typing check | Resolved by ruling W4 (D8, D9, M8, U1, Q2) | stands; E5's clock changed (R2-15) |
| **M1** E5 notes after events of the same stretch | E5 `continue`s to the drain; R-M1 declared | stands (row 88) |
| **M2** short `SendInput` count, injected generic codes, R8 test, feature 6 | W8, W9, D2, D3, tests | stands; D3 amended (R2-5) |
| **M3** the deferral adds one tick per burst | "one more wake per burst" | **corrected** by critique v2 M3: one more tick only for a burst of a single raw key event (D12) |
| **M4** compile-red tests without mutations | mutations for commits 5, 8 and 9 | extended to every compile-red test (critique v2 M4) |
| **Open questions 1, 2** | no longer decide a stuck key | stand |

## Critique v2 response

| Item | Disposition | Where |
|---|---|---|
| **I1** The drain's read never repairs a generic slot | **Resolved** (ruling W2a; the critic's option 1 plus the check). The drain read clears a generic slot when its code reads up and never sets one, and the check (D16) clears a held generic slot at the next decided key event; both read the generic code ("without distinguishing between left and right"). Row 88 and D11 no longer claim a bound that does not hold: the generic phantom is gone. Pinned red-first by R-C (the critic's script: generic key-down, unnoticed key-up, `hidden_input_noted()`, CapsLock fires `SwitchLanguage`, red on commit 7), R-D (the same without the note: the check alone), R-G (the drain alone) and C-2 (the check's generic read) | D3, D4, D16; commits 8, 9; X25, X29, X33, X34 |
| **I2** W10 argued backwards; a passed modifier that a later hook swallows stays held for the whole hold | **Resolved** (ruling W2a). W10 is corrected with the Hooks Overview quote: a hook installed before the daemon's, or before its 2.4 reinstall, runs after it. Its repair is the check at the next decided key event, for every press, auto-repeats included. The round-1 argument ("Windows does not apply it either") is kept as W14, where it is true: a hook ahead of the daemon's. Agreed with the critic's note that a drain read after the modifier's own event comes after the next key is decided, and that D5 leaves it out as fresh; that is why the repair is at the event. Pinned by R-A (the critic's AutoHotkey trace, J right after an LAlt repeat included) and R-B (the blocker trace, Alt+Tab reaches Windows as Alt+Tab), both red on commit 7; manual check M10 on a real AutoHotkey remap. What remains is the key-up variant (a later hook that keeps a key-up the daemon passed leaves Windows with the key down: the other program's doing), declared in row 88 | D4 (W10, W14), D16, D11; R-A, R-B; X26; M10 |
| **M1** "Matching only" understates two visible effects | **Fixed.** Goal 4 drops "matching only". D11 lists the four effects of the one remaining direction: a binding with that modifier misses and its sibling acts (CapsLock switches the layout); the Alt+Shift blocker misses its partner (Windows switches the layout); no menu mask with that Alt/Win (menu bar or Start opens, KS-12's symptom); the next auto-repeat counts as a press. Row 88 names them, and D11's owner summary goes into `results.md`. The critic's (b) case, a blocker swallowing a whole Alt or Shift press on a phantom-held partner, is now repaired by the check before the blocker matches (R-B) | Goal 4, D11, row 88 |
| **M2** S4 and S5 do not fail on commit 6's tree; S9 does not catch X10 | **Fixed.** The old X13 (on_key syncs again) now lists only the count tests (X21: c6's test and R-F). S3-S8 are now phantom-released scenarios, each listed against the mutation of its own source (X6, X7, X10, X11, X13, X14, X24), which they fail under on HEAD. They are listed against neither X21 (master's per-event read would heal them at CapsLock's own event) nor X26 (the check never sets a flag, so they pass with or without it). S9 asserts `expect_nothing_fired()` on the second LShift press and gains an A key-down, so it catches both KS-14 mutations (X17 for the drain, X28 for the check). R-C is listed only under the compound X26 + X33 | Commit 7 table; mutation table |
| **M3** The deferral adds a wake only for a burst of one raw event | **Fixed** as asked. `errors.md:215` keeps "about 10" and "about 1000". D12's Cost, the module Cost doc, M6 (still about 45), M7 (W as before) and the CHANGELOG (13) say: one more tick only for a burst made of a single raw key event, such as a key-up whose key-down came 2 s or more earlier | D12, touch set, M6, M7, CHANGELOG (13) |
| **M4** Some compile-red tests have no named mutation | **Fixed.** Commit 4: X1, X2, X3. Commit 5: X16 (`reset` skips `read_everything`) among X5-X20. Commit 7: X23 (`track_injected` tracks every code). Commit 13: X48 (`raw_pending` keeps the old `quiet_since`), X49 (the deferred wait keeps the batched bit). Every new W2a and D17 test: X25-X37 | mutation table |
| **M5** Two bench gates do not show what the ruling asks | **Fixed.** (a) R1 gates H mean ≤ N mean + 0.5 × (P mean − N mean), all from one session. A revert of the removal on the engine path puts H at about P and fails, and stop rule (b) guarantees a Δ the gate can resolve. (b) The tester runs P (commit 2), N (commit 2 with the one line removed, in a scratch worktree) and H interleaved in one session, and every gate (R12, R12m, R12m2, R4, R1) compares only those numbers. The developer's probe sets the expected values and stop rules only | D13 |
| **Open question 1 (U4)** | Recorded as unverified (ruling OQ1): it sets only how likely I1's trigger is; D3 and D16 read the generic code, which is right either way. No manual check | U4 |
| **Open question 2 (StickyKeys)** | Ruling OQ2: W-table row W13 (STICKYKEYS: a modifier "latches ... until the user presses a non-modifier key or clicks a mouse button"), and manual check M9 with both endings (a key, a mouse click). A failure is a 3.5 follow-up in `impl.md` | D4 (W13), M9, U9 |
| **Open question 3 (Remote Desktop server stamps)** | Ruling OQ3: recorded under W4/U1 as U10. A wrong stamp costs reads between events and an `[INFO]` line; M8's note names it | U10, M8 |
| Strengths noted (W1 clean, D10 and commit 3, Lemmas 1 and 2, D15, D5, D6, D8, D12, MS Learn quotes) | Kept. The Lemmas are unchanged and gain the "check is a subset of master" argument (D11); D15 keeps its noticed/unnoticed model and adds the construction drain (E0) and the later-hook and generic steps | D11, D15 |

## Developer addendum (2026-10-20)

Written by the developer at the end of the lane, on `perf/latency-trim`. The commits are listed in `impl.md`. The text
above is revision 2 as the critic checked it. Where this addendum differs from it, the addendum describes the code, and
`rulings.md` round 3 is the reason.

### Round 3, V3-I1: D17 is dropped

Nothing of D17 was built. Master's reading of a nested decided event is kept: the check reads the outer modifier as
Windows has it then.

| Cut text (line) | Revision 2 says | Now |
|---|---|---|
| Revision 2 table, R2-3 (:48) | D17, TLS `UNAPPLIED`, `Env::unapplied`, U7 | dropped (V3-I1) |
| Type sketch (:191-196, :231-235) | `Env::unapplied`; `UNAPPLIED`, `Unapplied`, `unapplied_while_running`, `HookEnv::unapplied` | none of them exists. `env.rs` gained docs only |
| Spec re-located (:81, :82, :84, :85) | `env.rs` "gains `unapplied`"; `HookEnv` "used by ... D17"; `carry_out` "E4, D17"; Re-entry docs "D17 cites them" | `env.rs` docs only. `HookEnv` serves the check and the drain. The callback gains E4 only. The Re-entry docs (`keyboard_hook_callback.rs:108-117`) state the declared residual |
| Goal 2 (:113) | "(D16, D17)" | D16 only |
| W-table, W11 (:408) | "D17 covers a callback still on the stack" | Declared residual (row 88). A key event decided inside the hook call of a passed modifier key-down, while every later hook passes that key-down, reads the modifier up and clears it, as master does. The modifier then counts as released (direction B) until its next event, an auto-repeat or a key-up. What the user sees: that one nested key misses a binding that names the modifier. No source of such an event is known (U7). The PowerToys case, where the later hook that injects also swallows the outer key-down, is right, as on master |
| D11, "The check is a subset of master" (:690) | "It skips the event's own key, swallowed modifiers and unapplied codes" | "... the event's own key and swallowed modifiers". The subset argument now holds with no exception |
| D11, direction A (:708) | third exception: "a key whose callback is still on the stack (D17)" | removed; the residual above replaces it |
| D16 (:963) | "and `m` is not in `env.unapplied()` (D17; read once ...)" | removed |
| D16 (:984) | "The one exception is a decided event that runs while an earlier event's callback is still on the stack; D17 skips those keys" | "A decided event that runs while an earlier event's callback is still on the stack reads that key as Windows has it then, as master's sync did (V3-I1)" |
| D17 (:1010-1057) | the whole decision | dropped |
| Win32 table (:1066) | "Passing an event on (D17 evidence)" | evidence for the declared residual only; no new call |
| U7 (:1083) | "D17 makes the engine right whether or not it happens" | The case is declared (row 88) with master parity. Manual check M11 exercises the PowerToys variant |
| Thread map (:1099, :1113, :1122) | "+ D17 mark"; "`UNAPPLIED` and `HIDDEN_INPUT` are main-thread TLS `Cell`s"; passed modifier key-down "+ two TLS accesses (D17)" | no mark and no TLS access on the key path; `HIDDEN_INPUT` is the one new main-thread TLS `Cell`; a passed modifier key-down costs `k` reads |
| Touch set (:1137-1139, :1145, :1156) | `env.rs` `unapplied` (9); callback `UNAPPLIED`, `Unapplied`, `unapplied_while_running`, `HookEnv::unapplied` (9); glossary "Unapplied key" | `env.rs` docs (7, 9); the callback's commit 9 change is the Re-entry docs only; the book sync adds "Hiding count" and no "Unapplied key" |
| Commit 9 (:1368-1389) | "D16 and D17"; `Env::unapplied`; the adapter items; C-3..C-7; X25-X37 | D16 and the drain's generic clear. C-3, C-4, C-5, C-6 and C-7 are not written. The V3-I1 pin is `engine::tests::a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it` (`src/engine/mod.rs:1468-1489`): LWin passes, F13 is decided before Windows applies it, the exact `F13` binding fires and LWin stops counting. Its mutation is X50 |
| Risks R1, R3, R8 (:1535, :1541, :1545, :1556, :1559) | the `Unapplied` guard and its restore; `UNAPPLIED`; `touched_by` "for `unapplied`"; `VIRTUAL_KEY(raw)` "at the callback's D17 call" | No guard and no `UNAPPLIED`. `touched_by` serves `fresh` only. No `VIRTUAL_KEY(raw)` is added anywhere |
| Q5, the chore after the merge (:1604, :1606) | principle 1 "D17's guard"; principle 5 "`UNAPPLIED`" | principle 1 sanctions the witness, the KS-05 read, the check (D16), E4's tick read and its late note. Principle 5 gains `HIDDEN_INPUT` and `unseen_input` |
| Readiness checklist (:1644) | "D1-D17" | D1-D16 |
| Mutation table (:1717-1719) | X35, X36, X37 | removed; X50 added (below) |
| CHANGELOG | "Changed (7, then extended in 9)" | no text names D17 or the guard |

New mutation:

| # | Mutation | Caught by |
|---|---|---|
| X50 | D17 restored: the key-down check leaves out the codes of a passed modifier key-down whose hook call is still running. For the engine test, the call at `src/engine/mod.rs:565` passes `ModifierSet::ALL.without(ModifierSet::touched_by(VK_LWIN))` | `engine::tests::a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it` |
| X51 | the witness term removed (`src/engine/mod.rs:999`: `decision.suppress = swallow;`) | `engine::properties::no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed` (config default, seed 8; M2) |

**Manual check M11** (added): start PowerToys Keyboard Manager first, remapping Left Win to F13. Then start the daemon,
so that PowerToys' hook runs after the daemon's. Add a temporary `( keys: "F13", action: SwitchLanguage )` and remove it
afterwards. Expected: every LWin press switches the layout, and Start does not open. The config is
`D:/tmp/ks-orch/p2-2.5-latency-trim/cfg/m11.ron`; its cfgcheck output is in `impl.md`.

### Round 3, Minor M1-M5

- **M1.** R-F has the step "A up with LAlt held, passing", which asks 0 (`src/engine/mod.rs:1553`). X31 is split:
  - X31a, the key-up check over every modifier at a swallowed key-up, is caught by "LWin up, swallowed" (it asks 2,
    not 0) and by "F13 up, swallowed, with LAlt held" (3, not 1).
  - X31b, the check at a key-up that passes, is caught by "A up with LAlt held, passing" (1, not 0).
- **M2.** The `Unnoticed` property's mutation is **X51**, the witness term removed. At `src/engine/mod.rs:999`,
  `decision.suppress = swallow && !env.os_key_down(vk);` becomes `decision.suppress = swallow;`. The developer ran it:
  the property fails at config default, seed 8, `Unnoticed` mode, with "scenario: up CapsLock at +6735 ms: VK 0x14 is
  down in Windows but physically up". The tester repeats it.
  - The other candidate, X4 (D10 reverted), does **not** fail the property: its 64 seeds per config pass under X4, and
    only commit 3's scenario catches X4 ("Windows got a lone key-up of LShift").
  - So the property's random scripts never take D10's path: a modifier held with no record, its release lost, and a
    repeat that a binding swallows. D10 is pinned by the scenario alone. A property case that reaches D10's path is
    left as a candidate for 3.5 (`impl.md`).
- **M3.** Manual check M9 gains a latched step: tap LShift, then press CapsLock with nothing in between. Expected:
  `ToggleCapsLock`, which master gives if `GetAsyncKeyState` reads the latch down. The owner records what HEAD does. A
  difference from master is a follow-up for 3.5 (`impl.md`). W13's phantom-released cell: while Shift is latched with no
  LL hook event, a Shift binding misses and its sibling acts (CapsLock switches the layout), until the next count
  change or Shift's next event.
- **M4.** Direction B gains a fifth visible effect, in D11, in row 88 (`book/src/config/known-issues.md:156`) and in the
  owner summary: with that Ctrl or Win down in Windows, the `Alt+Shift` blocker swallows the Shift or Alt key-down, so
  Ctrl+Alt+Shift+K reaches the application as Ctrl+Alt+K. The owner summary gains the line "Ctrl+Alt+Shift+K can arrive
  as Ctrl+Alt+K".
- **M5.** Goal 4, row 88, the owner summary and `keys.md:194` say "the next key-down". Row 88 also says: "pressed again
  before any other key-down, it reads as an auto-repeat (row 63)". These texts are corrected:
  - D7 (:496-498), "Master's sync never read that key either": master's sync ran at every event, key-ups included, so
    a passing key-up between the lost release and the next press repaired the flag on master. HEAD repairs it at the
    next key-down only.
  - D11 (:705-706), "as on master": that holds only with no key event at all in between. The case the critic traced (a
    lost LCtrl key-up, then a passing C key-up, then an LCtrl press read as a repeat) is row 63's kind and is declared
    in row 88. The record is `Passed`, so nothing sticks.

### Round 3, the architect's questions

- **Q1 (R16).** The provisional bound, D-R16d and the HEAD check are applied. The numbers are in `impl.md`.
  - D-R16d **misses on P**, the tree the cut names (commit 2, after which it is run), in all six P runs. It passes on
    N and on commit 11, the tree D12 is applied to, in all five runs.
  - I read the rule on commit 11 and committed 12 and 13.
  - R16 at HEAD meets the provisional bound in two of three runs (5.1 and 5.6 µs against 20.2). The median passes.
    The third run, at full CPU load from other programs, gives 24.0 and misses.
  - The lane stops here for the orchestrator's ruling (`impl.md`, "Stop rules").
- **Q2 (E4, E5 on `GetTickCount`).** R17t p50 is 0.001 µs in every developer run, against the 100 ns stop rule. No
  fallback: E4 and E5 read `GetTickCount`.
- **Q5.** The chore after the merge is as above, without D17.
- **Q9.** Commit 7 is `3f0dc62`, the replay tree of commit 8's (`d5458d2`) red-first tests.
- **Q10.** P is commit 2, `62c9fd8`. `git diff --stat master 62c9fd8 -- src` lists `src/bench.rs` and
  `src/platform/timers.rs` (commit 1's doc lines).
- **Q11.** Recorded as a follow-up for 3.5 in `impl.md`.

### Other differences from revision 2

- **Row 88's and `keys.md:194`'s links** pin `045b8b4` (commit 5, which adds `resync.rs` and `modifiers.rs`), not the
  merge commit: its sha is unknown before the merge. A `--no-ff` merge keeps `045b8b4` reachable from `master`.
- **D9.** `summary` returns `Option<Report>` with a `warns` flag, not `Option<(log::Level, String)>`, and the log
  writer turns the flag into the level. `counters.rs` is on the hook path and names no logging
  (`logging::tests::the_functions_on_the_hook_path_log_nothing`).
- **D12.** The deferral's condition is one function, `defers_first_read(woke, mask, batched, hook_installed)`
  (`src/hook/watchdog/mod.rs`). `run_loop` calls it, and the `Drive` model calls it too, so that X46 and X47 mutate
  code the `Drive` tests run. `raw_batch` starts the ticks through `raw_pending`, and the two share one branch. The
  mutation sites:
  - X46, `defers_first_read` returns false;
  - X47, it ignores `hook_installed`;
  - X48, `raw_pending` keeps the old `quiet_since`;
  - X49, `woke_idle` leaves the watchdog idle (`raw_pending` sets no next tick).
- **Developer probe files.** The probes of D13 went to `bench-dev-raw.md` (P, N, commit 11), `bench-dev-reads.md`
  (six N runs of R12, R12m, R12m2, R17t and R17a) and `bench-dev-resume.md` (this session's P, commit 11 and HEAD), not
  to `bench-dev-diag.md`.
- **Mutation IDs X6 and X7** in the table are round 2's new ones. Round 1's X6 and X7 went with D7 (critique_v3, open
  question 3).
