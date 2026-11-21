//! Bounded separable sampling with explicit pixel centers and borders.
use super::{ColorModel, Image, invalid, limit, sample_count};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_TAPS: usize = 64 << 20;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Kernel {
    Nearest,
    Bilinear,
    Bicubic,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WarpKernel {
    #[default]
    Nearest,
    Bilinear,
}

/// An inverse transform maps destination integer centers to source coordinates.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Transform {
    Affine { matrix: [[f64; 3]; 2] },
    Projective { matrix: [[f64; 3]; 3] },
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Border {
    Clamp,
    Constant {
        /// Omit or supply an empty vector for zero on every physical channel.
        #[serde(default)]
        #[schemars(length(max = 4))]
        samples: Vec<u8>,
    },
}

impl Default for Border {
    fn default() -> Self {
        Self::Constant { samples: vec![] }
    }
}

impl Transform {
    fn matrix(self) -> Result<[[f64; 3]; 3], OperationError> {
        let matrix = match self {
            Self::Affine { matrix: [x, y] } => [x, y, [0., 0., 1.]],
            Self::Projective { matrix } => matrix,
        };
        if matrix.iter().flatten().any(|value| !value.is_finite()) {
            return Err(invalid("inverse matrix must be finite").parameter("transform"));
        }
        let normalized = matrix.map(|row| {
            let scale = row.into_iter().map(f64::abs).fold(0., f64::max);
            if scale == 0. {
                row
            } else {
                row.map(|value| value / scale)
            }
        });
        let [[a, b, c], [d, e, f], [g, h, i]] = normalized;
        let determinant = a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g);
        if determinant == 0. {
            return Err(
                invalid("inverse matrix is singular at f64 precision").parameter("transform")
            );
        }
        Ok(matrix)
    }
}

fn point(matrix: [[f64; 3]; 3], x: u32, y: u32) -> [f64; 2] {
    let [u, v, w] = matrix.map(|[a, b, c]| a * f64::from(x) + b * f64::from(y) + c);
    if w == 0. || !w.is_finite() {
        return [f64::NAN; 2];
    }
    [u / w, v / w]
}

pub(super) fn physical_samples(image: &Image) -> Result<(), OperationError> {
    if image.model() == ColorModel::Indexed || !image.transparency().is_empty() {
        return Err(invalid(
            "expand indices and tRNS to physical channels explicitly first",
        ));
    }
    Ok(())
}

fn plan(image: &Image, width: u32, height: u32, kernel: Kernel) -> Result<usize, OperationError> {
    physical_samples(image)?;
    let count = sample_count(width, height, image.model().channels())?;
    let taps = kernel.axis_size().pow(2);
    if count > MAX_TAPS / taps {
        return Err(limit("resampling exceeds 64 Mi weighted channel taps"));
    }
    Ok(count)
}

impl Kernel {
    fn axis_size(self) -> usize {
        match self {
            Self::Nearest => 1,
            Self::Bilinear => 2,
            Self::Bicubic => 4,
        }
    }

    fn axis(self, value: f64) -> [(i32, f64); 4] {
        let index = value.floor() as i32;
        let fraction = value - f64::from(index);
        match self {
            Self::Nearest => [
                ((value + 0.5).floor() as i32, 1.),
                (0, 0.),
                (0, 0.),
                (0, 0.),
            ],
            Self::Bilinear => [
                (index, 1. - fraction),
                (index + 1, fraction),
                (0, 0.),
                (0, 0.),
            ],
            Self::Bicubic => [-1, 0, 1, 2].map(|n| (index + n, keys(fraction - f64::from(n)))),
        }
    }
}

fn keys(distance: f64) -> f64 {
    let x = distance.abs();
    if x <= 1. {
        (1.5 * x - 2.5) * x * x + 1.
    } else if x < 2. {
        ((-0.5 * x + 2.5) * x - 4.) * x + 2.
    } else {
        0.
    }
}

pub(super) fn round_sample(value: f64) -> u8 {
    value.clamp(0., 255.).round() as u8
}

