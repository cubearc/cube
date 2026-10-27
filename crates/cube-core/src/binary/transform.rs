//! Bounded byte arrays and independent unsigned big-endian integer units.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
pub const MAX_INPUT: usize = 16 * 1024 * 1024;
pub(crate) fn check(data: &[u8], op: &'static str) -> Result<(), OperationError> {
    if data.len() > MAX_INPUT {
        Err(
            OperationError::new(ErrorCode::ResourceLimit, "binary input exceeds 16 MiB")
                .operation(op),
        )
    } else {
        Ok(())
    }
}
fn invalid(op: &'static str, name: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
        .operation(op)
        .parameter(name)
}
fn combine(data: &[u8], key: &[u8], is_and: bool) -> Result<Vec<u8>, OperationError> {
    let op = if is_and { "binary.and" } else { "binary.or" };
    check(data, op)?;
    check(key, op)?;
    if data.len() != key.len() {
        return Err(invalid(
            op,
            "key",
            "AND/OR operands must have equal byte lengths; no repetition or padding",
        ));
    }
    Ok(data
        .iter()
        .zip(key)
        .map(|(a, b)| if is_and { a & b } else { a | b })
        .collect())
}
/// Per-byte AND with an equally long operand. Empty/empty is valid; no key cycling.
pub fn and(data: &[u8], key: &[u8]) -> Result<Vec<u8>, OperationError> {
    combine(data, key, true)
}
/// Per-byte OR with an equally long operand. Empty/empty is valid; no key cycling.
pub fn or(data: &[u8], key: &[u8]) -> Result<Vec<u8>, OperationError> {
    combine(data, key, false)
}
/// Invert each of the eight bits of every byte, preserving length.
pub fn not(data: &[u8]) -> Result<Vec<u8>, OperationError> {
    check(data, "binary.not")?;
    Ok(data.iter().map(|b| !b).collect())
}
/// Reverse byte positions, without reversing bits or interpreting text.
pub fn reverse(data: &[u8]) -> Result<Vec<u8>, OperationError> {
    check(data, "binary.reverse")?;
    Ok(data.iter().rev().copied().collect())
}
pub(crate) fn validate_unit(unit_bytes: usize, op: &'static str) -> Result<(), OperationError> {
    if ![1, 2, 4, 8].contains(&unit_bytes) {
        Err(invalid(op, "unit_bytes", "unit_bytes must be 1, 2, 4 or 8"))
    } else {
        Ok(())
    }
}
pub(crate) fn validate_shift(
    bits: u8,
    unit_bytes: usize,
    op: &'static str,
) -> Result<(), OperationError> {
    validate_unit(unit_bytes, op)?;
    if usize::from(bits) > unit_bytes * 8 {
        Err(invalid(
            op,
            "bits",
            "shift must not exceed the unsigned unit bit width",
        ))
    } else {
        Ok(())
    }
}
fn shift(data: &[u8], bits: u8, unit_bytes: usize, left: bool) -> Result<Vec<u8>, OperationError> {
    let op = if left {
        "binary.shift_left"
    } else {
        "binary.shift_right"
    };
    check(data, op)?;
    validate_shift(bits, unit_bytes, op)?;
    if !data.len().is_multiple_of(unit_bytes) {
        return Err(invalid(
            op,
            "unit_bytes",
            "input must contain complete unsigned integer units; no padding",
        ));
    }
    let mask = if unit_bytes == 8 {
        u64::MAX
    } else {
        (1u64 << (unit_bytes * 8)) - 1
    };
    let mut output = Vec::with_capacity(data.len());
    for unit in data.chunks_exact(unit_bytes) {
        let mut bytes = [0; 8];
        bytes[8 - unit_bytes..].copy_from_slice(unit);
        let n = u64::from_be_bytes(bytes);
        let n = if usize::from(bits) == unit_bytes * 8 {
            0
        } else if left {
            (n << bits) & mask
        } else {
            n >> bits
        };
        output.extend_from_slice(&n.to_be_bytes()[8 - unit_bytes..]);
    }
    Ok(output)
}
/// Zero-filled logical shift of independent 8/16/32/64-bit unsigned big-endian
/// units. High bits are discarded; no carry crosses unit boundaries. A shift
/// equal to the unit width yields zero, larger shifts and partial units fail.
pub fn shift_left(data: &[u8], bits: u8, unit_bytes: usize) -> Result<Vec<u8>, OperationError> {
    shift(data, bits, unit_bytes, true)
}
/// Rightward counterpart of `shift_left`, with the same fixed-width semantics.
pub fn shift_right(data: &[u8], bits: u8, unit_bytes: usize) -> Result<Vec<u8>, OperationError> {
    shift(data, bits, unit_bytes, false)
}
/// Reverse byte order within each complete 1/2/4/8-byte unit. Applying twice
/// restores input; this does not reverse the order of integer units themselves.
pub fn swap_endian(data: &[u8], unit_bytes: usize) -> Result<Vec<u8>, OperationError> {
    let op = "binary.swap_endian";
    check(data, op)?;
    validate_unit(unit_bytes, op)?;
    if !data.len().is_multiple_of(unit_bytes) {
        return Err(invalid(
            op,
            "unit_bytes",
            "input must contain complete units; no padding",
        ));
    }
    Ok(data
        .chunks_exact(unit_bytes)
        .flat_map(|u| u.iter().rev().copied())
        .collect())
}
/// XOR equally long buffers without repeating, padding or truncating either operand.
/// Each operand is limited to 16 MiB; empty/empty succeeds.
pub fn xor_exact(data: &[u8], key: &[u8]) -> Result<Vec<u8>, OperationError> {
    let op = "binary.xor_exact";
    paired(data, key, 1, op)?;
    Ok(data.iter().zip(key).map(|(a, b)| a ^ b).collect())
}
fn complete(data: &[u8], unit_bytes: usize, op: &'static str) -> Result<(), OperationError> {
    check(data, op)?;
    validate_unit(unit_bytes, op)?;
    if !data.len().is_multiple_of(unit_bytes) {
        return Err(invalid(
            op,
            "unit_bytes",
            "input must contain complete big-endian units",
        ));
    }
    Ok(())
}
fn paired(
    data: &[u8],
    key: &[u8],
    unit_bytes: usize,
    op: &'static str,
) -> Result<(), OperationError> {
    complete(data, unit_bytes, op)?;
    check(key, op)?;
    if data.len() != key.len() {
        return Err(invalid(op, "key", "operands must have equal byte lengths"));
    }
    Ok(())
}
fn unsigned(unit: &[u8]) -> u64 {
    let mut bytes = [0; 8];
    bytes[8 - unit.len()..].copy_from_slice(unit);
    u64::from_be_bytes(bytes)
}
fn rotate(
    data: &[u8],
    count: u64,
    unit_bytes: usize,
    left: bool,
) -> Result<Vec<u8>, OperationError> {
    let op = if left {
        "binary.rotate_left"
    } else {
        "binary.rotate_right"
    };
    complete(data, unit_bytes, op)?;
    let width = (unit_bytes * 8) as u32;
    let count = (count % u64::from(width)) as u32;
    let mut output = Vec::with_capacity(data.len());
    for unit in data.chunks_exact(unit_bytes) {
        let n = unsigned(unit);
        let rotated = if count == 0 {
            n
        } else if left {
            (n << count) | (n >> (width - count))
        } else {
            (n >> count) | n.wrapping_shl(width - count)
        };
        output.extend_from_slice(&rotated.to_be_bytes()[8 - unit_bytes..]);
    }
    Ok(output)
}
/// Rotate independent unsigned big-endian 1/2/4/8-byte units left.
/// The full u64 count is reduced modulo the unit bit width. At most 16 MiB.
///
/// ```
/// use cube_core::binary::transform::{rotate_left, rotate_right};
/// let rotated = rotate_left(&[0x80, 1], 1, 2)?;
/// assert_eq!(rotated, [0, 3]);
/// assert_eq!(rotate_right(&rotated, 1, 2)?, [0x80, 1]);
/// # Ok::<(), cube_core::errors::OperationError>(())
/// ```
pub fn rotate_left(data: &[u8], count: u64, unit_bytes: usize) -> Result<Vec<u8>, OperationError> {
    rotate(data, count, unit_bytes, true)
}
/// Rightward counterpart of [`rotate_left`], with identical bounds and byte order.
pub fn rotate_right(data: &[u8], count: u64, unit_bytes: usize) -> Result<Vec<u8>, OperationError> {
    rotate(data, count, unit_bytes, false)
}
fn arithmetic(
    data: &[u8],
    key: &[u8],
    unit_bytes: usize,
    subtract: bool,
) -> Result<Vec<u8>, OperationError> {
    let op = if subtract {
        "binary.sub_wrapping"
    } else {
        "binary.add_wrapping"
    };
    paired(data, key, unit_bytes, op)?;
    let mut output = Vec::with_capacity(data.len());
    for (a, b) in data
        .chunks_exact(unit_bytes)
        .zip(key.chunks_exact(unit_bytes))
    {
        let n = if subtract {
            unsigned(a).wrapping_sub(unsigned(b))
        } else {
            unsigned(a).wrapping_add(unsigned(b))
        };
        output.extend_from_slice(&n.to_be_bytes()[8 - unit_bytes..]);
    }
    Ok(output)
}
/// Add paired unsigned big-endian units modulo 2^(unit_bytes*8), discarding carry
/// independently. Equal lengths, complete 1/2/4/8-byte units, <=16 MiB per operand.
pub fn add_wrapping(data: &[u8], key: &[u8], unit_bytes: usize) -> Result<Vec<u8>, OperationError> {
    arithmetic(data, key, unit_bytes, false)
}
/// Subtract paired units with the same bounds as [`add_wrapping`]; no borrow
/// crosses a unit boundary, including for unsigned 64-bit underflow.
pub fn sub_wrapping(data: &[u8], key: &[u8], unit_bytes: usize) -> Result<Vec<u8>, OperationError> {
    arithmetic(data, key, unit_bytes, true)
}
const KEY: Parameter = Parameter {
    name: "key",
    description: "Exact equally long byte operand, including empty/empty. Never repeat or pad.",
    required: true,
    default: None,
    constraints: ParameterType::Bytes { min_length: 0 },
};
const UNIT: Parameter = Parameter {
    name: "unit_bytes",
    description: "Independent unsigned big-endian units: 1/2/4/8 bytes; partial units fail.",
    required: false,
    default: Some(ParameterValue::Unsigned(1)),
    constraints: ParameterType::UnsignedChoices {
        choices: &[1, 2, 4, 8],
    },
};
const BITS: Parameter = Parameter {
    name: "bits",
    description: "Logical shift count, 0..unit_bytes*8; equal width gives zero, larger shifts fail.",
    required: true,
    default: None,
    constraints: ParameterType::Unsigned { min: 0, max: 64 },
};
const COUNT: Parameter = Parameter {
    name: "count",
    description: "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
    required: true,
    default: None,
    constraints: ParameterType::Unsigned {
        min: 0,
        max: u64::MAX,
    },
};
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "binary.and",
        category: "binary",
        description: "Bounded per-byte AND of equal-length operands; no repetition/padding; <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[KEY],
        deterministic: true,
        reversible: Some(false),
        command: "binary and",
        example: "cube binary and --key-hex ff0f --text AB",
    },
    Descriptor {
        id: "binary.or",
        category: "binary",
        description: "Bounded per-byte OR of equal-length operands; no repetition/padding; <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[KEY],
        deterministic: true,
        reversible: Some(false),
        command: "binary or",
        example: "cube binary or --key-hex 0080 --text AB",
    },
    Descriptor {
        id: "binary.not",
        category: "binary",
        description: "Invert all eight bits per byte; preserve byte length, <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[],
        deterministic: true,
        reversible: Some(true),
        command: "binary not",
        example: "cube binary not --text AB",
    },
    Descriptor {
        id: "binary.reverse",
        category: "binary",
        description: "Reverse byte positions, not bits or Unicode scalars; <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[],
        deterministic: true,
        reversible: Some(true),
        command: "binary reverse",
        example: "cube binary reverse --text AB",
    },
    Descriptor {
        id: "binary.shift_left",
        category: "binary",
        description: "Zero-filled left shifts within independent unsigned big-endian units; overflow bits discarded; <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[BITS, UNIT],
        deterministic: true,
        reversible: Some(false),
        command: "binary shift-left",
        example: "cube binary shift-left --bits 1 --unit-bytes 2 --text AB",
    },
    Descriptor {
        id: "binary.shift_right",
        category: "binary",
        description: "Logical right shifts within independent unsigned big-endian units; no signed arithmetic; <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[BITS, UNIT],
        deterministic: true,
        reversible: Some(false),
        command: "binary shift-right",
        example: "cube binary shift-right --bits 1 --unit-bytes 2 --text AB",
    },
    Descriptor {
        id: "binary.swap_endian",
        category: "binary",
        description: "Reverse byte order per complete 1/2/4/8-byte unit; unit order unchanged; <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[UNIT],
        deterministic: true,
        reversible: Some(true),
        command: "binary swap-endian",
        example: "cube binary swap-endian --unit-bytes 2 --text ABCD",
    },
    Descriptor {
        id: "binary.rotate_left",
        category: "binary",
        description: "Rotate left per independent big-endian unit; u64 count modulo width; <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[COUNT, UNIT],
        deterministic: true,
        reversible: Some(true),
        command: "binary rotate-left",
        example: "cube binary rotate-left --count 1 --text A",
    },
    Descriptor {
        id: "binary.rotate_right",
        category: "binary",
        description: "Rotate right per independent big-endian unit; u64 count modulo width; <=16 MiB.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[COUNT, UNIT],
        deterministic: true,
        reversible: Some(true),
        command: "binary rotate-right",
        example: "cube binary rotate-right --count 1 --text A",
    },
    Descriptor {
        id: "binary.xor_exact",
        category: "binary",
        description: "Exact equal-length byte XOR; no key repetition; <=16 MiB per operand.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[KEY],
        deterministic: true,
        reversible: Some(true),
        command: "binary xor-exact",
        example: "cube binary xor-exact --key-hex ff --text A",
    },
    Descriptor {
        id: "binary.add_wrapping",
        category: "binary",
        description: "Add paired big-endian unsigned units modulo their width; <=16 MiB per operand.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[KEY, UNIT],
        deterministic: true,
        reversible: Some(true),
        command: "binary add-wrapping",
        example: "cube binary add-wrapping --key-hex 01 --text A",
    },
    Descriptor {
        id: "binary.sub_wrapping",
        category: "binary",
        description: "Subtract paired big-endian unsigned units modulo their width; <=16 MiB per operand.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: &[KEY, UNIT],
        deterministic: true,
        reversible: Some(true),
        command: "binary sub-wrapping",
        example: "cube binary sub-wrapping --key-hex 01 --text A",
    },
];
