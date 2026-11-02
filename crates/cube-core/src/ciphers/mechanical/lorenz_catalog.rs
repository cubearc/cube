//! Versioned typed adapters; the Lorenz mechanism lives in lorenz.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "Lorenz SZ40/SZ42a/SZ42b five-bit logical-contact profiles with explicit pins, starting indices, limitation histories and KT switch.";
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
            example: "cube cipher lorenz --help",
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
                key: "All 501 binary pins, twelve logical starting contact indices, previous Chi2/Psi1 signals, two previous plaintext fifth impulses and KT switch are required. Only trace defaults to false.",
                key_advancement: "Sample current Chi/Psi contacts, then advance Chi and M61; old M61 controls M37, old M37 or model-specific delayed limitation controls all Psi. Logical indices increment modulo each supplied wheel length; physical window numbering is not inferred.",
                block_bits: Some(5),
                output_bits: None,
            },
            resources: &[],
            resource_notes: "At most 1048576 unpacked codes; settings 512 KiB; optional trace 65536 symbols and complete serialized report 4 MiB, independently enforced. Bounded twelve-wheel work per symbol; pure local execution, no automatic retention or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::lorenz::{encode,decode}",
            python_api: python,
            references: &[
                "https://cryptocellar.org/Lorenz/T663_SZ42_Vol_1.pdf",
                "https://www.vintage-radio.net/forum/attachment.php?attachmentid=320238&d=1760352002",
                "https://www.codesandciphers.org.uk/documents/newman/PAGE335.HTM",
                "https://ep.liu.se/ecp/171/013/ecp2020_171_013.pdf",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.lorenz.encode",
        "Encipher with Lorenz",
        "cipher lorenz encode",
        "lorenz_encode",
    ),
    entry(
        "cipher.lorenz.decode",
        "Decipher with Lorenz",
        "cipher lorenz decode",
        "lorenz_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "codes",
        semantic_type: "json",
        schema: || c::field::<lorenz::Report>("codes"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<lorenz::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "codes",
        semantic_type: "json",
        schema: || c::field::<lorenz::Report>("codes"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: c::schema::<lorenz::State>,
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<lorenz::Report>("trace"),
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
        OperationError::new(ErrorCode::InvalidParameter, "unknown Lorenz operation")
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
    let report = if op == "cipher.lorenz.encode" {
        lorenz::encode(&codes, &p)
    } else {
        lorenz::decode(&codes, &p)
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
    // Full source pin facts and a hand-read logical-contact vector, not a default key.
    let v: Value =
        serde_json::from_str(include_str!("lorenz_vectors.json")).expect("reviewed Lorenz vector");
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
