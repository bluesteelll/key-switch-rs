# Binding kinds and key suppression

A binding has exactly one kind: `keys` (a combination held together), `sequence` (steps pressed in order) or `chord` (keys pressed almost at once). This page says when each kind fires and which key events it hides from applications. Facts describe commit `20af28d`, the tip of `master` after roadmap Phase 1; the run-time behaviour is derived from the code of the decision core ([src/engine](https://github.com/bluesteelll/key-switch-rs/tree/20af28d/src/engine)) and its scenario tests, not observed on a live daemon.

<a id="s7"></a>
## 7. The `keys` kind: combination

```ron
( keys: "Ctrl+Alt+L", action: PressKey("L") ),
( keys: "CapsLock",   action: SwitchLanguage ),
( keys: "CapsLock",   action: ToggleCapsLock, allow_extra_modifiers: true, when: TitleContains("Notepad") ),
```

The string is parsed by the rules of [§6.2](keys.md#s6-2). A `keys` binding fires, or arms its `Tap`/`Hold`/`DoubleTap` ([§11](triggers-conditions.md#s11)), on a key-down when all three of these hold ([src/engine/matching.rs:53-79](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/matching.rs#L53-L79)):

1. **The key just pressed is one of its trigger keys.** For a combination with a non-modifier key, the trigger keys are its non-modifier keys: `CapsLock` in `"Shift+CapsLock"`, `L` in `"Ctrl+Alt+L"`. For a combination made only of modifiers (`"Alt+Shift"`, `"LShift"`), every key of it is a trigger key ([src/engine/matching.rs:53-63](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/matching.rs#L53-L63)).
2. **All of its keys are held,** each on its side ([§6.7](keys.md#s6-7)). Other non-modifier keys held at the same time do not prevent a match.
3. **The modifiers held are exactly its own.** The engine looks at the four families Shift, Ctrl, Alt and Win, each held if either of its keys is. They must be the families the combination names ([src/engine/matching.rs:76-78](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/matching.rs#L76-L78)). `allow_extra_modifiers: true` drops this condition for one binding: it then fires with any further modifiers held.

What follows from these rules:

- **Press the modifiers first.** Shift, then CapsLock fires `"Shift+CapsLock"`. CapsLock, then Shift does not, and neither does J, then Ctrl for `"Ctrl+J"`.
- **A key typed while a binding's key is held is not that binding's key.** With CapsLock held, typing `abc` delivers `abc` and does not fire `"CapsLock"` again.
- **Extra modifiers block the match.** `"CapsLock"` does not fire while Shift, Ctrl, Alt or Win is held: CapsLock then reaches Windows. `"Ctrl+J"` does not fire on Ctrl+Shift+J, so that shortcut reaches the application. `"A"` does not fire on Ctrl+A.
- With the bindings `"Ctrl+A"` and `"A"`, pressing Ctrl+A matches only `"Ctrl+A"`, and pressing A alone matches only `"A"`.
- Matching is checked on every key-down, auto-repeats included ([src/engine/mod.rs:422-615](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L422-L615)); an `Immediate` binding fires on an auto-repeat only with `repeat` ([§11.3](triggers-conditions.md#s11-3)). Releasing keys does not fire a binding (except `Tap`, [§11](triggers-conditions.md#s11)).
- Only the trigger key's event is suppressed. The OS has already received the modifiers pressed before it ([§10](#s10)).
- `allow_extra_modifiers` is a load error on `sequence` and `chord` bindings: ``binding #0 [Chord[F13+F14]]: `allow_extra_modifiers` applies only to `keys` bindings``.

> [!NOTE]
> Until `1dfd80a` a combination matched any superset of the keys held, with no trigger-key rule ([KS-04](../audit.md#ks-04)): holding CapsLock and typing re-fired the binding and swallowed the typed keys. Exact modifier matching is an owner decision that came with the same change. A config written for the old behaviour may need `allow_extra_modifiers: true` on bindings that should fire with other modifiers held.

> [!NOTE]
> Until `feat/repeat-flag` an `Immediate` action repeated on **every** auto-repeat of the held key while the binding matched ([KS-10](../audit.md#ks-10)). Now only `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)` repeat by default, and every other action fires once per press ([§11.3](triggers-conditions.md#s11-3), [§13.1](actions.md#s13-1)). `repeat: true` on a binding restores the old behaviour.

---

<a id="s8"></a>
## 8. The `sequence` kind: steps pressed in order

```ron
( sequence: (steps: ["F13", "F14"]), action: Media(PlayPause) ),
( sequence: (steps: ["Ctrl+X", "Ctrl+S"], max_gap_ms: 1000), action: PressKey("F5") ),
```

A sequence fires when its steps are pressed one after another, leader-key style as in Vim or Emacs: first step 0, then step 1, and so on.

| Field | Type | Default | Constraints |
|---|---|---|---|
| `steps` | list of combination strings | required | at least 2 steps; each step is parsed the same way as `keys` ([§6.2](keys.md#s6-2)) |
| `max_gap_ms` | `u64`, ms | `500` ([src/config/schema.rs:122-128](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/schema.rs#L122-L128)) | 1 to 10000 |

<a id="s8-1"></a>
### 8.1 Loader checks

[src/config/loader.rs:545-562](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L545-L562):

- fewer than two steps, including `steps: []`: ``Sequence must have at least 2 steps; for a single combo use plain `keys: "..."` ``;
- an error in a step: `Sequence step #<i>: ...`, with steps numbered from zero. For example `Sequence step #1: unknown key "Foo" in combo "Foo"`, `Sequence step #1: combo is empty` (also for a step made only of spaces) or `Sequence step #0: empty segment in combo "Ctrl+" (stray '+'?)`;
- `max_gap_ms` outside 1 to 10000: `max_gap_ms: 0 is out of range: timings are 1 to 10000 ms` ([KS-27](../audit.md#ks-27), fixed in `1dfd80a`);
- the `steps` field is missing: ``Unexpected missing field named `steps` in `SequenceSpec` ``;
- an unknown field such as `max_gap: 300`: ``Unexpected field named `max_gap` in `SequenceSpec`, expected either `steps` or `max_gap_ms` instead``;
- `sequence` is written as a list instead of a struct (`sequence: ["F13", "F14"]`): ``Expected opening `(` for struct `SequenceSpec` ``;
- a string instead of a list (`steps: "F13 F14"`) or a tuple: ``Expected opening `[` ``; unquoted names (`steps: [F13, F14]`): `Expected string`;
- a `trigger` other than `Immediate`, `allow_extra_modifiers: true` or `repeat: true`: an error ([§11.2](triggers-conditions.md#s11-2), [§7](#s7), [§11.3](triggers-conditions.md#s11-3));
- repeated keys within a step collapse (`"Ctrl+Ctrl+F14"` equals `Ctrl+F14`). Identical consecutive steps are allowed: `["F13", "F13", "F14"]`.

<a id="s8-2"></a>
### 8.2 How it works

[src/engine/sequences.rs:53-120](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L53-L120), called from [src/engine/mod.rs:531-557](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L531-L557):

1. Only key-downs that are not auto-repeats count. Key releases do not change the state.
2. A step counts if, at the moment of **any** key-down, all keys of the step are held, each on its side (`"Shift"` either Shift key, `"LShift"` the left one, [§6.7](keys.md#s6-7)). This is a subset test: the pressed key does not have to belong to the step, and extra modifiers do not matter (see the defects in [§8.4](#s8-4)).
3. If nothing is pending yet and the key-down satisfies step 0, an entry "waiting for step 1" is created and the key-down is suppressed.
4. If an entry is pending and the key-down satisfies the next step, the sequence advances and the key-down is suppressed. The next step must arrive **strictly less** than `max_gap_ms` after the previous advancing key-down: the deadline is extended at every step ([src/engine/sequences.rs:79-82](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L79-L82)). There is no overall limit for the whole sequence.
5. On the last step the key-down is suppressed and the action fires on the hook thread: a fast action runs there, and `Launch`, `OpenUrl` and `Sequence` are queued for the action executor ([§13.1](actions.md#s13-1)). The key-ups of the last step's non-modifier keys are suppressed too, but only for keys still held whose key-down did not reach Windows ([src/engine/sequences.rs:122-142](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L122-L142), [src/engine/keys.rs:169-180](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L169-L180)).
6. Any other key-down resets the pending entry ([src/engine/sequences.rs:93-95](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L93-L95)): an unrelated key, a modifier, or any key-down at which the held keys do not cover the next step. Exceptions: auto-repeats and key-downs on which `when` is false (item 10). After the reset, the same key-down is **immediately checked** as step 0 ([src/engine/sequences.rs:98-116](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L98-L116)).
7. An expired pending entry is removed by its own timer, at the deadline, and in any case on the next key-down, which also checks the deadline ([src/engine/sequences.rs:72-74](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L72-L74); the timer since roadmap item 2.2). Nothing is executed and nothing is replayed on a timeout.
8. If a key-down is suppressed, its auto-repeats and its key-up are suppressed too ([§10](#s10)).
9. `block_default` has no effect on `sequence`: advancing key-downs are always suppressed.
10. If `when` is false at the moment of the key-down, the binding is skipped for that key-down. The pending entry neither advances nor resets, and this binding does not suppress the key.

<a id="s8-3"></a>
### 8.3 Overlaps

- **Common prefix.** `["F13","F14"]` and `["F13","F14","F15"]` keep separate pending entries. `F13 F14` runs the first action. If you then press `F15` within `max_gap_ms`, the second action runs as well. There is no "longest match" selection; both fire ([new feature 8](../roadmap.md#new-features-after-stabilisation)).
- **Overlap with `keys`/`chord`.** The kinds are processed independently. For example, `keys: "F13"` (Immediate) runs, and the same key-down starts a `sequence` whose step 0 is `"F13"`.

> [!WARNING]
> Known issue (derived from the code at [src/engine/sequences.rs:76-116](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L76-L116), not confirmed separately): bindings with an **identical** step list share one pending entry. For steps such as `["F13","F14"]`, the binding later in check order finds the entry the first one started, resets it on the same key-down and starts it again with its own action, so its action is the one that runs. For steps such as `["F13","F13"]`, the second binding immediately completes the first binding's pending entry, and the first binding's action runs after a **single** key-down. Do not duplicate sequences with overlapping `when` conditions: the conditions must be mutually exclusive. Since roadmap item 1.6 the loader warns about two `sequence` bindings with the same steps and the same `when` as written ([§18.6](errors.md#s18-6)); with different conditions it does not warn.

<a id="s8-4"></a>
### 8.4 Defects

> [!WARNING]
> Known issue ([KS-15](../audit.md#ks-15), open until roadmap [item 3.2](../roadmap.md#32-sequence-and-chord-matching)): overlapping key presses (rollover) complete the sequence ([src/engine/sequences.rs:79](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L79)). A step is checked against the keys held, not against the pressed key, so any key-down while a step's keys are held counts as that step. Example: `["F13","F13"]`. Press F13 and, without releasing it, press `A`: the action runs and `A` is lost. With `["F13","F14"]` and F13 held, every unrelated key-down resets the pending entry, is immediately counted as step 0 again, and is suppressed.

> [!WARNING]
> Known issue ([KS-16](../audit.md#ks-16), open until roadmap [item 3.2](../roadmap.md#32-sequence-and-chord-matching)): pressing a modifier at a later step aborts the sequence ([src/engine/sequences.rs:93-95](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L93-L95)). Example: `["F13","Ctrl+S"]`. After F13, `Ctrl+S` is expected. Pressing Ctrl gives the held keys `{Ctrl}`, which do not cover `Ctrl+S`, so the pending entry is reset; `Ctrl+S` then goes to the application. Workaround: the modifier must be held from step 0 onwards and never released. `["Ctrl+X","Ctrl+S"]` works only if Ctrl is held the whole time. Another option is not to add modifiers after the first step.

> [!NOTE]
> Fixed in Phase 0: sided names such as `LShift` and `RWin` now match in steps ([KS-07](../audit.md#ks-07)); auto-repeats of a suppressed step key stay suppressed, so the key no longer sticks ([KS-06](../audit.md#ks-06)); and completion no longer swallows the key-up of a key whose key-down reached the application, for example F14 in `["F14+F15","F14+F16"]` or `Win` in `"Win+F14"` ([KS-32](../audit.md#ks-32)).

---

<a id="s9"></a>
## 9. The `chord` kind: keys pressed together

```ron
( chord: (keys: ["F13", "F14"]), action: Media(Next) ),
( chord: (keys: ["F15", "F16", "F17"], window_ms: 60), action: Media(VolumeMute) ),
```

A chord fires when all of its keys are pressed almost simultaneously, in any order.

| Field | Type | Default | Constraints |
|---|---|---|---|
| `keys` | list of names of **single** keys (case and whitespace do not matter) | required | at least 2; no two keys may overlap |
| `window_ms` | `u64`, ms | `50` ([src/config/schema.rs:132-138](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/schema.rs#L132-L138)) | 1 to 10000 |

<a id="s9-1"></a>
### 9.1 Loader checks

[src/config/loader.rs:564-587](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/config/loader.rs#L564-L587), with keys numbered from zero:

- fewer than two keys, including `keys: []`: `Chord must have at least 2 keys; a single key is not a chord`;
- an unknown name: `Chord key #<i>: unknown key name "Foo"`. Each list element must be a single key, not a combination: `"Ctrl+J"` gives the same error. An empty string gives `unknown key name ""`;
- two keys that one press could satisfy: `Chord key #<i>: duplicate "<name>" (overlaps key #<j>)`. That covers repeats and synonyms (`["F13","f13"]`, `["Ctrl","Control"]`) and also an unsided key with a sided one of the same family (`["Shift","LShift"]`, `["Win","RWin"]`, `["Win","LWin"]`) ([src/data/key_spec.rs:150-160](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/key_spec.rs#L150-L160)). `["LShift","RShift"]` is allowed;
- `window_ms` outside 1 to 10000: `window_ms: 20000 is out of range: timings are 1 to 10000 ms`;
- the `keys` field is missing: ``Unexpected missing field named `keys` in `ChordSpec` ``;
- a `trigger` other than `Immediate`, `allow_extra_modifiers: true` or `repeat: true`: an error ([§11.2](triggers-conditions.md#s11-2), [§7](#s7), [§11.3](triggers-conditions.md#s11-3)).

<a id="s9-2"></a>
### 9.2 How it works

[src/engine/chords.rs:58-116](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L58-L116), called from [src/engine/mod.rs:558-576](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L558-L576):

1. The code of the pressed key is taken on a key-down that is not an auto-repeat. It counts for a chord key that accepts it: `"Shift"` accepts either Shift key, `"LShift"` only the left one ([§6.7](keys.md#s6-7)). Other held keys are not considered.
2. Every key-down of a key in the chord is **suppressed**, whether it starts, continues or completes an attempt. Its auto-repeats and its key-up are suppressed too ([§10](#s10)).
3. The window is counted from the first key-down of the attempt and is not extended ([src/engine/chords.rs:106-114](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L106-L114)). All keys of the chord must be pressed before `window_ms` has elapsed since the first key-down. They do not have to be held; only key-downs count. Pressing a key that has already been counted again does not change the attempt, but that key-down is suppressed too.
4. Once all keys are collected, the action fires on the hook thread: a fast action runs there, and `Launch`, `OpenUrl` and `Sequence` are queued for the action executor ([§13.1](actions.md#s13-1)). The key-ups of the chord keys that are still held, and whose key-down did not reach Windows, are suppressed ([src/engine/keys.rs:169-180](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L169-L180)).
5. An expired attempt is removed by its own timer, at the end of the window, and in any case on the next key-down, which also checks the window ([src/engine/chords.rs:76-78](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L76-L78); the timer since roadmap item 2.2). A late key starts a new attempt. The key-downs suppressed before it are lost.
6. The press order does not matter. Unrelated keys do not interfere: Ctrl+F13+F14 also fires, and Ctrl goes through to the application. An unrelated key-down between the chord's keys does not reset the attempt ([src/engine/chords.rs:72-74](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L72-L74)).
7. `block_default` has no effect on a chord. `trigger` can only be `Immediate`.
8. Chords with the same key set (the order in the list does not matter) share one attempt, and the first binding's action runs. Derived from the code ([src/engine/chords.rs:80-115](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L80-L115)): at the moment the chord fires, the second binding immediately opens a new attempt with the key just pressed. If you press another key of the chord within `window_ms`, the second binding's action runs. A chord and a superset chord, for example `[F13,F14]` and `[F13,F14,F15]`, are tracked separately, and both can fire.

<a id="s9-3"></a>
### 9.3 Defects and traps

> [!WARNING]
> A modifier in a chord disappears everywhere. With `["LCtrl","F13"]` every press of the left Ctrl is suppressed, and all shortcuts with the left Ctrl stop working; with `["Shift","Space"]` both Shift keys are. This is a direct consequence of item 2 in [§9.2](#s9-2), not an implementation bug, but it is an easy trap to fall into.

> [!NOTE]
> Fixed in Phase 0: `Shift`, `Ctrl` and `Alt` now complete a chord, and `Win` means either Win key ([KS-07](../audit.md#ks-07)); a held chord key no longer leaks auto-repeats and sticks ([KS-06](../audit.md#ks-06)); and a completed chord no longer marks keys already released, or a key whose key-down reached the application, for a swallowed key-up ([KS-30](../audit.md#ks-30)).

---

<a id="s10"></a>
## 10. Key suppression and `block_default`

To "suppress" means not to pass the key event on to the OS and the applications. The engine records, for every key, whether its latest key-down was suppressed or passed, and treats the rest of the press the same way ([src/engine/keys.rs:56-71](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/keys.rs#L56-L71), rule 5 of [src/engine/mod.rs:162-182](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L162-L182)):

- If a key-down is suppressed, its auto-repeats and its key-up are suppressed too. A binding that stops matching while the key is held does not let the repeats through.
- If a key-down reaches Windows, its auto-repeats and its key-up reach Windows too, even when a binding starts matching while the key is held or a `chord` or `sequence` completes.
- One exception keeps a key from sticking: the key-up of a key whose key-down was suppressed passes when Windows has that key down. The hook reads the key's state in Windows (`GetAsyncKeyState`), which inside the hook is still the state before the key-up. Down means Windows saw a key-down the hook did not, for example the key was released and pressed again in a higher-integrity window while the daemon was not elevated, or on the secure desktop; swallowing that release would leave the key down in Windows (roadmap [item 2.3](../roadmap.md#23-control-window-and-state-reset), [§19](known-issues.md#s19) row 78).

| Mechanism | What is suppressed | `block_default` |
|---|---|---|
| `keys` + `Immediate` | the trigger key's key-down, if the winning binding has `block_default: true` ([§16.2](matching.md#s16-2)) | honoured (default `true`) |
| `keys` + `Tap`/`Hold`/`DoubleTap` | the trigger key's key-down, if the winning binding, or one of its group, has `block_default: true`; the gesture is armed either way | honoured since `feat/deferred-block-default` (roadmap [item 1.3](../roadmap.md#13-block_default-for-tap-hold-and-doubletap)): default `false` for a combination made only of modifiers (`"LShift"`, `"Ctrl+Shift"`, `"Win"`), `true` otherwise. It used to be ignored: the key-down was **always** suppressed ([src/engine/mod.rs:413-452](https://github.com/bluesteelll/key-switch-rs/blob/1dfd80a/src/engine/mod.rs#L413-L452) at `1dfd80a`) |
| `sequence` | the key-downs that start or advance the sequence | ignored |
| `chord` | every key-down of any chord key, while `when` is true | ignored |

In every row the auto-repeats and the key-up follow the key-down.

- Only the key that caused the match is suppressed. The OS has already received the modifiers pressed earlier: for `keys: "Ctrl+W", action: DoNothing` the application sees Ctrl go down and up.
- With `block_default: false` the original key reaches the application **in addition** to the action. A `Tap`, `Hold` or `DoubleTap` binding still detects its gesture: the press, its auto-repeats and its release reach the application, and the action fires when the gesture completes. Another key pressed meanwhile cancels the gesture, so that a shortcut is not also the gesture (roadmap item 2.2; [§11.8](triggers-conditions.md#s11-8)).
- A foreground change keeps every key's record: a held key whose key-down was suppressed keeps its auto-repeats and its release suppressed, so Windows never sees a key-up without a key-down (rule L, roadmap [item 2.3](../roadmap.md#23-control-window-and-state-reset)). Until then the engine forgot which key-ups were due to be suppressed, and such a release reached Windows alone ([§19](known-issues.md#s19) row 55).
- `swch off` and `swch on` drop every pending gesture and take the keys Windows holds as already pressed; a key whose key-down was suppressed keeps its release suppressed ([Controlling the daemon with swch](../build-and-run.md#controlling-the-daemon-with-swch)). While remapping is off every event passes unread, so a key released then reaches Windows as a lone key-up (row 59).

> [!NOTE]
> Fixed on `feat/one-winner` ([KS-13](../audit.md#ks-13), roadmap [item 1.6](../roadmap.md#16-one-winner-per-event)): only the binding that wins the press, or its `Tap`/`Hold`/`DoubleTap` group, decides ([src/engine/mod.rs:436-578](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/mod.rs#L436-L578)). `block_default: false` on the winner lets the key through even when a losing binding has `block_default: true`, for example a binding with `allow_extra_modifiers: true` and fewer keys. Inside a group the key is suppressed if any member asks: with `( keys: "F13", action: SwitchLanguage, trigger: Tap(200), block_default: false )` and `( keys: "F13", action: ToggleCapsLock, trigger: Hold(200) )`, F13 is suppressed on every press. A `sequence`, a `chord` or an auto-blocker that takes the key can suppress it too.

> [!NOTE]
> Before `1dfd80a` the auto-repeats of a key suppressed by `Tap`/`Hold`/`DoubleTap`, `sequence` or `chord` reached the OS while its key-up stayed suppressed, so the key stuck ([KS-06](../audit.md#ks-06)). The modifier resync also erased modifiers whose key-down was suppressed ([KS-14](../audit.md#ks-14)). Both are fixed.

**There is no replay buffer.** Suppressed key-downs are not stored anywhere and are never replayed ([src/data/binding.rs:9-20](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/data/binding.rs#L9-L20), [src/engine/chords.rs:10-16](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/chords.rs#L10-L16), [src/engine/sequences.rs:12-17](https://github.com/bluesteelll/key-switch-rs/blob/20af28d/src/engine/sequences.rs#L12-L17); [new feature 5](../roadmap.md#new-features-after-stabilisation)):

| Mechanism | When input is lost |
|---|---|
| `chord` | always, when the chord is not completed. A chord key pressed on its own **never types** |
| `sequence` | when the sequence did not complete: the timeout expired or another key was pressed |
| `Tap` / `Hold` / `DoubleTap` | with `block_default: true`, the default unless the combination is made only of modifiers, the trigger key never reaches the application, whatever the outcome of the gesture. With `block_default: false` nothing is lost |

Practical rules:

- Assign `chord`, `sequence` and deferred triggers that suppress their key to F13–F24 or to rarely used keys. A binding on letters, such as a chord on `["j","k"]`, makes every typed j and k disappear.
- A `Tap`, `Hold` or `DoubleTap` on a key you also type with needs `block_default: false`, which a combination made only of modifiers has by default. Another key pressed while its gesture is pending cancels it ([§11.8](triggers-conditions.md#s11-8)), so Ctrl+C does not also fire a Ctrl `Tap`.
- Limit the damage with a `when: TitleContains("...")` or `when: AppEquals("...")` condition. In other windows the binding is skipped and the keys pass through as usual.
