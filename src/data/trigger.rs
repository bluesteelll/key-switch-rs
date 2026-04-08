//! Resolution mode for a binding: how to decide *when* its action fires
//! given the binding's combo has just been pressed.
//!
//! A binding now owns its trigger as a top-level field. The hot path
//! dispatches every matching binding into either the immediate-execute path
//! (default) or one of the deferred-gesture paths (`Tap`, `Hold`,
//! `DoubleTap`) handled by `crate::hook::tap_state`.
//!
//! Multiple bindings on the same combo with different triggers coexist:
//! e.g. one `Tap(200)` binding and one `Hold(200)` binding on `CapsLock`
//! arm both gestures on key-down and the appropriate one resolves on
//! key-up or timer expiry.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Trigger {
    /// Default. The action fires synchronously on key-down (the existing
    /// behaviour of every binding before triggers existed).
    #[default]
    Immediate,
    /// The action fires on key-up, but only if the key was released within
    /// `<ms>` milliseconds of being pressed. A longer hold cancels the
    /// gesture (this is the "tap-only" half of a tap-hold pair).
    Tap(u64),
    /// The action fires after the key has been held down for `<ms>`
    /// milliseconds without being released. A key-up before the deadline
    /// cancels the gesture (the "hold-only" half).
    Hold(u64),
    /// The action fires on the *second* key-down within `<ms>` of the
    /// first. A single press within the window emits nothing.
    DoubleTap(u64),
}

impl Trigger {
    /// True for the simple "execute now" path. Used by the hot path to
    /// skip the deferred-gesture machinery entirely when no binding needs it.
    pub fn is_immediate(self) -> bool {
        matches!(self, Trigger::Immediate)
    }
}

impl std::fmt::Display for Trigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Trigger::Immediate    => write!(f, "Immediate"),
            Trigger::Tap(ms)      => write!(f, "Tap({}ms)", ms),
            Trigger::Hold(ms)     => write!(f, "Hold({}ms)", ms),
            Trigger::DoubleTap(ms) => write!(f, "DoubleTap({}ms)", ms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_immediate() {
        assert_eq!(Trigger::default(), Trigger::Immediate);
        assert!(Trigger::Immediate.is_immediate());
        assert!(!Trigger::Tap(200).is_immediate());
        assert!(!Trigger::Hold(200).is_immediate());
        assert!(!Trigger::DoubleTap(200).is_immediate());
    }
}
