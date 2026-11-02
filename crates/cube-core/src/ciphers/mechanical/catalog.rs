use super::{disks::*, *};
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
#[derive(Clone, Copy)]
enum Family {
    M94,
    Jefferson,
    M138a,
}
impl Family {
    const fn variant(self) -> &'static str {
        match self {
            Self::M94 => "Standard M-94/CSP-488 wheels; FM24-5 section V,19 October1942.",
            Self::Jefferson => {
                "Configured numbered26-letter wheels from Jefferson's manuscript, before22 March1802; caller chooses the complete wheel set."
            }
            Self::M138a => {
                "M-138-A/CSP-845 raw30-track/100-strip mechanism with explicit selected strip identities; no automatic message procedure."
            }
        }
    }
    const fn references(self) -> &'static [&'static str] {
        match self {
            Self::M94 => &[
                "https://www.cryptomuseum.com/crypto/usa/m94/files/FM24-5_1942_pp70-81.pdf",
                "https://www.jproc.ca/crypto/m94.html",
            ],
            Self::Jefferson => &["https://founders.archives.gov/documents/Jefferson/01-37-02-0082"],
            Self::M138a => &[
                "https://www.nsa.gov/History/National-Cryptologic-Museum/Exhibits-Artifacts/Exhibit-View/Article/2719266/csp-845/",
                "https://www.nsa.gov/Portals/75/documents/news-features/declassified-documents/friedman-documents/lectures-speeches/FOLDER_166/41759739079888.pdf",
            ],
        }
    }
}
const fn entry(
    id: &'static str,
    name: &'static str,
    command: &'static str,
    python: &'static str,
    family: Family,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: family.variant(),
            input_type: "json",
            output_type: "text",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command,
            example: "cube cipher --help",
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "historical_mechanical",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::HistoricalCipher,
                limitations: "Historical interoperability only; no modern confidentiality or authentication. Wheel order and selected rows are key material.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: family.variant(),
                alphabet: "Prepared ASCII A-Z; exact empty input supported. No normalization, padding, random row choice or language guessing.",
                key: "Required complete wheel/strip definitions where configurable, explicit left-to-right IDs and per-block clockwise row offsets1..25.",
                key_advancement: "Left-to-right wheel/track slots restart at each block boundary; each complete/partial block consumes one supplied row displacement.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Text input/output <=1 MiB; settings JSON <=512 KiB. Jefferson <=64 wheels/65536 rows; M-94 exactly25 wheels/<=41944 rows; M-138-A30 selected of100 strips/<=34953 rows. Fixed26-cell inverse per selected wheel and one lookup per input letter. No network, I/O, implicit capture or pipeline adapter.",
            rust_api: "cube_core::ciphers::mechanical::disks",
            python_api: python,
            references: family.references(),
        }),
    }
}
const PROFILED: [m::Entry; 6] = [
    entry(
        "cipher.m94.encode",
        "Encipher with M-94",
        "cipher m94 encode",
        "m94_encode",
        Family::M94,
    ),
    entry(
        "cipher.m94.decode",
        "Decipher with M-94",
        "cipher m94 decode",
        "m94_decode",
        Family::M94,
    ),
    entry(
        "cipher.jefferson.encode",
        "Encipher with Jefferson wheels",
        "cipher jefferson encode",
        "jefferson_encode",
        Family::Jefferson,
    ),
    entry(
        "cipher.jefferson.decode",
        "Decipher with Jefferson wheels",
        "cipher jefferson decode",
        "jefferson_decode",
        Family::Jefferson,
    ),
    entry(
        "cipher.m138a.encode",
        "Encipher with M-138-A strips",
        "cipher m138a encode",
        "m138a_encode",
        Family::M138a,
    ),
    entry(
        "cipher.m138a.decode",
        "Decipher with M-138-A strips",
        "cipher m138a decode",
        "m138a_decode",
        Family::M138a,
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const TEXT: &[c::Port] = &[c::Port {
    name: "text",
    semantic_type: "text",
    schema: || c::field::<Report>("text"),
    required: true,
}];
#[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct NoParameters {}
const fn inputs(settings_schema: fn() -> schemars::Schema) -> [c::Port; 2] {
    [
        c::Port {
            name: "text",
            semantic_type: "text",
            schema: || c::field::<Report>("text"),
            required: true,
        },
        c::Port {
            name: "settings",
            semantic_type: "json",
            schema: settings_schema,
            required: true,
        },
    ]
}
const M94_INPUTS: &[c::Port] = &inputs(c::schema::<M94Parameters>);
const JEFFERSON_INPUTS: &[c::Port] = &inputs(c::schema::<JeffersonParameters>);
const M138A_INPUTS: &[c::Port] = &inputs(c::schema::<M138aParameters>);
const fn contract(index: usize, fixture: fn() -> Result<c::Fixture, OperationError>) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs: match index {
            0 | 1 => M94_INPUTS,
            2 | 3 => JEFFERSON_INPUTS,
            _ => M138A_INPUTS,
        },
        parameters: c::schema::<NoParameters>,
        outputs: TEXT,
        limits: c::Limits {
            max_input_bytes: Some((MAX_INPUT + MAX_PARAMETERS) as u64),
            max_output_bytes: Some((MAX_INPUT * 2) as u64),
            max_items: Some(65536),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[
    contract(0, encode_fixture),
    contract(1, decode_fixture),
    contract(2, jefferson_encode_fixture),
    contract(3, jefferson_decode_fixture),
    contract(4, m138a_encode_fixture),
    contract(5, m138a_decode_fixture),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
pub(crate) fn execute(
    op: &str,
    inputs: &Value,
    _parameters: &Value,
) -> Result<Value, OperationError> {
    let id = OPERATIONS
        .iter()
        .find(|d| d.id == op)
        .map(|d| d.id)
        .ok_or_else(|| {
            OperationError::new(ErrorCode::InvalidParameter, "unknown mechanical operation")
        })?;
    let text = inputs
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| error(id, ErrorCode::InvalidInput, "text", "expected text"))?;
    let settings = inputs.get("settings").ok_or_else(|| {
        error(
            id,
            ErrorCode::InvalidInput,
            "settings",
            "explicit typed settings input required",
        )
    })?;
    let report = match id {
        "cipher.m94.encode" => m94_encode(text, &parse_parameters(&settings.to_string(), id)?)?,
        "cipher.m94.decode" => m94_decode(text, &parse_parameters(&settings.to_string(), id)?)?,
        "cipher.jefferson.encode" => {
            jefferson_encode(text, &parse_parameters(&settings.to_string(), id)?)?
        }
        "cipher.jefferson.decode" => {
            jefferson_decode(text, &parse_parameters(&settings.to_string(), id)?)?
        }
        "cipher.m138a.encode" => m138a_encode(text, &parse_parameters(&settings.to_string(), id)?)?,
        "cipher.m138a.decode" => m138a_decode(text, &parse_parameters(&settings.to_string(), id)?)?,
        _ => unreachable!("installed disk operation"),
    };
    Ok(json!({"text":report.text}))
}
const PLAIN: &str =
    "HAVEJUSTREACHEDEASTERNEDGEOFWOODSALONGFIVEFIVETWODASHFIVENINETWOROADWILLREMAININOBSERVATION";
const CIPHER: &str =
    "JUKLDYKITZIIVCYCVUYVPYWHJYUYEZDHVUZDBQPOZMCFNBJJIXEAPTHYOWHKWWTNYGMPRZJIFADMEQRHBPOJTYUQNTW";
fn fixture(decode: bool) -> Result<c::Fixture, OperationError> {
    let p = json!({"profile":"m94_v1","order":[3,10,12,16,6,21,7,14,1,23,17,5,20,25,4,11,13,18,8,22,9,15,2,24,19],"row_offsets":[20,10,9,4]});
    let (input, expected) = if decode {
        (CIPHER, PLAIN)
    } else {
        (PLAIN, CIPHER)
    };
    let outputs = execute(
        OPERATIONS[usize::from(decode)].id,
        &json!({"text":input,"settings":p}),
        &json!({}),
    )?;
    assert_eq!(outputs["text"], expected);
    Ok(c::Fixture {
        inputs: json!({"text":input,"settings":p}),
        parameters: json!({}),
        outputs,
    })
}
fn encode_fixture() -> Result<c::Fixture, OperationError> {
    fixture(false)
}
fn decode_fixture() -> Result<c::Fixture, OperationError> {
    fixture(true)
}
fn configured_fixture(
    index: usize,
    parameters: Value,
    plain: String,
    cipher: String,
) -> Result<c::Fixture, OperationError> {
    let (input, expected) = if index % 2 == 1 {
        (cipher, plain)
    } else {
        (plain, cipher)
    };
    let inputs = json!({"text":input,"settings":parameters});
    let outputs = execute(OPERATIONS[index].id, &inputs, &json!({}))?;
    assert_eq!(outputs["text"], expected);
    Ok(c::Fixture {
        inputs,
        parameters: json!({}),
        outputs,
    })
}
fn jefferson_fixture(index: usize) -> Result<c::Fixture, OperationError> {
    configured_fixture(
        index,
        json!({"profile":"jefferson_configured_v1","wheels":["ABCDEFGHIJKLMNOPQRSTUVWXYZ","AZYXWVUTSRQPONMLKJIHGFEDCB","QWERTYUIOPASDFGHJKLZXCVBNM"],"order":[1,2,3],"row_offsets":[1,25]}),
        "AAJAAA".into(),
        "BZKZBP".into(),
    )
}
fn jefferson_encode_fixture() -> Result<c::Fixture, OperationError> {
    jefferson_fixture(2)
}
fn jefferson_decode_fixture() -> Result<c::Fixture, OperationError> {
    jefferson_fixture(3)
}
fn m138a_fixture(index: usize) -> Result<c::Fixture, OperationError> {
    let strips = (0..100)
        .map(|i| {
            if i % 2 == 0 {
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            } else {
                "AZYXWVUTSRQPONMLKJIHGFEDCB"
            }
        })
        .collect::<Vec<_>>();
    configured_fixture(
        index,
        json!({"profile":"m138a_configured_v1","strip_series":"qualification:alternating-v1","strips":strips,"order":(1..=30).collect::<Vec<_>>(),"row_offsets":[1,2]}),
        "A".repeat(31),
        "BZ".repeat(15) + "C",
    )
}
fn m138a_encode_fixture() -> Result<c::Fixture, OperationError> {
    m138a_fixture(4)
}
fn m138a_decode_fixture() -> Result<c::Fixture, OperationError> {
    m138a_fixture(5)
}
