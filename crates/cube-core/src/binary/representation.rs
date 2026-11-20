//! Bounded exact integer and IEEE 754 bit representations.
use crate::{
    errors::OperationError,
    formal_work::{check_bytes, invalid, parameter, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const RECORDS: usize = 65_536;
const MIB: usize = 1 << 20;

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ByteOrder {
    Big,
    Little,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Integer {
    Signed(i64),
    Unsigned(u64),
}

pub(super) fn decimal_wire_size(n: u64) -> usize {
    const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;
    let digits = n.checked_ilog10().unwrap_or_default() as usize + 1;
    digits + 2 * usize::from(n > MAX_SAFE_INTEGER)
}

impl Integer {
    pub(super) fn wire_size(self) -> usize {
        match self {
            Self::Signed(n) => decimal_wire_size(n.unsigned_abs()) + usize::from(n < 0),
            Self::Unsigned(n) => decimal_wire_size(n),
        }
    }
    pub fn signed(self) -> Result<i64, OperationError> {
        match self {
            Self::Signed(n) => Ok(n),
            Self::Unsigned(n) => i64::try_from(n).map_err(|_| invalid("integer exceeds i64")),
        }
    }

    pub fn unsigned(self) -> Result<u64, OperationError> {
        match self {
            Self::Signed(n) => {
                u64::try_from(n).map_err(|_| invalid("unsigned integer is negative"))
            }
            Self::Unsigned(n) => Ok(n),
        }
    }
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "signedness", rename_all = "snake_case")]
pub enum Integers {
    Signed { values: Vec<i64> },
    Unsigned { values: Vec<u64> },
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Signedness {
    Signed,
    Unsigned,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntegerFormat {
    #[schemars(schema_with = "integer_width_schema")]
    pub bits: u8,
    pub byte_order: ByteOrder,
    pub signedness: Signedness,
}

fn integer_width_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"integer","enum":[8,16,32,64]})
}

fn width(bits: u8) -> Result<usize, OperationError> {
    if matches!(bits, 8 | 16 | 32 | 64) {
        Ok(usize::from(bits / 8))
    } else {
        Err(parameter("integer width must be 8, 16, 32 or 64 bits").parameter("bits"))
    }
}

pub(super) fn raw(bytes: &[u8], order: ByteOrder) -> u64 {
    let mut buffer = [0; 8];
    match order {
        ByteOrder::Big => {
            buffer[8 - bytes.len()..].copy_from_slice(bytes);
            u64::from_be_bytes(buffer)
        }
        ByteOrder::Little => {
            buffer[..bytes.len()].copy_from_slice(bytes);
            u64::from_le_bytes(buffer)
        }
    }
}

fn write(output: &mut Vec<u8>, n: u64, width: usize, order: ByteOrder) {
    match order {
        ByteOrder::Big => output.extend_from_slice(&n.to_be_bytes()[8 - width..]),
        ByteOrder::Little => output.extend_from_slice(&n.to_le_bytes()[..width]),
    }
}

pub(super) fn check_values(values: &[Integer]) -> Result<(), OperationError> {
    check_items(values.len())?;
    let bytes =
        2 + values.len().saturating_sub(1) + values.iter().map(|v| v.wire_size()).sum::<usize>();
    if bytes > MIB {
        return Err(resource("integer values exceed 1 MiB of exact transport"));
    }
    Ok(())
}
fn check_items(count: usize) -> Result<(), OperationError> {
    if count > RECORDS {
        return Err(resource("representation exceeds 65536 records"));
    }
    Ok(())
}

/// Write exact complete units in the declared byte order and signedness.
pub fn encode(values: &[Integer], format: IntegerFormat) -> Result<Vec<u8>, OperationError> {
    let size = width(format.bits)?;
    check_values(values)?;
    let shift = 64 - format.bits;
    let narrow = |value: Integer| match format.signedness {
        Signedness::Signed => {
            let n = value.signed()?;
            if (i64::MIN >> shift..=i64::MAX >> shift).contains(&n) {
                Ok(n as u64)
            } else {
                Err(invalid("signed integer does not fit the declared width"))
            }
        }
        Signedness::Unsigned => {
            let n = value.unsigned()?;
            if n <= u64::MAX >> shift {
                Ok(n)
            } else {
                Err(invalid("unsigned integer does not fit the declared width"))
            }
        }
    };
    let mut output = Vec::with_capacity(values.len() * size);
    for &value in values {
        write(&mut output, narrow(value)?, size, format.byte_order);
    }
    Ok(output)
}

/// Read exact complete units, sign-extending two's-complement signed values.
pub fn decode(data: &[u8], format: IntegerFormat) -> Result<Integers, OperationError> {
    let size = width(format.bits)?;
    if !data.len().is_multiple_of(size) {
        return Err(invalid("input ends with a partial integer unit"));
    }
    check_items(data.len() / size)?;
    let unit = data
        .chunks_exact(size)
        .map(|bytes| raw(bytes, format.byte_order));
    let shift = 64 - format.bits;
    let count = data.len() / size;
    let commas = count.saturating_sub(1);
    match format.signedness {
        Signedness::Signed => {
            let values = unit.map(|n| ((n << shift) as i64) >> shift);
            let bytes: usize = values.clone().map(|n| Integer::Signed(n).wire_size()).sum();
            if bytes + commas + r#"{"signedness":"signed","values":[]}"#.len() > MIB {
                return Err(resource("integer report exceeds 1 MiB"));
            }
            Ok(Integers::Signed {
                values: values.collect(),
            })
        }
        Signedness::Unsigned => {
            let bytes: usize = unit.clone().map(decimal_wire_size).sum();
            if bytes + commas + r#"{"signedness":"unsigned","values":[]}"#.len() > MIB {
                return Err(resource("integer report exceeds 1 MiB"));
            }
            Ok(Integers::Unsigned {
                values: unit.collect(),
            })
        }
    }
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FloatFormat {
    Binary32,
    Binary64,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FloatLayout {
    pub format: FloatFormat,
    pub byte_order: ByteOrder,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    Zero,
    Subnormal,
    Normal,
    Infinity,
    QuietNan,
    SignalingNan,
}

#[derive(Serialize, JsonSchema)]
pub struct FloatRecord {
    pub bits: u64,
    pub sign: bool,
    pub biased_exponent: u16,
    pub fraction: u64,
    pub classification: Classification,
}

fn classify(bits: u64, exp: u32, frac: u32) -> FloatRecord {
    let exp_max = (1 << exp) - 1;
    let biased_exponent = ((bits >> frac) as u16) & exp_max;
    let fraction = bits & ((1 << frac) - 1);
    let classification = match (biased_exponent, fraction) {
        (0, 0) => Classification::Zero,
        (0, _) => Classification::Subnormal,
        (e, 0) if e == exp_max => Classification::Infinity,
        (e, f) if e == exp_max && f & (1 << (frac - 1)) != 0 => Classification::QuietNan,
        (e, _) if e == exp_max => Classification::SignalingNan,
        _ => Classification::Normal,
    };
    FloatRecord {
        bits,
        sign: bits >> (exp + frac) != 0,
        biased_exponent,
        fraction,
        classification,
    }
}

/// Inspect exact IEEE 754 encodings without performing floating-point arithmetic.
pub fn inspect_floats(
    data: &[u8],
    layout: FloatLayout,
) -> Result<Vec<FloatRecord>, OperationError> {
    let (width, exp, frac) = match layout.format {
        FloatFormat::Binary32 => (4, 8, 23),
        FloatFormat::Binary64 => (8, 11, 52),
    };
    if !data.len().is_multiple_of(width) {
        return Err(invalid("input ends with a partial IEEE 754 record"));
    }
    check_items(data.len() / width)?;
    let report: Vec<_> = data
        .chunks_exact(width)
        .map(|bytes| classify(raw(bytes, layout.byte_order), exp, frac))
        .collect();
    // Raw binary64 bits outside the safe JSON range are decimal strings on the wire.
    let quotes = 2 * report.iter().filter(|r| r.bits >= 1 << 53).count();
    check_bytes(&report, (4 * MIB) - quotes)?;
    Ok(report)
}
