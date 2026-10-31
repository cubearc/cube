//! Typed canonical parameter/input/result DTOs; actual operations enforce the
//! same structural bounds plus cross-field/table semantics. No execution registry.
use super::{alphabet, grandpre, homophonic};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Default, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unmapped {
    #[default]
    Reject,
    Preserve,
}
impl Unmapped {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Reject => "reject",
            Self::Preserve => "preserve",
        }
    }
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
/// Source/target contain equal counts of 1..4096 unique scalars and form a
/// bijection. Their combined UTF-8 length is <=64 KiB. Sets may differ; preserve
/// is therefore not always invertible outside the declared alphabet.
pub struct Target {
    #[schemars(length(min=1,max=4096),extend("x-cube-max-utf8-bytes"=65536))]
    pub source: String,
    #[schemars(length(min=1,max=4096),extend("x-cube-max-utf8-bytes"=65536))]
    pub target: String,
    #[serde(default)]
    pub unmapped: Unmapped,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
/// Source has 1..4096 unique scalars; every keyword scalar belongs to it. Keep
/// first keyword occurrences, then unused source scalars in source order. An
/// empty keyword gives identity. Source plus keyword is <=64 KiB of UTF-8.
pub struct Keyword {
    #[schemars(length(min=1,max=4096),extend("x-cube-max-utf8-bytes"=65536))]
    pub source: String,
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=65536))]
    pub keyword: String,
    #[serde(default)]
    pub unmapped: Unmapped,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(extend("x-cube-max-combined-alphabet-utf8-bytes" = super::MAX_DEFINITION))]
/// Choose exactly one target or keyword variant. Core checks uniqueness,
/// bijection and the combined source+target/source+keyword UTF-8 byte ceiling.
pub enum Substitution {
    Target(Target),
    Keyword(Keyword),
}
impl Substitution {
    /// Strict <=64 KiB JSON transport for explicit CLI/file selection, including
    /// escaped NUL/combining scalars that cannot be placed in an OS argv string.
    pub fn parse(document: &str) -> Result<Self, crate::errors::OperationError> {
        super::parse_data(
            document,
            super::MAX_DEFINITION,
            32768,
            "cipher.substitution.encode",
            "definition",
        )
    }

    pub fn mapping(&self) -> Result<alphabet::Mapping, crate::errors::OperationError> {
        match self {
            Self::Target(p) => alphabet::Mapping::new(&p.source, &p.target),
            Self::Keyword(p) => alphabet::Mapping::keyword(&p.source, &p.keyword),
        }
    }
    pub fn unmapped(&self) -> &'static str {
        match self {
            Self::Target(p) => p.unmapped.as_str(),
            Self::Keyword(p) => p.unmapped.as_str(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Empty {}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition<T> {
    pub definition: T,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HomophonicEncode {
    pub definition: homophonic::Definition,
    #[serde(default)]
    #[schemars(length(max = 4096))]
    pub counters: Vec<homophonic::Counter>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Text(
    #[schemars(length(max=1048576),extend("x-cube-max-utf8-bytes"=1048576))] pub String,
);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct ExpandedText(
    #[schemars(length(max=4194304),extend("x-cube-max-utf8-bytes"=4194304))] pub String,
);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("x-cube-max-serialized-bytes" = super::MAX_INPUT))]
pub struct Tokens(
    #[schemars(length(max = 65536), with = "Vec<homophonic::TokenText>")] pub Vec<String>,
);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("x-cube-max-serialized-bytes" = super::MAX_INPUT))]
pub struct Coordinates(#[schemars(length(max = 65536))] pub Vec<grandpre::Token>);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("x-cube-max-serialized-bytes" = super::MAX_OUTPUT))]
pub struct Alternatives(
    #[schemars(length(max = 65536), inner(length(min = 1, max = 3)))] pub Vec<Vec<char>>,
);

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct Rot8000Text(
    #[schemars(length(max=3145728),extend("x-cube-max-utf8-bytes"=3145728))] pub String,
);
