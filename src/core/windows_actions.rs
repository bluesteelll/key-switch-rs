use windows::Win32::{
        Foundation::*,
        UI::{
            Input::KeyboardAndMouse::*,
            WindowsAndMessaging::*,
        },
    };

use crate::data::key_combination::KeyCombination;
use crate::system::system_function::SystemFunction;

const INJECTED_EXTRA_INFO: usize = 0xDEADBEEF;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindAction {
    SwitchLanguage,
    SwitchLanguageBackward,
    ToggleCapsLock,
    PressKey(VIRTUAL_KEY),
    PostMessage { msg: u32, wparam: usize, lparam: isize },
    DoNothing,
}

impl BindAction {
    pub fn execute(&self) {
        if let Some(sys_func) = self.to_system_function() {
            sys_func.execute();
            return;
        }

        match self {
            BindAction::PressKey(vk) => Self::press_key(*vk),
            BindAction::PostMessage { msg, wparam, lparam } => {
                Self::post_message_to_foreground(*msg, *wparam, *lparam)
            }
            BindAction::DoNothing => {},
            _ => {},
        }
    }

    pub fn to_system_function(&self) -> Option<SystemFunction> {
        match self {
            BindAction::SwitchLanguage => Some(SystemFunction::SwitchLanguage),
            BindAction::SwitchLanguageBackward => Some(SystemFunction::SwitchLanguageBackward),
            BindAction::ToggleCapsLock => Some(SystemFunction::ToggleCapsLock),
            _ => None,
        }
    }

    pub fn get_system_combinations(&self) -> Vec<KeyCombination> {
        self.to_system_function()
            .and_then(|sf| sf.get_system_combination())
            .into_iter()
            .collect()
    }

    fn press_key(vk: VIRTUAL_KEY) {
        let mut inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYBD_EVENT_FLAGS(0),
                        time: 0,
                        dwExtraInfo: INJECTED_EXTRA_INFO,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: vk,
                        wScan: 0,
                        dwFlags: KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: INJECTED_EXTRA_INFO,
                    },
                },
            },
        ];

        unsafe {
            SendInput(&mut inputs, std::mem::size_of::<INPUT>() as i32);
        }
    }

    fn post_message_to_foreground(msg: u32, wparam: usize, lparam: isize) {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_invalid() {
                return;
            }

            if let Err(e) = PostMessageW(Some(hwnd), msg, WPARAM(wparam), LPARAM(lparam)) {
                eprintln!("Post message error {:#X}: {:?}", msg, e);
            }
        }
    }
}

impl std::fmt::Display for BindAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindAction::SwitchLanguage => write!(f, "switch language"),
            BindAction::SwitchLanguageBackward => write!(f, "switch language backward"),
            BindAction::ToggleCapsLock => write!(f, "toggle CapsLock"),
            BindAction::PressKey(vk) => write!(f, "press key {:?}", vk),
            BindAction::PostMessage { msg, .. } => write!(f, "post message {:#X}", msg),
            BindAction::DoNothing => write!(f, "do nothing"),
        }
    }
}

pub fn is_injected_event(extra_info: usize) -> bool {
    extra_info == INJECTED_EXTRA_INFO
}
