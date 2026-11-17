//! Bounded binary KEM input files and a named Base64 JSON result.
use clap::{Args, Subcommand};
use cube_core::{asymmetric::ml_kem, encoding::base64_encode};
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
    Decapsulate {
        #[command(flatten)]
        settings: Settings,
        /// Raw FIPS 203 expanded decapsulation key
        #[arg(long)]
        key_file: PathBuf,
        /// Raw ciphertext. Reads stdin when omitted.
        #[arg(short = 'f', long)]
        file: Option<PathBuf>,
    },
}

#[derive(Args)]
pub(crate) struct Settings {
    /// JSON containing the explicit FIPS 203 profile
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
        Command::Decapsulate {
            settings,
            key_file,
            file,
        } => {
            let key = Zeroizing::new(read(key_file, ml_kem::MAX_PRIVATE_KEY)?);
            let ct = crate::Input::file_or_stdin(file)
                .read_bounded(ml_kem::MAX_CIPHERTEXT, ml_kem::DECAPSULATE)?;
            let shared = ml_kem::decapsulate(&ct, &key, settings.profile()?)?;
            let encoded = Zeroizing::new(base64_encode(&shared[..]));
            serde_json::to_writer(
                &mut *writer,
                &SharedSecret {
                    shared_secret: Bytes { base64: &encoded },
                },
            )?;
        }
    }
    writer.write_all(b"\n")?;
    Ok(())
}
