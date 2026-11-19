//! Import and restrict the pinned rule grammar without a caller regex engine.
use super::{Matter, Type, Work};
use crate::errors::OperationError;
use std::{collections::BTreeMap, sync::LazyLock};

pub(super) const DATA: &str = include_str!("../../../../data/commons-codec-1.15/bm-rules.json");
pub(super) const DIGEST: &str = "834d0ae444e2aea76b808c242d7541ff1585c1fc29ae2ba47cd758974ca9a4a3";
const _: () = assert!(DATA.len() <= 512 << 10);

pub(super) type Mask = u32;
const LANGUAGES: &[&str] = &[
    "any",
    "arabic",
    "cyrillic",
    "czech",
    "dutch",
    "english",
    "french",
    "german",
    "greek",
    "greeklatin",
    "hebrew",
    "hungarian",
    "italian",
    "polish",
    "portuguese",
    "romanian",
    "russian",
    "spanish",
    "turkish",
    // Source typos can occur in rule expressions but are never request languages.
    "ebrew",
    "german,",
];
pub(super) const ANY: Mask = (1 << LANGUAGES.len()) - 1;

pub(super) fn bit(language: &str) -> Option<Mask> {
    LANGUAGES
        .iter()
        .position(|&name| name == language)
        .map(|i| 1 << i)
}

fn mask(names: impl IntoIterator<Item = impl AsRef<str>>) -> Mask {
    names.into_iter().fold(0, |set, name| {
        set | bit(name.as_ref()).expect("audited language")
    })
}

pub(super) fn names(mask: Mask) -> impl Iterator<Item = &'static str> {
    LANGUAGES
        .iter()
        .enumerate()
        .filter_map(move |(i, &name)| (mask & (1 << i) != 0).then_some(name))
}

#[derive(serde::Deserialize)]
struct Raw {
    languages: BTreeMap<String, Vec<String>>,
    guess: BTreeMap<String, Vec<(String, Vec<String>, bool)>>,
    tables: BTreeMap<String, BTreeMap<char, Vec<[String; 4]>>>,
}

enum Atom {
    Exact(char),
    Set { values: Box<[char]>, negative: bool },
}

impl Atom {
    fn matches(&self, scalar: char, work: &mut Work) -> Result<bool, OperationError> {
        match self {
            Self::Exact(expected) => {
                work.charge(1)?;
                Ok(*expected == scalar)
            }
            Self::Set { values, negative } => {
                work.charge(1 + values.len())?;
                Ok(values.contains(&scalar) != *negative)
            }
        }
    }
}

struct Pattern {
    start: bool,
    end: bool,
    atoms: Vec<Atom>,
}

impl Pattern {
    fn new(source: &str, apache_context: bool) -> Self {
        let start = source.starts_with('^');
        let source = source.strip_prefix('^').unwrap_or(source);
        let end = source.ends_with('$');
        let source = source.strip_suffix('$').unwrap_or(source);
        let mut atoms = Vec::new();
        let mut symbols = source.chars();
        while let Some(c) = symbols.next() {
            if c == '[' {
                let mut members = symbols
                    .by_ref()
                    .take_while(|&c| c != ']')
                    .collect::<Vec<_>>();
                let negative = members.first() == Some(&'^');
                if negative {
                    members.remove(0);
                }
                assert!(!members.is_empty() && !members.iter().any(|c| matches!(c, '-' | '[')));
                atoms.push(Atom::Set {
                    values: members.into_boxed_slice(),
                    negative,
                });
            } else {
                assert!(
                    apache_context
                        || !matches!(
                            c,
                            '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '|' | '{' | '}'
                        ),
                    "unexpected language grammar in pinned corpus"
                );
                atoms.push(Atom::Exact(c));
            }
        }
        Self { start, end, atoms }
    }

    fn left(source: &str) -> Self {
        Self::new(&format!("{source}$"), true)
    }
    fn right(source: &str) -> Self {
        Self::new(&format!("^{source}"), true)
    }

    fn prefix(&self, text: &str, finish: bool, work: &mut Work) -> Result<bool, OperationError> {
        let mut input = text.chars();
        for atom in &self.atoms {
            let Some(c) = input.next() else {
                return Ok(false);
            };
            if !atom.matches(c, work)? {
                return Ok(false);
            }
        }
        Ok(!finish || input.next().is_none())
    }

