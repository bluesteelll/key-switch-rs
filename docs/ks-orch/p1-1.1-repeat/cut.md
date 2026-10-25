# Cut: p1-1.1-repeat — roadmap item 1.1 "Per-binding `repeat` flag" / KS-10

Tree: `D:/key-switch-rs/key-switch-rs`, `master` = `dbd55ca` (read from `.git/refs/heads/master`). Branch `feat/repeat-flag` does not exist yet; the developer cuts it from `master`.

## Verdict

CUT READY

Two non-blocking questions are listed at the end, each with a recommendation.

## Spec re-located

| Spec reference (book @7cb3521 unless noted) | Current file:line @dbd55ca | Status | Note |
|---|---|---|---|
| roadmap 1.1 *Why*: the Immediate branch fires on every repeat, `src/hook/keyboard_hook_callback.rs:229-237` | `src/engine/mod.rs:403-412` (Immediate arm of `on_key_down`) | moved (16a0461 refactor/engine) | **Defect present.** The arm has no `was_repeat` check: `decision.fire.push(binding.action.clone())` runs on every matching key-down (`:405-408`) |
| KS-10: `was_repeat` from `active_keys[vk].swap(true)`, `keyboard_hook_callback.rs:191-195` | `src/engine/keys.rs:141-148` (`KeyState::press`), called at `src/engine/mod.rs:367` | moved | The LL hook has no repeat bit, so the engine's held table remains the only repeat detector (see Win32) |
| KS-10: the "debatable" comment, `keyboard_hook_callback.rs:186-190` | — | gone | The adapter no longer decides anything. The current rule is documented at `src/engine/mod.rs:120-122` |
| KS-10: injected-event check, `keyboard_hook_callback.rs:136-138` | `src/hook/keyboard_hook_callback.rs:166-168` (`decide_event`) | moved | Unchanged. Physical repeats carry no sentinel and reach the engine |
| KS-10: `switch_language_forward` posts on every call, `src/system/system_function.rs:45-63` | `src/system/system_function.rs:42-60` | moved | Unchanged. Nothing downstream debounces |
| roadmap 1.1 / KS-10: one thread per `spawn_sequence` call, `src/data/sequence.rs:119` | `src/data/sequence.rs:111-116` (`thread::spawn` at `:116`) | moved | Unchanged. The fix stops the call that each repeat used to trigger |
| roadmap 1.1 *Files*: `src/config/schema.rs` | `src/config/schema.rs:30-68` (`RawBinding`; `deny_unknown_fields` at `:31`; last field `allow_extra_modifiers` at `:62-67`) | moved | |
| roadmap 1.1 *Files*: `src/config/loader.rs` | `src/config/loader.rs:341-382` (`raw_binding_to_binding`); deferred-on-seq/chord check `:353-362`; `allow_extra_modifiers` check `:364-372`; builder chain `:374-379`; `IMPLICIT_SOME` `:238-243` | moved | |
| roadmap 1.1 *Files*: `src/data/binding.rs` | `src/data/binding.rs:49-76` (struct), `:85-96` `with_kind`, `:98-109` `new_auto_blocker`, `:121-144` builders, `:151-185` `Display` | moved | These are the only two struct literals of `Binding` in `src` (grep `Binding {`) |
| roadmap 1.1 *Files*: `src/engine/` | `src/engine/mod.rs:365-536` (`on_key_down`) | new since 7cb3521 | |
| A3 invariant: a repeat follows its key-down; its key-up is suppressed exactly once | `src/engine/mod.rs:502-534`, `:559-562` | unchanged since 44f79ac | This lane does not touch it |
| "Deferred triggers, sequences and chords already ignore repeats" | `src/engine/mod.rs:414-416`, `:426-428`, `:440-442`, `:456-458`, `:481-483` | unchanged | Pinned by new tests (commit 3) |
| Engine module docs that promise the KS-10 fix | `src/engine/mod.rs:120-122`, `:141-142`, `:156-159` | — | Rewritten in commit 4 |
| Scenarios that pin the old KS-10 behaviour | `src/engine/scenarios.rs:1306-1331` (comment `:1311`), `:1511-1527` (`expect_fired(&[SwitchLanguage])` at `:1526`) | — | Updated in commit 3 |
| Action carry-out | `src/hook/keyboard_hook_callback.rs:187-195` (`carry_out`) | unchanged | Runs `Decision::fire` as it does today; no change |
| Book @1dfd80a `src/engine/mod.rs:404-412` (known-issues row 8, kinds §7, triggers §11.3, actions §13.1) | `src/engine/mod.rs:404-412` | unchanged | |
| Owner decision on defaults | `book/src/roadmap.md:665` | — | `true` only for `PressKey`, `Media(VolumeUp)`, `Media(VolumeDown)` |
| CHANGELOG KS-06 entry: "such a repeat still fires an `Immediate` action (KS-10, roadmap item 1.1)" | `CHANGELOG.md:200-203` | — | Becomes false; edited in commit 4 |
| Reload path | `src/hook/config_watcher.rs:162` → `KeyboardHook::update_bindings` `src/hook/keyboard_hook.rs:110-115` (sort + one `ArcSwap` store, no engine reset) | unchanged | The flag travels inside the snapshot |

## Goal and non-goals

**Goal.** A new optional per-binding RON field `repeat: true | false`.

- It decides whether an `Immediate` `keys` binding fires again on each auto-repeat of its key.
- Its default is resolved at load time from the action (owner decision): `true` for `PressKey(_)`, `Media(VolumeUp)` and `Media(VolumeDown)`, and `false` for everything else.
- An auto-repeat of a non-repeating binding fires nothing. Its suppression still follows the key-down (`src/engine/mod.rs:502-534`, untouched).
- `Tap`/`Hold`/`DoubleTap`, `sequence` and `chord` keep ignoring repeats.
  - `repeat: true` on them is a load error.
  - `repeat: false` on them is accepted, because it states their behaviour.
- The startup listing marks a binding whose `repeat` differs from its action's default.

**Non-goals (later items / follow-ups).**

- **KS-29 / item 2.3.** A press after a lost key-up still reads as a repeat. See Q1.
- **KS-13 / item 1.6.** One winner per event. Matching Immediate bindings that lose still request suppression; that is unchanged.
- **Item 1.3.** `block_default` for deferred triggers.
- **KS-08 / item 2.1.** `Launch`/`OpenUrl` still run on the hook thread, now once per press.
- **Item 3.5.** The `when:` evaluation cost on repeats.
- **Not planned.** No custom repeat rate or limiter, and no "repeat while held" for `Hold`. Neither has a roadmap item.

**Later items that depend on this one.**

- **1.6.** The winner on an auto-repeat fires according to its `repeat`. D3's slot rule is already that rule.
- **2.1.** The executor gets one job per press instead of about 30 per second.
- **3.4.** It can reuse `Scenario::check_repeats_fire_only`.

## Decisions

### D1: `repeat` is resolved to a plain `bool` on `Binding` at load time; the raw field is `Option<bool>`

- **What.**
  - `RawBinding.repeat: Option<bool>` with `#[serde(default)]`, placed last after `allow_extra_modifiers` (`src/config/schema.rs:67`).
  - `Binding.repeat: bool` (pub, doc-commented).
  - `Binding::with_kind(kind, action)` sets `repeat: action.repeats_by_default()`. `Binding::new_auto_blocker` sets `repeat: false`.
  - The new consuming builder `Binding::with_repeat(self, repeat: bool) -> Self` applies the explicit config value: `raw.repeat.unwrap_or(action default)`, implemented as "apply `with_repeat` only when `Some`".
- **Why.**
  - The default depends on the action, and a serde `default = fn` cannot see a sibling field. `Option` is the only way the loader can tell "unset" from `false`.
  - `IMPLICIT_SOME` (`src/config/loader.rs:242-243`) lets users write `repeat: true` without `Some(..)`.
  - Resolving at load keeps the engine's hot path to one `bool` read. Bindings built in code (tests, `app.rs:224`) also get the owner's default for free.
  - The value lives in the immutable `Vec<Binding>` snapshot, so no new shared state is needed.
