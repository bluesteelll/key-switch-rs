# Architecture critique: p2-2.5-latency-trim / roadmap 2.5 Latency trim (round 2)

Verdict: CHANGES_REQUESTED (Critical 0, Important 2, Minor 5)
Plan: D:/tmp/ks-orch/p2-2.5-latency-trim/cut.md (revised)   Rulings: rulings.md   Code read at: master `50b56a8`

Rulings W1, W3, W4 and M1-M4 are carried out. M3 and M4 carry the caveats below. On the stuck-key and lone-key-up
side, W2 holds:

- I traced Lemma 1 and Lemma 2 (cut:494-505) against `keys.rs` and `mod.rs:823-899`.
- I traced D10 against commit 3's scenario.
- I traced D15 against eight existing scenarios.

I found no counterexample. Both Important findings sit in the matching-residual proof that ruling W2 made binding.
One is a stale state that no resync repairs. The other is a W-table row whose argument is inverted. Neither can make a
key stick. Both make row 88's text wrong.

## Critical (implementation must not start)

None.

## Important (must be resolved)

### I1. The drain's read never repairs a generic modifier slot, so a generic phantom outlives every resync

Where (cut):

- Type sketch :138: `resync_modifiers`, "never a generic code".
- Type sketch :140: `unswallowed_modifier_held`, "a sided modifier".
- D3 :241-251.
- D7 :361-363 (only a sided modifier counts as held).
- W-table W5-W7 :304-306.
- Row 88 :735-739: "keeps its old state for the daemon until the main thread next reads the modifiers from Windows".
- D11 :511: "all bounded by E0-E8 or D7".

Where (code):

- `src/engine/keys.rs:30-42`: today's `MODIFIERS` includes `VK_SHIFT`, `VK_CONTROL` and `VK_MENU`.
- `keys.rs:102-110` and `src/engine/mod.rs:522`: the per-event sync overwrites those slots.
- `keys.rs:246-250`: `AnySide` counts the generic slot as the family being held.
- `mod.rs:1289-1316`: the project already assumes that injectors send generic codes.

Problem:

- After commit 7, a generic slot (0x10-0x12) changes in only three ways:
  - an event under that code;
  - `resync_from_os` (`swch on`, a reinstall), which sets it to false;
  - `release_all` (input left this desktop).
- The drain's read never touches it. That read serves E1, E4, E5, E6, E7 and E8.
- D7's "held" test looks only at sided slots, so it never fires for a generic slot.
- So a generic key-up that the engine did not decide leaves the slot set. Three such cases:
  - a call that timed out, W5;
  - a call dropped while main was busy, W6;
  - a refused call, W7.
- The hiding count moves and the read runs, but the phantom stays.

Consequence:

1. Another program injects the generic `VK_SHIFT` down and up.
2. The hook call for its key-up is refused or times out. Windows applies the key-up.
3. E8 or E4 moves the count. The drain reads the eight sided codes, which are all up, and leaves `active[VK_SHIFT]` set.
4. From then on every key matches with the Shift family:
   - with the default config, CapsLock fires `ToggleCapsLock` instead of `SwitchLanguage`;
   - every exact binding without Shift misses.
5. This lasts until one of these happens:
   - that program presses Shift again;
   - `swch off` and `swch on`;
   - a hook reinstall;
   - a lock.

Master's sync clears the slot at the next key event, because `GetAsyncKeyState(VK_SHIFT)` reads up. No key sticks.

Confidence: CONFIRMED for the plan's logic (lines above). The trigger, a generic key-up that the engine did not
decide, is rare.

Breaks: ruling W2's bound "one binding misfires or misses until the next resync"; the text of row 88; D11's claim that
every stale state is bounded by E0-E8 or D7.

What is needed: either of these two.

- **Let the drain clear a generic slot, but never set one.**
  - [GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate)
    says the generic code "gives the state of the SHIFT, CTRL, or ALT keys without distinguishing between left and
    right". So "up" means neither side is down. A clear-only read therefore does not bring back the phantom that D3
    rejects.
  - Let a held generic slot count for D7.
  - Keep the KS-14 skip and the D5 rule for fresh keys.
