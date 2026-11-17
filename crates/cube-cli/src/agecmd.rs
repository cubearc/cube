//! Bounded binary age transport; authenticate completely before stdout.
use clap::{Args, Subcommand};
use cube_core::{ciphers::age, errors::OperationError};
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
    /// Private exact age profile and one X25519 identity JSON
    #[arg(long)]
    parameters_file: PathBuf,
    /// Binary agev1/X25519 input, <=16MiB; omit for stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let Command::Decrypt(p) = command;
    let mut doc = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(age::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut doc)
        .map_err(OperationError::io)?;
    let s = age::Settings::parse(
        std::str::from_utf8(&doc).map_err(|_| "Age settings require UTF-8 JSON")?,
    )?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(p.file).read_bounded(age::MAX_INPUT, age::DECRYPT)?,
    );
    let output = Zeroizing::new(age::decrypt(&input, &s)?);
    writer.write_all(&output)?;
    Ok(())
}
