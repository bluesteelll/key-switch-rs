# Binding kinds and key suppression

<a id="s7"></a>
## 7. The `keys` kind: combination

```ron
( keys: "Ctrl+Alt+L", action: PressKey("L") ),
( keys: "CapsLock",   action: SwitchLanguage ),
```

The string is parsed by the rules of [§6.2](keys.md#s6-2). Everything else in this section is derived from the hook code.

- A combination matches when **all** of its keys are held. `KeyCombination::matches` checks that each of its keys is in the set of held keys ([src/data/key_combination.rs:49-51](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/key_combination.rs#L49-L51)). Extra held keys do not prevent a match: `keys: "A"` also matches on Ctrl+A. When several bindings match, the check order ([§16](matching.md#s16)) decides which one runs: with the bindings `"Ctrl+A"` and `"A"`, pressing Ctrl+A runs only `"Ctrl+A"`.
- The press order does not matter. If you hold A and then press Ctrl, `"Ctrl+A"` matches on the Ctrl key-down, and Ctrl is the key that gets suppressed.
- Matching is checked on every key-down ([src/hook/keyboard_hook_callback.rs:216-292](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L216-L292)); releasing keys does not fire a binding (except `Tap`, [§11](triggers-conditions.md#s11)).
- Only the key pressed last is suppressed. The OS has already received the modifiers pressed before it ([§10](#s10)).

> [!WARNING]
> Known issue ([KS-04](../audit.md#ks-04)): matching is checked on **every** key-down while the combination's keys are held, and the pressed key does not have to belong to the combination ([src/hook/keyboard_hook_callback.rs:225](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L225)). If you hold a binding's key (for example CapsLock) and press any other key, the binding fires again and the pressed key is lost: with `Immediate` when `block_default: true` (the default), with `Tap`/`Hold`/`DoubleTap` always ([src/hook/keyboard_hook_callback.rs:238-252](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L238-L252)). When you type with overlapping key presses (rollover), the action repeats and the unrelated key is lost.

> [!WARNING]
> Known issue ([KS-10](../audit.md#ks-10)): an `Immediate` action repeats on **every** auto-repeat of the held key ([src/hook/keyboard_hook_callback.rs:229](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L229); details in [§13.1](actions.md#s13-1)).

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
| `max_gap_ms` | `u64`, ms | `500` ([src/config/schema.rs:88-90](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L88-L90)) | not validated; `0` and any large value are accepted |

<a id="s8-1"></a>
### 8.1 Loader checks

[src/config/loader.rs:274-289](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L274-L289):

- fewer than two steps, including `steps: []`: ``Sequence must have at least 2 steps; for a single combo use plain `keys: "..."` ``;
- an error in a step: `Sequence step #<i>: ...`, with steps numbered from zero. For example `Sequence step #1: unknown key "Foo" in combo "Foo"`, `Sequence step #1: combo is empty` (also for a step made only of spaces) or `Sequence step #0: empty segment in combo "Ctrl+" (stray '+'?)`;
- the `steps` field is missing: ``Unexpected missing field named `steps` in `SequenceSpec` ``;
- `sequence` is written as a list instead of a struct (`sequence: ["F13", "F14"]`): ``Expected opening `(` for struct `SequenceSpec` ``;
- a string instead of a list (`steps: "F13 F14"`) or a tuple: ``Expected opening `[` ``; unquoted names (`steps: [F13, F14]`): `Expected string`;
- a `trigger` other than `Immediate`: an error ([§11.2](triggers-conditions.md#s11-2));
- repeated keys within a step collapse (`"Ctrl+Ctrl+F14"` equals `Ctrl+F14`). Identical consecutive steps are allowed: `["F13", "F13", "F14"]`.

<a id="s8-2"></a>
### 8.2 How it works

[src/hook/sequence_state.rs:55-120](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L55-L120), called from [src/hook/keyboard_hook_callback.rs:255-272](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L255-L272):

1. Only key-downs that are not auto-repeats count. Key releases do not change the state.
2. A step counts if, at the moment of **any** key-down, the held keys include all keys of the step. This is a subset test (`KeyCombination::matches`). The held keys are normalised: `LShift/RShift → Shift`, `LCtrl/RCtrl → Ctrl`, `LAlt/RAlt → Alt`, `RWin → Win` ([§6.7](keys.md#s6-7)). The pressed key does not have to belong to the step (see the defects in [§8.4](#s8-4)).
3. If nothing is pending yet and the key-down satisfies step 0, an entry "waiting for step 1" is created and the key-down is suppressed.
4. If an entry is pending and the key-down satisfies the next step, the sequence advances and the key-down is suppressed. The next step must arrive **strictly less** than `max_gap_ms` after the previous advancing key-down: the deadline is extended at every step ([src/hook/sequence_state.rs:78](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L78)). There is no overall limit for the whole sequence.
5. On the last step the action runs synchronously on the hook thread. The key-down is suppressed, and the key-ups of the last step's keys are also marked for suppression.
6. Any other key-down resets the pending entry ([src/hook/sequence_state.rs:95](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L95)): an unrelated key, a modifier, or any key-down at which the set of held keys does not cover the next step. Exceptions: auto-repeats and key-downs on which `when` is false (item 10). After the reset, the same key-down is **immediately checked** as step 0 ([src/hook/sequence_state.rs:99](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L99)). The comment at [src/hook/sequence_state.rs:91-94](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L91-L94) claims the opposite; it is out of date.
7. Timeouts are handled lazily: an expired pending entry is removed on the next key-down ([src/hook/sequence_state.rs:71](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L71)). Nothing is executed and nothing is replayed on a timeout.
8. If a key-down is suppressed, its key-up is suppressed too, through the `blocked_keys` bitmap ([src/hook/keyboard_hook_callback.rs:294-297](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L294-L297), [src/hook/keyboard_hook_callback.rs:324](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L324)).
9. `block_default` has no effect on `sequence`: advancing key-downs are always suppressed.
10. If `when` is false at the moment of the key-down, the binding is skipped for that key-down. The pending entry neither advances nor resets, and this binding does not suppress the key.
11. The loader accepts `max_gap_ms: 0` ([KS-27](../audit.md#ks-27)). Such a sequence never completes, but step 0 key-downs are still suppressed.

<a id="s8-3"></a>
### 8.3 Overlaps

- **Common prefix.** `["F13","F14"]` and `["F13","F14","F15"]` keep separate pending entries. `F13 F14` runs the first action. If you then press `F15` within `max_gap_ms`, the second action runs as well. There is no "longest match" selection; both fire.
- **Overlap with `keys`/`chord`.** The kinds are processed independently. For example, `keys: "F13"` (Immediate) runs, and the same key-down starts a `sequence` whose step 0 is `"F13"`.

> [!WARNING]
> Known issue (derived from the code at [src/hook/sequence_state.rs:74-99](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L74-L99), not confirmed separately): bindings with an **identical** step list share one pending entry. For steps such as `["F13","F14"]`, the action of the binding that comes later in the sorted list runs, because that binding overwrites the pending entry. The doc comment at [src/hook/sequence_state.rs:7-9](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L7-L9) ("first binding's action wins") does not match this. For steps such as `["F13","F13"]`, the second binding immediately completes the first binding's pending entry, and the first binding's action runs after a **single** key-down. Do not duplicate sequences with overlapping `when` conditions: the conditions must be mutually exclusive.

<a id="s8-4"></a>
### 8.4 Defects

> [!WARNING]
> Known issue ([KS-15](../audit.md#ks-15)): overlapping key presses (rollover) complete the sequence ([src/hook/sequence_state.rs:76](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L76)). A step is checked against the set of held keys, not against the pressed key, so any key-down while a step's keys are held counts as that step. Example: `["F13","F13"]`. Press F13 and, without releasing it, press `A`: the action runs and `A` is lost. With `["F13","F14"]` and F13 held, every unrelated key-down resets the pending entry, is immediately counted as step 0 again, and is suppressed.

> [!WARNING]
> Known issue ([KS-16](../audit.md#ks-16)): pressing a modifier at a later step aborts the sequence ([src/hook/sequence_state.rs:95](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L95)). Example: `["F13","Ctrl+S"]`. After F13, `Ctrl+S` is expected. Pressing Ctrl gives the set `{Ctrl}`, which is not `Ctrl+S`, so the pending entry is reset; `Ctrl+S` then goes to the application. Workaround: the modifier must be held from step 0 onwards and never released. `["Ctrl+X","Ctrl+S"]` works only if Ctrl is held the whole time. Another option is not to add modifiers after the first step.

> [!WARNING]
> Known issue ([KS-07](../audit.md#ks-07)): `LShift`, `RShift`, `LCtrl`, `RCtrl`, `LAlt`, `RAlt`, `RWin` and their synonyms (`LeftShift`, `RightShift`, `LeftCtrl`, `LeftControl`, `RightCtrl`, `RightControl`, `LeftAlt`, `RightAlt`, `RightWin`) load in steps without an error but never match ([src/hook/keyboard_hook_callback.rs:335-341](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L335-L341), [§6.7](keys.md#s6-7)). Use `Shift`, `Ctrl`, `Alt` and `Win`; `Win` (like its synonyms `LWin`, `LeftWin` and `Super`) matches both Win keys.

> [!WARNING]
> Known issue ([KS-06](../audit.md#ks-06)): auto-repeats of a suppressed key reach the application while its key-up stays suppressed, and the key gets stuck ([src/hook/keyboard_hook_callback.rs:256](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L256) together with [src/hook/keyboard_hook_callback.rs:324](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L324)). Do not hold step keys down.

> [!NOTE]
> Low-severity remark ([KS-32](../audit.md#ks-32), [src/hook/sequence_state.rs:82](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L82)): on completion, the key-ups of all keys of the last step are marked for suppression, even if their key-downs already reached the application. With `["F14+F15","F14+F16"]` the F14 key can get stuck in the application.

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
| `keys` | list of names of **single** keys (case and whitespace do not matter) | required | at least 2; no duplicates |
| `window_ms` | `u64`, ms | `50` ([src/config/schema.rs:92-94](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/schema.rs#L92-L94)) | not validated; `0` and any large value are accepted |

<a id="s9-1"></a>
### 9.1 Loader checks

[src/config/loader.rs:292-309](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/loader.rs#L292-L309), with keys numbered from zero:

- fewer than two keys, including `keys: []`: `Chord must have at least 2 keys; a single key is not a chord`;
- an unknown name: `Chord key #<i>: unknown key name "Foo"`. Each list element must be a single key, not a combination: `"Ctrl+J"` gives the same error. An empty string gives `unknown key name ""`;
- a duplicate: `Chord key #<i>: duplicate "f13"`. Key codes are compared, so synonyms also count as duplicates: `["Ctrl","Control"]`, `["F13","f13"]`, `["Win","LWin"]`. `["Shift","LShift"]` and `["Win","RWin"]` are not duplicates, because their codes differ;
- the `keys` field is missing: ``Unexpected missing field named `keys` in `ChordSpec` ``;
- a `trigger` other than `Immediate`: an error ([§11.2](triggers-conditions.md#s11-2)).

<a id="s9-2"></a>
### 9.2 How it works

[src/hook/chord_state.rs:61-119](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L61-L119), called from [src/hook/keyboard_hook_callback.rs:273-290](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L273-L290):

1. The raw code of the pressed key (`vk_code`) is taken on a key-down that is not an auto-repeat. The code is not normalised ([§6.7](keys.md#s6-7)), and other held keys are not considered.
2. Every key-down of a key in the chord is **suppressed**, whether it starts, continues or completes an attempt ([src/hook/chord_state.rs:106](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L106), [src/hook/chord_state.rs:118](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L118)). The key-up of such a key is suppressed too.
3. The window is counted from the first key-down of the attempt and is not extended ([src/hook/chord_state.rs:111-116](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L111-L116)). All distinct keys of the chord must be pressed before `window_ms` has elapsed since the first key-down. They do not have to be held; only key-downs count. Pressing a key that has already been counted again does not change the attempt, but that key-down is suppressed too.
4. Once all keys are collected, the action runs synchronously on the hook thread, and the key-ups of all chord keys are marked for suppression.
5. An expired attempt is removed lazily ([src/hook/chord_state.rs:83](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L83)). A late key starts a new attempt. The key-downs suppressed before it are lost.
6. The press order does not matter. Unrelated keys do not interfere: Ctrl+F13+F14 also fires, and Ctrl goes through to the application. An unrelated key-down between the chord's keys does not reset the attempt ([src/hook/chord_state.rs:73-75](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L73-L75)).
7. `block_default` has no effect on a chord. `trigger` can only be `Immediate`.
8. Chords with the same key set (the order in the list does not matter) share one attempt, and the first binding's action runs. Derived from the code ([src/hook/chord_state.rs:86-118](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L86-L118)): at the moment the chord fires, the second binding immediately opens a new attempt with the key just pressed. If you press another key of the chord within `window_ms`, the second binding's action runs. A chord and a superset chord, for example `[F13,F14]` and `[F13,F14,F15]`, are tracked separately, and both can fire.
9. The loader accepts `window_ms: 0` ([KS-27](../audit.md#ks-27)). Such a chord never fires, but its keys are always suppressed.

<a id="s9-3"></a>
### 9.3 Defects

> [!WARNING]
> Known issue ([KS-07](../audit.md#ks-07)): `Shift`, `Ctrl` and `Alt` (and `Control`, `Menu`) never match in a chord: the chord never fires, and its other keys keep disappearing ([src/hook/chord_state.rs:73](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L73), [src/hook/keyboard_hook_callback.rs:276](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L276); [§6.7](keys.md#s6-7)).

> [!WARNING]
> A modifier in a chord disappears everywhere. With `["LCtrl","F13"]` every press of the left Ctrl is suppressed, and all shortcuts with the left Ctrl stop working. This is a direct consequence of item 2 in [§9.2](#s9-2), not an implementation bug, but it is an easy trap to fall into.

> [!WARNING]
> Known issue ([KS-06](../audit.md#ks-06)): auto-repeats of a suppressed chord key reach the application while its key-up is suppressed, and the key gets stuck ([src/hook/keyboard_hook_callback.rs:274](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L274) together with [src/hook/keyboard_hook_callback.rs:324](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L324)).

> [!NOTE]
> Low-severity remark ([KS-30](../audit.md#ks-30), [src/hook/keyboard_hook_callback.rs:287](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L287)): when the chord fires, the key-ups of all chord keys are marked for suppression, including keys that were already released. Such a bit stays set until the next key-up of that key or until the foreground window changes. If the next key-down of that key reaches the application (for example because `when` became false without a window change), its key-up is suppressed.

---

<a id="s10"></a>
## 10. Key suppression and `block_default`

To "suppress" means not to pass the key event on to the OS and the applications. If a key-down is suppressed, the matching key-up is suppressed too (the `blocked_keys` bitmap, [src/hook/keyboard_hook_callback.rs:294-297](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L294-L297), [src/hook/keyboard_hook_callback.rs:324](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L324)).

| Mechanism | What is suppressed | `block_default` |
|---|---|---|
| `keys` + `Immediate` | the key-down that completed the combination, if at least one matching binding has `block_default: true` | honoured (default `true`) |
| `keys` + `Tap`/`Hold`/`DoubleTap` | the key-down and key-up of the trigger key, **always** | ignored ([src/hook/keyboard_hook_callback.rs:238-252](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L238-L252)) |
| `sequence` | the key-downs that start or advance the sequence, and their key-ups | ignored |
| `chord` | every key-down and key-up of any chord key, while `when` is true | ignored |

- Only the key that caused the match is suppressed. The OS has already received the modifiers pressed earlier: for `keys: "Ctrl+W", action: DoNothing` the application sees Ctrl go down and up.
- With `block_default: false` the original key reaches the application **in addition** to the action.

> [!WARNING]
> Known issue ([KS-13](../audit.md#ks-13)): `block_default: false` on the binding that fired does not help if another `Immediate` binding with `block_default: true` also matched (for example a more general one further down the list): the key is suppressed anyway ([src/hook/keyboard_hook_callback.rs:234](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L234)).

> [!WARNING]
> Known issue ([KS-06](../audit.md#ks-06)): auto-repeats of a key suppressed by `Tap`/`Hold`/`DoubleTap`, `sequence` or `chord` reach the OS while its key-up stays suppressed, so the key "sticks" in the application ([src/hook/keyboard_hook_callback.rs:239](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/keyboard_hook_callback.rs#L239)).

**There is no replay buffer.** Suppressed key-downs are not stored anywhere and are never replayed ([src/data/binding.rs:17-19](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/data/binding.rs#L17-L19), [src/hook/chord_state.rs:12-17](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/chord_state.rs#L12-L17), [src/hook/sequence_state.rs:11-15](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/hook/sequence_state.rs#L11-L15)):

| Mechanism | When input is lost |
|---|---|
| `chord` | always, when the chord is not completed. A chord key pressed on its own **never types** |
| `sequence` | when the sequence did not complete: the timeout expired or another key was pressed |
| `Tap` / `Hold` / `DoubleTap` | the trigger key never reaches the application, whatever the outcome of the gesture (except the auto-repeat leak, [KS-06](../audit.md#ks-06)) |

Practical rules:

- Assign `chord`, `sequence` and deferred triggers to F13–F24 or to rarely used keys. The template at [src/config/default_config.ron:17-21](https://github.com/bluesteelll/key-switch-rs/blob/7cb3521/src/config/default_config.ron#L17-L21) suggests `["g","g"]` and `["j","k"]`; with them, every g, j and k you type disappears.
- Limit the damage with a `when: TitleContains("...")` condition. In other windows the binding is skipped and the keys pass through as usual.
- Do not hold suppressed keys down: auto-repeat makes them stick.
