# Cut: p1-1.6-one-winner — roadmap item 1.6 (one winner per key event), KS-13

Tree: `master` @ `9357cfa` (`.git/refs/heads/master` = `refs/remotes/origin/master`; merge of `feat/deferred-block-default`). Branch to cut: `feat/one-winner`. Every `path:line` below is at `9357cfa` unless it says `@7cb3521` or `@1dfd80a`.

## Verdict

CUT READY

Six non-blocking questions (Q1-Q6) at the end, each with a recommendation. None changes the commits below.

## Spec re-located

| Spec reference | Current file:line | Status | Note |
|---|---|---|---|
| roadmap.md:285 "specificity applies only between Immediate bindings", `keyboard_hook_callback.rs:213-238` @7cb3521 | binding loop `src/engine/mod.rs:379-517` in `on_key_down` (`:371-553`) | moved (Workstream A, engine) | Defect present |
| audit KS-13 `keyboard_hook_callback.rs:230` @7cb3521 (`immediate_fired`) | `src/engine/mod.rs:377` (declared), `:414-419` (the slot) | moved | Only guard; Immediate-only |
| audit KS-13 `:234-236` (every matching binding with `block_default` suppresses) | Immediate `src/engine/mod.rs:420-422`; Tap `:434-436`; Hold `:450-452`; DoubleTap `:466-468` | moved | OR over all matching `keys` bindings, now for every trigger (1.3) |
| audit KS-13 `:238-252` (deferred arms ignore the slot) | `src/engine/mod.rs:424-437` (Tap), `:438-453` (Hold), `:454-469` (DoubleTap) | moved | Defect present |
| audit KS-13 `keyboard_hook.rs:72` (sort by key count) | `src/hook/keyboard_hook.rs:110-115` calls `engine::sort_by_specificity` `src/engine/mod.rs:267-273` | moved | Stable sort; unchanged by this lane |
| audit KS-13 README.md:493 @7cb3521 ("more specific combinations are checked first") | gone; today `README.md:697` ("Priority bindings ... until roadmap item 1.6") and `README.md:391-393` (OR rule "until roadmap item 1.6") | gone/moved | Both rewritten (commit 2) |
| "candidates that pass the trigger-key rule and exact matching (merged in A2)" | `src/engine/matching.rs:58-63` (`is_trigger_key`), `:70-79` (`combo_matches`, `allow_extra_modifiers`) | moved (`82f542f`) | Reused unchanged |
| "condition checked first" | `src/engine/mod.rs:382-384` | unchanged | Reordered for `keys` bindings (D7) |
| "Auto-blockers (A4) keep their own rule" | `src/engine/mod.rs:388-400`, `matching.rs:92-101` | unchanged | Stays outside the election (D1) |
| Tap/Hold coexist on one key | `src/engine/gestures.rs:1-6` (separate tables), first-wins `arm_tap` `:94-112`, `arm_hold` `:128-154`; `double_tap` `:200-219` | unchanged | `double_tap` has no first-wins: the known two-DoubleTap defect (book known-issues row 25) |
| Repeat slot rule (1.1) "first match takes the slot on a repeat" | `src/engine/mod.rs:120-125` (doc), `:411-419` | unchanged | Generalised to the winner (D6) |
| Engine rule text and known-defects paragraph | `src/engine/mod.rs:112-140`, `:163-165` | — | Doc update (commit 2) |
| `Condition` has no `PartialEq` | `src/data/condition.rs:28` (`#[derive(Debug, Clone, Default)]`) | — | Needed by D3 and D9 |
| `KeyCombination` equality ignores order | `src/data/key_combination.rs:16-30` | unchanged | Group and duplicate key identity |
| Chord key-set equality | `src/engine/chords.rs:120-122` (`same_keys`, private) | unchanged | The loader needs its own (no `config -> engine` edge; D9) |
| Loader: where a warning can go | `src/config/loader.rs:229-269` (`from_ron_str_with_hotkeys`; existing `eprintln!("[WARN] config contains no bindings")` at `:264-266`), raw keys text `:479-490` | — | Seam (commit 3), warnings (commit 5) |
| Loader callers and threads | `src/main.rs:30` (`config::load`, main thread); `src/hook/config_watcher.rs:159` (`config::reload`, thread named `config-watcher` at `:124`) | unchanged | Where the warning prints |
| Pinned OR test, "until roadmap item 1.6" | `src/engine/scenarios.rs:3237-3259` (`passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole`) | — | Stays green (D5); comment updated |
| Test whose name states file-order precedence | `src/engine/scenarios.rs:257-272` (`first_binding_whose_condition_holds_wins_on_a_shared_key`) | — | Renamed (commit 2); body unchanged |
| README CapsLock Tap/Hold example next to the default `CapsLock` | `README.md:335-342`; template example `src/config/default_config.ron:259-271`, coexist text `:114-119` | — | Comment: replace the default CapsLock binding (D10) |
| Book @1dfd80a citations of the loop | `matching.md:113` (`mod.rs:365-500`), `:130` (`:403-453`); `triggers-conditions.md:41` (`:404-412`); `kinds.md:182`, `actions.md:257-260`, `known-issues.md:36-37` (`:409-411`, `:403-453`, `:261-267`) | moved | Current: `:371-553`, `:401-469`, `:410-423`, `:420-422`, `:267-273` |

KS issues: the item cites KS-13 only (`audit.md:390-403`, read). Its three scenarios, as they reproduce under exact modifiers (A2), are W1/W2, W4 and W6 below. The audit's literal first scenario (README pair without `allow_extra_modifiers` next to `Shift+CapsLock`) no longer reproduces at `9357cfa`: exact matching (`matching.rs:78`) keeps `"CapsLock"` from matching while Shift is held, which `capslock_binding_does_not_fire_while_shift_is_held` (`scenarios.rs:1139-1154`) pins. Also read for context: KS-04, KS-06, KS-10, KS-18 (CHANGELOG `:210-216`, `:228-250`, `:255-264`, `:288-301`).

## Goal and non-goals

**After the lane.**
- On every key-down, among the `keys` bindings that are not auto-blockers and whose keys match the press (trigger-key rule, exact modifiers or `allow_extra_modifiers`) and whose `when:` holds, one wins. Precedence: more keys; then a binding with a `when:` over one without; then the earlier one in the file.
- The winner alone runs and decides suppression. If the winner is `Tap`/`Hold`/`DoubleTap`, every deferred binding on the same keys with the same `when:` that matches the press joins it as one group, at most one binding per trigger kind. The group arms together and suppresses if any member has `block_default: true`.
- Auto-blockers, `sequence` and `chord` bindings work as today, next to the winner.
- The loader warns, once per later binding, about a binding that duplicates an earlier one. At this lane the warning goes to stderr; with 1.7 it goes through `log`.

**Non-goals (later items).**
- `sequence`/`chord` in the election: 3.2 (KS-15, KS-16) and 3.4 (Q1).
- Warning about an `Immediate` binding and a deferred binding on the same keys and `when:`, which no longer both act: 3.4 diagnostics, `roadmap.md:451` (Q2).
- One winner locked per press across its auto-repeats (Q3).
- Comparing `when:` conditions semantically, for example ignoring case (Q4).
- A warning when a binding that does not block replaces the auto-blocker: carried over from the 1.3 cut's Q2 (Q5).
- Sided keys as more specific than unsided ones (Q6).
- Interrupt rule for pass-through gestures (1.3 follow-up, 2.2).
- `when:` on auto-blockers (KS-35, 1.2).
- Switching to `log` (1.7).

**Dependants.** 3.4 (core consolidation, RC1 "by construction") builds on the election. 1.7 turns `eprintln!("[WARN] {w}")` into `log::warn!`. Checkpoint 1 smoke list: unaffected; the default config alone is unaffected (audit KS-13 "Verification").

## Decisions

### D1: Only `keys` user bindings compete

- **What.** The candidates are the `BindingKind::Combo` bindings with `!is_auto_blocker` that pass `matching::combo_matches` (`matching.rs:70-79`) and whose `when:` holds. These are not candidates and keep their branches unchanged:
  - auto-blockers (`mod.rs:388-400`);
  - `Sequence` (`:472-496`);
  - `Chord` (`:497-515`).
