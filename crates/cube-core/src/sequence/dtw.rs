//! Closed-endpoint dynamic time warping with compact bounded band rows.
use super::*;
use serde::Serialize;

#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cost {
    #[default]
    Squared,
    Absolute,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DtwRequest {
    #[schemars(length(max = 65536))]
    pub left: Vec<f64>,
    #[schemars(length(max = 65536))]
    pub right: Vec<f64>,
    #[serde(default)]
    pub cost: Cost,
    #[serde(default)]
    #[schemars(range(max = 65536))]
    pub radius: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct DtwReport {
    pub cost: f64,
    pub path: Vec<[usize; 2]>,
}

#[derive(Clone, Copy)]
struct Row {
    start: usize,
    end: usize,
    offset: usize,
}

fn bands(n: usize, m: usize, radius: usize) -> Result<(Vec<Row>, usize), OperationError> {
    let mut work = Work(1 << 20);
    let mut rows = Vec::with_capacity(n);
    let mut offset = 0;
    for i in 0..n {
        let row = Row {
            start: i.saturating_sub(radius),
            end: (i + radius + 1).min(m),
            offset,
        };
        let width = row.end.saturating_sub(row.start);
        work.charge(width)?;
        rows.push(row);
        offset += width;
    }
    Ok((rows, offset))
}

fn cost(input: &DtwRequest, a: usize, b: usize) -> f64 {
    let diff = input.left[a] - input.right[b];
    match input.cost {
        Cost::Squared => diff * diff,
        Cost::Absolute => diff.abs(),
    }
}

fn path(rows: &[Row], trace: &[u8], mut i: usize, mut j: usize) -> Vec<[usize; 2]> {
    let mut result = Vec::new();
    while i != 0 || j != 0 {
        result.push([i, j]);
        let row = rows[i];
        match trace[row.offset + j - row.start] {
            0 => {
                i -= 1;
                j -= 1;
            }
            1 => i -= 1,
            _ => j -= 1,
        }
    }
    result.push([0, 0]);
    result.reverse();
    result
}

/// Minimize accumulated sample cost with exact endpoints and a deterministic path.
pub fn dtw(input: &DtwRequest) -> Result<DtwReport, OperationError> {
    let (n, m) = (input.left.len(), input.right.len());
    if n.max(m) > MAX_UNITS || input.radius.is_some_and(|r| r > MAX_UNITS) {
        return Err(resource("DTW series or radius exceeds its bound"));
    }
    if input
        .left
        .iter()
        .chain(&input.right)
        .any(|x| !x.is_finite() || x.abs() > 1e150)
    {
        return Err(invalid("DTW samples must be finite and within ±1e150"));
    }
    if n == 0 && m == 0 {
        return Ok(DtwReport {
            cost: 0.0,
            path: Vec::new(),
        });
    }
    let radius = input.radius.unwrap_or(n.max(m));
    if n == 0 || m == 0 || n.abs_diff(m) > radius {
        return Err(invalid(
            "DTW requires connected, nonempty endpoints on both sides",
        ));
    }
    let (rows, total) = bands(n, m, radius)?;
    let mut trace = vec![0; total];
    let mut previous = vec![f64::INFINITY; m];
    let mut current = vec![f64::INFINITY; m];
    let mut previous_bounds = 0..0;
    for (i, row) in rows.iter().copied().enumerate() {
        let get = |j: usize| {
            if previous_bounds.contains(&j) {
                previous[j]
            } else {
                f64::INFINITY
            }
        };
        for j in row.start..row.end {
            let mut choice = (if j > 0 { get(j - 1) } else { f64::INFINITY }, 0);
            let up = get(j);
            if up < choice.0 {
                choice = (up, 1);
            }
            if j > row.start && current[j - 1] < choice.0 {
                choice = (current[j - 1], 2);
            }
            if i == 0 && j == 0 {
                choice = (0.0, 0);
            }
            trace[row.offset + j - row.start] = choice.1;
            current[j] = choice.0 + cost(input, i, j);
        }
        std::mem::swap(&mut current, &mut previous);
        previous_bounds = row.start..row.end;
    }
    let total = previous[m - 1];
    if !total.is_finite() {
        return Err(invalid("DTW endpoint does not have a finite path"));
    }
    Ok(DtwReport {
        cost: total,
        path: path(&rows, &trace, n - 1, m - 1),
    })
}
