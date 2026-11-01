use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use serde_json::Value;
const INPUTS: &[c::Port] = &[c::Port {
    name: "text",
    semantic_type: "text",
    schema: c::schema::<super::super::params::Text>,
    required: true,
}];
const OUTPUTS: [c::Port; 4] = [
    c::Port {
        name: "text",
        semantic_type: "text.optional",
        schema: || c::field::<Report>("text"),
        required: true,
    },
    c::Port {
        name: "ambiguous",
        semantic_type: "boolean",
        schema: || c::field::<Report>("ambiguous"),
        required: true,
    },
    c::Port {
        name: "records",
        semantic_type: "cipher.phrase_readings",
        schema: || c::field::<Report>("records"),
        required: true,
    },
    c::Port {
        name: "definition_sha256",
        semantic_type: "text",
        schema: || c::field::<Report>("definition_sha256"),
        required: true,
    },
];
const ENCODE_OUTPUTS: [c::Port; 4] = [
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<Report<String>>("text"),
        required: true,
    },
    OUTPUTS[1],
    OUTPUTS[2],
    OUTPUTS[3],
];
const PARAMS: &[o::Parameter] = &[o::Parameter {
    name: "parameters",
    description: "Required complete 26-letter uppercase phrase. Repeated letters preserve inverse ambiguity; see typed schema.",
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
        category: "ciphers",
        description: if decode {
            "Decode the ACA Key Phrase alphabet into all per-letter readings without guessing plaintext."
        } else {
            "Encode the ACA 26-letter Key Phrase alphabet, retaining repeated phrase letters and word divisions."
        },
        input_type: "text",
        output_type: if decode { "text.optional" } else { "text" },
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: "cipher key-phrase",
        example: "cube cipher key-phrase --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: INPUTS,
            parameters: c::schema::<Parameters>,
            outputs: if decode { &OUTPUTS } else { &ENCODE_OUTPUTS },
            limits: c::Limits {
                max_input_bytes: Some(MAX_INPUT as u64),
                max_output_bytes: Some(MAX_OUTPUT as u64),
                max_items: Some(MAX_TOKENS as u64),
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
                    "Decode Key Phrase (ACA)"
                } else {
                    "Encode Key Phrase (ACA)"
                },
                family: "many_to_one_substitution",
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::HistoricalCipher,
                    limitations: "Puzzle/historical use only; no modern confidentiality or authentication. ASCII letter case canonicalizes; proper-noun markers are caller supplied.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: descriptor.description,
                    alphabet: "Fixed A-Z straight source alphabet; uppercase ASCII letters, preserve printable punctuation/spaces and caller-inserted proper-noun markers.",
                    key: "Exactly 26 uppercase letters of a complete caller-chosen phrase, repetitions retained. No keyword deduplication or linguistic phrase completion.",
                    key_advancement: "Stateless scalar lookup; inverse retains every reading in A-Z order and original byte positions. No Cartesian expansion or scoring.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Input <=1 MiB/65536 printable ASCII scalars; phrase exactly 26 uppercase A-Z letters. Alternatives/report <=4 MiB with conservative preflight; no implicit proper-noun detection, phrase guessing or dictionary lookup.",
                rust_api: "cube_core::ciphers::mono::key_phrase::{encode,decode}",
                python_api: python,
                references: &[
                    "https://www.cryptogram.org/downloads/aca.info/ciphers/KeyPhrase.pdf",
                ],
            }),
        },
    }
}
const ROWS: [Row; 2] = [
    row(
        "cipher.key_phrase.encode",
        "key_phrase_encode",
        false,
        super::fixtures::encode,
    ),
    row(
        "cipher.key_phrase.decode",
        "key_phrase_decode",
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
        let text = i
            .get("text")
            .and_then(Value::as_str)
            .filter(|_| i.as_object().is_some_and(|v| v.len() == 1))
            .ok_or_else(|| {
                error(
                    op,
                    ErrorCode::InvalidInput,
                    "text",
                    "one named text input is required",
                )
            })?;
        if op.ends_with(".decode") {
            serde_json::to_value(decode(text, &p)?)
        } else {
            serde_json::to_value(encode(text, &p)?)
        }
        .map_err(|_| invalid("report"))
    };
    run().map_err(|e: OperationError| e::Failure(e.code.as_str()))
}
const EXECUTORS: &[e::Entry] = &[
    e::Entry::pure(
        "cipher.key_phrase.encode",
        &[e::Representation::Text],
        &[
            e::Representation::Text,
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Text,
        ],
        |i, p, _| native("cipher.key_phrase.encode", i, p),
    ),
    e::Entry::pure(
        "cipher.key_phrase.decode",
        &[e::Representation::Text],
        &[
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Text,
        ],
        |i, p, _| native("cipher.key_phrase.decode", i, p),
    ),
];
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