- **Why.**
  - The spec ("candidates that pass the trigger-key rule and exact matching") names rules that only `keys` bindings have.
  - The brief says "Auto-blockers keep their own rule". A blocker as a candidate would take the slot from a user binding on its keys (KS-18 regression; killed by `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order`, `scenarios.rs:2631-2659`).
  - Sequences and chords are state machines that must see every key-down to advance or abort (`sequences.rs:72-96`, `chords.rs:76-114`). Gating them on a winner would leave stale progress, and their matching rules are 3.2's job.
- **Rejected.** All kinds in one election: a sequence/chord must be fed before it is known whether it "matches", so a losing one would already have mutated state. That is 3.4's redesign.
- **Cost.** A key that is both a `keys` trigger and a sequence step or chord key still drives both (documented, Q1).

### D2: Precedence is an explicit rank, independent of the sort

- **What.** `rank(b) = (b.kind.key_count(), !b.condition.is_always())`, compared lexicographically. The winner is the first candidate in list order whose rank is strictly greater than every earlier candidate's (ties go to the earlier one). `when: Always` written out counts as no condition (`condition.rs:57-59`).
- **Why.**
  - The rank encodes the spec's three rules.
  - `update_bindings` keeps its stable sort (`keyboard_hook.rs:110-115`), so list order among equal key counts is file order, and the auto-blockers stay last (KS-18, R7).
  - The rank does not rely on the sort to put conditional bindings first. Sorting by condition (the alternative) would also reorder `sequence`/`chord` visits and change which of two same-step sequences stores its action (`sequences.rs:76-116`).
  - It uses the same `key_count` as the sort, so there is one definition of "more keys".
- **Rejected.**
  - (a) Extending `sort_by_specificity` with the condition: it changes sequence/chord/blocker visit order (above).
  - (b) Counting sided keys as more specific: not in the spec (Q6).
- **Cost.** One `(usize, bool)` comparison per candidate.

### D3: The gesture group

- **What.** If the winner `w` has a deferred trigger, binding `b` (not `w`) also acts on this key-down if all of these hold:
  - `b` is a `keys` user binding with a deferred trigger;
  - `b.combination() == w.combination()`, the order-insensitive equality at `key_combination.rs:16-30`;
  - `b.condition == w.condition`, a structural equality (new derive `PartialEq, Eq` on `Condition`);
  - `combo_matches(b, vk, held, b.allow_extra_modifiers)`.

  An `Immediate` winner acts alone.
- **Why.**
  - The spec says the same combo and the same condition form one group.
  - The equal-condition test means the condition is not evaluated again: it was already true for `w` in the same event. `HookEnv` caches the foreground per event (`keyboard_hook_callback.rs:100-119`), and `FakeEnv` is deterministic.
  - The `combo_matches` re-check is needed because members may differ in `allow_extra_modifiers`: a `CapsLock` Tap with the flag next to an exact `CapsLock` Hold, with Shift held. Pinned by W13.
  - Members have the same rank as `w`, so every matching member comes after `w` in list order: `w` is the first max-rank match.
- **Rejected.**
  - Grouping by combo only (ignoring `when:`): contradicts the spec.
  - Precomputed group ids on `Binding`: a new field that every constructor and the sort would have to fill. Runtime equality costs only a few comparisons, without allocation, and only when the winner is deferred.
- **Cost.**
  - A `Tap` and a `Hold` on the same keys with different `when:` no longer arm together where both hold (Breaking, W8, CHANGELOG).
  - Condition equality is "written the same way" (Q4).

### D4: At most one binding per trigger kind acts in a group

- **What.** While walking the list for the key-down, a local `GroupArms { tap, hold, double_tap: bool }` lets the first `Tap`, the first `Hold` and the first `DoubleTap` of the group act. A later one of the same kind is skipped entirely: no arm, no `block_default` contribution. The winner is visited first (D3 last bullet), so it is never the one skipped.
- **Why.**
  - "Exactly one binding wins". The group exception exists so that *different* triggers coexist (Tap+Hold).
  - `arm_tap` and `arm_hold` are already first-wins (`gestures.rs:101-106`, `:135-137`), so for them only the `block_default` of a dead duplicate stops counting.
  - `double_tap` has no first-wins. The second binding consumes the first's entry and fires on the *first* press (`gestures.rs:207-212`; book known-issues row 25, `known-issues.md:63`). D4 fixes that for one group (W9).
- **Rejected.** A first-wins inside `Gestures::double_tap`: it is keyed by vk only, so it would also change cross-press behaviour.
- **Cost.** Three bools on the stack per key-down.

### D5: Suppression comes from the winner or its group, plus the independent mechanisms

- **What.** The key-down asks for suppression if any of these holds:
  - the winner (Immediate) or any acting group member has `block_default: true`;
  - an auto-blocker matches;
  - a `sequence` or `chord` starts, advances or completes.

  Everything after the loop is unchanged: the repeat and record rules (`mod.rs:519-551`) and the key-up rule (`:555-581`).
- **Why.**
  - It is the spec's "only the winner decides suppression".
  - OR inside a group keeps `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (`scenarios.rs:3237-3259`) green. A press is either swallowed or not, and a Hold that turns CapsLock into Shift needs it swallowed.
  - The suppression invariant is still carried by the `LastDown` record (`keys.rs:51-66`), not by the bindings.
- **Rejected.** Using the group leader's `block_default` only: a silent change for Tap(false)+Hold(true) pairs, with no spec basis.
- **Cost.** None.

### D6: Election per key event, auto-repeats included

- **What.** Each key-down, auto-repeat or not, runs the election. An `Immediate` winner fires on a repeat only with `repeat` (`mod.rs:416`, unchanged). A deferred winner's group ignores repeats (`continue`, as at `:425`, `:439`, `:455`). A lower-ranked `Immediate` binding therefore never fires in its place on a repeat: 1.1's "takes the slot" rule (`mod.rs:120-125`), extended to deferred winners (W6).
- **Why.** The spec says "per key event". Re-election per event is today's semantics for the Immediate slot, and reload mid-hold relies on it (`reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat`, `scenarios.rs:1831-1850`). Repeat suppression follows the record whatever the winner is.
- **Rejected.** Locking the key-down's winner per vk for the press: new per-key state that must survive reloads (indices shift), for a case the spec does not ask for (Q3).
- **Cost.** A repeat after a modifier changes mid-hold can elect another binding (documented under Risks; unchanged from today).

### D7: Cheap checks before the foreground lookup

- **What.** Order inside the election for each candidate:
  1. Compute the rank (no I/O). If a winner so far has a rank `>=` this one, skip the candidate.
  2. `combo_matches` (table lookups, `matching.rs:70-79`).
  3. Only then `condition.evaluate`.

  `sequence` and `chord` keep "condition first" (`mod.rs:382-384`), because their feed mutates state.
- **Why.**
  - On the hook thread a `when:` evaluation may capture the foreground: `GetForegroundWindow`, `OpenProcess`, `QueryFullProcessImageNameW` or `GetWindowTextW` (`condition.rs:113-167`; LowLevelHooksTimeout quote below).
  - Today every binding's condition is evaluated on every key-down before its keys are checked (`mod.rs:382`). After D7 a conditional binding costs a lookup only on a press of its own keys.
  - Both predicates are pure within one event, so the order cannot change any decision. Pinned by W16.
- **Rejected.** Keeping condition first: needless Win32 work on the hook thread (KS-08 budget).
- **Cost.** None.

### D8: Where the code lives

- **What.** A new module `src/engine/winner.rs` (pure; under `#![forbid(unsafe_code)]` from `engine/mod.rs:167`), declared in `engine/mod.rs` next to `mod matching;`. Signatures:

  ```rust
  /// The `keys` binding that wins a key-down (roadmap item 1.6, KS-13).
  pub(super) struct Winner<'a> { index: usize, binding: &'a Binding }

  /// Picks the winner among `bindings` for the key-down of `vk`, with `held`
  /// the keys held after it; `None` when no `keys` binding matches (D1, D2, D7).
  pub(super) fn elect<'a>(
      vk: VIRTUAL_KEY,
      bindings: &'a [Binding],
      held: &KeyState,
      foreground: &dyn ForegroundInfo,
  ) -> Option<Winner<'a>>;

  impl Winner<'_> {
      /// Whether the `keys` user binding at `index` acts on this key-down:
      /// the winner itself, or a member of its gesture group (D3).
      pub(super) fn admits(&self, index: usize, binding: &Binding, vk: VIRTUAL_KEY, held: &KeyState) -> bool;
  }

  /// The trigger kinds the winning group has used on this key-down (D4).
  #[derive(Default)]
  pub(super) struct GroupArms { tap: bool, hold: bool, double_tap: bool }

  impl GroupArms {
      /// True for `Immediate` and for the first `Tap`, `Hold` and `DoubleTap`
      /// claimed on this key-down; false for a later one of the same kind.
      pub(super) fn claim(&mut self, trigger: Trigger) -> bool;
  }
  ```

  `Engine::on_key_down` (`mod.rs:371-553`) changes as follows:
  - It calls `elect` once, after `self.keys.press(vk)` (`:373`), with `EnvForeground(env)` (`:374`).
  - `immediate_fired` (`:377`) is removed.
  - In the loop, the `when:` check (`:382-384`) moves into the blocker, `Sequence` and `Chord` branches.
  - A non-blocker `Combo` continues unless `winner.admits(..)`.
  - The Immediate arm (`:410-423`) fires when `!was_repeat || binding.repeat`; it no longer needs the slot flag.
  - Each deferred arm, after its `was_repeat` `continue`, continues unless `arms.claim(binding.trigger)`.
  - Everything from `:519` on is unchanged.
