//! Classical ciphers for text challenges. These do not provide secure encryption.

pub mod bacon;
pub mod book;
pub mod classical;
pub mod enigma;
pub mod mono;
pub mod poly;
pub mod polybius;
pub use classical::{
    affine_decode, affine_encode, atbash, beaufort, vigenere_decode, vigenere_encode,
};

/// Encode text by rotating ASCII letters by a signed Caesar shift.
///
/// Positive shifts move toward `Z` or `z`, wrapping within each case. Every other
/// character is preserved, including whitespace, punctuation, and combining
/// characters; no Unicode normalization is performed.
///
/// The shift accepts the full [`i32`] range, from -2,147,483,648 to 2,147,483,647,
/// and is normalized modulo 26. Zero and multiples of 26 leave text unchanged.
/// Use [`caesar_decode`] with the same shift to restore the original text.
/// This classical cipher is for education and challenges, not secure encryption.
///
/// ```
/// assert_eq!(cube_core::ciphers::caesar_encode("Attack at dawn!", 3), "Dwwdfn dw gdzq!");
/// assert_eq!(cube_core::ciphers::caesar_encode("AaZz 世界", -1), "ZzYy 世界");
/// ```
pub fn caesar_encode(input: &str, shift: i32) -> String {
    let shift = shift.rem_euclid(26) as u8;
    input
        .chars()
        .map(|character| {
            let base = match character {
                'a'..='z' => b'a',
                'A'..='Z' => b'A',
                _ => return character,
            };
            char::from(base + (character as u8 - base + shift) % 26)
        })
        .collect()
}

/// Decode a Caesar cipher using the same signed [`i32`] shift used to encode it.
///
/// Only ASCII letters rotate; case and every other character are preserved
/// without normalization. The shift is normalized modulo 26 before inversion,
/// so decoding is safe even for [`i32::MIN`]. This reverses [`caesar_encode`].
///
/// ```
/// use cube_core::ciphers::{caesar_decode, caesar_encode};
/// assert_eq!(caesar_decode("Dwwdfn dw gdzq!", 3), "Attack at dawn!");
/// let encoded = caesar_encode("Hello, 世界!", i32::MIN);
/// assert_eq!(caesar_decode(&encoded, i32::MIN), "Hello, 世界!");
/// ```
pub fn caesar_decode(input: &str, shift: i32) -> String {
    // Normalize before negating: -i32::MIN would overflow.
    caesar_encode(input, -shift.rem_euclid(26))
}

/// Rotate ASCII letters by 13 places, preserving case and every other character.
///
/// Non-ASCII characters are unchanged; no Unicode normalization is performed.
/// Applying ROT13 twice restores the original string.
///
/// ```
/// assert_eq!(cube_core::ciphers::rot13("Hello, 世界!"), "Uryyb, 世界!");
/// ```
pub fn rot13(input: &str) -> String {
    caesar_encode(input, 13)
}

