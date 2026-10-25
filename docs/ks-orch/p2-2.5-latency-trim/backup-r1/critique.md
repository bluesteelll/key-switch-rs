# Architecture critique: p2-2.5-latency-trim / roadmap 2.5 Latency trim (round 1)

Verdict: CHANGES_REQUESTED (Critical 0, Important 2, Minor 4)
Plan: D:/tmp/ks-orch/p2-2.5-latency-trim/cut.md   Spec: D:/tmp/ks-orch/p2-2.5-latency-trim/spec.md
Code read at: master `50b56a8` (clean)

Sections 1-11 walked. The engine logic of D1-D3 holds up when I trace it against `src/engine/mod.rs:858-899`
and `src/engine/keys.rs`. I found no path where a stale stamp is newer than the truth: a stale stamp only
adds a read. The two Important findings are about the guarantee the cut claims and the evidence behind it.
They are not about the core rule.

## Critical (implementation must not start)

None.

## Important (must be resolved)

### I1. The residuals contradict the spec's "an unknown case keeps the read", and the cut sends their acceptance to the orchestrator

Where: spec.md:5 and :12. Cut §D4 W-table (cut.md:247-258): W1 "(unverified)", W2 "(unverified)", and W6
"how Windows handles a timed-out sent message is not documented". §Residuals R-E1..R-E4 (cut.md:861-872).
Q2 (cut.md:882-892). The rejected alternative "Reading `LowLevelHooksTimeout`" (cut.md:264-265).

Problem:
- The spec binds two things. The first is "no key may ever stick". The second is "argue every way an
  unseen key-down can happen without an epoch bump (cite Microsoft Learn; an unknown case keeps the read)".
- Three of the four residuals rest on behaviour that the cut itself marks as undocumented or unverified:
  - R-E1 depends on whether Windows still delivers a low-level hook call after it timed out (W6).
  - R-E3 depends on the delivery of the switch, lock and foreground events. Manual checks 24, 28, 35 and
    36 have no record of a run (`roadmap.md:607`).
  - R-E4 depends on an unverified UIPI rule.
- In these cases the cut drops the read. Under 2.3, the read at every swallowed key-up made each of them
  harmless.
- Q2 recommends that the orchestrator accept the residuals, and brings in the owner "only if the
  orchestrator rejects the recommendation". That trade sets one owner priority (latency) against another
  (no stuck key). It is a values question, which CLAUDE.md routes to the owner, not a technical fork.
- R-E2 is not even an unknown. It is a registry value the daemon may read (read-only; manual check 38
  already reads it). The cut rejected that read only as "a startup read and a code path".

Consequence (R-E1, as the cut words it at cut.md:863-866):
1. CapsLock (default config) was pressed and released once in this window, so its stamp is current.
2. CapsLock is held and swallowed.
3. The main thread is pre-empted inside its wait for at least 600 ms (2 x the 300 ms default timeout).
4. In that time the user releases CapsLock and presses it again. Windows times out both hook calls and
   applies both events.
5. If Windows drops timed-out calls, nothing notes before the next watchdog tick.
6. The user releases CapsLock within that tick. The stamps compare equal, the release is swallowed, and
   CapsLock stays down in Windows. Master's witness passes that release.

R-E3 gives the same result: if no switch, lock or foreground event arrives for one Ctrl+Alt+Del episode
(known issue 79 already lists ways that no signal comes), the manual-check-36 script leaves CapsLock down.
2.3 heals that case.

Confidence: CONFIRMED that the plan contradicts the spec (spec:5 and :12 against cut Q2 and the
Residuals). The stuck key itself depends on the unverified Windows behaviour.
Breaks: the spec's binding rule "no key may ever stick"; the spec's "an unknown case keeps the read";
owner-question routing (CLAUDE.md, Orchestration discipline).

What is needed:
- Q2 goes to the owner as a values question before commits 4-5, with the price stated both ways: about
  0.5 µs at the first swallowed key-up per key and bump, against R-E1..R-E4.
- Close what can be closed:
  - R-E2: a read-only startup read of `LowLevelHooksTimeout`. Check the value type and close the key on
    every path. Keep the read when the value is below the detection bound.
  - R-E1 and W6: get evidence. M4 can also record whether keys typed during the 1.5 s hold arrive later as
    late entry callbacks (`hook_late` above 2). That settles whether timed-out calls are delivered.
- The plan must say what happens to the trim if M1-M3 fail after the merge.

### I2. The trim's effect on the real machine rests on an unverified clock equivalence that no test or manual check can catch

Where: cut §D5 (cut.md:285-297, :310-312, :326-335), D8 R11 (cut.md:507), manual checks M1-M7
(cut.md:794-802). `src/hook/keyboard_hook_callback.rs:375-393`. `CHANGELOG.md:117-120`: while any counter
is not 0, a `[WARN] counters since the last report: ...` line is written every 60 s to the console and
`daemon.log`.

