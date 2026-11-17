//! Explicit legacy file transport, with no unauthenticated partial stdout.
use clap::{Args, Subcommand};
use cube_core::{ciphers::openssl, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Private exact profile, password bytes and explicit PBKDF2 cost JSON
    #[arg(long)]
    parameters_file: PathBuf,
    /// Binary Salted__ input, <=16MiB; omit for stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let Command::Decrypt(p) = command;
    let mut doc = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(openssl::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut doc)
        .map_err(OperationError::io)?;
    let s = openssl::Settings::parse(
        std::str::from_utf8(&doc).map_err(|_| "OpenSSL settings require UTF-8 JSON")?,
    )?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(p.file).read_bounded(openssl::MAX_INPUT, openssl::DECRYPT)?,
    );
    let output = Zeroizing::new(openssl::decrypt(&input, &s)?);
    writer.write_all(&output)?;
    Ok(())
}
