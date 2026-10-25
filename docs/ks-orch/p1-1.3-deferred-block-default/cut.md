# Cut: p1-1.3-deferred-block-default — roadmap item 1.3 (`block_default` for Tap, Hold and DoubleTap)

Tree: `master` @ `1afa720` (merge of `feat/repeat-flag`; `.git/refs/heads/master`). Branch to cut: `feat/deferred-block-default`. Every `path:line` below is at `1afa720`.

## Verdict

CUT READY

Three non-blocking questions (Q1-Q3) at the end, each with a recommendation. None changes the commits below.

## Spec re-located

| Spec reference (book) | Current file:line | Status | Note |
|---|---|---|---|
| roadmap.md:255 "deferred triggers suppress unconditionally", `keyboard_hook_callback.rs:241` (Tap) @7cb3521 | `src/engine/mod.rs:421-432`, unconditional `decision.suppress = true` at `:431` | moved (Workstream A, `16a0461`) | Defect still present |
| same, `:246` (Hold) | `src/engine/mod.rs:433-446`, `:445` | moved | Defect still present |
| same, `:251` (DoubleTap) | `src/engine/mod.rs:447-460`, `:459` | moved | Defect still present |
| book config pages cite `src/engine/mod.rs:413-452` @1dfd80a (kinds.md:170, triggers-conditions.md:59, known-issues.md:122) | `src/engine/mod.rs:421-460` | moved (lane 1.1 added the `repeat` lines) | |
| Immediate honours `block_default` (the model to copy) | `src/engine/mod.rs:407-420`, gate at `:417-419` | moved | Unchanged by this lane |
| "becomes `Option<bool>` in the schema" | `src/config/schema.rs:43-44` (`#[serde(default = "default_true")] pub block_default: bool`), `default_true` at `:229-231` | unchanged | `default_true` has no other user (Grep) |
| loader applies it | `src/config/loader.rs:385-390` (`.with_block_default(raw.block_default)` at `:386`) | — | `kind` built at `:345`, `trigger` at `:350` |
| `Binding` field and constructor defaults | `src/data/binding.rs:53` (field), `:98` (`with_kind` → `true`), `:111` (`new_auto_blocker` → `true`), `:131-134` (`with_block_default`) | unchanged | |
| "combos made only of modifiers" | inline `combo.keys.iter().all(\|k\| k.modifier().is_some())` in `is_trigger_key`, `src/engine/matching.rs:58-63` (`:61`) | moved | Becomes `KeyCombination::is_modifier_only` (D2) |
| "a key-up is suppressed if and only if its key-down was" | key-up rule `src/engine/mod.rs:562-570` (`suppress = was_blocked \|\| (gesture_bound && !was_passed)`); records `src/engine/keys.rs:51-66`; repeat rule `src/engine/mod.rs:510-542` | moved | Already correct for a passed key-down with a pending Tap/Hold; pinned in advance by `engine::tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending` (`src/engine/mod.rs:622-656`), whose comment says 1.3 makes the state reachable |
| Engine rule text "Tap, Hold and DoubleTap ... always suppress" | `src/engine/mod.rs:120-128` (`:125-128`) | — | Doc update |
| README LShift example + caveat | `README.md:340-347` | moved (`:262` @7cb3521, `:340` @1dfd80a) | |
| README behaviour rule "`block_default` is implicitly true for Tap/Hold/DoubleTap" | `README.md:381-383` | — | |
| README flags table row | `README.md:463`; feature bullet `README.md:666` | — | |
| Template LShift example ("until roadmap item 1.3 ...") | `src/config/default_config.ron:268-275` | moved (`251-258` @1dfd80a) | Brief: mark as working |
| Template flag comment | `src/config/default_config.ron:144-145` | — | |
| CHANGELOG KS-07 entry "until roadmap item 1.3 it swallows every left-Shift press" | `CHANGELOG.md:231-234` | — | Edit (Unreleased) |
| Checkpoint 1 smoke #6 "A double LShift toggles CapsLock; left Shift + A types A" (KS-07) | `book/src/roadmap.md:310` | unchanged | Manual check M1 |
| Phase-0 manual check 6 (LShift DoubleTap swallows) | `book/src/roadmap.md:199` | unchanged | Book sync updates the expectation |
| Tests that pin today's swallowing of modifier-only deferred bindings | `src/engine/scenarios.rs:900-928`, `:1337-1356`, `:2047-2071`, `:2073-2101`, `:2103-2134`, `:2192-2213`, `:2733-2758`; `src/engine/properties.rs:97-104`, `:126-135` | — | Grep for modifier-only `keys` with `trigger` found exactly these (plus `loader.rs:836`, `:1490`, parse-only) |

KS issues: the roadmap row lists none (`roadmap.md:218`). Smoke #6 cites KS-07 (`audit.md:287-306`, fixed in `82f542f`, read). Risks below touch KS-12, KS-13, KS-14, KS-19 (headers read: `audit.md:376`, `:391`, `:406`, `:481`).

## Goal and non-goals

**After the lane.**
- `block_default` is `Option<bool>` in `RawBinding`. Absent: `false` for a `keys` binding made only of modifiers with `Tap`, `Hold` or `DoubleTap`; `true` for everything else, every `Immediate` binding included (unchanged).
- `Tap`, `Hold` and `DoubleTap` ask for suppression only with `block_default`, exactly as `Immediate` does. With `false` every event of the key (key-down, repeats, key-up) reaches Windows and the gesture still arms and fires.
- The suppression invariant holds by the existing record (`keys.rs` `LastDown`): no key-up rule changes.
- The template's `LShift` `DoubleTap(250)` example is marked as working; README, CHANGELOG and the config pages say so.

**Non-goals (left to later items).**
- Cancelling a pending gesture when another key is pressed (Q1; 2.2 owns gesture arbitration, `roadmap.md:360`).
- Choosing one winner among bindings on one press: suppression stays the OR over matching bindings (1.6, KS-13).
- Mask key for a lone Alt/Win tap that now reaches Windows (1.2, KS-12).
- Releasing held modifiers around injected output (3.1, KS-19).
- Rejecting `block_default` on `sequence`/`chord`, where it stays accepted and ignored (follow-up F2).
- A startup-list marker for a resolved `block_default` (`Display for Binding` shows it for no binding today, `src/data/binding.rs:168-212`).

