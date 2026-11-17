//! Bounded IES byte/key transport; write output only after successful Core processing.
use clap::{Args, Subcommand};
use cube_core::{asymmetric::ies, errors::OperationError};
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
    /// JSON with an explicit IES suite and optional derivation/encoding contexts; key bytes use a separate file
    #[arg(long)]
    parameters_file: PathBuf,
    /// Suite-specific raw recipient public key65/256 bytes, or private key32 bytes
    #[arg(long)]
    key_file: PathBuf,
    /// Binary IES message/ciphertext; omit for stdin
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
        .take(ies::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut doc)
        .map_err(OperationError::io)?;
    let mut key = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.key_file)?
        .take(ies::MAX_KEY as u64 + 1)
        .read_to_end(&mut key)
        .map_err(OperationError::io)?;
    let doc = std::str::from_utf8(&doc).map_err(|_| "IES settings require UTF-8 JSON")?;
    let input = Zeroizing::new(crate::Input::file_or_stdin(p.file).read_bounded(
        ies::MAX_CIPHERTEXT,
        if decrypt { ies::DECRYPT } else { ies::ENCRYPT },
    )?);
    let output = Zeroizing::new(if decrypt {
        ies::decrypt(&input, &ies::DecryptionSettings::parse(doc, &key)?)?
    } else {
        ies::encrypt(&input, &ies::EncryptionSettings::parse(doc, &key)?)?
    });
    writer.write_all(&output)?;
    Ok(())
}
