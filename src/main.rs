mod core;
mod data;
mod hook;
mod system;

use core::app::App;
use binding::Binding;
use key_combination::KeyCombination;
use core::windows_actions::BindAction;
use windows::core::Result;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

use crate::data::{binding, key_combination};

fn main() -> Result<()> {
    App::new()
        .add_binding(
            Binding::new(
                KeyCombination::new(VK_CAPITAL),
                BindAction::SwitchLanguage,
            )
            .with_block_original_combo(true) 
        )
        .add_binding(
            Binding::new(
                KeyCombination::new(VK_CAPITAL).with(VK_SHIFT),
                BindAction::ToggleCapsLock,
            )
        )
        .run()
}