**Later items that build on this.** 1.6 acceptance "A winner with `block_default: false` lets the key through" (`roadmap.md:289`); Checkpoint 1 smoke #6.

## Decisions

### D1: The default is resolved once, at load time, into `Binding::block_default: bool`

- **What.** Only `RawBinding` gets `Option<bool>`. The loader turns `None` into `Binding::blocks_by_default(&kind, trigger)` and passes the result to `with_block_default`. `Binding` keeps `pub block_default: bool`.
- **Why.** The engine reads one resolved bool per matching binding on the hook thread. No policy runs on the hot path, and the rule lives in one place that unit tests reach. The cfgcheck output (`methodology.md:408`) and every test see the resolved value. Precedent: `repeat` is `Option` in the schema (`schema.rs:74-75`) and resolved by the loader (`loader.rs:391-395`).
- **Rejected.** (a) `Option<bool>` in `Binding`, resolved in `Engine::on_key_down`: config policy leaks into the engine, and every consumer (Display, cfgcheck, the dedup) would have to resolve it again. (b) Deriving it inside `with_trigger`: the result would depend on builder order (`with_block_default(true).with_trigger(..)` would overwrite an explicit value).
- **Cost.** Programmatic bindings (`Binding::new`, `with_kind`) keep `true` whatever their trigger (D4).

### D2: The default rule, and one definition of "made only of modifiers"

- **What.**
  - `KeyCombination::is_modifier_only(&self) -> bool` in `src/data/key_combination.rs`: every key has `KeySpec::modifier().is_some()` (`key_spec.rs:141-148`). That covers `AnySide`, sided codes and the generic codes. `parse_combo` never yields an empty combination (`parsing.rs:37-46`).
  - `is_trigger_key` (`matching.rs:58-63`) calls it.
  - `Binding::blocks_by_default(kind: &BindingKind, trigger: Trigger) -> bool` returns `false` iff `kind` is `Combo(c)`, `c.is_modifier_only()` and `!trigger.is_immediate()`. Otherwise it returns `true`.
- **Why.** The spec's rule, verbatim. The trigger-key rule and the default must agree on what counts as a modifier-only combo: `"LShift"` and `"Alt+Shift"` arm on any of their keys precisely because they have no payload key. `Win` (`AnySide(Win)`) and `LWin` both count. `Immediate` keeps `true`, so `"Alt+Shift"` Immediate still swallows its second modifier (spec: Immediate unchanged). Sequence and chord return `true`: the engine ignores the flag for them (`mod.rs:463-507`).
- **Rejected.** "Contains a modifier": that would make `"Shift+CapsLock"` with `Tap` pass CapsLock, contradicting "true for combos that contain a non-modifier key". A rule that also covers Immediate: forbidden by the spec.
- **Cost.** None.

### D3: The engine gates deferred suppression on `binding.block_default`

- **What.** In `Engine::on_key_down`, the three `decision.suppress = true;` statements (`mod.rs:431`, `:445`, `:459`) become the same gate as `:417-419`: `if binding.block_default { decision.suppress = true; }`.
  - Arming, firing and the timer request are untouched, so a gesture is detected whatever the flag says.
  - Repeats still `continue` before the gate (`:422-424`, `:434-436`, `:448-450`) and follow the record (`:510-534`).
  - The key-up path is untouched. A passed key-down is recorded `Passed` (`:541`), and its key-up passes even when its Tap fires or its Hold is pending (`:567-570`).
- **Why.** This is the smallest change, and it puts the logic where the scenario tests reach it. The invariant comes from the record, not from the binding, so a reload, a foreground change (`forget_blocked` keeps `Passed`, `keys.rs:212-218`) or a reset (`reset_from_os` marks held keys `Passed`, `keys.rs:117-130`) cannot split a press.
- **OR semantics.** Several bindings may match one press. The event is suppressed if any of them asks: an Immediate with `true`, a deferred one with `true`, a blocker, a chord or a sequence. That is today's rule for Immediate bindings, extended. 1.6 replaces it with one winner.
- **Rejected.** A separate "pass-through" record per gesture: redundant with `LastDown`.
- **Cost.** One bool read per matching deferred binding. No allocation, no lock, no Win32 call.

### D4: Constructors keep `block_default: true`

- **What.** `Binding::with_kind` (`binding.rs:93-105`) and `new_auto_blocker` (`:107-119`) are unchanged. The field doc says that the loader resolves a missing config value with `blocks_by_default`.
- **Why.** `with_kind` does not know the trigger yet, and `with_trigger` must not overwrite an explicit value (D1). Hook and adapter tests build F13 bindings with the builder and rely on `true` (`keyboard_hook_callback.rs:356-377`, `keyboard_hook.rs:443-448`).
- **Rejected.** A three-argument constructor: R5 allows at most two positional arguments.
- **Cost.** Documented asymmetry between the builder API and the config default.

### D5: `Some(..)` and `None` are accepted, like the other optional fields

- **What.** `#[serde(default)] pub block_default: Option<bool>`. Under `IMPLICIT_SOME` (`loader.rs:242-243`), `block_default: false`, `Some(false)` and `None` all load, and `None` means the default. `fn default_true` (`schema.rs:229-231`) is deleted: it would be dead code, and `clippy -D warnings` would fail.
- **Why.** Same behaviour as `keys`, `sequence`, `chord` (`ron-syntax.md:83-88`) and `repeat` (`schema.rs:74-75`).
- **Rejected.** A custom deserializer that rejects `None`: extra code for no user value.
- **Cost.** The book claim `block_default: Some(true)` gives `Expected boolean` (`ron-syntax.md:91`) becomes false. Commit 4 edits it, and test L3 pins the new behaviour.

### D6: No interrupt rule in this lane

- **What.** A pending pass-through Tap, DoubleTap or Hold is not cancelled by another key's key-down.
- **Why.** That is today's semantics for every gesture, pinned by `key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing` (`scenarios.rs:1044-1067`, the Tap fires after A was typed). The spec and the acceptance lines say nothing about it. Changing it touches every gesture or splits the rules by flag, and gesture arbitration is 2.2's scope (`roadmap.md:360`).
- **Consequences (documented, not pinned as correct).**
  - `"LShift"` with `Tap(200)` also fires when Shift+A is released within 200 ms.
  - `"LShift"` with `DoubleTap(250)` fires on two separately shifted letters typed within 250 ms.
  - See Q1.
