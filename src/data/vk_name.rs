use windows::Win32::UI::Input::KeyboardAndMouse::*;

/// Renders a virtual-key code as a short human-readable name (e.g. "Shift",
/// "F5", "A", "CapsLock"). Falls back to "VK(<raw>)" for unmapped codes.
pub fn vk_name(vk: VIRTUAL_KEY) -> String {
    let raw = vk.0;

    if (VK_A.0..=VK_Z.0).contains(&raw) {
        return ((b'A' + (raw - VK_A.0) as u8) as char).to_string();
    }
    if (VK_0.0..=VK_9.0).contains(&raw) {
        return ((b'0' + (raw - VK_0.0) as u8) as char).to_string();
    }
    if (VK_F1.0..=VK_F24.0).contains(&raw) {
        return format!("F{}", raw - VK_F1.0 + 1);
    }
    if (VK_NUMPAD0.0..=VK_NUMPAD9.0).contains(&raw) {
        return format!("Num{}", raw - VK_NUMPAD0.0);
    }

    match vk {
        VK_SHIFT    => "Shift",
        VK_LSHIFT   => "LShift",
        VK_RSHIFT   => "RShift",
        VK_CONTROL  => "Ctrl",
        VK_LCONTROL => "LCtrl",
        VK_RCONTROL => "RCtrl",
        VK_MENU     => "Alt",
        VK_LMENU    => "LAlt",
        VK_RMENU    => "RAlt",
        VK_LWIN     => "Win",
        VK_RWIN     => "RWin",
        VK_CAPITAL  => "CapsLock",
        VK_TAB      => "Tab",
        VK_ESCAPE   => "Esc",
        VK_RETURN   => "Enter",
        VK_SPACE    => "Space",
        VK_BACK     => "Backspace",
        VK_DELETE   => "Delete",
        VK_INSERT   => "Insert",
        VK_HOME     => "Home",
        VK_END      => "End",
        VK_PRIOR    => "PageUp",
        VK_NEXT     => "PageDown",
        VK_UP       => "Up",
        VK_DOWN     => "Down",
        VK_LEFT     => "Left",
        VK_RIGHT    => "Right",
        VK_OEM_3    => "`",
        VK_OEM_MINUS => "-",
        VK_OEM_PLUS  => "=",
        VK_OEM_COMMA => ",",
        VK_OEM_PERIOD => ".",
        VK_OEM_1    => ";",
        VK_OEM_2    => "/",
        VK_OEM_4    => "[",
        VK_OEM_5    => "\\",
        VK_OEM_6    => "]",
        VK_OEM_7    => "'",
        VK_NUMLOCK  => "NumLock",
        VK_SCROLL   => "ScrollLock",
        VK_PAUSE    => "Pause",
        VK_SNAPSHOT => "PrtSc",
        VK_MEDIA_PLAY_PAUSE => "MediaPlayPause",
        VK_MEDIA_STOP       => "MediaStop",
        VK_MEDIA_NEXT_TRACK => "MediaNext",
        VK_MEDIA_PREV_TRACK => "MediaPrev",
        VK_VOLUME_UP        => "VolumeUp",
        VK_VOLUME_DOWN      => "VolumeDown",
        VK_VOLUME_MUTE      => "VolumeMute",
        _ => return format!("VK({})", raw),
    }.to_string()
}

