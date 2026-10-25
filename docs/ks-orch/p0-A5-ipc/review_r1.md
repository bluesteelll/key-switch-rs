# Code review: p0-A5-ipc (fix/ipc-control) @ 2baadf1
APPROVED (Critical 0, Important 0, Minor 3)

Scope: `git diff master...fix/ipc-control` (master `b056b72`), 9 commits `04b5375..2baadf1`, read through
`git show`/`git diff` only. Note: the pack was written for HEAD `2b32441`; the branch moved to `2baadf1`
(a test-only commit, `src/hook/ipc_server.rs` +27, see m3) while this review ran. Both heads were gated.

## Gate: fmt OK, clippy OK
- Run on a `git archive` export of the lane ref (not the work tree, not the tester's `wt/mut`), own target dir:
  - `2b32441`: `cargo fmt --check` rc=0; `cargo clippy --all-targets --locked -- -D warnings` rc=0.
  - `2baadf1`: `cargo fmt --check` rc=0; `cargo clippy --all-targets --locked -- -D warnings` rc=0.
- `cargo test` not run (tester owns it).
- Acceptance rg on the lane ref (`git grep`): check 1 matches only `src/hook/keyboard_hook.rs` (`:8` use,
  `:170`/`:225`/`:244` comments, `:174`, `:192`, `:234`, `:246` inside `install`/`uninstall`); check 2 matches
  only `app.rs:131`, `app.rs:189` (`App::run`, main thread) and `keyboard_hook.rs:162/205/226/261`;
  check 3 (`KeyboardHook|keyboard_hook|install|fn (enable|disable)` in `ipc_server.rs`, `ipc.rs`, `swch.rs`)
  prints nothing, rc=1. No `SetWindowsHookExW` is reachable from IPC code: `ipc_server` cannot name
  `KeyboardHook`, and `DaemonHandle` (`app.rs:32-58`) calls only `set_enabled`/`is_enabled`/`bindings`.

## Remarks

### Critical
None.

### Important
None.

### Minor (optional)

#### m1. The callback's fast gate is not pinned by any test (R1)
**Where**: `src/hook/keyboard_hook_callback.rs:245-251` (gate), `:402-406` (K3 comment)  **Confidence**: CONFIRMED (traced)
**Problem**: K3's comment says it tests "the gate before the engine lock", but it cannot tell that gate
from the re-check under the lock (`:178-182`). **Failure**: delete `:245-251`. While remapping is off,
`call(f13, WM_KEYDOWN)` then reads `lparam`, enters `decide_event`, finds `!is_enabled()` under the lock,
gets `None` and still returns `CallNextHookEx`. So K3's off-asserts (`:446-452`) pass, and K2 (which
calls `decide_event` directly) passes too. The spec's "checked first ... when disabled, the callback only
calls CallNextHookEx" then holds only by code reading. The observable cost of that regression is small:
every disabled event would take the bindings snapshot and the engine lock, and it would wait out a reset.
**What to do**: either pin the gate (for example, in K3, hold the global hook's engine lock on another
thread and assert that the disabled callback returns within a bound), or narrow the K3 comment to what it
actually proves.

#### m2. README still says `open` does not use the pipe
**Where**: `README.md:548` ("All commands except `open` connect to the daemon's named pipe")  **Confidence**: CONFIRMED
**Problem**: since `bf669f3`, `swch open` sends a `status` request over the pipe (`src/bin/swch.rs:76`). The
swch module doc (`swch.rs:15-18`: "Every command, `open` included, sends one request ...") and the README table
row (`README.md:541`) both say so. Line 548 contradicts them. impl.md:101 records that it was kept on purpose.
**Failure**: a reader who reads only that paragraph concludes that `swch open` cannot be stopped by a hung pipe or
an access-denied pipe, which is exactly what the lane changed.
**What to do**: reword the sentence, for example "Every command connects to ...; `open` uses it only to check
for a running daemon."

#### m3. Commit `2baadf1`: the wrong trailer, committed by a role that makes no commits (process)
**Where**: commit `2baadf1` ("Tests: pin the pipe SDDL shape ..."), trailer `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`  **Confidence**: CONFIRMED
**Problem**: CLAUDE.md "Git flow" and docs/WORKFLOW.md:233-236 require every commit to end with
`Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`, and WORKFLOW.md:23 gives the tester "no commits".
The commit moved the lane HEAD after the context pack (`2b32441`) was written. Its body quotes no failing
line, so the red-first rule is not met either. The test itself is sound: it pins the literal shape of
`PIPE_SDDL` (three DACL ACEs, protected DACL, `S:(ML;;NWNR;;;ME)`), and its comment does not overclaim.
**Failure**: a `--no-ff` merge carries the off-policy trailer onto `master`, and history cannot be rewritten
afterwards (no amend, no force).
**What to do**: orchestrator decision. Accept the commit as it is, or have the developer re-land the test
before the merge in a correctly attributed commit, on a lane rebuilt without `2baadf1`. This is not a code
defect.

