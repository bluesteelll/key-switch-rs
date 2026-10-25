# Lane spec: p2-2.5-latency-trim - Latency trim (owner decision on roadmap open question 13)

Branch: perf/latency-trim

Roadmap item 2.5 (book/src/roadmap.md 'Latency trim'). The owner asked to trim, before Phase 3, the two key-path costs that items 2.3 and 2.4 declared. The owner decision that binds everything here: binds run with the lowest possible latency (CLAUDE.md), and no key may ever stick.
1. The OS witness at a swallowed key-up (item 2.3, src/engine/mod.rs around :885, bench row R11: about +0.6 us). Read the OS key state only when something that can hide key events from the hook happened after that key's key-down. Candidates:
   - a foreground change (UIPI: an elevated window hides events from a non-elevated hook);
   - a lost-input reset (lock, desktop switch);
   - a remapping switch;
   - a hook reinstall;
   - input the watchdog saw without a callback (a hook timeout or removal: the 2.4 heartbeat against raw input; the watchdog may bump the same epoch through an atomic).
   One way is an epoch that those events bump, recorded per key at its swallowed key-down; the witness reads only when the key's epoch is older than the current one. The design must keep every stuck-key test of 2.3 green, including the critic's unseen re-press script (down, foreground_changed, up_unseen, down_unseen, foreground_changed, up), and must argue every way an unseen key-down can happen without an epoch bump (cite Microsoft Learn; an unknown case keeps the read). The engine stays pure: the epoch is data the adapter feeds in.
2. The first key after an idle watchdog (item 2.4, bench row R16: about 50 us at the worst order). First settle, with evidence, cut question Q6 of item 2.4: does Windows generate raw input (WM_INPUT for RIDEV_INPUTSINK) before or after the low-level keyboard hook chain, and does a key the LL hook swallows produce raw input at all? Search Microsoft Learn, Raymond Chen's blog, and well-documented prior art (for example AutoHotkey, Interception, game input libraries). If the evidence shows the wake can never overlap its own key's callback, say so, and make R16 measure the realistic order. Then remove whatever cost remains, if any. Options include a timer-only idle poll at a slow rate instead of a raw-input wake, or less user32 work on the first pass. Each option's idle cost (wake-ups per second while idle) goes into the cut, and the cut's choice is argued against the owner's priority. Keep every 2.4 guarantee: detection within the acceptance bound, no false reinstall, no spin, no idle wake-ups unless the cut argues they are cheaper than the latency they save.
3. Leftovers from the Phase-2 book sync (D:/tmp/ks-orch/phase-2/book.md, 'follow-ups for the developer'): CHANGELOG.md around :744-745 still says a Hold runs its fast actions 'on its timer thread'; src/platform/timers.rs:2-3 still says the gesture timers of 2.2 come 'later'.
Acceptance:
- Bench R11 (the key-up of a swallowed key pressed and released in one window) is within the D14 rule of the ordinary key-up R12 (CLAUDE.md bench rules); a new row shows the witness still reading after a foreground change.
- Every stuck-key and lone-key-up test and property of 2.3 stays green; new scenario tests pin each epoch source (a foreground change, a reset, a remapping switch, a reinstall, a watchdog-seen gap re-enable the read).
- R16 meets a bound the cut sets from the evidence, or the cut shows from documented evidence that a real key never meets the watchdog's wake and redefines R16 to the realistic order; no other bench row regresses (tester runs the bench on master and HEAD per tester.md 4b).
- The 2.4 watchdog acceptance (unit rows with a fake clock) stays green.
Update CHANGELOG (Changed, citing roadmap 2.5), the README latency notes and the book config pages only where a stated cost changes (errors.md counters, if any). Non-config book pages go to the next book sync, listed in impl.md with file:line.
