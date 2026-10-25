# Context pack, round 1: p2-2.5-latency-trim (tester + code-reviewer)

Scout: project-analyst (read-only). Read on branch `perf/latency-trim`, HEAD `786e61f`, master `3cde1d0`, 2026-10-20.
Binding inputs: `spec.md`, `cut.md` (revision 2 + "Developer addendum" at `cut.md:1764-1950`), `rulings.md` (rounds 1-4;
rulings win over the cut), `impl.md`. All paths below are relative to `D:/key-switch-rs/key-switch-rs` unless they
start with `D:/tmp`.

## Scope and delta

`git log --oneline master..perf/latency-trim` (oldest last):

```
786e61f Docs: a modifier held for a key event costs about 0.6 us (ruling L4)                 [17]
e391a06 Bench: R16 is gated against R12 and costs about 4 us without the per-event reads (ruling L3) [16]
b106db9 Watchdog: drop the test that the first raw key event waits for a tick (ruling L3)    [15, revert of ddb9f3a]
47af1b1 Watchdog: an idle wake reads the raw input at once again (ruling L3)                 [14, revert of ea944da]
ea944da Watchdog: an idle wake only starts the ticks                                         [13, D12, reverted]
ddb9f3a Watchdog: the first raw key event after idle is read at the first tick (red)         [12, reverted]
aa86922 Watchdog: raw key events no callback explains make the main thread read the modifiers [11]
4703385 Hook: a late callback or a long stretch away from messages makes the main thread read the modifiers [10]
e84625e Engine: a key event asks Windows only about the modifiers held for it and drops those Windows has up [9]
d5458d2 Engine: a modifier Windows has up does not count for the next key event (red)        [8]
3f0dc62 Engine: the key path no longer reads every modifier from Windows                     [7]
b0db237 Engine: a key event with no modifier held asks Windows about no modifier (red)       [6]
045b8b4 Main thread: read the modifiers between events when something could have hidden them [5]
c6b186b Engine: a swallowed repeat with no record is recorded, so its key-up asks Windows   [4]
9a9bdcf Engine: a swallowed repeat of a modifier with no record keeps its release from Windows (red) [3]
62c9fd8 Bench: rows R12m, R12m2, R17t and R17a; the callback's events carry a time stamp     [2 = tree P]
7cd8237 Docs: a Hold's fast actions run on the main thread, and timers.rs serves three threads [1]
```

`git diff --stat master..perf/latency-trim`: 24 files, +3240/-295.

```
CHANGELOG.md +73/-  Cargo.toml +1  README.md 29  book/src/config/errors.md 2  keys.md 2  known-issues.md +1
src/bench.rs 158  src/engine/env.rs 22  keys.rs 443  matching.rs 25  mod.rs 606  modifiers.rs +201 (new)
properties.rs 111  resync.rs +241 (new)  scenarios.rs 354  testing.rs 332  src/hook/health.rs 31
keyboard_hook_callback.rs 318  main_thread.rs 293  watchdog/decision.rs 48  watchdog/mod.rs 66
src/logging/counters.rs 130  writer.rs 39  src/platform/timers.rs 9
```

Tree identities (verified with `git rev-parse <c>^{tree}`):
- `aa86922` (commit 11) and `b106db9` (15) share tree `b6a18e3`.
- `ddb9f3a` (12) and `47af1b1` (14) share tree `601c874`.
- `git diff --stat aa86922 786e61f -- src`: `src/bench.rs` 9 lines, `src/hook/watchdog/mod.rs` 5 lines; every changed
  line is a `//` / `//!` comment (filtered diff shows no code line).
- `git diff --stat master 62c9fd8 -- src`: only `src/bench.rs` (+119/-2) and `src/platform/timers.rs` (+6/-3).
- `#[test]` count under `src`: master 800, HEAD 858. `#[ignore` count: 7 on both; the diff adds or removes no
  `#[ignore`.

## File map

