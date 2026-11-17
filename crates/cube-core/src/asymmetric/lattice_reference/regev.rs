use super::{OperationError, invalid};

const Q: i32 = 97;
const DIMENSION: usize = 4;
const ROW_WIDTH: usize = DIMENSION + 1;
const CENTER: i32 = Q / 2;

fn canonical_residues(bytes: &[u8]) -> bool {
    bytes.iter().all(|&value| i32::from(value) < Q)
}

pub(super) fn encrypt(
    input: &[u8],
    key: &[u8],
    subset: [u8; 2],
) -> Result<Vec<u8>, OperationError> {
    if input[0] > 1 || !canonical_residues(key) {
        return Err(invalid());
    }
    let subset = u16::from_le_bytes(subset);
    let (rows, _) = key.as_chunks::<ROW_WIDTH>();
    let mut sum = [0; ROW_WIDTH];
    sum[DIMENSION] = CENTER * i32::from(input[0]);
    for (index, row) in rows.iter().enumerate() {
        let include = i32::from((subset >> index) & 1);
        for (value, component) in sum.iter_mut().zip(row) {
            *value += include * i32::from(*component);
        }
    }
    Ok(sum.map(|value| value.rem_euclid(Q) as u8).to_vec())
}

pub(super) fn decrypt(input: &[u8], key: &[u8]) -> Result<Vec<u8>, OperationError> {
    if !canonical_residues(input) || !canonical_residues(key) {
        return Err(invalid());
    }
    let dot: i32 = input
        .iter()
        .zip(key)
        .map(|(&a, &b)| i32::from(a) * i32::from(b))
        .sum();
    let residual = (i32::from(input[DIMENSION]) - dot).rem_euclid(Q);
    let distance_zero = residual.min(Q - residual);
    let shifted = (residual - CENTER).rem_euclid(Q);
    let distance_one = shifted.min(Q - shifted);
    if distance_zero == distance_one {
        return Err(invalid());
    }
    Ok(vec![u8::from(distance_one < distance_zero)])
}
