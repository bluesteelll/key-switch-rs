//! The payload of a `Paste` action or macro step (owner request 2026-10-30,
//! roadmap new feature 9): the phrase as the clipboard holds it, the
//! keystroke that pastes it, and how long the daemon waits before it puts
//! the clipboard back.
//!
//! The loader builds one [`PasteSpec`] per binding, behind an `Arc`, so a
//! firing only bumps its count and the clipboard text is encoded once.
//! Neither `Display` nor `Debug` shows any part of the phrase.

use std::fmt;
use std::ops::RangeInclusive;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY;

use crate::data::sequence::is_modifier;
use crate::data::vk_name::vk_name;

/// The keystroke a `Paste` sends when the config names none.
pub const DEFAULT_KEYS: &str = "Ctrl+V";
/// How long after the keystroke the clipboard is put back when the config
/// sets no `restore_delay`.
pub const DEFAULT_RESTORE_DELAY_MS: u64 = 500;
/// The `restore_delay` a config may set: 50 ms is about the shortest time
/// a local window reads the clipboard in, and 5000 ms is the longest the
/// daemon keeps a phrase on the clipboard for a window that does not answer.
pub const RESTORE_DELAY_MS: RangeInclusive<u64> = 50..=5000;

/// What a `Paste` puts on the clipboard and how it pastes it.
#[derive(Clone, PartialEq, Eq)]
pub struct PasteSpec {
    /// The phrase as `CF_UNICODETEXT`: UTF-16, every line ending with
    /// CR-LF, one NUL at the end.
    units: Box<[u16]>,
    /// How many characters the phrase has, for the startup list.
    chars: usize,
    keys: Box<[VIRTUAL_KEY]>,
    restore_delay: Duration,
}

impl PasteSpec {
    /// A spec for `text`, pasted with `keys`, put back `restore_delay_ms`
    /// after the keystroke; or the tail of the load error, which never
    /// shows the text. The first failure wins.
    pub fn new(text: &str, keys: Vec<VIRTUAL_KEY>, restore_delay_ms: u64) -> Result<Self, String> {
        if text.is_empty() {
            return Err("empty".to_owned());
        }
        if text.contains('\0') {
            return Err(
                "the text contains the character U+0000, which ends a text on the clipboard"
                    .to_owned(),
            );
        }
        if keys.iter().all(|&key| is_modifier(key)) {
            return Err(format!(
                "keys {:?} has no key besides modifiers",
                key_names(&keys)
            ));
        }
        if !RESTORE_DELAY_MS.contains(&restore_delay_ms) {
            return Err(format!(
                "restore_delay: {restore_delay_ms} is out of range: {} to {} ms",
                RESTORE_DELAY_MS.start(),
                RESTORE_DELAY_MS.end()
            ));
        }
        Ok(Self {
            units: clipboard_text(text),
            chars: text.chars().count(),
            keys: keys.into_boxed_slice(),
            restore_delay: Duration::from_millis(restore_delay_ms),
        })
    }

    /// The phrase as `CF_UNICODETEXT`, its NUL included.
    pub fn units(&self) -> &[u16] {
        &self.units
    }

    pub fn chars(&self) -> usize {
        self.chars
    }

    /// The keystroke, as `parse_key_list` gave it: modifiers and keys in
    /// the config's order.
    pub fn keys(&self) -> &[VIRTUAL_KEY] {
        &self.keys
    }

    pub fn restore_delay(&self) -> Duration {
        self.restore_delay
    }
}

/// The names of `keys` joined with `+`, as the config writes them.
fn key_names(keys: &[VIRTUAL_KEY]) -> String {
    keys.iter()
        .map(|&key| vk_name(key))
        .collect::<Vec<_>>()
        .join("+")
}

/// `text` as `CF_UNICODETEXT` wants it: UTF-16, a lone `\n` or a lone `\r`
/// becomes `\r\n`, a `\r\n` stays, and one NUL ends it ("Standard
/// Clipboard Formats": "Each line ends with a carriage return/linefeed
/// (CR-LF) combination. A null character signals the end of the data.").
pub fn clipboard_text(text: &str) -> Box<[u16]> {
    const CR: u16 = 0x0D;
    const LF: u16 = 0x0A;
    let mut units = Vec::with_capacity(text.len() + 1);
    let mut after_cr = false;
    for unit in text.encode_utf16() {
        match unit {
            LF if after_cr => units.push(LF),
            LF => units.extend([CR, LF]),
            CR => {
                if after_cr {
                    units.push(LF);
                }
                units.push(CR);
            }
            _ => {
                if after_cr {
                    units.push(LF);
                }
                units.push(unit);
            }
        }
        after_cr = unit == CR;
    }
    if after_cr {
        units.push(LF);
    }
    units.push(0);
    units.into_boxed_slice()
}

