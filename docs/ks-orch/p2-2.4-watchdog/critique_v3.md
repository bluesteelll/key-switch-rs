# Architecture critique: p2-2.4-watchdog, delta after ruling L1 (round 3, delta pass)
Verdict: CHANGES_REQUESTED (Critical 0, Important 2, Minor 3)
Plan: D:/tmp/ks-orch/p2-2.4-watchdog/cut.md, section "Delta after ruling L1" (lines 749-1114), against rulings.md round 2 (lines 70-96)
Code read at: `feat/hook-watchdog` `0e7165b` (`.git/HEAD` -> `refs/heads/feat/hook-watchdog` = `0e7165b04943eb1bcb1326261869a50c34762d4f`). This role has no shell, so `git show` was not available. I read the checkout files, as the architect did (cut.md:751).

Read in full: L1.0-L1.15; rulings.md; src/hook/watchdog/{mod,decision}.rs; src/hook/health.rs; src/hook/installer.rs:120-180; src/platform/timers.rs; src/bench.rs:1-80 and :380-480; src/data/sequence.rs:100-220; test_r1.md (bench table); bench-dev.md; `windows-0.62.2` `QS_*`/`QUEUE_STATUS_FLAGS` (WindowsAndMessaging/mod.rs:5533-5584) and `GetRawInputBuffer`/`DefRawInputProc` (UI/Input/mod.rs:16-39). I spot-checked every L1.1 line reference, and all were right.

