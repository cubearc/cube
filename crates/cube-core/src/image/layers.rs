//! Bounded exact-color PNG collections, reusing histogram and single-color extraction.
use super::{ColorModel, ColorOutput, Image, invalid, limit};
use crate::{
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
use std::collections::BTreeSet;

pub const MAX_OUTPUTS: usize = 64;
pub const MAX_WORK: u64 = 16 * 1024 * 1024;
pub const MAX_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct ColorExport {
    pub color: Vec<u8>,
    pub selected_pixels: u64,
    pub name: String,
    pub status: &'static str,
    #[serde(skip)]
    pub png: Vec<u8>,
}
#[derive(Debug, Serialize)]
pub struct ColorExports {
    pub representation: ColorOutput,
    pub color_model: ColorModel,
    pub pixel_work: u64,
    pub encoded_bytes: usize,
    pub outputs: Vec<ColorExport>,
}
impl Image {
    /// Extract sorted native colors (all physical channels) into bounded PNGs.
    /// `max_colors` limits the complete histogram, even with a selected subset.
    /// A subset must be distinct and present. Both limits are 1..=64; masks/layers
    /// retain the single-color API's semantics. No writes or implicit expansion.
    /// Preflight includes the histogram and every selected full-image scan, a
    /// conservative PNG bound (2*sample bytes+4096 per output), and 64 MiB output.
    pub fn color_layers(
        &self,
        colors: Option<&[Vec<u8>]>,
        representation: ColorOutput,
        max_colors: usize,
        max_outputs: usize,
    ) -> Result<ColorExports, OperationError> {
        let result = (|| {
            if !(1..=MAX_OUTPUTS).contains(&max_colors) || !(1..=MAX_OUTPUTS).contains(&max_outputs)
            {
                return Err(invalid("max_colors and max_outputs must be 1..=64"));
            }
            if colors.is_some_and(|c| c.is_empty() || c.len() > max_outputs) {
                return Err(invalid("color subset must be nonempty and fit max_outputs")
                    .parameter("colors"));
            }
            let histogram = self.histogram(None, None, max_colors)?;
            let mut selected = histogram.colors;
            if let Some(colors) = colors {
                let set: BTreeSet<_> = colors.iter().collect();
                if set.len() != colors.len()
                    || colors
                        .iter()
                        .any(|c| !selected.iter().any(|entry| &entry.values == c))
                {
                    return Err(invalid(
                        "selected colors must be distinct native tuples present in the histogram",
                    )
                    .parameter("colors"));
                }
                selected.retain(|entry| set.contains(&entry.values));
            }
            if selected.len() > max_outputs {
                return Err(limit("color output count exceeds max_outputs"));
            }
            let pixel_work = histogram
                .pixels
                .checked_mul(selected.len() as u64 + 1)
                .ok_or_else(|| limit("color work overflow"))?;
            let channels = if matches!(representation, ColorOutput::Mask) {
                1
            } else {
                4
            };
            let samples = histogram.pixels as usize * channels;
            let encoded_bound = samples
                .checked_mul(2)
                .and_then(|n| n.checked_add(4096))
                .and_then(|n| n.checked_mul(selected.len()))
                .ok_or_else(|| limit("color output bound overflow"))?;
            if pixel_work > MAX_WORK || encoded_bound > MAX_BYTES {
                return Err(limit(
                    "color batch exceeds 16 Mi pixel-work or 64 MiB PNG bound",
                ));
            }
            let mut outputs = Vec::with_capacity(selected.len());
            let mut encoded_bytes = 0usize;
            for color in selected {
                let layer =
                    self.exact_colors(std::slice::from_ref(&color.values), representation)?;
                let png = layer.image.encode_png()?;
                encoded_bytes = encoded_bytes
                    .checked_add(png.len())
                    .ok_or_else(|| limit("encoded output overflow"))?;
                if encoded_bytes > MAX_BYTES {
                    return Err(limit("color PNG outputs exceed 64 MiB"));
                }
                outputs.push(ColorExport {
                    name: format!("color-{}.png", crate::encoding::hex_encode(&color.values)),
                    color: color.values,
                    selected_pixels: layer.selected_pixels,
                    status: "ready",
                    png,
                });
            }
            Ok(ColorExports {
                representation,
                color_model: self.model,
                pixel_work,
                encoded_bytes,
                outputs,
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.colors.export"))
    }
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "image.colors.export",
    category: "image",
    description: "Export sorted exact-color masks/layers as PNG bytes and a manifest; <=64 outputs, 16 Mi pixel-work, 64 MiB conservative PNG bound; no implicit writes.",
    input_type: "image.samples",
    output_type: "image.color_exports",
    parameters: &[
        Parameter {
            name: "colors",
            description: "Optional nonempty distinct native hex tuples, all present; omitted selects all colors.",
            required: false,
            default: None,
            constraints: ParameterType::TextList {
                choices: &[],
                min_length: 1,
                max_length: 64,
                unique: true,
            },
        },
        Parameter {
            name: "representation",
            description: "0/255 mask or RGBA layer with transparent-black background.",
            required: false,
            default: Some(ParameterValue::Text("mask")),
            constraints: ParameterType::Text {
                choices: &["mask", "layer"],
            },
        },
        Parameter {
            name: "max_colors",
            description: "Complete histogram color cap, including with a subset.",
            required: false,
            default: Some(ParameterValue::Unsigned(16)),
            constraints: ParameterType::Unsigned { min: 1, max: 64 },
        },
        Parameter {
            name: "max_outputs",
            description: "Maximum selected outputs; rejects rather than truncates.",
            required: false,
            default: Some(ParameterValue::Unsigned(16)),
            constraints: ParameterType::Unsigned { min: 1, max: 64 },
        },
        super::analyze::EXPAND_PARAMETER,
    ],
    deterministic: true,
    reversible: Some(false),
    command: "image color-layers",
    example: "cube image color-layers sample.png --output-dir layers",
}];
