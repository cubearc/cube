//! Unicode observations and scalar-aligned differences, without rendering claims.
use super::{MAX_INDICES, MAX_OUTPUT, check_input, mapping::ByteBudget};
use crate::errors::{ErrorCode, OperationError};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use unicode_general_category::{GeneralCategory, get_general_category};
/// General-category data is pinned independently of Rust's derived case/space
/// properties and Cube's existing normalization data.
pub const CATEGORY_UNICODE_VERSION: (u64, u64, u64) = unicode_general_category::UNICODE_VERSION;
pub const MAX_WORK: u64 = 128 * 1024 * 1024;
pub const MAX_NORMALIZED_SCALARS: usize = 2 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    pub max_results: usize,
    pub max_work: u64,
    pub max_output: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            max_results: 1000,
            max_work: MAX_WORK,
            max_output: MAX_OUTPUT,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Scalar {
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub scalar: char,
    pub codepoint: u32,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Observation {
    #[serde(flatten)]
    pub source: Scalar,
    pub case: &'static str,
    pub category: &'static str,
    pub non_ascii: bool,
    pub control: bool,
    pub whitespace: bool,
    pub format: bool,
    pub zero_width_candidate: bool,
    pub bidi_control: bool,
}
#[derive(Debug, Default, Serialize, schemars::JsonSchema)]
pub struct Counts {
    pub scalars: usize,
    pub uppercase: usize,
    pub lowercase: usize,
    pub titlecase: usize,
    pub uncased: usize,
    pub non_ascii: usize,
    pub control: usize,
    pub whitespace: usize,
    pub format: usize,
    pub zero_width_candidates: usize,
    pub bidi_controls: usize,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Inspection {
    pub derived_case_space_unicode_version: (u8, u8, u8),
    pub category_unicode_version: (u64, u64, u64),
    pub zero_width_profile: u32,
    pub counts: Counts,
    pub records: Vec<Observation>,
    pub complete: bool,
    pub work_units: u64,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Difference {
    pub scalar_index: usize,
    pub left: Option<Scalar>,
    pub right: Option<Scalar>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Differences {
    pub coordinate_space: &'static str,
    pub left_scalars: usize,
    pub right_scalars: usize,
    pub total_differences: usize,
    pub records: Vec<Difference>,
    pub complete: bool,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Comparison {
    pub exact_equal: bool,
    pub normalized_equal: Option<bool>,
    pub normalization: String,
    pub normalization_unicode_version: (u8, u8, u8),
    pub original: Differences,
    pub normalized: Option<Differences>,
    pub complete: bool,
    pub work_units: u64,
}
fn error(op: &'static str, code: ErrorCode, message: &str) -> OperationError {
    OperationError::new(code, message).operation(op)
}
fn validate(options: &Options, op: &'static str) -> Result<(), OperationError> {
    if options.max_results > MAX_INDICES
        || options.max_work > MAX_WORK
        || options.max_output > MAX_OUTPUT
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "listing/work/report limits exceed supported hard bounds",
        ));
    }
    Ok(())
}
fn charge(
    work: &mut u64,
    units: u64,
    options: &Options,
    op: &'static str,
) -> Result<(), OperationError> {
    *work = work
        .checked_add(units)
        .filter(|n| *n <= options.max_work)
        .ok_or_else(|| {
            error(
                op,
                ErrorCode::ResourceLimit,
                "Unicode scan exceeds work budget",
            )
        })?;
    Ok(())
}
fn report_bytes(
    value: &impl Serialize,
    options: &Options,
    op: &'static str,
) -> Result<usize, OperationError> {
    let mut budget = ByteBudget::new(options.max_output);
    serde_json::to_writer(&mut budget, value).map_err(|_| {
        error(
            op,
            ErrorCode::ResourceLimit,
            "Unicode report exceeds output budget",
        )
    })?;
    Ok(budget.written())
}
fn observation(scalar_index: usize, byte_offset: usize, c: char) -> Observation {
    let category = get_general_category(c);
    let case = if category == GeneralCategory::TitlecaseLetter {
        "titlecase"
    } else if c.is_uppercase() {
        "uppercase"
    } else if c.is_lowercase() {
        "lowercase"
    } else {
        "uncased"
    };
    Observation {
        source: Scalar {
            scalar_index,
            byte_offset,
            scalar: c,
            codepoint: c as u32,
        },
        case,
        category: category.abbreviation(),
        non_ascii: !c.is_ascii(),
        control: c.is_control(),
        whitespace: c.is_whitespace(),
        format: category == GeneralCategory::Format,
        // Named profile v1 is a finite observation set, not an exhaustive
        // renderer-width test: joiners/ZWSP/WORD JOINER/BOM and variation selectors.
        zero_width_candidate: matches!(c,'\u{200b}'..='\u{200d}'|'\u{2060}'|'\u{feff}'|'\u{fe00}'..='\u{fe0f}'|'\u{e0100}'..='\u{e01ef}'),
        bidi_control: matches!(c,'\u{61c}'|'\u{200e}'|'\u{200f}'|'\u{202a}'..='\u{202e}'|'\u{2066}'..='\u{2069}'),
    }
}
/// Inspect exact original scalars. Scan all input for complete counts even when
/// listings are limited (default 1000, maximum 65536; zero is counts-only).
/// Derived upper/lowercase and whitespace use Rust's reported Unicode version;
/// titlecase/general categories use pinned Unicode 16.0 data. Profile-1 zero-width candidates are observations,
/// never font/rendered-visibility guarantees. <=1 MiB source, <=8 MiB report.
pub fn inspect(input: &str, options: &Options) -> Result<Inspection, OperationError> {
    const OP: &str = "text.unicode.inspect";
    check_input(input, OP)?;
    validate(options, OP)?;
    let mut work = 0;
    charge(&mut work, input.len() as u64, options, OP)?;
    let mut report = Inspection {
        derived_case_space_unicode_version: std::char::UNICODE_VERSION,
        category_unicode_version: CATEGORY_UNICODE_VERSION,
        zero_width_profile: 1,
        counts: Counts::default(),
        records: Vec::with_capacity(options.max_results.min(input.len())),
        complete: true,
        work_units: 0,
    };
    let mut retained = ByteBudget::new(options.max_output);
    for (i, (offset, c)) in input.char_indices().enumerate() {
        charge(&mut work, 1, options, OP)?;
        let record = observation(i, offset, c);
        let counts = &mut report.counts;
        counts.scalars += 1;
        match record.case {
            "uppercase" => counts.uppercase += 1,
            "lowercase" => counts.lowercase += 1,
            "titlecase" => counts.titlecase += 1,
            _ => counts.uncased += 1,
        };
        counts.non_ascii += usize::from(record.non_ascii);
        counts.control += usize::from(record.control);
        counts.whitespace += usize::from(record.whitespace);
        counts.format += usize::from(record.format);
        counts.zero_width_candidates += usize::from(record.zero_width_candidate);
        counts.bidi_controls += usize::from(record.bidi_control);
        if report.records.len() < options.max_results {
            charge(&mut work, 1536, options, OP)?;
            serde_json::to_writer(&mut retained, &record).map_err(|_| {
                error(
                    OP,
                    ErrorCode::ResourceLimit,
                    "Unicode retained records exceed report budget",
                )
            })?;
            report.records.push(record);
        }
    }
    charge(&mut work, 2048, options, OP)?;
    report.complete = report.records.len() == report.counts.scalars;
    report.work_units = work;
    report_bytes(&report, options, OP)?;
    Ok(report)
}
fn differences(
    left: &str,
    right: &str,
    space: &'static str,
    remaining: &mut usize,
    work: &mut u64,
    options: &Options,
    retained: &mut ByteBudget,
) -> Result<Differences, OperationError> {
    const OP: &str = "text.unicode.compare";
    let mut left = left.char_indices();
    let mut right = right.char_indices();
    let mut index = 0;
    let mut result = Differences {
        coordinate_space: space,
        left_scalars: 0,
        right_scalars: 0,
        total_differences: 0,
        records: vec![],
        complete: true,
    };
    loop {
        let l = left.next();
        let r = right.next();
        if l.is_none() && r.is_none() {
            break;
        }
        charge(work, 1, options, OP)?;
        result.left_scalars += usize::from(l.is_some());
        result.right_scalars += usize::from(r.is_some());
        if l.map(|(_, c)| c) != r.map(|(_, c)| c) {
            result.total_differences += 1;
            if *remaining > 0 {
                charge(work, 1536, options, OP)?;
                let record = Difference {
                    scalar_index: index,
                    left: l.map(|(byte_offset, scalar)| Scalar {
                        scalar_index: index,
                        byte_offset,
                        scalar,
                        codepoint: scalar as u32,
                    }),
                    right: r.map(|(byte_offset, scalar)| Scalar {
                        scalar_index: index,
                        byte_offset,
                        scalar,
                        codepoint: scalar as u32,
                    }),
                };
                serde_json::to_writer(&mut *retained, &record).map_err(|_| {
                    error(
                        OP,
                        ErrorCode::ResourceLimit,
                        "Unicode retained differences exceed report budget",
                    )
                })?;
                result.records.push(record);
                *remaining -= 1;
            }
        }
        index += 1;
    }
    result.complete = result.records.len() == result.total_differences;
    Ok(result)
}
fn normalized<'a>(
    input: &'a str,
    form: &str,
    work: &mut u64,
    options: &Options,
) -> Result<Cow<'a, str>, OperationError> {
    const OP: &str = "text.unicode.compare";
    if form == "none" {
        return Ok(Cow::Borrowed(input));
    }
    // Reuse the pinned normalizer's primitive for a nonallocating expansion
    // preflight. This bounds its pending canonical-order/composition buffers too.
    let mut scalars = 0usize;
    for c in input.chars() {
        let mut count = 0usize;
        if form == "NFC" || form == "NFD" {
            unicode_normalization::char::decompose_canonical(c, |_| count += 1);
        } else {
            unicode_normalization::char::decompose_compatible(c, |_| count += 1);
        }
        scalars = scalars
            .checked_add(count)
            .filter(|n| *n <= MAX_NORMALIZED_SCALARS)
            .ok_or_else(|| {
                error(
                    OP,
                    ErrorCode::ResourceLimit,
                    "normalized expansion exceeds 2097152 decomposed scalars",
                )
            })?;
        charge(work, count as u64, options, OP)?;
    }
    super::normalize(input, form)
        .map(Cow::Owned)
        .map_err(|mut e| {
            e.operation = Some(OP);
            e
        })
}
/// Compare by scalar index, never edit-distance alignment. Always retain original
/// differences; optional NFC/NFD/NFKC/NFKD adds an explicitly labeled normalized
/// coordinate space with its own offsets. Inputs are not replaced. One shared
/// listing budget serves original then normalized differences; exact counts scan
/// both views even when complete=false. Two normalized buffers are each <=8 MiB
/// <=2097152 decomposed scalars per view preflight normalizer buffers. Retained
/// normalized text <=16 MiB; a conservative <=128 MiB scratch ceiling includes
/// normalization queues/capacity slack. Reports <=8 MiB, work <=128 Mi units.
pub fn compare(
    left: &str,
    right: &str,
    form: &str,
    options: &Options,
) -> Result<Comparison, OperationError> {
    const OP: &str = "text.unicode.compare";
    check_input(left, OP)?;
    check_input(right, OP)?;
    validate(options, OP)?;
    if !["none", "NFC", "NFD", "NFKC", "NFKD"].contains(&form) {
        return Err(error(
            OP,
            ErrorCode::InvalidParameter,
            "normalization must be none, NFC, NFD, NFKC or NFKD",
        )
        .parameter("normalization"));
    }
    let mut work = 0;
    charge(&mut work, (left.len() + right.len()) as u64, options, OP)?;
    let mut remaining = options.max_results;
    let mut retained = ByteBudget::new(options.max_output);
    let original = differences(
        left,
        right,
        "original_scalar_index",
        &mut remaining,
        &mut work,
        options,
        &mut retained,
    )?;
    let (normalized, normalized_equal) = if form == "none" {
        (None, None)
    } else {
        charge(&mut work, (left.len() + right.len()) as u64, options, OP)?;
        let l = normalized(left, form, &mut work, options)?;
        let r = normalized(right, form, &mut work, options)?;
        charge(&mut work, (l.len() + r.len()) as u64, options, OP)?;
        let differences = differences(
            &l,
            &r,
            "normalized_scalar_index",
            &mut remaining,
            &mut work,
            options,
            &mut retained,
        )?;
        let equal = differences.total_differences == 0;
        (Some(differences), Some(equal))
    };
    let complete = original.complete && normalized.as_ref().is_none_or(|d| d.complete);
    charge(&mut work, 2048, options, OP)?;
    let report = Comparison {
        exact_equal: original.total_differences == 0,
        normalized_equal,
        normalization: form.into(),
        normalization_unicode_version: super::UNICODE_VERSION,
        original,
        normalized,
        complete,
        work_units: work,
    };
    report_bytes(&report, options, OP)?;
    Ok(report)
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const LISTING: &[Parameter] = &[
    Parameter {
        name: "max_results",
        description: "Shared listing budget; exact counts continue after it is exhausted. Zero is counts-only.",
        required: false,
        default: Some(ParameterValue::Unsigned(1000)),
        constraints: ParameterType::Unsigned { min: 0, max: 65536 },
    },
    Parameter {
        name: "max_work",
        description: "Lowerable work bound; conservative record/header charges include bounded serialization.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_WORK)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_WORK,
        },
    },
    Parameter {
        name: "max_output",
        description: "Lowerable serialized report-byte bound.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_OUTPUT as u64)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_OUTPUT as u64,
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "text.unicode.inspect",
        category: "text",
        description: "Versioned scalar case/category/control/space/format observations and finite zero-width candidates; exact counts, honest bounded listings.",
        input_type: "text",
        output_type: "text.unicode.inspection",
        parameters: LISTING,
        deterministic: true,
        reversible: Some(false),
        command: "text unicode-inspect",
        example: "cube text unicode-inspect --text 'Aaé'",
    },
    Descriptor {
        id: "text.unicode.compare",
        category: "text",
        description: "Exact scalar-index comparison plus optional separately labeled normalized-view differences; original evidence retained, no fuzzy realignment.",
        input_type: "text.pair",
        output_type: "text.unicode.comparison",
        parameters: &[
            Parameter {
                name: "other",
                description: "Caller-supplied right UTF-8 input, <=1 MiB.",
                required: true,
                default: None,
                constraints: ParameterType::Text { choices: &[] },
            },
            Parameter {
                name: "normalization",
                description: "Add a normalized coordinate space; never overwrite original differences.",
                required: false,
                default: Some(ParameterValue::Text("none")),
                constraints: ParameterType::Text {
                    choices: &["none", "NFC", "NFD", "NFKC", "NFKD"],
                },
            },
            LISTING[0],
            LISTING[1],
            LISTING[2],
        ],
        deterministic: true,
        reversible: Some(false),
        command: "text unicode-compare",
        example: "cube text unicode-compare --text A --other a",
    },
];
