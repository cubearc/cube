//! File transport for the small teaching profiles.
use clap::{Args, Subcommand};
use cube_core::asymmetric::lattice_reference::{self as l, Profile};
use std::{
    error::Error,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

#[derive(Subcommand)]
pub(crate) enum Command {
    Encrypt(Arguments),
    Decrypt(Arguments),
}

#[derive(Args)]
pub(crate) struct Arguments {
    /// JSON containing the exact educational profile
    #[arg(long)]
    parameters_file: PathBuf,
    /// Matching raw key
    #[arg(long)]
    key_file: PathBuf,
    /// Input bytes, or stdin if omitted
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}

fn bounded_file(path: &Path, limit: usize) -> Result<Zeroizing<Vec<u8>>, Box<dyn Error>> {
    let mut bytes = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(path)?
        .take((limit + 1) as u64)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

pub(crate) fn run(
    command: Command,
    profile: Profile,
    writer: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let (arguments, encrypt) = match command {
        Command::Encrypt(args) => (args, true),
        Command::Decrypt(args) => (args, false),
    };
    let parameters = bounded_file(&arguments.parameters_file, l::MAX_PARAMETERS)?;
    profile.validate_parameters(std::str::from_utf8(&parameters)?)?;
    let (plain, public, private, ciphertext) = profile.widths();
    let (input_limit, key_limit) = if encrypt {
        (plain, public)
    } else {
        (ciphertext, private)
    };
    let key = bounded_file(&arguments.key_file, key_limit)?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(arguments.file)
            .read_bounded(input_limit, "educational lattice cipher")?,
    );
    let output = Zeroizing::new(if encrypt {
        l::encrypt(&input, &key, profile)?
    } else {
        l::decrypt(&input, &key, profile)?
    });
    writer.write_all(&output)?;
    Ok(())
}
