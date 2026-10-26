//! Checked integer representations, distinct from byte encodings and floating point.
pub mod alphabet;
pub mod constants;
pub mod digital;
pub mod modular;
use crate::errors::{ErrorCode, OperationError};
/// Maximum ASCII digits, excluding one optional sign; leading zeros still count.
pub const MAX_DIGITS: usize = 4096;
/// Magnitude bound: 2^128-1, independent of sign. Output has <=128 digits plus sign.
pub const MAX_BITS: u32 = 128;
/// Convert signed ASCII integer text between explicit bases 2..36. Accept either
/// letter case, emit uppercase digits with no prefix, normalize zeros/signs.
/// Only one leading +/- is allowed; no spaces, separators, fractions or prefixes.
/// Maximum 4096 input digits and 128-bit magnitude; checked arithmetic never wraps.
pub fn convert_base(input: &str, from_base: u32, to_base: u32) -> Result<String, OperationError> {
    const OP: &str = "number.base.convert";
    let invalid = |parameter, message| {
        OperationError::new(ErrorCode::InvalidParameter, message)
            .operation(OP)
            .parameter(parameter)
    };
    if !(2..=36).contains(&from_base) {
        return Err(invalid("from_base", "source base must be in 2..36"));
    }
    if !(2..=36).contains(&to_base) {
        return Err(invalid("to_base", "target base must be in 2..36"));
    }
    let (negative, digits, offset) = signed_digits(input);
    if digits.len() > MAX_DIGITS {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "integer input exceeds 4096 digits",
        )
        .operation(OP));
    }
    if digits.is_empty() {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "integer requires at least one ASCII digit",
        )
        .operation(OP));
    }
    for (i, b) in digits.bytes().enumerate() {
        if ascii_digit(b) >= from_base {
            return Err(OperationError::new(
                ErrorCode::InvalidInput,
                "digit is invalid for source base; prefixes and whitespace are not accepted",
            )
            .operation(OP)
            .offset((i + offset) as u64));
        }
    }
    let value = magnitude(digits.bytes().map(ascii_digit), from_base, OP)?;
    let mut result = String::with_capacity(MAX_BITS as usize + 1);
    if negative && value != 0 {
        result.push('-');
    }
    result.extend(radix_digits(value, to_base).into_iter().map(|n| {
        if n < 10 {
            char::from(b'0' + n as u8)
        } else {
            char::from(b'A' + (n - 10) as u8)
        }
    }));
    Ok(result)
}
// Shared exact grammar and arithmetic used by decimal digital root as well.
fn signed_digits(input: &str) -> (bool, &str, usize) {
    match input.as_bytes().first() {
        Some(b'-') => (true, &input[1..], 1),
        Some(b'+') => (false, &input[1..], 1),
        _ => (false, input, 0),
    }
}
fn ascii_digit(b: u8) -> u32 {
    match b {
        b'0'..=b'9' => u32::from(b - b'0'),
        b'A'..=b'Z' => u32::from(b - b'A') + 10,
        b'a'..=b'z' => u32::from(b - b'a') + 10,
        _ => u32::MAX,
    }
}
fn accumulate(
    value: u128,
    digit: u32,
    base: u32,
    op: &'static str,
) -> Result<u128, OperationError> {
    value
        .checked_mul(u128::from(base))
        .and_then(|v| v.checked_add(u128::from(digit)))
        .ok_or_else(|| {
            OperationError::new(
                ErrorCode::ResourceLimit,
                "integer magnitude exceeds 128 bits",
            )
            .operation(op)
        })
}
// Shared fixed-width arithmetic; callers validate their distinct digit grammars first.
fn magnitude(
    digits: impl Iterator<Item = u32>,
    base: u32,
    op: &'static str,
) -> Result<u128, OperationError> {
    let mut value = 0u128;
    for digit in digits {
        value = accumulate(value, digit, base, op)?;
    }
    Ok(value)
}
fn radix_digits(mut value: u128, base: u32) -> Vec<u32> {
    let mut digits = Vec::with_capacity(MAX_BITS as usize);
    loop {
        digits.push((value % u128::from(base)) as u32);
        value /= u128::from(base);
        if value == 0 {
            break;
        }
    }
    digits.reverse();
    digits
}

use crate::operations::{Descriptor, Parameter, ParameterType};
/// This is integer text conversion, not a byte codec.
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "number.base.convert",
    category: "numbers",
    description: "Convert signed ASCII integers between bases 2..36; <=4096 input digits, <=128-bit magnitude; no floats or prefixes.",
    input_type: "text.integer",
    output_type: "text.integer",
    deterministic: true,
    reversible: Some(false),
    command: "number base",
    example: "cube number base --from-base 16 --to-base 2 --text FF",
    parameters: &[
        Parameter {
            name: "from_base",
            description: "Explicit source radix, case-insensitive ASCII digits.",
            required: true,
            default: None,
            constraints: ParameterType::Unsigned { min: 2, max: 36 },
        },
        Parameter {
            name: "to_base",
            description: "Explicit target radix, uppercase output; at most 128 digits plus sign.",
            required: true,
            default: None,
            constraints: ParameterType::Unsigned { min: 2, max: 36 },
        },
    ],
}];