Answers to the brief, in short:
- Is the per-key wake gone while ticking? Yes, for the keyboard source, if E4 holds. The Learn text supports it more strongly than the cut says (see Strengths). M9 is the right live check.
- Can raw input be lost, merged or overflow so that the watchdog errs? Loss and merging only lower the evidence, which is the safe direction. **A backlog read over several ticks is not safe**, though: see I1, where a live hook is reinstalled.
- Do I1 (tick), M1, M5 and the decision table hold on batched counts? I1, M1 and M5 hold, and I traced them below. The decision-table equivalence breaks only when a drain hits its per-pass cap (I1).
- Microsoft Learn claims: every quote E1-E12 is verbatim and correctly attributed. E3's reasoning about burst size is wrong (I1).
- Can the bench rows show the D11 bound honestly? R14 is not like-for-like with its baseline R12 (I2). R16 is under-specified (M2).
- Do the red-first tests fail for the stated reason? Yes. F2, F4, F6 and F8b name symbols that do not exist at `0e7165b`, and `Ticker::pass` takes 2 arguments (mod.rs:208). F8a fails by assertion on F7's per-event loop.
- Is there new cost on the key path or the main thread? None. L1 changes only the watchdog thread, the bench and the docs.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. The capped drain lets a large injected burst trail its callbacks past the one-interval credit, and a live hook is reinstalled
Where: plan L1.2 (cut.md:804, `MAX_BUFFERED_READS = 64` with `RAW_BUFFER_BYTES = 64 * size_of::<RAWINPUT>()`, which gives 76 keyboard records per call and 4 864 per pass; cut.md:808, `MAX_STANDARD_READS = 4096`); L1.5 "No spin: ... or 64 x 76 / 4 096 records of it" (cut.md:913); L1.3 E3 "an injected `type_text` burst is a few hundred" (cut.md:833); L1.7 "a per-event delivery is a run of batches of one" (cut.md:961); L1.13 "reads what accumulated at its next pass, in parts ... the safe direction" (cut.md:1072); src/hook/watchdog/decision.rs:148-160 (`owed` holds this tick's callbacks and is spent and zeroed in the next interval); src/data/sequence.rs:123-139 (`type_text` puts two events per UTF-16 unit into ONE `SendInput`, with no length bound).

Problem:
- Before L1, an uncapped pump read every `WM_INPUT` within microseconds. A raw event therefore trailed its callback by less than one interval, and the one-interval `owed` credit absorbed it.
- Under L1, while the thread ticks, nothing wakes it for raw input, and each pass reads at most 4 864 records (4 096 on the standard read).
- Raw input that arrives faster than about 19.5 k events/s (4 864 per 250 ms) builds a backlog. That backlog is read over several ticks, after its callbacks were counted.
- The credit lasts one interval, so whatever is left after the second pass becomes evidence.
- The batched `Drive` hands every queued event to each pass (`take(&mut queued)`, cut.md:965). No planned test can see this.

Consequence (traced with the plan's own constants):
1. The setup: a `TypeText` binding of about 5 000 characters, or another tool's long `SendInput` string, injects 10 000 key events. Windows pushes them through the LL hook in about 0.2-0.3 s. Our hook alone allows up to about 64 k events/s: R7 round trip 10.7 us plus R12 callback 4.7 us (test_r1.md:86,90).
2. The idle wake reads the few events that have arrived and starts ticking.
3. At tick T1 the heartbeat has moved by about 10 000. The silence resets and `owed = 10 000`. The drain reads 4 864 and leaves about 5 136.
4. At T2 the drain reads 4 864. They are explained by `owed`, which then becomes 0.
5. At T3 the drain reads the last ~270. With `owed = 0` they become evidence.
6. The user does not type for a second, as after any macro. At T1 + 1 s `decide` fires, and the desktop check is true. The watchdog sends `ReinstallRequest { seen: <current heartbeat> }`.
7. On main, `reinstall_if_silent` finds the heartbeat equal to `seen` and runs `replace()`. **A live keyboard hook is reinstalled**, and the engine is reset (a pending Tap/Hold/DoubleTap is dropped). The log gets `[WARN] the keyboard hook got no key event for 1000 ms while keys were typed; reinstalled it`, and `swch status` reports `reinstalls: 1`.

This contradicts D3's "So the reinstall is not needless" (cut.md:147) and L1.13's "the safe direction". Reading a backlog in parts is the unsafe direction for the credit.

Confidence: CONFIRMED for the mechanism (plan constants, decision.rs:148-160, sequence.rs:123-139). The size that triggers it depends on the machine's injection rate. It is about 4 900 `TypeText` characters at 64 k events/s, and about 10 000 at 30 k/s. It also depends on two unknowns: whether Windows reports raw input for injected `KEYEVENTF_UNICODE` events, and whether `WM_INPUT` shares the 10 000 posted-message quota (E3). Floods of VK-based keys from other tools are reported (hDevice NULL; D6 counts them).
Breaks: D3 rule 3 and its consequences; ruling L1 ("the D3 decision table keep working on the batched counts"); E3's burst bound.
What is needed: a raw event must never be read more than one interval after the tick that counted its callback, or the decision must know that the batch is incomplete. Possible directions:
- a drain bound that cannot bind at any reachable event rate (bounded by time rather than by count), and if it binds anyway, a re-pass before the credit expires, for example by readmitting raw input to the next wait;
- carrying the unused credit while a drain reports input left;
- or no decision while a backlog remains.

E3's "a few hundred" must give way to the real bound, which `type_text` does not have. Pin the fix with a loop row: "a flood of 10 000 raw events whose callbacks were all counted at one tick, then silence, draws no request". The `Drive` must model the per-pass cap for that row.

### I2. R14's acceptance line compares a paced row that posts before each call with a back-to-back R12
Where: plan L1.6 R14 (cut.md:932-937: "spin until 1 ms after the previous sample began, then `post_synthetic_raw`. Inside the timing: `call_keyboard_callback`") and its rule row (cut.md:949: R14 against HEAD R12, p50 <= R12 p50 + 300 ns); src/bench.rs:416-428 (R12: back-to-back timed calls, no post); test_r1.md:90,93.

Problem: R14 differs from R12 in three ways, and only one of them is the watchdog:
- each call comes 1 ms after the previous one, so caches and branch predictors are colder;
- a `PostThreadMessageW` system call runs immediately before every timed call;
- and the watchdog ticks.

The tester's own R14/settled used the same construction with a 300 us pause, and the watchdog had finished with each event before the call. It measured p50 5.0 / 4.9 / 5.0 us against R12's 4.7 us (test_r1.md:90,93), which is +0.2 to +0.3 us. The construction alone spent the whole 300 ns budget before any work by the watchdog. Its p99 (73.7-153.5 us) is no clean control either: in that build every post still woke the watchdog.

Consequence: the tester measures R14 p50 at about 5.0-5.2 us and reports a breach of L1's acceptance line again. No row says whether the watchdog or the pacing caused it, so the result is another ruling round or a waiver without evidence. A pass is just as uninformative: it does not isolate the watchdog either.
Confidence: PLAUSIBLE. The size of the effect at 1 ms pacing is not measured; at 300 us it is.
Breaks: ruling L1, "R14 within D11's bound in steady typing is the acceptance line" (rulings.md:94-95), which cannot then be judged; the owner's lowest-latency decision, which needs the extra cost attributed.
What is needed: R14 must differ from its baseline only by the ticking watchdog. Options:
- a control row with the same pacing and the same post before each timed call, but with no thread draining that queue. Report it next to R14, apply the bound to R14 against the control, and report R12 too.
- a construction in which the timed thread does exactly what R12 does, while the posts come from another thread.

If the control alone exceeds R12 + 300 ns, that goes to the orchestrator before the tester's runs.

## Minor (improvements, not blockers)

### M1. M9 (and amended M2) state two different pass rules, and the expected wake count has no margin
Where: cut.md:1058 (M9 expects `W <= 130`, but the FAIL rule is `W >= K`), cut.md:1060 (M2: `W <= 40`).
Problem: the 130 counts about 90 wakes for the hold and at most 40 for typing the two `swch` commands. It leaves out three bursts that the run itself contains:
- the Enter key-up of `swch open` once the hook is live, about 9 wakes (one wake and 2 s of ticks);
- opening Notepad by keyboard (Win, `notepad`, Enter), about 17-21;
- switching back to the terminal by keyboard, about 10.

Consequence: a correct build can report W of about 145-150 with K of about 600. That fails "Expected" but passes the FAIL rule, so the owner sends an ambiguous result and the lane loses a round.
Confidence: CONFIRMED (arithmetic on the steps as written).
What is needed: one criterion that is robust to the setup, for example derived from K (one wake per key would give W >= K), or steps that exclude the extra bursts (open Notepad by mouse; start the daemon before the idle minute and say so).

### M2. R16 is under-specified, and its sanity bound is a max on 100 samples
Where: cut.md:938-942 (R16), cut.md:950 (its bound), cut.md:1006 (the README sentence in F9).
Problem:
- R16 does not say `health.set_hook_installed(true)`, which R14 does (cut.md:933). Without it, the timed first pass takes the I3 discard path (mod.rs:176-179, L1.5 pass step 2): no `raw_batch` and no `SetWaitableTimer`. The row would then understate the first key's cost, and "stop takes at most one tick because the watchdog now ticks" would be false.
- The p99 of n = 100 is the largest or second-largest sample. A single preemption above about R12 p99 + 100 us (about 107 us) fails the sanity bound. R14/settled already reached 153 us at p99 over 2 000 samples (test_r1.md:93).
- F9 writes "about 20 us slower" into the README before F10 measures R16.

Consequence: a spurious R16 "breach", or a number in the README that no row measured.
Confidence: CONFIRMED (plan text).
What is needed: R16 sets the hook as installed, as R14 does. The bound uses a percentile that 100 samples can support, or more samples. The README number comes from the tester's R16, or gives no number until then.

### M3. The synthetic source is not the keyboard source "with one queue bit swapped": the two bits clear differently
Where: cut.md:819 ("the tests and the bench therefore run the same wait, pump, drain and pass sequence as the daemon, with one queue bit swapped"); cut.md:1035 (the mutation "the drain only at due ticks" is said to be caught because "the idle spin makes `wakes` explode").
Problem: Learn says `QS_POSTMESSAGE` "is cleared when you call GetMessage or PeekMessage, whether or not you are filtering messages" ([GetQueueStatus](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getqueuestatus)). `QS_RAWINPUT` is cleared once `GetRawInputBuffer` has read all raw input (E6). So in the synthetic source, the pump's range-filtered peeks clear the batched bit while `SYNTHETIC_RAW` stays queued, and an idle wait does not return for it. Under the mutation, the synthetic thread wakes once per post, not in a spin. F8a still catches the mutation (wakes >= n + 1 > 12), but not for the stated reason.
Consequence: no automated test can see an idle spin on the keyboard source, for example a raw-queue item that `GetRawInputBuffer` does not drain. Microsoft's own note on `RIDEV_DEVNOTIFY` describes exactly that spin. Only M2 and M9 on the owner's desktop would show it, as a burned core.
Confidence: CONFIRMED (Learn text against cut.md:819,1035).
What is needed: state the difference in L1.2 and in the mutation table. Say that the keyboard-only spin is covered by M2/M9 (a spin shows as millions of wakes). Optionally add the guard in Open question 1.

## Earlier findings touched by L1 (critique.md round 1, as ruled in rulings.md)
- **I1 (tick never lost): still holds.** While ticking, the tick sources are the timer (`fired`, then `pass` and `arm` on every wake, mod.rs:208-223,278-303) and the `wake_by` timeout when a set failed (mod.rs:197-202). Raw input brought passes before L1 and no longer does; that was a convenience, not a source the rule relied on. `a_timer_that_refuses_once_is_set_again_at_the_next_pass` still ends in one request in [1000, 1500] ms on the batched `Drive`: the refusal comes at 500 ms, the timeout fires at 750 ms and the timer is set again.
- **M1: still holds.** `last_raw_at` is the instant of the drain, at most one `TICK` after the event. Ticks run at T1 + 250n, so ticking stops at T1 + 2 000 <= last key + 2 250, the same bound as before.
- **M5: still holds.** I traced `a_batch_is_evidence_only_from_the_next_tick`: the pass at 1250 reads 2 raw events, the evidence is 0 at the check, and the callbacks at 1251 and 1252 are seen at 1500, so there is no request. Swapping the order gives a request at 1250 (quiet_since 250 + 1 s), so the row is red for the right reason.
- **Rule 4 / `raw_events_read_with_an_answer_are_no_evidence_after_it`: holds.** Under the order mutation, the second request falls at 3250 ms, as the cut predicts (cut.md:960).
- **Spacing row:** requests at 1000, 3250, 7500, 15 750 and 32 000 ms, with gaps of 2250, 4250, 8250 and 16 250 ms, each in [2^(k+1) s, 2^(k+1) s + 500 ms). Verified.
- **I3 order:** unchanged. The keyboard source keeps `QS_POSTMESSAGE` in both masks, so the daemon's `WM_QUIT` still wakes it at once.

## Strengths to keep
- Every Learn quotation I checked is verbatim and correctly attributed:
  - E1, E6, E11: [GetRawInputBuffer](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getrawinputbuffer).
  - E1 (sample), E7, E10: [Using Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/using-raw-input). The pump ranges `(0, WM_INPUT - 1)` and `(WM_INPUT + 1, 0xFFFF)` are exactly the sample's.
  - E4, E5, and the fallback's handle-closing sentence: [MsgWaitForMultipleObjectsEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex).
  - E7, E8: [Raw Input Overview](https://learn.microsoft.com/en-us/windows/win32/inputdev/about-raw-input).
  - E9: [DefRawInputProc](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-defrawinputproc) and [WM_INPUT](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-input).
  - E12: [WaitMessage](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-waitmessage).
  - E3: [PostMessageW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-postmessagew).
- Two Learn facts can strengthen E4 and E9:
  - E4: GetQueueStatus defines `QS_POSTMESSAGE` as "A posted message (other than those listed here)", and `WM_INPUT` is listed under `QS_RAWINPUT`. So the ticking keyboard mask, which keeps `QS_POSTMESSAGE`, does not let raw input back in through that bit. And F1 shows that the mask is honoured under `MWMO_INPUTAVAILABLE`.
  - E9: the WM_INPUT page requires `DefWindowProc` only for `RIM_INPUT`. `RIM_INPUTSINK`, which the sink receives whenever the daemon does not own the foreground, has no cleanup requirement.
- The `windows` crate facts are right: `QS_ALLINPUT` = 1279 = 0x4FF, so `QS_ALLINPUT & !QS_RAWINPUT` = 0xFF (WindowsAndMessaging/mod.rs:5534,5545,5579-5584), which F4 pins. `GetRawInputBuffer(Option<*mut RAWINPUT>, *mut u32, u32) -> u32` (UI/Input/mod.rs:31).
- `count_raw_keys` walks plain bytes in safe code, so no padding is read, and its step matches NEXTRAWINPUTBLOCK on an 8-aligned buffer. 32-bit builds take the standard read, so the WOW64 layout never applies.
- Ordering `raw_batch` before `take_answer` keeps rule 4, with a red row for the mutation. The `RIDEV_DEVNOTIFY` scan pins Microsoft's own spin warning.
- The fallback (a kernel wait while ticking) is named, together with its handle-lifetime rule.
- No new shared state, atomics or orderings. `LoopStats` crosses threads only through the `done` channel. Nothing new runs on main or in the callback.

## Open questions for the architect
1. **A guard against an idle spin.** If `GetRawInputBuffer` ever returns 0 while `QS_RAWINPUT` is still set, the idle wait (`QS_ALLINPUT` + `MWMO_INPUTAVAILABLE`) returns at once, forever. Learn's `RIDEV_DEVNOTIFY` note names one such item. A cheap self-healing step after a 0 return: check `GetQueueStatus(QS_RAWINPUT)`, and fall back to the standard read, which removes each message. Is it worth one call per drain? Check: M2's stop line (a spin shows as millions of wakes).
2. **WM_QUIT in the standard read.** The standard read filters by `Some(window)` (cut.md:808). Learn's "PeekMessage always retrieves WM_QUIT messages, no matter which values you specify for wMsgFilterMin and wMsgFilterMax" covers the range, not `hWnd`. A thread-posted `WM_QUIT` has a null `hwnd`, and a filtered peek clears `QS_POSTMESSAGE` (GetQueueStatus). In 32-bit builds, or after a failed buffered read, a `WM_QUIT` that arrives during that read could leave the idle thread asleep, and `stop` would time out with a false warning. `hWnd = None` changes nothing else, because only the sink receives `WM_INPUT`. Check: PeekMessageW's hWnd paragraph.
3. **The smoke test in F8b.** Does `GetRawInputBuffer` return 0, rather than `(UINT)-1`, in a process with no raw input registration? Learn does not say. If it returns -1, the test fails at F9 for a reason unrelated to the design. Check: run it once at F7 and record the result.
4. **Backlog and the USER object quota.** Whether pending `HRAWINPUT` objects count against the per-process USER object quota is not on Learn. It matters only if I1's backlog is kept rather than removed.

## Absent topics (walked, not applicable to this delta)
- Matching model, suppression lifetime, gestures, config and hot reload, `SendInput` and the sentinel, process boundary and DACL: L1 changes none of them (L1.13, verified against the touch set, cut.md:998-1011).
- Thread affinity: every read runs on the queue's own thread (E6). Install, uninstall and reinstall stay on main.
- Module cycles: `decision.rs` stays pure (`#![forbid(unsafe_code)]`), and the watchdog uses only `logging::counters` and `platform::timers`.
- Main-thread budget: no change. The only per-key cost added on any of our threads was the watchdog's wake, which L1 removes while ticking.
