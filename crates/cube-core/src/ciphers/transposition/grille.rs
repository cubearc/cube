//! Explicit masks and caller-owned cover text. No cover-text generation.
use super::*;
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TurningGrille {
    #[schemars(range(min = 2, max = 64))]
    #[serde(deserialize_with = "wire::integer")]
    pub side: usize,
    /// Zero-based row-major holes in orientation zero. Must be strictly sorted.
    #[schemars(length(min = 1, max = 1024))]
    #[serde(deserialize_with = "wire::indices")]
    pub holes: Vec<usize>,
    /// Quarter-turn counts 0..3; explicit order must visit all four orientations.
    #[schemars(length(equal = 4))]
    #[serde(deserialize_with = "wire::indices")]
    pub orientations: Vec<usize>,
}
fn turning_order(p: &TurningGrille, op: &'static str) -> Result<Vec<usize>, OperationError> {
    if !(2..=64).contains(&p.side) || !p.side.is_multiple_of(2) {
        return Err(invalid(
            op,
            "side",
            "turning grille needs an even side 2..64",
        ));
    }
    validate_order(&p.orientations, 4, op)?;
    let n = p.side * p.side;
    if p.holes.len() != n / 4
        || p.holes.windows(2).any(|w| w[0] >= w[1])
        || p.holes.iter().any(|&i| i >= n)
    {
        return Err(invalid(
            op,
            "holes",
            "holes must be sorted distinct cells covering one quarter of the square",
        ));
    }
    let mut order = Vec::with_capacity(n);
    for &rotation in &p.orientations {
        let mut holes: Vec<_> = p
            .holes
            .iter()
            .map(|&i| {
                let (mut r, mut c) = (i / p.side, i % p.side);
                for _ in 0..rotation {
                    (r, c) = (c, p.side - 1 - r);
                }
                r * p.side + c
            })
            .collect();
        holes.sort_unstable();
        order.extend(holes);
    }
    validate_order(&order, n, op)?;
    Ok(order)
}
/// Fill rotated holes row-major in caller-specified orientation order, read rows.
/// Text must contain exact blocks; explicit caller padding is never inferred.
pub fn turning_grille(
    text: &str,
    p: &TurningGrille,
    decode: bool,
) -> Result<String, OperationError> {
    let op = if decode {
        "cipher.turning_grille.decode"
    } else {
        "cipher.turning_grille.encode"
    };
    let x = scalars(text, op)?;
    let fill = turning_order(p, op)?;
    if !x.len().is_multiple_of(fill.len()) {
        return Err(invalid(
            op,
            "text",
            "turning grille requires complete blocks",
        ));
    }
    Ok(x.chunks(fill.len())
        .map(|block| apply(block, &fill, !decode))
        .collect())
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Cardan {
    #[schemars(range(min = 1, max = 65536))]
    #[serde(deserialize_with = "wire::integer")]
    pub rows: usize,
    #[schemars(range(min = 1, max = 65536))]
    #[serde(deserialize_with = "wire::integer")]
    pub columns: usize,
    /// Selected cover positions in strictly increasing row-major order.
    #[schemars(length(max = 65536))]
    #[serde(deserialize_with = "wire::indices")]
    pub holes: Vec<usize>,
}
fn mask(p: &Cardan, n: usize, op: &'static str) -> Result<(), OperationError> {
    if p.rows == 0 || p.columns == 0 || p.rows.checked_mul(p.columns) != Some(n) {
        return Err(invalid(
            op,
            "shape",
            "cover must fill the exact positive rectangle",
        ));
    }
    bound(op, "holes", p.holes.len(), MAX_SYMBOLS)?;
    if p.holes.windows(2).any(|w| w[0] >= w[1]) || p.holes.iter().any(|&i| i >= n) {
        return Err(invalid(
            op,
            "holes",
            "holes must be strictly sorted valid cover positions",
        ));
    }
    Ok(())
}
/// Extract selected cover scalars. Source-dependent and not generally reversible.
pub fn cardan_extract(cover: &str, p: &Cardan) -> Result<String, OperationError> {
    let op = "cipher.cardan.extract";
    let x = scalars(cover, op)?;
    mask(p, x.len(), op)?;
    Ok(p.holes.iter().map(|&i| x[i]).collect())
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Overlay {
    #[serde(flatten)]
    pub mask: Cardan,
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=262144))]
    pub replacements: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OverlayReport {
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=262144))]
    pub text: String,
    #[schemars(length(max = 65536))]
    pub positions: Vec<usize>,
    #[schemars(length(max=65536),extend("x-cube-max-utf8-bytes"=262144))]
    pub original_selected: String,
}
/// Overlay caller replacements, returning exact positions and original evidence.
pub fn cardan_overlay(cover: &str, p: &Overlay) -> Result<OverlayReport, OperationError> {
    let op = "cipher.cardan.overlay";
    let mut x = scalars(cover, op)?;
    mask(&p.mask, x.len(), op)?;
    let replacement = scalars(&p.replacements, op)?;
    if replacement.len() != p.mask.holes.len() {
        return Err(invalid(
            op,
            "replacements",
            "one replacement scalar per selected hole is required",
        ));
    }
    let original_selected = p.mask.holes.iter().map(|&i| x[i]).collect();
    for (&i, &c) in p.mask.holes.iter().zip(&replacement) {
        x[i] = c;
    }
    Ok(OverlayReport {
        text: x.into_iter().collect(),
        positions: p.mask.holes.clone(),
        original_selected,
    })
}
