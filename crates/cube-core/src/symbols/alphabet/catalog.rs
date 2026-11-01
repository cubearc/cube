use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use serde_json::Value;
const UNITS: &[c::Port] = &[c::Port {
    name: "units",
    semantic_type: "alphabet.units",
    schema: c::schema::<h::Tokens>,
    required: true,
}];
const TOKENS: &[c::Port] = &[c::Port {
    name: "tokens",
    semantic_type: "alphabet.tokens",
    schema: c::schema::<h::Tokens>,
    required: true,
}];
const PROVENANCE: c::Port = c::Port {
    name: "provenance",
    semantic_type: "alphabet.provenance",
    schema: c::schema::<Provenance>,
    required: true,
};
const ENCODE_OUTPUTS: &[c::Port] = &[TOKENS[0], PROVENANCE];
const DECODE_OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "readings",
        semantic_type: "alphabet.readings",
        schema: || c::field::<Decoded>("readings"),
        required: true,
    },
    TOKENS[0],
    PROVENANCE,
];
const PARAMS: &[o::Parameter] = &[o::Parameter {
    name: "parameters",
    description: "Required dated source profile. Units are exact and presegmented; no inferred font, case, language, digraph or reading order.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text { choices: &[] },
}];
#[derive(Clone, Copy)]
struct Row {
    profile: m::Entry,
    contract: c::Entry,
}
const fn row(
    id: &'static str,
    python: &'static str,
    decode: bool,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> Row {
    let descriptor = o::Descriptor {
        id,
        category: "symbols",
        description: if decode {
            "Decode a source-qualified symbolic alphabet, retaining all readings, exact tokens and positions."
        } else {
            "Encode exact presegmented units with an explicit source-qualified symbolic alphabet."
        },
        input_type: if decode {
            "alphabet.tokens"
        } else {
            "alphabet.units"
        },
        output_type: if decode {
            "alphabet.readings"
        } else {
            "alphabet.tokens"
        },
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: if decode {
            "decode alphabet"
        } else {
            "encode alphabet"
        },
        example: "cube encode alphabet --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: if decode { TOKENS } else { UNITS },
            parameters: c::schema::<Parameters>,
            outputs: if decode {
                DECODE_OUTPUTS
            } else {
                ENCODE_OUTPUTS
            },
            limits: c::Limits {
                max_input_bytes: Some(1048576),
                max_output_bytes: Some(MAX_REPORT as u64),
                max_items: Some(65536),
                max_work: None,
                max_memory_bytes: None,
            },
            fixture,
        },
        profile: m::Entry {
            descriptor,
            metadata: Some(m::Metadata {
                schema_version: 1,
                canonical_name: if decode {
                    "Decode a symbolic alphabet"
                } else {
                    "Encode a symbolic alphabet"
                },
                family: "symbolic_alphabet",
                aliases: &[
                    m::Alias {
                        name: "Aurebesh",
                        usage: "profile=aurebesh_ucsur_2016_v1",
                    },
                    m::Alias {
                        name: "Standard Galactic Alphabet",
                        usage: "profile=standard_galactic_ucsur_2019_v1",
                    },
                    m::Alias {
                        name: "SGA",
                        usage: "profile=standard_galactic_ucsur_2019_v1",
                    },
                ],
                security: m::Security {
                    classification: m::Classification::PuzzleCode,
                    limitations: "Fixed dated alphabet profiles, without confidentiality, OCR, font recognition, language translation or a guessed inverse.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: descriptor.description,
                    alphabet: "Exact source-listed uppercase units, digraphs and punctuation. UCSUR PUA assignments are private-use proposals, not Unicode-standard script encodings. Ordinary interword spaces are preserved. No normalization.",
                    key: "Fixed independently transcribed dated source tables, with source and effective-definition hashes. No fonts, source artwork or language translations bundled.",
                    key_advancement: "Canonical token per exact unit; decoding retains all listed alternatives in source order. Explicit segmentation distinguishes CH from C,H. Repeated tokens retain their positions.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Input <=65536 tokens/1MiB compact JSON; each token <=256 UTF-8 bytes; fixed definition <=64KiB; complete report <=1MiB. Unknown, joined or differently cased forms reject. Reserved/private-use holes and unsupported digits reject; no visual or text segmentation is inferred.",
                rust_api: "cube_core::symbols::alphabet::{encode,decode}",
                python_api: python,
                references: &[
                    "https://www.kreativekorp.com/ucsur/charts/aurebesh.htm",
                    "https://www.kreativekorp.com/ucsur/charts/sga.html",
                ],
            }),
        },
    }
}
const ROWS: [Row; 2] = [
    row(
        "symbol.alphabet.encode",
        "alphabet_encode",
        false,
        super::fixtures::encode,
    ),
    row(
        "symbol.alphabet.decode",
        "alphabet_decode",
        true,
        super::fixtures::decode,
    ),
];
const PROFILED: [m::Entry; 2] = [ROWS[0].profile, ROWS[1].profile];
const CONTRACTS: [c::Entry; 2] = [ROWS[0].contract, ROWS[1].contract];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn native(op: &'static str, i: &Value, p: &Value) -> e::Result<Value> {
    let run = || {
        let p: Parameters = crate::ciphers::poly::parse_parameters(&p.to_string())?;
        let field = if op.ends_with(".decode") {
            "tokens"
        } else {
            "units"
        };
        if i.as_object().is_none_or(|fields| fields.len() != 1) {
            return Err(crate::errors::OperationError::new(
                crate::errors::ErrorCode::InvalidInput,
                "one exact named unit-token input required",
            ));
        }
        let input =
            serde_json::from_value::<Vec<String>>(i.get(field).cloned().unwrap_or(Value::Null))
                .map_err(|_| {
                    crate::errors::OperationError::new(
                        crate::errors::ErrorCode::InvalidInput,
                        "expected exact presegmented unit strings",
                    )
                })?;
        if op.ends_with(".decode") {
            serde_json::to_value(decode(&input, &p)?)
        } else {
            serde_json::to_value(encode(&input, &p)?)
        }
        .map_err(|_| {
            crate::errors::OperationError::new(
                crate::errors::ErrorCode::InvalidInput,
                "typed report serialization failed",
            )
        })
    };
    run().map_err(|e: OperationError| e::Failure(e.code.as_str()))
}
const EXECUTORS: &[e::Entry] = &[
    e::Entry::pure(
        "symbol.alphabet.encode",
        &[e::Representation::Json],
        &[e::Representation::Json, e::Representation::Json],
        |i, p, _| native("symbol.alphabet.encode", i, p),
    ),
    e::Entry::pure(
        "symbol.alphabet.decode",
        &[e::Representation::Json],
        &[
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Json,
        ],
        |i, p, _| native("symbol.alphabet.decode", i, p),
    ),
];
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