- **Rejected.**
  - (a) `Binding.repeat: Option<bool>` resolved in the engine on every event. That puts per-event work on the hot path, and `Display` and tests would each have to re-derive the value.
  - (b) A plain `bool` with a serde default of `false`. That breaks the owner's default for `PressKey` and the volume keys.
  - (c) Computing the default in the engine from `binding.action`. That spreads the policy over two layers, and nothing could override it through the builder.
- **Cost.** One field, one builder. The constructors keep their two positional arguments (R5).

### D2: The default rule lives on the action, `BindAction::repeats_by_default(&self) -> bool`, as an exhaustive match

- **What.** The method lives in `src/core/windows_actions.rs`, in `impl BindAction` (`:117-159`).
  - It returns `true` for `PressKey(_)`, `Media(MediaKey::VolumeUp)` and `Media(MediaKey::VolumeDown)`, and `false` for every other variant, including the other five `MediaKey`s.
  - The match has an arm per `BindAction` variant and a nested arm per `MediaKey`, with no `_` arm.
- **Why.** Which actions are idempotent per repeat is a property of the action. The exhaustive match forces a decision when a variant is added later (R6 spirit). `data::binding` already imports `BindAction` (`src/data/binding.rs:7`), so no new module edge is added.
- **Rejected.**
  - (a) `matches!(..)`. A new variant would default silently.
  - (b) A free function in the loader. `Binding::with_kind` could not use it without a `data → config` edge, which would be a new cycle.
- **Cost.** About 15 lines.

### D3: Engine rule. The first matching Immediate binding takes the slot on every key-down; on a repeat it fires only if `repeat`

- **What.** In `Engine::on_key_down`'s Immediate arm (`src/engine/mod.rs:404-412`):
  - When `immediate_fired` (`:371`) is false, set it to true.
  - Push `binding.action.clone()` only when `!was_repeat || binding.repeat`. The clone happens only on that path.
  - The `block_default` handling (`:409-411`) and the whole `was_repeat` suppression block (`:502-534`) stay unchanged. Nothing else in `on_key_down`/`on_key_up` changes.
- **Why.**
  - (1) **The slot rule.** A non-repeating winner must still take the repeat. Otherwise a less specific or later binding on the same press starts firing on repeats. With `( keys: "F13", action: SwitchLanguage )` and `( keys: "F13", action: PressKey("Left") )`, a "skip" implementation would type Left at 30 Hz after one layout switch. This also matches item 1.6's "only the winner decides execution".
  - (2) **Suppression.** It is untouched, so the A3 invariant holds by construction. CapsLock's repeats are suppressed because its key-down is `Suppressed` (`:520-521`), not because the binding asked again.
  - (3) **Hot path.** One `bool` read. It also removes a `BindAction` clone per repeat (a `Launch` clone allocates two `String`s and a `Vec`) and the first `Vec` push under the engine lock. That is strictly less work in the LL callback (KS-08 budget).
  - (4) **Placement.** It lives in the engine, so scenario tests reach it.
- **Rejected.**
  - (a) Skip non-repeating bindings with `continue`. Fall-through, see (1).
  - (b) Filter repeats in the adapter (`carry_out`). The adapter does not know which binding produced an action, and the rule would leave the engine, where tests cannot reach it.
  - (c) Treat a key-down after a long gap as a fresh press (a typematic-delay heuristic). This is KS-29/2.3 scope. Microsoft also documents that the delay "may vary depending on the hardware" (see Win32), so a fixed threshold misclassifies.
- **Cost.** About 4 changed lines plus doc text.

### D4: Loader validation. `repeat: true` is an error outside `keys` + `Immediate`; `repeat: false` is accepted everywhere

- **What.**
  - A new check in `raw_binding_to_binding` after the `allow_extra_modifiers` check (`src/config/loader.rs:372`).
  - Condition: `raw.repeat == Some(true)` and (the kind is not `Combo`, or the trigger is not `Immediate`). Then return `Err(format!("{}: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger", err_prefix))`.
  - Exact message after the prefix: `` `repeat: true` applies only to `keys` bindings with the `Immediate` trigger ``.
  - Example: `binding #0 [F13]: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger`.
- **Why.**
  - The precedent is `allow_extra_modifiers` (`:364-372`): "an accepted but ignored flag would change meaning silently". A user who writes `repeat: true` on a `Hold` expects behaviour the engine never gives.
  - `repeat: false` there is true by construction. Rejecting it would only punish users who set the field on every binding.
  - Ordering after the existing checks keeps "first error per binding" (`book/src/config/errors.md:136`) deterministic.
- **Rejected.**
  - (a) Warn and accept. The house rule since KS-22 is to fail loudly on silent no-ops.
  - (b) Reject any explicit value on non-Immediate bindings. Hostile, and nothing is gained.
  - (c) Implement repeat for deferred triggers. Not in the spec, and the spec says to keep them ignoring repeats.
- **Cost.** One `if`, and one row in errors.md.

### D5: The startup listing marks only overrides: `  [repeat]` / `  [no repeat]`

- **What.**
  - In `impl Display for Binding` (`src/data/binding.rs:151-185`), a new suffix goes after `extra_suffix`.
    - `"  [repeat]"` when the binding is a user `Combo` with `Trigger::Immediate`, `repeat == true` and `!action.repeats_by_default()`.
    - `"  [no repeat]"` when it is a user `Combo` with `Trigger::Immediate`, `repeat == false` and `action.repeats_by_default()`.
    - `""` otherwise.
  - The auto-blocker branch (`:171-176`) is unchanged.
- **Why.** The banner already shows `[allow_extra_modifiers]`. Showing only overrides keeps every default config's banner byte-identical, while making a user's opt-in or opt-out visible. Deferred bindings never show it, because the flag is meaningless there.
- **Rejected.** Always printing the value. That adds noise to every line and changes the default banner.
- **Cost.** One `match`.

### D6: The default config and docs say it. No other surface changes

- The `swch`/IPC protocol, the hook adapter, the threads and `Cargo.toml` are unchanged.
- README, template, CHANGELOG and the `book/src/config` pages change. See the Touch set and the docs impact.

### D7: Test-harness additions to reach the checklist items "reload during a pending gesture" and "deferred/sequence/chord ignore repeats" from tests

- `Scenario::check_repeats_fire_only(self, allowed: &[BindAction]) -> Self`.
  - From then on, every `repeat` step may fire only actions contained in `allowed`. Otherwise it fails with `"{label}: fired {fired:?} on an auto-repeat; only {allowed:?} may fire on one"`.
  - It is stored as `Option<Vec<BindAction>>` and checked in `key_event` (`src/engine/testing.rs:445-469`) when `what == "repeat"`.
- `Scenario::reload(self, text: &str) -> Self`.
  - It models `KeyboardHook::update_bindings`: `from_ron_str(text)` (panics like `from_ron` on error), then `sort_by_specificity`, then it replaces `self.bindings`.
  - It does **not** reset the engine, matching `config_watcher.rs:162`.
  - It records a step labelled `"reload at {clock}"`.
- Both are test-only (`#[cfg(test)] mod testing`) and add no product code.
- Rejected: asserting fires per repeat inside `properties.rs` itself. The harness is where the other two whole-script checks live (`check_press_symmetry`, `check_nothing_stuck`).

## Win32 API choices

No new Win32 API, no new `windows` feature, no new thread, window, timer or COM apartment. The current features are at `Cargo.toml:41-50` and stay unchanged. The table lists the documented facts the design relies on.

