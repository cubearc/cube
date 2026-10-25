//! Certified, bounded fractional decimal digits with an explicit prefix limit.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use num_bigint::{BigInt, BigUint, Sign};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub const DEFAULT_MAX_DIGITS: usize = 256;
pub const MAX_DIGITS: usize = 4096;
pub const MAX_INDICES: usize = 1024;
pub const MAX_SELECTOR_BYTES: usize = 8192;
pub const MAX_OPERAND_BITS: u64 = 32768;
pub const MAX_SERIES_STEPS: usize = 8192;
const GUARD_DIGITS: usize = 16;
const ID: &str = "number.constant_digits";

fn invalid(parameter: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
        .operation(ID)
        .parameter(parameter)
}
fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message).operation(ID)
}

/// Exact supported constant names, independent of floating-point approximations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Constant {
    Pi,
    E,
    Phi,
    Sqrt2,
}
impl Constant {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "pi" => Ok(Self::Pi),
            "e" => Ok(Self::E),
            "phi" => Ok(Self::Phi),
            "sqrt2" => Ok(Self::Sqrt2),
            _ => Err(invalid("constant", "constant must be pi, e, phi or sqrt2")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Pi => "pi",
            Self::E => "e",
            Self::Phi => "phi",
            Self::Sqrt2 => "sqrt2",
        }
    }
    fn integer(self) -> &'static str {
        match self {
            Self::Pi => "3",
            Self::E => "2",
            Self::Phi | Self::Sqrt2 => "1",
        }
    }
}

/// Select one range (default 0,16), or an explicit ordered index list.
/// `max_digits` limits the generated fractional prefix, default 256, hard cap 4096.
/// It is never automatically increased to accommodate a selection.
#[derive(Clone, Copy, Debug)]
pub struct DigitOptions<'a> {
    pub start: Option<usize>,
    pub count: Option<usize>,
    pub indices: Option<&'a [usize]>,
    pub max_digits: usize,
}
impl Default for DigitOptions<'_> {
    fn default() -> Self {
        Self {
            start: None,
            count: None,
            indices: None,
            max_digits: DEFAULT_MAX_DIGITS,
        }
    }
}

/// Exact text plus the provenance of the entire certified `max_digits` prefix.
#[derive(Debug, Serialize)]
pub struct DigitSelection {
    pub constant: &'static str,
    pub radix: u8,
    pub integer_part: &'static str,
    pub fractional_zero_based: bool,
    pub digits: String,
    pub max_digits: usize,
    pub prefix_sha256: String,
    pub certification: &'static str,
}

/// Parse the CLI's strict decimal comma-separated index list (empty is allowed).
/// At most 8 KiB and 1024 entries; no signs, whitespace, prefixes or fractions.
/// Position/domain validation is shared with `digits`.
pub fn parse_indices(text: &str) -> Result<Vec<usize>, OperationError> {
    if text.len() > MAX_SELECTOR_BYTES {
        return Err(limit("index text exceeds 8 KiB").parameter("indices"));
    }
    if text.is_empty() {
        return Ok(vec![]);
    }
    let mut result = Vec::new();
    for part in text.split(',') {
        if result.len() == MAX_INDICES {
            return Err(limit("index selection exceeds 1024 entries").parameter("indices"));
        }
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return Err(invalid(
                "indices",
                "indexes must be comma-separated unsigned decimal integers",
            ));
        }
        result.push(
            part.parse()
                .map_err(|_| invalid("indices", "index exceeds platform integer range"))?,
        );
    }
    Ok(result)
}

struct Interval {
    lower: BigInt,
    upper: BigInt,
}
fn charge(steps: &mut usize) -> Result<(), OperationError> {
    if *steps == MAX_SERIES_STEPS {
        return Err(limit("constant calculation exceeds 8192 series steps"));
    }
    *steps += 1;
    Ok(())
}
fn bounded(value: &BigUint) -> Result<(), OperationError> {
    if value.bits() > MAX_OPERAND_BITS {
        return Err(limit("constant operand exceeds 32768 bits"));
    }
    Ok(())
}

// Each included scaled term t lies in [floor(t),floor(t)+1]. Signed sums
// propagate both endpoints; the first omitted alternating tail is below one
// scaled unit. This certifies the result instead of trusting guard digits.
fn atan(scale: &BigUint, q: u32, steps: &mut usize) -> Result<Interval, OperationError> {
    let mut power = BigUint::from(q);
    let mut lower = BigInt::from(0u8);
    let mut upper = lower.clone();
    for k in 0..MAX_SERIES_STEPS {
        charge(steps)?;
        let denominator = &power * (2 * k + 1);
        bounded(&denominator)?;
        let floor = scale / denominator;
        if floor == BigUint::from(0u8) {
            if k % 2 == 0 {
                upper += 1u8;
            } else {
                lower -= 1u8;
            }
            return Ok(Interval { lower, upper });
        }
        let floor = BigInt::from(floor);
        if k % 2 == 0 {
            lower += &floor;
            upper += floor + 1u8;
        } else {
            lower -= &floor + 1u8;
            upper -= floor;
        }
        power *= q * q;
        bounded(&power)?;
    }
    Err(limit(
        "arctangent calculation did not converge within its step budget",
    ))
}

