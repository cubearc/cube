//! Bounded JSON token/key adapters for explicitly educational Core profiles.
use clap::{Args, Subcommand};
use cube_core::{
    asymmetric::historical_public_key as h,
    errors::{ErrorCode, OperationError},
};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Clone, Copy)]
pub(crate) enum Family {
    Rabin,
    GoldwasserMicali,
    Benaloh,
    MerkleHellman,
}
#[derive(Subcommand)]
pub(crate) enum Command {
    Encrypt(Parameters),
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// JSON reference profile and optional lower work/memory ceilings
    #[arg(long)]
    parameters_file: PathBuf,
    /// Profile-specific JSON key with exact decimal string values, at most64KiB
    #[arg(long)]
    key_file: PathBuf,
    /// UTF8 JSON array of exact string tokens, at most4MiB; omit for stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}
pub(crate) fn run(
    command: Command,
    family: Family,
    writer: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let (p, decrypt) = match command {
        Command::Encrypt(p) => (p, false),
        Command::Decrypt(p) => (p, true),
    };
    let operation = match (family, decrypt) {
        (Family::Rabin, false) => h::RABIN_ENCRYPT,
        (Family::Rabin, true) => h::RABIN_DECRYPT,
        (Family::GoldwasserMicali, false) => h::GOLDWASSER_MICALI_ENCRYPT,
        (Family::GoldwasserMicali, true) => h::GOLDWASSER_MICALI_DECRYPT,
        (Family::Benaloh, false) => h::BENALOH_ENCRYPT,
        (Family::Benaloh, true) => h::BENALOH_DECRYPT,
        (Family::MerkleHellman, false) => h::MERKLE_HELLMAN_ENCRYPT,
        (Family::MerkleHellman, true) => h::MERKLE_HELLMAN_DECRYPT,
    };
    let mut document = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(h::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut document)
        .map_err(OperationError::io)?;
    let document = std::str::from_utf8(&document).map_err(|_| {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "reference settings require UTF8 JSON",
        )
    })?;
    let mut key = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.key_file)?
        .take(h::MAX_KEY_BYTES as u64 + 1)
        .read_to_end(&mut key)
        .map_err(OperationError::io)?;
    let data = Zeroizing::new(
        crate::Input::file_or_stdin(p.file).read_bounded(h::MAX_REPORT_BYTES, operation)?,
    );
    let tokens = Zeroizing::new(serde_json::from_slice::<Vec<String>>(&data).map_err(|_| {
        OperationError::new(
            ErrorCode::InvalidInput,
            "reference input requires a JSON array of string tokens",
        )
    })?);
    let output = match (family, decrypt) {
        (Family::Rabin, false) => {
            serde_json::json!({"tokens":h::rabin::encrypt(&tokens,&h::rabin::EncryptionSettings::parse(document,&key)?)?})
        }
        (Family::Rabin, true) => {
            serde_json::json!({"candidates":h::rabin::decrypt(&tokens,&h::rabin::DecryptionSettings::parse(document,&key)?)?})
        }
        (Family::GoldwasserMicali, false) => {
            serde_json::json!({"tokens":h::goldwasser_micali::encrypt(&tokens,&h::goldwasser_micali::EncryptionSettings::parse(document,&key)?)?})
        }
        (Family::GoldwasserMicali, true) => {
            serde_json::json!({"tokens":h::goldwasser_micali::decrypt(&tokens,&h::goldwasser_micali::DecryptionSettings::parse(document,&key)?)?})
        }
        (Family::Benaloh, false) => {
            serde_json::json!({"tokens":h::benaloh::encrypt(&tokens,&h::benaloh::EncryptionSettings::parse(document,&key)?)?})
        }
        (Family::Benaloh, true) => {
            serde_json::json!({"tokens":h::benaloh::decrypt(&tokens,&h::benaloh::DecryptionSettings::parse(document,&key)?)?})
        }
        (Family::MerkleHellman, false) => {
            serde_json::json!({"tokens":h::merkle_hellman::encrypt(&tokens,&h::merkle_hellman::EncryptionSettings::parse(document,&key)?)?})
        }
        (Family::MerkleHellman, true) => {
            serde_json::json!({"tokens":h::merkle_hellman::decrypt(&tokens,&h::merkle_hellman::DecryptionSettings::parse(document,&key)?)?})
        }
    };
    // CLI output is the typed result array; write only after Core succeeds.
    let port = if matches!(family, Family::Rabin) && decrypt {
        "candidates"
    } else {
        "tokens"
    };
    serde_json::to_writer(&mut *writer, &output[port])?;
    writer.write_all(b"\n")?;
    Ok(())
}
