//! Bacon's five-bit alphabets, with explicit notation and lossy canonicalization.
use crate::{
    errors::{ErrorCode, OperationError},
    text::check_input,
};
pub const CLASSICAL24: &str = "ABCDEFGHIKLMNOPQRSTUWXYZ";
pub const EXTENDED26: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
fn config(
    variant: &str,
    notation: &str,
    op: &'static str,
) -> Result<(&'static str, [u8; 2]), OperationError> {
    let alphabet = match variant {
        "classical24" => CLASSICAL24,
        "extended26" => EXTENDED26,
        _ => {
            return Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "variant must be classical24 or extended26",
            )
            .operation(op)
            .parameter("variant"));
        }
    };
    let symbols = match notation {
        "ab" => *b"AB",
        "binary" => *b"01",
        "binary_inverted" => *b"10",
        _ => {
            return Err(OperationError::new(
                ErrorCode::InvalidParameter,
                "notation must be ab, binary or binary_inverted",
            )
            .operation(op)
            .parameter("notation"));
        }
    };
    Ok((alphabet, symbols))
}
fn invalid(op: &'static str, offset: usize) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, "expected supported letters/spaces or canonical five-symbol groups separated by one space; / separates nonempty words").operation(op).offset(offset as u64)
}
/// Encode ASCII letters with a required 24/26-letter alphabet. Classical merges
/// J into I and V into U. ASCII space runs collapse and edge spaces disappear;
/// all other unsupported scalars fail at their byte offset. Notation is `ab`,
/// `binary` (A=0) or `binary_inverted` (A=1). <=1 MiB input, <=6 MiB output.
pub fn encode(input: &str, variant: &str, notation: &str) -> Result<String, OperationError> {
    const OP: &str = "cipher.bacon.encode";
    check_input(input, OP)?;
    let (alphabet, symbols) = config(variant, notation, OP)?;
    let mut output = String::new();
    let mut boundary = false;
    for (offset, mut c) in input.char_indices() {
        if c == ' ' {
            boundary = !output.is_empty();
            continue;
        }
        c = c.to_ascii_uppercase();
        if variant == "classical24" {
            c = match c {
                'J' => 'I',
                'V' => 'U',
                _ => c,
            };
        }
        let index = alphabet
            .chars()
            .position(|v| v == c)
            .ok_or_else(|| invalid(OP, offset))?;
        if !output.is_empty() {
            output.push_str(if boundary { " / " } else { " " });
        }
        for bit in (0..5).rev() {
            output.push(symbols[(index >> bit) & 1] as char);
        }
        boundary = false;
    }
    Ok(output)
}
/// Decode strict canonical grammar to uppercase letters and single word spaces.
/// Reject incomplete/unused codes and malformed boundaries with byte offsets.
/// Empty text is valid; <=1 MiB input/output. No automatic notation detection.
pub fn decode(input: &str, variant: &str, notation: &str) -> Result<String, OperationError> {
    const OP: &str = "cipher.bacon.decode";
    check_input(input, OP)?;
    let (alphabet, symbols) = config(variant, notation, OP)?;
    if input.is_empty() {
        return Ok(String::new());
    }
    let mut output = String::new();
    let mut offset = 0;
    let mut letter = false;
    for group in input.split(' ') {
        if group == "/" {
            if !letter {
                return Err(invalid(OP, offset));
            }
            output.push(' ');
            letter = false;
        } else {
            if group.len() != 5 {
                return Err(invalid(OP, offset));
            }
            let mut value = 0usize;
            for c in group.bytes() {
                value = value * 2
                    + if c == symbols[0] {
                        0
                    } else if c == symbols[1] {
                        1
                    } else {
                        return Err(invalid(OP, offset));
                    };
            }
            output.push(
                *alphabet
                    .as_bytes()
                    .get(value)
                    .ok_or_else(|| invalid(OP, offset))? as char,
            );
            letter = true;
        }
        offset += group.len() + 1;
    }
    if !letter {
        return Err(invalid(OP, input.len() - 1));
    }
    Ok(output)
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "variant",
        description: "Required alphabet; classical24 merges I/J and U/V.",
        required: true,
        default: None,
        constraints: ParameterType::Text {
            choices: &["classical24", "extended26"],
        },
    },
    Parameter {
        name: "notation",
        description: "Explicit AB or binary orientation; no detection.",
        required: false,
        default: Some(ParameterValue::Text("ab")),
        constraints: ParameterType::Text {
            choices: &["ab", "binary", "binary_inverted"],
        },
    },
];
const PROFILED: [crate::operations::metadata::Entry; 2] = [
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.bacon.encode",
            category: "ciphers",
            description: "Five-symbol Bacon alphabet; ASCII case/word-space canonicalization, <=1 MiB input.",
            input_type: "text",
            output_type: "text",
            parameters: PARAMETERS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher bacon",
            example: "cube cipher bacon --variant extended26 --text A",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Baconian",
                family: "classical_code",
                aliases: &[crate::operations::metadata::Alias {
                    name: "Bacon's cipher",
                    usage: "Five-symbol code groups, not automatic steganographic cover-text generation.",
                }],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Required classical24 (I/J and U/V merged) or extended26; uppercase canonical output. ASCII-space runs collapse; unsupported scalars fail.",
                    key: "No cryptographic key. Required alphabet variant and ab/binary/binary_inverted notation.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: 6 * crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::ciphers::bacon::encode",
                python_api: "bacon_encode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.bacon.decode",
            category: "ciphers",
            description: "Five-symbol Bacon alphabet; ASCII case/word-space canonicalization, <=1 MiB input.",
            input_type: "text",
            output_type: "text",
            parameters: PARAMETERS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher bacon",
            example: "cube cipher bacon --variant extended26 --decode --text AAAAA",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Baconian",
                family: "classical_code",
                aliases: &[crate::operations::metadata::Alias {
                    name: "Bacon's cipher",
                    usage: "Five-symbol code groups, not automatic steganographic cover-text generation.",
                }],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Required classical24 (I/J and U/V merged) or extended26; uppercase canonical output. ASCII-space runs collapse; unsupported scalars fail.",
                    key: "No cryptographic key. Required alphabet variant and ab/binary/binary_inverted notation.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. ",
                rust_api: "cube_core::ciphers::bacon::decode",
                python_api: "bacon_decode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
