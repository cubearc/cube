//! Explicit digit grammars; Tridigital ambiguity is retained as alternatives.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Alias {
    pub from: char,
    pub to: char,
}
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[schemars(length(equal = 10), regex(pattern = r"^[0-9]{10}$"))]
    pub digits: String,
    #[schemars(length(max = 2), regex(pattern = r"^[0-9]{0,2}$"))]
    pub prefixes: String,
    #[schemars(length(min = 1, max = 28), extend("x-cube-max-utf8-bytes" = 28))]
    pub alphabet: String,
    #[serde(default)]
    #[schemars(length(max = 26))]
    pub aliases: Vec<Alias>,
    #[serde(default)]
    pub digit_escape: Option<char>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Straddling,
    MonomeDinome,
    Tridigital,
}
impl Profile {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "straddling" => Ok(Self::Straddling),
            "monome_dinome" => Ok(Self::MonomeDinome),
            "tridigital" => Ok(Self::Tridigital),
            _ => Err(error(
                "cipher.straddling.encode",
                ErrorCode::InvalidParameter,
                "profile",
                "unknown digit grammar",
            )),
        }
    }
    fn operation(self, decode: bool) -> &'static str {
        match (self, decode) {
            (Self::Straddling, false) => "cipher.straddling.encode",
            (Self::Straddling, true) => "cipher.straddling.decode",
            (Self::MonomeDinome, false) => "cipher.monome_dinome.encode",
            (Self::MonomeDinome, true) => "cipher.monome_dinome.decode",
            (Self::Tridigital, false) => "cipher.tridigital.encode",
            (Self::Tridigital, true) => "cipher.tridigital.decode",
        }
    }
}
#[derive(Debug)]
pub struct Table {
    profile: Profile,
    forward: BTreeMap<char, String>,
    reverse: BTreeMap<String, Vec<char>>,
    prefixes: BTreeSet<u8>,
    escape: Option<String>,
}
impl Table {
    pub fn parse(profile: Profile, document: &str) -> Result<Self, OperationError> {
        let op = profile.operation(false);
        let value = crate::pipeline::json::parse_with_limits(document, MAX_DEFINITION, 6, 32768)
            .map_err(|e| error(op, e.error.code, "definition", "invalid bounded table JSON"))?;
        let definition = serde_json::from_value(value).map_err(|_| {
            error(
                op,
                ErrorCode::InvalidParameter,
                "definition",
                "invalid digit table fields",
            )
        })?;
        Self::new(profile, definition)
    }
    /// Caller-selected ten unique digits, ordered canonical alphabet, explicit
    /// prefixes/aliases/optional escape. No inferred table, keyword or whitespace.
    pub fn new(profile: Profile, def: Definition) -> Result<Self, OperationError> {
        let op = profile.operation(false);
        homophonic::serialized_size(&def, MAX_DEFINITION, op)?;
        let invalid = || {
            error(
                op,
                ErrorCode::InvalidParameter,
                "definition",
                "invalid alphabet, digit permutation, prefixes, aliases or escape for this profile",
            )
        };
        if def.digits.len() != 10
            || !def.digits.bytes().all(|c| c.is_ascii_digit())
            || def.digits.bytes().collect::<BTreeSet<_>>().len() != 10
        {
            return Err(invalid());
        }
        let symbols: Vec<_> = def.alphabet.chars().collect();
        let set: BTreeSet<_> = symbols.iter().copied().collect();
        if symbols.is_empty()
            || symbols.len() != set.len()
            || symbols
                .iter()
                .any(|c| !c.is_ascii() || c.is_ascii_lowercase() || c.is_ascii_control())
        {
            return Err(invalid());
        }
        let prefixes: BTreeSet<_> = def.prefixes.bytes().collect();
        if profile == Profile::Tridigital {
            if !def.prefixes.is_empty()
                || symbols.len() != 26
                || !symbols.iter().all(|c| c.is_ascii_uppercase())
                || !def.aliases.is_empty()
                || def.digit_escape.is_some()
            {
                return Err(invalid());
            }
        } else if def.prefixes.len() != 2
            || prefixes.len() != 2
            || !prefixes.iter().all(u8::is_ascii_digit)
        {
            return Err(invalid());
        }
        if profile == Profile::MonomeDinome
            && (symbols.len() != 24
                || !symbols.iter().all(|c| c.is_ascii_uppercase())
                || def.aliases.len() != 2
                || !def.aliases.iter().any(|a| a.from == 'J' && a.to == 'I')
                || def.digit_escape.is_some())
        {
            return Err(invalid());
        }
        if profile == Profile::Straddling && symbols.len() > 28 {
            return Err(invalid());
        }
        if let Some(escape) = def.digit_escape
            && (profile != Profile::Straddling
                || !set.contains(&escape)
                || escape.is_ascii_alphanumeric()
                || symbols.iter().any(char::is_ascii_digit))
        {
            return Err(invalid());
        }
        let mut codes = Vec::new();
        if profile == Profile::Tridigital {
            for i in 0..26 {
                codes.push((def.digits.as_bytes()[i % 9] as char).to_string());
            }
        } else {
            for d in def.digits.bytes().filter(|d| !prefixes.contains(d)) {
                codes.push((d as char).to_string());
            }
            for row in def.prefixes.bytes() {
                for col in def
                    .digits
                    .bytes()
                    .filter(|d| profile == Profile::Straddling || !prefixes.contains(d))
                {
                    codes.push(format!("{}{}", row as char, col as char));
                }
            }
        }
        let mut forward = BTreeMap::new();
        let mut reverse: BTreeMap<String, Vec<char>> = BTreeMap::new();
        for (c, code) in symbols.into_iter().zip(codes) {
            forward.insert(c, code.clone());
            reverse.entry(code).or_default().push(c);
        }
        if profile == Profile::Tridigital {
            let space = (def.digits.as_bytes()[9] as char).to_string();
            forward.insert(' ', space.clone());
            reverse.insert(space, vec![' ']);
        }
        let mut seen = set.clone();
        for alias in def.aliases {
            if !alias.from.is_ascii_uppercase()
                || !alias.to.is_ascii_uppercase()
                || !seen.insert(alias.from)
                || !set.contains(&alias.to)
            {
                return Err(invalid());
            }
            forward.insert(alias.from, forward[&alias.to].clone());
        }
        if profile == Profile::MonomeDinome
            && !(b'A'..=b'Z').all(|c| forward.contains_key(&(c as char)))
        {
            return Err(invalid());
        }
        let escape = def.digit_escape.map(|c| forward[&c].clone());
        Ok(Self {
            profile,
            forward,
            reverse,
            prefixes,
            escape,
        })
    }
    /// Canonicalize ASCII letters to uppercase. Every space/other symbol must have
    /// an explicit cell; Tridigital alone reserves its tenth digit for spaces.
    pub fn encode(&self, text: &str) -> Result<String, OperationError> {
        let op = self.profile.operation(false);
        bound(op, "text", text.len(), MAX_INPUT)?;
        let mut output = String::new();
        for (i, (offset, c)) in text.char_indices().enumerate() {
            bound(op, "text", i + 1, MAX_TOKENS)?;
            let c = c.to_ascii_uppercase();
            if let Some(escape) = &self.escape {
                if c.is_ascii_digit() {
                    bound(op, "output", output.len() + escape.len() + 1, MAX_OUTPUT)?;
                    output.push_str(escape);
                    output.push(c);
                    continue;
                }
                if self.forward.get(&c) == Some(escape) {
                    return Err(error(
                        op,
                        ErrorCode::InvalidInput,
                        "text",
                        "literal escape marker is reserved",
                    )
                    .offset(offset as u64));
                }
            }
            let code = self.forward.get(&c).ok_or_else(|| {
                error(
                    op,
                    ErrorCode::InvalidInput,
                    "text",
                    "unmapped ASCII source symbol",
                )
                .offset(offset as u64)
            })?;
            bound(op, "output", output.len() + code.len(), MAX_OUTPUT)?;
            output.push_str(code);
        }
        Ok(output)
    }
    /// Parse strict ungrouped digits. No ignored whitespace or guessed padding.
    /// Each returned group contains one canonical symbol, except lossy Tridigital.
    pub fn alternatives(&self, text: &str) -> Result<Vec<Vec<char>>, OperationError> {
        let op = self.profile.operation(true);
        bound(op, "text", text.len(), MAX_INPUT)?;
        let mut output = Vec::new();
        let mut offset = 0;
        while offset < text.len() {
            bound(op, "tokens", output.len() + 1, MAX_TOKENS)?;
            let d = text.as_bytes()[offset];
            if !d.is_ascii_digit() {
                return Err(error(
                    op,
                    ErrorCode::InvalidInput,
                    "text",
                    "digit transport accepts only ungrouped ASCII digits",
                )
                .offset(offset as u64));
            }
            let width = if self.prefixes.contains(&d) { 2 } else { 1 };
            let code = text
                .get(offset..offset + width)
                .filter(|v| v.bytes().all(|b| b.is_ascii_digit()))
                .ok_or_else(|| {
                    error(
                        op,
                        ErrorCode::InvalidInput,
                        "text",
                        "incomplete digit prefix",
                    )
                    .offset(offset as u64)
                })?;
            if self.escape.as_deref() == Some(code) {
                let digit = text
                    .as_bytes()
                    .get(offset + width)
                    .filter(|d| d.is_ascii_digit())
                    .ok_or_else(|| {
                        error(
                            op,
                            ErrorCode::InvalidInput,
                            "text",
                            "incomplete digit escape",
                        )
                        .offset(offset as u64)
                    })?;
                output.push(vec![*digit as char]);
                offset += width + 1;
            } else {
                let alternatives = self.reverse.get(code).ok_or_else(|| {
                    error(op, ErrorCode::InvalidInput, "text", "unassigned digit code")
                        .offset(offset as u64)
                })?;
                output.push(alternatives.clone());
                offset += width;
            }
        }
        homophonic::serialized_size(&output, MAX_OUTPUT, op)?;
        Ok(output)
    }
    /// Reversible profiles only. Tridigital must use alternatives, never guessing.
    pub fn decode(&self, text: &str) -> Result<String, OperationError> {
        if self.profile == Profile::Tridigital {
            return Err(error(
                self.profile.operation(true),
                ErrorCode::InvalidParameter,
                "profile",
                "Tridigital decoding returns alternatives",
            ));
        }
        Ok(self.alternatives(text)?.into_iter().map(|v| v[0]).collect())
    }
}
