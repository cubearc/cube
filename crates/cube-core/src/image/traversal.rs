//! Bounded coordinate ordering, independent of samples, channels and bits.
use super::{MAX_DIMENSION, MAX_PIXELS, Region, invalid, limit};
use crate::errors::OperationError;
use serde::{Deserialize, Serialize};

const MAX_EXTENDED_COORDINATES: usize = 1 << 20;

pub const NAMES: &[&str] = &[
    "row",
    "column",
    "diagonal",
    "zigzag",
    "spiral_cw",
    "spiral_ccw",
    "serpentine",
    "hilbert",
    "morton",
];
pub const CORNER_NAMES: &[&str] = &["top_left", "top_right", "bottom_right", "bottom_left"];
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Traversal {
    #[default]
    Row,
    Column,
    Diagonal,
    Zigzag,
    SpiralCw,
    SpiralCcw,
    Serpentine,
    Hilbert,
    Morton,
}
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Corner {
    #[default]
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}
impl Corner {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "top_left" => Ok(Self::TopLeft),
            "top_right" => Ok(Self::TopRight),
            "bottom_right" => Ok(Self::BottomRight),
            "bottom_left" => Ok(Self::BottomLeft),
            _ => Err(invalid(
                "start_corner must be top_left, top_right, bottom_right or bottom_left",
            )
            .parameter("start_corner")),
        }
    }
}
impl Traversal {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "row" => Ok(Self::Row),
            "column" => Ok(Self::Column),
            "diagonal" => Ok(Self::Diagonal),
            "zigzag" => Ok(Self::Zigzag),
            "spiral_cw" => Ok(Self::SpiralCw),
            "spiral_ccw" => Ok(Self::SpiralCcw),
            "serpentine" => Ok(Self::Serpentine),
            "hilbert" => Ok(Self::Hilbert),
            "morton" => Ok(Self::Morton),
            _ => Err(invalid("unknown explicit image traversal").parameter("traversal")),
        }
    }
    /// Visit every selected pixel once. Corners mirror row/column/diagonal axes;
    /// spirals rotate the starting edge, retaining the named absolute direction.
    /// Reverse reverses the entire coordinate sequence. Constant memory; diagonal
    /// and spiral rank lookup uses at most 15 bisection iterations per coordinate.
    pub fn coordinates(
        self,
        region: Region,
        corner: Corner,
        reverse: bool,
    ) -> Result<Coordinates, OperationError> {
        if region.width == 0
            || region.height == 0
            || region.x.checked_add(region.width).is_none()
            || region.y.checked_add(region.height).is_none()
        {
            return Err(invalid(
                "coordinate region must have positive, nonoverflowing extents",
            ));
        }
        let count = u64::from(region.width) * u64::from(region.height);
        if region.width > MAX_DIMENSION || region.height > MAX_DIMENSION || count > MAX_PIXELS {
            return Err(limit("coordinate traversal exceeds image limits"));
        }
        if matches!(self, Self::Hilbert | Self::Morton)
            && (region.width != region.height
                || !region.width.is_power_of_two()
                || region.width > 2048)
        {
            return Err(invalid(
                "Hilbert and Morton require a power-of-two square up to side 2048",
            ));
        }
        if matches!(self, Self::Serpentine | Self::Hilbert | Self::Morton)
            && count > MAX_EXTENDED_COORDINATES as u64
        {
            return Err(limit("extended traversal exceeds 1 Mi coordinates"));
        }
        Ok(Coordinates {
            traversal: self,
            region,
            corner,
            reverse,
            index: 0,
            count: count as usize,
        })
    }
}

fn hilbert(mut order: usize, side: usize) -> (usize, usize) {
    let (mut x, mut y, mut block) = (0, 0, 1);
    while block < side {
        let rx = (order >> 1) & 1;
        let ry = (order ^ rx) & 1;
        if ry == 0 {
            if rx != 0 {
                (x, y) = (block - 1 - x, block - 1 - y);
            }
            (x, y) = (y, x);
        }
        x += block * rx;
        y += block * ry;
        block <<= 1;
        order >>= 2;
    }
    (x, side - 1 - y)
}

fn morton(mut order: usize, side: usize) -> (usize, usize) {
    let (mut x, mut y) = (0, 0);
    for bit in 0..side.trailing_zeros() {
        x |= (order & 1) << bit;
        y |= ((order >> 1) & 1) << bit;
        order >>= 2;
    }
    (x, y)
}

