//! Explicit International Morse text and Unicode Braille cell encodings.
//! No audio interpretation, natural-language transcription or hidden normalization.
pub mod a1z26;
pub mod clock;
pub mod keypad;
pub mod semaphore;
use crate::{
    errors::{ErrorCode, OperationError},
    text::{MAX_INPUT, MAX_OUTPUT, check_input},
};
/// International Morse letters/digits from ITU-R M.1677-1:
/// <https://www.itu.int/rec/R-REC-M.1677-1-200910-I/>. No punctuation extensions.
pub const MORSE_ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const CODES: [&str; 36] = [
    ".-", "-...", "-.-.", "-..", ".", "..-.", "--.", "....", "..", ".---", "-.-", ".-..", "--",
    "-.", "---", ".--.", "--.-", ".-.", "...", "-", "..-", "...-", ".--", "-..-", "-.--", "--..",
    "-----", ".----", "..---", "...--", "....-", ".....", "-....", "--...", "---..", "----.",
];
fn invalid(message: &str, op: &'static str, offset: usize) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
        .operation(op)
        .offset(offset as u64)
}
fn append(output: &mut String, value: &str, op: &'static str) -> Result<(), OperationError> {
    if output.len() + value.len() > MAX_OUTPUT {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "symbol output exceeds 8 MiB")
                .operation(op),
        );
    }
    output.push_str(value);
    Ok(())
}
/// Encode A-Z/a-z and 0-9 as International Morse. Single spaces separate symbols,
/// " / " separates words. Runs of ASCII spaces collapse; edge spaces disappear.
/// Reject every unsupported character (including tabs/newlines) at its UTF-8 byte
/// offset. Decode normalizes case/word spacing rather than recovering it exactly.
/// Input <=1 MiB; output <=8 MiB.
pub fn morse_encode(input: &str) -> Result<String, OperationError> {
    const OP: &str = "symbol.morse.encode";
    check_input(input, OP)?;
    let mut output = String::new();
    let mut word_boundary = false;
    for (offset, c) in input.char_indices() {
        if c == ' ' {
            word_boundary = !output.is_empty();
            continue;
        }
        let code = if c.is_ascii_alphabetic() {
            CODES[(c as u8).to_ascii_uppercase() as usize - b'A' as usize]
        } else if c.is_ascii_digit() {
            CODES[26 + c as usize - '0' as usize]
        } else {
            return Err(invalid(
                "unsupported Morse input; only ASCII letters, digits and spaces are accepted",
                OP,
                offset,
            ));
        };
        if !output.is_empty() {
            append(&mut output, if word_boundary { " / " } else { " " }, OP)?;
        }
        append(&mut output, code, OP)?;
        word_boundary = false;
    }
    Ok(output)
}
/// Decode only canonical Morse grammar: '.'/'-' tokens, exactly one ASCII space
/// between tokens, slash tokens between nonempty words, no edge spaces. Return
/// uppercase ASCII text. Empty input is valid; malformed spacing/unknown tokens
/// fail with byte offsets. Input <=1 MiB; output <=8 MiB.
pub fn morse_decode(input: &str) -> Result<String, OperationError> {
    const OP: &str = "symbol.morse.decode";
    check_input(input, OP)?;
    if input.is_empty() {
        return Ok(String::new());
    }
    let mut output = String::new();
    let mut offset = 0;
    let mut symbol = false;
    for token in input.split(' ') {
        if token.is_empty() {
            return Err(invalid(
                "Morse tokens require exactly one ASCII space and no edge spaces",
                OP,
                offset,
            ));
        }
        if token == "/" {
            if !symbol {
                return Err(invalid(
                    "Morse word boundaries require nonempty words",
                    OP,
                    offset,
                ));
            }
            output.push(' ');
            symbol = false;
        } else {
            let index = CODES
                .iter()
                .position(|code| *code == token)
                .ok_or_else(|| {
                    invalid(
                        "unknown Morse token; expected a supported letter or digit",
                        OP,
                        offset,
                    )
                })?;
            output.push(char::from(MORSE_ALPHABET.as_bytes()[index]));
            symbol = true;
        }
        offset += token.len() + 1;
    }
    if !symbol {
        return Err(invalid(
            "Morse must not end with a word boundary",
            OP,
            input.len() - 1,
        ));
    }
    Ok(output)
}
/// Encode raw dot masks as U+2800..U+28FF; bit n-1 represents dot n, n=1..8.
/// Standard layout: left column 1,2,3,7; right 4,5,6,8. Blank 0 maps to U+2800.
/// This encodes cells, not language or contractions. Input <=1 MiB, output <=3 MiB.
pub fn braille_encode(masks: &[u8]) -> Result<String, OperationError> {
    const OP: &str = "symbol.braille.encode";
    if masks.len() > MAX_INPUT {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "Braille masks exceed 1 MiB")
                .operation(OP),
        );
    }
    Ok(masks
        .iter()
        .map(|v| char::from_u32(0x2800 + u32::from(*v)).expect("Braille range"))
        .collect())
}
/// Decode only Unicode Braille cells into exact bytes. Unsupported characters
/// (including spaces) fail at the original UTF-8 byte offset; input <=1 MiB.
/// <https://www.unicode.org/charts/PDF/U2800.pdf> defines the dot assignments.
pub fn braille_decode(input: &str) -> Result<Vec<u8>, OperationError> {
    const OP: &str = "symbol.braille.decode";
    check_input(input, OP)?;
    input
        .char_indices()
        .map(|(offset, c)| {
            if ('\u{2800}'..='\u{28ff}').contains(&c) {
                Ok((c as u32 - 0x2800) as u8)
            } else {
                Err(invalid(
                    "expected a Unicode Braille cell U+2800..U+28FF",
                    OP,
                    offset,
                ))
            }
        })
        .collect()
}
use crate::operations::Descriptor;
/// Metadata next to the cell/grammar contracts.
const PROFILED: [crate::operations::metadata::Entry; 4] = [
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "symbol.morse.encode",
            category: "symbols",
            description: "International Morse A-Z/0-9; collapse ASCII spaces, normalize decoded case; <=1 MiB input/8 MiB output.",
            input_type: "text",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: "encode morse",
            example: "cube encode morse --text 'SOS 2'",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "International Morse text",
                family: "puzzle_representations",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::PuzzleCode,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "International Morse A-Z/a-z/0-9 only; uppercase decode, canonical symbol/word separators; ASCII-space runs collapse.",
                    key: "No key.",
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
                        name: "output_bytes",
                        maximum: crate::text::MAX_OUTPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. These are the documented supported subsets, not linguistic decipherment, predictive text or a general secure cipher.",
                rust_api: "cube_core::symbols::morse_encode",
                python_api: "morse_encode",
                references: &["https://www.itu.int/rec/R-REC-M.1677-1-200910-I/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "symbol.morse.decode",
            category: "symbols",
            description: "Canonical dot/dash tokens with one space; slash separates words; uppercase A-Z/0-9, <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(false),
            command: "decode morse",
            example: "cube decode morse --text '... --- ... / ..---'",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "International Morse text",
                family: "puzzle_representations",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::PuzzleCode,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "International Morse A-Z/a-z/0-9 only; uppercase decode, canonical symbol/word separators; ASCII-space runs collapse.",
                    key: "No key.",
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
                        name: "output_bytes",
                        maximum: crate::text::MAX_OUTPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. These are the documented supported subsets, not linguistic decipherment, predictive text or a general secure cipher.",
                rust_api: "cube_core::symbols::morse_decode",
                python_api: "morse_decode",
                references: &["https://www.itu.int/rec/R-REC-M.1677-1-200910-I/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "symbol.braille.encode",
            category: "symbols",
            description: "Encode bytes as Unicode Braille dot masks, bit n-1=dot n; no language transcription; <=1 MiB.",
            input_type: "bytes",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "encode braille",
            example: "printf '\\001' | cube encode braille",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Unicode Braille dot masks",
                family: "puzzle_representations",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::PuzzleCode,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Exact 8-dot masks <-> U+2800..U+28FF; bit n-1 is dot n. No language transcription or contractions.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_mask_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_bytes",
                        maximum: (3 * crate::text::MAX_INPUT) as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. These are the documented supported subsets, not linguistic decipherment, predictive text or a general secure cipher.",
                rust_api: "cube_core::symbols::braille_encode",
                python_api: "braille_encode",
                references: &["https://www.unicode.org/charts/PDF/U2800.pdf"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "symbol.braille.decode",
            category: "symbols",
            description: "Decode U+2800..U+28FF cells to exact mask bytes; reject other characters; <=1 MiB.",
            input_type: "text",
            output_type: "bytes",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "decode braille",
            example: "cube decode braille --text '⠁⣿' --output hex",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Unicode Braille dot masks",
                family: "puzzle_representations",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::PuzzleCode,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Exact 8-dot masks <-> U+2800..U+28FF; bit n-1 is dot n. No language transcription or contractions.",
                    key: "No key.",
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
                        name: "output_bytes",
                        maximum: (crate::text::MAX_INPUT / 3) as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. These are the documented supported subsets, not linguistic decipherment, predictive text or a general secure cipher.",
                rust_api: "cube_core::symbols::braille_decode",
                python_api: "braille_decode",
                references: &["https://www.unicode.org/charts/PDF/U2800.pdf"],
            }),
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