| Purpose | `windows` path | Feature flag (present?) | Thread | Errors | Doc link / constraint relied on |
|---|---|---|---|---|---|
| Repeat detection stays in the engine's held table: the LL hook gives no repeat bit | `windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT` (already read at `src/hook/keyboard_hook_callback.rs:256`) | `Win32_UI_WindowsAndMessaging` (present, `Cargo.toml:44`) | main (hook) | n/a | <https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-kbdllhookstruct>: `flags` holds "The extended-key flag, event-injected flags, context code, and transition-state flag". There is no previous-state or repeat bit |
| What an auto-repeat is | (window message; not used by the LL hook) | — | — | — | <https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-keydown>: "Because of the autorepeat feature, more than one WM_KEYDOWN message may be posted before a WM_KEYUP message is posted." Bit 30 exists only in a window message's `lParam`, not in `KBDLLHOOKSTRUCT` |
| Hook budget: why the change must not add work | `LowLevelKeyboardProc` (the existing callback `keyboard_hook_callback`) | — | main | timeout → silent unhook | <https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc>: "If the hook procedure times out, the system passes the message to the next hook. However, on Windows 7 and later, the hook is silently removed without being called." The maximum is 1000 ms on 1709 and later. D3 removes one clone and one allocation per repeat; it adds none |
| Rejected heuristic (D3 c): typematic delay | `SystemParametersInfoW(SPI_GETKEYBOARDDELAY)`, **not used** | — | — | — | <https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow>: "0 (approximately 250 ms delay) through 3 (approximately 1 second delay). The actual delay associated with each value may vary depending on the hardware." |

Injection notes:
- `LLKHF_INJECTED`, the extended flag and `scanCode` do not matter for this lane. Repeat identity is the raw `vkCode` (`src/engine/keys.rs:143-148`).
- The daemon's own injected events are dropped before the engine by the sentinel (`keyboard_hook_callback.rs:166-168`). A `PressKey` repeat stream therefore never feeds back as a "repeat".
- Another process's injected key-down of a held key (AHK) reads as a repeat. That is the same as today.

## Threads

| Thread | What it does with `repeat` | Change |
|---|---|---|
| main (installs both hooks, pumps `GetMessageW`) | Reads `binding.repeat` inside `Engine::on_key` under `with_engine`, from the snapshot `decide_event` loaded (`keyboard_hook_callback.rs:172`), and carries out `fire` after the lock | No new work. One fewer clone per non-repeating repeat |
| `config-watcher` | Builds `Vec<Binding>` with the resolved `repeat` and publishes it through `update_bindings` (one `ArcSwap` store) | No new code path |
| `hold-timer`, IPC workers, macro workers | Never read `repeat` | None |

Atomics: none new. The flag is immutable data inside the `Arc<Vec<Binding>>`, published by arc-swap's own store/load pairing like every other field. No `static mut`, no new lock.

## Touch set

| File | Change | Why |
|---|---|---|
| `src/config/schema.rs` | `RawBinding`: add `#[serde(default)] pub repeat: Option<bool>` after `allow_extra_modifiers` (`:67`), with a doc comment: missing means the action's default; `Immediate` `keys` only; `true` elsewhere is an error | Spec: field on `RawBinding`; `deny_unknown_fields` (`:31`) stays |
| `src/core/windows_actions.rs` | `BindAction::repeats_by_default(&self) -> bool` (D2), exhaustive. New `#[cfg(test)] mod tests` at the bottom | D2 |
| `src/data/binding.rs` | `pub repeat: bool` with doc; `with_kind` sets the default; `new_auto_blocker` sets `false`; `with_repeat`; `Display` suffix (D5); unit test | D1, D5 |
| `src/config/loader.rs` | In `raw_binding_to_binding`: D4 check after `:372`; `.with_repeat(v)` when `raw.repeat` is `Some(v)` in the chain at `:374-379`. Tests (commits 1, 2) | D1, D4 |
| `src/engine/mod.rs` | Immediate arm `:404-412` (D3); module docs `:120-122`, `:141-142`, `:156-159` | D3 |
| `src/engine/scenarios.rs` | Helper `hold_with_repeats`, `const TEMPLATE`, new scenarios, two updated scenarios, harness self-test | Acceptance and KS-10 repro |
| `src/engine/testing.rs` | `check_repeats_fire_only`, `reload`; module doc "Two checks" (`:30-34`) becomes three, plus a reload paragraph | D7 |
| `src/engine/properties.rs` | `run`/`run_all` gain `repeat_check: bool`; new property test; module doc lists the third property | Pin "deferred/seq/chord ignore repeats" over random scripts |
| `README.md` | See the docs impact: `:94-105`, `:348-353`, `:378-402`, `:453-460`, `:505-527`, `:651-664` | Config meaning changed (CLAUDE.md rule) |
| `src/config/default_config.ron` | Trigger section `:104`, per-binding flags `:140-161`, Media examples `:212-217` | Same |
| `CHANGELOG.md` | Unreleased: Added, Changed (**Breaking:**), Fixed KS-10; edit `:200-203` | Keep a Changelog |
| `book/src/config/structure.md`, `book/src/config/README.md`, `kinds.md`, `triggers-conditions.md`, `actions.md`, `known-issues.md`, `errors.md`, `matching.md`, `example.md` | See the docs impact | The config reference must not publish false facts on push to `master` |
| **Not touched** | `src/hook/*`, `src/core/app.rs`, `src/ipc.rs`, `Cargo.toml`, `Cargo.lock`, `build.rs` | The adapter already carries out exactly `Decision::fire` |
| **Post-merge book sync (doc-writer, not this lane)** | `book/src/audit.md:33` and the KS-10 status `:349`; `roadmap.md:614` and a 1.1 status block; `architecture.md:207` (+ its other two KS-10 mentions); `glossary.md:239`; `overview.md:17,20,210,232`; `gaps.md` KS-10 mentions | These cite the merge sha, which does not exist until the merge (`docs/WORKFLOW.md:105-107`) |

## Commits, in order

Every commit message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.

Gate per commit: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. Commits 1 and 3 are the declared red commits: fmt and clippy pass, and `cargo test` fails exactly the listed tests. Every red test compiles on its parent, because it uses only RON and the existing API.

### 1. `Config: red tests for the per-binding repeat field (1.1)` — RED

Files: `src/config/loader.rs` (tests module) and `src/engine/scenarios.rs` (helper + one scenario).

Helper, added here in `scenarios.rs` and private:

```rust
/// Presses `key`, holds it past the 500 ms repeat delay for `repeats`
/// auto-repeats 33 ms apart, and releases it. Every event is expected
/// suppressed, every repeat to fire exactly `on_repeat`, the release nothing.
#[track_caller]
fn hold_with_repeats(scenario: Scenario, key: &str, repeats: usize, on_repeat: &[BindAction]) -> Scenario
```

Its steps:
- `down(key)`, then `expect_suppressed`;
- `advance_ms(500)`;
- `repeats` times: `repeat(key)`, `expect_suppressed`, `expect_fired(on_repeat)`, `advance_ms(33)`;
- `up(key)`, `expect_suppressed`, `expect_nothing_fired`.

It does not check the key-down's fire, because `Sequence` actions compare by `Arc` pointer. Callers check the key-down or `expect_total_fired`.

| Test | Scenario | PREDICTED RED on `master` |
|---|---|---|
| `config::loader::tests::repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings` | Rows, each `action: PressKey("A"), repeat: true`, each the only binding: `keys: "F13"` with `Tap(200)`, `Hold(200)`, `DoubleTap(250)`; `sequence: (steps: ["F13", "F14"])`; `chord: (keys: ["F13", "F14"])`. Assert `ConfigError::Bindings(errs)` with `errs == [format!("binding #0 [{label}]: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger")]`, label `F13` / `Sequence[F13 → F14]` / `Chord[F13+F14]` | panics `expected Bindings error for Tap(200), got Parse { … }`. The source is ``Unexpected field named `repeat` in `RawBinding` …`` |
| `config::loader::tests::repeat_false_loads_on_every_kind_and_trigger` | The same six shapes (plus a plain Immediate `keys: "F13"`) with `repeat: false` each load to one binding | panics `Immediate: repeat: false must load: config parse error (<in-memory>): …Unexpected field named `repeat` in `RawBinding`…` |
| `config::loader::tests::an_unknown_binding_field_lists_repeat_among_the_valid_names` | `( keys: "CapsLock", action: SwitchLanguage, tigger: Tap(200) )` gives `ConfigError::Parse`, and `err.to_string()` contains ``"`trigger`, `allow_extra_modifiers`, or `repeat` instead"``. This pins the README/book quote. The format follows ron 0.10's "one of …, or `x` instead" list, which `README.md:515` already shows | assertion fails; the message ends ``…`trigger`, or `allow_extra_modifiers` instead`` |
| `config::loader::tests::display_of_a_loaded_binding_marks_a_repeat_flag_that_differs_from_its_action_default` | One file, in file order (no sort): `(keys: "F13", action: SwitchLanguage, repeat: true)` → `to_string()` ends with `"  [repeat]"`; `(keys: "F14", action: PressKey("Left"), repeat: false)` → ends with `"  [no repeat]"`; `(keys: "F15", action: PressKey("Left"))`, `(keys: "F16", action: SwitchLanguage, repeat: false)` and `(keys: "F17", action: PressKey("Left"), trigger: Tap(200), repeat: false)` → `!contains("repeat]")`. Use `from_ron_str_with_hotkeys(.., &|_: &BindAction| None)`. The test lives in loader tests, so `data` gets no test edge to `config` | panics at the load: ``Parse … Unexpected field named `repeat` …`` |
| `engine::scenarios::repeat_true_on_switch_language_fires_on_every_repeat` | `( bindings: [ (keys: "CapsLock", action: SwitchLanguage, repeat: true) ] )`, `.check_press_symmetry().check_nothing_stuck()`, `hold_with_repeats(_, "CapsLock", 10, &[SwitchLanguage])`, `expect_total_fired(&vec![SwitchLanguage; 11])` | panics `scenario: config does not load: config parse error (<in-memory>): …Unexpected field named `repeat` in `RawBinding`…` |

