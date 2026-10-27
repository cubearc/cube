//! Exact row/column/layer coordinates on caller-owned physical key geometry.
mod catalog;
mod fixtures;
#[cfg(test)]
mod tests;
use super::{Entry as LegacyEntry, Layout};
use crate::{
    ciphers::{
        poly::{bound, invalid, json_bound},
        transposition::wire,
    },
    errors::{ErrorCode, OperationError},
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
const MAX_TOKENS: usize = 65536;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Key {
    #[schemars(length(min=1,max=128),extend("x-cube-max-utf8-bytes"=128))]
    pub key: String,
    #[schemars(length(min=1,max=128),extend("x-cube-max-utf8-bytes"=128))]
    pub layer: String,
    pub symbol: char,
    #[schemars(range(min = 0, max = 511))]
    #[serde(deserialize_with = "wire::integer")]
    pub row: u16,
    #[schemars(range(min = 0, max = 511))]
    #[serde(deserialize_with = "wire::integer")]
    pub column: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=65536))]
pub struct Geometry {
    #[schemars(length(min=1,max=128),extend("x-cube-max-utf8-bytes"=128))]
    pub geometry: String,
    #[schemars(length(min = 1, max = 512))]
    pub entries: Vec<Key>,
}
/// Wire order is always [row,column,layer]; the origin is explicitly 0 or 1.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct Coordinate(
    #[schemars(range(min = 0, max = 512))]
    #[serde(deserialize_with = "wire::integer")]
    pub u16,
    #[schemars(range(min = 0, max = 512))]
    #[serde(deserialize_with = "wire::integer")]
    pub u16,
    #[schemars(length(min=1,max=128),extend("x-cube-max-utf8-bytes"=128))] pub String,
);
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub layout: Geometry,
    #[schemars(range(min = 0, max = 1))]
    #[serde(deserialize_with = "wire::integer")]
    pub index_base: u8,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoordinateReport {
    #[schemars(length(max = 65536))]
    pub coordinates: Vec<Coordinate>,
    /// Exact [UTF-8 byte offset, Unicode scalar offset] in the source text.
    #[schemars(length(max = 65536))]
    pub positions: Vec<[u32; 2]>,
    #[schemars(length(equal = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TextReport {
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=262144))]
    pub text: String,
    /// Exact [UTF-8 byte offset, Unicode scalar offset] in the decoded output.
    #[schemars(length(max = 65536))]
    pub positions: Vec<[u32; 2]>,
    #[schemars(length(equal = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
