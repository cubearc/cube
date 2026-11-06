//! Private files and byte transport; Core owns XXTEA validation and arithmetic.
use clap::{Args, Subcommand};
use cube_core::{
    ciphers::xxtea::{self, Key},
    errors::OperationError,
};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Encrypt one complete little-endian word sequence; no padding
    Encrypt(Parameters),
    /// Decrypt one complete word sequence; no integrity verification
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Exact raw sixteen-byte private key file
    #[arg(long)]
    key_file: PathBuf,
    /// Whole 32-bit words, at most64KiB; omit to read stdin
    #[arg(short = 'f', long, visible_alias = "input")]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (args, inverse) = match command {
        Command::Encrypt(p) => (p, false),
        Command::Decrypt(p) => (p, true),
    };
    let operation = if inverse {
        xxtea::DECRYPT
    } else {
        xxtea::ENCRYPT
    };
    let mut bytes = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&args.key_file)?
        .take(17)
        .read_to_end(&mut bytes)
        .map_err(OperationError::io)?;
    let key = Key::new(&bytes)?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(args.file).read_bounded(xxtea::MAX_INPUT, operation)?,
    );
    let output = Zeroizing::new(if inverse {
        xxtea::decrypt(&input, &key)?
    } else {
        xxtea::encrypt(&input, &key)?
    });
    writer.write_all(&output)?;
    Ok(())
}