- **Why.**
  - The decision logic stays in `engine`, where `Scenario` tests reach it.
  - `mod.rs` does not grow.
  - `Winner` borrows the binding, so no index needs re-checking.
  - The module uses only existing edges (`engine -> data`, `engine::matching`, `engine::keys`), so it adds no module cycle.
- **Rejected.** Inline in `on_key_down`: the function is already about 180 lines.
- **Cost.** One file.

### D9: The duplicate warning

- **What.**
  - **Data-level predicates** (`src/data`):
    - `Trigger::same_kind(self, other: Trigger) -> bool`: an explicit `matches!` over the four pairs, ignoring the term.
    - `BindingKind::same_keys(&self, other: &BindingKind) -> bool`:
      - `Combo`: `KeyCombination ==` (any order).
      - `Sequence`: steps equal element-wise, in order; `max_gap` is not compared.
      - `Chord`: equal length and every key contained, as at `chords.rs:120-122`; `window` is not compared.
      - Different kinds: false.
    - `Binding::duplicates(&self, earlier: &Binding) -> bool`:
      - neither binding is an auto-blocker;
      - `self.kind.same_keys(&earlier.kind)`;
      - `self.trigger.same_kind(earlier.trigger)`;
      - `self.condition == earlier.condition`;
      - `self.allow_extra_modifiers == earlier.allow_extra_modifiers`.
  - **Loader**: `fn duplicate_warnings(raw: &[RawBinding], bindings: &[Binding]) -> Vec<String>`. For each `j` in file order, the smallest `i < j` with `bindings[j].duplicates(&bindings[i])` gives one line: `binding #{j} [{keys_j}] duplicates binding #{i} [{keys_i}]; remove one of them`, with `keys` from `format_raw_binding_keys` (`loader.rs:479-490`). `raw[k]` and `bindings[k]` are the same entry, because any conversion error returns before this step (`loader.rs:260-262`).
  - **Printing**: from `from_ron_str_with_hotkeys` as `eprintln!("[WARN] {warning}")`. That runs on the main thread at startup and on `config-watcher` on reload. The file still loads.
- **Why.**
  - The spec says "same kind, combo, trigger and condition".
  - The trigger is compared by kind, because a `Tap(300)` after a `Tap(200)` on the same keys and `when:` never acts either (D4, `gestures.rs:101-106`).
  - `allow_extra_modifiers` must match, because `CapsLock` with the flag still fires where an exact `CapsLock` cannot.
  - Sequence and chord duplicates share one pending slot (`sequences.rs:8-10`, `chords.rs:80-85`) and misbehave in ways that differ by case. The message therefore makes no claim about which one runs, only "remove one of them".
  - Data-level predicates are unit-testable, and the loader keeps no `config -> engine` edge (`engine::testing` imports `config`, `testing.rs:56`).
- **Rejected.**
  - Comparing the trigger term ("exact"): it misses dead `Tap(300)` copies.
  - Warning for `Immediate` vs deferred on the same keys and `when:`: beyond the spec (Q2).
  - A load error: the brief says warn.
- **Cost.** O(n²) at load time on the main or `config-watcher` thread, never on the hook thread.

### D10: A seam that makes the warning testable

- **What.** In `loader.rs`, a private `struct ParsedConfig { bindings: Vec<Binding>, warnings: Vec<String> }` and a private `fn parse_ron_str(text: &str, system_combination: &SystemCombination) -> Result<ParsedConfig, ConfigError>`, which holds today's body of `from_ron_str_with_hotkeys` (`:229-269`). `from_ron_str_with_hotkeys` becomes `parse_ron_str`, then one `eprintln!("[WARN] {w}")` per warning, then the bindings. Commit 3 keeps `warnings` empty, so it is a pure refactor.
- **Why.** The acceptance line asks for a loader test of a warning, and stderr cannot be captured in a unit test without new machinery. 1.7 will route `warnings` to `log`.
- **Rejected.** Changing the public `load`/`reload` signatures: that ripples into `main.rs` and `config_watcher.rs` for no gain now.
- **Cost.** One private type.

### D11: Documentation goes in the commits that change the meaning

- README, `default_config.ron` and CHANGELOG change in commit 2 (engine) and commit 5 (warning). The book config pages change in commit 6.
- The README example and the template example of the CapsLock Tap/Hold pair now say to *replace* the default `CapsLock` binding. Listed above it, the default binding would win every press. Today both act, and the layout switches twice per tap.

## Win32 API choices

No new API, no new `windows` feature (`Cargo.toml:41-51` unchanged), no change in `src/hook`. The existing calls this lane relies on:

| Purpose | `windows` path | Feature (present?) | Thread | Errors | Doc link and quoted constraint |
|---|---|---|---|---|---|
| Swallow or pass the event (adapter, unchanged, `keyboard_hook_callback.rs:257-267`) | `Win32::UI::WindowsAndMessaging::CallNextHookEx`; `LRESULT(1)` | `Win32_UI_WindowsAndMessaging` (yes, `Cargo.toml:44`) | main (installing) thread | n/a | [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc): "If the hook procedure processed the message, it may return a nonzero value to prevent the system from passing the message to the rest of the hook chain or the target window procedure." |
| Hook-thread budget (why D7 orders the checks) | same callback | — | main | — | Same page: "If the hook procedure times out, the system passes the message to the next hook. However, on Windows 7 and later, the hook is silently removed without being called. There is no way for the application to know whether the hook is removed." |
| Foreground title for `TitleContains`/`TitleEquals`, now asked less often (D7) | `Win32::UI::WindowsAndMessaging::GetWindowTextW` (`condition.rs:156-167`) | yes (`:44`) | main | 0 → `None` | [GetWindowTextW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowtextw): "If the target window is owned by the current process, GetWindowText causes a WM_GETTEXT message to be sent to the specified window or control." This is the re-entry case the adapter docs warn about (`keyboard_hook_callback.rs:66-69`). The daemon owns no window, so no change; fewer calls only lower the exposure |
| Foreground executable (`AppEquals`) | `Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW}` (`condition.rs:185-219`) | `Win32_System_Threading` (yes, `:50`) | main | `Result` → `None` | Unchanged (KS-03); fewer calls after D7 |

- **`LLKHF_INJECTED`, extended-key flag, `scanCode`.** Not read (unchanged). The daemon's own events are skipped before the engine (`keyboard_hook_callback.rs:166-168`). Other injectors' events compete in the election like physical ones, as today.
- **AltGr.** Its fake LCtrl comes with RAlt. With `Ctrl+RAlt` and `RAlt` + `allow_extra_modifiers: true` both bound, `Ctrl+RAlt` now wins (more keys). Today the first Immediate in list order (also `Ctrl+RAlt`) won, but a deferred `RAlt` armed too.
- **Threads.**
  - `elect` and the group run on the main thread, inside `KeyboardHook::with_engine` (`keyboard_hook_callback.rs:178-182`).
  - `on_timer` on `hold-timer` and `reset` on the IPC worker are unchanged.
  - The duplicate warning is built and printed on the main thread (`main.rs:30`) or on `config-watcher` (`config_watcher.rs:159`).
  - No new thread, timer, window, COM apartment, lock, atomic or `static`.
- **Elevation, UIPI, pipe DACL, token inheritance.** Unaffected.

