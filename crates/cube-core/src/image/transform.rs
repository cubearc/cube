//! Exact sample rearrangements and explicitly defined color transforms.
use super::{Channel, ColorModel, Image, Region, invalid};
use crate::errors::OperationError;

impl Image {
    /// Rotate clockwise by canonical quarter-turn angles 0, 90, 180 or 270.
    /// Samples, indices, palette and tRNS are exact; other metadata is dropped.
    pub fn rotate(&self, degrees: u32) -> Result<Self, OperationError> {
        let result = (|| {
            if ![0, 90, 180, 270].contains(&degrees) {
                return Err(
                    invalid("rotation must be 0, 90, 180 or 270 clockwise degrees")
                        .parameter("degrees"),
                );
            }
            let (width, height) = if degrees == 90 || degrees == 270 {
                (self.height, self.width)
            } else {
                (self.width, self.height)
            };
            let mut samples = Vec::with_capacity(self.samples.len());
            for y in 0..height {
                for x in 0..width {
                    let (sx, sy) = match degrees {
                        90 => (y, self.height - 1 - x),
                        180 => (self.width - 1 - x, self.height - 1 - y),
                        270 => (self.width - 1 - y, x),
                        _ => (x, y),
                    };
                    let position = self.position(sx, sy)?;
                    samples.extend_from_slice(
                        &self.samples[position..position + self.model.channels()],
                    );
                }
            }
            Self::from_samples(
                width,
                height,
                self.model,
                samples,
                self.palette.clone(),
                self.transparency.clone(),
            )
        })();
        result.map_err(|e: OperationError| e.operation("image.rotate"))
    }
    /// Mirror left/right (`horizontal`) or top/bottom (`vertical`) without interpolation.
    pub fn mirror(&self, axis: &str) -> Result<Self, OperationError> {
        let result = (|| {
            if !["horizontal", "vertical"].contains(&axis) {
                return Err(invalid("mirror axis must be horizontal or vertical").parameter("axis"));
            }
            let mut samples = Vec::with_capacity(self.samples.len());
            for y in 0..self.height {
                for x in 0..self.width {
                    let (sx, sy) = if axis == "horizontal" {
                        (self.width - 1 - x, y)
                    } else {
                        (x, self.height - 1 - y)
                    };
                    let position = self.position(sx, sy)?;
                    samples.extend_from_slice(
                        &self.samples[position..position + self.model.channels()],
                    );
                }
            }
            Self::from_samples(
                self.width,
                self.height,
                self.model,
                samples,
                self.palette.clone(),
                self.transparency.clone(),
            )
        })();
        result.map_err(|e: OperationError| e.operation("image.mirror"))
    }
    /// Extract a checked rectangle. This discards unselected pixels; no clipping.
    pub fn crop(&self, region: Region) -> Result<Self, OperationError> {
        let result = self.pixels(Some(region), None).and_then(|data| {
            Self::from_samples(
                region.width,
                region.height,
                self.model,
                data.data,
                self.palette.clone(),
                self.transparency.clone(),
            )
        });
        result.map_err(|e| e.operation("image.crop"))
    }
    fn require_color_samples(&self) -> Result<(), OperationError> {
        if self.model == ColorModel::Indexed {
            Err(invalid(
                "color transforms require explicit palette expansion",
            ))
        } else {
            Ok(())
        }
    }
    /// Invert selected sample channels as 255-value. Defaults preserve alpha;
    /// explicit alpha selection is allowed. tRNS color keys transform consistently.
    pub fn invert(&self, channels: Option<&[Channel]>) -> Result<Self, OperationError> {
        let result = (|| {
            self.require_color_samples()?;
            let default: Vec<_> = self
                .channels()
                .iter()
                .copied()
                .filter(|c| *c != Channel::Alpha)
                .collect();
            let (_, indices) = self.selection(Some(channels.unwrap_or(&default)))?;
            let mut samples = self.samples.clone();
            for pixel in samples.chunks_exact_mut(self.model.channels()) {
                for index in &indices {
                    pixel[*index] = 255 - pixel[*index];
                }
            }
            let mut transparency = self.transparency.clone();
            if !transparency.is_empty() {
                for index in &indices {
                    transparency[index * 2 + 1] = 255 - transparency[index * 2 + 1];
                }
            }
            Self::from_samples(
                self.width,
                self.height,
                self.model,
                samples,
                vec![],
                transparency,
            )
        })();
        result.map_err(|e: OperationError| e.operation("image.invert"))
    }
    /// Apply a full channel permutation; values move in the requested order while
    /// the output model stays fixed. Moving alpha is explicit, never implicit.
    pub fn reorder(&self, channels: &[Channel]) -> Result<Self, OperationError> {
        let result = (|| {
            self.require_color_samples()?;
            let (_, indices) = self.selection(Some(channels))?;
            if indices.len() != self.model.channels() {
                return Err(
                    invalid("reordering requires a complete channel permutation")
                        .parameter("channels"),
                );
            }
            let mut samples = Vec::with_capacity(self.samples.len());
            for pixel in self.samples.chunks_exact(self.model.channels()) {
                for index in &indices {
                    samples.push(pixel[*index]);
                }
            }
            let transparency = if self.transparency.is_empty() {
                vec![]
            } else {
                indices
                    .iter()
                    .flat_map(|i| [self.transparency[i * 2], self.transparency[i * 2 + 1]])
                    .collect()
            };
            Self::from_samples(
                self.width,
                self.height,
                self.model,
                samples,
                vec![],
                transparency,
            )
        })();
        result.map_err(|e: OperationError| e.operation("image.reorder"))
    }
    /// Convert stored RGB samples using (77R+150G+29B+128)>>8, without gamma or
    /// profile corrections. Color information is lost. Alpha is preserved; color
    /// keys become explicit per-pixel alpha to avoid grayscale transparency collisions.
    pub fn grayscale(&self) -> Result<Self, OperationError> {
        let result = (|| {
            self.require_color_samples()?;
            let alpha = matches!(self.model, ColorModel::GrayAlpha | ColorModel::Rgba)
                || !self.transparency.is_empty();
            let model = if alpha {
                ColorModel::GrayAlpha
            } else {
                ColorModel::Gray
            };
            let mut samples =
                Vec::with_capacity(self.width as usize * self.height as usize * model.channels());
            let key: Vec<_> = self
                .transparency
                .as_chunks::<2>()
                .0
                .iter()
                .map(|v| v[1])
                .collect();
            for pixel in self.samples.chunks_exact(self.model.channels()) {
                let gray = if matches!(self.model, ColorModel::Gray | ColorModel::GrayAlpha) {
                    pixel[0]
                } else {
                    ((77 * u32::from(pixel[0])
                        + 150 * u32::from(pixel[1])
                        + 29 * u32::from(pixel[2])
                        + 128)
                        >> 8) as u8
                };
                samples.push(gray);
                if alpha {
                    samples.push(match self.model {
                        ColorModel::GrayAlpha => pixel[1],
                        ColorModel::Rgba => pixel[3],
                        _ => {
                            if pixel == key {
                                0
                            } else {
                                255
                            }
                        }
                    });
                }
            }
            Self::from_samples(self.width, self.height, model, samples, vec![], vec![])
        })();
        result.map_err(|e: OperationError| e.operation("image.grayscale"))
    }
}

