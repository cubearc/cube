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
pub const OPERATIONS: &[crate::operations::Descriptor] = &[crate::operations::Descriptor {
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
}];

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