- **Declare the case without a bound** in row 88 and D11, with its real end conditions.

Pin whichever you choose with a scenario: a generic key-down event, an unnoticed generic key-up,
`hidden_input_noted()`, then CapsLock.

### I2. W10 is argued backwards: a modifier the daemon passed and a later hook swallowed stays held for the engine for the whole hold, and this is not declared

Where (cut): W-table W10 :309 ("Not needed: Windows does not apply it either"); D1 :190-193; D7 :365-369; D11
:511-522; row 88.

Where (code): `src/engine/mod.rs:518-522` (master syncs before matching); `src/engine/keys.rs:158-172` (`press` and
`release` set the flag when the event is decided).

Problem:

- [Hooks Overview](https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks) says: "The SetWindowsHookEx
  function always installs a hook procedure at the beginning of a hook chain ... Each hook procedure in the chain
  determines whether to pass the event to the next procedure".
- So a hook installed before the daemon's runs after it. So does a hook installed before the daemon's last 2.4
  reinstall, which calls `SetWindowsHookEx` again.
- When such a hook swallows an event the daemon passed, Windows does not apply the event. The engine already applied it
  when it decided the event.
- Master's per-event sync repairs this before the next match. After D1, nothing does:
  - no source moves the hiding count;
  - D7 cannot keep the repair, because the hook sees every auto-repeat of the held key, passes it and sets the flag
    again.

Consequence:

- **AutoHotkey remap.** AutoHotkey runs a `LAlt::LCtrl` remap and started before the daemon. Its hook swallows LAlt and
  injects LCtrl, which the daemon decides as an event.
  1. The user holds the left Alt key, meaning Ctrl, and presses J.
  2. The engine sees Alt and Ctrl held, so a daemon binding `Ctrl+J` (exact) never matches during that hold.
  3. Master matches it every time.
- **Default Alt+Shift blocker.** The same happens with a held LShift that a later hook swallows.
  1. A LAlt press matches the blocker, because `blocker_matches` asks Windows only about keys outside the hotkey
     (`src/engine/matching.rs:93-102`).
  2. The whole Alt press is swallowed, and Alt+Tab reaches Windows as a bare Tab.
- **Other plausible later hooks** that swallow keys: Win-key blockers, and the keyboard hook of a full-screen Remote
  Desktop client.
- **Frequency.** With such a hook in place this happens on every press, not as a rare race. No key sticks.

