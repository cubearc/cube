//! Explicit sRGB measurements in stored row-major order.
use super::{ColorModel, Image, invalid, limit, resample};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_PIXELS: usize = 65_536;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Hsv,
    Hsl,
    LabD50,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    pub target: Target,
    pub width: u32,
    pub height: u32,
    pub values: Vec<[f64; 3]>,
    pub alpha: Option<Vec<u8>>,
}

pub(super) fn decode_srgb(sample: u8) -> f64 {
    let value = f64::from(sample) / 255.;
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

pub(super) fn encode_srgb(value: f64) -> u8 {
    let encoded = if value <= 0.0031308 {
        12.92 * value
    } else {
        1.055 * value.powf(1. / 2.4) - 0.055
    };
    resample::round_sample(255. * encoded)
}

pub(super) fn project(matrix: [[f64; 3]; 3], signal: [f64; 3]) -> [f64; 3] {
    matrix.map(|[r, g, b]| r * signal[0] + g * signal[1] + b * signal[2])
}

fn lab(rgb: [u8; 3]) -> [f64; 3] {
    // W3C CSS Color 4, 2026-10-26 candidate recommendation, sample code.
    const SRGB_TO_D65: [[f64; 3]; 3] = [
        [506752. / 1228815., 87881. / 245763., 12673. / 70218.],
        [87098. / 409605., 175762. / 245763., 12673. / 175545.],
        [7918. / 409605., 87881. / 737289., 1001167. / 1053270.],
    ];
    const BRADFORD: [[f64; 3]; 3] = [
        [
            1.0479297925449969,
            0.022946870601609652,
            -0.05019226628920524,
        ],
        [
            0.02962780877005599,
            0.9904344267538799,
            -0.017073799063418826,
        ],
        [
            -0.009243040646204504,
            0.015055191490298152,
            0.7518742814281371,
        ],
    ];
    const WHITE: [f64; 3] = [0.3457 / 0.3585, 1., (1. - 0.3457 - 0.3585) / 0.3585];
    let xyz = project(BRADFORD, project(SRGB_TO_D65, rgb.map(decode_srgb)));
    let [x, y, z] = std::array::from_fn(|index| {
        let t = xyz[index] / WHITE[index];
        if t > 216. / 24389. {
            t.cbrt()
        } else {
            (t * (24389. / 27.) + 16.) / 116.
        }
    });
    [116. * y - 16., 500. * (x - y), 200. * (y - z)]
}

fn cylindrical(rgb: [u8; 3], target: Target) -> [f64; 3] {
    let [r, g, b] = rgb.map(i32::from);
    let high = r.max(g).max(b);
    let low = r.min(g).min(b);
    let delta = f64::from(high - low);
    let hue = if delta == 0. {
        0.
    } else {
        let sector = if high == r {
            f64::from(g - b) / delta
        } else if high == g {
            2. + f64::from(b - r) / delta
        } else {
            4. + f64::from(r - g) / delta
        };
        (sector * 60.).rem_euclid(360.)
    };
    let (denominator, value) = match target {
        Target::Hsv => (high, f64::from(high) / 255.),
        _ => (255 - (high + low - 255).abs(), f64::from(high + low) / 510.),
    };
    let saturation = if delta == 0. {
        0.
    } else {
        delta / f64::from(denominator)
    };
    [hue, saturation, value]
}

/// Measure explicit sRGB RGB/RGBA samples. Alpha remains in a separate u8 plane.
pub fn convert(image: &Image, target: Target) -> Result<Report, OperationError> {
    (|| {
        resample::physical_samples(image)?;
        if !matches!(image.model(), ColorModel::Rgb | ColorModel::Rgba) {
            return Err(invalid("sRGB conversion requires RGB or RGBA samples"));
        }
        let channels = image.model().channels();
        let count = image.samples().len() / channels;
        if count > MAX_PIXELS {
            return Err(limit("color conversion exceeds 65536 pixel records"));
        }
        let mut values = Vec::with_capacity(count);
        let mut alpha = (channels == 4).then(|| Vec::with_capacity(count));
        for pixel in image.samples().chunks_exact(channels) {
            let rgb = [pixel[0], pixel[1], pixel[2]];
            values.push(match target {
                Target::LabD50 => lab(rgb),
                _ => cylindrical(rgb, target),
            });
            if let Some(plane) = &mut alpha {
                plane.push(pixel[3]);
            }
        }
        Ok(Report {
            target,
            width: image.width(),
            height: image.height(),
            values,
            alpha,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.color.convert"))
}