| path:lines (HEAD) | what lives there | why it matters this round |
|---|---|---|
| `src/engine/mod.rs:544-574` | `Engine::on_key`: `resync.event` (:554), the key-down check `clear_released_modifiers(env, vk, ModifierSet::ALL)` (:564-565) | W2/W2a key path; X21, X26, X50 sites |
| `src/engine/mod.rs:641-701` | `on_foreground_changed` (:657, bump only), `resync_if_due` (:670-677), `input_returned` (:682), `track_injected` (:693-696), `holds` (test only, :700) | hiding-count sources; X13, X14, X12 |
| `src/engine/mod.rs:702-763` | `Engine::reset`: RemappingSwitched/HookReinstalled -> `resync_from_os` + `read_everything` (:750-753); `Reloaded => {}` (:756); InputLeft -> `release_all` + `input_left` (:757-760) | X15, X16, X24 |
| `src/engine/mod.rs:919-958` | repeat branch of `on_key_down`: `is_blocked` / `is_passed` / `mark_repeat_suppressed` (:945) / `mark_passed` | D10; X3, X4 |
| `src/engine/mod.rs:961-1009` | `on_key_up`: witness `decision.suppress = swallow && !env.os_key_down(vk);` (:999); key-up check over `ModifierSet::MENU` only when suppressed (:1000-1006) | W1 (witness unchanged), X31, X32, X51 |
| `src/engine/mod.rs:819` | the KS-05 blocker read `env.os_key_down(key)` (unchanged, Q10/Q6) | stays on key path |
| `src/engine/mod.rs:1022-` tests | new tests at :1410, :1469, :1492, :1581, :1628, :1644, :1656, :1789, :1831, :1852 | red-first / compile-red carriers |
| `src/engine/keys.rs:55-87` | `LastDown` gains `RepeatSuppressed` (:70); `KeyState` | D10 |
| `src/engine/keys.rs:96-117` | `resync_modifiers` (drain read): sided slot takes OS state (:110-111); generic slot cleared only if set and reads up (:112-113); KS-14 skip (:107) | X17, X33, X34 |
| `src/engine/keys.rs:119-146` | `clear_released_modifiers` (the check, D16): `active && modifier != own && !key_down_swallowed && !os_key_down` -> clear (:135-141) | X25-X30 |
| `src/engine/keys.rs:148-160` | `track_injected`: modifiers only, KS-14 skip, flag = direction | X23 |
| `src/engine/keys.rs:162-206` | `resync_from_os`: skips generic codes (:182, :190) and mouse buttons; keeps `Suppressed`/`RepeatSuppressed` records (:194-199) | X1, X22 |
| `src/engine/keys.rs:213-216, :244-250, :283-291, :304-310` | `key_down_swallowed` (only `Suppressed`), `mark_repeat_suppressed`, `take_blocked` (consumes both), `alt_or_win_down_in_windows` | X1, X2; mask rule |
| `src/engine/keys.rs:335-` tests | new: :549, :774, :801, :823 (R-G), :892 (C-1), :920 (C-2), :936; removed `sync_overwrites_modifiers_except_the_skipped_key` | carriers |
| `src/engine/modifiers.rs:1-201` (new) | `ModifierSet` bits: `EMPTY`, `ALL`, `SIDED` (:41-50), `MENU` (:54), `touched_by` (:94-106), test :146 | X18 |
| `src/engine/resync.rs:1-241` (new) | `Resync { hiding: 1, resynced: 0, notes_seen, fresh, owed, input_away }` (:31-45); `bump` (:61), `event` (:69-74), `input_left` (:77), `input_returned` (:83), `read_everything` (:90), `plan` (:98-117); tests :133-230 | X5-X12 |
| `src/engine/env.rs`, `src/engine/matching.rs` | diff vs master is doc comments only (no `+`/`-` code line) | none functional |
| `src/engine/testing.rs:384-500` | harness: `up_unseen`/`down_unseen` now note hidden input and drain (:389, :425, :408-410, :454-456); `up_unnoticed`/`down_unnoticed` do not (:396, :433); `hidden_input_noted` (:467); `drain` calls `resync_if_due` only while enabled (:483-489); `foreground_changed` bumps + drains (:491) | changes semantics of every existing `*_unseen` step |
| `src/engine/testing.rs:759-850` | new steps `down_generic`, `up_generic`, `up_generic_unnoticed`, `down_swallowed_later`, `repeat_swallowed_later`, `up_swallowed_later`, `daemon_injects_down/up` | R-A..R-H, G1, I-1 |
| `src/engine/properties.rs:56-66, :222-265, :276-300, :464-469` | `Mode::Unnoticed` (later-hook presses 1 in 10, :276-291); new property at :464 | Unnoticed property; X51 |
| `src/engine/scenarios.rs:2413-2740` | 16 new scenarios (list in Red-first section); 2.3 scenarios edited in comments only (all 12 `-` lines are `//`) | acceptance |
| `src/hook/keyboard_hook_callback.rs:236-296` | `decide_event`: sentinel -> `track_injected` then `None` (:246-249); `track_injected` adapter takes `with_main` (:287-296) | D2; R2 note below |
| `src/hook/keyboard_hook_callback.rs:394-460` | the extern callback: `stamp = event.time`, `injected = LLKHF_INJECTED` (:426-427); after `record_callback`, `age = 0` if injected else `event_age_ms(tick_count(), stamp)` (:440-446); `late_callback` -> `note_late()` (:447-449) | E4, W4; X38-X40 |
| `src/hook/keyboard_hook_callback.rs:463-473` | `event_age_ms` (wrap, `>= 1<<31` -> 0), `late_callback` (`ran >= 17 ms || age >= 17`) | X38 |
| `src/hook/keyboard_hook_callback.rs:485-503` | `foreground_changed` WinEvent: `with_main(main.foreground_changed)` then `wake_this_thread()` (:502) | E1, X20 |
| `src/hook/keyboard_hook_callback.rs:108-117` | module doc "Re-entry": declared residual of V3-I1 (nested decided event) | V3-I1 |
| `src/hook/keyboard_hook_callback.rs:550-` tests | :987, :1026, :1039, :1054, :1308, :1347 | carriers |
| `src/hook/main_thread.rs:122-186` | TLS `HIDDEN_INPUT: Cell<u64>` (:128); `note_hidden_input` (:135), `hidden_input_notes` (:141); `HOOK_LATE_MS = 17` (:151); `tick_count` (`GetTickCount`, SAFETY, :161-165); `note_late` (:170-173); `note_if_awake_too_long` (:180-186) | E4/E5/E8 |
| `src/hook/main_thread.rs:241-249, :294-308` | `note_unknown_desktop_switch` notes hidden input (:247); `with_main` refusal notes hidden input (:299) | E8, X20 |
| `src/hook/main_thread.rs:362-391` | `MainState::foreground_changed` -> `engine.on_foreground_changed()` (:364); `input_returned` (:371-374); `resync_if_due`: returns false if `!hook.is_enabled()`, notes = TLS + `health().unseen_input()` (:383-391) | X19, X45 |
| `src/hook/main_thread.rs:674-676` | `MainThreadHandle::wake_for_hidden_input` -> `wake()` | E6 |
| `src/hook/main_thread.rs:780-836` | `run`: `awake_since` (:788); E5 check before the wait, `continue` to drain (:802-806); `awake_since = tick_count()` after the wait (:820) | X41 |
| `src/hook/main_thread.rs:841-861` | `drain`: requests, signals, due timers loop, then `with_main(resync_if_due)` (:860) | X19 |
| `src/hook/main_thread.rs:1030-` tests | :1961, :1977, :1990, :2000, :2013, :2031, :2063 | carriers |
| `src/hook/health.rs:44, :132-142, :259` | `unseen_input: AtomicU64`; `note_unseen_input` `fetch_add(1, AcqRel)`; `unseen_input()` `load(Acquire)`; test | R4; X42 |
| `src/hook/watchdog/decision.rs:116-120, :172-174, :189, :416` | `unseen_at_last_tick` field/getter; set `arrived.saturating_sub(credit + answered)`; test | X43 |
| `src/hook/watchdog/mod.rs:15-24, :37-46, :103-121, :281-286` | module docs (E6; R16 "about 4 us ... gated ... L3"); `RequestReinstall::hidden_input` default no-op (:107), impl for `MainThreadHandle` (:118); `Ticker::pass` notes + wakes if `unseen_at_last_tick() > 0` (:283-286) | X44 |
| `src/hook/watchdog/mod.rs:1635, :1665-1680, :1689, :1871, :1920` | E6 test; 2.4 tests with restored expectations (`2250..5000` at :1677) | L3 guard |
| `src/logging/counters.rs:69, :86, :92-105, :124, :173, :238-271, :330` | `Counter::HookLate` (12th counter, `counts[11]`); `warns()` false only for `HookLate`; `summary -> Option<Report { warns, text }>`; test | W4, D9; X41 |
| `src/logging/writer.rs:233-236, :371` | level = `Warn` if `report.warns` else `Info`; test | W4 |
| `src/bench.rs:55-98, :350-376, :484-560, :698-, :1143-1280` | row docs (R12m, R12m2, R16 gate sentence :82-90, R17t, R17a, R18); `call_keyboard_callback` stamps `time: GetTickCount()` (:374); `call_injected_callback` (:357); R12m/R12m2 builder (:488); R17t/R17a/R18 (:522-560); `latency_bench` ignored (:1143) | bench §4b |
| `src/platform/timers.rs:1-4` | D14 doc: Delay (2.1), gesture timers (2.2), watchdog ticks (2.4) | D14 |
| `CHANGELOG.md:258-270, :643-676, :1063-1069` | Added (R12m.., R18, `hook_late`); Changed (modifier reads, 0.6 us, R16 4.4 us gated; counters level); Fixed (D10) | docs |
| `README.md` hunks `@@ -146`, `@@ -555`, `@@ -868`, `@@ -945` (HEAD :149, :558-570, :871-881, :955, :959) | GetAsyncKeyState uses; KS-14/D10; `hook_late`; watchdog wake about 4 us | docs |
| `book/src/config/known-issues.md:156` (row 88), `keys.md:194`, `errors.md:231` (`hook_late` level), `errors.md:214-215` (item 2.4 text restored) | config pages | docs |

