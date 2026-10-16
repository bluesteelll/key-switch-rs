use std::time::Duration;

use super::condition::Condition;
use super::key_combination::KeyCombination;
use super::key_spec::KeySpec;
use super::trigger::Trigger;
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
        keys: Vec<KeySpec>,
        window: Duration,
    },
}

impl BindingKind {
    /// Specificity ranking used by `KeyboardHook` to sort the bindings, and
    /// the first part of a `keys` binding's precedence when several match
    /// one key press (`engine::winner`, roadmap item 1.6): more keys win.
    /// For Sequence/Chord it returns step/key count so they sort alongside
    /// multi-key combos sensibly, even though sequences and chords take no
    /// part in that election.
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
    /// Whether the key events that fire or arm a `Combo` binding are
    /// suppressed, for every trigger. The constructors set `true`; the
    /// loader resolves a config without the field with
    /// [`Binding::blocks_by_default`]. Sequences and chords ignore it.
    pub block_default: bool,
    pub block_original_combo: bool,
    /// Foreground-context guard. `Condition::Always` (the default) makes the
    /// binding fire whenever the trigger matches; anything else gates it
    /// on the active window's exe name / title.
    pub condition: Condition,
    /// Resolution mode — when the action fires given a matching trigger.
    /// `Immediate` (default) means "fire on the triggering event"; the
    /// other variants defer to the gestures in `crate::engine`. Only
    /// meaningful for `BindingKind::Combo`; the loader rejects deferred
    /// triggers paired with `Sequence`/`Chord` (which have their own
    /// temporal semantics).
    pub trigger: Trigger,
    /// Lets a `Combo` binding fire while modifiers it does not name are held
    /// too. `false` (the default) requires the held modifier families to be
    /// exactly the combo's own. The loader rejects `true` on `Sequence` and
    /// `Chord`. Auto-blockers ignore it: they match only their hotkey on its
    /// own.
    pub allow_extra_modifiers: bool,
    /// Whether an `Immediate` `Combo` binding fires again on every
    /// auto-repeat of its key. Without it, the binding fires once per press,
    /// and its repeats are still suppressed when its key-down was. The
    /// constructors take the default from the action
    /// ([`BindAction::repeats_by_default`]). Deferred triggers, sequences and
    /// chords never fire on an auto-repeat, whatever it says; the loader
    /// rejects `true` on them.
    pub repeat: bool,
    /// Set only by [`Binding::new_auto_blocker`]. The engine matches such a
    /// binding by what Windows sees held and lets it only suppress: it fires
    /// nothing.
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
            repeat: action.repeats_by_default(),
            action,
            block_default: true,
            block_original_combo: false,
            condition: Condition::Always,
            trigger: Trigger::Immediate,
            allow_extra_modifiers: false,
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
            allow_extra_modifiers: false,
            repeat: false,
            is_auto_blocker: true,
        }
    }

    /// The `block_default` of a binding whose config does not set it:
    /// `false` for a `Tap`, `Hold` or `DoubleTap` combo made only of
    /// modifiers (`"LShift"`, `"Ctrl+Shift"`), which would otherwise take the
    /// modifier away from everything else it is used for; `true` for every
    /// other binding, every `Immediate` one included (roadmap item 1.3).
    pub fn blocks_by_default(kind: &BindingKind, trigger: Trigger) -> bool {
        match kind {
            BindingKind::Combo(combo) => trigger.is_immediate() || !combo.is_modifier_only(),
            BindingKind::Sequence { .. } | BindingKind::Chord { .. } => true,
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

    pub fn with_block_default(mut self, block: bool) -> Self {
        self.block_default = block;
        self
    }

    pub fn with_block_original_combo(mut self, block: bool) -> Self {
        self.block_original_combo = block;
        self
    }

    pub fn with_condition(mut self, condition: Condition) -> Self {
        self.condition = condition;
        self
    }

    pub fn with_trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn with_allow_extra_modifiers(mut self, allow: bool) -> Self {
        self.allow_extra_modifiers = allow;
        self
    }

    /// Overrides the auto-repeat default that the constructor took from the
    /// action.
    pub fn with_repeat(mut self, repeat: bool) -> Self {
        self.repeat = repeat;
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
        let extra_suffix = if self.allow_extra_modifiers {
            "  [allow_extra_modifiers]"
        } else {
            ""
        };
        // Only a value that differs from the action's default is shown, so
        // a config that never sets `repeat:` lists as before. The flag means
        // nothing to deferred triggers, sequences and chords.
        let honours_repeat =
            matches!(self.kind, BindingKind::Combo(_)) && self.trigger.is_immediate();
        let repeat_suffix = match (self.repeat, self.action.repeats_by_default()) {
            (true, false) if honours_repeat => "  [repeat]",
            (false, true) if honours_repeat => "  [no repeat]",
            _ => "",
        };

        if self.is_auto_blocker {
            write!(
                f,
                "[AUTO-BLOCK] {:<24} -> (blocked){}{}{}",
                keys_str, cond_suffix, trigger_suffix, extra_suffix
            )
        } else {
            write!(
                f,
                "{:<34} -> {}{}{}{}{}",
                keys_str, self.action, cond_suffix, trigger_suffix, extra_suffix, repeat_suffix
            )
        }
    }
}

fn format_kind(kind: &BindingKind) -> String {
    match kind {
        BindingKind::Combo(c) => c
            .keys
            .iter()
            .map(KeySpec::to_string)
            .collect::<Vec<_>>()
            .join(" + "),
        BindingKind::Sequence { steps, max_gap } => {
            let rendered: Vec<String> = steps.iter().map(KeyCombination::to_string).collect();
            format!("[Seq <{}ms>] {}", max_gap.as_millis(), rendered.join(" → "))
        }
        BindingKind::Chord { keys, window } => {
            let rendered: Vec<String> = keys.iter().map(KeySpec::to_string).collect();
            format!("[Chord <{}ms>] {}", window.as_millis(), rendered.join("+"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::key_spec::Modifier;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn display_marks_allow_extra_modifiers() {
        let exact = Binding::new(KeyCombination::new(VK_CAPITAL), BindAction::SwitchLanguage);
        let relaxed = exact.clone().with_allow_extra_modifiers(true);

        let exact_line = exact.to_string();
        assert!(exact_line.starts_with("CapsLock"), "{exact_line}");
        assert!(
            !exact_line.contains("allow_extra_modifiers"),
            "{exact_line}"
        );
        let relaxed_line = relaxed.to_string();
        assert!(
            relaxed_line.ends_with("  [allow_extra_modifiers]"),
            "{relaxed_line}"
        );
    }

    #[test]
    fn a_binding_takes_its_repeat_default_from_its_action() {
        let press_left = Binding::new(KeyCombination::new(VK_F13), BindAction::PressKey(VK_LEFT));
        assert!(press_left.repeat, "PressKey repeats by default");
        assert!(!press_left.with_repeat(false).repeat);

        let switch = Binding::new(KeyCombination::new(VK_F13), BindAction::SwitchLanguage);
        assert!(!switch.repeat, "SwitchLanguage does not repeat by default");
        assert!(switch.with_repeat(true).repeat);

        assert!(!Binding::new_auto_blocker(KeyCombination::new(VK_F13)).repeat);
    }

    #[test]
    fn blocks_by_default_is_false_only_for_a_deferred_combo_made_of_modifiers() {
        let combo =
            |keys: &[VIRTUAL_KEY]| BindingKind::Combo(KeyCombination::from_keys(keys.to_vec()));
        let deferred = [
            Trigger::Tap(200),
            Trigger::Hold(200),
            Trigger::DoubleTap(250),
        ];

        for trigger in deferred {
            assert!(!Binding::blocks_by_default(&combo(&[VK_LSHIFT]), trigger));
            assert!(!Binding::blocks_by_default(
                &combo(&[VK_CONTROL, VK_SHIFT]),
                trigger
            ));
            let win = BindingKind::Combo(KeyCombination::new(KeySpec::AnySide(Modifier::Win)));
            assert!(!Binding::blocks_by_default(&win, trigger));

            assert!(Binding::blocks_by_default(&combo(&[VK_CAPITAL]), trigger));
            assert!(Binding::blocks_by_default(
                &combo(&[VK_SHIFT, VK_CAPITAL]),
                trigger
            ));
        }

        assert!(Binding::blocks_by_default(
            &combo(&[VK_LSHIFT]),
            Trigger::Immediate
        ));
        assert!(Binding::blocks_by_default(
            &combo(&[VK_MENU, VK_SHIFT]),
            Trigger::Immediate
        ));

        let steps = vec![KeyCombination::new(VK_F13), KeyCombination::new(VK_F14)];
        let sequence = BindingKind::Sequence {
            steps,
            max_gap: Duration::from_millis(500),
        };
        assert!(Binding::blocks_by_default(&sequence, Trigger::Immediate));
        let chord = BindingKind::Chord {
            keys: vec![KeySpec::Key(VK_F13), KeySpec::Key(VK_F14)],
            window: Duration::from_millis(50),
        };
        assert!(Binding::blocks_by_default(&chord, Trigger::Immediate));
    }
}
