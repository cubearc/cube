use super::*;
use crate::{
    asymmetric::kem_wire::{byte_port, byte_schema},
    operations::{self as o, contracts as c, metadata as m},
};
use serde_json::{Value, json};

pub(crate) const MEMORY_RESERVATION: usize = 4 * 1024 * 1024;
const FRODO_PROFILE: Profile = Profile::Frodo640Shake;
const HQC_PROFILE: Profile = Profile::Hqc128February2026;
const MCELIECE_PROFILE: Profile = Profile::McEliece460896f;
const BIKE_PROFILE: Profile = Profile::BikeL1;
const NTRU_KEM_PROFILE: Profile = Profile::NtruHps2048509;
const FRODO_CHOICES: &[&str] = &[FRODO_PROFILE.name()];
const HQC_CHOICES: &[&str] = &[HQC_PROFILE.name()];
const MCELIECE_CHOICES: &[&str] = &[MCELIECE_PROFILE.name()];

const BIKE_CHOICES: &[&str] = &[BIKE_PROFILE.name()];

const NTRU_KEM_CHOICES: &[&str] = &[NTRU_KEM_PROFILE.name()];

const fn row(
    id: &'static str,
    parameters: &'static [o::Parameter],
    name: &'static str,
    reference: &'static [&'static str],
    command: (&'static str, &'static str),
    python_api: &'static str,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: name,
            input_type: "json",
            output_type: "json",
            parameters,
            deterministic: true,
            reversible: Some(false),
            command: command.0,
            example: command.1,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "key_encapsulation",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Exact raw KEM profile. Treat the shared secret as private and use explicit protocol authentication. A correctly sized ciphertext may produce an implicit-rejection secret.",
                private_input: true,
                private_output: true,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: name,
                alphabet: "Raw bytes in the selected profile's encoding.",
                key: "Exact-width private key and ciphertext in separate ports.",
                key_advancement: "One deterministic decapsulation with explicit parameters.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "Bounds are checked before backend parsing. Requires private retention consent in Play.",
            rust_api: "cube_core::asymmetric::alternate_kem::decapsulate",
            python_api,
            references: reference,
        }),
    }
}

const fn parameters(profile: Profile) -> o::Parameter {
    let choices: &'static [&'static str] = match profile {
        Profile::Frodo640Shake => FRODO_CHOICES,
        Profile::Hqc128February2026 => HQC_CHOICES,
        Profile::McEliece460896f => MCELIECE_CHOICES,
        Profile::NtruHps2048509 => NTRU_KEM_CHOICES,
        Profile::BikeL1 => BIKE_CHOICES,
    };
    o::Parameter {
        name: "profile",
        description: "Required exact raw KEM profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices },
    }
}

const FRODO_PARAMETERS: [o::Parameter; 1] = [parameters(FRODO_PROFILE)];
const HQC_PARAMETERS: [o::Parameter; 1] = [parameters(HQC_PROFILE)];
const MCELIECE_PARAMETERS: [o::Parameter; 1] = [parameters(MCELIECE_PROFILE)];

const BIKE_PARAMETERS: [o::Parameter; 1] = [parameters(BIKE_PROFILE)];

const NTRU_KEM_PARAMETERS: [o::Parameter; 1] = [parameters(NTRU_KEM_PROFILE)];

