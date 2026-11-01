//! Qualified ACA base Headlines profile; no news retrieval or implicit markers.
mod catalog;
mod fixtures;
use super::{MAX_INPUT, MAX_OUTPUT, MAX_TOKENS, alphabet, bound, error, homophonic};
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
const OP: &str = "cipher.headlines.encode";
const ALPHABET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    AcaBase,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Punctuation {
    Preserve,
    AcaPrinted,
}
/// Keys use canonical A-Z. HAT ties retain left-to-right order. Exactly five
/// setting letters select the initial symbols of the five cipher alphabets.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
    #[schemars(length(min = 1, max = 26), regex(pattern = "^[A-Z]+$"))]
    pub hat: String,
    #[schemars(length(min = 1, max = 26), regex(pattern = "^[A-Z]+$"))]
    pub key: String,
    #[schemars(length(min = 5, max = 5), regex(pattern = "^[A-Z]{5}$"))]
    pub setting: String,
    pub punctuation: Punctuation,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Headlines(
    #[schemars(length(min=5,max=5),extend("x-cube-max-aggregate-utf8-bytes"=1048576,"x-cube-max-aggregate-scalars"=65536))]
    pub Vec<String>,
);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub headlines: Headlines,
    #[schemars(length(min = 26, max = 26), regex(pattern = "^[A-Z]{26}$"))]
    pub alphabet: String,
    #[schemars(length(min = 5, max = 5))]
    pub mappings: Vec<String>,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
fn invalid(parameter: &'static str) -> OperationError {
    error(
        OP,
        ErrorCode::InvalidParameter,
        parameter,
        "unsupported or invalid ACA base Headlines definition",
    )
}
fn keys(p: &Parameters) -> Result<(String, Vec<String>), OperationError> {
    for (name, key) in [("hat", &p.hat), ("key", &p.key), ("setting", &p.setting)] {
        if key.is_empty() || key.len() > 26 || !key.bytes().all(|c| c.is_ascii_uppercase()) {
            return Err(invalid(name));
        }
    }
    if p.setting.len() != 5 {
        return Err(invalid("setting"));
    }
    let keyed = alphabet::keyword_alphabet(ALPHABET, &p.key)?;
    let mut order: Vec<_> = p.hat.char_indices().collect();
    order.sort_by_key(|(i, c)| (*c, *i));
    let mut mixed = String::with_capacity(26);
    for (column, _) in order {
        for index in (column..26).step_by(p.hat.len()) {
            mixed.push(char::from(keyed.as_bytes()[index]));
        }
    }
    let mut rows = Vec::with_capacity(5);
    for setting in p.setting.bytes() {
        let offset = mixed
            .bytes()
            .position(|c| c == setting)
            .expect("A-Z alphabet");
        // ACA specifies a different three-reversal profile for identity keying.
        if offset == 0 {
            return Err(invalid("identity_setting"));
        }
        rows.push(format!("{}{}", &mixed[offset..], &mixed[..offset]));
    }
    Ok((mixed, rows))
}
fn punctuation(text: &str, p: Punctuation, decode: bool) -> Result<String, OperationError> {
    if !text.bytes().all(|c| c.is_ascii_graphic() || c == b' ') {
        return Err(error(
            OP,
            ErrorCode::InvalidInput,
            "headlines",
            "headlines require printable ASCII and spaces; supply proper-noun markers explicitly",
        ));
    }
    if matches!(p, Punctuation::AcaPrinted) {
        if text.contains(if decode { '-' } else { '=' }) {
            return Err(error(
                OP,
                ErrorCode::InvalidInput,
                "headlines",
                "ACA printed hyphen convention rejects its ambiguous literal counterpart",
            ));
        }
        Ok(text.replace(
            if decode { '=' } else { '-' },
            if decode { "-" } else { "=" },
        ))
    } else {
        Ok(text.into())
    }
}
/// Uppercase ASCII letters and apply the five explicit substitution rows.
/// Other printable ASCII is preserved, except the selected printed hyphen rule.
/// Proper-noun '*' markers must already be present in caller text.
pub fn transform(
    headlines: &[String],
    p: &Parameters,
    decode: bool,
) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.headlines.decode"
    } else {
        OP
    };
    let run = || {
        if headlines.len() != 5 {
            return Err(error(
                OP,
                ErrorCode::InvalidInput,
                "headlines",
                "exactly five caller headlines are required",
            ));
        }
        let bytes = headlines.iter().try_fold(0usize, |sum, s| {
            sum.checked_add(s.len()).ok_or_else(|| invalid("headlines"))
        })?;
        bound(OP, "headlines", bytes, MAX_INPUT)?;
        bound(
            OP,
            "headline_scalars",
            headlines.iter().map(|s| s.chars().count()).sum(),
            MAX_TOKENS,
        )?;
        let (alphabet, mappings) = keys(p)?;
        let mut output = Vec::with_capacity(5);
        for (text, target) in headlines.iter().zip(&mappings) {
            let input = punctuation(text, p.punctuation, decode)?.to_ascii_uppercase();
            let mapping = alphabet::Mapping::new(&alphabet, target)?;
            output.push(if decode {
                mapping.decode(&input, "preserve")?
            } else {
                mapping.encode(&input, "preserve")?
            });
        }
        let effective = crate::operations::contracts::canonical_value(
            serde_json::to_value(p).expect("parameters"),
        );
        let result = Report {
            headlines: Headlines(output),
            alphabet,
            mappings,
            definition_sha256: crate::hashing::sha256(
                &serde_json::to_vec(&effective).expect("parameters"),
            ),
        };
        homophonic::serialized_size(&result, MAX_OUTPUT, OP)?;
        Ok(result)
    };
    run().map_err(|e: OperationError| e.operation(op))
}
