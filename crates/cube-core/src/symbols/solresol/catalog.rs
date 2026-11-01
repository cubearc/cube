use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use serde_json::Value;
const TERMS: &[c::Port] = &[c::Port {
    name: "terms",
    semantic_type: "solresol.terms",
    schema: c::schema::<h::Tokens>,
    required: true,
}];
const TOKENS: &[c::Port] = &[c::Port {
    name: "tokens",
    semantic_type: "solresol.forms",
    schema: c::schema::<h::Tokens>,
    required: true,
}];
const PROVENANCE: c::Port = c::Port {
    name: "provenance",
    semantic_type: "solresol.provenance",
    schema: c::schema::<Provenance>,
    required: true,
};
const ENCODE_OUTPUTS: &[c::Port] = &[TOKENS[0], PROVENANCE];
const DECODE_OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "readings",
        semantic_type: "solresol.readings",
        schema: || c::field::<Decoded>("readings"),
        required: true,
    },
    TOKENS[0],
    PROVENANCE,
];
const PARAMS: &[o::Parameter] = &[o::Parameter {
    name: "parameters",
    description: "Required gajewski_1902_monosyllables_v1 finite lexicon profile; no other grammar or vocabulary is inferred.",
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
            "Decode the Gajewski monosyllable lexicon, retaining all French glosses, original forms and word positions."
        } else {
            "Encode exact French gloss tokens with the complete Gajewski seven-monosyllable lexicon."
        },
        input_type: if decode {
            "solresol.forms"
        } else {
            "solresol.terms"
        },
        output_type: if decode {
            "solresol.readings"
        } else {
            "solresol.forms"
        },
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: if decode {
            "decode solresol"
        } else {
            "encode solresol"
        },
        example: "cube encode solresol --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: if decode { TOKENS } else { TERMS },
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
                    "Decode Solresol (Gajewski monosyllables)"
                } else {
                    "Encode Solresol (Gajewski monosyllables)"
                },
                family: "solresol_monosyllables",
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::PuzzleCode,
                    limitations: "Finite historical lexicon only; no general language translation, confidentiality, grammar restoration, musical transcription or guessed inverse.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: descriptor.description,
                    alphabet: "Exact lower-case relative syllables do/ré/mi/fa/sol/la/si and their seven published abbreviations; exact French gloss tokens, including accents and the conditional qualification. No normalization.",
                    key: "Fixed independently transcribed Gajewski1902 seven-entry table on facsimile PDF page4; fourteen forms and seventeen French glosses. No scan, font, score or modern English translation bundled.",
                    key_advancement: "Canonical full form per exact gloss; decode every source-listed gloss in table order. Presegmented words, including repetitions and abbreviations, retain their positions.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Input <=65536 tokens/1MiB compact JSON; each token <=256 UTF-8 bytes; fixed definition <=64KiB; complete report <=1MiB. Unknown joined, inflected or marked forms reject instead of erasing distinctions. No pitch, octave, duration, OCR or audio inference.",
                rust_api: "cube_core::symbols::solresol::{encode,decode}",
                python_api: python,
                references: &[
                    "https://upload.wikimedia.org/wikipedia/commons/b/b9/Grammaire_du_Solresol.pdf",
                ],
            }),
        },
    }
}
const ROWS: [Row; 2] = [
    row(
        "symbol.solresol.encode",
        "solresol_encode",
        false,
        super::fixtures::encode,
    ),
    row(
        "symbol.solresol.decode",
        "solresol_decode",
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
            "terms"
        };
        if i.as_object().is_none_or(|fields| fields.len() != 1) {
            return Err(crate::errors::OperationError::new(
                crate::errors::ErrorCode::InvalidInput,
                "one exact named word-token input required",
            ));
        }
        let input =
            serde_json::from_value::<Vec<String>>(i.get(field).cloned().unwrap_or(Value::Null))
                .map_err(|_| {
                    crate::errors::OperationError::new(
                        crate::errors::ErrorCode::InvalidInput,
                        "expected exact presegmented word strings",
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
        "symbol.solresol.encode",
        &[e::Representation::Json],
        &[e::Representation::Json, e::Representation::Json],
        |i, p, _| native("symbol.solresol.encode", i, p),
    ),
    e::Entry::pure(
        "symbol.solresol.decode",
        &[e::Representation::Json],
        &[
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Json,
        ],
        |i, p, _| native("symbol.solresol.decode", i, p),
    ),
];
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