use super::analyze::{CHANNELS_PARAMETER, EXPAND_PARAMETER, REGION_PARAMETER};
use crate::operations::{Descriptor, Parameter, ParameterType};
/// Static discovery alongside the exact transform implementation.
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "image.rotate",
        category: "image",
        description: "Rotate exact samples clockwise by 0/90/180/270; no interpolation.",
        input_type: "image.samples",
        output_type: "image.samples",
        deterministic: true,
        reversible: Some(true),
        command: "image rotate",
        example: "cube image rotate sample.png --degrees 90 --output rotated.png",
        parameters: &[Parameter {
            name: "degrees",
            description: "Canonical clockwise quarter turn; width/height swap at 90/270.",
            required: true,
            default: None,
            constraints: ParameterType::UnsignedChoices {
                choices: &[0, 90, 180, 270],
            },
        }],
    },
    Descriptor {
        id: "image.mirror",
        category: "image",
        description: "Mirror exact samples left/right or top/bottom.",
        input_type: "image.samples",
        output_type: "image.samples",
        deterministic: true,
        reversible: Some(true),
        command: "image mirror",
        example: "cube image mirror sample.png --axis horizontal --output mirrored.png",
        parameters: &[Parameter {
            name: "axis",
            description: "horizontal reverses x; vertical reverses y.",
            required: true,
            default: None,
            constraints: ParameterType::Text {
                choices: &["horizontal", "vertical"],
            },
        }],
    },
    Descriptor {
        id: "image.crop",
        category: "image",
        description: "Extract a checked rectangle without resampling; unselected pixels are discarded.",
        input_type: "image.samples",
        output_type: "image.samples",
        deterministic: true,
        reversible: Some(false),
        command: "image crop",
        example: "cube image crop sample.png --region 0,0,1,1 --output crop.png",
        parameters: &[Parameter {
            required: true,
            ..REGION_PARAMETER
        }],
    },
    Descriptor {
        id: "image.invert",
        category: "image",
        description: "Invert selected 8-bit samples as 255-value, preserving alpha by default.",
        input_type: "image.samples",
        output_type: "image.samples",
        deterministic: true,
        reversible: Some(true),
        command: "image invert",
        example: "cube image invert sample.png --output inverted.png",
        parameters: &[
            Parameter {
                description: "Ordered selected channels; omitted means native color channels excluding alpha.",
                ..CHANNELS_PARAMETER
            },
            EXPAND_PARAMETER,
        ],
    },
    Descriptor {
        id: "image.reorder",
        category: "image",
        description: "Apply a full sample channel permutation; moving alpha must be explicit.",
        input_type: "image.samples",
        output_type: "image.samples",
        deterministic: true,
        reversible: Some(true),
        command: "image reorder",
        example: "cube image reorder sample.png --channels blue,green,red,alpha --output reordered.png",
        parameters: &[
            Parameter {
                required: true,
                description: "Full ordered permutation of native sample channels; output model remains fixed.",
                ..CHANNELS_PARAMETER
            },
            EXPAND_PARAMETER,
        ],
    },
    Descriptor {
        id: "image.grayscale",
        category: "image",
        description: "Lossy color reduction using (77R+150G+29B+128)>>8 without gamma conversion; preserve alpha.",
        input_type: "image.samples",
        output_type: "image.samples",
        deterministic: true,
        reversible: Some(false),
        command: "image grayscale",
        example: "cube image grayscale sample.png --output gray.png",
        parameters: &[EXPAND_PARAMETER],
    },
];
