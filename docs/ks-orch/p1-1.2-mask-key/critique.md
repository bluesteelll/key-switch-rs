# Architecture critique: p1-1.2-mask-key / roadmap 1.2 (KS-12, rest of KS-05, KS-35) (round 1)
Verdict: APPROVED (Critical 0, Important 0, Minor 2)
Plan: D:/tmp/ks-orch/p1-1.2-mask-key/cut.md   Code read at: master `ece1767` (`.git/refs/heads/master` = ece17674ca6f69c8b639a4698192ce050e8c3364)

No Critical or Important findings. I walked sections 1-11 against the cut and the tree, and I checked every load-bearing claim about current behaviour and about ron 0.10.1 against the source (details under "Verified claims"). Two Minor findings, both about documentation and traceability, and four open questions. Each open question names the check that would settle it.

## Critical (implementation must not start)
None.

## Important (must be resolved; options may be discussed)
None.

## Minor (improvements, not blockers)

### M1. Config pages and book pages that will state KS-12 as current are missing from the touch set
Where: cut §Touch set (rows `book/src/config/triggers-conditions.md`, `book/src/config/example.md`, "Not touched"); `book/src/config/triggers-conditions.md:135`; `book/src/config/example.md:33-35`; `book/src/build-and-run.md:415`, `:457-465`; `book/src/glossary.md:141`, `:313`
Problem: The touch set changes only `triggers-conditions.md:222` and §12.4, and only `example.md:234`. It leaves out:
- §11.8 "Lone Win or Alt (KS-12)" at `triggers-conditions.md:135`. Its first sentence says that for `Win+...`/`Alt+...` with a deferred trigger "only the trigger key is suppressed, so releasing Win or Alt can open the Start menu or the menu bar (item 1.2)". Known-issues row 31 links to §11.8.
- The "Deliberately NOT used: Win+X / Alt+X ... opens the Start menu or the menu bar" comment in the example config at `example.md:33-35`.

The "Not touched" list names `architecture`, `audit`, `roadmap` and `overview` for the phase book sync. It does not name:
- `build-and-run.md`: the troubleshooting row at :415 ("keep Win and Alt out of such combos"), and §"The Start menu or a menu bar opens (KS-12)" at :457-465 ("no mask key is injected").
- `glossary.md`: :313 ("key-switch-rs has no mask key yet"), and :141 ("a `mask` flag reserved for roadmap item 1.2").

Consequence: After the merge, `book.yml` publishes a config reference that tells users two things. First, `( keys: "Win+Q", action: PostMessage(msg: "WM_CLOSE") )` opens Start on the Win release. Second, Win/Alt combos must be avoided. The same push's CHANGELOG **Fixed** KS-12 entry says the opposite. CLAUDE.md ("A developer whose change alters config semantics updates the matching `book/src/config/*.md` page ... in the same lane") covers the two config pages.
Confidence: CONFIRMED (grep of `book/src` for `KS-12|Start menu|menu bar|mask`; none of the lines above is in the touch set).
Breaks: CLAUDE.md docs rule; traceability (section 11).
What is needed:
- Add `triggers-conditions.md` §11.8 (:135) and `example.md:33-35` to commit 9. Keep the part of :135 that stays true: a `Tap`/`Hold`/`DoubleTap` on lone Alt/Win still opens the menu or Start (row 66).
- Name `build-and-run.md` and `glossary.md` in the hand-off to the phase book sync, next to `architecture.md`.

### M2. Checkpoint-1 smoke item 4 cannot pass with the shipped config after this lane, and the cut does not say so
Where: cut §Goal 2, §Non-goals ("The default stays `true` ..."), §Dependents ("Checkpoint-1 smoke items 4 and 5"); `book/src/roadmap.md:308`; `src/engine/scenarios.rs:2603-2629` (`shift_pressed_after_tab_was_released_is_still_swallowed`), `:2579-2600`
Problem: Checkpoint-1 item 4 reads "`Ctrl+Shift+T` and `Alt+Shift+Tab` work; a bare Alt+Shift does not switch the language (A, 1.2)". It is run on the release build with the template. The brief keeps `true` (swallow plus mask) as the default. The cut does say that only `MaskOnly` fixes the rest of KS-05. It still lists item 4 as a dependent that this lane serves, and it does not say that item 4 passes only with `MaskOnly` opted in.
Consequence: Under the default config and the default Alt+Shift hotkey, pressing Alt, Shift, Tab still swallows Shift, as `scenarios.rs:2603-2629` pins. The switcher therefore moves forward, so "Alt+Shift+Tab works" fails. With the Ctrl+Shift hotkey, pressing Ctrl, Shift, T arrives as Ctrl+T (`scenarios.rs:2579-2600`). At the Phase-1 checkpoint the owner will either record item 4 as failed against a lane reported as done, or mark it passed by mistake.
Confidence: CONFIRMED.
Breaks: traceability to roadmap acceptance (section 11).
What is needed: State in the cut, and later in `results.md`, that checkpoint item 4 is met only with `block_original_combo: MaskOnly` (M4/M5). Hand the rewording of item 4 to the phase book sync: "with `MaskOnly`" until M4/M5 decide the default.

