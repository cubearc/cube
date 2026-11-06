//! Raw key files and bytes only; Core owns every profile and algorithm.
use clap::{Args, Subcommand};
use cube_core::{
    ciphers::block_profiles::{self, Profile, Settings},
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
    /// Encrypt raw reference blocks; no padding or authentication
    Encrypt(Parameters),
    /// Decrypt raw reference blocks; success does not authenticate the key
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Exact full-round profile identifier; no default
    #[arg(long)]
    profile: String,
    /// Raw private key file; no command-line key or passphrase
    #[arg(long)]
    key_file: PathBuf,
    /// RC2 only: required effective key length in bits, 1..1024
    #[arg(long)]
    effective_key_bits: Option<u16>,
    /// Exact whole blocks, at most 1 MiB; omit to read stdin
    #[arg(short = 'f', long, visible_alias = "input")]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (args, operation, decrypting) = match command {
        Command::Encrypt(p) => (p, block_profiles::ENCRYPT, false),
        Command::Decrypt(p) => (p, block_profiles::DECRYPT, true),
    };
    let profile = Profile::parse(&args.profile)?;
    let mut key = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&args.key_file)
        .map_err(|e| e.operation(operation))?
        .take(129)
        .read_to_end(&mut key)
        .map_err(|e| OperationError::io(e).operation(operation))?;
    let settings = Settings::new(profile, &key, args.effective_key_bits)?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(args.file)
            .read_bounded(block_profiles::MAX_INPUT, operation)?,
    );
    let output = Zeroizing::new(if decrypting {
        block_profiles::decrypt(&input, &settings)?
    } else {
        block_profiles::encrypt(&input, &settings)?
    });
    writer.write_all(&output)?;
    Ok(())
}