    fn matches(&self, text: &str, work: &mut Work) -> Result<bool, OperationError> {
        work.charge(1)?;
        match (self.start, self.end) {
            (true, exact) => self.prefix(text, exact, work),
            (false, true) => {
                let mut input = text.chars().rev();
                for atom in self.atoms.iter().rev() {
                    let Some(c) = input.next() else {
                        return Ok(false);
                    };
                    if !atom.matches(c, work)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            (false, false) => {
                if self.atoms.is_empty() {
                    return Ok(true);
                }
                for (offset, _) in text.char_indices() {
                    work.charge(1)?;
                    if self.prefix(&text[offset..], false, work)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }
}

pub(super) struct Alternative {
    pub(super) text: String,
    pub(super) languages: Mask,
}

impl Alternative {
    fn new(source: &str) -> Self {
        if let Some((text, languages)) = source.split_once('[') {
            let languages = languages
                .strip_suffix(']')
                .expect("audited phoneme restriction");
            Self {
                text: text.to_owned(),
                languages: mask(languages.split('+')),
            }
        } else {
            Self {
                text: source.to_owned(),
                languages: ANY,
            }
        }
    }
}

pub(super) struct Rule {
    pub(super) pattern: String,
    left: Pattern,
    right: Pattern,
    pub(super) alternatives: Vec<Alternative>,
}

impl Rule {
    fn new([pattern, left, right, expression]: [String; 4]) -> Self {
        let alternatives = if let Some(body) = expression.strip_prefix('(') {
            let body = body
                .strip_suffix(')')
                .expect("audited alternative expression");
            body.split('|').map(Alternative::new).collect()
        } else {
            vec![Alternative::new(&expression)]
        };
        Self {
            pattern,
            left: Pattern::left(&left),
            right: Pattern::right(&right),
            alternatives,
        }
    }

    pub(super) fn matches(
        &self,
        text: &str,
        offset: usize,
        work: &mut Work,
    ) -> Result<bool, OperationError> {
        work.charge(1 + self.pattern.len())?;
        Ok(text[offset..].starts_with(&self.pattern)
            && self.left.matches(&text[..offset], work)?
            && self
                .right
                .matches(&text[offset + self.pattern.len()..], work)?)
    }
}

struct LanguageRule {
    pattern: Pattern,
    languages: Mask,
    accept: bool,
}

pub(super) type Table = BTreeMap<char, Vec<Rule>>;

pub(super) struct Corpus {
    possible: BTreeMap<String, Mask>,
    hints: BTreeMap<String, Vec<LanguageRule>>,
    tables: BTreeMap<String, Table>,
    input_letters: BTreeMap<char, ()>,
}

pub(super) static CORPUS: LazyLock<Corpus> = LazyLock::new(|| {
    let raw: Raw = serde_json::from_str(DATA).expect("pinned, validated Beider–Morse corpus");
    let rows = raw
        .tables
        .values()
        .flat_map(|table| table.values().flat_map(|rows| rows.iter()))
        .count();
    assert!(
        raw.tables.len() <= 128
            && rows <= 5000
            && raw.guess.values().map(Vec::len).sum::<usize>() <= 512
    );
    let possible = raw
        .languages
        .into_iter()
        .map(|(kind, list)| (kind, mask(list)))
        .collect();
    let hints = raw
        .guess
        .into_iter()
        .map(|(kind, rows)| {
            let rows = rows
                .into_iter()
                .map(|(pattern, names, accept)| LanguageRule {
                    pattern: Pattern::new(&pattern, false),
                    languages: mask(names),
                    accept,
                })
                .collect();
            (kind, rows)
        })
        .collect();
    let input_letters = raw
        .tables
        .iter()
        .filter(|(name, _)| name.contains("_rules_"))
        .flat_map(|(_, table)| table.keys().copied())
        .map(|key| (key, ()))
        .collect();
    let tables = raw
        .tables
        .into_iter()
        .map(|(key, table)| {
            let compiled = table
                .into_iter()
                .map(|(key, rows)| (key, rows.into_iter().map(Rule::new).collect()))
                .collect();
            (key, compiled)
        })
        .collect();
    Corpus {
        possible,
        hints,
        tables,
        input_letters,
    }
});

impl Corpus {
    pub(super) fn possible(&self, kind: Type) -> Mask {
        self.possible[kind.key()]
    }

    pub(super) fn accepts(&self, ch: char) -> bool {
        self.input_letters.contains_key(&ch)
    }

    pub(super) fn infer(
        &self,
        kind: Type,
        text: &str,
        work: &mut Work,
    ) -> Result<Mask, OperationError> {
        let mut languages = self.possible(kind);
        for rule in &self.hints[kind.key()] {
            if rule.pattern.matches(text, work)? {
                if rule.accept {
                    languages &= rule.languages;
                } else {
                    languages &= !rule.languages;
                }
            }
        }
        Ok(if languages == 0 { ANY } else { languages })
    }

    pub(super) fn table(&self, kind: Type, mode: Option<Matter>, language: &str) -> &Table {
        let mode = mode.map_or("rules", Matter::key);
        &self.tables[&format!("{}_{mode}_{language}", kind.key())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    fn immutable_corpus_matches_published_digest_and_all_provenance_tables_load() {
        assert_eq!(hex::encode(Sha256::digest(DATA)), DIGEST);
        assert_eq!(CORPUS.tables.len(), 114);
        assert!(CORPUS.accepts('š'));
        assert!(CORPUS.accepts('м'));
    }
}
