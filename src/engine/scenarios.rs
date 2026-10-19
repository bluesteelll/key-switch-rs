//! Scenario tests: RON bindings driven through key events on a fake clock.
//!
//! They pin the behaviour of the decision core that is correct today, for
//! every binding kind and trigger. Known defects (see the audit) are not
//! pinned here; the items that fix them add their own scenarios.

use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::testing::Scenario;
use crate::core::windows_actions::{BindAction, MediaKey};
use crate::data::binding::{Binding, BlockerMode};
use crate::data::key_combination::KeyCombination;

fn launch(exe: &str) -> BindAction {
    BindAction::Launch {
        exe: exe.to_owned(),
        args: Vec::new(),
    }
}

/// The default config's two bindings, without `block_original_combo`.
pub(super) const DEFAULT_PAIR: &str = r#"( bindings: [
    (keys: "CapsLock", action: SwitchLanguage),
    (keys: "Shift+CapsLock", action: ToggleCapsLock),
] )"#;

/// The `Alt+Shift` auto-blocker that `block_original_combo: true` adds for
/// the Windows language hotkey, followed by the bindings of `ron`. The
/// blocker comes first, where it used to shadow a binding on the same keys;
/// the loader now appends it. It is built directly: loading it from RON
/// would read the registry.
pub(super) fn with_alt_shift_blocker(ron: &str) -> Vec<Binding> {
    with_alt_shift_blocker_of(BlockerMode::Swallow, ron)
}

/// [`with_alt_shift_blocker`] with the experimental
/// `block_original_combo: MaskOnly` blocker.
pub(super) fn with_alt_shift_mask_only_blocker(ron: &str) -> Vec<Binding> {
    with_alt_shift_blocker_of(BlockerMode::MaskOnly, ron)
}

fn with_alt_shift_blocker_of(mode: BlockerMode, ron: &str) -> Vec<Binding> {
    let mut bindings = vec![Binding::new_auto_blocker(
        KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT]),
        mode,
    )];
    bindings.extend(
        crate::config::from_ron_str(ron)
            .unwrap_or_else(|e| panic!("scenario: config does not load: {e}")),
    );
    bindings
}

// ---- Immediate combos ----

#[test]
fn immediate_single_key_fires_on_down_and_swallows_both_edges() {
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(80)
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired();
}

#[test]
fn immediate_combo_fires_on_its_last_key_and_lets_the_modifiers_through() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Alt+L", action: Launch(exe: "notepad.exe")) ] )"#,
    )
    .down("Ctrl")
    .expect_passed()
    .expect_nothing_fired()
    .down("Alt")
    .expect_passed()
    .expect_nothing_fired()
    .down("L")
    .expect_suppressed()
    .expect_fired(&[launch("notepad.exe")])
    .up("L")
    .expect_suppressed()
    .up("Alt")
    .expect_passed()
    .up("Ctrl")
    .expect_passed()
    .expect_total_fired(&[launch("notepad.exe")]);
}

#[test]
fn right_hand_modifiers_match_unsided_combo_keys() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Alt+L", action: Launch(exe: "notepad.exe")) ] )"#,
    )
    .down("RCtrl")
    .down("RAlt")
    .down("L")
    .expect_suppressed()
    .expect_fired(&[launch("notepad.exe")]);
}

#[test]
fn combo_does_not_fire_without_all_of_its_keys() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Alt+L", action: Launch(exe: "notepad.exe")) ] )"#,
    )
    .down("L")
    .expect_passed()
    .expect_nothing_fired()
    .up("L")
    .expect_passed()
    .down("Ctrl")
    .down("L")
    .expect_passed()
    .expect_nothing_fired()
    .expect_total_fired(&[]);
}

#[test]
fn unbound_key_passes_through() {
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("A")
        .expect_passed()
        .expect_nothing_fired()
        .up("A")
        .expect_passed()
        .expect_nothing_fired();
}

#[test]
fn more_specific_immediate_binding_wins() {
    // Listed least specific first: the engine visits Shift+CapsLock first
    // anyway, and only the first matching Immediate binding fires.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage),
        (keys: "Shift+CapsLock", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .down("Shift")
        .expect_passed()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("CapsLock")
        .expect_suppressed()
        .up("Shift")
        .expect_passed()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn block_default_false_fires_and_lets_both_edges_through() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: ToggleCapsLock, block_default: false) ] )"#,
    )
    .down("F13")
    .expect_passed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("F13")
    .expect_passed()
    .expect_nothing_fired();
}

#[test]
fn press_key_fires_again_on_auto_repeat_and_the_repeats_stay_suppressed() {
    // Repeating is the intended policy for PressKey: holding the trigger
    // types the key repeatedly, like holding the key itself.
    Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: PressKey("Left")) ] )"#)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)])
        .advance_ms(500)
        .repeat("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)])
        .advance_ms(33)
        .repeat("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)])
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired();
}

#[test]
fn modifier_lost_by_the_os_is_resynced_before_matching() {
    // Shift's key-up went to the secure desktop. The next event reads the
    // OS state, so CapsLock alone matches instead of Shift+CapsLock.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage),
        (keys: "Shift+CapsLock", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .down("Shift")
        .up_unseen("Shift")
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn foreground_change_forgets_pending_key_up_suppression() {
    // A key-up that the hook might never see must not stay armed to
    // swallow the key's next release.
    Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: SwitchLanguage) ] )"#)
        .down("F13")
        .expect_suppressed()
        .foreground_changed()
        .up("F13")
        .expect_passed();
}

// ---- when: conditions ----

#[test]
fn when_app_equals_matches_the_foreground_app_case_insensitively() {
    let config = r#"( bindings: [
        (keys: "F13", action: SwitchLanguage, when: AppEquals("code.exe")),
    ] )"#;
    Scenario::from_ron(config)
        .foreground("Code.exe", "main.rs - Visual Studio Code")
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .expect_suppressed();

    Scenario::from_ron(config)
        .foreground("notepad.exe", "Untitled - Notepad")
        .down("F13")
        .expect_passed()
        .expect_nothing_fired()
        .up("F13")
        .expect_passed();
}

#[test]
fn when_title_contains_and_not_conditions() {
    let config = r#"( bindings: [
        (keys: "F13", action: Media(PlayPause), when: TitleContains("spotify")),
        (keys: "F14", action: SwitchLanguage, when: Not(AppEquals("chrome.exe"))),
    ] )"#;
    let play_pause = BindAction::Media(crate::core::windows_actions::MediaKey::PlayPause);

    Scenario::from_ron(config)
        .foreground("Spotify.exe", "Spotify Premium")
        .down("F13")
        .expect_fired(std::slice::from_ref(&play_pause))
        .up("F13")
        .down("F14")
        .expect_fired(&[BindAction::SwitchLanguage]);

    Scenario::from_ron(config)
        .foreground("chrome.exe", "New Tab - Google Chrome")
        .down("F13")
        .expect_passed()
        .expect_nothing_fired()
        .up("F13")
        .down("F14")
        .expect_passed()
        .expect_nothing_fired();
}

#[test]
fn a_binding_whose_condition_holds_wins_on_a_shared_key() {
    let config = r#"( bindings: [
        (keys: "F13", action: PressKey("F5"), when: AppEquals("code.exe")),
        (keys: "F13", action: SwitchLanguage),
    ] )"#;
    Scenario::from_ron(config)
        .foreground("code.exe", "main.rs")
        .down("F13")
        .expect_fired(&[BindAction::PressKey(VK_F5)]);

    Scenario::from_ron(config)
        .foreground("explorer.exe", "Downloads")
        .down("F13")
        .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn when_title_equals_and_app_equals_ignore_case_beyond_ascii() {
    // "Kalkulyator" and "Privet.exe", written in Cyrillic; the config names
    // them in upper case.
    let title = "\u{41a}\u{430}\u{43b}\u{44c}\u{43a}\u{443}\u{43b}\u{44f}\u{442}\u{43e}\u{440}";
    let app = "\u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}.exe";
    let config = format!(
        r#"( bindings: [
        (keys: "F13", action: SwitchLanguage, when: TitleEquals("{}")),
        (keys: "F14", action: ToggleCapsLock, when: AppEquals("{}")),
    ] )"#,
        title.to_uppercase(),
        app.to_uppercase()
    );

    Scenario::from_ron(&config)
        .foreground(app, title)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .expect_suppressed()
        .down("F14")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("F14")
        .expect_suppressed();

    Scenario::from_ron(&config)
        .foreground(app, "\u{41a}\u{430}\u{43b}")
        .down("F13")
        .expect_passed()
        .expect_nothing_fired()
        .up("F13")
        .expect_passed();
}

// ---- Tap ----

#[test]
fn tap_fires_on_a_release_within_its_term() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)) ] )"#,
    )
    .down("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(199)
    .expect_nothing_fired()
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn tap_held_past_its_term_fires_nothing_and_swallows_the_release() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)) ] )"#,
    )
    .down("CapsLock")
    .expect_suppressed()
    .advance_ms(200)
    .expect_nothing_fired()
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired();
}

#[test]
fn tap_held_past_its_term_with_auto_repeat_fires_nothing_on_release() {
    // An auto-repeat must not re-arm the overdue Tap for the release to
    // fire. The repeat itself is swallowed like its key-down (KS-06, see
    // "Auto-repeat follows its key-down").
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: PressKey("LShift"), trigger: Tap(200)) ] )"#,
    )
    .down("CapsLock")
    .advance_ms(300)
    .repeat("CapsLock")
    .advance_ms(10)
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired();
}

#[test]
fn two_quick_taps_fire_twice() {
    // KS-11: the first press's expiry must not delete the second press's
    // tap. Release at +80, press again at +150, release at +230: the
    // first press's term ends at +200, in the middle of the second tap.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)) ] )"#,
    )
    .down("CapsLock")
    .advance_ms(80)
    .up("CapsLock")
    .expect_fired(&[BindAction::SwitchLanguage])
    .advance_ms(70)
    .down("CapsLock")
    .advance_ms(80)
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .expect_total_fired(&[BindAction::SwitchLanguage, BindAction::SwitchLanguage]);
}

#[test]
fn stale_tap_gc_eats_second_quick_tap() {
    // Ported from the gesture repro crate (`book/src/methodology.md`,
    // "Gesture repro crate") with its assertion inverted: the name is the
    // defect guarded against (KS-11). Press 1's cleanup at +200 must not
    // delete press 2's Tap, armed at +120 and released at +230.
    Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: DoNothing, trigger: Tap(200)) ] )"#)
        .down("F13")
        .advance_ms(20)
        .up("F13")
        .expect_fired(&[BindAction::DoNothing])
        .advance_ms(100)
        .down("F13")
        .advance_ms(110)
        .up("F13")
        .expect_fired(&[BindAction::DoNothing])
        .expect_total_fired(&[BindAction::DoNothing, BindAction::DoNothing]);
}

// ---- Hold ----

#[test]
fn stale_hold_timer_fires_new_press_early() {
    // Ported from the gesture repro crate (`book/src/methodology.md`,
    // "Gesture repro crate") with its assertion inverted: the name is the
    // defect guarded against (KS-11). Press 1 is released at +20; press 2,
    // armed at +120, must still be pending at +230, after press 1's timer
    // fell due at +200, and fire at its own term, +320.
    Scenario::from_ron(r#"( bindings: [ (keys: "F14", action: DoNothing, trigger: Hold(200)) ] )"#)
        .down("F14")
        .advance_ms(20)
        .up("F14")
        .expect_nothing_fired()
        .advance_ms(100)
        .down("F14")
        .advance_ms(110)
        .expect_nothing_fired()
        .advance_ms(89)
        .expect_nothing_fired()
        .advance_ms(1)
        .expect_fired(&[BindAction::DoNothing])
        .up("F14")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::DoNothing]);
}

#[test]
fn hold_fires_once_its_term_passes_while_held() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)) ] )"#,
    )
    .down("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(199)
    .expect_nothing_fired()
    .advance_ms(1)
    .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
    .advance_ms(300)
    .expect_nothing_fired()
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired();
}

#[test]
fn hold_released_early_is_cancelled() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)) ] )"#,
    )
    .down("CapsLock")
    .advance_ms(120)
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(500)
    .expect_nothing_fired()
    .expect_total_fired(&[]);
}

#[test]
fn stale_hold_timer_does_not_fire_the_next_press_early() {
    // KS-11: press 1 is released at +80, press 2 starts at +150. Press 1's
    // timer falls due at +200 and must not fire press 2's hold 50 ms in;
    // press 2's own timer fires at +350.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)) ] )"#,
    )
    .down("CapsLock")
    .advance_ms(80)
    .up("CapsLock")
    .advance_ms(70)
    .down("CapsLock")
    .advance_ms(60)
    .expect_nothing_fired()
    .advance_ms(139)
    .expect_nothing_fired()
    .advance_ms(1)
    .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
    .up("CapsLock")
    .expect_suppressed()
    .expect_total_fired(&[BindAction::PressKey(VK_LSHIFT)]);
}

#[test]
fn tap_and_hold_with_equal_terms_fire_exactly_one() {
    // The README's CapsLock pair.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
    ] )"#;
    Scenario::from_ron(config)
        .down("CapsLock")
        .advance_ms(199)
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(500)
        .expect_total_fired(&[BindAction::SwitchLanguage]);

    Scenario::from_ron(config)
        .down("CapsLock")
        .advance_ms(201)
        .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::PressKey(VK_LSHIFT)]);
}

#[test]
fn hold_released_after_its_term_fires_when_its_late_timer_wakes() {
    // KS-25: the timer thread wakes late, and the key is released at +250,
    // after the 200 ms term but before the timer runs. The key was held for
    // the whole term, so the Hold fires, from its timer and never on the
    // key-up, exactly once.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)) ] )"#,
    )
    .down("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms_stalled(250)
    .expect_nothing_fired()
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(10)
    .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
    .advance_ms(500)
    .expect_nothing_fired()
    .expect_total_fired(&[BindAction::PressKey(VK_LSHIFT)]);
}

#[test]
fn equal_tap_and_hold_terms_with_a_late_timer_fire_exactly_one() {
    // KS-25 with the README's CapsLock pair and a late hold timer.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
    ] )"#;

    // Released right at the term: the Tap is over, the Hold is due.
    Scenario::from_ron(config)
        .down("CapsLock")
        .advance_ms_stalled(200)
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(50)
        .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
        .expect_total_fired(&[BindAction::PressKey(VK_LSHIFT)]);

    // Released 1 ms before the term: the Tap fires and the late timer
    // finds its Hold cancelled.
    Scenario::from_ron(config)
        .down("CapsLock")
        .advance_ms_stalled(199)
        .up("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(50)
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn hold_released_before_its_term_with_a_late_timer_is_cancelled() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)) ] )"#,
    )
    .down("CapsLock")
    .advance_ms_stalled(199)
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(50)
    .expect_nothing_fired()
    .expect_total_fired(&[]);
}

#[test]
fn hold_left_to_a_late_timer_does_not_block_the_next_press() {
    // Press 1 is held past the term and released at +250, before its timer
    // runs. Press 2 starts at +260 and is held. Press 1's late timer fires
    // press 1's Hold; press 2 arms its own Hold, which fires at +460.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)) ] )"#,
    )
    .down("CapsLock")
    .advance_ms_stalled(250)
    .up("CapsLock")
    .advance_ms_stalled(10)
    .down("CapsLock")
    .expect_suppressed()
    .advance_ms(10)
    .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
    .advance_ms(189)
    .expect_nothing_fired()
    .advance_ms(1)
    .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .expect_total_fired(&[
        BindAction::PressKey(VK_LSHIFT),
        BindAction::PressKey(VK_LSHIFT),
    ]);
}

#[test]
fn hold_whose_timer_cannot_start_never_fires() {
    // KS-33: the caller could not start the timer and tells the engine.
    // The Hold neither fires nor lingers: the key-up is still swallowed and
    // the next press arms and fires a fresh Hold.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)) ] )"#,
    )
    .down("CapsLock")
    .expect_suppressed()
    .lose_timers()
    .advance_ms(300)
    .expect_nothing_fired()
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .down("CapsLock")
    .expect_suppressed()
    .advance_ms(200)
    .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
    .up("CapsLock")
    .expect_suppressed()
    .expect_total_fired(&[BindAction::PressKey(VK_LSHIFT)]);
}

// ---- DoubleTap ----

#[test]
fn double_tap_fires_on_the_second_press_within_its_term() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(250)) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(60)
    .up("F13")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(100)
    .down("F13")
    .expect_suppressed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("F13")
    .expect_suppressed()
    .expect_nothing_fired();
}

