# Implementation: p2-2.1-executor rework 1

Branch `feat/action-executor`, from `6c83406` to `bb0a0aa` (one commit), tree clean. The input is results.md P1
(REWORK, docs only). The orchestrator wrote no P1 ruling into rulings.md. Because it dispatched this rework, I took
the analyst's recommendation as its decision: accept the ~16 ms granularity and document it. A millisecond-pacing
redesign would be a new architect item. I made no source change (re-check condition 4).

## Items
- results.md P1, condition 1: `CHANGELOG.md:371-380`, a new `[Unreleased]` -> **Changed** bullet citing roadmap
  item 2.1. It covers the thread timer on a clock tick (about 15.6 ms), the raise to 10 ms, that a `Delay` never ends
  early, the average and worst lateness, that `Delay(1)`..`Delay(9)` equal `Delay(10)` (about 16 ms), the old
  `thread::sleep` (about 1 ms), the slower short-Delay macros (100 x `Delay(5)`: about 1.6 s instead of about 0.5 s),
  and that `Delay(0)` waits for no timer. Not marked Breaking, as the analyst asked (a Changed bullet).
- P1, condition 2: `book/src/config/actions.md:318` (section 14.3). I dropped "as with the `thread::sleep` it
  replaces" and now state the tick, the 10 ms raise, the lateness, the old precision with a link to roadmap
  [item 2.1], and the slowdown.
- P1, condition 3: `README.md:537`, the `Delay(<ms>)` row now reads "Wait at least `<ms>` milliseconds ... on a
  system clock tick (about 15.6 ms): about 10 ms longer on average, and `Delay(1)` to `Delay(9)` last about 16 ms".
  I left `src/config/default_config.ron:151` ("wait N ms on the action executor's timer") unchanged. It makes no
  precision claim, so it does not contradict the new text, and changing it was optional.
- P1, optional reload wording: the four places that said "a reload (saving `config.ron`)" now say "a reload (a saved
  `config.ron` that loads)". Three of them also add that a save that does not load keeps the running macros (and
  the queued actions in section 13.1), as it keeps the bindings:
  - `CHANGELOG.md:511-517` (the "No cancellation" bullet);
  - `book/src/config/actions.md:51` (section 13.1);
  - `book/src/config/actions.md:297` (section 14.1, wording only);
  - `README.md:511-515`.

  Evidence: `src/hook/config_watcher.rs:157-169`. Only `Ok` calls `update_bindings`, and only that calls
  `cancel_actions` (`src/hook/keyboard_hook.rs:120-127`).
- Red-first: not applicable, because the rework is docs only and the results ask for no test.

## Remarks
- results.md P1 (the undeclared Delay pacing change and the false parity text in actions.md:318): RESOLVED at
  `CHANGELOG.md:371-380`, `book/src/config/actions.md:318` and `README.md:537`. It is pinned by
  `config::loader::tests::readme_matching_examples_load`, which includes README.md at compile time and stays green.
  No test can pin prose about timing.
- Optional reload wording: RESOLVED at the four locations above.
- Review m1 (a `///` line on `Executor::thread`): not done. results.md files it under the orchestrator hand-offs as
  optional, and re-check condition 4 says "No source change". It stays open for the orchestrator.

## Deviations from the cut
- The wording differs from the results' re-check condition 1 in two facts. In both cases my own measurement
  contradicts the prescribed claim, so I wrote what I measured:
  1. Prescribed: "any `Delay` can end up to about one clock tick (about 15.6 ms) late". Written: "on average about
     10 ms after its time and up to about 30 ms (two ticks) after it". The analyst's own table already shows more
     than one tick (`Delay(50)` max 74.5 ms = 24.5 ms late). My probe shows a late max of 23-30 ms on every row from
     `Delay(16)` up, in four runs. One cause is visible: the timer sometimes fires before the `Instant` deadline. The
     loop then re-arms for the rest, which costs another 10 ms raise plus a tick. "re-armed" counted 9/60 and 14/60
     at `Delay(16)`, 18/60 and 21/60 at `Delay(250)`, and 7/200 at `Delay(100)`.
  2. Prescribed: "a `Delay` under 10 ms waits at least 10 ms". Written: "Windows ... raises a timer shorter than
     10 ms to 10 ms, so ... `Delay(1)` to `Delay(9)` last as long as `Delay(10)`: about 16 ms on average". The probe
     saw a few short Delays end under 10 ms: 7/200 at `Delay(1)` (min 5.9 ms), 4/200 and 2/200 at `Delay(5)` (min
     5.6 ms), and one `Delay(1)` at 1.8 ms. A strict "at least 10 ms" is therefore false. The documented 10 ms raise
     (USER_TIMER_MINIMUM) is still stated, as the reason short Delays behave like `Delay(10)`. "Never ends early"
     still holds, because `Instant` decides and every sample was at least its own `ms`.
