//! Bounded SM2 byte/key transport; write output only after successful Core processing.
use clap::{Args, Subcommand};
use cube_core::{asymmetric::sm2, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    Encrypt(Parameters),
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// JSON with an explicit SM2 ciphertext representation; key bytes use a separate file
    #[arg(long)]
    parameters_file: PathBuf,
    /// Uncompressed SEC1 public key65 bytes, or raw private scalar32 bytes
    #[arg(long)]
    key_file: PathBuf,
    /// Binary SM2 message/ciphertext; omit for stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (p, decrypt) = match command {
        Command::Encrypt(p) => (p, false),
        Command::Decrypt(p) => (p, true),
    };
    let mut doc = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(sm2::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut doc)
        .map_err(OperationError::io)?;
    let mut key = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.key_file)?
        .take(sm2::MAX_KEY as u64 + 1)
        .read_to_end(&mut key)
        .map_err(OperationError::io)?;
    let doc = std::str::from_utf8(&doc).map_err(|_| "SM2 settings require UTF-8 JSON")?;
    let input = Zeroizing::new(crate::Input::file_or_stdin(p.file).read_bounded(
        sm2::MAX_CIPHERTEXT,
        if decrypt { sm2::DECRYPT } else { sm2::ENCRYPT },
    )?);
    let output = Zeroizing::new(if decrypt {
        sm2::decrypt(&input, &sm2::DecryptionSettings::parse(doc, &key)?)?
    } else {
        sm2::encrypt(&input, &sm2::EncryptionSettings::parse(doc, &key)?)?
    });
    writer.write_all(&output)?;
    Ok(())
}
