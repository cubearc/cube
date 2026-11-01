//! Local transport adapters for explicit historical profiles.
use crate::Input;
use clap::Args;
use cube_core::ciphers::mono::historical as h;
use serde_json::{Value, json};
use std::{error::Error, io::Read, path::PathBuf};
fn named_inputs<T: serde::de::DeserializeOwned>(
    document: &str,
    op: &'static str,
) -> Result<T, cube_core::errors::OperationError> {
    cube_core::ciphers::poly::parse_parameters(document).map_err(|mut e| {
        if e.code == cube_core::errors::ErrorCode::InvalidParameter {
            e.code = cube_core::errors::ErrorCode::InvalidInput;
        }
        e.operation(op).parameter("inputs")
    })
}
pub(crate) fn key_phrase(args: crate::polycmd::Parameters) -> Result<String, Box<dyn Error>> {
    use cube_core::ciphers::{mono::key_phrase as k, poly};
    let op = if args.decode {
        "cipher.key_phrase.decode"
    } else {
        "cipher.key_phrase.encode"
    };
    let (text, document, decode) = crate::polycmd::transport(args, op)?;
    let p: k::Parameters = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    Ok(if decode {
        serde_json::to_string(&k::decode(&text, &p)?)?
    } else {
        serde_json::to_string(&k::encode(&text, &p)?)?
    })
}
pub(crate) fn arnold(args: crate::polycmd::Parameters) -> Result<String, Box<dyn Error>> {
    use cube_core::ciphers::{book::words as a, poly};
    let op = if args.decode {
        "cipher.arnold.decode"
    } else {
        "cipher.arnold.encode"
    };
    let (text, document, decode) = crate::polycmd::transport(args, op)?;
    let p: a::Parameters = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let result = if decode {
        let input: a::DecodeInput = named_inputs(&text, op)?;
        a::decode(&input.pages.0, &input.coordinates, &p)?
    } else {
        let input: a::EncodeInput = named_inputs(&text, op)?;
        a::encode(&input.pages.0, &input.words.0, &p)?
    };
    Ok(serde_json::to_string(&result)?)
}
pub(crate) fn headlines(args: crate::polycmd::Parameters) -> Result<String, Box<dyn Error>> {
    use cube_core::ciphers::{mono::headlines as h, poly};
    let op = if args.decode {
        "cipher.headlines.decode"
    } else {
        "cipher.headlines.encode"
    };
    let (text, document, decode) = crate::polycmd::transport(args, op)?;
    let p: h::Parameters = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let input: Vec<String> = named_inputs(&text, op)?;
    Ok(serde_json::to_string(&h::transform(&input, &p, decode)?)?)
}
pub(crate) fn numbered_key(args: crate::polycmd::Parameters) -> Result<String, Box<dyn Error>> {
    use cube_core::ciphers::{mono::numbered_key as n, poly};
    let op = if args.decode {
        "cipher.numbered_key.decode"
    } else {
        "cipher.numbered_key.encode"
    };
    let (text, document, decode) = crate::polycmd::transport(args, op)?;
    let p: n::Parameters = poly::parse_parameters(&document).map_err(|e| e.operation(op))?;
    let report = if decode {
        n::decode(&named_inputs::<Vec<String>>(&text, op)?, &p)?
    } else {
        n::encode(&text, &p)?
    };
    Ok(serde_json::to_string(&report)?)
}
#[derive(Args)]
pub(crate) struct Pairs {
    #[command(flatten)]
    input: Input,
    /// JSON array of disjoint two-scalar pairs; no implicit alphabet
    #[arg(long)]
    pairs: String,
    #[arg(long, default_value = "reject")]
    unmapped: String,
}
pub(crate) fn paired(args: Pairs) -> Result<String, Box<dyn Error>> {
    let pairs: Vec<[char; 2]> = h::parse(&args.pairs)?;
    let parameters: h::Pairs =
        h::parse(&json!({"pairs":pairs,"unmapped":args.unmapped}).to_string())?;
    Ok(h::vatsyayana(
        &args.input.read_text("cipher.vatsyayana")?,
        &parameters,
    )?)
}
#[derive(Args)]
pub(crate) struct Table {
    #[command(flatten)]
    input: Input,
    /// Caller-owned local JSON table, <=1 MiB; never fetched or modified
    #[arg(long)]
    definition: PathBuf,
    /// Decode exact code tokens instead of encoding presegmented units
    #[arg(long)]
    decode: bool,
}
#[derive(Args)]
pub(crate) struct Great {
    #[command(flatten)]
    input: Input,
    /// Decode using the fixed Bazeries 1691 table, retaining alternative readings
    #[arg(long)]
    decode: bool,
}
fn definition(path: &std::path::Path) -> Result<Value, Box<dyn Error>> {
    let file = cube_core::forensics::open_regular(path)?;
    let mut document = String::new();
    file.take(h::MAX_BYTES as u64 + 1)
        .read_to_string(&mut document)?;
    Ok(h::parse(&document)?)
}
pub(crate) fn table(args: Table, syllabary: bool) -> Result<String, Box<dyn Error>> {
    let operation = match (syllabary, args.decode) {
        (false, false) => "cipher.nomenclator.encode",
        (false, true) => "cipher.nomenclator.decode",
        (true, false) => "cipher.syllabary.encode",
        (true, true) => "cipher.syllabary.decode",
    };
    run(
        operation,
        args.input,
        args.decode,
        json!({"definition":definition(&args.definition)?}),
    )
}
pub(crate) fn great(args: Great) -> Result<String, Box<dyn Error>> {
    run(
        if args.decode {
            "cipher.great_cipher.decode"
        } else {
            "cipher.great_cipher.encode"
        },
        args.input,
        args.decode,
        json!({}),
    )
}
fn run(
    operation: &'static str,
    input: Input,
    decode: bool,
    parameters: Value,
) -> Result<String, Box<dyn Error>> {
    let values: Value = h::parse(&input.read_text(operation)?)?;
    let inputs = if decode {
        json!({"tokens":values})
    } else {
        json!({"units":values})
    };
    let mut output = h::execute(operation, &inputs, &parameters)?;
    let field = if decode { "report" } else { "tokens" };
    Ok(serde_json::to_string(&output[field].take())?)
}
