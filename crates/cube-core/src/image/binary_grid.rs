//! Exact thresholds and traversal of bounded 0/255 planes.
use super::{
    Channel, Image, Region, invalid, limit,
    plane::{MIB, Plane},
    sample_count,
};
use crate::errors::OperationError;
use num_bigint::BigUint;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub(super) fn mask_shape(
    width: u32,
    height: u32,
    data: &[u8],
    max_pixels: usize,
) -> Result<(usize, usize), OperationError> {
    let count = sample_count(width, height, 1)?;
    if count > max_pixels {
        return Err(limit("binary mask exceeds its pixel budget"));
    }
    if data.len() != count || data.iter().any(|sample| !matches!(sample, 0 | 255)) {
        return Err(invalid(
            "provide exactly width by height binary samples (0 or 255)",
        ));
    }
    Ok((width as usize, height as usize))
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Threshold {
    pub region: Region,
    pub channel: Channel,
    pub threshold: u8,
    pub degenerate: bool,
    #[serde(skip)]
    pub mask: Vec<u8>,
}

fn otsu_threshold(bins: &[u64; 256], count: u64) -> (u8, bool) {
    let total: u64 = bins.iter().enumerate().map(|(i, n)| i as u64 * n).sum();
    let (mut amount, mut sum) = (0, 0);
    let mut best = None;
    let mut selected = 0;
    for (threshold, &size) in bins[..255].iter().enumerate() {
        amount += size;
        sum += size * threshold as u64;
        if amount == 0 || amount == count {
            continue;
        }
        // 1 Mi pixels, <=255 intensity. These products stay below 2^138.
        let margin = BigUint::from((count * sum).abs_diff(amount * total));
        let numerator = &margin * &margin;
        let denominator = BigUint::from(amount * (count - amount));
        if best
            .as_ref()
            .is_none_or(|(n, d): &(BigUint, BigUint)| &numerator * d > n * &denominator)
        {
            best = Some((numerator, denominator));
            selected = threshold as u8;
        }
    }
    (selected, best.is_none())
}

/// Select the smallest threshold with maximal between-class variance.
/// The mask tests sample > threshold even when all input samples are equal.
pub fn otsu(image: &Image, region: Region, channel: Channel) -> Result<Threshold, OperationError> {
    (|| {
        let plane = Plane::new(image, region, channel, MIB)?;
        let samples =
            || (0..plane.height).flat_map(|y| (0..plane.width).map(move |x| plane.sample(x, y)));
        let mut bins = [0; 256];
        for sample in samples() {
            bins[sample as usize] += 1;
        }
        let (threshold, degenerate) = otsu_threshold(&bins, plane.len() as u64);
        let mask = samples()
            .map(|sample| if sample > threshold { 255 } else { 0 })
            .collect();
        Ok(Threshold {
            region,
            channel,
            threshold,
            degenerate,
            mask,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.mask.otsu"))
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Morphology {
    Erode,
    Dilate,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ElementShape {
    Rectangle,
    Cross,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Element {
    #[schemars(range(min = 1, max = 31))]
    pub width: u8,
    #[schemars(range(min = 1, max = 31))]
    pub height: u8,
    pub shape: ElementShape,
}

impl Default for Element {
    fn default() -> Self {
        Self {
            width: 3,
            height: 3,
            shape: ElementShape::Rectangle,
        }
    }
}

impl Element {
    fn offsets(self) -> Result<Vec<(isize, isize)>, OperationError> {
        if [self.width, self.height]
            .iter()
            .any(|value| *value > 31 || value % 2 == 0)
        {
            return Err(invalid("element sides must be odd in 1..31"));
        }
        let (x, y) = (isize::from(self.width / 2), isize::from(self.height / 2));
        Ok((-y..=y)
            .flat_map(|dy| {
                (-x..=x).filter_map(move |dx| {
                    (matches!(self.shape, ElementShape::Rectangle) || dx == 0 || dy == 0)
                        .then_some((dx, dy))
                })
            })
            .collect())
    }
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct MorphologyReport {
    pub width: u32,
    pub height: u32,
    pub operation: Morphology,
    pub element: Element,
    pub iterations: u8,
    #[serde(skip)]
    pub mask: Vec<u8>,
}

/// Apply 1..16 explicit erosions or dilations. Outside the plane is background.
pub fn morphology(
    width: u32,
    height: u32,
    data: &[u8],
    operation: Morphology,
    element: Element,
    iterations: u8,
) -> Result<MorphologyReport, OperationError> {
    (|| {
        let (w, h) = mask_shape(width, height, data, MIB)?;
        let offsets = element.offsets()?;
        if !(1..=16).contains(&iterations) {
            return Err(invalid("morphology iterations must be 1..16"));
        }
        if data.len() as u64 * offsets.len() as u64 * u64::from(iterations) > 64 * MIB as u64 {
            return Err(limit("morphology exceeds 64 Mi neighbor visits"));
        }
        let mut current = data.to_vec();
        let mut next = vec![0; data.len()];
        let erode = matches!(operation, Morphology::Erode);
        for _ in 0..iterations {
            for (p, target) in next.iter_mut().enumerate() {
                let x = (p % w) as isize;
                let y = (p / w) as isize;
                let found = offsets.iter().any(|&(dx, dy)| {
                    let sx = x + dx;
                    let sy = y + dy;
                    let foreground = sx >= 0
                        && sy >= 0
                        && (sx as usize) < w
                        && (sy as usize) < h
                        && current[sy as usize * w + sx as usize] == 255;
                    foreground != erode
                });
                *target = if found != erode { 255 } else { 0 };
            }
            std::mem::swap(&mut current, &mut next);
        }
        Ok(MorphologyReport {
            width,
            height,
            operation,
            element,
            iterations,
            mask: current,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.mask.morphology"))
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Connectivity {
    Four,
    Eight,
}

pub(super) fn neighbors(
    p: usize,
    width: usize,
    height: usize,
    connectivity: Connectivity,
) -> impl Iterator<Item = usize> {
    let (x, y) = ((p % width) as isize, (p / width) as isize);
    let diagonal = matches!(connectivity, Connectivity::Eight);
    [-1, 0, 1].into_iter().flat_map(move |dy| {
        [-1, 0, 1].into_iter().filter_map(move |dx| {
            let (nx, ny) = (x + dx, y + dy);
            ((dx != 0 || dy != 0)
                && (diagonal || dx == 0 || dy == 0)
                && nx >= 0
                && ny >= 0
                && (nx as usize) < width
                && (ny as usize) < height)
                .then(|| ny as usize * width + nx as usize)
        })
    })
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Component {
    pub label: u32,
    pub area: usize,
    pub bounds: Region,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Components {
    pub width: u32,
    pub height: u32,
    pub connectivity: Connectivity,
    pub labels: Vec<u32>,
    pub components: Vec<Component>,
}

/// Label foreground components in first-pixel row-major order.
pub fn components(
    width: u32,
    height: u32,
    data: &[u8],
    connectivity: Connectivity,
) -> Result<Components, OperationError> {
    (|| {
        let (w, h) = mask_shape(width, height, data, MIB)?;
        let mut labels = vec![0; data.len()];
        let mut components = Vec::new();
        let mut queue = Vec::<u32>::new();
        for seed in 0..data.len() {
            if data[seed] == 0 || labels[seed] != 0 {
                continue;
            }
            if components.len() == 65_536 {
                return Err(limit("mask contains more than 65536 components"));
            }
            let label = components.len() as u32 + 1;
            labels[seed] = label;
            queue.clear();
            queue.push(seed as u32);
            let mut cursor = 0;
            let (mut left, mut top, mut right, mut bottom) = (w, h, 0, 0);
            while let Some(&position) = queue.get(cursor) {
                cursor += 1;
                let position = position as usize;
                let (x, y) = (position % w, position / w);
                (left, top, right, bottom) = (left.min(x), top.min(y), right.max(x), bottom.max(y));
                for next in neighbors(position, w, h, connectivity) {
                    if data[next] == 255 && labels[next] == 0 {
                        labels[next] = label;
                        queue.push(next as u32);
                    }
                }
            }
            let bounds = Region {
                x: left as u32,
                y: top as u32,
                width: (right - left + 1) as u32,
                height: (bottom - top + 1) as u32,
            };
            components.push(Component {
                label,
                area: cursor,
                bounds,
            });
        }
        Ok(Components {
            width,
            height,
            connectivity,
            labels,
            components,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.mask.components"))
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Flood {
    pub width: u32,
    pub height: u32,
    pub connectivity: Connectivity,
    pub seed_labels: Vec<u32>,
    #[serde(skip)]
    pub mask: Vec<u8>,
}

/// Return the union of each foreground seed's component. Background seeds
/// contribute label zero and never select background pixels.
pub fn flood(
    width: u32,
    height: u32,
    data: &[u8],
    connectivity: Connectivity,
    seeds: &[[u32; 2]],
) -> Result<Flood, OperationError> {
    (|| {
        if seeds.len() > 65_536 {
            return Err(limit("flood supports at most 65536 seeds"));
        }
        if seeds.iter().any(|&[x, y]| x >= width || y >= height) {
            return Err(invalid("seed pixel is outside the mask"));
        }
        let result = components(width, height, data, connectivity)?;
        let seed_labels: Vec<_> = seeds
            .iter()
            .map(|&[x, y]| result.labels[y as usize * width as usize + x as usize])
            .collect();
        let mut selected = vec![false; result.components.len() + 1];
        for &label in &seed_labels {
            selected[label as usize] = label != 0;
        }
        let mask = result
            .labels
            .into_iter()
            .map(|label| if selected[label as usize] { 255 } else { 0 })
            .collect();
        Ok(Flood {
            width,
            height,
            connectivity,
            seed_labels,
            mask,
        })
    })()
    .map_err(|error: OperationError| error.operation("image.mask.flood"))
}