## Open questions for the architect

1. **The mask while the Alt+Tab switcher is open (PLAUSIBLE, no proven consequence).**
   - With the template's `true` blocker, the common habit Alt held, Tab tapped, then Shift now injects E8 with Alt held while the switcher is on screen. The trigger is that Shift is swallowed while LAlt passed (D1; `scenarios.rs:2603-2629`).
   - AutoHotkey's field record does not cover this case. Its documented masking is "the next Alt or Win keyup" (https://www.autohotkey.com/docs/v2/lib/_MenuMaskKey.htm), so AutoHotkey never sends the mask into an open switcher.
   - Manual check M3 covers it, but the cut gives no fallback if M3 fails. If the switcher commits or closes on an unhandled key, every default-config user loses the way back with Shift. Today that path at least "moves forward".
   - Settling check: M3 with the release build. Please state what follows a failure: RETHINK at results, or a narrower rule for the default mode.

2. **D10 wording.** "The re-entry cannot reach the engine" holds for the sentinel-tagged E8 events: `decide_event` returns at `keyboard_hook_callback.rs:166-168` before the lock.
   - Microsoft does not document whether SendInput dispatches other callbacks to this thread while it waits on another process's LL hook, for example `foreground_changed`. If it does, that callback finds the lock free, because `carry_out` runs after `with_engine` returned. It then does only what any foreground change does: `forget_blocked`, then a resync.
   - So there is no deadlock and no new hazard. Still, the module doc should say "no event of ours reaches the engine; any other callback finds the lock free" rather than imply nothing can run.
   - Settling check: none needed for correctness; this is doc precision.

3. **Fail-safety of H2.** H2 calls the production `carry_out` with `mask: true` on purpose. The only barrier between `cargo test` and a real `SendInput` is the `#[cfg(test)]`/`#[cfg(not(test))]` pair on `send_menu_mask`.
   - If that seam ever regresses (a typo, or a later "simplification"), every gate run injects E8 once before H2 fails.
   - E8 is benign, but the agents' rule is absolute.
   - Settling check: the code-reviewer verifies the seam exactly as the cut's §Risks "Safety seam" already asks. Consider writing that in the cut as an explicit review checklist line.

4. **Q1 and Q2 (orchestrator questions).** I agree with both recommendations.
   - Keep the owed mask (D5). Under AutoHotkey's "injected before the event in the hook" model, it is the only mask that lands between Shift-down and the release. The match-time mask still satisfies the brief's literal "on an exact match only injects the mask".
   - Mask per swallowed event. The spec's "exactly when" requires it, and the cost is bounded by the repeat rate.

## Verified claims (what I checked, so the developer need not re-check)

- **D1 vs the tree.**
  - `sync_modifiers` (`keys.rs:97-105`) rewrites `active` from `os_key_down` for every modifier code except the event's key and swallowed records. The list includes `VK_MENU`, `VK_LMENU`, `VK_RMENU`, `VK_LWIN` and `VK_RWIN` (`keys.rs:25-37`).
  - On a key-down the record update comes at `mod.rs:536-568`; on a key-up, `release`/`take_blocked` at `mod.rs:574-596`. So "active and not Suppressed/Forgotten", computed after the update, equals "down in Windows" for every Alt/Win code. That holds for:
    - an Alt/Win event key that is itself swallowed (now `Suppressed`);
    - one being released (now inactive);
    - the generic `VK_MENU` slot, which never carries a record and is synced from the OS (`FakeEnv` models it at `testing.rs:96`);
    - an unmarked repeat of a modifier the resync found held.
  - `mark_completed` touches only non-modifier codes for sequences (`sequences.rs:128-142`), so it cannot turn a held Alt `Suppressed` behind the OS's back. The property oracle (D9) agrees with the engine on every script shape the generator produces, Focus mode included: `up_unseen`, then a foreground change, then a re-press.
