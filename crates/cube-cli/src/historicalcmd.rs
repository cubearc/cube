//! Local transport adapters for explicit historical profiles.
use crate::Input;
use clap::Args;
use cube_core::ciphers::mono::historical as h;
use serde_json::{Value, json};
use std::{error::Error, io::Read, path::PathBuf};
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