struct Validated {
    by_symbol: BTreeMap<char, Coordinate>,
    by_position: BTreeMap<(u16, u16, String), char>,
    hash: String,
}
impl Geometry {
    /// Reuse the existing US ANSI symbol tables. Rows contain13,13,11,10 keys;
    /// columns are ordinal keys within a row, not millimetres or OS coordinates.
    pub fn preset(name: &str) -> Result<Self, OperationError> {
        let old = Layout::preset(name)?;
        let mut entries = Vec::new();
        for e in old.entries {
            let i = e
                .key
                .strip_prefix("ansi")
                .and_then(|n| n.parse::<u16>().ok())
                .expect("existing physical key ID");
            let (row, column) = if i < 13 {
                (0, i)
            } else if i < 26 {
                (1, i - 13)
            } else if i < 37 {
                (2, i - 26)
            } else {
                (3, i - 37)
            };
            entries.push(Key {
                key: e.key,
                layer: e.layer,
                symbol: e.symbol,
                row,
                column,
            });
        }
        Ok(Self {
            geometry: old.geometry,
            entries,
        })
    }
    fn validate(&self, op: &'static str) -> Result<Validated, OperationError> {
        bound(op, "entries", self.entries.len(), 512)?;
        json_bound(self, 65536, op)?;
        let old = Layout {
            geometry: self.geometry.clone(),
            entries: self
                .entries
                .iter()
                .map(|e| LegacyEntry {
                    key: e.key.clone(),
                    layer: e.layer.clone(),
                    symbol: e.symbol,
                })
                .collect(),
        };
        old.validate().map_err(|e| e.operation(op))?;
        let mut by_symbol = BTreeMap::new();
        let mut by_position = BTreeMap::new();
        let mut physical = BTreeMap::new();
        for e in &self.entries {
            if e.row >= 512 || e.column >= 512 {
                return Err(invalid(op, "layout", "row and column must be 0..511"));
            }
            if physical
                .insert(e.key.as_str(), (e.row, e.column))
                .is_some_and(|p| p != (e.row, e.column))
            {
                return Err(invalid(
                    op,
                    "layout",
                    "a physical key must keep the same position in every layer",
                ));
            }
            if by_position
                .insert((e.row, e.column, e.layer.clone()), e.symbol)
                .is_some()
            {
                return Err(invalid(
                    op,
                    "layout",
                    "row,column,layer positions must be unique",
                ));
            }
            by_symbol.insert(e.symbol, Coordinate(e.row, e.column, e.layer.clone()));
        }
        Ok(Validated {
            by_symbol,
            by_position,
            hash: crate::hashing::sha256(&serde_json::to_vec(self).expect("validated definition")),
        })
    }
}
fn text(text: &str, op: &'static str) -> Result<Vec<(usize, char)>, OperationError> {
    bound(op, "text", text.len(), 1048576)?;
    let x: Vec<_> = text.char_indices().take(MAX_TOKENS + 1).collect();
    bound(op, "text", x.len(), MAX_TOKENS)?;
    Ok(x)
}
fn base(p: &Options, op: &'static str) -> Result<Validated, OperationError> {
    if p.index_base > 1 {
        return Err(invalid(op, "index_base", "index base must be 0 or 1"));
    }
    p.layout.validate(op)
}
fn missing(op: &'static str, offset: usize) -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "symbol/coordinate is absent from the declared layout",
    )
    .operation(op)
    .offset(offset as u64)
}
/// Map exact text to ordered [row,column,layer] tuples and original positions.
/// No case conversion, Unicode normalization, locale detection or passthrough.
pub fn encode(text_input: &str, p: &Options) -> Result<CoordinateReport, OperationError> {
    let op = "text.keyboard.coordinates.encode";
    let g = base(p, op)?;
    let input = text(text_input, op)?;
    let mut coordinates = Vec::with_capacity(input.len());
    let mut positions = Vec::with_capacity(input.len());
    for (i, (offset, c)) in input.into_iter().enumerate() {
        let Coordinate(r, col, layer) = g.by_symbol.get(&c).ok_or_else(|| missing(op, offset))?;
        coordinates.push(Coordinate(
            r + u16::from(p.index_base),
            col + u16::from(p.index_base),
            layer.clone(),
        ));
        positions.push([offset as u32, i as u32]);
    }
    let report = CoordinateReport {
        coordinates,
        positions,
        definition_sha256: g.hash,
    };
    json_bound(&report, 4194304, op)?;
    Ok(report)
}
/// Decode exact tuples in row,column,layer order. Tokens/transport <=65536/1 MiB;
/// output positions refer to the reconstructed text, not an invented source.
pub fn decode(coordinates: &[Coordinate], p: &Options) -> Result<TextReport, OperationError> {
    let op = "text.keyboard.coordinates.decode";
    bound(op, "coordinates", coordinates.len(), MAX_TOKENS)?;
    json_bound(&coordinates, 1048576, op)?;
    let g = base(p, op)?;
    let mut output = String::new();
    let mut positions = Vec::with_capacity(coordinates.len());
    for (i, Coordinate(r, c, layer)) in coordinates.iter().enumerate() {
        let key = r
            .checked_sub(u16::from(p.index_base))
            .zip(c.checked_sub(u16::from(p.index_base)))
            .map(|(r, c)| (r, c, layer.clone()))
            .ok_or_else(|| missing(op, i))?;
        let value = *g.by_position.get(&key).ok_or_else(|| missing(op, i))?;
        positions.push([output.len() as u32, i as u32]);
        output.push(value);
    }
    let report = TextReport {
        text: output,
        positions,
        definition_sha256: g.hash,
    };
    json_bound(&report, 4194304, op)?;
    Ok(report)
}
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgePolicy {
    #[default]
    Reject,
    Wrap,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Shift {
    pub layout: Geometry,
    #[serde(deserialize_with = "wire::integer")]
    pub dx: i64,
    #[serde(deserialize_with = "wire::integer")]
    pub dy: i64,
    #[serde(default)]
    pub edge: EdgePolicy,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShiftReport {
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=262144))]
    pub text: String,
    #[schemars(length(max = 65536))]
    pub source_coordinates: Vec<Coordinate>,
    #[schemars(length(max = 65536))]
    pub target_coordinates: Vec<Coordinate>,
    #[schemars(length(max = 65536))]
    pub positions: Vec<[u32; 2]>,
    pub bijective: bool,
    #[schemars(length(equal = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
