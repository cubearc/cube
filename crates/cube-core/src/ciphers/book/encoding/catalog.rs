//! Module-owned authored-once rows and schemas derived from actual DTOs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct InputText(#[schemars(length(max=1048576),extend("x-cube-max-utf8-bytes"=1048576))] String);
#[derive(serde::Serialize, serde::Deserialize, JsonSchema)]
#[serde(transparent)]
struct ScalarText(#[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=1048576))] String);
#[derive(Clone, Copy)]
struct Row {
    profile: m::Entry,
    contract: c::Entry,
}
struct Spec {
    name: &'static str,
    id: &'static str,
    description: &'static str,
    input_type: &'static str,
    python: &'static str,
    rust: &'static str,
    command: &'static str,
    parameters: fn() -> schemars::Schema,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    fixture: fn() -> Result<c::Fixture, OperationError>,
    reversible: bool,
}
const PARAMS: &[o::Parameter] = &[o::Parameter {
    name: "parameters",
    description: "CLI-only strict bounded local JSON parameter file; canonical typed fields are in the schema.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text { choices: &[] },
}];
const fn row(s: Spec) -> Row {
    let descriptor = o::Descriptor {
        id: s.id,
        category: CATEGORY,
        description: s.description,
        input_type: s.input_type,
        output_type: "json",
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(s.reversible),
        command: s.command,
        example: "cube --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: s.inputs,
            parameters: s.parameters,
            outputs: s.outputs,
            limits: c::Limits {
                max_input_bytes: Some(INPUT_MAX),
                max_output_bytes: Some(4194304),
                max_items: Some(65536),
                max_work: None,
                max_memory_bytes: Some(134217728),
            },
            fixture: s.fixture,
        },
        profile: m::Entry {
            descriptor,
            metadata: Some(m::Metadata {
                schema_version: 1,
                canonical_name: s.name,
                family: FAMILY,
                aliases: &[],
                security: m::Security {
                    classification: CLASSIFICATION,
                    limitations: "Source-defined puzzle representation only; no modern confidentiality or authentication. Provenance identifiers are data, never permissions or executable lookups.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: s.description,
                    alphabet: "Exact caller-owned Unicode scalars/tokens and explicit coordinates; no normalization or image recognition.",
                    key: "Caller-owned source/geometry/table; bounded typed fields, no inferred edition, key or locale.",
                    key_advancement: "Explicit first/round-robin source choices or per-token mapping/displacement; no random choices.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Each text <=1MiB; <=65536 tokens/coordinates. Geometry/table <=64KiB; <=512 keyboard entries or4096 token forms. Serialized output/report <=4MiB; combined direct-call temporary memory <=128MiB. No assets, retrieval, uploads or new pipeline adapter. See exact profile guide.",
                rust_api: s.rust,
                python_api: s.python,
                references: REFERENCES,
            }),
        },
    }
}
const CATEGORY: &str = "ciphers";
const FAMILY: &str = "book_puzzle";
const CLASSIFICATION: m::Classification = m::Classification::HistoricalCipher;
const INPUT_MAX: u64 = 2097152;
const REFERENCES: &[&str] = &["https://github.com/cubearc/cube/issues/74"];
static ROWS: [Row; 1] = [row(Spec {
    input_type: "text",
    name: "Book Coordinates",
    id: "cipher.book.encode",
    description: "Deterministic inverse book coordinates; exact source hash and existing word/line tokenization; no retrieval.",
    python: "book_encode",
    rust: "cube_core::ciphers::book::encoding::encode",
    command: "cipher book-encode",
    parameters: c::schema::<EncodeOptions>,
    inputs: &[
        c::Port {
            name: "source",
            semantic_type: "text",
            schema: c::schema::<InputText>,
            required: true,
        },
        c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<ScalarText>,
            required: true,
        },
    ],
    outputs: &[
        c::Port {
            name: "coordinates",
            semantic_type: "text",
            schema: || c::field::<EncodeReport>("coordinates"),
            required: true,
        },
        c::Port {
            name: "source_sha256",
            semantic_type: "text",
            schema: || c::field::<EncodeReport>("source_sha256"),
            required: true,
        },
        c::Port {
            name: "coordinate_count",
            semantic_type: "json",
            schema: || c::field::<EncodeReport>("coordinate_count"),
            required: true,
        },
        c::Port {
            name: "indexed_units",
            semantic_type: "json",
            schema: || c::field::<EncodeReport>("indexed_units"),
            required: true,
        },
    ],
    fixture: fixtures::book_encode,
    reversible: false,
})];
const fn profiles() -> [m::Entry; 1] {
    let mut out = [ROWS[0].profile; 1];
    let mut i = 0;
    while i < 1 {
        out[i] = ROWS[i].profile;
        i += 1;
    }
    out
}
const fn contracts() -> [c::Entry; 1] {
    let mut out = [ROWS[0].contract; 1];
    let mut i = 0;
    while i < 1 {
        out[i] = ROWS[i].contract;
        i += 1;
    }
    out
}
static PROFILES: [m::Entry; 1] = profiles();
const CONTRACTS: [c::Entry; 1] = contracts();
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILES);
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILES).with_contracts(&CONTRACTS);
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_typed_answers_and_boundaries_conform() {
        for row in ROWS {
            let fixture = (row.contract.fixture)().unwrap();
            let contract = c::describe(&row.contract.descriptor).unwrap();
            contract
                .validate_fixture(&fixture)
                .unwrap_or_else(|e| panic!("{}:{e}", contract.operation));
            let mut bad = fixture.parameters;
            bad.as_object_mut()
                .unwrap()
                .insert("unknown".into(), serde_json::json!(true));
            assert!(contract.validate_parameters(&bad).is_err());
        }
    }
}
