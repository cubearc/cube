use super::*;
use crate::{
    asymmetric::kem_wire::{byte_port, byte_schema},
    operations::{self as o, contracts as c, metadata as m},
};
use serde_json::json;

const fn parameters(choices: &'static [&'static str]) -> [o::Parameter; 1] {
    [o::Parameter {
        name: "profile",
        description: "Required fixed ciphertext format.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices },
    }]
}

const BFV_PARAMETERS: [o::Parameter; 1] = parameters(&[Profile::Bfv.name()]);
const BGV_PARAMETERS: [o::Parameter; 1] = parameters(&[Profile::Bgv.name()]);
const CKKS_PARAMETERS: [o::Parameter; 1] = parameters(&[Profile::Ckks.name()]);
const TFHE_PARAMETERS: [o::Parameter; 1] = parameters(&[Profile::Tfhe.name()]);

const fn row(
    id: &'static str,
    name: &'static str,
    parameters: &'static [o::Parameter],
    command: (&'static str, &'static str),
    api: (&'static str, &'static str),
    reference: &'static [&'static str],
    unary: bool,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: name,
            input_type: if unary { "bytes" } else { "json" },
            output_type: "bytes",
            parameters,
            deterministic: true,
            reversible: Some(unary),
            command: command.0,
            example: command.1,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "homomorphic_ciphertext",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PublicDerivation,
                limitations: "Use ciphertexts already encrypted in the named profile. The caller provides matching keys and levels when preparing the data.",
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: name,
                alphabet: "Tagged fixed-width little-endian ciphertext words.",
                key: "Public ciphertext evaluation with no key input.",
                key_advancement: "One linear gate or ciphertext addition.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Input and output widths are fixed by the profile. Validation rejects noncanonical residues and mismatched format tags.",
            rust_api: api.0,
            python_api: api.1,
            references: reference,
        }),
    }
}

const SEAL: &[&str] = &["https://github.com/microsoft/SEAL/tree/v4.1.2"];
const ADD_API: &str = "cube_core::asymmetric::homomorphic::add";
const PROFILED: [m::Entry; 4] = [
    row(
        BFV_ADD,
        "Add BFV ciphertexts",
        &BFV_PARAMETERS,
        (
            "cipher bfv add",
            "cube cipher bfv add --parameters-file settings.json --left a.bin --right b.bin",
        ),
        (ADD_API, "bfv_add"),
        SEAL,
        false,
    ),
    row(
        BGV_ADD,
        "Add BGV ciphertexts",
        &BGV_PARAMETERS,
        (
            "cipher bgv add",
            "cube cipher bgv add --parameters-file settings.json --left a.bin --right b.bin",
        ),
        (ADD_API, "bgv_add"),
        SEAL,
        false,
    ),
    row(
        CKKS_ADD,
        "Add CKKS ciphertexts",
        &CKKS_PARAMETERS,
        (
            "cipher ckks add",
            "cube cipher ckks add --parameters-file settings.json --left a.bin --right b.bin",
        ),
        (ADD_API, "ckks_add"),
        SEAL,
        false,
    ),
    row(
        TFHE_NOT,
        "Negate a TFHE encrypted Boolean",
        &TFHE_PARAMETERS,
        (
            "cipher tfhe not",
            "cube cipher tfhe not --parameters-file settings.json --file ciphertext.bin",
        ),
        ("cube_core::asymmetric::homomorphic::tfhe_not", "tfhe_not"),
        &["https://github.com/zama-ai/tfhe-rs"],
        true,
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

const fn contract(
    index: usize,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    parameters: fn() -> schemars::Schema,
    fixture: fn() -> Result<c::Fixture, OperationError>,
    profile: Profile,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs,
        outputs,
        parameters,
        limits: c::Limits {
            max_input_bytes: Some((inputs.len() * profile.bytes()) as u64),
            max_output_bytes: Some(profile.bytes() as u64),
            max_items: None,
            max_work: Some(profile.bytes() as u64),
            max_memory_bytes: Some(SCRATCH_BYTES as u64),
        },
        fixture,
    }
}

macro_rules! additive_contract {
    ($index:expr, $profile:expr, $prefix:literal) => {{
        const WIDTH: usize = $profile.bytes();
        contract(
            $index,
            &[
                byte_port("left", byte_schema::<WIDTH, WIDTH>),
                byte_port("right", byte_schema::<WIDTH, WIDTH>),
            ],
            &[byte_port("ciphertext", byte_schema::<WIDTH, WIDTH>)],
            || c::descriptor_parameters(&OPERATIONS[$index]),
            || {
                add_fixture(
                    $profile,
                    include_bytes!(concat!("vectors/", $prefix, "-lhs.bin")),
                    include_bytes!(concat!("vectors/", $prefix, "-rhs.bin")),
                    include_bytes!(concat!("vectors/", $prefix, "-sum.bin")),
                )
            },
            $profile,
        )
    }};
}

const TFHE_WIDTH: usize = Profile::Tfhe.bytes();
const CONTRACTS: &[c::Entry] = &[
    additive_contract!(0, Profile::Bfv, "bfv"),
    additive_contract!(1, Profile::Bgv, "bgv"),
    additive_contract!(2, Profile::Ckks, "ckks"),
    contract(
        3,
        &[byte_port(
            "ciphertext",
            byte_schema::<TFHE_WIDTH, TFHE_WIDTH>,
        )],
        &[byte_port(
            "ciphertext",
            byte_schema::<TFHE_WIDTH, TFHE_WIDTH>,
        )],
        || c::descriptor_parameters(&OPERATIONS[3]),
        tfhe_fixture,
        Profile::Tfhe,
    ),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);

fn add_fixture(
    profile: Profile,
    lhs: &[u8],
    rhs: &[u8],
    sum: &[u8],
) -> Result<c::Fixture, OperationError> {
    let result = add(lhs, rhs, profile)?;
    assert_eq!(result, sum);
    Ok(c::Fixture {
        inputs: json!({"left":c::values::Bytes{data:lhs.to_vec()},"right":c::values::Bytes{data:rhs.to_vec()}}),
        parameters: json!({"profile":profile.name()}),
        outputs: json!({"ciphertext":c::values::Bytes{data:result}}),
    })
}

fn tfhe_fixture() -> Result<c::Fixture, OperationError> {
    let lhs = include_bytes!("vectors/tfhe-input.bin");
    let expected = include_bytes!("vectors/tfhe-not.bin");
    let result = tfhe_not(lhs)?;
    assert_eq!(result, expected);
    Ok(c::Fixture {
        inputs: json!({"ciphertext":c::values::Bytes{data:lhs.to_vec()}}),
        parameters: json!({"profile":Profile::Tfhe.name()}),
        outputs: json!({"ciphertext":c::values::Bytes{data:result}}),
    })
}
