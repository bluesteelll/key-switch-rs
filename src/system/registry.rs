

use windows::{
    core::*,
    Win32::{
        System::Registry::*,
        UI::Input::KeyboardAndMouse::*,
    },
};

use crate::data::key_combination::KeyCombination;
use crate::system::system_function::SystemFunction;

struct RegistryLocation {
    hkey: HKEY,
    subkey: &'static str,
    value_names: &'static [&'static str],
    parser: fn(&str) -> Option<KeyCombination>,
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

        SystemFunction::LockWorkstation => {
            Some(RegistryLocation {
                hkey: HKEY_CURRENT_USER,
                subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                value_names: &["LockWorkstationHotkey"],
                parser: parse_win_key_combo,
            })
        }

        SystemFunction::ShowDesktop => {
            Some(RegistryLocation {
                hkey: HKEY_CURRENT_USER,
                subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced",
                value_names: &["ShowDesktopHotkey"],
                parser: parse_win_key_combo,
            })
        }

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
        && let Some(combo) = read_from_registry(&location)
    {
        println!("[INFO] {:?}: registry combination {:?}", function, combo.keys);
        return Some(combo);
    }

    let default = get_default_combination(function);
    if default.is_some() {
        println!("[INFO] {:?}: using default combination", function);
    }
    default
}

fn get_default_combination(function: SystemFunction) -> Option<KeyCombination> {
    match function {
        SystemFunction::SwitchLanguage | SystemFunction::SwitchLanguageBackward => {
            Some(KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT]))
        }
        SystemFunction::LockWorkstation => {
            Some(KeyCombination::from_keys(vec![VK_LWIN, VK_L]))
        }
        SystemFunction::ShowDesktop => {
            Some(KeyCombination::from_keys(vec![VK_LWIN, VK_D]))
        }
        SystemFunction::TaskManager => {
            Some(KeyCombination::from_keys(vec![VK_CONTROL, VK_SHIFT, VK_ESCAPE]))
        }
        SystemFunction::ToggleCapsLock => None,
    }
}

/// RAII guard that closes an HKEY on drop. Prevents leaking the key handle if
/// `read_value_and_parse` ever panics between open and the explicit close.
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

fn read_from_registry(location: &RegistryLocation) -> Option<KeyCombination> {
    let subkey_wide: Vec<u16> = location.subkey
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
    let guard = HKeyGuard(hkey);

    for value_name in location.value_names {
        if let Some(combo) = read_value_and_parse(guard.0, value_name, location.parser) {
            return Some(combo);
        }
    }

    None
}

fn read_value_and_parse(
    hkey: HKEY,
    value_name: &str,
    parser: fn(&str) -> Option<KeyCombination>,
) -> Option<KeyCombination> {
    let value_name_wide: Vec<u16> = value_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let mut data: [u16; 128] = [0; 128];
    let mut data_size = std::mem::size_of_val(&data) as u32;
    let mut reg_type = REG_VALUE_TYPE(0);

    // SAFETY: data buffer is sized and writable; data_size carries its capacity
    // in bytes per RegQueryValueExW's contract.
    let result = unsafe {
        RegQueryValueExW(
            hkey,
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
    let value_str = String::from_utf16_lossy(&data[..str_len]);
    parser(&value_str)
}

fn parse_language_hotkey(value: &str) -> Option<KeyCombination> {
    match value.trim() {
        "1" => Some(KeyCombination::from_keys(vec![VK_MENU, VK_SHIFT])),
        "2" => Some(KeyCombination::from_keys(vec![VK_CONTROL, VK_SHIFT])),
        "3" => None,
        "4" => Some(KeyCombination::new(VK_OEM_3)),
        _ => None,
    }
}

fn parse_win_key_combo(value: &str) -> Option<KeyCombination> {
    let value = value.trim().to_uppercase();

    if value.len() == 1 {
        let ch = value.chars().next()?;
        let vk = char_to_vk(ch)?;
        return Some(KeyCombination::from_keys(vec![VK_LWIN, vk]));
    }

    None
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

    #[test]
    fn test_parse_language_hotkey() {
        assert!(parse_language_hotkey("1").is_some());
        assert!(parse_language_hotkey("2").is_some());
        assert!(parse_language_hotkey("3").is_none());
        assert!(parse_language_hotkey("4").is_some());
        assert!(parse_language_hotkey("unknown").is_none());
    }

    #[test]
    fn test_get_system_hotkey_returns_combinations() {
        assert!(get_system_hotkey(SystemFunction::SwitchLanguage).is_some());
        assert!(get_system_hotkey(SystemFunction::LockWorkstation).is_some());
        assert!(get_system_hotkey(SystemFunction::ShowDesktop).is_some());
        assert!(get_system_hotkey(SystemFunction::TaskManager).is_some());
        assert!(get_system_hotkey(SystemFunction::ToggleCapsLock).is_none());
    }

    #[test]
    fn test_default_combinations() {
        let lang_default = get_default_combination(SystemFunction::SwitchLanguage).unwrap();
        assert!(lang_default.keys.contains(&VK_MENU));
        assert!(lang_default.keys.contains(&VK_SHIFT));

        let lock_default = get_default_combination(SystemFunction::LockWorkstation).unwrap();
        assert!(lock_default.keys.contains(&VK_LWIN));
        assert!(lock_default.keys.contains(&VK_L));
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
        let combo = parse_win_key_combo("L").unwrap();
        assert!(combo.keys.contains(&VK_LWIN));
        assert!(combo.keys.contains(&VK_L));

        let combo2 = parse_win_key_combo("D").unwrap();
        assert!(combo2.keys.contains(&VK_LWIN));
        assert!(combo2.keys.contains(&VK_D));
    }
}
