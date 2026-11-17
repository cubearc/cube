use super::*;
use crate::{
    asymmetric::kem_wire::{byte_port, byte_schema},
    operations::{self as o, contracts as c, metadata as m},
};
use serde::Deserialize;
use serde_json::json;

const fn parameters(choices: &'static [&'static str]) -> [o::Parameter; 1] {
    [o::Parameter {
        name: "profile",
        description: "Required educational parameter set.",
        required: true,
        default: None,
        constraints: o::ParameterType::Text { choices },
    }]
}

const NTRU: [o::Parameter; 1] = parameters(&[Profile::NtruEncrypt.name()]);
const REGEV: [o::Parameter; 1] = parameters(&[Profile::Regev.name()]);

const fn row(
    id: &'static str,
    name: &'static str,
    parameters: &'static [o::Parameter],
    command: (&'static str, &'static str),
    python_api: &'static str,
    reference: &'static [&'static str],
    encrypt: bool,
) -> m::Entry {
    m::Entry {
        descriptor: o::Descriptor {
            id,
            category: "ciphers",
            description: name,
            input_type: "json",
            output_type: "bytes",
            parameters,
            deterministic: !encrypt,
            reversible: Some(true),
            command: command.0,
            example: command.1,
        },
        metadata: Some(m::Metadata {
            schema_version: 1,
            canonical_name: name,
            family: "educational_lattice_encryption",
            aliases: &[],
            security: m::Security {
                classification: m::Classification::PrivateMaterial,
                limitations: "Teaching parameters with no security or authentication. Use synthetic inputs and matching demonstration keys.",
                private_input: true,
                private_output: !encrypt,
                sensitive_parameters: &[],
            },
            profile: m::Profile {
                variant: name,
                alphabet: "Unsigned one-byte residues, fixed widths and ordering.",
                key: "Supply the matching public or private demonstration key.",
                key_advancement: if encrypt {
                    "Fresh bounded system randomness."
                } else {
                    "One deterministic recovery."
                },
                block_bits: None,
                output_bits: None,
            },
            resources: &[],
            resource_notes: "One message and one key at the fixed profile widths.",
            rust_api: if encrypt {
                "cube_core::asymmetric::lattice_reference::encrypt"
            } else {
                "cube_core::asymmetric::lattice_reference::decrypt"
            },
            python_api,
            references: reference,
        }),
    }
}

const NTRU_SOURCE: &[&str] = &["https://www.ntru.org/f/hps98.pdf"];
const REGEV_SOURCE: &[&str] = &["https://cims.nyu.edu/~regev/papers/qcrypto.pdf"];
const ENTRIES: [m::Entry; 4] = [
    row(
        NTRU_ENCRYPT,
        "Encrypt a polynomial with educational NTRUEncrypt",
        &NTRU,
        (
            "cipher ntru-encrypt encrypt",
            "cube cipher ntru-encrypt encrypt --parameters-file settings.json --key-file public.bin --file message.bin",
        ),
        "ntru_encrypt_encrypt",
        NTRU_SOURCE,
        true,
    ),
    row(
        NTRU_DECRYPT,
        "Decrypt a polynomial with educational NTRUEncrypt",
        &NTRU,
        (
            "cipher ntru-encrypt decrypt",
            "cube cipher ntru-encrypt decrypt --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        ),
        "ntru_encrypt_decrypt",
        NTRU_SOURCE,
        false,
    ),
    row(
        REGEV_ENCRYPT,
        "Encrypt a bit with educational Regev PKE",
        &REGEV,
        (
            "cipher regev encrypt",
            "cube cipher regev encrypt --parameters-file settings.json --key-file public.bin --file bit.bin",
        ),
        "regev_encrypt",
        REGEV_SOURCE,
        true,
    ),
    row(
        REGEV_DECRYPT,
        "Decrypt a bit with educational Regev PKE",
        &REGEV,
        (
            "cipher regev decrypt",
            "cube cipher regev decrypt --parameters-file settings.json --key-file private.bin --file ciphertext.bin",
        ),
        "regev_decrypt",
        REGEV_SOURCE,
        false,
    ),
];

pub const OPERATIONS: &[o::Descriptor] = &m::descriptors(&ENTRIES);

macro_rules! contract {
    ($index:expr, $profile:expr, $encrypt:expr, $file:literal) => {{
        const WIDTHS: (usize, usize, usize, usize) = $profile.widths();
        const INPUT: usize = if $encrypt { WIDTHS.0 } else { WIDTHS.3 };
        const KEY: usize = if $encrypt { WIDTHS.1 } else { WIDTHS.2 };
        const OUTPUT: usize = if $encrypt { WIDTHS.3 } else { WIDTHS.0 };
        const INPUTS: &[c::Port] = &[
            byte_port("input", byte_schema::<INPUT, INPUT>),
            byte_port(
                if $encrypt {
                    "public_key"
                } else {
                    "private_key"
                },
                byte_schema::<KEY, KEY>,
            ),
        ];
        const OUTPUTS: &[c::Port] = &[byte_port("output", byte_schema::<OUTPUT, OUTPUT>)];
        c::Entry {
            descriptor: OPERATIONS[$index],
            operation_version: 1,
            inputs: INPUTS,
            outputs: OUTPUTS,
            parameters: || c::descriptor_parameters(&OPERATIONS[$index]),
            limits: c::Limits {
                max_input_bytes: Some((INPUT + KEY) as u64),
                max_output_bytes: Some(OUTPUT as u64),
                max_items: Some(1),
                max_work: Some(1),
                max_memory_bytes: Some(SCRATCH_BYTES as u64),
            },
            fixture: || fixture($profile, $encrypt, include_str!($file)),
        }
    }};
}

const CONTRACTS: &[c::Entry] = &[
    contract!(0, Profile::NtruEncrypt, true, "ntru-encrypt.json"),
    contract!(1, Profile::NtruEncrypt, false, "ntru-encrypt.json"),
    contract!(2, Profile::Regev, true, "regev.json"),
    contract!(3, Profile::Regev, false, "regev.json"),
];

pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &ENTRIES).with_contracts(CONTRACTS);

#[derive(Deserialize)]
struct Vector {
    plaintext: Vec<u8>,
    public_key: Vec<u8>,
    private_key: Vec<u8>,
    coins: [u8; 2],
    ciphertext: Vec<u8>,
}

fn fixture(profile: Profile, encrypting: bool, raw: &str) -> Result<c::Fixture, OperationError> {
    let vector: Vector = serde_json::from_str(raw).expect("independent lattice corpus");
    let encode = |data| c::values::Bytes { data };
    let (inputs, output) = if encrypting {
        let encrypted = match profile {
            Profile::NtruEncrypt => ntru::encrypt(
                &vector.plaintext,
                &vector.public_key,
                vector.coins.map(usize::from),
            ),
            Profile::Regev => regev::encrypt(&vector.plaintext, &vector.public_key, vector.coins),
        }?;
        assert_eq!(encrypted, vector.ciphertext);
        (
            json!({"input":encode(vector.plaintext),"public_key":encode(vector.public_key)}),
            encrypted,
        )
    } else {
        let decrypted = decrypt(&vector.ciphertext, &vector.private_key, profile)?;
        assert_eq!(decrypted, vector.plaintext);
        (
            json!({"input":encode(vector.ciphertext),"private_key":encode(vector.private_key)}),
            decrypted,
        )
    };
    Ok(c::Fixture {
        inputs,
        parameters: json!({"profile":profile.name()}),
        outputs: json!({"output":encode(output)}),
    })
}
