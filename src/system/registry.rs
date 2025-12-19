

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

        SystemFunction::TaskManager => {
            Some(RegistryLocation {
                hkey: HKEY_CURRENT_USER,
                subkey: "Software\\Microsoft\\Windows\\CurrentVersion\\Policies\\System",
                value_names: &["TaskManagerHotkey"],
                parser: parse_task_manager_hotkey,
            })
        }

        SystemFunction::ToggleCapsLock => None,
    }
}

pub fn get_system_hotkey(function: SystemFunction) -> Vec<KeyCombination> {
    if let Some(location) = get_registry_location(function) {
        if let Some(combo) = read_from_registry(&location) {
            println!("[INFO] {:?}: registry combination {:?}", function, combo.keys);
            return vec![combo];
        }

        println!("[INFO] {:?}: using default combination", function);
        get_default_combination(function).into_iter().collect()
    } else {
        vec![]
    }
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

fn read_from_registry(location: &RegistryLocation) -> Option<KeyCombination> {
    unsafe {
        let subkey_wide: Vec<u16> = location.subkey
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut hkey = HKEY::default();

        if RegOpenKeyExW(
            location.hkey,
            PCWSTR(subkey_wide.as_ptr()),
            None,
            KEY_READ,
            &mut hkey,
        ).is_err() {
            return None;
        }

        for value_name in location.value_names {
            if let Some(combo) = read_value_and_parse(hkey, value_name, location.parser) {
                let _ = RegCloseKey(hkey);
                return Some(combo);
            }
        }

        let _ = RegCloseKey(hkey);
        None
    }
}

fn read_value_and_parse(
    hkey: HKEY,
    value_name: &str,
    parser: fn(&str) -> Option<KeyCombination>,
) -> Option<KeyCombination> {
    unsafe {
        let value_name_wide: Vec<u16> = value_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

        let mut data: [u16; 128] = [0; 128];
        let mut data_size = (std::mem::size_of_val(&data)) as u32;
        let mut reg_type = REG_VALUE_TYPE(0);

        let result = RegQueryValueExW(
            hkey,
            PCWSTR(value_name_wide.as_ptr()),
            None,
            Some(&mut reg_type),
            Some(data.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        );

        if result.is_ok() {
            let str_len = data.iter().position(|&c| c == 0).unwrap_or(data.len());
            let value_str = String::from_utf16_lossy(&data[..str_len]);
            parser(&value_str)
        } else {
            None
        }
    }
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

fn parse_task_manager_hotkey(value: &str) -> Option<KeyCombination> {
    let _ = value;
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
        let switch_lang = get_system_hotkey(SystemFunction::SwitchLanguage);
        assert!(!switch_lang.is_empty());

        let lock = get_system_hotkey(SystemFunction::LockWorkstation);
        assert!(!lock.is_empty());

        let desktop = get_system_hotkey(SystemFunction::ShowDesktop);
        assert!(!desktop.is_empty());

        let taskmgr = get_system_hotkey(SystemFunction::TaskManager);
        assert!(!taskmgr.is_empty());

        let caps = get_system_hotkey(SystemFunction::ToggleCapsLock);
        assert!(caps.is_empty());
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