## Facts

1. On P (`62c9fd8`), `Engine::on_key` calls `self.keys.sync_modifiers(env, Some(event.vk));` at `src/engine/mod.rs:522`
   (`git show 62c9fd8:src/engine/mod.rs`), and `on_foreground_changed(env)` calls `sync_modifiers(env, None)` at :614.
   The N probe tree removes only :522 (`cut.md:830`).
2. At HEAD `sync_modifiers` does not exist (`git diff master..HEAD -- src` removes `pub(super) fn sync_modifiers`).
3. At HEAD the key-down path's only modifier reads are in `KeyState::clear_released_modifiers`
   (`src/engine/keys.rs:127-146`), called with `ModifierSet::ALL` at `src/engine/mod.rs:564-565`; it asks only codes whose
   `active` flag is set, that are not the event's own code and whose `last_down` is not `Suppressed` (`keys.rs:135-138`).
4. The check only clears (`self.active[index] = false`, `keys.rs:140`); it never writes `true`.
5. `key_down_swallowed` is `last_down == Suppressed` only (`keys.rs:214-216`); a `RepeatSuppressed` key is checked and
   counts for `alt_or_win_down_in_windows` (`keys.rs:304-310`).
6. The key-up path asks the OS for the event's own key only when `swallow` is true (`mod.rs:999`, `&&` short-circuit),
   then runs the check over `ModifierSet::MENU` only when `decision.suppress` is true (`mod.rs:1000-1006`).
7. `ModifierSet::MENU` = `{VK_LMENU, VK_RMENU, VK_MENU, VK_LWIN, VK_RWIN}` (`modifiers.rs:54`); `SIDED` is the eight
   sided codes (`modifiers.rs:41-50`).
8. The drain read `resync_modifiers` sets/clears sided slots from the OS and only clears a generic slot that is set and
   reads up (`keys.rs:110-114`); it skips `Suppressed` keys (`keys.rs:107`).
9. `resync_from_os` excludes the generic codes from its 1..=254 scan (`keys.rs:182`, `:190`) and keeps
   `Suppressed`/`RepeatSuppressed` records of keys not held (`keys.rs:194-199`).
10. `Resync::new` starts `hiding: 1, resynced: 0` (`resync.rs:49-57`), so the first `plan` reads `ALL`.
11. `Resync::plan` bumps when `notes != notes_seen` (`resync.rs:99-102`), returns `EMPTY` while `input_away`
    (`:103-105`), else plans `ALL` if `hiding != resynced`, otherwise `owed`, minus `fresh` (`:106-115`).
12. `Resync::event(vk, decided)` adds `touched_by(vk)` to `fresh` and clears `input_away` only when `decided`
    (`resync.rs:69-74`); `on_key` passes `true` (`mod.rs:554`), `track_injected` passes `false` (`mod.rs:695`).
13. Hiding-count sources at HEAD: `on_foreground_changed` -> `bump` (`mod.rs:657-659`); `input_left` /
    `input_returned` bump (`resync.rs:77-87`); notes from `with_main` refusal (`main_thread.rs:299`), unknown desktop
    switch (`main_thread.rs:247`), `note_late` (E4 at `keyboard_hook_callback.rs:447-449`, E5 at
    `main_thread.rs:180-186`), the watchdog's `unseen_input` summed in `MainState::resync_if_due`
    (`main_thread.rs:387`). `swch on` and a reinstall read everything in `reset` and call `read_everything`
    (`mod.rs:750-753`). `Reloaded` neither reads nor bumps (`mod.rs:756`).
