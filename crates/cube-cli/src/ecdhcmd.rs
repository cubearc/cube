//! Bounded peer key transport; raw shared secret is never captured.
use clap::{Args, Subcommand};
use cube_core::{asymmetric::ecdh, errors::OperationError};
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
    /// Private JSON with an explicit ECDH profile and private_key_hex
    #[arg(long)]
    parameters_file: PathBuf,
    /// Binary ECDH input, <=65 bytes; omit for stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let Command::Agree(p) = command;
    let mut doc = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(ecdh::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut doc)
        .map_err(OperationError::io)?;
    let s = ecdh::Settings::parse(
        std::str::from_utf8(&doc).map_err(|_| "ECDH settings require UTF-8 JSON")?,
    )?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(p.file).read_bounded(ecdh::MAX_PUBLIC_KEY, ecdh::AGREE)?,
    );
    let output = Zeroizing::new(ecdh::agree(&input, &s)?);
    writer.write_all(&output)?;
    Ok(())
}
