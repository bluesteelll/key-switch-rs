use windows::Win32::{
    Foundation::*,
    UI::{
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::*,
    },
};

use crate::{core::windows_actions, hook::keyboard_hook::KeyboardHook};

static mut KEYBOARD_HOOK: KeyboardHook = KeyboardHook::new();

pub fn get_hook() -> &'static mut KeyboardHook {
    unsafe {
        let hook = std::ptr::addr_of_mut!(KEYBOARD_HOOK);
        &mut *hook
    }
}

pub unsafe extern "system" fn keyboard_hook_callback(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    unsafe {
        let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);

        if windows_actions::is_injected_event(kb_struct.dwExtraInfo) {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let vk_code = VIRTUAL_KEY(kb_struct.vkCode as u16);
        let is_key_down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;

        let hook = get_hook();

        if is_key_down {
            if handle_key_down(hook, vk_code) {
                return LRESULT(1);
            }
        } else {
            if handle_key_up(hook, vk_code) {
                return LRESULT(1);
            }
        }

        CallNextHookEx(None, code, wparam, lparam)
    }
}

fn handle_key_down(hook: &mut KeyboardHook, vk_code: VIRTUAL_KEY) -> bool {
    let vk_index = vk_code.0 as usize;

    if vk_index < 256 {
        hook.active_keys[vk_index].store(true, std::sync::atomic::Ordering::Release);
    }

    let active_keys = get_active_keys(&hook.active_keys);

    for binding in &hook.bindings {
        if binding.combination.matches(&active_keys) {
            binding.execute();

            if binding.block_default {
                hook.blocked_key.store(vk_code.0, std::sync::atomic::Ordering::Release);
                return true;
            }

            return false;
        }
    }

    false
}

fn handle_key_up(hook: &mut KeyboardHook, vk_code: VIRTUAL_KEY) -> bool {
    let vk_index = vk_code.0 as usize;

    if vk_index < 256 {
        hook.active_keys[vk_index].store(false, std::sync::atomic::Ordering::Release);
    }

    let result = hook.blocked_key.compare_exchange(
        vk_code.0,
        0,
        std::sync::atomic::Ordering::AcqRel,
        std::sync::atomic::Ordering::Acquire,
    );

    result.is_ok()
}

fn get_active_keys(mask: &[std::sync::atomic::AtomicBool; 256]) -> Vec<VIRTUAL_KEY> {
    let mut keys = Vec::new();

    for vk in 0..256u16 {
        if !mask[vk as usize].load(std::sync::atomic::Ordering::Acquire) {
            continue;
        }

        let normalized_vk = match VIRTUAL_KEY(vk) {
            VK_LSHIFT | VK_RSHIFT => VK_SHIFT,
            VK_LCONTROL | VK_RCONTROL => VK_CONTROL,
            VK_LMENU | VK_RMENU => VK_MENU,
            VK_LWIN | VK_RWIN => VK_LWIN,
            other => other,
        };

        if !keys.contains(&normalized_vk) {
            keys.push(normalized_vk);
        }
    }

    keys
}
