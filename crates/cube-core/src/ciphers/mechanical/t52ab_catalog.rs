//! Versioned typed adapters; the Siemens T52a/b mechanism lives in t52ab.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "Siemens T52a/b configured logical-contact profile: required complete fixed cam facts and identity, wheel assignments and pluggable double-changeover circuit; XOR before permutation and regular advancement.";
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
            example: "cube cipher t52ab --help",
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "historical_teleprinter",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::HistoricalCipher,
                limitations: "Historical interoperability only; no modern confidentiality or authentication. Pins, plaintext and state traces are private.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: VARIANT,
                alphabet: "Unpacked numeric codes 0..31; impulse 1 is the least significant bit. No ITA2 letters/figures interpretation, framing or audio conversion.",
                key: "Required identified629-contact cam set in ascending wheel-length order, complete wheel assignment and socket pair partitions, and ten initial logical contact indices. No historical factory set or front-window offset is inferred; only trace defaults to false.",
                key_advancement: "Sample all ten current contacts; five select XOR impulses and five control the exact pluggable switch network. Encode XOR then permute; decode inverse-permute then XOR. All ten wheels advance once after each code. Cam1 passes straight and cam0 crosses; no other T52 variant borrows this advancement.",
                block_bits: Some(5),
                output_bits: None,
            },
            resources: &[],
            resource_notes: "At most 1048576 unpacked codes; settings 512 KiB; optional trace 65536 symbols and complete serialized report 4 MiB, independently enforced. Precomputed32 exact switch maps and bounded ten-wheel work per symbol; pure local execution, no automatic retention or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::t52ab::{encode,decode}",
            python_api: python,
            references: &[
                "https://www.cryptomuseum.com/crypto/siemens/t52/files/Geheimzusatz_FSM_T52_OKM_1937.pdf",
                "https://cryptocellar.org/pubs/weierud_sturgeon.pdf",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.t52ab.encode",
        "Encipher with Siemens T52a/b",
        "cipher t52ab encode",
        "t52ab_encode",
    ),
    entry(
        "cipher.t52ab.decode",
        "Decipher with Siemens T52a/b",
        "cipher t52ab decode",
        "t52ab_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "codes",
        semantic_type: "json",
        schema: || c::field::<t52ab::Report>("codes"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<t52ab::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "codes",
        semantic_type: "json",
        schema: || c::field::<t52ab::Report>("codes"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: || c::field::<t52ab::Report>("final_state"),
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<t52ab::Report>("trace"),
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
            "unknown Siemens T52a/b operation",
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
            "expected unpacked five-bit integer codes",
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
    let report = if op == "cipher.t52ab.encode" {
        t52ab::encode(&codes, &p)
    } else {
        t52ab::decode(&codes, &p)
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
    // Hand-laid primary Figure8 switch paths, not a historical factory cam set.
    let v: Value = serde_json::from_str(include_str!("t52ab_vectors.json"))
        .expect("reviewed Siemens T52a/b vector");
    let (codes, expected) = if index == 0 {
        (&v["codes"], &v["expected"])
    } else {
        (&v["expected"], &v["codes"])
    };
    let inputs = json!({"codes":codes,"settings":v["settings"]});
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