const CAESAR_PARAMETERS: &[crate::operations::Parameter] = &[crate::operations::Parameter {
    name: "shift",
    description: "Signed 32-bit ASCII letter shift, normalized modulo 26.",
    required: true,
    default: None,
    constraints: crate::operations::ParameterType::Integer {
        min: i32::MIN as i64,
        max: i32::MAX as i64,
    },
}];
/// Static metadata for operations implemented in this module.
const PROFILED: [crate::operations::metadata::Entry; 3] = [
    {
        const DECLARATION: crate::operations::Descriptor = crate::operations::Descriptor {
            id: "cipher.caesar.encode",
            category: "ciphers",
            description: "Encode ASCII letters with a signed Caesar shift; preserve other Unicode.",
            input_type: "text",
            output_type: "text",
            parameters: CAESAR_PARAMETERS,
            deterministic: true,
            reversible: Some(true),
            command: "cipher caesar",
            example: "cube cipher caesar --shift 3 --text Hello",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Caesar",
                family: "classical_substitution",
                aliases: &[crate::operations::metadata::Alias {
                    name: "ROT1..ROT25",
                    usage: "Use the corresponding Caesar shift; these are parameter profiles, not extra operation IDs.",
                }],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "ASCII A-Z/a-z separately; preserve case and every other scalar, NUL and whitespace without normalization.",
                    key: "Signed i32 shift reduced modulo 26, including i32::MIN.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[crate::operations::metadata::ResourceBound {
                    name: "input_utf8_bytes",
                    maximum: crate::text::MAX_INPUT as u64,
                    unit: "bytes",
                    scope: crate::operations::metadata::Scope::Cli,
                }],
                resource_notes: "No enforced direct Rust/native-Python input or allocation/work budget; caller-sized processing. CLI UTF-8 input is capped at 1 MiB. Direct output preserves UTF-8 byte length; pipeline adapters enforce their own value/work/parameter budgets.",
                rust_api: "cube_core::ciphers::caesar_encode",
                python_api: "caesar_encode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = crate::operations::Descriptor {
            id: "cipher.caesar.decode",
            category: "ciphers",
            description: "Decode ASCII letters with a signed Caesar shift; preserve other Unicode.",
            input_type: "text",
            output_type: "text",
            parameters: CAESAR_PARAMETERS,
            deterministic: true,
            reversible: Some(true),
            command: "cipher caesar",
            example: "cube cipher caesar --shift 3 --decode --text Hello",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Caesar",
                family: "classical_substitution",
                aliases: &[crate::operations::metadata::Alias {
                    name: "ROT1..ROT25",
                    usage: "Use the corresponding Caesar shift; these are parameter profiles, not extra operation IDs.",
                }],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "ASCII A-Z/a-z separately; preserve case and every other scalar, NUL and whitespace without normalization.",
                    key: "Signed i32 shift reduced modulo 26, including i32::MIN.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[crate::operations::metadata::ResourceBound {
                    name: "input_utf8_bytes",
                    maximum: crate::text::MAX_INPUT as u64,
                    unit: "bytes",
                    scope: crate::operations::metadata::Scope::Cli,
                }],
                resource_notes: "No enforced direct Rust/native-Python input or allocation/work budget; caller-sized processing. CLI UTF-8 input is capped at 1 MiB. Direct output preserves UTF-8 byte length; pipeline adapters enforce their own value/work/parameter budgets.",
                rust_api: "cube_core::ciphers::caesar_decode",
                python_api: "caesar_decode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = crate::operations::Descriptor {
            id: "cipher.rot13",
            category: "ciphers",
            description: "Rotate ASCII letters by 13; preserve other Unicode.",
            input_type: "text",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "cipher rot13",
            example: "cube cipher rot13 --text Hello",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "ROT13",
                family: "classical_substitution",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "ASCII A-Z/a-z separately; preserve case and every other scalar, NUL and whitespace without normalization.",
                    key: "ROT13 is fixed shift 13.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[crate::operations::metadata::ResourceBound {
                    name: "input_utf8_bytes",
                    maximum: crate::text::MAX_INPUT as u64,
                    unit: "bytes",
                    scope: crate::operations::metadata::Scope::Cli,
                }],
                resource_notes: "No enforced direct Rust/native-Python input or allocation/work budget; caller-sized processing. CLI UTF-8 input is capped at 1 MiB. Direct output preserves UTF-8 byte length; pipeline adapters enforce their own value/work/parameter budgets.",
                rust_api: "cube_core::ciphers::rot13",
                python_api: "rot13",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caesar_known_answers_and_alphabet_boundaries() {
        assert_eq!(caesar_encode("Attack at dawn!", 3), "Dwwdfn dw gdzq!");
        assert_eq!(caesar_decode("Dwwdfn dw gdzq!", 3), "Attack at dawn!");
        assert_eq!(
            caesar_encode("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ", 3),
            "defghijklmnopqrstuvwxyzabcDEFGHIJKLMNOPQRSTUVWXYZABC"
        );
        assert_eq!(caesar_encode("AaZz", -1), "ZzYy");
        assert_eq!(caesar_decode("ZzYy", -1), "AaZz");
        assert_eq!(caesar_encode("AaZz", 27), "BbAa");
    }

    #[test]
    fn caesar_handles_full_signed_shift_range() {
        assert_eq!(caesar_encode("AaZz", i32::MIN), "CcBb");
        assert_eq!(caesar_decode("CcBb", i32::MIN), "AaZz");
        assert_eq!(caesar_encode("AaZz", i32::MAX), "XxWw");
        assert_eq!(caesar_decode("XxWw", i32::MAX), "AaZz");
        for shift in [0, 26, -26, 52, -52] {
            assert_eq!(caesar_encode("AaZz", shift), "AaZz");
            assert_eq!(caesar_decode("AaZz", shift), "AaZz");
        }
    }

    #[test]
    fn caesar_preserves_non_letters_without_normalization() {
        let text = "@AZ[ `az{\x7f 123\0\t\r\nOlá, 世界 🧊 e\u{301} K Ａ ﬀ";
        let encoded = "@DC[ `dc{\x7f 123\0\t\r\nRoá, 世界 🧊 h\u{301} K Ａ ﬀ";
        assert_eq!(caesar_encode(text, 3), encoded);
        assert_eq!(caesar_decode(encoded, 3), text);
        assert_eq!(caesar_encode("", i32::MIN), "");
        assert_eq!(caesar_decode("", i32::MAX), "");
    }

    #[test]
    fn caesar_round_trips_and_agrees_with_rot13() {
        let text: String = (0..=127).map(char::from).collect();
        for shift in (-52..=52).chain([i32::MIN, i32::MAX]) {
            let encoded = caesar_encode(&text, shift);
            assert_eq!(caesar_decode(&encoded, shift), text);
            assert_eq!(caesar_encode(&caesar_decode(&text, shift), shift), text);
            assert_eq!(caesar_encode(&text, shift), encoded);
        }
        assert_eq!(caesar_encode(&text, 13), rot13(&text));
        assert_eq!(caesar_decode(&text, 13), rot13(&text));
    }

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

pub mod fractionation;
pub mod transposition;

pub mod stream;