## Checklist walked (absent items, one line each)
- **Plan conformance**: D1-D10 implemented as cut. The deviations listed in impl.md:54-60 are justified. The critique addendum's items (I1 M5, I2 rg check 3) are verified on the ref.
- **Spec and acceptance**: the gate is checked first in the callback; `on`/`off` only flip the flag; the reset runs on a real transition only (`keyboard_hook.rs:146-155`); `status` reads the flag (`app.rs:40-45`); the SDDL goes in through `ListenerOptionsExt::security_descriptor` with a comment per ACE (`ipc_server.rs:61-77`); P1/P2 parse; the integration test T1 runs on a unique name with a stand-in control; T2 covers the 5 s deadline; the placeholder is gone.
- **R1**: `code < 0` comes first (`:238-243`). The gate is one `Acquire` load before the `lparam` deref. Every path returns `CallNextHookEx` (`:242`, `:250`, `:267`) or `LRESULT(1)` (`:261`). No new indexing in the callback; `reset_from_os` indexes `usize::from(code)` with `code <= 254`.
- **R1/hot path**: no allocation, I/O, sleep or spawn added to the callback. Under the lock the callback adds only one `Acquire` load. The reset (at most 249 `GetAsyncKeyState`, no foreground capture) runs on the IPC worker. The hook thread waits for it at most once per toggle and nothing pumps messages under the lock, so there is no deadlock or re-entry.
- **Suppression invariant**: (a) a race between the flip and the lock lets a key-down that passed the fast check see `false` under the lock, so it passes, and its key-up passes too: symmetric. (b) A key-down swallowed before `off` gets its key-up passed while off: a lone key-up, the known RK6, pinned by S2 and documented in the CHANGELOG. (c) A key held through `on` is seeded from the OS as `Passed`, so its repeats and key-up pass (S6). (d) A no-op toggle does not reset (K1, K1b). RK7 (a key-down within microseconds of the reset) is documented on `reset_from_os`.
- **Other binding kinds and triggers**: with no toggle, the enabled path is byte-for-byte the old decision (the fast check is true, the locked re-check is true, then `on_key`). Combo, Sequence, Chord, Tap, Hold and DoubleTap logic is untouched. `Engine::reset` drops every engine field (`keys`, `gestures`, `chords`, `sequences`), and chords and sequences have no replay to lose.
- **Timers (KS-11/KS-25)**: `Gestures::clear` keeps `last_generation` (U1, S5). A stale `hold-timer` finds nothing (S1, K1b). RK8 (an `on_timer` that ran just before the reset) is an accepted race.
- **R2**: no new `SendInput`. While off, injected and physical events both pass unread; the sentinel check still runs before any state mutation (`:166-168`).
- **R3**: no `static mut`, `RefCell` or new hook-wide lock. `with_engine` remains the only engine accessor; `set_enabled` is a new taker on the IPC worker, and its nested refusal is pinned by K1c. `install`/`uninstall` are `pub(crate)` and called only from `App::run`; the handle fields are private. The KS-31 check-then-act on install is no longer reachable from two workers.
- **R4**: `enabled` uses `swap(AcqRel)` under the lock and `Acquire` loads, with a pairing doc on the field (`:50-59`). One `MAIN_THREAD_ID` remains (`app.rs:20`), and `DaemonHandle.main_thread_id` is published through `Arc` and the thread spawn. The test `COUNTER` uses `Relaxed` with a written justification.
- **R5, R7, R9**: untouched: no builders, no specificity change, no registry.
- **R6**: `RequestError` has one `Display` arm per variant (`ipc.rs:73-83`), with no `_ =>`.
- **R8**: `reset_from_os` writes sided and generic codes the way `sync_modifiers` does. The generic slots are resynced from the OS on every event, so a seeded `VK_SHIFT` cannot stick. Mouse buttons are skipped before `os_key_down` is called (`||` short-circuit).
- **R10**: the main thread installs once, uninstalls after the loop, and `exit` and Ctrl+C both post `WM_QUIT`. `UnhookWinEvent`'s result is now logged (D10). The startup order is unchanged (the KS-20 window belongs to 1.5).
- **Unsafe**: the new blocks have concrete SAFETY comments: `keyboard_hook_callback.rs:247-250`, `keyboard_hook.rs:231-234` and `app.rs:51-55`. The pre-existing blocks without SAFETY (`app.rs:115`, `:122`, `:180`, `:202`) are on untouched lines and are not exposed by the diff.
- **Win32 errors and handles**: `PostThreadMessageW` failure becomes `ERR:` (before, it was a silent `OK`). `classify_connect_error` uses `u32::try_from` and casts nothing. No raw handles are opened by our code; interprocess owns them.
- **KS-02 SDDL**: SY/BA full (GA); IU gets GR+GW, and the GW is required because the interprocess client opens with GENERIC_WRITE (FILE_APPEND_DATA == FILE_CREATE_PIPE_INSTANCE, a trade-off D6 documents); `ME` NW+NR; no `O:`/`G:`, because interprocess `try_clone.rs:66-69` writes the group into the owner slot (verified in the 2.4.2 source). The first instance uses `FILE_FLAG_FIRST_PIPE_INSTANCE` (`create_instance.rs:86`), so a second daemon still cannot share the name.
- **swch client**: one deadline covers connect, write and read: `ConnectWaitMode::Timeout` (`ctor.rs:55-77`) inside the thread and `recv_timeout` outside. There is no flush; the server has always read the line before it replies. The helper thread is abandoned on timeout, and swch exits anyway.
- **Tests that can fail**: P2 shows the parse rejects bad input. T2 fails after about 30 s (the dummy drops the connection) if the outer deadline is removed, so it does not hang forever. K2 fails if the locked re-check is removed. K1b fails if a no-op toggle resets. S5/S6 have a declared red commit (`867da7c`) with quoted lines. No un-ignored test installs a hook, injects input or writes the registry: K3 calls the extern fn directly with no hook installed, and A1/A2/K1* use local or leaked hooks and read only `GetAsyncKeyState`.
- **Docs**: the CHANGELOG `[Unreleased]` has Fixed entries KS-01 (with KS-31), KS-02 (with the cross-user caveat) and the swch timeout (KS-20 partly, KS-39), and Added `tests/ipc_roundtrip.rs`, in Keep a Changelog form. The README is updated except m2. No `config/*.md` meaning changed. The book drift (`architecture.md:170-172` and others) is deferred to the phase book sync by cut Q5, an orchestrator decision.
- **Style**: no new `#[allow]`, no new dependency (`Cargo.toml`/`Cargo.lock` unchanged), no new non-ASCII beyond the moved `✓` line, no `unwrap`/`expect` in new non-test code.

