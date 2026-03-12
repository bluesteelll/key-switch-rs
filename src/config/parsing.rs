//! Parses combo strings (`"Ctrl+Shift+Esc"`) and WM_* names.

use windows::Win32::UI::{
    Input::KeyboardAndMouse::VIRTUAL_KEY,
    WindowsAndMessaging::*,
};

use crate::data::key_combination::KeyCombination;
use crate::data::vk_name::parse_vk;

/// Splits `"Ctrl+Shift+Esc"` into segments by `'+'`, parses each into a
/// `VIRTUAL_KEY`, and returns the assembled `KeyCombination`.
///
/// Empty input or any unrecognised segment is a hard error — surfacing it to
/// the user is much better than silently dropping a binding.
pub(crate) fn parse_combo(s: &str) -> Result<KeyCombination, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err("combo is empty".to_string());
    }

    let mut keys: Vec<VIRTUAL_KEY> = Vec::new();
    for raw_segment in trimmed.split('+') {
        let segment = raw_segment.trim();
        if segment.is_empty() {
            return Err(format!("empty segment in combo {:?} (stray '+'?)", s));
        }
        let vk = parse_vk(segment).ok_or_else(|| {
            format!("unknown key {:?} in combo {:?}", segment, s)
        })?;
        keys.push(vk);
    }

    Ok(KeyCombination::from_keys(keys))
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
        "WM_CLOSE"                  => Some(WM_CLOSE),
        "WM_QUIT"                   => Some(WM_QUIT),
        "WM_DESTROY"                => Some(WM_DESTROY),
        "WM_COMMAND"                => Some(WM_COMMAND),
        "WM_SYSCOMMAND"             => Some(WM_SYSCOMMAND),
        "WM_INPUTLANGCHANGEREQUEST" => Some(WM_INPUTLANGCHANGEREQUEST),
        "WM_KEYDOWN"                => Some(WM_KEYDOWN),
        "WM_KEYUP"                  => Some(WM_KEYUP),
        "WM_SYSKEYDOWN"             => Some(WM_SYSKEYDOWN),
        "WM_SYSKEYUP"               => Some(WM_SYSKEYUP),
        "WM_HOTKEY"                 => Some(WM_HOTKEY),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn parse_combo_single_key() {
        let c = parse_combo("CapsLock").unwrap();
        assert_eq!(c.keys, vec![VK_CAPITAL]);
    }

    #[test]
    fn parse_combo_multi() {
        let c = parse_combo("Ctrl+Shift+Esc").unwrap();
        assert!(c.keys.contains(&VK_CONTROL));
        assert!(c.keys.contains(&VK_SHIFT));
        assert!(c.keys.contains(&VK_ESCAPE));
    }

    #[test]
    fn parse_combo_whitespace_tolerant() {
        let c = parse_combo("  Ctrl + Shift + Esc  ").unwrap();
        assert_eq!(c.keys.len(), 3);
    }

    #[test]
    fn parse_combo_case_insensitive() {
        let c = parse_combo("ctrl+a").unwrap();
        assert!(c.keys.contains(&VK_CONTROL));
        assert!(c.keys.contains(&VK_A));
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
        assert!(err.contains("Foo"), "error should mention failing segment: {}", err);
    }

    #[test]
    fn parse_wm_known() {
        assert_eq!(parse_wm_name("WM_CLOSE"), Some(WM_CLOSE));
        assert_eq!(parse_wm_name("wm_close"), Some(WM_CLOSE));
        assert_eq!(parse_wm_name("WM_INPUTLANGCHANGEREQUEST"), Some(WM_INPUTLANGCHANGEREQUEST));
    }

    #[test]
    fn parse_wm_unknown() {
        assert_eq!(parse_wm_name("WM_NOT_REAL"), None);
        assert_eq!(parse_wm_name(""), None);
    }
}
