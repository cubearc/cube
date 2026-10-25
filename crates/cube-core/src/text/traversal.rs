//! Bounded retained-ring and removal-based scalar traversal.
use super::{MAX_INDICES, MAX_OUTPUT, check_input, mapping::ByteBudget};
use crate::errors::{ErrorCode, OperationError};
use serde::{Deserialize, Serialize};
pub const MAX_WORK: u64 = 16_777_216;
pub const MAX_MEMORY: usize = 8 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    #[default]
    Forward,
    Reverse,
}
/// Limits can only be lowered. Start defaults to the first scalar in the chosen
/// input origin; output provenance always uses original zero-based indexes.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Options {
    pub start: Option<u64>,
    pub index_base: u8,
    pub direction: Direction,
    pub wrap: bool,
    pub max_work: u64,
    pub max_memory: usize,
    pub max_output: usize,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            start: None,
            index_base: 0,
            direction: Direction::Forward,
            wrap: true,
            max_work: MAX_WORK,
            max_memory: MAX_MEMORY,
            max_output: MAX_OUTPUT,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Record {
    pub scalar_index: usize,
    pub byte_offset: usize,
    pub scalar: char,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub extracted: String,
    pub records: Vec<Record>,
    pub remaining: usize,
    pub work_units: u64,
    pub temporary_bytes: usize,
}
fn error(op: &'static str, code: ErrorCode, message: &str) -> OperationError {
    OperationError::new(code, message).operation(op)
}
fn charge(work: &mut u64, units: u64, limit: u64, op: &'static str) -> Result<(), OperationError> {
    *work = work
        .checked_add(units)
        .filter(|v| *v <= limit)
        .ok_or_else(|| {
            error(
                op,
                ErrorCode::ResourceLimit,
                "traversal exceeds charged work limit",
            )
        })?;
    Ok(())
}
/// Strict positive decimal comma-separated step sequence, <=65536 entries.
pub fn parse_steps(text: &str) -> Result<Vec<u64>, OperationError> {
    const OP: &str = "text.skip.keyed";
    if text.len() > MAX_INDICES * 21 {
        return Err(error(
            OP,
            ErrorCode::ResourceLimit,
            "step text exceeds finite byte bound",
        ));
    }
    let mut result = Vec::new();
    for part in text.split(',') {
        if result.len() == MAX_INDICES {
            return Err(error(
                OP,
                ErrorCode::ResourceLimit,
                "steps exceed 65536 entries",
            ));
        }
        if part.is_empty() || !part.bytes().all(|c| c.is_ascii_digit()) {
            return Err(error(
                OP,
                ErrorCode::InvalidParameter,
                "steps require positive decimal integers",
            )
            .parameter("steps"));
        }
        result.push(part.parse().ok().filter(|n| *n > 0).ok_or_else(|| {
            error(
                OP,
                ErrorCode::InvalidParameter,
                "steps must be positive u64 integers",
            )
            .parameter("steps")
        })?);
    }
    Ok(result)
}
// Fenwick tree: each cell is the active count in its covered interval.
struct Ring {
    tree: Vec<u32>,
}
impl Ring {
    fn new(n: usize) -> Self {
        Self {
            tree: (0..=n).map(|i| (i.isolate_lowest_one()) as u32).collect(),
        }
    }
    fn select(
        &self,
        mut rank: usize,
        work: &mut u64,
        options: &Options,
        op: &'static str,
    ) -> Result<usize, OperationError> {
        let mut index = 0;
        let mut bit = 1usize << (usize::BITS - 1 - (self.tree.len() - 1).leading_zeros());
        while bit > 0 {
            charge(work, 1, options.max_work, op)?;
            let next = index + bit;
            if next < self.tree.len() && self.tree[next] as usize <= rank {
                rank -= self.tree[next] as usize;
                index = next;
            }
            bit >>= 1;
        }
        Ok(index)
    }
    fn remove(
        &mut self,
        index: usize,
        work: &mut u64,
        options: &Options,
        op: &'static str,
    ) -> Result<(), OperationError> {
        let mut i = index + 1;
        while i < self.tree.len() {
            charge(work, 1, options.max_work, op)?;
            self.tree[i] -= 1;
            i += i.isolate_lowest_one();
        }
        Ok(())
    }
}
fn selected(anchor: usize, step: u64, n: usize, direction: Direction, wrap: bool) -> Option<usize> {
    let distance = if wrap {
        ((step - 1) % n as u64) as usize
    } else {
        usize::try_from(step - 1).ok()?
    };
    match direction {
        Direction::Forward if wrap => Some((anchor + distance) % n),
        Direction::Reverse if wrap => Some((anchor + n - distance) % n),
        Direction::Forward => anchor.checked_add(distance).filter(|i| *i < n),
        Direction::Reverse => anchor.checked_sub(distance),
    }
}
fn execute(
    input: &str,
    steps: &[u64],
    count: Option<usize>,
    options: &Options,
    remove: bool,
) -> Result<Report, OperationError> {
    let op = if remove {
        "text.josephus.extract"
    } else {
        "text.skip.keyed"
    };
    check_input(input, op)?;
    if options.index_base > 1
        || options.max_work > MAX_WORK
        || options.max_memory > MAX_MEMORY
        || options.max_output > MAX_OUTPUT
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "origin or limits exceed supported hard bounds",
        ));
    }
    if steps.is_empty() || steps.len() > MAX_INDICES || steps.contains(&0) {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "provide 1..65536 positive u64 steps",
        )
        .parameter("steps"));
    }
    if remove && !options.wrap {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "Josephus requires wrapping",
        )
        .parameter("wrap"));
    }
    let mut work = 0;
    charge(&mut work, input.len() as u64, options.max_work, op)?;
    let n = input.chars().count();
    let count = count.unwrap_or(n);
    if count > MAX_INDICES || (remove && count > n) {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "selection count exceeds source or 65536 records",
        )
        .parameter("count"));
    }
    let start = options.start.unwrap_or(u64::from(options.index_base));
    let start = start
        .checked_sub(u64::from(options.index_base))
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidParameter,
                "start precedes origin or exceeds platform width",
            )
            .parameter("start")
        })?;
    if (n == 0 && (count != 0 || start != 0)) || (n > 0 && start >= n) {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "start or positive count is outside the source",
        )
        .parameter("start"));
    }
    // Exact-capacity source chars/offsets and optional u32 Fenwick cells. The
    // extraction phase releases those buffers before reusing text::extract.
    let indexing = n * 8 + if remove && count > 0 { (n + 1) * 4 } else { 0 };
    let extraction = n * 4 + count * std::mem::size_of::<usize>();
    let temporary_bytes = indexing.max(extraction);
    if temporary_bytes > options.max_memory {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "traversal exceeds temporary-memory limit",
        ));
    }
    charge(&mut work, n as u64, options.max_work, op)?;
    let mut characters = Vec::with_capacity(n);
    let mut offsets = Vec::with_capacity(n);
    for (offset, c) in input.char_indices() {
        characters.push(c);
        offsets.push(offset as u32);
    }
    let mut ring = if remove && count > 0 {
        charge(&mut work, n as u64, options.max_work, op)?;
        Some(Ring::new(n))
    } else {
        None
    };
    let mut records = Vec::with_capacity(count);
    let mut anchor = Some(start);
    let mut active = n;
    for i in 0..count {
        charge(&mut work, 1, options.max_work, op)?;
        let rank = selected(
            anchor.ok_or_else(|| {
                error(
                    op,
                    ErrorCode::InvalidInput,
                    "nonwrapping selection leaves the source",
                )
            })?,
            steps[i % steps.len()],
            active,
            options.direction,
            options.wrap,
        )
        .ok_or_else(|| {
            error(
                op,
                ErrorCode::InvalidInput,
                "nonwrapping selection leaves the source",
            )
        })?;
        let index = if let Some(ring) = &mut ring {
            ring.select(rank, &mut work, options, op)?
        } else {
            rank
        };
        records.push(Record {
            scalar_index: index,
            byte_offset: offsets[index] as usize,
            scalar: characters[index],
        });
        if let Some(ring) = &mut ring {
            ring.remove(index, &mut work, options, op)?;
            active -= 1;
            anchor = if active == 0 {
                None
            } else {
                Some(match options.direction {
                    Direction::Forward => rank % active,
                    Direction::Reverse => (rank + active - 1) % active,
                })
            };
        } else {
            anchor = match options.direction {
                Direction::Forward if options.wrap => Some((rank + 1) % active),
                Direction::Reverse if options.wrap => Some((rank + active - 1) % active),
                Direction::Forward => rank.checked_add(1).filter(|i| *i < active),
                Direction::Reverse => rank.checked_sub(1),
            };
        }
    }
    drop(ring);
    drop(characters);
    drop(offsets);
    charge(&mut work, (n * 2 + count) as u64, options.max_work, op)?;
    let indexes: Vec<_> = records.iter().map(|r| r.scalar_index).collect();
    let extracted = super::extract(input, Some(&indexes), None, None, None).map_err(|mut e| {
        e.operation = Some(op);
        e
    })?;
    let mut report = Report {
        extracted,
        records,
        remaining: if remove { active } else { n },
        work_units: work,
        temporary_bytes,
    };
    let mut budget = ByteBudget::new(options.max_output);
    serde_json::to_writer(&mut budget, &report).map_err(|_| {
        error(
            op,
            ErrorCode::ResourceLimit,
            "traversal report exceeds output limit",
        )
    })?;
    // Charge an upper bound for serialized evidence, including the larger final
    // work counter. Serialization itself has a fixed <=8 MiB ceiling.
    let serialized = budget.written();
    charge(&mut work, serialized as u64 * 2 + 40, options.max_work, op)?;
    report.work_units = work;
    let mut budget = ByteBudget::new(options.max_output);
    serde_json::to_writer(&mut budget, &report).map_err(|_| {
        error(
            op,
            ErrorCode::ResourceLimit,
            "traversal report exceeds output limit",
        )
    })?;
    Ok(report)
}
/// Retain the source ring. First selection advances step-1 from start;
/// subsequent selections advance the next cyclic step from the selected scalar.
/// Count is mandatory; nonwrapping range failures never return a partial report.
pub fn keyed(
    input: &str,
    steps: &[u64],
    count: usize,
    options: &Options,
) -> Result<Report, OperationError> {
    execute(input, steps, Some(count), options, false)
}
/// Remove selected scalars, resume at the next remaining scalar in the chosen
/// direction. Count defaults to all source scalars; explicit count may stop early.
pub fn josephus(
    input: &str,
    steps: &[u64],
    count: Option<usize>,
    options: &Options,
) -> Result<Report, OperationError> {
    execute(input, steps, count, options, true)
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "steps",
        description: "Nonempty cyclic positive u64 step list, <=65536 entries.",
        required: true,
        default: None,
        constraints: ParameterType::UnsignedList {
            min: 1,
            max: u64::MAX,
            min_length: 1,
            max_length: MAX_INDICES,
            unique: false,
        },
    },
    Parameter {
        name: "count",
        description: "Required for keyed skip; Josephus defaults to all, or stops at explicit count.",
        required: false,
        default: None,
        constraints: ParameterType::Unsigned { min: 0, max: 65536 },
    },
    Parameter {
        name: "start",
        description: "Initial scalar anchor, default first scalar in explicit origin.",
        required: false,
        default: None,
        constraints: ParameterType::Unsigned {
            min: 0,
            max: u64::MAX,
        },
    },
    Parameter {
        name: "index_base",
        description: "Input start origin; report indexes always zero-based.",
        required: false,
        default: Some(ParameterValue::Unsigned(0)),
        constraints: ParameterType::UnsignedChoices { choices: &[0, 1] },
    },
    Parameter {
        name: "direction",
        description: "Direction of traversal and resumption.",
        required: false,
        default: Some(ParameterValue::Text("forward")),
        constraints: ParameterType::Text {
            choices: &["forward", "reverse"],
        },
    },
    Parameter {
        name: "wrap",
        description: "Wrap retained ring; Josephus rejects false.",
        required: false,
        default: Some(ParameterValue::Boolean(true)),
        constraints: ParameterType::Boolean,
    },
    Parameter {
        name: "max_work",
        description: "Lowerable charged work ceiling including serialized evidence.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_WORK)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_WORK,
        },
    },
    Parameter {
        name: "max_memory",
        description: "Lowerable scratch-byte ceiling, excluding bounded retained report.",
        required: false,
        default: Some(ParameterValue::Unsigned(MAX_MEMORY as u64)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_MEMORY as u64,
        },
    },
    Parameter {
        name: "max_output",
        description: "Lowerable serialized report-byte ceiling.",
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
        id: "text.skip.keyed",
        category: "text",
        description: "Bounded cyclic-step retained scalar ring with mandatory count and original positions; first selection is step-1 advances from start.",
        input_type: "text",
        output_type: "text.traversal.report",
        parameters: PARAMETERS,
        deterministic: true,
        reversible: Some(false),
        command: "text keyed-skip",
        example: "cube text keyed-skip --steps 2 --count 5 --text ABCDE",
    },
    Descriptor {
        id: "text.josephus.extract",
        category: "text",
        description: "Bounded removal-based scalar traversal, resume next remaining in chosen direction; Fenwick rank selection preserves original positions.",
        input_type: "text",
        output_type: "text.traversal.report",
        parameters: PARAMETERS,
        deterministic: true,
        reversible: Some(false),
        command: "text josephus",
        example: "cube text josephus --steps 2 --text ABCDE",
    },
];