- **Rejected.** Shipping an interrupt rule now: scope creep and a semantic change the owner has not decided.

### D7: The auto-blocker dedup is unchanged

- **What.** A `keys` binding on exactly the Windows hotkey's keys still replaces the blocker (`loader.rs:271-312`). Its own block_default, now resolved, decides whether Windows sees the hotkey, which is what the doc comment at `loader.rs:276-279` already says.
- **Consequence (Breaking, documented).** `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200) )` with the Alt+Shift hotkey now lets Windows switch as well (net: no change of layout). `block_default: true` restores the old behaviour. Pinned by scenario k. See Q2.
- **Rejected.** Keeping the blocker when the replacing binding does not block: that contradicts an explicit `block_default: false` on an Immediate binding, which KS-18 settled (`CHANGELOG.md:268-281`).

### D8: Harness method `Scenario::expect_down_in_windows(key)`

- **What.** A test-only method in `src/engine/testing.rs`:
  - signature: `#[track_caller] pub(crate) fn expect_down_in_windows(self, key: &str) -> Self`;
  - it reads `FakeEnv::os_down` (`testing.rs:66`) for `physical_key(key)`;
  - failure message: `"{label}: {key} is not down in Windows"`, where `label` is the last step's.
- **Why.** Acceptance 2 says "A passes *together with Shift*". This makes the claim literal: at A's key-down, the modelled OS has LShift down.
- **Rejected.** Inferring it from `expect_passed` on the LShift key-down: indirect, and a reader cannot see it.

### D9: Regression tests keep their precondition explicitly

- **What.** Seven existing scenarios and two property configs used a modifier-only deferred binding *in order to* get a swallowed modifier (KS-06, KS-14 cases 1-3, the blocker case). They get `block_default: true`, so they keep testing what they were written for (green before and after). The pass-through path gets its own scenarios (commit 2).
- **Why.** Flipping them all to "passes" would silently drop the KS-14 regressions: those defects exist only for a swallowed modifier.
- **Rejected.** Rewriting their expectations.

## Win32 API choices

No new API and no new feature flag (`Cargo.toml:41-50` unchanged). The adapter `src/hook/keyboard_hook_callback.rs` is not touched. The existing APIs whose behaviour this lane relies on:

| Purpose | `windows` path | Feature (present?) | Thread | Errors | Doc link and quoted constraint |
|---|---|---|---|---|---|
| Drop or pass an event | `Win32::UI::WindowsAndMessaging::CallNextHookEx`, `LRESULT(1)` (`keyboard_hook_callback.rs:257-267`) | `Win32_UI_WindowsAndMessaging` (yes, `Cargo.toml:44`) | main (installing) thread | n/a | [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc): "it may return a nonzero value to prevent the system from passing the message to the rest of the hook chain or the target window procedure" |
| Resync of passed modifiers (a pass-through modifier is now `Passed`, so the resync reads it) | `Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState` (`keyboard_hook_callback.rs:126-132`) | `Win32_UI_Input_KeyboardAndMouse` (yes, `:45`) | main thread (and IPC worker for `reset`) | 0 on failure reads as up | [GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate): "The most significant bit of the return value is set if the specified key is currently down." [LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc): "the callback function is called before the asynchronous state of the key is updated". That is why the event's own key is skipped (`keys.rs:97-105`) |
| `ToggleCapsLock` fired from a pass-through LShift double tap is injected while Windows has Shift down | `Win32::UI::Input::KeyboardAndMouse::SendInput` (`system_function.rs:117-160`), sentinel unchanged | yes | main thread (hook callback, after the engine lock) | count checked (`:152-158`) | [SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput): "This function does not reset the keyboard's current state. Any keys that are already pressed when the function is called might interfere with the events that this function generates." KS-19, item 3.1. Manual check M1 |

- **`LLKHF_INJECTED`, the extended-key flag, `scanCode`.** Still ignored (`architecture.md:200`), so nothing changes for them.
- **AltGr.** Its fake LCtrl goes down before RAlt. A `"LCtrl"` deferred binding therefore arms on it with only Ctrl held. Before this lane it swallowed that fake LCtrl. Now it passes it, so AltGr characters work, and the Tap still fires on a quick AltGr press, as before. Recorded under Risks.
- **Threads.**
  - Loader resolution runs on the main thread (startup `config::load`) or on `config-watcher` (reload).
  - The engine gate runs on the main thread inside `with_engine`.
  - `hold-timer` threads and `on_timer` are unchanged.
  - No new thread, timer, window, COM apartment, lock, atomic or static.
- **Elevation, UIPI, pipe, tokens.** Unaffected. A passed modifier reaches every window exactly as an unbound key does.

## Touch set

