use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

#[derive(Debug, Clone)]
pub struct KeyCombination {
    pub keys: Vec<VIRTUAL_KEY>,
}

impl PartialEq for KeyCombination {
    fn eq(&self, other: &Self) -> bool {
        if self.keys.len() != other.keys.len() {
            return false;
        }

        for key in &self.keys {
            if !other.keys.contains(key) {
                return false;
            }
        }

        true
    }
}

impl Eq for KeyCombination {}

impl KeyCombination {
    pub fn new(key: VIRTUAL_KEY) -> Self {
        Self { keys: vec![key] }
    }

    #[allow(dead_code)] // Part of the public builder API; also used by tests.
    pub fn with(mut self, key: VIRTUAL_KEY) -> Self {
        if !self.keys.contains(&key) {
            self.keys.push(key);
        }
        self
    }

    pub fn from_keys(keys: Vec<VIRTUAL_KEY>) -> Self {
        let mut deduped: Vec<VIRTUAL_KEY> = Vec::with_capacity(keys.len());
        for k in keys {
            if !deduped.contains(&k) {
                deduped.push(k);
            }
        }
        Self { keys: deduped }
    }

    pub fn matches(&self, pressed_keys: &[VIRTUAL_KEY]) -> bool {
        self.keys.iter().all(|key| pressed_keys.contains(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    #[test]
    fn test_single_key() {
        let combo = KeyCombination::new(VK_CAPITAL);
        assert!(combo.matches(&[VK_CAPITAL]));
        assert!(!combo.matches(&[VK_A]));
        assert!(!combo.matches(&[]));
    }

    #[test]
    fn test_key_combination() {
        let combo = KeyCombination::new(VK_CAPITAL).with(VK_SHIFT);
        assert!(combo.matches(&[VK_CAPITAL, VK_SHIFT]));
        assert!(combo.matches(&[VK_SHIFT, VK_CAPITAL]));
        assert!(!combo.matches(&[VK_CAPITAL]));
        assert!(!combo.matches(&[VK_SHIFT]));
    }

    #[test]
    fn test_extra_keys() {
        let combo = KeyCombination::new(VK_CAPITAL);
        assert!(combo.matches(&[VK_CAPITAL, VK_SHIFT]));
    }
}
