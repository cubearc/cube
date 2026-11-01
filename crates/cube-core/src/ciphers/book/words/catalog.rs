use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use serde_json::Value;
const PAGES: c::Port = c::Port {
    name: "pages",
    semantic_type: "book.pages",
    schema: c::schema::<Pages>,
    required: true,
};
const ENCODE_INPUTS: &[c::Port] = &[
    PAGES,
    c::Port {
        name: "words",
        semantic_type: "book.words",
        schema: c::schema::<Words>,
        required: true,
    },
];
const DECODE_INPUTS: &[c::Port] = &[
    PAGES,
    c::Port {
        name: "coordinates",
        semantic_type: "book.word_coordinates",
        schema: || c::field::<DecodeInput>("coordinates"),
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "words",
        semantic_type: "book.words",
        schema: || c::field::<Report>("words"),
        required: true,
    },
    c::Port {
        name: "coordinates",
        semantic_type: "book.word_coordinates",
        schema: || c::field::<Report>("coordinates"),
        required: true,
    },
    c::Port {
        name: "source_id",
        semantic_type: "text",
        schema: || c::field::<Report>("source_id"),
        required: true,
    },
    c::Port {
        name: "source_sha256",
        semantic_type: "text",
        schema: || c::field::<Report>("source_sha256"),
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
    description: "Required arnold_paged_words_v1 profile, explicit source edition ID, index base and first/round-robin choice; source pages are named input data.",
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
            "Decode complete words from exact caller page/line/word coordinates, retaining source provenance."
        } else {
            "Encode exact whole words with an explicit paginated Arnold-style source and deterministic coordinate choices."
        },
        input_type: "book.pages",
        output_type: "book.words",
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: "cipher arnold",
        example: "cube cipher arnold --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: if decode { DECODE_INPUTS } else { ENCODE_INPUTS },
            parameters: c::schema::<Parameters>,
            outputs: OUTPUTS,
            limits: c::Limits {
                max_input_bytes: Some((2 * MAX_INPUT) as u64),
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
                    "Decode Arnold-style word code (caller pages)"
                } else {
                    "Encode Arnold-style word code (caller pages)"
                },
                family: "book_word_code",
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::HistoricalCipher,
                    limitations: "Caller-source-dependent historical word code; no modern confidentiality or authentication. Source IDs are assertions, not an edition lookup or verification service.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: descriptor.description,
                    alphabet: "Exact caller Unicode words split on shared Rust Unicode whitespace within explicit LF/CRLF lines and caller page boundaries. Case, punctuation and normalization remain exact.",
                    key: "Caller pages and source/edition ID are required; no Blackstone/dictionary edition, pagination, code names or letter fallback is bundled or inferred.",
                    key_advancement: "First or independent per-word round robin, reset each call. Decoding returns exact presegmented words; source spacing cannot be reconstructed.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Source pages <=1 MiB compact JSON and65536 combined pages+lines+words; words <=256 UTF-8 bytes; message/coordinates <=65536 items/1 MiB compact JSON; report <=4 MiB with conservative preflight. Lower host budgets still apply.",
                rust_api: "cube_core::ciphers::book::words::{encode,decode}",
                python_api: python,
                references: &[
                    "https://www.nsa.gov/Portals/75/documents/news-features/declassified-documents/friedman-documents/lectures-speeches/FOLDER_019/41700339073966.pdf",
                ],
            }),
        },
    }
}
const ROWS: [Row; 2] = [
    row(
        "cipher.arnold.encode",
        "arnold_encode",
        false,
        super::fixtures::encode,
    ),
    row(
        "cipher.arnold.decode",
        "arnold_decode",
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
        let result = if op.ends_with(".decode") {
            let input: DecodeInput = serde_json::from_value(i.clone()).map_err(|_| {
                error(
                    op,
                    ErrorCode::InvalidInput,
                    "inputs",
                    "expected exact pages and word coordinates",
                )
            })?;
            decode(&input.pages.0, &input.coordinates, &p)?
        } else {
            let input: EncodeInput = serde_json::from_value(i.clone()).map_err(|_| {
                error(
                    op,
                    ErrorCode::InvalidInput,
                    "inputs",
                    "expected exact pages and presegmented words",
                )
            })?;
            encode(&input.pages.0, &input.words.0, &p)?
        };
        serde_json::to_value(result).map_err(|_| invalid("report"))
    };
    run().map_err(|e: OperationError| e::Failure(e.code.as_str()))
}
const EXECUTORS: &[e::Entry] = &[
    e::Entry::pure(
        "cipher.arnold.encode",
        &[e::Representation::Json, e::Representation::Json],
        &[
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Text,
            e::Representation::Text,
            e::Representation::Text,
        ],
        |i, p, _| native("cipher.arnold.encode", i, p),
    ),
    e::Entry::pure(
        "cipher.arnold.decode",
        &[e::Representation::Json, e::Representation::Json],
        &[
            e::Representation::Json,
            e::Representation::Json,
            e::Representation::Text,
            e::Representation::Text,
            e::Representation::Text,
        ],
        |i, p, _| native("cipher.arnold.decode", i, p),
    ),
];
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
