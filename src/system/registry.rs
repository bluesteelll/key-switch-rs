use windows::{
    Win32::{System::Registry::*, UI::Input::KeyboardAndMouse::*},
    core::*,
};

use crate::data::key_combination::KeyCombination;
use crate::data::key_spec::{KeySpec, Modifier};
use crate::system::system_function::SystemFunction;

struct RegistryLocation {
    hkey: HKEY,
    subkey: &'static str,
    value_names: &'static [&'static str],
    parser: fn(&str) -> HotkeySetting,
}

/// What a registry value says about a Windows hotkey (KS-17).
#[derive(Debug, Clone, PartialEq, Eq)]
enum HotkeySetting {
    /// The value names this combination.
    Combo(KeyCombination),
    /// The value turns the hotkey off: "3", "Not assigned" in Windows'
    /// settings.
    Disabled,
    /// The value is a string this parser does not know.
    Unknown,
}

fn get_registry_location(function: SystemFunction) -> Option<RegistryLocation> {
    match function {
        SystemFunction::SwitchLanguage | SystemFunction::SwitchLanguageBackward => {
            Some(RegistryLocation {
                hkey: HKEY_CURRENT_USER,
                subkey: "Keyboard Layout\\Toggle",
                value_names: &["Hotkey", "Language Hotkey"],
                parser: parse_language_hotkey,
            })
        }

        SystemFunction::LockWorkstation => Some(RegistryLocation {
            hkey: HKEY_CURRENT_USER,
            subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
            value_names: &["LockWorkstationHotkey"],
            parser: parse_win_key_combo,
        }),

        SystemFunction::ShowDesktop => Some(RegistryLocation {
            hkey: HKEY_CURRENT_USER,
            subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
            value_names: &["ShowDesktopHotkey"],
            parser: parse_win_key_combo,
        }),

        // Ctrl+Shift+Esc is handled directly by Winlogon and has no
        // user-configurable registry value. The `TaskManagerHotkey` name under
        // Policies\System used to appear in audits but does not actually exist
        // as a hotkey definition — fall back to the default combination only.
        SystemFunction::TaskManager => None,

        SystemFunction::ToggleCapsLock => None,
    }
}

pub fn get_system_hotkey(function: SystemFunction) -> Option<KeyCombination> {
    if let Some(location) = get_registry_location(function)
        && let Some(key) = open_key(&location)
    {
        return hotkey_from_values(
            function,
            location
                .value_names
                .iter()
                .map(|name| read_string_value(&key, name)),
        );
    }
    hotkey_from_values(function, std::iter::empty::<Option<String>>())
}

/// The hotkey of `function`, given the data of its registry values in the
/// order of the location's `value_names`: `Some(text)` for a value read as
/// a string, `None` for one that is missing, unreadable or not a string.
/// The first value whose text is a `Combo` decides; the values after it are
/// not read. With no deciding value, or for a function without a registry
/// location, the built-in default. Prints the [INFO] line that says where
/// the hotkey came from.
pub(crate) fn hotkey_from_values<S: AsRef<str>>(
    function: SystemFunction,
    values: impl IntoIterator<Item = Option<S>>,
) -> Option<KeyCombination> {
    if let Some(location) = get_registry_location(function) {
        for value in values.into_iter().flatten() {
            match (location.parser)(value.as_ref()) {
                HotkeySetting::Combo(combo) => {
                    println!("[INFO] {function:?}: registry combination {combo}");
                    return Some(combo);
                }
                // KS-17: "3" is still read past like an unknown value;
                // roadmap item 1.4 makes it decide.
                HotkeySetting::Disabled | HotkeySetting::Unknown => {}
            }
        }
    }

    let default = get_default_combination(function);
    if default.is_some() {
        println!("[INFO] {function:?}: using default combination");
    }
    default
}

fn get_default_combination(function: SystemFunction) -> Option<KeyCombination> {
    match function {
        SystemFunction::SwitchLanguage | SystemFunction::SwitchLanguageBackward => {
            Some(KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT]))
        }
        SystemFunction::LockWorkstation => Some(win_with(VK_L)),
        SystemFunction::ShowDesktop => Some(win_with(VK_D)),
        SystemFunction::TaskManager => Some(KeyCombination::from_keys(vec![
            VK_CONTROL, VK_SHIFT, VK_ESCAPE,
        ])),
        SystemFunction::ToggleCapsLock => None,
    }
}

/// RAII guard that closes an HKEY on drop, so the key handle is closed on
/// every path out of `get_system_hotkey`, a panic included.
struct HKeyGuard(HKEY);

