//! Caller-owned exact token mappings. Provenance identifiers never execute or
//! retrieve content, prove a licence, or authorize redistribution of glyphs.
mod catalog;
mod fixtures;
#[cfg(test)]
mod tests;
use crate::{
    ciphers::{
        mono::alphabet::Mapping,
        poly::{bound, invalid, json_bound},
    },
    errors::{ErrorCode, OperationError},
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[schemars(length(min=1,max=256),extend("x-cube-max-utf8-bytes"=256))]
    pub token: String,
    #[schemars(length(min=1,max=256),extend("x-cube-max-utf8-bytes"=256))]
    pub value: String,
    #[serde(default)]
    #[schemars(length(max = 4096))]
    pub aliases: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=65536))]
pub struct Table {
    #[schemars(length(min=1,max=1024),extend("x-cube-max-utf8-bytes"=1024))]
    pub source_id: String,
    #[schemars(length(min=1,max=1024),extend("x-cube-max-utf8-bytes"=1024))]
    pub profile_id: String,
    #[schemars(length(min=1,max=1024),extend("x-cube-max-utf8-bytes"=1024))]
    pub license_id: String,
    #[schemars(length(min = 1, max = 4096))]
    pub entries: Vec<Entry>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Boundaries {
    Separated {
        #[schemars(length(min=1,max=64),extend("x-cube-max-utf8-bytes"=64))]
        separator: String,
    },
    Concatenated,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub table: Table,
    pub boundaries: Boundaries,
    /// Explicit separator between output tokens. Empty requires prefix-free output.
    #[schemars(length(max=64),extend("x-cube-max-utf8-bytes"=64))]
    pub output_separator: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[schemars(length(min=1,max=256),extend("x-cube-max-utf8-bytes"=256))]
    pub input_token: String,
    #[schemars(length(min=1,max=256),extend("x-cube-max-utf8-bytes"=256))]
    pub canonical_token: String,
    #[schemars(length(min=1,max=256),extend("x-cube-max-utf8-bytes"=256))]
    pub output_token: String,
    #[schemars(range(min = 0, max = 1048576))]
    pub byte_offset: u32,
    #[schemars(range(min = 0, max = 1048576))]
    pub scalar_offset: u32,
    pub alias_used: bool,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max=4194304),extend("x-cube-max-utf8-bytes"=4194304))]
    pub text: String,
    #[schemars(length(max = 65536))]
    pub records: Vec<Record>,
    #[schemars(length(min=1,max=1024),extend("x-cube-max-utf8-bytes"=1024))]
    pub source_id: String,
    #[schemars(length(min=1,max=1024),extend("x-cube-max-utf8-bytes"=1024))]
    pub profile_id: String,
    #[schemars(length(min=1,max=1024),extend("x-cube-max-utf8-bytes"=1024))]
    pub license_id: String,
    #[schemars(length(equal = 64), regex(pattern = r"^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
    /// Exact whole-token bijection, including every accepted alias form.
    pub bijective: bool,
}
struct Validated<'a> {
    forward: BTreeMap<&'a str, (&'a str, &'a str)>,
    reverse: BTreeMap<&'a str, &'a str>,
    unique_values: bool,
    aliases: bool,
    scalar: Option<Mapping>,
}
fn literal(
    value: &str,
    max: usize,
    op: &'static str,
    field: &'static str,
) -> Result<(), OperationError> {
    bound(op, field, value.len(), max)?;
    if value.is_empty() {
        return Err(invalid(op, field, "identifier/token must be nonempty"));
    }
    Ok(())
}
impl Table {
    fn validate(&self, op: &'static str) -> Result<Validated<'_>, OperationError> {
        if self.entries.is_empty() {
            return Err(invalid(
                op,
                "table",
                "table must contain at least one entry",
            ));
        }
        bound(op, "entries", self.entries.len(), 4096)?;
        json_bound(self, 65536, op)?;
        for value in [&self.source_id, &self.profile_id, &self.license_id] {
            literal(value, 1024, op, "provenance")?;
        }
        let mut forward = BTreeMap::new();
        let mut reverse = BTreeMap::new();
        let mut unique_values = true;
        let mut aliases = false;
        let mut scalar = true;
        let mut source = String::new();
        let mut target = String::new();
        for e in &self.entries {
            literal(&e.token, 256, op, "token")?;
            literal(&e.value, 256, op, "value")?;
            bound(op, "aliases", e.aliases.len(), 4096)?;
            if reverse.insert(e.value.as_str(), e.token.as_str()).is_some() {
                unique_values = false;
            }
            scalar &= e.token.chars().count() == 1 && e.value.chars().count() == 1;
            source.push_str(&e.token);
            target.push_str(&e.value);
            for token in std::iter::once(&e.token).chain(&e.aliases) {
                literal(token, 256, op, "token")?;
                if forward
                    .insert(token.as_str(), (e.token.as_str(), e.value.as_str()))
                    .is_some()
                {
                    return Err(invalid(
                        op,
                        "table",
                        "canonical tokens and aliases must have unique ownership",
                    ));
                }
                bound(op, "token_forms", forward.len(), 4096)?;
            }
            aliases |= !e.aliases.is_empty();
        }
        let scalar = if scalar && unique_values && !aliases {
            Some(Mapping::new(&source, &target).map_err(|e| e.operation(op))?)
        } else {
            None
        };
        Ok(Validated {
            forward,
            reverse,
            unique_values,
            aliases,
            scalar,
        })
    }
}
#[derive(Default)]
struct Node {
    next: BTreeMap<char, usize>,
    terminal: bool,
}
struct Trie {
    nodes: Vec<Node>,
}
impl Trie {
    fn new(keys: &BTreeSet<&str>, op: &'static str) -> Result<Self, OperationError> {
        let mut nodes = vec![Node::default()];
        for key in keys {
            let mut current = 0;
            for c in key.chars() {
                if nodes[current].terminal {
                    return Err(invalid(
                        op,
                        "boundaries",
                        "concatenated token alphabet is prefix-ambiguous",
                    ));
                }
                let next = if let Some(&next) = nodes[current].next.get(&c) {
                    next
                } else {
                    let next = nodes.len();
                    nodes.push(Node::default());
                    nodes[current].next.insert(c, next);
                    next
                };
                current = next;
            }
            if !nodes[current].next.is_empty() {
                return Err(invalid(
                    op,
                    "boundaries",
                    "concatenated token alphabet is prefix-ambiguous",
                ));
            }
            nodes[current].terminal = true;
        }
        Ok(Self { nodes })
    }
}
struct Parsed<'a> {
    token: &'a str,
    byte: usize,
    scalar: usize,
}
fn parse<'a>(
    input: &'a str,
    boundaries: &Boundaries,
    keys: &BTreeSet<&str>,
    op: &'static str,
) -> Result<Vec<Parsed<'a>>, OperationError> {
    let mut out = Vec::new();
    let unknown = |offset: usize| {
        OperationError::new(
            ErrorCode::InvalidInput,
            "unknown or empty token in the exact transcription",
        )
        .operation(op)
        .parameter("text")
        .offset(offset as u64)
    };
    match boundaries {
        Boundaries::Separated { separator } => {
            literal(separator, 64, op, "separator")?;
            if keys.iter().any(|key| key.contains(separator)) {
                return Err(invalid(
                    op,
                    "separator",
                    "separator occurs inside a declared token; boundaries would be ambiguous",
                ));
            }
            if input.is_empty() {
                return Ok(out);
            }
            let mut byte = 0;
            let mut scalar = 0;
            for token in input.split(separator) {
                if token.is_empty() || !keys.contains(token) {
                    return Err(unknown(byte));
                }
                bound(op, "tokens", out.len() + 1, 65536)?;
                out.push(Parsed {
                    token,
                    byte,
                    scalar,
                });
                byte += token.len() + separator.len();
                scalar += token.chars().count() + separator.chars().count();
            }
        }
        Boundaries::Concatenated => {
            let trie = Trie::new(keys, op)?;
            let mut byte = 0;
            let mut scalar = 0;
            while byte < input.len() {
                let mut current = 0;
                let mut matched = None;
                for (i, (offset, c)) in input[byte..].char_indices().enumerate() {
                    current = *trie.nodes[current]
                        .next
                        .get(&c)
                        .ok_or_else(|| unknown(byte))?;
                    if trie.nodes[current].terminal {
                        matched = Some((byte + offset + c.len_utf8(), i + 1));
                        break;
                    }
                }
                let (end, scalars) = matched.ok_or_else(|| unknown(byte))?;
                bound(op, "tokens", out.len() + 1, 65536)?;
                out.push(Parsed {
                    token: &input[byte..end],
                    byte,
                    scalar,
                });
                byte = end;
                scalar += scalars;
            }
        }
    }
    Ok(out)
}
/// Decode supplied token IDs/glyph scalars, or encode through a validated
/// alias-free bijection. Aliases decode with explicit evidence; inverse emits
/// canonical forms. Text <=1 MiB/65536 tokens, table <=64 KiB/4096 entries and
/// total token forms, report/output <=4 MiB. No normalization, assets or lookups.
/// Scalar bijections reuse the existing substitution Mapping implementation.
pub fn transform(input: &str, p: &Parameters, encode: bool) -> Result<Report, OperationError> {
    let op = if encode {
        "symbol.tokens.encode"
    } else {
        "symbol.tokens.decode"
    };
    bound(op, "text", input.len(), 1048576)?;
    let table = p.table.validate(op)?;
    bound(op, "output_separator", p.output_separator.len(), 64)?;
    if encode && (!table.unique_values || table.aliases) {
        return Err(invalid(
            op,
            "table",
            "inverse requires a bijective token/value table without aliases",
        ));
    }
    let input_keys: BTreeSet<_> = if encode {
        table.reverse.keys().copied().collect()
    } else {
        table.forward.keys().copied().collect()
    };
    let output_keys: BTreeSet<_> = if encode {
        table.reverse.values().copied().collect()
    } else {
        table.reverse.keys().copied().collect()
    };
    if p.output_separator.is_empty() {
        Trie::new(&output_keys, op)?;
    } else if output_keys
        .iter()
        .any(|key| key.contains(&p.output_separator))
    {
        return Err(invalid(
            op,
            "output_separator",
            "output separator occurs inside a declared output token",
        ));
    }
    let parsed = parse(input, &p.boundaries, &input_keys, op)?;
    let mut records = Vec::with_capacity(parsed.len());
    let mut output = String::new();
    for item in parsed {
        let (canonical_token, output_token, alias_used) = if encode {
            let canonical = table.reverse[item.token];
            (canonical, canonical, false)
        } else {
            let (canonical, value) = table.forward[item.token];
            (canonical, value, item.token != canonical)
        };
        if !records.is_empty() {
            bound(
                op,
                "output",
                output.len() + p.output_separator.len(),
                4194304,
            )?;
            output.push_str(&p.output_separator);
        }
        bound(op, "output", output.len() + output_token.len(), 4194304)?;
        output.push_str(output_token);
        records.push(Record {
            input_token: item.token.into(),
            canonical_token: canonical_token.into(),
            output_token: output_token.into(),
            byte_offset: item.byte as u32,
            scalar_offset: item.scalar as u32,
            alias_used,
        });
    }
    if let Some(mapping) = table.scalar
        && matches!(p.boundaries, Boundaries::Concatenated)
        && p.output_separator.is_empty()
    {
        output = if encode {
            mapping.decode(input, "reject")
        } else {
            mapping.encode(input, "reject")
        }
        .map_err(|e| e.operation(op))?;
    }
    let report = Report {
        text: output,
        records,
        source_id: p.table.source_id.clone(),
        profile_id: p.table.profile_id.clone(),
        license_id: p.table.license_id.clone(),
        definition_sha256: crate::hashing::sha256(
            &serde_json::to_vec(&p.table).expect("validated definition"),
        ),
        bijective: table.unique_values && !table.aliases,
    };
    json_bound(&report, 4194304, op)?;
    Ok(report)
}
