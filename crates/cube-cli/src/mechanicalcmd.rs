//! Local bounded settings transport. Historical algorithms belong to Core.
use crate::Input;
use clap::{Args, Subcommand};
use cube_core::{
    ciphers::mechanical::{
        self, alberti, disks, hagelin, kryha, lorenz, nema, t52ab, typex, wheatstone,
    },
    errors::{ErrorCode, OperationError},
};
use std::{error::Error, io::Read, path::PathBuf};
#[derive(Subcommand)]
pub(crate) enum Command {
    Encode(Parameters),
    Decode(Parameters),
}
#[derive(Args)]
pub(crate) struct Parameters {
    #[command(flatten)]
    input: Input,
    /// Required local typed machine settings; at most512 KiB.
    #[arg(long)]
    parameters: PathBuf,
}
type Transform<P, R> = fn(&str, &P) -> Result<R, OperationError>;
fn run<P: serde::de::DeserializeOwned, R: serde::Serialize>(
    command: Command,
    encode_id: &'static str,
    decode_id: &'static str,
    encode: Transform<P, R>,
    decipher: Transform<P, R>,
) -> Result<String, Box<dyn Error>> {
    let (args, decode) = match command {
        Command::Encode(p) => (p, false),
        Command::Decode(p) => (p, true),
    };
    let op = if decode { decode_id } else { encode_id };
    let p = settings(&args, op)?;
    let input = args.input.read_bounded(mechanical::MAX_INPUT, op)?;
    let text = std::str::from_utf8(&input).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "text requires valid UTF-8").operation(op)
    })?;
    let result = if decode {
        decipher(text, &p)?
    } else {
        encode(text, &p)?
    };
    Ok(serde_json::to_string(&result)?)
}
fn settings<P: serde::de::DeserializeOwned>(
    args: &Parameters,
    op: &'static str,
) -> Result<P, Box<dyn Error>> {
    let file = cube_core::forensics::open_regular(&args.parameters).map_err(|e| e.operation(op))?;
    let mut document = String::new();
    file.take(mechanical::MAX_PARAMETERS as u64 + 1)
        .read_to_string(&mut document)
        .map_err(|e| OperationError::io(e).operation(op).parameter("parameters"))?;
    Ok(mechanical::parse_parameters(&document, op)?)
}
pub(crate) fn m94(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.m94.encode",
        "cipher.m94.decode",
        disks::m94_encode,
        disks::m94_decode,
    )
}
pub(crate) fn jefferson(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.jefferson.encode",
        "cipher.jefferson.decode",
        disks::jefferson_encode,
        disks::jefferson_decode,
    )
}
pub(crate) fn m138a(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.m138a.encode",
        "cipher.m138a.decode",
        disks::m138a_encode,
        disks::m138a_decode,
    )
}

pub(crate) fn wheatstone(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.wheatstone.encode",
        "cipher.wheatstone.decode",
        wheatstone::encode,
        wheatstone::decode,
    )
}

pub(crate) fn kryha(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.kryha.encode",
        "cipher.kryha.decode",
        kryha::encode,
        kryha::decode,
    )
}

pub(crate) fn alberti(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.alberti.encode",
        "cipher.alberti.decode",
        alberti::encode,
        alberti::decode,
    )
}

type CodeTransform<P, R> = fn(&[u8], &P) -> Result<R, OperationError>;
fn run_codes<P: serde::de::DeserializeOwned, R: serde::Serialize>(
    command: Command,
    encode_id: &'static str,
    decode_id: &'static str,
    encode: CodeTransform<P, R>,
    decipher: CodeTransform<P, R>,
) -> Result<String, Box<dyn Error>> {
    let (args, decode) = match command {
        Command::Encode(p) => (p, false),
        Command::Decode(p) => (p, true),
    };
    let op = if decode { decode_id } else { encode_id };
    let p = settings(&args, op)?;
    let input = args.input.read_bounded(mechanical::MAX_OUTPUT, op)?;
    let document = std::str::from_utf8(&input).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "codes require a UTF-8 JSON array")
            .operation(op)
            .parameter("codes")
    })?;
    let codes = lorenz::parse_codes(document, op)?;
    let report = if decode {
        decipher(&codes, &p)
    } else {
        encode(&codes, &p)
    }?;
    Ok(serde_json::to_string(&report)?)
}
pub(crate) fn lorenz(command: Command) -> Result<String, Box<dyn Error>> {
    run_codes(
        command,
        "cipher.lorenz.encode",
        "cipher.lorenz.decode",
        lorenz::encode,
        lorenz::decode,
    )
}
pub(crate) fn t52ab(command: Command) -> Result<String, Box<dyn Error>> {
    run_codes(
        command,
        "cipher.t52ab.encode",
        "cipher.t52ab.decode",
        t52ab::encode,
        t52ab::decode,
    )
}

pub(crate) fn hagelin(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.hagelin.encode",
        "cipher.hagelin.decode",
        hagelin::encode,
        hagelin::decode,
    )
}

pub(crate) fn nema(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.nema.encode",
        "cipher.nema.decode",
        nema::encode,
        nema::decode,
    )
}

pub(crate) fn typex(command: Command) -> Result<String, Box<dyn Error>> {
    run(
        command,
        "cipher.typex.encode",
        "cipher.typex.decode",
        typex::encode,
        typex::decode,
    )
}
