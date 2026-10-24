//! ASCII classical ciphers for puzzles, never secure encryption.
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
fn key(input: &str, op: &'static str) -> Result<(), OperationError> {
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
    key(key_text, op)?;
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
/// normalization. Input <=1 MiB; key 1..=65536 bytes. Classical puzzle cipher.
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
fn affine(input: &str, a: i32, b: i32, decode: bool) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.affine.decode"
    } else {
        "cipher.affine.encode"
    };
    if !(0..26).contains(&a) {
        return Err(invalid("a must be in 0..25 and coprime to 26", op, "a"));
    }
    if !(0..26).contains(&b) {
        return Err(invalid("b must be in 0..25", op, "b"));
    }
    let inverse = (1..26)
        .find(|n| (a * n) % 26 == 1)
        .ok_or_else(|| invalid("a must be coprime to 26", op, "a"))?;
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
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
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
    },
    Descriptor {
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
    },
    Descriptor {
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
    },
    Descriptor {
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
    },
    Descriptor {
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
    },
    Descriptor {
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
    },
];