### 2. `Config: per-binding repeat field with per-action defaults (1.1)` — makes commit 1 green

Files: `src/config/schema.rs`, `src/core/windows_actions.rs`, `src/data/binding.rs`, `src/config/loader.rs`.

Content: D1, D2, D4, D5. **The engine is unchanged in this commit.** The commit message says so: the flag is accepted, and the engine still fires every repeat until the next commit. All five commit-1 tests turn green; the last one passes because the engine still fires every repeat.

New API unit tests. They are green on arrival and cannot precede the API without a compile error:
- `core::windows_actions::tests::only_press_key_volume_up_and_volume_down_repeat_by_default`: every `BindAction` variant and all 7 `MediaKey`s.
- `data::binding::tests::a_binding_takes_its_repeat_default_from_its_action`:
  - `Binding::new(F13, PressKey(VK_LEFT)).repeat` is true, and `.with_repeat(false)` gives false;
  - with `SwitchLanguage` it is false, and `.with_repeat(true)` gives true;
  - `new_auto_blocker(..)` gives false.
- `config::loader::tests::repeat_defaults_follow_the_action_and_an_explicit_value_wins`:
  - `PressKey("A")` → true, `Media(VolumeDown)` → true;
  - `Media(VolumeMute)` → false, `Launch(exe: "wt.exe")` → false;
  - `SwitchLanguage, repeat: true` → true, `PressKey("A"), repeat: false` → false.
- Row added to `unknown_fields_are_errors_that_name_the_field` (`loader.rs:1035-1097`): `("repaet", in_list(r#"( keys: "F13", action: PressKey("A"), repaet: true )"#))`.

Invariant kept: `deny_unknown_fields`; one error per binding; `Display` of every default-config binding unchanged.

Gate: green.

### 3. `Engine: red scenarios for auto-repeat of Immediate bindings (KS-10)` — RED

Files: `src/engine/testing.rs` (D7), `src/engine/scenarios.rs`, `src/engine/properties.rs`.

`scenarios.rs` gets `const TEMPLATE: &str = include_str!("../config/default_config.ron");`, the shipped template, so the KS-10 repro runs against what users get. Load it with `Scenario::from_ron_with_language_hotkey(TEMPLATE, &[VK_MENU, VK_SHIFT])`.

The parent is commit 2, whose engine equals `master`.

| Test | Scenario | PREDICTED on parent |
|---|---|---|
| `engine::scenarios::default_capslock_held_with_ten_repeats_switches_the_language_once` (acceptance 1) | `TEMPLATE`, `.check_press_symmetry().check_nothing_stuck()`, `hold_with_repeats(_, "CapsLock", 10, &[])`, `expect_total_fired(&[SwitchLanguage])` | RED `scenario: repeat CapsLock at +500 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::press_key_backspace_held_with_ten_repeats_fires_eleven_times` (acceptance 2) | `( keys: "F13", action: PressKey("Backspace") )`, symmetry + stuck checks, `hold_with_repeats(_, "F13", 10, &[PressKey(VK_BACK)])`, total `vec![PressKey(VK_BACK); 11]` | **GREEN** (characterization: it goes red under a fix that drops every repeat) |
| `engine::scenarios::volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not` (acceptance 4) | Three fresh `Scenario`s on `F13` with `Media(VolumeUp)`, `Media(VolumeDown)`, `Media(PlayPause)`; each `hold_with_repeats(_, "F13", 3, on_repeat)` with `on_repeat` `[Media(VolumeUp)]`, `[Media(VolumeDown)]`, `[]`; totals 4, 4, 1 | RED on the third: `scenario: repeat F13 at +500 ms: expected to fire [], fired [Media(PlayPause)]` |
| `engine::scenarios::only_press_key_volume_up_and_volume_down_fire_again_on_auto_repeat_by_default` | Table of `(&str, Option<BindAction>)` over `SwitchLanguage`, `SwitchLanguageBackward`, `ToggleCapsLock`, `PressKey("Backspace")`→`Some(PressKey(VK_BACK))`, `PostMessage(msg: "WM_CLOSE")`, `Sequence([Text("x")])`, `Launch(exe: "wt.exe")`, `OpenUrl("https://example.com")`, `Media(PlayPause|Stop|Next|Previous|VolumeMute)`, `Media(VolumeUp)`→`Some`, `Media(VolumeDown)`→`Some`, `DoNothing`. For each: `Scenario::from_ron((keys: "F13", action: <row>)).with_context(row)`, then `down("F13")`, `advance_ms(500)`, `repeat("F13")`, then `expect_fired(&[])` or `&[a]`. The engine never executes actions, so no process, URL or message is started | RED `scenario: repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]` / `context: SwitchLanguage` |
| `engine::scenarios::repeat_false_on_press_key_fires_once_per_press` | `( keys: "F13", action: PressKey("Left"), repeat: false )`, `hold_with_repeats(_, "F13", 10, &[])`, total `[PressKey(VK_LEFT)]` | RED `scenario: repeat F13 at +500 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(37))]` |
| `engine::scenarios::held_shift_capslock_toggles_capslock_once` (KS-10 scenario) | `TEMPLATE`, `down("LShift").expect_passed()`, `hold_with_repeats(_, "CapsLock", 10, &[])`, `up("LShift").expect_passed()`, total `[ToggleCapsLock]` | RED `…repeat CapsLock at +500 ms: expected to fire [], fired [ToggleCapsLock]` |
| `engine::scenarios::held_launch_and_sequence_hotkeys_fire_once_per_press` (KS-10 scenario) | Scenario A: `( keys: "Ctrl+Alt+T", action: Launch(exe: "wt.exe") )`; `down LCtrl`, `down LAlt` (passed); `hold_with_repeats(_, "T", 5, &[])`; total `[launch("wt.exe")]`. Scenario B: `( keys: "Ctrl+Alt+N", action: Sequence([Text("note")]) )`, same script on `N`, no total (Arc equality) | RED A: `…repeat T at +500 ms: expected to fire [], fired [Launch { exe: "wt.exe", args: [] }]` |
| `engine::scenarios::non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` (D3) | `( keys: "F13", action: SwitchLanguage ), ( keys: "F13", action: PressKey("Left") )`; `down F13` → fired `[SwitchLanguage]`; `advance_ms(500)`; `repeat F13` → suppressed, `expect_fired(&[])`; `up F13` suppressed | RED `…repeat F13 at +500 ms: expected to fire [], fired [SwitchLanguage]`. A wrong "skip" fix would show `fired [PressKey(VIRTUAL_KEY(37))]` |
| `engine::scenarios::capslock_held_across_a_focus_change_switches_the_language_once` (foreground change mid-press) | `TEMPLATE`; `down CapsLock` → fired `[SwitchLanguage]`; `foreground_changed`; `advance_ms(500)`; three times (`repeat` → suppressed, nothing fired; `advance_ms(33)`); `up CapsLock` → passed (row 55 behaviour, unchanged); total `[SwitchLanguage]` | RED `…repeat CapsLock at +500 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::capslock_held_through_remapping_off_and_on_switches_the_language_once` (`swch off`/`on` mid-press) | `TEMPLATE`, `check_nothing_stuck`; `down CapsLock` (suppressed, fired); `remapping_off`; `advance_ms(500)`; `repeat CapsLock` → passed, nothing fired (off); `remapping_on` (reset: Windows now has CapsLock down, so it is held + passed); `advance_ms(33)`; `repeat` → passed, nothing fired; `advance_ms(33)`; `repeat` → passed, nothing fired; `up` → passed; total `[SwitchLanguage]` | RED `…repeat CapsLock at +533 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat` (reload mid-press) | `( keys: "CapsLock", action: SwitchLanguage )`; `down` fired; `advance_ms(500)`; `repeat` → nothing fired; `.reload("( bindings: [ (keys: \"CapsLock\", action: SwitchLanguage, repeat: true) ] )")`; `advance_ms(33)`; `repeat` → suppressed, fired `[SwitchLanguage]`; `up` suppressed | RED `…repeat CapsLock at +500 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat` (spec: pin) | All with `action: PressKey("Backspace")` (default `repeat: true`). **Tap(200)** on F13: down; `advance_ms(500)`; three repeats (suppressed, nothing fired); up → nothing fired; total `[]`. **Hold(200)**: down; `advance_ms(500)` → fired `[PressKey(VK_BACK)]` once; three repeats nothing; up nothing; total one. **DoubleTap(600)**: down; `advance_ms(300)`; three repeats inside the term → nothing fired (a repeat is not a second press); up; total `[]`. **sequence** `(steps: ["F13", "F14"], max_gap_ms: 2000)`: down F13 suppressed; `advance_ms(500)`; three F13 repeats nothing; up F13; down F14 → fired once; `advance_ms(500)`; three F14 repeats nothing; up; total one. **chord** `(keys: ["F13", "F14"], window_ms: 50)`: down F13; `advance_ms(20)`; down F14 → fired once; `advance_ms(500)`; three F14 repeats nothing (suppressed); up F14, up F13; total one | **GREEN** (characterization: it goes red if the `PressKey` default leaks into the deferred, sequence or chord arms) |
| `engine::scenarios::press_after_a_lost_key_up_reads_as_a_repeat_and_fires_only_a_repeating_binding` (**replaces** `press_after_a_lost_key_up_and_a_focus_change_follows_its_binding`, `:1511-1527`) | Part 1 is the existing script with `SwitchLanguage`, final `down("F13")` → `.expect_suppressed().expect_nothing_fired()`. Part 2 is the same script with `( keys: "F13", action: PressKey("Left") )` → `.expect_suppressed().expect_fired(&[PressKey(VK_LEFT)])`. The comment cites KS-29 / item 2.3 (Q1) | RED part 1 `…down F13 at +1000 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up` (**edited**, `:1306-1331`) | Add `.expect_nothing_fired()` after each of the three repeats. The comment `:1311` becomes: "A binding that does not repeat fires nothing on a repeat, even once it starts matching (KS-10)" | RED `…repeat CapsLock at +533 ms: expected to fire [], fired [SwitchLanguage]` |
| `engine::scenarios::repeat_check_reports_an_action_fired_on_an_auto_repeat` (harness self-test, in the "harness itself" block `:2455+`) | `#[should_panic(expected = "on an auto-repeat")]`; `( keys: "F13", action: PressKey("Left") )`, `.check_repeats_fire_only(&[])`, `.down("F13").repeat("F13")` | **GREEN** (it panics as expected on parent and after) |
| `engine::properties::auto_repeats_fire_only_actions_that_repeat` | `run_all(Mode::Focus, true)`. `run` applies `scenario.check_repeats_fire_only(&[BindAction::PressKey(VK_LEFT)])`. `PressKey("Left")` (`properties.rs:77`) is the only Immediate repeating action in `CONFIGS`; `PressKey("LShift")` (`:88`) is a `Hold` and fires only from timers. The existing two tests become `run_all(Mode::Strict, false)` / `run_all(Mode::Focus, false)` | RED: a panic containing `on an auto-repeat` with `fired [SwitchLanguage]` (or `[ToggleCapsLock]`). The context is expected to be `config default, seed <n>, Focus mode`; the seed is not predicted |

