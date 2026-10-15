//! Parses combo strings (`"Ctrl+Shift+Esc"`) and WM_* names.

use windows::Win32::UI::{Input::KeyboardAndMouse::VIRTUAL_KEY, WindowsAndMessaging::*};

use crate::data::key_combination::KeyCombination;
use crate::data::key_spec::KeySpec;
use crate::data::vk_name::parse_vk;

/// Parses the trigger keys of a `keys` binding or a sequence step
/// (`"Ctrl+Shift+Esc"`) into a `KeyCombination` of [`KeySpec`]s: `"Shift"`
/// is either Shift key, `"LShift"` the left one. Duplicates are dropped.
///
/// Empty input or any unrecognised segment is a hard error — surfacing it to
/// the user is much better than silently dropping a binding.
pub(crate) fn parse_combo(s: &str) -> Result<KeyCombination, String> {
    parse_segments(s, KeySpec::parse).map(KeyCombination::from_keys)
}

/// Parses the keys a macro `Combo("Ctrl+S")` step sends: the codes
/// `SendInput` presses, so `"Win"` is the left Win key and `"Shift"` the
/// generic Shift code. Duplicates are dropped, the order is kept. Errors as
/// [`parse_combo`].
pub(crate) fn parse_key_list(s: &str) -> Result<Vec<VIRTUAL_KEY>, String> {
    let keys = parse_segments(s, parse_vk)?;
    let mut deduped: Vec<VIRTUAL_KEY> = Vec::with_capacity(keys.len());
    for key in keys {
        if !deduped.contains(&key) {
            deduped.push(key);
        }
    }
    Ok(deduped)
}

/// Splits `s` into segments at `'+'` and parses each with `parse`.
fn parse_segments<T>(s: &str, parse: impl Fn(&str) -> Option<T>) -> Result<Vec<T>, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("combo is empty".to_string());
    }

    let mut keys = Vec::new();
    for raw_segment in trimmed.split('+') {
        let segment = raw_segment.trim();
        if segment.is_empty() {
            return Err(format!("empty segment in combo {:?} (stray '+'?)", s));
        }
        let key =
            parse(segment).ok_or_else(|| format!("unknown key {:?} in combo {:?}", segment, s))?;
        keys.push(key);
    }
    Ok(keys)
}

/// Resolves a symbolic WM_* name to its numeric value. Only the constants
/// most likely to appear in user configs are mapped; anything else should be
/// written as a numeric literal in TOML.
pub(crate) fn parse_wm_name(name: &str) -> Option<u32> {
    let normalized: String = name
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_uppercase())
        .collect();

    match normalized.as_str() {
        "WM_CLOSE" => Some(WM_CLOSE),
        "WM_QUIT" => Some(WM_QUIT),
        "WM_DESTROY" => Some(WM_DESTROY),
        "WM_COMMAND" => Some(WM_COMMAND),
        "WM_SYSCOMMAND" => Some(WM_SYSCOMMAND),
        "WM_INPUTLANGCHANGEREQUEST" => Some(WM_INPUTLANGCHANGEREQUEST),
        "WM_KEYDOWN" => Some(WM_KEYDOWN),
        "WM_KEYUP" => Some(WM_KEYUP),
        "WM_SYSKEYDOWN" => Some(WM_SYSKEYDOWN),
        "WM_SYSKEYUP" => Some(WM_SYSKEYUP),
        "WM_HOTKEY" => Some(WM_HOTKEY),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::key_spec::Modifier;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn parse_combo_single_key() {
        let c = parse_combo("CapsLock").unwrap();
        assert_eq!(c.keys, vec![KeySpec::Key(VK_CAPITAL)]);
    }

    #[test]
    fn parse_combo_multi() {
        let c = parse_combo("Ctrl+Shift+Esc").unwrap();
        assert!(c.keys.contains(&KeySpec::AnySide(Modifier::Ctrl)));
        assert!(c.keys.contains(&KeySpec::AnySide(Modifier::Shift)));
        assert!(c.keys.contains(&KeySpec::Key(VK_ESCAPE)));
    }

    #[test]
    fn parse_combo_keeps_sides() {
        let c = parse_combo("LShift+RWin+A").unwrap();
        assert_eq!(
            c.keys,
            vec![
                KeySpec::Key(VK_LSHIFT),
                KeySpec::Key(VK_RWIN),
                KeySpec::Key(VK_A)
            ]
        );
    }

    #[test]
    fn parse_combo_win_is_either_side_lwin_is_left() {
        let any_win = KeySpec::AnySide(Modifier::Win);
        assert_eq!(parse_combo("Win+E").unwrap().keys[0], any_win);
        assert_eq!(parse_combo("Super+E").unwrap().keys[0], any_win);
        assert_eq!(
            parse_combo("LWin+E").unwrap().keys[0],
            KeySpec::Key(VK_LWIN)
        );
        // Different keys: both kept.
        assert_eq!(parse_combo("Win+LWin+Super+E").unwrap().keys.len(), 3);
    }

    #[test]
    fn parse_key_list_keeps_output_codes() {
        assert_eq!(
            parse_key_list("Win+Shift+S").unwrap(),
            vec![VK_LWIN, VK_SHIFT, VK_S]
        );
        assert_eq!(
            parse_key_list("Ctrl+Ctrl+A").unwrap(),
            vec![VK_CONTROL, VK_A]
        );
        assert!(parse_key_list("").is_err());
        assert!(parse_key_list("Ctrl++A").is_err());
        assert!(parse_key_list("Ctrl+Foo").unwrap_err().contains("Foo"));
    }

    #[test]
    fn parse_combo_whitespace_tolerant() {
        let c = parse_combo("  Ctrl + Shift + Esc  ").unwrap();
        assert_eq!(c.keys.len(), 3);
    }

    #[test]
    fn parse_combo_case_insensitive() {
        let c = parse_combo("ctrl+a").unwrap();
        assert!(c.keys.contains(&KeySpec::AnySide(Modifier::Ctrl)));
        assert!(c.keys.contains(&KeySpec::Key(VK_A)));
    }

    #[test]
    fn parse_combo_dedup() {
        let c = parse_combo("Ctrl+Ctrl+A").unwrap();
        // KeyCombination dedupes via from_keys.
        assert_eq!(c.keys.len(), 2);
    }

    #[test]
    fn parse_combo_rejects_empty() {
        assert!(parse_combo("").is_err());
        assert!(parse_combo("   ").is_err());
    }

    #[test]
    fn parse_combo_rejects_stray_plus() {
        assert!(parse_combo("Ctrl++A").is_err());
        assert!(parse_combo("+A").is_err());
        assert!(parse_combo("A+").is_err());
    }

    #[test]
    fn parse_combo_rejects_unknown() {
        let err = parse_combo("Ctrl+Foo").unwrap_err();
        assert!(
            err.contains("Foo"),
            "error should mention failing segment: {}",
            err
        );
    }

    #[test]
    fn parse_wm_known() {
        assert_eq!(parse_wm_name("WM_CLOSE"), Some(WM_CLOSE));
        assert_eq!(parse_wm_name("wm_close"), Some(WM_CLOSE));
        assert_eq!(
            parse_wm_name("WM_INPUTLANGCHANGEREQUEST"),
            Some(WM_INPUTLANGCHANGEREQUEST)
        );
    }

    #[test]
    fn parse_wm_unknown() {
        assert_eq!(parse_wm_name("WM_NOT_REAL"), None);
        assert_eq!(parse_wm_name(""), None);
    }
}
