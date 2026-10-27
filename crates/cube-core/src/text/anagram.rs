//! Exact unordered word multisets from a caller-owned dictionary. No corpus or I/O.
use super::{
    lexical::{self, Entry, charge, error},
    multiset::{self, Profile},
    statistics,
};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
use std::collections::BTreeMap;
pub const MAX_ENTRIES: usize = 10000;
pub const MAX_DICTIONARY_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_WORK: usize = 1_000_000;
pub const MAX_MEMORY: usize = 32 * 1024 * 1024;
pub const MAX_REPORT: usize = 8 * 1024 * 1024;
const OP: &str = "text.anagram.search";
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct Options {
    pub max_words: usize,
    pub reuse: bool,
    pub max_results: usize,
    pub max_work: usize,
    pub max_memory_bytes: usize,
    pub max_report_bytes: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            max_words: 8,
            reuse: false,
            max_results: 100,
            max_work: MAX_WORK,
            max_memory_bytes: MAX_MEMORY,
            max_report_bytes: MAX_REPORT,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Report {
    pub profile: Profile,
    pub normalization_unicode_version: (u8, u8, u8),
    pub options: Options,
    pub dictionary: Vec<Entry>,
    pub solutions: Vec<Vec<usize>>,
    pub complete: bool,
    pub stop_reason: Option<&'static str>,
    pub nodes: usize,
    pub work: usize,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
struct Search<'a> {
    counts: &'a [BTreeMap<char, u64>],
    lengths: &'a [u64],
    options: Options,
    work: usize,
    nodes: usize,
    solutions: Vec<Vec<usize>>,
    stop: Option<&'static str>,
}
impl Search<'_> {
    fn visit(
        &mut self,
        residual: &mut BTreeMap<char, u64>,
        left: u64,
        start: usize,
        path: &mut Vec<usize>,
    ) {
        if self.stop.is_some() {
            return;
        }
        if self.work == self.options.max_work {
            self.stop = Some("work_limit");
            return;
        }
        self.work += 1;
        self.nodes += 1;
        if left == 0 {
            if self.solutions.len() == self.options.max_results {
                self.stop = Some("result_limit");
                return;
            }
            let n = path.len() + 1;
            if self.work + n > self.options.max_work {
                self.stop = Some("work_limit");
                return;
            }
            self.work += n;
            self.solutions.push(path.clone());
            return;
        }
        if path.len() == self.options.max_words {
            return;
        }
        for i in start..self.counts.len() {
            let c = &self.counts[i];
            let cost = 1 + 3 * c.len();
            if self.work + cost > self.options.max_work {
                self.stop = Some("work_limit");
                return;
            }
            self.work += cost;
            if !multiset::consume(residual, c) {
                continue;
            }
            let used = self.lengths[i];
            path.push(i);
            self.visit(
                residual,
                left - used,
                if self.options.reuse { i } else { i + 1 },
                path,
            );
            path.pop();
            for (&ch, &n) in c {
                *residual.get_mut(&ch).expect("consumed scalar exists") += n;
            }
            if self.stop.is_some() {
                return;
            }
        }
    }
}
/// Deterministic first-occurrence dictionary order; alias indexes refer to the
/// original list. A result contains indexes into `dictionary`, not permutations.
/// Preprocessing exhaustion is an error; search exhaustion returns an incomplete
/// prefix. The result cap is proven by looking for one extra solution.
pub fn search(
    source: &str,
    words: &[String],
    profile: Profile,
    options: Options,
) -> Result<Report, OperationError> {
    if !(1..=8).contains(&options.max_words)
        || options.max_results > 1000
        || options.max_work > MAX_WORK
        || options.max_memory_bytes > MAX_MEMORY
        || options.max_report_bytes > MAX_REPORT
    {
        return Err(error(
            OP,
            ErrorCode::InvalidParameter,
            "options",
            "anagram options exceed supported ceilings",
        ));
    }
    super::check_input(source, OP)?;
    if words.len() > MAX_ENTRIES {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "dictionary",
            "at most 10000 entries",
        ));
    }
    let bytes = words
        .iter()
        .try_fold(0usize, |n, s| n.checked_add(s.len()))
        .filter(|n| *n <= MAX_DICTIONARY_BYTES)
        .ok_or_else(|| {
            error(
                OP,
                ErrorCode::ResourceLimit,
                "dictionary",
                "aggregate dictionary exceeds 4 MiB",
            )
        })?;
    // Cover one shared normalization temporary, a prepared source Vec (including
    // spare capacity), its <=10000 count nodes, canonical strings/alias vectors,
    // dictionary count nodes and bounded output paths before retaining them.
    let normalized = !matches!(profile.normalization, statistics::Normalization::None);
    let report = 4096 + bytes * 6 + words.len() * 128 + options.max_results * 256;
    let source_scalars = if normalized {
        statistics::MAX_SCALARS
    } else {
        source.len()
    };
    let factor = if normalized { 18 } else { 1 };
    let memory = 8192
        + source.len()
        + source_scalars * 8
        + source_scalars.min(statistics::MAX_DISTINCT) * 128
        + if normalized { super::MAX_OUTPUT } else { 0 }
        + bytes * 3
        + words
            .iter()
            .map(|w| (w.len() * factor).min(64) * 128 + 1024)
            .sum::<usize>()
        + report * 2;
    if memory > options.max_memory_bytes || report > options.max_report_bytes {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "limits",
            "conservative anagram scratch/report exceeds budget",
        ));
    }
    let mut work = 0;
    let source = lexical::prepare(source, profile, &mut work, options.max_work, OP)?;
    charge(&mut work, source.len(), options.max_work, OP)?;
    let mut counts = statistics::scalar_counts(&source, OP)?;
    let left = source.len() as u64;
    let dictionary = lexical::entries(words, profile, 64, &mut work, options.max_work, OP)?;
    let actual_report = 4096
        + dictionary
            .iter()
            .map(|e| crate::pipeline::json::string_bytes(&e.text) + 128 + e.aliases.len() * 24)
            .sum::<usize>()
        + options.max_results * 256;
    let report = report.max(actual_report);
    if report > options.max_report_bytes || memory + actual_report > options.max_memory_bytes {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "limits",
            "prepared inventory report exceeds budget",
        ));
    }
    let memory = memory + actual_report;
    let mut inventories = Vec::with_capacity(dictionary.len());
    let mut lengths = Vec::with_capacity(dictionary.len());
    for e in &dictionary {
        let v: Vec<char> = e.text.chars().collect();
        charge(&mut work, v.len(), options.max_work, OP)?;
        lengths.push(v.len() as u64);
        inventories.push(statistics::scalar_counts(&v, OP)?);
    }
    let mut s = Search {
        counts: &inventories,
        lengths: &lengths,
        options,
        work,
        nodes: 0,
        solutions: Vec::new(),
        stop: None,
    };
    s.visit(&mut counts, left, 0, &mut Vec::new());
    Ok(Report {
        profile,
        normalization_unicode_version: super::UNICODE_VERSION,
        options,
        dictionary,
        solutions: s.solutions,
        complete: s.stop.is_none(),
        stop_reason: s.stop,
        nodes: s.nodes,
        work: s.work,
        memory_bytes_upper_bound: memory,
        report_bytes_upper_bound: report,
    })
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "normalization",
        description: "Explicit bounded shared Unicode normalization.",
        required: false,
        default: Some(ParameterValue::Text("none")),
        constraints: ParameterType::Text {
            choices: &["none", "NFC", "NFD", "NFKC", "NFKD"],
        },
    },
    Parameter {
        name: "case",
        description: "Preserve exact scalars or lowercase ASCII.",
        required: false,
        default: Some(ParameterValue::Text("preserve")),
        constraints: ParameterType::Text {
            choices: &["preserve", "ascii_lower"],
        },
    },
    Parameter {
        name: "filter",
        description: "Keep all scalars or ASCII letters after preparation.",
        required: false,
        default: Some(ParameterValue::Text("all")),
        constraints: ParameterType::Text {
            choices: &["all", "ascii_letters"],
        },
    },
    Parameter {
        name: "dictionary",
        description: "Explicit ordered bounded JSON string array.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "max_words",
        description: "Maximum words per unordered solution.",
        required: false,
        default: Some(ParameterValue::Unsigned(8)),
        constraints: ParameterType::Unsigned { min: 1, max: 8 },
    },
    Parameter {
        name: "reuse",
        description: "Permit repeated use of one canonical word.",
        required: false,
        default: Some(ParameterValue::Boolean(false)),
        constraints: ParameterType::Boolean,
    },
    Parameter {
        name: "max_results",
        description: "Listed solutions; one-extra-solution proves incompleteness.",
        required: false,
        default: Some(ParameterValue::Unsigned(100)),
        constraints: ParameterType::Unsigned { min: 0, max: 1000 },
    },
    Parameter {
        name: "max_work",
        description: "Charged validation, index and execution work; exhaustion is explicit.",
        required: false,
        default: Some(ParameterValue::Unsigned(1000000)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 1000000,
        },
    },
    Parameter {
        name: "max_memory_bytes",
        description: "Conservative aggregate scratch and retained-report allocation bound.",
        required: false,
        default: Some(ParameterValue::Unsigned(33554432)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 33554432,
        },
    },
    Parameter {
        name: "max_report_bytes",
        description: "Conservative complete compact JSON report bound.",
        required: false,
        default: Some(ParameterValue::Unsigned(8388608)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 8388608,
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "text.anagram.search",
    category: "text",
    description: "Exact caller-dictionary unordered anagrams; 10000 entries, 64 scalars/entry, 4 MiB dictionary, 1 MiB source; incomplete prefixes are explicit.",
    input_type: "text_and_dictionary",
    output_type: "anagram_report",
    command: "text anagram search",
    example: "cube text anagram search ab --dictionary '[\"ab\",\"a\",\"b\"]",
    parameters: PARAMETERS,
    deterministic: true,
    reversible: None,
}];
