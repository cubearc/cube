//! Raw key files and binary output only; Core owns block validation and AES.
use clap::{Args, Subcommand};
use cube_core::{ciphers::aes_blocks as aes, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Encrypt independent raw blocks; no secure message-encryption scheme
    Encrypt(Parameters),
    /// Decrypt raw blocks; success does not authenticate or verify the key
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Raw 16/24/32-byte key file; never a command-line key or passphrase
    #[arg(long)]
    key_file: PathBuf,
    /// Exact input bytes, <=16 MiB; omit to read stdin (no text conversion)
    #[arg(short = 'f', long, visible_alias = "input")]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (args, operation, decrypt) = match command {
        Command::Encrypt(args) => (args, aes::ENCRYPT, false),
        Command::Decrypt(args) => (args, aes::DECRYPT, true),
    };
    let mut key = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&args.key_file)
        .map_err(|e| e.operation(operation))?
        .take(33)
        .read_to_end(&mut key)
        .map_err(|e| OperationError::io(e).operation(operation))?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(args.file).read_bounded(aes::MAX_INPUT, operation)?,
    );
    let output = Zeroizing::new(if decrypt {
        aes::decrypt(&input, &key)?
    } else {
        aes::encrypt(&input, &key)?
    });
    writer.write_all(&output)?;
    Ok(())
}
