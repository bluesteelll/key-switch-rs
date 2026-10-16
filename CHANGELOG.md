# Changelog

All notable changes to key-switch-rs are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
Until 1.0, a breaking change to the `config.ron` syntax or behaviour bumps the
minor version; such entries sit under **Changed** and start with
"**Breaking:**". Issue ids such as KS-38 refer to the
[audit](https://bluesteelll.github.io/key-switch-rs/audit.html).

## [Unreleased]

### Added

- CI on `windows-latest` for every push and pull request
  (`.github/workflows/ci.yml`):
  - `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
    `cargo test --locked` and `cargo build --release --locked`;
  - a check that the release `key-switch-rs.exe`, and not `swch.exe`, embeds
    the `requireAdministrator` manifest;
  - a check of cargo's JSON messages that `cargo test --release --no-run` and
    `cargo bench --no-run` build no libtest harness for the daemon binary
    (KS-38);
  - a release build for `x86_64-pc-windows-gnu` with the GNU toolchain, which
    must link and warn that no UAC manifest is embedded (KS-42);
  - `cargo check` with Rust 1.88, the minimum supported version.
- Documentation book (mdBook): overview, architecture, building and running,
  the `config.ron` reference, the audit (KS-01 to KS-42), gaps and
  documentation drift, and the roadmap. `.github/workflows/book.yml` publishes
  it to [GitHub Pages](https://bluesteelll.github.io/key-switch-rs/) with the
  Node 24 releases of its actions: `actions/checkout@v7`,
  `actions/configure-pages@v6`, `actions/upload-pages-artifact@v5` and
  `actions/deploy-pages@v5`.
- Licence: `MIT OR Apache-2.0`, with `LICENSE-MIT` and `LICENSE-APACHE`.
- Package metadata in `Cargo.toml`: description, licence, repository,
  homepage, readme, keywords and categories, and `rust-version = "1.88"` as
  the minimum supported Rust version.
- This changelog.
- Per-binding `allow_extra_modifiers: true` for `keys` bindings: the binding
  also fires while modifiers it does not name are held. It is a load error
  on `sequence` and `chord` bindings.
- `tests/ipc_roundtrip.rs`: the `swch` client against test pipes with
  unique names: a server that never answers, a missing pipe, a pipe that
  refuses the client, a busy pipe and a server that answers, and the
  daemon's pipe server answering `status`, `off`, `on`, `status` through a
  stand-in for the daemon, with no keyboard hook installed.
- Per-binding `repeat: true | false` for `keys` bindings with the
  `Immediate` trigger (roadmap item 1.1): whether the action fires again on
  every auto-repeat while its key is held. It defaults to `true` for
  `PressKey`, `Media(VolumeUp)` and `Media(VolumeDown)` and to `false` for
  every other action. `repeat: true` on a `Tap`, `Hold`, `DoubleTap`,
  `sequence` or `chord` binding is a load error, because those never fire
  on an auto-repeat. The startup list marks a value that differs from the
  default with `[repeat]` or `[no repeat]`.
- The loader warns about a binding that duplicates one above it: the same
  kind (`keys`, `sequence` or `chord`) and keys (in any order for `keys`
  and `chord`, in order for `sequence` steps), the same trigger whatever its
  term, the same `when:` written the same way and, for `keys`, the same
  `allow_extra_modifiers` (roadmap item 1.6). For example
  `[WARN] binding #2 [capslock] duplicates binding #0 [CapsLock]; remove one of them`.
  The file still loads. The warning is printed at startup and on every
  reload. If the copy you remove sets `block_original_combo: true`, move
  the flag to the one you keep.
- `block_original_combo: MaskOnly`, experimental (roadmap item 1.2, KS-05):
  an auto-blocker that swallows nothing. When Windows would see its hotkey
  and no other key, it sends the menu mask key (0xE8, see KS-12 under
  **Fixed**) on the key-down that completes the hotkey and again on the
  next key event, so that Windows does not see a clean hotkey press.
  `Ctrl+Shift+T`, `Alt+Shift+Tab` and `Alt+Shift+X` then reach applications,
  where `true` swallows the hotkey's second modifier. It rests on an
  unverified hypothesis: that Windows switches the layout only on a clean
  press and release of `Alt+Shift` or `Ctrl+Shift`. `true` and `false` keep
  their meaning, and the template keeps `true`. The startup list shows such
  a blocker as `-> (masked, experimental)`. On a hotkey that is not made only
  of modifiers (the `` ` `` hotkey) `MaskOnly` blocks as `true` does and
  warns: ``[WARN] binding #0: block_original_combo: MaskOnly needs a Windows hotkey made only of modifiers; the ` hotkey is blocked as with true``.
  A misspelled value is a load error that lists the valid ones,
  ``Unexpected variant named `Maskonly` in enum `RawBlockOriginalCombo`, expected one of `true`, `false`, or `MaskOnly` instead``;
  `Some(true)`, an error before too, now gets that message for `` `Some` ``.
  Manual check (no test can tell what Windows does): with
  `( keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly )`
  and the `Ctrl+Shift` hotkey, `Ctrl+Shift+T` reopens a closed browser tab
  and a bare Ctrl+Shift pressed and released five times never switches the
  layout; with the `Alt+Shift` hotkey, a bare Alt+Shift five times never
  switches it, Alt+Shift+Tab steps back in the switcher, and Alt, Shift,
  release in Notepad activates no menu. If the layout still switches, the
  hypothesis does not hold: keep `true`.

### Changed

- The source is formatted with `cargo fmt`.
- The keyboard decisions moved into a pure decision core, `src/engine`
  (roadmap Workstream A, scope additions). The hook callback translates each
  event, asks `Engine::on_key` for a `Decision` and carries it out on the
  same thread as before; Hold actions still run on their own `hold-timer`
  thread. Tap and DoubleTap expire by deadline instead of a cleanup thread
  per press, so only Hold still starts a timer thread. Scenario tests
  (`engine::testing::Scenario`: a RON config, a script of key events and a
  fake clock) cover every binding kind and trigger.
- **Breaking:** key names keep their side in every trigger: `keys`,
  `sequence` steps and `chord` keys (KS-07). `Shift`, `Ctrl`, `Alt` and
  `Win` (`Super`) match either key; `LShift`, `RShift`, `LCtrl`, `RCtrl`,
  `LAlt`, `RAlt`, `LWin` and `RWin` (and the long forms such as `LeftShift`
  and `RightWin`) match only that key. `LWin` used to match both Win keys in
  `keys` and `sequence`, and `Win` meant the left Win key only in `chord`. A
  `chord` whose keys overlap, such as `["Shift", "LShift"]` or
  `["Win", "RWin"]`, is now a load error. Output keys (`PressKey`, `Key`,
  `Combo`) are unchanged: `Win` still sends the left Win key.
- The auto-blocker line printed at load names its keys
  (`SwitchLanguage: registry combination Alt+Shift`) instead of key codes.
- **Breaking:** modifiers match exactly (owner decision; roadmap open
  question 2). A `keys` binding fires only when the held modifiers (Shift,
  Ctrl, Alt, Win; either key of a pair counts as that modifier) are exactly
  the ones it names: `"CapsLock"` no longer fires while Shift is held, and
  `"Ctrl+J"` no longer fires on Ctrl+Shift+J. With the default config,
  CapsLock pressed with Ctrl, Alt or Win held now reaches Windows instead of
  switching the language. Auto-blockers match exactly too: the `Alt+Shift`
  blocker ignores Ctrl+Alt+Shift, and the `` ` `` blocker lets Shift+` through.
  Add `allow_extra_modifiers: true` to a binding for the old behaviour. On
  layouts with AltGr, right Alt arrives together with a left Ctrl, so a
  binding on `RAlt` needs `Ctrl+RAlt` or `allow_extra_modifiers: true`.
- **Breaking:** a `keys` binding on exactly the keys of the Windows hotkey
  now replaces the `block_original_combo` auto-blocker for that hotkey
  (KS-18). With a `when:` on that binding, such as
  `( keys: "Alt+Shift", action: SwitchLanguageBackward, when: AppEquals("code.exe") )`,
  the Windows hotkey now reaches Windows in every other window; it used to
  be blocked everywhere. The auto-blocker also lets the hotkey through
  while another key is down in Windows (KS-05): with the `` ` `` hotkey
  (registry value 4), a `` ` `` typed while another key is held now reaches
  Windows and may switch the layout.
- **Breaking:** unknown and misspelled field names are load errors
  (KS-22): in a binding (`tigger: Tap(200)`, `block_defualt: false`,
  `whne: AppEquals("code.exe")`, `allow_extra_modifier: true`), at the top
  level (`bindngs: [...]`), in `sequence: (...)` and `chord: (...)`, and
  in `Launch(...)`, `PostMessage(...)` and a macro's `Launch(...)` step,
  where `cwd: "C:/temp"` used to be dropped silently. `bindings` is
  required: `()` is an error. The message names the field, for example
  ``1:10: Unexpected field named `bindngs` in `RawConfig`, expected `bindings` instead``.
  Such a file stops the daemon at startup; saved while it runs, it is
  rejected and the previous bindings stay live, where a misspelled
  `bindings` used to leave zero bindings live. `bindings: []` still loads,
  with `[WARN] config contains no bindings`.
- **Breaking:** a config path given on the command line must exist. With
  a missing file the daemon stops with `[FATAL] config file <path> does
  not exist` instead of writing the default template there (creating its
  folders) and starting with it; `key-switch-rs.exe --help` no longer
  creates a file named `--help` (KS-41, an unverified report, partly;
  roadmap item 4.2 adds argument parsing). Without an argument nothing
  changes: a missing `config.ron` next to `key-switch-rs.exe` is still
  created from the template at startup.
- **Breaking:** timing values must be 1 to 10000 ms: the `Tap`, `Hold`
  and `DoubleTap` terms, `max_gap_ms` and `window_ms` (KS-27, an
  unverified report). `Tap(0)` used to swallow the key and almost never
  fire, and `window_ms: 0` gave a chord that swallowed its keys and never
  completed. The error names the binding, counting from 0:
  `binding #1 [CapsLock]: Tap(0) is out of range: timings are 1 to 10000 ms`.
- **Breaking:** an `Immediate` `keys` binding whose action is not
  `PressKey`, `Media(VolumeUp)` or `Media(VolumeDown)` now fires once per
  press instead of on every auto-repeat (KS-10, roadmap item 1.1). Add
  `repeat: true` for the old behaviour, for example to skip tracks with a
  held `Media(Next)` key. A `PressKey` still repeats, one press and release
  per auto-repeat: for a toggle key or a modifier, `repeat: false` keeps
  it to one press.
- **Breaking:** `block_default` applies to `Tap`, `Hold` and `DoubleTap`
  bindings (roadmap item 1.3); it was ignored there, and their key was
  always swallowed. With `block_default: false` the press, the auto-repeats
  and the release of the key reach applications, and the gesture still
  fires. Without the field, a `Tap`, `Hold` or `DoubleTap` binding made only
  of modifiers (`"LShift"`, `"Ctrl+Shift"`, `"Win"`) now lets its keys
  through. Every other binding, every `Immediate` binding included, still
  swallows its key. Write `block_default: true` for the old behaviour.
  - A key pressed while such a gesture is pending does not cancel it: an
    `LShift` `DoubleTap` also fires when the left Shift is pressed twice
    quickly for two capitals (typing `<T>`), and an `LCtrl` `Tap` also
    fires on a quick Ctrl+C.
  - A deferred binding on exactly the keys of the Windows language hotkey,
    such as `( keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200) )`,
    lets Windows switch the layout as well, also when that binding sets
    `block_original_combo: true` itself.
  - A `Tap` on `Alt` or `Win` alone lets Windows see the tap, which
    activates the menu bar or opens Start. The menu mask key (KS-12, under
    **Fixed**) does not cover it: nothing is swallowed there.
  - `block_default: Some(true)` and `block_default: None` (the default)
    load, as for `keys`.
- **Breaking:** one binding wins each key press (roadmap item 1.6, KS-13).
  Among the `keys` bindings that match a key-down, the one with more keys
  wins; with equal key counts, a binding with `when:` beats one without;
  then the one higher in the file. Only the winner runs its action and
  decides whether the key is swallowed. `Tap`, `Hold` and `DoubleTap`
  bindings on the same keys with the same `when:` win or lose together, so
  the README's `Tap(200)` + `Hold(200)` pair on CapsLock works as before. Of
  two such bindings with the same trigger, only the first acts.
  Auto-blockers, `sequence` and `chord` bindings keep their own rules.
  Consequences:
  - A losing binding no longer swallows the key: with
    `( keys: "Shift+F13", action: ToggleCapsLock, block_default: false )`
    and `( keys: "F13", action: SwitchLanguage, allow_extra_modifiers: true )`,
    Shift+F13 reaches applications.
  - A losing `Tap`, `Hold` or `DoubleTap` no longer arms: next to
    `Shift+CapsLock`, a `CapsLock` `Tap(200)` with
    `allow_extra_modifiers: true` no longer switches the language when
    Shift+CapsLock is released. A more specific `Tap` on `Ctrl+F22` no
    longer lets `F22` with `allow_extra_modifiers: true` fire on the same
    press.
  - A binding with `when:` wins over one without on the same keys wherever
    it sits in the file; it used to win only when listed first.
  - An `Immediate` binding and a `Tap`, `Hold` or `DoubleTap` binding on
    the same keys with the same `when:` no longer both act: the higher one
    wins every press. Adding the README's CapsLock `Tap`/`Hold` pair below
    the default `CapsLock` binding leaves the pair inert (it used to switch
    the layout twice per tap); replace the default binding with the pair.
  - A `Tap` and a `Hold` on the same keys with different `when:` no longer
    arm together where both conditions hold; give them the same `when:`.
  - The winner is elected again on every auto-repeat, and the repeats
    follow the key-down. With
    `( keys: "J", action: PressKey("Down"), allow_extra_modifiers: true )`
    and `( keys: "Ctrl+J", action: SwitchLanguage, trigger: Tap(200), block_default: false )`,
    holding Ctrl+J and then releasing Ctrl now lets `j` repeat in the
    application next to one Down per repeat; the losing `J` binding used to
    swallow the press and its repeats.
- **Breaking:** an auto-blocker acts only where the `when:` of a binding
  that asked for it holds (roadmap item 1.2, KS-35); it used to act in every
  window. With `when: Not(AppEquals("game.exe"))` added to the default
  config's CapsLock binding, the Windows hotkey (`Alt+Shift` by default) now
  reaches the game and switches the layout there, while CapsLock reaches it
  as CapsLock; the hotkey used to stay blocked in the game too. Bindings that
  ask for a blocker of the same hotkey and mode share one, which acts where
  any of their `when:` holds, and everywhere if one of them has no `when:`.
  The startup list shows the blocker's condition as for any binding:
  `[AUTO-BLOCK] Alt + Shift              -> (blocked)  [Not(AppEquals("game.exe"))]`.
  The bindings' triggers are not inherited: the blocker acts on the
  hotkey's own press. Its condition is asked only once the hotkey matches,
  so it costs the hook no foreground lookup on other keys. Manual check:
  with `( keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("notepad.exe") )`,
  Alt+Shift in Notepad neither switches the layout nor activates the menu
  bar, and in another window Windows switches the layout.

### Fixed

- KS-01: `swch off` followed by `swch on` no longer leaves remapping dead.
  The keyboard and foreground hooks are installed once, on the daemon's
  main thread, and stay installed until it exits; they used to be removed
  and reinstalled on a short-lived IPC thread, whose exit took the new hooks
  with it, and `swch exit` then stopped at a `[FATAL]` prompt. `swch off`
  now makes the hook pass every key on untouched and `swch on` turns
  remapping back on; `swch status` answers `disabled` while remapping is
  off (it used to say `running` whenever a hook handle was stored). Each
  switch drops pending `Tap`, `Hold`, `DoubleTap`, `chord` and `sequence`
  state, so a `Hold` armed just before `swch off` no longer fires (KS-31,
  an unverified report), and a key still held at `swch on` counts as
  already pressed. A key swallowed by a binding and released while
  remapping is off reaches Windows as a lone key-up. A running macro keeps
  running after `swch off` (roadmap item 2.1).
- KS-02: the daemon's pipe has an explicit security descriptor: full
  control for SYSTEM and Administrators, read and write for interactive
  users, and a medium integrity label. `swch` in a normal console now
  reaches a daemon started elevated by `swch open`, a double-click or a
  Scheduled Task; it used to fail with "Access is denied. (os error 5)".
  Everyone and anonymous logons no longer get read access, and processes
  below medium integrity cannot open the pipe. Read and write go to every
  interactive user of the machine, though: under fast user switching or
  Remote Desktop another signed-in user can switch remapping on or off,
  stop the daemon, or create a pipe instance that answers `swch` in the
  daemon's place, until roadmap item 4.1 gives each session its own pipe.
- `swch` no longer waits forever for a daemon that accepts the connection
  but does not answer, or whose pipe stays busy: after 5 s it prints
  `swch: cannot reach daemon (the daemon did not answer within 5s).` and
  exits with 1. Without a daemon it prints
  ``swch: the daemon is not running. Use `swch open` to start it.``.
  `swch open` treats a pipe that refuses this console, a busy pipe and a
  daemon that does not answer as a running daemon and starts no second one
  (KS-20, partly; roadmap item 1.5 adds the single-instance guard). Its
  check is now a `status` request, so it no longer leaves an
  `IPC read error` or `IPC write error` warning in the daemon console
  (KS-39, an unverified report).
- KS-03: `AppEquals` matches the foreground application. The daemon read
  the executable name with `GetModuleBaseNameW`, which fails on the
  limited process handle it opens, so every `AppEquals` was false,
  `Not(AppEquals(..))` was true even inside that application, and
  `And([AppEquals(..), ..])` never held. It now reads the image path with
  `QueryFullProcessImageNameW` and compares the executable's file name:
  `notepad.exe` for `C:\Windows\System32\notepad.exe`. A binding with
  `when: Not(AppEquals("game.exe"))` now stays silent in that game. The
  auto-blocker of a `block_original_combo` binding follows that binding's
  `when:` too (KS-35, under **Changed**): with
  `when: Not(AppEquals("game.exe"))` added to the default config's CapsLock
  binding, CapsLock reaches the game as CapsLock and the Windows hotkey
  (`Alt+Shift` by default) switches the layout there. Windows of UWP apps such as
  Calculator and Settings belong to `ApplicationFrameHost.exe`; match
  those by title. Manual check (no automated test can focus a window):
  with `( keys: "F9", action: PressKey("A"), when: AppEquals("notepad.exe") )`
  loaded, F9 presses the A key in Notepad (it types that key's letter in
  the active layout) and reaches every other window unchanged.
- KS-04: a `keys` binding fires, or arms its `Tap`/`Hold`/`DoubleTap`, only
  on the key-down of one of its own keys: a non-modifier key, or any of its
  keys for a binding made only of modifiers (`"Alt+Shift"`, `"LShift"`).
  Typing while CapsLock is held no longer re-fires the CapsLock binding or
  swallows the typed keys, Shift pressed while CapsLock is held no longer
  fires `Shift+CapsLock`, and pressing a combo's modifier after its other key
  (J, then Ctrl) no longer fires `Ctrl+J`.
- KS-05 (partly with `true`; the experimental `MaskOnly` under **Added**
  covers the rest): an auto-blocker swallows a key only when Windows would
  see the hotkey's keys and no other key held. Shift pressed while Alt and
  Tab are both held, Shift pressed while Ctrl and T are held, and Alt+Shift
  pressed while a letter is still held in fast typing now reach Windows. A
  key Windows does not have down (one a binding swallowed, or one whose
  release went to the lock screen) does not count. Pressed in the order
  Alt, Shift, Tab, the hotkey's Shift is still swallowed with
  `block_original_combo: true` and the application gets Alt+Tab (Ctrl+T for
  Ctrl, Shift, T with the `Ctrl+Shift` hotkey); `MaskOnly` lets that Shift
  through. That includes the usual way back in the Alt+Tab switcher, Alt
  held, Tab released, then Shift+Tab, which still moves forward with
  `true`.
- KS-06: an auto-repeat is suppressed exactly when its key's first key-down
  was, and a key-down that reaches Windows clears any suppression left for
  its key-up. Holding a key past the repeat delay no longer lets its
  repeats through and then swallows its release, which left the key held
  in Windows: CapsLock with the README `Tap`/`Hold` pair, a `chord` key, a
  `keys: "LShift"` `DoubleTap` key, the key of a combo whose modifiers are
  released first, or Shift after the `Alt+Shift` auto-blocker once Alt is
  released. A key whose key-down reached Windows (CapsLock pressed with
  Ctrl, which no longer matches `"CapsLock"`) keeps its repeats and its
  release when the binding starts matching after Ctrl is released; such a
  repeat fires an `Immediate` action only if the binding repeats (KS-10,
  below).
  A foreground change while a key is held forgets only that its release is
  due to be swallowed (roadmap item 2.3): the release reaches Windows unless
  a `Tap` or `Hold` of the press is still pending, and its repeats stay
  swallowed for every binding kind. They used to be swallowed only while an
  `Immediate` binding matched them, so a key held while the window its
  binding opened took the foreground (a `chord` key, CapsLock with a
  `Hold`, the key of a combo whose modifiers were released) typed into
  that window. A key whose key-down reached Windows keeps its repeats and
  its release across the change. So does a modifier pressed while
  remapping was off: once one of its auto-repeats reaches Windows, the rest
  of the press does too, even when a binding starts matching it.
- KS-07: sided names in `keys` and `sequence` steps fire, and `Shift`,
  `Ctrl` and `Alt` complete a `chord`. The `keys: "LShift"` +
  `DoubleTap(250)` example works, and the left Shift keeps typing capitals
  (roadmap item 1.3).
- KS-10: holding a key no longer re-runs a one-shot action at the
  auto-repeat rate. With the default config, holding CapsLock switches the
  layout once and holding Shift+CapsLock toggles CapsLock once. A held
  `Launch` or `OpenUrl` hotkey starts one program, and a held `Sequence`
  macro runs once. The repeats are still swallowed when the key-down was.
  A press that the daemon reads as an auto-repeat, because the key's
  previous release went to the lock screen or another desktop (KS-29), now
  fires nothing for such a binding; the next press works. For a key whose key-down the daemon swallowed, which is the usual
  trigger key, no roadmap item repairs that record yet: the key resync of
  roadmap item 2.3 leaves such keys alone.
- KS-11: a timer left over from an earlier press no longer acts on a newer
  press of the same key. Two quick taps fire twice, a Hold no longer fires
  early on the next press, and a quick series of DoubleTaps no longer loses
  fires.
- KS-12 (roadmap item 1.2): releasing Win or Alt no longer opens the Start
  menu or activates the menu bar after the daemon swallowed a key pressed
  with it. When the hook swallows a key event (a key-down, an auto-repeat
  or a key-up) while Windows has a Win or Alt key down whose key-down it
  saw, it first sends the menu mask key, the unassigned virtual-key code
  0xE8, down and up, as AutoHotkey's `#MenuMaskKey` does, so Windows no
  longer sees a lone Win or Alt press. This covers
  `( keys: "Win+Q", action: PostMessage(msg: "WM_CLOSE") )`, Alt then Shift
  with the default config's auto-blocker, and
  `( keys: "Alt+X", action: DoNothing )`. Applications, games, key-logging
  tools and other keyboard hooks see the injected 0xE8 events; 0xE8 types
  nothing. A `Tap`, `Hold` or `DoubleTap` on `Alt` or `Win` alone that lets
  the key through still activates the menu bar or opens Start: nothing is
  swallowed there, so nothing is masked. Manual checks (no test injects
  input): in Notepad, Alt down, Shift down, Shift up, Alt up with the
  default config activates no menu; with the `Win+Q` binding, Notepad
  closes and the Start menu stays closed when Win is released. Tests:
  `engine::scenarios::win_q_post_message_masks_the_swallowed_q_and_its_release`,
  `alt_then_shift_with_the_template_blocker_masks_the_swallowed_shift`; the
  engine's property runs check the rule on every generated event.
- KS-25 (equal terms): with `Tap(n)` and `Hold(n)` on one key, exactly one
  fires: `Tap` when the key is released before `n` ms, `Hold` when it is
  held for `n` ms or more. A Hold whose key is released after its term but
  before its timer thread wakes now fires when the timer wakes, still on
  that thread, instead of being cancelled. Different terms behave as before
  until roadmap item 2.2.
- KS-13: specificity decides across every trigger. Of the matching `keys`
  bindings only the winner (or its `Tap`/`Hold`/`DoubleTap` group) acts. A
  less specific binding with `block_default: true` no longer swallows the
  key of a more specific one with `block_default: false`, and a less
  specific `Tap`, `Hold` or `DoubleTap` no longer arms next to a more
  specific binding. Tests:
  `engine::scenarios::shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers`,
  `a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order`,
  `a_more_specific_winner_with_block_default_false_lets_the_key_through`.
  The `when:` of a `keys` binding is now evaluated only when its keys match
  the press, so the hook thread no longer looks up the foreground window
  for it on presses of other keys.
- Two `DoubleTap` bindings on the same keys and `when:` no longer fire the
  first one's action on the first press; only the first binding acts
  (roadmap item 1.6).
- KS-14: the resync of modifiers from the Windows key state leaves alone a
  modifier whose key-down was swallowed, also after a foreground change.
  Windows never saw that key go down, so the next event of any other key
  used to read it as released. It now stays held for matching and its
  auto-repeats stay auto-repeats: with the `Alt+Shift` auto-blocker,
  `Alt+Shift+X` bindings fire in either key order; Shift held by a
  `DoubleTap` binding still counts for `Shift+CapsLock`; one long Shift
  press no longer fires its `DoubleTap(600)`; and `"Ctrl+Shift"` with
  `Hold(1000)`, Ctrl released first, no longer leaves Shift held in Windows.
  Shift pressed after Alt with the default config's auto-blocker now stays
  swallowed for its whole press, auto-repeats and release included, even
  after Alt is released; its repeats used to reach Windows from the first
  one on. `block_original_combo: MaskOnly` (under **Added**) lets that
  Shift through instead.
- KS-18: `block_original_combo` no longer adds an auto-blocker when a
  `keys` binding anywhere in the file uses the same keys as the Windows
  hotkey, in any order and with any trigger or `when:`, the flagged binding
  itself included. That binding now runs its action wherever it sits in the
  file; a binding on `Alt+Shift` below the flagged one, or
  `( keys: "Alt+Shift", action: SwitchLanguageBackward, block_original_combo: true )`
  on its own, never did. The console says so at load
  (`[INFO] no auto-blocker for Alt+Shift: binding #1 uses the same keys`);
  with a `when:` on that binding the Windows hotkey is not blocked where the
  condition does not hold. Auto-blockers are listed after the file's
  bindings at startup. An auto-blocker also no longer runs in place of a
  binding of yours that matches the same press: with `LAlt+LShift` bound,
  that binding fires, and the `Alt+Shift` blocker still swallows Shift
  pressed with the right Alt.
- KS-21: hot reload works when the config path given on the command line
  is relative (`key-switch-rs.exe my.ron`, or a Scheduled Task with a
  relative argument and a "Start in" folder) or differs in letter case
  from the file on disk (`MY.RON` for `my.ron`). The daemon makes the path
  absolute once at startup, so the console shows the full path, and the
  watcher compares file names the way Windows does, ignoring case. It used
  to print `Watching my.ron for changes` and then never reload.
- KS-23: deleting or renaming `config.ron` while the daemon runs keeps the
  live bindings. The daemon used to write the default template in its
  place about 250 ms later and switch to it, so CapsLock switched the
  language again. The console shows `[ERROR] config reload failed:` with
  `config file <path> does not exist`, and putting the file back reloads
  it. The template is written only at startup.
- KS-24: a `config.ron` saved as UTF-8 with a byte-order mark (Notepad's
  "UTF-8 with BOM", or `Set-Content -Encoding UTF8` in Windows PowerShell
  5.1) loads. It used to stop the daemon at startup with
  ``1:1: Expected opening `(` for struct `RawConfig` `` and was rejected
  on every reload.
- KS-28 (an unverified report): `TitleEquals` and `AppEquals` ignore case
  in every alphabet, with the same Unicode lowercasing as `TitleContains`:
  `TitleEquals("КАЛЬКУЛЯТОР")` matches a window titled "Калькулятор". They
  used to ignore case for ASCII letters only.
- KS-30 (an unverified report): a completed `chord` no longer marks keys
  already released for a swallowed key-up, and no longer swallows the
  key-up of a chord key whose key-down reached Windows (J pressed again
  while the chord's `when:` did not hold, before K completed it); that key
  was left held in Windows.
- KS-32 (an unverified report): a completed `sequence` no longer swallows
  the key-up of a key whose key-down reached Windows: a modifier held in
  its last step (for example `Win` in `"Win+F14"`), or F14 pressed first
  for `["F14+F15", "F14+F16"]`, which was left held in Windows.
- KS-33 (partly; an unverified report): Tap and DoubleTap no longer start a
  thread per press, and a Hold timer thread that cannot be started is
  logged instead of aborting the daemon; that Hold then does not fire.
- KS-38: `cargo test --release` and `cargo bench` no longer build a libtest
  harness from `src/main.rs` (`test = false` and `bench = false` on the
  daemon binary). That harness inherited the daemon's `requireAdministrator`
  manifest and failed with "The requested operation requires elevation"
  (os error 740) in a non-elevated shell; `cargo bench` uses the bench
  profile, which inherits release, so it failed on every run. An explicit
  target selection still builds the harness: `cargo test --release` or
  `cargo bench` together with `--all-targets`, `--bins` or
  `--bin key-switch-rs` hits the same error, so run them without those flags.
- KS-42: `build.rs` passes the MSVC linker switches `/MANIFEST:EMBED` and
  `/MANIFESTUAC` only when the target environment is `msvc`. A release build
  for `x86_64-pc-windows-gnu` now links, and warns that no UAC manifest is
  embedded.
- Documentation drift (roadmap Workstream B): the generated `config.ron` no
  longer says that changes need a restart; its `sequence` and `chord`
  examples use F13 and F14 instead of `g`, `j` and `k`, which such bindings
  swallow while you type (KS-15); it lists all eight macro steps, as the
  README now does (it said five). The README lists the key names it left
  out (`Caps`, `Break`, `PrtSc`, `Scroll`, the media keys and more), the
  `swch` command aliases, every `BindAction` variant and `Binding` field,
  and the `windows` features in use. `Launch` is no longer called
  detached: until roadmap item 3.3 the program it starts gets the daemon's
  administrator rights and console (KS-09, KS-34). The README no longer
  says that the elevated daemon sees keys in the UAC consent prompt, which
  runs on the Secure Desktop, or that the UAC prompt appears only once.
  `swch help` is listed in the usage text.
- A `Tap`, `Hold` or `DoubleTap` on a modifier no longer swallows that
  modifier (roadmap item 1.3). With the README and template example
  `( keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250) )`, a
  double press of the left Shift toggles CapsLock, and the left Shift types
  capitals again. Pressing it twice within 250 ms for two capitals toggles
  CapsLock too (see the **Breaking** entry for `block_default` under
  **Changed**).

## [0.1.0] - 2026-05-20

Baseline: the code at commit
[`7cb3521`](https://github.com/bluesteelll/key-switch-rs/commit/7cb3521),
before this changelog existed. It was never tagged or released. The
[audit](https://bluesteelll.github.io/key-switch-rs/audit.html) lists its
known issues, KS-01 to KS-42.

[Unreleased]: https://github.com/bluesteelll/key-switch-rs/compare/7cb3521...HEAD
[0.1.0]: https://github.com/bluesteelll/key-switch-rs/tree/7cb3521
