# Code review: p0-A6-appequals @ 32703a8
APPROVED (Critical 0, Important 0, Minor 1)

Branch `fix/app-equals`, `master...fix/app-equals` = `276caf1..32703a8` (5 commits), base `a1b5a27`. Read through
`git show`/`git diff` only. Inputs: `cut.md` (incl. the critique-response addendum `:433-472`), `critique.md`,
`impl.md`, `pack_r1.md`.

## Gate: fmt ✅ · clippy ✅

Run on a `git archive fix/app-equals` export (scratchpad, own target dir; not the work tree, not the tester's tree):
`cargo fmt --check` exit 0; `cargo clippy --all-targets --locked -- -D warnings` exit 0, no warnings (lib, both bins,
tests and `tests/ipc_roundtrip.rs` checked). `git diff master...fix/app-equals --check` clean. No added non-ASCII line
under `src/` or in `Cargo.toml`. `cargo test` not run (tester owns it).

## Remarks

### 🔴 Critical

None.

### 🟡 Important

None.

### 🟢 Minor

#### m1. `IMAGE_PATH_CAPACITY` doc says "four times MAX_PATH"; 1024 is not 4 x 260
**Where**: `src/data/condition.rs:180-182`  **Confidence**: CONFIRMED (arithmetic)
**Problem**: `/// ... four times MAX_PATH, so long-path installs fit without a heap retry.` over
`const IMAGE_PATH_CAPACITY: usize = 1024;`. 4 x `MAX_PATH` is 1040. The real limit, which `fetch_app`'s doc at `:139-141`
states correctly, is a native image path of at most 1023 units (the NUL needs the last slot).
**Failure**: none at run time; a reader sizing roadmap 3.5's cache from this comment gets a wrong bound.
**What to do**: say "1024 units, about four times MAX_PATH" or drop the multiple. Optional.

## Checklist walked (items absent with no reachable failure)

Plan conformance
- Spec "replace `GetModuleBaseNameW` with `QueryFullProcessImageNameW`, take the file name": `process_image_name`
  `:185-219`, `image_file_name` `:223-227`; `GetModuleBaseNameW`/`ProcessStatus` gone from `src` and `Cargo.toml`.
  D1 deviation (`PROCESS_NAME_NATIVE`) is justified with measurements in `impl.md` and the commit body; the last path
  component is identical in both forms, and the native form also works for volumes without a drive letter.
- RAII: `Owned<HANDLE>` (`:193-194`). Verified in the crate: `OpenProcess` returns `Err` for an invalid handle
  (`windows-0.62.2 .../Threading/mod.rs:1192-1196`), `Owned::drop -> Free::free -> CloseHandle` only when
  `!is_invalid()` (`windows-core-0.62.2/src/handles.rs`, `Foundation/mod.rs:5297-5307`). Closed on every path,
  including both `?` (`:213`, `:217`); never closed twice.
- Abstraction kept: `env.rs`, `testing.rs`, `src/hook/*` untouched; T4 runs through `Engine::on_key` on `FakeEnv`.
- Non-ignored current-pid test with the real function: T1 `process_image_name_resolves_the_current_process`
  (`:347-360`). Roadmap A acceptance (`roadmap.md:97`): T2 (`:362-388`). Wiring through the type the hook uses: T7
  (`:440-456`) kills a `fetch_app -> None` mutant.
- Folding (KS-28): `equal_ignoring_case` `:86-92`; the ASCII fast path is exactly `to_lowercase` equality for ASCII
  input; one non-ASCII side takes the slow path (Kelvin row). `TitleEquals(s) => TitleContains(s)` holds for every title
  because both use the same mapping (Greek final-sigma row). Roadmap 3.5 folding line (`roadmap.md:385`): T4 scenario A.
- CHANGELOG: KS-03 and KS-28 under `## [Unreleased]` / `### Fixed`, IDs cited, manual-check note present
  (`CHANGELOG.md:130-148`, `:230-233`). The KS-35 sentence (critique I1) traced: `Binding::new_auto_blocker` keeps
  `Condition::Always` (`binding.rs:98-109`) and `expand_with_auto_blockers` does not copy `when:` (`loader.rs:187-219`),
  so the text is accurate. README and template updated in the commits that change meaning.
- Book pages deferred to the phase book sync (cut Q1, `docs/WORKFLOW.md:242-243`), as in earlier Phase-0 lanes.

Unsafe (4 blocks added, 2 removed)
- `OpenProcess`/`Owned::new` `:191-194`: SAFETY concrete; the `Owned::new` contract (owned, safe to free) is met by a
  fresh `Ok` handle.
- `QueryFullProcessImageNameW` `:203-214`: `PWSTR(buffer.as_mut_ptr())` on a live stack array; `size` in UTF-16 units
  (not bytes), in/out; the result slice clamped with `.min(IMAGE_PATH_CAPACITY)` (`:215`), so no panic path.
- Test `CreateWindowExW` `:403-423` and `DestroyWindow` `:430-433`: title buffer NUL-terminated and alive for the call;
  `HWND` has no `Send` impl in windows 0.62.2 (grep of `windows`/`windows-core` sources), so the drop runs on the creating
  thread, as the SAFETY line says. Message-only (`HWND_MESSAGE`): never shown, activated or broadcast to; no hook, no
  input, no foreground change on the owner's machine.

Hot path (R1, KS-08)
- Lookup runs on the main thread inside `with_engine` (`keyboard_hook_callback.rs:173-181` -> `engine/mod.rs:368-376`),
  at most once per event (`OnceCell`). `OpenProcess`, `QueryFullProcessImageNameW` and `CloseHandle` neither wait on the
  target nor retrieve messages, so they cannot re-enter the LL hook. New per-key-down cost (one `String`, two more for a
  non-ASCII fold, ~2 us query) is what the cut prescribes (D1/D5 cost, RK1), deferred to roadmap 3.4/3.5. Stack buffer
  2 KiB. No `unwrap`/`expect`/indexing panic path in `:139-227`.
- Conditions are evaluated only at `engine/mod.rs:376` (key-down); key-up, repeats (`was_repeat` branch, records-based),
  `on_timer`, `reset` and `on_foreground_changed` never consult them, so the suppression invariant does not depend on the
  now-working `AppEquals`.

R1-R10
- R1: adapter unchanged; every path still ends in one outcome. R2: no `SendInput` added. R3: no static, lock, `Cell` in
  shared state; `ForegroundContext` per event on the stack. R4: no atomics. R5: no builder change. R6: no new variant,
  no `_ =>` touched. R7: no ordering or publish-path change. R8: no key-identity change. R9: process read now uses a right
  that suffices (KS-03), read-only, handle RAII; no registry access touched. R10: no thread, window or pump in non-test
  code; shutdown untouched.

Tests that can fail
- T1/T2/T7 fail on `276caf1` (GetModuleBaseNameW on a limited handle: access denied); T3/T4 fail on `de03573` (ASCII
  fold). A whole-path mutant of `image_file_name` fails T1 (no `\`) and T5. Observation, not a remark: a mutant that drops
  `&& b.is_ascii()` in `equal_ignoring_case` survives because the Kelvin rows put the non-ASCII string only on the actual
  side; the only real input it would break is a config string containing U+212A against an ASCII name.
- T6 is `#[ignore = "manual: ..."]` and read-only (sleep + `GetForegroundWindow`).

Style
- Grouped `windows::{..}` imports; `expect("invariant: ...")` in test helpers; commit subjects `<Area>: <what>`, each
  body ends with the Co-Authored-By line; red commits quote their failing lines.

## Positive

- `Owned<HANDLE>` replaces a hand-written guard and a dead `c_void` placeholder; one fewer `unsafe` close.
- `process_image_name` returns `windows::core::Result`, so T1's panic shows the HRESULT while the hook path still maps
  every failure to `None`.
- The native-format choice is backed by a read-only measurement and cuts the per-lookup cost tenfold on the hook thread.
- T7 closes the gap that let KS-03 ship: the real `ForegroundContext` path is under a non-ignored test without touching
  the foreground.
- The KS-35 interaction is declared where users read it (CHANGELOG, README row, template comment).

## Open questions

- RK3 (non-elevated debug daemon against an elevated foreground) stays unverified, as accepted in the cut; the shipped
  daemon is elevated. M3 run with a debug daemon from a normal console would settle it.
- `.claude/agents/code-reviewer.md` R9 still names `GetModuleBaseNameW`; the orchestrator owns that edit after the merge
  (cut Q2).
