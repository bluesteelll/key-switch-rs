//! Scenario tests: RON bindings driven through key events on a fake clock.
//!
//! They pin the behaviour of the decision core that is correct today, for
//! every binding kind and trigger. Known defects (see the audit) are not
//! pinned here; the items that fix them add their own scenarios.

use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::testing::Scenario;
use crate::core::windows_actions::{BindAction, MediaKey};
use crate::data::binding::Binding;
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

/// The `Alt+Shift` auto-blocker that `block_original_combo` adds for the
/// Windows language hotkey, followed by the bindings of `ron`. The blocker
/// comes first, where it used to shadow a binding on the same keys; the
/// loader now appends it. It is built directly: loading it from RON would
/// read the registry.
pub(super) fn with_alt_shift_blocker(ron: &str) -> Vec<Binding> {
    let mut bindings = vec![Binding::new_auto_blocker(KeyCombination::from_keys(vec![
        VK_MENU, VK_SHIFT,
    ]))];
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
fn first_binding_whose_condition_holds_wins_on_a_shared_key() {
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

// ---- Hold ----

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
    // The template's commented example (`default_config.ron`).
    Scenario::from_ron(
        r#"( bindings: [ (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250)) ] )"#,
    )
    .down("LShift")
    .expect_suppressed()
    .expect_nothing_fired()
    .up("LShift")
    .expect_suppressed()
    .advance_ms(100)
    .down("LShift")
    .expect_suppressed()
    .expect_fired(&[BindAction::ToggleCapsLock])
    .up("LShift")
    .expect_suppressed()
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
    Scenario::with_bindings(vec![Binding::new_auto_blocker(KeyCombination::from_keys(
        vec![VK_MENU, VK_SHIFT],
    ))])
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
    // up.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "LShift", action: ToggleCapsLock, trigger: DoubleTap(250)) ] )"#,
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
    // instead of "Shift+CapsLock".
    let config = r#"( bindings: [
        (keys: "Shift", action: SwitchLanguageBackward, trigger: DoubleTap(300)),
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
    // inside the 600 ms term.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600)) ] )"#,
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
    // Hold then swallowed the key-up, leaving Shift held there.
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Ctrl+Shift", action: SwitchLanguage, trigger: Hold(1000)) ] )"#,
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
    Scenario::from_ron(
        r#"( bindings: [ (keys: "Shift", action: ToggleCapsLock, trigger: DoubleTap(600)) ] )"#,
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
    // Ctrl+Shift when Shift goes down: the blocker swallows Shift, and T
    // reaches Windows as Ctrl+T until roadmap item 1.2.
    Scenario::from_ron_with_language_hotkey(DEFAULT_CONFIG, &[VK_CONTROL, VK_SHIFT])
        .check_nothing_stuck()
        .check_press_symmetry()
        .down("LCtrl")
        .expect_passed()
        .down("LShift")
        .expect_suppressed()
        .expect_nothing_fired()
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
    // down, so the blocker swallows it and Tab reaches Windows as Alt+Tab
    // until roadmap item 1.2.
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
    blocker_last.push(Binding::new_auto_blocker(KeyCombination::from_keys(vec![
        VK_MENU, VK_SHIFT,
    ])));
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
    Scenario::with_bindings(with_alt_shift_blocker(
        r#"( bindings: [ (keys: "Win", action: ToggleCapsLock, trigger: Tap(200)) ] )"#,
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

// ---- The harness itself ----

#[test]
#[should_panic(expected = "expected the event to be passed, it was suppressed")]
fn a_failed_expectation_panics() {
    Scenario::from_ron(r#"( bindings: [ (keys: "CapsLock", action: SwitchLanguage) ] )"#)
        .down("CapsLock")
        .expect_passed();
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
