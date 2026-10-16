# Known configuration pitfalls

This page gathers in one place every known problem that the configuration reference flags with a warning, so that you can check a config against it. It is an index, not the issue list. The authoritative list of defects, with severity, cause, verification and suggested fix, is the [Audit results](../audit.md) chapter, and the **Audit** column below links each row to its KS entry there. IDs from KS-27 upward are low-severity reports, most of them from a single reviewer, that were not independently verified. "none" means the audit has no entry for the row; where [Gaps and documentation drift](../gaps.md) covers it, the column links there.

<a id="s19"></a>
## 19. Known issues (as of `1dfd80a`)

The rows keep the numbers they had in the audit version of this page, which described commit `7cb3521`; rows 54 and later are new. The **Status** column says what applies at `1dfd80a`:

- **Open**: the problem still applies; **Where** links the code at `1dfd80a`.
- **Fixed**: the problem is gone; **Where** links the merge commit that fixed it.
- **By design**: the behaviour is intended, but easy to trip over.

"From code" means that the behaviour was inferred from reading the code and was not separately reproduced on a running daemon.

<a id="s19-1"></a>
### 19.1 Loading and hot reload

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 1 | A typo in `bindings`, or a missing `bindings`, gave 0 bindings with no error; on reload, all remapping was silently switched off | **Fixed**: now a load error that names the field; `bindings: []` still loads with a `[WARN]` | [`1dfd80a`](https://github.com/bluesteelll/key-switch-rs/commit/1dfd80a) | [§2.3](structure.md#s2-3), [§4](structure.md#s4) | [KS-22](../audit.md#ks-22) |
| 2 | Unknown fields were silently ignored at every level: a typo in an optional field left its default value in effect | **Fixed**: an unknown field is a load error | [`1dfd80a`](https://github.com/bluesteelll/key-switch-rs/commit/1dfd80a) | [§5.3](structure.md#s5-3) | [KS-22](../audit.md#ks-22) |
| 3 | Deleting or renaming the file while the daemon ran wrote the default template and applied it in place of your bindings | **Fixed**: the live bindings stay, and nothing is written | [`1dfd80a`](https://github.com/bluesteelll/key-switch-rs/commit/1dfd80a) | [§2.3](structure.md#s2-3) | [KS-23](../audit.md#ks-23) |
| 4 | A relative config path, or a path whose letter case differed from the name on disk: reload never fired | **Fixed** | [`1dfd80a`](https://github.com/bluesteelll/key-switch-rs/commit/1dfd80a) | [§2.3](structure.md#s2-3) | [KS-21](../audit.md#ks-21) |
| 5 | A transient read error during reload (file locked, `os error 32`): there is no retry, and the changes apply only at the next save | Open | [src/hook/config_watcher.rs:156-170](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/config_watcher.rs#L156-L170) | [§2.3](structure.md#s2-3) | none |
| 6 | UTF-8 with a BOM produced the unhelpful error ``1:1: Expected opening `(` for struct `RawConfig` `` | **Fixed** | [`1dfd80a`](https://github.com/bluesteelll/key-switch-rs/commit/1dfd80a) | [§3.1](ron-syntax.md#s3-1) | [KS-24](../audit.md#ks-24) |

<a id="s19-2"></a>
### 19.2 Key matching and suppression

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 7 | A combo matched on any key-down while its keys were held: with overlapping typing (rollover) the action repeated and the unrelated key was lost | **Fixed**: a combo fires only on one of its own keys | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f) | [§7](kinds.md#s7) | [KS-04](../audit.md#ks-04) |
| 8 | `Immediate` actions repeat on every auto-repeat (the layout cycles round, dozens of `Launch` windows, macros running in parallel) | **Fixed**: an `Immediate` binding fires on auto-repeat only with `repeat`; see rows 62 to 64 | `feat/repeat-flag` | [§7](kinds.md#s7), [§11.3](triggers-conditions.md#s11-3), [§13.1](actions.md#s13-1) | [KS-10](../audit.md#ks-10) |
| 9 | The auto-repeat of a key suppressed by `Tap`/`Hold`/`DoubleTap`, `sequence` or `chord` reached the OS, while its key-up stayed suppressed: the key stuck | **Fixed**; see row 55 for what remains after a foreground change | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f), [`44f79ac`](https://github.com/bluesteelll/key-switch-rs/commit/44f79ac) | [§10](kinds.md#s10) | [KS-06](../audit.md#ks-06) |
| 10 | `block_default: false` does not help if another `Immediate` binding with `block_default: true` also matched | Open | [src/engine/mod.rs:409-411](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L409-L411) | [§10](kinds.md#s10), [§13.9](actions.md#s13-9) | [KS-13](../audit.md#ks-13) |
| 11 | A more specific binding does not override less specific deferred ones (and a more specific deferred binding does not override a less specific `Immediate` one). With exact modifiers this needs bindings with the same keys or `allow_extra_modifiers: true` | Open | [src/engine/mod.rs:403-453](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L403-L453) | [§16.3](matching.md#s16-3) | [KS-13](../audit.md#ks-13) |
| 12 | The names `LShift`, `RShift`, `LCtrl`, `RCtrl`, `LAlt`, `RAlt`, `RWin` and the `Left…`/`Right…` synonyms loaded in `keys` and in `sequence` steps but never matched | **Fixed**: they match their own side | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f) | [§6.7](keys.md#s6-7) | [KS-07](../audit.md#ks-07) |
| 13 | `Shift`, `Ctrl`, `Alt` (and `Control`, `Menu`) in a `chord` never matched, and the other keys of such a chord were lost | **Fixed**: they match either side | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f) | [§6.7](keys.md#s6-7) | [KS-07](../audit.md#ks-07) |
| 14 | A modifier in a chord is suppressed on every press and breaks every shortcut that uses it | By design | [src/engine/chords.rs:10-16](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/chords.rs#L10-L16) | [§6.7](keys.md#s6-7), [§9.3](kinds.md#s9-3) | none; context in [KS-07](../audit.md#ks-07) |
| 15 | With an AltGr layout (Russian, German, Polish), right Alt arrives together with a fake left Ctrl, which the hook cannot tell from a real one. Right Alt+X fires `Ctrl+Alt+X` but not `Alt+X`, `RAlt+X` or `Ctrl+X` (inferred from the layout tables and the code) | Open | [src/hook/keyboard_hook_callback.rs:253-257](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L253-L257) | [§6.6](keys.md#s6-6) | none, see [gaps](../gaps.md#other-functional-gaps) |
| 16 | `Ctrl+Pause` and `Ctrl+Break` load but apparently never fire: Windows sends `VK_CANCEL` (inferred from the layout tables) | Open | [src/hook/keyboard_hook_callback.rs:253-257](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L253-L257) (only `vkCode` is read) | [§6.6](keys.md#s6-6) | none |

<a id="s19-3"></a>
### 19.3 `sequence` and `chord`

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 17 | Overlapping key presses (rollover) complete a sequence: a step is checked against the set of held keys | Open | [src/engine/sequences.rs:79](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/sequences.rs#L79) | [§8.4](kinds.md#s8-4) | [KS-15](../audit.md#ks-15) |
| 18 | Pressing a modifier on the second or a later step aborts the sequence | Open | [src/engine/sequences.rs:93-95](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/sequences.rs#L93-L95) | [§8.4](kinds.md#s8-4) | [KS-16](../audit.md#ks-16) |
| 19 | Sequences with identical steps share one pending entry; with `["F13","F13"]` the action fires on a single press (from code) | Open | [src/engine/sequences.rs:76-116](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/sequences.rs#L76-L116) | [§8.3](kinds.md#s8-3) | none, see [gaps](../gaps.md#other-functional-gaps) |
| 20 | Low severity: when a sequence completed, the key-ups of all keys of the last step were suppressed, even keys whose key-down had already reached the application | **Fixed** | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f), [`44f79ac`](https://github.com/bluesteelll/key-switch-rs/commit/44f79ac) | [§8.4](kinds.md#s8-4) | [KS-32](../audit.md#ks-32) |
| 21 | Low severity: after a chord, the key-up suppression stayed set even for keys that were already released | **Fixed** | [`44f79ac`](https://github.com/bluesteelll/key-switch-rs/commit/44f79ac) | [§9.3](kinds.md#s9-3) | [KS-30](../audit.md#ks-30) |

<a id="s19-4"></a>
### 19.4 `Tap` / `Hold` / `DoubleTap` triggers

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 22 | A gesture armed on any key pressed while the trigger key was held, and that key was lost | **Fixed** | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f) | [§11.4](triggers-conditions.md#s11-4) | [KS-04](../audit.md#ks-04) |
| 23 | Stale timer threads carried no generation number and removed or executed the entry of the next press: lost Taps, premature Holds | **Fixed** | [`16a0461`](https://github.com/bluesteelll/key-switch-rs/commit/16a0461) | [§11.5](triggers-conditions.md#s11-5) | [KS-11](../audit.md#ks-11) |
| 24 | `Tap` and `Hold` on one key: near the boundary the outcome depended on the thread scheduler | **Partly fixed**: with equal terms exactly one fires; with different terms both or neither can fire | [`16a0461`](https://github.com/bluesteelll/key-switch-rs/commit/16a0461) | [§11.6](triggers-conditions.md#s11-6) | [KS-25](../audit.md#ks-25) |
| 25 | Two `DoubleTap` bindings matching one press: the action of the first fires on a single press (from code) | Open | [src/engine/gestures.rs:200-219](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/gestures.rs#L200-L219) | [§11.7](triggers-conditions.md#s11-7) | none |
| 26 | `Tap` and `DoubleTap` actions and `chord`/`sequence` completions run inside the hook callback. (The `DoubleTap` action no longer runs under a mutex.) | Open | [src/hook/keyboard_hook_callback.rs:185-195](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L185-L195) | [§11.8](triggers-conditions.md#s11-8), [§13.1](actions.md#s13-1) | [KS-08](../audit.md#ks-08) |
| 27 | `sync_modifiers` cleared a suppressed modifier on another key's event; the modifier's auto-repeat then looked like a new press | **Fixed** | [`44f79ac`](https://github.com/bluesteelll/key-switch-rs/commit/44f79ac) | [§11.8](triggers-conditions.md#s11-8) | [KS-14](../audit.md#ks-14) |

<a id="s19-5"></a>
### 19.5 Actions

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 28 | `Launch` and `OpenUrl` run synchronously on the hook thread (except under a `Hold` trigger); a long call can exceed `LowLevelHooksTimeout`, and Windows then silently removes the hook | Open | [src/core/windows_actions.rs:137-138](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L137-L138) | [§13.1](actions.md#s13-1), [§13.6](actions.md#s13-6), [§13.7](actions.md#s13-7) | [KS-08](../audit.md#ks-08) |
| 29 | `OpenUrl` is called without initialising COM | Open | [src/core/windows_actions.rs:235-261](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L235-L261) | [§13.7](actions.md#s13-7) | [KS-08](../audit.md#ks-08) |
| 30 | Synthesized keys combine with physically held modifiers (`Ctrl+J → Down` gives Ctrl+Down) | Open | [src/core/windows_actions.rs:163](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L163), [src/data/sequence.rs:182](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/sequence.rs#L182), [:202](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/sequence.rs#L202) | [§6.8](keys.md#s6-8), [§13.1](actions.md#s13-1) | [KS-19](../audit.md#ks-19) |
| 31 | `Win+X`/`Alt+X` with an action that injects no keys (`SwitchLanguage*`, `PostMessage`, `Launch`, `OpenUrl`, `DoNothing`) opens the Start menu or the menu bar on release | Open | [src/engine/mod.rs:255-258](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L255-L258) (the mask is never set) | [§11.8](triggers-conditions.md#s11-8), [§13.1](actions.md#s13-1), [§13.9](actions.md#s13-9) | [KS-12](../audit.md#ks-12) |
| 32 | `Launch` processes and `OpenUrl` handlers inherit the daemon's administrator rights without a UAC prompt | Open | [src/core/windows_actions.rs:225](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L225), [:242](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L242) | [§13.6](actions.md#s13-6), [§13.7](actions.md#s13-7) | [KS-09](../audit.md#ks-09) |
| 33 | Console programs started by `Launch` attach to the daemon's console. (The doc comment that called the child detached was corrected in [`13ce323`](https://github.com/bluesteelll/key-switch-rs/commit/13ce323).) | Open | [src/core/windows_actions.rs:225](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L225) | [§13.6](actions.md#s13-6) | [KS-09](../audit.md#ks-09), [KS-34](../audit.md#ks-34) |
| 34 | `\n`, `\t` and `\r` in `Text(…)` are sent as characters, not as Enter or Tab key presses | Open | [src/data/sequence.rs:150-153](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/sequence.rs#L150-L153) | [§14.4](actions.md#s14-4) | none |
| 35 | `Combo(…)` releases a modifier that the user is physically holding | Open | [src/data/sequence.rs:223-225](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/sequence.rs#L223-L225) | [§14.6](actions.md#s14-6) | [KS-19](../audit.md#ks-19) |
| 36 | Macros have no limit and no cancellation: every trigger starts a new thread, and a `Delay` cannot be interrupted | Open | [src/data/sequence.rs:111-122](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/sequence.rs#L111-L122) | [§14.1](actions.md#s14-1) | none, see [gaps](../gaps.md#no-cancellation-for-macro-workers) |
| 37 | `DoNothing` (like any combo) matched supersets: `Ctrl+W` also swallowed Ctrl+Shift+W | **Fixed**: modifiers match exactly | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f) | [§13.9](actions.md#s13-9) | [KS-04](../audit.md#ks-04) |
| 38 | Not verified: programs that read scan codes may take synthesized key presses, which are sent without `KEYEVENTF_EXTENDEDKEY` and with `wScan = 0`, for other keys (right Ctrl/Alt for left, arrows for numeric keypad keys) | Open | [src/core/windows_actions.rs:163-202](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L163-L202), [src/data/sequence.rs:182-251](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/sequence.rs#L182-L251) | [§6.8](keys.md#s6-8) | [KS-36](../audit.md#ks-36) |

<a id="s19-6"></a>
### 19.6 Auto-blocker (`block_original_combo`)

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 39 | The blocker matched supersets: it swallowed every `Alt+Shift+X` (or `Ctrl+Shift+X`) and AltGr+Shift input | **Partly fixed**: the blocker acts only on the hotkey on its own. Pressed modifiers first (Alt, Shift, X), it still swallows Shift, so the application gets Alt+X | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f), [`b056b72`](https://github.com/bluesteelll/key-switch-rs/commit/b056b72) | [§15.4](matching.md#s15-4) | [KS-05](../audit.md#ks-05) |
| 40 | The registry value `"3"` (hotkey disabled) still produces an `Alt+Shift` blocker | Open | [src/system/registry.rs:185](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/system/registry.rs#L185) | [§15.4](matching.md#s15-4) | [KS-17](../audit.md#ks-17) |
| 41 | An `Immediate` binding on the same combo placed below the flagged binding (or the owning binding itself, when it had that combo) never executed its action | **Fixed**: no blocker is added for a hotkey that a binding uses, and a blocker fires nothing | [`b056b72`](https://github.com/bluesteelll/key-switch-rs/commit/b056b72) | [§15.3](matching.md#s15-3) | [KS-18](../audit.md#ks-18) |
| 42 | A `keys` binding on exactly the hotkey's keys, anywhere in the file, replaces the blocker whatever its `when` and `trigger`: where its `when` is false, the hotkey reaches Windows | By design since [`b056b72`](https://github.com/bluesteelll/key-switch-rs/commit/b056b72) | [src/config/loader.rs:294-307](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/loader.rs#L294-L307) | [§15.3](matching.md#s15-3) | related: [KS-18](../audit.md#ks-18) |
| 43 | The blocker does not inherit its owner's `when` and `trigger`, and acts in every window | Open | [src/data/binding.rs:98-109](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/binding.rs#L98-L109) | [§12.4](triggers-conditions.md#s12-4), [§15.4](matching.md#s15-4) | [KS-35](../audit.md#ks-35) |

<a id="s19-7"></a>
### 19.7 `when` conditions

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 44 | `AppEquals` never matched: `GetModuleBaseNameW` was called on a handle opened with `PROCESS_QUERY_LIMITED_INFORMATION` | **Fixed**: the name comes from `QueryFullProcessImageNameW` | [`4177540`](https://github.com/bluesteelll/key-switch-rs/commit/4177540) | [§12.3](triggers-conditions.md#s12-3) | [KS-03](../audit.md#ks-03) |
| 45 | `TitleEquals` and `AppEquals` ignored case only for ASCII Latin letters (`"ПРИВЕТ"` ≠ `"Привет"`) | **Fixed** | [`4177540`](https://github.com/bluesteelll/key-switch-rs/commit/4177540) | [§12.1](triggers-conditions.md#s12-1) | [KS-28](../audit.md#ks-28) |

<a id="s19-8"></a>
### 19.8 Documentation in code and the README

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 46 | The template said "Reload requires restarting the program.", although hot reload exists | **Fixed** | [`13ce323`](https://github.com/bluesteelll/key-switch-rs/commit/13ce323) | [§2.2](structure.md#s2-2) | none, see [gaps](../gaps.md#user-facing-documentation) |
| 47 | The template's list of macro steps named 5 of the 8 (`Launch`, `OpenUrl` and `Media` were missing) | **Fixed** | [`13ce323`](https://github.com/bluesteelll/key-switch-rs/commit/13ce323) | [§2.2](structure.md#s2-2) | none, see [gaps](../gaps.md#user-facing-documentation) |
| 48 | The template promised "case-insensitive string comparisons" for all conditions | **Fixed**: the promise is now true | [`4177540`](https://github.com/bluesteelll/key-switch-rs/commit/4177540) | [§2.2](structure.md#s2-2) | [KS-28](../audit.md#ks-28) |
| 49 | "resolves whichever wins" and the example with equal `Tap(200)`/`Hold(200)`; `Hold → PressKey("LShift")` does not produce a held Shift | **Fixed**: equal terms are exact, and the template says the Hold sends one Shift press | [`16a0461`](https://github.com/bluesteelll/key-switch-rs/commit/16a0461), [src/config/default_config.ron:112-117](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/default_config.ron#L112-L117) | [§2.2](structure.md#s2-2), [§11.6](triggers-conditions.md#s11-6) | [KS-25](../audit.md#ks-25); see also [gaps](../gaps.md#hold-that-keeps-a-synthesized-modifier-held) |
| 50 | The `AppEquals` examples never fired | **Fixed** | [`4177540`](https://github.com/bluesteelll/key-switch-rs/commit/4177540) | [§2.2](structure.md#s2-2), [§12.3](triggers-conditions.md#s12-3) | [KS-03](../audit.md#ks-03) |
| 51 | The `Ctrl+Pause` example apparently does not fire | Open | [src/config/default_config.ron:214](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/default_config.ron#L214) | [§2.2](structure.md#s2-2), [§6.6](keys.md#s6-6) | none |
| 52 | The `keys: "LShift"` example with `DoubleTap(250)` never fired | **Fixed**, but it now swallows every left-Shift press (row 54); the template and the README say so | [`82f542f`](https://github.com/bluesteelll/key-switch-rs/commit/82f542f), [src/config/default_config.ron:251-258](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/config/default_config.ron#L251-L258) | [§2.2](structure.md#s2-2), [§11.8](triggers-conditions.md#s11-8) | [KS-07](../audit.md#ks-07) |
| 53 | The doc comments of the sequence state machine contradicted the code | **Fixed**: the comments of `src/engine/sequences.rs` describe the code | [`16a0461`](https://github.com/bluesteelll/key-switch-rs/commit/16a0461), [src/engine/sequences.rs:1-17](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/sequences.rs#L1-L17) | [§8.3](kinds.md#s8-3) | none, see [gaps](../gaps.md#code-comments-and-contributor-documentation) |

<a id="s19-9"></a>
### 19.9 New with Phase 0 and later

| # | Problem | Status | Where | Section | Audit |
|---|---|---|---|---|---|
| 54 | A deferred trigger (`Tap`, `Hold`, `DoubleTap`) on a modifier, such as `keys: "LShift"`, swallows every press of that modifier, so it stops working as a modifier | Open until roadmap [item 1.3](../roadmap.md#13-block_default-for-tap-hold-and-doubletap) | [src/engine/mod.rs:413-452](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L413-L452) | [§11.8](triggers-conditions.md#s11-8) | none |
| 55 | After a foreground change, the release of a held key whose key-down was swallowed reaches Windows as a lone key-up, unless a `Tap` or `Hold` of that press is still pending. Its repeats stay swallowed | Open until roadmap [item 2.3](../roadmap.md#23-control-window-and-state-reset) | [src/engine/mod.rs:325-340](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L325-L340) | [§10](kinds.md#s10) | [KS-06](../audit.md#ks-06) status |
| 56 | Modifiers match exactly: a binding such as `"CapsLock"` that used to fire with Shift held no longer does. Add `allow_extra_modifiers: true` for the old behaviour | By design (owner decision) | [src/engine/matching.rs:70-79](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/matching.rs#L70-L79) | [§7](kinds.md#s7) | none |
| 57 | A combo fires only when its modifiers go down first: J, then Ctrl does not fire `"Ctrl+J"`, and CapsLock, then Shift does not fire `"Shift+CapsLock"` | By design | [src/engine/matching.rs:53-63](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/matching.rs#L53-L63) | [§7](kinds.md#s7) | [KS-04](../audit.md#ks-04) |
| 58 | `LWin` in `keys` and `sequence` now means the left Win key only; it used to mean either. Use `Win` for either | By design | [src/data/key_spec.rs:162-170](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/key_spec.rs#L162-L170) | [§6.7](keys.md#s6-7) | [KS-07](../audit.md#ks-07) |
| 59 | A key swallowed by a binding and released while remapping is off (`swch off`) reaches Windows as a lone key-up (from code) | Open | [src/hook/keyboard_hook_callback.rs:245-251](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/hook/keyboard_hook_callback.rs#L245-L251) | [§10](kinds.md#s10) | none |
| 60 | A macro that is running keeps running after `swch off` | Open until roadmap [item 2.1](../roadmap.md#21-action-executor) | [src/data/sequence.rs:111-122](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/data/sequence.rs#L111-L122) | [§14.1](actions.md#s14-1) | none, see [gaps](../gaps.md#no-cancellation-for-macro-workers) |
| 61 | With the `` ` `` language hotkey (registry value `"4"`), the blocker swallows only a lone `` ` ``: a `` ` `` typed while another key is held, or with Shift, reaches Windows and may switch the layout | By design since [`b056b72`](https://github.com/bluesteelll/key-switch-rs/commit/b056b72) | [src/engine/matching.rs:81-101](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/matching.rs#L81-L101) | [§15.2](matching.md#s15-2) | [KS-05](../audit.md#ks-05) |
| 62 | `Immediate` bindings other than `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)` fire once per press: a held `Media(Next)` no longer skips track after track. Add `repeat: true` | By design (owner decision) | `feat/repeat-flag` | [§11.3](triggers-conditions.md#s11-3) | [KS-10](../audit.md#ks-10) |
| 63 | A press that follows a key-up the hook never saw (the key released on the lock screen or the secure desktop) reads as an auto-repeat: a binding that does not repeat fires nothing for it, and the next press works (from code; pinned by a scenario test) | Open. For a key whose key-down was swallowed, which is the usual trigger key, no roadmap item repairs it: the key resync of roadmap [item 2.3](../roadmap.md#23-control-window-and-state-reset) leaves such keys alone, because Windows never saw them go down | [src/engine/keys.rs:141-148](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/keys.rs#L141-L148) | [§11.3](triggers-conditions.md#s11-3) | [KS-29](../audit.md#ks-29) |
| 64 | `PressKey` repeats by default, and each repeat sends a new press and release: a held trigger of `PressKey("CapsLock")` or `PressKey("NumLock")` toggles the key at the repeat rate, and one of a modifier taps it, where the real key held down toggles once or stays down. Write `repeat: false` | By design (owner decision) | [src/core/windows_actions.rs:163-202](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/core/windows_actions.rs#L163-L202), `feat/repeat-flag` | [§13.5](actions.md#s13-5) | [KS-10](../audit.md#ks-10) |

---

> [!NOTE]
> This reference describes commit `1dfd80a`, and the `repeat` field the branch `feat/repeat-flag`. Its examples were checked with the `cfgcheck` validator ([Validation harness](../methodology.md#validation-harness)), which calls the real loader; the hook's real-time behaviour was inferred from the code and the engine's scenario tests.
