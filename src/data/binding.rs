use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use super::condition::Condition;
use super::key_combination::KeyCombination;
use super::trigger::Trigger;
use super::vk_name::vk_name;
use crate::core::windows_actions::BindAction;

/// What kind of input pattern triggers this binding.
///
/// - `Combo` is the classic "all these keys held at once" rule used by every
///   binding pre-existing this enum. Hot path matches via `KeyCombination`.
/// - `Sequence` is a Vim/Emacs leader-key flow: each step is its own combo,
///   pressed in order with at most `max_gap` between consecutive steps.
///   Keys involved in an in-progress sequence are suppressed from the
///   foreground; if the sequence times out or is broken, those keys are
///   *lost* (no replay). Bind only to keys not used in normal typing.
/// - `Chord` requires the listed keys to all go down within `window` of
///   each other (the QMK "combo" concept — not to be confused with this
///   project's `Combo`). Order doesn't matter.
#[derive(Debug, Clone)]
pub enum BindingKind {
    Combo(KeyCombination),
    Sequence {
        steps: Vec<KeyCombination>,
        max_gap: Duration,
    },
    Chord {
        keys: Vec<VIRTUAL_KEY>,
        window: Duration,
    },
}

impl BindingKind {
    /// Specificity ranking used by `KeyboardHook` to sort combo bindings —
    /// longer combos win the matching loop first. For Sequence/Chord it
    /// returns step/key count so they sort alongside multi-key combos
    /// sensibly, even though sequence/chord resolution doesn't go through
    /// the combo-match loop.
    pub fn key_count(&self) -> usize {
        match self {
            BindingKind::Combo(c) => c.keys.len(),
            BindingKind::Sequence { steps, .. } => steps.len(),
            BindingKind::Chord { keys, .. } => keys.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub kind: BindingKind,
    pub action: BindAction,
    pub block_default: bool,
    pub block_original_combo: bool,
    /// Foreground-context guard. `Condition::Always` (the default) makes the
    /// binding fire whenever the trigger matches; anything else gates it
    /// on the active window's exe name / title.
    pub condition: Condition,
    /// Resolution mode — when the action fires given a matching trigger.
    /// `Immediate` (default) means "fire on the triggering event"; the
    /// other variants defer to `tap_state`. Only meaningful for
    /// `BindingKind::Combo`; the loader rejects deferred triggers paired
    /// with `Sequence`/`Chord` (which have their own temporal semantics).
    pub trigger: Trigger,
    pub(crate) is_auto_blocker: bool,
}

impl Binding {
    #[allow(dead_code)] // Public builder API: callers (including tests) construct
    // bindings via either `new` or `with_kind`.
    pub fn new(combination: KeyCombination, action: BindAction) -> Self {
        Self::with_kind(BindingKind::Combo(combination), action)
    }

    pub fn with_kind(kind: BindingKind, action: BindAction) -> Self {
        Self {
            kind,
            action,
            block_default: true,
            block_original_combo: false,
            condition: Condition::Always,
            trigger: Trigger::Immediate,
            is_auto_blocker: false,
        }
    }

    pub(crate) fn new_auto_blocker(combination: KeyCombination) -> Self {
        Self {
            kind: BindingKind::Combo(combination),
            action: BindAction::DoNothing,
            block_default: true,
            block_original_combo: false,
            condition: Condition::Always,
            trigger: Trigger::Immediate,
            is_auto_blocker: true,
        }
    }

    /// Return the underlying `KeyCombination` if this binding is the simple
    /// (combo) kind. Used by auto-blocker dedup, tests, and any code that
    /// only cares about classic bindings.
    pub fn combination(&self) -> Option<&KeyCombination> {
        match &self.kind {
            BindingKind::Combo(c) => Some(c),
            _ => None,
        }
    }

    #[allow(dead_code)] // Part of the public builder API.
    pub fn with_block_default(mut self, block: bool) -> Self {
        self.block_default = block;
        self
    }

    pub fn with_block_original_combo(mut self, block: bool) -> Self {
        self.block_original_combo = block;
        self
    }

    #[allow(dead_code)] // Part of the public builder API.
    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = condition;
        self
    }

    #[allow(dead_code)] // Part of the public builder API.
    pub fn with_trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn execute(&self) {
        self.action.execute();
    }
}

impl std::fmt::Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keys_str = format_kind(&self.kind);

        let cond_suffix = if self.condition.is_always() {
            String::new()
        } else {
            format!("  [{:?}]", self.condition)
        };
        let trigger_suffix = if self.trigger.is_immediate() {
            String::new()
        } else {
            format!("  @{}", self.trigger)
        };

        if self.is_auto_blocker {
            write!(f, "[AUTO-BLOCK] {:<24} -> (blocked){}{}", keys_str, cond_suffix, trigger_suffix)
        } else {
            write!(f, "{:<34} -> {}{}{}", keys_str, self.action, cond_suffix, trigger_suffix)
        }
    }
}

fn format_kind(kind: &BindingKind) -> String {
    match kind {
        BindingKind::Combo(c) => c
            .keys
            .iter()
            .map(|k| vk_name(*k))
            .collect::<Vec<_>>()
            .join(" + "),
        BindingKind::Sequence { steps, max_gap } => {
            let rendered: Vec<String> = steps
                .iter()
                .map(|s| {
                    s.keys
                        .iter()
                        .map(|k| vk_name(*k))
                        .collect::<Vec<_>>()
                        .join("+")
                })
                .collect();
            format!("[Seq <{}ms>] {}", max_gap.as_millis(), rendered.join(" → "))
        }
        BindingKind::Chord { keys, window } => {
            let rendered: Vec<String> = keys.iter().map(|k| vk_name(*k)).collect();
            format!("[Chord <{}ms>] {}", window.as_millis(), rendered.join("+"))
        }
    }
}
