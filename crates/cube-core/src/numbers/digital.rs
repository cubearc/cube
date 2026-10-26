//! Decimal digital root of an exact bounded signed magnitude.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
const OP: &str = "number.digital_root";
/// At most 4096 ASCII decimal digits and one optional leading +/- byte.
pub const MAX_INPUT_BYTES: usize = super::MAX_DIGITS + 1;
fn radix_check(radix: u32) -> Result<(), OperationError> {
    if radix != 10 {
        return Err(OperationError::new(
            ErrorCode::InvalidParameter,
            "only decimal radix 10 is supported",
        )
        .operation(OP)
        .parameter("radix"));
    }
    Ok(())
}
/// Nonnegative decimal digital root; the sign of a signed magnitude is ignored.
/// Zero maps to zero; all other magnitudes map to 1+(magnitude-1)%9.
/// Radix must be 10. Constant-size arithmetic, no allocation and no inverse.
///
/// ```
/// use cube_core::numbers::digital::{digital_root, from_decimal};
/// assert_eq!(digital_root(38, 10).unwrap(), 2);
/// assert_eq!(from_decimal("-00038", 10).unwrap(), 2);
/// ```
pub fn digital_root(magnitude: u128, radix: u32) -> Result<u8, OperationError> {
    radix_check(radix)?;
    Ok(if magnitude == 0 {
        0
    } else {
        (1 + (magnitude - 1) % 9) as u8
    })
}
/// Strict signed ASCII decimal transport using the existing sign/digit/checked
/// magnitude primitives. Leading zeros count toward the 4096-digit limit;
/// prefixes, spaces, fractions and separators are invalid. A single bounded
/// parsing pass visits <=4096 digits, followed by constant arithmetic. Magnitude
/// overflow is resource_limit, never a partial/modulo-only parsing success.
pub fn from_decimal(input: &str, radix: u32) -> Result<u8, OperationError> {
    radix_check(radix)?;
    let (_, digits, offset) = super::signed_digits(input);
    if input.len() > MAX_INPUT_BYTES || digits.len() > super::MAX_DIGITS {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "decimal input exceeds 4096 digits and optional sign",
        )
        .operation(OP));
    }
    if digits.is_empty() {
        return Err(OperationError::new(
            ErrorCode::InvalidInput,
            "integer requires at least one ASCII decimal digit",
        )
        .operation(OP));
    }
    let mut value = 0;
    for (i, b) in digits.bytes().enumerate() {
        let digit = super::ascii_digit(b);
        if digit >= 10 {
            return Err(OperationError::new(
                ErrorCode::InvalidInput,
                "decimal digits only; no whitespace, prefixes or fractions",
            )
            .operation(OP)
            .offset((offset + i) as u64));
        }
        value = super::accumulate(value, digit, 10, OP)?;
    }
    digital_root(value, radix)
}
/// Direct operation only; generalized radices and persistence are deferred.
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "numbers",
    description: "Decimal digital root of a signed 128-bit magnitude; <=4096 ASCII input digits, no float conversion; zero maps to zero.",
    input_type: "number.signed_magnitude",
    output_type: "number.decimal_digit",
    deterministic: true,
    reversible: Some(false),
    command: "number digital-root",
    example: "cube number digital-root --text=-38",
    parameters: &[Parameter {
        name: "radix",
        description: "Only decimal 10; generalized radices are not implemented.",
        required: false,
        default: Some(ParameterValue::Unsigned(10)),
        constraints: ParameterType::UnsignedChoices { choices: &[10] },
    }],
}];
