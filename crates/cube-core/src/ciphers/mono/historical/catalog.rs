use super::*;
use crate::operations::{self as o, contracts as c, execution as e, metadata as m};
use e::Representation::{Json, Text};
use serde_json::{Value, json};
const DEFINITION: &[o::Parameter] = &[o::Parameter {
    name: "definition",
    description: "Explicit JSON codebook or 10x10 syllabary; units are presegmented.",
    required: true,
    default: None,
    constraints: o::ParameterType::Text { choices: &[] },
}];
const PAIRS: &[o::Parameter] = &[
    o::Parameter {
        name: "pairs",
        description: "Required disjoint two-scalar pairs, JSON array.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices: &[] },
    },
    o::Parameter {
        name: "unmapped",
        description: "Reject unmatched scalars or preserve them exactly.",
        required: false,
        default: Some(o::ParameterValue::Text("reject")),
        constraints: o::ParameterType::Text {
            choices: &["reject", "preserve"],
        },
    },
];
const TEXT: &[c::Port] = &[c::Port {
    name: "text",
    semantic_type: "text",
    schema: c::schema::<super::super::params::Text>,
    required: true,
}];
const UNITS: &[c::Port] = &[c::Port {
    name: "units",
    semantic_type: "cipher.plain_units",
    schema: c::schema::<Tokens>,
    required: true,
}];
const TOKENS: &[c::Port] = &[c::Port {
    name: "tokens",
    semantic_type: "cipher.symbol_tokens",
    schema: c::schema::<Tokens>,
    required: true,
}];
const REPORT: &[c::Port] = &[c::Port {
    name: "report",
    semantic_type: "cipher.codebook_report",
    schema: c::schema::<DecodeReport>,
    required: true,
}];
#[derive(Clone, Copy)]
struct Row {
    profile: m::Entry,
    contract: c::Entry,
}
struct Spec {
    family: &'static str,
    python: &'static str,
    rust: &'static str,
    reversible: bool,
    output_bytes: usize,
    max_items: Option<u64>,
    id: &'static str,
    name: &'static str,
    description: &'static str,
    command: &'static str,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    params: fn() -> schemars::Schema,
    parameters: &'static [o::Parameter],
    fixture: fn() -> Result<c::Fixture, OperationError>,
    references: &'static [&'static str],
}
const fn row(s: Spec) -> Row {
    let descriptor = o::Descriptor {
        id: s.id,
        category: "ciphers",
        description: s.description,
        input_type: s.inputs[0].semantic_type,
        output_type: s.outputs[0].semantic_type,
        parameters: s.parameters,
        deterministic: true,
        reversible: Some(s.reversible),
        command: s.command,
        example: "cube cipher --help",
    };
    Row {
        contract: c::Entry {
            operation_version: 1,
            descriptor,
            inputs: s.inputs,
            parameters: s.params,
            outputs: s.outputs,
            limits: c::Limits {
                max_input_bytes: Some(MAX_BYTES as u64),
                max_output_bytes: Some(s.output_bytes as u64),
                max_items: s.max_items,
                max_work: None,
                max_memory_bytes: None,
            },
            fixture: s.fixture,
        },
        profile: m::Entry {
            descriptor,
            metadata: Some(m::Metadata {
                schema_version: 1,
                canonical_name: s.name,
                family: s.family,
                aliases: &[],
                security: m::Security {
                    classification: m::Classification::HistoricalCipher,
                    limitations: "Historical/challenge use only; no modern confidentiality or authentication. Explicit units and alternative readings do not imply linguistic reconstruction.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: m::Profile {
                    variant: s.description,
                    alphabet: "Exact Unicode scalar/unit matching; no normalization. Fixed Great Cipher transcription is ASCII, case retained.",
                    key: "Explicit disjoint pairs/codebook/keysquare, or the fixed Bazeries 1691 table.",
                    key_advancement: "First listed code per exact unit, reset each call. No randomness, longest-match or language dictionary.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[],
                resource_notes: "Codebooks: at most 1 MiB per serialized input/output/table, 65536 units, 4096 entries and total codes, 256 UTF-8 bytes per unit/code, 16 alternatives per entry. Syllabary has exactly 100 row-major cells. Text profiles: 1 MiB input; Malespin output <=1 MiB, Vatsyayana output <=4 MiB. Pairs use existing 4096-scalar/64 KiB alphabet bounds.",
                rust_api: s.rust,
                python_api: s.python,
                references: s.references,
            }),
        },
    }
}
const ROWS: [Row; 8] = [
    row(Spec {
        id: "cipher.malespin",
        python: "malespin",
        rust: "cube_core::ciphers::mono::historical::malespin",
        reversible: true,
        family: "monoalphabetic_substitution",
        max_items: None,
        output_bytes: MAX_BYTES,
        name: "Malespín (ten-letter written profile)",
        description: "Reciprocal ASCII a/e, i/o, b/t, f/g, p/m swaps; retain case and preserve other scalars, including accents.",
        command: "cipher malespin",
        inputs: TEXT,
        outputs: TEXT,
        params: c::schema::<super::super::params::Empty>,
        parameters: &[],
        fixture: super::fixtures::malespin,
        references: &["https://doi.org/10.4000/bulletinhispanique.16749"],
    }),
    row(Spec {
        id: "cipher.vatsyayana",
        python: "vatsyayana",
        rust: "cube_core::ciphers::mono::historical::vatsyayana",
        reversible: true,
        family: "monoalphabetic_substitution",
        max_items: None,
        output_bytes: super::super::MAX_OUTPUT,
        name: "Vatsyayana (explicit pairs)",
        description: "Reciprocal exact caller-defined disjoint scalar pairs; reject unmatched symbols by default, without a guessed historical alphabet.",
        command: "cipher vatsyayana",
        inputs: TEXT,
        outputs: &[c::Port {
            name: "text",
            semantic_type: "text",
            schema: c::schema::<super::super::params::ExpandedText>,
            required: true,
        }],
        params: c::schema::<Pairs>,
        parameters: PAIRS,
        fixture: super::fixtures::vatsyayana,
        references: &[
            "https://cemc.uwaterloo.ca/sites/default/files/documents/2020/2020BCCContestSolutions7_8.pdf",
        ],
    }),
    row(Spec {
        id: "cipher.nomenclator.encode",
        python: "nomenclator_encode",
        rust: "cube_core::ciphers::mono::historical::Book::encode",
        reversible: false,
        family: "historical_codebook",
        max_items: Some(MAX_ITEMS as u64),
        output_bytes: MAX_BYTES,
        name: "Nomenclator encode",
        description: "Encode explicit plaintext units using a caller-owned codebook; select the first matching code deterministically.",
        command: "cipher nomenclator",
        inputs: UNITS,
        outputs: TOKENS,
        params: c::schema::<BookParameters>,
        parameters: DEFINITION,
        fixture: super::fixtures::nomenclator_encode,
        references: &["https://doi.org/10.1080/01611194.2024.2396800"],
    }),
    row(Spec {
        id: "cipher.nomenclator.decode",
        python: "nomenclator_decode",
        rust: "cube_core::ciphers::mono::historical::Book::decode",
        reversible: false,
        family: "historical_codebook",
        max_items: Some(MAX_ITEMS as u64),
        output_bytes: MAX_BYTES,
        name: "Nomenclator decode",
        description: "Decode explicit code tokens with alternative readings and original positions; apply declared nulls and previous-group cancellation.",
        command: "cipher nomenclator",
        inputs: TOKENS,
        outputs: REPORT,
        params: c::schema::<BookParameters>,
        parameters: DEFINITION,
        fixture: super::fixtures::nomenclator_decode,
        references: &["https://doi.org/10.1080/01611194.2024.2396800"],
    }),
    row(Spec {
        id: "cipher.syllabary.encode",
        python: "syllabary_encode",
        rust: "cube_core::ciphers::mono::historical::Book::encode",
        reversible: false,
        family: "historical_codebook",
        max_items: Some(MAX_ITEMS as u64),
        output_bytes: MAX_BYTES,
        name: "Syllabary encode",
        description: "Encode caller-segmented units with an explicit ACA 10x10 keysquare and digit coordinate permutations.",
        command: "cipher syllabary",
        inputs: UNITS,
        outputs: TOKENS,
        params: c::schema::<SyllabaryParameters>,
        parameters: DEFINITION,
        fixture: super::fixtures::syllabary_encode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Syllabary.pdf"],
    }),
    row(Spec {
        id: "cipher.syllabary.decode",
        python: "syllabary_decode",
        rust: "cube_core::ciphers::mono::historical::Book::decode",
        reversible: false,
        family: "historical_codebook",
        max_items: Some(MAX_ITEMS as u64),
        output_bytes: MAX_BYTES,
        name: "Syllabary decode",
        description: "Decode exact ACA syllabary coordinate pairs; retain unit boundaries and original token positions.",
        command: "cipher syllabary",
        inputs: TOKENS,
        outputs: REPORT,
        params: c::schema::<SyllabaryParameters>,
        parameters: DEFINITION,
        fixture: super::fixtures::syllabary_decode,
        references: &["https://www.cryptogram.org/downloads/aca.info/ciphers/Syllabary.pdf"],
    }),
    row(Spec {
        id: "cipher.great_cipher.encode",
        python: "great_cipher_encode",
        rust: "cube_core::ciphers::mono::historical::Book::encode",
        reversible: false,
        family: "historical_codebook",
        max_items: Some(MAX_ITEMS as u64),
        output_bytes: MAX_BYTES,
        name: "Great Cipher encode (Bazeries 1691)",
        description: "Encode exact presegmented ASCII transcription units from Bazeries' 1893 Grand Chiffre de 1691 deciphering table; deterministic first code.",
        command: "cipher great-cipher",
        inputs: UNITS,
        outputs: TOKENS,
        params: c::schema::<super::super::params::Empty>,
        parameters: &[],
        fixture: super::fixtures::great_cipher_encode,
        references: &["https://gallica.bnf.fr/ark:/12148/bpt6k1523532j"],
    }),
    row(Spec {
        id: "cipher.great_cipher.decode",
        python: "great_cipher_decode",
        rust: "cube_core::ciphers::mono::historical::Book::decode",
        reversible: false,
        family: "historical_codebook",
        max_items: Some(MAX_ITEMS as u64),
        output_bytes: MAX_BYTES,
        name: "Great Cipher decode (Bazeries 1691)",
        description: "Decode Bazeries' 1691 table with explicit I/J, U/V and other alternatives, null positions and group-27 cancellation; unknown surviving entries reject.",
        command: "cipher great-cipher",
        inputs: TOKENS,
        outputs: REPORT,
        params: c::schema::<super::super::params::Empty>,
        parameters: &[],
        fixture: super::fixtures::great_cipher_decode,
        references: &["https://gallica.bnf.fr/ark:/12148/bpt6k1523532j"],
    }),
];
const fn profiles() -> [m::Entry; 8] {
    let mut rows = [ROWS[0].profile; 8];
    let mut i = 0;
    while i < 8 {
        rows[i] = ROWS[i].profile;
        i += 1;
    }
    rows
}
const fn contracts() -> [c::Entry; 8] {
    let mut rows = [ROWS[0].contract; 8];
    let mut i = 0;
    while i < 8 {
        rows[i] = ROWS[i].contract;
        i += 1;
    }
    rows
}
const PROFILED: [m::Entry; 8] = profiles();
const CONTRACTS: [c::Entry; 8] = contracts();
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
pub(crate) const MODULE: o::Module = o::Module::profiled(OPERATIONS, &PROFILED)
    .with_contracts(&CONTRACTS)
    .with_executors(EXECUTORS);
fn field<T: serde::de::DeserializeOwned>(
    value: &Value,
    name: &'static str,
) -> Result<T, OperationError> {
    let failure = || {
        if ["text", "units", "tokens"].contains(&name) {
            error(CONFIG, ErrorCode::InvalidInput, name, "invalid named input")
        } else {
            invalid(name)
        }
    };
    if value.as_object().is_none_or(|fields| fields.len() != 1) {
        return Err(failure());
    }
    serde_json::from_value(value.get(name).cloned().ok_or_else(failure)?).map_err(|_| failure())
}
/// Thin named-port adapter shared by native hosts and Wasm. Algorithms remain
/// in their concrete profile/Book modules; it grants no host capabilities.
pub fn execute(
    operation: &str,
    inputs: &Value,
    parameters: &Value,
) -> Result<Value, OperationError> {
    let id = OPERATIONS
        .iter()
        .find(|d| d.id == operation)
        .ok_or_else(|| invalid("operation"))?
        .id;
    if parameters
        .as_object()
        .is_none_or(|fields| !fields.is_empty())
        && [
            "cipher.malespin",
            "cipher.great_cipher.encode",
            "cipher.great_cipher.decode",
        ]
        .contains(&operation)
    {
        return Err(invalid("parameters").operation(id));
    }
    dispatch(operation, inputs, parameters).map_err(|e| e.operation(id))
}
fn dispatch(operation: &str, inputs: &Value, parameters: &Value) -> Result<Value, OperationError> {
    match operation {
        "cipher.malespin" => Ok(json!({"text":super::malespin(&field::<String>(inputs,"text")?)?})),
        "cipher.vatsyayana" => Ok(
            json!({"text":super::vatsyayana(&field::<String>(inputs,"text")?,&parse(&parameters.to_string())?)?}),
        ),
        _ => {
            let book = match operation {
                "cipher.nomenclator.encode" | "cipher.nomenclator.decode" => {
                    Book::new(field(parameters, "definition")?)?
                }
                "cipher.syllabary.encode" | "cipher.syllabary.decode" => {
                    field::<Syllabary>(parameters, "definition")?.book()?
                }
                "cipher.great_cipher.encode" | "cipher.great_cipher.decode" => great_cipher()?,
                _ => return Err(invalid("operation")),
            };
            if operation.ends_with(".encode") {
                Ok(json!({"tokens":book.encode(&field::<Vec<String>>(inputs,"units")?)?}))
            } else {
                Ok(json!({"report":book.decode(&field::<Vec<String>>(inputs,"tokens")?)?}))
            }
        }
    }
}
fn native(operation: &str, i: &Value, p: &Value) -> e::Result<Value> {
    execute(operation, i, p).map_err(|error| e::Failure(error.code.as_str()))
}
const EXECUTORS: &[e::Entry] = &[
    e::Entry::pure("cipher.malespin", &[Text], &[Text], |i, p, _| {
        native("cipher.malespin", i, p)
    }),
    e::Entry::pure("cipher.vatsyayana", &[Text], &[Text], |i, p, _| {
        native("cipher.vatsyayana", i, p)
    }),
    e::Entry::pure("cipher.nomenclator.encode", &[Json], &[Json], |i, p, _| {
        native("cipher.nomenclator.encode", i, p)
    }),
    e::Entry::pure("cipher.nomenclator.decode", &[Json], &[Json], |i, p, _| {
        native("cipher.nomenclator.decode", i, p)
    }),
    e::Entry::pure("cipher.syllabary.encode", &[Json], &[Json], |i, p, _| {
        native("cipher.syllabary.encode", i, p)
    }),
    e::Entry::pure("cipher.syllabary.decode", &[Json], &[Json], |i, p, _| {
        native("cipher.syllabary.decode", i, p)
    }),
    e::Entry::pure("cipher.great_cipher.encode", &[Json], &[Json], |i, p, _| {
        native("cipher.great_cipher.encode", i, p)
    }),
    e::Entry::pure("cipher.great_cipher.decode", &[Json], &[Json], |i, p, _| {
        native("cipher.great_cipher.decode", i, p)
    }),
];