Confidence: CONFIRMED for the mechanism (Microsoft Learn, the plan and master's code). How often the owner's machine has
such a hook is not known.

Breaks:

- ruling W2: the engine tracks the flags from events "where they change Windows' state", and "declare the residual
  matching misses in known issues";
- D11's list of residuals;
- it is a regression against master.

What is needed:

- Correct W10.
- Then either declare the case or bring a mitigation.
  - To declare: write it into row 88, D11 and the lane report to the owner, with its real bound: "for as long as the
    key is held".
  - To mitigate: bring a between-events mitigation, argued against W2.
- Note that a read at the drain after the modifier's own event comes after the next key is decided, so it does not
  repair the shortcut. D5 also leaves that event's key out of that read as fresh.

## Minor (improvements, not blockers)

### M1. "Matching only" understates two stale-flag effects the user would see

Where: Goal 3 (cut:73-74, "stale flags affect matching only"); D11 :511-519; row 88; rulings Q2 ("the residuals are
matching misses only").

Problem and consequence:

- (a) D11 item 3 (:516-517). A mask that is not sent while Windows has Alt or Win down leaves Windows with a lone Alt
  or Win press. The menu bar or the Start menu then opens, which is KS-12's symptom.
- (b) The Alt+Shift auto-blocker on a phantom Shift (or Alt) swallows a whole ordinary Alt (or Shift) press.
  - Alt+Tab becomes Tab, and a capital letter comes out in lower case.
  - The press's own record keeps it swallowed (`mod.rs:839-847`) even after D7 has healed the phantom.
- In the abstract both are "a binding fires" or "the mask is not sent". Row 88 does not say what the user sees, and
  the owner is told "matching misses only".

Confidence: CONFIRMED (cut :516-517; `matching.rs:93-102`).

What is needed: row 88 and the lane report name these two visible effects, and Goal 3 drops "matching only".

### M2. S4 and S5 do not fail on commit 6's tree, and S9 does not catch X10

Where: cut :850-853, :873-874; mutation X13 :1088, X10 :1085; S9 :859-862.

Problem:

- **S4 and S5** expect `ToggleCapsLock`, with LShift held.
  - On commit 6, `on_key` still calls `sync_modifiers` (`mod.rs:522`).
  - `down_unnoticed` sets `os_down` (`testing.rs:119-126`), so the sync reads LShift down: at CapsLock's own event in
    S4, at F13's in S5.
  - Both scenarios therefore pass under X13. They pin X5 and X11, not X13.
- **S9** asserts only suppression. Under X10, the read ignores KS-14:
  1. The read sets LShift up.
  2. The second LShift down is a first press. It is still suppressed (`block_default: true`).
  3. `DoNothing` fires, which S9 does not check.

Consequence: the tester runs X13, sees S4 and S5 green and records the cut's prediction as wrong. S9's entry in the
X10 row catches nothing.

Confidence: CONFIRMED by trace.

What is needed: X13's list becomes `a_key_event_asks_the_os_about_no_modifier`, S1 and S2. S9 asserts
`expect_nothing_fired()` on the second LShift down.

### M3. The deferral adds a wake only for a burst made of one raw event, not for every burst (this corrects my round-1 M3)

Where: D12 Cost :577-580; touch set `errors.md:214-215` ("about 11", "about 1100"); M6 :972 ("about 48"); M7 :973
("about one higher"); CHANGELOG text (11) :937-940.

Problem:

- `last_raw_at` is the time of the drain that read the newest raw event (`decision.rs:142-143`).
- The waking pass reads only what is queued microseconds after the first raw event.
- Every later raw event of a key press is read at a tick in both designs. The key-up alone comes tens of ms later.
- So for every key press and every shortcut, three things stay the same:
  - `last_raw_at`;
  - the end of ticking (`decision.rs:227-234`);
  - the wake count.
- Only a burst whose last raw event is read at the waking pass gets one more tick. Examples: a lone key-up whose
  key-down came 2 s or more earlier, or the Drive test's single `raw(0)` (`watchdog/mod.rs:1607-1628`).
- The cut's own moved expectation shows this. In `a_batch_read_at_a_tick_counts_all_its_events`, (250, 8) becomes
  (250, 9): the first event moves to the first tick, and the end does not move.
- My round-1 M3 generalised from that single-event test.

Consequence:

- `errors.md:215` would publish "about 11" and "about 1100", where its current "about 10" and "about 1000" stay right.
- The expected values of M6 and M7 are off by the wake the cut claims.

Confidence: CONFIRMED (`decision.rs:138-149`, `:224-234`; `watchdog/mod.rs:1631-1634`).

What is needed: keep the numbers at `errors.md:215`. The D12 Cost, the module Cost doc, M6, M7 and the CHANGELOG
entry should say "one more tick only for a burst made of a single raw event, such as a key-up whose key-down came 2 s
or more earlier".

### M4. Some compile-red tests still have no named mutation (round-1 M4, partly open)

Where: the mutation table :1072-1101, which says it covers "every compile-red test of commits 5, 8 and 9".

Problem: commits 4, 5, 7 and 11 add tests of new API that no listed mutation covers:

- commit 4:
  - `a_repeat_suppressed_record_is_consumed_like_a_swallowed_one_and_kept_by_a_reset`;
  - `a_repeat_suppressed_key_is_not_skipped_by_the_modifier_resync_and_counts_for_the_mask`;
  - `a_repeat_with_no_record_that_passes_after_a_swallowed_one_is_recorded_as_passed`;
- commit 5: `swch_on_and_a_reinstall_read_everything_themselves_so_the_next_drain_reads_nothing`;
- commit 7: `an_injected_event_of_a_key_that_is_not_a_modifier_changes_nothing`;
- commit 11:
  - `a_deferred_first_read_starts_the_silence_at_the_wake`;
  - `a_deferred_wake_is_followed_by_a_wait_that_leaves_only_the_batched_input_out`.

Consequence: tester.md:178 ("Every compile-red test is covered by a mutation") leaves uncovered rows.

Confidence: CONFIRMED.

What is needed: one mutation for each. Examples:

- `resync_from_os` drops `RepeatSuppressed`;
- `key_down_swallowed` counts `RepeatSuppressed`;
- `reset` skips `read_everything`;
- `track_injected` tracks every code;
- `raw_pending` keeps the old `quiet_since`;
- the deferred wait keeps the batched bit.

### M5. Two bench gates do not show what the ruling asks

Where: D13 :586-588, :596-603, :613-614.

Problem:

- (a) **R1.** The gate is "HEAD mean ≤ parent mean + 0.005 us".
  - It passes when nothing changed, so it cannot "show the reads gone" (ruling W2).
  - D-R1n is recorded and then not used.
- (b) **R12 and R4.** These gates compare two numbers measured in different sessions:
  - the tester's median of three runs, in one session;
  - D-R12n or D-R4n, from one developer run in another session.
  - The margin is 100 ns, on a row of about 0.5-1 us.
  - Every earlier lane compared parent and HEAD from the same session.

Consequence:

- (a) If the removal were reverted on the engine path (`BenchEnv`), R1 would still pass.
- (b) Drift between the two sessions can show as a breach that the code did not cause.

Confidence: CONFIRMED for (a); PLAUSIBLE for (b).

What is needed:

- Gate R1 against D-R1n, or require a drop.
- For R12 and R4, either have the tester apply the one-line probe to the parent in its own session, or restate the
  bound as a same-session comparison.

## Previous findings

- **I1 RESOLVED** (ruling W1).
  - The witness reads at every swallowed key-up.
  - `up_since`, the stamp, the old hiding gate and the old residuals are gone. I searched the cut for leftovers and
    found none.
  - R11's rule is now the witness term.
- **I2 RESOLVED** (ruling W4).
  - `hook_late` is only a trigger.
  - An `LLKHF_INJECTED` event has age 0, and so does a stamp ahead of the clock.
  - A line whose only non-zero count is `hook_late` is written at `[INFO]`. A truly slow callback still gives `[WARN]`
    through the 5 ms gauge (`counters.rs:102`, `:207-211`), which is right.
  - M8 and U1 were added.
  - I checked the GetTickCount resolution quote on
    [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-gettickcount).
- **M1 RESOLVED.** E5 now `continue`s to the drain, and R-M1 is in row 88.
- **M2 RESOLVED as asked.** W8, W9, D2, D3, the tests and feature 6 are in place. Following its generic-code thread
  further gives the new I1.
- **M3 RESOLVED as worded.** But my wording over-generalised; see the new M3.
- **M4 PARTLY OPEN.** See the new M4.
- **Open questions 1 and 2 of round 1.** They no longer decide whether a key sticks, as the cut says. Agreed.

## Strengths to keep

- **W1 is carried out cleanly.** The non-goals, the acceptance mapping and the R11 rule all say "witness unchanged".
- **D10 and commit 3.**
  - Commit 3 fails on `50b56a8` with the predicted message, at the lone-key-up check (`testing.rs:644-657`). Traced:
    master's sync skips LShift at its own event; the Alt+Shift blocker swallows the repeat that has no record; nothing
    is recorded; the key-up passes while FakeEnv has LShift up.
  - The commit-6 red test counts 44 asks as predicted.
  - The commit-10 red test fits `Drive` (`reads`, `raw_wakes`, `watchdog/mod.rs:1383`, `:1377`).
- **Lemmas 1 and 2 hold as far as I could test them.** I tried these cases and found no counterexample:
  - every record state with a phantom-held and with a phantom-released flag;
  - an injected key-up of a modifier the user still holds;
  - `RepeatSuppressed` through `resync_from_os` and through `release_all`;
  - a refused key-down;
  - the KS-14 skip in `track_injected`.
- **D15 keeps the meaning of existing tests.**
  - Noticed unseen steps keep the outcomes of these scenarios, which I traced one by one:
    - `scenarios.rs:189-203`, `:1011-1058`, `:2121-2148`, `:3005-3025`;
    - `:3203-3233`, `:3526-3549`, `:5154-5167`, `:5543-5557`.
  - In the existing property modes the flags equal FakeEnv at every decision, because each unseen step's drain reads
    with nothing fresh. So their checks, the mask rule included, are unaffected.
- **D5** rests on the LowLevelKeyboardProc note about the asynchronous state. It does not rest on an unverified
  ordering claim.
- **D6** keeps principle 6 (no OS read while input is away), and a missing return signal costs only one decided event.
- **D8** reads its stamp once at exit and uses QPC for the run time.
- **D12.** D-R16d's stop rule and the HEAD gate both bound the wake's interference at 20 us, so they agree. D-R12n has
  a stop rule too.
- **Microsoft Learn quotes.** I checked the GetAsyncKeyState quotes the cut uses: the UIPI failure case, the inactive
  desktop, and the generic codes.

## Open questions for the architect

1. **U4: does an injected generic `VK_SHIFT` reach the LL hook under 0x10?** The answer sets how likely I1 is, not
   whether it is correct.
   - How to settle it: a debug daemon logs one counter for generic codes. Then type with `osk.exe` or an AutoHotkey
     `SendInput {Shift down}`.
2. **Can Windows change a modifier's async state with no LL hook event?** StickyKeys releasing a latched Shift is the
   example. If it can, the W-table needs a row: the phantom lasts until D7.
   - Manual check: turn StickyKeys on, tap Shift, type a letter, then press CapsLock. The layout should switch.
3. **The stamps of input that reaches this machine as a Remote Desktop server.** These are physical-looking events
   whose stamp may not be on this machine's tick count, a variant of U1. M8 covers local typing only. A failure there
   costs extra reads and an `[INFO]` line, nothing more.

## Absent topics (walked, not applicable)

- **§1 Hot path.** The 11 reads go. What is added:
  - per physical callback: two field copies, one `GetTickCount` and two compares;
  - per sentinel modifier: one `with_main` and one store.

  No allocation, lock, `log::` or post is added. `code < 0`, the heartbeat and the remapping gate keep their order.
  Every path still ends in exactly one outcome. No new panic site: `slot()`, wrapping age and `try_with`.
- **§2 Threads.** No install or uninstall moves, and no new thread. The drain read runs on main, under the borrow, and
  pumps nothing. `GetAsyncKeyState` dispatches nothing.
- **§6 Purity.** `engine::resync` reads the clock only through `Env::now`. `hook` already depends on `engine`, so no new
  dependency cycle appears.
- **§7 Config.** No schema change. `default_config.ron` is untouched.
- **§8 Shared state.** `HIDDEN_INPUT` is a main-thread TLS `Cell`. `unseen_input` uses `AcqRel` for the increment and
  `Acquire` for the load. No `Relaxed`.
- **§9 Process boundary.** None.
- **§10 Input synthesis.** No new `SendInput`. Every daemon injection still pairs its key-down and key-up in one call
  (`windows_actions.rs:258-287`, `sequence.rs:163-212`, `system_function.rs:125-155`).
- **§11 Fit.** `HookLate` gets its `Display` arm, its `ALL` entry, its slot and its `warns` arm. The new windows feature
  adds no crate.