impl Drop for HKeyGuard {
    fn drop(&mut self) {
        // SAFETY: the inner HKEY is the result of a successful RegOpenKeyExW
        // and has not been closed elsewhere.
        unsafe {
            let _ = RegCloseKey(self.0);
        }
    }
}

/// `location`'s subkey opened for reading, or `None` if it cannot be opened
/// (it does not exist, or access is denied).
fn open_key(location: &RegistryLocation) -> Option<HKeyGuard> {
    let subkey_wide: Vec<u16> = location
        .subkey
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut hkey = HKEY::default();

    // SAFETY: subkey_wide is null-terminated and outlives the call; &mut hkey
    // points to a stack-local that is valid for the duration of the call.
    let open_result = unsafe {
        RegOpenKeyExW(
            location.hkey,
            PCWSTR(subkey_wide.as_ptr()),
            None,
            KEY_READ,
            &mut hkey,
        )
    };
    if open_result.is_err() {
        return None;
    }
    Some(HKeyGuard(hkey))
}

/// The data of the value `value_name` under `key`, up to its first NUL, if
/// it is a string (`REG_SZ` or `REG_EXPAND_SZ`). `None` if the value is
/// missing, does not fit the buffer, or has another type (with a [WARN]).
fn read_string_value(key: &HKeyGuard, value_name: &str) -> Option<String> {
    let value_name_wide: Vec<u16> = value_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut data: [u16; 128] = [0; 128];
    let mut data_size = std::mem::size_of_val(&data) as u32;
    let mut reg_type = REG_VALUE_TYPE(0);

    // SAFETY: key.0 is an open key handle (the guard closes it only on drop,
    // and it is borrowed for this call); value_name_wide is null-terminated
    // and outlives the call; data buffer is sized and writable; data_size
    // carries its capacity in bytes per RegQueryValueExW's contract.
    let result = unsafe {
        RegQueryValueExW(
            key.0,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            Some(&mut reg_type),
            Some(data.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        )
    };

    if result.is_err() {
        return None;
    }

    // Reject non-string types instead of reinterpreting REG_DWORD/REG_BINARY
    // bytes as UTF-16 and parsing garbage.
    if reg_type != REG_SZ && reg_type != REG_EXPAND_SZ {
        eprintln!(
            "[WARN] registry value {:?} has type {:?}, expected REG_SZ/REG_EXPAND_SZ",
            value_name, reg_type
        );
        return None;
    }

    // data_size is in bytes; clamp the scan to what the API actually wrote.
    let returned_wchars = (data_size as usize) / 2;
    let scan_len = returned_wchars.min(data.len());
    let str_len = data[..scan_len]
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(scan_len);
    Some(String::from_utf16_lossy(&data[..str_len]))
}

fn parse_language_hotkey(value: &str) -> HotkeySetting {
    match value.trim() {
        "1" => HotkeySetting::Combo(KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT])),
        "2" => HotkeySetting::Combo(KeyCombination::from_keys(vec![VK_CONTROL, VK_SHIFT])),
        "3" => HotkeySetting::Disabled,
        "4" => HotkeySetting::Combo(KeyCombination::new(VK_OEM_3)),
        _ => HotkeySetting::Unknown,
    }
}

fn parse_win_key_combo(value: &str) -> HotkeySetting {
    let value = value.trim().to_uppercase();

    if value.len() == 1
        && let Some(vk) = value.chars().next().and_then(char_to_vk)
    {
        return HotkeySetting::Combo(win_with(vk));
    }

    HotkeySetting::Unknown
}

/// Either Win key with `key`: Windows' Win hotkeys work with both.
fn win_with(key: VIRTUAL_KEY) -> KeyCombination {
    KeyCombination::new(KeySpec::AnySide(Modifier::Win)).with(key)
}

