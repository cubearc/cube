//! Transform arbitrary bytes without UTF-8 conversion.
pub mod transform;

use crate::Error;

/// XOR each input byte with the corresponding byte of a repeating key.
///
/// Applying the same key twice restores the original bytes. Repeating-key XOR
/// is not secure encryption.
///
/// # Errors
/// Returns [`Error::EmptyKey`] when `key` is empty, even if `input` is empty.
///
/// ```
/// # fn main() -> Result<(), cube_core::Error> {
/// assert_eq!(cube_core::binary::xor(b"Hello", b"key")?, [35, 0, 21, 7, 10]);
/// # Ok(())
/// # }
/// ```
pub fn xor(input: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    if key.is_empty() {
        return Err(Error::EmptyKey);
    }
    Ok(input
        .iter()
        .zip(key.iter().cycle())
        .map(|(a, b)| a ^ b)
        .collect())
}

const XOR_PARAMETERS: &[crate::operations::Parameter] = &[crate::operations::Parameter {
    name: "key",
    description: "Nonempty repeating binary key.",
    required: true,
    default: None,
    constraints: crate::operations::ParameterType::Bytes { min_length: 1 },
}];
/// Static metadata for operations implemented in this module.
const PROFILED: [crate::operations::metadata::Entry; 1] = [{
    const DECLARATION: crate::operations::Descriptor = crate::operations::Descriptor {
        id: "binary.xor",
        category: "binary",
        description: "XOR exact bytes with a nonempty repeating key; not secure encryption.",
        input_type: "bytes",
        output_type: "bytes",
        parameters: XOR_PARAMETERS,
        deterministic: true,
        reversible: Some(true),
        command: "binary xor",
        example: "cube binary xor --key-hex ff --text Hello",
    };
    crate::operations::metadata::Entry {
        descriptor: DECLARATION,
        metadata: Some(crate::operations::metadata::Metadata {
            schema_version: 1,
            canonical_name: "Repeating-key XOR",
            family: "binary_transforms",
            aliases: &[],
            security: crate::operations::metadata::Security {
                classification: crate::operations::metadata::Classification::BinaryTransform,
                limitations: "A reversible/bitwise transformation does not by itself provide secure encryption or authentication.",
                private_input: false,
                private_output: false,
                sensitive_parameters: &["key"],
            },
            profile: crate::operations::metadata::Profile {
                variant: DECLARATION.description,
                alphabet: "Exact byte sequence, including NUL/non-UTF8; no text interpretation.",
                key: "Required nonempty repeated byte key.",
                key_advancement: "Advance one key byte per data byte, repeating modulo nonempty key length.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[
                crate::operations::metadata::ResourceBound {
                    name: "input_bytes",
                    maximum: crate::binary::transform::MAX_INPUT as u64,
                    unit: "bytes",
                    scope: crate::operations::metadata::Scope::Cli,
                },
                crate::operations::metadata::ResourceBound {
                    name: "key_bytes",
                    maximum: crate::binary::transform::MAX_INPUT as u64,
                    unit: "bytes",
                    scope: crate::operations::metadata::Scope::Cli,
                },
                crate::operations::metadata::ResourceBound {
                    name: "pipeline_operand_parameter_bytes",
                    maximum: 65536,
                    unit: "bytes",
                    scope: crate::operations::metadata::Scope::Pipeline,
                },
            ],
            resource_notes: "No enforced direct Rust/native-Python input or allocation/work budget; caller-sized processing. CLI operands are capped at 16 MiB. The key must be nonempty and repeats over the input; this is not exact-length XOR.",
            rust_api: "cube_core::binary::xor",
            python_api: "xor",
            references: &["https://doc.rust-lang.org/std/primitive.u64.html"],
        }),
    }
}];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_answer_and_key_lengths() -> Result<(), Error> {
        assert_eq!(xor(b"Hello", b"key")?, [35, 0, 21, 7, 10]);
        assert_eq!(xor(&[0, 255, 128], &[255])?, [255, 0, 127]);
        assert_eq!(xor(&[1, 2], &[1, 2, 3])?, [0, 0]);
        assert!(xor(b"", b"key")?.is_empty());
        Ok(())
    }

    #[test]
    fn rejects_empty_key_even_with_empty_input() {
        assert_eq!(xor(b"Hello", b""), Err(Error::EmptyKey));
        assert_eq!(xor(b"", b""), Err(Error::EmptyKey));
    }
}

pub mod packing;
