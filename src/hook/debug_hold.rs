#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::WPARAM;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_A, VK_SCROLL};
    use windows::Win32::UI::WindowsAndMessaging::{
        WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    };

    use super::*;

    #[test]
    fn the_hold_key_is_named_like_a_config_key() {
        assert_eq!(parse("ScrollLock"), Some(VK_SCROLL));
        assert_eq!(parse("nope"), None);
        assert_eq!(parse(""), None);
    }

    #[test]
    fn only_a_key_down_of_the_armed_key_holds() {
        let scroll = u32::from(VK_SCROLL.0);
        let message = |msg: u32| WPARAM(msg as usize);
        assert!(should_hold(VK_SCROLL.0, message(WM_KEYDOWN), scroll));
        assert!(should_hold(VK_SCROLL.0, message(WM_SYSKEYDOWN), scroll));
        assert!(!should_hold(VK_SCROLL.0, message(WM_KEYUP), scroll));
        assert!(!should_hold(VK_SCROLL.0, message(WM_SYSKEYUP), scroll));
        assert!(!should_hold(
            VK_SCROLL.0,
            message(WM_KEYDOWN),
            u32::from(VK_A.0)
        ));
        assert!(
            !should_hold(0, message(WM_KEYDOWN), 0),
            "no key armed: nothing holds"
        );
    }
}
