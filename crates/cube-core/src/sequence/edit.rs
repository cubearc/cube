//! Unit-cost edit distance with separate transposition and bounded algorithms.
use super::*;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EditMethod {
    #[default]
    Levenshtein,
    Dl,
    Osa,
    Ukkonen,
    MyersBitvector,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EditRequest {
    pub operands: Pair,
    #[serde(default)]
    pub method: EditMethod,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(range(max = 4096))]
    pub max_distance: Option<u16>,
}

#[derive(Serialize, JsonSchema)]
pub struct EditReport {
    pub method: EditMethod,
    pub distance: Option<u64>,
    pub above_threshold: bool,
}

/// Compute an exact edit distance, or a separate above-threshold result for Ukkonen.
pub fn edit_distance(request: &EditRequest) -> Result<EditReport, OperationError> {
    let (left, right) = request.operands.symbols(MAX_UNITS)?;
    let distance = match (request.method, request.max_distance) {
        (EditMethod::Ukkonen, Some(k @ 0..=4096)) => band(&left, &right, usize::from(k))?,
        (EditMethod::Ukkonen, _) | (_, Some(_)) => {
            return Err(parameter(
                "max_distance is required only for Ukkonen and must be in 0..=4096",
            ));
        }
        (method, None) => Some(match method {
            EditMethod::Dl => damerau(&left, &right)?,
            EditMethod::MyersBitvector => bit_vector(&left, &right)?,
            _ => {
                cells(left.len(), right.len(), DP_CELLS)?;
                rolling(&left, &right, matches!(method, EditMethod::Osa))
            }
        }),
    };
    Ok(EditReport {
        method: request.method,
        distance: distance.map(|n| n as u64),
        above_threshold: distance.is_none(),
    })
}

pub(super) fn levenshtein(left: &[u32], right: &[u32]) -> usize {
    rolling(left, right, false)
}

fn rolling(left: &[u32], right: &[u32], osa: bool) -> usize {
    let mut previous: Vec<usize> = (0..=right.len()).collect();
    let mut current = vec![0; previous.len()];
    let mut older = vec![0; previous.len()];
    for (i, &a) in left.iter().enumerate() {
        current[0] = i + 1;
        for (j, &b) in right.iter().enumerate() {
            let edit = (previous[j + 1] + 1)
                .min(current[j] + 1)
                .min(previous[j] + usize::from(a != b));
            current[j + 1] = if osa && i > 0 && j > 0 && left[i - 1] == b && right[j - 1] == a {
                edit.min(older[j - 1] + 1)
            } else {
                edit
            };
        }
        std::mem::swap(&mut older, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

fn damerau(left: &[u32], right: &[u32]) -> Result<usize, OperationError> {
    let width = right.len() + 2;
    let size = cells(left.len() + 2, width, DP_CELLS)?;
    let mut matrix = vec![(left.len() + right.len()) as u32; size];
    for i in 0..=left.len() {
        matrix[(i + 1) * width + 1] = i as u32;
    }
    for j in 0..=right.len() {
        matrix[width + j + 1] = j as u32;
    }
    let mut last = BTreeMap::new();
    for (i, &a) in left.iter().enumerate() {
        let mut equal_col = 0;
        for (j, &b) in right.iter().enumerate() {
            let old_row = *last.get(&b).unwrap_or(&0);
            let old_col = equal_col;
            if a == b {
                equal_col = j + 1;
            }
            let row = i + 2;
            let col = j + 2;
            matrix[row * width + col] = (matrix[(row - 1) * width + col] + 1)
                .min(matrix[row * width + col - 1] + 1)
                .min(matrix[(row - 1) * width + col - 1] + u32::from(a != b))
                .min(matrix[old_row * width + old_col] + (i + j + 1 - old_row - old_col) as u32);
        }
        last.insert(a, i + 1);
    }
    Ok(matrix[size - 1] as usize)
}

fn band(left: &[u32], right: &[u32], k: usize) -> Result<Option<usize>, OperationError> {
    if left.len().abs_diff(right.len()) > k {
        return Ok(None);
    }
    let width = right.len() + 1;
    let mut budget = Work(WORK);
    budget.charge(width)?;
    budget.charge(cells(left.len(), width.min(2 * k + 1), WORK)?)?;
    let beyond = k + 1;
    let mut previous: Vec<_> = (0..width).map(|j| j.min(beyond)).collect();
    let mut current = vec![beyond; width];
    for (i, &a) in left.iter().enumerate() {
        let row = i + 1;
        let low = row.saturating_sub(k);
        let high = (row + k).min(right.len());
        if low == 0 {
            current[0] = row;
        } else {
            current[low - 1] = beyond;
        }
        if high < right.len() {
            current[high + 1] = beyond;
        }
        for col in low.max(1)..=high {
            current[col] = (previous[col] + 1)
                .min(current[col - 1] + 1)
                .min(previous[col - 1] + usize::from(a != right[col - 1]))
                .min(beyond);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    let n = previous[right.len()];
    Ok((n <= k).then_some(n))
}

fn bit_vector(pattern: &[u32], text: &[u32]) -> Result<usize, OperationError> {
    if !(1..=63).contains(&pattern.len()) {
        return Err(parameter(
            "Myers bit-vector profile requires a left pattern of 1..=63 units",
        ));
    }
    let mut equality = BTreeMap::<u32, u64>::new();
    for (i, &symbol) in pattern.iter().enumerate() {
        *equality.entry(symbol).or_default() |= 1 << i;
    }
    let high = 1u64 << (pattern.len() - 1);
    let (mut positive, mut negative) = (!0u64, 0u64);
    let mut score = pattern.len();
    for symbol in text {
        let equal = *equality.get(symbol).unwrap_or(&0);
        let horizontal = (((equal & positive).wrapping_add(positive)) ^ positive) | equal;
        let vertical = equal | negative;
        let plus = negative | !(horizontal | positive);
        let minus = positive & horizontal;
        score += usize::from(plus & high != 0);
        score -= usize::from(minus & high != 0);
        let shifted_plus = (plus << 1) | 1;
        let shifted_minus = minus << 1;
        positive = shifted_minus | !(vertical | shifted_plus);
        negative = shifted_plus & vertical;
    }
    Ok(score)
}
