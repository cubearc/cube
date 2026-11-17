//! Secret settings files and binary key data; no unverified stdout writes.
use clap::{Args, Subcommand};
use cube_core::{ciphers::key_wrap as kw, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    Wrap(Parameters),
    Unwrap(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Private UTF-8 JSON with exact profile and hexadecimal KEK, <=1024 bytes
    #[arg(long)]
    parameters_file: PathBuf,
    /// Binary key data or wrapped input; omit for stdin
    #[arg(short = 'f', long, visible_alias = "input")]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (p, inverse) = match command {
        Command::Wrap(p) => (p, false),
        Command::Unwrap(p) => (p, true),
    };
    let mut private = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(kw::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut private)
        .map_err(OperationError::io)?;
    let doc = std::str::from_utf8(&private).map_err(|_| "key-wrap settings require UTF-8 JSON")?;
    let settings = kw::Settings::parse(doc)?;
    let input = Zeroizing::new(
        crate::Input::file_or_stdin(p.file)
            .read_bounded(kw::MAX_WRAPPED, if inverse { kw::UNWRAP } else { kw::WRAP })?,
    );
    let output = Zeroizing::new(if inverse {
        kw::unwrap(&input, &settings)?
    } else {
        kw::wrap(&input, &settings)?
    });
    writer.write_all(&output)?;
    Ok(())
}