Problem:
- E4's entry check, `GetTickCount().wrapping_sub(event.time) >= 17`, runs on every callback while
  remapping is on. The cut marks "that both read the same counter" as (unverified).
- Every automated proof stamps `time: GetTickCount()` itself: the bench helper and the test helpers
  (cut.md:310-312). R11 therefore passes by construction, whatever Windows really puts in `time`.
- A stamp ahead of `GetTickCount` by even 1 ms wraps to about 4.29e9 and counts as late.
- Two more sources read as "late" when nothing is hidden:
  - (a) An injector's own nonzero `KEYBDINPUT.time`. SendInput keeps the caller's stamp unless it is 0
    (SendInput / KEYBDINPUT, Microsoft Learn).
  - (b) Time spent in low-level hooks installed after the daemon's, which Windows calls first.
- No manual check asks for `hook_late` to stay at 0 during normal typing. M4 only asks for at least 1.

Consequence:
- If the stamp uses a different or finer base, or a remote-input tool or a slow foreign hook runs, every
  swallowed key-up reads again. With the default config that is every CapsLock release.
- The item-1 trim is then void on the owner's machine while bench R11 shows it done.
- `daemon.log` and the console also get a `[WARN] ... hook_late=N` line every minute.
- No key sticks.

Confidence: PLAUSIBLE.
Breaks: the owner's latency priority (this lane's deliverable); the proof of the R11 acceptance line,
which is established only for a synthetic stamp.

What is needed:
- A manual check (a new row, or M7 amended) that normal typing in the release daemon leaves `hook_late` at
  0 or out of the counters line.
- A stated rule for a stamp ahead of the clock (a wrapped age).
- A decision on whether the entry check applies to `LLKHF_INJECTED` events, whose stamp the injector sets.
  The exit check is QPC-based and does not depend on the stamp.

## Minor (improvements, not blockers)

### M1. E5 notes only right before the next wait, after key events dispatched in the same awake stretch

Where: cut §D5 E5 (cut.md:298-303) and W6 (cut.md:252). `src/hook/main_thread.rs:698-718` (the pump
after the wait) and `:43-47` (sent messages are dispatched first). `:845` and `:879-886`:
`dispatch_sent_messages` runs inside the drain. `MainState::foreground_changed` (`:282-286`) reads the
process name under the borrow, inside the foreground callback (`keyboard_hook_callback.rs:411-426`).

Problem:
- The stall can sit inside a dispatched non-keyboard message, for example the foreground cache's
  process-name read under page-fault pressure, or inside the drain before `dispatch_sent_messages`.
- Either way, it ends with key callbacks dispatched in the same awake stretch, before E5's note.
- E4 entry catches only events that waited at least 17 ms. A key event that arrives right as the stall
  ends is fresh.

Consequence:
1. While CapsLock is held and swallowed, the foreground callback stalls for at least
   2 x `LowLevelHooksTimeout`.
2. CapsLock is released and pressed again inside the stall. Both calls time out, and, if Windows drops
   them, they are never delivered.
3. The release that comes as the stall ends is dispatched by the same pump loop. It is decided with the old
   count and swallowed, and CapsLock stays down.

W6 says "E5 when the thread was busy outside a callback" covers this case.

Confidence: PLAUSIBLE. The ordering is confirmed in the code; the stuck key needs dropped calls and narrow
timing.
What is needed: either the awake-stretch check also runs at the keyboard callback's entry, which already
has `started`, or this case joins R-E1's declaration (I1).

### M2. W-table: two ways Windows gets a key-down that the engine does not see under that key's code

Where: cut W8 and W9 (cut.md:254-255) and R8 (cut.md:836-838). `src/logging/counters.rs:24-26`
(`KeyPressShort`). `src/data/sequence.rs:139-142`, `:169-172` and `:205-208` (short counts are logged).

Problem:
- (a) W8 argues that every injection pairs a key-down with its key-up in one `SendInput`. The code counts
  and logs short counts. A key-down inserted without its key-up leaves the key down in Windows, the
  sentinel hides it from the engine, and nothing notes it.
- (b) W9 says "a repeat of a held, swallowed k is swallowed". That holds per slot only. An injected
  generic `VK_SHIFT`, `VK_CONTROL` or `VK_MENU` passes in its own slot while the swallowed record of the
  sided key stands, and Windows applies it to a side.

Consequence:
- (a) A `PressKey` of a bound, swallowed key comes up short at its key-down. At the next press and
  release of that key, master passes the release, because Windows has the key down, and heals it. After
  this lane the release is swallowed, and the key stays down until a bump and another press.
- (b) Bounded: Windows has the left key down until the injector's own generic key-up, which passes.

Confidence: PLAUSIBLE. Whether SendInput can insert a key-down without its key-up is not documented.
What is needed:
- A short insertion notes hidden input: the TLS for the fast actions, an atomic for the executor. Or W8
  declares the case.
- W9 argues the generic codes.
- One test pins the behaviour change of R8.
- List new feature 6 (`HoldModifier`, which injects holds by design) among the later items that depend on
  this.

