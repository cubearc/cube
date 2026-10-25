//! Exact pixel/channel exports, histograms and comparisons; no inferred messages.
use super::{ColorModel, Image, invalid, limit};
use crate::errors::OperationError;
use serde::Serialize;
use std::collections::BTreeMap;

/// Named physical sample channel. Palette expansion must be an explicit operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Red,
    Green,
    Blue,
    Alpha,
    Gray,
    Index,
}
impl Channel {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "blue" => Ok(Self::Blue),
            "alpha" => Ok(Self::Alpha),
            "gray" => Ok(Self::Gray),
            "index" => Ok(Self::Index),
            _ => Err(
                invalid("channel must be red, green, blue, alpha, gray or index")
                    .parameter("channels"),
            ),
        }
    }
}

/// Checked half-open rectangle in absolute top-left image coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl std::str::FromStr for Region {
    type Err = OperationError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut fields = value.split(',');
        let mut number = || {
            fields
                .next()
                .and_then(|v| v.parse::<u32>().ok())
                .ok_or_else(|| {
                    invalid("region must be x,y,width,height with four unsigned integers")
                        .parameter("region")
                })
        };
        let result = Self {
            x: number()?,
            y: number()?,
            width: number()?,
            height: number()?,
        };
        if fields.next().is_some() {
            return Err(invalid("region must contain exactly four integers").parameter("region"));
        }
        Ok(result)
    }
}
impl Region {
    pub fn full(image: &Image) -> Self {
        Self {
            x: 0,
            y: 0,
            width: image.width,
            height: image.height,
        }
    }
    pub fn validate(self, image: &Image) -> Result<Self, OperationError> {
        if self.width == 0
            || self.height == 0
            || self
                .x
                .checked_add(self.width)
                .is_none_or(|end| end > image.width)
            || self
                .y
                .checked_add(self.height)
                .is_none_or(|end| end > image.height)
        {
            return Err(
                invalid("region must be positive and entirely inside the image")
                    .parameter("region"),
            );
        }
        Ok(self)
    }
}

/// Shape and exact interleaved row-major selected channel samples.
#[derive(Debug, Serialize)]
pub struct PixelData {
    pub region: Region,
    pub channels: Vec<Channel>,
    pub bit_depth: u8,
    #[serde(skip)]
    pub data: Vec<u8>,
}
#[derive(Debug, Serialize)]
pub struct PixelRecord {
    pub x: u32,
    pub y: u32,
    pub values: Vec<u8>,
}
impl PixelData {
    /// Produce coordinate records under a 10000-pixel JSON/record budget.
    pub fn records(&self) -> Result<Vec<PixelRecord>, OperationError> {
        let count = u64::from(self.region.width) * u64::from(self.region.height);
        if count > 10000 {
            return Err(limit(
                "coordinate records exceed 10000 pixels; use raw samples",
            ));
        }
        if self.channels.is_empty()
            || self.data.len() as u64 != count * self.channels.len() as u64
            || self.region.width == 0
            || self.region.height == 0
            || self.region.x.checked_add(self.region.width).is_none()
            || self.region.y.checked_add(self.region.height).is_none()
        {
            return Err(invalid("invalid exported pixel shape"));
        }
        Ok(self
            .data
            .chunks_exact(self.channels.len())
            .enumerate()
            .map(|(i, values)| PixelRecord {
                x: self.region.x + (i as u32 % self.region.width),
                y: self.region.y + (i as u32 / self.region.width),
                values: values.to_vec(),
            })
            .collect())
    }
    /// Render one raw channel as grayscale without scaling or normalization.
    pub fn grayscale_image(&self) -> Result<Image, OperationError> {
        if self.channels.len() != 1 {
            return Err(invalid(
                "grayscale channel rendering requires exactly one channel",
            ));
        }
        Image::from_bytes(
            self.region.width,
            self.region.height,
            ColorModel::Gray,
            &self.data,
            &[],
            &[],
        )
    }
}

#[derive(Debug, Serialize)]
pub struct ColorCount {
    pub values: Vec<u8>,
    pub count: u64,
}
/// Exact histogram or an explicit resource-limit error; results are never quantized.
#[derive(Debug, Serialize)]
pub struct Histogram {
    pub region: Region,
    pub channels: Vec<Channel>,
    pub pixels: u64,
    pub colors: Vec<ColorCount>,
}
#[derive(Debug, Serialize)]
pub struct PixelDifference {
    pub x: u32,
    pub y: u32,
    pub left: Vec<u8>,
    pub right: Vec<u8>,
}
#[derive(Debug, Serialize)]
pub struct Comparison {
    pub region: Region,
    pub channels: Vec<Channel>,
    pub equal: bool,
    pub differing_pixels: u64,
    pub complete: bool,
    pub differences: Vec<PixelDifference>,
    #[serde(skip)]
    pub mask: Option<Image>,
}

