//! Caller-owned token ownership and per-symbol round-robin selection.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const ENCODE: &str = "cipher.homophonic.encode";
const DECODE: &str = "cipher.homophonic.decode";
/// Token string schema mirrors the exact serialized String representation and
/// its existing byte/scalar limits; it does not select or execute an operation.
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct TokenText(
    #[schemars(length(min=1,max=256),extend("x-cube-max-utf8-bytes"=256))] pub String,
);
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
/// One unique canonical scalar with 1..256 nonempty tokens, each <=256 UTF-8
/// bytes. Every token is globally unique, including within the same entry.
pub struct Entry {
    pub symbol: char,
    #[schemars(length(min = 1, max = 256), with = "Vec<TokenText>")]
    pub tokens: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes" = MAX_DEFINITION))]
#[schemars(extend("x-cube-max-total-tokens" = MAX_SYMBOLS))]
/// Caller-owned table whose compact JSON is <=64 KiB, with 1..4096 total tokens.
/// Source scalars are unique and already canonical. When ascii_uppercase is true,
/// ASCII source letters must be uppercase and input ASCII letters uppercase;
/// other scalars remain exact, without Unicode normalization or case folding.
pub struct Definition {
    pub ascii_uppercase: bool,
    #[schemars(length(min = 1, max = 4096))]
    pub entries: Vec<Entry>,
}
/// Optional starting selection indexes; 0..65535, reduced modulo that symbol's list.
/// The bound is exact on every JSON/JavaScript transport; it is not hidden state.
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Counter {
    pub symbol: char,
    pub index: u16,
}
#[derive(Debug)]
pub struct Mapping {
    entries: Vec<Entry>,
    forward: BTreeMap<char, usize>,
    reverse: BTreeMap<String, char>,
    uppercase: bool,
}
pub(crate) fn serialized_size(
    value: &impl Serialize,
    maximum: usize,
    op: &'static str,
) -> Result<(), OperationError> {
    let mut budget = crate::text::mapping::ByteBudget::new(maximum);
    serde_json::to_writer(&mut budget, value).map_err(|_| {
        error(
            op,
            ErrorCode::ResourceLimit,
            "document",
            "serialized value exceeds its byte budget",
        )
    })
}
impl Mapping {
    /// Parse strict <=64 KiB JSON, bounded depth/nodes and duplicate-key rejection.
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        let value = crate::pipeline::json::parse_with_limits(document, MAX_DEFINITION, 6, 32768)
            .map_err(|e| {
                error(
                    ENCODE,
                    e.error.code,
                    "definition",
                    "invalid bounded mapping JSON",
                )
            })?;
        let def: Definition = serde_json::from_value(value).map_err(|_| {
            error(
                ENCODE,
                ErrorCode::InvalidParameter,
                "definition",
                "invalid mapping fields or scalar/token types",
            )
        })?;
        Self::new(def)
    }
    /// 1..4096 total unique nonempty tokens, 1..256 per source scalar, each <=256
    /// UTF-8 bytes, combined JSON <=64 KiB. Token concatenation is never inferred.
    pub fn new(def: Definition) -> Result<Self, OperationError> {
        Self::with_token_limit(def, 256)
    }
    /// Internal constructors can retain a complete bounded repeated-position key.
    /// The public caller-table API keeps its 256-token-per-symbol contract.
    pub(super) fn with_token_limit(
        def: Definition,
        per_symbol: usize,
    ) -> Result<Self, OperationError> {
        Self::with_limits(def, per_symbol, MAX_DEFINITION)
    }
    /// Derived numeric tables have a separately bounded internal serialization;
    /// public caller definitions retain their existing64-KiB/256-token policy.
    pub(super) fn with_limits(
        def: Definition,
        per_symbol: usize,
        definition_bytes: usize,
    ) -> Result<Self, OperationError> {
        serialized_size(&def, definition_bytes, ENCODE)?;
        bound(ENCODE, "entries", def.entries.len(), MAX_SYMBOLS)?;
        if def.entries.is_empty() {
            return Err(error(
                ENCODE,
                ErrorCode::InvalidParameter,
                "entries",
                "mapping must not be empty",
            ));
        }
        let mut forward = BTreeMap::new();
        let mut reverse = BTreeMap::new();
        for (i, entry) in def.entries.iter().enumerate() {
            let symbol = if def.ascii_uppercase {
                entry.symbol.to_ascii_uppercase()
            } else {
                entry.symbol
            };
            if symbol != entry.symbol
                || forward.insert(symbol, i).is_some()
                || entry.tokens.is_empty()
                || entry.tokens.len() > per_symbol
            {
                return Err(error(
                    ENCODE,
                    ErrorCode::InvalidParameter,
                    "entries",
                    "source scalars must be canonical/unique with 1..256 tokens",
                ));
            }
            for token in &entry.tokens {
                bound(ENCODE, "tokens", reverse.len() + 1, MAX_SYMBOLS)?;
                if token.is_empty()
                    || token.len() > 256
                    || reverse.insert(token.clone(), symbol).is_some()
                {
                    return Err(error(
                        ENCODE,
                        ErrorCode::InvalidParameter,
                        "tokens",
                        "tokens must be unique, nonempty and <=256 UTF-8 bytes",
                    ));
                }
            }
        }
        Ok(Self {
            entries: def.entries,
            forward,
            reverse,
            uppercase: def.ascii_uppercase,
        })
    }
    /// <=1 MiB text, <=65536 source scalars/output tokens and <=1 MiB serialized
    /// output. Counters start from the supplied indexes on every call, then wrap.
    pub fn encode(&self, text: &str, starting: &[Counter]) -> Result<Vec<String>, OperationError> {
        bound(ENCODE, "text", text.len(), MAX_INPUT)?;
        bound(ENCODE, "counters", starting.len(), self.entries.len())?;
        let mut counters = BTreeMap::new();
        for counter in starting {
            if !self.forward.contains_key(&counter.symbol)
                || counters
                    .insert(counter.symbol, usize::from(counter.index))
                    .is_some()
            {
                return Err(error(
                    ENCODE,
                    ErrorCode::InvalidParameter,
                    "counters",
                    "counter scalars must be canonical, mapped and unique",
                ));
            }
        }
        let mut output = Vec::new();
        // Charge JSON brackets, commas and escaped token bytes before cloning.
        let mut size = 2;
        for (offset, c) in text.char_indices() {
            bound(ENCODE, "text", output.len() + 1, MAX_TOKENS)?;
            let c = if self.uppercase {
                c.to_ascii_uppercase()
            } else {
                c
            };
            let i = self.forward.get(&c).ok_or_else(|| {
                error(
                    ENCODE,
                    ErrorCode::InvalidInput,
                    "text",
                    "unmapped source scalar",
                )
                .offset(offset as u64)
            })?;
            let tokens = &self.entries[*i].tokens;
            let counter = counters.entry(c).or_default();
            let token = &tokens[*counter % tokens.len()];
            *counter = (*counter % tokens.len() + 1) % tokens.len();
            let mut budget = crate::text::mapping::ByteBudget::new(MAX_INPUT - size);
            serde_json::to_writer(&mut budget, token).map_err(|_| {
                error(
                    ENCODE,
                    ErrorCode::ResourceLimit,
                    "output",
                    "encoded tokens exceed 1 MiB JSON",
                )
            })?;
            size += budget.written() + usize::from(!output.is_empty());
            bound(ENCODE, "output", size, MAX_INPUT)?;
            output.push(token.clone());
        }
        Ok(output)
    }
    /// Exact tokens -> canonical scalars; <=65536 tokens/1 MiB input JSON/text output.
    pub fn decode(&self, tokens: &[String]) -> Result<String, OperationError> {
        bound(DECODE, "tokens", tokens.len(), MAX_TOKENS)?;
        serialized_size(&tokens, MAX_INPUT, DECODE)?;
        let mut output = String::new();
        for token in tokens {
            let c = self
                .reverse
                .get(token)
                .ok_or_else(|| error(DECODE, ErrorCode::InvalidInput, "tokens", "unknown token"))?;
            bound(DECODE, "output", output.len() + c.len_utf8(), MAX_INPUT)?;
            output.push(*c);
        }
        Ok(output)
    }
}