fn char_to_vk(ch: char) -> Option<VIRTUAL_KEY> {
    match ch {
        'A' => Some(VK_A),
        'B' => Some(VK_B),
        'C' => Some(VK_C),
        'D' => Some(VK_D),
        'E' => Some(VK_E),
        'F' => Some(VK_F),
        'G' => Some(VK_G),
        'H' => Some(VK_H),
        'I' => Some(VK_I),
        'J' => Some(VK_J),
        'K' => Some(VK_K),
        'L' => Some(VK_L),
        'M' => Some(VK_M),
        'N' => Some(VK_N),
        'O' => Some(VK_O),
        'P' => Some(VK_P),
        'Q' => Some(VK_Q),
        'R' => Some(VK_R),
        'S' => Some(VK_S),
        'T' => Some(VK_T),
        'U' => Some(VK_U),
        'V' => Some(VK_V),
        'W' => Some(VK_W),
        'X' => Some(VK_X),
        'Y' => Some(VK_Y),
        'Z' => Some(VK_Z),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alt_shift() -> KeyCombination {
        KeyCombination::from_keys([VK_MENU, VK_SHIFT])
    }

    fn ctrl_shift() -> KeyCombination {
        KeyCombination::from_keys([VK_CONTROL, VK_SHIFT])
    }

    fn grave_accent() -> KeyCombination {
        KeyCombination::new(VK_OEM_3)
    }

    /// The language hotkey that the strings of the values `Hotkey` and
    /// `Language Hotkey`, in that order, resolve to (`None` for a missing
    /// value). Both language functions resolve them alike.
    fn language(values: [Option<&str>; 2]) -> Option<KeyCombination> {
        let forward = hotkey_from_values(SystemFunction::SwitchLanguage, values);
        let backward = hotkey_from_values(SystemFunction::SwitchLanguageBackward, values);
        assert_eq!(forward, backward, "{values:?}");
        forward
    }

    /// The data a value can hold, a missing value included.
    const ANYTHING: [Option<&str>; 7] = [
        None,
        Some("1"),
        Some("2"),
        Some("3"),
        Some("4"),
        Some(""),
        Some("x"),
    ];

    #[test]
    fn parse_language_hotkey_is_tri_state() {
        assert_eq!(
            parse_language_hotkey("1"),
            HotkeySetting::Combo(alt_shift())
        );
        assert_eq!(
            parse_language_hotkey("2"),
            HotkeySetting::Combo(ctrl_shift())
        );
        assert_eq!(
            parse_language_hotkey("4"),
            HotkeySetting::Combo(grave_accent())
        );
        assert_eq!(parse_language_hotkey("3"), HotkeySetting::Disabled);
        assert_eq!(parse_language_hotkey(" 3 "), HotkeySetting::Disabled);
        for value in ["", "0", "5", "33", "unknown", "Alt+Shift"] {
            assert_eq!(
                parse_language_hotkey(value),
                HotkeySetting::Unknown,
                "{value:?}"
            );
        }
    }

    #[test]
    fn the_language_hotkey_is_read_from_hotkey_then_language_hotkey() {
        // The resolution tests inject the values by position; this pins
        // which registry value takes which position, so that `Hotkey`
        // decides before `Language Hotkey` (roadmap item 1.4).
        for function in [
            SystemFunction::SwitchLanguage,
            SystemFunction::SwitchLanguageBackward,
        ] {
            let location = get_registry_location(function)
                .unwrap_or_else(|| panic!("{function:?} has a registry location"));
            assert_eq!(location.hkey, HKEY_CURRENT_USER, "{function:?}");
            assert_eq!(location.subkey, "Keyboard Layout\\Toggle", "{function:?}");
            assert_eq!(
                location.value_names,
                ["Hotkey", "Language Hotkey"],
                "{function:?}"
            );
        }
    }

    #[test]
    fn the_hotkey_value_decides_before_the_language_hotkey_value() {
        for other in ANYTHING {
            assert_eq!(
                language([Some("2"), other]),
                Some(ctrl_shift()),
                "{other:?}"
            );
            assert_eq!(language([Some("1"), other]), Some(alt_shift()), "{other:?}");
        }
    }

    #[test]
    fn no_usable_value_falls_back_to_alt_shift() {
        for values in [[None, None], [Some(""), Some("x")], [Some("5"), None]] {
            assert_eq!(language(values), Some(alt_shift()), "{values:?}");
        }
        // The key `Keyboard Layout\Toggle` does not exist.
        assert_eq!(
            hotkey_from_values(
                SystemFunction::SwitchLanguage,
                std::iter::empty::<Option<&str>>()
            ),
            Some(alt_shift())
        );
    }

    #[test]
    fn a_missing_or_unknown_hotkey_value_leaves_the_language_hotkey_value_to_decide() {
        assert_eq!(language([None, Some("2")]), Some(ctrl_shift()));
        assert_eq!(language([Some("x"), Some("4")]), Some(grave_accent()));
        assert_eq!(language([None, Some("1")]), Some(alt_shift()));
    }

    #[test]
    fn the_values_after_the_deciding_one_are_not_read() {
        let values =
            std::iter::once(Some("2")).chain(std::iter::repeat_with(|| -> Option<&'static str> {
                panic!("a value after the deciding one was read")
            }));
        assert_eq!(
            hotkey_from_values(SystemFunction::SwitchLanguage, values),
            Some(ctrl_shift())
        );
    }

    #[test]
    fn other_functions_keep_their_rules() {
        assert_eq!(
            hotkey_from_values(SystemFunction::ToggleCapsLock, [Some("2")]),
            None
        );
        // No registry location: the values are not looked at.
        assert_eq!(
            hotkey_from_values(SystemFunction::TaskManager, [Some("3")]),
            Some(KeyCombination::from_keys([VK_CONTROL, VK_SHIFT, VK_ESCAPE]))
        );
        // "3" is no Win key letter: only the language hotkey can be
        // disabled.
        assert_eq!(
            hotkey_from_values(SystemFunction::LockWorkstation, [Some("3")]),
            Some(win_with(VK_L))
        );
        assert_eq!(
            hotkey_from_values(SystemFunction::LockWorkstation, [Some("K")]),
            Some(win_with(VK_K))
        );
    }

    #[test]
    fn test_get_system_hotkey_returns_combinations() {
        // The language hotkey depends on this machine's Windows setting,
        // which may also be "Not assigned".
        let language = get_system_hotkey(SystemFunction::SwitchLanguage);
        assert!(
            language.as_ref().is_none_or(|combo| [
                KeyCombination::from_keys([VK_MENU, VK_SHIFT]),
                KeyCombination::from_keys([VK_CONTROL, VK_SHIFT]),
                KeyCombination::new(VK_OEM_3),
            ]
            .contains(combo)),
            "not a Windows language hotkey: {language:?}"
        );
        assert!(get_system_hotkey(SystemFunction::LockWorkstation).is_some());
        assert!(get_system_hotkey(SystemFunction::ShowDesktop).is_some());
        assert!(get_system_hotkey(SystemFunction::TaskManager).is_some());
        assert!(get_system_hotkey(SystemFunction::ToggleCapsLock).is_none());
    }

    #[test]
    fn test_default_combinations() {
        let lang_default = get_default_combination(SystemFunction::SwitchLanguage).unwrap();
        assert!(lang_default.keys.contains(&KeySpec::AnySide(Modifier::Alt)));
        assert!(
            lang_default
                .keys
                .contains(&KeySpec::AnySide(Modifier::Shift))
        );

        let lock_default = get_default_combination(SystemFunction::LockWorkstation).unwrap();
        assert!(lock_default.keys.contains(&KeySpec::AnySide(Modifier::Win)));
        assert!(lock_default.keys.contains(&KeySpec::Key(VK_L)));
    }

    #[test]
    fn language_hotkeys_are_either_side_modifiers() {
        let HotkeySetting::Combo(ctrl_shift) = parse_language_hotkey("2") else {
            panic!("\"2\" names a combination");
        };
        assert_eq!(
            ctrl_shift.keys,
            vec![
                KeySpec::AnySide(Modifier::Ctrl),
                KeySpec::AnySide(Modifier::Shift)
            ]
        );
        assert_eq!(ctrl_shift.to_string(), "Ctrl+Shift");
        let HotkeySetting::Combo(grave_accent) = parse_language_hotkey("4") else {
            panic!("\"4\" names a combination");
        };
        assert_eq!(grave_accent.keys, vec![KeySpec::Key(VK_OEM_3)]);
    }

    #[test]
    fn test_char_to_vk() {
        assert_eq!(char_to_vk('L'), Some(VK_L));
        assert_eq!(char_to_vk('D'), Some(VK_D));
        assert_eq!(char_to_vk('A'), Some(VK_A));
        assert_eq!(char_to_vk('1'), None);
    }

    #[test]
    fn test_parse_win_key_combo() {
        let HotkeySetting::Combo(combo) = parse_win_key_combo("L") else {
            panic!("\"L\" names a combination");
        };
        assert!(combo.keys.contains(&KeySpec::AnySide(Modifier::Win)));
        assert!(combo.keys.contains(&KeySpec::Key(VK_L)));

        let HotkeySetting::Combo(combo2) = parse_win_key_combo("D") else {
            panic!("\"D\" names a combination");
        };
        assert!(combo2.keys.contains(&KeySpec::AnySide(Modifier::Win)));
        assert!(combo2.keys.contains(&KeySpec::Key(VK_D)));
        assert_eq!(combo2.to_string(), "Win+D");

        for value in ["1", "", "LL"] {
            assert_eq!(
                parse_win_key_combo(value),
                HotkeySetting::Unknown,
                "{value:?}"
            );
        }
    }
}
