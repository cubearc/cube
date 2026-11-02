//! Versioned typed adapters; the PURPLE Type B mechanism lives in purple.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "PURPLE Type B with qualified twenty-five-position sixes and twenties switches, complete plugboard and physical switch state, latched post-symbol movement.";
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
            example: "cube cipher purple --help",
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
                alphabet: "Prepared uppercase ASCII A-Z; no grouping, garble handling or automatic message preparation.",
                key: "Required complete alphabet, distinct fast and middle switch identities 1..3 and four positions 0..24. Only trace defaults to false.",
                key_advancement: "Read old sixes and middle positions; step sixes every symbol and exactly one twenties: slow at sixes23/middle24, middle at sixes24, otherwise fast.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Text1 MiB; settings512 KiB; optional trace65536 symbols and independent complete serialized report4 MiB; byte ceiling may be reached before trace cardinality. Fixed bounded machine work; private local execution and named outputs, no automatic capture or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::purple::{encode,decode}",
            python_api: python,
            references: &[
                "https://github.com/gremmie/purple/blob/0647dfe89719bf8ab1c0badbb99936d04eaa4499/purple/machine.py",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.purple.encode",
        "Encipher with PURPLE Type B",
        "cipher purple encode",
        "purple_encode",
    ),
    entry(
        "cipher.purple.decode",
        "Decipher with PURPLE Type B",
        "cipher purple decode",
        "purple_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<purple::Report>("text"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<purple::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<purple::Report>("text"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: c::schema::<purple::State>,
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<purple::Report>("trace"),
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
        OperationError::new(
            ErrorCode::InvalidParameter,
            "unknown PURPLE Type B operation",
        )
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
    let report = if op == "cipher.purple.encode" {
        purple::encode(text, &p)
    } else {
        purple::decode(text, &p)
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
    // Independent published reference check; all key state remains explicit.
    let v: Value = serde_json::from_str(include_str!("purple_vectors.json"))
        .expect("reviewed PURPLE Type B vectors");
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
