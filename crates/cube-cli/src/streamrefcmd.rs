//! Private file transport; Core owns stream initialization and arithmetic.
use clap::{Args, Subcommand};
use cube_core::{ciphers::stream::reference, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Encrypt exact bytes with explicit private stream settings
    Encrypt(Parameters),
    /// Decrypt the same exact profile; no authenticity verification
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Private JSON settings file, at most 4096 bytes; never secret argv values
    #[arg(long)]
    parameters_file: PathBuf,
    /// Input bytes, at most 64 KiB; omit to read stdin
    #[arg(short = 'f', long, visible_alias = "input")]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (args, inverse) = match command {
        Command::Encrypt(p) => (p, false),
        Command::Decrypt(p) => (p, true),
    };
    let operation = if inverse {
        reference::DECRYPT
    } else {
        reference::ENCRYPT
    };
    let mut private = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&args.parameters_file)?
        .take(reference::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut private)
        .map_err(OperationError::io)?;
    let document =
        std::str::from_utf8(&private).map_err(|_| "stream settings require UTF-8 JSON")?;
    let settings = reference::Settings::parse(document)?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(args.file).read_bounded(reference::MAX_INPUT, operation)?,
    );
    let output = Zeroizing::new(if inverse {
        reference::decrypt(&input, &settings)?
    } else {
        reference::encrypt(&input, &settings)?
    });
    writer.write_all(&output)?;
    Ok(())
}
