use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use serde_json::Value;
const INPUT_TEXT: &[c::Port] = &[c::Port {
    name: "text",
    semantic_type: "text",
    schema: c::schema::<super::super::params::Text>,
    required: true,
}];
const INPUT_TOKENS: &[c::Port] = &[c::Port {
    name: "tokens",
    semantic_type: "cipher.symbol_tokens",
    schema: c::schema::<super::super::historical::Tokens>,
    required: true,
}];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<Report>("text"),
        required: true,
    },
    c::Port {
        name: "tokens",
        semantic_type: "cipher.symbol_tokens",
        schema: || c::field::<Report>("tokens"),
        required: true,
    },
    c::Port {
        name: "table",
        semantic_type: "cipher.homophonic_table",
        schema: || c::field::<Report>("table"),
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
    description: "Required explicit alphabet, key, preparation, rotation, decimal origin, token width and selection policy; see typed schema.",
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
            "Decode exact ACA Numbered Key tokens with the complete repeated-position table."
        } else {
            "Construct the repeated-position ACA Numbered Key table and encode with explicit deterministic choices."
        },
        input_type: if decode {
            "cipher.symbol_tokens"
        } else {
            "text"
        },
        output_type: "text",
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: "cipher numbered-key",
        example: "cube cipher numbered-key --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: if decode { INPUT_TOKENS } else { INPUT_TEXT },
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
                    "Decode Numbered Key (ACA)"
                } else {
                    "Encode Numbered Key (ACA)"
                },
                family: "homophonic_substitution",
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::HistoricalCipher,
                    limitations: "Puzzle/historical use only; no modern confidentiality or authentication. Text preparation can discard ASCII separators and punctuation.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: descriptor.description,
                    alphabet: "Explicit unique Unicode scalars in caller order; exact or ASCII-lowercase-letter preparation. No Unicode normalization.",
                    key: "Retain repeated prepared key positions, append each absent alphabet scalar once, rotate left by the explicit offset, then number from the exact u128 decimal origin.",
                    key_advancement: "First listed token or per-symbol round robin, reset each invocation. Decode uses all exact padded tokens.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "1 MiB input/65536 message scalars or tokens, 4096 extended-key positions, <=64 KiB serialized parameters/table, exact decimal token width 1..40, <=4 MiB report. Origin arithmetic is checked.",
                rust_api: "cube_core::ciphers::mono::numbered_key::{encode,decode}",
                python_api: python,
                references: &[
                    "https://www.cryptogram.org/downloads/aca.info/ciphers/NumberedKey.pdf",
                ],
            }),
        },
    }
}
const ROWS: [Row; 2] = [
    row(
        "cipher.numbered_key.encode",
        "numbered_key_encode",
        false,
        super::fixtures::encode,
    ),
    row(
        "cipher.numbered_key.decode",
        "numbered_key_decode",
        true,
        super::fixtures::decode,
    ),
];
const PROFILED: [m::Entry; 2] = [ROWS[0].profile, ROWS[1].profile];
const CONTRACTS: [c::Entry; 2] = [ROWS[0].contract, ROWS[1].contract];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
fn native(op: &'static str, i: &Value, p: &Value) -> e::Result<Value> {
    let result = (|| {
        let parameters: Parameters = crate::ciphers::poly::parse_parameters(&p.to_string())?;
        if i.as_object().is_none_or(|fields| fields.len() != 1) {
            return Err(error(
                op,
                ErrorCode::InvalidInput,
                "inputs",
                "exactly one named input is required",
            ));
        }
        let report = if op.ends_with(".decode") {
            let tokens = serde_json::from_value::<Vec<String>>(
                i.get("tokens").cloned().unwrap_or(Value::Null),
            )
            .map_err(|_| {
                error(
                    op,
                    ErrorCode::InvalidInput,
                    "tokens",
                    "expected a token array",
                )
            })?;
            decode(&tokens, &parameters)?
        } else {
            encode(
                i.get("text")
                    .and_then(Value::as_str)
                    .ok_or_else(|| error(op, ErrorCode::InvalidInput, "text", "expected text"))?,
                &parameters,
            )?
        };
        serde_json::to_value(report).map_err(|_| invalid("report"))
    })();
    result.map_err(|err: OperationError| e::Failure(err.code.as_str()))
}
const EXECUTORS: &[e::Entry] = &[
    e::Entry::pure(
        "cipher.numbered_key.encode",
        &[e::Representation::Text],
        &[
            e::Representation::Text,
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Text,
        ],
        |i, p, _| native("cipher.numbered_key.encode", i, p),
    ),
    e::Entry::pure(
        "cipher.numbered_key.decode",
        &[e::Representation::Json],
        &[
            e::Representation::Text,
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Text,
        ],
        |i, p, _| native("cipher.numbered_key.decode", i, p),
    ),
];
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