Why a harness `repeat` step is always an engine repeat in the property. Generated scripts never use `down_unseen`. A held non-modifier stays `active` until a seen key-up. A held modifier is either swallowed (the resync skips it, `keys.rs:100`) or passed (Windows has it down, so the resync keeps it). So after the fix, any non-allowed fire on a `repeat` step is a real defect.

### 4. `Engine: Immediate bindings fire on auto-repeat only with repeat (KS-10)` — makes commit 3 green

Files: `src/engine/mod.rs`, `README.md`, `src/config/default_config.ron`, `CHANGELOG.md`. This is the commit that changes the meaning, so README, template and CHANGELOG go here.

- **Code.** D3 only.
- **Module docs** (`src/engine/mod.rs`):
  - `:120-122` becomes: "`Immediate`: the first matching Immediate user binding takes the event. It fires on a first key-down and, on an auto-repeat, only if it sets `repeat` (by default `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)`; owner decision, KS-10). A binding that does not repeat still takes the auto-repeat, so a later binding never fires in its place. Every matching Immediate binding with `block_default` asks for suppression."
  - `:141-142` "Actions still fire on repeats as rule 4 says." stays.
  - `:156-159`: drop "repeat-firing Immediate actions (KS-10)" from the list of known defects.
- **Results.**
  - Every commit-3 RED turns green.
  - `press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed` (`:157-176`), `immediate_key_held_across_a_focus_change_keeps_its_repeats_suppressed` (`:1440-1463`) and the default-config/template tests stay green.
  - `default_config_parses`, `default_config_commented_examples_load` (with the new example) and `readme_matching_examples_load` pass.
- **Invariant kept.** The A3 symmetry code `:502-534` is byte-identical. Every path still ends in exactly one outcome.

### 5. `Book: the repeat field in the config reference (1.1, KS-10)` — docs only

Files: the nine `book/src/config/*.md` pages in the docs impact.

- Every RON block added or changed in the book is validated with a `cfgcheck` built against the branch, outside the repo under `D:/tmp/ks-orch/p1-1.1-repeat/cfgcheck` (`book/src/methodology.md` "Validation harness"; it uses `config::reload`).
- `mdbook build book -d D:/tmp/ks-orch/p1-1.1-repeat/book`, plus a link check.
- `example.md` must still print `OK: 25 bindings`.
- Gate: green (no code).

## Docs impact (README, template, CHANGELOG, book)

### README.md (commit 4)

- `:94-105`, the `binding.rs` field list: add "- `repeat`: whether an `Immediate` `keys` binding fires again on each auto-repeat".
- `:348-353`, the triggers table: the `Immediate` row becomes "Synchronously on key-down (default — same as no `trigger:` field); on an auto-repeat again only if `repeat` is on (see Per-binding flags)".
- `:378-402`, behaviour rules: add a bullet. "An `Immediate` binding fires on an auto-repeat only if its `repeat` is on: by default for `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)`. Every other action fires once per press. Its repeats are still swallowed when its key-down was. `Tap`, `Hold`, `DoubleTap`, `sequence` and `chord` never fire on an auto-repeat."
- `:453-460`, the per-binding flags table: new row after `allow_extra_modifiers`.

  | `repeat` | `true` for `PressKey`, `Media(VolumeUp)`, `Media(VolumeDown)`; `false` otherwise | `keys` bindings with the `Immediate` trigger: fire again on every auto-repeat while the key is held. With `false` the action fires once per press; the repeats and the release are still swallowed when the press was. `repeat: true` is a load error on `Tap`, `Hold`, `DoubleTap`, `sequence` and `chord` bindings, which never fire on an auto-repeat; `repeat: false` is accepted there. The startup list marks a value that differs from the default with `[repeat]` or `[no repeat]`. |