#[test]
fn double_tap_second_press_after_the_term_arms_again() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(250)) ] )"#,
    )
    .down("F13")
    .up("F13")
    .advance_ms(300)
    .down("F13")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("F13")
    .advance_ms(100)
    .down("F13")
    .expect_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn four_quick_presses_double_tap_twice() {
    // KS-11: presses at 0, 100, 190 and 260 ms with a 200 ms term. The
    // first press's expiry (at 200) must not delete the entry armed at 190.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(200)) ] )"#,
    )
    .down("F13")
    .advance_ms(40)
    .up("F13")
    .advance_ms(60)
    .down("F13")
    .expect_fired(&[BindAction::ToggleCapsLock])
    .advance_ms(40)
    .up("F13")
    .advance_ms(50)
    .down("F13")
    .expect_nothing_fired()
    .advance_ms(40)
    .up("F13")
    .advance_ms(30)
    .down("F13")
    .expect_fired(&[BindAction::ToggleCapsLock])
    .expect_total_fired(&[BindAction::ToggleCapsLock, BindAction::ToggleCapsLock]);
}

// ---- One gesture per press (roadmap item 2.2, KS-25, known issue 74) ----

/// A Tap(300) and a Hold(200) on CapsLock.
const TAP_300_HOLD_200: &str = r#"( bindings: [
    (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(300)),
    (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
] )"#;

/// A Tap(200) and a DoubleTap(300) on F13.
const TAP_AND_DOUBLE_TAP_F13: &str = r#"( bindings: [
    (keys: "F13", action: SwitchLanguage, trigger: Tap(200)),
    (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(300)),
] )"#;

#[test]
fn tap_300_and_hold_200_released_at_250_fire_only_the_hold() {
    // KS-25 for different terms: the Hold resolves the press at 200 ms, so
    // the release at 250 ms, within the Tap's term, fires nothing more.
    Scenario::from_ron(TAP_300_HOLD_200)
        .down("CapsLock")
        .advance_ms(200)
        .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
        .advance_ms(50)
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(500)
        .expect_total_fired(&[BindAction::PressKey(VK_LSHIFT)]);

    // The same with the Hold's timer late: the key-up at 250 ms leaves the
    // Hold to its timer and fires no Tap.
    Scenario::from_ron(TAP_300_HOLD_200)
        .down("CapsLock")
        .advance_ms_stalled(250)
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(10)
        .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
        .expect_total_fired(&[BindAction::PressKey(VK_LSHIFT)]);
}

#[test]
fn tap_100_and_hold_200_released_at_150_fire_nothing() {
    // Too long for the Tap, too short for the Hold: a declared gap of
    // unequal terms (triggers-conditions.md, 11.6).
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(100)),
            (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
        ] )"#,
    )
    .down("CapsLock")
    .advance_ms(150)
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(500)
    .expect_total_fired(&[]);
}

#[test]
fn a_hold_closes_the_double_tap_window() {
    // A press that its Hold resolved opens no DoubleTap window: the next
    // press is a first press.
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "F13", action: SwitchLanguage, trigger: Hold(200)),
            (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(300)),
        ] )"#,
    )
    .down("F13")
    .advance_ms(200)
    .expect_fired(&[BindAction::SwitchLanguage])
    .advance_ms(50)
    .up("F13")
    .expect_nothing_fired()
    .advance_ms(30)
    .down("F13")
    .expect_nothing_fired()
    .up("F13")
    .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn a_double_tap_press_arms_no_hold() {
    // The second press completes the DoubleTap and is spent: it arms no
    // Hold of its own.
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(300)),
            (keys: "F13", action: PressKey("LShift"), trigger: Hold(200)),
        ] )"#,
    )
    .down("F13")
    .advance_ms(50)
    .up("F13")
    .advance_ms(50)
    .down("F13")
    .expect_suppressed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .advance_ms(300)
    .expect_nothing_fired()
    .up("F13")
    .expect_suppressed()
    .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn tap_and_double_tap_on_one_key_wait_for_the_double_tap_window_before_the_tap() {
    // Open question 5 of the roadmap: with a DoubleTap on the same keys the
    // Tap waits until the DoubleTap window closes, 300 ms after the press,
    // and fires then, on its timer.
    Scenario::from_ron(TAP_AND_DOUBLE_TAP_F13)
        .down("F13")
        .expect_suppressed()
        .advance_ms(80)
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(219)
        .expect_nothing_fired()
        .advance_ms(1)
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(500)
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn a_double_press_of_a_tap_and_double_tap_key_fires_only_the_double_tap() {
    Scenario::from_ron(TAP_AND_DOUBLE_TAP_F13)
        .down("F13")
        .advance_ms(80)
        .up("F13")
        .expect_nothing_fired()
        .advance_ms(70)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .advance_ms(50)
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(500)
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn another_key_fires_a_tap_that_waits_for_the_double_tap_window() {
    // The waiting Tap completed before A went down, so it fires on A's
    // key-down, ahead of A, as a Tap without a DoubleTap would have fired
    // at its release; that closes the window, and the next F13 press is a
    // first press, whose Tap waits for its own window.
    Scenario::from_ron(TAP_AND_DOUBLE_TAP_F13)
        .down("F13")
        .advance_ms(80)
        .up("F13")
        .expect_nothing_fired()
        .advance_ms(40)
        .down("A")
        .expect_passed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("A")
        .expect_passed()
        .advance_ms(80)
        .down("F13")
        .expect_nothing_fired()
        .advance_ms(50)
        .up("F13")
        .expect_nothing_fired()
        .advance_ms(249)
        .expect_nothing_fired()
        .advance_ms(1)
        .expect_fired(&[BindAction::SwitchLanguage])
        .expect_total_fired(&[BindAction::SwitchLanguage, BindAction::SwitchLanguage]);
}

#[test]
fn a_stale_hold_timer_does_not_fire_a_tap_waiting_for_the_double_tap_window() {
    // The key-up at 50 ms cancels the Hold, and the waiting press takes a
    // new number: the Hold's timer at 200 ms finds nothing, and the Tap
    // fires at the window's end, 300 ms.
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "F13", action: SwitchLanguage, trigger: Tap(100)),
            (keys: "F13", action: PressKey("LShift"), trigger: Hold(200)),
            (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(300)),
        ] )"#,
    )
    .down("F13")
    .advance_ms(50)
    .up("F13")
    .expect_nothing_fired()
    .advance_ms(150)
    .expect_nothing_fired()
    .advance_ms(99)
    .expect_nothing_fired()
    .advance_ms(1)
    .expect_fired(&[BindAction::SwitchLanguage])
    .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn a_double_tap_entry_is_not_consumed_by_a_press_another_group_wins() {
    // Known issue 74: the CapsLock press that Shift+CapsLock wins ends the
    // CapsLock DoubleTap's window instead of completing it, and starts the
    // window of its own group.
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, trigger: DoubleTap(300)),
            (keys: "Shift+CapsLock", action: ToggleCapsLock, trigger: DoubleTap(300)),
        ] )"#,
    )
    .down("CapsLock")
    .advance_ms(40)
    .up("CapsLock")
    .advance_ms(20)
    .down("LShift")
    .advance_ms(20)
    .down("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(40)
    .up("CapsLock")
    .advance_ms(40)
    .down("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("CapsLock")
    .up("LShift")
    .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn a_tap_released_after_the_double_tap_window_closed_fires_at_its_key_up() {
    // The window is shorter than the Tap: once it has closed, nothing can
    // turn the press into a DoubleTap, so the Tap fires at its release.
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "F13", action: SwitchLanguage, trigger: Tap(300)),
            (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(100)),
        ] )"#,
    )
    .down("F13")
    .advance_ms(150)
    .up("F13")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .advance_ms(500)
    .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn a_tap_whose_window_timer_is_lost_fires_at_the_next_key_down() {
    // A waiting Tap whose timer could not be set is not dropped: the next
    // key-down fires it, ahead of that key (orchestrator ruling M6).
    Scenario::from_ron(TAP_AND_DOUBLE_TAP_F13)
        .down("F13")
        .advance_ms(80)
        .up("F13")
        .expect_nothing_fired()
        .lose_timers()
        .advance_ms(400)
        .expect_nothing_fired()
        .down("A")
        .expect_passed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("A")
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn a_waiting_tap_whose_window_closed_while_its_timer_was_late_fires_at_its_own_key_down() {
    // The clock decides, not the timer (roadmap item 2.2, results P1): the
    // window closed at +300 ms while its timer was stalled, so the press at
    // +381 ms is a first press. It fires the waiting Tap ahead of itself,
    // completes no DoubleTap and waits for a window of its own.
    Scenario::from_ron(TAP_AND_DOUBLE_TAP_F13)
        .down("F13")
        .advance_ms(80)
        .up("F13")
        .expect_nothing_fired()
        .advance_ms_stalled(301)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(50)
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(300)
        .expect_fired(&[BindAction::SwitchLanguage])
        .expect_total_fired(&[BindAction::SwitchLanguage, BindAction::SwitchLanguage]);
}

#[test]
fn a_press_after_a_lost_key_up_replaces_the_stale_press_and_keeps_a_hold_due_by_the_clock() {
    // Orchestrator ruling M6: the left Shift passes, its key-up goes
    // unseen, and A's key-up resyncs it as up, with no key-down in between.
    // The next LShift press replaces the stale one. The stale press's Hold
    // was due by the clock before that, so it stays for its late timer
    // (ruling I1), and the new press arms a Hold of its own.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "LShift", action: DoNothing, trigger: Hold(100)) ] )"#,
    )
    .down("A")
    .advance_ms(10)
    .down("LShift")
    .expect_passed()
    .advance_ms(10)
    .up_unseen("LShift")
    .advance_ms_stalled(200)
    .up("A")
    .advance_ms_stalled(10)
    .down("LShift")
    .expect_passed()
    .advance_ms(10)
    .expect_fired(&[BindAction::DoNothing])
    .advance_ms(90)
    .expect_fired(&[BindAction::DoNothing])
    .up("LShift")
    .expect_passed()
    .expect_total_fired(&[BindAction::DoNothing, BindAction::DoNothing]);

    // A stale Tap that is still live does not answer for the new press: the
    // new press is timed from its own key-down, so its release 450 ms later,
    // after the stale term but within its own, fires it.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "LShift", action: DoNothing, trigger: Tap(500)) ] )"#,
    )
    .down("A")
    .advance_ms(10)
    .down("LShift")
    .advance_ms(10)
    .up_unseen("LShift")
    .up("A")
    .advance_ms(80)
    .down("LShift")
    .advance_ms(450)
    .up("LShift")
    .expect_fired(&[BindAction::DoNothing])
    .expect_total_fired(&[BindAction::DoNothing]);
}

// ---- Sequence ----

#[test]
fn sequence_fires_when_its_steps_follow_within_max_gap() {
    Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("F13")
    .expect_suppressed()
    .advance_ms(300)
    .down("F14")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("F14")
    .expect_suppressed()
    .expect_nothing_fired();
}

#[test]
fn sequence_times_out_after_max_gap() {
    Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["F13", "F14"], max_gap_ms: 500), action: SwitchLanguage) ] )"#,
    )
    .down("F13")
    .up("F13")
    .advance_ms(500)
    .down("F14")
    .expect_passed()
    .expect_nothing_fired()
    .up("F14")
    .expect_passed()
    .expect_total_fired(&[]);
}

#[test]
fn sequence_broken_by_another_key_does_not_fire() {
    Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage) ] )"#,
    )
    .down("F13")
    .up("F13")
    .down("A")
    .expect_passed()
    .up("A")
    .down("F14")
    .expect_passed()
    .expect_nothing_fired()
    .expect_total_fired(&[]);
}

#[test]
fn sequence_with_a_modifier_held_through_both_steps() {
    // Emacs-style Ctrl+X Ctrl+S with Ctrl held the whole time. Ctrl itself
    // reaches the OS; X and S are swallowed.
    Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["Ctrl+X", "Ctrl+S"]), action: SwitchLanguage) ] )"#,
    )
    .down("Ctrl")
    .expect_passed()
    .down("X")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("X")
    .expect_suppressed()
    .down("S")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("S")
    .expect_suppressed()
    .up("Ctrl")
    .expect_passed();
}

// ---- Chord ----

#[test]
fn chord_fires_when_all_keys_go_down_within_its_window() {
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 50), action: SwitchLanguage) ] )"#,
    )
    .down("J")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(30)
    .down("K")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("J")
    .expect_suppressed()
    .up("K")
    .expect_suppressed()
    .expect_nothing_fired();
}

#[test]
fn chord_keys_can_go_down_in_any_order() {
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 50), action: SwitchLanguage) ] )"#,
    )
    .down("K")
    .down("J")
    .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn chord_does_not_fire_once_its_window_has_passed() {
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 50), action: SwitchLanguage) ] )"#,
    )
    .down("J")
    .advance_ms(50)
    .down("K")
    .expect_suppressed()
    .expect_nothing_fired()
    .expect_total_fired(&[]);
}

#[test]
fn key_outside_the_chord_passes() {
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 50), action: SwitchLanguage) ] )"#,
    )
    .down("A")
    .expect_passed()
    .up("A")
    .expect_passed();
}

#[test]
fn chord_completed_after_a_focus_change_swallows_the_key_up_of_its_first_key() {
    // The focus change forgets that J's key-up is due to be swallowed; the
    // chord completing on K marks every chord key again, so J's key-up is
    // swallowed as well and the app never sees an unmatched J key-up.
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 500), action: SwitchLanguage) ] )"#,
    )
    .down("J")
    .expect_suppressed()
    .foreground_changed()
    .down("K")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("J")
    .expect_suppressed()
    .up("K")
    .expect_suppressed();
}

// ---- Sided keys (KS-07) ----

#[test]
fn lshift_in_keys_fires_only_for_the_left_shift() {
    Scenario::from_ron(r#"( bindings: [ (keys: "LShift+F13", action: SwitchLanguage) ] )"#)
        .down("LShift")
        .expect_passed()
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .expect_suppressed()
        .up("LShift")
        .expect_passed()
        .down("RShift")
        .expect_passed()
        .down("F13")
        .expect_passed()
        .expect_nothing_fired()
        .up("F13")
        .expect_passed()
        .up("RShift")
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn rwin_and_lwin_in_keys_fire_only_for_their_side() {
    let config = r#"( bindings: [
        (keys: "RWin+F13", action: SwitchLanguage),
        (keys: "LWin+F14", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .down("RWin")
        .expect_passed()
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .up("RWin")
        .down("LWin")
        .down("F13")
        .expect_passed()
        .expect_nothing_fired()
        .up("F13")
        .up("LWin")
        .down("LWin")
        .down("F14")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("F14")
        .up("LWin")
        .down("RWin")
        .down("F14")
        .expect_passed()
        .expect_nothing_fired()
        .up("F14")
        .up("RWin")
        .expect_total_fired(&[BindAction::SwitchLanguage, BindAction::ToggleCapsLock]);
}

#[test]
fn unsided_shift_and_win_in_keys_fire_for_either_side() {
    let config = r#"( bindings: [
        (keys: "Shift+F13", action: SwitchLanguage),
        (keys: "Win+F14", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .down("LShift")
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .up("LShift")
        .down("RShift")
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .up("RShift")
        .down("LWin")
        .down("F14")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("F14")
        .up("LWin")
        .down("RWin")
        .down("F14")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn lshift_double_tap_fires_only_for_the_left_shift() {
    // The template's commented example (`default_config.ron`). A binding
    // made only of modifiers lets its key through by default (roadmap item
    // 1.3), so the left Shift reaches Windows on every edge.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250)) ] )"#,
    )
    .down("LShift")
    .expect_passed()
    .expect_nothing_fired()
    .up("LShift")
    .expect_passed()
    .advance_ms(100)
    .down("LShift")
    .expect_passed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("LShift")
    .expect_passed()
    .advance_ms(500)
    .down("RShift")
    .expect_passed()
    .up("RShift")
    .expect_passed()
    .advance_ms(100)
    .down("RShift")
    .expect_passed()
    .expect_nothing_fired()
    .up("RShift")
    .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn unsided_shift_in_a_chord_fires_for_either_side() {
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["Shift", "Space"], window_ms: 50), action: SwitchLanguage) ] )"#,
    )
    .down("LShift")
    .expect_suppressed()
    .advance_ms(20)
    .down("Space")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("Space")
    .expect_suppressed()
    .up("LShift")
    .expect_suppressed()
    .advance_ms(100)
    .down("RShift")
    .expect_suppressed()
    .advance_ms(20)
    .down("Space")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("Space")
    .expect_suppressed()
    .up("RShift")
    .expect_suppressed()
    .expect_total_fired(&[BindAction::SwitchLanguage, BindAction::SwitchLanguage]);
}