impl Image {
    /// Borrow native samples or explicitly allocate an indexed-to-RGBA view.
    pub fn view(&self, expand_palette: bool) -> Result<std::borrow::Cow<'_, Self>, OperationError> {
        if expand_palette {
            Ok(std::borrow::Cow::Owned(self.expand_palette()?))
        } else {
            Ok(std::borrow::Cow::Borrowed(self))
        }
    }
    pub fn channels(&self) -> &'static [Channel] {
        match self.model {
            ColorModel::Gray => &[Channel::Gray],
            ColorModel::GrayAlpha => &[Channel::Gray, Channel::Alpha],
            ColorModel::Rgb => &[Channel::Red, Channel::Green, Channel::Blue],
            ColorModel::Rgba => &[Channel::Red, Channel::Green, Channel::Blue, Channel::Alpha],
            ColorModel::Indexed => &[Channel::Index],
        }
    }
    pub(super) fn selection(
        &self,
        channels: Option<&[Channel]>,
    ) -> Result<(Vec<Channel>, Vec<usize>), OperationError> {
        let selected = channels.unwrap_or_else(|| self.channels());
        if selected.is_empty() || selected.len() > 4 {
            return Err(
                invalid("select between one and four distinct channels").parameter("channels")
            );
        }
        let mut indices = Vec::new();
        for channel in selected {
            let index = self.channels().iter().position(|c| c == channel).ok_or_else(|| invalid("channel unavailable in this model; explicitly expand indexed palettes if needed").parameter("channels"))?;
            if indices.contains(&index) {
                return Err(invalid("duplicate channel selection").parameter("channels"));
            }
            indices.push(index);
        }
        Ok((selected.to_vec(), indices))
    }
    pub(super) fn position(&self, x: u32, y: u32) -> Result<usize, OperationError> {
        if x >= self.width || y >= self.height {
            return Err(invalid("pixel coordinate lies outside the image").parameter("coordinates"));
        }
        Ok((y as usize * self.width as usize + x as usize) * self.model.channels())
    }
    /// Read exact native samples at a coordinate; indexed results remain indices.
    pub fn pixel(&self, x: u32, y: u32) -> Result<Vec<u8>, OperationError> {
        let position = self
            .position(x, y)
            .map_err(|e| e.operation("image.pixels.read"))?;
        Ok(self.samples[position..position + self.model.channels()].to_vec())
    }
    /// Export a checked region and ordered channel selection as exact bytes.
    /// Omitted channels use the native model (including raw indices for indexed PNG).
    pub fn pixels(
        &self,
        region: Option<Region>,
        channels: Option<&[Channel]>,
    ) -> Result<PixelData, OperationError> {
        self.pixels_with_traversal(
            region,
            channels,
            super::Traversal::Row,
            super::Corner::TopLeft,
            false,
        )
        .map(|v| v.pixels)
    }
    /// Select one named channel; missing alpha and implicit palette conversion fail.
    pub fn channel(
        &self,
        channel: Channel,
        region: Option<Region>,
    ) -> Result<PixelData, OperationError> {
        self.pixels(region, Some(&[channel]))
            .map_err(|e| e.operation("image.channel.extract"))
    }

    /// Count exact channel tuples, lexicographically sorted. Alpha participates when
    /// selected; transparent pixels are never blended. Native indexed histograms
    /// count indices; explicitly expand palettes to count decoded RGBA colors.
    pub fn histogram(
        &self,
        region: Option<Region>,
        channels: Option<&[Channel]>,
        max_colors: usize,
    ) -> Result<Histogram, OperationError> {
        let result = (|| {
            if !(1..=10000).contains(&max_colors) {
                return Err(
                    invalid("max_colors must be between 1 and 10000").parameter("max_colors")
                );
            }
            let region = region
                .unwrap_or_else(|| Region::full(self))
                .validate(self)?;
            let (channels, indices) = self.selection(channels)?;
            let mut counts = BTreeMap::<Vec<u8>, u64>::new();
            for y in region.y..region.y + region.height {
                for x in region.x..region.x + region.width {
                    let position = self.position(x, y)?;
                    let color: Vec<_> = indices
                        .iter()
                        .map(|index| self.samples[position + index])
                        .collect();
                    if !counts.contains_key(&color) && counts.len() == max_colors {
                        return Err(limit(
                            "exact histogram exceeds distinct-color budget; choose a smaller region",
                        ));
                    }
                    *counts.entry(color).or_default() += 1;
                }
            }
            Ok(Histogram {
                region,
                channels,
                pixels: u64::from(region.width) * u64::from(region.height),
                colors: counts
                    .into_iter()
                    .map(|(values, count)| ColorCount { values, count })
                    .collect(),
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.histogram"))
    }

    /// Compare exact samples in row-major order. Default comparison requires the
    /// same physical model and palette/tRNS semantics; an explicit channel subset
    /// may intentionally ignore other channels/metadata. No resizing or blending.
    /// Counts remain exact when the listing is capped. Optional masks use 0/255.
    pub fn compare(
        &self,
        other: &Self,
        region: Option<Region>,
        channels: Option<&[Channel]>,
        max_results: usize,
        include_mask: bool,
    ) -> Result<Comparison, OperationError> {
        let result = (|| {
            if self.width != other.width || self.height != other.height {
                return Err(invalid("image dimensions differ; comparison never resizes"));
            }
            if channels.is_none()
                && (self.model != other.model
                    || self.palette != other.palette
                    || self.transparency != other.transparency)
            {
                return Err(invalid(
                    "sample models or palette/transparency semantics differ; choose an explicit conversion or channel subset",
                ));
            }
            if !(1..=10000).contains(&max_results) {
                return Err(
                    invalid("max_results must be between 1 and 10000").parameter("max_results")
                );
            }
            let region = region
                .unwrap_or_else(|| Region::full(self))
                .validate(self)?;
            let (names, left_indices) = self.selection(channels)?;
            let (_, right_indices) = other.selection(Some(&names))?;
            let mut differences = Vec::new();
            let mut differing_pixels = 0;
            let mut mask = if include_mask {
                Vec::with_capacity(region.width as usize * region.height as usize)
            } else {
                Vec::new()
            };
            for y in region.y..region.y + region.height {
                for x in region.x..region.x + region.width {
                    let a = self.position(x, y)?;
                    let b = other.position(x, y)?;
                    let differs = left_indices
                        .iter()
                        .zip(&right_indices)
                        .any(|(l, r)| self.samples[a + l] != other.samples[b + r]);
                    if include_mask {
                        mask.push(if differs { 255 } else { 0 });
                    }
                    if differs {
                        differing_pixels += 1;
                        if differences.len() < max_results {
                            differences.push(PixelDifference {
                                x,
                                y,
                                left: left_indices.iter().map(|i| self.samples[a + i]).collect(),
                                right: right_indices.iter().map(|i| other.samples[b + i]).collect(),
                            });
                        }
                    }
                }
            }
            Ok(Comparison {
                region,
                channels: names,
                equal: differing_pixels == 0,
                differing_pixels,
                complete: differences.len() as u64 == differing_pixels,
                differences,
                mask: if include_mask {
                    Some(Image::from_samples(
                        region.width,
                        region.height,
                        ColorModel::Gray,
                        mask,
                        vec![],
                        vec![],
                    )?)
                } else {
                    None
                },
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.compare"))
    }
}

pub(super) const CHANNEL_NAMES: &[&str] = &["red", "green", "blue", "alpha", "gray", "index"];
pub(super) const REGION_PARAMETER: crate::operations::Parameter = crate::operations::Parameter {
    name: "region",
    description: "Optional [x,y,width,height] rectangle; width/height must be positive and bounds checked against the image.",
    required: false,
    default: None,
    constraints: crate::operations::ParameterType::UnsignedList {
        min: 0,
        max: u32::MAX as u64,
        min_length: 4,
        max_length: 4,
        unique: false,
    },
};
pub(super) const CHANNELS_PARAMETER: crate::operations::Parameter = crate::operations::Parameter {
    name: "channels",
    description: "Ordered distinct channels; omitted selects native sample order, including raw palette indices.",
    required: false,
    default: None,
    constraints: crate::operations::ParameterType::TextList {
        choices: CHANNEL_NAMES,
        min_length: 1,
        max_length: 4,
        unique: true,
    },
};
pub(super) const EXPAND_PARAMETER: crate::operations::Parameter = crate::operations::Parameter {
    name: "expand_palette",
    description: "Explicitly convert indexed samples to RGBA before analysis.",
    required: false,
    default: Some(crate::operations::ParameterValue::Boolean(false)),
    constraints: crate::operations::ParameterType::Boolean,
};
/// Metadata is kept with the reusable analysis methods.
pub const OPERATIONS: &[crate::operations::Descriptor] = &[
    crate::operations::Descriptor {
        id: "image.pixels.read",
        category: "image",
        description: "Read exact native pixel samples at top-left coordinates.",
        input_type: "image.samples",
        output_type: "bytes",
        deterministic: true,
        reversible: None,
        command: "image pixel",
        example: "cube image pixel sample.png --x 0 --y 0",
        parameters: &[
            crate::operations::Parameter {
                name: "x",
                description: "Zero-based horizontal coordinate.",
                required: true,
                default: None,
                constraints: crate::operations::ParameterType::Unsigned {
                    min: 0,
                    max: u32::MAX as u64,
                },
            },
            crate::operations::Parameter {
                name: "y",
                description: "Zero-based vertical coordinate.",
                required: true,
                default: None,
                constraints: crate::operations::ParameterType::Unsigned {
                    min: 0,
                    max: u32::MAX as u64,
                },
            },
            EXPAND_PARAMETER,
        ],
    },
    crate::operations::Descriptor {
        id: "image.pixels.export",
        category: "image",
        description: "Export selected samples with explicit shape/channel metadata or bounded coordinate records.",
        input_type: "image.samples",
        output_type: "image.pixel_data",
        deterministic: true,
        reversible: None,
        command: "image pixels",
        example: "cube image pixels sample.png --json",
        parameters: &[
            REGION_PARAMETER,
            CHANNELS_PARAMETER,
            EXPAND_PARAMETER,
            super::traversal::TRAVERSAL_PARAMETER,
            super::traversal::CORNER_PARAMETER,
            super::traversal::REVERSE_PARAMETER,
        ],
    },
    crate::operations::Descriptor {
        id: "image.channel.extract",
        category: "image",
        description: "Extract one exact channel as bytes and optional grayscale PNG.",
        input_type: "image.samples",
        output_type: "image.pixel_data",
        deterministic: true,
        reversible: None,
        command: "image channel",
        example: "cube image channel sample.png --channel red --output red.bin",
        parameters: &[
            crate::operations::Parameter {
                name: "channel",
                description: "One named available channel.",
                required: true,
                default: None,
                constraints: crate::operations::ParameterType::Text {
                    choices: CHANNEL_NAMES,
                },
            },
            REGION_PARAMETER,
            EXPAND_PARAMETER,
        ],
    },
    crate::operations::Descriptor {
        id: "image.histogram",
        category: "image",
        description: "Count exact selected channel tuples without blending, approximation or quantization.",
        input_type: "image.samples",
        output_type: "image.histogram",
        deterministic: true,
        reversible: None,
        command: "image histogram",
        example: "cube image histogram sample.png --json",
        parameters: &[
            REGION_PARAMETER,
            CHANNELS_PARAMETER,
            EXPAND_PARAMETER,
            crate::operations::Parameter {
                name: "max_colors",
                description: "Maximum distinct tuples; exceeding the budget fails rather than returning approximate counts.",
                required: false,
                default: Some(crate::operations::ParameterValue::Unsigned(1000)),
                constraints: crate::operations::ParameterType::Unsigned { min: 1, max: 10000 },
            },
        ],
    },
    crate::operations::Descriptor {
        id: "image.compare",
        category: "image",
        description: "Compare exact image/channel samples, with bounded differences and an optional binary mask.",
        input_type: "image.sample_pair",
        output_type: "image.comparison",
        deterministic: true,
        reversible: None,
        command: "image compare",
        example: "cube image compare first.png second.png --json",
        parameters: &[
            REGION_PARAMETER,
            CHANNELS_PARAMETER,
            crate::operations::Parameter {
                name: "max_results",
                description: "Maximum difference records; total counts stay exact.",
                required: false,
                default: Some(crate::operations::ParameterValue::Unsigned(1000)),
                constraints: crate::operations::ParameterType::Unsigned { min: 1, max: 10000 },
            },
            crate::operations::Parameter {
                name: "mask",
                description: "Create an explicit 0/255 difference-mask image.",
                required: false,
                default: Some(crate::operations::ParameterValue::Boolean(false)),
                constraints: crate::operations::ParameterType::Boolean,
            },
        ],
    },
];
