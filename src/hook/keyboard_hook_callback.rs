use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use windows::Win32::{
    Foundation::*,
    UI::{
        Accessibility::HWINEVENTHOOK,
        Input::KeyboardAndMouse::*,
        WindowsAndMessaging::*,
    },
};

use crate::core::constants;
use crate::data::binding::BindingKind;
use crate::data::condition::ForegroundContext;
use crate::data::trigger::Trigger;
use crate::hook::chord_state::{mark_completed_keys_blocked, ChordOutcome};
use crate::hook::keyboard_hook::KeyboardHook;
use crate::hook::sequence_state::SequenceOutcome;

/// Process-wide hook singleton, lazily initialised on first access. `LazyLock`
/// rather than a plain `static` because `KeyboardHook::new()` allocates the
/// `ArcSwap` heap slot for the bindings list and can no longer be `const`.
static KEYBOARD_HOOK: LazyLock<KeyboardHook> = LazyLock::new(KeyboardHook::new);

/// Shared reference to the process-wide hook singleton. Returning `&'static`
/// (not `&'static mut`) keeps re-entrant callback invocations sound: multiple
/// shared references may coexist without violating Rust's aliasing rules. All
/// mutable state lives behind atomics or an `ArcSwap` inside `KeyboardHook`.
pub fn get_hook() -> &'static KeyboardHook {
    &KEYBOARD_HOOK
}

/// Modifier virtual keys we keep in sync with physical state. These are the
/// keys most prone to getting stuck after a focus transition into a
/// higher-integrity window (Task Manager / UAC / lock screen) that swallowed
/// the corresponding key-up event.
const MODIFIERS: &[VIRTUAL_KEY] = &[
    VK_SHIFT,    VK_LSHIFT,   VK_RSHIFT,
    VK_CONTROL,  VK_LCONTROL, VK_RCONTROL,
    VK_MENU,     VK_LMENU,    VK_RMENU,
    VK_LWIN,     VK_RWIN,
];

/// Bounded buffer of normalized active keys, used per-keystroke to evaluate
/// bindings without allocating on the hot path. 16 simultaneous distinct keys
/// covers any realistic combination; extras past the cap are silently dropped,
/// which is acceptable because we sort bindings by specificity (longer combos
/// match first) and no real binding uses more than a handful of keys.
const ACTIVE_KEYS_CAP: usize = 16;

struct ActiveKeys {
    keys: [VIRTUAL_KEY; ACTIVE_KEYS_CAP],
    len: usize,
}

impl ActiveKeys {
    fn new() -> Self {
        Self {
            keys: [VIRTUAL_KEY(0); ACTIVE_KEYS_CAP],
            len: 0,
        }
    }

    fn push_unique(&mut self, vk: VIRTUAL_KEY) {
        if self.len >= ACTIVE_KEYS_CAP {
            return;
        }
        for i in 0..self.len {
            if self.keys[i] == vk {
                return;
            }
        }
        self.keys[self.len] = vk;
        self.len += 1;
    }

    fn as_slice(&self) -> &[VIRTUAL_KEY] {
        &self.keys[..self.len]
    }
}

/// Reconcile the modifier bits in `active_keys` with `GetAsyncKeyState` ground
/// truth. `skip_vk` is excluded because MSDN warns that the async state of the
/// key currently being delivered to a low-level hook is not yet updated.
fn sync_modifiers(active_keys: &[AtomicBool; 256], skip_vk: Option<VIRTUAL_KEY>) {
    for &m in MODIFIERS {
        if Some(m) == skip_vk {
            continue;
        }
        // SAFETY: GetAsyncKeyState is thread-safe and has no preconditions
        // beyond a valid vkey in [0, 254]. All MODIFIERS satisfy that.
        let raw = unsafe { GetAsyncKeyState(m.0 as i32) };
        let down = (raw as u16) & 0x8000 != 0;
        active_keys[m.0 as usize].store(down, Ordering::Release);
    }
}

fn mark_blocked(blocked: &[AtomicU64; 4], vk: u16) {
    let idx = (vk as usize) / 64;
    let bit = 1u64 << ((vk as usize) % 64);
    if idx < blocked.len() {
        blocked[idx].fetch_or(bit, Ordering::AcqRel);
    }
}

/// Atomically clears the `vk` bit and reports whether it was previously set.
fn take_blocked(blocked: &[AtomicU64; 4], vk: u16) -> bool {
    let idx = (vk as usize) / 64;
    let bit = 1u64 << ((vk as usize) % 64);
    if idx >= blocked.len() {
        return false;
    }
    let prev = blocked[idx].fetch_and(!bit, Ordering::AcqRel);
    (prev & bit) != 0
}

