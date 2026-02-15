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
