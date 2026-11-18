//! Exact overlapping KMP, Boyer–Moore and verified Rabin–Karp.
use super::*;

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExactAlgorithm {
    Kmp,
    BoyerMoore,
    RabinKarp,
}
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExactRequest {
    pub haystack: String,
    pub pattern: String,
    pub algorithm: ExactAlgorithm,
    #[serde(default = "default_results")]
    #[schemars(range(max = 65536))]
    pub max_results: usize,
}
#[derive(Serialize, JsonSchema)]
pub struct ExactReport {
    pub algorithm: ExactAlgorithm,
    pub count: usize,
    pub complete: bool,
    pub matches: Vec<ByteRange>,
}

fn extend_match(
    value: u8,
    pattern: &[u8],
    table: &[usize],
    mut state: usize,
    work: &mut Work,
) -> Result<usize, OperationError> {
    loop {
        work.charge(1)?;
        if value == pattern[state] {
            return Ok(state + 1);
        }
        if state == 0 {
            return Ok(0);
        }
        state = table[state - 1];
    }
}

fn kmp(
    h: &[u8],
    p: &[u8],
    work: &mut Work,
    mut hit: impl FnMut(usize),
) -> Result<(), OperationError> {
    let mut table = vec![0; p.len()];
    for i in 1..p.len() {
        table[i] = extend_match(p[i], p, &table, table[i - 1], work)?;
    }
    let mut state = 0;
    for (i, &value) in h.iter().enumerate() {
        state = extend_match(value, p, &table, state, work)?;
        if state == p.len() {
            hit(i + 1 - p.len());
            state = table[state - 1];
        }
    }
    Ok(())
}

fn good_suffix(p: &[u8], work: &mut Work) -> Result<Vec<usize>, OperationError> {
    let mut shift = vec![0; p.len() + 1];
    let mut borders = vec![0; p.len() + 1];
    let (mut i, mut j) = (p.len(), p.len() + 1);
    borders[i] = j;
    while i > 0 {
        work.charge(1)?;
        while j <= p.len() && p[i - 1] != p[j - 1] {
            work.charge(1)?;
            if shift[j] == 0 {
                shift[j] = j - i;
            }
            j = borders[j];
        }
        i -= 1;
        j -= 1;
        borders[i] = j;
    }
    j = borders[0];
    for (i, value) in shift.iter_mut().enumerate() {
        work.charge(1)?;
        if *value == 0 {
            *value = j;
        }
        if i == j {
            j = borders[j];
        }
    }
    Ok(shift)
}

fn boyer_moore(
    h: &[u8],
    p: &[u8],
    work: &mut Work,
    mut hit: impl FnMut(usize),
) -> Result<(), OperationError> {
    let shift = good_suffix(p, work)?;
    let mut last = [None; 256];
    for (i, &b) in p.iter().enumerate() {
        last[b as usize] = Some(i);
    }
    work.charge(p.len())?;
    let mut base = 0;
    while base + p.len() <= h.len() {
        let mut k = p.len();
        while k > 0 {
            work.charge(1)?;
            if p[k - 1] != h[base + k - 1] {
                break;
            }
            k -= 1;
        }
        if k == 0 {
            hit(base);
            base += shift[0];
        } else {
            let bad = last[h[base + k - 1] as usize]
                .map_or(k, |prev| (k - 1).saturating_sub(prev).max(1));
            base += bad.max(shift[k]);
        }
    }
    Ok(())
}

fn rabin_karp(
    h: &[u8],
    p: &[u8],
    work: &mut Work,
    mut hit: impl FnMut(usize),
) -> Result<(), OperationError> {
    const BASE: u64 = 257;
    const PRIME: u64 = 1_000_000_007;
    let mut power = 1;
    let mut pattern = 0;
    let mut window = 0;
    work.charge(p.len() * 2)?;
    for (i, (&a, &b)) in p.iter().zip(h).enumerate() {
        pattern = (pattern * BASE + u64::from(a)) % PRIME;
        window = (window * BASE + u64::from(b)) % PRIME;
        if i > 0 {
            power = power * BASE % PRIME;
        }
    }
    for start in 0..=h.len() - p.len() {
        if window == pattern {
            let mut same = true;
            for (&a, &b) in p.iter().zip(&h[start..]) {
                work.charge(1)?;
                if a != b {
                    same = false;
                    break;
                }
            }
            if same {
                hit(start);
            }
        }
        if start + p.len() < h.len() {
            work.charge(1)?;
            let old = u64::from(h[start]) * power % PRIME;
            window = ((window + PRIME - old) * BASE + u64::from(h[start + p.len()])) % PRIME;
        }
    }
    Ok(())
}

/// Enumerate overlapping occurrences and optionally cap the retained listing.
pub fn exact_search(input: &ExactRequest) -> Result<ExactReport, OperationError> {
    matches_limit(input.max_results)?;
    let (h, p) = (
        bytes(&input.haystack, HAYSTACK)?,
        bytes(&input.pattern, PATTERN)?,
    );
    if p.is_empty() {
        return Err(invalid("exact search requires a nonempty pattern"));
    }
    let mut report = ExactReport {
        algorithm: input.algorithm,
        count: 0,
        complete: true,
        matches: Vec::new(),
    };
    if p.len() > h.len() {
        return Ok(report);
    }
    let mut work = Work(TRANSITIONS);
    let hit = |start| {
        report.count += 1;
        if report.matches.len() < input.max_results {
            report.matches.push(ByteRange {
                start,
                end: start + p.len(),
            });
        }
    };
    match input.algorithm {
        ExactAlgorithm::Kmp => kmp(&h, &p, &mut work, hit)?,
        ExactAlgorithm::BoyerMoore => boyer_moore(&h, &p, &mut work, hit)?,
        ExactAlgorithm::RabinKarp => rabin_karp(&h, &p, &mut work, hit)?,
    }
    report.complete = report.count == report.matches.len();
    Ok(report)
}
