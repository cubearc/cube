//! Reproducible scalar statistics, never cipher identification.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
    text,
};
use serde::Serialize;
use std::{borrow::Cow, collections::BTreeMap};
pub const MAX_SCALARS: usize = 1024 * 1024;
pub const MAX_DISTINCT: usize = 10000;
pub const MAX_WORK: usize = 8 * 1024 * 1024;
fn invalid(name: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
        .operation("text.statistics")
        .parameter(name)
}
fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message).operation("text.statistics")
}
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub enum Normalization {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "NFC")]
    Nfc,
    #[serde(rename = "NFD")]
    Nfd,
    #[serde(rename = "NFKC")]
    Nfkc,
    #[serde(rename = "NFKD")]
    Nfkd,
}
impl Normalization {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "none" => Ok(Self::None),
            "NFC" => Ok(Self::Nfc),
            "NFD" => Ok(Self::Nfd),
            "NFKC" => Ok(Self::Nfkc),
            "NFKD" => Ok(Self::Nfkd),
            _ => Err(invalid(
                "normalization",
                "normalization must be none, NFC, NFD, NFKC or NFKD",
            )),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Nfc => "NFC",
            Self::Nfd => "NFD",
            Self::Nfkc => "NFKC",
            Self::Nfkd => "NFKD",
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaseMode {
    Preserve,
    AsciiLower,
}
impl CaseMode {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "preserve" => Ok(Self::Preserve),
            "ascii_lower" => Ok(Self::AsciiLower),
            _ => Err(invalid("case", "case must be preserve or ascii_lower")),
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TextFilter {
    All,
    AsciiLetters,
}
impl TextFilter {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "all" => Ok(Self::All),
            "ascii_letters" => Ok(Self::AsciiLetters),
            _ => Err(invalid("filter", "filter must be all or ascii_letters")),
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct Options {
    pub ngram_size: usize,
    pub normalization: Normalization,
    pub case: CaseMode,
    pub filter: TextFilter,
    pub max_results: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            ngram_size: 1,
            normalization: Normalization::None,
            case: CaseMode::Preserve,
            filter: TextFilter::All,
            max_results: 1000,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Frequency {
    pub ngram: String,
    pub count: u64,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Statistics {
    #[serde(flatten)]
    pub options: Options,
    pub unit: &'static str,
    pub normalization_unicode_version: (u8, u8, u8),
    pub scalar_count: usize,
    pub windows: usize,
    pub distinct_ngrams: usize,
    pub complete: bool,
    pub frequencies: Vec<Frequency>,
    pub index_of_coincidence: Option<f64>,
    pub shannon_entropy_bits: Option<f64>,
}
// Shared preparation/counter for complete inventories and statistics. The
// optional visit ceiling bounds filtering work without changing legacy behavior.
pub(crate) fn prepare(
    input: &str,
    normalization: Normalization,
    case: CaseMode,
    filter: TextFilter,
    max_visits: usize,
    op: &'static str,
) -> Result<(Vec<char>, usize), OperationError> {
    text::check_input(input, op)?;
    let normalized = match normalization {
        Normalization::None => Cow::Borrowed(input),
        mode => Cow::Owned(text::normalize(input, mode.name()).map_err(|e| e.operation(op))?),
    };
    let mut scalars = Vec::new();
    let mut visits = 0;
    for c in normalized.chars() {
        if visits == max_visits {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "text preparation visit budget exhausted",
            )
            .operation(op)
            .parameter("max_work"));
        }
        visits += 1;
        let c = match case {
            CaseMode::Preserve => c,
            CaseMode::AsciiLower => c.to_ascii_lowercase(),
        };
        if matches!(filter, TextFilter::AsciiLetters) && !c.is_ascii_alphabetic() {
            continue;
        }
        if scalars.len() == MAX_SCALARS {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "normalized/filtered text exceeds 1 Mi scalars",
            )
            .operation(op));
        }
        scalars.push(c);
    }
    Ok((scalars, visits))
}
pub(crate) fn scalar_counts(
    scalars: &[char],
    op: &'static str,
) -> Result<BTreeMap<char, u64>, OperationError> {
    let mut counts = BTreeMap::new();
    for c in scalars {
        if counts.len() == MAX_DISTINCT && !counts.contains_key(c) {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "character frequencies exceed 10000 distinct scalars",
            )
            .operation(op));
        }
        *counts.entry(*c).or_default() += 1;
    }
    Ok(counts)
}
/// Normalize only when requested, then apply ASCII case and filtering choices.
/// N-grams overlap and use Unicode scalars; counts sort descending, ties by text.
/// IC and entropy use single-scalar frequencies regardless of n. IC is undefined
/// below two scalars, entropy is undefined when empty and zero for one distinct
/// scalar. Floating results are f64 observations, not cryptographic guesses.
pub fn analyze(input: &str, options: Options) -> Result<Statistics, OperationError> {
    text::check_input(input, "text.statistics")?;
    if !(1..=16).contains(&options.ngram_size) {
        return Err(invalid("ngram_size", "ngram_size must be in 1..=16"));
    }
    if !(1..=MAX_DISTINCT).contains(&options.max_results) {
        return Err(invalid("max_results", "max_results must be in 1..=10000"));
    }
    let (scalars, _) = prepare(
        input,
        options.normalization,
        options.case,
        options.filter,
        usize::MAX,
        "text.statistics",
    )?;
    if scalars.len() * options.ngram_size > MAX_WORK {
        return Err(limit("statistics exceed 8 Mi scalar-window work units"));
    }
    let counts = scalar_counts(&scalars, "text.statistics")?;
    let n = scalars.len() as u64;
    let index_of_coincidence = if n < 2 {
        None
    } else {
        Some(counts.values().map(|c| c * (c - 1)).sum::<u64>() as f64 / (n * (n - 1)) as f64)
    };
    let shannon_entropy_bits = if n == 0 {
        None
    } else if counts.len() == 1 {
        Some(0.0)
    } else {
        Some(
            -counts
                .values()
                .map(|c| {
                    let p = *c as f64 / n as f64;
                    p * p.log2()
                })
                .sum::<f64>(),
        )
    };
    let mut ngrams = BTreeMap::<String, u64>::new();
    let mut windows = 0;
    for window in scalars.windows(options.ngram_size) {
        let key: String = window.iter().collect();
        if ngrams.len() == MAX_DISTINCT && !ngrams.contains_key(&key) {
            return Err(limit("n-grams exceed 10000 distinct values"));
        }
        *ngrams.entry(key).or_default() += 1;
        windows += 1;
    }
    let distinct_ngrams = ngrams.len();
    let mut frequencies: Vec<_> = ngrams
        .into_iter()
        .map(|(ngram, count)| Frequency { ngram, count })
        .collect();
    frequencies.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.ngram.cmp(&b.ngram)));
    frequencies.truncate(options.max_results);
    Ok(Statistics {
        options,
        unit: "unicode_scalars",
        normalization_unicode_version: text::UNICODE_VERSION,
        scalar_count: scalars.len(),
        windows,
        distinct_ngrams,
        complete: frequencies.len() == distinct_ngrams,
        frequencies,
        index_of_coincidence,
        shannon_entropy_bits,
    })
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "text.statistics",
    category: "text",
    description: "Bounded Unicode-scalar frequencies, overlapping n-grams, IC and entropy under explicit normalization/case/filter policies; numeric observations, never cipher identification.",
    input_type: "text",
    output_type: "text.statistics",
    parameters: &[
        Parameter {
            name: "ngram_size",
            description: "Overlapping scalar window width; IC/entropy still use individual scalars.",
            required: false,
            default: Some(ParameterValue::Unsigned(1)),
            constraints: ParameterType::Unsigned { min: 1, max: 16 },
        },
        Parameter {
            name: "normalization",
            description: "No normalization by default; otherwise reuse the versioned Unicode normalizer.",
            required: false,
            default: Some(ParameterValue::Text("none")),
            constraints: ParameterType::Text {
                choices: &["none", "NFC", "NFD", "NFKC", "NFKD"],
            },
        },
        Parameter {
            name: "case",
            description: "Preserve scalar case or lowercase ASCII A-Z only.",
            required: false,
            default: Some(ParameterValue::Text("preserve")),
            constraints: ParameterType::Text {
                choices: &["preserve", "ascii_lower"],
            },
        },
        Parameter {
            name: "filter",
            description: "Keep all scalars or retain ASCII letters only, after normalization/case handling.",
            required: false,
            default: Some(ParameterValue::Text("all")),
            constraints: ParameterType::Text {
                choices: &["all", "ascii_letters"],
            },
        },
        Parameter {
            name: "max_results",
            description: "Return the most frequent values, with explicit completeness; >10000 distinct values fail rather than silently losing counts.",
            required: false,
            default: Some(ParameterValue::Unsigned(1000)),
            constraints: ParameterType::Unsigned { min: 1, max: 10000 },
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "text statistics",
    example: "cube text statistics --text ABAB --ngram-size 2 --json",
}];