#[test]
fn sided_shift_in_a_chord_is_only_its_side() {
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["LShift", "F13"], window_ms: 50), action: SwitchLanguage) ] )"#,
    )
    .down("RShift")
    .expect_passed()
    .down("F13")
    .expect_suppressed()
    .expect_nothing_fired();
}

#[test]
fn sided_shift_in_a_sequence_step_matches_only_its_side() {
    let config =
        r#"( bindings: [ (sequence: (steps: ["F13", "LShift+F14"]), action: SwitchLanguage) ] )"#;
    Scenario::from_ron(config)
        .down("LShift")
        .expect_passed()
        .down("F13")
        .expect_suppressed()
        .up("F13")
        .expect_suppressed()
        .down("F14")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F14")
        .expect_suppressed()
        .up("LShift")
        .expect_passed();

    Scenario::from_ron(config)
        .down("RShift")
        .expect_passed()
        .down("F13")
        .expect_suppressed()
        .up("F13")
        .expect_suppressed()
        .down("F14")
        .expect_passed()
        .expect_nothing_fired()
        .expect_total_fired(&[]);
}

#[test]
fn win_in_the_last_sequence_step_does_not_swallow_its_key_up() {
    // Win's key-down reached the OS, so its key-up must too.
    Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["F13", "Win+F14"]), action: SwitchLanguage) ] )"#,
    )
    .down("Win")
    .expect_passed()
    .down("F13")
    .expect_suppressed()
    .up("F13")
    .expect_suppressed()
    .down("F14")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("F14")
    .expect_suppressed()
    .up("Win")
    .expect_passed();
}

// ---- Trigger key (KS-04) ----

#[test]
fn key_pressed_while_an_immediate_binding_key_is_held_passes() {
    // Rollover: `a` goes down before CapsLock is released.
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(30)
        .down("A")
        .expect_passed()
        .expect_nothing_fired()
        .up("A")
        .expect_passed()
        .up("CapsLock")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn key_pressed_while_a_tap_hold_key_is_held_passes_and_arms_nothing() {
    // The README's CapsLock pair.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
    ] )"#;
    Scenario::from_ron(config)
        .down("CapsLock")
        .expect_suppressed()
        .advance_ms(50)
        .down("A")
        .expect_passed()
        .expect_nothing_fired()
        .up("A")
        .expect_passed()
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(500)
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn modifier_pressed_while_capslock_is_held_does_not_fire_shift_capslock() {
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage),
        (keys: "Shift+CapsLock", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .down("LShift")
        .expect_passed()
        .expect_nothing_fired()
        .up("LShift")
        .expect_passed()
        .up("CapsLock")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn modifier_pressed_last_does_not_fire_a_combo() {
    Scenario::from_ron(r#"( bindings: [ (keys: "Ctrl+J", action: SwitchLanguage) ] )"#)
        .down("J")
        .expect_passed()
        .down("LCtrl")
        .expect_passed()
        .expect_nothing_fired()
        .up("LCtrl")
        .expect_passed()
        .up("J")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn deferred_combo_arms_only_on_its_trigger_key() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Shift+CapsLock", action: ToggleCapsLock, trigger: Tap(200)) ] )"#,
    )
    .down("LShift")
    .expect_passed()
    .down("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(50)
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("LShift")
    .expect_passed()
    .advance_ms(300)
    .down("CapsLock")
    .expect_passed()
    .down("LShift")
    .expect_passed()
    .expect_nothing_fired()
    .advance_ms(50)
    .up("LShift")
    .expect_passed()
    .expect_nothing_fired()
    .up("CapsLock")
    .expect_passed()
    .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

// ---- Exact modifiers ----

#[test]
fn capslock_binding_does_not_fire_while_shift_is_held() {
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("LShift")
        .expect_passed()
        .down("CapsLock")
        .expect_passed()
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn allow_extra_modifiers_lets_capslock_fire_while_shift_is_held() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, allow_extra_modifiers: true) ] )"#,
    )
    .down("LShift")
    .expect_passed()
    .down("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn ctrl_j_does_not_fire_on_ctrl_shift_j() {
    Scenario::from_ron(r#"( bindings: [ (keys: "Ctrl+J", action: SwitchLanguage) ] )"#)
        .down("LCtrl")
        .down("LShift")
        .down("J")
        .expect_passed()
        .expect_nothing_fired()
        .up("J")
        .expect_passed()
        .up("LShift")
        .down("J")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn allow_extra_modifiers_lets_ctrl_j_fire_on_ctrl_shift_j() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+J", action: SwitchLanguage, allow_extra_modifiers: true) ] )"#,
    )
    .down("LCtrl")
    .down("LShift")
    .down("J")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn either_shift_counts_as_the_same_modifier() {
    // Exactness is per modifier family: the right Shift is not an extra
    // modifier for "LShift+F13".
    Scenario::from_ron(r#"( bindings: [ (keys: "LShift+F13", action: SwitchLanguage) ] )"#)
        .down("LShift")
        .down("RShift")
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn auto_blocker_matches_its_modifiers_exactly() {
    // The Alt+Shift blocker that `block_original_combo` adds, built
    // directly: loading it from RON would read the registry.
    Scenario::with_bindings(vec![Binding::new_auto_blocker(
        KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT]),
        BlockerMode::Swallow,
    )])
    .down("LAlt")
    .expect_passed()
    .down("LShift")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("LShift")
    .expect_suppressed()
    .up("LAlt")
    .expect_passed()
    .down("LCtrl")
    .expect_passed()
    .down("LAlt")
    .expect_passed()
    .down("LShift")
    .expect_passed()
    .expect_nothing_fired();
}

#[test]
fn shift_capslock_fires_toggle_capslock_with_either_shift() {
    // The default config's pair, without `block_original_combo`.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage),
        (keys: "Shift+CapsLock", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .down("LShift")
        .expect_passed()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("CapsLock")
        .expect_suppressed()
        .up("LShift")
        .expect_passed()
        .down("RShift")
        .expect_passed()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("CapsLock")
        .expect_suppressed()
        .up("RShift")
        .expect_passed()
        .expect_total_fired(&[BindAction::ToggleCapsLock, BindAction::ToggleCapsLock]);
}

#[test]
fn deferred_binding_does_not_arm_while_shift_is_held() {
    // The exact rule covers Tap/Hold/DoubleTap too, not only `Immediate`.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)) ] )"#,
    )
    .down("LShift")
    .expect_passed()
    .advance_ms(20)
    .down("CapsLock")
    .expect_passed()
    .expect_nothing_fired()
    .advance_ms(20)
    .up("CapsLock")
    .expect_passed()
    .expect_nothing_fired()
    .up("LShift")
    .expect_passed()
    .advance_ms(400)
    .down("CapsLock")
    .expect_suppressed()
    .advance_ms(20)
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn deferred_binding_arms_while_shift_is_held_with_allow_extra_modifiers() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true) ] )"#,
    )
    .down("LShift")
    .expect_passed()
    .advance_ms(20)
    .down("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(20)
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage]);
}

// ---- Auto-repeat follows its key-down (KS-06) ----

#[test]
fn repeat_of_a_key_whose_down_passed_is_passed_and_so_is_its_key_up() {
    // Ctrl+CapsLock reaches Windows (exact modifiers). Once Ctrl is
    // released, CapsLock's repeats match "CapsLock" exactly, but the OS saw
    // the key-down, so neither a repeat nor the key-up may be swallowed.
    // A binding that does not repeat fires nothing on a repeat, even once
    // it starts matching (KS-10).
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("LCtrl")
        .expect_passed()
        .down("CapsLock")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(500)
        .repeat("CapsLock")
        .expect_passed()
        .expect_nothing_fired()
        .up("LCtrl")
        .expect_passed()
        .advance_ms(33)
        .repeat("CapsLock")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(33)
        .repeat("CapsLock")
        .expect_passed()
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_passed();
}

#[test]
fn double_tap_lshift_held_past_the_repeat_delay_swallows_its_repeats_too() {
    // Holding the left Shift for a Shift+click: its key-down was swallowed,
    // so its repeats must be too, or the OS sees Shift go down and never
    // up. `block_default: true` keeps the swallowed key-down that a binding
    // made only of modifiers no longer asks for by default (roadmap item
    // 1.3).
    Scenario::from_ron(
        r#"( bindings: [ (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250), block_default: true) ] )"#,
    )
    .down("LShift")
    .expect_suppressed()
    .advance_ms(500)
    .repeat("LShift")
    .expect_suppressed()
    .advance_ms(33)
    .repeat("LShift")
    .expect_suppressed()
    .up("LShift")
    .expect_suppressed()
    .expect_total_fired(&[]);
}

#[test]
fn chord_keys_held_past_the_repeat_delay_swallow_their_repeats_too() {
    let config = r#"( bindings: [ (chord: (keys: ["Shift", "Space"], window_ms: 50), action: SwitchLanguage) ] )"#;
    // Shift held alone: the chord never completes.
    Scenario::from_ron(config)
        .down("LShift")
        .expect_suppressed()
        .advance_ms(500)
        .repeat("LShift")
        .expect_suppressed()
        .up("LShift")
        .expect_suppressed()
        .expect_total_fired(&[]);

    // Space held after the chord completed.
    Scenario::from_ron(config)
        .down("RShift")
        .expect_suppressed()
        .advance_ms(20)
        .down("Space")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(500)
        .repeat("Space")
        .expect_suppressed()
        .up("Space")
        .expect_suppressed()
        .up("RShift")
        .expect_suppressed();
}

#[test]
fn repeat_of_a_suppressed_key_is_suppressed_after_its_hold_fired() {
    // KS-06 with the README's CapsLock pair: the Hold fires at 200 ms and
    // the key stays down past the typematic delay.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
    ] )"#;
    Scenario::from_ron(config)
        .down("CapsLock")
        .expect_suppressed()
        .advance_ms(200)
        .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
        .advance_ms(300)
        .repeat("CapsLock")
        .expect_suppressed()
        .advance_ms(33)
        .repeat("CapsLock")
        .expect_suppressed()
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::PressKey(VK_LSHIFT)]);
}

#[test]
fn immediate_combo_that_stops_matching_mid_hold_keeps_its_key_suppressed_once() {
    // Ctrl and Alt are released while L is still held: L's repeats no
    // longer match, but its key-down was swallowed, so they stay swallowed
    // and so does its key-up, once. The next L press is plain typing.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Alt+L", action: Launch(exe: "notepad.exe")) ] )"#,
    )
    .down("Ctrl")
    .down("Alt")
    .down("L")
    .expect_suppressed()
    .expect_fired(&[launch("notepad.exe")])
    .up("Ctrl")
    .expect_passed()
    .up("Alt")
    .expect_passed()
    .advance_ms(500)
    .repeat("L")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("L")
    .expect_suppressed()
    .down("L")
    .expect_passed()
    .up("L")
    .expect_passed()
    .expect_total_fired(&[launch("notepad.exe")]);
}

#[test]
fn immediate_key_held_across_a_focus_change_keeps_its_repeats_suppressed() {
    // The focus change forgets that F13's key-up is due to be swallowed,
    // but not that its key-down was: its repeats stay swallowed, so the new
    // window gets the Left presses and no raw F13. The key-up passes, like
    // every key-up whose swallowed key-down a focus change forgot
    // (foreground_change_forgets_pending_key_up_suppression).
    Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: PressKey("Left")) ] )"#)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)])
        .foreground_changed()
        .advance_ms(500)
        .repeat("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)])
        .advance_ms(33)
        .repeat("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)])
        .up("F13")
        .expect_passed()
        .expect_nothing_fired();
}

#[test]
fn capslock_held_across_a_focus_change_does_not_reach_windows() {
    // The default config. A CapsLock repeat that reached Windows would be
    // the first CapsLock key-down it sees and would turn CapsLock on.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage),
        (keys: "Shift+CapsLock", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .foreground_changed()
        .advance_ms(500)
        .repeat("CapsLock")
        .expect_suppressed()
        .advance_ms(33)
        .repeat("CapsLock")
        .expect_suppressed()
        .up("CapsLock")
        .expect_passed();
}

#[test]
fn win_alt_t_held_across_a_focus_change_keeps_the_t_repeats_suppressed() {
    // Roadmap item 2.3's acceptance for an Immediate binding: the window
    // the binding opens takes the foreground while T is still held.
    Scenario::from_ron(r#"( bindings: [ (keys: "Win+Alt+T", action: Launch(exe: "wt.exe")) ] )"#)
        .down("Win")
        .expect_passed()
        .down("Alt")
        .expect_passed()
        .down("T")
        .expect_suppressed()
        .expect_fired(&[launch("wt.exe")])
        .foreground_changed()
        .advance_ms(500)
        .repeat("T")
        .expect_suppressed()
        .advance_ms(33)
        .repeat("T")
        .expect_suppressed()
        .up("T")
        .expect_passed();
}

#[test]
fn press_after_a_lost_key_up_reads_as_a_repeat_and_fires_only_a_repeating_binding() {
    // F13's key-up went to another desktop, so the engine still holds F13
    // and reads the next press as a repeat (KS-29). Its key-down was
    // swallowed, so the repeat is swallowed too. A binding that does not
    // repeat fires nothing on it (KS-10); its release passes, and the press
    // after it fires again. A binding that repeats fires on it. Windows
    // never saw F13 go down, so its key state cannot repair the record:
    // roadmap item 2.3's resync leaves such keys alone, and no roadmap item
    // repairs them yet.
    Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: SwitchLanguage) ] )"#)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up_unseen("F13")
        .foreground_changed()
        .advance_ms(1000)
        .down("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("F13")
        .expect_passed()
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage]);

    Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: PressKey("Left")) ] )"#)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)])
        .up_unseen("F13")
        .foreground_changed()
        .advance_ms(1000)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)]);
}

#[test]
fn repeat_of_a_key_whose_down_passed_is_passed_across_a_focus_change() {
    // A focus change forgets only swallowed key-downs. CapsLock's key-down
    // reached Windows, so its repeats and its key-up still pass once Ctrl
    // is released and "CapsLock" matches.
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("LCtrl")
        .expect_passed()
        .down("CapsLock")
        .expect_passed()
        .foreground_changed()
        .up("LCtrl")
        .expect_passed()
        .advance_ms(500)
        .repeat("CapsLock")
        .expect_passed()
        .advance_ms(33)
        .repeat("CapsLock")
        .expect_passed()
        .up("CapsLock")
        .expect_passed();
}

// ---- The OS witness at a swallowed key-up (roadmap item 2.3, ruling R1) ----

#[test]
fn a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_is_released_in_windows() {
    // CapsLock's key-up and its next key-down went where the hook does not
    // see them (a higher-integrity window in a daemon that is not elevated,
    // the secure desktop). Windows has CapsLock down; its release, which
    // the hook sees, must reach Windows although the engine swallowed the
    // last key-down it saw, or CapsLock stays down for good.
    Scenario::from_ron(DEFAULT_PAIR)
        .check_nothing_stuck()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up_unseen("CapsLock")
        .down_unseen("CapsLock")
        .up("CapsLock")
        .expect_passed()
        .expect_nothing_fired();
}

#[test]
fn a_swallowed_key_pressed_again_where_the_hook_cannot_see_it_across_focus_changes_is_released_in_windows()
 {
    // The critique's script (critique I1), with a focus change before and
    // after the unseen press.
    Scenario::from_ron(DEFAULT_PAIR)
        .check_nothing_stuck()
        .down("CapsLock")
        .expect_suppressed()
        .foreground_changed()
        .up_unseen("CapsLock")
        .down_unseen("CapsLock")
        .foreground_changed()
        .up("CapsLock")
        .expect_passed();
}

#[test]
fn a_swallowed_key_released_while_remapping_is_off_and_pressed_again_unseen_is_released_in_windows()
{
    // CapsLock was swallowed before `swch off`, released while remapping
    // was off (a lone key-up, known issue 59) and pressed again before the
    // hook saw any of it: its release after `swch on` must reach Windows.
    Scenario::from_ron(DEFAULT_PAIR)
        .check_nothing_stuck()
        .down("CapsLock")
        .expect_suppressed()
        .remapping_off()
        .up("CapsLock")
        .expect_passed()
        .remapping_on()
        .down_unseen("CapsLock")
        .up("CapsLock")
        .expect_passed();
}

// ---- Auto-repeat fires only bindings that repeat (KS-10) ----

