#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "manual: run it at the unlocked desktop (not over Remote Desktop with the session locked); it expects Ok(true): this desktop receives keyboard input"]
    fn input_is_on_this_desktop_while_a_human_uses_it() {
        assert_eq!(input_is_on_this_desktop(), Ok(true));
    }
}