/// Parses a single human-readable key name back into a `VIRTUAL_KEY`. Matching
/// is case-insensitive and tolerates internal whitespace so users can write
/// `"Caps Lock"`, `"CAPSLOCK"`, or `"capslock"` interchangeably.
///
/// Supported forms:
///   - Letters `A`–`Z` (case-insensitive)
///   - Digits `0`–`9`
///   - Function keys `F1`–`F24`
///   - Numpad digits `Num0`–`Num9` / `Numpad0`–`Numpad9`
///   - Modifier aliases (`Ctrl`/`Control`, `Alt`/`Menu`, `Win`/`Super`/`LWin`, ...)
///   - Named keys (`Esc`/`Escape`, `Del`/`Delete`, `Enter`/`Return`, ...)
///   - OEM punctuation as a single character (`` ` ``, `-`, `=`, `,`, `.`,
///     `;`, `/`, `[`, `]`, `\`, `'`)
///
/// Returns `None` for anything not in the table above. Callers should surface
/// the failing name to the user — never silently drop unparseable bindings.
pub fn parse_vk(name: &str) -> Option<VIRTUAL_KEY> {
    // Normalize: strip ASCII whitespace, fold to lowercase. Length grows by at
    // most the input length, so a String is fine — config parsing is not on a
    // hot path.
    let normalized: String = name
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_lowercase())
        .collect();

    if normalized.is_empty() {
        return None;
    }

    // Single-character forms: letters, digits, OEM punctuation.
    if normalized.chars().count() == 1 {
        let ch = normalized.chars().next().unwrap();
        if let Some(vk) = single_char_vk(ch) {
            return Some(vk);
        }
    }

    // F1..F24
    if let Some(rest) = normalized.strip_prefix('f')
        && let Ok(n) = rest.parse::<u16>()
        && (1..=24).contains(&n)
    {
        return Some(VIRTUAL_KEY(VK_F1.0 + n - 1));
    }

    // Numpad digits: accept `num0`..`num9` and `numpad0`..`numpad9`.
    for prefix in ["numpad", "num"] {
        if let Some(rest) = normalized.strip_prefix(prefix)
            && let Ok(n) = rest.parse::<u16>()
            && (0..=9).contains(&n)
        {
            return Some(VIRTUAL_KEY(VK_NUMPAD0.0 + n));
        }
    }

    // Named keys & modifier aliases.
    match normalized.as_str() {
        "shift"                                 => Some(VK_SHIFT),
        "lshift" | "leftshift"                  => Some(VK_LSHIFT),
        "rshift" | "rightshift"                 => Some(VK_RSHIFT),
        "ctrl" | "control"                      => Some(VK_CONTROL),
        "lctrl" | "leftctrl" | "leftcontrol"    => Some(VK_LCONTROL),
        "rctrl" | "rightctrl" | "rightcontrol"  => Some(VK_RCONTROL),
        "alt" | "menu"                          => Some(VK_MENU),
        "lalt" | "leftalt"                      => Some(VK_LMENU),
        "ralt" | "rightalt"                     => Some(VK_RMENU),
        "win" | "super" | "lwin" | "leftwin"    => Some(VK_LWIN),
        "rwin" | "rightwin"                     => Some(VK_RWIN),
        "capslock" | "caps"                     => Some(VK_CAPITAL),
        "tab"                                   => Some(VK_TAB),
        "esc" | "escape"                        => Some(VK_ESCAPE),
        "enter" | "return"                      => Some(VK_RETURN),
        "space" | "spacebar"                    => Some(VK_SPACE),
        "backspace" | "back"                    => Some(VK_BACK),
        "del" | "delete"                        => Some(VK_DELETE),
        "ins" | "insert"                        => Some(VK_INSERT),
        "home"                                  => Some(VK_HOME),
        "end"                                   => Some(VK_END),
        "pageup" | "pgup"                       => Some(VK_PRIOR),
        "pagedown" | "pgdn"                     => Some(VK_NEXT),
        "up" | "uparrow"                        => Some(VK_UP),
        "down" | "downarrow"                    => Some(VK_DOWN),
        "left" | "leftarrow"                    => Some(VK_LEFT),
        "right" | "rightarrow"                  => Some(VK_RIGHT),
        "numlock"                               => Some(VK_NUMLOCK),
        "scrolllock" | "scroll"                 => Some(VK_SCROLL),
        "pause" | "break"                       => Some(VK_PAUSE),
        "prtsc" | "printscreen" | "prtscn"      => Some(VK_SNAPSHOT),
        // Media / volume keys. Recognised so hardware multimedia buttons can
        // be used as combo triggers.
        "mediaplaypause" | "playpause"          => Some(VK_MEDIA_PLAY_PAUSE),
        "mediastop"                             => Some(VK_MEDIA_STOP),
        "medianext" | "nexttrack"               => Some(VK_MEDIA_NEXT_TRACK),
        "mediaprev" | "mediaprevious" | "prevtrack" => Some(VK_MEDIA_PREV_TRACK),
        "volumeup" | "volup"                    => Some(VK_VOLUME_UP),
        "volumedown" | "voldown"                => Some(VK_VOLUME_DOWN),
        "volumemute" | "volmute" | "mute"       => Some(VK_VOLUME_MUTE),
        _ => None,
    }
}