/// Sends `count` auto-repeats of the held `key`, each one 33 ms after the
/// step before it. Every repeat is expected suppressed and to fire exactly
/// `on_repeat`.
#[track_caller]
fn repeats(mut scenario: Scenario, key: &str, count: usize, on_repeat: &[BindAction]) -> Scenario {
    for _ in 0..count {
        scenario = scenario
            .repeat(key)
            .expect_suppressed()
            .expect_fired(on_repeat)
            .advance_ms(33);
    }
    scenario
}

/// Presses `key`, holds it past the 500 ms repeat delay for `count`
/// auto-repeats 33 ms apart, and releases it. Every event is expected
/// suppressed, every repeat to fire exactly `on_repeat`, the release nothing.
/// The key-down's fire is left to the caller: `Sequence` actions compare by
/// `Arc` pointer.
#[track_caller]
fn hold_with_repeats(
    scenario: Scenario,
    key: &str,
    count: usize,
    on_repeat: &[BindAction],
) -> Scenario {
    let scenario = scenario.down(key).expect_suppressed().advance_ms(500);
    repeats(scenario, key, count, on_repeat)
        .up(key)
        .expect_suppressed()
        .expect_nothing_fired()
}

#[test]
fn repeat_true_on_switch_language_fires_on_every_repeat() {
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, repeat: true) ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck();
    hold_with_repeats(scenario, "CapsLock", 10, &[BindAction::SwitchLanguage])
        .expect_total_fired(&vec![BindAction::SwitchLanguage; 11]);
}

/// The shipped template, so that the KS-10 scenarios run on what users get.
const TEMPLATE: &str = include_str!("../config/default_config.ron");

/// [`TEMPLATE`] with `Alt+Shift` standing in for the Windows language
/// hotkey that its `block_original_combo` reads from the registry.
fn template() -> Scenario {
    Scenario::from_ron_with_language_hotkey(TEMPLATE, &[VK_MENU, VK_SHIFT])
}

#[test]
fn default_capslock_held_with_ten_repeats_switches_the_language_once() {
    // Roadmap item 1.1's acceptance: one switch, every event suppressed.
    let scenario = template().check_press_symmetry().check_nothing_stuck();
    hold_with_repeats(scenario, "CapsLock", 10, &[])
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn press_key_backspace_held_with_ten_repeats_fires_eleven_times() {
    let scenario =
        Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: PressKey("Backspace")) ] )"#)
            .check_press_symmetry()
            .check_nothing_stuck();
    hold_with_repeats(scenario, "F13", 10, &[BindAction::PressKey(VK_BACK)])
        .expect_total_fired(&vec![BindAction::PressKey(VK_BACK); 11]);
}

#[test]
fn volume_up_and_volume_down_repeat_by_default_and_play_pause_does_not() {
    for (key, repeats) in [
        (MediaKey::VolumeUp, true),
        (MediaKey::VolumeDown, true),
        (MediaKey::PlayPause, false),
    ] {
        let action = BindAction::Media(key);
        let on_repeat: &[BindAction] = if repeats {
            std::slice::from_ref(&action)
        } else {
            &[]
        };
        let total = if repeats { 4 } else { 1 };
        let scenario = Scenario::from_ron(&format!(
            r#"( bindings: [ (keys: "F13", action: Media({key:?})) ] )"#
        ))
        .with_context(format!("Media({key:?})"));
        hold_with_repeats(scenario, "F13", 3, on_repeat)
            .expect_total_fired(&vec![action.clone(); total]);
    }
}

#[test]
fn only_press_key_volume_up_and_volume_down_fire_again_on_auto_repeat_by_default() {
    // The engine never executes an action, so no process, URL or message
    // is started here.
    for (action, on_repeat) in [
        ("SwitchLanguage", None),
        ("SwitchLanguageBackward", None),
        ("ToggleCapsLock", None),
        (
            r#"PressKey("Backspace")"#,
            Some(BindAction::PressKey(VK_BACK)),
        ),
        (r#"PostMessage(msg: "WM_CLOSE")"#, None),
        (r#"Sequence([Text("x")])"#, None),
        (r#"Launch(exe: "wt.exe")"#, None),
        (r#"OpenUrl("https://example.com")"#, None),
        ("Media(PlayPause)", None),
        ("Media(Stop)", None),
        ("Media(Next)", None),
        ("Media(Previous)", None),
        ("Media(VolumeMute)", None),
        (
            "Media(VolumeUp)",
            Some(BindAction::Media(MediaKey::VolumeUp)),
        ),
        (
            "Media(VolumeDown)",
            Some(BindAction::Media(MediaKey::VolumeDown)),
        ),
        ("DoNothing", None),
    ] {
        Scenario::from_ron(&format!(
            r#"( bindings: [ (keys: "F13", action: {action}) ] )"#
        ))
        .with_context(action)
        .down("F13")
        .advance_ms(500)
        .repeat("F13")
        .expect_fired(on_repeat.as_slice());
    }
}

#[test]
fn repeat_false_on_press_key_fires_once_per_press() {
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: PressKey("Left"), repeat: false) ] )"#,
    );
    hold_with_repeats(scenario, "F13", 10, &[])
        .expect_total_fired(&[BindAction::PressKey(VK_LEFT)]);
}

#[test]
fn held_shift_capslock_toggles_capslock_once() {
    // KS-10: CapsLock flickered at the repeat rate, where the native key
    // toggles once per press.
    let scenario = template().down("LShift").expect_passed();
    hold_with_repeats(scenario, "CapsLock", 10, &[])
        .up("LShift")
        .expect_passed()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn held_launch_and_sequence_hotkeys_fire_once_per_press() {
    // KS-10: a held Launch hotkey started a process per repeat, and a held
    // Sequence macro a worker thread per repeat.
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Alt+T", action: Launch(exe: "wt.exe")) ] )"#,
    )
    .down("LCtrl")
    .expect_passed()
    .down("LAlt")
    .expect_passed();
    hold_with_repeats(scenario, "T", 5, &[]).expect_total_fired(&[launch("wt.exe")]);

    // `Sequence` actions compare by `Arc` pointer, so only the repeats are
    // checked.
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Alt+N", action: Sequence([Text("note")])) ] )"#,
    )
    .down("LCtrl")
    .expect_passed()
    .down("LAlt")
    .expect_passed();
    hold_with_repeats(scenario, "N", 5, &[]);
}

#[test]
fn non_repeating_winner_keeps_the_immediate_slot_on_a_repeat() {
    // The first matching Immediate binding takes a repeat even though it
    // does not fire on it, so the binding after it never fires in its
    // place: no Left at the repeat rate after one switch.
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "F13", action: SwitchLanguage),
            (keys: "F13", action: PressKey("Left")),
        ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .advance_ms(500)
    .repeat("F13")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("F13")
    .expect_suppressed();
}

#[test]
fn capslock_held_across_a_focus_change_switches_the_language_once() {
    // The repeats stay swallowed after the focus change and fire nothing;
    // the release passes (foreground_change_forgets_pending_key_up_suppression).
    let scenario = template()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .foreground_changed()
        .advance_ms(500);
    repeats(scenario, "CapsLock", 3, &[])
        .up("CapsLock")
        .expect_passed()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn capslock_held_through_remapping_off_and_on_switches_the_language_once() {
    // The repeat sent while remapping is off reaches Windows, so the reset
    // of `swch on` takes CapsLock as held and passed: its later repeats
    // pass and, as repeats, fire nothing.
    template()
        .check_nothing_stuck()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .remapping_off()
        .advance_ms(500)
        .repeat("CapsLock")
        .expect_passed()
        .expect_nothing_fired()
        .remapping_on()
        .advance_ms(33)
        .repeat("CapsLock")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(33)
        .repeat("CapsLock")
        .expect_passed()
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_passed()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn reload_mid_hold_applies_the_new_repeat_flag_from_the_next_repeat() {
    // A hot reload leaves the engine alone, so the press goes on and the
    // next repeat reads the new binding's flag.
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(500)
        .repeat("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .reload(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, repeat: true) ] )"#)
        .advance_ms(33)
        .repeat("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("CapsLock")
        .expect_suppressed();
}

#[test]
fn deferred_sequence_and_chord_bindings_never_fire_on_auto_repeat() {
    // Every binding presses Backspace, which repeats by default: the flag
    // must not reach the deferred, sequence and chord arms.
    let back = BindAction::PressKey(VK_BACK);

    // Tap: held past its term, it fires nothing at all.
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: PressKey("Backspace"), trigger: Tap(200)) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .advance_ms(500);
    repeats(scenario, "F13", 3, &[])
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_total_fired(&[]);

    // Hold: fires once at its term, not again on the repeats.
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: PressKey("Backspace"), trigger: Hold(200)) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .advance_ms(500)
    .expect_fired(std::slice::from_ref(&back));
    repeats(scenario, "F13", 3, &[])
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_total_fired(std::slice::from_ref(&back));

    // DoubleTap: a repeat inside the term is not a second press.
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: PressKey("Backspace"), trigger: DoubleTap(600)) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .advance_ms(300);
    repeats(scenario, "F13", 3, &[])
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_total_fired(&[]);

    // Sequence: the repeats of neither step advance or complete it again.
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["F13", "F14"], max_gap_ms: 2000), action: PressKey("Backspace")) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .advance_ms(500);
    let scenario = repeats(scenario, "F13", 3, &[])
        .up("F13")
        .expect_suppressed()
        .down("F14")
        .expect_suppressed()
        .expect_fired(std::slice::from_ref(&back))
        .advance_ms(500);
    repeats(scenario, "F14", 3, &[])
        .up("F14")
        .expect_suppressed()
        .expect_total_fired(std::slice::from_ref(&back));

    // Chord: the repeats of its last key fire nothing after it completed.
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["F13", "F14"], window_ms: 50), action: PressKey("Backspace")) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .advance_ms(20)
    .down("F14")
    .expect_suppressed()
    .expect_fired(std::slice::from_ref(&back))
    .advance_ms(500);
    repeats(scenario, "F14", 3, &[])
        .up("F14")
        .expect_suppressed()
        .up("F13")
        .expect_suppressed()
        .expect_total_fired(std::slice::from_ref(&back));
}

#[test]
fn held_hold_binding_with_repeating_action_fires_once_across_many_repeats() {
    // Backspace repeats by default. If the flag leaked into the Hold arm, the
    // first repeat (+500 ms) would re-arm a 200 ms term that fires at +700 ms,
    // so the key has to stay down past that instant for the leak to show.
    let back = BindAction::PressKey(VK_BACK);
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: PressKey("Backspace"), trigger: Hold(200)) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .advance_ms(500)
    .expect_fired(std::slice::from_ref(&back));
    // Ten repeats 33 ms apart: the last one lands at +797 ms, the release after
    // +830 ms, well past the first repeat plus the 200 ms term.
    repeats(scenario, "F13", 10, &[])
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_total_fired(std::slice::from_ref(&back));
}

#[test]
fn a_repeat_of_a_sequence_step_does_not_count_as_the_next_step() {
    // Both steps are the same key, so a repeat that reached the sequence
    // would complete it from a single physical press (KS-10, KS-06).
    let back = BindAction::PressKey(VK_BACK);
    let scenario = Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["F13", "F13"], max_gap_ms: 2000), action: PressKey("Backspace")) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(500);
    repeats(scenario, "F13", 3, &[])
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_total_fired(&[])
        .advance_ms(100)
        .down("F13")
        .expect_suppressed()
        .expect_fired(std::slice::from_ref(&back));
}

#[test]
fn repeats_of_both_chord_keys_do_not_complete_the_chord_again() {
    // After the chord fired, a repeat of each key inside the window would
    // complete a fresh pending entry if repeats reached the chord table.
    let back = BindAction::PressKey(VK_BACK);
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["F13", "F14"], window_ms: 50), action: PressKey("Backspace")) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(10)
    .down("F14")
    .expect_suppressed()
    .expect_fired(std::slice::from_ref(&back))
    .advance_ms(5)
    .repeat("F13")
    .expect_suppressed()
    .expect_nothing_fired()
    .repeat("F14")
    .expect_suppressed()
    .expect_nothing_fired()
    .expect_total_fired(std::slice::from_ref(&back));
}

// ---- Suppression symmetry (KS-06, KS-14) ----

#[test]
fn alt_shift_x_fires_after_the_auto_blocker_swallowed_the_second_modifier() {
    // KS-14 case 1. The blocker swallows the second modifier's key-down, so
    // Windows never sees it go down. The resync on X's key-down must not
    // read that modifier as released, or `Alt+Shift+X` never matches.
    let bindings = || {
        with_alt_shift_blocker(r#"( bindings: [ (keys: "Alt+Shift+X", action: ToggleCapsLock) ] )"#)
    };
    Scenario::with_bindings(bindings())
        .check_nothing_stuck()
        .down("LAlt")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .down("X")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("X")
        .expect_suppressed()
        .up("LShift")
        .expect_suppressed()
        .up("LAlt")
        .expect_passed();

    Scenario::with_bindings(bindings())
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .down("LAlt")
        .expect_suppressed()
        .expect_nothing_fired()
        .down("X")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("X")
        .expect_suppressed()
        .up("LAlt")
        .expect_suppressed()
        .up("LShift")
        .expect_passed();
}

#[test]
fn shift_held_by_a_double_tap_still_counts_for_shift_capslock() {
    // KS-14 case 2. The DoubleTap swallows Shift's key-down; the resync on
    // CapsLock's key-down must not drop Shift, or "CapsLock" matches
    // instead of "Shift+CapsLock". The defect needs a swallowed Shift, hence
    // `block_default: true` (roadmap item 1.3).
    let config = r#"( bindings: [
        (keys: "Shift", action: SwitchLanguageBackward, trigger: DoubleTap(300), block_default: true),
        (keys: "CapsLock", action: SwitchLanguage),
        (keys: "Shift+CapsLock", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .check_nothing_stuck()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(50)
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("CapsLock")
        .expect_suppressed()
        .up("LShift")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn one_long_press_of_shift_does_not_fire_its_double_tap() {
    // KS-14 case 3. A's key-up during rollover used to resync the swallowed
    // Shift as released, so Shift's first auto-repeat read as a second press
    // inside the 600 ms term. The defect needs a swallowed Shift, hence
    // `block_default: true` (roadmap item 1.3).
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600), block_default: true) ] )"#,
    )
    .check_nothing_stuck()
    .down("A")
    .expect_passed()
    .advance_ms(10)
    .down("LShift")
    .expect_suppressed()
    .advance_ms(10)
    .up("A")
    .expect_passed()
    .advance_ms(480)
    .repeat("LShift")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(33)
    .repeat("LShift")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("LShift")
    .expect_suppressed()
    .expect_total_fired(&[]);
}

#[test]
fn ctrl_shift_hold_with_ctrl_released_first_neither_leaks_nor_sticks_shift() {
    // Ctrl is released while the Hold on Shift is pending. Shift's key-down
    // was swallowed, so its repeats and its key-up are too; its repeats
    // used to read as a fresh press that reached Windows, and the pending
    // Hold then swallowed the key-up, leaving Shift held there. The defect
    // needs a swallowed Shift, hence `block_default: true` (roadmap item
    // 1.3).
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000), block_default: true) ] )"#,
    )
    .check_nothing_stuck()
    .down("LCtrl")
    .expect_passed()
    .advance_ms(20)
    .down("LShift")
    .expect_suppressed()
    .advance_ms(100)
    .up("LCtrl")
    .expect_passed()
    .advance_ms(380)
    .repeat("LShift")
    .expect_suppressed()
    .advance_ms(33)
    .repeat("LShift")
    .expect_suppressed()
    .advance_ms(67)
    .up("LShift")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(1000)
    .expect_nothing_fired()
    .expect_total_fired(&[]);
}

