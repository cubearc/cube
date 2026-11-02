//! Versioned typed adapters; the SIGABA CSP889 mechanism lives in sigaba.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "SIGABA CSP889 with qualified ten large and five index rotors, independently reversed banks, complete physical windows, four energized control inputs and post-symbol control/cipher stepping.";
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
            example: "cube cipher sigaba --help",
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
                alphabet: "Prepared uppercase ASCII A-Z contact symbols. Plaintext space is physical contact Z, with no automatic substitution, grouping or message preparation.",
                key: "Required five cipher, five control and five index rotor identities/orientations plus all physical windows. Use each large reference rotor0..9 and index rotor0..4 once. No implicit O reset or message-key procedure. Only trace defaults to false.",
                key_advancement: "Encipher/decipher at old cipher positions. Energize F-I through the old inverse control bank, group outputs into the static index bank, OR the five cipher motor selections, then step. Cipher/control cores move backward; reversed visible letters increase. Fast control always moves; middle carries at old fast O; slow carries only when old fast and middle are O. Outer control and index wheels remain fixed.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Text1 MiB; settings512 KiB; optional trace65536 symbols and independent complete serialized report4 MiB; byte ceiling may be reached before trace cardinality. Fixed bounded machine work; private local execution and named outputs, no automatic capture or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::sigaba::{encode,decode}",
            python_api: python,
            references: &[
                "https://maritime.org/doc/crypto/ecm/sec02.php",
                "https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/sigaba.cpp",
                "https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/sigaba_test.cpp",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.sigaba.encode",
        "Encipher with SIGABA CSP889",
        "cipher sigaba encode",
        "sigaba_encode",
    ),
    entry(
        "cipher.sigaba.decode",
        "Decipher with SIGABA CSP889",
        "cipher sigaba decode",
        "sigaba_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<sigaba::Report>("text"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<sigaba::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<sigaba::Report>("text"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: c::schema::<sigaba::State>,
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<sigaba::Report>("trace"),
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
            "unknown SIGABA CSP889 operation",
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
    let report = if op == "cipher.sigaba.encode" {
        sigaba::encode(text, &p)
    } else {
        sigaba::decode(text, &p)
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
    let v: Value = serde_json::from_str(include_str!("sigaba_vectors.json"))
        .expect("reviewed SIGABA CSP889 vectors");
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
