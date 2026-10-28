//! Deterministic column scoring and explicitly pruned Vigenere beam search.
use super::{candidates::*, *};
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub text: String,
    pub model: scoring::Model,
    #[serde(deserialize_with = "wire::indices")]
    pub lengths: Vec<usize>,
    #[serde(deserialize_with = "wire::integer")]
    pub beam_width: usize,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Report {
    pub original_byte_offsets: Vec<usize>,
    pub pruned_states: usize,
    #[serde(flatten)]
    pub ranking: candidates::Report,
}
fn retain(
    beam: &mut Vec<(String, i64)>,
    candidate: (String, i64),
    width: usize,
    b: &mut Budget<'_>,
    pruned: &mut usize,
) -> bool {
    let (mut lo, mut hi) = (0, beam.len());
    while lo < hi {
        if !b.take(candidate.0.len() + 1) {
            return false;
        }
        let mid = (lo + hi) / 2;
        let order = beam[mid]
            .1
            .cmp(&candidate.1)
            .reverse()
            .then_with(|| beam[mid].0.cmp(&candidate.0));
        if order.is_lt() {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    if lo < width {
        beam.insert(lo, candidate);
    }
    if lo >= width || beam.len() > width {
        *pruned += 1;
        beam.truncate(width);
    }
    true
}
fn beam(
    text: &str,
    length: usize,
    width: usize,
    model: &scoring::Prepared,
    b: &mut Budget<'_>,
    pruned: &mut usize,
) -> Option<Vec<(String, i64)>> {
    let mut beam = vec![(String::new(), 0i64)];
    for column in 0..length {
        let indices = (column..text.len()).step_by(length);
        if !b.take(indices.len().max(1)) {
            return None;
        }
        let letters: String = indices.map(|i| char::from(text.as_bytes()[i])).collect();
        let mut shifts = Vec::new();
        for shift in 0..26 {
            if !b.take(letters.len().max(1)) {
                return None;
            }
            let plain = crate::ciphers::caesar_decode(&letters, shift);
            let score = model.score(&plain, shift as usize, b)?;
            shifts.push(score.mean_nanos.unwrap_or(i64::MIN / 128));
        }
        let mut expanded = Vec::new();
        for (prefix, score) in beam {
            for (shift, &s) in shifts.iter().enumerate() {
                // Charge every partial-key copy, including branches pruned below.
                if !b.take(prefix.len() + 1) {
                    return None;
                }
                let mut key = prefix.clone();
                key.push(char::from(b'A' + shift as u8));
                if !retain(&mut expanded, (key, score + s), width, b, pruned) {
                    return None;
                }
            }
        }
        beam = expanded;
    }
    Some(beam)
}
/// Each column tries all 26 shifts. Beam ties use lexical keys; final plaintexts
/// are rescored with the same full model. Any pruning marks non-exhaustion.
pub fn rank(r: &Request, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    json_bound(r, MAX_REQUEST)?;
    text_bound(&r.text, 65536)?;
    bound(r.lengths.len(), 32)?;
    if r.lengths.is_empty()
        || r.lengths.iter().any(|n| !(1..=64).contains(n))
        || !(1..=256).contains(&r.beam_width)
    {
        return Err(invalid("lengths 1..64 and beam width 1..256 required"));
    }
    let model = scoring::Prepared::new(&r.model)?;
    let mut b = Budget::new(&r.limits, cancel)?;
    let (letters, offsets) = letters(&r.text);
    let mut out = Retained::new(r.limits.max_results);
    let mut attempted = 0;
    let mut pruned = 0;
    'outer: for &length in &r.lengths {
        let Some(keys) = beam(&letters, length, r.beam_width, &model, &mut b, &mut pruned) else {
            break;
        };
        for (key, _) in keys {
            if !b.take(r.text.chars().count().max(1)) {
                break 'outer;
            }
            let i = attempted;
            attempted += 1;
            let plain = crate::ciphers::classical::vigenere_decode(&r.text, &key)?;
            let Some(score) = model.score(&plain, i, &mut b) else {
                break 'outer;
            };
            if !out.accept(
                plain,
                score,
                Alias {
                    input_index: i,
                    key: Key::Vigenere { key },
                    source_positions: vec![],
                },
                &mut b,
            ) {
                break 'outer;
            }
        }
    }
    if pruned > 0 && b.status.complete {
        b.stop(Stop::Pruned);
    }
    finish(Report {
        original_byte_offsets: offsets,
        pruned_states: pruned,
        ranking: out.report(model, b, attempted)?,
    })
}
