//! Bounded binary KEM input files and a named Base64 JSON result.
use clap::{Args, Subcommand};
use cube_core::{
    asymmetric::{alternate_kem, kyber, ml_kem},
    encoding::base64_encode,
};
use serde::Serialize;
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;

#[derive(Subcommand)]
pub(crate) enum Command {
    Keygen(Settings),
    Encapsulate {
        #[command(flatten)]
        settings: Settings,
        /// Raw FIPS 203 encapsulation key
        #[arg(long)]
        key_file: PathBuf,
    },
    Decapsulate(Decapsulation),
}

#[derive(Subcommand)]
pub(crate) enum KyberCommand {
    Decapsulate(Decapsulation),
}

pub(crate) use KyberCommand as RawKemCommand;

#[derive(Args)]
pub(crate) struct Decapsulation {
    #[command(flatten)]
    settings: Settings,
    /// Raw expanded decapsulation key
    #[arg(long)]
    key_file: PathBuf,
    /// Raw ciphertext. Reads stdin when omitted.
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}

#[derive(Args)]
pub(crate) struct Settings {
    /// JSON containing the explicit key encapsulation profile
    #[arg(long)]
    parameters_file: PathBuf,
}

impl Settings {
    fn profile(self) -> Result<ml_kem::Profile, Box<dyn Error>> {
        let document = read(self.parameters_file, ml_kem::MAX_PARAMETERS)?;
        Ok(ml_kem::Profile::parse(std::str::from_utf8(&document)?)?)
    }
}

fn read(path: PathBuf, max: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = Vec::new();
    cube_core::forensics::open_regular(&path)?
        .take(max as u64 + 1)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

#[derive(Serialize)]
struct Bytes<'a> {
    base64: &'a str,
}

#[derive(Serialize)]
struct KeyPair<'a> {
    public_key: Bytes<'a>,
    private_key: Bytes<'a>,
}

#[derive(Serialize)]
struct Encapsulation<'a> {
    ciphertext: Bytes<'a>,
    shared_secret: Bytes<'a>,
}

#[derive(Serialize)]
struct SharedSecret<'a> {
    shared_secret: Bytes<'a>,
}

pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    match command {
        Command::Keygen(settings) => {
            let keys = ml_kem::keygen(settings.profile()?)?;
            let public = base64_encode(&keys.public_key);
            let private = Zeroizing::new(base64_encode(&keys.private_key));
            serde_json::to_writer(
                &mut *writer,
                &KeyPair {
                    public_key: Bytes { base64: &public },
                    private_key: Bytes { base64: &private },
                },
            )?;
        }
        Command::Encapsulate { settings, key_file } => {
            let key = read(key_file, ml_kem::MAX_PUBLIC_KEY)?;
            let result = ml_kem::encapsulate(&key, settings.profile()?)?;
            let ct = base64_encode(&result.ciphertext);
            let shared = Zeroizing::new(base64_encode(&result.shared_secret[..]));
            serde_json::to_writer(
                &mut *writer,
                &Encapsulation {
                    ciphertext: Bytes { base64: &ct },
                    shared_secret: Bytes { base64: &shared },
                },
            )?;
        }
        Command::Decapsulate(args) => {
            let profile = args.settings.profile()?;
            let key = Zeroizing::new(read(args.key_file, ml_kem::MAX_PRIVATE_KEY)?);
            let ct = crate::Input::file_or_stdin(args.file)
                .read_bounded(ml_kem::MAX_CIPHERTEXT, ml_kem::DECAPSULATE)?;
            return write_shared(writer, &ml_kem::decapsulate(&ct, &key, profile)?[..]);
        }
    }
    writer.write_all(b"\n")?;
    Ok(())
}

pub(crate) fn run_kyber(
    command: KyberCommand,
    writer: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let KyberCommand::Decapsulate(args) = command;
    let document = read(args.settings.parameters_file, kyber::MAX_PARAMETERS)?;
    let profile = kyber::Profile::parse(std::str::from_utf8(&document)?)?;
    let key = Zeroizing::new(read(args.key_file, kyber::MAX_PRIVATE_KEY)?);
    let ct = crate::Input::file_or_stdin(args.file)
        .read_bounded(kyber::MAX_CIPHERTEXT, kyber::DECAPSULATE)?;
    write_shared(writer, &kyber::decapsulate(&ct, &key, profile)?[..])
}

fn write_shared(writer: &mut impl Write, shared: &[u8]) -> Result<(), Box<dyn Error>> {
    let encoded = Zeroizing::new(base64_encode(shared));
    serde_json::to_writer(
        &mut *writer,
        &SharedSecret {
            shared_secret: Bytes { base64: &encoded },
        },
    )?;
    writer.write_all(b"\n")?;
    Ok(())
}

pub(crate) fn run_alternate(
    command: RawKemCommand,
    profile: alternate_kem::Profile,
    writer: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let RawKemCommand::Decapsulate(args) = command;
    let document = read(args.settings.parameters_file, alternate_kem::MAX_PARAMETERS)?;
    profile.validate_parameters(std::str::from_utf8(&document)?)?;
    let (key_bytes, ciphertext_bytes, _) = profile.widths();
    let key = Zeroizing::new(read(args.key_file, key_bytes)?);
    let ciphertext = crate::Input::file_or_stdin(args.file)
        .read_bounded(ciphertext_bytes, profile.operation())?;
    let shared = alternate_kem::decapsulate(&ciphertext, &key, profile)?;
    write_shared(writer, &shared)
}
