# Results analysis: p1-1.4-hotkey-tristate / roadmap 1.4 (KS-17)

## VERDICT: ACCEPT
**Summary**: The lane meets roadmap 1.4 and the brief. `parse_language_hotkey` now returns a private tri-state (`Combo`/`Disabled`/`Unknown`). A pure `hotkey_from_values` resolves the injected value strings: the first `Combo` or `"3"` decides, and only missing, unreadable or unknown values fall back to Alt+Shift. With `"3"` the loader adds no auto-blocker. Each acceptance line has a hermetic unit test. I replayed the KS-17 regression myself: it is red on the pre-fix tree for the right reason and green at HEAD. The gate is green in my own run, the review is APPROVED with 0 remarks, and the tester round is GREEN. The registry is only read. No hook, engine, IPC or thread code changed. On the owner's machine the `Toggle` key has no values (`reg query` gives an empty key), so the daemon keeps its current Alt+Shift fallback blocker and typing does not change. What remains: three manual checks for the owner, and status lines outside `config/` that the Phase-1 book sync owns.

## Quality gate (own run, HEAD 9fcbc93, fresh `CARGO_TARGET_DIR`)
| Check | Result |
|---|---|
| cargo fmt --check | OK (exit 0) |
| cargo clippy --all-targets --locked -- -D warnings | OK (exit 0; fresh target dir `target-analyst2`; log `analyst-clippy.txt`) |
| cargo test --locked | lib: 479 passed, 0 failed, 3 ignored (482 tests); ipc_roundtrip: 6 passed; swch: 0. Total 485 passed, 3 ignored. Master f28da35: 472 lib tests, 2 ignored, so 476 passed in total. Delta: +10 tests (registry.rs 6 -> 14 `#[test]`, loader.rs 87 -> 89), +1 manual ignore. Log `analyst-test.txt` |
| cargo test -- --list --ignored | 3: `data::condition::tests::foreground_context_names_the_focused_notepad`, `hook::keyboard_hook_callback::tests::the_menu_mask_key_is_inserted_into_the_input_stream`, `system::registry::tests::live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (new). All have `manual:` reasons. Not run |
| cargo doc --no-deps | 4 warnings, all on lines this lane does not touch: loader.rs:156, :176, keyboard_hook.rs:132, vk_name.rs:4 |
| mdbook build | OK (`analyst-book`). The anchors `config/matching.html#s15-2`, `#s15-4`, `roadmap.html#14-tri-state-language-hotkey` and `audit.html#ks-17` exist |
| Red-first replay (own) | `git archive 8ef0ab7` built in a fresh target dir: `a_language_hotkey_not_assigned_in_windows_has_no_combination` FAILED at registry.rs:442 `[Some("3"), Some("3")]` left `Some(KeyCombination { keys: [AnySide(Alt), AnySide(Shift)] })` right `None`; `a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker` FAILED at loader.rs:1060 left `1` right `0` (log `analyst-red.txt`). Both pass at HEAD |

