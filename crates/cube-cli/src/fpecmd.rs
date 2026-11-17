//! Numeric JSON and private settings-file transport; all FPE lives in Core.
use clap::{Args, Subcommand};
use cube_core::{ciphers::fpe, errors::OperationError};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Subcommand)]
pub(crate) enum Command {
    Encrypt(Parameters),
    Decrypt(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    /// Private exact FPE settings JSON; never put keys in argv
    #[arg(long)]
    parameters_file: PathBuf,
    /// Exact JSON array of numeral digits; omit for stdin
    #[arg(short = 'f', long)]
    file: Option<PathBuf>,
}
pub(crate) fn run(command: Command, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
    let (p, inverse) = match command {
        Command::Encrypt(p) => (p, false),
        Command::Decrypt(p) => (p, true),
    };
    let mut doc = Zeroizing::new(Vec::new());
    cube_core::forensics::open_regular(&p.parameters_file)?
        .take(fpe::MAX_PARAMETERS as u64 + 1)
        .read_to_end(&mut doc)
        .map_err(OperationError::io)?;
    let doc = std::str::from_utf8(&doc).map_err(|_| "FPE settings require UTF-8 JSON")?;
    let s = fpe::Settings::parse(doc)?;
    let raw = Zeroizing::new(
        crate::Input::file_or_stdin(p.file).read_bounded(fpe::MAX_DIGITS * 6 + 2, fpe::ENCRYPT)?,
    );
    let input = Zeroizing::new(
        serde_json::from_slice::<Vec<u16>>(&raw)
            .map_err(|_| "FPE input requires an exact JSON digit array")?,
    );
    let output = Zeroizing::new(if inverse {
        fpe::decrypt(&input, &s)?
    } else {
        fpe::encrypt(&input, &s)?
    });
    let result = Zeroizing::new(serde_json::to_vec(&*output)?);
    writer.write_all(&result)?;
    Ok(())
}
