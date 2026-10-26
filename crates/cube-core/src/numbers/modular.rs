//! Exact u64 Euclidean and modular arithmetic, with fixed storage.
//!
//! The Euclidean algorithm takes fewer than 128 divisions for u64 inputs
//! (successive Fibonacci numbers are the worst case). We enforce a conservative
//! 256-round ceiling. In extended Euclid, generated coefficient magnitudes,
//! including the terminal zero-remainder pair, are at most max(a,b)/gcd(a,b).
//! Their alternating signs make each quotient product no larger than the next
//! coefficient magnitude. Thus all intermediate products/subtractions fit i128;
//! checked arithmetic also enforces that invariant. Modular products of two u64
//! values fit u128, and exponentiation takes at most 64 bit rounds. These routines
//! allocate no input-dependent storage and are not constant-time cryptography.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType},
};

/// Conservative enforced limit for Euclidean rounds, independent of inputs.
pub const MAX_ITERATIONS: usize = 256;
/// Strict unsigned decimal transport: at most 20 ASCII digits, no sign/space.
pub const MAX_DECIMAL_BYTES: usize = 20;

fn error(op: &'static str, code: ErrorCode, message: &str) -> OperationError {
    OperationError::new(code, message).operation(op)
}
fn limit(op: &'static str) -> OperationError {
    error(
        op,
        ErrorCode::ResourceLimit,
        "fixed-width arithmetic budget exceeded",
    )
}
/// Parse one strict u64 decimal operand for a known operation and parameter.
/// Leading zeros are allowed within the 20-byte cap; signs/prefixes are invalid.
pub fn parse_operand(
    text: &str,
    op: &'static str,
    parameter: &'static str,
) -> Result<u64, OperationError> {
    if text.len() > MAX_DECIMAL_BYTES {
        return Err(limit(op).parameter(parameter));
    }
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "operand requires unsigned ASCII decimal digits",
        )
        .parameter(parameter));
    }
    text.parse().map_err(|_| limit(op).parameter(parameter))
}
/// Greatest common divisor; gcd(0,0)=0. At most 256 rounds, fixed storage.
///
/// ```
/// assert_eq!(cube_core::numbers::modular::gcd(48, 18).unwrap(), 6);
/// ```
pub fn gcd(mut a: u64, mut b: u64) -> Result<u64, OperationError> {
    for _ in 0..MAX_ITERATIONS {
        if b == 0 {
            return Ok(a);
        }
        (a, b) = (b, a % b);
    }
    Err(limit("number.gcd"))
}
/// Exact Bézout identity a*x+b*y=gcd, with deterministic Euclidean coefficients.
/// Serialization uses decimal strings so JSON consumers never round integers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ExtendedGcd {
    #[serde(serialize_with = "decimal")]
    pub gcd: u64,
    #[serde(serialize_with = "decimal")]
    pub x: i128,
    #[serde(serialize_with = "decimal")]
    pub y: i128,
}
fn decimal<S: serde::Serializer, T: std::fmt::Display>(
    value: &T,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}
