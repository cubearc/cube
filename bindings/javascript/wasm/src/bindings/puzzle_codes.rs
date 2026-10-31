//! Typed named-port binding adapters over the owning Core APIs.
use cube_core::{
    ciphers::{book, poly},
    errors::{ErrorCode, OperationError},
    symbols::tokens,
    text::keyboard::{self, coordinates as keys},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Text {
    text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Book {
    source: String,
    text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Coordinates {
    coordinates: Vec<keys::Coordinate>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Legacy {
    input: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BookDecode {
    source: String,
    #[serde(default = "scheme")]
    scheme: String,
    #[serde(default = "one")]
    index_base: u8,
}
fn scheme() -> String {
    "word_character".into()
}
fn one() -> u8 {
    1
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Remap {
    source: String,
    target: String,
    #[serde(default = "preserve")]
    unmapped: String,
}
fn preserve() -> String {
    "preserve".into()
}
fn parse<T: DeserializeOwned>(value: Value) -> Result<T, OperationError> {
    poly::parse_parameters(&value.to_string())
}
fn output<T: Serialize>(value: T) -> Result<Value, OperationError> {
    serde_json::to_value(value).map_err(|_| {
        OperationError::new(ErrorCode::InvalidInput, "typed output serialization failed")
    })
}
pub(super) fn supports(op: &str) -> bool {
    matches!(
        op,
        "cipher.book.encode"
            | "cipher.book.decode"
            | "text.keyboard.remap"
            | "text.keyboard.coordinates.encode"
            | "text.keyboard.coordinates.decode"
            | "text.keyboard.shift.encode"
            | "text.keyboard.shift.decode"
            | "symbol.tokens.encode"
            | "symbol.tokens.decode"
    )
}
pub(super) fn execute(
    operation: &str,
    inputs: Value,
    parameters: Value,
) -> Result<Value, OperationError> {
    let op = cube_core::operations::require(operation)?.id;
    (||match op {
 "cipher.book.encode"=>{ let i:Book=parse(inputs)?;let p:book::EncodeOptions=parse(parameters)?;output(book::encode(&i.source,&i.text,&p)?) },
 "cipher.book.decode"=>{ let i:Legacy=parse(inputs)?;let p:BookDecode=parse(parameters)?;Ok(json!({"output":book::decode(&p.source,&i.input,book::Scheme::parse(&p.scheme)?,p.index_base)?})) },
 "text.keyboard.remap"=>{ let i:Legacy=parse(inputs)?;let p:Remap=parse(parameters)?;Ok(json!({"output":keyboard::remap(&i.input,&keyboard::Layout::parse(&p.source)?,&keyboard::Layout::parse(&p.target)?,&p.unmapped)?})) },
 "text.keyboard.coordinates.encode"=>{ let i:Text=parse(inputs)?;let p:keys::Options=parse(parameters)?;output(keys::encode(&i.text,&p)?) },
 "text.keyboard.coordinates.decode"=>{ let i:Coordinates=parse(inputs)?;let p:keys::Options=parse(parameters)?;output(keys::decode(&i.coordinates,&p)?) },
 "text.keyboard.shift.encode"|"text.keyboard.shift.decode"=>{ let i:Text=parse(inputs)?;let p:keys::Shift=parse(parameters)?;output(keys::shift(&i.text,&p,op.ends_with(".decode"))?) },
 "symbol.tokens.encode"|"symbol.tokens.decode"=>{ let i:Text=parse(inputs)?;let p:tokens::Parameters=parse(parameters)?;output(tokens::transform(&i.text,&p,op.ends_with(".encode"))?) },
 _=>Err(OperationError::new(ErrorCode::InvalidParameter,"unsupported caller-table operation"))
 })().map_err(|e:OperationError|e.operation(op))
}