fn clear_all_blocked(blocked: &[AtomicU64; 4]) {
    for slot in blocked {
        slot.store(0, Ordering::Release);
    }
}

pub(crate) unsafe extern "system" fn keyboard_hook_callback(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if code < 0 {
        return unsafe { CallNextHookEx(None, code, wparam, lparam) };
    }

    unsafe {
        let kb_struct = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

        if constants::is_injected_event(kb_struct.dwExtraInfo) {
            return CallNextHookEx(None, code, wparam, lparam);
        }

        let vk_code = VIRTUAL_KEY(kb_struct.vkCode as u16);
        let hook = get_hook();

        // Repair phantom-held modifiers from any prior focus transition before
        // matching. We skip the just-delivered vk because its async state lags
        // the callback by design (see MSDN remarks on LowLevelKeyboardProc).
        sync_modifiers(&hook.active_keys, Some(vk_code));

        let is_key_down = wparam.0 == WM_KEYDOWN as usize || wparam.0 == WM_SYSKEYDOWN as usize;
        let is_key_up = wparam.0 == WM_KEYUP as usize || wparam.0 == WM_SYSKEYUP as usize;

        if is_key_down {
            if handle_key_down(hook, vk_code) {
                return LRESULT(1);
            }
        } else if is_key_up && handle_key_up(hook, vk_code) {
            return LRESULT(1);
        }

        CallNextHookEx(None, code, wparam, lparam)
    }
}

/// Foreground-window-change callback installed via `SetWinEventHook`.
/// Resyncs modifier state and clears any stuck blocked-key bits, repairing
/// the invariant that the keyboard hook may have lost while a higher-integrity
/// window held focus.
pub(crate) unsafe extern "system" fn foreground_changed(
    _hook: HWINEVENTHOOK,
    _event: u32,
    _hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _thread: u32,
    _time: u32,
) {
    let hook = get_hook();
    sync_modifiers(&hook.active_keys, None);
    // Bits whose key-up we never observed (because focus was elsewhere) would
    // otherwise suppress the next legitimate release.
    clear_all_blocked(&hook.blocked_keys);
}

fn handle_key_down(hook: &KeyboardHook, vk_code: VIRTUAL_KEY) -> bool {
    let vk_index = vk_code.0 as usize;

    // Detect OS auto-repeat (key was already down before this event).
    // Sequence/Chord state machines must ignore repeats — they expect
    // deliberate presses. Combo-kind bindings still fire on every repeat
    // (existing behaviour; for the Launch action that's debatable, but
    // typing-style PressKey actions rely on it).
    let was_repeat = if vk_index < 256 {
        hook.active_keys[vk_index].swap(true, Ordering::AcqRel)
    } else {
        false
    };

    let active_keys = collect_active_keys(&hook.active_keys);

    // Snapshot the binding list under a single ArcSwap load — the Arc keeps
    // it stable for the rest of this callback even if a hot-reload swaps the
    // list mid-execution.
    let bindings = hook.bindings();

    // Only collect foreground info (which costs a few Win32 calls) when at
    // least one binding actually gates on it. The common case — no `when:`
    // anywhere — pays nothing.
    let ctx: Option<ForegroundContext> = if bindings.iter().any(|b| !b.condition.is_always()) {
        Some(ForegroundContext::capture())
    } else {
        None
    };

    let mut suppress = false;
    let mut immediate_fired = false;

    for binding in bindings.iter() {
        if let Some(ctx) = &ctx
            && !binding.condition.evaluate(ctx)
        {
            continue;
        }

        match &binding.kind {
            BindingKind::Combo(combo) => {
                if !combo.matches(active_keys.as_slice()) {
                    continue;
                }
                match binding.trigger {
                    Trigger::Immediate => {
                        if !immediate_fired {
                            binding.execute();
                            immediate_fired = true;
                        }
                        if binding.block_default {
                            suppress = true;
                        }
                    }
                    Trigger::Tap(term_ms) => {
                        if was_repeat { continue; }
                        hook.tap_state.arm_tap(vk_code, binding.action.clone(), term_ms);
                        suppress = true;
                    }
                    Trigger::Hold(term_ms) => {
                        if was_repeat { continue; }
                        hook.tap_state.arm_hold(vk_code, binding.action.clone(), term_ms);
                        suppress = true;
                    }
                    Trigger::DoubleTap(term_ms) => {
                        if was_repeat { continue; }
                        hook.tap_state.handle_double_tap(vk_code, binding.action.clone(), term_ms);
                        suppress = true;
                    }
                }
            }
            BindingKind::Sequence { steps, max_gap } => {
                if was_repeat { continue; }
                match hook.sequence_state.handle_keydown(
                    active_keys.as_slice(),
                    &binding.action,
                    steps,
                    *max_gap,
                ) {
                    SequenceOutcome::NotMatching => {}
                    SequenceOutcome::Advanced => {
                        suppress = true;
                    }
                    SequenceOutcome::Completed { last_step_keys } => {
                        suppress = true;
                        mark_completed_keys_blocked(&hook.blocked_keys, &last_step_keys);
                    }
                }
            }
            BindingKind::Chord { keys, window } => {
                if was_repeat { continue; }
                match hook.chord_state.handle_keydown(
                    vk_code,
                    &binding.action,
                    keys,
                    *window,
                ) {
                    ChordOutcome::NotInChord => {}
                    ChordOutcome::Suppress { completed_keys: None } => {
                        suppress = true;
                    }
                    ChordOutcome::Suppress { completed_keys: Some(fired) } => {
                        suppress = true;
                        mark_completed_keys_blocked(&hook.blocked_keys, &fired);
                    }
                }
            }
        }
    }

    if suppress {
        mark_blocked(&hook.blocked_keys, vk_code.0);
        return true;
    }

    false
}

