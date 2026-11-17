//! Bounded peer key transport; raw shared secret is never captured.
use clap::{Args, Subcommand};
use cube_core::{asymmetric::dh, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    Agree(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Private JSON with an explicit DH profile; the private exponent is a separate file
    #[arg(long)]
    parameters_file: PathBuf,
    /// Fixed-width big-endian private exponent, <=64 bytes
    #[arg(long)]
    key_file: PathBuf,
    /// Binary DH input, <=1024 bytes; omit for stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let Command::Agree(p) = command;
    let mut doc = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(dh::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut doc)
        .map_err(OperationError::io)?;
    let mut key = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.key_file)?
        .take(dh::MAX_PRIVATE_KEY as u64 + 1)
        .read_to_end(&mut key)
        .map_err(OperationError::io)?;
    let s = dh::Settings::parse(
        std::str::from_utf8(&doc).map_err(|_| "DH settings require UTF-8 JSON")?,
        &key,
    )?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(p.file).read_bounded(dh::MAX_PUBLIC_KEY, dh::AGREE)?,
    );
    let output = Zeroizing::new(dh::agree(&input, &s)?);
    writer.write_all(&output)?;
    Ok(())
}
