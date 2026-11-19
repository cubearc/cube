//! Bounded, offline Beider–Morse alternatives under a pinned Apache rule corpus.
mod corpus;

use super::{check_report, invalid, parameter, resource};
use crate::errors::{ErrorCode, OperationError};
use corpus::{ANY, CORPUS, Mask, Table};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const OP: &str = "text.beider_morse";
const PROFILE: &str = "apache-commons-codec-1.15-bm/cube-v1";
const MAX_WORK: usize = 1 << 20;
const MAX_ALTERNATIVES: usize = 4096;
const MAX_FRONTIER_BYTES: usize = 2 << 20;
const MAX_OUTPUT: usize = 1 << 20;

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Generic,
    Ashkenazi,
    Sephardic,
}

impl Type {
    const fn key(self) -> &'static str {
        match self {
            Self::Generic => "gen",
            Self::Ashkenazi => "ash",
            Self::Sephardic => "sep",
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Matter {
    Exact,
    Approximate,
}

impl Matter {
    const fn key(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Approximate => "approx",
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum LanguageSelection {
    Guess,
    Any,
    Explicit {
        #[schemars(length(min = 1, max = 18))]
        languages: Vec<String>,
    },
}

const fn concatenate() -> bool {
    true
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub name: String,
    pub name_type: Type,
    pub rule_type: Matter,
    pub language: LanguageSelection,
    #[serde(default = "concatenate")]
    pub concatenate: bool,
}

#[derive(Clone, Copy, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PathStep {
    PrefixSeparated,
    PrefixJoined,
    Word { index: usize },
}

#[derive(Serialize, JsonSchema)]
pub struct Group {
    pub path: Vec<PathStep>,
    pub table_language: &'static str,
    pub languages: Vec<&'static str>,
    pub alternatives: Vec<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub profile: &'static str,
    pub profile_sha256: &'static str,
    pub name_type: Type,
    pub rule_type: Matter,
    pub concatenate: bool,
    pub encoding: String,
    pub groups: Vec<Group>,
    pub work_units: usize,
    pub peak_alternatives: usize,
}

struct Work {
    remaining: usize,
    peak: usize,
}

impl Work {
    fn charge(&mut self, units: usize) -> Result<(), OperationError> {
        self.remaining = self
            .remaining
            .checked_sub(units)
            .ok_or_else(|| resource("Beider–Morse rule-state limit exceeded"))?;
        Ok(())
    }
}

#[derive(Default)]
struct Frontier {
    states: BTreeMap<String, Mask>,
    bytes: usize,
}

impl Frontier {
    fn empty(languages: Mask, work: &mut Work) -> Result<Self, OperationError> {
        let mut states = Self::default();
        states.add(String::new(), languages, work)?;
        Ok(states)
    }

    fn add(
        &mut self,
        text: String,
        languages: Mask,
        work: &mut Work,
    ) -> Result<(), OperationError> {
        if languages == 0 {
            return Ok(());
        }
        if let Some(prior) = self.states.get_mut(&text) {
            *prior |= languages;
        } else {
            let cost = text.capacity() + 128;
            if self.states.len() == MAX_ALTERNATIVES || cost > MAX_FRONTIER_BYTES - self.bytes {
                return Err(resource(
                    "Beider–Morse live alternative or frontier-size limit exceeded",
                ));
            }
            self.bytes += cost;
            self.states.insert(text, languages);
            work.peak = work.peak.max(self.states.len());
        }
        Ok(())
    }
}

fn combined(a: &str, b: &str, work: &mut Work) -> Result<String, OperationError> {
    let len = a.len() + b.len();
    work.charge(1 + len)?;
    if len > MAX_OUTPUT {
        return Err(resource(
            "one Beider–Morse alternative exceeds the result byte limit",
        ));
    }
    let mut output = String::with_capacity(len);
    output.push_str(a);
    output.push_str(b);
    Ok(output)
}

fn rewrite(
    input: &str,
    table: &Table,
    languages: Mask,
    preserve_unmatched: bool,
    work: &mut Work,
) -> Result<Frontier, OperationError> {
    let mut states = Frontier::empty(languages, work)?;
    let mut offset = 0;
    while offset < input.len() && !states.states.is_empty() {
        work.charge(1)?;
        let scalar = input[offset..].chars().next().expect("scalar boundary");
        let mut matching = None;
        if let Some(candidates) = table.get(&scalar) {
            for rule in candidates {
                if rule.matches(input, offset, work)? {
                    matching = Some(rule);
                    break;
                }
            }
        }
        let Some(rule) = matching else {
            let end = offset + scalar.len_utf8();
            if preserve_unmatched {
                let mut next = Frontier::default();
                for (text, language) in states.states {
                    next.add(combined(&text, &input[offset..end], work)?, language, work)?;
                }
                states = next;
            }
            offset = end;
            continue;
        };
        let mut next = Frontier::default();
        for (text, available) in states.states {
            for phoneme in &rule.alternatives {
                work.charge(1)?;
                let common = available & phoneme.languages;
                if common != 0 {
                    next.add(combined(&text, &phoneme.text, work)?, common, work)?;
                }
            }
        }
        states = next;
        offset += rule.pattern.len();
    }
    Ok(states)
}

fn final_rules(
    input: Frontier,
    table: &Table,
    work: &mut Work,
) -> Result<Frontier, OperationError> {
    if table.is_empty() {
        return Ok(input);
    }
    let mut combined = Frontier::default();
    for (alternative, languages) in input.states {
        let rewritten = rewrite(&alternative, table, languages, true, work)?;
        for (alternative, mask) in rewritten.states {
            work.charge(1 + alternative.len())?;
            combined.add(alternative, mask, work)?;
        }
    }
    Ok(combined)
}

// The Apache 1.15 reference evaluates "de" before the overlapping "de la".
const GENERIC_PREFIXES: &[&str] = &[
    "de", "de la", "della", "dela", "dal", "del", "des", "dos", "van", "von", "da", "di", "do",
    "du",
];
const ASH_PREFIXES: &[&str] = &["bar", "ben", "da", "de", "van", "von"];

fn generic_prefix(input: &str) -> Option<(&str, &str)> {
    if let Some(remainder) = input.strip_prefix("d'") {
        return Some(("d", remainder));
    }
    GENERIC_PREFIXES.iter().find_map(|&prefix| {
        input
            .strip_prefix(prefix)?
            .strip_prefix(' ')
            .map(|rest| (prefix, rest))
    })
}

fn explicit(kind: Type, names: &[String]) -> Result<Mask, OperationError> {
    if names.is_empty() || names.len() > 18 {
        return Err(parameter(
            "explicit languages require one to 18 declared languages",
        ));
    }
    let mut set = 0;
    for name in names {
        let bit = corpus::bit(name).filter(|_| name != "any").ok_or_else(|| {
            parameter("unknown explicit language; use mode any for unrestricted rules")
        })?;
        if bit & CORPUS.possible(kind) == 0 {
            return Err(OperationError::new(
                ErrorCode::UnsupportedFormat,
                "language is unavailable for this Beider–Morse name type",
            )
            .parameter("language"));
        }
        if set & bit != 0 {
            return Err(parameter("explicit languages must be unique"));
        }
        set |= bit;
    }
    Ok(set)
}

fn separators(ch: char) -> bool {
    matches!(ch, ' ' | '\r' | '\n' | '\t' | '\x0b' | '\x0c')
}

struct Engine<'a> {
    request: &'a Request,
    explicit: Option<Mask>,
    groups: Vec<Group>,
    path: Vec<PathStep>,
    alternatives: usize,
    result_bytes: usize,
    work: Work,
}

impl Engine<'_> {
    fn encode(&mut self, input: &str) -> Result<String, OperationError> {
        self.work.charge(1 + input.len())?;
        let tidied = input.replace('-', " ");
        let tidied = tidied.trim_matches(separators);
        if matches!(self.request.name_type, Type::Generic)
            && let Some((prefix, remainder)) = generic_prefix(tidied)
        {
            self.path.push(PathStep::PrefixSeparated);
            let isolated = self.encode(remainder)?;
            self.path.pop();
            self.path.push(PathStep::PrefixJoined);
            let joined = self.encode(&format!("{prefix}{remainder}"))?;
            self.path.pop();
            return assemble(&format!("({isolated})"), "-", &format!("({joined})"));
        }
        let mut words = tidied.split(separators).filter(|word| !word.is_empty());
        let first = words.clone().next().unwrap_or("");
        let selected: Vec<_> = match self.request.name_type {
            Type::Generic => words.collect(),
            Type::Ashkenazi => words.filter(|word| !ASH_PREFIXES.contains(word)).collect(),
            Type::Sephardic => {
                const ADDITIONAL: &[&str] = &["al", "el"];
                words
                    .by_ref()
                    .filter_map(|word| word.rsplit('\'').find(|piece| !piece.is_empty()))
                    .filter(|word| !GENERIC_PREFIXES.contains(word) && !ADDITIONAL.contains(word))
                    .collect()
            }
        };
        if self.request.concatenate {
            self.leaf(&selected.join(" "), input)
        } else {
            match selected.as_slice() {
                [] => self.leaf("", input),
                [_] => self.leaf(first, input),
                words => {
                    let mut result = String::new();
                    for (index, word) in words.iter().enumerate() {
                        self.path.push(PathStep::Word { index });
                        let segment = self.leaf(word, word)?;
                        self.path.pop();
                        if index != 0 {
                            append(&mut result, "-")?;
                        }
                        append(&mut result, &segment)?;
                    }
                    Ok(result)
                }
            }
        }
    }

    fn leaf(&mut self, text: &str, language_source: &str) -> Result<String, OperationError> {
        let kind = self.request.name_type;
        let mode = Some(self.request.rule_type);
        let languages = match self.explicit {
            Some(mask) => mask,
            None => CORPUS.infer(kind, language_source, &mut self.work)?,
        };
        let table_language = if languages.count_ones() == 1 {
            corpus::names(languages).next().expect("one language")
        } else {
            "any"
        };
        let initial = rewrite(
            text,
            CORPUS.table(kind, None, table_language),
            languages,
            false,
            &mut self.work,
        )?;
        let shared = final_rules(initial, CORPUS.table(kind, mode, "common"), &mut self.work)?;
        let finished = final_rules(
            shared,
            CORPUS.table(kind, mode, table_language),
            &mut self.work,
        )?;
        let alternatives: Vec<_> = finished.states.into_keys().collect();
        self.alternatives += alternatives.len();
        self.result_bytes += alternatives.iter().map(String::len).sum::<usize>()
            + self.path.len() * 64
            + alternatives.len() * 8
            + 256;
        if self.alternatives > MAX_ALTERNATIVES || self.result_bytes > MAX_OUTPUT {
            return Err(resource(
                "Beider–Morse total alternatives or result byte limit exceeded",
            ));
        }
        let encoded = alternatives.join("|");
        let names = if languages == ANY {
            Vec::new()
        } else {
            corpus::names(languages).collect()
        };
        self.groups.push(Group {
            path: self.path.clone(),
            table_language,
            languages: names,
            alternatives,
        });
        Ok(encoded)
    }
}

fn append(output: &mut String, fragment: &str) -> Result<(), OperationError> {
    if fragment.len() > MAX_OUTPUT - output.len() {
        return Err(resource(
            "Beider–Morse encoding exceeds its result byte limit",
        ));
    }
    output.push_str(fragment);
    Ok(())
}

fn assemble(left: &str, separator: &str, right: &str) -> Result<String, OperationError> {
    let mut result = String::new();
    append(&mut result, left)?;
    append(&mut result, separator)?;
    append(&mut result, right)?;
    Ok(result)
}

fn normalize(name: &str) -> Result<String, OperationError> {
    if name.len() > 1024 || name.chars().take(257).count() > 256 {
        return Err(resource(
            "Beider–Morse name exceeds 256 scalars or 1024 UTF-8 bytes",
        ));
    }
    for (offset, c) in name.char_indices() {
        if !separators(c)
            && !matches!(c, '-' | '\'')
            && !c.to_lowercase().all(|c| CORPUS.accepts(c))
        {
            return Err(
                invalid("Beider–Morse name contains an unmapped scalar").offset(offset as u64)
            );
        }
    }
    Ok(name.to_lowercase())
}

fn execute(request: &Request) -> Result<Report, OperationError> {
    let name = normalize(&request.name)?;
    let explicit = match &request.language {
        LanguageSelection::Guess => None,
        LanguageSelection::Any => Some(ANY),
        LanguageSelection::Explicit { languages } => Some(explicit(request.name_type, languages)?),
    };
    let mut engine = Engine {
        request,
        explicit,
        groups: Vec::new(),
        path: Vec::new(),
        alternatives: 0,
        result_bytes: 0,
        work: Work {
            remaining: MAX_WORK,
            peak: 0,
        },
    };
    let encoding = engine.encode(&name)?;
    let report = Report {
        profile: PROFILE,
        profile_sha256: corpus::DIGEST,
        name_type: request.name_type,
        rule_type: request.rule_type,
        concatenate: request.concatenate,
        encoding,
        groups: engine.groups,
        work_units: MAX_WORK - engine.work.remaining,
        peak_alternatives: engine.work.peak,
    };
    check_report(&report, MAX_OUTPUT)?;
    Ok(report)
}

/// Derive rule-compatible alternatives and retain explicit word and prefix branches.
pub fn encode(request: &Request) -> Result<Report, OperationError> {
    execute(request).map_err(|error| error.operation(OP))
}