## Acceptance criteria
| # | Criterion (roadmap 1.4 + brief) | Evidence | Status |
|---|---|---|---|
| 1 | `parse_language_hotkey` returns `Combo(keys)`, `Disabled` ("3") or `Unknown` | `system::registry::tests::parse_language_hotkey_is_tri_state` (registry.rs:323): "1"/"2"/"4" give Combo, "3" and " 3 " give Disabled, and "", "0", "5", "33", "unknown", "Alt+Shift" give Unknown. Type at registry.rs:17-28, parser at :226-234 | PASS |
| 2 | (`Hotkey`, `Language Hotkey`) = ("3","3") gives no blocker | `registry::tests::a_language_hotkey_not_assigned_in_windows_has_no_combination` (registry.rs:438, row 1); `config::loader::tests::a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker` (loader.rs:1054): the real template gives 0 blockers and 2 bindings, and `MaskOnly` gives 0 blockers with no warning. Red-first confirmed | PASS |
| 3 | With `Disabled`, no auto-blocker and NO fallback to Alt+Shift | Same registry test, rows `["3",None]`, `[None,"3"]`, `["3","2"]`, `["x","3"]`, `[" 3 ","1"]`, plus a laziness check (the value after "3" is never read, registry.rs:451-458). Code: registry.rs:100-105 returns `None`. The loader already skips `None` (loader.rs:350-352) | PASS |
| 4 | ("2", anything) gives Ctrl+Shift | `registry::tests::the_hotkey_value_decides_before_the_language_hotkey_value` (registry.rs:369) over `ANYTHING` (None, "1".."4", "", "x"); loader `the_registry_values_choose_the_auto_blocker_keys` (loader.rs:1031), row `[Some("2"), None]` on the template | PASS |
| 5 | ("1", anything) gives Alt+Shift | Same registry test (registry.rs:376); loader row `[Some("1"), Some("3")]` | PASS |
| 6 | No values give Alt+Shift | `registry::tests::no_usable_value_falls_back_to_alt_shift` (registry.rs:381): `[None,None]`, unknown texts, and the absent key (empty iterator); loader row `[None, None]` | PASS |
| 7 | Only a missing or unparsable value falls back | registry.rs:381 and `a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide` (registry.rs:396). Tester mutations M4 and M5 are caught | PASS |
| 8 | "Hotkey" (by name) takes precedence over "Language Hotkey" (critique I1) | `the_language_hotkey_is_read_from_hotkey_then_language_hotkey` (registry.rs:348) pins `hkey`, `subkey` and `value_names` for both language functions. Tester mutation M2 (swapped names) is caught | PASS |
| 9 | Registry stays read-only; resolution testable with injected values | `hotkey_from_values` (registry.rs:87-116) is pure over `Option<S: AsRef<str>>`. I/O is limited to `open_key` (`KEY_READ`, :148-172) and `read_string_value` (:177-224). The `+` lines of the diff contain no `RegSet*`, `RegCreate*`, `RegDelete*`, `SystemParametersInfo*` or `SendInput` (own grep) | PASS |
| 10 | CHANGELOG entry | CHANGELOG.md:411-422, under `## [Unreleased]` (:12) / `### Fixed` (:239), in KS order, citing KS-17 and roadmap 1.4 | PASS |
| 11 | Live behaviour with Windows set to Not assigned | Manual: `#[ignore = "manual: ..."]` `live_registry_with_the_language_hotkey_not_assigned_has_no_combination` (registry.rs:481-490), plus the lane checklist M1-M3 (cut.md "Manual checks", impl.md "Gates"). Listed below for the owner | MANUAL |

## Issues claimed closed
| KS-ID | Regression test | Fails on old code? | Status |
|---|---|---|---|
| KS-17 | `config::loader::tests::a_language_hotkey_not_assigned_in_windows_gives_no_auto_blocker` (the audit's scenario: default config plus "Not assigned" gives an Alt+Shift blocker); `system::registry::tests::a_language_hotkey_not_assigned_in_windows_has_no_combination` | Yes. Commit order: `5493b3a` (behaviour-neutral refactor; its `Disabled \| Unknown => {}` arm keeps master's skip, verified by `git show`) -> `8ef0ab7` (tests, red) -> `38029df` (fix). Own replay of `8ef0ab7`: 2 failed by assertion with master's Alt+Shift fallback. On `master` itself the tests do not compile, because the seam arrives in `5493b3a`. That is declared in the cut, and the master code path ("3" -> None -> next value -> default) is the same as `8ef0ab7`'s | CLOSED (audit status line pending the book sync) |

