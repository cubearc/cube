//! Grandpre repeated-letter square with explicit digit labels and token spaces.
use super::*;
pub use crate::ciphers::polybius::Token;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[schemars(
        length(min = 8, max = 10),
        inner(length(min = 8, max = 10), regex(pattern = r"^[A-Z]{8,10}$"))
    )]
    pub rows: Vec<String>,
    #[schemars(length(min = 8, max = 10), regex(pattern = r"^[0-9]{8,10}$"))]
    pub row_labels: String,
    #[schemars(length(min = 8, max = 10), regex(pattern = r"^[0-9]{8,10}$"))]
    pub column_labels: String,
}
#[derive(Debug)]
pub struct Square {
    rows: Vec<String>,
    row_labels: Vec<u8>,
    column_labels: Vec<u8>,
    positions: [Vec<[u8; 2]>; 26],
}
impl Square {
    pub fn parse(document: &str) -> Result<Self, OperationError> {
        let value = crate::pipeline::json::parse_with_limits(document, MAX_DEFINITION, 4, 256)
            .map_err(|e| {
                error(
                    "cipher.grandpre.encode",
                    e.error.code,
                    "definition",
                    "invalid bounded Grandpre JSON",
                )
            })?;
        Self::new(serde_json::from_value(value).map_err(|_| {
            error(
                "cipher.grandpre.encode",
                ErrorCode::InvalidParameter,
                "definition",
                "invalid Grandpre fields",
            )
        })?)
    }
    /// Supported subset: 8x8, 9x9 and 10x10 uppercase ASCII-letter grids. Labels
    /// are explicit unique digits in cell order; 1234567890 expresses ten as zero.
    /// Repeated letters are legal; no word list or all-letter completeness is inferred.
    pub fn new(def: Definition) -> Result<Self, OperationError> {
        let op = "cipher.grandpre.encode";
        homophonic::serialized_size(&def, MAX_DEFINITION, op)?;
        let side = def.rows.len();
        if ![8, 9, 10].contains(&side)
            || def
                .rows
                .iter()
                .any(|r| r.len() != side || !r.bytes().all(|c| c.is_ascii_uppercase()))
        {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "rows",
                "Grandpre requires an 8x8, 9x9 or 10x10 ASCII uppercase grid",
            ));
        }
        let labels = |s: &str| -> Result<Vec<u8>, OperationError> {
            if s.len() != side
                || !s.bytes().all(|b| b.is_ascii_digit())
                || s.bytes().collect::<std::collections::BTreeSet<_>>().len() != side
            {
                return Err(error(
                    op,
                    ErrorCode::InvalidParameter,
                    "labels",
                    "labels must be unique ASCII digits matching the side",
                ));
            }
            Ok(s.bytes().map(|b| b - b'0').collect())
        };
        let row_labels = labels(&def.row_labels)?;
        let column_labels = labels(&def.column_labels)?;
        let mut positions: [Vec<[u8; 2]>; 26] = std::array::from_fn(|_| Vec::new());
        for (r, row) in def.rows.iter().enumerate() {
            for (c, b) in row.bytes().enumerate() {
                positions[usize::from(b - b'A')].push([row_labels[r], column_labels[c]]);
            }
        }
        Ok(Self {
            rows: def.rows,
            row_labels,
            column_labels,
            positions,
        })
    }
    /// Per-letter row-major round robin resets each call. Literal ASCII spaces
    /// become null tokens; letters canonicalize to uppercase. <=65536 tokens.
    pub fn encode(&self, text: &str) -> Result<Vec<Token>, OperationError> {
        let op = "cipher.grandpre.encode";
        bound(op, "text", text.len(), MAX_INPUT)?;
        let mut counters = [0usize; 26];
        let mut output = Vec::new();
        for (offset, c) in text.char_indices() {
            bound(op, "text", output.len() + 1, MAX_TOKENS)?;
            if c == ' ' {
                output.push(None);
                continue;
            }
            if !c.is_ascii_alphabetic() {
                return Err(error(
                    op,
                    ErrorCode::InvalidInput,
                    "text",
                    "Grandpre accepts only ASCII letters and tokenized spaces",
                )
                .offset(offset as u64));
            }
            let i = usize::from(c.to_ascii_uppercase() as u8 - b'A');
            let choices = &self.positions[i];
            if choices.is_empty() {
                return Err(error(
                    op,
                    ErrorCode::InvalidInput,
                    "text",
                    "source letter is absent from the grid",
                )
                .offset(offset as u64));
            }
            output.push(Some(choices[counters[i] % choices.len()]));
            counters[i] += 1;
        }
        homophonic::serialized_size(&output, MAX_INPUT, op)?;
        Ok(output)
    }
    /// Exact row/column digit pairs or null spaces. No unique-square restriction.
    pub fn decode(&self, tokens: &[Token]) -> Result<String, OperationError> {
        let op = "cipher.grandpre.decode";
        bound(op, "tokens", tokens.len(), MAX_TOKENS)?;
        homophonic::serialized_size(&tokens, MAX_INPUT, op)?;
        let mut output = String::new();
        for token in tokens {
            if let Some([row, col]) = token {
                let r = self.row_labels.iter().position(|n| n == row);
                let c = self.column_labels.iter().position(|n| n == col);
                let (Some(r), Some(c)) = (r, c) else {
                    return Err(error(
                        op,
                        ErrorCode::InvalidInput,
                        "tokens",
                        "coordinate is absent from explicit labels",
                    ));
                };
                output.push(self.rows[r].as_bytes()[c] as char);
            } else {
                output.push(' ');
            }
        }
        Ok(output)
    }
}