## Touch set

| File | Commit | Change | Why |
|---|---|---|---|
| `src/engine/scenarios.rs` | 1 | New section `// ---- One winner per key event (roadmap item 1.6, KS-13) ----` with W1-W14 (table below) | Red-first |
| `src/engine/properties.rs` | 1 | `CONFIGS: [Config; 7]`, new config `winners` (below); module doc (`:1-28`) names it | Symmetry and stuck checks over the election |
| `src/engine/mod.rs` (tests) | 1 | `ForegroundForbiddenEnv` + W16 | Red-first (D7) |
| `src/engine/winner.rs` | 2 | New module (D8), module doc, unit tests `rank_puts_more_keys_before_a_when_condition`, `group_arms_claims_each_deferred_kind_once` | D1-D4, D7 |
| `src/engine/mod.rs` | 2 | `mod winner;`; `on_key_down` per D8; rule 4 text `:112-140` (election, group, suppression, blockers/sequence/chord unchanged); `:163-165` drop KS-13 from the known defects; `sort_by_specificity` doc `:267-273` (the sort gives file order among equal key counts; precedence is `winner::elect`) | D2, D5, D6 |
| `src/data/condition.rs` | 2 | `#[derive(Debug, Clone, Default, PartialEq, Eq)]` (`:28`) plus a doc line: equality is structural, used to group gestures and find duplicates | D3, D9 |
| `src/data/trigger.rs` | 2 | Module doc `:9-12`: coexist "on the same keys with the same `when:`; an `Immediate` binding on those keys competes with them (roadmap item 1.6)" | Accuracy |
| `src/data/binding.rs` | 2 | `key_count` doc `:35-39`: the sort key and the first part of a `keys` binding's precedence | Accuracy |
| `src/engine/scenarios.rs` | 2 | Rename `first_binding_whose_condition_holds_wins_on_a_shared_key` → `a_binding_whose_condition_holds_wins_on_a_shared_key` (`:257`); comment of `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (`:3239-3241`): "Tap and Hold on the same keys and `when:` are one group, which swallows the key if either asks (roadmap item 1.6)" | Names and comments match the rule |
| `README.md` | 2 | New paragraph **Which binding wins.** after the exact-modifiers RON block (`:248-255`), with a RON block (validated by `readme_matching_examples_load`, `loader.rs:1893-1922`); triggers intro `:331-333`; CapsLock example comment `:336` ("replace the default CapsLock binding with these two"); resolution bullet `:364-369` ("the winning group arms its gestures, one per trigger"); behaviour rule `:391-393` (replaces the OR sentence); `block_default` row `:490` ("only the binding that wins the press counts"); feature bullet `:697` | Meaning change |
| `src/config/default_config.ron` | 2 | Coexist text `:114-119` (+ precedence, 4-5 ASCII lines); `block_default` comment `:144-150` (+ "only the binding that wins a key press counts"); example comment `:259-260` ("Replace the CapsLock binding at the top with these two: a binding listed first on the same keys wins every press") | Meaning change |
| `CHANGELOG.md` | 2 | **Changed**: Breaking entry (text below), appended after `:132-151`. **Fixed**: KS-13 entry after the KS-25 entry (`:269-274`), and the two-DoubleTap entry | Meaning change |
| `src/config/loader.rs` | 3 | `ParsedConfig`, `parse_ron_str`; `from_ron_str_with_hotkeys` prints and returns (D10) | Seam |
| `src/config/loader.rs` (tests) | 4 | L1-L4; helper `fn template_examples_ron() -> String`, split out of `template_examples` (`:662-678`), which then uses it | Red-first |
| `src/data/trigger.rs` | 5 | `pub fn same_kind(self, other: Trigger) -> bool` + unit test `same_kind_ignores_the_term` | D9 |
| `src/data/binding.rs` | 5 | `BindingKind::same_keys`, `Binding::duplicates` (doc comments) + unit tests `same_keys_compares_combos_and_chords_as_sets_and_steps_in_order`, `duplicates_needs_the_same_kind_keys_trigger_kind_when_and_flag`, `an_auto_blocker_duplicates_nothing` | D9 |
| `src/config/loader.rs` | 5 | `duplicate_warnings`, called by `parse_ron_str` on the user bindings before `expand_with_auto_blockers` (`:268`) | D9 |
| `README.md` | 5 | Errors section `:539-567`: a paragraph after the list, quoting the warning; one sentence at the end of **Which binding wins.** | New console message |
| `CHANGELOG.md` | 5 | **Added** entry after the `repeat` entry (`:48-55`) | New message |
| `book/src/config/matching.md` | 6 | §16.2 `:111-120` rewritten (election, group, blockers/sequence/chord, suppression); §16.3 `:122-132`: the `when:` priority bullet `:126`, the duplicate bullet `:127` (now warned about), the KS-13 warning `:129-130` → NOTE "fixed on `feat/one-winner`" with the Immediate+deferred and different-`when:` consequences | Config reference |
| `book/src/config/triggers-conditions.md` | 6 | §11.3 `:43-44`, `:47`; §11.4 `:63`; §11.6 `:86` (one group: same keys and `when:`); §11.7 `:110-117` (first of each kind in the group; the DoubleTap warning `:116-117` → NOTE fixed; the loader warns) | Config reference |
| `book/src/config/kinds.md` | 6 | §10 table rows `:169-170` ("if the winning binding, or one of its group, has `block_default: true`"); warning `:181-182` → NOTE fixed (`feat/one-winner`) | Config reference |
| `book/src/config/actions.md` | 6 | §13.9 `:257-260`: "takes the slot" rewritten as winning; warning → fixed | Config reference |
| `book/src/config/known-issues.md` | 6 | Rows 10 (`:36`), 11 (`:37`) and 25 (`:63`) → **Fixed** (`feat/one-winner`); new rows 69-72: Immediate + deferred on the same keys and `when:` (first in file wins, no warning, Q2); Tap/Hold with different `when:` do not group; sided vs unsided equal key count, file order (Q6); the duplicate warning compares `when:` as written (Q4) | Pitfalls index |
| `book/src/config/errors.md` | 6 | §18.6 table `:170-191`: row `[WARN] binding #N [...] duplicates binding #M [...]; remove one of them`; §18.7 `:202`: "duplicate bindings are warned about; an Immediate and a deferred binding on the same keys and `when:` are not" | Config reference |
| `book/src/config/example.md` | 6 | Comment `:126-127`: the binding with the condition wins over the general one wherever it sits | Accuracy |
| Not touched | — | `src/hook/keyboard_hook.rs` (listed by `roadmap.md:287`): the sort and publish stay as they are (D2), and only doc lines would mention the rule. `src/config/schema.rs` and `src/config/mod.rs`: no field and no export changes | — |
| Deferred to the Phase-1 book sync (doc-writer, `docs/book-phase-1`) | — | `architecture.md:207`, `:212`, `:543`; `glossary.md:85`, `:401`; `gaps.md:185`; `overview.md:17`; `roadmap.md:283-289` status, `:617`; `audit.md:36`, `:394` status lines (Fixed in `<sha>` on `feat/one-winner`) | Book owner |

All new and changed RON in README, the template and the book is validated with `cfgcheck` (`methodology.md#validation-harness`), built under `D:/tmp/ks-orch/p1-1.6-one-winner/cfgcheck`. The book is built with `mdbook build book -d D:/tmp/ks-orch/p1-1.6-one-winner/book` and its links are checked.

**New README block (commit 2), validated by `readme_matching_examples_load`:**

```ron
// Shift+CapsLock has more keys: it wins, and the Tap does not arm with it.
( keys: "Shift+CapsLock", action: ToggleCapsLock ),
( keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true ),
// In Spotify the binding with a when: wins, wherever it sits in the file.
( keys: "Pause", action: SwitchLanguage ),
( keys: "Pause", action: Media(PlayPause), when: AppEquals("Spotify.exe") ),
```

