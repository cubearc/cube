//! Equal-length positional comparisons without allocating a sequence copy.
use super::*;
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::Serialize;

const MAX_PACKED: usize = 16 << 20;
const CHUNK: usize = 4096;

/// Select bytes, Unicode scalars, or every bit of equal-length packed bytes.
#[derive(Deserialize, JsonSchema)]
#[serde(tag = "unit", rename_all = "snake_case", deny_unknown_fields)]
pub enum HammingRequest {
    Bytes { left: String, right: String },
    Bits { left: String, right: String },
    Scalars { left: String, right: String },
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct HammingReport {
    pub differing_units: u64,
    pub total_units: u64,
    pub normalized: Option<f64>,
}

fn report(diff: u64, total: u64) -> HammingReport {
    HammingReport {
        differing_units: diff,
        total_units: total,
        normalized: (total != 0).then(|| diff as f64 / total as f64),
    }
}

fn unequal() -> OperationError {
    invalid("Hamming distance requires equal lengths in the selected unit")
}

fn decode(chunk: &[u8], output: &mut [u8], final_chunk: bool) -> Result<usize, OperationError> {
    if !final_chunk && chunk.ends_with(b"=") {
        return Err(invalid(
            "padding is only valid at the end of a byte sequence",
        ));
    }
    STANDARD
        .decode_slice(chunk, output)
        .map_err(|_| invalid("packed sequences require canonical padded Base64"))
}

fn decoded_length(value: &str) -> Result<usize, OperationError> {
    if !value.len().is_multiple_of(4) {
        return Err(invalid("packed sequences require canonical padded Base64"));
    }
    let padding = usize::from(value.ends_with('=')) + usize::from(value.ends_with("=="));
    let size = value.len() / 4 * 3 - padding;
    if size > MAX_PACKED {
        return Err(resource("packed byte sequence exceeds 16 MiB"));
    }
    Ok(size)
}

fn packed(left: &str, right: &str, bits: bool) -> Result<HammingReport, OperationError> {
    let cap = base64::encoded_len(MAX_PACKED, true).unwrap();
    if left.len().max(right.len()) > cap {
        return Err(resource("packed byte sequence exceeds 16 MiB"));
    }
    let length = decoded_length(left)?;
    if length != decoded_length(right)? {
        return Err(unequal());
    }
    let mut left_bytes = [0; CHUNK / 4 * 3];
    let mut right_bytes = [0; CHUNK / 4 * 3];
    let mut diff = 0;
    for (i, (a, b)) in left
        .as_bytes()
        .chunks(CHUNK)
        .zip(right.as_bytes().chunks(CHUNK))
        .enumerate()
    {
        let last = (i + 1) * CHUNK >= left.len();
        let len = decode(a, &mut left_bytes, last)?;
        if len != decode(b, &mut right_bytes, last)? {
            return Err(unequal());
        }
        diff += left_bytes[..len]
            .iter()
            .zip(&right_bytes[..len])
            .map(|(a, b)| {
                if bits {
                    u64::from((a ^ b).count_ones())
                } else {
                    u64::from(a != b)
                }
            })
            .sum::<u64>();
    }
    Ok(report(diff, length as u64 * if bits { 8 } else { 1 }))
}

/// Count positional mismatches; the normalized ratio is undefined for empty pairs.
pub fn hamming(request: &HammingRequest) -> Result<HammingReport, OperationError> {
    match request {
        HammingRequest::Bytes { left, right } => packed(left, right, false),
        HammingRequest::Bits { left, right } => packed(left, right, true),
        HammingRequest::Scalars { left, right } => {
            let len = scalar_count(left, usize::MAX)?;
            if len != scalar_count(right, usize::MAX)? {
                return Err(unequal());
            }
            Ok(report(
                left.chars()
                    .zip(right.chars())
                    .filter(|(a, b)| a != b)
                    .count() as u64,
                len as u64,
            ))
        }
    }
}
