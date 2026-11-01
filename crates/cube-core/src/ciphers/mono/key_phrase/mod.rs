//! ACA Key Phrase: a many-to-one alphabet with explicit alternative readings.
mod catalog;
mod fixtures;
use super::{MAX_INPUT, MAX_OUTPUT, MAX_TOKENS, bound, error, homophonic};
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const OP: &str = "cipher.key_phrase.encode";
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    /// Exactly the 26 uppercase letters of a complete caller-chosen phrase,
    /// with spaces/punctuation already removed. Repeated letters are retained.
    #[schemars(length(min = 26, max = 26), regex(pattern = "^[A-Z]{26}$"))]
    pub phrase: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[schemars(range(max = 65535))]
    pub byte_offset: u32,
    pub input: char,
    #[schemars(length(min = 1, max = 26), with = "Vec<char>")]
    pub alternatives: Vec<String>,
    pub mapped: bool,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report<T = Option<String>> {
    /// Null when any decoded letter has several valid source letters.
    #[schemars(length(max = 65536))]
    pub text: T,
    pub ambiguous: bool,
    #[schemars(length(max = 65536))]
    pub records: Vec<Record>,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
fn invalid(parameter: &'static str) -> OperationError {
    error(
        OP,
        ErrorCode::InvalidParameter,
        parameter,
        "Key Phrase requires exactly 26 canonical A-Z phrase letters",
    )
}
struct Phrase<'a> {
    forward: &'a [u8],
    reverse: BTreeMap<u8, Vec<String>>,
}
impl<'a> Phrase<'a> {
    fn new(p: &'a Parameters) -> Result<Self, OperationError> {
        if p.phrase.len() != 26 || !p.phrase.bytes().all(|c| c.is_ascii_uppercase()) {
            return Err(invalid("phrase"));
        }
        let mut reverse: BTreeMap<u8, Vec<String>> = BTreeMap::new();
        for (index, letter) in p.phrase.bytes().enumerate() {
            reverse
                .entry(letter)
                .or_default()
                .push(char::from(b'A' + index as u8).to_string());
        }
        Ok(Self {
            forward: p.phrase.as_bytes(),
            reverse,
        })
    }
    fn choices(&self, c: u8, decode: bool) -> Result<Vec<String>, OperationError> {
        if !c.is_ascii_alphabetic() {
            return Ok(vec![char::from(c).to_string()]);
        }
        let c = c.to_ascii_uppercase();
        if decode {
            self.reverse.get(&c).cloned().ok_or_else(|| {
                error(
                    OP,
                    ErrorCode::InvalidInput,
                    "text",
                    "cipher letter has no Key Phrase reading",
                )
            })
        } else {
            Ok(vec![
                char::from(self.forward[usize::from(c - b'A')]).to_string(),
            ])
        }
    }
}
/// Retain word boundaries, printable ASCII punctuation and explicit '*' markers.
/// ASCII case canonicalizes to uppercase. Inverse results preserve every reading
/// in A-Z order; no combinations, dictionary lookup or guessed plaintext.
fn transform(text: &str, p: &Parameters, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.key_phrase.decode"
    } else {
        OP
    };
    let run = || {
        bound(OP, "text", text.len(), MAX_INPUT)?;
        bound(OP, "text", text.chars().count(), MAX_TOKENS)?;
        if !text.bytes().all(|c| c.is_ascii_graphic() || c == b' ') {
            return Err(error(
                OP,
                ErrorCode::InvalidInput,
                "text",
                "Key Phrase requires printable ASCII and spaces",
            ));
        }
        let phrase = Phrase::new(p)?;
        let mut records = Vec::new();
        let mut unambiguous = Some(String::new());
        // Reserve a conservative bound for framing, hash and the optional text.
        let mut report_bytes = 256 + 2 * text.len();
        for (offset, c) in text.bytes().enumerate() {
            let alternatives = phrase
                .choices(c, decode)
                .map_err(|e| e.offset(offset as u64))?;
            match (&mut unambiguous, alternatives.as_slice()) {
                (Some(result), [value]) => result.push_str(value),
                _ => unambiguous = None,
            }
            let record = Record {
                byte_offset: offset as u32,
                input: char::from(c),
                alternatives,
                mapped: c.is_ascii_alphabetic(),
            };
            let mut budget =
                crate::text::mapping::ByteBudget::new(MAX_OUTPUT.saturating_sub(report_bytes));
            serde_json::to_writer(&mut budget, &record).map_err(|_| {
                error(
                    OP,
                    ErrorCode::ResourceLimit,
                    "report",
                    "Key Phrase alternatives exceed the bounded report",
                )
            })?;
            report_bytes += budget.written() + 1;
            bound(OP, "report", report_bytes, MAX_OUTPUT)?;
            records.push(record);
        }
        let result = Report {
            ambiguous: unambiguous.is_none(),
            text: unambiguous,
            records,
            definition_sha256: crate::hashing::sha256(
                &serde_json::to_vec(&crate::operations::contracts::canonical_value(
                    serde_json::to_value(p).expect("parameters"),
                ))
                .expect("parameters"),
            ),
        };
        homophonic::serialized_size(&result, MAX_OUTPUT, OP)?;
        Ok(result)
    };
    run().map_err(|e: OperationError| e.operation(op))
}
/// Encode the complete caller-supplied phrase alphabet; forward text is always unique.
pub fn encode(text: &str, p: &Parameters) -> Result<Report<String>, OperationError> {
    let result = transform(text, p, false)?;
    Ok(Report {
        text: result
            .text
            .expect("forward lookup has one output per scalar"),
        ambiguous: false,
        records: result.records,
        definition_sha256: result.definition_sha256,
    })
}
/// Decode without choosing among repeated phrase positions. Text is null when
/// any position is ambiguous; every source letter remains in the ordered records.
pub fn decode(text: &str, p: &Parameters) -> Result<Report, OperationError> {
    transform(text, p, true)
}
