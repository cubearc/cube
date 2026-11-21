//! Signed discrete responses and one bounded Canny profile.
use super::{
    Channel, Image, Region,
    binary_grid::{Connectivity, neighbors},
    filters::{FilterBorder, gaussian_plane, gaussian_weights},
    invalid, limit,
    plane::{MIB, Plane, border_index},
};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKernel {
    Sobel,
    Prewitt,
    Laplacian,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeBorder {
    #[default]
    Replicate,
    ConstantZero,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Responses {
    pub region: Region,
    pub channel: Channel,
    pub kernel: EdgeKernel,
    pub border: EdgeBorder,
    pub gx: Option<Vec<i32>>,
    pub gy: Option<Vec<i32>>,
    pub laplacian: Option<Vec<i32>>,
    pub magnitude: Option<Vec<f64>>,
}

fn gradient(point: impl Fn(isize, isize) -> f64, middle: f64) -> [f64; 2] {
    let [nw, n, ne, w, e, sw, s, se] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ]
    .map(|(x, y)| point(x, y));
    [
        (ne - nw) + middle * (e - w) + (se - sw),
        (sw - nw) + middle * (s - n) + (se - ne),
    ]
}

fn at<T>(
    x: usize,
    y: usize,
    w: usize,
    h: usize,
    replicate: bool,
    zero: T,
    read: impl Fn(usize, usize) -> T,
) -> impl Fn(isize, isize) -> T
where
    T: Copy,
{
    move |dx, dy| {
        border_index(x as isize + dx, w, replicate)
            .zip(border_index(y as isize + dy, h, replicate))
            .map_or(zero, |(sx, sy)| read(sx, sy))
    }
}

/// Return signed correlation responses in physical row-major order without
/// clipping or thresholding. Horizontal X and downward Y follow the image origin.
pub fn responses(
    image: &Image,
    region: Region,
    channel: Channel,
    kernel: EdgeKernel,
    border: EdgeBorder,
    magnitude: bool,
) -> Result<Responses, OperationError> {
    (|| {
        let plane = Plane::new(image, region, channel, MIB)?;
        // The combined budget includes exact JSON output and allocated arrays.
        let lap = matches!(kernel, EdgeKernel::Laplacian);
        if lap && magnitude {
            return Err(invalid("L2 magnitude requires Sobel or Prewitt responses"));
        }
        let count = plane.len();
        let bytes_per_pixel = if lap {
            10
        } else if magnitude {
            64
        } else {
            20
        };
        if count > (32 * MIB - 1024) / bytes_per_pixel {
            return Err(limit("edge responses exceed 32 MiB scratch and report"));
        }
        let mut gx = (!lap).then(|| Vec::with_capacity(count));
        let mut gy = (!lap).then(|| Vec::with_capacity(count));
        let mut laplacian = lap.then(|| Vec::with_capacity(count));
        let mut magnitudes = magnitude.then(|| Vec::with_capacity(count));
        let read = |x, y| f64::from(plane.sample(x, y));
        let replicate = matches!(border, EdgeBorder::Replicate);
        for y in 0..plane.height {
            for x in 0..plane.width {
                let point = at(x, y, plane.width, plane.height, replicate, 0., read);
                if let Some(values) = &mut laplacian {
                    let center = point(0, 0);
                    values.push(
                        (point(0, -1) + point(-1, 0) + point(1, 0) + point(0, 1) - 4. * center)
                            as i32,
                    );
                } else if let (Some(gx), Some(gy)) = (&mut gx, &mut gy) {
                    let [a, b] = gradient(
                        point,
                        if matches!(kernel, EdgeKernel::Sobel) {
                            2.
                        } else {
                            1.
                        },
                    );
                    gx.push(a as i32);
                    gy.push(b as i32);
                    if let Some(values) = &mut magnitudes {
                        values.push(a.hypot(b));
                    }
                }
            }
        }
        Ok(Responses {
            region,
            channel,
            kernel,
            border,
            gx,
            gy,
            laplacian,
            magnitude: magnitudes,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.edge.response"))
}

fn default_sigma() -> f64 {
    1.
}
fn default_radius() -> u8 {
    2
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Canny {
    #[serde(default = "default_radius")]
    #[schemars(range(min = 1, max = 15))]
    pub radius: u8,
    #[serde(default = "default_sigma")]
    pub sigma: f64,
    pub low: f64,
    pub high: f64,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CannyReport {
    pub region: Region,
    pub channel: Channel,
    pub settings: Canny,
    #[serde(skip)]
    pub mask: Vec<u8>,
}

fn sector(gx: f64, gy: f64) -> u8 {
    let angle = gy.atan2(gx).to_degrees().rem_euclid(180.);
    ((angle + 22.5) / 45.).floor() as u8 % 4
}

fn suppress(
    magnitudes: &[f64],
    directions: &[u8],
    w: usize,
    h: usize,
    settings: Canny,
) -> (Vec<u8>, Vec<u32>) {
    let mut states = Vec::with_capacity(magnitudes.len());
    let mut strong = Vec::new();
    for (i, &current) in magnitudes.iter().enumerate() {
        let point = at(i % w, i / w, w, h, false, 0., |x, y| magnitudes[y * w + x]);
        let (dx, dy) = [(1, 0), (1, 1), (0, 1), (-1, 1)][directions[i] as usize];
        let retained =
            current >= settings.low && current > point(-dx, -dy) && current >= point(dx, dy);
        let state = if !retained {
            0
        } else if current >= settings.high {
            strong.push(i as u32);
            255
        } else {
            1
        };
        states.push(state);
    }
    (states, strong)
}

/// Use unnormalized Sobel L2 units after replicate-border Gaussian smoothing.
/// At half-sector boundaries choose the increasing sector. Retain strict
/// preceding and non-strict following maxima, then link via eight neighbors.
pub fn canny(
    image: &Image,
    region: Region,
    channel: Channel,
    settings: Canny,
) -> Result<CannyReport, OperationError> {
    (|| {
        if channel != Channel::Gray {
            return Err(invalid("Canny requires an explicit physical gray channel"));
        }
        if !settings.low.is_finite()
            || !settings.high.is_finite()
            || settings.low < 0.
            || settings.low > settings.high
        {
            return Err(invalid(
                "Canny thresholds must satisfy finite 0 <= low <= high",
            ));
        }
        let plane = Plane::new(image, region, channel, MIB)?;
        let weights = gaussian_weights(settings.radius, settings.sigma)?;
        if plane.len() > 64 * MIB / (2 * weights.len() + 19) {
            return Err(limit("Canny exceeds 64 Mi taps and neighbor visits"));
        }
        let (w, h) = (plane.width, plane.height);
        let smooth = gaussian_plane(w, h, &weights, FilterBorder::Replicate, |x, y| {
            f64::from(plane.sample(x, y))
        });
        let mut magnitudes = Vec::with_capacity(smooth.len());
        let mut directions = Vec::with_capacity(smooth.len());
        for y in 0..h {
            for x in 0..w {
                let [gx, gy] = gradient(at(x, y, w, h, true, 0., |x, y| smooth[y * w + x]), 2.);
                magnitudes.push(gx.hypot(gy));
                directions.push(sector(gx, gy));
            }
        }
        drop(smooth);
        let (mut mask, mut strong) = suppress(&magnitudes, &directions, w, h, settings);
        let mut cursor = 0;
        while let Some(&p) = strong.get(cursor) {
            cursor += 1;
            for q in neighbors(p as usize, w, h, Connectivity::Eight) {
                if mask[q] == 1 {
                    mask[q] = 255;
                    strong.push(q as u32);
                }
            }
        }
        for sample in &mut mask {
            if *sample == 1 {
                *sample = 0;
            }
        }
        Ok(CannyReport {
            region,
            channel,
            settings,
            mask,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.edge.canny"))
}
