//! Explicit Unicode text operations and shared limits for new text interfaces.
pub mod anagram;
pub mod classes;
pub mod combinations;
pub mod grid;
pub mod keyboard;
pub mod lexical;
pub(crate) mod mapping;
pub mod multiset;
pub mod positions;
pub mod selection;
pub mod statistics;
pub mod transform;
pub mod traversal;
pub mod unicode;
pub mod values;
pub mod word;
pub mod word_ladder;
use crate::errors::{ErrorCode, OperationError};
/// Maximum UTF-8 input bytes for v0.5 text/symbol/cipher operations (1 MiB).
pub const MAX_INPUT: usize = 1024 * 1024;
/// Maximum UTF-8 output bytes, enforced while producing expanding results (8 MiB).
pub const MAX_OUTPUT: usize = 8 * 1024 * 1024;
pub(crate) fn check_input(input: &str, operation: &'static str) -> Result<(), OperationError> {
    if input.len() > MAX_INPUT {
        Err(
            OperationError::new(ErrorCode::ResourceLimit, "text input exceeds 1 MiB")
                .operation(operation),
        )
    } else {
        Ok(())
    }
}

use unicode_normalization::UnicodeNormalization;
/// Unicode data used by normalization. Changes require an intentional dependency
/// update and compatibility review; other text operations do not normalize.
pub const UNICODE_VERSION: (u8, u8, u8) = unicode_normalization::UNICODE_VERSION;
/// Maximum explicit positional indexes; repeated indexes are permitted.
pub const MAX_INDICES: usize = 65536;
pub(crate) fn collect_bounded(
    chars: impl Iterator<Item = char>,
    operation: &'static str,
) -> Result<String, OperationError> {
    let mut output = String::new();
    for c in chars {
        if output.len() + c.len_utf8() > MAX_OUTPUT {
            return Err(
                OperationError::new(ErrorCode::ResourceLimit, "text output exceeds 8 MiB")
                    .operation(operation),
            );
        }
        output.push(c);
    }
    Ok(output)
}
/// Apply explicit NFC/NFD/NFKC/NFKD using the published Unicode data version.
/// UAX #15: <https://www.unicode.org/reports/tr15/>. Canonical forms compose or
/// decompose; compatibility forms additionally lose distinctions. No form is
/// generally reversible. Input <=1 MiB, output <=8 MiB; unknown forms are errors.
pub fn normalize(input: &str, form: &str) -> Result<String, OperationError> {
    const OP: &str = "text.normalize";
    check_input(input, OP)?;
    match form {
        "NFC" => collect_bounded(input.nfc(), OP),
        "NFD" => collect_bounded(input.nfd(), OP),
        "NFKC" => collect_bounded(input.nfkc(), OP),
        "NFKD" => collect_bounded(input.nfkd(), OP),
        _ => Err(OperationError::new(
            ErrorCode::InvalidParameter,
            "form must be NFC, NFD, NFKC or NFKD",
        )
        .operation(OP)
        .parameter("form")),
    }
}
/// Reverse Unicode scalar values without normalization or trimming. This can
/// split grapheme clusters, reverse combining sequences and change CRLF to LFCR.
/// Applying twice restores the exact input; input/output <=1 MiB.
pub fn reverse(input: &str) -> Result<String, OperationError> {
    check_input(input, "text.reverse")?;
    Ok(input.chars().rev().collect())
}
/// Select exactly one mode: an ordered list of zero-based Unicode-scalar indexes
/// (duplicates kept), or an explicit positive stride with optional start/stop.
/// Default start=0, stop=scalar length; stop is exclusive. Empty ranges are valid.
/// No grapheme segmentation or normalization. Input <=1 MiB; list <=65536 indexes.
/// All indexes/ranges validate before output; negative values cannot enter usize.
pub fn extract(
    input: &str,
    indices: Option<&[usize]>,
    start: Option<usize>,
    stride: Option<usize>,
    stop: Option<usize>,
) -> Result<String, OperationError> {
    const OP: &str = "text.extract";
    check_input(input, OP)?;
    let invalid = |name, message| {
        OperationError::new(ErrorCode::InvalidParameter, message)
            .operation(OP)
            .parameter(name)
    };
    if let Some(indices) = indices {
        if start.is_some() || stride.is_some() || stop.is_some() {
            return Err(invalid(
                "indices",
                "indices cannot be combined with start/stride/stop",
            ));
        }
        if indices.len() > MAX_INDICES {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "selection exceeds 65536 indexes",
            )
            .operation(OP)
            .parameter("indices"));
        }
        let mut chars = Vec::with_capacity(input.chars().count());
        chars.extend(input.chars());
        for (position, index) in indices.iter().enumerate() {
            if *index >= chars.len() {
                return Err(invalid(
                    "indices",
                    &format!("scalar index at list position {position} is out of range"),
                ));
            }
        }
        collect_bounded(indices.iter().map(|i| chars[*i]), OP)
    } else {
        let stride = stride
            .ok_or_else(|| invalid("stride", "provide indices or an explicit positive stride"))?;
        if stride == 0 {
            return Err(invalid("stride", "stride must be positive"));
        }
        let length = input.chars().count();
        let start = start.unwrap_or(0);
        let stop = stop.unwrap_or(length);
        if start > length {
            return Err(invalid("start", "start exceeds scalar length"));
        }
        if stop > length {
            return Err(invalid("stop", "stop exceeds scalar length"));
        }
        collect_bounded(input.chars().take(stop).skip(start).step_by(stride), OP)
    }
}

