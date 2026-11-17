//! Exact raw group bytes and separate bounded key/settings file transport.
use clap::{Args, Subcommand};
use cube_core::{asymmetric::group_encryption as g, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Clone, Copy)]
pub(crate) enum Family {
    ElGamal,
    CramerShoup,
}
#[derive(Subcommand)]
pub(crate) enum Command {
    Encrypt(Parameters),
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// JSON with an exact profile and optional Cramer-Shoup label
    #[arg(long)]
    parameters_file: PathBuf,
    /// Suite-specific fixed-width public or private raw key bytes
    #[arg(long)]
    key_file: PathBuf,
    /// Exactly256-byte group message, or raw512/1024-byte ciphertext; omit for stdin
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
    let mut document = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(g::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut document)
        .map_err(OperationError::io)?;
    let document =
        std::str::from_utf8(&document).map_err(|_| "group settings require UTF-8 JSON")?;
    let mut key = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.key_file)?
        .take(g::MAX_KEY as u64 + 1)
        .read_to_end(&mut key)
        .map_err(OperationError::io)?;
    let operation = match (family, decrypt) {
        (Family::ElGamal, false) => g::ELGAMAL_ENCRYPT,
        (Family::ElGamal, true) => g::ELGAMAL_DECRYPT,
        (Family::CramerShoup, false) => g::CRAMER_SHOUP_ENCRYPT,
        (Family::CramerShoup, true) => g::CRAMER_SHOUP_DECRYPT,
    };
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(p.file).read_bounded(g::MAX_CIPHERTEXT, operation)?,
    );
    let output = Zeroizing::new(match (family, decrypt) {
        (Family::ElGamal, false) => g::elgamal_encrypt(
            &input,
            &g::ElGamalEncryptionSettings::parse(document, &key)?,
        )?,
        (Family::ElGamal, true) => g::elgamal_decrypt(
            &input,
            &g::ElGamalDecryptionSettings::parse(document, &key)?,
        )?,
        (Family::CramerShoup, false) => g::cramer_shoup_encrypt(
            &input,
            &g::CramerShoupEncryptionSettings::parse(document, &key)?,
        )?,
        (Family::CramerShoup, true) => g::cramer_shoup_decrypt(
            &input,
            &g::CramerShoupDecryptionSettings::parse(document, &key)?,
        )?,
    });
    writer.write_all(&output)?;
    Ok(())
}
