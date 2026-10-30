# Phase 3 plan draft: input correctness and the core

PLAN DRAFT READY: 8 lanes in order, 10 owner questions, 6 open points no scope report assigned (all now placed or
deferred with a reason). Missing input: `research-3.3.md` was never written; the 3.3 spec makes a researcher report the
first step of that lane.

- **Read:** master `252daef` (working tree on `docs/book-2.5`; `git diff --stat 252daef -- src` is empty, so `src/` was
  read on disk with Read/Grep). `book/`, `README.md` and `CHANGELOG.md` were read only with `git show 252daef:<path>`.
  No cargo command, no build, no edit in the repository. The owner's `daemon.log` was read, read-only, to check one
  claim (section 7).
- **Inputs:** `<orch>/phase-3/scope/{3.1-input,3.2-seq-chord,3.3-launch,3.4-core,3.5-budget}.md`; roadmap,
  audit, known-issues, gaps and architecture at 252daef; lane reports under `<orch>/*/` read with grep and
  ranged reads.
- **Rule of this draft:** the code wins. Every scope claim marked "verified" in section 7 was re-read on 252daef; claims
  not listed there are taken from the scope reports with their citations.

## 1. Completeness: every open point and where it goes

Lane names are short here: **bench** = p3-3.5-budget-bench, **ground** = p3-3.4-groundwork, **seq** =
p3-3.2-seq-chord-matching, **input** = p3-3.1-input-module, **held** = p3-3.1-held-modifiers, **launch** =
p3-3.3-launch-de-elevate, **compiled** = p3-3.4-compiled-bindings, **zero** = p3-3.4-zero-alloc. **Sync** = the Phase 3
book sync (doc-writer). Roadmap lines are `book/src/roadmap.md` at 252daef.

### 1.1 The roadmap's Phase 3 items (roadmap :662-739)

| Part | Status at 252daef | Goes to |
|---|---|---|
| 3.1 What and acceptance (:676-682): one `SendInput`, `wScan`, extended flag, `Text` `\n`/`\t` | Open: six `SendInput(` lines in four files (Grep), not "five in three" | input |
| 3.1 release and restore of held modifiers, the `0xE8` mask after Alt/Win, `(Left,[LAlt])` line | Open (KS-19) | held |
| 3.2 What and acceptance (:688-694) | KS-30/KS-32/KS-07 done (`44f79ac`, `82f542f`); KS-15, KS-16 open | seq |
| 3.3 What and acceptance (:700-706) | Open (KS-09, KS-34) | launch |
| 3.4 W1 `KeyState`, W3 `thread_local!`, W4 `forbid(unsafe_code)` | Done (W1 `16a0461`/`82f542f`, W3 `283d8aa` with its counter in `bf549c0`, W4 `16a0461`) | none |
| 3.4 W2 `CompiledBindings`, `by_vk`, `os_guards`, generation, diagnostics | Open (duplicates warning done in `ece1767`) | compiled |
| 3.4 A1 `rg "Mutex\|thread::\|unsafe" src/engine` | Cannot pass literally (section 7, V3); replaced by a scan test | ground |
| 3.4 A3 10,000 scenarios | Partly: 1,152 symmetry scripts (SEEDS 64, 9 configs, 2 modes) | ground |
| 3.4 A4 counting allocator | Open | zero |
| 3.5 (a) one name query per window, (b) Cyrillic `TitleEquals` | Done (`3befec7`, `4177540`) | none |
| 3.5 (c) under 100 us with 50 bindings | Open: no bench row has more than 7 bindings | bench (rows, gate); every later lane re-checks |
| 3.5 `when:` after match for `sequence`/`chord` (:737, :739) | Open (`src/engine/mod.rs:860`, :892) | seq (section 2, C1) |
| 3.5 F3, the foreground race of known issue 80 (:992) | Open, unmeasured | bench (R17f evidence); decided: kept declared (section 5) |
| 3.5 the witness skip (:996) | Lapsed: 2.5 ruling W1 keeps the read (`p2-2.5-latency-trim/rulings.md:13-17`) | Sync closes the row |
| 3.5 FU-4 (:999) | Moot: 2.5 ruling L3 gates R16; tester R16 0.9 us (`p2-2.5-latency-trim/results.md:108`) | Sync closes the row; manual check 44 may reopen it |

### 1.2 Follow-ups from Phase 1 (roadmap :959-972)

