//! Versioned typed adapters; the Fialka M-125 mechanism lives in fialka.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "Fialka M-125 6K PROTON I full thirty-contact electrical path, complete punched card, ten distinct rotors, explicit rings and post-symbol counter-rotating pin-controlled banks.";
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
            example: "cube cipher fialka --help",
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
                alphabet: "Unpacked integer contacts0..29 in the published thirty-key layout order; no automatic language, figures or printer conversion.",
                key: "Required rotor IDs1..10, ten ring offsets0..29, all thirty punched-card contacts and ten rotor positions0..29. Only trace defaults to false.",
                key_advancement: "Current passes keyboard, punched card, entry disk, fixed3-contact frame, ten rotors and directional reflector; old blocking pins control two opposing post-step chains.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Unpacked contacts1048576; settings512 KiB; optional trace65536 symbols and independent complete serialized report4 MiB; byte ceiling may be reached before trace cardinality. Fixed bounded machine work; private local execution and named outputs, no automatic capture or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::fialka::{encode,decode}",
            python_api: python,
            references: &[
                "https://github.com/CrypToolProject/CrypTool-2/blob/4c8f2bb2794e2263a0fd11bf1425f4badd7fef45/CrypPlugins/Fialka/FialkaCore.cs",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.fialka.encode",
        "Encipher with Fialka M-125",
        "cipher fialka encode",
        "fialka_encode",
    ),
    entry(
        "cipher.fialka.decode",
        "Decipher with Fialka M-125",
        "cipher fialka decode",
        "fialka_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "codes",
        semantic_type: "json",
        schema: || c::field::<fialka::Report>("codes"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<fialka::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "codes",
        semantic_type: "json",
        schema: || c::field::<fialka::Report>("codes"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: c::schema::<fialka::State>,
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<fialka::Report>("trace"),
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
            max_input_bytes: Some((MAX_OUTPUT + MAX_PARAMETERS) as u64),
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
            "unknown Fialka M-125 operation",
        )
    })?;
    let op = descriptor.id;
    let codes: Vec<u8> = serde_json::from_value(
        inputs.get("codes").cloned().unwrap_or(Value::Null),
    )
    .map_err(|_| {
        error(
            op,
            ErrorCode::InvalidInput,
            "codes",
            "expected unpacked integer contacts 0..29",
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
    let report = if op == "cipher.fialka.encode" {
        fialka::encode(&codes, &p)
    } else {
        fialka::decode(&codes, &p)
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
    let v: Value = serde_json::from_str(include_str!("fialka_vectors.json"))
        .expect("reviewed Fialka M-125 vectors");
    let (text, expected) = if index == 0 {
        (&v["codes"], &v["expected"])
    } else {
        (&v["expected"], &v["codes"])
    };
    let mut settings = v["settings"].clone();
    settings["trace"] = json!(false);
    let inputs = json!({"codes":text,"settings":settings});
    let outputs = execute(OPERATIONS[index].id, &inputs, &json!({}))?;
    assert_eq!(&outputs["codes"], expected);
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
