//! Private file transports and exact byte ports; Core authenticates before output.
use clap::{Args, Subcommand};
use cube_core::{ciphers::modern_aead as aead, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Return JSON ciphertext_hex and detached tag_hex with explicit AAD
    Encrypt(Parameters),
    /// Authenticate completely before writing raw plaintext bytes
    Decrypt {
        #[command(flatten)]
        parameters: Parameters,
        #[arg(long)]
        tag_file: PathBuf,
    },
    /// Deterministic nonce-free SIV with ordered associated-data byte components
    Siv {
        #[command(subcommand)]
        command: SivCommand,
    },
}
#[derive(Args)]
pub(crate) struct Input {
    /// Private UTF-8 JSON settings, at most2048 bytes; no secret argv values
    #[arg(long)]
    parameters_file: PathBuf,
    /// Exact binary input; omit to read stdin
    #[arg(short = 'f', long, visible_alias = "input")]
    file: Option<PathBuf>,
}
#[derive(Args)]
pub(crate) struct Parameters {
    #[command(flatten)]
    input: Input,
    /// Exact associated data; supply an empty file for empty AAD
    #[arg(long)]
    aad_file: PathBuf,
}
#[derive(Args)]
pub(crate) struct SivParameters {
    #[command(flatten)]
    input: Input,
    /// JSON array of ordered byte arrays, <=16MiB transport and <=126 components
    #[arg(long)]
    aad_components_file: PathBuf,
}
#[derive(Subcommand)]
pub(crate) enum SivCommand {
    Encrypt(SivParameters),
    Decrypt {
        #[command(flatten)]
        parameters: SivParameters,
        #[arg(long)]
        tag_file: PathBuf,
    },
}
fn read(path: &Path, maximum: usize) -> Result<Zeroizing<Vec<u8>>, Box<dyn Error>> {
    let mut data = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(path)?
        .take(maximum as u64 + 1)
        .read_to_end(&mut data)
        .map_err(OperationError::io)?;
    if data.len() > maximum {
        return Err("authenticated input file exceeds its transport bound".into());
    }
    Ok(data)
}
fn document(private: &[u8]) -> Result<&str, Box<dyn Error>> {
    std::str::from_utf8(private).map_err(|_| "authenticated settings require UTF-8 JSON".into())
}
fn body(
    input: Option<PathBuf>,
    operation: &'static str,
) -> Result<Zeroizing<Vec<u8>>, Box<dyn Error>> {
    Ok(Zeroizing::new(
        crate::Input::file_or_stdin(input).read_bounded(aead::MAX_INPUT, operation)?,
    ))
}
fn write_detached(result: aead::Detached, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let ciphertext = Zeroizing::new(result.ciphertext);
    let tag = Zeroizing::new(result.tag);
    serde_json::to_writer(
        &mut *writer,
        &serde_json::json!({"ciphertext_hex":cube_core::encoding::hex_encode(&ciphertext),"tag_hex":cube_core::encoding::hex_encode(&tag)}),
    )?;
    writer.write_all(b"\n")?;
    Ok(())
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (p, tag) = match command {
        Command::Encrypt(p) => (p, None),
        Command::Decrypt {
            parameters,
            tag_file,
        } => (parameters, Some(tag_file)),
        Command::Siv { command } => return siv(command, writer),
    };
    let private = read(&p.input.parameters_file, aead::MAX_PARAMETERS)?;
    let s = aead::Settings::parse(document(&private)?)?;
    let aad = read(&p.aad_file, aead::MAX_INPUT)?;
    let data = body(
        p.input.file,
        if tag.is_some() {
            aead::DECRYPT
        } else {
            aead::ENCRYPT
        },
    )?;
    if let Some(path) = tag {
        let tag = read(&path, 32)?;
        let plain = Zeroizing::new(aead::decrypt(&data, &aad, &tag, &s)?);
        writer.write_all(&plain)?;
        Ok(())
    } else {
        write_detached(aead::encrypt(&data, &aad, &s)?, writer)
    }
}
fn siv(command: SivCommand, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (p, tag) = match command {
        SivCommand::Encrypt(p) => (p, None),
        SivCommand::Decrypt {
            parameters,
            tag_file,
        } => (parameters, Some(tag_file)),
    };
    let private = read(&p.input.parameters_file, aead::MAX_PARAMETERS)?;
    let s = aead::SivSettings::parse(document(&private)?)?;
    let components = read(&p.aad_components_file, aead::MAX_INPUT)?;
    let components: Vec<Vec<u8>> = serde_json::from_slice(&components)
        .map_err(|_| "use a bounded JSON array of ordered AAD byte arrays")?;
    let data = body(
        p.input.file,
        if tag.is_some() {
            aead::SIV_DECRYPT
        } else {
            aead::SIV_ENCRYPT
        },
    )?;
    if let Some(path) = tag {
        let tag = read(&path, 16)?;
        let plain = Zeroizing::new(aead::siv_decrypt(&data, &components, &tag, &s)?);
        writer.write_all(&plain)?;
        Ok(())
    } else {
        write_detached(aead::siv_encrypt(&data, &components, &s)?, writer)
    }
}
