//! Bounded numerical tone evidence shared by focused audio detectors.
use super::{
    pcm::Samples,
    spectrum::{self, invalid, limit},
};
use crate::errors::OperationError;
pub const MAX_FRAMES: usize = 262144;
pub const MAX_WINDOWS: usize = 4096;
pub(super) fn selected(samples: &Samples, op: &'static str) -> Result<Vec<f64>, OperationError> {
    if samples.channels().len() != 1 {
        return Err(
            invalid(op, "select exactly one channel; no implicit mixing").parameter("channels"),
        );
    }
    if samples.frames() as usize > MAX_FRAMES {
        return Err(limit(op, "signal analysis exceeds 262144 frames"));
    }
    Ok(spectrum::channel(samples, 0))
}
pub(super) fn rms(values: &[f64]) -> f64 {
    if values.is_empty() {
        0.0
    } else {
        (values.iter().map(|n| n * n).sum::<f64>() / values.len() as f64).sqrt()
    }
}
/// Generalized Goertzel evaluation at the stated frequency; no frequency search.
/// Return peak full-scale amplitude. Rectangular windows, finite PCM input only.
pub(super) fn amplitude(values: &[f64], rate: u32, hz: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let coefficient = 2.0 * (std::f64::consts::TAU * hz / f64::from(rate)).cos();
    let (mut previous, mut before) = (0.0, 0.0);
    for value in values {
        let next = value + coefficient * previous - before;
        before = previous;
        previous = next;
    }
    2.0 * (previous * previous + before * before - coefficient * previous * before)
        .max(0.0)
        .sqrt()
        / values.len() as f64
}