#[test]
fn shift_held_after_the_auto_blocker_and_alt_released_stays_swallowed_for_its_whole_press() {
    // The audit's KS-06 auto-blocker route, with the default config. The
    // blocker swallowed Shift's key-down, so its repeats are swallowed even
    // after Alt is released, and its key-up exactly once; the next Shift
    // press reaches Windows on both edges.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .down("LAlt")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("LAlt")
        .expect_passed()
        .advance_ms(500)
        .repeat("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(33)
        .repeat("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("LShift")
        .expect_suppressed()
        .advance_ms(100)
        .down("LShift")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn key_up_of_a_suppressed_press_is_suppressed_only_once() {
    // "Its key-up is suppressed exactly once": the swallowed key-up spends
    // the record. A later key-up of the same key, here for a press the hook
    // never saw (remapping was off), is a different press that Windows has
    // down, so it must reach Windows or Shift stays held there.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .down("LAlt")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .up("LShift")
        .expect_suppressed()
        .up("LAlt")
        .expect_passed()
        .advance_ms(1000)
        .down_unseen("LShift")
        .up("LShift")
        .expect_passed()
        .expect_nothing_stuck();
}

#[test]
fn suppressed_modifier_stays_an_auto_repeat_across_a_focus_change() {
    // The focus change forgets that Shift's key-down was swallowed, but
    // Windows still never saw it go down, so the resync leaves Shift held
    // and its repeat is not a second press for the DoubleTap. Whether that
    // repeat and the key-up reach Windows is roadmap item 2.3's policy.
    // `block_default: true` keeps Shift's key-down swallowed (roadmap item
    // 1.3).
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600), block_default: true) ] )"#,
    )
    .check_nothing_stuck()
    .down("A")
    .down("LShift")
    .expect_suppressed()
    .foreground_changed()
    .up("A")
    .expect_passed()
    .advance_ms(500)
    .repeat("LShift")
    .expect_nothing_fired()
    .up("LShift")
    .expect_total_fired(&[]);
}

#[test]
fn chord_keys_held_across_a_focus_change_keep_their_repeats_suppressed() {
    // The chord opens a window that takes the foreground while J and K are
    // still held. K, pressed last, auto-repeats. Its key-down was swallowed,
    // so Windows must not see its repeats either (no "kkk" in the new
    // window). The key-ups pass, like every key-up whose swallowed key-down
    // a focus change forgot.
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 50), action: Launch(exe: "wt.exe")) ] )"#,
    )
    .check_nothing_stuck()
    .down("J")
    .expect_suppressed()
    .advance_ms(20)
    .down("K")
    .expect_suppressed()
    .expect_fired(&[launch("wt.exe")])
    .foreground_changed()
    .advance_ms(480)
    .repeat("K")
    .expect_suppressed()
    .advance_ms(33)
    .repeat("K")
    .expect_suppressed()
    .up("J")
    .expect_passed()
    .up("K")
    .expect_passed()
    .expect_total_fired(&[launch("wt.exe")]);
}

#[test]
fn deferred_key_held_across_a_focus_change_keeps_its_repeats_suppressed() {
    // Roadmap item 2.3's example: the Hold opens a terminal, which takes the
    // foreground while CapsLock is still held. A CapsLock repeat that reached
    // Windows would be the first CapsLock key-down it sees and would turn
    // CapsLock on in the terminal.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: Launch(exe: "wt.exe"), trigger: Hold(300)) ] )"#,
    )
    .check_nothing_stuck()
    .down("CapsLock")
    .expect_suppressed()
    .advance_ms(300)
    .expect_fired(&[launch("wt.exe")])
    .foreground_changed()
    .advance_ms(200)
    .repeat("CapsLock")
    .expect_suppressed()
    .advance_ms(33)
    .repeat("CapsLock")
    .expect_suppressed()
    .up("CapsLock")
    .expect_passed()
    .expect_total_fired(&[launch("wt.exe")]);

    // A Tap still pending at the key-up: the key-up stays swallowed with it.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(2000)) ] )"#,
    )
    .check_nothing_stuck()
    .down("CapsLock")
    .expect_suppressed()
    .foreground_changed()
    .advance_ms(500)
    .repeat("CapsLock")
    .expect_suppressed()
    .advance_ms(100)
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn immediate_combo_that_stops_matching_after_a_focus_change_keeps_its_key_suppressed() {
    // Ctrl+F13 opens a window and Ctrl is released while F13 is still
    // held. "Ctrl+F13" no longer matches F13's repeats, but its key-down was
    // swallowed, so they stay swallowed.
    Scenario::from_ron(r#"( bindings: [ (keys: "Ctrl+F13", action: Launch(exe: "wt.exe")) ] )"#)
        .check_nothing_stuck()
        .down("LCtrl")
        .expect_passed()
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[launch("wt.exe")])
        .foreground_changed()
        .up("LCtrl")
        .expect_passed()
        .advance_ms(500)
        .repeat("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(33)
        .repeat("F13")
        .expect_suppressed()
        .up("F13")
        .expect_passed()
        .expect_total_fired(&[launch("wt.exe")]);
}

#[test]
fn sequence_completion_keeps_the_key_up_of_a_key_whose_down_passed() {
    // KS-32's example. F14 alone matches nothing, so its key-down reaches
    // Windows; the completing step must not swallow its key-up.
    Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["F14+F15", "F14+F16"]), action: SwitchLanguage) ] )"#,
    )
    .check_nothing_stuck()
    .down("F14")
    .expect_passed()
    .down("F15")
    .expect_suppressed()
    .up("F15")
    .expect_suppressed()
    .down("F16")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("F16")
    .expect_suppressed()
    .up("F14")
    .expect_passed();
}

#[test]
fn chord_completion_keeps_the_key_up_of_a_chord_key_whose_down_passed() {
    // J's first press starts the chord in the Docs tab. The title changes
    // without a foreground event, so J's second press reaches Windows (the
    // chord's `when:` no longer holds); the title changes back and K
    // completes the chord inside its window. J's second key-down reached
    // Windows, so its key-up must too.
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 2000), action: SwitchLanguage, when: TitleContains("Docs")) ] )"#,
    )
    .foreground("browser.exe", "Docs - Browser")
    .check_nothing_stuck()
    .down("J")
    .expect_suppressed()
    .up("J")
    .expect_suppressed()
    .foreground("browser.exe", "Mail - Browser")
    .down("J")
    .expect_passed()
    .foreground("browser.exe", "Docs - Browser")
    .down("K")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .up("K")
    .expect_suppressed()
    .up("J")
    .expect_passed();
}

#[test]
fn modifier_held_before_the_engine_saw_it_keeps_passing_once_a_repeat_passed() {
    // Ctrl, Alt, A and then Shift went down while remapping was off, so
    // Windows has all four down and Shift, pressed last, auto-repeats. A's
    // key-up makes the engine resync the modifiers, which finds Shift held
    // with no record of its key-down. Shift's next repeat matches nothing
    // (Ctrl is held too) and reaches Windows; from then on the press is one
    // Windows saw, so once Ctrl is released, a repeat that the Alt+Shift
    // blocker now matches still passes.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .down_unseen("LCtrl")
        .down_unseen("LAlt")
        .down_unseen("A")
        .down_unseen("LShift")
        .up("A")
        .expect_passed()
        .advance_ms(500)
        .repeat("LShift")
        .expect_passed()
        .expect_nothing_fired()
        .up("LCtrl")
        .expect_passed()
        .advance_ms(33)
        .repeat("LShift")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .up("LAlt")
        .expect_passed();
}

// ---- Auto-blockers (KS-05, KS-18) ----

#[test]
fn user_binding_on_the_language_hotkey_fires_in_either_file_order() {
    // The audit's KS-18 repro: a binding on the hotkey's own keys, above or
    // below the flagged one or the flagged one itself, lost its fire to the
    // auto-blocker.
    let configs = [
        (
            "flagged binding above",
            r#"( bindings: [
                (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
                (keys: "Alt+Shift", action: SwitchLanguageBackward),
            ] )"#,
        ),
        (
            "flagged binding below",
            r#"( bindings: [
                (keys: "Alt+Shift", action: SwitchLanguageBackward),
                (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
            ] )"#,
        ),
        (
            "flagged binding on the hotkey",
            r#"( bindings: [
                (keys: "Alt+Shift", action: SwitchLanguageBackward, block_original_combo: true),
            ] )"#,
        ),
    ];
    for (name, config) in configs {
        Scenario::from_ron_with_language_hotkey(config, &[VK_MENU, VK_SHIFT])
            .with_context(name)
            .check_nothing_stuck()
            .down("LAlt")
            .expect_passed()
            .expect_nothing_fired()
            .down("LShift")
            .expect_suppressed()
            .expect_fired(&[BindAction::SwitchLanguageBackward])
            .up("LShift")
            .expect_suppressed()
            .up("LAlt")
            .expect_passed();
    }
}

/// The default config's two bindings, with `block_original_combo`.
const DEFAULT_CONFIG: &str = r#"( bindings: [
    (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
    (keys: "Shift+CapsLock", action: ToggleCapsLock),
] )"#;

