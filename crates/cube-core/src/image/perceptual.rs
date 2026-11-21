//! Explicit versioned similarity fingerprints of opaque images.
use super::{
    ColorModel, Image, invalid, limit,
    resample::{self, Kernel},
};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Ahash8,
    Dhash8,
    #[serde(rename = "phash32-8-v1")]
    Phash32_8V1,
}

/// Compute an MSB-first, eight-byte similarity fingerprint using the specified
/// bilinear size and the existing integer gray conversion.
pub fn digest(image: &Image, profile: Profile) -> Result<[u8; 8], OperationError> {
    (|| {
        resample::physical_samples(image)?;
        if !matches!(image.model(), ColorModel::Gray | ColorModel::Rgb) {
            return Err(invalid(
                "perceptual hash requires an opaque gray or RGB view",
            ));
        }
        if u64::from(image.width()) * u64::from(image.height()) > 1 << 20 {
            return Err(limit("perceptual hash exceeds 1 Mi source pixels"));
        }
        let (width, height) = match profile {
            Profile::Ahash8 => (8, 8),
            Profile::Dhash8 => (9, 8),
            Profile::Phash32_8V1 => (32, 32),
        };
        let resized = resample::resize(image, width, height, Kernel::Bilinear)?;
        let gray = resized.grayscale()?;
        let values = gray.samples();
        let bits = match profile {
            Profile::Ahash8 => {
                let sum: u32 = values.iter().map(|sample| u32::from(*sample)).sum();
                std::array::from_fn(|i| 64 * u32::from(values[i]) >= sum)
            }
            Profile::Dhash8 => std::array::from_fn(|i| {
                let position = (i / 8) * 9 + i % 8;
                values[position] > values[position + 1]
            }),
            Profile::Phash32_8V1 => dct_bits(values),
        };
        Ok(std::array::from_fn(|i| {
            (0..8).fold(0, |byte, j| (byte << 1) | u8::from(bits[8 * i + j]))
        }))
    })()
    .map_err(|error: OperationError| error.operation("image.hash.perceptual"))
}

fn basis() -> [[f64; 16]; 8] {
    std::array::from_fn(|k| {
        std::array::from_fn(|n| {
            if k == 0 {
                1. / 32_f64.sqrt()
            } else {
                let angle = (2 * n + 1) as f64 * k as f64 * std::f64::consts::PI / 64.;
                angle.cos() / 4.
            }
        })
    })
}

fn projection(basis: &[[f64; 16]; 8], k: usize, value: impl Fn(usize) -> f64) -> f64 {
    // Reflection produces symmetric or antisymmetric basis pairs. For non-DC
    // even orders, subtract the first sample using the basis's zero total.
    let baseline = if k != 0 && k.is_multiple_of(2) {
        2. * value(0)
    } else {
        0.
    };
    let sign = if k.is_multiple_of(2) { 1. } else { -1. };
    (0..16)
        .map(|i| (value(i) + sign * value(31 - i) - baseline) * basis[k][i])
        .sum()
}

fn dct_bits(samples: &[u8]) -> [bool; 64] {
    let weights = basis();
    let horizontal: [[f64; 8]; 32] = std::array::from_fn(|row| {
        std::array::from_fn(|k| {
            projection(&weights, k, |column| f64::from(samples[row * 32 + column]))
        })
    });
    let coefficients: [f64; 64] =
        std::array::from_fn(|i| projection(&weights, i / 8, |row| horizontal[row][i % 8]));
    let mut ac = coefficients;
    ac[1..].sort_by(f64::total_cmp);
    let median = ac[32];
    std::array::from_fn(|i| i != 0 && coefficients[i] > median)
}
