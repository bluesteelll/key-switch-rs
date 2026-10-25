# Architecture critique: p0-A1-engine, Workstream A scope additions (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 2, Minor 3)
Plan: D:/tmp/ks-orch/p0-A1-engine/cut.md   Code read at: master `73bbf40`, `refactor/engine` `1d12c7b` (read via `git show`, no checkout)

Line references: `cut:L<n>` = cut.md line; `M:` = master `73bbf40`; `B:` = branch `1d12c7b`.

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. D6 runs an overdue Hold action on the hook thread. That contradicts the spec's "same threads", the cut's own D2 rationale and a guarantee in the config reference
Where: plan D6 (cut:L169-186), D3 `carry_out` on the callback path (cut:L126, L131-133), D2 Rejected (a) (cut:L113-114), D5 fallback (cut:L162-163), threading table (cut:L245-246), Behaviour deltas (7) (cut:L396), Docs impact §11.5 (cut:L431); today's Hold thread `M:src/hook/tap_state.rs:105-116`; `book/src/config/triggers-conditions.md:65`; `book/src/audit.md:309` (KS-08).
Problem: D6 puts `Overdue(action)` into the key-up `Decision::fire` ("fire order `[overdue hold, tap]`"). D3 carries out every key-up decision inside `keyboard_hook_callback` on the main thread. An overdue Hold action therefore runs in the LL callback. Today a Hold action never runs there: it runs on its `hold-timer` thread (`M:tap_state.rs:105-116`). The audit names Hold as one of only two exits from the hook thread (`audit.md:309`). The config reference promises users: "This is the only trigger whose action runs outside the hook thread, so `Launch` and `OpenUrl` can safely be used with `Hold` directly" (`triggers-conditions.md:65`). The spec says the adapter "executes the Decision exactly as today (same threads, same timing)". D2 rejects option (a) for this exact reason ("the Hold action would then run on the hook thread ... KS-08 regression"). D6 brings that regression back on a narrower path and does not record it. The threading table lists Immediate/Tap/DoubleTap/Chord/Sequence actions on main and the Hold action on `hold-timer`. Behaviour delta (7) says a failed spawn makes "the Hold fire at release" without naming the thread. Docs impact for §11.5 covers only Tap and DoubleTap.
Consequence: `( keys: "F13", action: Launch(exe: "notepad.exe"), trigger: Hold(300) )` is the configuration the book recommends for `Launch`. Under CPU load the `hold-timer` thread wakes late, and the user releases F13 at 305 ms before that thread takes the engine lock. The key-up decision then carries `Overdue(Launch)`, and `CreateProcessW` runs inside `keyboard_hook_callback`. Keyboard input stalls system-wide for the duration. If the stall passes `LowLevelHooksTimeout`, Windows silently removes the hook and every binding stops working (KS-08). The window is narrow at idle and widest under load, which is also when `CreateProcessW` is slowest. After a failed `hold-timer` spawn (D5), every such Hold runs on the hook thread.
Confidence: CONFIRMED (plan D3/D6 text plus `M:tap_state.rs:105-116` and the book line)
Breaks: spec "same threads, same timing"; KS-08; R1; the documented guarantee in `config/triggers-conditions.md` §11.5; the cut's own D2 rejection.
What is needed: the cut must decide which thread runs an overdue Hold action, and the threading table, Behaviour deltas, CHANGELOG, Docs impact (book §11.5) and manual checks must all say the same thing.
Options: (a) keep Hold actions off the hook thread on the overdue path too. The adapter would need to know which `fire` entries are Hold actions and hand them to a short-lived thread, as timers do today. (b) Accept hook-thread execution as a declared delta: CHANGELOG "Changed", the threading table, a book §11.5 correction in this lane or a blocking note for the book sync, and a manual check that releases a `Hold`+`Launch` key exactly at its term under load. (c) Take Q3's veto path and drop D6.
Q3's rationale also needs correcting. D6's Why calls the release-in-`[due, wake)` window "a new 'neither' window". Master already has a narrow both-or-neither window at equal terms (`audit.md` KS-25). The deadline design does not create a window. It widens "neither" from the gap between two equally late threads to the whole lateness of the Hold timer. That is still a reason for D6, but the owner should decide Q3 on the accurate statement.

### I2. The proof for "no decision logic is left in `src/hook`" fails on the cut's own C4 test
Where: plan C4 `injected_and_unknown_events_bypass_the_engine` builds `Binding::new(..).with_trigger(Trigger::Tap(10_000))` in `hook::keyboard_hook_callback::tests` (cut:L312-313); Acceptance mapping (cut:L343) requires `rg -n "Trigger::|BindingKind::|\.matches\(|..." src/hook` to return no hits.
Problem: the prescribed `rg` searches the whole of `src/hook`, test modules and comments included. C4 adds `Trigger::Tap` to `src/hook/keyboard_hook_callback.rs`. The new `//!` adapter header from D10 (cut:L214) will also probably name trigger kinds. The engine criterion excludes comments ("outside comments"); this criterion excludes neither comments nor tests.
Consequence: after C6 the tester runs the mapped command on HEAD, gets at least one hit at `src/hook/keyboard_hook_callback.rs` (the C4 test), and must report the acceptance line FAIL. That costs a triage and fix round, or invites a developer to bend a correct test to satisfy the grep. Nothing on the owner's desktop is affected. The lane cannot reach ACCEPT as the cut is written.
Confidence: CONFIRMED (cut:L313 vs cut:L343)
Breaks: "acceptance criterion with a proof"; the spec acceptance line "no decision logic is left in src/hook".
What is needed: a check that can pass and still catches real decision logic. Exclude `#[cfg(test)]` modules and comment lines explicitly, the way the `src/engine` criterion excludes comments, and say how the tester applies the exclusion.