struct Sampler<'a> {
    image: &'a Image,
    kernel: Kernel,
    border: &'a Border,
}

impl Sampler<'_> {
    fn sample(&self, x: i32, y: i32, channel: usize) -> u8 {
        let (width, height) = (self.image.width() as i32, self.image.height() as i32);
        if let Border::Constant { samples } = self.border
            && (x < 0 || y < 0 || x >= width || y >= height)
        {
            return samples.get(channel).copied().unwrap_or(0);
        }
        let pixel = y.clamp(0, height - 1) * width + x.clamp(0, width - 1);
        self.image.samples()[pixel as usize * self.image.model().channels() + channel]
    }

    fn pixel(&self, [x, y]: [f64; 2], out: &mut Vec<u8>) {
        let x = self
            .kernel
            .axis(x.clamp(-2., f64::from(self.image.width()) + 1.));
        let y = self
            .kernel
            .axis(y.clamp(-2., f64::from(self.image.height()) + 1.));
        let size = self.kernel.axis_size();
        for channel in 0..self.image.model().channels() {
            let sum = y[..size]
                .iter()
                .map(|&(sy, wy)| {
                    x[..size]
                        .iter()
                        .map(|&(sx, wx)| wx * f64::from(self.sample(sx, sy, channel)))
                        .sum::<f64>()
                        * wy
                })
                .sum();
            out.push(round_sample(sum));
        }
    }

    fn image(
        self,
        width: u32,
        height: u32,
        count: usize,
        map: impl Fn(u32, u32) -> [f64; 2],
    ) -> Result<Image, OperationError> {
        let mut out = Vec::with_capacity(count);
        for y in 0..height {
            for x in 0..width {
                self.pixel(map(x, y), &mut out);
            }
        }
        Image::from_samples(width, height, self.image.model(), out, vec![], vec![])
    }
}

/// Resize using half-pixel centers and clamp-to-edge. Physical alpha is sampled
/// independently of color. All weighted taps are charged before allocation.
pub fn resize(
    image: &Image,
    width: u32,
    height: u32,
    kernel: Kernel,
) -> Result<Image, OperationError> {
    (|| {
        let count = plan(image, width, height, kernel)?;
        let center = |i, src, dst| (f64::from(i) + 0.5) * f64::from(src) / f64::from(dst) - 0.5;
        Sampler {
            image,
            kernel,
            border: &Border::Clamp,
        }
        .image(width, height, count, |x, y| {
            [
                center(x, image.width(), width),
                center(y, image.height(), height),
            ]
        })
    })()
    .map_err(|error: OperationError| error.operation("image.resize"))
}

/// Apply an explicit inverse affine or projective warp. The destination begins
/// at integer pixel centers; a pole at any evaluated center rejects the image.
pub fn warp(
    image: &Image,
    width: u32,
    height: u32,
    transform: Transform,
    kernel: WarpKernel,
    border: &Border,
) -> Result<Image, OperationError> {
    (|| {
        let kernel = match kernel {
            WarpKernel::Nearest => Kernel::Nearest,
            WarpKernel::Bilinear => Kernel::Bilinear,
        };
        let count = plan(image, width, height, kernel)?;
        if let Border::Constant { samples } = border
            && !samples.is_empty()
            && samples.len() != image.model().channels()
        {
            return Err(
                invalid("constant border needs one byte per physical channel").parameter("border"),
            );
        }
        let matrix = transform.matrix()?;
        let map = |x, y| point(matrix, x, y);
        for y in 0..height {
            for x in 0..width {
                if map(x, y).into_iter().any(|value| !value.is_finite()) {
                    return Err(invalid(
                        "nonfinite inverse coordinate or zero homogeneous denominator",
                    )
                    .parameter("transform"));
                }
            }
        }
        Sampler {
            image,
            kernel,
            border,
        }
        .image(width, height, count, map)
    })()
    .map_err(|error: OperationError| error.operation("image.warp"))
}
