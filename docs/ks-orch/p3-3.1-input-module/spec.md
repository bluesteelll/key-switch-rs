# Lane spec: p3-3.1-input-module - One input module (roadmap item 3.1, part 1 of 2; KS-36)

Branch: fix/input-module

Roadmap item 3.1 (book/src/roadmap.md 'Single input module', :674-684 at 252daef), the builder half. This lane does not
change which modifiers an output is sent with; that is p3-3.1-held-modifiers. Binding: lowest key-to-action latency; no
key may ever stick; every synthesized event carries `injected_sentinel()` (CLAUDE.md principle 3; 2.5 D2 tracks the
daemon's own modifiers through it, p2-2.5-latency-trim/results.md:264-266). Scope report:
<orch>/phase-3/scope/3.1-input.md (sections 1-5, 8.2, 9 lane A).

1. A new `src/platform/input.rs` is the only caller of `SendInput` in `src`. It holds key strokes (a vk with a direction,
   and a raw stroke with a given scan code and extended flag, for new feature 5's replay and feature 6's down-only and
   up-only strokes); a pure builder that fills a fixed-capacity array of `INPUT` (no heap); `dwExtraInfo =
   injected_sentinel()` on every record (`src/core/constants.rs:15-32`); the one `unsafe { SendInput(..) }` with its
   SAFETY comment, returning the inserted count; and a `cfg(test)` recorder in place of the call, one ordered
   thread-local log shared with the record of `FastAction::run` (CLAUDE.md rule 9: under test it only records).
2. Port the four builders and delete them (today six `SendInput(` lines in four files, Grep at 252daef):
   - `press_vk`, `src/core/windows_actions.rs:252-291` (SendInput :287);
   - `SystemFunction::toggle_caps_lock` and `press_key`, `src/system/system_function.rs:119-159` (:155);
   - `type_text`, `unicode_input`, `press_key`, `press_combo`, `vk_input`, `is_modifier`,
     `src/data/sequence.rs:119-244` (:139, :169, :205); `is_modifier` duplicates `ModifierSet::ALL`
     (`src/engine/modifiers.rs:15-60`);
   - `menu_mask_inputs`, `inject_menu_mask`, `send_menu_mask`, `src/hook/keyboard_hook_callback.rs:336-392` (:364).
3. `wScan` from `MapVirtualKeyW(MAPVK_VK_TO_VSC_EX)`, read once on the main thread at startup into a table (probe the
   per-call cost with row R21v first; the cut names where the table is built). `KEYEVENTF_EXTENDEDKEY` is set when the
   high byte is `0xE0`, not from a hand list (Microsoft's list is longer than the roadmap's: Print Screen, Break,
   numpad Divide and Enter, the consumer keys). An `0xE1` code (Pause) goes out with its low byte and no flag. Vk 0xE8
   gets whatever the table gives, expected 0 (closes the Phase 1 follow-up 'the mask is sent with scan code 0' as by
   design). Residual to declare: `MapVirtualKeyW` follows the calling thread's layout.
4. `Text`: `\n`, `\r` and `\r\n` each press Enter once, `\t` presses Tab, everything else stays `KEYEVENTF_UNICODE`
   (known issue 34, `book/src/config/known-issues.md:78`).

   Note (lane feat-text-action, merged before Phase 3): `BindAction::Text(Arc<str>)` is a slow action that
   `Win32Runner::action` types through `sequence::type_text`, the macro step's builder. Port it with the step. `Text`, as
   the action and as the step, stays ONE `SendInput` for the whole text whatever its length: item 1's fixed-capacity
   array must not split it into several calls, which would let the user's keys land between the chunks (feat-paste
   critique M5). The fixed capacity is the hook path's; the executor's `Text` may use a heap buffer of any length.
   Item 4 applies to the action too.
5. `carry_out` (`src/hook/keyboard_hook_callback.rs:303-320`) sends the decision's mask in the same `SendInput` as its
   first key-injecting fast action, and keeps the order: mask, then the actions in order. An action that is not input
   ends a batch.
6. Counters and scans: `menu_mask_short` and `key_press_short` (`src/logging/counters.rs:24-28`) are redefined per batch
   or kept per batch kind (the cut chooses; `errors.md:231` follows). The hook-path no-`log::` scan
   (`src/logging/mod.rs:405`, the list of functions near :413 and :433) names the module's hook-path functions. The new
   `SendInput(` scan uses the lexical scan helper of p3-3.4-groundwork.
