use super::*;
use crate::{
    asymmetric::kem_wire::{byte_port, byte_schema},
    operations::{self as o, contracts as c, metadata as m},
};
use serde_json::json;

const fn row(
    id: &'static str,
    name: &'static str,
    parameters: &'static [o::Parameter],
    command: (&'static str, &'static str),
    python_api: &'static str,
    reference: &'static [&'static str],
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: name,
            input_type: "json",
            output_type: "bytes",
            parameters,
            deterministic: true,
            reversible: Some(false),
            command: command.0,
            example: command.1,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "additive_ciphertext",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PublicDerivation,
                limitations: "Works on ciphertexts under one externally validated public modulus and generator. Additive evaluation is deliberately malleable.",
                private_input: false,
                private_output: false,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: name,
                alphabet: "Fixed-width unsigned big-endian integers.",
                key: "A full-width 2048-, 3072- or 4096-bit odd public modulus.",
                key_advancement: "One multiplication in the ciphertext group.",
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "One ciphertext pair. Both residues are checked for group membership.",
            rust_api: "cube_core::asymmetric::additive::add",
            python_api,
            references: reference,
        }),
    }
}

const fn parameters(choices: &'static [&'static str]) -> [o::Parameter; 1] {
    [o::Parameter {
        name: "profile",
        description: "Required generator and extension profile.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices },
    }]
}
const PAILLIER: [o::Parameter; 1] = parameters(&[Profile::Paillier.name()]);
const DAMGARD_JURIK: [o::Parameter; 1] = parameters(&[Profile::DamgardJurik.name()]);

const ENTRIES: [m::Entry; 2] = [
    row(
        PAILLIER_ADD,
        "Add Paillier ciphertexts",
        &PAILLIER,
        (
            "cipher paillier add",
            "cube cipher paillier add --parameters-file settings.json --modulus modulus.bin --left a.bin --right b.bin",
        ),
        "paillier_add",
        &["https://www.cs.tau.ac.il/~fiat/crypt07/papers/Pai99pai.pdf"],
    ),
    row(
        DAMGARD_JURIK_ADD,
        "Add Damgård–Jurik ciphertexts",
        &DAMGARD_JURIK,
        (
            "cipher damgard-jurik add",
            "cube cipher damgard-jurik add --parameters-file settings.json --modulus modulus.bin --left a.bin --right b.bin",
        ),
        "damgard_jurik_add",
        &["https://tidsskrift.dk/brics/article/view/20212"],
    ),
];

pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&ENTRIES);

macro_rules! contract {
    ($index:expr, $profile:expr, $prefix:literal) => {{
        const MIN: usize = $profile.ciphertext_bytes(256);
        const MAX: usize = $profile.ciphertext_bytes(MAX_MODULUS_BYTES);
        const INPUTS: &[c::Port] = &[
            byte_port("left", byte_schema::<MIN, MAX>),
            byte_port("right", byte_schema::<MIN, MAX>),
            byte_port("public_modulus", byte_schema::<256, MAX_MODULUS_BYTES>),
        ];
        const OUTPUTS: &[c::Port] = &[byte_port("ciphertext", byte_schema::<MIN, MAX>)];
        c::Entry {
            descriptor: OPERATIONS[$index],
            operation_version: 1,
            inputs: INPUTS,
            outputs: OUTPUTS,
            parameters: || c::descriptor_parameters(&OPERATIONS[$index]),
            limits: c::Limits {
                max_input_bytes: Some((2 * MAX + MAX_MODULUS_BYTES) as u64),
                max_output_bytes: Some(MAX as u64),
                max_items: Some(1),
                max_work: Some(1),
                max_memory_bytes: Some(SCRATCH_BYTES as u64),
            },
            fixture: || {
                fixture(
                    $profile,
                    include_bytes!(concat!("vectors/", $prefix, "-modulus.bin")),
                    include_bytes!(concat!("vectors/", $prefix, "-lhs.bin")),
                    include_bytes!(concat!("vectors/", $prefix, "-rhs.bin")),
                    include_bytes!(concat!("vectors/", $prefix, "-sum.bin")),
                )
            },
        }
    }};
}

const CONTRACTS: &[c::Entry] = &[
    contract!(0, Profile::Paillier, "paillier"),
    contract!(1, Profile::DamgardJurik, "damgard-jurik"),
];

pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &ENTRIES).with_contracts(CONTRACTS);

fn fixture(
    profile: Profile,
    modulus: &[u8],
    left: &[u8],
    right: &[u8],
    sum: &[u8],
) -> Result<c::Fixture, OperationError> {
    assert_eq!(add(left, right, modulus, profile)?, sum);
    let encode = |data: &[u8]| c::values::Bytes {
        data: data.to_vec(),
    };
    Ok(c::Fixture {
        inputs: json!({"left": encode(left), "right": encode(right), "public_modulus": encode(modulus)}),
        parameters: json!({"profile":profile.name()}),
        outputs: json!({"ciphertext":encode(sum)}),
    })
}