#[test]
fn shift_pressed_while_alt_and_tab_are_held_reaches_windows() {
    // KS-05: Windows sees Alt, Tab and Shift, not its language hotkey, so
    // Shift reaches it and the Alt+Tab switcher steps back.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LAlt")
        .expect_passed()
        .down("Tab")
        .expect_passed()
        .advance_ms(500)
        .repeat("Tab")
        .expect_passed()
        .down("LShift")
        .expect_passed()
        .advance_ms(500)
        .repeat("LShift")
        .expect_passed()
        .up("Tab")
        .expect_passed()
        .down("Tab")
        .expect_passed()
        .up("Tab")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .up("LAlt")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn shift_pressed_while_ctrl_and_t_are_held_reaches_windows() {
    // KS-05 with the Ctrl+Shift hotkey, on both sides.
    Scenario::from_ron_with_language_hotkey(DEFAULT_CONFIG, &[VK_CONTROL, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LCtrl")
        .expect_passed()
        .down("T")
        .expect_passed()
        .down("LShift")
        .expect_passed()
        .up("T")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .up("LCtrl")
        .expect_passed()
        .down("RCtrl")
        .expect_passed()
        .down("T")
        .expect_passed()
        .down("RShift")
        .expect_passed()
        .up("T")
        .expect_passed()
        .up("RShift")
        .expect_passed()
        .up("RCtrl")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn alt_shift_pressed_while_a_letter_is_held_reaches_windows() {
    // Typing rollover: A is still down in Windows when Alt+Shift completes.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("A")
        .expect_passed()
        .down("LAlt")
        .expect_passed()
        .down("LShift")
        .expect_passed()
        .up("A")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .up("LAlt")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn grave_blocker_acts_only_on_a_lone_grave() {
    // The "`" hotkey (registry value "4"): a grave typed with another key or
    // with Shift held reaches Windows, a grave on its own does not.
    Scenario::from_ron_with_language_hotkey(DEFAULT_CONFIG, &[VK_OEM_3])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("A")
        .expect_passed()
        .down("`")
        .expect_passed()
        .up("`")
        .expect_passed()
        .up("A")
        .expect_passed()
        .down("LShift")
        .expect_passed()
        .down("`")
        .expect_passed()
        .up("`")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .down("`")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("`")
        .expect_suppressed()
        .expect_total_fired(&[]);
}

#[test]
fn ctrl_shift_blocker_swallows_a_bare_shift_and_fires_nothing() {
    // Pressed in the order Ctrl, Shift, T, Windows would see a bare
    // Ctrl+Shift when Shift goes down: with `true` the blocker swallows
    // Shift, and T reaches Windows as Ctrl+T. Neither Alt nor Win is down,
    // so nothing is masked. `MaskOnly` lets Shift through
    // (mask_only_blocker_lets_ctrl_shift_t_reach_the_application).
    Scenario::from_ron_with_language_hotkey(DEFAULT_CONFIG, &[VK_CONTROL, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LCtrl")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_unmasked()
        .down("T")
        .expect_passed()
        .up("T")
        .expect_passed()
        .up("LShift")
        .expect_suppressed()
        .up("LCtrl")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn shift_pressed_after_tab_was_released_is_still_swallowed() {
    // The usual way back in the Alt+Tab switcher: Alt held, Tab tapped,
    // then Shift and Tab. Windows would see a bare Alt+Shift when Shift goes
    // down, so with `true` the blocker swallows it, with the menu mask key
    // since Alt is down in Windows, and Tab reaches Windows as Alt+Tab.
    // `MaskOnly` lets Shift through.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LAlt")
        .expect_passed()
        .down("Tab")
        .expect_passed()
        .up("Tab")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_masked()
        .down("Tab")
        .expect_passed()
        .up("Tab")
        .expect_passed()
        .up("LShift")
        .expect_suppressed()
        .up("LAlt")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn user_binding_on_alt_shift_fires_with_the_blocker_in_either_order() {
    // A blocker next to a binding on its own keys, wherever it sits in the
    // list, never takes that binding's fire.
    const USER: &str = r#"( bindings: [ (keys: "Alt+Shift", action: SwitchLanguageBackward) ] )"#;
    let mut blocker_last = crate::config::from_ron_str(USER).expect("config loads");
    blocker_last.push(Binding::new_auto_blocker(
        KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT]),
        BlockerMode::Swallow,
    ));
    for (name, bindings) in [
        ("blocker first", with_alt_shift_blocker(USER)),
        ("blocker last", blocker_last),
    ] {
        Scenario::with_bindings(bindings)
            .with_context(name)
            .check_nothing_stuck()
            .check_press_symmetry()
            .down("LAlt")
            .expect_passed()
            .expect_nothing_fired()
            .down("LShift")
            .expect_suppressed()
            .expect_fired(&[BindAction::SwitchLanguageBackward])
            .up("LShift")
            .expect_suppressed()
            .up("LAlt")
            .expect_passed();
    }
}

#[test]
fn sided_user_binding_fires_and_the_blocker_keeps_the_other_sides() {
    // `LAlt+LShift` is not the hotkey's keys, so the loader keeps the
    // blocker; it swallows right Alt+Shift and leaves the left to the
    // binding.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
        (keys: "LAlt+LShift", action: SwitchLanguageBackward),
    ] )"#;
    Scenario::from_ron_with_language_hotkey(config, &[VK_MENU, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LAlt")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguageBackward])
        .up("LShift")
        .expect_suppressed()
        .up("LAlt")
        .expect_passed()
        .down("RAlt")
        .expect_passed()
        .down("RShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("RShift")
        .expect_suppressed()
        .up("RAlt")
        .expect_passed()
        .expect_total_fired(&[BindAction::SwitchLanguageBackward]);
}

#[test]
fn key_whose_release_the_hook_missed_does_not_disable_the_blocker() {
    // Win+L: the key-ups go to the secure desktop, so the engine still
    // holds L. Windows does not, and would see a bare Alt+Shift.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LWin")
        .expect_passed()
        .down("L")
        .expect_passed()
        .up_unseen("L")
        .up_unseen("LWin")
        .down("LAlt")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("LShift")
        .expect_suppressed()
        .up("LAlt")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn key_swallowed_by_a_binding_does_not_disable_the_blocker() {
    // CapsLock's key-down was swallowed, so Windows does not have it down
    // and would see a bare Alt+Shift.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .down("LAlt")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("LShift")
        .expect_suppressed()
        .up("LAlt")
        .expect_passed()
        .up("CapsLock")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn modifier_swallowed_by_a_binding_does_not_disable_the_blocker() {
    // The Tap swallowed Win's key-down, so Windows has only Alt and Shift
    // down: its language hotkey. A Win that reached Windows would keep the
    // blocker off, like the Ctrl in auto_blocker_matches_its_modifiers_exactly.
    // `block_default: true` keeps Win's key-down swallowed (roadmap item
    // 1.3).
    Scenario::with_bindings(with_alt_shift_blocker(
        r#"( bindings: [ (keys: "Win", action: ToggleCapsLock, trigger: Tap(200), block_default: true) ] )"#,
    ))
    .check_nothing_stuck()
    .check_press_symmetry()
    .down("LWin")
    .expect_suppressed()
    .expect_nothing_fired()
    .down("LAlt")
    .expect_passed()
    .down("LShift")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("LShift")
    .expect_suppressed()
    .up("LAlt")
    .expect_passed()
    .up("LWin")
    .expect_suppressed()
    .expect_fired(&[BindAction::ToggleCapsLock]);
}

// ---- Remapping off and on (KS-01, KS-31) ----

const HOLD_F13: &str =
    r#"( bindings: [ (keys: "F13", action: SwitchLanguage, trigger: Hold(200)) ] )"#;

const TAP_F13: &str =
    r#"( bindings: [ (keys: "F13", action: SwitchLanguage, trigger: Tap(200)) ] )"#;

#[test]
fn remapping_off_drops_a_pending_hold() {
    // KS-31: the Hold's timer still runs while remapping is off and must
    // find nothing to fire.
    Scenario::from_ron(HOLD_F13)
        .down("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(50)
        .remapping_off()
        .advance_ms(300)
        .expect_nothing_fired()
        .up("F13")
        .expect_passed()
        .remapping_on()
        .down("F13")
        .expect_suppressed()
        .advance_ms(199)
        .expect_nothing_fired()
        .advance_ms(1)
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn tap_swallowed_before_remapping_off_works_again_after_remapping_on() {
    // Without the reset the second press would read as an auto-repeat of
    // the first one, whose key-up the engine never saw.
    Scenario::from_ron(TAP_F13)
        .down("F13")
        .expect_suppressed()
        .advance_ms(20)
        .remapping_off()
        // Windows gets a lone key-up: remapping off passes every event.
        .up("F13")
        .expect_passed()
        .remapping_on()
        .advance_ms(1000)
        .down("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(50)
        .up("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn remapping_off_drops_a_pending_chord() {
    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["J", "K"], window_ms: 50), action: SwitchLanguage) ] )"#,
    )
    .down("J")
    .expect_suppressed()
    .remapping_off()
    .up("J")
    .expect_passed()
    .remapping_on()
    .advance_ms(10)
    .down("K")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("K")
    .expect_suppressed()
    .expect_total_fired(&[]);
}

#[test]
fn remapping_off_drops_a_pending_sequence() {
    Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["F13", "F14"]), action: SwitchLanguage) ] )"#,
    )
    .down("F13")
    .expect_suppressed()
    .up("F13")
    .expect_suppressed()
    .remapping_off()
    .remapping_on()
    .down("F14")
    .expect_passed()
    .expect_nothing_fired()
    .up("F14")
    .expect_passed()
    .expect_total_fired(&[]);
}

#[test]
fn stale_hold_timer_from_before_remapping_off_does_not_fire_a_newer_press() {
    // KS-11 across a reset: the first press's timer falls due at +200 while
    // the second press, armed at +100, is due at +300. A reset that restarts
    // the Hold generations lets the old timer fire the new Hold early.
    Scenario::from_ron(HOLD_F13)
        .down("F13")
        .advance_ms(50)
        .remapping_off()
        .up("F13")
        .remapping_on()
        .advance_ms(50)
        .down("F13")
        .expect_suppressed()
        .advance_ms(150)
        .expect_nothing_fired()
        .advance_ms(50)
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn key_held_through_remapping_on_keeps_its_repeats_and_its_release() {
    // Windows saw the key-down while remapping was off, so the key's
    // repeats and its key-up must reach it too, or the key stays held there.
    Scenario::from_ron(TAP_F13)
        .check_nothing_stuck()
        .remapping_off()
        .down("F13")
        .expect_passed()
        .remapping_on()
        .advance_ms(500)
        .repeat("F13")
        .expect_passed()
        .expect_nothing_fired()
        .up("F13")
        .expect_passed()
        .expect_nothing_fired()
        .down("F13")
        .expect_suppressed()
        .advance_ms(50)
        .up("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage]);
}

// ---- block_default on deferred triggers (roadmap item 1.3) ----

/// The template's commented example (`default_config.ron`). It is made only
/// of modifiers, so it lets its key through unless it sets `block_default`.
const LSHIFT_DOUBLE_TAP: &str =
    r#"( bindings: [ (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250)) ] )"#;

const LSHIFT_DOUBLE_TAP_BLOCKING: &str = r#"( bindings: [ (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250), block_default: true) ] )"#;

const PASSING_TAP_F13: &str = r#"( bindings: [ (keys: "F13", action: SwitchLanguage, trigger: Tap(200), block_default: false) ] )"#;

const PASSING_HOLD_F13: &str = r#"( bindings: [ (keys: "F13", action: SwitchLanguage, trigger: Hold(200), block_default: false) ] )"#;

#[test]
fn explicit_block_default_false_on_a_tap_passes_every_event_and_still_fires() {
    Scenario::from_ron(PASSING_TAP_F13)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("F13")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(80)
        .up("F13")
        .expect_passed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .down("F13")
        .expect_passed()
        .advance_ms(500)
        .repeat("F13")
        .expect_passed()
        .up("F13")
        .expect_passed()
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn explicit_block_default_false_on_a_hold_passes_every_event_and_still_fires() {
    Scenario::from_ron(PASSING_HOLD_F13)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("F13")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(200)
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(300)
        .repeat("F13")
        .expect_passed()
        .expect_nothing_fired()
        .up("F13")
        .expect_passed()
        .expect_nothing_fired()
        .down("F13")
        .expect_passed()
        .advance_ms(100)
        .up("F13")
        .expect_passed()
        .advance_ms(300)
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn explicit_block_default_false_on_a_double_tap_passes_every_event_and_still_fires() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(250), block_default: false) ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("F13")
    .expect_passed()
    .expect_nothing_fired()
    .advance_ms(60)
    .up("F13")
    .expect_passed()
    .advance_ms(100)
    .down("F13")
    .expect_passed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("F13")
    .expect_passed()
    .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn lshift_double_tap_fires_and_every_lshift_event_passes() {
    // Roadmap item 1.3's acceptance: a double LShift fires ToggleCapsLock,
    // and Windows sees every LShift event.
    Scenario::from_ron(LSHIFT_DOUBLE_TAP)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(60)
        .up("LShift")
        .expect_passed()
        .advance_ms(100)
        .down("LShift")
        .expect_passed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .advance_ms(60)
        .up("LShift")
        .expect_passed()
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn lshift_held_with_its_double_tap_bound_types_a_capital_a() {
    // Roadmap item 1.3's acceptance: with LShift held, A reaches Windows
    // while Windows has Shift down, and nothing is swallowed.
    Scenario::from_ron(LSHIFT_DOUBLE_TAP)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .advance_ms(30)
        .down("A")
        .expect_passed()
        .expect_nothing_fired()
        .expect_down_in_windows("LShift")
        .advance_ms(30)
        .up("A")
        .expect_passed()
        .advance_ms(30)
        .up("LShift")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn capslock_tap_with_the_default_block_default_still_swallows_capslock() {
    // Roadmap item 1.3's acceptance: a combo with a key that is not a
    // modifier keeps swallowing its key by default.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)) ] )"#,
    )
    .check_press_symmetry()
    .down("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .advance_ms(80)
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .down("CapsLock")
    .expect_suppressed()
    .advance_ms(500)
    .repeat("CapsLock")
    .expect_suppressed()
    .up("CapsLock")
    .expect_suppressed()
    .expect_nothing_fired()
    .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn explicit_block_default_true_on_a_modifier_only_double_tap_swallows() {
    // Roadmap item 1.3's acceptance: an explicit value wins over the
    // default of a binding made only of modifiers.
    Scenario::from_ron(LSHIFT_DOUBLE_TAP_BLOCKING)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(60)
        .up("LShift")
        .expect_suppressed()
        .advance_ms(100)
        .down("LShift")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("LShift")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn passing_lshift_double_tap_held_past_the_repeat_delay_passes_its_repeats_too() {
    // The pass-path twin of
    // double_tap_lshift_held_past_the_repeat_delay_swallows_its_repeats_too:
    // Windows saw Shift go down, so it sees its repeats and its release.
    Scenario::from_ron(LSHIFT_DOUBLE_TAP)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .advance_ms(500)
        .repeat("LShift")
        .expect_passed()
        .advance_ms(33)
        .repeat("LShift")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn one_long_press_of_a_passing_shift_does_not_fire_its_double_tap() {
    // KS-14 case 3 on the pass path: Shift reached Windows, so the resync on
    // A's key-up finds it held, and its auto-repeats stay auto-repeats.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600)) ] )"#,
    )
    .check_nothing_stuck()
    .down("A")
    .expect_passed()
    .advance_ms(10)
    .down("LShift")
    .expect_passed()
    .advance_ms(10)
    .up("A")
    .expect_passed()
    .advance_ms(480)
    .repeat("LShift")
    .expect_passed()
    .expect_nothing_fired()
    .advance_ms(33)
    .repeat("LShift")
    .expect_passed()
    .expect_nothing_fired()
    .up("LShift")
    .expect_passed()
    .expect_total_fired(&[]);
}

#[test]
fn modifier_only_hold_and_tap_pass_their_keys_by_default() {
    let config = r#"( bindings: [
        (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)),
        (keys: "RShift", action: ToggleCapsLock, trigger: Tap(300)),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LCtrl")
        .expect_passed()
        .advance_ms(20)
        .down("LShift")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(1000)
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("LShift")
        .expect_passed()
        .up("LCtrl")
        .expect_passed()
        .advance_ms(100)
        .down("RShift")
        .expect_passed()
        .advance_ms(50)
        .up("RShift")
        .expect_passed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .expect_total_fired(&[BindAction::SwitchLanguage, BindAction::ToggleCapsLock]);
}

#[test]
fn deferred_binding_on_the_language_hotkey_keys_lets_the_hotkey_through_by_default() {
    // A binding on exactly the hotkey's keys replaces the auto-blocker
    // (KS-18), and its own block_default decides whether Windows sees the
    // hotkey. Made only of modifiers, it lets Windows switch the layout
    // too, also when the flag that asked for the blocker is its own.
    let hotkey = &[VK_MENU, VK_SHIFT];
    let passing = [
        r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
            (keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200)),
        ] )"#,
        r#"( bindings: [
            (keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200), block_original_combo: true),
        ] )"#,
    ];
    for config in passing {
        Scenario::from_ron_with_language_hotkey(config, hotkey)
            .with_context(config)
            .check_press_symmetry()
            .check_nothing_stuck()
            .down("LAlt")
            .expect_passed()
            .advance_ms(20)
            .down("LShift")
            .expect_passed()
            .expect_nothing_fired()
            .advance_ms(50)
            .up("LShift")
            .expect_passed()
            .expect_fired(&[BindAction::SwitchLanguageBackward])
            .up("LAlt")
            .expect_passed()
            .expect_total_fired(&[BindAction::SwitchLanguageBackward]);
    }

    // `block_default: true` keeps the hotkey from Windows, as before.
    let blocking = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true),
        (keys: "Alt+Shift", action: SwitchLanguageBackward, trigger: Tap(200), block_default: true),
    ] )"#;
    Scenario::from_ron_with_language_hotkey(blocking, hotkey)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LAlt")
        .expect_passed()
        .advance_ms(20)
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(50)
        .up("LShift")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguageBackward])
        .up("LAlt")
        .expect_passed()
        .expect_total_fired(&[BindAction::SwitchLanguageBackward]);
}

