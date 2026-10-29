//! Ordered, bounded composition without implicit encoding or normalization.
mod contracts;
pub mod structured;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use contracts::CONTRACTS;

/// Combined operand/separator and result ceiling, in bytes (1 MiB).
pub const MAX_BYTES: usize = 1024 * 1024;

fn size(lengths: &[usize], operation: &'static str) -> Result<usize, OperationError> {
    lengths.iter().try_fold(0usize, |total, length| {
        total
            .checked_add(*length)
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| {
                OperationError::new(
                    ErrorCode::ResourceLimit,
                    "combined operands and separator exceed 1 MiB",
                )
                .operation(operation)
            })
    })
}

/// Return `left + separator + right` exactly, including empty operands, NUL,
/// CRLF and combining scalars. The separator is always included once. No
/// normalization, trimming or newline is added. Combined UTF-8 bytes <=1 MiB;
/// validate the full size before allocating the result.
pub fn concat_text(left: &str, right: &str, separator: &str) -> Result<String, OperationError> {
    let length = size(&[left.len(), separator.len(), right.len()], "text.concat")?;
    let mut output = String::with_capacity(length);
    output.push_str(left);
    output.push_str(separator);
    output.push_str(right);
    Ok(output)
}

/// Append right bytes to left bytes, preserving every byte and operand order.
/// Empty operands are valid. No text interpretation or separator is implicit.
/// Combined input/output <=1 MiB, checked before allocation.
pub fn concat_bytes(left: &[u8], right: &[u8]) -> Result<Vec<u8>, OperationError> {
    let length = size(&[left.len(), right.len()], "bytes.concat")?;
    let mut output = Vec::with_capacity(length);
    output.extend_from_slice(left);
    output.extend_from_slice(right);
    Ok(output)
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue, metadata as m};
const fn profile(
    descriptor: Descriptor,
    name: &'static str,
    rust_api: &'static str,
    python_api: &'static str,
) -> m::Entry {
    m::Entry {
        descriptor,
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "ordered_composition",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::BinaryTransform,
                limitations: "Ordered composition provides no secrecy, authentication or normalization.",
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: descriptor.description,
                alphabet: "Exact ordered operands; Unicode text and arbitrary bytes have distinct operations.",
                key: "No key.",
                key_advancement: "Not applicable.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[m::ResourceBound {
                name: "combined_input_and_output_bytes",
                maximum: MAX_BYTES as u64,
                unit: "bytes",
                scope: m::Scope::Core,
            }],
            resource_notes: "Combined operand bytes, including the text separator, are checked before result allocation. The output has exactly that length. Interfaces may impose lower limits.",
            rust_api,
            python_api,
            references: &[
                "https://github.com/cubearc/cube/blob/main/docs/reference/composition.md",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    profile(
        Descriptor {
            id: "text.concat",
            category: "text",
            description: "Concatenate left text, one explicit separator and right text in order; preserve all Unicode/NUL/line endings, combined <=1 MiB.",
            input_type: "text.pair",
            output_type: "text",
            parameters: &[Parameter {
                name: "separator",
                description: "Included once between operands, including when either is empty; default empty text.",
                required: false,
                default: Some(ParameterValue::Text("")),
                constraints: ParameterType::Text { choices: &[] },
            }],
            deterministic: true,
            reversible: Some(false),
            command: "compose text",
            example: "cube compose text --left The --right key --separator ' '",
        },
        "Concatenate text",
        "cube_core::composition::concat_text",
        "concat_text",
    ),
    profile(
        Descriptor {
            id: "bytes.concat",
            category: "binary",
            description: "Concatenate left bytes then right bytes exactly; empty operands valid, no implicit encoding/separator, combined <=1 MiB.",
            input_type: "bytes.pair",
            output_type: "bytes",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: "compose bytes",
            example: "cube compose bytes --left-hex 00ff --right-hex 800a",
        },
        "Concatenate bytes",
        "cube_core::composition::concat_bytes",
        "concat_bytes",
    ),
];
pub const OPERATIONS: &[Descriptor] = &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