### M3. The deferral adds one tick per burst

Where: cut D7 (cut.md:431-434, :473, :482-483) and M6 (cut.md:801). `src/hook/watchdog/decision.rs:138-149`
and `:227-234`.

Problem:
- `last_raw_at` is now set by the first tick's drain, one tick later than today's wake pass.
- Ticking therefore ends one tick later. The cut's own change of "2250..5000" to "2500..5000" shows it.
- D7's Cost and M6 ("the deferral adds no wake") say otherwise.

Consequence:
- A burst costs one more wake: about 10 passes instead of 9 for a single key.
- M6 and M7 still pass (W ≤ 100, W ≤ K/2). A tester who compares W with "adds no wake" would report a
  defect that is not one.

Confidence: CONFIRMED.
What is needed: state the extra wake per burst in D7's Cost, in the module Cost doc (`watchdog/mod.rs:23-42`)
and in the expected values of M6 and M7.

### M4. The compile-red tests of commit 3 are only partly covered by mutations

Where: cut commit 3 tests (cut.md:659-682), the mutation table (cut.md:982-997), and `.claude/agents/tester.md:83`
and `:170`.

Problem: `tester.md` requires a mutation for every compile-red test. M9-M14 cover six of them. None is named
for:
- the E4 exit note;
- E5 (`an_awake_stretch_of_17_ms_notes_hidden_input`, `the_main_loop_checks_its_awake_stretch_before_every_wait`);
- the refused foreground change;
- `unseen_input_counts_each_note`;
- `a_tick_reports_raw_key_events_no_callback_explains`;
- `a_callback_for_a_fresh_event_notes_nothing`.

Consequence: the tester's table shows uncovered compile-red rows. An E5 that never notes (a comparison
inverted, say) also passes the gate unnoticed.

Confidence: CONFIRMED.
What is needed: name one mutation for each remaining compile-red test. Examples: no exit note; E5 never
notes; the E1 refusal is not noted; `unseen_at_last_tick` is always 0.

## Strengths to keep

- **D2.** The key is stamped at its last release, not at its swallowed key-down, and the cut shows the
  case that breaks the spec's own suggestion (manual check 36, known issue 76). The `Unknown` rule in
  `mark_completed`, with its exemption for the key just pressed, is correct as I traced it against
  `keys.rs:185-192` and `mod.rs:788-853`.
- **D1.** The fold is lazy, only at key-ups the engine would swallow. It compares with `!=` (a smaller sum
  counts too), and the engine bumps the count itself for what it is told directly. The engine stays pure.
- **E3** notes in the callback, not in the drain, and notes every signal, "back" included. That covers the
  late signals of known issue 79.
- **S1-S7** each start with a press and release of the key, so `NEVER_KNOWN_UP` cannot make them pass.
  Mutation M3 targets D2 exactly. Most existing 2.3 tests start with a first press and would read anyway.
- **D7.** The Q6 evidence table says how each source was read (fetched or search summary). The argument
  about "the next event of the same HID report" justifies keeping R16's construction. The probe D-R16d,
  with its stop rule, comes before the bound is committed. The deferral cannot spin, and it keeps I3, the
  start of the silence and the evidence timing (checked against `decision.rs` and `run_loop`).
- **Re-location and touch set** are accurate at `50b56a8`. I checked every `impl Env`, every call site of
  `HookEnv::new`, `foreground_changed` and `hook_reinstalled`, `CHANGELOG.md:116` and `:745`, and
  `timers.rs:1-3`.

## Open questions for the architect

1. Does Windows still deliver a low-level hook call after it has timed out? This decides R-E1 and M1. M4
   can record it: keys typed during the debug hold, and a count of late entries.
2. Are out-of-context WinEvent callbacks and low-level hook calls dispatched to the main thread in the
   order they arrive? W1's argument holds anyway, through the foreground change into the hiding window,
   which comes after the key's last release. The order matters only together with M1's stall.

## Absent topics (walked, not applicable)

- §2 Thread affinity: no install or uninstall moves. The watchdog's timer stays on its own thread.
  `HIDDEN_INPUT` is a TLS of the main thread, and `unseen_input` uses `AcqRel` and `Acquire`.
- §1 Hot path: one `GetTickCount` and two compares per callback. At a key-up the engine would swallow, a
  TLS read and one `Acquire` load replace `GetAsyncKeyState`. No allocation, lock or `log::`, and every
  path still ends in one outcome.
- §7 Config: no schema change, and `default_config.ron` is untouched.
- §9 Process boundary: no pipe, token or registry change. I1's R-E2 option would add one read-only read.
- §10 Input synthesis: no new `SendInput`, and the sentinel discipline is unchanged.
- §11 Fit: no `Binding` change. `HookLate` gets its `Display` arm, its `ALL` entry and its slot. The
  CHANGELOG, README and config pages are routed, and the non-config pages are deferred to the book sync as
  the spec asks.
