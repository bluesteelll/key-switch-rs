use std::sync::OnceLock;

use windows::Win32::System::Threading::GetCurrentProcessId;

static SENTINEL: OnceLock<usize> = OnceLock::new();

/// Per-process sentinel placed in `KEYBDINPUT::dwExtraInfo` for every key event
/// this daemon synthesizes. The hook callback uses it to short-circuit our own
/// injections and avoid feedback loops.
///
/// The value mixes a constant marker with the process ID so it is
/// extraordinarily unlikely to collide with `dwExtraInfo` values used by other
/// applications that inject input (e.g. AHK, PowerToys), which historically
/// settled on small constants like `0xDEADBEEF` or `0`.
pub fn injected_sentinel() -> usize {
    *SENTINEL.get_or_init(|| {
        // SAFETY: GetCurrentProcessId has no preconditions.
        let pid = unsafe { GetCurrentProcessId() };
        #[cfg(target_pointer_width = "64")]
        {
            ((pid as usize) << 32) | 0xDEAD_BEEFusize
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            (pid as usize).rotate_left(16) ^ 0xDEAD_BEEFusize
        }
    })
}

pub fn is_injected_event(extra_info: usize) -> bool {
    extra_info == injected_sentinel()
}

/// The menu mask key (roadmap item 1.2, KS-12): the virtual-key code the
/// hook sends, down and up, when it swallows a key event while Windows has a
/// Win or Alt key down whose key-down it saw. Windows then sees a key
/// between that press and its release, so the release neither opens the
/// Start menu nor activates the menu bar.
///
/// 0xE8 is unassigned in Microsoft's virtual-key table and maps to no
/// character in any layout; AutoHotkey recommends it for its `#MenuMaskKey`
/// and warns that Microsoft may assign it some day. Applications and other
/// hooks see it as an injected key. Its events carry [`injected_sentinel`],
/// so this daemon's hook passes them on without deciding them.
///
/// A `u16`, not a `VIRTUAL_KEY`: the code becomes a key only where the hook
/// adapter builds its `INPUT` records.
pub const MENU_MASK_VK: u16 = 0xE8;