fn handle_key_up(hook: &KeyboardHook, vk_code: VIRTUAL_KEY) -> bool {
    let vk_index = vk_code.0 as usize;

    if vk_index < 256 {
        hook.active_keys[vk_index].store(false, Ordering::Release);
    }

    // Resolve every deferred gesture pending for this key. Tap fires its
    // action here (if still within window); Hold simply cancels (key
    // released too early). Both gestures can be pending simultaneously on
    // the same key — they were armed independently and are resolved
    // independently.
    let tap_fired = hook.tap_state.resolve_tap_on_keyup(vk_code);
    let hold_cancelled = hook.tap_state.cancel_hold_on_keyup(vk_code);

    if tap_fired || hold_cancelled {
        // Gesture-bound key: suppress the key-up too. Clear any blocked
        // bit set on the corresponding key-down so it doesn't leak.
        let _ = take_blocked(&hook.blocked_keys, vk_code.0);
        return true;
    }

    take_blocked(&hook.blocked_keys, vk_code.0)
}

fn collect_active_keys(mask: &[AtomicBool; 256]) -> ActiveKeys {
    let mut keys = ActiveKeys::new();

    for vk in 0..256u16 {
        if !mask[vk as usize].load(Ordering::Acquire) {
            continue;
        }

        let normalized_vk = match VIRTUAL_KEY(vk) {
            VK_LSHIFT | VK_RSHIFT => VK_SHIFT,
            VK_LCONTROL | VK_RCONTROL => VK_CONTROL,
            VK_LMENU | VK_RMENU => VK_MENU,
            VK_LWIN | VK_RWIN => VK_LWIN,
            other => other,
        };

        keys.push_unique(normalized_vk);
    }

    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_keys_dedup() {
        let mut buf = ActiveKeys::new();
        buf.push_unique(VK_SHIFT);
        buf.push_unique(VK_CONTROL);
        buf.push_unique(VK_SHIFT);
        assert_eq!(buf.as_slice(), &[VK_SHIFT, VK_CONTROL]);
    }

    #[test]
    fn active_keys_capacity_bound() {
        let mut buf = ActiveKeys::new();
        for i in 0..(ACTIVE_KEYS_CAP as u16 + 4) {
            buf.push_unique(VIRTUAL_KEY(i));
        }
        assert_eq!(buf.as_slice().len(), ACTIVE_KEYS_CAP);
    }

    #[test]
    fn blocked_bitmap_independent_bits() {
        let blocked: [AtomicU64; 4] = [
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
        ];
        mark_blocked(&blocked, 20);   // VK_CAPITAL, slot 0
        mark_blocked(&blocked, 16);   // VK_SHIFT,  slot 0
        mark_blocked(&blocked, 200);  // slot 3

        // Releasing VK_CAPITAL must not touch VK_SHIFT or 200.
        assert!(take_blocked(&blocked, 20));
        assert!(!take_blocked(&blocked, 20)); // second take is a no-op
        assert!(take_blocked(&blocked, 16));
        assert!(take_blocked(&blocked, 200));
    }

    #[test]
    fn clear_all_resets_every_slot() {
        let blocked: [AtomicU64; 4] = [
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
        ];
        mark_blocked(&blocked, 20);
        mark_blocked(&blocked, 200);
        clear_all_blocked(&blocked);
        assert!(!take_blocked(&blocked, 20));
        assert!(!take_blocked(&blocked, 200));
    }
}