- `:515`, Errors: the quoted message becomes ``… `trigger`, `allow_extra_modifiers`, or `repeat` instead``. Pinned by `an_unknown_binding_field_lists_repeat_among_the_valid_names`. Add a bullet: `` binding #0 [F13]: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger `` — `repeat: true` on a `Tap`, `Hold`, `DoubleTap`, `sequence` or `chord` binding.
- `:651-664`, Implementation Features: add "**Auto-repeat policy**: holding a key repeats `PressKey` and the volume keys like a real key; every other action fires once per press unless the binding sets `repeat: true`".

### src/config/default_config.ron (commit 4)

- `:104`: `trigger: Immediate — fire on key-down (the default); on an auto-repeat only with repeat (see the per-binding flags)`.
- `:158-161`: after the `allow_extra_modifiers` block, add:

  ```
  // repeat: true | false  (default true for PressKey, Media(VolumeUp) and
  //                        Media(VolumeDown), false for every other action)
  //                       - keys bindings with the Immediate trigger: also fire
  //                         on every auto-repeat while the key is held. With
  //                         false the action fires once per press; the
  //                         repeats are still swallowed when the press was.
  //                         repeat: true is a load error on Tap, Hold,
  //                         DoubleTap, sequence and chord bindings, which
  //                         never fire on an auto-repeat.
  ```

- `:212-217`, the Media block: add the comment `// VolumeUp/VolumeDown keep changing the volume while held; the others fire once per press unless repeat: true:` and the example `// ( keys: "Ctrl+Alt+PageDown", action: Media(Next), repeat: true ),`. It must load through `default_config_commented_examples_load` (`loader.rs:641-661`); `pagedown` parses (`vk_name.rs:153`).
- The active bindings (`:178-186`) are unchanged: CapsLock now switches once per press by default.

### CHANGELOG.md, `## [Unreleased]` (commit 4)

- **Added** (end of the list): "Per-binding `repeat: true | false` for `keys` bindings with the `Immediate` trigger (roadmap item 1.1): whether the action fires again on every auto-repeat while its key is held. It defaults to `true` for `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)` and to `false` for every other action. `repeat: true` on a `Tap`, `Hold`, `DoubleTap`, `sequence` or `chord` binding is a load error, because those never fire on an auto-repeat. The startup list marks a value that differs from the default with `[repeat]` or `[no repeat]`."
- **Changed**: "**Breaking:** an `Immediate` `keys` binding whose action is not `PressKey`, `Media(VolumeUp)` or `Media(VolumeDown)` now fires once per press instead of on every auto-repeat (KS-10). Add `repeat: true` for the old behaviour, for example to skip tracks with a held `Media(Next)` key."
- **Fixed**: "KS-10: holding a key no longer re-runs a one-shot action at the auto-repeat rate. With the default config, holding CapsLock switches the layout once and holding Shift+CapsLock toggles CapsLock once. A held `Launch` or `OpenUrl` hotkey starts one program, and a held `Sequence` macro runs once. The repeats are still swallowed when the key-down was. A press the daemon reads as an auto-repeat, because the key's previous release went to the lock screen or another desktop, now fires nothing for such a binding; the next press works (KS-29, roadmap item 2.3)."
- Edit `:202-203`: "such a repeat still fires an `Immediate` action (KS-10, roadmap item 1.1)" becomes "such a repeat fires an `Immediate` action only if the binding repeats (KS-10, below)".

### book/src/config (commit 5; facts become "after `feat/repeat-flag`"; pinned links are added by the book sync)

- **`structure.md`**
  - `:141-151`: field-table row `repeat`, same text as the README row.
  - `:155-165`: add `repeat: true,` to "Example with all fields"; `PressKey` makes it a no-op, so write `repeat: false` to show an override, and validate it.
  - `:177-183`: kind table row: `repeat` | honoured with `Immediate`; `true` is an error with `Tap`/`Hold`/`DoubleTap` | `true` is an error | `true` is an error.
  - `:192`: the message gets `` `allow_extra_modifiers`, or `repeat` instead ``.
- **`README.md` (config)** `:39-43`: add the sketch line `repeat: true | false,  // = true for PressKey, Media(VolumeUp/VolumeDown), else false; keys + Immediate only (§11.3)`.
- **`kinds.md`**
  - `:26`: "Matching is checked on every key-down, auto-repeats included; an `Immediate` binding fires on an auto-repeat only with `repeat` ([§11.3](triggers-conditions.md#s11-3))."
  - `:33-34`: replace the KS-10 warning with a NOTE that the issue is fixed by `repeat` and that the old behaviour is `repeat: true`.
  - `:65`, `:132`: add "`repeat: true`" to the list of errors on `sequence`/`chord`.
- **`triggers-conditions.md`**
  - `:13`: `Immediate` | … | "on key-down; on an auto-repeat again only with `repeat` (§11.3)".
  - `:38-48`: §11.3 gets a bullet for the repeat rule and the slot rule (D3). Replace the KS-10 warning with a short example pair (`CapsLock` once; `( keys: "F13", action: Media(Next), repeat: true )` repeats), validated.
  - `:59`: unchanged.
- **`actions.md`**
  - `:42`: "key press; on auto-repeat only with `repeat`".
  - `:56-57`: replace the KS-10 warning with a NOTE: defaults, opt-in, and that `Tap(…)` is no longer needed for one-shot actions.
  - `:167`: "With `Immediate`, a held trigger produces a series of taps at the auto-repeat rate" stays true for `PressKey` (default `repeat: true`); add "unless `repeat: false`".
  - `:293`: drop "with `Immediate` auto-repeat that means dozens of parallel macros"; a macro now starts once per press unless `repeat: true`.
- **`known-issues.md`**
  - Row 8 (`:34`): Status "**Fixed** (`feat/repeat-flag`): an `Immediate` binding fires on auto-repeat only with `repeat`".
  - New row 62 in §19.9: "`Immediate` bindings other than `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)` fire once per press; a held `Media(Next)` no longer skips repeatedly. Add `repeat: true`" | By design (owner decision) | `feat/repeat-flag` | §11.3 | KS-10.
  - New row 63: "A press that follows a key-up the hook never saw (released on the lock screen or the secure desktop) reads as an auto-repeat; a non-repeating `Immediate` binding fires nothing for it, and the next press works" | Open until item 2.3 | §11.3 | KS-29.
- **`errors.md`**
  - `:136`: append ", and `repeat: true` outside `keys` + `Immediate`" to the order sentence.
  - New table row 18: "`repeat: true` only together with `keys` and `Immediate`" | `` `repeat: true` applies only to `keys` bindings with the `Immediate` trigger ``.
- **`matching.md`** `:132`: "firing on auto-repeat ([§7] …; [KS-10] …)" becomes "the `repeat` flag ([§11.3])".
- **`example.md`**
  - `:46-47`: drop the "Known issue: Immediate repeats" lines (CapsLock now switches once).
  - `:67-68`: keep (PressKey repeats by default).
  - `:75-77`: change "Immediate is handy here" to "VolumeUp and VolumeDown repeat while held by default".
  - `:100-102`: change "All three F22 bindings are Tap(300): with Immediate, auto-repeat would minimize or close windows one after another" to "Tap(300): a short press only, so a held F22 does nothing".
  - `:120-129`: add `repeat: true` to the VS Code `F24` binding (`Sequence` does not repeat by default; with it, F24 "repeats like an ordinary key" in both windows, as the comment says). The binding count stays 25.
  - `:181-182`: "Tap protects against auto-repeat" becomes "Tap(400): the macro starts on a short press only".
  - `:229`: delete the KS-10 bullet from the WARNING list.
  - `:6`: "every binding kind, every trigger, … and `allow_extra_modifiers`" gets "and `repeat`".

Book-sync follow-ups (post-merge, doc-writer): the files listed under "Post-merge book sync" in the Touch set.

## Acceptance mapping

