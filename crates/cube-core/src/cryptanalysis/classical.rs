//! Finite Caesar and Affine lists, reusing the exact Core inverse transforms.
use super::{candidates::*, *};
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Caesar {
    pub text: String,
    pub model: scoring::Model,
    /// None selects shifts 0..25; Some preserves caller order and duplicates.
    #[serde(default)]
    pub shifts: Option<Vec<i32>>,
    #[serde(default)]
    pub limits: Limits,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AffineKey {
    pub a: i32,
    pub b: i32,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Affine {
    pub text: String,
    pub model: scoring::Model,
    /// None selects the 312 coprime-a, b pairs in ascending a then b order.
    #[serde(default)]
    pub keys: Option<Vec<AffineKey>>,
    #[serde(default)]
    pub limits: Limits,
}
fn rank(
    text: &str,
    keys: Vec<Key>,
    model: &scoring::Model,
    limits: &Limits,
    cancel: Option<&AtomicBool>,
) -> Result<Report, OperationError> {
    text_bound(text, 65536)?;
    bound(keys.len(), 1024)?;
    let model = scoring::Prepared::new(model)?;
    let mut b = Budget::new(limits, cancel)?;
    let mut out = Retained::new(limits.max_results);
    let mut attempted = 0;
    for (i, key) in keys.into_iter().enumerate() {
        if !b.take(text.chars().count().max(1)) {
            break;
        }
        attempted += 1;
        let plaintext = match key {
            Key::Caesar { shift } if (0..26).contains(&shift) => {
                Ok(crate::ciphers::caesar_decode(text, shift))
            }
            Key::Affine { a, b } => crate::ciphers::classical::affine_decode(text, a, b),
            _ => Err(invalid("invalid finite key")),
        };
        let Ok(plaintext) = plaintext else {
            out.reject(i);
            continue;
        };
        let Some(score) = model.score(&plaintext, i, &mut b) else {
            break;
        };
        if !out.accept(
            plaintext,
            score,
            Alias {
                input_index: i,
                key,
                source_positions: vec![],
            },
            &mut b,
        ) {
            break;
        }
    }
    out.report(model, b, attempted)
}
/// Stable score ordering; exact plaintext dedup retains every attempted key alias.
pub fn caesar(r: &Caesar, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    json_bound(r, MAX_REQUEST)?;
    if let Some(keys) = &r.shifts {
        bound(keys.len(), 1024)?;
    }
    let keys = r
        .shifts
        .clone()
        .unwrap_or_else(|| (0..26).collect())
        .into_iter()
        .map(|shift| Key::Caesar { shift })
        .collect();
    rank(&r.text, keys, &r.model, &r.limits, cancel)
}
/// Invalid supplied pairs consume transform work and produce explicit rejections.
pub fn affine(r: &Affine, cancel: Option<&AtomicBool>) -> Result<Report, OperationError> {
    json_bound(r, MAX_REQUEST)?;
    if let Some(keys) = &r.keys {
        bound(keys.len(), 1024)?;
    }
    let keys = r
        .keys
        .clone()
        .unwrap_or_else(|| {
            [1, 3, 5, 7, 9, 11, 15, 17, 19, 21, 23, 25]
                .into_iter()
                .flat_map(|a| (0..26).map(move |b| AffineKey { a, b }))
                .collect()
        })
        .into_iter()
        .map(|k| Key::Affine { a: k.a, b: k.b })
        .collect();
    rank(&r.text, keys, &r.model, &r.limits, cancel)
}
