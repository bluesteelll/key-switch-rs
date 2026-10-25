# Architecture critique: p0-A6-appequals (KS-03, folding part of KS-28) (round 1)
Verdict: CHANGES_REQUESTED (Critical 0, Important 1, Minor 4)
Plan: D:/tmp/ks-orch/p0-A6-appequals/cut.md   Code read at: master a1b5a27 (clean)

Verified against the tree and the crate sources: `src/data/condition.rs` (whole file), `src/engine/mod.rs:300-420`,
`src/engine/env.rs`, `src/engine/testing.rs:1-260, 440-592`, `src/engine/scenarios.rs:200-280, 1840-1880`,
`src/hook/keyboard_hook_callback.rs:1-240`, `src/data/binding.rs:40-170`, `src/config/loader.rs:170-219`,
`src/core/windows_actions.rs:155-195`, `Cargo.toml`, `README.md:425-479`, `src/config/default_config.ron:130-214`,
`CHANGELOG.md:1-240`; `windows-0.62.2` `System/Threading/mod.rs:1192-1196, 1252-1261, 2797-2798`,
`Foundation/mod.rs:5290-5307`, `extensions/Win32/Foundation/WIN32_ERROR.rs:39-43`; `windows-core-0.62.2`
`handles.rs:16-46`, `windows.rs:12-13`, `lib.rs:13-14`; `windows-result-0.4.1` `error.rs:213-222`; book
`audit.md` KS-03/KS-28/KS-35, `roadmap.md:60-98, 225-236, 370-385`, `methodology.md:165-206, 522-532`,
`config/*.md` hits for AppEquals/KS-35; `docs/WORKFLOW.md` red-first and book-sync rules.

Every factual claim of the re-location table that I checked holds (line numbers, the defect, the `OnceCell` cache, the
lookup running under `with_engine`, `on_timer`/`reset`/`on_foreground_changed` never asking for the foreground, the
`ProcessStatus` feature having no other user, `Owned<HANDLE>` closing through `Free::free` only when `!is_invalid()`,
the `OpenProcess` wrapper never returning an invalid handle in `Ok`, `From<WIN32_ERROR> for Error`).

## Critical (implementation must not start)

None.

## Important (must be resolved; options may be discussed)