| File | Commit | Change | Why |
|---|---|---|---|
| `src/data/key_combination.rs` | 1 | `pub fn is_modifier_only(&self) -> bool` with a doc comment; unit test `is_modifier_only_needs_every_key_to_be_a_modifier` | D2 |
| `src/engine/matching.rs` | 1 | `is_trigger_key` (`:58-63`) uses `combo.is_modifier_only()`; doc unchanged | D2, no behaviour change |
| `src/engine/scenarios.rs` | 2 | New section `// ---- block_default on deferred triggers (roadmap item 1.3) ----` with tests a-m3 and the harness self-test; `lshift_double_tap_fires_only_for_the_left_shift` (`:900-928`) flips its four LShift `expect_suppressed` (`:907`, `:910`, `:913`, `:916`) to `expect_passed`, with a new comment; `block_default: true` added to the configs at `:1343`, `:2053`, `:2079`, `:2110`, `:2199`, `:2739`, with comments | Red-first; D9 |
| `src/engine/testing.rs` | 2 | `expect_down_in_windows`; one sentence in the module doc (`:24-35` list of checks) | D8 |
| `src/engine/properties.rs` | 2 | `gestures` (`:97-104`): `block_default: true` on `Ctrl+Shift` Hold, new row `(keys: "F15", action: SwitchLanguage, trigger: Tap(200), block_default: false)`. `mixed` (`:126-135`): `block_default: true` on `LShift` DoubleTap. Module doc (`:1-25`) says both paths are covered | The invariant over both the pass and the swallow paths |
| `src/config/loader.rs` (tests) | 2 | Tests L1-L4; test helper `fn template_examples() -> Vec<Binding>` extracted from `default_config_commented_examples_load` (`:657-677`), which then uses it | Red-first |
| `src/config/schema.rs` | 3 | `:43-44` → `#[serde(default)] pub block_default: Option<bool>` with a doc comment (meaning, default, why `Option`, ignored by `sequence`/`chord`); delete `default_true` `:229-231` | Spec; D5 |
| `src/data/binding.rs` | 3 | `pub fn blocks_by_default(kind: &BindingKind, trigger: Trigger) -> bool` (associated fn, doc comment); field doc at `:53`; unit test `blocks_by_default_is_false_only_for_a_deferred_combo_made_of_modifiers` | D1, D2, D4 |
| `src/config/loader.rs` | 3 | `:385-386`: compute `raw.block_default.unwrap_or_else(\|\| Binding::blocks_by_default(&kind, trigger))` before `kind` moves into `with_kind`; pass it to `with_block_default` | D1 |
| `src/engine/mod.rs` | 3 | Gate at `:431`, `:445`, `:459`; rule 4 text `:120-128`; comment of the unit test at `:624-629` (the state is now reachable; points to scenario a) | D3 |
| `README.md` | 3 | `:340-347` example comment and caveat; `:379-383` behaviour rule plus three pitfall bullets (hotkey keys, lone Alt/Win, D6 consequences); `:463` flags row (default and meaning); `:666` feature bullet | Meaning change |
| `src/config/default_config.ron` | 3 | `:144-145` flag comment (new default, ASCII); `:268-275` LShift example comment: "every left-Shift press still reaches applications" | Meaning change; brief |
| `CHANGELOG.md` | 3 | Unreleased: **Changed** "**Breaking:** ..." entry, **Fixed** entry, KS-07 entry edit (`:231-234`); text below | Meaning change |
| `book/src/config/kinds.md` | 4 | §10: table row `:170` (honoured; default false for modifier-only); `:177` (applies to deferred too, gesture still detected); replay table row `:193`; practical rules `:197` | Config reference |
| `book/src/config/triggers-conditions.md` | 4 | Facts line `:3` (branch `feat/deferred-block-default`); §11.4 bullet `:64`; §11.5 "the keystroke is lost" at `:74` and `:78` (only with `block_default: true`); §11.8 warning `:126` → NOTE "fixed on `feat/deferred-block-default`" plus warnings for the hotkey keys, lone Alt/Win and D6 | Config reference |
| `book/src/config/structure.md` | 4 | Facts line `:8`; §2.2 warning `:63`; §5.1 row `:149`; §5.2 row `:182` | Config reference |
| `book/src/config/README.md` | 4 | Sketch line `:41` | Config reference |
| `book/src/config/errors.md` | 4 | §18.7 `:204` → "(`sequence`, `chord`)"; drop `:205` | Config reference |
| `book/src/config/example.md` | 4 | Comment at `:151` ("which works only for keys + Immediate"); facts line `:6`, `:18-19` | Config reference |
| `book/src/config/known-issues.md` | 4 | Row 52 (`:114`) and row 54 (`:122`) → **Fixed** (`feat/deferred-block-default`); new rows 65-68 (D7, lone Alt/Win KS-12, D6, KS-19); note `:137` | Pitfalls index |
| `book/src/config/ron-syntax.md` | 4 | §3.6 `:83-94`: `block_default` (and `repeat`, already `Option` since 1.1) take `Some`/`None`; remove `:91` | D5 |
| `book/src/config/actions.md` | 4 | `:248` "(the default)" → "(the default for every `Immediate` binding)" | Accuracy |
| Deferred to the Phase 1 book sync (doc-writer, `docs/book-phase-1`, WORKFLOW.md:103-107), not this lane | — | `book/src/architecture.md:207-208`, `:238`, `:336`; `glossary.md:85`; `overview.md:41`; `build-and-run.md:418`; `roadmap.md:9`, `:158`, 1.3 status block after `:259`, `:199` expectation, `:594` "Shipped" | Listed so nothing drifts |

New and changed RON blocks in commits 3-4 (README, template, book) are validated with the `cfgcheck` harness (`methodology.md:376-430`, built under `D:/tmp/ks-orch/p1-1.3-deferred-block-default/cfgcheck`). `default_config_commented_examples_load` covers the template. Build the book with `mdbook build book -d D:/tmp/ks-orch/p1-1.3-deferred-block-default/book` and check its links.

**CHANGELOG text (commit 3).**
- **Changed.** "**Breaking:** `block_default` applies to `Tap`, `Hold` and `DoubleTap` bindings (roadmap item 1.3); it was ignored there, and their key was always swallowed. With `block_default: false` the press, the auto-repeats and the release of the key reach applications, and the gesture still fires. Without the field, a `Tap`, `Hold` or `DoubleTap` binding made only of modifiers (`"LShift"`, `"Ctrl+Shift"`, `"Win"`) now lets its keys through. Every other binding, every `Immediate` binding included, still swallows its key. Write `block_default: true` for the old behaviour. With the new default, a deferred binding on exactly the keys of the Windows language hotkey, such as `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200) )`, lets Windows switch the layout as well. A `Tap` on `Alt` or `Win` alone lets Windows see the tap, which activates the menu bar or opens Start (until roadmap item 1.2). `block_default: Some(true)` and `block_default: None` (the default) load, as for `keys`."
- **Fixed.** "A `Tap`, `Hold` or `DoubleTap` on a modifier no longer swallows that modifier (roadmap item 1.3). With the README and template example `( keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250) )`, a double press of the left Shift toggles CapsLock, and the left Shift types capitals again."
- **KS-07 entry** (`:232-234`): replace "; until roadmap item 1.3 it swallows every left-Shift press." with ", and the left Shift keeps typing capitals (roadmap item 1.3)."

## Commits, in order

