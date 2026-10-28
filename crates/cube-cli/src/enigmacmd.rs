//! Local bounded settings transport; simulation and validation belong to Core.
use crate::Input;
use clap::Args;
use cube_core::{
    ciphers::enigma,
    errors::{ErrorCode, OperationError},
};
use std::{error::Error, io::Read, path::PathBuf};

#[derive(Args)]
pub(crate) struct Parameters {
    #[command(flatten)]
    input: Input,
    /// Local JSON settings (<=4096 bytes); profile, rotors, rings, reflector, windows and plugs are required.
    #[arg(long)]
    parameters: PathBuf,
}

pub(crate) fn run(args: Parameters) -> Result<String, Box<dyn Error>> {
    let file = cube_core::forensics::open_regular(&args.parameters)
        .map_err(|e| e.operation(enigma::ID))?;
    let mut document = String::new();
    file.take(enigma::MAX_PARAMETERS as u64 + 1)
        .read_to_string(&mut document)
        .map_err(|e| {
            OperationError::io(e)
                .operation(enigma::ID)
                .parameter("parameters")
        })?;
    let p = enigma::parse_parameters(&document)?;
    let bytes = args.input.read_bounded(enigma::MAX_INPUT, enigma::ID)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "text requires valid UTF-8")
            .operation(enigma::ID)
    })?;
    Ok(serde_json::to_string(&enigma::transform(text, &p)?)?)
}