### I1. The fix makes KS-35 bite a `Not(AppEquals(..))` binding that requests an auto-blocker, and no user-facing text says so
Where: cut §Non-goals row 3 (line 60), D7 (lines 167-169: "spells out the one visible change"), C3 CHANGELOG entry
(lines 273-290), RK9 (line 355); `src/data/binding.rs:98-109` (`new_auto_blocker` hard-codes `Condition::Always`),
`src/config/loader.rs:187-219` (the blocker is added for any `block_original_combo` binding whose hotkey no `keys`
binding uses, without copying its `when:`), `src/engine/mod.rs:373-393` (the blocker's own `Always` condition is
evaluated, then `blocker_matches` suppresses); `README.md:430`; `src/config/default_config.ron:151-155`.
Problem: the cut records the KS-35 interaction only internally (non-goal row, RK9) and only in the direction that is a
strict improvement (`AppEquals("game.exe")`: before the fix neither CapsLock nor Alt+Shift switched anywhere; after it
CapsLock works in the game). The other direction is a regression that this lane causes, and D7's "one visible change"
and the CHANGELOG text hide it.
Consequence: the default config's remap plus the `when:` the audit and this cut's CHANGELOG entry use as the motivating
example, `( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: Not(AppEquals("game.exe")) )`,
with the Windows hotkey Alt+Shift. On master the condition is always true, so CapsLock switches the layout everywhere,
the game included, and Alt+Shift is blocked everywhere. After C3, inside `game.exe` the CapsLock binding is skipped and
CapsLock reaches the game as CapsLock, while the auto-blocker (condition `Always`) still swallows Alt+Shift. In the game
the user can no longer switch the keyboard layout by any hotkey. The CHANGELOG entry tells them the binding "now stays
silent in that game". README:430 says only that "where its `when:` does not match, the hotkey is not blocked", and that
sentence is about the KS-18 case. A reader can take it to mean that the blocker follows `when:` in general.
Confidence: CONFIRMED (traced through loader -> `new_auto_blocker` -> engine blocker branch; the master behaviour
follows from KS-03 itself).
Breaks: roadmap.md:62 ("when the meaning of the config changes, the same pull request updates README.md, the template
and CHANGELOG"); the brief's "undeclared config-compatibility break"; KS-35 (owned by roadmap 1.2, fix not required here).
What is needed: a declaration, not a code fix. The KS-03 CHANGELOG entry, and the README text where the `when:` /
`block_original_combo` interaction is read, must state that the auto-blocker of a binding with `when:` still acts in
every window until roadmap item 1.2 (KS-35), with the `Not(AppEquals(..))` consequence spelled out (no layout hotkey
inside that app). Correct D7's "one visible change". A manual check for it in the results is optional. Options: (a)
documentation only, which is in scope and recommended; (b) pull KS-35 forward, which is scope creep into 1.2 and an
orchestrator decision.

## Minor (improvements, not blockers)

### M1. `PROCESS_NAME_WIN32` on the hot path is chosen on a weak ground
Where: cut D1 "Rejected" (lines 88-89), RK4 (line 350), C3 T5 table (line 260 already has a `\Device\...` row).
Problem: the lane only needs the last path component, which is the same in both formats. The cut's only reason to
prefer flag 0 is that the audit probe used it. T1 would verify `PROCESS_NAME_NATIVE` on this machine just as well. The
Win32 format adds a device-to-drive-letter translation, and until roadmap 3.5 that cost is paid on every key-down under
the engine lock whenever one binding has an `AppEquals`.
Consequence: PLAUSIBLE only. Kernel volume-to-DOS-name lookups (`IoVolumeDeviceToDosName` / `RtlVolumeDeviceToDosName`)
have reported stalls while volumes arrive (OSR threads). If `ProcessImageFileNameWin32` goes through such a lookup,
plugging in a drive while typing under an `AppEquals` binding would hold up every key on the system. I could not
establish that it does; Microsoft does not document the cost of either format.
Confidence: PLAUSIBLE.
Breaks: R1 / KS-08 (hot-path budget), principle 1 in CLAUDE.md.
What is needed: either justify flag 0 against the hot-path cost, or take the format that needs no translation. T5
already covers the parsing of both forms.

### M2. The `fetch_app` wiring has no non-ignored test; a mutant `fetch_app -> None` survives
Where: cut D2 "Rejected" (lines 105-107), T2 (line 248, a local `OwnProcess` `ForegroundInfo`), T6 ignored
(lines 265-269); `src/data/condition.rs:116-118, 126-154`.
Problem: T1 and T2 call `process_image_name` directly. `ForegroundContext::app()` -> `fetch_app` -> `process_image_name`
is exercised only by the ignored T6 and the manual M1. That matches the spec, which asks only for the pid-level test,
but it is the same shape of gap that let KS-03 ship ("no test evaluates `AppEquals` against a real process" through the
type the hook actually uses).
Consequence: a later edit (roadmap 3.5 rewrites exactly this function to cache per HWND) that returns `None` from
`fetch_app`, or drops the `process_image_name` call, passes the whole gate and silently kills every `AppEquals` again.
The tester's mutation step will report it as a survivor.
Confidence: CONFIRMED (by inspection of what T1/T2/T5 reach).
What is needed: either state in the cut that this mutant survives by design (so tester and triage do not spend a round
on it), or give the real `ForegroundContext` path a test seam that needs no foreground window.

### M3. The manual-check expectations depend on the keyboard layout
Where: C3 CHANGELOG note (lines 286-289: "F9 types `a`"), M1 (line 338: "`aaa` in Notepad"), M2 (line 339: "`b`");
`src/core/windows_actions.rs:162-192` (`press_vk` sends `wVk` with `wScan: 0`, no `KEYEVENTF_UNICODE`).
Problem: `PressKey("A")` synthesizes the virtual key, so the character comes from the foreground thread's active layout.
Consequence: with a Russian layout active in Notepad, a correct build types `ф` (and `и` for M2). The owner may read the
check as failed, or the published CHANGELOG note gives a non-English user a wrong expectation.
Confidence: CONFIRMED.
What is needed: phrase the expectation layout-independently ("the character of the A key"), or use an action whose
effect does not depend on the layout, as M3 does with `ToggleCapsLock`.

### M4. One acceptance proof fails on a correct implementation
Where: Acceptance mapping, line 324: `rg "ProcessHandleGuard|CloseHandle" src/data/condition.rs finds nothing`.
Problem: D3's rationale naturally ends up in the new SAFETY or doc comment ("`Owned` closes it with `CloseHandle`").
Consequence: the tester runs the proof as written, gets a hit on a comment and reports a false failure: one wasted
triage item.
Confidence: CONFIRMED (the proof as written matches comments).
What is needed: scope the proof to code (for example a call form), or drop `CloseHandle` from it.

## Strengths to keep

- Every book reference is re-located against `a1b5a27`, and every one I checked matches the tree.
- The red-first predictions rest on observed evidence: the audit probe got `ERROR_ACCESS_DENIED` on its own process
  with a LIMITED handle. T2's target is ASCII upper case on purpose, so on C1 it fails because of KS-03 and not
  because of the fold. T3/T4 fail at a named step with the harness's exact message format (`testing.rs:549-555, 585`).
- C1 is a refactor that keeps master's behaviour. It makes T1/T2 compilable on a parent, and the cut says that the
  replay runs on C1, not on `master`.
- `Owned<HANDLE>` instead of a hand-written guard: the claims about `Free`, `is_invalid` and the `OpenProcess` wrapper
  hold in 0.62.2. The clamp in D4 leaves no panic path.
- D5 is exact. The ASCII fast path is equivalent to `to_lowercase` equality. The Kelvin sign forces the slow path when
  only one side is ASCII. The final-sigma case rules out the char-wise alternative and keeps
  `TitleEquals(s) => TitleContains(s)`.
- The engine, `Env`, `FakeEnv`, the adapter and the lock discipline are untouched. The thread table is accurate: none of
  `OpenProcess`, `QueryFullProcessImageNameW` and `CloseHandle` retrieves messages, so they cannot re-enter the LL hook
  under `with_engine`.
- Feature hygiene (D6) is correct: features are not recorded in `Cargo.lock`, and `QueryFullProcessImageNameW` is
  ungated under `Win32_System_Threading`.

## Open questions for the architect

1. RK3 (a non-elevated debug daemon against an elevated foreground) is accepted because the shipped daemon is elevated.
   I agree. If the owner wants it settled, M3 run with a debug daemon from a normal console decides it.
2. M1: if flag 0 stays, say whether one timing of T1's call (Win32 against native, both on the own process) was taken.
   That is cheap, read-only, and turns RK4 from "undocumented" into a number.

## Absent topics (walked, not applicable)

- §2 thread affinity, §4 suppression lifetime, §5 timers/gestures, §8 atomics, §10 `SendInput`/sentinel: the lane adds
  no hook, window, timer, thread, atomic, lock or injected event, and reads no event field. RK10 correctly notes that
  conditions are read only at key-down.
- §3 matching model: no key-identity, specificity or modifier change. The owner decisions (exact modifiers, repeat
  policy, RON, logging, licence) are untouched.
- §7 config/reload: no schema change, and `Condition`'s data and `Debug` text are unchanged.
- §9 process boundary: read-only process query with the limited right, no child process, no pipe change.
- Book pages deferred to the phase book sync (Q1): consistent with `docs/WORKFLOW.md:242-243` and the earlier Phase-0
  lanes. The published config pages keep saying "AppEquals never matches" until that sync, which is known drift, not a
  keyboard hazard.
