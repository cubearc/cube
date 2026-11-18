//! Shift-And exact search and Wu–Manber bit states for Levenshtein endings.
use super::*;

#[derive(Clone, Copy, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BitapMode {
    #[default]
    Exact,
    Levenshtein,
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BitapRequest {
    pub haystack: String,
    pub pattern: String,
    #[serde(default)]
    pub mode: BitapMode,
    #[serde(default)]
    #[schemars(range(max = 8))]
    pub max_errors: Option<usize>,
    #[serde(default)]
    pub start_witness: bool,
}
#[derive(Serialize, JsonSchema)]
pub struct EndingMatch {
    pub end: usize,
    pub distance: usize,
    pub start: Option<usize>,
}
#[derive(Serialize, JsonSchema)]
pub struct BitapReport {
    pub mode: BitapMode,
    pub matches: Vec<EndingMatch>,
}

fn errors(input: &BitapRequest, len: usize) -> Result<usize, OperationError> {
    if len == 0 {
        return Err(invalid("Bitap patterns must contain 1..=63 bytes"));
    }
    match (input.mode, input.max_errors) {
        (BitapMode::Exact, None) => Ok(0),
        (BitapMode::Levenshtein, Some(k)) if k <= 8 && k < len => Ok(k),
        _ => Err(parameter(
            "approximate mode alone requires max_errors in 0..=8 below pattern length",
        )),
    }
}

fn witness(
    h: &[u8],
    pattern: &[u8],
    end: usize,
    distance: usize,
    work: &mut Work,
) -> Result<usize, OperationError> {
    let first = end.saturating_sub(pattern.len() + distance);
    let last = end - (pattern.len() - distance);
    let pattern = pattern.iter().copied().map(u32::from).collect::<Vec<_>>();
    for start in first..=last {
        let count = end - start;
        work.charge((pattern.len() + 1) * (count + 1))?;
        let candidate = h[start..end]
            .iter()
            .copied()
            .map(u32::from)
            .collect::<Vec<_>>();
        if edit::levenshtein(&pattern, &candidate) == distance {
            return Ok(start);
        }
    }
    Err(invalid(
        "bit-state result has no matching bounded traceback",
    ))
}

/// Find nonempty approximate endings; start witnesses are computed explicitly.
pub fn bitap(input: &BitapRequest) -> Result<BitapReport, OperationError> {
    let (h, p) = (
        bytes(&input.haystack, HAYSTACK)?,
        bytes(&input.pattern, 63)?,
    );
    let k = errors(input, p.len())?;
    let mut work = Work(TRANSITIONS);
    work.charge(h.len() * (k + 1))?;
    let mut alphabet = [0; 256];
    for (i, &b) in p.iter().enumerate() {
        alphabet[b as usize] |= 1u64 << i;
    }
    let terminal = 1u64 << (p.len() - 1);
    let mut states = [0; 9];
    for (i, state) in states[..=k].iter_mut().enumerate() {
        *state = (1u64 << i) - 1;
    }
    let mut matches = Vec::new();
    for (index, &b) in h.iter().enumerate() {
        let mask = alphabet[b as usize];
        let mut previous = states[0];
        states[0] = ((states[0] << 1) | 1) & mask;
        for error in 1..=k {
            let old = states[error];
            states[error] = (((old << 1) | 1) & mask)
                | ((previous << 1) | 1)
                | previous
                | ((states[error - 1] << 1) | 1);
            previous = old;
        }
        if let Some(distance) = states[..=k].iter().position(|state| state & terminal != 0) {
            if matches.len() == RESULTS {
                return Err(resource("Bitap match count exceeds 65536"));
            }
            let end = index + 1;
            let start = if distance == 0 {
                Some(end - p.len())
            } else if input.start_witness {
                Some(witness(&h, &p, end, distance, &mut work)?)
            } else {
                None
            };
            matches.push(EndingMatch {
                end,
                distance,
                start,
            });
        }
    }
    Ok(BitapReport {
        mode: input.mode,
        matches,
    })
}
