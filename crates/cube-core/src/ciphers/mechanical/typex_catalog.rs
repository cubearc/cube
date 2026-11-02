//! Versioned typed adapters; the Typex mechanism lives in typex.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "Configured Typex Mark22 with three moving drums, two stators, reversible cores, arbitrary entry/plug permutation, paired reflector, explicit wiring/notches and physical window/ring settings.";
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
            example: "cube cipher typex --help",
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "historical_mechanical",
            aliases: &[m::Alias {
                name: "Type X",
                usage: "Configured Mark22 raw contact symbols; complete caller wiring required.",
            }],
            security: m::Security {
                classification: m::Classification::HistoricalCipher,
                limitations: "Historical interoperability only; no modern confidentiality or authentication. Wheel settings, plaintext and state traces are private.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: VARIANT,
                alphabet: "Prepared ASCII A-Z contact symbols. No automatic case folding, keyboard figure/letter shifts, space substitution, grouping, padding or message-key procedure.",
                key: "Required wheel-set identity, five distinct named core wirings/orientations physically left to right, five rings/windows, middle/fast turnover windows, entry wiring, arbitrary plugboard and paired reflector. No historical wiring set or daily key is inferred. Only trace defaults to false.",
                key_advancement: "Sample the old middle and fast casing notch windows, then step before current flows: fast always, middle if either is active, slow if middle is active. Stators remain fixed. Core reversal is -W^-1(-x), while casing notches remain unchanged. Electrical displacement is window minus ring.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Text1 MiB; settings512 KiB; optional trace65536 symbols; complete serialized report4 MiB. Fixed five-core reflected path, pure local execution, private named outputs, no automatic retention or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::typex::{encode,decode}",
            python_api: python,
            references: &[
                "https://www.cryptomuseum.com/crypto/uk/typex/files/Typex_Manual_IB_II_III_VI.pdf",
                "https://www.cryptomuseum.com/crypto/uk/typex/index.htm",
                "https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/typex_test.cpp",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.typex.encode",
        "Encipher with configured Typex Mark22",
        "cipher typex encode",
        "typex_encode",
    ),
    entry(
        "cipher.typex.decode",
        "Decipher with configured Typex Mark22",
        "cipher typex decode",
        "typex_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<typex::Report>("text"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<typex::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<typex::Report>("text"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: c::schema::<typex::State>,
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<typex::Report>("trace"),
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
        OperationError::new(ErrorCode::InvalidParameter, "unknown Typex operation")
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
    let report = if op == "cipher.typex.encode" {
        typex::encode(text, &p)
    } else {
        typex::decode(text, &p)
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
    let vectors: Vec<Value> =
        serde_json::from_str(include_str!("typex_vectors.json")).expect("reviewed Typex vectors");
    let v = &vectors[0];
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
