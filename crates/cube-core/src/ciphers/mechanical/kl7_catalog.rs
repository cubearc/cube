//! Versioned typed adapters; the KL7 mechanism lives in kl7.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "KL7 Rijmenants reference rotor/notch set with eight36-contact rotors, fixed fourth rotor, seven independent notch rings, explicit raw core state, old-notch post-stepping and bounded electrical re-entry.";
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
            example: "cube cipher kl7 --help",
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "historical_mechanical",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::HistoricalCipher,
                limitations: "Historical interoperability only; no modern confidentiality or authentication. Wheel settings, plaintext and state traces are private.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: VARIANT,
                alphabet: "Prepared uppercase ASCII A-Z keyboard contact symbols; J denotes the figure-shift contact and Z the space contact. No automatic shifts, spaces, grouping, padding or preparatory motion.",
                key: "Required eight distinct reference rotors A-M, seven distinct notch rings1..11, seven raw notch offsets0..35 and all eight raw core positions0..35. The fourth rotor stays fixed. Alphabet ring settings are represented in the raw core displacements. No implicit preparatory cycle. Only trace defaults to false.",
                key_advancement: "Current enters the inverse fixed base plate, passes through all eight rotors and repeats on the ten fixed feedback contacts, at most11 passes. Decode reverses every pass. Then sample all old notches at core position plus notch offset plus10 and simultaneously increment selected moving rotors. Rotor4 remains fixed.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Text1 MiB; settings512 KiB; optional trace65536 symbols and independent complete serialized report4 MiB; byte ceiling may be reached before trace cardinality. Fixed bounded machine work; private local execution and named outputs, no automatic capture or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::kl7::{encode,decode}",
            python_api: python,
            references: &[
                "https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/kl7.cpp",
                "https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/kl7_test.cpp",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.kl7.encode",
        "Encipher with KL7",
        "cipher kl7 encode",
        "kl7_encode",
    ),
    entry(
        "cipher.kl7.decode",
        "Decipher with KL7",
        "cipher kl7 decode",
        "kl7_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<kl7::Report>("text"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<kl7::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<kl7::Report>("text"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: c::schema::<kl7::State>,
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<kl7::Report>("trace"),
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
    let descriptor = OPERATIONS
        .iter()
        .find(|d| d.id == op)
        .ok_or_else(|| OperationError::new(ErrorCode::InvalidParameter, "unknown KL7 operation"))?;
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
    let report = if op == "cipher.kl7.encode" {
        kl7::encode(text, &p)
    } else {
        kl7::decode(text, &p)
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
    // Configured simulator interoperability check; never a default or alleged wartime key.
    let v: Value =
        serde_json::from_str(include_str!("kl7_vectors.json")).expect("reviewed KL7 vectors");
    let (text, expected) = if index == 0 {
        (&v["text"], &v["expected"])
    } else {
        (&v["expected"], &v["text"])
    };
    let mut settings = v["settings"].clone();
    settings["trace"] = json!(false);
    let inputs = json!({"text":text,"settings":settings});
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