/// Explicitly encode text as UTF-8 bytes, preserving NUL/newlines and every scalar.
/// Input/output <=1 MiB. This does not normalize or add a newline.
pub fn to_utf8(input: &str) -> Result<Vec<u8>, OperationError> {
    check_input(input, "text.utf8.encode")?;
    Ok(input.as_bytes().to_vec())
}
/// Decode exact bytes as UTF-8 without replacement or normalization; <=1 MiB.
pub fn from_utf8(input: &[u8]) -> Result<String, OperationError> {
    if input.len() > MAX_INPUT {
        return Err(
            OperationError::new(ErrorCode::ResourceLimit, "UTF-8 input exceeds 1 MiB")
                .operation("text.utf8.decode"),
        );
    }
    std::str::from_utf8(input).map(str::to_owned).map_err(|e| {
        OperationError::new(
            ErrorCode::InvalidInput,
            "bytes are not valid UTF-8; use a byte operation or choose an explicit encoding",
        )
        .operation("text.utf8.decode")
        .offset(e.valid_up_to() as u64)
        .caused_by(e)
    })
}

use crate::operations::{Descriptor, Parameter, ParameterType};
/// Static metadata is adjacent to the text implementation.
const PROFILED: [crate::operations::metadata::Entry; 5] = [
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "text.utf8.encode",
            category: "text",
            description: "Explicit text-to-UTF-8 bytes, no normalization/newline; <=1 MiB.",
            input_type: "text",
            output_type: "bytes",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "text to-bytes",
            example: "cube text to-bytes --text Hello --output hex",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "UTF-8",
                family: "character_encodings",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::Encoding,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Exact Unicode scalar text <-> valid UTF-8 bytes, including NUL/newlines; reject malformed bytes without replacement or normalization.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. Existing UTF-8 conversion only; ASCII range validation, UTF-16/32 byte order and BOM policies are separate pending profiles.",
                rust_api: "cube_core::text::to_utf8",
                python_api: "utf8_encode",
                references: &[
                    "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
                ],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "text.utf8.decode",
            category: "text",
            description: "Explicit UTF-8 decoding, reject invalid bytes without replacement; <=1 MiB.",
            input_type: "bytes",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "text from-bytes",
            example: "printf Hello | cube text from-bytes",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "UTF-8",
                family: "character_encodings",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::Encoding,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Exact Unicode scalar text <-> valid UTF-8 bytes, including NUL/newlines; reject malformed bytes without replacement or normalization.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "input_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. Existing UTF-8 conversion only; ASCII range validation, UTF-16/32 byte order and BOM policies are separate pending profiles.",
                rust_api: "cube_core::text::from_utf8",
                python_api: "utf8_decode",
                references: &[
                    "https://www.unicode.org/versions/Unicode17.0.0/core-spec/chapter-3/",
                ],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "text.normalize",
            category: "text",
            description: "Explicit UAX #15 normalization; Unicode data version exposed by UNICODE_VERSION; <=1 MiB input/8 MiB output.",
            input_type: "text",
            output_type: "text",
            parameters: &[Parameter {
                name: "form",
                description: "Canonical compose/decompose (NFC/NFD) or compatibility forms (NFKC/NFKD); no implicit normalization.",
                required: true,
                default: None,
                constraints: ParameterType::Text {
                    choices: &["NFC", "NFD", "NFKC", "NFKD"],
                },
            }],
            deterministic: true,
            reversible: Some(false),
            command: "text normalize",
            example: "cube text normalize --form NFC --text café",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: None,
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "text.reverse",
            category: "text",
            description: "Reverse Unicode scalar values, including combining marks/CRLF; not grapheme or byte reversal; <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command: "text reverse",
            example: "cube text reverse --text 'Cube 🧊'",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: None,
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "text.extract",
            category: "text",
            description: "Select Unicode scalar positions (not UTF-8 bytes); preserve list order/duplicates, no normalization; <=1 MiB.",
            input_type: "text",
            output_type: "text",
            parameters: &[
                Parameter {
                    name: "indices",
                    description: "Zero-based list, <=65536 items; mutually exclusive with start/stride/stop. An empty list is valid.",
                    required: false,
                    default: None,
                    constraints: ParameterType::UnsignedList {
                        min: 0,
                        max: usize::MAX as u64,
                        min_length: 0,
                        max_length: MAX_INDICES,
                        unique: false,
                    },
                },
                Parameter {
                    name: "start",
                    description: "Start scalar index for stride mode; default 0 only in stride mode.",
                    required: false,
                    default: None,
                    constraints: ParameterType::Unsigned {
                        min: 0,
                        max: usize::MAX as u64,
                    },
                },
                Parameter {
                    name: "stride",
                    description: "Explicit positive stride, required unless indices are provided.",
                    required: false,
                    default: None,
                    constraints: ParameterType::Unsigned {
                        min: 1,
                        max: usize::MAX as u64,
                    },
                },
                Parameter {
                    name: "stop",
                    description: "Exclusive scalar stop in stride mode; omitted means scalar length.",
                    required: false,
                    default: None,
                    constraints: ParameterType::Unsigned {
                        min: 0,
                        max: usize::MAX as u64,
                    },
                },
            ],
            deterministic: true,
            reversible: Some(false),
            command: "text extract",
            example: "cube text extract --start 1 --stride 2 --text abcdef",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: None,
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