## Minor (improvements, not blockers)

### M1. The engine depends on `core::windows_actions`, and D8 claims more than `forbid(unsafe_code)` enforces
Where: plan D1 (cut:L80-86), D8 (cut:L197-199), checklist "No new module cycle (`hook -> engine -> {core, data}`)" (cut:L469); `B:src/engine/mod.rs:112`, `B:gestures.rs:25`, `B:chords.rs:24`, `B:sequences.rs:21` (`use crate::core::windows_actions::BindAction`); `M:src/core/windows_actions.rs:117-141` (`execute`, which reaches `press_vk`, `launch_process` and `shell_open`, all safe fns); `M:src/data/condition.rs:93` (`ForegroundContext::capture`, a safe fn that calls `GetForegroundWindow`).
Problem: the critic's checklist says the plan must not make `engine` depend on `core::windows_actions`. The branch does, through `BindAction`, and D1 keeps it without saying so. The cut's checklist also counts no new cycle, but `core::app` -> `hook` -> `engine` -> `core` is one at module level. D8 says `#![forbid(unsafe_code)]` makes "no Win32 calls in the engine" a compile-time property. It only rules out direct FFI. Safe crate wrappers that do Win32 work stay callable. Neither acceptance grep (`SendInput|...`, `\.execute\(`) covers those wrappers or a direct `Instant::now()` (checklist §6: the core must not read the clock).
Consequence: a later lane (for example 1.2's mask key) can write `ForegroundContext::capture()`, `Instant::now()` or a call to a `windows_actions` helper inside `src/engine`. It compiles under `forbid(unsafe_code)` and passes every grep in this cut, so the purity guarantee is unenforced. Today's branch code is clean (`git grep` on `1d12c7b` finds none of these outside tests).
Confidence: CONFIRMED (structure); the consequence is prospective
Breaks: critic checklist §6 (engine dependencies, clock); spec "the engine executes NOTHING itself" (enforced only by review)
What is needed: reword D8 to describe what the attribute actually enforces, declare the `engine -> core::windows_actions` dependency, and either add non-test greps for the clock, foreground-capture and `windows_actions` helper calls or record moving `BindAction` into `data` as a follow-up for 3.4.

### M2. The C4 red-first test leaves the `WM_SYSKEYUP` arm of the four-way translation untested
Where: plan D3 `key_event` (cut:L124), C4 `physical_key_events_reach_the_engine` (cut:L308-311), which covers `WM_KEYDOWN`, `WM_SYSKEYDOWN` and `WM_KEYUP`.
Problem: an implementation that maps only `WM_KEYUP` to `Up` passes every C4 test.
Consequence: `( keys: "Alt+J", action: PressKey("Left") )`. J's key-down (`WM_SYSKEYDOWN`) is suppressed and marked blocked. J's key-up arrives as `WM_SYSKEYUP` while Alt is still held, `key_event` returns `None`, and the up passes. The engine never releases J or clears its blocked flag. On the next plain J press the engine treats the down as a repeat, and the up is swallowed, so the application sees J stuck down (the KS-06 symptom).
Confidence: CONFIRMED (coverage gap); the faulty implementation is PLAUSIBLE
Breaks: suppression invariant (CLAUDE.md principle 2); "red-first that bites"
What is needed: the adapter test must cover all four key messages, and one of them must be an up whose down was suppressed.

### M3. Re-entry refusal is symmetric only for key-downs; the Risks section claims it holds in general
Where: plan D2 `with_engine` returns `None` on re-entry (cut:L95-98); Risks "Re-entrancy" (cut:L381-384): "A passed key-down was never recorded, so its key-up also passes: suppression symmetry holds."
Problem: the argument covers only a refused key-down. A refused key-up of a key whose down was suppressed passes without reaching `on_key_up`. `blocked[vk]` stays set and `active[vk]` stays true (`B:keys.rs:115-154`).
Consequence: if re-entry ever happens (the cut's example is a future own-process window, roadmap 4.6), the refused key-up of a suppressed F13 passes. The next F13 down then counts as a repeat, so a Tap or Hold binding skips it and it passes, and the next F13 up is swallowed: the application sees F13 stuck down. The cut's own analysis says nothing under the lock pumps messages today, so this cannot happen now.
Confidence: PLAUSIBLE (needs a re-entry path that does not exist today)
Breaks: suppression invariant (KS-06 class), latent
What is needed: correct the Risks text and record the residual risk, for example a refusal counter or log line, plus an explicit hand-over to 2.2/3.4, which replace the lock with main-thread ownership. Then a later lane that adds a window in the daemon does not inherit the risk unknowingly.

## Strengths to keep

- **D0** (merge `master` into the lane branch first) is correct and necessary. `git diff --stat da58f5f master` touches only `.claude/`, `CLAUDE.md`, `docs/WORKFLOW.md` and `.gitattributes`; the branch touches only `src/`. Without the merge, checking out the branch removes the role files and deny rules from the shared work tree.
- **Re-location table** is accurate. I spot-checked about 30 `M:`/`B:` references, including `M:keyboard_hook_callback.rs:186-318/320-343/139-141/149`, `M:tap_state.rs:61-67/105-116/152-164`, `B:mod.rs:223-251/253-403` and `B:gestures.rs:66-169`. All match.
- **D4 clock under the lock.** I traced both lock orders: a key-up that reads `now < due` takes the lock before the timer's check, so the timer finds the entry cancelled. A key-up that takes the lock after the check reads `now >= due`, so with D6 the Hold fires at the key-up and the timer finds nothing. With equal terms exactly one fires (I-TH holds). Rejecting `KBDLLHOOKSTRUCT::time` (a different 32-bit clock that wraps) is sound.
- **D2 `DECIDING` guard** with a test bounded by `recv_timeout`, poison recovery through `into_inner`, no `expect` reachable from the `extern "system"` functions, no action, spawn or `ArcSwap` guard under the lock. Rejecting `try_lock` for breaking symmetry is correctly argued.
- **D5** turns KS-33's abort into a logged degrade, and Tap/DoubleTap no longer spawn threads.
- **D7** lazy foreground capture: same Win32 calls, never more.
- **C1 harness extension** (`advance_ms_stalled`). The predicted red matches the panic format in `B:testing.rs:252-262, 283-297` and the label format `"{what} {key} at +N ms"`. The guard scenario (run B / test 3) kills an "always fire at release" implementation. "Every other scenario unchanged" holds, because `advance_ms` delivers every timer due at or before the clock (`B:testing.rs:210-227`), so the existing scripts never observe an overdue entry.
- **D9 test mapping**: all 21 removed tests have ported counterparts under the names the cut gives (`gestures`, `chords`, `sequences`, `keys`). `git grep` on `1d12c7b` finds none of `GetAsyncKeyState|GetForegroundWindow|SendInput|Mutex|thread::|unsafe|\.execute\(` in `src/engine`.
- No new Win32 API, Cargo feature, `SendInput` path or handle. Tests never install a hook, inject input or write the registry; `HookEnv` in the C4 tests only reads `GetAsyncKeyState`, and the F13 single-key binding matches whatever the owner's real modifiers are.

## Open questions for the architect

1. Could a unit test drive the `extern "system"` callback directly (a stack `KBDLLHOOKSTRUCT`, `code = HC_ACTION`, a `DoNothing` binding on the global hook) to pin "`LRESULT(1)` iff `Decision::suppress`" and the `code < 0` short-circuit, instead of leaving them to review (cut:L342)? Check before adopting it: confirm on Microsoft Learn that `CallNextHookEx` with `hhk` ignored is a harmless no-op outside a hook chain. If that cannot be confirmed, keep the review.
2. Q4 against CLAUDE.md "Docs and the book" (a change to config semantics updates `book/src/config/*.md` in the same lane). If D6 stays, is §11.5/§11.6 of `triggers-conditions.md` a semantics change (resolution and thread of `Hold`) or only a known-issue note? I1 already requires §11.5 to be correct; the orchestrator should decide whether the developer or the book sync makes that edit.
3. `foreground_change_forgets_pending_key_up_suppression` (`B:scenarios.rs:170-180`) pins today's repair, which item 2.3 narrows. The cut flags it (cut:L400-402). No change asked; it is listed so 2.3's cut picks it up.

## Absent topics (walked, not applicable)

- §2 thread affinity: install/uninstall stay on main and do not move (cut:L250). KS-01 is a separate lane.
- §3 matching model: unchanged by design (superset, folding, specificity are later items). The engine reproduces `M:` behaviour, and I checked the sequence and chord ports line by line against `M:sequence_state.rs:55-124` and `M:chord_state.rs:61-123`.
- §7 config and hot reload: no schema change. Bindings are passed per call, so a reload never touches the engine, as today.
- §8 atomics: none added. `DECIDING` is thread-local. The R3 wording conflict is raised as Q1, and the spec sanctions the Mutex.
- §9 process boundary, §10 input synthesis: no pipe, token, registry or `SendInput` change. `Decision::mask` stays reserved.
- COM, windows, `SetTimer`: none involved. No new handle, and `ForegroundContext`'s `ProcessHandleGuard` is unchanged.
- Owner decisions: none touched (`eprintln!` until 1.7, RON unchanged).