14. `MainState::resync_if_due` returns `false` without asking while `!hook.is_enabled()` (`main_thread.rs:383-386`).
15. The drain calls `resync_if_due` once, after the request/signal/timer loop finds no work (`main_thread.rs:841-861`,
    call at :860).
16. The foreground WinEvent callback posts `wake_this_thread()` after `with_main` (`keyboard_hook_callback.rs:502`).
17. `decide_event` handles a sentinel event by calling the adapter `track_injected` and returning `None`
    (`keyboard_hook_callback.rs:246-249`); the adapter returns early for non-modifiers or `!is_enabled()`, otherwise
    takes `with_main(|main| main.engine.track_injected(event))` (`:287-296`). The sentinel path therefore mutates the
    engine's modifier flag (it changed no engine state on master).
18. The extern callback adds per physical event one `GetTickCount` (`tick_count()`, `main_thread.rs:161-165`) after
    `counters::record_callback(ran)` and before the return (`keyboard_hook_callback.rs:440-446`); an
    `LLKHF_INJECTED` event uses `age = 0` (`:442-443`).
19. `late_callback(age, ran)` is `ran >= HOOK_LATE || age_ms >= HOOK_LATE_MS` with `HOOK_LATE_MS = 17`
    (`keyboard_hook_callback.rs:471-473`, `main_thread.rs:151-154`); `event_age_ms` returns 0 for a wrapped difference
    `>= 1 << 31` (`keyboard_hook_callback.rs:463-466`).
20. `note_late` = `note_hidden_input()` + `counters::bump(Counter::HookLate)`; it names no `log::`
    (`main_thread.rs:170-173`).
21. `HIDDEN_INPUT` is a main-thread TLS `Cell<u64>` updated with `try_with` and `wrapping_add` (`main_thread.rs:128-142`).
22. `HealthState::note_unseen_input` is `fetch_add(1, AcqRel)`; `unseen_input()` is `load(Acquire)`
    (`health.rs:132-142`).
23. The watchdog sets `unseen_at_last_tick = arrived.saturating_sub(credit.saturating_add(answered))` at each tick
    (`watchdog/decision.rs:189`) and, if `> 0`, `Ticker::pass` calls `note_unseen_input()` and
    `requests.hidden_input()` (`watchdog/mod.rs:283-286`); the trait default is a no-op (`:107`), the main-thread
    impl posts `MAIN_WAKE` (`:118-120`, `main_thread.rs:674-676`).
24. In `MainThread::run` the E5 check runs after `plan_wake` and before `MsgWaitForMultipleObjectsEx`, `continue`s on a
    note, and `awake_since` is reset right after the wait (`main_thread.rs:799-820`).
25. `Counter::warns()` returns `false` only for `HookLate` (`counters.rs:92-105`); `summary` returns
    `Option<Report { warns, text }>` (`counters.rs:238-271`); the writer picks `Warn`/`Info` from `report.warns`
    (`writer.rs:233-236`).
26. Harness: `up_unseen`/`down_unseen` now call `note_hidden_input()` -> `drain()` -> `resync_if_due`
    (`testing.rs:408-410`, `:454-456`, `:475-489`); `up_unnoticed`/`down_unnoticed` do not. Existing 2.3 scenarios
    use `*_unseen` (32 occurrences in `scenarios.rs` at HEAD, 31 on master).
27. Every `-` line of `src/engine/scenarios.rs` vs master is a `//` comment (12 lines): no 2.3 scenario's steps or
    assertions changed.
28. Commits 3, 6 and 8 are test-only: their hunks in `keys.rs`/`mod.rs` fall inside `mod tests`
    (`git show d5458d2`/`b0db237` hunk headers), and `testing.rs`, `scenarios.rs`, `properties.rs` are
    `#[cfg(test)]` modules (`mod.rs:264-271`).
29. The witness line `decision.suppress = swallow && !env.os_key_down(vk);` (`mod.rs:999`) is unchanged in behaviour
    from master (W1); R11 keeps its witness term.
30. The KS-05 blocker read `env.os_key_down(key)` remains at `mod.rs:819`.
31. `CLAUDE.md:72` still says "R16, one wake per burst, about 50 us, is declared and not gated", and
    `.claude/agents/tester.md:101` still says "R16 is a declared row, not gated"; ruling L3 supersedes both for this
    lane (`rulings.md`, Round 4, L3). Neither file is in the diff.
32. `src/platform/timers.rs:1-3` names Delay (2.1), gesture timers (2.2) and watchdog ticks (2.4); `grep "timer thread"
    CHANGELOG.md` finds nothing (D14 done).
33. New unsafe at HEAD outside tests: `tick_count` (`main_thread.rs:161-165`) and bench `GetTickCount` /
    `GetAsyncKeyState` reads, each with a SAFETY comment (`impl.md:223-234`); no new `Relaxed` (`impl.md:233-234`).
    UNVERIFIED by this scout beyond the lines read.

## Red-first tests (tester replays; each on the commit named)

Run each in a scratch worktree (`git worktree add <dir> <sha>`, own `CARGO_TARGET_DIR`). Command pattern:
`cargo test --locked --lib <path> -- --exact` (add `--exact` per test). The red commit's tree already holds the test, so
"red at commit X" means: check out X and run.

