//! Bounded Daitch–Mokotoff branching under the Apache Commons Codec extension.
use super::{check_report, invalid, resource};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

const OP: &str = "text.daitch_mokotoff";
const PROFILE: &str = "apache-commons-codec-1.15-dm";
const DATA: &str = include_str!("../../../data/commons-codec-1.15/dmrules.txt");
const MAX_BRANCHES: usize = 4096;
const MAX_UPDATES: usize = 65_536;
const CODE: usize = 6;
const _: () = assert!(DATA.len() <= 1 << 20);

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub name: String,
    #[serde(default)]
    pub folding: bool,
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub profile: &'static str,
    pub folding: bool,
    pub codes: Vec<String>,
}

struct Rule {
    pattern: &'static str,
    values: [&'static str; 3],
}

#[derive(Default)]
struct Rules {
    indexed: BTreeMap<char, Vec<Rule>>,
    folding: BTreeMap<char, char>,
}

fn only_char(input: &str) -> char {
    let mut chars = input.chars();
    let value = chars.next().expect("nonempty audited rule column");
    assert!(chars.next().is_none());
    value
}

static RULES: LazyLock<Rules> = LazyLock::new(|| {
    let mut result = Rules::default();
    let mut comment = false;
    for row in DATA.lines().map(str::trim) {
        if row.starts_with("/*") {
            comment = true;
        }
        if comment {
            if row.ends_with("*/") {
                comment = false;
            }
            continue;
        }
        let row = row.split("//").next().unwrap().trim();
        if row.is_empty() {
            continue;
        }
        if let Some((from, to)) = row.split_once('=') {
            result.folding.insert(only_char(from), only_char(to));
        } else {
            let fields: Vec<_> = row
                .split_whitespace()
                .map(|field| field.trim_matches('"'))
                .collect();
            let [pattern, initial, before_vowel, other] =
                fields.try_into().expect("audited rule columns");
            let first = pattern.chars().next().expect("nonempty rule pattern");
            assert!(
                [initial, before_vowel, other]
                    .into_iter()
                    .all(|field: &str| { field.bytes().all(|b| b.is_ascii_digit() || b == b'|') })
            );
            result.indexed.entry(first).or_default().push(Rule {
                pattern,
                values: [initial, before_vowel, other],
            });
        }
    }
    for rules in result.indexed.values_mut() {
        rules.sort_by_key(|rule| std::cmp::Reverse(rule.pattern.chars().count()));
    }
    result
});

#[derive(Clone, Copy, Default)]
struct Branch {
    prefix: [u8; CODE],
    used: usize,
    last: Option<&'static str>,
}

impl Branch {
    fn append(self, next: &'static str, forced: bool) -> Self {
        let mut branch = Self {
            last: Some(next),
            ..self
        };
        if forced || self.last.is_none_or(|prior| !prior.ends_with(next)) {
            let add = next.len().min(CODE - self.used);
            branch.prefix[self.used..self.used + add].copy_from_slice(&next.as_bytes()[..add]);
            branch.used += add;
        }
        branch
    }

    fn finish(mut self) -> String {
        self.prefix[self.used..].fill(b'0');
        String::from_utf8(self.prefix.to_vec()).expect("only ASCII digit replacements")
    }
}

fn prepare(name: &str, folding: bool) -> Result<String, OperationError> {
    if name.len() > 1024 || name.chars().take(257).count() > 256 {
        return Err(resource(
            "Daitch–Mokotoff name exceeds 256 scalars or 1024 UTF-8 bytes",
        ));
    }
    let mut text = String::with_capacity(name.len());
    for (offset, c) in name.char_indices() {
        let mut lower = c.to_lowercase();
        let first = lower.next().expect("case traversal is nonempty");
        let value = if folding {
            RULES.folding.get(&first).copied().unwrap_or(first)
        } else {
            first
        };
        if lower.next().is_some()
            || !value.is_ascii_alphabetic() && !RULES.indexed.contains_key(&value)
        {
            return Err(
                invalid("Daitch–Mokotoff name contains an unmapped scalar").offset(offset as u64)
            );
        }
        text.push(value);
    }
    Ok(text)
}

fn charge(remaining: &mut usize) -> Result<(), OperationError> {
    *remaining = remaining
        .checked_sub(1)
        .ok_or_else(|| resource("Daitch–Mokotoff rule-state budget exhausted"))?;
    Ok(())
}

fn branches(
    text: &str,
    branch_limit: usize,
    work_limit: usize,
) -> Result<Vec<Branch>, OperationError> {
    let mut states = vec![Branch::default()];
    let mut remaining = work_limit;
    let mut previous = None;
    let mut suffix = text;
    while let Some(first) = suffix.chars().next() {
        let mut matched = None;
        if let Some(candidates) = RULES.indexed.get(&first) {
            for rule in candidates {
                charge(&mut remaining)?;
                if suffix.starts_with(rule.pattern) {
                    matched = Some(rule);
                    break;
                }
            }
        }
        let Some(rule) = matched else {
            // The audited A–Z vocabulary and every allowed mapped scalar have a fallback rule.
            return Err(invalid("Daitch–Mokotoff rule pattern has no match"));
        };
        suffix = &suffix[rule.pattern.len()..];
        let condition = if previous.is_none() {
            0
        } else if suffix
            .as_bytes()
            .first()
            .is_some_and(|b| b"aeiou".contains(b))
        {
            1
        } else {
            2
        };
        let forced = matches!((previous, first), (Some('m'), 'n') | (Some('n'), 'm'));
        let mut next = Vec::new();
        let mut seen = BTreeSet::new();
        for state in states {
            for replacement in rule.values[condition].split('|') {
                charge(&mut remaining)?;
                let branch = state.append(replacement, forced);
                // Reference branches compare their digit prefix; the first
                // compatible last replacement wins when prefixes coincide.
                if seen.contains(&branch.prefix) {
                    continue;
                }
                if next.len() == branch_limit {
                    return Err(resource("Daitch–Mokotoff live branch limit exceeded"));
                }
                seen.insert(branch.prefix);
                next.push(branch);
            }
        }
        states = next;
        previous = Some(first);
    }
    Ok(states)
}

fn encode_inner(request: &Request) -> Result<Report, OperationError> {
    let text = prepare(&request.name, request.folding)?;
    let mut codes: Vec<_> = branches(&text, MAX_BRANCHES, MAX_UPDATES)?
        .into_iter()
        .map(Branch::finish)
        .collect();
    codes.sort_unstable();
    codes.dedup();
    let report = Report {
        profile: PROFILE,
        folding: request.folding,
        codes,
    };
    check_report(&report, 1 << 20)?;
    Ok(report)
}

/// Return unique, lexically ordered six-digit codes for one exact name.
pub fn encode(request: &Request) -> Result<Report, OperationError> {
    encode_inner(request).map_err(|error| error.operation(OP))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_and_rule_budgets_reject_before_retaining_extra_state() {
        assert_eq!(branches("c", 2, 16).unwrap().len(), 2);
        assert!(branches("c", 1, 16).is_err());
        assert_eq!(branches("a", 1, 6).unwrap().len(), 1);
        assert!(branches("a", 1, 5).is_err());
        assert!(branches("ck".repeat(128).as_str(), MAX_BRANCHES, MAX_UPDATES).is_ok());
    }
}
