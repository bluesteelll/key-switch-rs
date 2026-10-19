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

    /// Whether both kinds bind the same keys: `keys` combinations and
    /// `chord` keys in any order, `sequence` steps in order (each step's
    /// keys in any order). `max_gap` and `window` do not count, and two
    /// different kinds never bind the same keys.
    pub fn same_keys(&self, other: &BindingKind) -> bool {
        match (self, other) {
            (BindingKind::Combo(a), BindingKind::Combo(b)) => a == b,
            (BindingKind::Sequence { steps: a, .. }, BindingKind::Sequence { steps: b, .. }) => {
                a == b
            }
            // The loader rejects a chord whose keys overlap, so equal
            // lengths and containment make the same set.
            (BindingKind::Chord { keys: a, .. }, BindingKind::Chord { keys: b, .. }) => {
                a.len() == b.len() && a.iter().all(|key| b.contains(key))
            }
            _ => false,
        }
    }
}

/// What an auto-blocker does when Windows would see its hotkey and nothing
/// else (`block_original_combo`, roadmap item 1.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockerMode {
    /// `block_original_combo: true`: swallow the key-down that completes the
    /// hotkey, and with it that key's auto-repeats and key-up.
    Swallow,
    /// `block_original_combo: MaskOnly` (experimental, KS-05): let the
    /// hotkey through and ask for the menu mask key on that key-down and on
    /// the next key event, so that Windows does not see a clean hotkey.
    MaskOnly,
}

