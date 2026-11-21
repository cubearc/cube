//! Explicit local smoothing of physical image samples.
use super::{
    Channel, Image, invalid, limit,
    plane::{MIB, border_index},
    resample,
};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Filter {
    Gaussian {
        #[schemars(range(min = 1, max = 15))]
        radius: u8,
        sigma: f64,
    },
    Median {
        #[schemars(range(min = 3, max = 15))]
        width: u8,
    },
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum FilterBorder {
    #[default]
    Replicate,
    Constant {
        sample: u8,
    },
}

pub(super) fn gaussian_weights(radius: u8, sigma: f64) -> Result<Vec<f64>, OperationError> {
    if !(1..=15).contains(&radius) || !sigma.is_finite() || sigma <= 0. {
        return Err(invalid(
            "Gaussian radius must be 1..15 and sigma positive and finite",
        ));
    }
    let r = i32::from(radius);
    let mut weights: Vec<_> = (-r..=r)
        .map(|i| {
            let scaled = f64::from(i) / sigma;
            (-0.5 * scaled * scaled).exp()
        })
        .collect();
    let sum: f64 = weights.iter().sum();
    for weight in &mut weights {
        *weight /= sum;
    }
    Ok(weights)
}

pub(super) fn gaussian_plane(
    width: usize,
    height: usize,
    kernel: &[f64],
    border: FilterBorder,
    sample: impl Fn(usize, usize) -> f64,
) -> Vec<f64> {
    let mut horizontal = Vec::with_capacity(width * height);
    let replicate = matches!(border, FilterBorder::Replicate);
    let background = match border {
        FilterBorder::Constant { sample } => f64::from(sample),
        _ => 0.,
    };
    let radius = (kernel.len() / 2) as isize;
    for y in 0..height {
        for x in 0..width {
            horizontal.push(
                kernel
                    .iter()
                    .enumerate()
                    .map(|(tap, weight)| {
                        let source =
                            border_index(x as isize + tap as isize - radius, width, replicate);
                        weight * source.map_or(background, |sx| sample(sx, y))
                    })
                    .sum::<f64>(),
            );
        }
    }
    let mut output = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            output.push(
                kernel
                    .iter()
                    .enumerate()
                    .map(|(tap, weight)| {
                        let source =
                            border_index(y as isize + tap as isize - radius, height, replicate);
                        weight * source.map_or(background, |sy| horizontal[sy * width + x])
                    })
                    .sum::<f64>(),
            );
        }
    }
    output
}

fn median_plane(
    width: usize,
    height: usize,
    window: u8,
    border: FilterBorder,
    sample: impl Fn(usize, usize) -> u8,
    mut output: impl FnMut(usize, u8),
) {
    let size = usize::from(window);
    let radius = (size / 2) as isize;
    let mut neighborhood = [0; 225];
    let replicate = matches!(border, FilterBorder::Replicate);
    let background = match border {
        FilterBorder::Constant { sample } => sample,
        _ => 0,
    };
    for y in 0..height {
        for x in 0..width {
            for (i, tap) in neighborhood[..size * size].iter_mut().enumerate() {
                let sx = border_index(x as isize + (i % size) as isize - radius, width, replicate);
                let sy = border_index(y as isize + (i / size) as isize - radius, height, replicate);
                *tap = sx.zip(sy).map_or(background, |(sx, sy)| sample(sx, sy));
            }
            let (_, value, _) = neighborhood[..size * size].select_nth_unstable(size * size / 2);
            output(y * width + x, *value);
        }
    }
}

/// Smooth selected physical channels. The default set excludes physical alpha.
pub fn apply(
    image: &Image,
    filter: Filter,
    channels: Option<&[Channel]>,
    border: FilterBorder,
) -> Result<Image, OperationError> {
    (|| {
        resample::physical_samples(image)?;
        let defaults: Vec<_> = image
            .channels()
            .iter()
            .copied()
            .filter(|c| *c != Channel::Alpha)
            .collect();
        let (_, selected) = image.selection(Some(channels.unwrap_or(&defaults)))?;
        let (width, height) = (image.width() as usize, image.height() as usize);
        let area = width * height;
        let (visits, scratch, gaussian) = match filter {
            Filter::Gaussian { radius, sigma } => {
                let weights = gaussian_weights(radius, sigma)?;
                (
                    2 * weights.len(),
                    16 * area + 8 * weights.len(),
                    Some(weights),
                )
            }
            Filter::Median { width } if (3..=15).contains(&width) && width % 2 == 1 => {
                (usize::from(width).pow(2), 225, None)
            }
            Filter::Median { .. } => return Err(invalid("median width must be odd in 3..15")),
        };
        if area as u64 * selected.len() as u64 * visits as u64 > 64 * MIB as u64
            || scratch + image.samples().len() > 32 * MIB
        {
            return Err(limit(
                "filter exceeds 64 Mi visits or 32 MiB scratch and output",
            ));
        }
        let model = image.model();
        let mut result = image.samples().to_vec();
        for channel in selected {
            let index = |pixel: usize| pixel * model.channels() + channel;
            let sample = |x, y| image.samples()[index(y * width + x)];
            match (&gaussian, filter) {
                (Some(weights), _) => {
                    let filtered = gaussian_plane(width, height, weights, border, |x, y| {
                        f64::from(sample(x, y))
                    });
                    for (i, value) in filtered.into_iter().enumerate() {
                        result[index(i)] = resample::round_sample(value);
                    }
                }
                (None, Filter::Median { width: window }) => {
                    median_plane(width, height, window, border, sample, |i, value| {
                        result[index(i)] = value
                    });
                }
                _ => unreachable!("Gaussian preparation always supplies weights"),
            }
        }
        Image::from_samples(image.width(), image.height(), model, result, vec![], vec![])
    })()
    .map_err(|error: OperationError| error.operation("image.filter"))
}
