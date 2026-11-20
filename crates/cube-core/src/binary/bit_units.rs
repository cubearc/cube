//! Bit packing and fixed-width bit counts over exact bytes.
use super::{
    packing,
    representation::{ByteOrder, raw},
    transform,
};
use crate::{
    errors::OperationError,
    formal_work::{invalid, parameter, resource},
};
use schemars::JsonSchema;
use serde::Deserialize;

const BIT_BYTES: usize = 1 << 20;

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Tail {
    #[default]
    Reject,
    PadZero,
}
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pack {
    pub bit_order: packing::BitOrder,
    #[serde(default)]
    pub tail: Tail,
}
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Unpack {
    pub bit_order: packing::BitOrder,
    #[schemars(range(max = 8388608))]
    pub valid_bits: usize,
}

/// Pack byte-valued bits and return packed data with the exact valid bit count.
pub fn pack(bits: &[u8], layout: Pack) -> Result<(Vec<u8>, usize), OperationError> {
    if bits.len() > BIT_BYTES {
        return Err(resource("packing input exceeds 1 MiB"));
    }
    if !bits.len().is_multiple_of(8) && matches!(layout.tail, Tail::Reject) {
        return Err(invalid("packing input ends with a partial byte"));
    }
    if bits.iter().any(|&bit| bit > 1) {
        return Err(invalid("packing accepts only byte-valued bits 0 and 1"));
    }
    let mut output = vec![0; bits.len().div_ceil(8)];
    for (i, &bit) in bits.iter().enumerate() {
        packing::store(&mut output, i, bit, layout.bit_order);
    }
    Ok((output, bits.len()))
}

/// Unpack the requested bit prefix to exact bytes whose values are zero or one.
pub fn unpack(data: &[u8], layout: Unpack) -> Result<(Vec<u8>, usize), OperationError> {
    if data.len() > BIT_BYTES {
        return Err(resource("unpacking input exceeds 1 MiB"));
    }
    if layout.valid_bits > data.len() * 8 {
        return Err(parameter("valid_bits exceeds the input capacity").parameter("valid_bits"));
    }
    let output = (0..layout.valid_bits)
        .map(|i| (data[i / 8] >> packing::bit_shift(i, layout.bit_order)) & 1)
        .collect();
    Ok((output, layout.valid_bits))
}

fn unit_width_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    schemars::json_schema!({"type":"integer","enum":[1,2,4,8]})
}

fn byte() -> usize {
    1
}
#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mask {
    #[serde(default = "byte")]
    #[schemars(schema_with = "unit_width_schema")]
    pub unit_bytes: usize,
    pub mask: u64,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CountKind {
    Population,
    LeadingZeros,
    TrailingZeros,
}

#[derive(Clone, Copy, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Count {
    #[serde(default = "byte")]
    #[schemars(schema_with = "unit_width_schema")]
    pub unit_bytes: usize,
    pub kind: CountKind,
}

/// Repeat an explicit mask over independent complete big-endian units.
pub fn mask(data: &[u8], layout: Mask) -> Result<Vec<u8>, OperationError> {
    const OP: &str = "binary.mask";
    transform::complete(data, layout.unit_bytes, OP)?;
    let shift = 8 * (8 - layout.unit_bytes);
    if layout.mask > u64::MAX >> shift {
        return Err(parameter("mask does not fit the selected unit width").parameter("mask"));
    }
    let operand = layout.mask.to_be_bytes();
    let operand = &operand[8 - layout.unit_bytes..];
    let mut output = Vec::with_capacity(data.len());
    for unit in data.chunks_exact(layout.unit_bytes) {
        output.extend(
            unit.iter()
                .zip(operand)
                .map(|(&left, &right)| transform::and_byte(left, right)),
        );
    }
    Ok(output)
}

/// Count selected bits in ordered big-endian units, including zero units.
pub fn count(data: &[u8], layout: Count) -> Result<Vec<u32>, OperationError> {
    transform::complete(data, layout.unit_bytes, "binary.count_bits")?;
    if data.len() / layout.unit_bytes > 65_536 {
        return Err(resource("bit count report exceeds 65536 units"));
    }
    let bits = (layout.unit_bytes * 8) as u32;
    Ok(data
        .chunks_exact(layout.unit_bytes)
        .map(|bytes| {
            let n = raw(bytes, ByteOrder::Big);
            match layout.kind {
                CountKind::Population => n.count_ones(),
                CountKind::LeadingZeros => n.leading_zeros() - (64 - bits),
                CountKind::TrailingZeros => n.trailing_zeros().min(bits),
            }
        })
        .collect())
}