impl std::fmt::Display for BlockerMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            BlockerMode::Swallow => "swallow",
            BlockerMode::MaskOnly => "mask only",
        })
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
    /// The auto-blocker this binding asks the loader for, for the Windows
    /// hotkey of its action's system function: `None` for none
    /// (`block_original_combo: false`, the default).
    pub block_original_combo: Option<BlockerMode>,
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
    /// Set only by [`Binding::new_auto_blocker`], to the mode of the
    /// blocker. The engine matches such a binding by what Windows sees held
    /// and lets it only swallow or mask: it fires nothing.
    pub(crate) auto_blocker: Option<BlockerMode>,
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
            block_original_combo: None,
            condition: Condition::Always,
            trigger: Trigger::Immediate,
            allow_extra_modifiers: false,
            auto_blocker: None,
        }
    }

    /// An auto-blocker for the Windows hotkey `combination`, acting in
    /// every window until [`with_condition`](Self::with_condition) narrows
    /// it.
    pub(crate) fn new_auto_blocker(combination: KeyCombination, mode: BlockerMode) -> Self {
        Self {
            kind: BindingKind::Combo(combination),
            action: BindAction::DoNothing,
            block_default: true,
            block_original_combo: None,
            condition: Condition::Always,
            trigger: Trigger::Immediate,
            allow_extra_modifiers: false,
            repeat: false,
            auto_blocker: Some(mode),
        }
    }

    /// Whether the loader added this binding as an auto-blocker.
    pub(crate) fn is_auto_blocker(&self) -> bool {
        self.auto_blocker.is_some()
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

    /// Whether this binding repeats `earlier` (roadmap item 1.6): neither is
    /// an auto-blocker, and both have the same kind and keys
    /// ([`BindingKind::same_keys`]), the same trigger kind whatever its term
    /// ([`Trigger::same_kind`]), the same `when:` as written and the same
    /// `allow_extra_modifiers`. The action and the other flags do not count.
    /// Of two `keys` bindings like that only the first ever acts; two
    /// sequences or chords like that share one pending entry.
    pub fn duplicates(&self, earlier: &Binding) -> bool {
        !self.is_auto_blocker()
            && !earlier.is_auto_blocker()
            && self.kind.same_keys(&earlier.kind)
            && self.trigger.same_kind(earlier.trigger)
            && self.condition == earlier.condition
            && self.allow_extra_modifiers == earlier.allow_extra_modifiers
    }

    pub fn with_block_default(mut self, block: bool) -> Self {
        self.block_default = block;
        self
    }

    pub fn with_block_original_combo(mut self, mode: Option<BlockerMode>) -> Self {
        self.block_original_combo = mode;
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

        if let Some(mode) = self.auto_blocker {
            let effect = match mode {
                BlockerMode::Swallow => "(blocked)",
                BlockerMode::MaskOnly => "(masked, experimental)",
            };
            write!(
                f,
                "[AUTO-BLOCK] {:<24} -> {}{}{}{}",
                keys_str, effect, cond_suffix, trigger_suffix, extra_suffix
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

        assert!(
            !Binding::new_auto_blocker(KeyCombination::new(VK_F13), BlockerMode::Swallow).repeat
        );
    }

    #[test]
    fn display_marks_a_mask_only_blocker() {
        let alt_shift = KeyCombination::from_keys([VK_MENU, VK_SHIFT]);
        let swallow = Binding::new_auto_blocker(alt_shift.clone(), BlockerMode::Swallow);
        let mask_only = Binding::new_auto_blocker(alt_shift, BlockerMode::MaskOnly);

        let swallow_line = swallow.to_string();
        assert!(swallow_line.starts_with("[AUTO-BLOCK] "), "{swallow_line}");
        assert!(swallow_line.ends_with(" -> (blocked)"), "{swallow_line}");
        let mask_line = mask_only.to_string();
        assert!(mask_line.starts_with("[AUTO-BLOCK] "), "{mask_line}");
        assert!(
            mask_line.ends_with(" -> (masked, experimental)"),
            "{mask_line}"
        );

        let line = mask_only
            .with_condition(Condition::AppEquals("x.exe".to_owned()))
            .to_string();
        assert!(
            line.ends_with(r#" -> (masked, experimental)  [AppEquals("x.exe")]"#),
            "the blocker's when: is shown: {line}"
        );
    }

    #[test]
    fn blocker_mode_display() {
        assert_eq!(BlockerMode::Swallow.to_string(), "swallow");
        assert_eq!(BlockerMode::MaskOnly.to_string(), "mask only");
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

    fn sequence(steps: &[&[VIRTUAL_KEY]], max_gap_ms: u64) -> BindingKind {
        BindingKind::Sequence {
            steps: steps
                .iter()
                .map(|keys| KeyCombination::from_keys(keys.to_vec()))
                .collect(),
            max_gap: Duration::from_millis(max_gap_ms),
        }
    }

    fn chord(keys: &[VIRTUAL_KEY], window_ms: u64) -> BindingKind {
        BindingKind::Chord {
            keys: keys.iter().map(|&vk| KeySpec::from(vk)).collect(),
            window: Duration::from_millis(window_ms),
        }
    }

    #[test]
    fn same_keys_compares_combos_and_chords_as_sets_and_steps_in_order() {
        let ctrl_alt_l = BindingKind::Combo(KeyCombination::from_keys([VK_CONTROL, VK_MENU, VK_L]));
        let alt_ctrl_l = BindingKind::Combo(KeyCombination::from_keys([VK_MENU, VK_CONTROL, VK_L]));
        let lctrl_alt_l =
            BindingKind::Combo(KeyCombination::from_keys([VK_LCONTROL, VK_MENU, VK_L]));
        assert!(ctrl_alt_l.same_keys(&alt_ctrl_l));
        assert!(!ctrl_alt_l.same_keys(&lctrl_alt_l));

        let f13_then_ctrl_f14 = sequence(&[&[VK_F13], &[VK_CONTROL, VK_F14]], 500);
        assert!(f13_then_ctrl_f14.same_keys(&sequence(&[&[VK_F13], &[VK_F14, VK_CONTROL]], 900)));
        assert!(!f13_then_ctrl_f14.same_keys(&sequence(&[&[VK_CONTROL, VK_F14], &[VK_F13]], 500)));
        assert!(!f13_then_ctrl_f14.same_keys(&sequence(&[&[VK_F13]], 500)));

        let f13_f14 = chord(&[VK_F13, VK_F14], 50);
        assert!(f13_f14.same_keys(&chord(&[VK_F14, VK_F13], 80)));
        assert!(!f13_f14.same_keys(&chord(&[VK_F13, VK_F15], 50)));
        assert!(!f13_f14.same_keys(&chord(&[VK_F13, VK_F14, VK_F15], 50)));

        let combo_f13_f14 = BindingKind::Combo(KeyCombination::from_keys([VK_F13, VK_F14]));
        assert!(!f13_f14.same_keys(&combo_f13_f14));
        assert!(!combo_f13_f14.same_keys(&f13_f14));
        assert!(!sequence(&[&[VK_F13], &[VK_F14]], 500).same_keys(&f13_f14));
    }

    #[test]
    fn duplicates_needs_the_same_kind_keys_trigger_kind_when_and_flag() {
        let capslock_tap =
            Binding::new(KeyCombination::new(VK_CAPITAL), BindAction::SwitchLanguage)
                .with_trigger(Trigger::Tap(200));

        // The action, the term and block_default do not count.
        let copy = Binding::new(KeyCombination::new(VK_CAPITAL), BindAction::ToggleCapsLock)
            .with_trigger(Trigger::Tap(300))
            .with_block_default(false);
        assert!(copy.duplicates(&capslock_tap));

        let hold = capslock_tap.clone().with_trigger(Trigger::Hold(200));
        assert!(!hold.duplicates(&capslock_tap));
        let conditional = capslock_tap
            .clone()
            .with_condition(Condition::AppEquals("code.exe".to_owned()));
        assert!(!conditional.duplicates(&capslock_tap));
        let relaxed = capslock_tap.clone().with_allow_extra_modifiers(true);
        assert!(!relaxed.duplicates(&capslock_tap));
        let shift_capslock = Binding::new(
            KeyCombination::from_keys([VK_SHIFT, VK_CAPITAL]),
            BindAction::SwitchLanguage,
        )
        .with_trigger(Trigger::Tap(200));
        assert!(!shift_capslock.duplicates(&capslock_tap));

        let f13_f14 = Binding::with_kind(chord(&[VK_F13, VK_F14], 50), BindAction::DoNothing);
        let f14_f13 = Binding::with_kind(chord(&[VK_F14, VK_F13], 80), BindAction::SwitchLanguage);
        assert!(f14_f13.duplicates(&f13_f14));
    }

    #[test]
    fn an_auto_blocker_duplicates_nothing() {
        let alt_shift = KeyCombination::from_keys([VK_MENU, VK_SHIFT]);
        let blocker = Binding::new_auto_blocker(alt_shift.clone(), BlockerMode::Swallow);
        let user = Binding::new(alt_shift, BindAction::DoNothing);
        assert!(!blocker.duplicates(&user));
        assert!(!user.duplicates(&blocker));
        assert!(!blocker.duplicates(&blocker.clone()));
    }
}
