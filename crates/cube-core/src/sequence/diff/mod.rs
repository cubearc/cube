//! Minimal byte/scalar/line scripts and bounded line review strategies.
mod anchors;
mod myers;
mod unified;

use super::{spans::Indexed, *};
use serde::Serialize;
use std::collections::BTreeMap;
pub use unified::UnifiedOptions;

const MAX_REPORT: usize = 8 << 20;

#[derive(Deserialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum LineSource {
    Text(String),
    Bytes { base64: String },
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "unit", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    Bytes { left: String, right: String },
    Scalars { left: String, right: String },
    Lines { left: LineSource, right: LineSource },
}

#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    #[default]
    Myers,
    Patience,
    Histogram,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DiffRequest {
    pub input: Input,
    #[serde(default)]
    pub method: Method,
    #[serde(default)]
    #[schemars(range(min = 1, max = 256))]
    pub occurrence_cap: Option<usize>,
    #[serde(default)]
    pub unified: Option<UnifiedOptions>,
}

#[derive(Serialize, JsonSchema)]
pub struct DiffReport {
    pub method: Method,
    pub profile: &'static str,
    pub minimal: bool,
    pub runs: Vec<Run>,
    pub unified: Option<String>,
}

struct Lines {
    bytes: Vec<u8>,
    offsets: Vec<usize>,
}
impl Lines {
    fn new(input: &LineSource) -> Result<Self, OperationError> {
        let bytes = match input {
            LineSource::Text(text) => {
                if text.len() > TEXT_BYTES {
                    return Err(resource("line input exceeds 1 MiB"));
                }
                text.as_bytes().to_vec()
            }
            LineSource::Bytes { base64 } => search::bytes(base64, TEXT_BYTES)?,
        };
        let mut offsets = vec![0];
        for (index, &value) in bytes.iter().enumerate() {
            if value == b'\n' {
                offsets.push(index + 1);
            }
            if offsets.len() > MAX_UNITS + 1 {
                return Err(resource("line input exceeds 65536 lines"));
            }
        }
        if offsets.last() != Some(&bytes.len()) {
            offsets.push(bytes.len());
        }
        if offsets.len() > MAX_UNITS + 1 {
            return Err(resource("line input exceeds 65536 lines"));
        }
        Ok(Self { bytes, offsets })
    }
    fn get(&self, i: usize) -> &[u8] {
        &self.bytes[self.offsets[i]..self.offsets[i + 1]]
    }
}

type Intern<'a> = BTreeMap<u64, Vec<(&'a [u8], u32)>>;
fn line_ids<'a>(
    lines: &'a Lines,
    ids: &mut Intern<'a>,
    next: &mut u32,
    work: &mut Work,
) -> Result<Indexed, OperationError> {
    let mut units = Vec::with_capacity(lines.offsets.len() - 1);
    for index in 0..lines.offsets.len() - 1 {
        let line = lines.get(index);
        work.charge(line.len() + 1)?;
        let hash = line.iter().fold(0xcbf29ce484222325u64, |acc, &b| {
            (acc ^ u64::from(b)).wrapping_mul(0x100000001b3)
        });
        let bucket = ids.entry(hash).or_default();
        let mut equal = None;
        for &(other, id) in bucket.iter() {
            work.charge(
                1 + if other.len() == line.len() {
                    line.len()
                } else {
                    0
                },
            )?;
            if other == line {
                equal = Some(id);
                break;
            }
        }
        units.push(equal.unwrap_or_else(|| {
            let id = *next;
            *next += 1;
            bucket.push((line, id));
            id
        }));
    }
    Ok(Indexed::with_offsets(units, lines.offsets.clone()))
}

fn runs(steps: &[EditKind], left: &Indexed, right: &Indexed) -> Vec<Run> {
    let (mut a, mut b, mut i) = (0, 0, 0);
    let mut runs = Vec::new();
    while let Some(&kind) = steps.get(i) {
        let count = steps[i..]
            .iter()
            .take_while(|&&value| value == kind)
            .count();
        let (n, m) = (
            if kind == EditKind::Insert { 0 } else { count },
            if kind == EditKind::Delete { 0 } else { count },
        );
        runs.push(Run {
            kind,
            left: left.span(a, n),
            right: right.span(b, m),
        });
        (a, b, i) = (a + n, b + m, i + count);
    }
    runs
}

fn limited(report: DiffReport) -> Result<DiffReport, OperationError> {
    serialized_bound(&report, MAX_REPORT, "diff report exceeds 8 MiB")?;
    Ok(report)
}

fn compare(
    left: Indexed,
    right: Indexed,
    input: &DiffRequest,
    cap: usize,
    work: &mut Work,
    line_data: Option<(&Lines, &Lines)>,
) -> Result<DiffReport, OperationError> {
    let mut steps = Vec::with_capacity(left.units.len() + right.units.len());
    let (minimal, profile) = match input.method {
        Method::Myers => {
            myers::script(&left.units, &right.units, work, &mut steps)?;
            (true, "myers_1986_delete_ties")
        }
        method => {
            anchors::script(&left.units, &right.units, method, cap, work, &mut steps)?;
            (
                false,
                match method {
                    Method::Patience => "git_2_51_patience_lines",
                    _ => "git_2_51_histogram_lines",
                },
            )
        }
    };
    let mut index = 0;
    while index < steps.len() {
        if steps[index] == EditKind::Equal {
            index += 1;
            continue;
        }
        let edits = steps[index..]
            .iter()
            .take_while(|&&kind| kind != EditKind::Equal)
            .count();
        let deletions = steps[index..index + edits]
            .iter()
            .filter(|&&kind| kind == EditKind::Delete)
            .count();
        steps[index..index + deletions].fill(EditKind::Delete);
        steps[index + deletions..index + edits].fill(EditKind::Insert);
        index += edits;
    }
    let unified = match (&input.unified, line_data) {
        (Some(options), Some((a, b))) => Some(unified::render(&steps, a, b, options)?),
        _ => None,
    };
    limited(DiffReport {
        method: input.method,
        profile,
        minimal,
        runs: runs(&steps, &left, &right),
        unified,
    })
}

/// Produce a reproducible edit script and optionally render UTF-8 line hunks.
pub fn diff(input: &DiffRequest) -> Result<DiffReport, OperationError> {
    let cap = input.occurrence_cap.unwrap_or(64);
    if !(1..=256).contains(&cap)
        || input.occurrence_cap.is_some() && !matches!(input.method, Method::Histogram)
    {
        return Err(parameter(
            "occurrence_cap applies only to Histogram and must be in 1..=256",
        ));
    }
    let mut work = Work(WORK);
    let pair = match &input.input {
        Input::Lines { left, right } => {
            let (left, right) = (Lines::new(left)?, Lines::new(right)?);
            let mut names = Intern::new();
            let mut next = 0;
            let (a, b) = (
                line_ids(&left, &mut names, &mut next, &mut work)?,
                line_ids(&right, &mut names, &mut next, &mut work)?,
            );
            drop(names);
            return compare(a, b, input, cap, &mut work, Some((&left, &right)));
        }
        Input::Bytes { left, right } => Pair::Bytes {
            left: left.clone(),
            right: right.clone(),
        },
        Input::Scalars { left, right } => Pair::Scalars {
            left: left.clone(),
            right: right.clone(),
        },
    };
    if input.unified.is_some() || !matches!(input.method, Method::Myers) {
        return Err(parameter(
            "unified, Patience and Histogram require line mode",
        ));
    }
    let (left, right) = pair.indexed(MAX_UNITS)?;
    compare(left, right, input, cap, &mut work, None)
}