fn exponential(scale: &BigUint, steps: &mut usize) -> Result<Interval, OperationError> {
    let mut factorial = BigUint::from(1u8);
    let mut lower = BigUint::from(0u8);
    let mut upper = lower.clone();
    for k in 0..MAX_SERIES_STEPS {
        charge(steps)?;
        if k > 0 {
            factorial *= k;
        }
        bounded(&factorial)?;
        let floor = scale / &factorial;
        if floor == BigUint::from(0u8) {
            // First omitted index k>=2: sum(j=k..inf, 1/j!) < 2/k!.
            upper += 2u8;
            return Ok(Interval {
                lower: lower.into(),
                upper: upper.into(),
            });
        }
        lower += &floor;
        upper += floor + 1u8;
    }
    Err(limit(
        "factorial series did not converge within its step budget",
    ))
}

fn root(scale: &BigUint, radicand: u8, phi: bool) -> Result<Interval, OperationError> {
    let square = scale * scale * radicand;
    bounded(&square)?;
    // Maintained integer sqrt returns floor(sqrt(square)), with no float inputs.
    let lower = square.sqrt();
    let upper = &lower + 1u8;
    if phi {
        Ok(Interval {
            lower: ((scale + lower) / 2u8).into(),
            upper: ((scale + upper + 1u8) / 2u8).into(),
        })
    } else {
        Ok(Interval {
            lower: lower.into(),
            upper: upper.into(),
        })
    }
}

fn certify(bounds: Interval, divisor: &BigInt) -> Result<String, OperationError> {
    if bounds.lower.sign() == Sign::Minus
        || bounds.upper < bounds.lower
        || bounds.lower.bits() > MAX_OPERAND_BITS
        || bounds.upper.bits() > MAX_OPERAND_BITS
    {
        return Err(limit(
            "invalid or oversized constant certification interval",
        ));
    }
    let lower = bounds.lower / divisor;
    let upper = bounds.upper / divisor;
    if lower != upper {
        return Err(limit(
            "requested digits cannot be certified within the fixed precision budget",
        ));
    }
    Ok(lower.to_str_radix(10))
}

/// Look up exact fractional decimal digits with explicit bounded generation.
///
/// Preflights range/index-list semantics before arithmetic. Index 0 excludes the
/// integer part and decimal point. Empty selections are valid; duplicates/order
/// are preserved. The complete chosen prefix is generated, certified and hashed.
/// At most 4096 prefix digits, 1024 explicit indexes, 8192 series steps and 32768
/// bits per integer are allowed. Roots operate on one bounded integer. Structured
/// output stays below 16 KiB. No I/O, runtime precision knob or pipeline adapter.
///
/// pi uses the exact Machin identity; e uses a factorial tail bound; sqrt2/phi
/// use integer-root intervals. Sixteen guard places are fixed: endpoint agreement
/// certifies truncation, and uncertainty is an error, never a guessed digit.
pub fn digits(
    constant: Constant,
    options: DigitOptions<'_>,
) -> Result<DigitSelection, OperationError> {
    let maximum = options.max_digits;
    if !(1..=MAX_DIGITS).contains(&maximum) {
        return Err(invalid(
            "max_digits",
            "max_digits must be 1..=4096 (default 256)",
        ));
    }
    let range = if let Some(indices) = options.indices {
        if options.start.is_some() || options.count.is_some() {
            return Err(invalid("indices", "choose a range or indices, never both"));
        }
        if indices.len() > MAX_INDICES {
            return Err(limit("index selection exceeds 1024 entries").parameter("indices"));
        }
        if indices.iter().any(|&i| i >= maximum) {
            return Err(invalid(
                "indices",
                "index exceeds max_digits; explicitly increase max_digits up to 4096",
            ));
        }
        None
    } else {
        let start = options.start.unwrap_or(0);
        let count = options.count.unwrap_or(16);
        let end = start
            .checked_add(count)
            .ok_or_else(|| invalid("count", "range end overflow"))?;
        if start > maximum || end > maximum {
            return Err(invalid(
                "count",
                "range exceeds max_digits; explicitly increase max_digits up to 4096",
            ));
        }
        Some(start..end)
    };
    // Validated exponent <=4112: scale <13660 bits and root radicands <27324
    // bits. Small-factor series products stay below the independent 32768 cap.
    let scale = BigUint::from(10u8).pow((maximum + GUARD_DIGITS) as u32);
    bounded(&scale)?;
    let mut steps = 0;
    let bounds = match constant {
        Constant::Pi => {
            let a = atan(&scale, 5, &mut steps)?;
            let b = atan(&scale, 239, &mut steps)?;
            Interval {
                lower: 16u8 * a.lower - 4u8 * b.upper,
                upper: 16u8 * a.upper - 4u8 * b.lower,
            }
        }
        Constant::E => exponential(&scale, &mut steps)?,
        Constant::Phi => root(&scale, 5, true)?,
        Constant::Sqrt2 => root(&scale, 2, false)?,
    };
    let divisor = BigInt::from(BigUint::from(10u8).pow(GUARD_DIGITS as u32));
    let certified = certify(bounds, &divisor)?;
    if certified.len() != maximum + 1 || !certified.starts_with(constant.integer()) {
        return Err(limit("certified constant has an unexpected decimal shape"));
    }
    let prefix = &certified[1..];
    let selected = if let Some(range) = range {
        prefix[range].to_owned()
    } else {
        options
            .indices
            .unwrap_or_default()
            .iter()
            .map(|&i| prefix.as_bytes()[i] as char)
            .collect()
    };
    Ok(DigitSelection {
        constant: constant.name(),
        radix: 10,
        integer_part: constant.integer(),
        fractional_zero_based: true,
        digits: selected,
        max_digits: maximum,
        prefix_sha256: hex::encode(Sha256::digest(prefix.as_bytes())),
        certification: "exact_fixed_point_interval_truncation_v1",
    })
}

pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: ID,
    category: "numbers",
    description: "Certified decimal fractional digits of pi/e/phi/sqrt2; index 0 excludes integer part. Explicit max_digits defaults 256, hard cap 4096. Bounded pure-Rust integer intervals, <=8192 series steps/32768-bit operands; ordered selections, exact strings and prefix SHA-256, <=16 KiB result. No I/O or floating approximation.",
    input_type: "constant.selection",
    output_type: "constant.digits",
    deterministic: true,
    reversible: None,
    command: "number digits",
    example: "cube number digits pi --start 256 --count 16 --max-digits 512 --json",
    parameters: &[
        Parameter {
            name: "constant",
            description: "Exact constant name.",
            required: true,
            default: None,
            constraints: ParameterType::Text {
                choices: &["pi", "e", "phi", "sqrt2"],
            },
        },
        Parameter {
            name: "start",
            description: "Zero-based fractional range start; exclusive with indices. A zero-count range may end at max_digits.",
            required: false,
            default: Some(ParameterValue::Unsigned(0)),
            constraints: ParameterType::Unsigned { min: 0, max: 4096 },
        },
        Parameter {
            name: "count",
            description: "Range length, exclusive with indices; checked start+count must fit max_digits.",
            required: false,
            default: Some(ParameterValue::Unsigned(16)),
            constraints: ParameterType::Unsigned { min: 0, max: 4096 },
        },
        Parameter {
            name: "indices",
            description: "Optional ordered fractional indexes; repeats and empty selections preserved. Every index must be below max_digits.",
            required: false,
            default: None,
            constraints: ParameterType::UnsignedList {
                min: 0,
                max: 4095,
                min_length: 0,
                max_length: 1024,
                unique: false,
            },
        },
        Parameter {
            name: "max_digits",
            description: "Explicit certified prefix budget; overrides default 256 but never hard cap 4096. The complete chosen prefix is generated and hashed.",
            required: false,
            default: Some(ParameterValue::Unsigned(256)),
            constraints: ParameterType::Unsigned { min: 1, max: 4096 },
        },
    ],
}];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uncertainty_and_work_operand_budgets_fail_closed() {
        let divisor = BigInt::from(10u8);
        assert!(
            certify(
                Interval {
                    lower: 19u8.into(),
                    upper: 20u8.into()
                },
                &divisor
            )
            .is_err()
        );
        assert!(
            certify(
                Interval {
                    lower: (-1i8).into(),
                    upper: 0u8.into()
                },
                &divisor
            )
            .is_err()
        );
        assert!(
            certify(
                Interval {
                    lower: 3u8.into(),
                    upper: 2u8.into()
                },
                &divisor
            )
            .is_err()
        );
        let huge = BigUint::from(1u8) << MAX_OPERAND_BITS as usize;
        assert!(bounded(&huge).is_err());
        assert!(
            certify(
                Interval {
                    lower: 0u8.into(),
                    upper: huge.into()
                },
                &divisor
            )
            .is_err()
        );
        let mut exhausted = MAX_SERIES_STEPS;
        assert!(charge(&mut exhausted).is_err());
        assert!(atan(&BigUint::from(10u8), 5, &mut exhausted).is_err());
        assert!(exponential(&BigUint::from(10u8), &mut exhausted).is_err());
    }
}
