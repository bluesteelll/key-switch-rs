//! Scenario tests: RON bindings driven through key events on a fake clock.
//!
//! They pin the behaviour of the decision core that is correct today, for
//! every binding kind and trigger. Known defects (see the audit) are not
//! pinned here; the items that fix them add their own scenarios.

use windows::Win32::UI::Input::KeyboardAndMouse::*;

use super::testing::Scenario;
use crate::core::windows_actions::BindAction;
use crate::data::binding::Binding;
use crate::data::key_combination::KeyCombination;

fn launch(exe: &str) -> BindAction {
    BindAction::Launch {
        exe: exe.to_owned(),
        args: Vec::new(),
    }
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
    // fire. Whether the repeat itself passes is KS-06 and not pinned here.
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
    // The focus change forgets that J's key-down was swallowed; the chord
    // completing on K marks every chord key again, so J's key-up is
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
    .expect_fired(&[BindAction::DoNothing])
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