impl fmt::Display for PasteSpec {
    /// The length and the keystroke only: the startup list must not show
    /// the phrase.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "paste ({} chars, {})", self.chars, key_names(&self.keys))
    }
}

impl fmt::Debug for PasteSpec {
    /// Written out so that the phrase never reaches a `{:?}`, an `expect`
    /// or a panic message.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PasteSpec {{ chars: {}, keys: {}, restore_delay: {:?} }}",
            self.chars,
            key_names(&self.keys),
            self.restore_delay
        )
    }
}

#[cfg(test)]
mod tests {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;

    use super::*;

    const CR: u16 = 0x0D;
    const LF: u16 = 0x0A;

    fn units(text: &str) -> Vec<u16> {
        clipboard_text(text).into_vec()
    }

    #[test]
    fn clipboard_text_ends_every_line_with_crlf_and_the_text_with_one_nul() {
        let a = u16::from(b'a');
        let b = u16::from(b'b');
        for text in ["a\nb", "a\r\nb", "a\rb"] {
            assert_eq!(units(text), [a, CR, LF, b, 0], "{text:?}");
        }
        assert_eq!(units("a\n\r"), [a, CR, LF, CR, LF, 0]);
        assert_eq!(units("\r\r\n"), [CR, LF, CR, LF, 0]);
        assert_eq!(units("\n\n"), [CR, LF, CR, LF, 0]);
        assert_eq!(units("\u{1F680}"), [0xD83D, 0xDE80, 0]);
        assert_eq!(units("x"), [u16::from(b'x'), 0]);
    }

    #[test]
    fn a_paste_spec_rejects_what_cannot_go_on_the_clipboard() {
        let ctrl_v = || vec![VK_CONTROL, VK_V];
        assert_eq!(PasteSpec::new("", ctrl_v(), 500), Err("empty".to_owned()));
        assert_eq!(
            PasteSpec::new("a\u{0}b", ctrl_v(), 500),
            Err(
                "the text contains the character U+0000, which ends a text on the clipboard"
                    .to_owned()
            )
        );
        assert_eq!(
            PasteSpec::new("a", vec![VK_CONTROL, VK_SHIFT], 500),
            Err(r#"keys "Ctrl+Shift" has no key besides modifiers"#.to_owned())
        );
        for ms in [0, 49, 5001] {
            assert_eq!(
                PasteSpec::new("a", ctrl_v(), ms),
                Err(format!(
                    "restore_delay: {ms} is out of range: 50 to 5000 ms"
                ))
            );
        }
        for ms in [50, 5000] {
            assert!(PasteSpec::new("a", ctrl_v(), ms).is_ok(), "{ms} ms");
        }
        assert_eq!(
            PasteSpec::new("", vec![VK_CONTROL], 0),
            Err("empty".to_owned()),
            "the first failure wins"
        );
        let spec = PasteSpec::new("Kind regards,\nIvan", vec![VK_SHIFT, VK_INSERT], 800)
            .expect("a valid spec");
        assert_eq!(spec.keys(), [VK_SHIFT, VK_INSERT]);
        assert_eq!(spec.restore_delay(), Duration::from_millis(800));
        assert_eq!(spec.chars(), 18);
        assert_eq!(spec.units(), &*clipboard_text("Kind regards,\nIvan"));
    }

    /// Whether `rendered` holds no 3-character run of `secret`.
    fn shows_no_part_of(rendered: &str, secret: &str) -> bool {
        let chars: Vec<char> = secret.chars().collect();
        chars.windows(3).all(|window| {
            let part: String = window.iter().collect();
            !rendered.contains(&part)
        })
    }

    #[test]
    fn a_paste_spec_shows_its_length_and_keys_and_never_its_text() {
        let secret = "P4ssw0rd-QZX";
        let spec = PasteSpec::new(secret, vec![VK_CONTROL, VK_V], 500).expect("a valid spec");

        assert_eq!(spec.to_string(), "paste (12 chars, Ctrl+V)");
        assert_eq!(
            format!("{spec:?}"),
            "PasteSpec { chars: 12, keys: Ctrl+V, restore_delay: 500ms }"
        );
        for rendered in [
            spec.to_string(),
            format!("{spec:?}"),
            format!("{spec:#?}"),
            format!("{:?}", Some(spec.clone())),
        ] {
            assert!(
                shows_no_part_of(&rendered, secret),
                "a part of the text shows in {rendered}"
            );
        }
        let emoji = PasteSpec::new("\u{1F680}\u{451}", vec![VK_SHIFT, VK_INSERT], 50)
            .expect("a valid spec");
        assert_eq!(
            emoji.to_string(),
            "paste (2 chars, Shift+Insert)",
            "characters, not code units"
        );
    }
}
