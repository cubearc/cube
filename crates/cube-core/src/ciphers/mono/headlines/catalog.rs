use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use serde_json::Value;
const INPUTS: &[c::Port] = &[c::Port {
    name: "headlines",
    semantic_type: "text.headlines",
    schema: c::schema::<Headlines>,
    required: true,
}];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "headlines",
        semantic_type: "text.headlines",
        schema: || c::field::<Report>("headlines"),
        required: true,
    },
    c::Port {
        name: "alphabet",
        semantic_type: "text",
        schema: || c::field::<Report>("alphabet"),
        required: true,
    },
    c::Port {
        name: "mappings",
        semantic_type: "cipher.substitution_rows",
        schema: || c::field::<Report>("mappings"),
        required: true,
    },
    c::Port {
        name: "definition_sha256",
        semantic_type: "text",
        schema: || c::field::<Report>("definition_sha256"),
        required: true,
    },
];
const PARAMS: &[o::Parameter] = &[o::Parameter {
    name: "parameters",
    description: "Required ACA base profile, HAT, KEY, five-letter SETTING and punctuation convention; see typed schema.",
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
            "Decode five caller Headlines with the qualified ACA base substitution rows."
        } else {
            "Construct the ACA HAT/KEY/SETTING mixed alphabet and encode five caller Headlines."
        },
        input_type: "text.headlines",
        output_type: "text.headlines",
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: "cipher headlines",
        example: "cube cipher headlines --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: INPUTS,
            parameters: c::schema::<Parameters>,
            outputs: OUTPUTS,
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
                    "Decode Headlines (ACA)"
                } else {
                    "Encode Headlines (ACA)"
                },
                family: "polyalphabetic_substitution",
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
                    alphabet: "Fixed A-Z alphabet; uppercase ASCII letters, preserve printable punctuation/spaces with an explicit hyphen convention.",
                    key: "Keyword alphabet from KEY, stable alphabetical HAT column order read down, five substitution rows starting at SETTING letters. Identity keying and other variants reject.",
                    key_advancement: "One explicit setting row per caller headline; exactly five headlines, including empty strings.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Exactly five headlines, aggregate <=1 MiB/65536 scalars; HAT and KEY 1..26 A-Z letters; SETTING exactly five A-Z letters. Report <=4 MiB. No source retrieval or implicit proper-noun detection.",
                rust_api: "cube_core::ciphers::mono::headlines::transform",
                python_api: python,
                references: &[
                    "https://www.cryptogram.org/downloads/aca.info/ciphers/Headlines.pdf",
                ],
            }),
        },
    }
}
const ROWS: [Row; 2] = [
    row(
        "cipher.headlines.encode",
        "headlines_encode",
        false,
        super::fixtures::encode,
    ),
    row(
        "cipher.headlines.decode",
        "headlines_decode",
        true,
        super::fixtures::decode,
    ),
];
const PROFILED: [m::Entry; 2] = [ROWS[0].profile, ROWS[1].profile];
const CONTRACTS: [c::Entry; 2] = [ROWS[0].contract, ROWS[1].contract];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn native(op: &'static str, i: &Value, p: &Value) -> e::Result<Value> {
    let result = (|| {
        let p: Parameters = crate::ciphers::poly::parse_parameters(&p.to_string())?;
        if i.as_object().is_none_or(|fields| fields.len() != 1) {
            return Err(error(
                op,
                ErrorCode::InvalidInput,
                "inputs",
                "one named headlines input required",
            ));
        }
        let headlines = serde_json::from_value::<Vec<String>>(
            i.get("headlines").cloned().unwrap_or(Value::Null),
        )
        .map_err(|_| {
            error(
                op,
                ErrorCode::InvalidInput,
                "headlines",
                "expected five strings",
            )
        })?;
        serde_json::to_value(transform(&headlines, &p, op.ends_with(".decode"))?)
            .map_err(|_| invalid("report"))
    })();
    result.map_err(|e: OperationError| e::Failure(e.code.as_str()))
}
const EXECUTORS: &[e::Entry] = &[
    e::Entry::pure(
        "cipher.headlines.encode",
        &[e::Representation::Json],
        &[
            e::Representation::Json,
            e::Representation::Text,
            e::Representation::Json,
            e::Representation::Text,
        ],
        |i, p, _| native("cipher.headlines.encode", i, p),
    ),
    e::Entry::pure(
        "cipher.headlines.decode",
        &[e::Representation::Json],
        &[
            e::Representation::Json,
            e::Representation::Text,
            e::Representation::Json,
            e::Representation::Text,
        ],
        |i, p, _| native("cipher.headlines.decode", i, p),
    ),
];
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
