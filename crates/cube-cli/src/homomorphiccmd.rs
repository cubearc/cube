//! Bounded ciphertext-file transport for deterministic public evaluation.
use clap::{Args, Subcommand};
use cube_core::asymmetric::homomorphic::{self as h, Profile};
use std::{
    error::Error,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[derive(Subcommand)]
pub(crate) enum AddCommand {
    Add(AddArguments),
}

#[derive(Args)]
pub(crate) struct AddArguments {
    /// JSON containing the explicit ciphertext profile
    #[arg(long)]
    parameters_file: PathBuf,
    /// Complete left ciphertext
    #[arg(long)]
    left: PathBuf,
    /// Complete right ciphertext under the same key
    #[arg(long)]
    right: PathBuf,
}

#[derive(Subcommand)]
pub(crate) enum NotCommand {
    Not(NotArguments),
}

#[derive(Args)]
pub(crate) struct NotArguments {
    /// JSON containing the explicit TFHE profile
    #[arg(long)]
    parameters_file: PathBuf,
    /// Complete input ciphertext, or stdin if omitted
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}

fn read(path: &Path, limit: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut data = Vec::new();
    cube_core::forensics::open_regular(path)?
        .take((limit + 1) as u64)
        .read_to_end(&mut data)?;
    Ok(data)
}

fn configured(profile: Profile, parameters: &Path) -> Result<(), Box<dyn Error>> {
    let data = read(parameters, h::MAX_PARAMETERS)?;
    profile.validate_parameters(std::str::from_utf8(&data)?)?;
    Ok(())
}

pub(crate) fn run_add(
    command: AddCommand,
    profile: Profile,
    writer: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let AddCommand::Add(args) = command;
    configured(profile, &args.parameters_file)?;
    let left = read(&args.left, profile.bytes())?;
    let right = read(&args.right, profile.bytes())?;
    writer.write_all(&h::add(&left, &right, profile)?)?;
    Ok(())
}

pub(crate) fn run_not(command: NotCommand, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let NotCommand::Not(args) = command;
    let profile = Profile::Tfhe;
    configured(profile, &args.parameters_file)?;
    let input = crate::Input::file_or_stdin(args.file)
        .read_bounded(profile.bytes(), profile.operation())?;
    writer.write_all(&h::tfhe_not(&input)?)?;
    Ok(())
}

#[derive(Subcommand)]
pub(crate) enum AdditiveCommand {
    Add(AdditiveArguments),
}

#[derive(Args)]
pub(crate) struct AdditiveArguments {
    #[command(flatten)]
    operands: AddArguments,
    /// Full-width big-endian public modulus
    #[arg(long)]
    modulus: PathBuf,
}

pub(crate) fn run_additive(
    command: AdditiveCommand,
    profile: cube_core::asymmetric::additive::Profile,
    writer: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    use cube_core::asymmetric::additive;
    let AdditiveCommand::Add(args) = command;
    let data = read(&args.operands.parameters_file, additive::MAX_PARAMETERS)?;
    profile.validate_parameters(std::str::from_utf8(&data)?)?;
    let modulus = read(&args.modulus, additive::MAX_MODULUS_BYTES)?;
    let maximum = profile.ciphertext_bytes(additive::MAX_MODULUS_BYTES);
    let left = read(&args.operands.left, maximum)?;
    let right = read(&args.operands.right, maximum)?;
    writer.write_all(&additive::add(&left, &right, &modulus, profile)?)?;
    Ok(())
}