| # | Test(s) (path at HEAD) | Red at | Expected red output | Green at |
|---|---|---|---|---|
| RF-3 | `engine::scenarios::a_swallowed_repeat_of_a_modifier_with_no_record_keeps_its_release_from_windows` (`scenarios.rs:2413`) | `9a9bdcf` (tree = P + test; master has the API) | panic `scenario: up LShift at +0 ms: Windows got a lone key-up of LShift` | `c6b186b`, HEAD |
| RF-6 | `engine::tests::a_key_event_with_no_modifier_held_asks_the_os_about_no_modifier` (`mod.rs:1410`) | `b0db237` (tree = commit 5 + test; needs commit 5's `CountingEnv`/drain seam) | ``assertion `left == right` failed: a key event with no modifier held asks Windows about no modifier (roadmap item 2.5)``, `left: 44`, `right: 0` | `3f0dc62`, HEAD |
| RF-8 | eight tests, all at `d5458d2` (tree = commit 7 `3f0dc62` + tests; rulings Q9: replay on commit 7, never master) | `d5458d2` | see rows below | `e84625e`, HEAD |
| R-A | `engine::scenarios::a_modifier_that_a_later_hook_swallows_does_not_count_for_matching` (`:2619`) | `d5458d2` | `scenario: down J at +0 ms: expected to fire [DoNothing], fired []` | |
| R-B | `engine::scenarios::a_shift_that_a_later_hook_swallows_does_not_make_alt_the_language_hotkey` (`:2654`) | `d5458d2` | `scenario: down LAlt at +0 ms: expected the event to be passed, it was suppressed` | |
| R-C | `engine::scenarios::another_programs_generic_shift_released_unnoticed_stops_counting_after_a_note` (`:2676`) | `d5458d2` | `scenario: down CapsLock at +0 ms: expected to fire [SwitchLanguage], fired [ToggleCapsLock]` | |
| R-D | `engine::scenarios::another_programs_generic_shift_released_unnoticed_stops_counting_at_the_next_key` (`:2692`) | `d5458d2` | same as R-C | |
| R-E | `engine::scenarios::a_modifier_released_where_the_hook_cannot_see_it_stops_counting_at_the_next_key` (`:2706`) | `d5458d2` | same as R-C | |
| R-F | `engine::tests::a_key_event_asks_windows_once_for_each_modifier_held_for_it_and_never_for_its_own_key_or_a_swallowed_one` (`mod.rs:1492`) | `d5458d2` | ``A down with LCtrl held: a key event asks Windows once about each modifier held for it, never about its own key or a swallowed one (roadmap item 2.5)``, `left: 0`, `right: 1` | |
| R-G | `engine::keys::tests::the_drain_read_clears_a_generic_slot_only_when_its_code_reads_up_and_never_sets_one` (`keys.rs:823`) | `d5458d2` | panic `Windows has neither Shift down: the drain read clears the generic slot (roadmap item 2.5)` | |
| R-H | `engine::scenarios::a_swallowed_key_up_asks_no_mask_for_an_alt_that_windows_has_up` (`:2723`) | `d5458d2` | `scenario: up F13 at +0 ms: mask true, the rule says false (suppressed true, Alt or Win down in Windows false)` | |
| RF-12 | `hook::watchdog::tests::the_first_raw_key_event_after_idle_is_read_at_the_first_tick` | removed from HEAD by `b106db9` (L3); nothing to replay (`cut.md:1911`) | - | - |

Declared red commits (fail `cargo test`): 3, 6, 8, 12 (`cut.md:1186-1187`) and `47af1b1` (tree of commit 12; red by
"read at the first tick: [(0, 1)]", `left: Some((0, 1))`, `right: Some((250, 1))`; `impl.md:198-206`, `cut.md:1893-1897`).

Source of the expected messages: commit bodies of `9a9bdcf`, `b0db237`, `d5458d2` (`git log -1 --format=%B <sha>`),
`cut.md:1235`, `:1305-1307`, `:1357-1366`.

## Compile-red tests and their mutations (tester applies at HEAD, runs the catcher, restores)

Table: `cut.md:1681-1733` (X1-X45) as amended by the addendum (`cut.md:1795`, `:1800-1803`, `:1932-1935`).
**Removed:** X35-X37 (D17, round 3) and X46-X49 (D12, L3). **Added:** X50, X51. X31 is split into X31a/X31b
(`cut.md:1812-1815`).

| # | Mutation site at HEAD | Catcher(s) |
|---|---|---|
| X1 | `keys.rs:194-199` (reset drops `RepeatSuppressed`) or `keys.rs:283-291` (`take_blocked` ignores it) | `engine::keys::tests::a_repeat_suppressed_record_is_consumed_like_a_swallowed_one_and_kept_by_a_reset` (`keys.rs:774`) |
| X2 | `keys.rs:214-216` counts `RepeatSuppressed` | `..._is_not_skipped_by_the_modifier_resync_and_counts_for_the_mask` (`keys.rs:801`) |
| X3 | `mod.rs:946-948` records `Passed` only over `Unknown` | `engine::tests::a_repeat_with_no_record_that_passes_after_a_swallowed_one_is_recorded_as_passed` (`mod.rs:1789`) |
| X4 | `mod.rs:944-945` (no `mark_repeat_suppressed`) | RF-3 scenario only (the Unnoticed property passes under X4: `impl.md:133`) |
| X5 | `resync.rs:51` `hiding: 0` | `engine::resync::tests::a_new_resync_plans_a_full_read_at_its_first_plan` |
| X6 | `resync.rs:61-63` no-op | `a_bump_plans_one_full_read`; S3 |
| X7 | `resync.rs:99-102` no fold / `>` | `a_changed_note_count_plans_a_full_read_a_smaller_one_included`; S8 |
| X8 | `resync.rs:110-114` reads fresh / drops owed | `a_modifier_touched_since_the_last_plan_is_left_out_and_read_at_the_next` |
| X9 | `resync.rs:103-105` ignores `input_away` | `nothing_is_read_while_input_is_away_until_it_returns_or_a_key_event_is_decided` |
| X10 | `resync.rs:71-73` decided does not clear `input_away` | same; S5 |
| X11 | `resync.rs:77-80` `input_left` no bump | S5 |
| X12 | `mod.rs:695` passes `true` | `an_injected_event_marks_its_modifier_fresh_and_does_not_bring_input_back` |
| X13 | `mod.rs:682-684` no-op | `engine::tests::every_source_of_the_hiding_count_makes_the_next_drain_read_the_modifiers` (`mod.rs:1581`); S4 |
| X14 | `mod.rs:657-659` no bump | same; S3 |
| X15 | `mod.rs:756` `Reloaded` bumps | `a_reload_makes_no_read` (`mod.rs:1644`) |
| X16 | `mod.rs:751-752` skip `read_everything` | `swch_on_and_a_reinstall_read_everything_themselves_so_the_next_drain_reads_nothing` (`mod.rs:1628`) |
| X17 | `keys.rs:104-117` asks unset generic / drops KS-14 skip | `a_drain_read_asks_the_eight_sided_modifiers_and_skips_a_swallowed_one` (`mod.rs:1656`); S9 |
| X18 | `modifiers.rs:94-106` | `engine::modifiers::tests::a_sided_modifier_touches_its_generic_code_and_a_generic_one_both_sides` (`modifiers.rs:146`) |
| X19 | `main_thread.rs:860` / `:371-374` | `hook::main_thread::tests::the_drain_reads_the_modifiers_after_a_note_and_not_again` (`:1961`) / `a_return_signal_makes_the_next_drain_read_the_modifiers` (`:2013`) |
| X20 | `main_thread.rs:299` / `:247` / `keyboard_hook_callback.rs:502` | `a_refused_call_notes_hidden_input` (`:1990`) / `a_desktop_switch_to_an_unknown_destination_notes_hidden_input` (`:2000`) / `hook::keyboard_hook_callback::tests::a_foreground_change_queues_a_wake_for_the_drain` (`:1347`) |
| X21 | `sync_modifiers` restored in `on_key` | RF-6 test; R-F |
| X22 | `keys.rs:182`/`:190` reads 0x10-0x12 | `engine::keys::tests::a_reset_never_takes_a_generic_modifier_from_the_os` (`keys.rs:936`) |
| X23 | `keys.rs:150-160` / `keyboard_hook_callback.rs:246-247` | I-1 (`scenarios.rs:2598`) and `the_daemons_own_injected_modifiers_pass_and_are_tracked` (`callback:1308`) / `an_injected_key_up_of_a_held_swallowed_modifier_leaves_it_held` (`mod.rs:1831`) / `an_injected_event_of_a_key_that_is_not_a_modifier_changes_nothing` (`mod.rs:1852`) |
| X24 | `mod.rs:750` keeps keys for RemappingSwitched/HookReinstalled | S6 (`:2517`), S7 (`:2532`) |
| X25 | `keys.rs:135-140` clears a held generic slot regardless | G1 (`scenarios.rs:2574`) |
| X26 | remove `mod.rs:564-565` and `:1003-1006` | R-A, R-B, R-D, R-E, R-F, R-H |
| X27 | check visits every slot and writes OS state | R-F; C-1 (`keys.rs:892`) |
| X28 | drop `!self.key_down_swallowed(index)` at `keys.rs:137` | R-F; S9 (`scenarios.rs:2547`); C-1 |
| X29 | generic read replaced by sided in the check | C-2 (`keys.rs:920`) |
| X30 | drop `modifier != own` (`keys.rs:136`) | R-F; C-1 |
| X31a | key-up check over `ModifierSet::ALL` (`mod.rs:1005`) | R-F "LWin up, swallowed" (2 vs 0), "F13 up ... LAlt held" (3 vs 1) |
| X31b | key-up check also at a passing key-up (`mod.rs:1001`) | R-F "A up with LAlt held, passing" (`mod.rs:1553`, 1 vs 0) |
| X32 | remove `mod.rs:1000-1006` | R-H |
| X33 | `keys.rs:112-113` leaves generic alone / sets it | R-G |
| X34 | `keys.rs:112` generic read -> sided | R-G |
| X26+X33 | both repairs removed | R-C |
| X38 | `keyboard_hook_callback.rs:472` `>` / `:465` signed rule removed | `lateness_counts_age_and_run_with_a_17_ms_boundary` (`:987`) |
| X39 | no note at `:447-449` / always | `a_callback_for_an_event_a_second_old_notes_hidden_input_and_counts_hook_late` (`:1026`) / `a_callback_for_a_fresh_event_notes_nothing` (`:1039`) |
| X40 | `:442-443` injected stamp counts | `an_injected_events_own_stamp_is_not_its_age` (`:1054`) |
| X41 | `main_thread.rs:180-186` never notes / `:802-806` removed / `counters.rs:105` warns | `an_awake_stretch_of_17_ms_notes_hidden_input` (`:2031`) / `the_main_loop_checks_its_awake_stretch_before_every_wait` (`:2063`) / `logging::counters::tests::hook_late_alone_is_reported_at_info_and_with_another_count_at_warn` (`counters.rs:330`) |
| X42 | `health.rs:132-136` no-op | `hook::health::tests::unseen_input_counts_each_note` (`:259`) |
| X43 | `watchdog/decision.rs:189` always 0 | `hook::watchdog::decision::tests::a_tick_reports_raw_key_events_no_callback_explains` (`:416`) |
| X44 | `watchdog/mod.rs:283-286` one of the two calls dropped | `hook::watchdog::tests::a_tick_with_unexplained_raw_input_notes_and_wakes_the_main_thread` (`:1635`) |
| X45 | `main_thread.rs:387` drops `unseen_input()` | `the_drain_counts_the_watchdogs_notes` (`:1977`) |
| X50 | `mod.rs:565` passes `ModifierSet::ALL.without(ModifierSet::touched_by(VK_LWIN))` | `engine::tests::a_key_decided_inside_a_passed_modifiers_hook_call_finds_it_up_when_a_later_hook_swallowed_it` (`mod.rs:1469`); developer saw `left: []`, `right: [DoNothing]` |
| X51 | `mod.rs:999` -> `decision.suppress = swallow;` | `engine::properties::no_key_sticks_and_no_key_up_reaches_windows_alone_when_hidden_modifier_events_go_unnoticed` (`properties.rs:464`); developer saw config default, seed 8, Unnoticed: `scenario: up CapsLock at +6735 ms: VK 0x14 is down in Windows but physically up` |

Developer-confirmed (`impl.md:127-133`): X50, X31a, X31b, X51 red; X4 green under the property. Rows X1-X45 were run
in earlier developer runs ("still valid at HEAD", `impl.md:122-135`); UNVERIFIED by this scout.

S-scenario names (S3-S9, G1, I-1; `cut.md:1325-1335`) at HEAD: S8 `:2440`, S3 `:2461`, S4 `:2480`, S5 `:2497`,
S6 `:2517`, S7 `:2532`, S9 `:2547`, G1 `:2574`, I-1 `:2598` (`src/engine/scenarios.rs`).

## Acceptance proofs (cut `:1487-1509`, as amended by rulings W1-W4, W2a, V3-I1, L3, L4)

- **Gate** (CLAUDE.md): `cargo fmt --check`; `cargo clippy --all-targets --locked -- -D warnings`;
  `cargo test --locked`. Developer ran fmt + clippy at HEAD (pass) and only `hook::watchdog` (79 passed),
  `config::loader` (91), `bench` (1 passed, 1 ignored) (`impl.md:236-247`). Full suite: not yet run.
- **2.3 stuck-key / lone-key-up stays green:** all of `engine::scenarios` incl.
  `a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows`
  (`scenarios.rs:2033`); the six 2.3 properties `properties.rs:434-459` plus the new Unnoticed one (`:464`);
  `engine::tests::only_a_key_up_the_engine_would_swallow_asks_the_os_about_its_own_key` (`mod.rs:1353`, W1 pin).
- **Each hiding source triggers a resync:** S3-S8, `mod.rs:1581`, adapter tests `main_thread.rs:1990`, `:2000`,
  `callback:1026`, `main_thread.rs:2031`, `watchdog/mod.rs:1635`, `main_thread.rs:1977`.
- **2.4 watchdog acceptance (fake clock) green**, with item 2.4 expectations restored by L3:
  `a_batch_read_at_a_tick_counts_all_its_events` `(250, 8)` (`watchdog/mod.rs:1689`),
  `a_flood_whose_callbacks_were_counted_at_one_tick_draws_no_request` last read 749 ms (`:1871`),
  `the_first_raw_key_event_after_idle_wakes_the_watchdog` no pass in `2250..5000` (`:1665-1677`); X47's remaining
  catcher `raw_input_before_the_hook_is_installed_is_read_and_not_counted` (`:1920`).
- **W4:** `hook_late_alone_is_reported_at_info_and_with_another_count_at_warn` (`counters.rs:330`),
  `logging::writer::tests::a_report_whose_only_count_is_hook_late_is_written_at_info` (`writer.rs:371`),
  `an_injected_events_own_stamp_is_not_its_age`; manual M8.

### Bench (tester §4b, ruling M5 + L3 + "For the tester")

Trees: **P** = `62c9fd8`; **N** = `62c9fd8` with `src/engine/mod.rs:522` (`self.keys.sync_modifiers(env, Some(event.vk));`)
deleted, uncommitted, in its own scratch worktree; **H** = `786e61f`. Nine release runs in one session, interleaved
P1 N1 H1 P2 N2 H2 P3 N3 H3 (`cut.md:836-839`); median of three p50s (means for R1), largest p99. Record CPU load per
run; a breach only from a run at >= 79 % load is re-run once before it is reported (`rulings.md`, round 4).
Command (`.claude/agents/tester.md:101`):
`KS_BENCH_OUT=<file> KS_BENCH_LABEL=<P|N|H> cargo test --release --locked --lib bench::latency_bench -- --ignored --exact --test-threads=1`
(`KS_BENCH_LOAD=1` adds R8; the lane touches the main loop, `main_thread.rs:780-836`). Q10 check:
`git diff --stat master 62c9fd8 -- src` lists bench code only (Scope above).

| Row | Gate (`cut.md:842-854`, L3 at `cut.md:1907`) |
|---|---|
| R12 | H p50 <= N p50 + R17t p50 + 100 ns; H p99 <= 1.5 x N p99 + 2 us |
| R12m | H p50 <= N p50 + 1 x R17a p50 + R17t p50 + 100 ns |
| R12m2 | H p50 <= N p50 + 2 x R17a p50 + R17t p50 + 100 ns |
| R4 | H p50 <= N p50 + R17t p50 + 100 ns |
| R1 | Delta = P mean - N mean; H mean <= N mean + 0.5 x Delta; Delta < 0.010 us -> "cannot tell a revert apart", not a breach |
| R11 | H (R11 - R12) p50 <= P (R11 - R12) p50 + 0.2 us |
| R16 | **H R16 p50 <= H R12 p50 + 20 us**, p99 reported (L3; supersedes 2.4 L2 and the cut's provisional `min(...)`) |
| R17t, R17a | evidence (R17a stop rule closed by L4: ~0.6 us/read) |
| R18 | evidence; sanity p50 <= 8 x R17a p50 + 0.5 us (same H run) |
| R2, R3, R5-R10, R13-R15 | existing rules (2.2 D10, 2.3 D14 (R11 vs 5.4 us there), 2.4: R14 p50 <= R14c p50 + 300 ns; 1 ms-paced rows on p50) |

Developer numbers (p50 us; `impl.md:70-84`): P R12 4.8-8.8, N R12 0.2, commit-11/HEAD R12 0.2, R12m 0.8-1.4,
R12m2 1.3-3.0, R16 at commit 11 4.3/4.4 (65-79 % load) and 21.7 (100 %), P R16 58.3-85.8, R17t 0.001, R17a median
0.613 (19 runs), R18 4.6-6.7. HEAD code = commit 11 code (Scope). Raw: `D:/tmp/ks-orch/p2-2.5-latency-trim/bench-dev-*.md`.

## Manual checks (human only; no `#[ignore]` added)

`cut.md:1516-1527` with addendum changes (`cut.md:1805-1808`, `:1825-1829`, `:1915`; `impl.md:334-353`):
M1-M4, M8, M10 as in the cut; M5/M6/M7 = roadmap checks 44/39/47 as written for item 2.4 (L3); M9 gains the
StickyKeys latched step; M11 PowerToys LWin->F13 (`cfg/m11.ron`). Configs validated by cfgcheck
(`D:/tmp/ks-orch/p2-2.5-latency-trim/cfgcheck-output.txt`; `impl.md:249-267`).

## Invariants in play

- **R1 Hook callback** - added per-event work: `GetTickCount` + compare at exit (`keyboard_hook_callback.rs:440-449`);
  removed: 11 `GetAsyncKeyState` per event (P `mod.rs:522`); added: one read per held unswallowed modifier at a
  key-down (`keys.rs:127-146`). `code < 0` first (`:399-404`); outcome `LRESULT(1)` or `CallNextHookEx`
  (`:450-457`); no `log::` in the callback file outside tests (CLAUDE.md principle 1; `note_late` at
  `main_thread.rs:170-173`).
- **R2 Sentinel** - checked first in `decide_event` (`:246`); the sentinel path now updates a modifier flag under
  `with_main` (`:287-296`, Fact 17).
- **R3 Shared state** - `HIDDEN_INPUT` TLS `Cell` (`main_thread.rs:128`); `with_main` refusal notes hidden input
  (`:297-302`); drain read under the borrow (`main_thread.rs:383-391`, `:860`).
- **R4 Orderings** - `unseen_input` AcqRel/Acquire (`health.rs:132-142`).
- **R6 Display** - new enum variants: `LastDown::RepeatSuppressed` (private, no Display), `Counter::HookLate` (name arm
  `counters.rs:124`).
- **R8 Key identity** - generic vs sided codes: `touched_by` (`modifiers.rs:94-106`), generic handling in check/drain
  (`keys.rs:110-114`, `:135-140`), `resync_from_os` skip (`keys.rs:182`).
- **R10 Message loop** - E5 placement (`main_thread.rs:799-820`); foreground wake (`callback:502`); watchdog wake
  (`main_thread.rs:674-676`).
- **Suppression invariant / KS-06 / KS-14** - D10 `RepeatSuppressed` (`keys.rs:63-70`, `mod.rs:940-948`,
  `keys.rs:283-291`); KS-14 skip in check, drain read and `track_injected` (`keys.rs:107`, `:137`, `:155`).
- **KS-05** blocker read unchanged (`mod.rs:819`). **KS-12** mask after the key-up check (`mod.rs:1000-1007`).
- **Timers** - no new sleeper; E5 `continue` cannot spin because `awake_since = now` (`main_thread.rs:803-805`).

## Open points (from `impl.md:198-221`, `:355-371`)

1. `47af1b1` fails the gate (tree of commit 12, red by one assertion); declared, history not rewritten.
2. `CLAUDE.md:72` and `.claude/agents/tester.md:101` still state 2.4 L2 for R16; chore after the merge (Q5).
3. No pre-existing warnings reported. Build dirs `D:/wt/_targets/ks25-probe`, `ks25-probe11`, `ks25-head` left in place
   (no registered worktree).
4. Commit-message wording not amended: `786e61f` "R12m2 is 1.1 to 1.4 us above it" (means above R12); `47af1b1` "the
   next commits set R16's gate" (only `e391a06`).
5. R16 figures in `README.md:955`, `watchdog/mod.rs:39-42`, `CHANGELOG.md:664-668` come from commit-11 runs, not a new
   HEAD run.
6. Row 88 and `keys.md:194` links pin `045b8b4`.
7. D9 deviation: `summary` returns `Option<Report>` (not `Option<(Level, String)>`).
8. Follow-ups for 3.5: Q11 (own-key skip keeps row 63's modifier case), M9 latch, property coverage of D10 (property
   passes under X4), FU-4 (likely moot).
9. Non-config book pages left for the phase book sync: `architecture.md:237, :248, :251, :265, :294, :331, :350, :362,
   :566-567, :605`; `glossary.md:67, :377, :569, :577` (+ "Hiding count"); `overview.md:16`; `gaps.md:167, :300`;
   `roadmap.md:584, :586-595, :654, :739, :952, :999, :1005` (`impl.md:293-332`).

## Not covered

- Did not run `cargo fmt`, `clippy`, `cargo test` or the bench; no gate result here is the scout's own.
- Did not re-run any mutation; "confirmed red" rows cite `impl.md`.
- Did not read: `cut.md` D1-D12/D15-D16 rationale text (`:254-1057`), W-table, Win32/thread map/touch set
  (`:1059-1182`), risks (`:1529-1585`), the three critiques; `src/engine/testing.rs` beyond `:384-500` and the step list;
  `properties.rs` beyond the mode/press helpers; `bench.rs` row bodies beyond `:484-560` and the docs; `writer.rs` beyond
  `:233`; README and book hunks beyond the cited lines; `src/engine/scenarios.rs` new scenario bodies (names and lines
  only, scripts per `cut.md:1325-1366`).
- Did not verify unsafe SAFETY wording in test code (`callback` tests `PeekMessageW`, bench) beyond `impl.md:223-234`.
