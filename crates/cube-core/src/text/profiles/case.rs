//! Unicode case folding and General_Category filtering with reported data revisions.
use super::{check_report, parameter, resource};
use crate::errors::OperationError;
use crate::text::{MAX_OUTPUT, check_input, unicode::CATEGORY_UNICODE_VERSION};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::BTreeMap, sync::LazyLock};
use unicode_general_category::{GeneralCategory as Category, get_general_category};

const OP: &str = "text.unicode_transform";
const DATA: &str = include_str!("../../../data/unicode-17/CaseFolding.txt");
const CASE_VERSION: (u64, u64, u64) = (17, 0, 0);
const MAX_WORK: usize = 8 << 20;

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FoldProfile {
    #[default]
    Full,
    Simple,
    Turkic,
}

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    #[default]
    Include,
    Exclude,
}

const CATEGORIES: [Category; 30] = [
    Category::UppercaseLetter,
    Category::LowercaseLetter,
    Category::TitlecaseLetter,
    Category::ModifierLetter,
    Category::OtherLetter,
    Category::NonspacingMark,
    Category::SpacingMark,
    Category::EnclosingMark,
    Category::DecimalNumber,
    Category::LetterNumber,
    Category::OtherNumber,
    Category::ConnectorPunctuation,
    Category::DashPunctuation,
    Category::OpenPunctuation,
    Category::ClosePunctuation,
    Category::InitialPunctuation,
    Category::FinalPunctuation,
    Category::OtherPunctuation,
    Category::MathSymbol,
    Category::CurrencySymbol,
    Category::ModifierSymbol,
    Category::OtherSymbol,
    Category::SpaceSeparator,
    Category::LineSeparator,
    Category::ParagraphSeparator,
    Category::Control,
    Category::Format,
    Category::Surrogate,
    Category::PrivateUse,
    Category::Unassigned,
];

#[derive(Deserialize)]
#[serde(transparent)]
pub struct CategoryName(String);

impl JsonSchema for CategoryName {
    fn schema_name() -> Cow<'static, str> {
        "GeneralCategoryName".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        let names = CATEGORIES.map(|category| category.abbreviation());
        json_schema!({"type": "string", "enum": names})
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "transform", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    AsciiUpper {
        text: String,
    },
    AsciiLower {
        text: String,
    },
    Casefold {
        text: String,
        #[serde(default)]
        profile: FoldProfile,
    },
    CategoryFilter {
        text: String,
        #[schemars(length(max = 30))]
        categories: Vec<CategoryName>,
        #[serde(default)]
        mode: Mode,
    },
}

#[derive(Serialize, JsonSchema)]
pub struct Report {
    pub text: String,
    pub profile: &'static str,
    pub unicode_version: Option<(u64, u64, u64)>,
}

#[derive(Clone, Copy, Default)]
struct Fold {
    full: [char; 3],
    simple: [char; 3],
    turkic: [char; 3],
}

fn scalar(hex: &str) -> char {
    char::from_u32(u32::from_str_radix(hex, 16).expect("audited UCD code point"))
        .expect("audited UCD scalar")
}

static FOLD: LazyLock<Vec<(char, Fold)>> = LazyLock::new(|| {
    let mut table = BTreeMap::<char, Fold>::new();
    for row in DATA
        .lines()
        .map(|line| line.split('#').next().unwrap().trim())
        .filter(|row| !row.is_empty())
    {
        let mut fields = row.split(';').map(str::trim);
        let key = scalar(fields.next().expect("UCD source column"));
        let status = fields.next().expect("UCD status column");
        let mut values = [char::default(); 3];
        for (index, value) in fields
            .next()
            .expect("UCD mapping column")
            .split_whitespace()
            .enumerate()
        {
            values[index] = scalar(value);
        }
        let entry = table.entry(key).or_default();
        match status {
            "C" => {
                entry.full = values;
                entry.simple = values;
            }
            "F" => entry.full = values,
            "S" => entry.simple = values,
            "T" => entry.turkic = values,
            _ => unreachable!("audited UCD mapping status"),
        }
    }
    table.into_iter().collect()
});

