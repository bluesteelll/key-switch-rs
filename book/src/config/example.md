# Complete example

<a id="s17"></a>
## 17. Complete `config.ron` example

The example uses every binding kind, every trigger, every action, every macro step, every condition variant and `allow_extra_modifiers`. It contains no bindings that are known not to work: no `Ctrl+Pause`, no modifiers in a chord, and no Win/Alt combinations with actions that inject no keystrokes ([KS-12](../audit.md#ks-12)). The workarounds are described in the comments. The file can be copied as a whole; the keys F13–F24 exist on macro keyboards and are often assigned to mouse buttons through the mouse vendor's own software. Facts describe commit `1dfd80a`.

Checked with the validator (the real loader) on a machine where the `Hotkey`/`Language Hotkey` values did not yield a combination, so the fallback `Alt+Shift` was used:

```text
[INFO] SwitchLanguage: using default combination
OK: 25 bindings (after auto-blocker expansion)
```

That is 24 bindings from the file plus the auto-blocker `Alt+Shift`, which the loader appends after them. On a machine whose registry sets another language hotkey the blocker has other keys, and the count is the same.

```ron
// config.ron — complete example for key-switch-rs (behaviour of commit 1dfd80a).
//
// Save the file as UTF-8, with or without a BOM. After a save, the bindings
// reload on their own after about 250 ms. A file that does not load (for
// example, a misspelled field) is reported in the daemon's console, and the
// previous bindings stay live.
//
// Modifiers match exactly: "CapsLock" does not fire while Shift, Ctrl, Alt or
// Win is held, and "Ctrl+J" does not fire on Ctrl+Shift+J. Press the
// modifiers first, then the key.
//
// Deliberately NOT used:
//   * Ctrl+Pause / Ctrl+Break: Windows sends VK_CANCEL, not Pause;
//   * Win+X / Alt+X with an action that injects no keystrokes (SwitchLanguage*,
//     PostMessage, Launch, OpenUrl, DoNothing): releasing Win/Alt
//     opens the Start menu or the menu bar;
//   * modifiers in a chord: every press of a chord key is swallowed;
//   * three-key shortcuts on Alt and Shift (Alt+Shift+X): the auto-blocker
//     from the CapsLock binding below swallows whichever of the two
//     modifiers is pressed second, so the application gets Alt+X or Shift+X.
(
    bindings: [
        // ───────────────────────── 1. Language and CapsLock ───────────────────

        // CapsLock switches the language forward. block_original_combo: true
        // appends an auto-blocker for the system language-switch hotkey
        // (taken from the registry: Alt+Shift, Ctrl+Shift or `, Alt+Shift by
        // default). The blocker acts only on the hotkey pressed on its own.
        // Known issue: Immediate repeats on auto-repeat: while you hold
        //   CapsLock, the layouts keep cycling.
        (
            keys: "CapsLock",
            action: SwitchLanguage,
            block_original_combo: true,
        ),

        // With Ctrl held, only this binding matches: "CapsLock" above needs
        // no modifier at all.
        ( keys: "Ctrl+CapsLock", action: SwitchLanguageBackward ),

        // Shift+CapsLock is a regular CapsLock (a synthesized VK_CAPITAL
        // press; the daemon's own hook lets it through, so there is no recursion).
        ( keys: "Shift+CapsLock", action: ToggleCapsLock, trigger: Immediate ),

        // ───────────────────────── 2. Single-key remaps ───────────────────────

        // Insert is disabled completely (block_default is true by default).
        ( keys: "Insert", action: DoNothing ),

        // ScrollLock works as Esc. PressKey is one press (down+up);
        // when held, it repeats at the auto-repeat rate.
        ( keys: "Scroll Lock", action: PressKey("Esc") ),   // a space inside the name is allowed

        // Pause: play/pause, also with Shift, Alt or Win held (with Ctrl,
        // Windows sends VK_CANCEL instead of Pause).
        ( keys: "Pause", action: Media(PlayPause), allow_extra_modifiers: true ),

        // Ctrl+Alt+= / Ctrl+Alt+-: volume. Immediate is handy here:
        // while the keys are held, the volume changes continuously. OEM keys
        // are named by their US-layout character ("=" is the =/+ key).
        ( keys: "Ctrl+Alt+=", action: Media(VolumeUp) ),
        ( keys: "Ctrl+Alt+-", action: Media(VolumeDown) ),

        // ───────────────────────── 3. Tap / Hold / DoubleTap ──────────────────

        // F13: a short press (< 200 ms) is play/pause, a 200 ms hold is next
        // track. With equal terms exactly one of the two fires. F13 itself
        // never reaches applications.
        ( keys: "F13", action: Media(PlayPause), trigger: Tap(200) ),
        ( keys: "F13", action: Media(Next),      trigger: Hold(200) ),

        // Double press of F14 within 250 ms: mute. A single press is
        // lost. Do not put a second DoubleTap on the same key.
        ( keys: "F14", action: Media(VolumeMute), trigger: DoubleTap(250) ),

        // Hold F21 for 400 ms: Task Manager. Hold runs the action on the
        // timer thread, so Launch does not block the hook here.
        // Known issue: the launched program gets administrator rights, like the daemon.
        ( keys: "F21", action: Launch(exe: "taskmgr.exe"), trigger: Hold(400) ),

        // ───────────────────────── 4. PostMessage ─────────────────────────────

        // All three F22 bindings are Tap(300): with Immediate, auto-repeat
        // would minimize or close windows one after another. Modifiers match
        // exactly, so each press arms exactly one of them.

        // F22: minimize the active window: WM_SYSCOMMAND + SC_MINIMIZE (0xF020).
        ( keys: "F22", action: PostMessage(msg: "WM_SYSCOMMAND", wparam: 0xF020), trigger: Tap(300) ),

        // Shift+F22: restore the window; msg is given as a number (0x0112 = WM_SYSCOMMAND),
        // SC_RESTORE = 0xF120. lparam has type i64 and accepts negative numbers.
        (
            keys: "Shift+F22",
            action: PostMessage(msg: 0x0112, wparam: 0xF120, lparam: 0),
            trigger: Tap(300),
        ),

        // Ctrl+F22: close the window (like the X button). WM_* names are case-insensitive.
        ( keys: "Ctrl+F22", action: PostMessage(msg: "wm_close"), trigger: Tap(300) ),

        // ───────────────────────── 5. when conditions ─────────────────────────

        // The binding with a condition sits ABOVE the general binding on the same
        // key: with equal key counts, file order decides. In VS Code F24 sends
        // Ctrl+Shift+F5, in all other windows F5. Immediate: when held,
        // F24 repeats like an ordinary key.
        (
            keys: "F24",
            action: Sequence([ Combo("Ctrl+Shift+F5") ]),
            when: AppEquals("Code.exe"),
        ),
        ( keys: "F24", action: PressKey("F5"), when: Always ),

        // F19 in Chrome/Firefox, except private windows: reopen the closed
        // tab. Every comparison ignores case, in every alphabet.
        (
            keys: "F19",
            action: Sequence([ Combo("Ctrl+Shift+T") ]),
            trigger: Tap(300),
            when: And([
                Or([ AppEquals("chrome.exe"), AppEquals("firefox.exe") ]),
                Not(TitleContains("Incognito")),
                Not(TitleContains("Private Browsing")),
            ]),
        ),

        // Esc in Calculator: the ordinary Esc passes through to the application
        // (block_default: false, which works only for keys + Immediate),
        // and then the window is minimized. Calculator belongs to
        // ApplicationFrameHost.exe, so it is matched by its title.
        (
            keys: "Esc",
            action: Sequence([ Delay(100), Window(Minimize) ]),
            block_default: false,
            when: Or([ TitleEquals("Калькулятор"), TitleEquals("Calculator") ]),
        ),

        // ───────────────────────── 6. sequence and chord ──────────────────────

        // Sequence: F17, then F18 no later than 600 ms after it.
        // The F17 press is always suppressed; if F18 does not arrive in time,
        // the F17 press is lost (there is no replay buffer).
        (
            sequence: (steps: ["F17", "F18"], max_gap_ms: 600),
            action: Sequence([
                Text("С уважением,"),
                Key("Enter"),               // a real Enter, not "\n"
                Text("Иван 🚀"),
            ]),
        ),

        // Chord: F15 and F16 almost simultaneously (a window of 80 ms from the
        // first press): maximize the active window. On their own, F15 and F16
        // no longer work: every press of either one is suppressed.
        (
            chord: (keys: ["F15", "F16"], window_ms: 80),
            action: Sequence([ Window(Maximize) ]),
        ),

        // ───────────────────────── 7. Macros ──────────────────────────────────

        // Ctrl+Alt+N: a note. The first Delay leaves time to release Ctrl+Alt:
        // synthesized input combines with physically held
        // modifiers. Tap protects against auto-repeat (every firing
        // starts a separate macro thread, which cannot be cancelled).
        (
            keys: "Ctrl+Alt+N",
            trigger: Tap(400),
            action: Sequence([
                Delay(250),
                Window(Restore),            // the window that was active when the binding fired
                Delay(100),
                Text("Заметка: купить хлеб"),
                Key("Enter"),
                Text("Дата: 2026-10-14"),
                Combo("Ctrl+S"),
                Delay(1_000),
                Window(Minimize),
            ]),
        ),

        // F20: workspace. Launch and OpenUrl inside Sequence run on the
        // macro thread and do not block the hook. A backslash is doubled in an
        // ordinary string and written once in a raw string r"...".
        (
            keys: "F20",
            trigger: Tap(300),
            action: Sequence([
                Launch(exe: "wt.exe", args: ["-d", "D:\\projects"]),
                Delay(1500),
                OpenUrl(r"C:\Windows"),          /* the folder opens in Explorer */
                OpenUrl("https://github.com"),
                Media(PlayPause),
            ]),
        ),

        // Hold F23 for 300 ms: display settings. As with F21, Hold runs the
        // action on the timer thread, so OpenUrl works here without Sequence.
        // With Immediate or Tap it would have to be wrapped: Sequence([OpenUrl(...)]).
        ( keys: "F23", action: OpenUrl("ms-settings:display"), trigger: Hold(300) ),
    ],
)
```

> [!NOTE]
> Some string values in the example are Russian text. They are data, not comments, so they are left untranslated. In the `F17 → F18` sequence, `Text("С уважением,")` types "Best regards," and the second `Text` types the name "Иван" (Ivan) followed by an emoji. In the `Ctrl+Alt+N` macro, `Text("Заметка: купить хлеб")` types "Note: buy bread" and `Text("Дата: 2026-10-14")` types "Date: 2026-10-14". `TitleEquals("Калькулятор")` matches the Calculator window title on a Russian-language Windows.

> [!WARNING]
> The comments in the example mark or work around these known defects, all still open at `1dfd80a`:
> - Win/Alt combinations with an action that injects no keystrokes open the Start menu or the menu bar: [KS-12](../audit.md#ks-12);
> - the `CapsLock` auto-blocker swallows the second modifier of Alt, Shift, X (or Shift, Alt, X), so the application gets Alt+X (or Shift+X): the rest of [KS-05](../audit.md#ks-05);
> - `Immediate` actions repeat on auto-repeat (`CapsLock`, and the reason the `F22` bindings and the `Ctrl+Alt+N` macro use `Tap`): [KS-10](../audit.md#ks-10);
> - programs started by `Launch` run with the daemon's administrator rights (`F21`, and `wt.exe` in the `F20` macro): [KS-09](../audit.md#ks-09);
> - most actions run on the hook thread, which is why `Launch` and `OpenUrl` are placed under `Hold` or inside `Sequence` (`F21`, `F20`, `F23`): [KS-08](../audit.md#ks-08);
> - synthesized input combines with physically held modifiers (the first `Delay` of `Ctrl+Alt+N`): [KS-19](../audit.md#ks-19).
>
> The audit version of this example also worked around sided names, `AppEquals`, unequal Tap and Hold terms, stuck keys, ASCII-only `TitleEquals`, relative config paths and the BOM ([KS-07](../audit.md#ks-07), [KS-03](../audit.md#ks-03), [KS-25](../audit.md#ks-25), [KS-06](../audit.md#ks-06), [KS-28](../audit.md#ks-28), [KS-21](../audit.md#ks-21), [KS-24](../audit.md#ks-24)). Phase 0 fixed those.

Checking order after sorting ([§16](matching.md#s16)): first the three-key bindings (`Ctrl+Alt+=`, `Ctrl+Alt+-`, `Ctrl+Alt+N`), then the two-key ones in load order (`Ctrl+CapsLock`, `Shift+CapsLock`, `Shift+F22`, `Ctrl+F22`, the sequence `F17 → F18`, the chord `F15+F16`, and last the `Alt+Shift` auto-blocker), then the single-key ones. What follows from this:

- with Ctrl held, CapsLock matches only `Ctrl+CapsLock`, and with Shift held only `Shift+CapsLock`: modifiers match exactly, so the order does not matter here;
- on Shift+F22 and Ctrl+F22 only the two-key `Tap` is armed: the `Tap` on bare `F22` does not match while a modifier is held;
- in a VS Code window `F24` runs the binding with `when`, because it comes earlier in the file than the general one.