- Probe: a scratch binary in the session scratchpad (`delayprobe`), outside the repo, built `--offline` against the
  lane's `Cargo.lock` (`windows` 0.62.2). On a thread of its own it does what the executor's wait does. It sets a
  deadline with `Instant`, arms `SetCoalescableTimer(None, 0, timer_ms(now, at), None, TIMERV_NO_COALESCING)` with the
  executor's `timer_ms` rounding, waits in `GetMessageW` for that `WM_TIMER`, calls `KillTimer`, and re-arms while
  `now < at`. It also times `thread::sleep` for comparison. It installs no hook, injects no input and writes no
  registry. Results (ms):

  | Delay | min | avg | max | late max | re-armed |
  |---|---|---|---|---|---|
  | 1 | 1.8-12.2 | 15.5-16.1 | 19.6-31.2 | 18.6-30.2 | 0 |
  | 5 | 5.6-12.2 | 15.6-15.8 | 19.4-30.3 | 14.4-25.3 | 0-1 |
  | 9 | 10.4-10.6 | 15.5-15.7 | 20.0-21.3 | 11.0-12.3 | 0/60 |
  | 10 | 13.0-15.1 | 15.6-16.0 | 17.9-31.5 | 7.9-21.5 | 0-1/60 |
  | 16 | 16.0-16.1 | 28.2-28.3 | 40.2-45.7 | 24.2-29.7 | 9-14/60 |
  | 20 | 20.2-27.8 | 31.2-31.5 | 34.6-43.4 | 14.6-23.4 | 0 |
  | 50 | 50.1-59.2 | 62.5-62.7 | 67.8-78.5 | 17.8-28.5 | 0 |
  | 100 | 101.2-105.7 | 109.3-110.7 | 115.3-125.9 | 15.3-25.9 | 0-7 |
  | 250 | 250.1 | 259.0-260.8 | 269.2-280.3 | 19.2-30.3 | 18-21/60 |
  | 0 | 0.0 | 0.0 | 0.0 | 0.0 | 0 (no timer) |

  `thread::sleep` avg / max on the same machine: 1 -> 1.45 / 1.98; 5 -> 5.34 / 7.63; 20 -> 20.63 / 29.50;
  50 -> 50.45 / 52.78; 100 -> 101.55 / 119.99. This confirms the analyst's premise (high-resolution sleep, about
  1 ms). 100 x `Delay(5)` is about 1.57 s with the timer and 0.53 s with sleep.

## Unsafe / atomics added
- none (no source change)

## Gates
- `cargo fmt --check`: pass (exit 0). check and clippy: not rerun, because no Rust source changed.
  `config::loader::tests::readme_matching_examples_load` covers the README through `include_str!`, so its rebuild is
  the compile check for the touched input: 1 passed.
- `mdbook build book -d D:/tmp/ks-orch/p2-2.1-executor/rework1-book`: OK (only the known mdbook-mermaid version
  warning). The new link `../roadmap.md#21-action-executor` uses the anchor that actions.md:297 already used.
- Added lines are ASCII except the pre-existing `§` on actions.md:51. `default_config.ron` is untouched.
- Tests run: `readme_matching_examples_load`. The full suite is the tester's job.
- Manual ignores added: none.

## Docs
- CHANGELOG `[Unreleased]`: a new Changed bullet (Delay pacing, roadmap 2.1), and the Fixed "No cancellation" bullet
  reworded (a reload that loads; a failed save keeps the macros).
- `book/src/config/actions.md` sections 13.1, 14.1 and 14.3; `README.md` Sequence section and `Delay(<ms>)` row.
- Left for the Phase-2 book sync, as results.md hands off: `book/src/roadmap.md:786` open question 7 (extend it to
  macro `Delay`, and its "about 15.6 ms" now has measured company: up to about two ticks with the re-arm).

## Commits
- `bb0a0aa` Docs: a macro Delay ends on a system clock tick, no longer within 1 ms

## Open points / pre-existing warnings
- Orchestrator: rulings.md R1 line 22 ("the same as the old `thread::sleep`") and cut D6 still carry the false
  premise. Correcting them is the orchestrator's job (results.md hand-off).
- The early-firing timer adds a second 10 ms raise plus a tick to some Delays (the "re-armed" column). It is not a
  defect against the rulings (`Instant` decides, and nothing ends early), but it explains the up-to-two-ticks bound.
  If millisecond pacing is ever wanted, this is the item for the architect (see results.md P1, the
  `MsgWaitForMultipleObjectsEx` option).
- Review m1 (`Executor::thread` doc line) and review m2 (the tester's 200 ms margins) are still open, as listed
  above.
- Pre-existing warnings: none in the touched gates.