## Invariants
| Area | Status | Notes (file:line) |
|---|---|---|
| Hook callback (passthrough, no blocking/alloc, symmetry) | OK | `src/hook/**` and `src/engine/**` are not in the diff. The registry is read only through `loader.rs:223` (`from_ron_str`), on the main thread at startup or on `config-watcher` on reload. The new `String` per value read (registry.rs:223) is load-time only |
| Engine purity, matching per owner decisions | OK | Engine untouched. The blocker's exact matching, `allow_extra_modifiers`, repeat defaults and RON are unchanged. The language combos stay `AnySide` (`language_hotkeys_are_either_side_modifiers`, registry.rs:508). A reload that drops the blocker mid-press keeps the swallowed key's key-up swallowed through the press record (pre-existing `engine::scenarios::reload_during_a_passing_double_tap_keeps_each_press_whole`, scenarios.rs:3318) |
| Thread ownership, sentinel, atomics, SAFETY comments | OK | No new `unsafe`, atomic, static or lock. The three moved/kept blocks carry concrete SAFETY comments (registry.rs:138-139, :157-158, :187-190; the last is stronger than master's). `HKeyGuard` outlives the lazy iterator (registry.rs:66-75). `HKEY_CURRENT_USER` is never closed. No `SendInput` |
| Config reload, IPC | OK | No schema or IPC change. A failed or empty reload path is untouched. `None` from the hotkey lookup means no blocker and no `MaskOnly` warning (loader.rs:350-352), pinned by loader.rs:1065-1070 |
| No key logging | OK | The new `[INFO]` line names the function only (registry.rs:101-103) |

## Review status
- Critique r1: CHANGES_REQUESTED (0 Critical, 1 Important, 1 Minor). Both items are resolved in the tree, not only answered in prose:
  - I1: `the_language_hotkey_is_read_from_hotkey_then_language_hotkey` (registry.rs:348). Tester mutation M2 confirms it.
  - M1: `book/src/config/matching.md:48`, `book/src/config/example.md:8`.
- Code review r1: APPROVED (0/0/0). No triage was needed.
- Tester r1: GREEN. Gate green, red-first replayed, 7/7 mutations caught (M1-M7 in test_r1.md).
- No CONFIRMED Critical or Important item is open.
- Minor items worth doing while the context is fresh (none blocks):
  - m1: two modified test lines keep `.unwrap()` (loader.rs:896, :969). House style asks for `expect(..)`. Carried over from master.
  - m2: the manual reason at registry.rs:482 names the setup and the restore, but not the check ("run it with --ignored; it passes"). The test name and its asserts make the check clear.

## Docs and hygiene
- Config meaning changed only on machines with `"3"`. Updated in the same branch, commit `38029df`:
  - `README.md:539`;
  - `src/config/default_config.ron:172-174` (comment only, ASCII);
  - `CHANGELOG.md:411-422`.
- Book pages (commit `9fcbc93`):
  - `book/src/config/matching.md` §15.2 (:48, :52-55) and the §15.4 note (:112-113);
  - `known-issues.md:90` (row 40 Fixed);
  - `errors.md:178`;
  - `example.md:8`.
- No new RON block, so no `cfgcheck` run was needed.
- No new dependency. `Cargo.toml` and `Cargo.lock` are untouched, and `--locked` passes.
- 5 commits, all with `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>` (own count: 5). Subjects follow the `<Area>: <what>` form.
- The branch is cut from the current `master` tip `f28da35` (merge-base equals master) and holds only lane 1.4 work, with no merges. The `--no-ff` merge is trivial.
- Hand-off to the Phase-1 book sync. The cut's Q3 and WORKFLOW.md "ks-phase" assign these to the sync, and the 1.2 and 1.6 lanes followed the same pattern. Each text below describes the pre-fix behaviour:
  - `book/src/audit.md:40` and `:453`: status lines.
  - `book/src/roadmap.md:621`: KS-17 is still "Open".
  - `book/src/glossary.md:287`: "treats `"3"` like a missing value".
  - `book/src/overview.md:39`: "That includes `"3"` ... open".
  - `book/src/architecture.md:548`: "KS-17 ... stays open".
  - `book/src/build-and-run.md:119`: lists only two `[INFO]` lines.
  - `book/src/gaps.md:124`.
  - `book/src/config/matching.md:46`: the `1dfd80a` permalinks still show `"3" => None`.

## Technical debt
- No `TODO`, `FIXME`, `#[allow]` or `SeqCst` added (own grep of the `+` lines).
- Accepted limitations and where they are recorded:
  - Q1: which of `Hotkey` and `Language Hotkey` Windows honours when they differ. Under D2, a hand-edited `("3","1")` gives no blocker, which is benign: the behaviour without the flag, with nothing swallowed. Recorded only in cut.md, critique.md and review_r1.md, and settled by step 1 of manual check M1. **Not in the book.** The book sync should add it to `gaps.md`.
  - F1: a daemon elevated as a different admin account reads that account's `HKCU\Keyboard Layout\Toggle`. The problem predates this lane. Recorded only in cut.md (risk 9). **Not in the book.** It is a candidate for `gaps.md`.
  - `Layout Hotkey` and Win+Space are not read. Recorded: `gaps.md:117-124`, roadmap feature #7 (`roadmap.md:560`, `:569`), `matching.md:57`. Side effect worth a sentence at the book sync: with `Language Hotkey`="3" and `Layout Hotkey`="1", Alt+Shift used to be blocked by the old fallback by accident, and it now reaches Windows (critique OQ2).
  - Q2: unknown value text stays silent and falls back. Q4: `REG_DWORD` stays `Unknown` with the existing `[WARN]`. Both are documented in `matching.md:48`, `:54`.

## Problems
None that block the merge. m1 and m2 above are optional Minors, and the book-sync items are hand-offs, not lane defects.

## Manual checks for the owner (human only, not counted as passes)
| # | Setup | Expected |
|---|---|---|
| M1 | Settings > Time & language > Typing > Advanced keyboard settings > Input language hot keys > Switch Input Language: **Not assigned**. Note the old setting first and restore it afterwards. | 1. `reg query "HKCU\Keyboard Layout\Toggle"`: record `Hotkey`, `Language Hotkey` and `Layout Hotkey` (settles Q1). 2. `cargo test --locked -- --ignored live_registry_with_the_language_hotkey_not_assigned_has_no_combination` passes. 3. The release daemon with the default config prints `[INFO] SwitchLanguage: hotkey not assigned in Windows, no auto-blocker`, prints no `[AUTO-BLOCK]` line, and `swch status` shows 2 bindings. 4. In Notepad, Alt, Shift, X loses no key. 5. CapsLock switches the layout. |
| M2 | Switch Input Language: Left Alt+Shift | `[INFO] SwitchLanguage: registry combination Alt+Shift` and one `[AUTO-BLOCK] Alt + Shift` line (regression) |
| M3 | Switch Input Language: Ctrl+Shift | `[INFO] SwitchLanguage: registry combination Ctrl+Shift` and a `Ctrl + Shift` blocker (regression) |

## Positives
- The I/O/decision split is minimal. `hotkey_from_values` takes exactly what `reg query` shows, so each acceptance line is a literal unit test, with no trait, no fake registry and no registry write.
- Commit 1 made the three machine-dependent tests hermetic before the behaviour changed, so a CI or dev machine set to "Not assigned" stays green.
- The red-first is genuine and replayable. The refactor commit is behaviour-neutral, which the `Disabled | Unknown => {}` arm and the tester's mutations show, and the fix flips one match arm.
- Laziness ("values after the deciding one are not read") is pinned for both `Combo` and `Disabled`.
- The loader-level tests run the real template through `to_system_function` -> `hotkey_from_values`, which is the production chain minus the registry read.

## Notes for later lanes
- Replaying an old commit from a `git archive` export with a **shared** `CARGO_TARGET_DIR` gives stale results. Cargo strips the path from a path package's metadata hash, and the archived files carry the commit's mtime, so it reuses the newer binary. My first replay of `8ef0ab7` ran HEAD's tests and passed. Always give an exported tree its own target dir.
- Feature #7 (`Layout Hotkey`) will extend the `value_names` list that `the_language_hotkey_is_read_from_hotkey_then_language_hotkey` pins, and will reuse `HotkeySetting`. Its cut should decide Q1 from the M1 data first.