/// Explicit displacement. Reject treats absent destinations as errors for used
/// source symbols. Wrap requires contiguous rows/columns in each layer and wraps
/// dy over rows, then dx over the destination row's length. Reverse requires a
/// total bijection over the whole layout; ragged-row collisions are never guessed.
pub fn shift(input: &str, p: &Shift, reverse: bool) -> Result<ShiftReport, OperationError> {
    let op = if reverse {
        "text.keyboard.shift.decode"
    } else {
        "text.keyboard.shift.encode"
    };
    let g = p.layout.validate(op)?;
    let input = text(input, op)?;
    let mut rows: BTreeMap<&str, BTreeMap<u16, Vec<u16>>> = BTreeMap::new();
    for (r, c, layer) in g.by_position.keys() {
        rows.entry(layer)
            .or_default()
            .entry(*r)
            .or_default()
            .push(*c);
    }
    if matches!(p.edge, EdgePolicy::Wrap) {
        for layer in rows.values() {
            if !layer.keys().copied().eq(0..layer.len() as u16)
                || layer
                    .values()
                    .any(|row| !row.iter().copied().eq(0..row.len() as u16))
            {
                return Err(invalid(
                    op,
                    "layout",
                    "wrap requires contiguous rows and row columns from zero in each layer",
                ));
            }
        }
    }
    let mut map = BTreeMap::new();
    let mut unique = BTreeSet::new();
    for (&symbol, Coordinate(r, c, layer)) in &g.by_symbol {
        let target = match p.edge {
            EdgePolicy::Reject => {
                let nr = i128::from(*r) + i128::from(p.dy);
                let nc = i128::from(*c) + i128::from(p.dx);
                u16::try_from(nr).ok().zip(u16::try_from(nc).ok())
            }
            EdgePolicy::Wrap => {
                let rs = &rows[layer.as_str()];
                let nr = (i64::from(*r) + p.dy.rem_euclid(rs.len() as i64))
                    .rem_euclid(rs.len() as i64) as u16;
                let len = rs[&nr].len() as i64;
                let nc = (i64::from(*c) + p.dx.rem_euclid(len)).rem_euclid(len) as u16;
                Some((nr, nc))
            }
        };
        if let Some((nr, nc)) = target
            && let Some(&target) = g.by_position.get(&(nr, nc, layer.clone()))
        {
            map.insert(symbol, target);
            unique.insert(target);
        }
    }
    let bijective = map.len() == g.by_symbol.len() && unique.len() == map.len();
    if reverse {
        if !bijective {
            return Err(invalid(
                op,
                "layout",
                "inverse requires a total bijection; displacement loses or merges keys",
            ));
        }
        map = map.into_iter().map(|(a, b)| (b, a)).collect();
    }
    let mut output = String::new();
    let mut source_coordinates = Vec::with_capacity(input.len());
    let mut target_coordinates = Vec::with_capacity(input.len());
    let mut positions = Vec::with_capacity(input.len());
    for (i, (offset, c)) in input.into_iter().enumerate() {
        let target = *map.get(&c).ok_or_else(|| missing(op, offset))?;
        source_coordinates.push(g.by_symbol[&c].clone());
        target_coordinates.push(g.by_symbol[&target].clone());
        positions.push([offset as u32, i as u32]);
        output.push(target);
    }
    let report = ShiftReport {
        text: output,
        source_coordinates,
        target_coordinates,
        positions,
        bijective,
        definition_sha256: g.hash,
    };
    json_bound(&report, 4194304, op)?;
    Ok(report)
}