#[test]
fn passing_and_blocking_deferred_bindings_on_one_key_keep_its_press_whole() {
    // Tap and Hold on the same keys and `when:` are one group, which
    // swallows the key if either asks (roadmap item 1.6): the Hold's default
    // swallows F13 although the Tap lets it through.
    let config = r#"( bindings: [
        (keys: "F13", action: SwitchLanguage, trigger: Tap(200), block_default: false),
        (keys: "F13", action: ToggleCapsLock, trigger: Hold(200)),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(50)
        .up("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(300)
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn passing_tap_fires_after_a_foreground_change_and_its_key_up_passes() {
    // A foreground change forgets only swallowed key-downs. F13's passed,
    // so its key-up still reaches Windows, and its Tap still fires.
    Scenario::from_ron(PASSING_TAP_F13)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("F13")
        .expect_passed()
        .foreground_changed()
        .advance_ms(50)
        .up("F13")
        .expect_passed()
        .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn remapping_off_during_a_passing_hold_drops_it_and_nothing_sticks() {
    Scenario::from_ron(PASSING_HOLD_F13)
        .check_nothing_stuck()
        .down("F13")
        .expect_passed()
        .advance_ms(50)
        .remapping_off()
        .advance_ms(300)
        .expect_nothing_fired()
        .up("F13")
        .expect_passed()
        .remapping_on()
        .down("F13")
        .expect_passed()
        .advance_ms(200)
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .expect_passed()
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn reload_during_a_passing_double_tap_keeps_each_press_whole() {
    // A press follows what its key-down got, whatever the bindings say by
    // its key-up. The entry armed by the first press keeps its action, and
    // the second key-down follows the binding loaded by then.
    Scenario::from_ron(LSHIFT_DOUBLE_TAP)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .advance_ms(60)
        .reload(LSHIFT_DOUBLE_TAP_BLOCKING)
        .up("LShift")
        .expect_passed()
        .advance_ms(40)
        .down("LShift")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("LShift")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);

    // And the other way round.
    Scenario::from_ron(LSHIFT_DOUBLE_TAP_BLOCKING)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_suppressed()
        .advance_ms(60)
        .reload(LSHIFT_DOUBLE_TAP)
        .up("LShift")
        .expect_suppressed()
        .advance_ms(40)
        .down("LShift")
        .expect_passed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("LShift")
        .expect_passed()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn a_key_typed_between_two_passing_lshift_presses_cancels_the_double_tap() {
    // Known issue 67, fixed (owner decision, roadmap item 2.2): another key
    // pressed while a gesture whose key passes is pending cancels it, as
    // AutoHotkey does. Typing `<T>` with the left Shift pressed again for
    // the `T` no longer toggles CapsLock with the template's example.
    Scenario::from_ron(LSHIFT_DOUBLE_TAP)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .advance_ms(30)
        .down(",")
        .expect_passed()
        .advance_ms(30)
        .up(",")
        .expect_passed()
        .advance_ms(30)
        .up("LShift")
        .expect_passed()
        .advance_ms(60)
        .down("LShift")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(30)
        .down("T")
        .expect_passed()
        .up("T")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .expect_total_fired(&[]);
}

#[test]
fn a_quick_ctrl_shortcut_cancels_a_passing_ctrl_tap() {
    // Known issue 67, fixed: Ctrl+C released within the term no longer
    // sends Escape too.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "LCtrl", action: PressKey("Escape"), trigger: Tap(200)) ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("LCtrl")
    .expect_passed()
    .advance_ms(30)
    .down("C")
    .expect_passed()
    .expect_nothing_fired()
    .advance_ms(30)
    .up("C")
    .expect_passed()
    .advance_ms(30)
    .up("LCtrl")
    .expect_passed()
    .expect_nothing_fired()
    .expect_total_fired(&[]);
}

#[test]
fn selecting_words_with_ctrl_shift_held_cancels_a_passing_ctrl_shift_hold() {
    // Known issue 67, fixed: holding Ctrl+Shift to select words with Right
    // no longer switches the layout once the term has passed.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)) ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("LCtrl")
    .expect_passed()
    .advance_ms(20)
    .down("LShift")
    .expect_passed()
    .advance_ms(280)
    .down("Right")
    .expect_passed()
    .expect_nothing_fired()
    .advance_ms(30)
    .up("Right")
    .expect_passed()
    .advance_ms(800)
    .expect_nothing_fired()
    .up("LShift")
    .expect_passed()
    .up("LCtrl")
    .expect_passed()
    .expect_total_fired(&[]);
}

#[test]
fn a_passing_ctrl_shift_double_tap_is_not_cancelled_by_its_own_keys() {
    // Roadmap item 2.3 (2.2 review OQ2): "another key-down" of the owner
    // decision on open question 10 is a key outside the pending gesture's
    // combo. The second press goes down LCtrl first, which must neither
    // cancel the window nor fire its waiting Tap; LShift then completes it.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Shift", action: ToggleCapsLock, trigger: DoubleTap(300)) ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("LCtrl")
    .expect_passed()
    .down("LShift")
    .expect_passed()
    .advance_ms(20)
    .up("LShift")
    .expect_passed()
    .up("LCtrl")
    .expect_passed()
    .advance_ms(30)
    .down("LCtrl")
    .expect_passed()
    .expect_nothing_fired()
    .down("LShift")
    .expect_passed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("LShift")
    .expect_passed()
    .up("LCtrl")
    .expect_passed()
    .expect_total_fired(&[BindAction::ToggleCapsLock]);

    // With a Tap that waits for the window: the own key fires no Tap.
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Tap(200)),
            (keys: "Ctrl+Shift", action: ToggleCapsLock, trigger: DoubleTap(300)),
        ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("LCtrl")
    .down("LShift")
    .advance_ms(20)
    .up("LShift")
    .up("LCtrl")
    .expect_nothing_fired()
    .advance_ms(30)
    .down("LCtrl")
    .expect_passed()
    .expect_nothing_fired()
    .down("LShift")
    .expect_passed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("LShift")
    .up("LCtrl")
    .advance_ms(500)
    .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn a_waiting_tap_of_a_passing_ctrl_shift_double_tap_fires_with_ctrl_down_in_windows() {
    // The declared cost of the rule above (known issue 68, critique M1):
    // the waiting Tap no longer fires at the own key's key-down, before
    // Windows sees it, but at the next other key, with LCtrl down in
    // Windows. Windows then gets Ctrl+Esc; the modifiers an output is sent
    // with belong to roadmap item 3.1.
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "Ctrl+Shift", action: PressKey("Escape"), trigger: Tap(200)),
            (keys: "Ctrl+Shift", action: ToggleCapsLock, trigger: DoubleTap(300)),
        ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("LCtrl")
    .down("LShift")
    .advance_ms(20)
    .up("LShift")
    .up("LCtrl")
    .expect_nothing_fired()
    .advance_ms(30)
    .down("LCtrl")
    .expect_passed()
    .expect_nothing_fired()
    .down("C")
    .expect_passed()
    .expect_fired(&[BindAction::PressKey(VK_ESCAPE)])
    .expect_down_in_windows("LCtrl")
    .up("C")
    .up("LCtrl")
    .advance_ms(500)
    .expect_total_fired(&[BindAction::PressKey(VK_ESCAPE)]);
}

#[test]
fn a_passing_hold_past_its_term_fires_whether_its_timer_runs_before_or_after_another_key_down() {
    // Orchestrator ruling I1: a Hold whose term has passed is resolved by
    // the clock. A key-down that comes after the term but before the
    // timer runs (a busy or late main thread) does not cancel it, so the
    // outcome does not depend on which of the two the main loop sees first.
    let config =
        r#"( bindings: [ (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)) ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LCtrl")
        .down("LShift")
        .advance_ms(1001)
        .expect_fired(&[BindAction::SwitchLanguage])
        .down("Right")
        .expect_passed()
        .expect_nothing_fired()
        .up("Right")
        .up("LShift")
        .up("LCtrl")
        .expect_total_fired(&[BindAction::SwitchLanguage]);

    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LCtrl")
        .down("LShift")
        .advance_ms_stalled(1001)
        .down("Right")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(0)
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("Right")
        .up("LShift")
        .up("LCtrl")
        .expect_total_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn a_lone_passing_ctrl_tap_still_fires() {
    Scenario::from_ron(
        r#"( bindings: [ (keys: "LCtrl", action: PressKey("Escape"), trigger: Tap(200)) ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("LCtrl")
    .expect_passed()
    .advance_ms(80)
    .up("LCtrl")
    .expect_passed()
    .expect_fired(&[BindAction::PressKey(VK_ESCAPE)]);
}

#[test]
fn another_key_does_not_cancel_a_swallowed_double_tap() {
    // Only a gesture whose key passes is cancelled: F13's swallowed press
    // keeps its DoubleTap window across A, as before roadmap item 2.2.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(300)) ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("F13")
    .expect_suppressed()
    .advance_ms(50)
    .up("F13")
    .expect_suppressed()
    .advance_ms(30)
    .down("A")
    .expect_passed()
    .advance_ms(20)
    .up("A")
    .expect_passed()
    .advance_ms(50)
    .down("F13")
    .expect_suppressed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("F13")
    .expect_suppressed()
    .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

// ---- One winner per key event (roadmap item 1.6, KS-13) ----

/// KS-13's first scenario under exact modifiers: a `CapsLock` Tap that
/// allows extra modifiers also matches Shift+CapsLock.
const SHIFT_CAPS_AND_CAPS_TAP: &str = r#"( bindings: [
    (keys: "Shift+CapsLock", action: ToggleCapsLock),
    (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true),
] )"#;

#[test]
fn shift_capslock_fires_only_toggle_capslock_next_to_a_capslock_tap_that_allows_extra_modifiers() {
    // Shift+CapsLock has more keys, so it wins the press and the Tap does
    // not arm: releasing CapsLock within the term switches nothing.
    Scenario::from_ron(SHIFT_CAPS_AND_CAPS_TAP)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .advance_ms(20)
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .advance_ms(50)
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("LShift")
        .expect_passed()
        .advance_ms(300)
        .expect_nothing_fired()
        .down("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(50)
        .up("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .expect_total_fired(&[BindAction::ToggleCapsLock, BindAction::SwitchLanguage]);
}

#[test]
fn shift_capslock_held_past_the_term_fires_no_capslock_hold() {
    let config = r#"( bindings: [
        (keys: "Shift+CapsLock", action: ToggleCapsLock),
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200), allow_extra_modifiers: true),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .advance_ms(20)
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .advance_ms(300)
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("LShift")
        .expect_passed()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn a_conditional_pause_binding_wins_over_an_unconditional_one_in_either_file_order() {
    let play_pause = BindAction::Media(MediaKey::PlayPause);
    let configs = [
        (
            "conditional binding first",
            r#"( bindings: [
                (keys: "Pause", action: Media(PlayPause), when: AppEquals("Spotify.exe")),
                (keys: "Pause", action: SwitchLanguage),
            ] )"#,
        ),
        (
            "unconditional binding first",
            r#"( bindings: [
                (keys: "Pause", action: SwitchLanguage),
                (keys: "Pause", action: Media(PlayPause), when: AppEquals("Spotify.exe")),
            ] )"#,
        ),
    ];
    for (name, config) in configs {
        Scenario::from_ron(config)
            .with_context(name)
            .check_press_symmetry()
            .check_nothing_stuck()
            .foreground("Spotify.exe", "Spotify Premium")
            .down("Pause")
            .expect_suppressed()
            .expect_fired(std::slice::from_ref(&play_pause))
            .up("Pause")
            .expect_suppressed()
            .foreground("notepad.exe", "Untitled - Notepad")
            .foreground_changed()
            .down("Pause")
            .expect_suppressed()
            .expect_fired(&[BindAction::SwitchLanguage])
            .up("Pause")
            .expect_suppressed();
    }
}

#[test]
fn a_more_specific_winner_with_block_default_false_lets_the_key_through() {
    // KS-13's Shift+X case: the losing `F13` asks to swallow the key, the
    // winning `Shift+F13` does not, and only the winner decides.
    let config = r#"( bindings: [
        (keys: "F13", action: SwitchLanguage, allow_extra_modifiers: true),
        (keys: "Shift+F13", action: ToggleCapsLock, block_default: false),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .expect_passed()
        .down("F13")
        .expect_passed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .advance_ms(500)
        .repeat("F13")
        .expect_passed()
        .expect_nothing_fired()
        .up("F13")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F13")
        .expect_suppressed();
}

#[test]
fn a_conditional_winner_with_block_default_false_lets_the_key_through() {
    let config = r#"( bindings: [
        (keys: "F14", action: SwitchLanguage),
        (keys: "F14", action: ToggleCapsLock, block_default: false, when: AppEquals("code.exe")),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .foreground("code.exe", "main.rs")
        .down("F14")
        .expect_passed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .advance_ms(500)
        .repeat("F14")
        .expect_passed()
        .expect_nothing_fired()
        .up("F14")
        .expect_passed()
        .foreground("notepad.exe", "Untitled - Notepad")
        .foreground_changed()
        .down("F14")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .up("F14")
        .expect_suppressed();
}

#[test]
fn a_more_specific_tap_keeps_a_less_specific_immediate_binding_from_firing() {
    // KS-13 the other way round: the more specific binding is deferred and
    // the less specific one Immediate. Ctrl+F22 wins every event of the
    // press, its auto-repeats included, so `F22` sends no Left.
    let config = r#"( bindings: [
        (keys: "F22", action: PressKey("Left"), allow_extra_modifiers: true),
        (keys: "Ctrl+F22", action: ToggleCapsLock, trigger: Tap(200), block_default: false),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LCtrl")
        .expect_passed()
        .down("F22")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(50)
        .up("F22")
        .expect_passed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .down("F22")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(500)
        .repeat("F22")
        .expect_passed()
        .expect_nothing_fired()
        .up("F22")
        .expect_passed()
        .expect_nothing_fired()
        .up("LCtrl")
        .expect_passed()
        .down("F22")
        .expect_suppressed()
        .expect_fired(&[BindAction::PressKey(VK_LEFT)])
        .up("F22")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::ToggleCapsLock, BindAction::PressKey(VK_LEFT)]);
}

#[test]
fn a_conditional_tap_and_hold_pair_arms_as_one_group_over_an_unconditional_binding() {
    let config = r#"( bindings: [
        (keys: "CapsLock", action: ToggleCapsLock),
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), when: AppEquals("code.exe")),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200), when: AppEquals("code.exe")),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .foreground("code.exe", "main.rs")
        .down("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(80)
        .up("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(100)
        .down("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(200)
        .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .foreground("notepad.exe", "Untitled - Notepad")
        .foreground_changed()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .advance_ms(300)
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_suppressed()
        .expect_total_fired(&[
            BindAction::SwitchLanguage,
            BindAction::PressKey(VK_LSHIFT),
            BindAction::ToggleCapsLock,
        ]);
}

#[test]
fn a_tap_and_a_hold_with_different_when_conditions_do_not_arm_together() {
    // A group is the same keys with the same `when:`. In VS Code the Tap
    // wins (a `when:` beats none) and the Hold does not arm with it.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), when: AppEquals("code.exe")),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .foreground("code.exe", "main.rs")
        .down("CapsLock")
        .expect_suppressed()
        .advance_ms(300)
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(50)
        .down("CapsLock")
        .expect_suppressed()
        .advance_ms(80)
        .up("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .foreground("notepad.exe", "Untitled - Notepad")
        .foreground_changed()
        .down("CapsLock")
        .expect_suppressed()
        .advance_ms(300)
        .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired();
}

#[test]
fn two_double_taps_on_the_same_keys_fire_only_the_first_on_the_second_press() {
    // The second DoubleTap of the group is skipped, so it neither consumes
    // the first one's entry on the first press nor re-arms its own.
    let config = r#"( bindings: [
        (keys: "F13", action: ToggleCapsLock, trigger: DoubleTap(250)),
        (keys: "F13", action: SwitchLanguage, trigger: DoubleTap(250)),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(60)
        .up("F13")
        .expect_suppressed()
        .advance_ms(100)
        .down("F13")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("F13")
        .expect_suppressed()
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn a_more_specific_tap_keeps_a_less_specific_hold_from_arming() {
    let config = r#"( bindings: [
        (keys: "F13", action: SwitchLanguage, trigger: Hold(200), allow_extra_modifiers: true),
        (keys: "Shift+F13", action: ToggleCapsLock, trigger: Tap(200)),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .down("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(300)
        .expect_nothing_fired()
        .up("F13")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("LShift")
        .down("LShift")
        .down("F13")
        .advance_ms(50)
        .up("F13")
        .expect_fired(&[BindAction::ToggleCapsLock])
        .up("LShift")
        .expect_total_fired(&[BindAction::ToggleCapsLock]);
}

#[test]
fn a_gesture_armed_by_the_winner_resolves_after_a_reload_adds_a_more_specific_binding() {
    // A pending gesture keeps its action, and the key-up does not consult
    // the bindings: the reload changes only the next press.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true) ] )"#,
    )
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("CapsLock")
    .expect_suppressed()
    .advance_ms(50)
    .reload(SHIFT_CAPS_AND_CAPS_TAP)
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .advance_ms(100)
    .down("CapsLock")
    .expect_suppressed()
    .advance_ms(50)
    .up("CapsLock")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage]);
}

#[test]
fn a_passing_user_binding_does_not_stop_the_auto_blocker() {
    // Auto-blockers stay outside the election: a user binding that wins
    // the press and lets it through leaves the blocker's suppression alone.
    Scenario::with_bindings(with_alt_shift_blocker(
        r#"( bindings: [ (keys: "LAlt+LShift", action: SwitchLanguageBackward, block_default: false) ] )"#,
    ))
    .check_press_symmetry()
    .check_nothing_stuck()
    .down("LAlt")
    .expect_passed()
    .down("LShift")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguageBackward])
    .up("LShift")
    .expect_suppressed()
    .up("LAlt")
    .expect_passed();
}

#[test]
fn a_group_member_arms_only_if_it_matches_the_press_itself() {
    // The Tap allows extra modifiers and wins Shift+CapsLock; the exact
    // Hold is in its group but does not match with Shift held.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200), allow_extra_modifiers: true),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LShift")
        .down("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(300)
        .expect_nothing_fired()
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .up("LShift")
        .expect_total_fired(&[]);
}

#[test]
fn tap_and_hold_on_capslock_still_arm_together_as_one_group() {
    // The README's Tap(200) + Hold(200) pair, listed above an Immediate
    // binding on the same key: the pair wins by file order and arms as one
    // group, and the Immediate binding never fires.
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, trigger: Tap(200)),
        (keys: "CapsLock", action: PressKey("LShift"), trigger: Hold(200)),
        (keys: "CapsLock", action: ToggleCapsLock),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .advance_ms(199)
        .up("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::SwitchLanguage])
        .advance_ms(100)
        .down("CapsLock")
        .expect_suppressed()
        .advance_ms(201)
        .expect_fired(&[BindAction::PressKey(VK_LSHIFT)])
        .up("CapsLock")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::SwitchLanguage, BindAction::PressKey(VK_LSHIFT)]);
}

#[test]
fn repeats_after_the_modifier_is_released_elect_again_and_follow_the_key_down() {
    // The election runs on every key-down, auto-repeats included, while a
    // repeat is suppressed or passed like its key-down (KS-06). Ctrl+J wins
    // the key-down and lets it through; once Ctrl is released, `J` wins the
    // repeats and sends Down on each, and the repeats still reach Windows.
    // Pinned until a winner is locked per press (a follow-up of roadmap
    // item 1.6).
    let config = r#"( bindings: [
        (keys: "J", action: PressKey("Down"), allow_extra_modifiers: true),
        (keys: "Ctrl+J", action: SwitchLanguage, trigger: Tap(200), block_default: false),
    ] )"#;
    Scenario::from_ron(config)
        .check_press_symmetry()
        .check_nothing_stuck()
        .down("LCtrl")
        .expect_passed()
        .down("J")
        .expect_passed()
        .expect_nothing_fired()
        .advance_ms(100)
        .up("LCtrl")
        .expect_passed()
        .advance_ms(400)
        .repeat("J")
        .expect_passed()
        .expect_fired(&[BindAction::PressKey(VK_DOWN)])
        .advance_ms(33)
        .repeat("J")
        .expect_passed()
        .expect_fired(&[BindAction::PressKey(VK_DOWN)])
        .up("J")
        .expect_passed()
        .expect_nothing_fired()
        .expect_total_fired(&[BindAction::PressKey(VK_DOWN), BindAction::PressKey(VK_DOWN)]);
}

// ---- Menu mask key (roadmap item 1.2, KS-12) ----

/// `PostMessage(msg: "WM_CLOSE")` as the loader builds it.
const POST_WM_CLOSE: BindAction = BindAction::PostMessage {
    msg: 0x0010,
    wparam: 0,
    lparam: 0,
};

#[test]
fn win_q_post_message_masks_the_swallowed_q_and_its_release() {
    // KS-12: Windows saw Win go down and nothing after it, so Win's release
    // would open the Start menu. The swallowed Q and its release ask for
    // the mask key; the events that pass do not.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Win+Q", action: PostMessage(msg: "WM_CLOSE")) ] )"#,
    )
    .check_nothing_stuck()
    .check_press_symmetry()
    .down("LWin")
    .expect_passed()
    .expect_unmasked()
    .down("Q")
    .expect_suppressed()
    .expect_fired(&[POST_WM_CLOSE])
    .expect_masked()
    .up("Q")
    .expect_suppressed()
    .expect_masked()
    .up("LWin")
    .expect_passed()
    .expect_unmasked();
}

#[test]
fn alt_then_shift_with_the_template_blocker_masks_the_swallowed_shift() {
    // The template's `block_original_combo: true` swallows Shift pressed
    // after Alt; Windows saw a lone Alt, whose release would activate the
    // menu bar.
    template()
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LAlt")
        .expect_passed()
        .expect_unmasked()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
        .expect_masked()
        .advance_ms(500)
        .repeat("LShift")
        .expect_suppressed()
        .expect_masked()
        .up("LShift")
        .expect_suppressed()
        .expect_masked()
        .up("LAlt")
        .expect_passed()
        .expect_unmasked()
        .expect_total_fired(&[]);
}

#[test]
fn right_alt_and_right_win_mask_a_swallowed_key() {
    Scenario::from_ron(
        r#"( bindings: [
            (keys: "Alt+J", action: DoNothing),
            (keys: "Win+J", action: DoNothing),
        ] )"#,
    )
    .check_nothing_stuck()
    .check_press_symmetry()
    .down("RAlt")
    .expect_passed()
    .expect_unmasked()
    .down("J")
    .expect_suppressed()
    .expect_fired(&[BindAction::DoNothing])
    .expect_masked()
    .up("J")
    .expect_suppressed()
    .expect_masked()
    .up("RAlt")
    .expect_passed()
    .expect_unmasked()
    .down("RWin")
    .expect_passed()
    .expect_unmasked()
    .down("J")
    .expect_suppressed()
    .expect_fired(&[BindAction::DoNothing])
    .expect_masked()
    .up("J")
    .expect_suppressed()
    .expect_masked()
    .up("RWin")
    .expect_passed()
    .expect_unmasked();
}