/// Extended Euclid with floor quotients and initial pairs (1,0), (0,1).
/// Zero conventions: (0,0)->(0,0,0), (a,0)->(a,1,0), (0,b)->(b,0,1).
/// See the module-level intermediate-width proof; checked operations never wrap.
///
/// ```
/// let result = cube_core::numbers::modular::extended_gcd(240, 46).unwrap();
/// assert_eq!((result.gcd, result.x, result.y), (2, -9, 47));
/// ```
pub fn extended_gcd(a: u64, b: u64) -> Result<ExtendedGcd, OperationError> {
    const OP: &str = "number.extended_gcd";
    if a == 0 && b == 0 {
        return Ok(ExtendedGcd { gcd: 0, x: 0, y: 0 });
    }
    let (mut r, mut next_r) = (a, b);
    let (mut x, mut next_x) = (1i128, 0i128);
    let (mut y, mut next_y) = (0i128, 1i128);
    for _ in 0..MAX_ITERATIONS {
        if next_r == 0 {
            return Ok(ExtendedGcd { gcd: r, x, y });
        }
        let q = i128::from(r / next_r);
        let subtract = |old: i128, next: i128| {
            q.checked_mul(next)
                .and_then(|product| old.checked_sub(product))
                .ok_or_else(|| limit(OP))
        };
        (x, next_x) = (next_x, subtract(x, next_x)?);
        (y, next_y) = (next_y, subtract(y, next_y)?);
        (r, next_r) = (next_r, r % next_r);
    }
    Err(limit(OP))
}
/// Canonical inverse in 0..modulus, requiring modulus>=2 and coprime operands.
/// A nonexistent inverse is invalid_input, not a zero result.
pub fn mod_inverse(a: u64, modulus: u64) -> Result<u64, OperationError> {
    const OP: &str = "number.mod_inverse";
    if modulus < 2 {
        return Err(error(
            OP,
            ErrorCode::InvalidParameter,
            "inverse modulus must be at least 2",
        )
        .parameter("modulus"));
    }
    let result = extended_gcd(a, modulus).map_err(|e| e.operation(OP))?;
    if result.gcd != 1 {
        return Err(error(
            OP,
            ErrorCode::InvalidInput,
            "inverse does not exist for noncoprime operands",
        ));
    }
    Ok(result.x.rem_euclid(i128::from(modulus)) as u64)
}
fn multiply(a: u64, b: u64, modulus: u64) -> Result<u64, OperationError> {
    let product = u128::from(a)
        .checked_mul(u128::from(b))
        .ok_or_else(|| limit("number.mod_pow"))?;
    Ok((product % u128::from(modulus)) as u64)
}
/// Exact modular power in at most 64 exponent-bit rounds. Modulus must be >0.
/// a^0 (including 0^0) is 1 mod modulus; modulus 1 always yields 0.
///
/// ```
/// assert_eq!(cube_core::numbers::modular::mod_pow(4, 13, 497).unwrap(), 445);
/// ```
pub fn mod_pow(base: u64, mut exponent: u64, modulus: u64) -> Result<u64, OperationError> {
    const OP: &str = "number.mod_pow";
    if modulus == 0 {
        return Err(
            error(OP, ErrorCode::InvalidParameter, "modulus must be positive").parameter("modulus"),
        );
    }
    let mut base = base % modulus;
    let mut result = 1 % modulus;
    for _ in 0..64 {
        if exponent == 0 {
            return Ok(result);
        }
        if exponent & 1 == 1 {
            result = multiply(result, base, modulus)?;
        }
        exponent >>= 1;
        if exponent != 0 {
            base = multiply(base, base, modulus)?;
        }
    }
    Ok(result)
}
const fn operand(name: &'static str) -> Parameter {
    Parameter {
        name,
        description: "Exact u64; CLI <=20 unsigned ASCII decimal digits, Python builtin int (no bool/float).",
        required: true,
        default: None,
        constraints: ParameterType::Unsigned {
            min: 0,
            max: u64::MAX,
        },
    }
}
const MODULUS: Parameter = Parameter {
    name: "modulus",
    description: "Explicit positive modulus; inverse additionally requires >=2 and coprime operands.",
    required: true,
    default: None,
    constraints: ParameterType::Unsigned {
        min: 1,
        max: u64::MAX,
    },
};
const INVERSE_MODULUS: Parameter = Parameter {
    constraints: ParameterType::Unsigned {
        min: 2,
        max: u64::MAX,
    },
    ..MODULUS
};
/// Direct operations only; these numeric results have no implicit pipeline adapter.
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "number.gcd",
        category: "numbers",
        description: "Exact u64 GCD, including gcd(0,0)=0; fixed storage, <=256 Euclidean rounds.",
        input_type: "number.u64_pair",
        output_type: "number.u64",
        deterministic: true,
        reversible: Some(false),
        command: "number gcd",
        example: "cube number gcd --a 48 --b 18",
        parameters: &[operand("a"), operand("b")],
    },
    Descriptor {
        id: "number.extended_gcd",
        category: "numbers",
        description: "Exact GCD and signed i128 Bezout coefficients; deterministic Euclid; JSON decimal strings.",
        input_type: "number.u64_pair",
        output_type: "number.bezout",
        deterministic: true,
        reversible: Some(false),
        command: "number extended-gcd",
        example: "cube number extended-gcd --a 240 --b 46 --json",
        parameters: &[operand("a"), operand("b")],
    },
    Descriptor {
        id: "number.mod_inverse",
        category: "numbers",
        description: "Canonical u64 modular inverse; noncoprime input errors; <=256 Euclidean rounds.",
        input_type: "number.modular_inverse_request",
        output_type: "number.u64",
        deterministic: true,
        reversible: Some(false),
        command: "number mod-inverse",
        example: "cube number mod-inverse --a 3 --modulus 11",
        parameters: &[operand("a"), INVERSE_MODULUS],
    },
    Descriptor {
        id: "number.mod_pow",
        category: "numbers",
        description: "Exact u64 modular power with u128 products, <=64 bit rounds; exponent zero gives 1 mod m.",
        input_type: "number.modular_power_request",
        output_type: "number.u64",
        deterministic: true,
        reversible: Some(false),
        command: "number mod-pow",
        example: "cube number mod-pow --base 4 --exponent 13 --modulus 497",
        parameters: &[operand("base"), operand("exponent"), MODULUS],
    },
];
