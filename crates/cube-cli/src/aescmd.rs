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
    /// RFC3394/5649 wrapping; release key data only after integrity verification
    KeyWrap {
        #[command(subcommand)]
        command: crate::keywrapcmd::Command,
    },
    /// Encrypt independent raw blocks; no secure message-encryption scheme
    Encrypt(Parameters),
    /// Decrypt raw blocks; success does not authenticate or verify the key
    Decrypt(Parameters),
    /// Explicit AES byte, CBC-CTS and storage profiles; no authentication
    Mode {
        #[command(subcommand)]
        command: ModeCommand,
    },
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
#[derive(Subcommand)]
pub(crate) enum ModeCommand {
    Encrypt(ModeParameters),
    Decrypt(ModeParameters),
}
#[derive(Args)]
pub(crate) struct ModeParameters {
    /// Private UTF-8 JSON settings; secret keys must never appear in argv
    #[arg(long)]
    parameters_file: PathBuf,
    /// Exact binary input; omit for stdin
    #[arg(short = 'f', long, visible_alias = "input")]
    file: Option<PathBuf>,
}
fn mode(command: ModeCommand, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    use cube_core::ciphers::aes_modes as modes;
    let (args, inverse) = match command {
        ModeCommand::Encrypt(p) => (p, false),
        ModeCommand::Decrypt(p) => (p, true),
    };
    let mut private = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&args.parameters_file)?
        .take(modes::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut private)?;
    let document =
        std::str::from_utf8(&private).map_err(|_| "AES mode settings require UTF-8 JSON")?;
    let settings = modes::Settings::parse(document)?;
    let operation = if inverse {
        modes::DECRYPT
    } else {
        modes::ENCRYPT
    };
    let data = Zeroizing::new(
        crate::Input::file_or_stdin(args.file).read_bounded(modes::MAX_INPUT + 16, operation)?,
    );
    let output = Zeroizing::new(if inverse {
        modes::decrypt(&data, &settings)?
    } else {
        modes::encrypt(&data, &settings)?
    });
    writer.write_all(&output)?;
    Ok(())
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (args, operation, decrypt) = match command {
        Command::Encrypt(args) => (args, aes::ENCRYPT, false),
        Command::Decrypt(args) => (args, aes::DECRYPT, true),
        Command::Mode { command } => return mode(command, writer),
        Command::KeyWrap { command } => return crate::keywrapcmd::run(command, writer),
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