#[test]
fn swallowed_repeats_and_key_up_while_win_is_held_are_masked() {
    // Q's key-down was swallowed before Win went down, so its repeats and
    // its key-up stay swallowed; with Win down in Windows they mask too.
    Scenario::from_ron(r#"( bindings: [ (keys: "Q", action: DoNothing) ] )"#)
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("Q")
        .expect_suppressed()
        .expect_unmasked()
        .down("LWin")
        .expect_passed()
        .expect_unmasked()
        .advance_ms(500)
        .repeat("Q")
        .expect_suppressed()
        .expect_masked()
        .up("Q")
        .expect_suppressed()
        .expect_masked()
        .up("LWin")
        .expect_passed()
        .expect_unmasked();
}

#[test]
fn a_swallowed_event_without_alt_or_win_in_windows_is_not_masked() {
    // CapsLock alone, the Ctrl+Shift blocker and Shift+CapsLock swallow
    // keys while neither Alt nor Win is down: nothing to mask.
    Scenario::from_ron_with_language_hotkey(DEFAULT_CONFIG, &[VK_CONTROL, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("CapsLock")
        .expect_suppressed()
        .expect_unmasked()
        .up("CapsLock")
        .expect_suppressed()
        .expect_unmasked()
        .down("LCtrl")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_unmasked()
        .up("LShift")
        .expect_suppressed()
        .expect_unmasked()
        .up("LCtrl")
        .expect_passed()
        .down("LShift")
        .expect_passed()
        .down("CapsLock")
        .expect_suppressed()
        .expect_fired(&[BindAction::ToggleCapsLock])
        .expect_unmasked()
        .up("CapsLock")
        .expect_suppressed()
        .expect_unmasked()
        .up("LShift")
        .expect_passed()
        .expect_unmasked();
}

#[test]
fn a_swallowed_key_is_not_masked_when_the_alt_or_win_key_down_was_swallowed() {
    // Windows never saw that Alt or Win go down, so its release opens
    // nothing.
    Scenario::with_bindings(with_alt_shift_blocker(DEFAULT_PAIR))
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LShift")
        .expect_passed()
        .down("LAlt")
        .expect_suppressed()
        .expect_unmasked()
        .up("LAlt")
        .expect_suppressed()
        .expect_unmasked()
        .up("LShift")
        .expect_passed();

    Scenario::from_ron(
        r#"( bindings: [
            (keys: "Win", action: ToggleCapsLock, trigger: Tap(200), block_default: true),
            (keys: "Q", action: DoNothing, allow_extra_modifiers: true),
        ] )"#,
    )
    .check_nothing_stuck()
    .check_press_symmetry()
    .down("LWin")
    .expect_suppressed()
    .expect_unmasked()
    .down("Q")
    .expect_suppressed()
    .expect_fired(&[BindAction::DoNothing])
    .expect_unmasked()
    .up("Q")
    .expect_suppressed()
    .expect_unmasked()
    .up("LWin")
    .expect_suppressed()
    .expect_unmasked();
}

#[test]
fn a_passed_event_is_not_masked() {
    // An event that reaches Windows tells it that Alt is not alone.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Alt+L", action: DoNothing, block_default: false) ] )"#,
    )
    .check_nothing_stuck()
    .check_press_symmetry()
    .down("LCtrl")
    .expect_passed()
    .down("LAlt")
    .expect_passed()
    .down("L")
    .expect_passed()
    .expect_fired(&[BindAction::DoNothing])
    .expect_unmasked()
    .up("L")
    .expect_passed()
    .expect_unmasked()
    .up("LCtrl")
    .expect_passed()
    .down("A")
    .expect_passed()
    .expect_unmasked()
    .up("A")
    .expect_passed()
    .up("LAlt")
    .expect_passed()
    .expect_unmasked();
}

#[test]
fn alt_held_before_the_hook_saw_it_masks_a_swallowed_key() {
    // The resync finds Alt held with no record of its key-down: Windows
    // has it down.
    Scenario::from_ron(r#"( bindings: [ (keys: "Alt+J", action: DoNothing) ] )"#)
        .down_unseen("LAlt")
        .down("J")
        .expect_suppressed()
        .expect_fired(&[BindAction::DoNothing])
        .expect_masked()
        .up("J")
        .expect_suppressed()
        .expect_masked();
}

#[test]
fn win_held_through_remapping_on_masks_a_swallowed_key() {
    // `swch on` takes Win from the OS as a key-down Windows saw.
    Scenario::from_ron(r#"( bindings: [ (keys: "Win+Q", action: DoNothing) ] )"#)
        .check_nothing_stuck()
        .remapping_off()
        .down("LWin")
        .expect_passed()
        .remapping_on()
        .down("Q")
        .expect_suppressed()
        .expect_fired(&[BindAction::DoNothing])
        .expect_masked()
        .up("Q")
        .expect_suppressed()
        .expect_masked()
        .up("LWin")
        .expect_passed()
        .expect_unmasked();
}

#[test]
fn win_held_across_a_foreground_change_masks_a_swallowed_key() {
    // A foreground change keeps the records of key-downs that passed.
    Scenario::from_ron(r#"( bindings: [ (keys: "Win+Q", action: DoNothing) ] )"#)
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LWin")
        .expect_passed()
        .foreground_changed()
        .down("Q")
        .expect_suppressed()
        .expect_fired(&[BindAction::DoNothing])
        .expect_masked()
        .up("Q")
        .expect_suppressed()
        .expect_masked()
        .up("LWin")
        .expect_passed()
        .expect_unmasked();
}

#[test]
fn sequence_and_chord_keys_swallowed_while_win_is_held_are_masked() {
    Scenario::from_ron(
        r#"( bindings: [ (sequence: (steps: ["Win+F13", "Win+F14"]), action: SwitchLanguage) ] )"#,
    )
    .check_nothing_stuck()
    .check_press_symmetry()
    .down("LWin")
    .expect_passed()
    .down("F13")
    .expect_suppressed()
    .expect_masked()
    .up("F13")
    .expect_suppressed()
    .expect_masked()
    .down("F14")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .expect_masked()
    .up("F14")
    .expect_suppressed()
    .expect_masked()
    .up("LWin")
    .expect_passed()
    .expect_unmasked();

    Scenario::from_ron(
        r#"( bindings: [ (chord: (keys: ["F13", "F14"]), action: SwitchLanguage) ] )"#,
    )
    .check_nothing_stuck()
    .check_press_symmetry()
    .down("LWin")
    .expect_passed()
    .down("F13")
    .expect_suppressed()
    .expect_masked()
    .down("F14")
    .expect_suppressed()
    .expect_fired(&[BindAction::SwitchLanguage])
    .expect_masked()
    .up("F13")
    .expect_suppressed()
    .expect_masked()
    .up("F14")
    .expect_suppressed()
    .expect_masked()
    .up("LWin")
    .expect_passed()
    .expect_unmasked();
}

#[test]
fn a_hold_timer_never_masks() {
    // `advance_ms` asserts that no timer decision asks for the mask: a
    // timer has no event in the hook to put it in front of.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Win+F13", action: DoNothing, trigger: Hold(200)) ] )"#,
    )
    .down("LWin")
    .expect_passed()
    .down("F13")
    .expect_suppressed()
    .advance_ms(250)
    .expect_fired(&[BindAction::DoNothing])
    .up("F13")
    .expect_suppressed()
    .up("LWin")
    .expect_passed();
}

// ---- block_original_combo: MaskOnly (roadmap item 1.2, KS-05) ----

/// The default config's two bindings, with the experimental
/// `block_original_combo: MaskOnly`.
const MASK_ONLY_CONFIG: &str = r#"( bindings: [
    (keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly),
    (keys: "Shift+CapsLock", action: ToggleCapsLock),
] )"#;

#[test]
fn mask_only_blocker_lets_alt_and_shift_through_and_masks_the_exact_hotkey() {
    // The hypothesis under test by hand: Windows switches the layout only
    // on a clean Alt+Shift, so a mask key inside it stops the switch. The
    // order of a key injected from inside the hook against the event in
    // the hook is not documented, so the next event masks once more: that
    // mask lands after Shift's key-down whatever the order.
    Scenario::from_ron_with_language_hotkey(MASK_ONLY_CONFIG, &[VK_MENU, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LAlt")
        .expect_passed()
        .expect_unmasked()
        .down("LShift")
        .expect_passed()
        .expect_masked()
        .up("LShift")
        .expect_passed()
        .expect_masked()
        .up("LAlt")
        .expect_passed()
        .expect_unmasked()
        .expect_total_fired(&[]);
}

#[test]
fn mask_only_blocker_lets_ctrl_shift_t_reach_the_application() {
    // KS-05: Ctrl, Shift, T arrived as Ctrl+T with `true`.
    Scenario::from_ron_with_language_hotkey(MASK_ONLY_CONFIG, &[VK_CONTROL, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LCtrl")
        .expect_passed()
        .expect_unmasked()
        .down("LShift")
        .expect_passed()
        .expect_masked()
        .down("T")
        .expect_passed()
        .expect_masked()
        .up("T")
        .expect_passed()
        .expect_unmasked()
        .up("LShift")
        .expect_passed()
        .expect_unmasked()
        .up("LCtrl")
        .expect_passed()
        .expect_unmasked()
        .expect_total_fired(&[]);
}

#[test]
fn mask_only_blocker_does_not_mask_a_hotkey_pressed_with_another_key_held() {
    // Windows sees Alt, Tab and Shift, not its hotkey alone.
    Scenario::from_ron_with_language_hotkey(MASK_ONLY_CONFIG, &[VK_MENU, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LAlt")
        .expect_passed()
        .down("Tab")
        .expect_passed()
        .down("LShift")
        .expect_passed()
        .expect_unmasked()
        .up("Tab")
        .expect_passed()
        .expect_unmasked()
        .up("LShift")
        .expect_passed()
        .expect_unmasked()
        .up("LAlt")
        .expect_passed()
        .expect_unmasked();
}

#[test]
fn mask_only_blocker_masks_only_the_first_event_after_the_match() {
    Scenario::from_ron_with_language_hotkey(MASK_ONLY_CONFIG, &[VK_MENU, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LAlt")
        .down("LShift")
        .expect_passed()
        .expect_masked()
        .advance_ms(500)
        .repeat("LShift")
        .expect_passed()
        .expect_masked()
        .advance_ms(33)
        .repeat("LShift")
        .expect_passed()
        .expect_unmasked()
        .up("LShift")
        .expect_passed()
        .expect_unmasked()
        .up("LAlt")
        .expect_passed()
        .expect_unmasked();
}

#[test]
fn alt_shift_x_fires_next_to_a_mask_only_blocker() {
    Scenario::from_ron_with_language_hotkey(
        r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: MaskOnly),
            (keys: "Alt+Shift+X", action: ToggleCapsLock),
        ] )"#,
        &[VK_MENU, VK_SHIFT],
    )
    .check_nothing_stuck()
    .check_press_symmetry()
    .down("LAlt")
    .expect_passed()
    .down("LShift")
    .expect_passed()
    .expect_masked()
    .down("X")
    .expect_suppressed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .expect_masked()
    .up("X")
    .expect_suppressed()
    .expect_masked()
    .up("LShift")
    .expect_passed()
    .expect_unmasked()
    .up("LAlt")
    .expect_passed()
    .expect_unmasked();
}

#[test]
fn remapping_off_drops_an_owed_mask() {
    Scenario::from_ron_with_language_hotkey(MASK_ONLY_CONFIG, &[VK_MENU, VK_SHIFT])
        .check_nothing_stuck()
        .down("LAlt")
        .down("LShift")
        .expect_masked()
        .remapping_off()
        .remapping_on()
        .down("F13")
        .expect_passed()
        .expect_unmasked()
        .up("F13")
        .expect_passed()
        .up("LShift")
        .expect_passed()
        .up("LAlt")
        .expect_passed();
}

// ---- The auto-blocker's when: (roadmap item 1.2, KS-35) ----

#[test]
fn a_blocker_asked_for_by_a_binding_with_when_app_equals_is_inactive_in_other_apps() {
    // The binding remaps the language switch in x.exe only, so Windows
    // keeps its hotkey everywhere else.
    Scenario::from_ron_with_language_hotkey(
        r#"( bindings: [
            (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("x.exe")),
        ] )"#,
        &[VK_MENU, VK_SHIFT],
    )
    .check_nothing_stuck()
    .check_press_symmetry()
    .foreground("y.exe", "Y")
    .down("LAlt")
    .expect_passed()
    .down("LShift")
    .expect_passed()
    .expect_unmasked()
    .up("LShift")
    .expect_passed()
    .up("LAlt")
    .expect_passed()
    .foreground("x.exe", "X")
    .down("LAlt")
    .expect_passed()
    .down("LShift")
    .expect_suppressed()
    .expect_masked()
    .up("LShift")
    .expect_suppressed()
    .up("LAlt")
    .expect_passed()
    .expect_total_fired(&[]);
}

#[test]
fn blockers_act_where_any_owner_when_holds() {
    let config = r#"( bindings: [
        (keys: "CapsLock", action: SwitchLanguage, block_original_combo: true, when: AppEquals("a.exe")),
        (keys: "F13", action: SwitchLanguageBackward, block_original_combo: true, when: AppEquals("b.exe")),
    ] )"#;
    for (app, swallowed) in [("a.exe", true), ("b.exe", true), ("c.exe", false)] {
        let scenario = Scenario::from_ron_with_language_hotkey(config, &[VK_MENU, VK_SHIFT])
            .with_context(app)
            .check_nothing_stuck()
            .check_press_symmetry()
            .foreground(app, app)
            .down("LAlt")
            .expect_passed()
            .down("LShift");
        let scenario = if swallowed {
            scenario.expect_suppressed()
        } else {
            scenario.expect_passed()
        };
        scenario.up("LShift").up("LAlt").expect_passed();
    }
}

// ---- The harness itself ----

#[test]
#[should_panic(expected = "expected the event to be passed, it was suppressed")]
fn a_failed_expectation_panics() {
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("CapsLock")
        .expect_passed();
}

#[test]
#[should_panic(expected = "down CapsLock at +0 ms: CapsLock is not down in Windows")]
fn down_in_windows_check_reports_a_key_windows_does_not_have_down() {
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("CapsLock")
        .expect_down_in_windows("CapsLock");
}

#[test]
#[should_panic(expected = "the key is not held")]
fn releasing_a_key_that_is_up_is_a_script_error() {
    Scenario::from_ron(r#"( bindings: [] )"#).up("A");
}

#[test]
#[should_panic(expected = "press of F13: key-down suppressed, up passed")]
fn press_symmetry_check_reports_a_key_up_that_differs_from_its_key_down() {
    // A foreground change forgets that F13's key-down was swallowed, so its
    // key-up passes (roadmap item 2.3).
    Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: SwitchLanguage) ] )"#)
        .check_press_symmetry()
        .down("F13")
        .foreground_changed()
        .up("F13");
}

#[test]
#[should_panic(expected = "press of LShift: key-down passed, repeat suppressed")]
fn press_symmetry_check_reports_a_repeat_that_differs_from_its_key_down() {
    // A and then LShift went down while the hook was not looking, so Windows
    // has both down and LShift auto-repeats. A's key-up makes the engine
    // resync the modifiers, which finds LShift held with no record of its
    // key-down. A repeat without a record is swallowed when a binding asks
    // for it.
    Scenario::from_ron(r#"( bindings: [ (keys: "LShift", action: SwitchLanguage) ] )"#)
        .check_press_symmetry()
        .down_unseen("A")
        .down_unseen("LShift")
        .up("A")
        .repeat("LShift");
}

#[test]
#[should_panic(expected = "on an auto-repeat")]
fn repeat_check_reports_an_action_fired_on_an_auto_repeat() {
    Scenario::from_ron(r#"( bindings: [ (keys: "F13", action: PressKey("Left")) ] )"#)
        .check_repeats_fire_only(&[])
        .down("F13")
        .repeat("F13");
}

#[test]
fn nothing_stuck_holds_for_a_plain_press_and_an_unseen_release() {
    Scenario::from_ron(r#"( bindings: [] )"#)
        .check_nothing_stuck()
        .down("A")
        .up("A")
        .down("Shift")
        .up_unseen("Shift")
        .expect_nothing_stuck();
}

#[test]
#[should_panic(expected = "context: seed 7")]
fn context_is_printed_with_a_failure() {
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .with_context("seed 7")
        .down("CapsLock")
        .expect_passed();
}