const PROFILED: [m::Entry; 5] = [
    row(
        FRODO,
        &FRODO_PARAMETERS,
        "Decapsulate FrodoKEM-640-SHAKE",
        &["https://frodokem.org/"],
        (
            "cipher frodo-kem decapsulate",
            "cube cipher frodo-kem decapsulate --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        ),
        "frodo_kem_decapsulate",
    ),
    row(
        HQC,
        &HQC_PARAMETERS,
        "Decapsulate HQC-128 February 2026",
        &["https://pqc-hqc.org/"],
        (
            "cipher hqc decapsulate",
            "cube cipher hqc decapsulate --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        ),
        "hqc_decapsulate",
    ),
    row(
        MCELIECE,
        &MCELIECE_PARAMETERS,
        "Decapsulate Classic McEliece 460896f",
        &["https://classic.mceliece.org/mceliece-spec-20221023.pdf"],
        (
            "cipher classic-mceliece decapsulate",
            "cube cipher classic-mceliece decapsulate --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        ),
        "classic_mceliece_decapsulate",
    ),
    row(
        BIKE,
        &BIKE_PARAMETERS,
        "Decapsulate BIKE-L1 Round 4",
        &["https://bikesuite.org/"],
        (
            "cipher bike decapsulate",
            "cube cipher bike decapsulate --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        ),
        "bike_decapsulate",
    ),
    row(
        NTRU_KEM,
        &NTRU_KEM_PARAMETERS,
        "Decapsulate NTRU-HPS-2048-509",
        &["https://github.com/jschanck/ntru/tree/6d96ed37"],
        (
            "cipher ntru-kem decapsulate",
            "cube cipher ntru-kem decapsulate --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        ),
        "ntru_kem_decapsulate",
    ),
];
pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&PROFILED);

const fn contract<const KEY: usize, const CIPHERTEXT: usize, const SECRET: usize>(
    index: usize,
    inputs: &'static [c::Port],
    outputs: &'static [c::Port],
    parameters: fn() -> schemars::Schema,
    fixture: fn() -> Result<c::Fixture, OperationError>,
) -> c::Entry {
    c::Entry {
        operation_version: 1,
        descriptor: OPERATIONS[index],
        inputs,
        parameters,
        outputs,
        limits: c::Limits {
            max_input_bytes: Some((KEY + CIPHERTEXT) as u64),
            max_output_bytes: Some(SECRET as u64),
            max_items: Some(1),
            max_work: Some(1),
            max_memory_bytes: Some(MEMORY_RESERVATION as u64),
        },
        fixture,
    }
}

macro_rules! contract_for {
    ($index:expr, $profile:expr, $file:literal) => {{
        const WIDTHS: (usize, usize, usize) = $profile.widths();
        contract::<{ WIDTHS.0 }, { WIDTHS.1 }, { WIDTHS.2 }>(
            $index,
            &[
                byte_port("ciphertext", byte_schema::<{ WIDTHS.1 }, { WIDTHS.1 }>),
                byte_port("private_key", byte_schema::<{ WIDTHS.0 }, { WIDTHS.0 }>),
            ],
            &[byte_port(
                "shared_secret",
                byte_schema::<{ WIDTHS.2 }, { WIDTHS.2 }>,
            )],
            || c::descriptor_parameters(&OPERATIONS[$index]),
            || fixture($profile, include_str!($file)),
        )
    }};
}

const CONTRACTS: &[c::Entry] = &[
    contract_for!(0, FRODO_PROFILE, "frodo640.json"),
    contract_for!(1, HQC_PROFILE, "hqc128.json"),
    contract_for!(2, MCELIECE_PROFILE, "mceliece460896f.json"),
    contract_for!(3, BIKE_PROFILE, "bike-l1.json"),
    contract_for!(4, NTRU_KEM_PROFILE, "ntruhps2048509.json"),
];
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &PROFILED).with_contracts(CONTRACTS);

fn fixture(profile: Profile, document: &str) -> Result<c::Fixture, OperationError> {
    let case: Value = serde_json::from_str(document).expect("reviewed independent KEM vectors");
    let raw = |name: &str| hex::decode(case[name].as_str().unwrap()).unwrap();
    let ciphertext = raw("ct");
    let private_key = raw("sk");
    let expected = raw("ss");
    let actual = decapsulate(&ciphertext, &private_key, profile)?;
    assert!(
        *actual == expected,
        "{} failed its known answer",
        profile.name()
    );
    Ok(c::Fixture {
        inputs: json!({"ciphertext":c::values::Bytes{data:ciphertext},"private_key":c::values::Bytes{data:private_key}}),
        parameters: json!({"profile":profile.name()}),
        outputs: json!({"shared_secret":c::values::Bytes{data:expected}}),
    })
}
