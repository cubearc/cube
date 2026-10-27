//! Exact scalar inventories, sharing preparation and counting with statistics.
use super::{
    MAX_OUTPUT, UNICODE_VERSION,
    statistics::{self, CaseMode, Normalization, TextFilter},
};
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
pub const MAX_WORK: usize = 16 * 1024 * 1024;
pub const MAX_MEMORY: usize = 32 * 1024 * 1024;
pub const MAX_REPORT: usize = 8 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Profile {
    pub normalization: Normalization,
    pub case: CaseMode,
    pub filter: TextFilter,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            normalization: Normalization::None,
            case: CaseMode::Preserve,
            filter: TextFilter::All,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Limits {
    pub max_work: usize,
    pub max_memory_bytes: usize,
    pub max_report_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_work: MAX_WORK,
            max_memory_bytes: MAX_MEMORY,
            max_report_bytes: MAX_REPORT,
        }
    }
}
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ScalarCount {
    pub scalar: char,
    pub count: u64,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub profile: Profile,
    pub normalization_unicode_version: (u8, u8, u8),
    pub equal: bool,
    pub contains: bool,
    pub source: Vec<ScalarCount>,
    pub requested: Vec<ScalarCount>,
    pub residual: Vec<ScalarCount>,
    pub deficits: Vec<ScalarCount>,
    pub work: usize,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
// Shared exact residual subtraction. A deficit leaves the source unchanged.
pub(crate) fn consume(
    source: &mut std::collections::BTreeMap<char, u64>,
    requested: &std::collections::BTreeMap<char, u64>,
) -> bool {
    if requested
        .iter()
        .any(|(c, n)| source.get(c).copied().unwrap_or(0) < *n)
    {
        return false;
    }
    for (c, n) in requested {
        *source.get_mut(c).expect("validated available count") -= n;
    }
    true
}
fn error(op: &'static str, code: ErrorCode, name: &'static str, msg: &str) -> OperationError {
    OperationError::new(code, msg).operation(op).parameter(name)
}
fn run(
    source: &str,
    requested: &str,
    profile: Profile,
    limits: Limits,
    op: &'static str,
) -> Result<Report, OperationError> {
    if limits.max_work > MAX_WORK
        || limits.max_memory_bytes > MAX_MEMORY
        || limits.max_report_bytes > MAX_REPORT
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "limits",
            "inventory limits exceed hard ceilings",
        ));
    }
    super::check_input(source, op)?;
    super::check_input(requested, op)?;
    let normalized = !matches!(profile.normalization, Normalization::None);
    let n = if normalized {
        2 * statistics::MAX_SCALARS
    } else {
        source.len() + requested.len()
    };
    let distinct = n.min(2 * statistics::MAX_DISTINCT);
    // Two prepared vectors, both trees, owned inventories and a serialization
    // buffer; at most one normalized 8 MiB temporary is alive at a time.
    let report_bound = 1024 + distinct * 2 * 64;
    let memory = source.len()
        + requested.len()
        + if normalized { MAX_OUTPUT } else { 0 }
        + if normalized { n * 4 } else { n * 8 }
        + distinct * 128
        + distinct * 2 * 16
        + report_bound;
    if memory > limits.max_memory_bytes {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "max_memory_bytes",
            "conservative inventory allocation bound exceeds budget",
        ));
    }
    if report_bound > limits.max_report_bytes {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "max_report_bytes",
            "conservative inventory report bound exceeds budget",
        ));
    }
    let mut work = source.len() + requested.len();
    if work > limits.max_work {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "max_work",
            "input visits exceed work budget",
        ));
    }
    let (a, visits) = statistics::prepare(
        source,
        profile.normalization,
        profile.case,
        profile.filter,
        limits.max_work - work,
        op,
    )?;
    work += visits;
    let (b, visits) = statistics::prepare(
        requested,
        profile.normalization,
        profile.case,
        profile.filter,
        limits.max_work - work,
        op,
    )?;
    work += visits;
    let count_work = a.len()
        + b.len()
        + 2 * a.len().min(statistics::MAX_DISTINCT)
        + 3 * b.len().min(statistics::MAX_DISTINCT);
    work = work.checked_add(count_work).ok_or_else(|| {
        error(
            op,
            ErrorCode::ResourceLimit,
            "max_work",
            "inventory work overflows",
        )
    })?;
    if work > limits.max_work {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "max_work",
            "count/merge visits exceed work budget",
        ));
    }
    let mut a = statistics::scalar_counts(&a, op)?;
    let b = statistics::scalar_counts(&b, op)?;
    let equal = a == b;
    let original_source = a
        .iter()
        .map(|(&scalar, &count)| ScalarCount { scalar, count })
        .collect();
    let contains = consume(&mut a, &b);
    let mut residual = Vec::new();
    let mut deficits = Vec::new();
    for (&scalar, &count) in &a {
        let used = if contains {
            0
        } else {
            b.get(&scalar).copied().unwrap_or(0)
        };
        if count > used {
            residual.push(ScalarCount {
                scalar,
                count: count - used,
            });
        }
    }
    for (&scalar, &count) in &b {
        if contains {
            break;
        }
        let available = a.get(&scalar).copied().unwrap_or(0);
        if count > available {
            deficits.push(ScalarCount {
                scalar,
                count: count - available,
            });
        }
    }
    let inventory = |map: std::collections::BTreeMap<char, u64>| {
        map.into_iter()
            .map(|(scalar, count)| ScalarCount { scalar, count })
            .collect()
    };
    Ok(Report {
        profile,
        normalization_unicode_version: UNICODE_VERSION,
        equal,
        contains,
        source: original_source,
        requested: inventory(b),
        residual,
        deficits,
        work,
        memory_bytes_upper_bound: memory,
        report_bytes_upper_bound: report_bound,
    })
}
/// Exact canonical inventories and both subtraction directions. Default profile
/// preserves every Unicode scalar, including NUL, accents and punctuation.
/// ```
/// use cube_core::text::multiset::{compare,Profile,Limits};
/// assert!(compare("listen","silent",Profile::default(),Limits::default())?.equal);
/// # Ok::<(),cube_core::errors::OperationError>(())
/// ```
pub fn compare(
    source: &str,
    requested: &str,
    profile: Profile,
    limits: Limits,
) -> Result<Report, OperationError> {
    run(source, requested, profile, limits, "text.multiset.compare")
}
/// Same complete evidence as [`compare`], under a distinct subtraction operation ID.
pub fn subtract(
    source: &str,
    requested: &str,
    profile: Profile,
    limits: Limits,
) -> Result<Report, OperationError> {
    run(source, requested, profile, limits, "text.multiset.subtract")
}
const PARAMS: &[Parameter] = &[
    Parameter {
        name: "requested",
        description: "Requested UTF-8 text, <=1 MiB; exact counts use the same profile as source.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "normalization",
        description: "Reuse existing bounded Unicode normalization.",
        required: false,
        default: Some(ParameterValue::Text("none")),
        constraints: ParameterType::Text {
            choices: &["none", "NFC", "NFD", "NFKC", "NFKD"],
        },
    },
    Parameter {
        name: "case",
        description: "Preserve case or lowercase ASCII only.",
        required: false,
        default: Some(ParameterValue::Text("preserve")),
        constraints: ParameterType::Text {
            choices: &["preserve", "ascii_lower"],
        },
    },
    Parameter {
        name: "filter",
        description: "All scalars or ASCII letters, after normalization/case.",
        required: false,
        default: Some(ParameterValue::Text("all")),
        constraints: ParameterType::Text {
            choices: &["all", "ascii_letters"],
        },
    },
    Parameter {
        name: "max_work",
        description: "Raw byte, prepared scalar and conservative count/merge visits.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_WORK as u64)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_WORK as u64,
        },
    },
    Parameter {
        name: "max_memory_bytes",
        description: "Combined scratch, retained inventories and compact JSON bound, <=32 MiB.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_MEMORY as u64)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_MEMORY as u64,
        },
    },
    Parameter {
        name: "max_report_bytes",
        description: "Conservative complete report cap, <=8 MiB; no listing truncation.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_REPORT as u64)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_REPORT as u64,
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "text.multiset.compare",
        category: "text",
        description: "Exact sorted scalar inventories with equality, containment, residuals and deficits; <=1 MiB per raw input, shared statistics preparation/counting.",
        input_type: "text",
        output_type: "scalar_multiset_report",
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: "text multiset compare",
        example: "cube text multiset compare --text listen --requested silent",
    },
    Descriptor {
        id: "text.multiset.subtract",
        category: "text",
        description: "Exact complete scalar subtraction and deficits; no underflow, hidden normalization or truncated listings.",
        input_type: "text",
        output_type: "scalar_multiset_report",
        parameters: PARAMS,
        deterministic: true,
        reversible: Some(false),
        command: "text multiset subtract",
        example: "cube text multiset subtract --text aab --requested ab",
    },
];