| Roadmap / brief acceptance line | Covered by |
|---|---|
| CapsLock down, 10 repeats, CapsLock up give exactly one `SwitchLanguage`, and every event is suppressed | `engine::scenarios::default_capslock_held_with_ten_repeats_switches_the_language_once` (shipped template, symmetry + stuck checks); manual #1 |
| `PressKey("Backspace")` on a key held with 10 repeats fires 11 times | `engine::scenarios::press_key_backspace_held_with_ten_repeats_fires_eleven_times`; manual #3 |
| `repeat: true` on `SwitchLanguage` restores firing on every repeat | `engine::scenarios::repeat_true_on_switch_language_fires_on_every_repeat`, `engine::scenarios::reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat`; manual #5 |
| `Media(VolumeUp)` repeats by default and `Media(PlayPause)` does not | `engine::scenarios::volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not`, `engine::scenarios::only_press_key_volume_up_and_volume_down_fire_again_on_auto_repeat_by_default`, `core::windows_actions::tests::only_press_key_volume_up_and_volume_down_repeat_by_default`; manual #4 |
| A repeat of a non-repeating binding is still suppressed when the key-down was (A3) | `…default_capslock_held_with_ten_repeats…` (`check_press_symmetry`), `…capslock_held_across_a_focus_change_switches_the_language_once`, `…repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up`, existing `engine::properties::every_press_is_suppressed_or_passed_as_a_whole` |
| Deferred triggers, sequences and chords ignore repeats; pinned | `engine::scenarios::deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat`, `engine::properties::auto_repeats_fire_only_actions_that_repeat`, `config::loader::tests::repeat_true_is_an_error_on_deferred_sequence_and_chord_bindings` |
| Field on `RawBinding`, `deny_unknown_fields` stays | `config::loader::tests::an_unknown_binding_field_lists_repeat_among_the_valid_names`, `…unknown_fields_are_errors_that_name_the_field` (`repaet` row), `…repeat_false_loads_on_every_kind_and_trigger`, `…repeat_defaults_follow_the_action_and_an_explicit_value_wins` |
| README, `default_config.ron`, book config page, CHANGELOG updated | Commit 4/5 diff; `config::loader::tests::default_config_parses`, `…default_config_commented_examples_load`, `…readme_matching_examples_load`; cfgcheck + mdbook in commit 5 |
| KS-10 scenarios: Shift+CapsLock flicker, `Launch` per repeat, `Sequence` worker per repeat | `…held_shift_capslock_toggles_capslock_once`, `…held_launch_and_sequence_hotkeys_fire_once_per_press` |
| Checkpoint 1 smoke #2 ("Holding CapsLock for 2 s gives one switch … osk.exe") | manual #1 |

## Manual checks (human only)

No `#[ignore]` test is needed: every decision is in the engine and is covered by scenarios. These checks need a live hook. Use a release build, `swch open` from a normal console, and the default config unless noted.

| # | Setup | Steps | Expected | Ignored test that names it |
|---|---|---|---|---|
| 1 | Default config, Notepad, `osk.exe` open, two layouts | Hold CapsLock 2 s, release | The layout switches exactly once; the CapsLock state and light are unchanged; nothing is held in `osk.exe` | none |
| 2 | Default config | Hold Shift+CapsLock 2 s | CapsLock toggles exactly once | none |
| 3 | Add `( keys: "F9", action: PressKey("Backspace") )`; Notepad with text | Hold F9 1.5 s | Characters are deleted continuously at the auto-repeat rate | none |
| 4 | Add `( keys: "Ctrl+Alt+Up", action: Media(VolumeUp) )` and `( keys: "Ctrl+Alt+P", action: Media(PlayPause) )`; media playing | Hold each 2 s | The volume climbs continuously; playback toggles once | none |
| 5 | Change the CapsLock binding to `repeat: true` and save (hot reload) | Read the console; hold CapsLock 2 s | `[INFO] config reloaded`; the listed line ends with `[repeat]`; the layouts cycle | none |
| 6 | Add `( keys: "Win+Alt+T", action: Launch(exe: "wt.exe") )` | Hold Win+Alt+T 2 s | Exactly one terminal window | none |
| 7 | Daemon running; save `( keys: "F9", action: SwitchLanguage, trigger: Tap(200), repeat: true )` | Read the console | `[ERROR] config reload failed:` with ``binding #N [F9]: `repeat: true` applies only to `keys` bindings with the `Immediate` trigger``; the old bindings stay live | none |

## Risks and edge cases (R1–R10 and lane-specific)

- **R1, hook/engine.**
  - The only code change in the decision path is D3: one `bool` read, and the action clone and `Vec` push move under a condition.
  - Every path still ends in `LRESULT(1)` or `CallNextHookEx`; the adapter is untouched (`keyboard_hook_callback.rs:233-268`).
  - The suppression code `:502-534` and the key-up code `:538-564` are untouched. Per-key indices still go through `slot()` (`keys.rs:46-49`).
  - Allocation per key-down goes down: no clone and no `fire` allocation for a non-repeating repeat.
  - No panic path is added (no indexing, no `expect`).
- **R2, sentinel.** No new `SendInput`. `PressKey`/`Media` repeats still go through `press_vk` (`windows_actions.rs:163-202`), which sends a down/up pair tagged with `injected_sentinel()`.
- **R3, shared state.** No new shared state. The flag is immutable inside the `ArcSwap` snapshot. `with_engine` is unchanged.
- **R4, orderings.** None new.
- **R5, builders.** `with_repeat(self, bool) -> Self`, no setter; `with_kind`/`new` keep two positional arguments.
- **R6, Display.** No new enum variant. The `Binding` suffix follows D5. `repeats_by_default` has no `_` arm, so a future `BindAction` variant fails to compile until someone decides its default.
- **R7, specificity.** `sort_by_specificity`/`update_bindings` are untouched. D3 keeps "first matching Immediate" on repeats too, and `non_repeating_winner_keeps_the_immediate_slot_on_a_repeat` pins it.
- **R8, key identity.**
  - Repeat identity is the raw sided code (`keys.rs:143-148`).
  - With `keys: "Shift"` → `SwitchLanguage`, LShift's repeats fire nothing, while RShift pressed during the hold is a fresh press of another physical key and fires. That is the correct physical semantics, and it is the same in combo, chord (ignores repeats) and sequence (ignores repeats).
  - AltGr's fake LCtrl repeats are repeats of LCtrl, so a `Ctrl+RAlt` binding fires once per AltGr press.
- **R9, registry.** Untouched. The `block_original_combo` lookup is unchanged, and the template test uses a stand-in hotkey.
- **R10, message loop and shutdown.** Untouched.

Lane-specific risks:
- **Lost key-up (KS-29).** After a key-up goes to the secure desktop, the next press reads as a repeat, is swallowed and, for a non-repeating binding, fires nothing; the following press works. On `master` it fired. This is pinned by the replaced `press_after_a_lost_key_up_…` test and documented (CHANGELOG Fixed, known-issues row 63). The root fix is item 2.3. See Q1.
- **A binding that starts matching mid-hold** (a modifier released, a `when:` title change, `allow_extra_modifiers`) does not fire on the repeats of a non-repeating binding. A `repeat: true` binding does fire. Pinned by the edited `:1306-1331` test.
- **A repeat without a record** (a modifier Windows has down that the engine never saw): it gets no fire for non-repeating bindings; its suppression is unchanged (`:524-526`).
- **Reload mid-hold.** The next repeat uses the new snapshot's flag (no engine reset, `keyboard_hook.rs:110-115`). Pinned by `reload_mid_hold_…`.
- **`swch off`/`on` mid-hold.** Pinned by `capslock_held_through_remapping_off_and_on_…`.
- **Foreground change mid-hold.** Pinned by `capslock_held_across_a_focus_change_…`.
- **Shutdown.** No new state to tear down.
- **`PressKey` on a `Hold`/`Tap`** gets `repeat == true` by default, and the engine ignores it there. Pinned by `deferred_sequence_and_chord_…`. `Display` hides it (D5), so a `Hold` line never shows `[no repeat]` when `repeat: false` is written.
- **UIPI and elevation.** No IPC or process change. Fewer elevated `Launch` children per held hotkey reduces KS-09 exposure.
- **IME and layouts.** VK repeats are layout-independent; nothing new.
- **Compatibility.** Every existing config loads unchanged. A held non-repeating `Immediate` hotkey now fires once (**Breaking:** in CHANGELOG). A config that uses `repeat:` fails on an older daemon with "Unexpected field named `repeat`", which is acceptable: there is no forward-compatibility promise.

