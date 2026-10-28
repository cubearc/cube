//! Exact repeat/column observations and explicitly heuristic period ranking.
use super::*;
use std::collections::BTreeMap;
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub text: String,
    #[schemars(length(min = 1, max = 4096))]
    pub periods: Vec<usize>,
    #[schemars(range(min = 2, max = 8))]
    pub ngram: usize,
    pub expected_ic: f64,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Repeat {
    pub ngram: String,
    pub positions: Vec<usize>,
    pub original_byte_offsets: Vec<usize>,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Distance {
    pub distance: usize,
    pub pairs: u64,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Period {
    pub period: usize,
    pub valid_columns: usize,
    pub mean_column_ic: Option<f64>,
    pub divisor_hits: u64,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub method: String,
    pub status: Status,
    pub letters: usize,
    pub ic: Option<f64>,
    pub friedman: Option<f64>,
    pub uniform_ic: f64,
    pub expected_ic: f64,
    pub repeats: Vec<Repeat>,
    pub distances: Vec<Distance>,
    pub periods: Vec<Period>,
    pub ranking: Vec<usize>,
}
fn ic(counts: &[u64; 26]) -> Option<f64> {
    let n: u64 = counts.iter().sum();
    (n > 1).then(|| {
        counts.iter().map(|&c| c * c.saturating_sub(1)).sum::<u64>() as f64 / (n * (n - 1)) as f64
    })
}
fn counts(text: &[u8]) -> [u64; 26] {
    let mut counts = [0; 26];
    for &b in text {
        counts[(b - b'A') as usize] += 1;
    }
    counts
}
fn repeats(text: &[u8], offsets: &[usize], width: usize, b: &mut Budget<'_>) -> Vec<Repeat> {
    let mut groups: BTreeMap<&[u8], Vec<usize>> = BTreeMap::new();
    for (i, w) in text.windows(width).enumerate() {
        if i == 65536 {
            b.stop(Stop::ResultLimit);
            break;
        }
        if !b.take(width) {
            break;
        }
        groups.entry(w).or_default().push(i);
    }
    groups
        .into_iter()
        .filter(|(_, p)| p.len() > 1)
        .map(|(key, positions)| Repeat {
            ngram: String::from_utf8(key.to_vec()).expect("ASCII"),
            original_byte_offsets: positions.iter().map(|&i| offsets[i]).collect(),
            positions,
        })
        .collect()
}
fn distances(repeats: &[Repeat], periods: &mut [Period], b: &mut Budget<'_>) -> Vec<Distance> {
    let mut counts = BTreeMap::new();
    'outer: for r in repeats {
        for (i, &a) in r.positions.iter().enumerate() {
            for &z in &r.positions[i + 1..] {
                if !b.take(1 + periods.len()) {
                    break 'outer;
                }
                *counts.entry(z - a).or_insert(0) += 1;
                for p in &mut *periods {
                    if (z - a) % p.period == 0 {
                        p.divisor_hits += 1;
                    }
                }
            }
        }
    }
    counts
        .into_iter()
        .map(|(distance, pairs)| Distance { distance, pairs })
        .collect()
}
/// Kasiski pairs use all recorded positions, with every pair/divisor charged.
/// Column IC excludes columns shorter than two. Friedman is undefined for N<2
/// or a nonpositive denominator. No candidate length is silently inferred.
pub fn estimate(r: &Request, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    text_bound(&r.text, MAX_TEXT)?;
    bound(r.periods.len(), 4096)?;
    if r.periods.is_empty()
        || r.periods.iter().any(|p| !(1..=4096).contains(p))
        || !(2..=8).contains(&r.ngram)
        || !r.expected_ic.is_finite()
        || !(1.0 / 26.0..=1.0).contains(&r.expected_ic)
    {
        return Err(invalid(
            "periods 1..4096, ngram 2..8 and expected IC in [1/26,1] required",
        ));
    }
    let mut budget = Budget::new(&r.limits, cancel)?;
    let (text, offsets) = letters(&r.text);
    let bytes = text.as_bytes();
    let observed = if budget.take(r.text.chars().count().max(1)) {
        ic(&counts(bytes))
    } else {
        None
    };
    let uniform = 1.0 / 26.0;
    let friedman = observed.and_then(|i| {
        let n = bytes.len() as f64;
        let denominator = (n - 1.0) * i - uniform * n + r.expected_ic;
        (denominator > 0.0).then_some((r.expected_ic - uniform) * n / denominator)
    });
    let mut periods = Vec::new();
    if budget.status.complete {
        for &period in &r.periods {
            if periods.len() == r.limits.max_results {
                budget.stop(Stop::ResultLimit);
                break;
            }
            if !budget.take(bytes.len() + period) {
                break;
            }
            let mut columns = vec![[0u64; 26]; period];
            for (i, &c) in bytes.iter().enumerate() {
                columns[i % period][(c - b'A') as usize] += 1;
            }
            let ics: Vec<_> = columns.iter().filter_map(ic).collect();
            periods.push(Period {
                period,
                valid_columns: ics.len(),
                mean_column_ic: (!ics.is_empty())
                    .then(|| ics.iter().sum::<f64>() / ics.len() as f64),
                divisor_hits: 0,
            });
        }
    }
    let repeats = if budget.status.complete {
        repeats(bytes, &offsets, r.ngram, &mut budget)
    } else {
        vec![]
    };
    let distances = if budget.status.stop == Stop::Cancelled {
        vec![]
    } else {
        distances(&repeats, &mut periods, &mut budget)
    };
    let mut ranking: Vec<_> = (0..periods.len()).collect();
    ranking.sort_by(|&a, &b| {
        let gap = |i: usize| {
            periods[i]
                .mean_column_ic
                .map(|x| (x - r.expected_ic).abs())
                .unwrap_or(f64::INFINITY)
        };
        gap(a).total_cmp(&gap(b))
    });
    finish(Report {
        method: "ascii-kasiski-column-ic-friedman-v1".into(),
        status: budget.status,
        letters: bytes.len(),
        ic: observed,
        friedman,
        uniform_ic: uniform,
        expected_ic: r.expected_ic,
        repeats,
        distances,
        periods,
        ranking,
    })
}
