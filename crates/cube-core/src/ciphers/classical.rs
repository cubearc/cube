//! ASCII classical ciphers for challenges, never secure encryption.
use crate::{
    errors::{ErrorCode, OperationError},
    text::check_input,
};
/// Maximum ASCII-letter key size (64 KiB); validation precedes transformation.
pub const MAX_KEY: usize = 65536;
fn invalid(message: &str, op: &'static str, parameter: &'static str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
        .operation(op)
        .parameter(parameter)
}
pub(crate) fn validate_key(input: &str, op: &'static str) -> Result<(), OperationError> {
    if input.len() > MAX_KEY {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "key exceeds 64 KiB")
                .operation(op)
                .parameter("key"),
        );
    }
    if input.is_empty() || !input.bytes().all(|b| b.is_ascii_alphabetic()) {
        return Err(invalid(
            "key must contain only ASCII letters and must not be empty",
            op,
            "key",
        ));
    }
    Ok(())
}
fn alphabet(character: char) -> Option<u8> {
    match character {
        'A'..='Z' => Some(b'A'),
        'a'..='z' => Some(b'a'),
        _ => None,
    }
}
/// Reverse ASCII alphabets, preserving case and all other Unicode scalars exactly.
/// Self-inverse; no normalization. Maximum input 1 MiB. Not secure encryption.
pub fn atbash(input: &str) -> Result<String, OperationError> {
    check_input(input, "cipher.atbash")?;
    Ok(input
        .chars()
        .map(|c| alphabet(c).map_or(c, |base| char::from(base + 25 - (c as u8 - base))))
        .collect())
}
fn polyalphabetic(
    input: &str,
    key_text: &str,
    op: &'static str,
    decode: bool,
    beaufort: bool,
) -> Result<String, OperationError> {
    validate_key(key_text, op)?;
    check_input(input, op)?;
    let mut position = 0;
    Ok(input
        .chars()
        .map(|c| {
            let Some(base) = alphabet(c) else {
                return c;
            };
            let k = key_text.as_bytes()[position].to_ascii_uppercase() - b'A';
            position = (position + 1) % key_text.len();
            let p = c as u8 - base;
            let value = if beaufort {
                (k + 26 - p) % 26
            } else if decode {
                (p + 26 - k) % 26
            } else {
                (p + k) % 26
            };
            char::from(base + value)
        })
        .collect())
}
/// Vigenère C=(P+K) mod 26, A=0. Repeat an ASCII-letter key, advancing only for
/// ASCII input letters; preserve case, all other scalars and whitespace. No
/// normalization. Input <=1 MiB; key 1..=65536 bytes. Not secure encryption.
pub fn vigenere_encode(input: &str, key: &str) -> Result<String, OperationError> {
    polyalphabetic(input, key, "cipher.vigenere.encode", false, false)
}
/// Invert Vigenère with P=(C-K) mod 26 and the same advancement/preservation rules.
pub fn vigenere_decode(input: &str, key: &str) -> Result<String, OperationError> {
    polyalphabetic(input, key, "cipher.vigenere.decode", true, false)
}
/// Standard reciprocal Beaufort C=(K-P) mod 26. Same case, advancement and limits
/// as Vigenère. Apply twice to restore input; this is not Variant Beaufort.
pub fn beaufort(input: &str, key: &str) -> Result<String, OperationError> {
    polyalphabetic(input, key, "cipher.beaufort", false, true)
}
pub(crate) fn validate_affine_keys(
    a: i32,
    b: i32,
    op: &'static str,
) -> Result<i32, OperationError> {
    if !(0..26).contains(&a) {
        return Err(invalid("a must be in 0..25 and coprime to 26", op, "a"));
    }
    if !(0..26).contains(&b) {
        return Err(invalid("b must be in 0..25", op, "b"));
    }
    let inverse = (1..26)
        .find(|n| (a * n) % 26 == 1)
        .ok_or_else(|| invalid("a must be coprime to 26", op, "a"))?;
    Ok(inverse)
}
fn affine(input: &str, a: i32, b: i32, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.affine.decode"
    } else {
        "cipher.affine.encode"
    };
    let inverse = validate_affine_keys(a, b, op)?;
    check_input(input, op)?;
    Ok(input
        .chars()
        .map(|c| {
            let Some(base) = alphabet(c) else {
                return c;
            };
            let p = i32::from(c as u8 - base);
            let value = if decode {
                (inverse * (p - b)).rem_euclid(26)
            } else {
                (a * p + b) % 26
            };
            char::from(base + value as u8)
        })
        .collect())
}
/// Affine C=(aP+b) mod 26. Keys are in 0..25 and gcd(a,26)=1; preserve ASCII
/// case and all other Unicode. Input <=1 MiB. Classical cipher, not secure encryption.
pub fn affine_encode(input: &str, a: i32, b: i32) -> Result<String, OperationError> {
    affine(input, a, b, false)
}
/// Invert Affine using a's modular inverse; same validation and preservation rules.
pub fn affine_decode(input: &str, a: i32, b: i32) -> Result<String, OperationError> {
    affine(input, a, b, true)
}