- **D2.** `carry_out` (`keyboard_hook_callback.rs:187-195`) runs after `decide_event`, and the snapshot and lock are dropped at `:170-182`. `carry_out` also runs on `hold-timer` (`:209-211`), but `on_timer` never sets `mask` (`mod.rs:333-339`), and D9's timer assert pins that. The extern-callback test (`:401-461`) is the only existing path from a test into `carry_out`. `tests/ipc_roundtrip.rs` is the only integration test.
- **D4 (backward compatibility of `true`/`false`).** ron 0.10.1:
  - `deserialize_enum` → `Enum::variant_seed` → `deserialize_identifier` → `Parser::identifier` (`de/mod.rs:731-766`, `:912-925`; `parse.rs:883-953`). The identifier path accepts `true`/`false` with no keyword check.
  - `NoSuchEnumVariant` and `OneOf` (`error.rs:212-237`, `:443-465`) produce exactly ``expected one of `true`, `false`, or `MaskOnly` instead``. `Identifier` prints `true` without `r#` (`:469-482`).
  - Existing `block_original_combo: true|false` configs therefore keep loading. The only text change is the error for `Some(true)`, which the cut documents (`ron-syntax.md:93`). `IMPLICIT_SOME` (`loader.rs:264-265`) does not touch the enum path.
- **D8.** The blocker's condition is evaluated before the match today, at `mod.rs:409`. With inherited conditions, that would capture the foreground on every key-down (KS-08, against `mod.rs:644-666`). E1 is red for the right reason at commit 7: `ForegroundForbiddenEnv` panics in the pre-D8 order (`mod.rs:627-642`).
- **Red-firsts.**
  - S1-S4, S8-S11 and the property run are red at commit 1, because `mask` is never set.
  - S5-S7 and S12 are green guards. Each kills a named mutation:
    - S5 kills `mask = suppress`;
    - S6 kills "count Win/Alt whatever its record". With Shift first and then Alt, the blocker swallows Alt, and the mutant would mask;
    - S7 kills "mask without suppress";
    - S12 kills "mask in `on_timer`".
  - The commit-5 reds come from ron's `Expected boolean` on a `bool` field (`schema.rs:52-53`).
  - L7, L8 and S13 are red because `new_auto_blocker` hard-codes `Always` (`binding.rs:131-143`, `loader.rs:357`).
- **Re-locations.** I checked every row of §Spec re-located, and each one matches the tree at `ece1767`.
- **Win32.**
  - SendInput inserts "serially into the keyboard or mouse input stream" (Microsoft Learn, SendInput). The LL-hook timeout and silent removal are stated as cited.
  - AutoHotkey's `#MenuMaskKey` page confirms: the default `vk11sc01D`; vkE8 as a good candidate, "documented as unassigned"; "Microsoft can assign an effect to an unassigned key code at any time". Its warning about zero VK with non-zero SC does not apply to vkE8 with scan 0.

## Strengths to keep
- The mask is decided in the engine from records the resync already keeps, with no new `Env` call. It is pinned by a property oracle that reads the modelled OS instead of the engine's own records.
- The `cfg(test)` seam on the one Win32 side effect is motivated by a concrete hazard: `cargo test` while the owner holds Win.
- The mask is sent outside the lock, before the timers and actions. `carry_out` destructures `Decision` with no `..`, so a future field fails to compile.
- D4 is backed by reading ron's source rather than assuming. The untagged alternative is rejected for a concrete, checked reason.
- D8 moves the blocker's `when:` after the match, which stops KS-35's fix from becoming a KS-08 regression.
- v2 ships opt-in and experimental, with the double-mask hedge reasoned against both injection-order models and a clear failure path: if M4 or M5 fails, the default stays `true`.
- Clean red/green alternation that bisects; docs land with the commits that change meaning.

## Absent topics (walked, not applicable)
- **Section 2, thread affinity.** No hook is installed, uninstalled or moved, and no thread is added. `SendInput` runs on the main thread, which already does it for `PressKey` (`windows_actions.rs:190-229`). Nothing to flag.
- **Section 3, matching model.** `blocker_matches` and the election are unchanged. Exact modifiers and `allow_extra_modifiers` are untouched. The repeat policy is untouched. Nothing to flag.
- **Section 7, reload leniency, BOM, unknown fields.** Unchanged. The loader still runs on the main thread or `config-watcher`. KS-17 (value "3") stays with item 1.4 as the cut says.
- **Section 8, atomics.** None are added. `mask_owed` lives inside `Mutex<Engine>`, and `MASKS_SENT` is a `cfg(test)` thread-local.
- **Section 9, process boundary.** Not touched.
- **Section 10, builder count.** A fourth `SendInput` builder is added (`menu_mask_inputs`). It is justified: H1 pins the sentinel without injecting, and item 3.1 absorbs it. It carries `injected_sentinel()` and checks the returned count.