fn single_char_vk(ch: char) -> Option<VIRTUAL_KEY> {
    if ch.is_ascii_alphabetic() {
        let upper = ch.to_ascii_uppercase() as u16;
        return Some(VIRTUAL_KEY(VK_A.0 + (upper - b'A' as u16)));
    }
    if ch.is_ascii_digit() {
        return Some(VIRTUAL_KEY(VK_0.0 + (ch as u16 - b'0' as u16)));
    }
    match ch {
        '`' => Some(VK_OEM_3),
        '-' => Some(VK_OEM_MINUS),
        '=' => Some(VK_OEM_PLUS),
        ',' => Some(VK_OEM_COMMA),
        '.' => Some(VK_OEM_PERIOD),
        ';' => Some(VK_OEM_1),
        '/' => Some(VK_OEM_2),
        '[' => Some(VK_OEM_4),
        ']' => Some(VK_OEM_6),
        '\\' => Some(VK_OEM_5),
        '\'' => Some(VK_OEM_7),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_map_to_chars() {
        assert_eq!(vk_name(VK_A), "A");
        assert_eq!(vk_name(VK_Z), "Z");
    }

    #[test]
    fn digits_map_to_chars() {
        assert_eq!(vk_name(VK_0), "0");
        assert_eq!(vk_name(VK_9), "9");
    }

    #[test]
    fn function_keys() {
        assert_eq!(vk_name(VK_F1), "F1");
        assert_eq!(vk_name(VK_F12), "F12");
        assert_eq!(vk_name(VK_F24), "F24");
    }

    #[test]
    fn modifiers() {
        assert_eq!(vk_name(VK_SHIFT), "Shift");
        assert_eq!(vk_name(VK_CONTROL), "Ctrl");
        assert_eq!(vk_name(VK_MENU), "Alt");
        assert_eq!(vk_name(VK_LWIN), "Win");
        assert_eq!(vk_name(VK_CAPITAL), "CapsLock");
    }

    #[test]
    fn unknown_falls_back() {
        assert_eq!(vk_name(VIRTUAL_KEY(0xFE)), "VK(254)");
    }

    #[test]
    fn parse_letters_case_insensitive() {
        assert_eq!(parse_vk("A"), Some(VK_A));
        assert_eq!(parse_vk("a"), Some(VK_A));
        assert_eq!(parse_vk("z"), Some(VK_Z));
    }

    #[test]
    fn parse_digits() {
        assert_eq!(parse_vk("0"), Some(VK_0));
        assert_eq!(parse_vk("7"), Some(VK_7));
    }

    #[test]
    fn parse_function_keys() {
        assert_eq!(parse_vk("F1"), Some(VK_F1));
        assert_eq!(parse_vk("f12"), Some(VK_F12));
        assert_eq!(parse_vk("F24"), Some(VK_F24));
        assert_eq!(parse_vk("F25"), None);
        assert_eq!(parse_vk("F0"), None);
    }

    #[test]
    fn parse_numpad() {
        assert_eq!(parse_vk("Num0"), Some(VK_NUMPAD0));
        assert_eq!(parse_vk("Numpad9"), Some(VK_NUMPAD9));
    }

    #[test]
    fn parse_modifier_aliases() {
        assert_eq!(parse_vk("Ctrl"), Some(VK_CONTROL));
        assert_eq!(parse_vk("Control"), Some(VK_CONTROL));
        assert_eq!(parse_vk("Alt"), Some(VK_MENU));
        assert_eq!(parse_vk("Menu"), Some(VK_MENU));
        assert_eq!(parse_vk("Win"), Some(VK_LWIN));
        assert_eq!(parse_vk("Super"), Some(VK_LWIN));
        assert_eq!(parse_vk("RWin"), Some(VK_RWIN));
    }

    #[test]
    fn parse_named_keys() {
        assert_eq!(parse_vk("CapsLock"), Some(VK_CAPITAL));
        assert_eq!(parse_vk("caps"), Some(VK_CAPITAL));
        assert_eq!(parse_vk("Caps Lock"), Some(VK_CAPITAL)); // whitespace stripped
        assert_eq!(parse_vk("Esc"), Some(VK_ESCAPE));
        assert_eq!(parse_vk("Escape"), Some(VK_ESCAPE));
        assert_eq!(parse_vk("Delete"), Some(VK_DELETE));
        assert_eq!(parse_vk("Del"), Some(VK_DELETE));
    }

    #[test]
    fn parse_oem_punctuation() {
        assert_eq!(parse_vk("`"), Some(VK_OEM_3));
        assert_eq!(parse_vk(";"), Some(VK_OEM_1));
        assert_eq!(parse_vk("/"), Some(VK_OEM_2));
        assert_eq!(parse_vk("["), Some(VK_OEM_4));
        assert_eq!(parse_vk("\\"), Some(VK_OEM_5));
    }

    #[test]
    fn parse_unknown_returns_none() {
        assert_eq!(parse_vk(""), None);
        assert_eq!(parse_vk("foobar"), None);
        assert_eq!(parse_vk("Ctrl+"), None); // splitting is parser's job, not this fn
    }

    #[test]
    fn vk_name_roundtrip_for_common_keys() {
        for vk in [
            VK_A, VK_Z, VK_0, VK_9, VK_F1, VK_F24, VK_SHIFT, VK_CONTROL,
            VK_MENU, VK_LWIN, VK_CAPITAL, VK_TAB, VK_ESCAPE, VK_RETURN,
            VK_SPACE, VK_DELETE, VK_INSERT, VK_HOME, VK_END, VK_UP, VK_DOWN,
            VK_LEFT, VK_RIGHT, VK_NUMPAD0, VK_NUMPAD9,
        ] {
            let name = vk_name(vk);
            assert_eq!(
                parse_vk(&name),
                Some(vk),
                "round-trip failed for {:?} (rendered as {})",
                vk, name,
            );
        }
    }
}
