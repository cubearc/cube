//! Substitution-only lexical paths. Explicit indexed adjacency, shared graph BFS.
use super::{
    lexical::{self, Entry, charge, error},
    multiset::Profile,
    statistics::TextFilter,
};
use crate::{
    errors::{ErrorCode, OperationError},
    graph::{self, Definition, Edge, Graph},
};
use serde::Serialize;
use std::collections::BTreeMap;
pub const MAX_WORK: usize = 16 * 1024 * 1024;
pub const MAX_MEMORY: usize = 32 * 1024 * 1024;
pub const MAX_REPORT: usize = 8 * 1024 * 1024;
pub const MAX_DICTIONARY_BYTES: usize = 1024 * 1024;
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct Options {
    pub max_work: usize,
    pub max_memory_bytes: usize,
    pub max_report_bytes: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            max_work: MAX_WORK,
            max_memory_bytes: MAX_MEMORY,
            max_report_bytes: MAX_REPORT,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Failure {
    pub position: usize,
    pub step: Option<usize>,
    pub reason: &'static str,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Report {
    pub profile: Profile,
    pub normalization_unicode_version: (u8, u8, u8),
    pub dictionary: Vec<Entry>,
    pub path: Option<Vec<usize>>,
    pub valid: bool,
    pub failure: Option<Failure>,
    pub status: &'static str,
    pub complete: bool,
    pub edge_count: usize,
    pub work: usize,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
struct Prepared {
    entries: Vec<Entry>,
    scalars: Vec<Vec<char>>,
    lookup: BTreeMap<String, usize>,
    work: usize,
    memory: usize,
    report: usize,
    width: usize,
}
fn prepare(
    words: &[String],
    profile: Profile,
    o: Options,
    op: &'static str,
) -> Result<Prepared, OperationError> {
    if !matches!(profile.filter, TextFilter::All)
        || o.max_work > MAX_WORK
        || o.max_memory_bytes > MAX_MEMORY
        || o.max_report_bytes > MAX_REPORT
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "options",
            "ladder preserves all scalars and limits may only be lowered",
        ));
    }
    if words.len() > 10000 {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "dictionary",
            "at most 10000 supplied aliases and 4096 distinct words",
        ));
    }
    let bytes = words.iter().map(|s| s.len()).sum::<usize>();
    if bytes > MAX_DICTIONARY_BYTES {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "dictionary",
            "dictionary exceeds 1 MiB",
        ));
    }
    let base = super::MAX_OUTPUT + bytes * 3 + words.len() * 512 + 8192;
    if base > o.max_memory_bytes {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "max_memory_bytes",
            "dictionary preparation exceeds memory budget",
        ));
    }
    let mut work = 0;
    let mut entries = lexical::entries(words, profile, 32, &mut work, o.max_work, op)?;
    if entries.is_empty() {
        return Err(error(
            op,
            ErrorCode::InvalidInput,
            "dictionary",
            "dictionary must contain endpoints",
        ));
    }
    if entries.len() > 4096 {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "dictionary",
            "more than 4096 distinct words",
        ));
    }
    charge(
        &mut work,
        entries.len() * 32 * (1 + entries.len().ilog2() as usize),
        o.max_work,
        op,
    )?;
    entries.sort_by(|a, b| a.text.cmp(&b.text));
    let scalars: Vec<Vec<char>> = entries.iter().map(|e| e.text.chars().collect()).collect();
    let width = scalars[0].len();
    if scalars.iter().any(|s| s.len() != width) {
        return Err(error(
            op,
            ErrorCode::InvalidInput,
            "dictionary",
            "all words must have the same nonzero prepared scalar length",
        ));
    }
    let report = 4096
        + entries
            .iter()
            .map(|e| crate::pipeline::json::string_bytes(&e.text) + 128 + e.aliases.len() * 24)
            .sum::<usize>()
        + entries.len() * 32;
    let memory = base + entries.len() * width * 512 + report * 2;
    if memory > o.max_memory_bytes || report > o.max_report_bytes {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "limits",
            "dictionary index/report bound exceeds budget",
        ));
    }
    let lookup = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.text.clone(), i))
        .collect();
    Ok(Prepared {
        entries,
        scalars,
        lookup,
        work,
        memory,
        report,
        width,
    })
}
fn endpoint(
    s: &str,
    p: Profile,
    d: &mut Prepared,
    o: Options,
    op: &'static str,
) -> Result<(String, Vec<char>), OperationError> {
    let v = lexical::prepare(s, p, &mut d.work, o.max_work, op)?;
    if v.len() != d.width {
        return Err(error(
            op,
            ErrorCode::InvalidInput,
            "word",
            "word must have the dictionary's prepared scalar length",
        ));
    }
    Ok((v.iter().collect(), v))
}
fn report(
    d: Prepared,
    p: Profile,
    path: Option<Vec<usize>>,
    failure: Option<Failure>,
    status: &'static str,
    edges: usize,
) -> Report {
    Report {
        profile: p,
        normalization_unicode_version: super::UNICODE_VERSION,
        dictionary: d.entries,
        valid: path.is_some(),
        path,
        failure,
        status,
        complete: true,
        edge_count: edges,
        work: d.work,
        memory_bytes_upper_bound: d.memory,
        report_bytes_upper_bound: d.report,
    }
}
/// Check caller steps in order. Membership and bad adjacencies are evidence;
/// invalid model/options or incompatible lengths are typed errors. Paths <=4096.
pub fn validate(
    words: &[String],
    path: &[String],
    p: Profile,
    o: Options,
) -> Result<Report, OperationError> {
    const OP: &str = "text.word_ladder.validate";
    let mut d = prepare(words, p, o, OP)?;
    if path.len() > 4096 || path.iter().map(|s| s.len()).sum::<usize>() > MAX_DICTIONARY_BYTES {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "path",
            "path exceeds 4096 words or 1 MiB",
        ));
    }
    if path.is_empty() {
        return Err(error(
            OP,
            ErrorCode::InvalidInput,
            "path",
            "a ladder must contain at least one word",
        ));
    }
    d.report += path.len() * 24;
    d.memory += path.len() * 128 + path.iter().map(|s| s.len() * 3).sum::<usize>();
    if d.report > o.max_report_bytes || d.memory > o.max_memory_bytes {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "limits",
            "path retention/report exceeds budget",
        ));
    }
    let mut indexes = Vec::new();
    let mut previous: Option<Vec<char>> = None;
    for (i, s) in path.iter().enumerate() {
        let (text, v) = endpoint(s, p, &mut d, o, OP)?;
        charge(
            &mut d.work,
            d.width * (1 + d.entries.len().ilog2() as usize),
            o.max_work,
            OP,
        )?;
        let Some(&index) = d.lookup.get(&text) else {
            return Ok(report(
                d,
                p,
                None,
                Some(Failure {
                    position: i,
                    step: i.checked_sub(1),
                    reason: "not_in_dictionary",
                }),
                "invalid",
                0,
            ));
        };
        if let Some(prev) = previous
            && prev.iter().zip(&v).filter(|(a, b)| a != b).count() != 1
        {
            return Ok(report(
                d,
                p,
                None,
                Some(Failure {
                    position: i,
                    step: Some(i - 1),
                    reason: "not_one_substitution",
                }),
                "invalid",
                0,
            ));
        }
        previous = Some(v);
        indexes.push(index);
    }
    Ok(report(d, p, Some(indexes), None, "valid", 0))
}
/// Canonical prepared-word lexicographic order resolves BFS ties. The wildcard
/// key includes the removed position, so no insertion/deletion edge is generated.
/// Exhausted construction/BFS budgets fail; `no_path` is a completed proof.
pub fn shortest(
    words: &[String],
    start: &str,
    end: &str,
    p: Profile,
    o: Options,
) -> Result<Report, OperationError> {
    const OP: &str = "text.word_ladder.shortest";
    let mut d = prepare(words, p, o, OP)?;
    let (a, _) = endpoint(start, p, &mut d, o, OP)?;
    let (b, _) = endpoint(end, p, &mut d, o, OP)?;
    let (&a, &b) = d.lookup.get(&a).zip(d.lookup.get(&b)).ok_or_else(|| {
        error(
            OP,
            ErrorCode::InvalidInput,
            "endpoints",
            "both endpoints must belong to the dictionary",
        )
    })?;
    let mut buckets: BTreeMap<(usize, Vec<char>), Vec<usize>> = BTreeMap::new();
    for (i, word) in d.scalars.iter().enumerate() {
        for pos in 0..d.width {
            charge(
                &mut d.work,
                d.width * (1 + (d.entries.len() * d.width).max(1).ilog2() as usize),
                o.max_work,
                OP,
            )?;
            let key = (
                pos,
                word.iter()
                    .enumerate()
                    .filter_map(|(j, c)| (j != pos).then_some(*c))
                    .collect(),
            );
            buckets.entry(key).or_default().push(i);
        }
    }
    let mut pairs = Vec::new();
    for bucket in buckets.values() {
        for (j, &u) in bucket.iter().enumerate() {
            for &v in &bucket[j + 1..] {
                charge(&mut d.work, 1, o.max_work, OP)?;
                if pairs.len() == graph::MAX_EDGES {
                    return Err(error(
                        OP,
                        ErrorCode::ResourceLimit,
                        "edges",
                        "more than 32768 substitution edges",
                    ));
                }
                pairs.push((u, v));
            }
        }
    }
    charge(
        &mut d.work,
        pairs.len() * 2 * (1 + pairs.len().max(1).ilog2() as usize),
        o.max_work,
        OP,
    )?;
    pairs.sort_unstable();
    let count = pairs.len();
    let extra = 8192 + count * 1024 + d.entries.len() * 1024;
    if d.memory + extra > o.max_memory_bytes {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "max_memory_bytes",
            "combined lexical/graph memory exceeds budget",
        ));
    }
    // Drop wildcard keys before allocating the graph; reserve their conservative
    // peak above. Numeric IDs avoid imposing graph's 64-byte limit on word text.
    drop(buckets);
    let limits = graph::Limits {
        max_work: (o.max_work - d.work).min(graph::MAX_WORK as usize) as u64,
        max_memory_bytes: o.max_memory_bytes - d.memory,
        max_report_bytes: o.max_report_bytes.min(graph::MAX_REPORT),
    };
    let definition = Definition {
        vertices: (0..d.entries.len()).map(|i| i.to_string()).collect(),
        directed: false,
        edges: pairs
            .into_iter()
            .enumerate()
            .map(|(i, (u, v))| Edge {
                id: i.to_string(),
                from: u.to_string(),
                to: v.to_string(),
            })
            .collect(),
    };
    let g = Graph::new(definition, limits).map_err(|e| e.operation(OP))?;
    let result = g
        .shortest_path(&a.to_string(), &b.to_string(), limits)
        .map_err(|e| e.operation(OP))?;
    d.work += result.work as usize;
    d.memory += result.memory_bytes_upper_bound;
    let path = result.result.map(|path| {
        path.vertices
            .into_iter()
            .map(|s| s.parse().expect("numeric graph IDs"))
            .collect()
    });
    let status = if path.is_some() { "found" } else { "no_path" };
    Ok(report(d, p, path, None, status, count))
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
        name: "dictionary",
        description: "Explicit JSON string array; retain original alias indexes.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "max_work",
        description: "Charged validation, index and execution work; exhaustion is explicit.",
        required: false,
        default: Some(ParameterValue::Unsigned(16777216)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 16777216,
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
const VALIDATE_PARAMETERS: &[Parameter] = &[
    PARAMETERS[0],
    PARAMETERS[1],
    PARAMETERS[2],
    PARAMETERS[3],
    PARAMETERS[4],
    PARAMETERS[5],
    Parameter {
        name: "path",
        description: "Required ordered JSON word array, 1..4096 entries and <=1 MiB.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
];
const SHORTEST_PARAMETERS: &[Parameter] = &[
    PARAMETERS[0],
    PARAMETERS[1],
    PARAMETERS[2],
    PARAMETERS[3],
    PARAMETERS[4],
    PARAMETERS[5],
    Parameter {
        name: "start",
        description: "Required prepared-length word present in the dictionary.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "end",
        description: "Required prepared-length word present in the dictionary.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
];
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "text.word_ladder.validate",
        category: "text",
        description: "Validate exact single-scalar substitutions and caller dictionary membership; 4096 distinct words, 32 scalars/word, 1 MiB dictionary, 4096 path words.",
        input_type: "dictionary_and_path",
        output_type: "word_ladder_report",
        command: "text word-ladder validate",
        example: "cube text word-ladder validate --dictionary '[\"cat\",\"cot\"]' --path '[\"cat\",\"cot\"]'",
        parameters: VALIDATE_PARAMETERS,
        deterministic: true,
        reversible: None,
    },
    Descriptor {
        id: "text.word_ladder.shortest",
        category: "text",
        description: "Indexed substitution adjacency and shared graph BFS; lexicographic ties, 32768 edges; resource exhaustion never means no_path.",
        input_type: "dictionary_and_endpoints",
        output_type: "word_ladder_report",
        command: "text word-ladder shortest",
        example: "cube text word-ladder shortest --dictionary '[\"cat\",\"cot\"]' --start cat --end cot",
        parameters: SHORTEST_PARAMETERS,
        deterministic: true,
        reversible: None,
    },
];
