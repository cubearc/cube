//! Bounded 64-bit LEB128, Protobuf uint64 and ZigZag encodings.
use super::representation::{Integer, check_values, decimal_wire_size};
use crate::{
    errors::OperationError,
    formal_work::{invalid, resource},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_BYTES: usize = 1 << 20;
const MAX_VALUES: usize = 65_536;

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Uleb128,
    Sleb128,
    ProtobufUint64,
    Zigzag64,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Encode {
    pub profile: Profile,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Prefix,
    Full,
    Batch,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Decode {
    pub profile: Profile,
    pub mode: Mode,
    #[serde(default)]
    pub allow_noncanonical: bool,
}

#[derive(Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Value {
    Signed(i64),
    Unsigned(u64),
}

#[derive(Serialize, JsonSchema)]
pub struct Record {
    pub value: Value,
    pub consumed: usize,
}

fn write_unsigned(mut n: u64, output: &mut Vec<u8>) {
    while n >= 0x80 {
        output.push((n as u8 & 0x7f) | 0x80);
        n >>= 7;
    }
    output.push(n as u8);
}

fn write_signed(mut n: i64, output: &mut Vec<u8>) {
    loop {
        let byte = n as u8 & 0x7f;
        n >>= 7;
        if (n == 0 && byte & 0x40 == 0) || (n == -1 && byte & 0x40 != 0) {
            output.push(byte);
            return;
        }
        output.push(byte | 0x80);
    }
}

/// Encode each integer using its canonical shortest representation.
pub fn encode(values: &[Integer], options: Encode) -> Result<Vec<u8>, OperationError> {
    check_values(values)?;
    let mut output = Vec::with_capacity(values.len());
    for &value in values {
        match options.profile {
            Profile::Uleb128 | Profile::ProtobufUint64 => {
                write_unsigned(value.unsigned()?, &mut output)
            }
            Profile::Sleb128 => write_signed(value.signed()?, &mut output),
            Profile::Zigzag64 => {
                let n = value.signed()?;
                write_unsigned(((n as u64) << 1) ^ ((n >> 63) as u64), &mut output);
            }
        }
    }
    Ok(output)
}

fn read(data: &[u8], options: Decode) -> Result<Record, OperationError> {
    let signed = matches!(options.profile, Profile::Sleb128);
    let mut bits = 0u64;
    for (index, &byte) in data.iter().take(10).enumerate() {
        let payload = byte & 0x7f;
        let last = byte & 0x80 == 0;
        if index == 9
            && (!last
                || if signed {
                    !matches!(payload, 0 | 0x7f)
                } else {
                    payload > 1
                })
        {
            return Err(invalid("varint exceeds its 64-bit domain"));
        }
        bits |= u64::from(payload) << (index * 7);
        if !last {
            continue;
        }
        if !options.allow_noncanonical && index > 0 {
            let prior_sign = data[index - 1] & 0x40 != 0;
            let redundant = if signed {
                (payload == 0 && !prior_sign) || (payload == 0x7f && prior_sign)
            } else {
                payload == 0
            };
            if redundant {
                return Err(invalid("varint is not in shortest canonical form"));
            }
        }
        if signed && index < 9 && payload & 0x40 != 0 {
            bits |= u64::MAX << ((index + 1) * 7);
        }
        let value = match options.profile {
            Profile::Sleb128 => Value::Signed(bits as i64),
            Profile::Zigzag64 => Value::Signed((bits >> 1) as i64 ^ -((bits & 1) as i64)),
            Profile::Uleb128 | Profile::ProtobufUint64 => Value::Unsigned(bits),
        };
        return Ok(Record {
            value,
            consumed: index + 1,
        });
    }
    Err(invalid("truncated varint"))
}

impl Record {
    fn wire_size(&self) -> usize {
        let value = match self.value {
            Value::Signed(n) => Integer::Signed(n).wire_size(),
            Value::Unsigned(n) => Integer::Unsigned(n).wire_size(),
        };
        22 + value + decimal_wire_size(self.consumed as u64)
    }
}

/// Decode one prefix, exactly one complete value, or a complete batch.
pub fn decode(data: &[u8], options: Decode) -> Result<Vec<Record>, OperationError> {
    if data.len() > MAX_BYTES {
        return Err(resource("varint input exceeds 1 MiB"));
    }
    if !matches!(options.mode, Mode::Batch) {
        let record = read(data, options)?;
        if matches!(options.mode, Mode::Full) && record.consumed != data.len() {
            return Err(invalid("trailing bytes after the complete varint"));
        }
        return Ok(vec![record]);
    }
    let mut offset = 0;
    let mut report = Vec::new();
    let mut wire_size = 1;
    while offset < data.len() {
        if report.len() == MAX_VALUES {
            return Err(resource("varint report exceeds 65536 integers"));
        }
        let record = read(&data[offset..], options)?;
        wire_size += record.wire_size() + 1;
        if wire_size > MAX_BYTES {
            return Err(resource("varint report exceeds 1 MiB"));
        }
        offset += record.consumed;
        report.push(record);
    }
    Ok(report)
}
