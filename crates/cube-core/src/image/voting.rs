//! Polar line votes over a caller-supplied binary edge mask.
use super::{binary_grid::mask_shape, invalid, limit, plane::MIB};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{cmp::Reverse, collections::BinaryHeap, f64::consts::PI};

fn one_step() -> f64 {
    1.
}
fn one_vote() -> u32 {
    1
}
fn default_peaks() -> usize {
    32
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HoughOptions {
    #[schemars(range(min = 1, max = 720))]
    pub theta_bins: usize,
    #[serde(default = "one_step")]
    pub rho_step: f64,
    #[serde(default = "one_vote")]
    pub minimum_votes: u32,
    #[serde(default = "default_peaks")]
    #[schemars(range(min = 1, max = 256))]
    pub max_peaks: usize,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Peak {
    pub theta_index: usize,
    pub rho_index: usize,
    pub theta: f64,
    pub rho: f64,
    pub votes: u32,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    pub options: HoughOptions,
    pub rho_origin: f64,
    pub rho_bins: usize,
    pub foreground: usize,
    pub accumulator: Vec<u32>,
    pub peaks: Vec<Peak>,
}

/// Vote in a theta-major accumulator. Rho centers start at minus the ceiled
/// image diagonal, advance by the requested step and stay inside the span.
/// Half-bin ties use the increasing index. Peaks sort by votes then indices.
pub fn hough(
    width: u32,
    height: u32,
    mask: &[u8],
    options: HoughOptions,
) -> Result<Report, OperationError> {
    (|| {
        let (width, _) = mask_shape(width, height, mask, 262_144)?;
        if !(1..=720).contains(&options.theta_bins)
            || !(1..=256).contains(&options.max_peaks)
            || !options.rho_step.is_finite()
            || options.rho_step <= 0.
            || options.minimum_votes == 0
        {
            return Err(invalid(
                "invalid Hough bins, step, minimum votes or peak count",
            ));
        }
        let radius = (width as f64).hypot(f64::from(height)).ceil();
        let bins = (2. * radius / options.rho_step).floor() + 1.;
        if bins > (MIB / options.theta_bins) as f64 {
            return Err(limit("Hough accumulator exceeds 1 Mi cells"));
        }
        let rho_bins = bins as usize;
        let foreground = mask.iter().filter(|&&sample| sample == 255).count();
        if foreground > 65_536 || foreground * options.theta_bins > 16 * MIB {
            return Err(limit(
                "Hough exceeds 65536 foreground pixels or 16 Mi votes",
            ));
        }
        let theta = |index: usize| index as f64 * PI / options.theta_bins as f64;
        let rho = |index: usize| -radius + index as f64 * options.rho_step;
        let mut accumulator = vec![0u32; rho_bins * options.theta_bins];
        let angles: Vec<_> = (0..options.theta_bins)
            .map(|index| theta(index).sin_cos())
            .collect();
        for (position, _) in mask
            .iter()
            .enumerate()
            .filter(|&(_, &sample)| sample == 255)
        {
            let (x, y) = ((position % width) as f64, (position / width) as f64);
            for (angle, &(sin, cos)) in angles.iter().enumerate() {
                let value = x * cos + y * sin;
                let index = ((value + radius) / options.rho_step + 0.5).floor() as usize;
                accumulator[angle * rho_bins + index.min(rho_bins - 1)] += 1;
            }
        }
        let mut ranked = BinaryHeap::with_capacity(options.max_peaks + 1);
        for (i, &votes) in accumulator.iter().enumerate() {
            if votes >= options.minimum_votes {
                ranked.push((Reverse(votes), i));
                if ranked.len() > options.max_peaks {
                    ranked.pop();
                }
            }
        }
        let peaks = ranked
            .into_sorted_vec()
            .into_iter()
            .map(|(Reverse(votes), i)| {
                let (theta_index, rho_index) = (i / rho_bins, i % rho_bins);
                Peak {
                    theta_index,
                    rho_index,
                    theta: theta(theta_index),
                    rho: rho(rho_index),
                    votes,
                }
            })
            .collect();
        Ok(Report {
            options,
            rho_origin: -radius,
            rho_bins,
            foreground,
            accumulator,
            peaks,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.hough.lines"))
}