## Positive
- The `DaemonControl` seam makes the KS-01 rg acceptance structural: the server module cannot name the hook, and the tests drive the real server with a stand-in, so a later regression cannot install a hook from `cargo test`.
- The re-check under the lock closes the real race: the fast check passes, `off` resets, then a Hold is armed. The flip and the reset share one critical section.
- Keeping the Hold generation across the reset (S5) and seeding held keys as `Passed` (S6) are exactly the two places where a naive reset would reintroduce KS-11 or a stuck key. Both have behavioural red-first tests.
- The client design (one helper thread, one deadline) avoids `unsafe` and overlapped I/O, and the cut traces each interprocess behaviour to source.

## Open questions (not remarks; each names the fact that would settle it)
1. **Our own macro holding a key at the `on` reset** (PLAUSIBLE, RK9 territory). `reset_from_os` seeds a key that a still-running macro holds through `SendInput` as held+`Passed`. Its injected key-up is passed by the sentinel check before the engine, so the engine keeps a non-modifier key `active` until the user presses and releases it once. That press reads as an auto-repeat and passes, so a binding on that key misses one activation. Modifiers heal through `sync_modifiers`. Reaching it requires a macro that started before `off` and still holds a key at the instant of `swch on`. Settled by deciding whether 2.1 (macro epoch) or 2.3 owns it; it is not this lane's.
2. **Pipe-instance exhaustion is now reachable by any interactive process** (PLAUSIBLE, pre-existing loop shape). IU's GW grants FILE_CREATE_PIPE_INSTANCE, and the listener's instance limit defaults to 255. With every instance taken, interprocess `accept` connects the stored instance and then fails `create_instance` (`listener.rs:63-72`). The `ipc-server` loop (`ipc_server.rs:111-123`) would spin on `[WARN] IPC accept error` until an instance frees. Before the lane, only SYSTEM/BA/the owner could create instances. Settled by whether 4.1 (KS-26) adds an instance limit and backoff on accept errors. It needs a hostile local process, and D6 documents this trade-off.
3. **A non-interactive `swch` of the same user against a debug (non-elevated) daemon.** Examples: a Task Scheduler task set to "run whether logged on or not", or SSH. The default DACL's owner ACE admitted it; the new DACL has no owner, IU or BA entry for that token, so it now gets `AccessDenied` (and `swch open` reports "running"). Release daemons are unaffected, because they were denied before too (KS-02). Settled by whether any supported workflow drives `swch` from a non-interactive logon; the cut rejected `OW` for want of a quotable doc.
