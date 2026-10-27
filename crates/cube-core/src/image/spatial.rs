//! Exact lagged moments of one physical channel. No FFT or inferred periodicity.
use super::{Channel, Image, Region, invalid, limit};
use crate::errors::OperationError;
use serde::{Deserialize, Serialize};
pub const MAX_PIXELS: usize = 1024 * 1024;
pub const MAX_LAGS: usize = 256;
pub const MAX_WORK: usize = 16 * 1024 * 1024;
pub const MAX_MEMORY: usize = 16 * 1024 * 1024;
pub const MAX_REPORT: usize = 1024 * 1024;
const OP: &str = "image.spatial.autocorrelation";
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lag {
    pub dx: i32,
    pub dy: i32,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Limits {
    pub max_work: usize,
    pub max_memory_bytes: usize,
    pub max_report_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_work: MAX_WORK,
            max_memory_bytes: MAX_MEMORY,
            max_report_bytes: MAX_REPORT,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Measurement {
    pub lag: Lag,
    pub source_overlap: Region,
    pub target_overlap: Region,
    pub count: u64,
    pub sum_x: u64,
    pub sum_y: u64,
    pub sum_x_squared: u64,
    pub sum_y_squared: u64,
    pub sum_xy: u64,
    pub squared_difference: u64,
    pub population_covariance: f64,
    pub correlation: Option<f64>,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub region: Region,
    pub channel: Channel,
    pub representation: &'static str,
    pub source_format: &'static str,
    pub measurements: Vec<Measurement>,
    pub work: usize,
    pub memory_bytes_upper_bound: usize,
    pub report_bytes_upper_bound: usize,
}
/// Parse <=256 ordered signed lags with unique object keys; duplicates are retained.
pub fn parse_lags(s: &str) -> Result<Vec<Lag>, OperationError> {
    let v = crate::pipeline::json::parse_with_limits(s, 64 * 1024, 3, 1024)
        .map_err(|e| e.error.operation(OP))?;
    let l: Vec<Lag> = serde_json::from_value(v).map_err(|e| {
        invalid("lags must be [{dx,dy},...] signed i32 offsets")
            .operation(OP)
            .caused_by(e)
    })?;
    if l.len() > MAX_LAGS {
        return Err(limit("at most 256 lags").operation(OP));
    }
    Ok(l)
}
fn overlap(region: Region, lag: Lag) -> Result<(Region, Region), OperationError> {
    let dx = i64::from(lag.dx);
    let dy = i64::from(lag.dy);
    let width = i64::from(region.width) - dx.abs();
    let height = i64::from(region.height) - dy.abs();
    if width <= 0 || height <= 0 {
        return Err(invalid("lag has zero overlap inside the selected region")
            .operation(OP)
            .parameter("lags"));
    }
    let a = Region {
        x: region.x + (-dx).max(0) as u32,
        y: region.y + (-dy).max(0) as u32,
        width: width as u32,
        height: height as u32,
    };
    let b = Region {
        x: (i64::from(a.x) + dx) as u32,
        y: (i64::from(a.y) + dy) as u32,
        ..a
    };
    Ok((a, b))
}
impl Image {
    /// Pair (x,y) with (x+dx,y+dy), both inside the region; no wrapping/padding.
    /// Covariance uses population N; correlation uses exact centered moments, null
    /// for zero variance. Exact integer moments are authoritative; derived f64
    /// scores may vary by 1e-12 across supported platforms. Palette color expansion
    /// must be explicit; `index` measures physical indices rather than colors.
    pub fn autocorrelation(
        &self,
        region: Region,
        channel: Channel,
        lags: &[Lag],
        limits: Limits,
    ) -> Result<Report, OperationError> {
        if limits.max_work > MAX_WORK
            || limits.max_memory_bytes > MAX_MEMORY
            || limits.max_report_bytes > MAX_REPORT
        {
            return Err(invalid("spatial limits may only lower hard ceilings")
                .operation(OP)
                .parameter("limits"));
        }
        let region = region.validate(self).map_err(|e| e.operation(OP))?;
        if region.width as usize * region.height as usize > MAX_PIXELS || lags.len() > MAX_LAGS {
            return Err(limit("spatial region exceeds 1 Mi pixels or 256 lags").operation(OP));
        }
        let (_, indices) = self
            .selection(Some(&[channel]))
            .map_err(|e| e.operation(OP))?;
        let index = indices[0];
        let report = 4096 + lags.len() * 1024;
        let memory = 8192 + lags.len() * 256 + 2 * report;
        if memory > limits.max_memory_bytes || report > limits.max_report_bytes {
            return Err(limit("conservative spatial scratch/report exceeds budget")
                .operation(OP)
                .parameter("limits"));
        }
        let mut work = 0usize;
        for &lag in lags {
            let (a, _) = overlap(region, lag)?;
            work = work
                .checked_add(a.width as usize * a.height as usize)
                .filter(|v| *v <= limits.max_work)
                .ok_or_else(|| {
                    limit("aggregate overlap-pixel visits exceed budget")
                        .operation(OP)
                        .parameter("max_work")
                })?;
        }
        let mut measurements = Vec::with_capacity(lags.len());
        let channels = self.model.channels();
        for &lag in lags {
            let (a, b) = overlap(region, lag)?;
            let (mut sx, mut sy, mut xx, mut yy, mut xy, mut diff) =
                (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
            // At most 1 Mi pairs of 8-bit samples per lag. Each sum <=1 Mi*65025,
            // proven to fit u64 by the validated model and region bounds.
            for y in 0..a.height {
                for x in 0..a.width {
                    let u = u64::from(
                        self.samples[((a.y + y) as usize * self.width as usize
                            + (a.x + x) as usize)
                            * channels
                            + index],
                    );
                    let v = u64::from(
                        self.samples[((b.y + y) as usize * self.width as usize
                            + (b.x + x) as usize)
                            * channels
                            + index],
                    );
                    sx += u;
                    sy += v;
                    xx += u * u;
                    yy += v * v;
                    xy += u * v;
                    diff += u.abs_diff(v).pow(2);
                }
            }
            let count = u64::from(a.width) * u64::from(a.height);
            let n = u128::from(count);
            let vx = n * u128::from(xx) - u128::from(sx).pow(2);
            let vy = n * u128::from(yy) - u128::from(sy).pow(2);
            let centered = (n * u128::from(xy)) as i128 - (u128::from(sx) * u128::from(sy)) as i128;
            let covariance = centered as f64 / (count as f64).powi(2);
            let correlation = if vx == 0 || vy == 0 {
                None
            } else {
                Some((centered as f64 / ((vx * vy) as f64).sqrt()).clamp(-1.0, 1.0))
            };
            measurements.push(Measurement {
                lag,
                source_overlap: a,
                target_overlap: b,
                count,
                sum_x: sx,
                sum_y: sy,
                sum_x_squared: xx,
                sum_y_squared: yy,
                sum_xy: xy,
                squared_difference: diff,
                population_covariance: covariance,
                correlation,
            });
        }
        Ok(Report {
            region,
            channel,
            representation: self.representation,
            source_format: self.source_format,
            measurements,
            work,
            memory_bytes_upper_bound: memory,
            report_bytes_upper_bound: report,
        })
    }
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "region",
        description: "Required x,y,width,height rectangle.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "channel",
        description: "One explicit physical channel; expand palette to interpret indexed colors.",
        required: true,
        default: None,
        constraints: ParameterType::Text {
            choices: &["red", "green", "blue", "gray", "alpha", "index"],
        },
    },
    Parameter {
        name: "lags",
        description: "Ordered JSON array of signed i32 dx/dy; overlap only, no wrapping.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "max_work",
        description: "Charged validation, index and execution work; exhaustion is explicit.",
        required: false,
        default: Some(ParameterValue::Unsigned(16777216)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 16777216,
        },
    },
    Parameter {
        name: "max_memory_bytes",
        description: "Conservative aggregate scratch and retained-report allocation bound.",
        required: false,
        default: Some(ParameterValue::Unsigned(16777216)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 16777216,
        },
    },
    Parameter {
        name: "max_report_bytes",
        description: "Conservative complete compact JSON report bound.",
        required: false,
        default: Some(ParameterValue::Unsigned(1048576)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: 1048576,
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "image.spatial.autocorrelation",
    category: "image",
    description: "Ordered signed lag overlaps on one physical channel; exact moments and population covariance, null zero-variance correlation; 1 Mi selected pixels, 256 lags.",
    input_type: "image_and_lags",
    output_type: "spatial_autocorrelation",
    command: "image autocorrelation",
    example: "cube image autocorrelation --input stripes.png --region 0,0,4,1 --channel gray --lags '[{\"dx\":1,\"dy\":0}]'",
    parameters: PARAMETERS,
    deterministic: true,
    reversible: None,
}];
