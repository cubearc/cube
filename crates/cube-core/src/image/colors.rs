//! Exact native color selection, without distance metrics or message inference.
use super::{ColorModel, Image, invalid, sample_count};
use crate::{
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ColorOutput {
    Mask,
    Layer,
}
impl ColorOutput {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "mask" => Ok(Self::Mask),
            "layer" => Ok(Self::Layer),
            _ => Err(invalid("representation must be mask or layer").parameter("representation")),
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct ColorLayer {
    pub selected_pixels: u64,
    pub representation: ColorOutput,
    #[serde(skip)]
    pub image: Image,
}
impl Image {
    fn rgba_pixel(&self, pixel: &[u8]) -> [u8; 4] {
        match self.model {
            ColorModel::Indexed => {
                let index = pixel[0] as usize;
                let [r, g, b] = self.palette[index];
                [
                    r,
                    g,
                    b,
                    self.transparency.get(index).copied().unwrap_or(255),
                ]
            }
            ColorModel::Gray => [
                pixel[0],
                pixel[0],
                pixel[0],
                if self.transparency == [0, pixel[0]] {
                    0
                } else {
                    255
                },
            ],
            ColorModel::GrayAlpha => [pixel[0], pixel[0], pixel[0], pixel[1]],
            ColorModel::Rgb => [
                pixel[0],
                pixel[1],
                pixel[2],
                if self.transparency == [0, pixel[0], 0, pixel[1], 0, pixel[2]] {
                    0
                } else {
                    255
                },
            ],
            ColorModel::Rgba => [pixel[0], pixel[1], pixel[2], pixel[3]],
        }
    }
    /// Match 1..=256 distinct exact native tuples, including physical alpha.
    /// Indexed tuples are single indices; explicitly expand palettes before this
    /// call to match RGBA colors instead. A mask uses 255 for selected pixels and
    /// zero elsewhere. A layer always uses RGBA, preserving selected colors/alpha
    /// (including tRNS) and making all unselected pixels transparent black.
    pub fn exact_colors(
        &self,
        colors: &[Vec<u8>],
        representation: ColorOutput,
    ) -> Result<ColorLayer, OperationError> {
        let result = (|| {
            if colors.is_empty()
                || colors.len() > 256
                || colors.iter().any(|c| c.len() != self.model.channels())
            {
                return Err(invalid(
                    "select 1..=256 exact colors, each matching the native channel count",
                )
                .parameter("colors"));
            }
            let selected: BTreeSet<&[u8]> = colors.iter().map(Vec::as_slice).collect();
            if selected.len() != colors.len() {
                return Err(invalid("duplicate exact color selection").parameter("colors"));
            }
            let model = match representation {
                ColorOutput::Mask => ColorModel::Gray,
                ColorOutput::Layer => ColorModel::Rgba,
            };
            let mut samples =
                Vec::with_capacity(sample_count(self.width, self.height, model.channels())?);
            let mut count = 0;
            for pixel in self.samples.chunks_exact(self.model.channels()) {
                let matches = selected.contains(pixel);
                count += u64::from(matches);
                match representation {
                    ColorOutput::Mask => samples.push(if matches { 255 } else { 0 }),
                    ColorOutput::Layer => samples.extend_from_slice(&if matches {
                        self.rgba_pixel(pixel)
                    } else {
                        [0; 4]
                    }),
                }
            }
            Ok(ColorLayer {
                selected_pixels: count,
                representation,
                image: Image::from_samples(
                    self.width,
                    self.height,
                    model,
                    samples,
                    vec![],
                    vec![],
                )?,
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.colors.extract"))
    }
}
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "image.colors.extract",
    category: "image",
    description: "Isolate exact native colors as a coordinate-preserving 0/255 mask or RGBA layer with transparent-black background.",
    input_type: "image.samples",
    output_type: "image.color_layer",
    parameters: &[
        Parameter {
            name: "colors",
            description: "1..=256 distinct exact native tuples, rendered as hexadecimal sample bytes. Indexed input matches indices unless explicitly expanded.",
            required: true,
            default: None,
            constraints: ParameterType::TextList {
                choices: &[],
                min_length: 1,
                max_length: 256,
                unique: true,
            },
        },
        Parameter {
            name: "representation",
            description: "Mask: selected=255, other=0. Layer: selected original RGBA, other transparent black.",
            required: false,
            default: Some(ParameterValue::Text("mask")),
            constraints: ParameterType::Text {
                choices: &["mask", "layer"],
            },
        },
        super::analyze::EXPAND_PARAMETER,
    ],
    deterministic: true,
    reversible: Some(false),
    command: "image colors",
    example: "cube image colors sample.png --colors ff0000,0000ff --output mask.png",
}];