use crate::operations::{Descriptor, Parameter, ParameterType};
const KEY: &[Parameter] = &[Parameter {
    name: "key",
    description: "1..=65536 ASCII letters; case-insensitive; advance only on ASCII input letters.",
    required: true,
    default: None,
    constraints: ParameterType::Text { choices: &[] },
}];
const AFFINE: &[Parameter] = &[
    Parameter {
        name: "a",
        description: "Multiplier coprime to 26 (1,3,5,7,9,11,15,17,19,21,23,25).",
        required: true,
        default: None,
        constraints: ParameterType::UnsignedChoices {
            choices: &[1, 3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25],
        },
    },
    Parameter {
        name: "b",
        description: "Additive key in 0..25.",
        required: true,
        default: None,
        constraints: ParameterType::Integer { min: 0, max: 25 },
    },
];
/// Lightweight metadata beside the reusable functions, without runtime dispatch.
const PROFILED: [crate::operations::metadata::Entry; 6] = [
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.atbash",
            category: "ciphers",
            description: "Reverse ASCII alphabets; preserve case and other Unicode, <=1 MiB; not secure encryption.",
            input_type: "text",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "cipher atbash",
            example: "cube cipher atbash --text 'ABC xyz'",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Atbash",
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
                    alphabet: "ASCII A-Z/a-z; preserve case and all other scalars/whitespace exactly; no normalization.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::ciphers::classical::atbash",
                python_api: "atbash",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.vigenere.encode",
            category: "ciphers",
            description: "Vigenère C=(P+K) mod 26; ASCII letters only, preserve other Unicode; <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: KEY,
            deterministic: true,
            reversible: Some(true),
            command: "cipher vigenere",
            example: "cube cipher vigenere --key LEMON --text ATTACKATDAWN",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Vigenère",
                family: "classical_substitution",
                aliases: &[crate::operations::metadata::Alias {
                    name: "Variant Beaufort",
                    usage: "For C=P-K use vigenere_decode; its inverse uses vigenere_encode with identical ASCII advancement. This is not standard reciprocal Beaufort.",
                }],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &["key"],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "ASCII A-Z/a-z; preserve case and all other scalars/whitespace exactly; no normalization.",
                    key: "Required nonempty ASCII-letter key, 1..65536 bytes; key case is ignored.",
                    key_advancement: "Advance/repeat the key only on transformed ASCII input letters.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "key_bytes",
                        maximum: crate::ciphers::classical::MAX_KEY as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::ciphers::classical::vigenere_encode",
                python_api: "vigenere_encode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.vigenere.decode",
            category: "ciphers",
            description: "Vigenère P=(C-K) mod 26 with the same key advancement; <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: KEY,
            deterministic: true,
            reversible: Some(true),
            command: "cipher vigenere",
            example: "cube cipher vigenere --decode --key LEMON --text LXFOPVEFRNHR",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Vigenère",
                family: "classical_substitution",
                aliases: &[crate::operations::metadata::Alias {
                    name: "Variant Beaufort",
                    usage: "For C=P-K use vigenere_decode; its inverse uses vigenere_encode with identical ASCII advancement. This is not standard reciprocal Beaufort.",
                }],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &["key"],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "ASCII A-Z/a-z; preserve case and all other scalars/whitespace exactly; no normalization.",
                    key: "Required nonempty ASCII-letter key, 1..65536 bytes; key case is ignored.",
                    key_advancement: "Advance/repeat the key only on transformed ASCII input letters.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "key_bytes",
                        maximum: crate::ciphers::classical::MAX_KEY as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::ciphers::classical::vigenere_decode",
                python_api: "vigenere_decode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.beaufort",
            category: "ciphers",
            description: "Standard reciprocal Beaufort C=(K-P) mod 26, ASCII letters only; <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: KEY,
            deterministic: true,
            reversible: Some(true),
            command: "cipher beaufort",
            example: "cube cipher beaufort --key B --text A",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Beaufort",
                family: "classical_substitution",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &["key"],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "ASCII A-Z/a-z; preserve case and all other scalars/whitespace exactly; no normalization.",
                    key: "Required nonempty ASCII-letter key, 1..65536 bytes; key case is ignored.",
                    key_advancement: "Advance/repeat the key only on transformed ASCII input letters.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "key_bytes",
                        maximum: crate::ciphers::classical::MAX_KEY as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::ciphers::classical::beaufort",
                python_api: "beaufort",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.affine.encode",
            category: "ciphers",
            description: "Affine C=(aP+b) mod 26; preserve case and other Unicode, <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: AFFINE,
            deterministic: true,
            reversible: Some(true),
            command: "cipher affine",
            example: "cube cipher affine --a 5 --b 8 --text AFFINECIPHER",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Affine",
                family: "classical_substitution",
                aliases: &[crate::operations::metadata::Alias {
                    name: "Multiplicative",
                    usage: "Use the existing Affine profile with b=0; keep a coprime to 26.",
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
                    alphabet: "ASCII A-Z/a-z; preserve case and all other scalars/whitespace exactly; no normalization.",
                    key: "a in 0..25 coprime to 26; b in 0..25. Inversion uses the modular inverse of a.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::ciphers::classical::affine_encode",
                python_api: "affine_encode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.affine.decode",
            category: "ciphers",
            description: "Invert Affine with a's modular inverse; preserve case and other Unicode, <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: AFFINE,
            deterministic: true,
            reversible: Some(true),
            command: "cipher affine",
            example: "cube cipher affine --decode --a 5 --b 8 --text IHHWVCSWFRCP",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Affine",
                family: "classical_substitution",
                aliases: &[crate::operations::metadata::Alias {
                    name: "Multiplicative",
                    usage: "Use the existing Affine profile with b=0; keep a coprime to 26.",
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
                    alphabet: "ASCII A-Z/a-z; preserve case and all other scalars/whitespace exactly; no normalization.",
                    key: "a in 0..25 coprime to 26; b in 0..25. Inversion uses the modular inverse of a.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::ciphers::classical::affine_decode",
                python_api: "affine_decode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