**CHANGELOG text.**
- **Changed (commit 2).** "**Breaking:** one binding wins each key press (roadmap item 1.6, KS-13).
  - Among the `keys` bindings that match a key-down, the one with more keys wins; with equal key counts, a binding with `when:` beats one without; then the one higher in the file.
  - Only the winner runs its action and decides whether the key is swallowed. `Tap`, `Hold` and `DoubleTap` bindings on the same keys with the same `when:` win or lose together, so the README's `Tap(200)` + `Hold(200)` pair on CapsLock works as before. Of two such bindings with the same trigger, only the first acts.
  - Auto-blockers, `sequence` and `chord` bindings keep their own rules.

  Consequences:
  - A losing binding no longer swallows the key: with `( keys: "Shift+F13", action: ToggleCapsLock, block_default: false )` and `( keys: "F13", action: SwitchLanguage, allow_extra_modifiers: true )`, Shift+F13 reaches applications.
  - A losing `Tap`, `Hold` or `DoubleTap` no longer arms: next to `Shift+CapsLock`, a `CapsLock` `Tap(200)` with `allow_extra_modifiers: true` no longer switches the language when Shift+CapsLock is released. A more specific `Tap` on `Ctrl+F22` no longer lets `F22` with `allow_extra_modifiers: true` fire on the same press.
  - A binding with `when:` wins over one without on the same keys wherever it sits in the file; it used to win only when listed first.
  - An `Immediate` binding and a `Tap`, `Hold` or `DoubleTap` binding on the same keys with the same `when:` no longer both act: the higher one wins every press. Adding the README's CapsLock `Tap`/`Hold` pair below the default `CapsLock` binding leaves the pair inert (it used to switch the layout twice per tap); replace the default binding with the pair.
  - A `Tap` and a `Hold` on the same keys with different `when:` no longer arm together where both conditions hold; give them the same `when:`."
- **Fixed (commit 2).**
  - "KS-13: specificity decides across every trigger. Of the matching `keys` bindings only the winner (or its `Tap`/`Hold`/`DoubleTap` group) acts. A less specific binding with `block_default: true` no longer swallows the key of a more specific one with `block_default: false`, and a less specific `Tap`, `Hold` or `DoubleTap` no longer arms next to a more specific binding. Tests: `engine::scenarios::shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers`, `a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order`, `a_more_specific_winner_with_block_default_false_lets_the_key_through`."
  - "Two `DoubleTap` bindings on the same keys and `when:` no longer fire the first one's action on the first press; only the first binding acts."
- **Added (commit 5).** "The loader warns about a binding that duplicates one above it: same kind (`keys`, `sequence` or `chord`) and keys (in any order for `keys` and `chord`, in order for `sequence` steps), the same trigger whatever its term, the same `when:` written the same way and, for `keys`, the same `allow_extra_modifiers` (roadmap item 1.6). For example `[WARN] binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them`. The file still loads. The warning is printed at startup and on every reload."

## Commits, in order

Each message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. The gate after each commit is `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`. Commits 1 and 4 are declared red: they compile, and only the listed tests fail. The developer replays each red set on `master` (commit 1) and on commit 3 (commit 4) and pastes the first failing line of each test into `impl.md`.

**1. `Engine: red-first: one winner per key event (1.6, KS-13)`.** Declared red; compiles on `master`.

Shared configs: `SHIFT_CAPS_AND_CAPS_TAP` = `(keys: "Shift+CapsLock", action: ToggleCapsLock), (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true)`. Each scenario uses `check_press_symmetry()` and `check_nothing_stuck()` unless the row says otherwise.