fn mapping(c: char, profile: FoldProfile, work: &mut usize) -> [char; 3] {
    let identity = [c, '\0', '\0'];
    *work += 1;
    if c.is_ascii() {
        let folded = if c == 'I' && matches!(profile, FoldProfile::Turkic) {
            'ı'
        } else {
            c.to_ascii_lowercase()
        };
        return [folded, '\0', '\0'];
    }
    let Ok(index) = FOLD.binary_search_by(|&(key, _)| {
        *work += 1;
        key.cmp(&c)
    }) else {
        return identity;
    };
    let (_, row) = FOLD[index];
    let mapped = match profile {
        FoldProfile::Full => row.full,
        FoldProfile::Simple => row.simple,
        FoldProfile::Turkic if row.turkic[0] != '\0' => row.turkic,
        FoldProfile::Turkic => row.full,
    };
    if mapped[0] == '\0' { identity } else { mapped }
}

fn fold(text: &str, profile: FoldProfile) -> Result<String, OperationError> {
    let mut result = String::with_capacity(text.len());
    let mut work = 0;
    for c in text.chars() {
        let [first, second, third] = mapping(c, profile, &mut work);
        work += 1 + usize::from(second != '\0') + usize::from(third != '\0');
        if work > MAX_WORK {
            return Err(resource("case folding exceeds its work limit"));
        }
        result.push(first);
        if second != '\0' {
            result.push(second);
        }
        if third != '\0' {
            result.push(third);
        }
        if result.len() > MAX_OUTPUT {
            return Err(resource("case fold exceeds 8 MiB of UTF-8 output"));
        }
    }
    Ok(result)
}

fn category_key(name: &str) -> usize {
    let [upper, lower] = name.as_bytes() else {
        unreachable!("validated category abbreviation")
    };
    usize::from(upper - b'A') * 26 + usize::from(lower - b'a')
}

fn filter(text: &str, categories: &[CategoryName], mode: Mode) -> Result<String, OperationError> {
    if categories.len() > CATEGORIES.len() {
        return Err(parameter("category names must be unique"));
    }
    let mut selected = [false; 26 * 26];
    for CategoryName(name) in categories {
        if !CATEGORIES
            .iter()
            .any(|category| category.abbreviation() == name)
        {
            return Err(parameter(
                "use a Unicode two-letter General_Category abbreviation",
            ));
        }
        if std::mem::replace(&mut selected[category_key(name)], true) {
            return Err(parameter("category names must be unique"));
        }
    }
    let include = matches!(mode, Mode::Include);
    Ok(text
        .chars()
        .filter(|&c| selected[category_key(get_general_category(c).abbreviation())] == include)
        .collect())
}

fn transform(request: &Request) -> Result<Report, OperationError> {
    let text = match request {
        Request::AsciiUpper { text }
        | Request::AsciiLower { text }
        | Request::Casefold { text, .. }
        | Request::CategoryFilter { text, .. } => text,
    };
    check_input(text, OP)?;
    let (text, profile, unicode_version) = match request {
        Request::AsciiUpper { .. } => (text.to_ascii_uppercase(), "ascii_upper", None),
        Request::AsciiLower { .. } => (text.to_ascii_lowercase(), "ascii_lower", None),
        Request::Casefold { profile, .. } => {
            let name = match profile {
                FoldProfile::Full => "casefold_full",
                FoldProfile::Simple => "casefold_simple",
                FoldProfile::Turkic => "casefold_turkic",
            };
            (fold(text, *profile)?, name, Some(CASE_VERSION))
        }
        Request::CategoryFilter {
            categories, mode, ..
        } => (
            filter(text, categories, *mode)?,
            "category_filter",
            Some(CATEGORY_UNICODE_VERSION),
        ),
    };
    let report = Report {
        text,
        profile,
        unicode_version,
    };
    check_report(&report, MAX_OUTPUT)?;
    Ok(report)
}

/// Apply exact ASCII casing, a pinned Unicode fold, or a General_Category filter.
pub fn apply(request: &Request) -> Result<Report, OperationError> {
    transform(request).map_err(|error| error.operation(OP))
}