| Row | Goes to |
|---|---|
| Lone Alt/Win gesture opens menu or Start (known issue 66) | held, item 9, if owner Q3 is yes |
| Modifier-only Hold/DoubleTap output under its modifier (known issue 68) | held |
| Mask per swallowed event at the repeat rate | held, item 8 (per press, AutoHotkey's rule) |
| No test pins mask before actions | input (ordered test seam) |
| Mask sent with scan 0, no extended flag (KS-36) | input (closed as by design: 0xE8 has no scan code) |
| DoubleTap keyed by vk across bindings | Closed in `283d8aa` |
| Winner re-elected on every repeat (known issue 73) | compiled, item 5, owner Q8 |
| Immediate vs deferred on the same keys, no warning (known issue 69) | compiled, warning (a) |
| OS key state for a passed key | Closed in `3befec7` |
| Harness checks no timer fires on repeat steps | ground, item 5 |
| `swch open` does not wait for the daemon | Deferred: 4.3/4.5 (as the row says) |
| `Local\key-switch-rs` squatting | Deferred: 4.1 |

### 1.3 Follow-ups from Phase 2 (roadmap :978-1012)

| Row | Goes to |
|---|---|
| A blocking step (`Launch`, `OpenUrl`, a shell dialog) holds the executor (:980) | launch, item 5 (measure R25/M6, declare; launcher thread is a cut fork) |
| `LockWorkstation`/`ShowDesktop`/`TaskManager` fast/slow placement (:981) | Deferred: new feature 1 (unreachable from config) |
| R-7, wake to a reused thread id (:982) | ground, item 4 (comment; close: moot by code and Win32 semantics) |
| Executor `MWMO_INPUTAVAILABLE` unpinned; one shared wait and pump (:983) | ground, items 1-2 |
| `Window(Restore)` under `ShowWindowAsync` (:984) | Deferred: Phase 2 manual check 12 decides |
| Engine clock is `Instant::now()`, not `KBDLLHOOKSTRUCT.time` (:985) | Deferred, no item. Reason: the timer and the late key-up race on main whatever the clock; R8 p99 under full load is 99-290 us (`p2-2.5-latency-trim/bench/H1.md`) against terms of 100 ms and more; the owner's log shows a longest callback of 4.2 ms in 4.5 days of use (section 7, V8) |
| No warning for unequal Tap and Hold terms (:986) | compiled, warning (b) |
| `MainThreadHandle::reload` repeats `set_enabled` (:987) | ground, item 3 |
| Three Closed rows (:988-990) | none |
| F1, `mark_completed` marks a held key with no record (:991) | seq, item 4 (delete the function) |
| F3 (:992) | see 1.1 |
| F4 console/remote disconnect (:993) | Deferred: 4.1 |
| F5/FU-2 no switch-away event (:994) | Deferred: Phase 2 manual checks 24 and 28 decide |
| F6 retry `WTSRegisterSessionNotification` (:995) | Deferred: 4.5 |
| Witness skip (:996) | see 1.1 |
| FU-1, uninstall guard in `App::run` (:997) | ground, item 7. **No scope report assigned it** (section 1.6) |
| FU-3 (:998) | Closed |
| FU-4 (:999) | see 1.1 |
| Panic hook without fallback (:1000) | Deferred: 4.2, which relies on the panic line |
| 'Never two hooks or none' wording (:1001) | Closed as is: the comment already says "exactly one hook of ours" (`src/hook/keyboard_hook.rs:266-267`) and the CHANGELOG "two hooks of the daemon" (`CHANGELOG.md:194`); Sync closes the row |
| Lexical installer scan (:1002) | ground, item 8. **No scope report assigned it** |
| `HookWatchdog::start` 5 s window (:1003) | Deferred: cosmetic, as the row says |
| Not-verified rows Q5, Q6, Q7, Q8, E2-E4, E9, E11, `KEYEVENTF_UNICODE` (:1004-1012) | Deferred: each names the manual check that settles it; all fall back to a safe behaviour |

### 1.4 Lane residuals and notes (2.5 and earlier)

| Point (source) | Goes to |
|---|---|
| Every injection keeps the sentinel (2.5 results :264-265) | input (scan test), held |
| A held injected modifier released on `off`, reload, Win+L (2.5 results :265-266) | Deferred: new feature 6, which holds modifiers |
| Flaky `core::executor::tests::a_delay_ends_within_a_few_ms_of_its_deadline` (2.5 results :272-273; failed in `gate_test_final.log:421`) | ground, item 6 (the first lane that touches `core/executor`) |
| Q11, the own-key skip (2.5 impl.md :357-358) | Decided: kept declared (section 5, D5) |
| M9, StickyKeys latch (2.5 impl.md :341-345) | Deferred: the owner's manual check; a difference becomes a fix in the next lane open on `engine/keys.rs` |
| D10 property coverage (2.5 results :174) | ground, item 5 |
| U1, the `KBDLLHOOKSTRUCT.time` base (2.5 results :175-177) | Closed by field evidence (section 7, V9) |
| P1 README drift, P2 rustdoc warnings (2.5 results :185-202) | Closed in `28566a4` (verified: `src/engine/modifiers.rs:30`, :127 now plain code text). The 3.5 scope's F-15 is stale |
| P3 R16 figure (2.5 results :204-208) | The running 2.5 book sync |
| R16m, the first key after idle with a held modifier (2.5 results :269-271) | bench, optional row |
| Known issue 88, the 2.5 residuals | Declared; no lane |
| 2.1 N1, active macro runs uncapped (p2-2.1-executor/rulings.md:110, "revisit with 3.5") | Deferred: known issue 36 declares it; off the hook path; a cap is a behaviour choice with no owner request |
| 2.1 "3.5: `Binding.action: Arc`" (results.md:162) | zero, item 1 (launch adds the payload `Arc` first) |
| 2.2 R1-R3 at the `Instant` floor (results.md:191) | bench, R19e batch-timed |
| A1/A2 "move `BindAction` into `data`" (p0-A1-engine/results.md:103) | Deferred: a chore after Phase 3; 21 files, no acceptance line needs it, it would conflict with every lane in flight |
| 1.1 re-run the leak-form mutations after an arms refactor (results.md:103) | seq (tester), and the testers of compiled and zero |
| 1.2 m1, `MaskOnly` warning before the KS-18 skip (results.md:60) | Closed: the warning is pushed only where the blocker is added (`src/config/loader.rs:400-411`) |
| 1.3 F2 / gaps `block_default` ignored on `sequence`/`chord` (gaps.md:169) | compiled, warning (d). **The 3.2 scope deferred it to 3.4 and the 3.4 scope did not take it** |
| 1.6 F1, `sequence`/`chord` in the winner election (cut.md:478, :487; matching.md section 16.2) | Deferred: by design and documented in section 16.2; a cross-kind precedence rule is a semantics choice new feature 8 (longest match) meets anyway. **3.2 deferred it to 3.4 and 3.4 did not take it** |
| 1.7 log folder hardening (gaps.md:235) | Deferred: 4.1 (it is `src/logging`, not the launch path; moot if the daemon stops running elevated) |
| 3.3 scope F-i: config write still reaches admin code | launch, item 6 (docs); a new follow-up for 4.4 |
| 3.5 scope F-16: the owner's field tail of `longest_callback_us` | bench, item 2 (the over-budget counter, split by action if the cut chooses) |
| 3.5 scope F-17: R8 run order may confound the priority comparison | bench, optional interleaved R8 (evidence; reopens nothing) |

### 1.5 Known issues and gaps that are still open

| Row (`book/src/config/known-issues.md` at 252daef) | Goes to |
|---|---|
| 5, a transient read error on reload is not retried (:25) | Deferred: 4.4 (the config location item rewrites the watcher's file handling); not on the input path |
| 15, AltGr's fake left Ctrl (:41) | held, item 6 (output safety on `kbdru.dll`); matching only if owner Q9 is yes, else new feature 2 |
| 16 and 51, `Ctrl+Pause`/`Ctrl+Break` arrive as `VK_CANCEL` (:42, :113) | Deferred: new feature 2 (key names) |
| 17, 18, 19 (:49-51) | seq |
| 30, 35, 68, 75 (:74, :79, :136, :143) | held |
| 32, 33 (:76-77) | launch |
| 34, `Text` control characters (:78) | input |
| 36, macro runs not capped (:80) | Deferred (1.4) |
| 38, scan codes, unverified (:82) | input (builder half); the application half stays a manual check |
| 39, `true` still swallows the second modifier; `MaskOnly` experimental (:89) | Deferred: open question 6, decided by Phase 1 manual checks 4 and 5 |
| 59, a swallowed key released while remapping is off reaches Windows as a lone key-up (:127) | Deferred, no item. **No scope report assigned it.** Verified open: while remapping is off the callback passes every event unread (`src/hook/keyboard_hook_callback.rs:410-415`; `decide_event` too, :255-257), after the reset of `swch off` (`src/hook/main_thread.rs:318-322`). Today it needs a key held while `swch off` is typed in a console; it becomes likelier when 4.6 puts on/off on the tray, so 4.6 (or 4.3) takes it |
| 63/79, a key-up lost with no signal (:131, :147) | Declared; F5/F6 as in 1.3 |
| 66 (:134) | held, owner Q3 |
| 69, 73 (:137, :141) | compiled |
| 80 (:148) | Decided: kept declared, measured by R17f, manual check added (section 6) |
| 81-87 (:149-155) | Declared (2.4) |
| 88 (:156) | Declared (2.5) |
| gaps: replay buffer, `HoldModifier`, unreachable system actions, `Layout Hotkey`, key names (gaps.md:37, :43, :95, :119, :166) | Deferred: new features 5, 6, 1, 7, 2 |
| gaps: no IPC reload/check/list (gaps.md:134) | Deferred: 4.3 |
| gaps: no longest match (gaps.md:165) | seq takes the identical-shape half (known issue 19); the prefix half stays new feature 8 |
| gaps: "What no test exercises" item 4 (gaps.md:285) | input (builders), launch (pure launch parts); the post of `system_function::post_to_foreground` (until `6d67816` `windows_actions::post_message_to_foreground`) stays untested (R17p is cost evidence only), while its choice of recipient, the pure `system_function::post_target`, has tests since lane fix-dialog-lang |
| gaps: versioning, release packaging (gaps.md:233, :240); the config-reference RON test (roadmap :856) | Deferred: project infrastructure, before the first release / after 4.4 |

### 1.6 Open points no scope report assigned

1. **FU-1**, the uninstall guard (roadmap :997): verified real. `KeyboardHook` is a static that is never dropped, and on
   a main-thread panic the unwind runs the bounded stops of the executor (2 s, `src/core/executor/mod.rs:358-361` via
   `src/core/app.rs:63-67`) and the watchdog (1 s, `src/hook/watchdog/mod.rs:549-553`) with the keyboard hook still
   installed and no thread pumping. Placed in **ground**, item 7.
2. **The lexical installer scan** (roadmap :1002). Placed in **ground**, item 8, as one scan helper for every scan test.
3. **`block_default` on `sequence`/`chord`** (gaps.md:169). Placed in **compiled**, warning (d).
4. **Known issue 59** (a lone key-up after `swch off`). Deferred to 4.6/4.3 with the reason in 1.5.
5. **1.6 F1, `sequence`/`chord` in the winner election.** The 3.2 scope deferred it to 3.4, which did not take it.
   Deferred with the reason in 1.4.
6. **Known issue 5** (no retry of a transient read error at reload). No scope report names it. Deferred to 4.4 (1.5).

## 2. Overlaps and contradictions between the scope reports

| # | Overlap or contradiction | Resolution and reason |
|---|---|---|
| C1 | The `when:`-after-match rule for `sequence`/`chord`: 3.2 claims it (T4); 3.5 proposes folding it into 3.2 (TF-1) or a lane B after 3.2 and 3.4; 3.4 lane A says "3.5 moves their `when:`" | **seq owns it.** It edits the same lines (`src/engine/mod.rs:857-916`, `sequences.rs:71-136`) and tests as KS-15/KS-16; a second pass would rewrite the arms twice. With it gone, 3.5's lane B keeps only F3 (decided, no code) and closure rows (Sync): **there is no second 3.5 lane.** compiled must keep the rule (its spec says so) |
| C2 | Bench row ids: 3.1 (R19, R19v, R20, R20m, R21), 3.2 (R19, R20), 3.3 (R19, R20), 3.4 (R19, R20), 3.5 (R19 family) all name new rows R19/R20 | **Central allocation** (section 4). The 50-binding R19 of 3.4 lane A and of 3.5 is the same row, built once in bench; 3.2's title-`when:` row is bench's R19w |
| C3 | Flaky executor test: 3.1 (held lane, or 3.3), 3.4 (lane C), 2.5 note ("the next lane that touches `core/executor`") | **ground**, item 6: it is the first lane in the order that touches `core/executor`, and the flake fails gates at random (2.5's final gate run) |
| C4 | Action `Arc`: 3.3 puts the `Launch` payload behind an `Arc`; 3.4 lane B makes every action an `Arc<BindAction>`; 3.5 F-11 sends the 2.1 note to 3.4 | **launch** adds the payload `Arc` (a `cwd: String` would otherwise add an allocation per fire on the hook path, which the latency rule forbids); **zero** decides whether to subsume it. Both specs say so |
| C5 | Mask per press: 3.1 (held, T7) and 3.5 (F-6, "out of 3.5, to 3.1") | **held**, item 8; agreed by both |
| C6 | D10 property coverage: 3.5 F-9 sends it to 3.4; the 3.4 scope does not list it | **ground**, item 5 (the property net) |
| C7 | 3.2 F-i (`block_default` on `sequence`/`chord`) and F-h (cross-kind election) deferred "to 3.4"; the 3.4 scope lists neither | F-i to **compiled** (warning d); F-h deferred (1.4) |
| C8 | 3.5 F-15 says two rustdoc warnings remain at `src/engine/modifiers.rs:30`, :127 | Stale: fixed in `28566a4` (verified). Nothing to do |
| C9 | 3.2 states the title read costs about 1.7 us per key (R10 6.5 vs R12 4.8 at item 2.3) | Stale: those rows carried the 4.7 us per-event modifier read that 2.5 removed. Current R10 - R9 is 0.2 us at the median (`p2-2.5-latency-trim/bench/H1.md:20-21`). The seq spec states the corrected figure; the win is real but smaller |
| C10 | 3.1 recommends releasing **every** held modifier around output, no opt-out (its Q1), citing AutoHotkey's `Send` | AutoHotkey's **remap** (`a::b`) is the closer analogue of `PressKey`: it sends `{Blind}`, so extra held modifiers stay in effect, and releases only the source's own modifiers (AutoHotkey v2 docs, Remapping; source 1). Under exact matching an extra modifier is held only where `allow_extra_modifiers` opted in. Raised as owner Q1 with three options and recommendation (b); not decided here |
| C11 | 3.1 lane B would release and restore Shift around the template's `Shift+CapsLock -> ToggleCapsLock`, the owner's daily binding, with an UNVERIFIED risk under 'Press the SHIFT key to turn off Caps Lock' (scope T10) | **held**, item 3: `ToggleCapsLock` keeps today's output unless the cut proves a release is needed with the manual check. Correctness of daily typing first: the binding works at 252daef |
| C12 | AltGr: 3.1 lists `RAlt+J` on the Russian layout only as a manual check; `kbdru.dll` sets `KLLF_ALTGR` (keys.md section 6.6), so the release and restore of lane B meet the fake left Ctrl on the owner's layout | **held**, item 6: output safety with the fake LCtrl (scan 0x21D) is in scope as a technical duty; matching changes only if owner Q9 says so |
| C13 | 3.2 asks the owner whether steps match modifiers exactly (its Q1) and whether duplicate shapes fire once (its Q2) | Not asked. Exact steps implement the binding owner decision "exact modifier matching, with a per-binding `allow_extra_modifiers`", which A2 deferred only in time (`p0-A2-matching/cut.md:115-121`, :518-519). One shape per key-down implements the 1.6 one-winner rule and the module's own promise (`src/engine/sequences.rs:8-10`). Both are in **seq**, listed for the owner in section 5 |
| C14 | 3.5 asks four owner questions (OQ-1 to OQ-4) | Decided technically or by the latency priority (section 5, D3-D6); none is a values question the owner has not already answered |
| C15 | 3.4 lane A: "sequences and chords keep their own list"; 3.5 TF-6: index every kind | The cut of **compiled** chooses; acceptance does not require it; the R19 family shows the cost either way |
| C16 | 3.4 orders "C first, then 3.2, then A, then B, with 3.1 and 3.3 before B"; 3.5 wants its rows first; 3.1 wants lane A before 3.3 | Merged into one order (section 3); every stated dependency holds |
| C17 | Shared files across lanes: `engine/mod.rs` (seq, held, compiled, zero), `engine/keys.rs` (seq, held, compiled), `engine/scenarios.rs` and `testing.rs` (ground, seq, held, compiled, zero), `core/executor/mod.rs` (ground, held, launch, zero), `core/windows_actions.rs` (input, held, launch, zero), `data/sequence.rs` (input, held, launch), `hook/keyboard_hook_callback.rs` (bench, input, held, compiled, zero), `hook/main_thread.rs` (ground, compiled, zero), `config/loader.rs` and `schema.rs` (seq, held under Q2, launch, compiled), `logging/counters.rs` (bench, input, zero), `src/bench.rs` (all) | Lanes run one after another in one checkout, each merged before the next is cut from master, so no two lanes edit a file at once. Each spec cites lines at 252daef; a later cut re-maps them to the then-current master |

## 3. Lane order

| # | Lane | Size | Why here |
|---|---|---|---|
| 1 | p3-3.5-budget-bench | S | No product change except one counter. Its 50-binding rows (R19 family) must exist before seq and compiled change the engine, so every later P/H bench carries them (3.5 TF-4). The counter starts collecting field evidence on the owner's machine early |
| 2 | p3-3.4-groundwork | M | No behaviour change in normal operation. It fixes the flaky test that fails gates at random, and raises the property net (10,000 symmetry scripts, timer-fire attribution, D10) before the five engine-changing lanes. FU-1 and the scan helper ride along |
| 3 | p3-3.2-seq-chord-matching | M | The first typing fix: rollover no longer completes a sequence and a later step's modifier no longer aborts it. Pure engine work with no owner question left. It is on the roadmap's critical path (:1016) and a hard prerequisite of compiled. It also removes the title read from unrelated keys |
| 4 | p3-3.1-input-module | M | The prerequisite of held (one builder, one `SendInput`, the test seam). Low risk, no engine semantics. It goes before launch so that `platform/launch.rs` follows the same module shape (3.1 scope section 6) |
| 5 | p3-3.1-held-modifiers | L | The largest daily-typing fix (KS-19: `Ctrl+J -> Down` gives Ctrl+Down; row 68: an `LWin` Hold with `PressKey("L")` locks the session). It waits for owner Q1-Q3 and Q9, which lanes 1-4 give time for, and it carries most Windows unknowns, so it runs after the safety net |
| 6 | p3-3.3-launch-de-elevate | M, risk L | A privilege fix, not a typing fix (KS-09, KS-34). It waits for owner Q4-Q7 and a research step. Soft prerequisite of zero (the `Launch` payload) |
| 7 | p3-3.4-compiled-bindings | L | Hard dependency on seq (the same loop). After held, so that the engine's `Decision` shape is settled. Owner Q8 decides its item 5 |
| 8 | p3-3.4-zero-alloc | L | Last: it extends `CompiledBindings` and rewrites the `Decision`, `FastAction` and `Launch` shapes that held, launch and compiled set |

After lane 8: the Phase 3 book sync (doc-writer; every lane's impl.md book-sync list), the orchestrator's chore
(CLAUDE.md bench rules for the R19-R26 rows, principle 1 for the counter compare, principle 3 for the input module, the
Layout for `platform/input.rs`, `platform/message_loop.rs`, `platform/launch.rs`, `engine/compiled.rs`), then
Checkpoint 3. Total about 16 days against the roadmap's 10-13 (:664): the scope reports found 3.1 to be L and 3.4 XL.

## 4. Lane specs and the bench row allocation

| # | Lane | Branch | Spec |
|---|---|---|---|
| 1 | p3-3.5-budget-bench | perf/budget-bench | `<orch>/p3-3.5-budget-bench/spec.md` |
| 2 | p3-3.4-groundwork | refactor/core-groundwork | `<orch>/p3-3.4-groundwork/spec.md` |
| 3 | p3-3.2-seq-chord-matching | fix/sequence-chord-matching | `<orch>/p3-3.2-seq-chord-matching/spec.md` |
| 4 | p3-3.1-input-module | fix/input-module | `<orch>/p3-3.1-input-module/spec.md` |
| 5 | p3-3.1-held-modifiers | fix/held-modifiers | `<orch>/p3-3.1-held-modifiers/spec.md` |
| 6 | p3-3.3-launch-de-elevate | fix/launch-de-elevate | `<orch>/p3-3.3-launch-de-elevate/spec.md` |
| 7 | p3-3.4-compiled-bindings | feat/compiled-bindings | `<orch>/p3-3.4-compiled-bindings/spec.md` |
| 8 | p3-3.4-zero-alloc | perf/zero-alloc-engine | `<orch>/p3-3.4-zero-alloc/spec.md` |

Bench rows (existing: R1-R18 with R12m, R12m2, R14c, R17a, R17t, and since `6d67816` R17c, R17g; `src/bench.rs`):

| Row | Lane | What | Role |
|---|---|---|---|
| R19, R19b, R19c, R19w, R19e | bench | 50 bindings: unbound key, bound key with `when:`, chord key, title `when:` on chords/sequences, batch-timed engine | p99 <= 100 us (3.5 c); later lanes gate p50 by D10's R4 rule; seq gates R19w <= R19 + 0.1 us |
| R17f, R17p | bench | `GetForegroundWindow`; `PostMessageW` to an own window | evidence |
| R17c, R17g | fix-dialog-lang (merged `6d67816`; taken) | `platform::focus::is_dialog` of the foreground (`GetClassWord(GCW_ATOM)`); `platform::focus::focus_window_of` of the same window (`GetWindowThreadProcessId` + `GetGUIThreadInfo`) | evidence; that lane's HEAD p50 <= 1.0 us and <= 5.0 us (measured 0.012 and 0.39 us) |
| R20 | seq | `on_key` of a sequence start and completion | optional evidence |
| R21, R21v, R22 | input | batch build; one `MapVirtualKeyW` (probe); `carry_out` of a masked `PressKey` through the recorder | evidence, probe, gate |
| R22m, R23 | held | R22 with held {LAlt}; one daemon-injected modifier event | gate, evidence |
| R24, R25 | launch | `Launch` key-down with a FakeRunner executor; Explorer lookup chain | gate (H <= P), evidence |
| R26 | zero | `on_key` firing `Launch`; arming a Tap+Hold+DoubleTap group | gate (H < P) |

## 5. Owner questions (values and scope only, deduplicated)

Technical forks stay with each lane's cut. Each question names the lane it blocks; all can be asked now in one batch.

**Q1. Which modifiers the user holds are released around a key output?** (KS-19; blocks **held**)
- (a) Every held modifier the output does not press, for every key action and macro step. The roadmap text (:676);
  AutoHotkey's default `Send`.
- (b) `PressKey` behaves like an AutoHotkey remap: the modifiers of the binding's own trigger are released, extra
  modifiers that `allow_extra_modifiers: true` let in stay in effect (`CapsLock+J -> Left` with Shift held selects).
  Macro `Key`/`Text`/`Combo` steps release every held modifier, as (a).
- (c) (a), plus a per-binding opt-out (AutoHotkey's `{Blind}`).
- **Recommendation: (b).** Every KS-19 and row-30/68 example involves the trigger's own modifiers, so (b) fixes them
  all; under the owner's exact matching an extra modifier is held only where the binding opted in, so keeping it is what
  the user asked for; it injects fewer events than (a), so less latency and a smaller restore race.
- Evidence: AutoHotkey v2 Remapping ("`a::b` ... `Send "{Blind}{b DownR}"`"; "modifiers included on the left side but
  not the right side are automatically released"; source 1) and Send (Blind mode "avoids releasing the modifier keys ...
  if they started out in the down position"; source 2); the 3.1 scope recommended (a) (its section 8.1 Q1).

**Q2. `PressKey("Ctrl+S")`: a key combination on the fast path?** (additive syntax; blocks **held** item 10)
- Options: yes / no (the only form stays `Sequence([Combo("Ctrl+S")])`, through the executor's queue and a thread wake).
- **Recommendation: yes.** The migration path for configs that relied on combining, at the lowest latency. Size S.
- Evidence: 3.1 scope section 8.1 Q2; `src/hook/keyboard_hook_callback.rs:326-334` (slow actions are queued).

**Q3. Known issue 66 inside Phase 3: a passing `Tap` or `Hold` on a lone Alt or Win no longer opens the menu bar or
Start?** (blocks **held** item 9)
- Options: yes for `Tap` and `Hold`, `DoubleTap` documented (its first tap stays a real lone tap) / yes for all three /
  no (keep row 66 as declared).
- **Recommendation: yes for `Tap` and `Hold`.** The daemon cannot know at the first release whether a second press
  comes; `block_default: true` stays the workaround for `DoubleTap`.
- Evidence: `book/src/config/known-issues.md:134` ("No roadmap item yet; a candidate for item 3.1"); roadmap :425;
  AutoHotkey documents `~LWin::Send "{Blind}{vkE8}"` for this case (3.1 scope source 4).

**Q4. Roadmap open question 3: deploy with `uiAccess="true"` (signed, under Program Files) or keep
`requireAdministrator`?** (blocks **launch**; also 4.5)
- Options: keep `requireAdministrator`, no code signing now / move to uiAccess (needs a code-signing certificate and an
  installer).
- **Recommendation: keep `requireAdministrator` for now.** The launch route is chosen at run time from the daemon's
  token, so a later uiAccess deployment needs no rework of 3.3.
- Evidence: `build.rs:32-34`; roadmap :942, :857; gaps.md:240 (no signing or packaging).

**Q5. What does `elevated: true` do: start silently with the daemon's administrator token, or show a UAC prompt each
time?** (blocks **launch**)
- **Recommendation: prompt** (the `runas` verb through Explorer), so nothing runs as administrator without consent.
- Evidence: whoever can write `config.ron` would otherwise get admin code without a prompt (build-and-run.md :311-312;
  audit KS-09), and 4.4 moves the config to a folder medium-integrity code can write (roadmap :788-796). Cost: one UAC
  prompt per elevated launch, which also triggers 2.3's input-left reset on the secure desktop.

**Q6. Does `OpenUrl` get `elevated:` too?** (blocks **launch**; changes an acceptance line)
- **Recommendation: no, `Launch` only.** Reword acceptance line 3 (:706) to "`Launch(exe: "cmd.exe", args: ["/k",
  "whoami /groups"], elevated: true)` reports `High Mandatory Level`".
- Evidence: the syntax plan names only `Launch` (:871); `OpenUrl` is a RON newtype (`src/config/schema.rs:199-201`),
  so it needs a second syntax form; Chromium browsers de-elevate themselves, so "with `elevated: true` it is" cannot be
  shown with Edge or Chrome (audit KS-09).

**Q7. The working directory of a launched program without `cwd:`?** (blocks **launch**)
- Options: the user's profile folder (as the Run dialog) / the program's own folder / whatever Explorer passes
  (likely System32, UNVERIFIED).
- **Recommendation: the user's profile folder.** The third repeats KS-34's System32 problem (audit KS-34).

**Q8. Auto-repeats when a modifier or the window changes during a held press (known issue 73)?** (blocks **compiled**
item 5)
- (a) Lock the winner for the whole press, until key-up or a reload; (b) keep today's re-election and document it as
  by design; (c) lock, but stop repeating once the winner's keys or `when:` no longer match.
- **Recommendation: (a).** Firing then follows suppression, which already follows the key-down (rule L, KS-06), and the
  double output of row 73 goes. Cost: a held `Shift+H -> Home` keeps sending Home after Shift is released.
- Evidence: `src/engine/mod.rs:770-774`; `engine::scenarios::repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down`
  (scenarios.rs:5214-5248); 1.6 cut Q3 deferred it to 3.4 (`p1-1.6-one-winner/cut.md:489`).

**Q9. Should right Alt on an AltGr layout stop counting as Ctrl+Alt for matching in Phase 3?** (the AltGr half of new
feature 2; blocks only **held** item 6's matching part)
- Options: yes, in held (**Breaking**: `Ctrl+Alt+X` bindings stop firing on AltGr+X) / no, keep it new feature 2.
- **Recommendation: no.** held makes the output safe with AltGr's fake Ctrl either way; the owner's two bindings do not
  meet the matching problem; the change breaks configs written by keys.md's advice ("Write `Ctrl+RAlt+X`").
- Evidence: `kbdru.dll` sets `KLLF_ALTGR` (keys.md section 6.6 at 252daef); roadmap :827 ("High with AltGr layouts").

**Q10. Versions: tag v0.2.0 at all, and when v0.3.0?** (blocks only the Phase 3 closing chore)
- Options: skip v0.2.0 and cut v0.3.0 after Checkpoint 3's manual checks / tag v0.2.0 at `252daef` after the Phase 2
  checks are run / tag v0.2.0 at `252daef` now.
- **Recommendation: skip v0.2.0.** It was never published, its checkpoint was never run, and `CHANGELOG.md` keeps
  everything since 0.1.0 under `[Unreleased]` (:12); one `[0.3.0]` section then covers Phases 0-3.
- Evidence: no tags in the repository; `Cargo.toml:3` is `0.1.0`; roadmap :852.

**Decided without a question (for the owner's information):**
- D1. Sequence steps match modifiers exactly, and `allow_extra_modifiers` becomes legal on `sequence` (the owner
  decision on exact matching; **Breaking**: `["F13","S"]` with Shift held now types a capital S). Chords keep matching
  whatever modifiers are held. In seq.
- D2. Two `sequence` or `chord` bindings of the same shape whose `when:` both hold: the first in file order fires once
  (the 1.6 one-winner rule; known issue 19). In seq.
- D3. The 100 us budget is gated by the bench p99 of the R19 family; a field counter of callbacks of 100 us or more is
  evidence, not a gate (3.5 OQ-1). In bench.
- D4. Known issue 80 stays declared: a fix costs a `GetForegroundWindow` per `AppEquals` key-down for every user and
  breaks "the hook path never queries a process" (CLAUDE.md principle 5); R17f measures it and a manual check counts
  misses (3.5 OQ-4).
- D5. Q11 of 2.5 stays declared: the fix costs one `GetAsyncKeyState` (R17a, 0.545 us) per auto-repeat of every held
  passed modifier, for a case that needs an unseen release and no key-down after it (3.5 OQ-2).
- D6. The witness-skip follow-up closes: the owner merged 2.5 with ruling W1 (3.5 OQ-3).
- D7. `ToggleCapsLock` keeps today's output in held unless a manual check proves a change is needed (C11).

## 6. Checkpoint 3 (v0.3.0)

Release build, daemon started with `swch open`, `swch` from a non-elevated console, the default config unless a row
says otherwise. Automated items are proved by the gate and the bench on the final tip; manual items need the owner.

1. Gate green on the last Phase 3 merge; `cargo test` counts reported apart from the ignored manual and bench tests.
2. Automated: `SendInput(` on one line of `src` (input); the engine purity scan and >= 10,000 press-symmetry scripts
   (ground); `Engine::on_key`/`on_timer` allocate nothing under the counting allocator (zero).
3. Bench, P = the merge of bench, H = the last merge, one session, 3 runs each: the R19 family p99 <= 100 us; no row
   regressed beyond the CLAUDE.md rules except where a lane's ruling accepted it.
4. Manual: `Alt+J -> PressKey("Left")` moves the cursor in Notepad, no menu bar after Alt goes up; `Ctrl+Alt+L ->
   PressKey("L")` types `l` (3.1; KS-19).
5. Manual: `( sequence: (steps: ["ScrollLock", "Ctrl+S"]), action: SwitchLanguage )` switches the layout, opens no
   Save dialog, and `a` typed afterwards gives `a`; holding ScrollLock of `["ScrollLock","ScrollLock"]` and typing `x`
   types `x` (3.2; KS-15, KS-16).
6. Manual: `Launch(exe: "cmd.exe", args: ["/k", "whoami /groups"])` from the release daemon reports `Medium Mandatory
   Level` in its own console (3.3; KS-09, KS-34).
7. Manual: 30 minutes of daily typing on the default config with Alt+Tab and one Win+L: CapsLock switches the layout,
   Shift+CapsLock toggles caps lock (under the owner's 'turn off Caps Lock' setting), nothing stuck in `osk.exe`, no
   `[ERROR]` in `daemon.log`; record the over-budget counter and `longest_callback_us` lines.
8. Release chore (after the owner's go, Q10): `Cargo.toml` version 0.3.0, `CHANGELOG.md` `## [0.3.0]`, tag `v0.3.0`.

**Manual checks after Phase 3** (the lanes add `#[ignore = "manual: ..."]` stubs where a test can carry the steps):

| # | Lane | Check |
|---|---|---|
| 1 | bench | After a week of use, read the counters lines for the over-budget counter(s) next to `longest_callback_us` |
| 2 | ground | M-C1: typing, `swch off`/`on`, a `Sequence` with `Delay`, `Launch`, `OpenUrl`: nothing hangs |
| 3-5 | seq | M1-M3 of its spec (ScrollLock then Ctrl+S; ScrollLock rollover; Ctrl+Shift+S after ScrollLock opens Save As) |
| 6-9 | input | `PressKey("Up")` in RDP/Hyper-V with NumLock on; `PressKey("RAlt")` as AltGr on the Russian layout; `Text("a\nb")` in Notepad; the moved mask test by hand |
| 10-18 | held | The nine checks of its spec: Alt+J; Ctrl+Alt+L; LCtrl Hold -> Escape opens no Start; Phase 1 check 10 (if Q3); Shift+CapsLock under both Caps Lock settings; Ctrl+J held 2 s; RAlt+J with AltGr on the Russian layout; a `Ctrl+Alt+N` macro released mid-way; `longest_callback_us` before and after |
| 19-26 | launch | M1-M8 of its spec (Medium level; browser not elevated; `elevated: true`; `cwd`; Explorer stopped; executor block; `regedit.exe` prompts; debug daemon) |
| 27 | compiled | M-A1, only if Q8 locks |
| 28 | zero | M-B1: `Launch`, `OpenUrl` and `Sequence` bound to keys run once each |
| 29 | F3 (D4) | AutoHotkey `WinActivate ahk_exe notepad.exe` then `Send {F13}` against `( keys: "F13", action: ToggleCapsLock, when: AppEquals("notepad.exe") )`, 50 times from another window; count the misses for known issue 80 |

Still unrun and owed before the tag (owner, 2026-10-25: none was run): Phase 1 checks 1-18 (roadmap :394-413), Phase 2
checks 1-48 (:609-658), 2.5's M1-M11 (`p2-2.5-latency-trim/impl.md:334-353`; M8 clock base, M9 StickyKeys, M10
AutoHotkey, M11 PowerToys). Checks 43, 44 and 47 decide FU-4 and the watchdog's fallback; checks 4 and 5 of Phase 1
decide open question 6.

## 7. Claims checked against 252daef

| # | Claim (source) | Result |
|---|---|---|
| V1 | Six `SendInput(` lines in four files (3.1) | Verified: `system_function.rs:155`, `windows_actions.rs:287`, `sequence.rs:139`, :169, :205, `keyboard_hook_callback.rs:364` |
| V2 | `mark_completed` changes only a held key with no record or a `RepeatSuppressed` one (3.2) | Verified by reading `src/engine/keys.rs:249-261` and the arms at `mod.rs:879-913`; the mutation run in seq confirms it |
| V3 | Roadmap A1 cannot pass literally (3.4) | Verified: Grep matches `src/engine/mod.rs:31-32`, :251 and `schedule.rs:6`, none of them code |
| V4 | SEEDS 64, STEPS 250, 9 configs (3.4) | Verified: `src/engine/properties.rs:98-99`, :126 |
| V5 | Flaky test fails gates (2.5, 3.4) | Verified: `p2-2.5-latency-trim/gate_test_final.log:421`, :887 ("861 passed; 1 failed; 7 ignored") |
| V6 | Waits and pumps (3.4) | Verified: `MsgWaitForMultipleObjectsEx(` at `executor/mod.rs:649`, `main_thread.rs:813`, `watchdog/mod.rs:735` outside tests; pumps at `main_thread.rs:1008`, `executor/mod.rs:678` |
| V7 | `Win32_System_Ole` and `Win32_System_Variant` missing, `Win32_System_Com` present (3.3) | Verified: `Cargo.toml:41-59` |
| V8 | Field tail: 22 of 74 reported minutes at 100 us or more, median 56 us, maximum 4246 us (3.5 F-16) | Verified in `%LOCALAPPDATA%\key-switch-rs\daemon.log` (read only): 74 counters lines; tail values 728, 744, 1718, 1999, 2633, 4246 us; no `[WARN]`/`[ERROR]`, no reinstall in 4.5 days |
| V9 | 121 `hook_late` in 74 reports settles U1 (3.5 F-10) | Verified: the sum of `hook_late` over the 74 lines is 121; a wrong time base would note every key |
| V10 | Rustdoc warnings remain (3.5 F-15) | Refuted: fixed in `28566a4` |
| V11 | The 1.2 m1 warning order (1.2 results :60) | Fixed: `src/config/loader.rs:400-411` warns only where the blocker is added |
| V12 | Title read about 1.7 us (3.2) | Stale: R10 0.4 us, R9 0.2 us p50 in `p2-2.5-latency-trim/bench/H1.md:20-21` |
| V13 | 'Never two hooks or none' wording needs a fix (roadmap :1001) | Already scoped: `src/hook/keyboard_hook.rs:266-267` ("exactly one hook of ours"), `CHANGELOG.md:194` |
| V14 | Known issue 59 is open (known-issues.md:127) | Verified: the callback passes every event while remapping is off (`src/hook/keyboard_hook_callback.rs:410-415`), so the key-up of a key swallowed before `swch off` reaches Windows alone |
| V15 | FU-1 is real (roadmap :997) | Verified: `src/core/app.rs:295-322` uninstalls only on the normal path; `Drop` of `AttachedExecutor` (app.rs:63-67) and `HookWatchdog` (watchdog/mod.rs:549-553) wait with the hook installed during an unwind |

## Sources

1. AutoHotkey v2, Remapping Keys: <https://www.autohotkey.com/docs/v2/misc/Remap.htm>
2. AutoHotkey v2, Send (Blind mode): <https://www.autohotkey.com/docs/v2/lib/Send.htm>
3. Raymond Chen, "How can I launch an unelevated process from my elevated process and vice versa?", 2013-11-18:
   <https://devblogs.microsoft.com/oldnewthing/20131118-00/?p=2643> (the Explorer route of 3.3; details left to the
   launch lane's research step)

## Not covered

- `research-3.3.md` does not exist; the web facts of 3.3 (how `IShellDispatch2::ShellExecute` reports errors, its
  working directory, the parent-process alternative, uiAccess token inheritance) are open and are the first step of
  the launch lane.
- The scope reports' traced red-first expectations were not run (no cargo by brief); V2's trace was re-read, not
  executed.
- `engine/gestures.rs`, `winner.rs` and `testing.rs` were not re-read beyond the cited lines; the scope reports' line
  numbers there are taken as given.
- No prior-art check beyond AutoHotkey's remap and Send semantics (C10) and Chen's launch route.
