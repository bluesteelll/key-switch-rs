//! Foreground-context predicates used to gate bindings.
//!
//! A `Condition` decides whether a binding fires given the currently focused
//! window's executable name and title. It reads them through the
//! [`ForegroundInfo`] trait, so the decision core in `crate::engine` can
//! evaluate conditions against a fake foreground in tests. The hook adapter
//! backs the trait with the main thread's foreground cache
//! (`platform::foreground`, roadmap item 2.3): the executable name comes
//! from the cache with no Win32 call, and the title is read at most once
//! per keystroke, only when a title condition asks (`Condition::Always`
//! asks nothing).
//!
//! Every string comparison ignores case with Unicode lowercasing
//! (`str::to_lowercase`), so Cyrillic or Greek titles match in any case.
//! Windows treats exe and path names case-insensitively, and humans don't
//! reliably remember the casing of window titles either.

/// Equality is structural, the condition as written: `AppEquals("Code.exe")`
/// and `AppEquals("code.exe")` differ. The engine uses it to group Tap, Hold
/// and DoubleTap bindings on the same keys, and the loader to find duplicate
/// bindings (roadmap item 1.6).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Condition {
    /// Always true — the binding fires whenever the combo matches. Default
    /// when the config doesn't specify `when:`.
    #[default]
    Always,
    /// True when the file name of the focused window's executable, `.exe`
    /// included, equals the string (case-insensitive, Unicode-aware), so
    /// `"code.exe"` matches `C:\...\Microsoft VS Code\Code.exe`.
    AppEquals(String),
    /// True when the focused window's title contains the substring
    /// (case-insensitive, Unicode-aware). Useful for picking out an app by
    /// tab name even when many windows share the same executable.
    TitleContains(String),
    /// True when the focused window's title is exactly the given string
    /// (case-insensitive, Unicode-aware).
    TitleEquals(String),
    /// Negation.
    Not(Box<Condition>),
    /// Conjunction — all sub-conditions must hold.
    And(Vec<Condition>),
    /// Disjunction — at least one sub-condition must hold.
    Or(Vec<Condition>),
}

impl Condition {
    /// Cheap probe: is this condition guaranteed to skip the foreground
    /// lookup? Used by the hook callback to avoid Win32 calls when no
    /// binding cares about the focused window.
    pub fn is_always(&self) -> bool {
        matches!(self, Condition::Always)
    }

    pub fn evaluate(&self, ctx: &dyn ForegroundInfo) -> bool {
        match self {
            Condition::Always => true,
            Condition::AppEquals(target) => match ctx.app() {
                Some(actual) => equal_ignoring_case(actual, target),
                None => false,
            },
            Condition::TitleContains(needle) => match ctx.title() {
                Some(actual) => actual.to_lowercase().contains(&needle.to_lowercase()),
                None => false,
            },
            Condition::TitleEquals(target) => match ctx.title() {
                Some(actual) => equal_ignoring_case(actual, target),
                None => false,
            },
            Condition::Not(inner) => !inner.evaluate(ctx),
            Condition::And(parts) => parts.iter().all(|c| c.evaluate(ctx)),
            Condition::Or(parts) => parts.iter().any(|c| c.evaluate(ctx)),
        }
    }
}

/// Equality under the lowercasing `TitleContains` uses, so a title that
/// `TitleEquals(s)` accepts also contains `s`. ASCII strings lowercase byte
/// by byte, so two of them compare without allocating.
fn equal_ignoring_case(a: &str, b: &str) -> bool {
    if a.is_ascii() && b.is_ascii() {
        a.eq_ignore_ascii_case(b)
    } else {
        a.to_lowercase() == b.to_lowercase()
    }
}

/// What a [`Condition`] can ask about the focused window. Both answers are
/// `None` when there is no foreground window or it cannot be inspected.
pub trait ForegroundInfo {
    /// Base name of the focused window's executable, e.g. `"chrome.exe"`.
    fn app(&self) -> Option<&str>;
    /// Title bar text of the focused window.
    fn title(&self) -> Option<&str>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A foreground that needs no window: what the engine's tests use.
    struct FakeForeground {
        app: Option<&'static str>,
        title: Option<&'static str>,
    }

    impl ForegroundInfo for FakeForeground {
        fn app(&self) -> Option<&str> {
            self.app
        }

        fn title(&self) -> Option<&str> {
            self.title
        }
    }

