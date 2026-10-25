# Code review: p2-2.4-watchdog (round 2) @ 959fce1
CHANGES_REQUESTED (Critical 0, Important 1, Minor 2)

Scope: `git diff 140b831..feat/hook-watchdog` (20 commits, 8 files), read through `git show 959fce1:<path>`. I read these in full: `src/hook/watchdog/mod.rs` (production part 1-1195, plus every new test), `src/hook/watchdog/decision.rs` (production part and the V3-I1 rows), the `src/bench.rs` diff, the S4 scan in `src/hook/installer.rs`, the `CHANGELOG.md`/`README.md`/`errors.md`/`counters.rs` diffs, and `core/app.rs:300-336` for the stop order. I checked the work against `spec.md`, `rulings.md` (rounds 2 and 3), `cut.md` "Delta after ruling L1" plus the Developer addendum, `triage_r1.md` and `fix_r1.md`.

## Gate: fmt ✅ · clippy ✅
- Tree: `git archive 959fce1` exported to `D:/tmp/ks-orch/p2-2.4-watchdog/review-src-r2`, which the tester cannot touch. `CARGO_TARGET_DIR=D:/tmp/ks-orch/p2-2.4-watchdog/target-review`.
- `cargo fmt --check`: exit 0, empty output (`review-fmt-r2.log`).
- `cargo clippy --all-targets --locked -- -D warnings`: exit 0 (`review-clippy-r2.log`). I re-ran it after touching the crate roots, so the line `Checking key-switch-rs` is in the log: the crate was re-checked, not replayed.
- I did not run `cargo test`, because the tester owns it.

## Round-1 items (triage_r1.md)
- **T-F1 = R-I2** (R14 regression): ✅ closed in code by ruling L1, as built in `a0aaf7c`.
  - While the watchdog ticks it does no work per key event: the mask is `wake_mask(.., true)` (`mod.rs:750-756`), the pump skips `WM_INPUT` (`mod.rs:764-803`), and each tick drains the queue.
  - Developer's single run: R14 p50 equals R14c p50 (4.8 us); R14 p99 is +3.8 us over R14c, above the +2 us rule. The tester's 3 runs decide (Open questions 1).
  - The new row R16 misses its own bound: I1 below.
- **R-I1** (S4 scan): ✅ closed (`ee9ef4d`/`0e7165b`, test code only).
  - `scan_source` (`installer.rs:486-512`) reads every line of every file, test modules included.
  - `INSTALLERS` (`:386-392`) matches the prefix `SetWindowsHook`.
  - `scanned_sources` (`:414-419`) adds `build.rs`.
  - The three red tests (`:527-577`) fail on the old scan for the right reason (fix_r1: 3 failed on `ee9ef4d`).
  - A subfolder under `tests/` would make `read_to_string` panic (`:612`), which fails loudly, not silently.
  - Residual limits are lexical only: a `'"'` char literal before a call on the same line; multi-line strings; block comments. None is a reachable false negative in this tree.
- **R-m1**: ✅ closed (`counters.rs:54-57`).
- **R-m2, R-m3, R-m4, T-note**: NOTED, and ruling Delta Q5 keeps them outside this delta. Not re-opened.

## Remarks

### 🔴 Critical
None.

### 🟡 Important