Each message ends with a blank line and `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. Gate after each commit: `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`, `cargo test --locked`. The only exception is commit 2, which is declared red.

**1. `Data: KeyCombination::is_modifier_only for the trigger-key rule (1.3)`.** Refactor; green.
- New unit test `data::key_combination::tests::is_modifier_only_needs_every_key_to_be_a_modifier`. Combos: `LShift` → true; `AnySide(Alt)+AnySide(Shift)` → true; `AnySide(Win)` → true; `Ctrl+RWin` → true; `Shift+CapsLock` → false; `CapsLock` → false; `LShift+F13` → false.
- `matching.rs:61` uses it.
- The existing matching tests (`matching.rs:143-177`) must stay green unchanged. They are the proof that behaviour did not change.

**2. `Engine: red-first: block_default on Tap, Hold and DoubleTap (1.3)`.** Declared red; compiles on master.

The harness method (D8), the `template_examples` helper and every test below are added here. "Red" means the predicted failure on today's code.

| # | Test (`engine::scenarios::` unless named) | Config / script | Predicted on master |
|---|---|---|---|
| a | `explicit_block_default_false_on_a_tap_passes_every_event_and_still_fires` | `(keys: "F13", action: SwitchLanguage, trigger: Tap(200), block_default: false)`; symmetry + stuck checks; down F13 passed; +80 up passed, fired `[SwitchLanguage]`; down; +500 repeat passed; up passed, nothing fired | RED: `down F13 at +0 ms: expected the event to be passed, it was suppressed` |
| b | `explicit_block_default_false_on_a_hold_passes_every_event_and_still_fires` | F13 `Hold(200)`, `block_default: false`; down passed; +200 fired `[SwitchLanguage]`; +300 repeat passed, nothing fired; up passed; down; +100 up passed; +300 nothing; total `[SwitchLanguage]` | RED: same message, F13 at +0 ms |
| c | `explicit_block_default_false_on_a_double_tap_passes_every_event_and_still_fires` | F13 `DoubleTap(250)` false, `ToggleCapsLock`; down passed; +60 up passed; +100 down passed, fired `[ToggleCapsLock]`; up passed | RED: same, F13 at +0 ms |
| d | `lshift_double_tap_fires_and_every_lshift_event_passes` (acceptance 1) | `(keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250))`; symmetry + stuck; down passed; +60 up passed; +100 down passed, fired `[ToggleCapsLock]`; +60 up passed; total `[ToggleCapsLock]` | RED: `down LShift at +0 ms: expected the event to be passed, it was suppressed` |
| e | `lshift_held_with_its_double_tap_bound_types_a_capital_a` (acceptance 2) | Same config; symmetry + stuck; down LShift passed; +30 down A passed, nothing fired, `expect_down_in_windows("LShift")`; +30 up A passed; +30 up LShift passed; total `[]` | RED: `down LShift at +0 ms: ...passed, it was suppressed` |
| f | `capslock_tap_with_the_default_block_default_still_swallows_capslock` (acceptance 3) | `(keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200))`; symmetry; down suppressed; +80 up suppressed, fired; down; +500 repeat suppressed; up suppressed, nothing | GREEN (guard) |
| g | `explicit_block_default_true_on_a_modifier_only_double_tap_swallows` (acceptance 4) | d's config plus `block_default: true`; every LShift event suppressed; the second down fires `[ToggleCapsLock]`; symmetry + stuck | GREEN (guard) |
| h | `passing_lshift_double_tap_held_past_the_repeat_delay_passes_its_repeats_too` | d's config; symmetry + stuck; down passed; +500 repeat passed; +33 repeat passed; up passed; total `[]` | RED: LShift at +0 ms |
| i | `one_long_press_of_a_passing_shift_does_not_fire_its_double_tap` (KS-14 case 3, pass path) | `(keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600))`; stuck; down A; +10 down LShift passed; +10 up A passed; +480 repeat LShift passed, nothing; +33 repeat passed, nothing; up passed; total `[]` | RED: `down LShift at +10 ms: ...` |
| j | `modifier_only_hold_and_tap_pass_their_keys_by_default` | `(keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)), (keys: "RShift", action: ToggleCapsLock, trigger: Tap(300))`; symmetry + stuck; down LCtrl passed; +20 down LShift passed; +1000 fired `[SwitchLanguage]`; up LShift passed; up LCtrl passed; +100 down RShift passed; +50 up RShift passed, fired `[ToggleCapsLock]` | RED: `down LShift at +20 ms: ...` |
| k | `deferred_binding_on_the_language_hotkey_keys_lets_the_hotkey_through_by_default` | `Scenario::from_ron_with_language_hotkey(.., &[VK_MENU, VK_SHIFT])`: `(keys: "CapsLock", action: SwitchLanguage, block_original_combo: true), (keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200))`; down LAlt passed; +20 down LShift passed; +50 up LShift passed, fired `[SwitchLanguageBackward]`; up LAlt passed. Second scenario with `block_default: true`: LShift down and up suppressed, the up fires | RED: `down LShift at +20 ms: ...` (second part green) |
| l | `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` | `(keys: "F13", action: SwitchLanguage, trigger: Tap(200), block_default: false), (keys: "F13", action: ToggleCapsLock, trigger: Hold(200))`; symmetry; down suppressed; +50 up suppressed, fired `[SwitchLanguage]`; comment: OR rule until 1.6 (KS-13) | GREEN (guard) |
| m1 | `passing_tap_fires_after_a_foreground_change_and_its_key_up_passes` | a's config; stuck; down passed; `foreground_changed`; +50 up passed, fired `[SwitchLanguage]` | RED: F13 at +0 ms |
| m2 | `remapping_off_during_a_passing_hold_drops_it_and_nothing_sticks` | b's config; stuck; down passed; +50 `remapping_off`; +300 nothing fired; up passed; `remapping_on`; down passed; +200 fired `[SwitchLanguage]`; up passed | RED: F13 at +0 ms |
| m3 | `reload_during_a_passing_double_tap_keeps_each_press_whole` | d's config; symmetry + stuck; down passed; +60 up passed; `reload(d + block_default: true)`; +40 down suppressed, fired `[ToggleCapsLock]` (the first press's entry); up suppressed | RED: LShift at +0 ms |
| s | `down_in_windows_check_reports_a_key_windows_does_not_have_down` (harness self-test, `#[should_panic(expected = "is not down in Windows")]`) | `(keys: "CapsLock", action: SwitchLanguage)`; down CapsLock; `expect_down_in_windows("CapsLock")` | GREEN (panics as expected) |
| flip | `lshift_double_tap_fires_only_for_the_left_shift` (`:900-928`), four LShift lines → `expect_passed` | as today | RED: `down LShift at +0 ms: ...` |
| D9 | `block_default: true` added to `:1343`, `:2053`, `:2079`, `:2110`, `:2199`, `:2739` | unchanged scripts | GREEN (no-op on master) |
| P | `engine::properties::*` (3 tests) with the edited configs | — | GREEN (master ignores the flag; F15 swallows) |
| L1 | `config::loader::tests::block_default_defaults_to_false_only_for_deferred_bindings_made_only_of_modifiers` | `from_ron_str_with_hotkeys(.., &\|_: &BindAction\| None)` over rows `LShift`+DoubleTap(250), `Ctrl+Shift`+Hold(1000), `Win`+Tap(200), `RShift`+Tap(300), `CapsLock`+Tap(200), `Shift+CapsLock`+Hold(200), `LShift` Immediate, `Alt+Shift` Immediate, `sequence ["F13","F14"]`, `chord ["F13","F14"]`; `assert_eq!` of the `block_default` vector against `[false, false, false, false, true, true, true, true, true, true]` | RED: `assert_eq!` left `[true, true, true, true, true, true, true, true, true, true]` |
| L2 | `config::loader::tests::an_explicit_block_default_wins_over_its_default` | `LShift` DoubleTap `true`; `F13` Tap `false`; `Ctrl+Shift` Hold `false`; `LShift` Immediate `false`; `F14` Hold `true` → `[true, false, false, false, true]` | GREEN (guard) |
| L3 | `config::loader::tests::block_default_takes_some_and_none_like_the_other_optional_fields` | `(keys: "F13", action: DoNothing, block_default: Some(false))`, `(keys: "LShift", action: DoNothing, trigger: DoubleTap(250), block_default: None)`, `(keys: "F14", action: DoNothing, block_default: None)` → `[false, false, true]`; load with `unwrap_or_else(\|e\| panic!("block_default takes Some and None: {e}"))` | RED: panic containing `Expected boolean` (as `ron-syntax.md:91` documents) |
| L4 | `config::loader::tests::the_template_lshift_double_tap_example_lets_the_left_shift_through` | `template_examples()`; find the binding with `combination() == Some(&KeyCombination::new(VK_LSHIFT))` and `trigger == Trigger::DoubleTap(250)`; `assert!(!example.block_default)` | RED: `assertion failed: !example.block_default` |

