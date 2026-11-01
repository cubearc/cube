//! Bounded caller file/text transport only. Algorithms and validation are Core-owned.
use crate::Input;
use clap::Args;
use cube_core::{
    ciphers::{book, poly},
    errors::{ErrorCode, OperationError},
    symbols::tokens,
    text::keyboard::coordinates as keyboard,
};
use std::{
    error::Error,
    io::Read,
    path::{Path, PathBuf},
};
#[derive(Args)]
pub(crate) struct Parameters {
    #[command(flatten)]
    input: Input,
    /// Exact typed local JSON, <=2MiB; no duplicate/unknown fields.
    #[arg(long)]
    parameters: PathBuf,
}
#[derive(Args)]
pub(crate) struct Reversible {
    #[command(flatten)]
    parameters: Parameters,
    #[arg(long)]
    decode: bool,
}
#[derive(Args)]
pub(crate) struct Book {
    #[command(flatten)]
    parameters: Parameters,
    /// Exact caller source, <=1MiB, with original UTF-8 bytes and line endings.
    #[arg(long)]
    source: PathBuf,
}
pub(crate) fn solresol(args: Parameters, encode: bool) -> Result<String, Box<dyn Error>> {
    use cube_core::symbols::solresol as s;
    let op = if encode {
        "symbol.solresol.encode"
    } else {
        "symbol.solresol.decode"
    };
    let (text, document) = transport(args, op)?;
    let p: s::Parameters = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let input: Vec<String> = poly::parse_parameters(&text).map_err(|mut e| {
        if e.code == ErrorCode::InvalidParameter {
            e.code = ErrorCode::InvalidInput;
        }
        e.operation(op).parameter("input")
    })?;
    Ok(if encode {
        serde_json::to_string(&s::encode(&input, &p)?)?
    } else {
        serde_json::to_string(&s::decode(&input, &p)?)?
    })
}
fn file(
    path: &Path,
    max: usize,
    op: &'static str,
    field: &'static str,
) -> Result<String, Box<dyn Error>> {
    let file =
        cube_core::forensics::open_regular(path).map_err(|e| e.operation(op).parameter(field))?;
    let mut bytes = Vec::new();
    file.take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| OperationError::io(e).operation(op).parameter(field))?;
    if bytes.len() > max {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "caller file exceeds its byte budget",
        )
        .operation(op)
        .parameter(field)
        .into());
    }
    String::from_utf8(bytes).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "caller file requires valid UTF-8")
            .operation(op)
            .parameter(field)
            .into()
    })
}
fn transport(p: Parameters, op: &'static str) -> Result<(String, String), Box<dyn Error>> {
    let document = file(&p.parameters, poly::MAX_PARAMETERS, op, "parameters")?;
    let bytes = p.input.read_bounded(1048576, op)?;
    let text = String::from_utf8(bytes).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "input requires valid UTF-8").operation(op)
    })?;
    Ok((text, document))
}
pub(crate) fn book_encode(p: Book) -> Result<String, Box<dyn Error>> {
    let op = "cipher.book.encode";
    let source = file(&p.source, 1048576, op, "source")?;
    let (text, document) = transport(p.parameters, op)?;
    let options: book::EncodeOptions =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&book::encode(
        &source, &text, &options,
    )?)?)
}
pub(crate) fn keyboard_coordinates(p: Reversible) -> Result<String, Box<dyn Error>> {
    let op = if p.decode {
        "text.keyboard.coordinates.decode"
    } else {
        "text.keyboard.coordinates.encode"
    };
    let decode = p.decode;
    let (text, document) = transport(p.parameters, op)?;
    let options: keyboard::Options =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    if decode {
        let coordinates: Vec<keyboard::Coordinate> =
            poly::parse_parameters(&text).map_err(|e| e.operation(op))?;
        Ok(serde_json::to_string(&keyboard::decode(
            &coordinates,
            &options,
        )?)?)
    } else {
        Ok(serde_json::to_string(&keyboard::encode(&text, &options)?)?)
    }
}
pub(crate) fn keyboard_shift(p: Reversible) -> Result<String, Box<dyn Error>> {
    let op = if p.decode {
        "text.keyboard.shift.decode"
    } else {
        "text.keyboard.shift.encode"
    };
    let decode = p.decode;
    let (text, document) = transport(p.parameters, op)?;
    let options: keyboard::Shift =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&keyboard::shift(
        &text, &options, decode,
    )?)?)
}
pub(crate) fn tokens(p: Parameters, encode: bool) -> Result<String, Box<dyn Error>> {
    let op = if encode {
        "symbol.tokens.encode"
    } else {
        "symbol.tokens.decode"
    };
    let (text, document) = transport(p, op)?;
    let options: tokens::Parameters =
        poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(serde_json::to_string(&tokens::transform(
        &text, &options, encode,
    )?)?)
}
