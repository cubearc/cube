//! Exact RSA bytes with settings in a bounded private file; no partial output.
use clap::{Args, Subcommand};
use cube_core::{asymmetric::rsa, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    Oaep {
        #[command(subcommand)]
        command: Oaep,
    },
    #[command(name = "pkcs1-v1-5")]
    Pkcs1V15 {
        #[command(subcommand)]
        command: Legacy,
    },
}
#[derive(Subcommand)]
pub(crate) enum Oaep {
    Encrypt(Parameters),
    Decrypt(Parameters),
}
#[derive(Subcommand)]
pub(crate) enum Legacy {
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Private UTF-8 JSON with explicit profile, key format and OAEP label
    #[arg(long)]
    parameters_file: PathBuf,
    /// SPKI public or unencrypted PKCS8 private key file, at most65536 bytes
    #[arg(long)]
    key_file: PathBuf,
    /// Binary message/ciphertext; omit for stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (p, operation) = match command {
        Command::Oaep {
            command: Oaep::Encrypt(p),
        } => (p, rsa::ENCRYPT),
        Command::Oaep {
            command: Oaep::Decrypt(p),
        } => (p, rsa::DECRYPT),
        Command::Pkcs1V15 {
            command: Legacy::Decrypt(p),
        } => (p, rsa::LEGACY_DECRYPT),
    };
    let mut doc = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(rsa::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut doc)
        .map_err(OperationError::io)?;
    let doc = std::str::from_utf8(&doc).map_err(|_| "RSA settings require UTF-8 JSON")?;
    let mut key = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.key_file)?
        .take(cube_core::asymmetric::keys::MAX_KEY as u64 + 1)
        .read_to_end(&mut key)
        .map_err(OperationError::io)?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(p.file).read_bounded(rsa::MAX_CIPHERTEXT, operation)?,
    );
    let output = Zeroizing::new(match operation {
        rsa::ENCRYPT => rsa::encrypt(&input, &rsa::EncryptionSettings::parse(doc, &key)?)?,
        rsa::DECRYPT => rsa::decrypt(&input, &rsa::DecryptionSettings::parse(doc, &key)?)?,
        _ => rsa::legacy_decrypt(&input, &rsa::LegacySettings::parse(doc, &key)?)?,
    });
    writer.write_all(&output)?;
    Ok(())
}
