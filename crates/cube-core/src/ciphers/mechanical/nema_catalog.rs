//! Versioned typed adapters; the NEMA mechanism lives in nema.rs.
use super::*;
use crate::operations::{self as o, contracts as c, metadata as m};
use serde_json::{Value, json};
const VARIANT: &str = "Swiss NEMA Model45 operational/training profiles with measured A-F contact wiring, moving reflector, distinct drive/red-notch sets, fixed Army blocking arms and explicit lower-row window state.";
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
            example: "cube cipher nema --help",
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "historical_mechanical",
            aliases: &[m::Alias {
                name: "T-D",
                usage: "Swiss Model45 operational/training profiles with required physical lower-row windows.",
            }],
            security: m::Security {
                classification: m::Classification::HistoricalCipher,
                limitations: "Historical interoperability only; no modern confidentiality or authentication. Pins, plaintext and state traces are private.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: VARIANT,
                alphabet: "Prepared ASCII A-Z. No automatic case folding, number shifts, space substitution, grouping, padding or message-key procedure.",
                key: "Required distinct four contact wheels and four paired drive wheels, physically left to right, plus ten lower-row window letters. Operational uses A-F/12,13,14,15,17,18/red22,1; training uses A-D/16,19,20,21/red23,2. Only trace defaults to false.",
                key_advancement: "Before current flows, sample old notch states at window-10 and red right probe atwindow-9, then decrement selected windows. Physical slots1,5,9 always move; slots3/7 and4/8 use the red right blocking control. Post-step rotor contact displacement is window-10 in A-numbered arrays; return through the inverse path and the moving involutive reflector.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Text1 MiB; settings512 KiB; optional trace65536 symbols; complete serialized report4 MiB, independently enforced. Fixed ten-wheel motion and five wired-wheel reflection per symbol, pure local execution, no automatic retention or linear recipe adapter.",
            rust_api: "cube_core::ciphers::mechanical::nema::{encode,decode}",
            python_api: python,
            references: &[
                "https://cryptocellar.org/pubs/nema.pdf",
                "https://www.cryptomuseum.com/crypto/nema/index.htm",
                "https://github.com/rmsk2/rmsk2/blob/0cbcbde313a48f1bd1915b32ae20e0910aff430a/nema_test.cpp",
            ],
        }),
    }
}
const PROFILED: [m::Entry; 2] = [
    entry(
        "cipher.nema.encode",
        "Encipher with NEMA Model45",
        "cipher nema encode",
        "nema_encode",
    ),
    entry(
        "cipher.nema.decode",
        "Decipher with NEMA Model45",
        "cipher nema decode",
        "nema_decode",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);
const INPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<nema::Report>("text"),
        required: true,
    },
    c::Port {
        name: "settings",
        semantic_type: "json",
        schema: c::schema::<nema::Settings>,
        required: true,
    },
];
const OUTPUTS: &[c::Port] = &[
    c::Port {
        name: "text",
        semantic_type: "text",
        schema: || c::field::<nema::Report>("text"),
        required: true,
    },
    c::Port {
        name: "final_state",
        semantic_type: "json",
        schema: c::schema::<nema::State>,
        required: true,
    },
    c::Port {
        name: "trace",
        semantic_type: "json",
        schema: || c::field::<nema::Report>("trace"),
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
        OperationError::new(ErrorCode::InvalidParameter, "unknown NEMA operation")
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
    let report = if op == "cipher.nema.encode" {
        nema::encode(text, &p)
    } else {
        nema::decode(text, &p)
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
    // Complete independently recorded Sullivan simulator check; no default key.
    let v: Value =
        serde_json::from_str(include_str!("nema_vectors.json")).expect("reviewed NEMA vector");
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