    #[test]
    fn leaf_conditions_read_the_foreground_info() {
        let code = FakeForeground {
            app: Some("Code.exe"),
            title: Some("main.rs - Visual Studio Code"),
        };
        assert!(Condition::AppEquals("code.exe".into()).evaluate(&code));
        assert!(!Condition::AppEquals("chrome.exe".into()).evaluate(&code));
        assert!(Condition::TitleContains("visual studio".into()).evaluate(&code));
        assert!(Condition::TitleEquals("MAIN.RS - visual studio code".into()).evaluate(&code));
        assert!(!Condition::TitleEquals("main.rs".into()).evaluate(&code));

        // No foreground window: every leaf is false, so `Not` is true.
        let nothing = FakeForeground {
            app: None,
            title: None,
        };
        assert!(!Condition::AppEquals("code.exe".into()).evaluate(&nothing));
        assert!(!Condition::TitleContains("".into()).evaluate(&nothing));
        assert!(
            Condition::Not(Box::new(Condition::AppEquals("code.exe".into()))).evaluate(&nothing)
        );
    }

    /// No foreground window, as during a desktop switch.
    const NO_WINDOW: FakeForeground = FakeForeground {
        app: None,
        title: None,
    };

    /// `Always` short-circuits and never touches the context.
    #[test]
    fn always_is_true() {
        let ctx = NO_WINDOW;
        assert!(Condition::Always.evaluate(&ctx));
    }

    #[test]
    fn is_always_helper() {
        assert!(Condition::Always.is_always());
        assert!(!Condition::AppEquals("x".into()).is_always());
        assert!(!Condition::Not(Box::new(Condition::Always)).is_always());
    }

    #[test]
    fn not_inverts() {
        let ctx = NO_WINDOW;
        let always_true = Condition::Always;
        let always_false = Condition::Not(Box::new(always_true.clone()));
        assert!(always_true.evaluate(&ctx));
        assert!(!always_false.evaluate(&ctx));
    }

    #[test]
    fn and_short_circuits_on_first_false() {
        let ctx = NO_WINDOW;
        let cond = Condition::And(vec![
            Condition::Not(Box::new(Condition::Always)), // false
            Condition::Always,                           // not evaluated, but must not panic
        ]);
        assert!(!cond.evaluate(&ctx));
    }

    #[test]
    fn or_short_circuits_on_first_true() {
        let ctx = NO_WINDOW;
        let cond = Condition::Or(vec![
            Condition::Always,
            Condition::Not(Box::new(Condition::Always)),
        ]);
        assert!(cond.evaluate(&ctx));
    }

    #[test]
    fn nested_and_or_evaluates_correctly() {
        let ctx = NO_WINDOW;
        // (true AND (false OR true)) = true
        let cond = Condition::And(vec![
            Condition::Always,
            Condition::Or(vec![
                Condition::Not(Box::new(Condition::Always)),
                Condition::Always,
            ]),
        ]);
        assert!(cond.evaluate(&ctx));
    }

    #[test]
    fn equals_conditions_fold_case_like_title_contains() {
        // "Kalkulyator" and "Privet.exe", written in Cyrillic.
        let title = "\u{41a}\u{430}\u{43b}\u{44c}\u{43a}\u{443}\u{43b}\u{44f}\u{442}\u{43e}\u{440}";
        let app = "\u{41f}\u{440}\u{438}\u{432}\u{435}\u{442}.exe";
        let cyrillic = FakeForeground {
            app: Some(app),
            title: Some(title),
        };
        assert_ne!(title.to_uppercase(), title);
        assert!(Condition::TitleContains(title.to_uppercase()).evaluate(&cyrillic));
        assert!(
            Condition::TitleEquals(title.to_uppercase()).evaluate(&cyrillic),
            "TitleEquals ignores Cyrillic case like TitleContains"
        );
        assert!(
            Condition::AppEquals(app.to_uppercase()).evaluate(&cyrillic),
            "AppEquals ignores Cyrillic case"
        );
        assert!(!Condition::TitleEquals("\u{41a}\u{430}\u{43b}".into()).evaluate(&cyrillic));

        // Lowercasing turns a word-final capital sigma into the final form,
        // so a per-char fold would miss what TitleContains finds.
        let greek = FakeForeground {
            app: None,
            title: Some("\u{39f}\u{394}\u{39f}\u{3a3}"),
        };
        let final_sigma = "\u{3bf}\u{3b4}\u{3bf}\u{3c2}";
        assert!(Condition::TitleContains(final_sigma.into()).evaluate(&greek));
        assert!(Condition::TitleEquals(final_sigma.into()).evaluate(&greek));

        // The Kelvin sign lowercases to an ASCII k: one non-ASCII side is
        // enough to need the Unicode comparison.
        let kelvin = FakeForeground {
            app: Some("\u{212a}.exe"),
            title: None,
        };
        assert!(Condition::AppEquals("k.exe".into()).evaluate(&kelvin));
        assert!(Condition::AppEquals("K.EXE".into()).evaluate(&kelvin));
    }
}
