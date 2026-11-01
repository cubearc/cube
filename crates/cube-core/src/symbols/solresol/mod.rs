//! Independently transcribed finite Gajewski lexicon; no musical inference.
mod catalog;
mod fixtures;
use crate::{
    ciphers::mono::{historical as h, homophonic},
    errors::OperationError,
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const MAX_REPORT: usize = 1048576;
const SOURCE_ID: &str = "gajewski-1902:facsimile-1997:pdf-page-4:monosyllables";
const SOURCE_URL: &str =
    "https://upload.wikimedia.org/wikipedia/commons/b/b9/Grammaire_du_Solresol.pdf";
const SOURCE_SHA256: &str = "c5dda1a678112e54c2de790fe3b7612ba18fef6483ee43d42f8b6f74699d6a34";
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "gajewski_1902_monosyllables_v1")]
    Gajewski1902MonosyllablesV1,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    #[schemars(length(min = 1, max = 1024))]
    pub source_id: String,
    pub profile_id: Profile,
    #[schemars(length(min = 1, max = 1024))]
    pub source_url: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub source_sha256: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Encoded {
    pub tokens: h::Tokens,
    pub provenance: Provenance,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Decoded {
    /// Exact supplied canonical/abbreviated forms, in original order.
    pub tokens: h::Tokens,
    #[schemars(length(max = 65536))]
    pub readings: Vec<h::Unit>,
    pub provenance: Provenance,
}
fn book(p: &Parameters) -> Result<(h::Book, Provenance), OperationError> {
    let definition: h::Definition =
        serde_json::from_str(include_str!("gajewski1902_monosyllables.json"))
            .expect("reviewed fixed source transcription");
    homophonic::serialized_size(&definition, 65536, "symbol.solresol.encode")?;
    let effective = crate::operations::contracts::canonical_value(
        serde_json::json!({"profile":p.profile,"source_sha256":SOURCE_SHA256,"definition":definition}),
    );
    let provenance = Provenance {
        source_id: SOURCE_ID.into(),
        profile_id: p.profile,
        source_url: SOURCE_URL.into(),
        source_sha256: SOURCE_SHA256.into(),
        definition_sha256: crate::hashing::sha256(
            &serde_json::to_vec(&effective).expect("definition"),
        ),
    };
    Ok((h::Book::new(definition)?, provenance))
}
/// Encode exact presegmented French glosses using canonical full relative
/// syllables. Synonyms share a form; no grammar, case folding or word splitting.
pub fn encode(terms: &[String], p: &Parameters) -> Result<Encoded, OperationError> {
    let run = || {
        let (book, provenance) = book(p)?;
        let result = Encoded {
            tokens: h::Tokens(book.encode(terms)?),
            provenance,
        };
        homophonic::serialized_size(&result, MAX_REPORT, "symbol.solresol.encode")?;
        Ok(result)
    };
    run().map_err(|e: OperationError| e.operation("symbol.solresol.encode"))
}
/// Decode full or explicitly published abbreviated words, preserving every
/// French gloss and zero-based token position. Repeated words remain separate;
/// unknown joined/inflected/marked forms reject, without normalization or guessing.
pub fn decode(tokens: &[String], p: &Parameters) -> Result<Decoded, OperationError> {
    let run = || {
        let (book, provenance) = book(p)?;
        let readings = book.decode(tokens)?.units;
        let result = Decoded {
            tokens: h::Tokens(tokens.to_vec()),
            readings,
            provenance,
        };
        homophonic::serialized_size(&result, MAX_REPORT, "symbol.solresol.decode")?;
        Ok(result)
    };
    run().map_err(|e: OperationError| e.operation("symbol.solresol.decode"))
}
