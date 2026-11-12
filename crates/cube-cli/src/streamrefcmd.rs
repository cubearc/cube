//! Private file transport; Core owns stream initialization and authentication.
use clap::{Args, Subcommand};
use cube_core::{
    ciphers::stream::reference::{self, authentication},
    errors::OperationError,
};
use serde_json::json;
use std::{
    error::Error,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Encrypt exact bytes or packed bits with explicit private stream settings
    Encrypt(Parameters),
    /// Decrypt the same exact raw profile; no authenticity verification
    Decrypt(Parameters),
    /// Encrypt with a detached MAC or verify before returning plaintext
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },
}
#[derive(Subcommand)]
pub(crate) enum AuthCommand {
    /// Return JSON ciphertext_hex and tag_hex for the exact authenticated profile
    Encrypt(Parameters),
    /// Verify the detached tag before writing any plaintext bytes
    Decrypt {
        #[command(flatten)]
        parameters: Parameters,
        #[arg(long)]
        tag_file: PathBuf,
    },
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Private JSON settings file, at most4096 bytes; never secret argv values
    #[arg(long)]
    parameters_file: PathBuf,
    /// Input bytes, at most64KiB; packed-bit profiles require exact counts in settings
    #[arg(short = 'f', long, visible_alias = "input")]
    file: Option<PathBuf>,
}
fn read_document(path: &Path) -> Result<Zeroizing<Vec<u8>>, Box<dyn Error>> {
    let mut private = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(path)?
        .take(reference::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut private)
        .map_err(OperationError::io)?;
    Ok(private)
}
fn document(private: &[u8]) -> Result<&str, Box<dyn Error>> {
    std::str::from_utf8(private).map_err(|_| "stream settings require UTF-8 JSON".into())
}
fn input(
    file: Option<PathBuf>,
    operation: &'static str,
) -> Result<Zeroizing<Vec<u8>>, Box<dyn Error>> {
    Ok(Zeroizing::new(
        crate::Input::file_or_stdin(file).read_bounded(reference::MAX_INPUT, operation)?,
    ))
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        Command::Encrypt(p) => raw(p, false, writer),
        Command::Decrypt(p) => raw(p, true, writer),
        Command::Auth { command } => authenticated(command, writer),
    }
}
fn raw(args: Parameters, inverse: bool, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let private = read_document(&args.parameters_file)?;
    let settings = reference::Settings::parse(document(&private)?)?;
    let operation = if inverse {
        reference::DECRYPT
    } else {
        reference::ENCRYPT
    };
    let input = input(args.file, operation)?;
    let output = Zeroizing::new(if inverse {
        reference::decrypt(&input, &settings)?
    } else {
        reference::encrypt(&input, &settings)?
    });
    writer.write_all(&output)?;
    Ok(())
}
fn authenticated(command: AuthCommand, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        AuthCommand::Encrypt(args) => {
            let private = read_document(&args.parameters_file)?;
            let settings = authentication::Settings::parse(document(&private)?)?;
            let data = input(args.file, authentication::ENCRYPT)?;
            let result = authentication::encrypt(&data, &settings)?;
            let ciphertext = Zeroizing::new(result.ciphertext);
            let tag = Zeroizing::new(result.tag);
            serde_json::to_writer(
                &mut *writer,
                &json!({"ciphertext_hex":cube_core::encoding::hex_encode(&ciphertext),"tag_hex":cube_core::encoding::hex_encode(&tag)}),
            )?;
            writer.write_all(b"\n")?;
            Ok(())
        }
        AuthCommand::Decrypt {
            parameters: args,
            tag_file,
        } => {
            let private = read_document(&args.parameters_file)?;
            let settings = authentication::Settings::parse(document(&private)?)?;
            let mut tag = Zeroizing::new(Vec::new());
            cube_core::forensics::open_regular(&tag_file)?
                .take(authentication::MAX_TAG_BYTES as u64 + 1)
                .read_to_end(&mut tag)
                .map_err(OperationError::io)?;
            let data = input(args.file, authentication::DECRYPT)?;
            let plaintext = Zeroizing::new(authentication::decrypt(&data, &tag, &settings)?);
            writer.write_all(&plaintext)?;
            Ok(())
        }
    }
}
