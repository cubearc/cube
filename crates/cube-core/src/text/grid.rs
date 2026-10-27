//! Exact Unicode-scalar grids; factorization and traversal are separate from rendering.
use crate::{
    errors::{ErrorCode, OperationError},
    image::{ColorModel, Corner, Image, MAX_DIMENSION, Region, Traversal},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};
pub const MAX_UNITS: usize = 65536;
fn error(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}
fn source(input: &str, drop_whitespace: bool) -> Result<(Vec<char>, usize), OperationError> {
    super::check_input(input, "text.grid.render")?;
    let mut chars = Vec::new();
    let mut removed = 0;
    for c in input.chars() {
        if drop_whitespace && c.is_whitespace() {
            removed += 1;
        } else {
            if chars.len() == MAX_UNITS {
                return Err(OperationError::new(
                    ErrorCode::ResourceLimit,
                    "grid exceeds 65536 retained scalars",
                ));
            }
            chars.push(c);
        }
    }
    if chars.is_empty() {
        return Err(error("grid requires nonempty retained input"));
    }
    Ok((chars, removed))
}
#[derive(Debug, Serialize, PartialEq, Eq, schemars::JsonSchema)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Factors {
    pub retained_scalars: usize,
    pub removed_scalars: usize,
    pub candidates: Vec<Dimensions>,
}
/// Positive factor pairs in ascending width, including both orientations when
/// each side fits 16384. Unicode scalars are retained without normalization.
/// Optional filtering uses Rust White_Space (selection::WORD_UNICODE_VERSION).
pub fn factors(
    input: &str,
    drop_whitespace: bool,
    max_candidates: usize,
) -> Result<Factors, OperationError> {
    let result = (|| {
        let (chars, removed) = source(input, drop_whitespace)?;
        if !(1..=128).contains(&max_candidates) {
            return Err(error("max_candidates must be 1..=128"));
        }
        let mut candidates = Vec::new();
        for width in 1..=chars.len().min(MAX_DIMENSION as usize) {
            if chars.len() % width == 0 && chars.len() / width <= MAX_DIMENSION as usize {
                if candidates.len() == max_candidates {
                    return Err(OperationError::new(
                        ErrorCode::ResourceLimit,
                        "factor candidate count exceeds limit",
                    ));
                }
                candidates.push(Dimensions {
                    width: width as u32,
                    height: (chars.len() / width) as u32,
                });
            }
        }
        Ok(Factors {
            retained_scalars: chars.len(),
            removed_scalars: removed,
            candidates,
        })
    })();
    result.map_err(|e: OperationError| e.operation("text.grid.factors"))
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub retained_scalars: usize,
    pub removed_scalars: usize,
    pub rows: Vec<Vec<String>>,
}
/// Place source scalars at existing traversal coordinates, from the top-left.
/// Width must divide retained length exactly; no padding or inferred dimensions.
pub fn render(
    input: &str,
    width: u32,
    drop_whitespace: bool,
    traversal: Traversal,
) -> Result<Grid, OperationError> {
    let result = (|| {
        let (chars, removed) = source(input, drop_whitespace)?;
        if width == 0 || width > MAX_DIMENSION || chars.len() % width as usize != 0 {
            return Err(error(
                "width must be positive, <=16384 and divide retained scalar count",
            ));
        }
        let height = (chars.len() / width as usize) as u32;
        let coordinates = traversal.coordinates(
            Region {
                x: 0,
                y: 0,
                width,
                height,
            },
            Corner::TopLeft,
            false,
        )?;
        let mut cells = vec![String::new(); chars.len()];
        for (c, (x, y)) in chars.into_iter().zip(coordinates) {
            cells[(y * width + x) as usize] = c.to_string();
        }
        Ok(Grid {
            width,
            height,
            retained_scalars: cells.len(),
            removed_scalars: removed,
            rows: cells
                .chunks(width as usize)
                .map(<[String]>::to_vec)
                .collect(),
        })
    })();
    result.map_err(|e: OperationError| e.operation("text.grid.render"))
}
impl Grid {
    fn validate(&self) -> Result<(), OperationError> {
        let count = u64::from(self.width) * u64::from(self.height);
        if self.width == 0
            || self.height == 0
            || self.width > MAX_DIMENSION
            || self.height > MAX_DIMENSION
            || count > MAX_UNITS as u64
            || self.rows.len() != self.height as usize
            || self
                .rows
                .iter()
                .any(|r| r.len() != self.width as usize || r.iter().any(|s| s.chars().count() != 1))
        {
            return Err(error("invalid bounded scalar grid").operation("text.grid.render"));
        }
        Ok(())
    }
    /// Tab-separated JSON-quoted scalars, one row per LF, including a final LF.
    /// Embedded tabs/newlines/controls remain unambiguous escaped symbols.
    pub fn text(&self) -> Result<String, OperationError> {
        self.validate()?;
        let mut text = String::new();
        for row in &self.rows {
            for (i, symbol) in row.iter().enumerate() {
                if i > 0 {
                    text.push('\t');
                }
                text.push_str(
                    &serde_json::to_string(symbol)
                        .map_err(|e| error("grid text serialization failed").caused_by(e))?,
                );
            }
            text.push('\n');
        }
        Ok(text)
    }
    /// Exact symbol-to-RGBA mapping, with no fonts. Require every symbol exactly
    /// once and no unused keys. At most 256 mappings and 65536 output pixels.
    pub fn image(&self, colors: &[(String, [u8; 4])]) -> Result<Image, OperationError> {
        let result = (|| {
            self.validate()?;
            if colors.is_empty() || colors.len() > 256 {
                return Err(error("select 1..=256 symbol color mappings"));
            }
            let map: BTreeMap<_, _> = colors.iter().map(|(s, c)| (s.as_str(), c)).collect();
            let symbols: BTreeSet<_> = self.rows.iter().flatten().map(String::as_str).collect();
            if map.len() != colors.len()
                || map.len() != symbols.len()
                || symbols.iter().any(|s| !map.contains_key(s))
            {
                return Err(error(
                    "color mappings must cover each grid symbol exactly once",
                ));
            }
            let samples = self
                .rows
                .iter()
                .flatten()
                .flat_map(|s| map[s.as_str()].iter().copied())
                .collect();
            Image::from_samples(
                self.width,
                self.height,
                ColorModel::Rgba,
                samples,
                vec![],
                vec![],
            )
        })();
        result.map_err(|e: OperationError| e.operation("text.grid.render"))
    }
}
/// CLI mapping transport: strict JSON array of [single scalar, eight Hex digits].
pub fn parse_colors(document: &str) -> Result<Vec<(String, [u8; 4])>, OperationError> {
    let result = (|| {
        if document.len() > 8192 {
            return Err(error("color mapping document exceeds 8192 bytes"));
        }
        let value = crate::pipeline::json::parse(document)
            .map_err(|e| error("invalid bounded color JSON").caused_by(e))?;
        let rows = value
            .as_array()
            .ok_or_else(|| error("colors must be an array of [symbol,RGBA Hex] pairs"))?;
        if rows.len() > 256 {
            return Err(error("color mapping count exceeds 256"));
        }
        rows.iter()
            .map(|row| {
                let row = row
                    .as_array()
                    .filter(|r| r.len() == 2)
                    .ok_or_else(|| error("each mapping requires [symbol,RGBA Hex]"))?;
                let symbol = row[0]
                    .as_str()
                    .filter(|s| s.chars().count() == 1)
                    .ok_or_else(|| error("mapping key must be one Unicode scalar"))?;
                let hex = row[1]
                    .as_str()
                    .filter(|s| s.len() == 8)
                    .ok_or_else(|| error("RGBA requires eight Hex digits"))?;
                let bytes = crate::encoding::hex_decode(hex)
                    .map_err(|e| error("invalid RGBA Hex").caused_by(e))?;
                Ok((
                    symbol.to_owned(),
                    <[u8; 4]>::try_from(bytes).map_err(|_| error("RGBA requires four bytes"))?,
                ))
            })
            .collect()
    })();
    result.map_err(|e: OperationError| e.operation("text.grid.render").parameter("colors"))
}
const FILTER: Parameter = Parameter {
    name: "drop_whitespace",
    description: "Explicit Rust Unicode White_Space removal; no normalization or other filtering.",
    required: false,
    default: Some(ParameterValue::Boolean(false)),
    constraints: ParameterType::Boolean,
};
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "text.grid.factors",
        category: "text",
        description: "Enumerate ascending-width factors of 1..=65536 retained Unicode scalars; sides <=16384, <=128 candidates; no inference.",
        input_type: "text",
        output_type: "text.grid_factors",
        parameters: &[
            FILTER,
            Parameter {
                name: "max_candidates",
                description: "Reject excessive candidate count, never truncate.",
                required: false,
                default: Some(ParameterValue::Unsigned(128)),
                constraints: ParameterType::Unsigned { min: 1, max: 128 },
            },
        ],
        deterministic: true,
        reversible: None,
        command: "text grid",
        example: "cube text grid --text ABCDEF --json",
    },
    Descriptor {
        id: "text.grid.render",
        category: "text",
        description: "Place exact scalars using existing image traversal; quoted-cell text or explicit RGBA mapping, <=65536 cells; no fonts or writes.",
        input_type: "text",
        output_type: "text.grid",
        parameters: &[
            FILTER,
            Parameter {
                name: "width",
                description: "Explicit divisor of retained length; sides <=16384.",
                required: true,
                default: None,
                constraints: ParameterType::Unsigned { min: 1, max: 16384 },
            },
            Parameter {
                name: "traversal",
                description: "Existing coordinate traversal, starting top-left.",
                required: false,
                default: Some(ParameterValue::Text("row")),
                constraints: ParameterType::Text {
                    choices: crate::image::traversal::NAMES,
                },
            },
            Parameter {
                name: "colors",
                description: "Optional explicit JSON array of [single scalar, eight RGBA Hex digits]; <=256 mappings and 8192 bytes.",
                required: false,
                default: None,
                constraints: ParameterType::Text { choices: &[] },
            },
        ],
        deterministic: true,
        reversible: Some(false),
        command: "text grid",
        example: "cube text grid --text ABCDEF --width 3",
    },
];
