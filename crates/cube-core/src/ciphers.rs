//! Classical ciphers for text puzzles. These do not provide secure encryption.

/// Rotate ASCII letters by 13 places, preserving case and every other character.
///
/// Non-ASCII characters are unchanged; no Unicode normalization is performed.
/// Applying ROT13 twice restores the original string.
///
/// ```
/// assert_eq!(cube_core::ciphers::rot13("Hello, 世界!"), "Uryyb, 世界!");
/// ```
pub fn rot13(input: &str) -> String {
    input
        .chars()
        .map(|character| match character {
            'a'..='m' | 'A'..='M' => char::from(character as u8 + 13),
            'n'..='z' | 'N'..='Z' => char::from(character as u8 - 13),
            _ => character,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alphabet_and_case() {
        assert_eq!(
            rot13("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"),
            "nopqrstuvwxyzabcdefghijklmNOPQRSTUVWXYZABCDEFGHIJKLM"
        );
        assert_eq!(rot13("Hello, World! 123"), "Uryyb, Jbeyq! 123");
    }

    #[test]
    fn preserves_unicode_and_empty_text() {
        assert_eq!(rot13(""), "");
        assert_eq!(rot13("Olá, 世界 🧊 e\u{301}"), "Byá, 世界 🧊 r\u{301}");
        let text = "Hello\0\n café 🧊";
        assert_eq!(rot13(&rot13(text)), text);
        assert_eq!(rot13(text), rot13(text));
    }
}
