//! Versioned typed adapters; the Hagelin mechanism lives in hagelin.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "Explicit C-36 fixed25, C-38 standard27, M-209, C-52 C2/27 and later CX-52M-27 logical-contact profiles with required pins, lugs and starting indices.";
const fn entry(
    id: &'static str,
    name: &'static str,
    command: &'static str,
    python: &'static str,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: VARIANT,
            input_type: "json",
            output_type: "json",
            parameters: &[],
            deterministic: true,
            reversible: Some(true),
            command,
            example: "cube cipher hagelin --help",
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "historical_mechanical",
            aliases: &[
                m::Alias {
                    name: "C-36",
                    usage: "Required c36_fixed25_bouch_v1 profile only; other hardware variants are not inferred.",
                },
                m::Alias {
                    name: "C-38",
                    usage: "Required c38_standard27_v1 profile only; other hardware variants are not inferred.",
                },
                m::Alias {
                    name: "M-209",
                    usage: "Required m209_standard_v1 profile only; other hardware variants are not inferred.",
                },
                m::Alias {
                    name: "C-52",
                    usage: "Required c52_c2_27_v1 profile only; other hardware variants are not inferred.",
                },
                m::Alias {
                    name: "CX-52",
                    usage: "Required cx52_m27_late_v1 profile only; other hardware variants are not inferred.",
                },
            ],
            security: m::Security {
                classification: m::Classification::HistoricalCipher,
                limitations: "Historical interoperability only; no modern confidentiality or authentication. Pins, plaintext and state traces are private.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: VARIANT,
                alphabet: "Prepared ASCII A-Z with standard reversed alphabets. No automatic case folding, space substitution, grouping, padding or message procedure.",
                key: "Required binary pins and logical contact indices. C-36 fixed lug weights1,2,3,7,12; other profiles require all27 printing-bar lug masks. Slide is fixed25 for M-209 and required0..25 otherwise. Only trace defaults to false.",
                key_advancement: "Sample current pins once per cycle; count every matching printing bar once. C-36/C-38/M-209 step every wheel once; C-52 C2 steps1,1,2,2,2,2; later CX-52M-27 steps wheel1 once and each following wheel when any preceding latched pin is effective. No physical window offset is inferred.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Text1 MiB; settings512 KiB; optional trace65536 symbols; complete serialized report4 MiB, independently enforced. Fixed27 printing-bar and six-wheel work per symbol, pure local execution, no automatic retention or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::hagelin::{encode,decode}",
            python_api: python,
            references: &[
                "https://www.cryptomuseum.com/crypto/hagelin/m209/files/m209_tm11-380_19420427.pdf",
                "https://www.jfbouch.fr/crypto/m209/compatibility.html",
                "https://www.cryptomuseum.com/crypto/hagelin/cx52/files/CX52_Desc_3027.pdf",
                "https://www.cryptomuseum.com/crypto/hagelin/cx52/files/19641125_CX52_Usage.pdf",
                "https://www.cryptomuseum.com/pub/files/BW_C52_CX52.pdf",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.hagelin.encode",
        "Encipher with Hagelin",
        "cipher hagelin encode",
        "hagelin_encode",
    ),
    entry(
        "cipher.hagelin.decode",
        "Decipher with Hagelin",
        "cipher hagelin decode",
        "hagelin_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<hagelin::Report>("text"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<hagelin::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<hagelin::Report>("text"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: || c::field::<hagelin::Report>("final_state"),
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<hagelin::Report>("trace"),
        required: true,
    },
];
const fn contract(index: usize, fixture: fn() -> Result<c::Fixture, OperationError>) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: PROFILED[index].descriptor,
        inputs: INPUTS,
        parameters: c::schema::<super::catalog::NoParameters>,
        outputs: OUTPUTS,
        limits: c::Limits {
            max_input_bytes: Some((MAX_INPUT + MAX_PARAMETERS) as u64),
            max_output_bytes: Some(MAX_OUTPUT as u64),
            max_items: Some(MAX_INPUT as u64),
            max_work: None,
            max_memory_bytes: None,
        },
        fixture,
    }
}
const CONTRACTS: &[c::Entry] = &[contract(0, encode_fixture), contract(1, decode_fixture)];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);
pub(super) fn execute(
    op: &str,
    inputs: &Value,
    _parameters: &Value,
) -> Result<Value, OperationError> {
    let descriptor = OPERATIONS.iter().find(|d| d.id == op).ok_or_else(|| {
        OperationError::new(ErrorCode::InvalidParameter, "unknown Hagelin operation")
    })?;
    let op = descriptor.id;
    let text = inputs.get("text").and_then(Value::as_str).ok_or_else(|| {
        error(
            op,
            ErrorCode::InvalidInput,
            "text",
            "prepared ASCII text required",
        )
    })?;
    let settings = inputs.get("settings").ok_or_else(|| {
        error(
            op,
            ErrorCode::InvalidInput,
            "settings",
            "explicit machine settings required",
        )
    })?;
    let p = parse_parameters(&settings.to_string(), op)?;
    let report = if op == "cipher.hagelin.encode" {
        hagelin::encode(text, &p)
    } else {
        hagelin::decode(text, &p)
    }?;
    serde_json::to_value(report).map_err(|_| {
        error(
            op,
            ErrorCode::InvalidInput,
            "report",
            "typed report serialization failed",
        )
    })
}
fn fixture(index: usize) -> Result<c::Fixture, OperationError> {
    // Complete1942 manual check, not a default operational key.
    let v: Value = serde_json::from_str(include_str!("hagelin_vectors.json"))
        .expect("reviewed Hagelin vector");
    let (text, expected) = if index == 0 {
        (&v["text"], &v["expected"])
    } else {
        (&v["expected"], &v["text"])
    };
    let inputs = json!({"text":text,"settings":v["settings"]});
    let outputs = execute(OPERATIONS[index].id, &inputs, &json!({}))?;
    assert_eq!(&outputs["text"], expected);
    Ok(c::Fixture {
        inputs,
        parameters: json!({}),
        outputs,
    })
}
fn encode_fixture() -> Result<c::Fixture, OperationError> {
    fixture(0)
}
fn decode_fixture() -> Result<c::Fixture, OperationError> {
    fixture(1)
}
