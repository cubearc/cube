//! Explicit integer representations and decimal arithmetic witnesses.
use super::{Unsigned, invalid, overflow, parameter, resource};
use crate::{errors::OperationError, numbers};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub(super) fn decimal(input: &str, op: &'static str) -> Result<(bool, u128, u32), OperationError> {
    let (negative, digits, leading) = numbers::signed_digits(input);
    if digits.len() > numbers::MAX_DIGITS {
        return Err(resource("integer exceeds 4096 decimal digits"));
    }
    if digits.is_empty() {
        return Err(invalid("integer requires a decimal digit"));
    }
    let mut magnitude = 0;
    let mut sum = 0;
    for (offset, b) in digits.bytes().enumerate() {
        if !b.is_ascii_digit() {
            return Err(
                invalid("expected exact ASCII decimal digits").offset((leading + offset) as u64)
            );
        }
        let digit = u32::from(b - b'0');
        magnitude = numbers::accumulate(magnitude, digit, 10, op)?;
        sum += digit;
    }
    Ok((negative, magnitude, sum))
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Equation {
    Add,
    Subtract,
    Multiply,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum DecimalRequest {
    Summarize {
        value: String,
    },
    Check {
        operation: Equation,
        left: String,
        right: String,
        claimed: String,
    },
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum DecimalReport {
    Summarize {
        digit_sum: u32,
        digital_root: u8,
        residue9: u8,
    },
    Check {
        left_residue: u8,
        right_residue: u8,
        expected_residue: u8,
        claimed_residue: u8,
        congruent: bool,
    },
}

fn residue(negative: bool, sum: u32) -> u8 {
    let rest = (sum % 9) as u8;
    if negative && rest != 0 {
        9 - rest
    } else {
        rest
    }
}

/// Compute decimal witnesses using the same strict signed magnitude grammar.
pub fn decimal_check(request: &DecimalRequest) -> Result<DecimalReport, OperationError> {
    const OP: &str = "number.decimal_check";
    let read = |value| decimal(value, OP);
    Ok(match request {
        DecimalRequest::Summarize { value } => {
            let (negative, magnitude, digit_sum) = read(value.as_str())?;
            DecimalReport::Summarize {
                digit_sum,
                digital_root: numbers::digital::digital_root(magnitude, 10)?,
                residue9: residue(negative, digit_sum),
            }
        }
        DecimalRequest::Check {
            operation,
            left,
            right,
            claimed,
        } => {
            let get = |text: &str| decimal(text, OP).map(|(sign, _, sum)| residue(sign, sum));
            let (left_residue, right_residue, claimed_residue) =
                (get(left)?, get(right)?, get(claimed)?);
            let (a, b) = (i16::from(left_residue), i16::from(right_residue));
            let expected_residue = (match operation {
                Equation::Add => a + b,
                Equation::Subtract => a - b,
                Equation::Multiply => a * b,
            })
            .rem_euclid(9) as u8;
            DecimalReport::Check {
                left_residue,
                right_residue,
                expected_residue,
                claimed_residue,
                congruent: expected_residue == claimed_residue,
            }
        }
    })
}

const ROMAN: [(u16, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "direction", rename_all = "snake_case", deny_unknown_fields)]
pub enum RomanRequest {
    Encode {
        #[schemars(range(min = 1, max = 3999))]
        value: u16,
    },
    Decode {
        text: String,
        #[serde(default)]
        ascii_case_insensitive: bool,
    },
}
#[derive(Serialize, JsonSchema)]
pub struct RomanReport {
    pub value: u16,
    pub text: String,
}

fn roman_string(mut value: u16) -> String {
    let mut text = String::with_capacity(15);
    for (number, token) in ROMAN {
        for _ in 0..value / number {
            text.push_str(token);
        }
        value %= number;
    }
    text
}

/// Convert a canonical 1–3999 ASCII Roman numeral.
pub fn roman(request: &RomanRequest) -> Result<RomanReport, OperationError> {
    let value = match request {
        RomanRequest::Encode { value } if (1..=3999).contains(value) => *value,
        RomanRequest::Encode { .. } => {
            return Err(parameter("Roman value requires 1 through 3999"));
        }
        RomanRequest::Decode {
            text,
            ascii_case_insensitive,
        } => {
            if text.len() > 15 {
                return Err(resource("Roman numeral exceeds 15 bytes"));
            }
            if text.is_empty() || !text.is_ascii() {
                return Err(invalid("expected nonempty ASCII Roman numeral"));
            }
            let input = if *ascii_case_insensitive {
                text.to_ascii_uppercase()
            } else {
                text.clone()
            };
            let mut rest = input.as_str();
            let mut value = 0;
            for (number, token) in ROMAN {
                while let Some(suffix) = rest.strip_prefix(token) {
                    value += number;
                    rest = suffix;
                }
            }
            if !rest.is_empty() || !(1..=3999).contains(&value) || roman_string(value) != input {
                return Err(invalid("Roman numeral is outside canonical 1–3999 grammar"));
            }
            value
        }
    };
    Ok(RomanReport {
        value,
        text: roman_string(value),
    })
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Encode,
    Decode,
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "direction", rename_all = "snake_case", deny_unknown_fields)]
pub enum TernaryRequest {
    Encode {
        value: String,
    },
    Decode {
        text: String,
        #[serde(default)]
        canonicalize: bool,
    },
}
#[derive(Serialize, JsonSchema)]
pub struct TernaryReport {
    pub value: String,
    pub text: String,
    pub trits: Vec<i8>,
}

fn ternary_digits(mut magnitude: u128, negative: bool) -> Vec<i8> {
    let mut result = Vec::with_capacity(82);
    loop {
        let rem = (magnitude % 3) as i8;
        magnitude /= 3;
        let trit = if rem == 2 {
            magnitude += 1;
            -1
        } else {
            rem
        };
        result.push(if negative { -trit } else { trit });
        if magnitude == 0 {
            break;
        }
    }
    result.reverse();
    result
}

fn ternary_magnitude(text: &str, canonicalize: bool) -> Result<(bool, u128), OperationError> {
    if text.len() > 82 {
        return Err(resource("balanced ternary exceeds 82 trits"));
    }
    if text.is_empty() || !canonicalize && text.len() > 1 && text.starts_with('0') {
        return Err(invalid(
            "balanced ternary requires canonical nonempty digits",
        ));
    }
    let mut magnitude = 0u128;
    let mut negative = false;
    for (offset, b) in text.bytes().enumerate() {
        let digit = match b {
            b'T' => -1,
            b'0' => 0,
            b'1' => 1,
            _ => {
                return Err(invalid("balanced ternary digits are T, 0 and 1").offset(offset as u64));
            }
        };
        if magnitude == 0 {
            magnitude = u128::from(digit != 0);
            negative = digit < 0;
        } else {
            magnitude = magnitude.checked_mul(3).ok_or_else(overflow)?;
            magnitude = match (digit, negative) {
                (0, _) => magnitude,
                (-1, false) | (1, true) => magnitude - 1,
                _ => magnitude.checked_add(1).ok_or_else(overflow)?,
            };
        }
    }
    Ok((negative, magnitude))
}

/// Convert a signed 128-bit magnitude to or from canonical balanced ternary.
pub fn balanced_ternary(request: &TernaryRequest) -> Result<TernaryReport, OperationError> {
    let (negative, magnitude) = match request {
        TernaryRequest::Encode { value } => {
            let (sign, magnitude, _) = decimal(value, "number.balanced_ternary")?;
            (sign, magnitude)
        }
        TernaryRequest::Decode { text, canonicalize } => ternary_magnitude(text, *canonicalize)?,
    };
    let trits = ternary_digits(magnitude, negative);
    let text = trits
        .iter()
        .map(|trit| match trit {
            -1 => 'T',
            0 => '0',
            _ => '1',
        })
        .collect();
    let value = if negative && magnitude != 0 {
        format!("-{magnitude}")
    } else {
        magnitude.to_string()
    };
    Ok(TernaryReport { value, text, trits })
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GrayRequest {
    pub direction: Direction,
    #[schemars(range(min = 1, max = 128))]
    pub width: u8,
    pub value: Unsigned,
    #[serde(default)]
    pub binary: bool,
}
#[derive(Serialize, JsonSchema)]
pub struct GrayReport {
    pub value: Unsigned,
    pub binary: Option<String>,
}

/// Encode or decode an unsigned reflected Gray code of the supplied width.
pub fn gray(request: &GrayRequest) -> Result<GrayReport, OperationError> {
    let width = u32::from(request.width);
    if width == 0 || width > 128 {
        return Err(parameter("width must be within 1..128"));
    }
    let mut value = request.value.0;
    if width < 128 && value >= 1u128 << width {
        return Err(invalid("integer does not fit the supplied width"));
    }
    match request.direction {
        Direction::Encode => value ^= value >> 1,
        Direction::Decode => {
            for shift in [1, 2, 4, 8, 16, 32, 64] {
                value ^= value >> shift;
            }
        }
    }
    let binary = request
        .binary
        .then(|| format!("{value:0width$b}", width = width as usize));
    Ok(GrayReport {
        value: Unsigned(value),
        binary,
    })
}
