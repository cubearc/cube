//! Explicit paginated whole-word coordinates for an Arnold-style book profile.
mod catalog;
mod fixtures;
use crate::{
    ciphers::{
        book::MatchPolicy,
        mono::{MAX_INPUT, MAX_OUTPUT, MAX_TOKENS, bound, error, homophonic},
    },
    errors::{ErrorCode, OperationError},
    text::selection,
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const OP: &str = "cipher.arnold.encode";
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    ArnoldPagedWordsV1,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
    #[schemars(length(min=1,max=1024),extend("x-cube-max-utf8-bytes"=1024))]
    pub source_id: String,
    #[schemars(range(min = 0, max = 1))]
    pub index_base: u8,
    pub choice: MatchPolicy,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("x-cube-max-serialized-bytes"=1048576))]
pub struct Pages(#[schemars(length(max = 65536))] pub Vec<String>);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("x-cube-max-serialized-bytes"=1048576))]
pub struct Words(
    #[schemars(length(max = 65536), with = "Vec<homophonic::TokenText>")] pub Vec<String>,
);
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Coordinate {
    #[schemars(range(max = 65536))]
    pub page: u32,
    #[schemars(range(max = 65536))]
    pub line: u32,
    #[schemars(range(max = 65536))]
    pub word: u32,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EncodeInput {
    pub pages: Pages,
    pub words: Words,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DecodeInput {
    pub pages: Pages,
    #[schemars(length(max=65536),extend("x-cube-max-serialized-bytes"=1048576))]
    pub coordinates: Vec<Coordinate>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max = 65536), with = "Vec<homophonic::TokenText>")]
    pub words: Vec<String>,
    #[schemars(length(max = 65536))]
    pub coordinates: Vec<Coordinate>,
    #[schemars(length(min=1,max=1024),extend("x-cube-max-utf8-bytes"=1024))]
    pub source_id: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub source_sha256: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
fn invalid(parameter: &'static str) -> OperationError {
    error(
        OP,
        ErrorCode::InvalidParameter,
        parameter,
        "invalid explicit paginated word-code profile",
    )
}
struct Index<'a> {
    forward: BTreeMap<&'a str, Vec<Coordinate>>,
    reverse: BTreeMap<Coordinate, &'a str>,
    hash: String,
}
fn charge(units: &mut usize) -> Result<(), OperationError> {
    *units += 1;
    bound(OP, "pages_lines_words", *units, MAX_TOKENS)
}
fn index<'a>(pages: &'a [String], p: &Parameters) -> Result<Index<'a>, OperationError> {
    if p.index_base > 1 || p.source_id.is_empty() || p.source_id.len() > 1024 {
        return Err(invalid("source_id_or_index_base"));
    }
    bound(OP, "pages", pages.len(), MAX_TOKENS)?;
    homophonic::serialized_size(&pages, MAX_INPUT, OP)?;
    let mut result = Index {
        forward: BTreeMap::new(),
        reverse: BTreeMap::new(),
        hash: crate::hashing::sha256(&serde_json::to_vec(&pages).expect("pages")),
    };
    let mut units = 0;
    for (page_index, page) in pages.iter().enumerate() {
        charge(&mut units)?;
        for (line_index, line) in selection::lines(page).enumerate() {
            charge(&mut units)?;
            for (word_index, word) in selection::words(line).enumerate() {
                charge(&mut units)?;
                bound(OP, "source_word", word.len(), 256)?;
                let label = |index| {
                    u32::try_from(index + usize::from(p.index_base)).expect("bounded indexed unit")
                };
                let coordinate = Coordinate {
                    page: label(page_index),
                    line: label(line_index),
                    word: label(word_index),
                };
                result.forward.entry(word).or_default().push(coordinate);
                result.reverse.insert(coordinate, word);
            }
        }
    }
    Ok(result)
}
fn report(
    index: Index<'_>,
    coordinates: Vec<Coordinate>,
    p: &Parameters,
) -> Result<Report, OperationError> {
    let mut budget = crate::text::mapping::ByteBudget::new(MAX_OUTPUT);
    serde_json::to_writer(&mut budget, &coordinates).map_err(|_| {
        error(
            OP,
            ErrorCode::ResourceLimit,
            "report",
            "coordinate report exceeds its bound",
        )
    })?;
    let mut size = 512 + 2 * p.source_id.len() + budget.written();
    let mut words = Vec::new();
    for (i, coordinate) in coordinates.iter().enumerate() {
        let word = index.reverse.get(coordinate).ok_or_else(|| {
            error(
                OP,
                ErrorCode::InvalidInput,
                "coordinates",
                "coordinate does not select a source word",
            )
            .offset(i as u64)
        })?;
        let mut budget = crate::text::mapping::ByteBudget::new(MAX_OUTPUT.saturating_sub(size));
        serde_json::to_writer(&mut budget, word).map_err(|_| {
            error(
                OP,
                ErrorCode::ResourceLimit,
                "report",
                "word report exceeds its bound",
            )
        })?;
        size += budget.written() + 1;
        bound(OP, "report", size, MAX_OUTPUT)?;
        words.push((*word).to_owned());
    }
    let definition = crate::operations::contracts::canonical_value(
        serde_json::json!({"source_sha256":index.hash,"parameters":p}),
    );
    let result = Report {
        words,
        coordinates,
        source_id: p.source_id.clone(),
        source_sha256: index.hash,
        definition_sha256: crate::hashing::sha256(
            &serde_json::to_vec(&definition).expect("definition"),
        ),
    };
    homophonic::serialized_size(&result, MAX_OUTPUT, OP)?;
    Ok(result)
}
/// Select complete exact words with first or independent round-robin matching.
/// Pages are explicit strings; lines and Unicode whitespace reuse book semantics.
/// No source edition, pagination, word normalization, code names or fallback is inferred.
pub fn encode(
    pages: &[String],
    words: &[String],
    p: &Parameters,
) -> Result<Report, OperationError> {
    let run = || {
        bound(OP, "words", words.len(), MAX_TOKENS)?;
        homophonic::serialized_size(&words, MAX_INPUT, OP)?;
        let index = index(pages, p)?;
        let mut counters = BTreeMap::new();
        let mut coordinates = Vec::with_capacity(words.len());
        for (offset, word) in words.iter().enumerate() {
            let choices = index.forward.get(word.as_str()).ok_or_else(|| {
                error(
                    OP,
                    ErrorCode::InvalidInput,
                    "words",
                    "word is absent from the exact source",
                )
                .offset(offset as u64)
            })?;
            let counter = counters.entry(word.as_str()).or_insert(0usize);
            coordinates.push(choices[*counter % choices.len()]);
            if matches!(p.choice, MatchPolicy::RoundRobin) {
                *counter = (*counter + 1) % choices.len();
            }
        }
        report(index, coordinates, p)
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
/// Decode complete words at exact coordinates. Input whitespace is not restored;
/// return presegmented words and source provenance, never guessed source text.
pub fn decode(
    pages: &[String],
    coordinates: &[Coordinate],
    p: &Parameters,
) -> Result<Report, OperationError> {
    let run = || {
        bound(OP, "coordinates", coordinates.len(), MAX_TOKENS)?;
        homophonic::serialized_size(&coordinates, MAX_INPUT, OP)?;
        report(index(pages, p)?, coordinates.to_vec(), p)
    };
    run().map_err(|e: OperationError| e.operation("cipher.arnold.decode"))
}
