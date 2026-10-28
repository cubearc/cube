//! Explicit private file/stdin transport; no secret argv values or captures.
use clap::Args;
use cube_core::{
    ciphers::stream::{self, Profile},
    errors::{ErrorCode, OperationError},
};
use std::{
    error::Error,
    io::{Read, Write},
    path::PathBuf,
};
use zeroize::Zeroizing;
#[derive(Args)]
#[command(group(clap::ArgGroup::new("stream_destination").required(true).multiple(false).args(["output","stdout"])))]
pub(crate) struct Parameters {
    /// Read message bytes from a regular file; omit to use exact stdin
    #[arg(long, visible_alias = "input")]
    file: Option<PathBuf>,
    /// Read exact raw key/pad bytes from a regular file; never use argv secrets
    #[arg(long)]
    key_file: PathBuf,
    /// Public nonce in hexadecimal; required for stream ciphers, absent for Vernam
    #[arg(long)]
    nonce_hex: Option<String>,
    /// Initial 64-byte block counter; required for stream ciphers, absent for Vernam
    #[arg(long)]
    counter: Option<u64>,
    /// Create a new private output file; existing paths are rejected
    #[arg(long)]
    output: Option<PathBuf>,
    /// Explicitly emit private result bytes to stdout
    #[arg(long)]
    stdout: bool,
}
fn read(path: Option<PathBuf>, limit: usize) -> Result<Vec<u8>, OperationError> {
    let mut bytes = Zeroizing::new(Vec::new());
    let mut reader: Box<dyn Read> = match path {
        Some(p) => Box::new(cube_core::forensics::open_regular(&p)?),
        None => Box::new(std::io::stdin()),
    };
    reader
        .by_ref()
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(OperationError::io)?;
    if bytes.len() > limit {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "operand exceeds documented byte ceiling",
        ));
    }
    Ok(std::mem::take(&mut *bytes))
}
pub(crate) fn run(
    p: Option<Profile>,
    args: Parameters,
    out: &mut impl Write,
) -> Result<(), Box<dyn Error>> {
    let state = match p {
        Some(profile) => {
            let nonce = args
                .nonce_hex
                .ok_or("stream profile requires --nonce-hex")?;
            if nonce.len() != profile.nonce_bytes() * 2 {
                return Err("nonce length does not match profile".into());
            }
            let nonce = cube_core::encoding::hex_decode(&nonce)
                .map_err(|_| "nonce must use exact hexadecimal bytes")?;
            Some((
                nonce,
                args.counter.ok_or("stream profile requires --counter")?,
            ))
        }
        None => {
            if args.nonce_hex.is_some() || args.counter.is_some() {
                return Err("Vernam accepts an exact pad without nonce/counter".into());
            }
            None
        }
    };
    // Open destination exclusively before reading private inputs or transforming.
    let mut destination = args
        .output
        .map(|path| {
            let mut o = std::fs::OpenOptions::new();
            o.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                o.mode(0o600);
            }
            o.open(path)
        })
        .transpose()?;
    let key = Zeroizing::new(read(
        Some(args.key_file),
        if p.is_some() { 32 } else { stream::MAX_INPUT },
    )?);
    let data = Zeroizing::new(read(args.file, stream::MAX_INPUT)?);
    let result = Zeroizing::new(match (p, state) {
        (Some(p), Some((nonce, counter))) => stream::transform(p, &data, &key, &nonce, counter)?,
        _ => stream::vernam(&data, &key)?,
    });
    if let Some(file) = &mut destination {
        file.write_all(&result)?;
        file.sync_all()?;
    } else {
        out.write_all(&result)?;
    }
    out.flush()?;
    Ok(())
}
