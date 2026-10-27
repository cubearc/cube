//! Deterministic bounded Cartesian strings, generated incrementally.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
pub const MAX_GROUPS: usize = 16;
pub const MAX_FRAGMENTS: usize = 4096;
pub const MAX_FRAGMENT_BYTES: usize = 65536;
pub const MAX_INPUT_BYTES: usize = 1024 * 1024;
pub const MAX_RESULTS: usize = 10000;
pub const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_SEPARATOR_BYTES: usize = 4096;
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).operation("text.combinations")
}
fn limit(message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message).operation("text.combinations")
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct CombinationPlan {
    pub input_groups: usize,
    pub total_results: usize,
    pub input_bytes: usize,
    pub max_result_bytes: usize,
    pub aggregate_output_bytes: usize,
    pub jsonl_output_bytes: usize,
    pub work_bytes_upper_bound: usize,
}
pub struct Combinations {
    groups: Vec<Vec<String>>,
    separator: String,
    plan: CombinationPlan,
    index: usize,
}
impl Combinations {
    /// Rightmost group varies fastest. Duplicates and empty fragments remain
    /// distinct candidates. No groups yields one empty string; any empty group
    /// yields zero. Plans exceeding limits are rejected, never silently truncated.
    pub fn new(
        groups: Vec<Vec<String>>,
        separator: String,
        max_results: usize,
    ) -> Result<Self, OperationError> {
        if groups.len() > MAX_GROUPS || separator.len() > MAX_SEPARATOR_BYTES {
            return Err(limit(
                "at most 16 groups and 4096 separator bytes are supported",
            ));
        }
        if !(1..=MAX_RESULTS).contains(&max_results) {
            return Err(invalid("max_results must be 1..=10000"));
        }
        let mut fragments = 0usize;
        let mut input_bytes = separator.len();
        for group in &groups {
            fragments = fragments
                .checked_add(group.len())
                .ok_or_else(|| limit("fragment count overflow"))?;
            if fragments > MAX_FRAGMENTS {
                return Err(limit("input exceeds 4096 fragments"));
            }
            for fragment in group {
                if fragment.len() > MAX_FRAGMENT_BYTES {
                    return Err(limit("fragment exceeds 64 KiB"));
                }
                input_bytes = input_bytes
                    .checked_add(fragment.len())
                    .ok_or_else(|| limit("input size overflow"))?;
            }
        }
        if input_bytes > MAX_INPUT_BYTES {
            return Err(limit("combination input exceeds 1 MiB"));
        }
        let count = if groups.iter().any(Vec::is_empty) {
            0
        } else {
            groups.iter().try_fold(1usize, |n, g| {
                n.checked_mul(g.len())
                    .ok_or_else(|| limit("Cartesian result count overflow"))
            })?
        };
        if count > max_results {
            return Err(limit(
                "Cartesian result count exceeds max_results; no prefix is returned",
            ));
        }
        let mut max_bytes = 0usize;
        let mut aggregate = 0usize;
        let mut jsonl = 0usize;
        if count > 0 {
            max_bytes = separator
                .len()
                .checked_mul(groups.len().saturating_sub(1))
                .ok_or_else(|| limit("separator expansion overflow"))?;
            aggregate = max_bytes
                .checked_mul(count)
                .ok_or_else(|| limit("aggregate separator size overflow"))?;
            for group in &groups {
                max_bytes = max_bytes
                    .checked_add(group.iter().map(String::len).max().unwrap_or(0))
                    .ok_or_else(|| limit("per-result size overflow"))?;
                let sum: usize = group.iter().map(String::len).sum();
                aggregate = aggregate
                    .checked_add(
                        sum.checked_mul(count / group.len())
                            .ok_or_else(|| limit("aggregate size overflow"))?,
                    )
                    .ok_or_else(|| limit("aggregate size overflow"))?;
            }
            let escaped = |s: &str| -> Result<usize, OperationError> {
                serde_json::to_string(s)
                    .map(|s| s.len() - 2)
                    .map_err(|e| invalid("JSON string encoding failed").caused_by(e))
            };
            jsonl = count
                .checked_mul(3 + escaped(&separator)? * groups.len().saturating_sub(1))
                .ok_or_else(|| limit("JSONL separator size overflow"))?;
            for group in &groups {
                let sum = group
                    .iter()
                    .try_fold(0usize, |n, s| Ok::<_, OperationError>(n + escaped(s)?))?;
                jsonl = jsonl
                    .checked_add(
                        sum.checked_mul(count / group.len())
                            .ok_or_else(|| limit("JSONL byte count overflow"))?,
                    )
                    .ok_or_else(|| limit("JSONL byte count overflow"))?;
            }
        }
        if max_bytes > MAX_FRAGMENT_BYTES
            || aggregate > MAX_OUTPUT_BYTES
            || jsonl > MAX_OUTPUT_BYTES
        {
            return Err(limit(
                "combination result exceeds 64 KiB per string or 16 MiB aggregate/JSONL bytes",
            ));
        }
        let work = input_bytes
            .checked_mul(7)
            .and_then(|n| n.checked_add(aggregate))
            .and_then(|n| n.checked_add(jsonl))
            .and_then(|n| n.checked_add(count * groups.len() + fragments + groups.len()))
            .ok_or_else(|| limit("combination work overflow"))?;
        if work > 2 * MAX_OUTPUT_BYTES {
            return Err(limit(
                "combination validation/concatenation/JSONL work exceeds 32 MiB",
            ));
        }
        let plan = CombinationPlan {
            input_groups: groups.len(),
            total_results: count,
            input_bytes,
            max_result_bytes: max_bytes,
            aggregate_output_bytes: aggregate,
            jsonl_output_bytes: jsonl,
            work_bytes_upper_bound: work,
        };
        Ok(Self {
            groups,
            separator,
            plan,
            index: 0,
        })
    }
    pub fn plan(&self) -> &CombinationPlan {
        &self.plan
    }
}
impl Iterator for Combinations {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        if self.index == self.plan.total_results {
            return None;
        }
        let mut index = self.index;
        self.index += 1;
        let mut positions = [0usize; MAX_GROUPS];
        for i in (0..self.groups.len()).rev() {
            positions[i] = index % self.groups[i].len();
            index /= self.groups[i].len();
        }
        let mut result = String::with_capacity(self.plan.max_result_bytes);
        for (i, group) in self.groups.iter().enumerate() {
            if i > 0 {
                result.push_str(&self.separator);
            }
            result.push_str(&group[positions[i]]);
        }
        Some(result)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.plan.total_results - self.index;
        (remaining, Some(remaining))
    }
}
impl ExactSizeIterator for Combinations {}
/// Shared bounded JSON transport for CLI groups; no expression execution.
pub fn parse_groups(document: &str) -> Result<Vec<Vec<String>>, OperationError> {
    let value = crate::pipeline::json::parse(document)
        .map_err(|e| invalid("invalid bounded string-collection JSON").caused_by(e))?;
    let groups = value
        .as_array()
        .ok_or_else(|| invalid("groups must be an array of string arrays"))?;
    if groups.len() > MAX_GROUPS {
        return Err(limit("too many groups"));
    }
    let mut total = 0;
    let mut output = Vec::new();
    for group in groups {
        let group = group
            .as_array()
            .ok_or_else(|| invalid("each group must be a string array"))?;
        total += group.len();
        if total > MAX_FRAGMENTS {
            return Err(limit("too many fragments"));
        }
        output.push(
            group
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| invalid("fragments must be strings"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    Ok(output)
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "text.combinations",
    category: "text",
    description: "Incremental ordered Cartesian strings; rightmost group fastest, duplicates retained, zero groups=>one empty string, empty group=>zero. At most 16 groups/4096 fragments/1 MiB input, 10000 results, 64 KiB per result, 16 MiB raw/JSONL output and 32 MiB work; reject excessive plans, never truncate.",
    input_type: "text.collections",
    output_type: "text.iterator",
    parameters: &[
        Parameter {
            name: "separator",
            description: "Exact UTF-8 separator, <=4096 bytes; empty by default.",
            required: false,
            default: Some(ParameterValue::Text("")),
            constraints: ParameterType::Text { choices: &[] },
        },
        Parameter {
            name: "max_results",
            description: "Finite result cap, checked before generation.",
            required: false,
            default: Some(ParameterValue::Unsigned(1000)),
            constraints: ParameterType::Unsigned { min: 1, max: 10000 },
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "text combinations",
    example: "cube text combinations --text '[[\"a\",\"b\"],[\"1\",\"2\"]]'",
}];
