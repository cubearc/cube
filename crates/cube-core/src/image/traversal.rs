//! Bounded coordinate ordering, independent of samples, channels and bits.
use super::{MAX_DIMENSION, MAX_PIXELS, Region, invalid, limit};
use crate::errors::OperationError;
use serde::Serialize;

pub const NAMES: &[&str] = &[
    "row",
    "column",
    "diagonal",
    "zigzag",
    "spiral_cw",
    "spiral_ccw",
];
pub const CORNER_NAMES: &[&str] = &["top_left", "top_right", "bottom_right", "bottom_left"];
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Traversal {
    #[default]
    Row,
    Column,
    Diagonal,
    Zigzag,
    SpiralCw,
    SpiralCcw,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
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
            _ => Err(invalid(
                "traversal must be row, column, diagonal, zigzag, spiral_cw or spiral_ccw",
            )
            .parameter("traversal")),
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
                    _ => unreachable!(),
                };
                (
                    if matches!(self.corner, Corner::TopRight | Corner::BottomRight) {
                        w - 1 - x
                    } else {
                        x
                    },
                    if matches!(self.corner, Corner::BottomLeft | Corner::BottomRight) {
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
pub(super) const TRAVERSAL_PARAMETER: Parameter = Parameter {
    name: "traversal",
    description: "Row/column, anti-diagonals, alternating zigzag or clockwise/counterclockwise inward spiral.",
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
