//! Source-qualified symbolic alphabets over exact presegmented units.
//! No OCR, font inference, case folding, transliteration or implicit segmentation.
mod catalog;
mod fixtures;
use crate::{
    ciphers::mono::{historical as h, homophonic},
    errors::{ErrorCode, OperationError},
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const MAX_REPORT: usize = 1048576;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "aurebesh_ucsur_2016_v1")]
    AurebeshUcsur2016V1,
    #[serde(rename = "standard_galactic_ucsur_2019_v1")]
    StandardGalacticUcsur2019V1,
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
    /// Exact supplied tokens, in original order.
    pub tokens: h::Tokens,
    #[schemars(length(max = 65536))]
    pub readings: Vec<h::Unit>,
    pub provenance: Provenance,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    source_id: String,
    source_url: String,
    source_sha256: String,
    definition: h::Definition,
}
fn book(p: &Parameters) -> Result<(h::Book, Provenance), OperationError> {
    let source: Source = serde_json::from_str(match p.profile {
        Profile::AurebeshUcsur2016V1 => include_str!("aurebesh_ucsur_2016_v1.json"),
        Profile::StandardGalacticUcsur2019V1 => {
            include_str!("standard_galactic_ucsur_2019_v1.json")
        }
    })
    .expect("reviewed fixed source transcription");
    let definition = source.definition;
    homophonic::serialized_size(&definition, 65536, "symbol.alphabet.encode")?;
    let effective = crate::operations::contracts::canonical_value(
        serde_json::json!({"profile":p.profile,"source_sha256":source.source_sha256,"definition":definition}),
    );
    let provenance = Provenance {
        source_id: source.source_id,
        profile_id: p.profile,
        source_url: source.source_url,
        source_sha256: source.source_sha256,
        definition_sha256: crate::hashing::sha256(
            &serde_json::to_vec(&effective).expect("definition"),
        ),
    };
    Ok((h::Book::new(definition)?, provenance))
}
/// Encode exact, presegmented units using the selected dated source table.
/// Digraphs remain distinct from separate letters; unsupported units reject.
pub fn encode(units: &[String], p: &Parameters) -> Result<Encoded, OperationError> {
    let run = || {
        let (book, provenance) = book(p)?;
        let result = Encoded {
            tokens: h::Tokens(book.encode(units).map_err(|e| input_failure(e, "units"))?),
            provenance,
        };
        homophonic::serialized_size(&result, MAX_REPORT, "symbol.alphabet.encode")?;
        Ok(result)
    };
    run().map_err(|e: OperationError| e.operation("symbol.alphabet.encode"))
}
/// Decode exact tokens, preserving every source-listed alternative and position.
/// Private-use proposals require a matching font for display; none is bundled.
pub fn decode(tokens: &[String], p: &Parameters) -> Result<Decoded, OperationError> {
    let run = || {
        let (book, provenance) = book(p)?;
        let readings = book
            .decode(tokens)
            .map_err(|e| input_failure(e, "tokens"))?
            .units;
        let result = Decoded {
            tokens: h::Tokens(tokens.to_vec()),
            readings,
            provenance,
        };
        homophonic::serialized_size(&result, MAX_REPORT, "symbol.alphabet.decode")?;
        Ok(result)
    };
    run().map_err(|e: OperationError| e.operation("symbol.alphabet.decode"))
}
fn input_failure(mut error: OperationError, field: &'static str) -> OperationError {
    if error.code == ErrorCode::InvalidParameter {
        error.code = ErrorCode::InvalidInput;
    }
    error.parameter(field)
}