## Questions for the orchestrator

1. **The KS-29 corner.** Owner-visible, rare. After a key's release is lost (Win+L or UAC while holding CapsLock), the next CapsLock press is read as an auto-repeat. With this lane it is swallowed and switches nothing; the press after it works. On `master` it switched. **Recommendation: accept it as is, documented** (CHANGELOG Fixed entry, known-issues row 63, the scenario comment).
   - Item 2.3 fixes the root by resyncing all 256 keys on unlock and desktop switch.
   - A time-gap heuristic would be KS-29 scope creep and hardware-dependent (`SPI_GETKEYBOARDDELAY` doc).
   - This does not block the lane.
2. **Book config pages: in-lane (commit 5, developer) or in the phase book sync (doc-writer)?** **Recommendation: in-lane for the nine `book/src/config/*.md` pages**, as CLAUDE.md requires: "updates the matching `book/src/config/*.md` page … in the same lane". The book deploys on every push to `master`, so a deferred update would publish "Known issue: Immediate repeats" after the fix. The audit, roadmap, architecture, overview, glossary and gaps status lines stay with the book sync, because they cite the merge sha. This does not block the lane.

## Cut readiness checklist

### Spec
- [x] KS-10 read in `audit.md:345-358`. Its scenarios (CapsLock about 30 Hz, Shift+CapsLock flicker, `Launch` per repeat, `Sequence` worker per repeat) are reproduced as tests in commit 3. KS-06 (A3 invariant) and KS-29 (Q1) were read for interplay.
- [x] Every book reference is re-located (table above). The defect is present on `dbd55ca`.
- [x] Goal, non-goals and dependent items (1.6, 2.1, 3.4) are stated.

### Design
- [x] D1–D7 each have a rationale and rejected alternatives.
- [x] Owner decisions: defaults exactly `PressKey`/`VolumeUp`/`VolumeDown` (D2); exact modifiers and `allow_extra_modifiers` untouched; RON kept; `log` N/A (no new logging); licence N/A.
- [x] Thread named for every path (Threads table). Nothing is added to the LL callback; work is removed.
- [x] Install, uninstall and timers: N/A, none touched.
- [x] `SendInput`: no new path; the `PressKey` repeat pairs are unchanged and sentinel-tagged.
- [x] Sided and unsided: repeat identity by raw code, consistent across combo, chord and sequence (R8).
- [x] Reload, `swch off`/`on`, foreground change and lost key-up during a press: each has a scenario. Shutdown: N/A.
- [x] Atomics: none new; no `static mut`.
- [x] No new module cycle (`data → core` and `engine → data` edges already exist). The logic is in `engine` and reached by scenarios.

### Win32
- [x] No new API. The facts relied on are linked and quoted; features are unchanged against `Cargo.toml:41-50`.
- [x] Elevation, UIPI, pipe DACL and token: N/A (no change). KS-09 exposure is reduced.

### Commits and tests
- [x] Red-first: commits 1 and 3 precede fixes 2 and 4. Exact predicted messages are given; the characterization tests are declared GREEN.
- [x] All tests go through `Engine::on_key`/`on_timer`, the fake `Env` and the fake clock, or through the loader. Nothing installs a hook or injects input. `Launch`/`OpenUrl` rows are never executed by the engine.
- [x] No test needs real Win32, so there are no `#[ignore]` tests; the manual checks are listed.
- [x] Every commit passes fmt, clippy and test, except the declared red commits 1 and 3, which still compile.
- [x] Bisectable. README, template and CHANGELOG are in commit 4, which changes the meaning. Commit 2 adds the field without the engine honouring it; its commit message says so.

### Acceptance
- [x] Every roadmap and brief acceptance line maps to tests and manual checks.
- [x] Every question has a recommendation, and neither blocks.

## Addendum (developer, round 1): critique remarks

Critique: `D:/tmp/ks-orch/p1-1.1-repeat/critique.md` (Critical 0, Important 1, Minor 4). Branch `feat/repeat-flag`, HEAD `ad96e73`.

### I1 (Important): RESOLVED in the docs; Q1 restated for the orchestrator

The critique is right: `book/src/roadmap.md:377` has `reset_transient_state()` resync "all 256 keys, except blocked ones", and `KeyState::key_down_swallowed` (`src/engine/keys.rs:134-139`) counts `Suppressed` and `Forgotten` as swallowed. Item 2.3 as written repairs a lost key-up only for a key whose key-down passed.

Changes:
- `CHANGELOG.md`, Fixed, KS-10 entry: "For a key whose key-down the daemon swallowed, which is the usual trigger key, no roadmap item repairs that record yet: the key resync of roadmap item 2.3 leaves such keys alone." (no promise of 2.3).
- `book/src/config/known-issues.md` row 63: Status "Open. For a key whose key-down was swallowed, which is the usual trigger key, no roadmap item repairs it: the key resync of roadmap item 2.3 leaves such keys alone, because Windows never saw them go down".
- `src/engine/scenarios.rs:1516` (`press_after_a_lost_key_up_reads_as_a_repeat_and_fires_only_a_repeating_binding`): the comment says the same and the test pins that the press after the lost-key-up press fires again.

**Q1, restated (for the owner, via the orchestrator).** After a key's release is lost (Win+L, UAC or Ctrl+Alt+Del while the key is held), the engine still holds the key, so its next press reads as an auto-repeat. With this lane that press fires nothing for a binding that does not repeat (on `master` it fired); the press after it works.
- Key whose key-down **passed** (L after Win+L): roadmap item 2.3's resync on unlock/desktop switch repairs it.
- Key whose key-down was **swallowed** (CapsLock, the T of `Ctrl+Alt+T`, any `Immediate` trigger key with `block_default`): item 2.3 explicitly excludes it, because `GetAsyncKeyState` reports such a key up whether it is held or not. No roadmap item repairs it today.
- Decision needed: accept the lost first press for swallowed keys as documented, or extend item 2.3's acceptance to swallowed keys on unlock and desktop switch (a roadmap/owner call; no code in this lane).

### M1: RESOLVED in impl.md

Manual check #5 in `impl.md` checks the `[repeat]` marker after a restart; a hot reload prints only `[INFO] config reloaded (N bindings live)` (`src/hook/config_watcher.rs:163`). The marker is also pinned by `config::loader::tests::display_of_a_loaded_binding_marks_a_repeat_flag_that_differs_from_its_action_default`.

### M2: RESOLVED

The owner default is untouched. The "repeats like a real key" wording is gone, and the pitfall is documented: README per-binding flags row and "Auto-repeat policy" bullet; `src/config/default_config.ron` repeat block ("Each repeat of a PressKey sends a new press: for a toggle key (CapsLock, NumLock) or a modifier, write repeat: false."); CHANGELOG Changed entry; `book/src/config/actions.md` 13.1 note and 13.5 `PressKey`; `book/src/config/known-issues.md` row 64.

### M3: RESOLVED (master-replay message given)

`engine::scenarios::repeat_false_on_press_key_fires_once_per_press` sets `repeat: false` in its initial config. Replayed on `master` it fails with `scenario: config does not load: config parse error (<in-memory>): 1:..: Unexpected field named `repeat` in `RawBinding`, ..` (the same mechanism as the commit-1 red of `repeat_true_on_switch_language_fires_on_every_repeat`, observed: `1:64: Unexpected field named `repeat` in `RawBinding``). On commit 2 (`96528bb`) it fails with the predicted `scenario: repeat F13 at +500 ms: expected to fire [], fired [PressKey(VIRTUAL_KEY(37))]` (observed on commit 3). `reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat` puts `repeat:` only in the reload after the first failing repeat, so its message is the same on `master`. Both messages are in the commit-3 message body.

### M4: RESOLVED

`book/src/config/example.md`: the VS Code `F24` `Sequence` keeps the default (once per press); its comment now says so. `repeat: true` is shown on a new binding `( keys: "Ctrl+Alt+]", action: Media(Next), repeat: true )` in section 2. The example therefore has 25 bindings from the file and `OK: 26 bindings` with the auto-blocker (validated with cfgcheck on the branch); the text and the three-key checking order are updated.