#### I1. Bench row R16 misses the sanity bound that ruling Delta Q2 accepted (spec acceptance "no unjustified regression"; ruling L1 / V3-M2)
**Where**: `src/bench.rs:583-606` (`r16_first_key_of_a_burst`, n = 1000 at `:1128`); the bound is `cut.md:984`, accepted at `rulings.md:133-135`; the first pass is `src/hook/watchdog/mod.rs:688-744`. **Confidence**: CONFIRMED (the developer's measured run plus diagnostics; I did not re-run the bench).

**Problem**:
- L1 keeps one watchdog wake per burst and "declares" it (`rulings.md:78-79`).
- The cut bounds that declared cost: R16 p50 <= R12 p50 + 25 us, p99 <= R12 p99 + 100 us. The orchestrator accepted the bound ("Delta Q1-Q6 accepted ... R16's sanity bound").
- The row as built does not meet it, and no ruling restates it.

**Failure**:
- `bench-dev-r1.md` row R16: p50 53.5 us, p99 168.4 us, against R12 4.7 / 5.9 us. The bounds are 29.7 / 105.9 us, so the row misses both, p50 by about 2x.
- `bench-dev-diag.md` shows the cost is systematic, not noise:
  - D-A 51.7 us; D-B (no `raw_batch`, no timer set) 53.6 us; R16 with the bench thread at `HIGHEST` 58.5 us.
  - D-D, a warm thread idle 2.4 s before each sample, 79.7 us.
  - D-pre, R16's construction on pre-L1 `0e7165b`: 35.6 us, also above the bound.
- So the tester's 3 runs will not bring it under the bound. In the daemon the first key after >= 2 s without keys pays this when its raw input reaches the watchdog before its hook call (cut Q6, unknown). The README states the number honestly (`README.md:939`, "about 50 microseconds"). The open issue is the bound, not the README.

**What to do**: this needs an orchestrator ruling before the merge, as T-F1 did in round 1. Either:
- restate R16 as a declared evidence row with its measured numbers (the bound was derived from the pre-L1 R14, whose posts went to a busy thread; `fix_r1.md:147`); or
- open a delta that trims the idle wake's first pass. One direction: on the idle wake, only start ticking and arm the timer, and leave the pump and drain to the first tick. Whether that helps is unproven, since D-B shows the decision's own work is not the cost.

The code itself is not wrong.

### 🟢 Minor

#### m1. The errors page tells users that a correct build's stop line is a defect
**Where**: `book/src/config/errors.md:215`: "so N stays far below M ... N about as large as M says that Windows wakes it for every key event, which is worth reporting". **Confidence**: CONFIRMED (arithmetic from `decision.rs:68,70` and `keeps_ticking` at `:227-234`).

**Failure**:
- Each burst costs 1 wake plus a tick every 250 ms until the first tick at or after `last_raw_at + ACTIVE_FOR`. That is about 10 wakes per burst whatever its length.
- Take a run of 100 Ctrl+C presses, each more than 2 s apart: 4 raw events each, so M = 400.
- That run gives N of about 1000 = 2.5 M on a correct build. The row tells this user to report a defect.
- N < M holds only for sustained typing (about 0.4 M for prose at 10 events/s).
- M9's rule (`W <= K / 2` over a 20 s hold) is calibrated for its own script, not for the general case the page describes.

**What to do**: describe N as "about 4 a second while keys are typed, plus about 10 per isolated burst, and nothing idle". Name the per-key symptom as N about M during sustained typing, not N about M in general.

#### m2. The batched read leaves the last batches' key codes in a thread-lived buffer (spec "No key data from Raw Input is kept")
**Where**: `src/hook/watchdog/mod.rs:659` (`let mut buffer = RawBuffer([0; RAW_BUFFER_BYTES]);`, alive for the whole thread), `:1079-1097` (each `GetRawInputBuffer` writes from offset 0), `:884-917` (the walk reads the bytes and leaves them). **Confidence**: PLAUSIBLE.

**Failure**:
- A large batch, for example 76 records read after a stall or an injected burst, writes VKey and MakeCode for 76 events in order.
- Later tick batches of about 10 records overwrite only the first about 400 bytes. Records 10..75 of the old batch stay on the watchdog thread's stack until another batch that large comes.
- Also, `type_text` injects with `KEYEVENTF_UNICODE` and `wScan` = the character (`data/sequence.rs:153-155`). If raw input reports `wScan` as MakeCode, those bytes are typed text (unverified).
- A crash dump or debugger snapshot of the elevated daemon then holds that key history.
- Pre-L1 the residue was one 48-byte `RAWINPUT` local that the next call overwrote. `README.md:939` says "It keeps no key data".

**What to do**: either zero the written prefix after `count_raw_keys` (at most 3 KB at 4 Hz), or state the transient residue as accepted in the module's "# No key data" section.

## Checklist walked (absent items, no reachable failure)
- **S1-S4**: no new installer call. The synthetic source never registers (`mod.rs:634-641`). The scan now covers test modules and `build.rs`. The OQ3 smoke test (`:2319`) only reads its own thread's empty queue, with a `SinkWindow` and no registration. No test installs a hook, injects input or calls `logging::init`/`attach_file` (diff grep).
- **R1 hook callback**: `keyboard_hook_callback.rs`, `keyboard_hook.rs` and `main_thread.rs` are untouched in this delta. No new code runs on the callback or on main. No log call, I/O, allocation, lock or panic path is added to the key path.
- **R2 sentinel**: no new `SendInput` path. `the_watchdog_injects_no_input` (`mod.rs:2690`) still scans the watchdog, decision, health, installer and the 4 reinstall functions.
- **R3 shared state**: no new production shared state. `LoopStats` crosses threads only through `done` (`mod.rs:456-466`). `RefCell`/`Cell` appear only in the test-only `FakeTimer`/clock helpers.
- **R4 orderings**: no new atomics in production. The test `Recorder` uses `fetch_update(AcqRel, Acquire)`, unchanged.
- **R5/R7/R8/R9**: not touched.
- **R6 Display**: no new `Counter` variant. `Read`, `Drained`, `BlockCount`, `LoopStats` and `Plan` are internal and `Debug`-only. `RawBuffer` has no `Debug`, so its bytes cannot be formatted.
- **R10 loop and shutdown**:
  - The thread pumps on every wake, with no window filter (OQ2). The keyboard source keeps `QS_POSTMESSAGE` in both masks (`mod.rs:368`, `:750-756`), so `WM_QUIT` ends any wait at once.
  - On quit the loop drains once more into the stats only (`:737-741`). The stop line is queued before `done` is sent (`:672-677`), and `App::run` stops the watchdog after `uninstall` and before main's `logging::flush` (`app.rs:316-322`, `main.rs:52`).
  - The registration drops before the window, on the owning thread (`:630-641`, `:599-605`).
  - The tick timer is auto-reset (`platform/timers.rs:117-121`, no `MANUAL_RESET`), so an idle transition with no `cancel` cannot leave it signalled and spinning.
- **Lost wake-ups**:
  - Idle waits use `QS_ALLINPUT` plus `MWMO_INPUTAVAILABLE`, so raw input that arrives between the drain and the wait ends the wait.
  - While ticking, the tick comes from the timer or from the `wake_by` timeout (I1). Every wake runs `pass` and then `arm`.
  - A drain that spends its budget gives a timeout of 0 (`:1056-1059`).
  - A false `stuck` needs the standard read to miss a `WM_INPUT` that the queue bit reports, which cannot happen between two consecutive calls.
- **Decision (V3-I1)**: I traced these against `decision.rs:171-222`:
  - critic trace: `kept` is held while `backlog`;
  - dead hook during the burst: unexplained raw input becomes evidence;
  - credit lapse: older than one interval only without a backlog;
  - baseline at live ticks.

  Callback-before-raw skew within one event is covered by the one-interval surplus credit. Rule 4 holds: `raw_batch` runs before `take_answer` (`mod.rs:240-247`). M5 holds: `decide` runs before `evidence +=` (`decision.rs:196-199`). M1 holds (`keeps_ticking`).
- **Unsafe**: all 31 `unsafe` sites in `mod.rs` and the 3 new ones in the bench (`bench.rs:532-553`) carry a concrete `// SAFETY:`.
  - `GetRawInputBuffer`: buffer 8-aligned, `size` reset to its byte length before each call, header size `RAWINPUTHEADER`'s; `u32::MAX` and 0 are handled.
  - `count_raw_keys` is safe code with bounds-checked `get`, and every step advances by at least the header size.
- **Tests can fail**: red-first was observed on the parent commit for every pair (pack table 1-9). The `Drive` model makes a pass on a raw event only while `wake_mask` includes `QS_RAWINPUT`. Characterisations F1 and OQ2 run on spawned threads with their own queues. `PostQuitMessage` in OQ2 affects only that spawned thread.
- **Rulings**:
  - L1, V3-I1, V3-I2, V3-M2, V3-M3, OQ2, OQ3, OQ4 (recorded) and Delta Q1/Q3-Q6 are carried out as written.
  - V3-M1: the M9 rule is `W <= K / 2` with `K >= 400` instead of the ruling's example, and the developer's reason checks out (the 20 s hold alone is about 90 wakes).
  - Delta Q2 (R16 bound): see I1.
- **Docs**:
  - The CHANGELOG has `[Unreleased]` Added entries for the watchdog and the bench, edited in place.
  - README `:905` and `:939` give the measured R16 number and the M6 system-side cost.
  - errors.md has a new row and a reworded counters row (see m1).
  - No config meaning changed, so no config page is due.

## Positive
- The drain is a pure function of `RawReads` plus a clock, and `next_wait` is pure. So the time budget, the V3-M3 stuck guard and the never-wait-while-pending rule are all tested on fakes, with no Win32.
- `count_raw_keys` walks plain initialised bytes at `offset_of!` offsets with bounds-checked reads. No typed `RAWINPUT` padding is ever read, and no `unsafe` is needed in the walk.
- The OQ2 characterisation (`mod.rs:2561`) caught a real error in the cut's E7 reading, a posted `WM_QUIT` that a filtered peek does not return. The pump design was confirmed against it rather than assumed.
- The decision's credit rule is stated in its docs (`decision.rs:16-27`), and each half of V3-I1 is pinned in both directions, including 10-minute drift rows.

## Open questions
1. **R14 p99 vs R14c p99**: +3.8 us in one developer run, against the +2 us rule. Both p99s sit near 100 us and move by tens of us between runs. Settled by the tester's 3 runs on HEAD.
2. **Two M9 rules**: `cut.md:1092` (L1.12, the architect's M9 row) still says `K >= 150` and `W <= 130`, while the addendum (`cut.md:762`) and `fix_r1.md:109` say `K >= 400` and `W <= K / 2`. Ruling V3-M1 asks for one rule. The orchestrator should hand the owner only the `fix_r1.md` text.
3. **E4 for `QS_RAWINPUT`** (does a pending `WM_INPUT` outside the mask leave the wait asleep): only manual check M9 settles it. If it fails, the cost reverts to one wake per key event, never to a spin.
4. **m2's weight**: does raw input report a `KEYEVENTF_UNICODE` event's `wScan` as `RAWKEYBOARD.MakeCode`? A Microsoft Learn or researcher check would settle it.