The commit body lists the red set by name: a, b, c, d, e, h, i, j, k, m1, m2, m3, flip, L1, L3, L4. The developer replays it on `master` and pastes the first failing line of each into `impl.md`.

**3. `Engine: block_default applies to Tap, Hold and DoubleTap, off by default for modifier-only combos (1.3)`.** Makes commit 2 green.
- Touches `schema.rs`, `binding.rs` (plus unit test), `loader.rs`, `engine/mod.rs`, `README.md`, `default_config.ron`, `CHANGELOG.md`.
- Invariants kept:
  - The key-up rule (`mod.rs:562-570`) and the repeat rule (`:510-542`) are untouched.
  - The blocker branch (`:385-397`) does not read `block_default`.
  - Sort and publish are unchanged (R7).
  - No change in `src/hook`.
- Clippy: `default_true` is removed in the same commit.

**4. `Book: block_default on deferred triggers in the config reference (1.3)`.** Docs only; green.
- The config pages of the touch set.
- RON validated with cfgcheck; `mdbook build` into the lane directory, links checked.

Bisect: commit 1 is green. Commit 2 is red only for the listed tests. Commits 3 and 4 are green. README, the template and CHANGELOG change in commit 3, the commit that changes the meaning.

## Acceptance mapping

| Acceptance line (brief / `roadmap.md:251-259`) | Covered by |
|---|---|
| A double LShift with DoubleTap(250) fires ToggleCapsLock, and every LShift event passes | d; flipped `lshift_double_tap_fires_only_for_the_left_shift`; M1 |
| With LShift held, A passes together with Shift and nothing is swallowed ("LShift + A types A") | e (`expect_down_in_windows("LShift")` at A's key-down, symmetry, stuck); h; M1 |
| A CapsLock Tap with the default block_default still swallows CapsLock | f; L1 (`CapsLock`+Tap row → `true`) |
| An explicit `block_default: true` on a modifier-only DoubleTap swallows | g; L2 |
| `block_default` becomes `Option<bool>` in the schema | L1, L3 |
| Applies to Tap, Hold and DoubleTap; with `false` every event passes while the gesture is detected | a, b, c, j, m1, m2 |
| Default false only for combos made only of modifiers; true with a non-modifier key | L1; `blocks_by_default_is_false_only_for_a_deferred_combo_made_of_modifiers`; `is_modifier_only_needs_every_key_to_be_a_modifier` |
| Suppression invariant holds (key-up suppressed iff key-down was) | `check_press_symmetry` in a-e, g, h, j, l, m3; `engine::properties::every_press_is_suppressed_or_passed_as_a_whole`, `nothing_is_left_down_in_windows_across_focus_changes_and_lost_key_ups` (configs now hold F15 pass-through, LShift/RShift pass-through and explicit swallowing rows); `engine::tests::key_up_of_a_key_whose_down_passed_passes_even_with_a_tap_pending` |
| Immediate semantics (default true) unchanged | L1 (`LShift`, `Alt+Shift` Immediate → `true`); existing `block_default_false_fires_and_lets_both_edges_through` (`scenarios.rs:144-155`), `immediate_single_key_fires_on_down_and_swallows_both_edges` (`:45-56`), `parse_simple_binding` (`loader.rs:705`) |
| Template marks the LShift DoubleTap example as working | L4; `default_config_commented_examples_load` |
| Reload, `swch off`/`on`, foreground change during a pending pass-through gesture | m3, m2, m1 |
| Checkpoint 1 smoke #6 (`roadmap.md:310`) | M1 |

## Manual checks (human only)

No `#[ignore]` test is added. Every acceptance line can be expressed through the engine and is covered above. These checks observe how Windows reacts, which no test here can do. They map to Checkpoint 1 smoke #6 (`roadmap.md:310`) and to the new pitfall rows.

| # | Setup | Steps | Expected | Ignored test |
|---|---|---|---|---|
| M1 | Release build, `swch open` from a normal console. Uncomment the template's `LShift` `DoubleTap(250)` example. Notepad and `osk.exe` open. Record `reg query "HKCU\Keyboard Layout" /v Attributes` (read only), which tells whether Shift turns CapsLock off | Hold LShift and type `abc`; LShift+A quickly; double-tap LShift, type `a`; double-tap LShift again, type `a`; double-tap RShift; hold LShift 2 s and release | `ABC`; `A`; CapsLock turns on and `a` gives `A`; CapsLock turns off and `a` gives `a`; RShift toggles nothing; `osk.exe` shows no Shift stuck. If the toggle misbehaves with Shift held (KS-19), note the `Attributes` value | none (smoke #6) |
| M2 | Default config with the Alt+Shift Windows hotkey, plus `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200) )` | Read the startup lines; tap Alt+Shift; add `block_default: true`, save, tap again | `[INFO] no auto-blocker for Alt+Shift: binding #2 uses the same keys`; first tap: the layout ends where it started (Windows forward, binding backward); second tap: one switch backward | none (pitfall row 65) |
| M3 | `( keys: "LWin", action: ToggleCapsLock, trigger: Tap(200) )` | Tap LWin; then add `block_default: true` and tap LWin, then press LWin+E | First: Start opens and CapsLock toggles. Second: CapsLock toggles and Start does not open, but LWin+E types `e` (LWin is swallowed while the binding matches; item 1.2) | none (pitfall row 66) |

## Risks and edge cases

**R1, hook callback and engine.**
- The callback is unchanged.
- The engine adds one bool test per matching deferred binding. No allocation, no `format!`, no new lock. All indices stay inside `keys.rs` `slot()` (`< 256`).
- Every path still ends in exactly one of `LRESULT(1)` or `CallNextHookEx`.

**R2, the sentinel.** No new `SendInput` call.
- Interplay: `ToggleCapsLock` from a pass-through LShift double tap runs in `carry_out` before the LShift key-down is passed on. Windows therefore has Shift down when the injected CapsLock arrives, and may treat it as Shift+CapsLock (KS-19, item 3.1; `SendInput` doc quote above). Behaviour is unverified and checked in M1.
- If a user reports a wrong toggle under the "Press SHIFT to turn off" setting, the remedy is `block_default: true` or item 3.1.

**R3, shared state; R4, atomics.** Nothing new. The engine state stays plain data under `Mutex<Engine>`. No `static`, no atomic.

**R5, builders.**
- `blocks_by_default` is an associated fn, not a setter.
- `with_block_default` is unchanged.
- Constructors keep two arguments (D4).

**R6, Display.** No new variant. `Binding` Display is unchanged (non-goal). `ConfigError` is unchanged.

**R7, specificity.** Sort and publish are untouched. The OR suppression across matching bindings is unchanged in kind (test l pins symmetry under OR). 1.6 will change it.

**R8, key identity.**
- `is_modifier_only` uses `KeySpec::modifier()`, so `Shift` (AnySide), `LShift`, `VK_SHIFT` and both Win keys count alike, in combos.
- Sequence and chord are unaffected: they ignore the flag.
- An unsided `"Shift"` DoubleTap keys its entry by event code. LShift followed by RShift is not a double tap, as today (`gestures.rs:207`).
- The R8 text in `code-reviewer.md` still describes the old `active_keys` folding. Orchestrator follow-up F6.

**R9, registry.** Tests that need a blocker use `from_ron_with_language_hotkey` or `from_ron_str_with_hotkeys`, never the registry. M1 reads `HKCU\Keyboard Layout` and never writes it.

**R10, message loop and shutdown.** Nothing new.
- At exit with a pass-through key held, Windows already has the key down and gets its key-up after the unhook. This is safer than a swallowed key, which reaches Windows as a lone key-up.

**Repeats.** A passed key-down is `Passed`, so its repeats pass (`mod.rs:530-531`). Deferred arms skip repeats (`:422`, `:434`, `:448`). Tests h and i.

**Sided modifiers and KS-14.** A passed modifier is resynced from `GetAsyncKeyState` like any unbound modifier (`keys.rs:97-105`). Its lost key-up (Secure Desktop) is therefore repaired, unlike a swallowed one. Test i covers the KS-14 case 3 timing on the pass path. The swallow-path regressions keep `block_default: true` (D9).

**Reload mid-gesture.** The pending entry keeps its old action. The key-up follows the record, not the new binding's flag (m3).

**`swch off`/`on`.** `reset` drops gestures and marks held keys `Passed` (m2).

**Foreground change.** `Passed` survives `forget_blocked` (m1).

**UIPI and elevation.** A pass-through modifier reaches elevated and medium windows like any key. No change.

**IME and layouts.**
- AltGr's fake LCtrl now passes for a `"LCtrl"` deferred binding, so AltGr characters work. A quick AltGr press still fires that binding's Tap, as before.
- Other injectors' events (`LLKHF_INJECTED`) are matched like physical ones, unchanged.

**Typing with a pass-through modifier gesture (D6).** Fast shifted capitals can fire an `LShift` Tap or DoubleTap. This is documented as pitfall row 67 and tracked in Q1.

**Windows hotkey keys (D7).** A modifier-only deferred binding on Alt+Shift or Ctrl+Shift lets Windows act on the hotkey too. Breaking, documented, pinned by k and checked in M2.

**Lone Alt or Win (KS-12).** A `Tap` on Alt or Win alone now lets the tap reach Windows (menu bar, Start). Documented and checked in M3. Item 1.2's mask key fixes it.

**Follow-ups (not this lane).**
- F1: the interrupt rule (Q1).
- F2: the loader accepts and ignores `block_default` on `sequence`/`chord` (`errors.md:204`; no KS id).
- F3: KS-19 (3.1).
- F4: KS-12 (1.2) for pass-through Alt/Win gestures.
- F5: KS-13 (1.6), OR suppression.
- F6: stale R8 text in `.claude/agents/code-reviewer.md`.

## Questions for the orchestrator

1. **Should a pass-through modifier gesture be cancelled when another key goes down while its key is held or between its two presses?** That is AutoHotkey's `A_PriorKey` idiom, and it would stop Shift+A from also firing an `LShift` Tap. Recommendation: not in 1.3. It changes gesture semantics beyond the spec and contradicts a pinned scenario (`scenarios.rs:1044-1067`). Record it as a follow-up for item 2.2, whose "one mutually exclusive state machine" (`roadmap.md:360`) is the natural home, and ask the owner whether it should become part of 2.2's acceptance. Non-blocking: 1.3's acceptance holds without it.
2. **Should the loader warn when a binding that does not block replaces the auto-blocker for the Windows hotkey (D7)?** Recommendation: no code in 1.3. Document it as Breaking (CHANGELOG, README pitfall, known-issues row 65, M2) and let 1.6 handle it, since that item already adds loader warnings for conflicting bindings (`roadmap.md:283`). It is a console-message change in `loader.rs:303` that the book and manual check 10 (`roadmap.md:203`) quote. Non-blocking.
3. **Research, optional, non-blocking.** Query for `researcher`: "Windows 10/11: with VK_SHIFT physically down, does a SendInput VK_CAPITAL press and release toggle CapsLock under the default 'Press the CAPS LOCK key' setting, and under KLF_SHIFTLOCK ('Press the SHIFT key', `HKCU\Keyboard Layout\Attributes` 0x00010000)? Microsoft Learn or another authoritative source." Recommendation: run it only if M1 shows a wrong toggle. The design does not depend on the answer.

## Readiness checklist

- [x] Every cited KS issue read: none cited by the item. KS-07 (smoke #6) read, and its scenario is test d plus M1. KS-12, KS-13, KS-14 and KS-19 are referenced in the risks.
- [x] Every book reference re-located (table above). None is gone. The defect is still present at `mod.rs:431/445/459`.
- [x] Goal, non-goals and dependants stated.
- [x] Every decision has a rationale and the alternatives it rejected (D1-D9).
- [x] Owner decisions respected: exact modifiers untouched; repeat defaults untouched (deferred arms still skip repeats); RON; no logging change (only existing `println!` at `loader.rs:303`); licence untouched.
- [x] Thread named for every changed path (loader: main and `config-watcher`; engine: main inside `with_engine`). Nothing new in the LL callback.
- [x] Install, uninstall and timers: no change; Hold timers unchanged until 2.2.
- [x] No new `SendInput` path. The existing ToggleCapsLock interplay is noted (M1).
- [x] Sided and unsided handled through `KeySpec::modifier()`. Chord and sequence unaffected (they ignore the flag).
- [x] Reload, `swch off`/`on`, foreground change and shutdown during a pending pass-through gesture covered (m1-m3, R10).
- [x] No new shared state, atomics or `static mut`.
- [x] No new module cycle (`data` gains a method; `config` → `data` exists). Logic sits in `engine` and `data`, reachable by scenarios.
- [x] Win32: no new API; existing ones listed with paths, features (present), threads and doc quotes.
- [x] Elevation, UIPI, pipe DACL and token inheritance considered: unaffected.
- [x] Red-first with exact predicted failures (commit 2 table).
- [x] Tests use `Engine::on_key`/`on_timer` through `Scenario`, a fake `Env` and a fake clock. Nothing installs a hook or injects input.
- [x] No ignored test needed. Manual checks M1-M3 are listed with the reason.
- [x] Every commit passes the gate except the declared red commit 2. `default_true` is removed with its last use.
- [x] Bisectable order. README, template and CHANGELOG are in commit 3, the commit that changes the meaning.
- [x] Every acceptance line mapped.
- [x] Every question has a recommendation.

## Developer addendum: critique response (round 1)

Branch `feat/deferred-block-default`, commits `5adf19c`..`1562acd`. Line numbers are at `1562acd`.

- **I1 (Important), RESOLVED by option (b).** The interrupt rule stays out of the lane, as D6 decides: choosing option (a) would change gesture semantics beyond the cut, which a developer may not decide. The artefacts now match the behaviour that ships:
  - Template caveat at the example itself: `src/config/default_config.ron:273-278` (another left-Shift press within 250 ms for two capitals, `<T>`, toggles CapsLock; `block_default: true` keeps Shift for the gesture alone).
  - CHANGELOG: sub-bullet of the Breaking `block_default` entry under **Changed**, and the **Fixed** entry says that a quick second Shift press toggles too.
  - README: the caveat under the trigger example and a behaviour-rule bullet (`<T>`, `Shift+A` Tap, quick Ctrl+C).
  - Book: `triggers-conditions.md` §11.8 warning "Another key does not cancel a gesture whose key passes" (the `<T>`, Ctrl+C and Ctrl+Shift-select cases), `kinds.md` §10 practical rules, and `known-issues.md` row 67.
  - Pinned known-issue scenarios, red-first ready for the follow-up: `engine::scenarios::a_key_typed_between_two_passing_lshift_presses_still_completes_the_double_tap` (`src/engine/scenarios.rs:3341`) and `a_quick_ctrl_shortcut_still_fires_a_passing_ctrl_tap` (`:3376`). Each has a comment: "Known issue, pinned until an interrupt rule is decided (a follow-up of roadmap item 1.3)".
  - M1 gains the steps `<T>` and `A B` with the left Shift pressed again quickly (impl.md, Manual checks).
  - Still open, for the orchestrator to put to the owner: should the interrupt rule become an acceptance line of 2.2 or a new item?
- **M1 (Minor), RESOLVED by the documentation option.** That exact case is named in the CHANGELOG (Breaking entry, sub-bullet), the README (behaviour-rule bullet), `known-issues.md` row 65, `matching.md` §15.3 (now in commit 4) and `triggers-conditions.md` §11.8. It is pinned by the second config of `deferred_binding_on_the_language_hotkey_keys_lets_the_hotkey_through_by_default` (`src/engine/scenarios.rs:3179`), where the binding itself sets `block_original_combo: true`. The alternative was to resolve a missing `block_default` to `true` on a binding that sets `block_original_combo: true`. It was not taken: it adds a second rule to D2's single default, which is a design change outside the cut. If it is wanted, it is one condition in `raw_binding_to_binding` (`src/config/loader.rs:385-387`) plus flipping that scenario part.
- **M2 (Minor), RESOLVED.** The KS-13 text now covers deferred bindings in these places: `kinds.md` §10 (table rows and the KS-13 warning, with the `F13` Tap-false plus Hold example), `known-issues.md` row 10, `triggers-conditions.md` §11.3, `actions.md` §13.9 (the KS-13 warning) and `matching.md` §16.2. The behaviour is pinned by `passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole` (`src/engine/scenarios.rs:3238`).
- **Open question 1 (merge order against 1.2):** a phase-plan question for the orchestrator. Nothing is changed in this lane. The interim behaviour is documented: CHANGELOG, README, `known-issues.md` row 66, §11.8.
- **Open question 2 (M3 wording), RESOLVED.** Manual check M3 in impl.md now says that `block_default: true` itself swallows LWin while the binding matches, and that item 1.2 does not change that. The README bullet, §11.8 and row 66 say the same.