7. Move the manual injection test (`keyboard_hook_callback.rs:1683-1687`,
   `the_menu_mask_key_is_inserted_into_the_input_stream`) into the module, still `#[ignore = "manual: ..."]`.

Acceptance:
- Unit (scan): outside tests, `SendInput(` occurs on exactly one line of `src`, `KEYBDINPUT {` only in
  `src/platform/input.rs`, and `injected_sentinel()` only in `src/core/constants.rs` and `src/platform/input.rs`.
- Unit: `PressKey(Left)` builds `[Left down, Left up]` with `wScan` 0x4B, `KEYEVENTF_EXTENDEDKEY` on both,
  `KEYEVENTF_KEYUP` on the second, `time` 0 and the sentinel on both.
- Unit (read-only `MapVirtualKeyW`, no injection): the extended flag is set for RCtrl, RAlt, Ins, Del, Home, End, PgUp,
  PgDn, the four arrows, LWin, RWin, Apps, PrintScreen and Divide; never for LCtrl, LAlt, LShift, RShift, F1-F24, Esc,
  Tab, Enter or the numpad digits; Pause has none. NumLock and the media keys are pinned as measured (the run settles
  the UNVERIFIED values). Letters are layout-dependent and not asserted.
- Unit: the mask is vk 0xE8 down then up with the table's scan code (expected 0), no extended flag, the sentinel.
- Unit: `Text("a\nb\tc\r\nd")` builds unicode `a`, one Enter press, unicode `b`, one Tab press, unicode `c`, one Enter
  press, unicode `d`.
- Unit (lane feat-text-action's note to item 4): a 2,000-character `Text`, through the action's entry and as the step,
  records one batch of 4,000 records and one `SendInput` through the recorder.
- Unit (test seam): `carry_out` of `Decision { mask: true, fire: [PressKey(Left), PostMessage(WM_NULL), PressKey(Home)] }`
  records `[E8 down, E8 up, Left down, Left up]`, then the PostMessage, then `[Home down, Home up]` (closes the 1.2
  tester gap M14, p1-1.2-mask-key/results.md:79). Nothing reaches Windows under test.
- Unit: a short count of a batch bumps its counter once, through the seam.
- Review: no heap allocation on the hook path (the batch is a fixed-capacity array; `carry_out` and `FastAction::run`
  allocate nothing new).
- Bench (counted apart): R1-R18 and the R19 family within the CLAUDE.md rules, parent and HEAD, 3 runs each. New rows:
  R21, the pure build of a `PressKey` batch (evidence); R21v, one `MapVirtualKeyW` (probe); R22, `carry_out` of a
  masked `PressKey` decision through the recorder, bound: today's `carry_out` with the mask plus R21.
- Manual (owner, later batch): `PressKey("Up")` in an RDP or Hyper-V window with NumLock on moves the cursor up
  (KS-36); `PressKey("RAlt")` acts as AltGr on the Russian layout (`kbdru.dll` sets `KLLF_ALTGR`, keys.md section 6.6);
  `Text("a\nb")` gives two lines in Notepad; the moved mask test passes when run by hand.
- Gate green.

Latency: unchanged for every event that fires no key action (R1, R4, R11, R12, R12m must not move). A key action sends
the same number of events plus a table index for the scan code; no Win32 call if the table is built at startup. A masked
key action makes one `SendInput` instead of two. No new allocation, lock, timer or wake-up. The OS cost of an injected
event cannot be benched (no injection); state it as unmeasured.

Docs: CHANGELOG `[Unreleased]`: Changed (scan codes and the extended flag, KS-36, roadmap 3.1; the mask joins the first
key batch); Fixed (`Text` newline and tab, known issue 34). README module tree (:85-89), the `PressKey` (:367) and `Text`
(:610) rows; `src/config/default_config.ron` comments :158-161; `book/src/config/actions.md` section 13.3 (:97), 13.5
(:168), section 14 `Text` (:342-344); `keys.md` section 6.8 (:219-226); `known-issues.md` rows 34 and 38 (builder half
fixed, the application-level half stays a manual check); `errors.md` :231, :233. Non-config pages go to the book sync,
listed in impl.md with file:line: architecture.md 'Input synthesis' (:467-469), the audit KS-36 status line (with the
NumLock drift: Microsoft does not count NumLock as extended), gaps.md 'What no test exercises' item 4 (:285), the roadmap
3.1 status. The orchestrator updates CLAUDE.md principle 3 and the Layout (`src/platform/input.rs`).