| # | Test (`engine::scenarios::` unless named) | Config / script | Predicted on `master` |
|---|---|---|---|
| W1 | `shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers` (acceptance 1) | `SHIFT_CAPS_AND_CAPS_TAP`. down LShift passed; +20 down CapsLock suppressed, fired `[ToggleCapsLock]`; +50 up CapsLock suppressed, nothing fired; up LShift passed; +300 nothing. Then down CapsLock suppressed, nothing; +50 up suppressed, fired `[SwitchLanguage]`; total `[ToggleCapsLock, SwitchLanguage]` | RED: `up CapsLock at +70 ms: expected to fire [], fired [SwitchLanguage]` |
| W2 | `shift_capslock_held_past_the_term_fires_no_capslock_hold` (KS-13 "held for 200 ms") | `SHIFT_CAPS_AND_CAPS_TAP` + `(keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200), allow_extra_modifiers: true)`. down LShift; +20 down CapsLock suppressed, fired `[ToggleCapsLock]`; advance 300 nothing fired; up CapsLock suppressed, nothing; up LShift passed; total `[ToggleCapsLock]` | RED: `advance 300 ms to +320 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(160))]` (Debug of `VK_LSHIFT`) |
| W3 | `a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order` (acceptance 2) | Two configs in a loop with `with_context`: "conditional binding first" = `(keys: "Pause", action: Media(PlayPause), when: AppEquals("Spotify.exe")), (keys: "Pause", action: SwitchLanguage)`; "unconditional binding first" = the same two swapped. Each: `foreground("Spotify.exe", "Spotify Premium")`; down Pause suppressed, fired `[Media(PlayPause)]`; up suppressed; `foreground("notepad.exe", "Untitled - Notepad").foreground_changed()`; down Pause suppressed, fired `[SwitchLanguage]`; up suppressed | RED (context `unconditional binding first`): `down Pause at +0 ms: expected to fire [Media(PlayPause)], fired [SwitchLanguage]` |
| W4 | `a_more_specific_winner_with_block_default_false_lets_the_key_through` (acceptance 3; KS-13 Shift+X) | `(keys: "F13", action: SwitchLanguage, allow_extra_modifiers: true), (keys: "Shift+F13", action: ToggleCapsLock, block_default: false)`. down LShift passed; down F13 passed, fired `[ToggleCapsLock]`; +500 repeat F13 passed, nothing; up F13 passed; up LShift passed; down F13 suppressed, fired `[SwitchLanguage]`; up suppressed | RED: `down F13 at +0 ms: expected the event to be passed, it was suppressed` |
| W5 | `a_conditional_winner_with_block_default_false_lets_the_key_through` (acceptance 3) | `(keys: "F14", action: SwitchLanguage), (keys: "F14", action: ToggleCapsLock, block_default: false, when: AppEquals("code.exe"))`; `foreground("code.exe", "main.rs")`; down F14 passed, fired `[ToggleCapsLock]`; +500 repeat passed, nothing; up passed; `foreground("notepad.exe", "Untitled - Notepad").foreground_changed()`; down F14 suppressed, fired `[SwitchLanguage]`; up suppressed | RED: `down F14 at +0 ms: expected the event to be passed, it was suppressed` |
| W6 | `a_more_specific_tap_keeps_a_less_specific_immediate_binding_from_firing` (KS-13 "deferred more specific, Immediate less specific") | `(keys: "F22", action: PressKey("Left"), allow_extra_modifiers: true), (keys: "Ctrl+F22", action: ToggleCapsLock, trigger: Tap(200), block_default: false)`. down LCtrl passed; down F22 passed, nothing; +50 up F22 passed, fired `[ToggleCapsLock]`; down F22 passed, nothing; +500 repeat F22 passed, nothing; up F22 passed, nothing; up LCtrl passed; down F22 suppressed, fired `[PressKey(VK_LEFT)]`; up suppressed; total `[ToggleCapsLock, PressKey(VK_LEFT)]` | RED: `down F22 at +0 ms: expected the event to be passed, it was suppressed` |
| W7 | `a_conditional_tap_and_hold_pair_arms_as_one_group_over_an_unconditional_binding` | `(keys: "CapsLock", action: ToggleCapsLock), (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), when: AppEquals("code.exe")), (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200), when: AppEquals("code.exe"))`; `foreground("code.exe", "main.rs")`; down CapsLock suppressed, nothing; +80 up suppressed, fired `[SwitchLanguage]`; +100 down, nothing; +200 fired `[PressKey(VK_LSHIFT)]`; up suppressed, nothing; `foreground("notepad.exe", ..).foreground_changed()`; down suppressed, fired `[ToggleCapsLock]`; +300 nothing; up suppressed; total `[SwitchLanguage, PressKey(VK_LSHIFT), ToggleCapsLock]` | RED: `down CapsLock at +0 ms: expected to fire [], fired [ToggleCapsLock]` |
| W8 | `a_tap_and_a_hold_with_different_when_conditions_do_not_arm_together` | `(keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), when: AppEquals("code.exe")), (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200))`; `foreground("code.exe", ..)`; down CapsLock suppressed; advance 300 nothing; up suppressed, nothing; +50 down; +80 up fired `[SwitchLanguage]`; `foreground("notepad.exe", ..).foreground_changed()`; down; advance 300 fired `[PressKey(VK_LSHIFT)]`; up nothing | RED: `advance 300 ms to +300 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(160))]` |
| W9 | `two_double_taps_on_the_same_keys_fire_only_the_first_on_the_second_press` | `(keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(250)), (keys: "F13", action: SwitchLanguage, trigger: DoubleTap(250))`; down F13 suppressed, nothing; +60 up; +100 down fired `[ToggleCapsLock]`; up; total `[ToggleCapsLock]` | RED: `down F13 at +0 ms: expected to fire [], fired [ToggleCapsLock]` |
| W10 | `a_more_specific_tap_keeps_a_less_specific_hold_from_arming` | `(keys: "F13", action: SwitchLanguage, trigger: Hold(200), allow_extra_modifiers: true), (keys: "Shift+F13", action: ToggleCapsLock, trigger: Tap(200))`; down LShift; down F13 suppressed, nothing; advance 300 nothing; up F13 suppressed, nothing; up LShift; down LShift; down F13; +50 up F13 fired `[ToggleCapsLock]`; up LShift; total `[ToggleCapsLock]` | RED: `advance 300 ms to +300 ms: expected to fire [], fired [SwitchLanguage]` |
| W11 | `a_gesture_armed_by_the_winner_resolves_after_a_reload_adds_a_more_specific_binding` (reload mid-gesture) | `(keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true)`; down CapsLock suppressed; +50 `reload(SHIFT_CAPS_AND_CAPS_TAP)`; up CapsLock suppressed, fired `[SwitchLanguage]`; +100 down CapsLock; +50 up fired `[SwitchLanguage]` | GREEN (guard) |
| W12 | `a_passing_user_binding_does_not_stop_the_auto_blocker` (blockers keep their rule) | `with_alt_shift_blocker(r#"( bindings: [ (keys: "LAlt+LShift", action: SwitchLanguageBackward, block_default: false) ] )"#)`; down LAlt passed; down LShift suppressed, fired `[SwitchLanguageBackward]`; up LShift suppressed; up LAlt passed | GREEN (guard) |
| W13 | `a_group_member_arms_only_if_it_matches_the_press_itself` | `(keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true), (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200))`; down LShift; down CapsLock suppressed, nothing; advance 300 nothing; up CapsLock suppressed, nothing; up LShift | GREEN (kills a group without the `combo_matches` re-check) |
| W14 | `tap_and_hold_on_capslock_still_arm_together_as_one_group` (acceptance 4) | `(keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)), (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)), (keys: "CapsLock", action: ToggleCapsLock)`; down CapsLock suppressed, nothing; +199 up suppressed, fired `[SwitchLanguage]`; +100 down; +201 fired `[PressKey(VK_LSHIFT)]`; up suppressed, nothing; total `[SwitchLanguage, PressKey(VK_LSHIFT)]` | RED: `down CapsLock at +0 ms: expected to fire [], fired [ToggleCapsLock]` |
| W15 | `engine::properties::*` (3 tests) with the new `winners` config, `blocker: false`: `(keys: "Shift+CapsLock", action: ToggleCapsLock)`, `(keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true)`, `(keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200), allow_extra_modifiers: true)`, `(keys: "F13", action: SwitchLanguage, allow_extra_modifiers: true)`, `(keys: "Shift+F13", action: ToggleCapsLock, block_default: false)`, `(keys: "F14", action: PressKey("Left"), allow_extra_modifiers: true)`, `(keys: "Ctrl+F14", action: ToggleCapsLock, trigger: Tap(200), block_default: false)`, `(keys: "F15", action: SwitchLanguage)`, `(keys: "F15", action: ToggleCapsLock, block_default: false, when: Not(AppEquals("game.exe")))` | — | GREEN on both trees: the record keeps every press whole whatever the bindings ask, and `PressKey("Left")` is the only repeating action |
| W16 | `engine::tests::a_when_condition_is_asked_only_for_a_binding_whose_keys_match` (D7) | Env with fixed clock, `os_key_down` false, `foreground_app`/`foreground_title` `panic!("the foreground was asked for although no binding with a when: matches this key")`; `(keys: "F13", action: SwitchLanguage, when: AppEquals("code.exe")), (keys: "F14", action: ToggleCapsLock)`; `on_key(down F14)` → `fire == [ToggleCapsLock]`, `suppress` | RED: panic `the foreground was asked for although no binding with a when: matches this key` (the loop evaluates F13's `when:` first, `mod.rs:382`) |

Red set named in the commit body: W1-W10, W14, W16.

**2. `Engine: one binding or gesture group wins each key-down (1.6, KS-13)`.** Makes commit 1 green.
- Touches: `engine/winner.rs` (new, with unit tests), `engine/mod.rs`, `data/condition.rs`, `data/trigger.rs` (doc), `data/binding.rs` (doc), `engine/scenarios.rs` (one rename, one comment), `README.md`, `default_config.ron`, `CHANGELOG.md`.
- Invariants kept:
  - The key-up rule (`mod.rs:555-581`) and the repeat and record block (`:519-551`) are unchanged.
  - The blocker branch logic (`:394-399`), the sequence and chord feeds, and the sort and publish (`keyboard_hook.rs:110-115`) are unchanged.
  - Nothing in `src/hook` changes.
  - No allocation is added per key-down: `Winner` is a borrow and `GroupArms` is three bools.
- Every existing scenario and property stays green unchanged except the one renamed test. The review list for regressions: `more_specific_immediate_binding_wins` (`:121`), `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (`:1764`), `tap_and_hold_with_equal_terms_fire_exactly_one` (`:440`), `shift_held_by_a_double_tap_still_counts_for_shift_capslock` (`:2051`), `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order` (`:2631`), `sided_user_binding_fires_and_the_blocker_keeps_the_other_sides` (`:2661`), `deferred_binding_on_the_language_hotkey_keys_lets_the_hotkey_through_by_default` (`:3178`), `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (`:3237`), and every template-based test.

**3. `Config: parse_ron_str keeps the loader's warnings next to the bindings (1.6)`.** Refactor (D10); green. `warnings` is always empty. No test changes; the whole loader suite is the proof.

**4. `Config: red-first: duplicate bindings are warned about (1.6)`.** Declared red; compiles on commit 3. Loader tests call `parse_ron_str(text, &|_: &BindAction| None)`.

| # | Test (`config::loader::tests::`) | Config | Expected `warnings` | Predicted on commit 3 |
|---|---|---|---|---|
| L1 | `a_binding_that_duplicates_an_earlier_one_is_warned_about` (acceptance 5) | `(keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200))`, `(keys: "Shift+CapsLock", action: ToggleCapsLock)`, `(keys: "capslock", action: ToggleCapsLock, trigger: Tap(300), block_default: false)`; also `bindings.len() == 3` | `["binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them"]` | RED: ``assertion `left == right` failed`` with `left: []` |
| L2 | `bindings_that_differ_in_kind_keys_trigger_when_or_allow_extra_modifiers_are_not_duplicates` | `CapsLock` Tap(200); `CapsLock` Hold(200); `CapsLock` DoubleTap(250); `CapsLock` Immediate; `CapsLock` with `when: AppEquals("code.exe")`; `CapsLock` with `allow_extra_modifiers: true`; `LShift+CapsLock`; `Shift+CapsLock`; `sequence ["F13","F14"]`; `sequence ["F14","F13"]`; `chord ["F13","F14"]`; `chord ["F13","F15"]` | `[]` | GREEN (guard; pins the narrow rule, Q2) |
| L3 | `duplicates_are_found_across_key_order_terms_timings_and_binding_kinds` | #0 `"Ctrl+Alt+L"` PressKey("L"); #1 `"alt + ctrl + l"` Launch(exe: "wt.exe"); #2 `"F13"` Hold(200) `when: AppEquals("code.exe")`; #3 `"F13"` Hold(500) same `when:`; #4 `sequence ["F14","F15"]`; #5 `sequence ["F14","F15"]`, `max_gap_ms: 900`; #6 `chord ["F16","F17"]`; #7 `chord ["F17","F16"]`, `window_ms: 80`; #8 `"Ctrl+Alt+L"` DoNothing | `binding #1 [alt + ctrl + l] duplicates binding #0 [Ctrl+Alt+L]; remove one of them`, `binding #3 [F13] duplicates binding #2 [F13]; ...`, `binding #5 [Sequence[F14 \u{2192} F15]] duplicates binding #4 [Sequence[F14 \u{2192} F15]]; ...`, `binding #7 [Chord[F17+F16]] duplicates binding #6 [Chord[F16+F17]]; ...`, `binding #8 [Ctrl+Alt+L] duplicates binding #0 [Ctrl+Alt+L]; ...` (only the first earlier one is named) | RED: same assertion, `left: []` |
| L4 | `the_template_and_its_examples_have_no_duplicate_bindings` | `DEFAULT_CONFIG` and `template_examples_ron()` | `[]` for both | GREEN (guard) |

Red set: L1, L3. The expected strings are written with `\u{2192}` so that the source stays ASCII.

**5. `Config: warn about a binding that duplicates an earlier one (1.6)`.** Makes commit 4 green.
- Touches: `data/trigger.rs`, `data/binding.rs` (each with unit tests), `config/loader.rs`, `README.md`, `CHANGELOG.md`.
- Invariant kept: the binding list and its order are unchanged, so every existing loader test stays green.

**6. `Book: one winner per key event in the config reference (1.6)`.** Docs only; green.
- Touches the `book/src/config/*.md` rows of the touch set.
- `cfgcheck` validates the RON; `mdbook build` goes into the lane directory, and the links are checked.

Bisect: commits 2, 3, 5 and 6 are green, and 1 and 4 are red only for their listed tests. README, the template and CHANGELOG change in the commits that change the meaning (2 and 5).

## Acceptance mapping

| Roadmap / brief acceptance line | Covered by |
|---|---|
| Shift+CapsLock (Immediate) with CapsLock (Tap, `allow_extra_modifiers: true`) fires only ToggleCapsLock | W1, W2; M1 |
| A conditional Pause binding wins over an unconditional one in either file order | W3; M2 |
| A winner with `block_default: false` lets the key through even when a loser has `block_default: true` | W4 (more keys), W5 (condition), W6 (deferred winner); M3 |
| Tap(200)+Hold(200) on CapsLock still arm together as one group | W14; existing `tap_and_hold_with_equal_terms_fire_exactly_one` (`scenarios.rs:440`) and `equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one` (`:491`); W7 (conditional group); M5 |
| Duplicates produce a warning in a loader test | L1, L3 (L2, L4 guard the negatives); M4 |
| Exactly one binding wins; precedence keys > `when:` > file order | W1-W10, W14; `more_specific_immediate_binding_wins`; `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (ties go to the earlier one) |
| Only the winner/group decides execution and suppression | W4-W6, W10; `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (OR inside the group) |
| Auto-blockers keep their own rule | W12; `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order`, `sided_user_binding_fires_and_the_blocker_keeps_the_other_sides` |
| Suppression invariant (key-up iff key-down) | `check_press_symmetry` in W1-W14; W15 (properties, strict and focus modes) |
| Reload / `swch off`/`on` / foreground change during a pending gesture | W11 (reload); W5, W7, W8 (`foreground_changed` between presses); the election holds no state, so the existing `remapping_off_drops_a_pending_hold`, `passing_tap_fires_after_a_foreground_change_and_its_key_up_passes` and `reload_during_a_passing_double_tap_keeps_each_press_whole` cover the rest unchanged |
| README and CHANGELOG updated | commits 2 and 5; `readme_matching_examples_load` validates the new block |

## Manual checks (human only)

No `#[ignore]` test is added: every acceptance line is expressible through the engine and the loader. These checks observe Windows' reaction. Use a release build started with `swch open` from a normal console, with Notepad and `osk.exe` open.

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | Default config with the `CapsLock` binding replaced by `( keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true )` and `( keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200), allow_extra_modifiers: true )`; `Shift+CapsLock` kept | Shift+CapsLock quickly; Shift+CapsLock held 1 s; CapsLock tap; CapsLock held 1 s | CapsLock toggles once per Shift+CapsLock, the layout does not change, `osk.exe` shows no extra Shift; the tap switches the layout; the hold sends one Shift press; nothing stuck | none |
| M2 | `( keys: "F9", action: SwitchLanguage )` above `( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") )` | F9 in Notepad, then in the browser address bar; swap the two lines, save, wait for `[INFO] config reloaded`, repeat | Notepad: the A key is typed and the layout is unchanged; browser: the layout switches. The same after the swap | none |
| M3 | `( keys: "Ctrl+Z", action: Media(VolumeMute), block_default: false )` and `( keys: "Z", action: DoNothing, allow_extra_modifiers: true )` | In Notepad type `abc`, press Ctrl+Z; press z; press Shift+Z | Ctrl+Z undoes the typing **and** mutes (volume flyout); z and Shift+Z type nothing | none |
| M4 | Default config plus a copy of its first binding at the end | Save; then restart the daemon | `[WARN] binding #2 [CapsLock] duplicates binding #0 [CapsLock]; remove one of them` before `[INFO] config reloaded`, and again at startup; CapsLock still switches once per press | none |
| M5 | Phase-0 manual check 7 (`roadmap.md:200`): the README Tap/Hold pair replacing the default `CapsLock` | Tap CapsLock 20 times quickly; hold it 2 s and release | 20 switches; one Shift press; nothing stuck in `osk.exe` | none |

## Risks and edge cases

**R1, hook callback and engine.**
- The adapter is unchanged; every path still ends in exactly one of `LRESULT(1)` or `CallNextHookEx`.
- The engine walks the list twice per key-down (election, then the act loop).
  - Neither walk allocates or locks; neither takes a new lock, nor does any I/O.
  - The second walk does real work for a `keys` binding only for the winner, or, with a deferred winner, for deferred candidates with equal keys (`KeyCombination ==` and `Condition ==`, no allocation).
- D7 removes foreground lookups for conditional bindings whose keys do not match. No index outside `keys.rs` tables.

**R2, sentinel.** No new `SendInput`. Fewer actions fire (losers no longer do), so no new feedback path.

**R3, shared state; R4, atomics.** Nothing new. The engine state stays plain data under `Mutex<Engine>`, reached only through `with_engine`. `Winner` and `GroupArms` live on the stack of one call. No `static`, no atomic, no `Cell`.

**R5, builders.** No new `Binding` field and no setter. The new predicates (`Trigger::same_kind`, `BindingKind::same_keys`, `Binding::duplicates`) are query methods. The constructors are unchanged.

**R6, Display.** No new enum variant. `Binding` Display is unchanged. The warning text is built in the loader, not through `Display`.

**R7, specificity order.**
- The stable sort and its single publish in `update_bindings` are untouched.
- The matching loop's precedence is replaced explicitly by `winner::elect`'s rank (D2). The sort still gives file order among equal key counts, and the blockers stay last for the sequence/chord/blocker visits (KS-18).
- Mutation risks the tests kill:
  - a blocker counted as a candidate: `user_binding_on_alt_shift_fires_with_the_blocker_in_either_order`;
  - `>=` instead of `>` (later wins ties): `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat`;
  - dropping the condition rank: W3;
  - dropping the group: W14, `tap_and_hold_with_equal_terms_fire_exactly_one`;
  - a group without the keys check: W10;
  - a group without the `when:` check: W8;
  - a group without the `combo_matches` re-check: W13;
  - dropping `GroupArms`: W9;
  - OR over losers: W4-W6.

**R8, key identity.** The election reuses `combo_matches` (KeySpec; sided and unsided by family, `matching.rs:28-46`), so combo, group and duplicate identity all use `KeySpec` equality.
- `LShift+F13` and `Shift+F13` are different keys: they are not one group and not duplicates. At equal key count file order decides between them (Q6).
- Chords and sequences are not in the election (D1).
- The stale R8 text in `.claude/agents/code-reviewer.md:126` (`active_keys` folding) is follow-up F6 from 1.3, unchanged.

**R9, registry and process reads.** Tests never read the registry for blockers: `with_alt_shift_blocker`, and `parse_ron_str` with `&|_| None`. The foreground reads (`condition.rs`) are only reduced. Nothing writes.

**R10, message loop and shutdown.** Nothing new. The warning prints during `config::load`, before the hooks are installed (`main.rs:30`), or on `config-watcher`.

**Repeats.** The winner is re-elected per repeat (D6). If Ctrl is released while `Ctrl+F22` (Tap, passing) is held, later repeats of F22 elect `F22` (`PressKey("Left")`, repeats) and inject Left per repeat while F22's own repeats pass. That is today's behaviour, too (the Immediate fired on those repeats already). Recorded as Q3.

**Gesture tables keyed by vk.** Across presses, entries of different bindings on the same key still interact: a DoubleTap armed by an exact `CapsLock` binding is consumed by the next press won by a `CapsLock` DoubleTap with `allow_extra_modifiers` (`gestures.rs:207-212`). This is unchanged and out of scope; the 2.2 state machine owns it.

**Reload mid-gesture.** Pending entries keep their actions, and the key-up does not consult the bindings (W11).

**`swch off`/`on`, foreground change.** The election has no state; `reset` and `on_foreground_changed` are unchanged.

**UIPI, elevation, IME and layouts.**
- No change for UIPI or elevation.
- AltGr: see the Win32 section.
- A layout switch fired by a winner reaches Windows as before.

**Sided modifiers and KS-14.** The resync is unchanged (`keys.rs:97-105`). The election reads the same `KeyState`.

**Duplicate warning noise.** Scenario configs that are duplicates on purpose (`non_repeating_winner_keeps_the_immediate_slot_on_a_repeat`, W9) print a `[WARN]` line in the test output. This is harmless, and cargo captures it.

**Follow-ups (not this lane).**
- F1: `sequence`/`chord` in the election (3.2/3.4; KS-15, KS-16).
- F2: warning for `Immediate` vs deferred on the same keys and `when:` (3.4, Q2).
- F3: semantic condition equality (Q4).
- F4: per-press winner lock (Q3).
- F5: the blocker-replacement warning (Q5; KS-18/1.2).
- F6: stale reviewer R8 text (from 1.3).

## Questions for the orchestrator

1. **Should `sequence` and `chord` bindings join the election?** Recommendation: not in 1.6. They are state machines fed on every key-down, the spec's candidate rules (trigger key, exact modifiers) do not exist for them, and their matching is 3.2's (KS-15, KS-16) and 3.4's ("RC1 by construction"). A key that is both a `keys` trigger and a step still drives both; this is documented in §16.2. Non-blocking.
2. **Should the loader also warn when an `Immediate` binding and a `Tap`/`Hold`/`DoubleTap` on the same keys and `when:` shadow each other?** After this lane the later one never runs (consequence 4 of the Breaking entry; the README and template CapsLock examples hit it). Recommendation: keep the brief's narrow rule (exact duplicates), document the case (CHANGELOG, README, the template comment, known-issues row 69), and let 3.4's diagnostics (`roadmap.md:451`) add it. If wanted now, it is one more disjunct in `Binding::duplicates` (trigger kinds differ and exactly one is `Immediate`), one more message, and L2's `CapsLock` Immediate row flips. Non-blocking.
3. **Should one winner be locked for a whole press, auto-repeats included?** Recommendation: no. The spec says "per key event", 1.1's repeat semantics and `reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat` re-evaluate per event, and repeat suppression already follows the key-down's record. Revisit in 3.4 if the owner reports surprise. Non-blocking.
4. **Should `when:` equality ignore case, so that `AppEquals("Code.exe")` and `AppEquals("code.exe")` form one group and count as duplicates?** Recommendation: no. Structural equality is predictable and allocation-free on the hook thread. "The same `when:`, written the same way" is documented (README, known-issues row 72), and a mismatch is visible (only the winner arms). Non-blocking.
5. **The 1.3 cut's Q2 handed 1.6 a warning for a binding that does not block replacing the Windows-hotkey auto-blocker.** Recommendation: not in this lane. It is not in the brief's spec, it changes `[INFO] no auto-blocker ...` (`loader.rs:303`), which manual check 10 (`roadmap.md:203`) quotes, and item 1.2 (blocker v2) reworks the blocker. Hand it to 1.2's cut. Non-blocking.
6. **Should a sided key count as more specific than its unsided family at equal key count (`LShift+F13` over `Shift+F13`)?** Recommendation: no. The owner-approved precedence (keys, `when:`, file order) has no such rule, file order already lets the user choose, and it is documented as known-issues row 71. Ask the owner only if users report it. Non-blocking.

Optional research (non-blocking, not needed for the design): none.

## Readiness checklist

**Spec**
- [x] KS-13 read (`audit.md:390-403`). Its three scenarios, as they reproduce under exact modifiers, are W1/W2, W4 and W6 plus M1 and M3. The literal first scenario is pinned as not reproducing (`scenarios.rs:1139`).
- [x] Every book and audit reference re-located (table). `README.md:493` @7cb3521 is gone, and its successors `:391-393` and `:697` are rewritten.
- [x] Goal, non-goals and dependants stated (3.4, 1.7).

**Design**
- [x] D1-D11 each have a rationale and the alternatives they reject.
- [x] Owner decisions respected:
  - exact modifiers: the election runs on `combo_matches`, with `allow_extra_modifiers` per binding;
  - repeat defaults: an Immediate winner fires on a repeat only with `repeat`, and deferred groups skip repeats;
  - RON: no schema change;
  - logging: `eprintln!` until 1.7, and the warnings go through one seam that 1.7 swaps;
  - licence: untouched.
- [x] A thread is named for every changed path: engine on the main thread inside `with_engine`; warnings on main or `config-watcher`. Nothing heavy is added to the LL callback, and D7 removes Win32 lookups.
- [x] Install, uninstall and timers are unchanged; Hold timers stay as they are until 2.2.
- [x] No new `SendInput` path.
- [x] Sided and unsided modifiers go through `KeySpec` in the election, the group and the duplicates; chords and sequences are out of the election (D1).
- [x] Reload, `swch off`/`on`, foreground change and shutdown are covered (W11, W5/W7/W8, R10; the election holds no state).
- [x] No new shared state, atomics or `static mut`.
- [x] No new module cycle (`engine::winner` → `data`, `engine::{matching, keys}`; `config` → `data`). The logic lives in `engine` and is reached by the scenarios.

**Win32**
- [x] No new API. The relied-on calls are listed with `windows` paths, features (present), threads and quoted Microsoft Learn constraints.
- [x] Elevation, UIPI, pipe DACL and token inheritance are unaffected.

**Commits and tests**
- [x] Red-first with exact predicted failures (commits 1 and 4).
- [x] Tests go through `Engine::on_key`/`on_timer` via `Scenario`, a fake `Env` and a fake clock, or through `parse_ron_str`. Nothing installs a hook or injects input.
- [x] No ignored test needed; manual checks M1-M5 are listed.
- [x] Every commit passes the gate except the declared red commits 1 and 4.
- [x] The order bisects. README, template and CHANGELOG are in commits 2 and 5; the book is in 6.

**Acceptance**
- [x] Every acceptance line is mapped.
- [x] Every question has a recommendation; none blocks.

## Developer addendum (implementation round 1)

Critique round 1 had no Critical or Important remark. The three Minor remarks are resolved, none refuted:

- **M1 (Repeats risk sentence).** The Risks paragraph "Repeats" is wrong as the critic says. Corrected text: "The winner is re-elected per repeat (D6). If Ctrl is released while `Ctrl+F22` (Tap, passing) is held, later repeats of F22 elect `F22` (`PressKey("Left")`, repeats), which injects Left per repeat, and the repeats themselves pass, because the key-down passed. On master the losing `F22` binding swallowed the key-down, so every repeat was swallowed too; after the lane the application gets the key's own repeats and Left interleaved. Nothing sticks (the record keeps the press whole). Recorded as Q3." Pinned by the new scenario `engine::scenarios::repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down` (commit 2, `006ff5e`), and documented in the CHANGELOG Breaking entry (last consequence), README behaviour rules (`block_default` bullet) and `known-issues.md` row 73, `triggers-conditions.md` 11.3.
- **M2 (stale comments in `src/hook/keyboard_hook.rs`).** Both comments rewritten in commit 2: the `update_bindings` doc (`keyboard_hook.rs:106-109`) and the comment of `equal_specificity_keeps_config_order` (`:312-315`) now say the stable sort gives the file-order tie-break between `keys` bindings of equal precedence (roadmap item 1.6). Doc-only; no code in `src/hook` changed.
- **M3 (duplicate warning and `block_original_combo`).** Settled in the docs, the message and L1-L4 unchanged: the README Errors paragraph (commit 5), the CHANGELOG Added entry (commit 5) and the `errors.md` 18.6 row (commit 6) say that a `block_original_combo: true` on the copy that is removed must move to the one that is kept, or its auto-blocker goes. Excluding such pairs was rejected: the later copy never acts as a binding, and hiding it would hide a dead binding.