/// Validate destination-to-source row-major indices and return their absolute
/// source coordinates. A packed bitset enforces an exact rectangular bijection.
pub fn permutation(region: Region, indices: &[u32]) -> Result<Vec<[u32; 2]>, OperationError> {
    let size = Traversal::Row
        .coordinates(region, Corner::TopLeft, false)?
        .len();
    if size > MAX_EXTENDED_COORDINATES {
        return Err(limit("grid permutation exceeds 1 Mi coordinates"));
    }
    if indices.len() != size {
        return Err(invalid(
            "permutation needs exactly one source index per destination",
        ));
    }
    let mut seen = vec![0u64; size.div_ceil(64)];
    for &index in indices {
        if index as usize >= size {
            return Err(invalid("permutation source index exceeds region"));
        }
        let word = &mut seen[index as usize / 64];
        let bit = 1 << (index % 64);
        if *word & bit != 0 {
            return Err(invalid("permutation repeats a source index"));
        }
        *word |= bit;
    }
    Ok(indices
        .iter()
        .map(|&index| {
            [
                region.x + index % region.width,
                region.y + index / region.width,
            ]
        })
        .collect())
}
pub struct Coordinates {
    traversal: Traversal,
    region: Region,
    corner: Corner,
    reverse: bool,
    index: usize,
    count: usize,
}
fn segment(index: usize, mut end: usize, before: impl Fn(usize) -> usize) -> usize {
    let mut start = 0;
    while start + 1 < end {
        let mid = (start + end) / 2;
        if before(mid) <= index {
            start = mid;
        } else {
            end = mid;
        }
    }
    start
}
fn triangle(n: usize) -> usize {
    n * (n + 1) / 2
}
fn diagonal(index: usize, w: usize, h: usize, zigzag: bool) -> (usize, usize) {
    let before =
        |d: usize| triangle(d) - triangle(d.saturating_sub(w)) - triangle(d.saturating_sub(h));
    let d = segment(index, w + h - 1, before);
    let first = d.saturating_sub(h - 1);
    let last = d.min(w - 1);
    let offset = index - before(d);
    let x = if zigzag && d % 2 == 1 {
        last - offset
    } else {
        first + offset
    };
    (x, d - x)
}
fn spiral(index: usize, w: usize, h: usize) -> (usize, usize) {
    let before = |layer: usize| w * h - (w - 2 * layer) * (h - 2 * layer);
    let l = segment(index, w.min(h).div_ceil(2), before);
    let mut i = index - before(l);
    let rw = w - 2 * l;
    let rh = h - 2 * l;
    if rh == 1 {
        return (l + i, l);
    }
    if rw == 1 {
        return (l, l + i);
    }
    if i < rw - 1 {
        return (l + i, l);
    }
    i -= rw - 1;
    if i < rh - 1 {
        return (l + rw - 1, l + i);
    }
    i -= rh - 1;
    if i < rw - 1 {
        return (l + rw - 1 - i, l + rh - 1);
    }
    i -= rw - 1;
    (l, l + rh - 1 - i)
}
impl Iterator for Coordinates {
    type Item = (u32, u32);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.count {
            return None;
        }
        let i = if self.reverse {
            self.count - 1 - self.index
        } else {
            self.index
        };
        self.index += 1;
        let w = self.region.width as usize;
        let h = self.region.height as usize;
        let (x, y) = match self.traversal {
            Traversal::SpiralCw | Traversal::SpiralCcw => {
                let (vw, vh) = if matches!(self.corner, Corner::TopRight | Corner::BottomLeft) {
                    (h, w)
                } else {
                    (w, h)
                };
                let (x, y) = if self.traversal == Traversal::SpiralCcw {
                    let (y, x) = spiral(i, vh, vw);
                    (x, y)
                } else {
                    spiral(i, vw, vh)
                };
                match self.corner {
                    Corner::TopLeft => (x, y),
                    Corner::TopRight => (w - 1 - y, x),
                    Corner::BottomRight => (w - 1 - x, h - 1 - y),
                    Corner::BottomLeft => (y, h - 1 - x),
                }
            }
            pattern => {
                let (x, y) = match pattern {
                    Traversal::Row => (i % w, i / w),
                    Traversal::Column => (i / h, i % h),
                    Traversal::Diagonal => diagonal(i, w, h, false),
                    Traversal::Zigzag => diagonal(i, w, h, true),
                    Traversal::Serpentine => {
                        let (x, y) = (i % w, i / w);
                        (if y % 2 == 0 { x } else { w - 1 - x }, y)
                    }
                    Traversal::Hilbert => hilbert(i, w),
                    Traversal::Morton => morton(i, w),
                    _ => unreachable!(),
                };
                // Hilbert's canonical x/y coordinates start at mathematical
                // bottom-left. After converting to image y-down coordinates,
                // reflect that starting position into the caller's corner.
                let bottom = matches!(self.corner, Corner::BottomLeft | Corner::BottomRight);
                (
                    if matches!(self.corner, Corner::TopRight | Corner::BottomRight) {
                        w - 1 - x
                    } else {
                        x
                    },
                    if bottom != (pattern == Traversal::Hilbert) {
                        h - 1 - y
                    } else {
                        y
                    },
                )
            }
        };
        Some((self.region.x + x as u32, self.region.y + y as u32))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.count - self.index;
        (remaining, Some(remaining))
    }
}
impl ExactSizeIterator for Coordinates {}

use crate::operations::{Parameter, ParameterType, ParameterValue};
pub(super) const REVERSE_PARAMETER: Parameter = Parameter {
    name: "reverse",
    description: "Reverse the complete pixel coordinate sequence, retaining channel and bit order.",
    required: false,
    default: Some(ParameterValue::Boolean(false)),
    constraints: ParameterType::Boolean,
};
pub(super) const TRAVERSAL_PARAMETER: Parameter = Parameter {
    name: "traversal",
    description: "Select row, column, diagonal, zigzag, directed spiral, serpentine, Hilbert or Morton order.",
    required: false,
    default: Some(ParameterValue::Text("row")),
    constraints: ParameterType::Text { choices: NAMES },
};
pub(super) const CORNER_PARAMETER: Parameter = Parameter {
    name: "start_corner",
    description: "Starting corner; axis reflection for linear/diagonal orders, rotated starting edge for directed spirals.",
    required: false,
    default: Some(ParameterValue::Text("top_left")),
    constraints: ParameterType::Text {
        choices: CORNER_NAMES,
    },
};
