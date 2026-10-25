//! Exact signed-magnitude integers over caller-defined scalar alphabets.
use crate::errors::{ErrorCode, OperationError};
use crate::operations::{Descriptor, Parameter, ParameterType};
use std::collections::BTreeMap;
pub const MAX_INPUT: usize = 16385;
pub const MAX_ALPHABET: usize = 1024;
pub const MAX_OUTPUT: usize = 513;
const OP: &str = "number.alphabet.convert";
fn alphabet(value: &str, base: u32, parameter: &'static str) -> Result<Vec<char>, OperationError> {
    let invalid = |message| {
        OperationError::new(ErrorCode::InvalidParameter, message)
            .operation(OP)
            .parameter(parameter)
    };
    if value.len() > MAX_ALPHABET {
        return Err(invalid("alphabet exceeds 1024 UTF-8 bytes"));
    }
    let chars: Vec<_> = value.chars().collect();
    if !(2..=256).contains(&chars.len()) || base < 2 || base as usize > chars.len() {
        return Err(invalid(
            "alphabet requires 2..256 scalars and an explicit base in 2..alphabet length",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for c in &chars {
        if matches!(c, '+' | '-') || c.is_whitespace() || !seen.insert(*c) {
            return Err(invalid(
                "alphabet scalars must be unique, without ASCII signs or Unicode whitespace",
            ));
        }
    }
    Ok(chars)
}
/// Convert exact scalar digits using explicit ordered alphabets and bases.
/// Magnitude <=u128::MAX, <=4096 digits/16385 input bytes, alphabets <=256
/// scalars/1024 bytes, output <=128 digits/513 bytes. No case folding or
/// normalization. Leading +/- and zeros canonicalize; negative zero becomes zero.
/// Larger integers are deliberately deferred; this shares checked arithmetic with
/// ordinary base conversion, not byte-codec semantics.
pub fn convert(
    input: &str,
    source_alphabet: &str,
    target_alphabet: &str,
    from_base: u32,
    to_base: u32,
) -> Result<String, OperationError> {
    let source = alphabet(source_alphabet, from_base, "source_alphabet")?;
    let target = alphabet(target_alphabet, to_base, "target_alphabet")?;
    if input.len() > MAX_INPUT {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "integer input exceeds 16385 UTF-8 bytes",
        )
        .operation(OP));
    }
    let (negative, digits, offset) = match input.as_bytes().first() {
        Some(b'-') => (true, &input[1..], 1),
        Some(b'+') => (false, &input[1..], 1),
        _ => (false, input, 0),
    };
    if digits.is_empty() {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "integer requires at least one digit",
        )
        .operation(OP));
    }
    if digits.chars().count() > super::MAX_DIGITS {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "integer exceeds 4096 digit scalars",
        )
        .operation(OP));
    }
    let map: BTreeMap<_, _> = source
        .iter()
        .take(from_base as usize)
        .enumerate()
        .map(|(i, c)| (*c, i as u32))
        .collect();
    let mut values = Vec::with_capacity(digits.chars().count());
    for (i, c) in digits.char_indices() {
        values.push(*map.get(&c).ok_or_else(|| {
            OperationError::new(
                ErrorCode::InvalidInput,
                "digit is outside the active source alphabet",
            )
            .operation(OP)
            .offset((i + offset) as u64)
        })?);
    }
    let value = super::magnitude(values.into_iter(), from_base, OP)?;
    let positions = super::radix_digits(value, to_base);
    let mut output =
        String::with_capacity(positions.len() * 4 + usize::from(negative && value != 0));
    if negative && value != 0 {
        output.push('-');
    }
    output.extend(positions.into_iter().map(|i| target[i as usize]));
    Ok(output)
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "numbers",
    description: "Exact positional integers over explicit unique scalar alphabets/bases; u128 magnitude, <=4096 digits, no normalization/case folding.",
    input_type: "text.integer",
    output_type: "text.integer",
    deterministic: true,
    reversible: Some(false),
    command: "number alphabet",
    example: "cube number alphabet --source-alphabet 01 --target-alphabet abcdefghij --from-base 2 --to-base 10 --text 1010",
    parameters: &[
        Parameter {
            name: "source_alphabet",
            description: "Ordered 2..256 exact unique non-whitespace scalars, no ASCII signs; <=1024 bytes.",
            required: true,
            default: None,
            constraints: ParameterType::Text { choices: &[] },
        },
        Parameter {
            name: "target_alphabet",
            description: "Ordered target digits; same validation as source.",
            required: true,
            default: None,
            constraints: ParameterType::Text { choices: &[] },
        },
        Parameter {
            name: "from_base",
            description: "Explicit active source prefix length.",
            required: true,
            default: None,
            constraints: ParameterType::Unsigned { min: 2, max: 256 },
        },
        Parameter {
            name: "to_base",
            description: "Explicit active target prefix length.",
            required: true,
            default: None,
            constraints: ParameterType::Unsigned { min: 2, max: 256 },
        },
    ],
}];
