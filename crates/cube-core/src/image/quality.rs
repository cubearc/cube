//! Exact sample error and the bounded Wang 2004 single-scale SSIM profile.
use super::{
    Channel, Image, Region,
    filters::gaussian_weights,
    invalid, limit,
    plane::{MIB, Plane},
};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Mse,
    Psnr,
    Ssim,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Report {
    pub metric: Metric,
    pub region: Region,
    pub channel: Channel,
    pub sum_squared_difference: u128,
    pub count: usize,
    pub mse: f64,
    pub value: Option<f64>,
    pub perfect: bool,
    pub windows: Option<usize>,
}

fn ssim_window(a: Plane<'_>, b: Plane<'_>, x: usize, y: usize, weights: &[f64]) -> f64 {
    let (mut sx, mut sy, mut xx, mut yy, mut xy) = (0., 0., 0., 0., 0.);
    for (v, &wy) in weights.iter().enumerate() {
        for (u, &wx) in weights.iter().enumerate() {
            let p = f64::from(a.sample(x + u, y + v));
            let q = f64::from(b.sample(x + u, y + v));
            let w = wx * wy;
            sx += w * p;
            sy += w * q;
            xx += w * p * p;
            yy += w * q * q;
            xy += w * p * q;
        }
    }
    let variance = |second, mean: f64| {
        let v = second - mean * mean;
        if (-1e-9..0.).contains(&v) { 0. } else { v }
    };
    let (vx, vy, cov) = (variance(xx, sx), variance(yy, sy), xy - sx * sy);
    let c1 = (0.01_f64 * 255.).powi(2);
    let c2 = (0.03_f64 * 255.).powi(2);
    ((2. * sx * sy + c1) * (2. * cov + c2)) / ((sx * sx + sy * sy + c1) * (vx + vy + c2))
}

/// Compare one physical channel at equal coordinates. SSIM uses valid 11×11
/// windows and population moments. Exact error authority always accompanies
/// the derived metric. Infinite PSNR is reported as a perfect result with null.
pub fn measure(
    a: &Image,
    b: &Image,
    region: Region,
    channel: Channel,
    metric: Metric,
) -> Result<Report, OperationError> {
    (|| {
        if (a.width(), a.height()) != (b.width(), b.height()) {
            return Err(invalid("quality inputs must have the same dimensions"));
        }
        let p = Plane::new(a, region, channel, MIB)?;
        let q = Plane::new(b, region, channel, MIB)?;
        let window_shape = if matches!(metric, Metric::Ssim) {
            if channel != Channel::Gray || p.width < 11 || p.height < 11 {
                return Err(invalid(
                    "SSIM requires explicit grayscale regions at least 11 by 11",
                ));
            }
            let shape = (p.width - 10, p.height - 10);
            if shape.0 * shape.1 * 121 + p.len() > 64 * MIB {
                return Err(limit("SSIM exceeds 64 Mi kernel and error visits"));
            }
            Some(shape)
        } else {
            None
        };
        let mut sum = 0u128;
        for y in 0..p.height {
            for x in 0..p.width {
                sum += u128::from(
                    i32::from(p.sample(x, y))
                        .abs_diff(i32::from(q.sample(x, y)))
                        .pow(2),
                );
            }
        }
        let count = p.len();
        let mse = sum as f64 / count as f64;
        let perfect = sum == 0;
        let value = match (metric, window_shape) {
            (Metric::Mse, _) => Some(mse),
            (Metric::Psnr, _) => (!perfect).then(|| 10. * (65_025. / mse).log10()),
            (Metric::Ssim, Some((width, height))) => {
                let weights = gaussian_weights(5, 1.5)?;
                let mut score = 0.;
                for y in 0..height {
                    for x in 0..width {
                        score += if perfect {
                            1.
                        } else {
                            ssim_window(p, q, x, y, &weights)
                        };
                    }
                }
                Some(score / (width * height) as f64)
            }
            _ => unreachable!("SSIM preparation always supplies a valid window grid"),
        };
        Ok(Report {
            metric,
            region,
            channel,
            sum_squared_difference: sum,
            count,
            mse,
            value,
            perfect,
            windows: window_shape.map(|(w, h)| w * h),
        })
    })()
    .map_err(|error: OperationError| error.operation("image.quality.measure"))
}
