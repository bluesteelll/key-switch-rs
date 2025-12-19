
use super::key_combination::KeyCombination;
use crate::core::windows_actions::BindAction;

#[derive(Debug, Clone)]
pub struct Binding {
    pub combination: KeyCombination,
    pub action: BindAction,
    pub block_default: bool,
    pub block_original_combo: bool,
    pub(crate) is_auto_blocker: bool,
}

impl Binding {
    pub fn new(combination: KeyCombination, action: BindAction) -> Self {
        Self {
            combination,
            action,
            block_default: true,
            block_original_combo: false,
            is_auto_blocker: false,
        }
    }

    pub(crate) fn new_auto_blocker(combination: KeyCombination) -> Self {
        Self {
            combination,
            action: BindAction::DoNothing,
            block_default: true,
            block_original_combo: false,
            is_auto_blocker: true,
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

    pub fn execute(&self) {
        self.action.execute();
    }
}

impl std::fmt::Display for Binding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let keys_str = self.combination
            .keys
            .iter()
            .map(|k| format!("{:?}", k))
            .collect::<Vec<_>>()
            .join(" + ");

        if self.is_auto_blocker {
            write!(f, "[AUTO-BLOCK] {:<20} → (blocked)", keys_str)
        } else {
            write!(f, "{:<30} → {}", keys_str, self.action)
        }
    }
}
