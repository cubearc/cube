//! Deterministic sample bits and packed extraction. No message inference or search.
pub use super::traversal::Traversal;
use super::{Channel, ColorModel, Image, PixelData, Region, invalid};
use crate::errors::OperationError;
use serde::Serialize;
/// First extracted bit occupies bit 7 (MSB) or bit 0 (LSB) of an output byte.
#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BitOrder {
    #[default]
    Msb,
    Lsb,
}
impl BitOrder {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "msb" => Ok(Self::Msb),
            "lsb" => Ok(Self::Lsb),
            _ => Err(invalid("bit_order must be msb or lsb").parameter("bit_order")),
        }
    }
}
/// Policy for a final incomplete byte. Pad fills unused packing positions with zero.
#[derive(Clone, Copy, Debug, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Incomplete {
    #[default]
    Reject,
    Drop,
    Pad,
}
impl Incomplete {
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        match value {
            "reject" => Ok(Self::Reject),
            "drop" => Ok(Self::Drop),
            "pad" => Ok(Self::Pad),
            _ => Err(invalid("incomplete must be reject, drop or pad").parameter("incomplete")),
        }
    }
}

/// Explicit extraction parameters. Default channels mean native sample order,
/// including alpha or raw palette indices; bits default to `[0]`.
#[derive(Clone, Debug)]
pub struct LsbOptions {
    pub region: Option<Region>,
    pub channels: Option<Vec<Channel>>,
    pub bits: Vec<u8>,
    pub traversal: Traversal,
    pub reverse: bool,
    pub bit_order: BitOrder,
    pub incomplete: Incomplete,
}
impl Default for LsbOptions {
    fn default() -> Self {
        Self {
            region: None,
            channels: None,
            bits: vec![0],
            traversal: Traversal::Row,
            reverse: false,
            bit_order: BitOrder::Msb,
            incomplete: Incomplete::Reject,
        }
    }
}
/// Packed raw bytes plus effective parameters and explicit partial-byte accounting.
#[derive(Debug, Serialize)]
pub struct LsbResult {
    pub region: Region,
    pub channels: Vec<Channel>,
    pub bits: Vec<u8>,
    pub traversal: Traversal,
    pub reverse: bool,
    pub bit_order: BitOrder,
    pub incomplete: Incomplete,
    pub available_bits: usize,
    pub valid_bits: usize,
    pub dropped_bits: usize,
    #[serde(skip)]
    pub data: Vec<u8>,
}
/// Extended traversal result preserves the original public LsbResult layout.
#[derive(Debug, Serialize)]
pub struct OrderedLsbResult {
    #[serde(flatten)]
    pub result: LsbResult,
    pub start_corner: super::Corner,
}
/// Exact row-major 0/1 samples and MSB-first zero-padded bytes.
#[derive(Debug, Serialize)]
pub struct BitPlane {
    pub pixels: PixelData,
    pub bit: u8,
    pub valid_bits: usize,
    #[serde(skip)]
    pub packed: Vec<u8>,
}
impl BitPlane {
    /// Explicit display representation: zero -> 0, one -> 255; raw values unchanged.
    pub fn visualization(&self) -> Result<Image, OperationError> {
        if self.pixels.data.iter().any(|v| *v > 1) {
            return Err(invalid("bit plane values must be 0 or 1"));
        }
        Image::from_samples(
            self.pixels.region.width,
            self.pixels.region.height,
            ColorModel::Gray,
            self.pixels.data.iter().map(|v| v * 255).collect(),
            vec![],
            vec![],
        )
    }
}
fn store(bytes: &mut [u8], position: usize, bit: u8, order: BitOrder) {
    let shift = match order {
        BitOrder::Msb => 7 - position % 8,
        BitOrder::Lsb => position % 8,
    };
    bytes[position / 8] |= bit << shift;
}
impl Image {
    /// Read one sample bit (0=LSB) as row-major 0/1 values. Packing is MSB-first
    /// with zero padding; valid_bits distinguishes the padding from sample data.
    pub fn bit_plane(
        &self,
        channel: Channel,
        bit: u8,
        region: Option<Region>,
    ) -> Result<BitPlane, OperationError> {
        let result = (|| {
            if bit > 7 {
                return Err(invalid("bit must be in 0..=7 for PNG8").parameter("bit"));
            }
            let mut pixels = self.channel(channel, region)?;
            for value in &mut pixels.data {
                *value = (*value >> bit) & 1;
            }
            pixels.bit_depth = 1;
            let valid_bits = pixels.data.len();
            let mut packed = vec![0; valid_bits.div_ceil(8)];
            for (i, v) in pixels.data.iter().enumerate() {
                store(&mut packed, i, *v, BitOrder::Msb);
            }
            Ok(BitPlane {
                pixels,
                bit,
                valid_bits,
                packed,
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.bit_plane.extract"))
    }
    /// Extract bits nested as pixel, requested channel, requested bit. Reverse
    /// changes only coordinate order. This returns bytes without text conversion.
    /// Image sample limits also bound output to at most 16 MiB; no per-bit buffer.
    pub fn lsb(&self, options: &LsbOptions) -> Result<LsbResult, OperationError> {
        self.lsb_with_corner(options, super::Corner::TopLeft)
            .map(|v| v.result)
    }
    /// Extended coordinate traversal with an explicit starting corner.
    pub fn lsb_with_corner(
        &self,
        options: &LsbOptions,
        start_corner: super::Corner,
    ) -> Result<OrderedLsbResult, OperationError> {
        let result = (|| {
            let region = options
                .region
                .unwrap_or_else(|| Region::full(self))
                .validate(self)?;
            let (channels, indices) = self.selection(options.channels.as_deref())?;
            if options.bits.is_empty()
                || options.bits.len() > 8
                || options
                    .bits
                    .iter()
                    .enumerate()
                    .any(|(i, v)| *v > 7 || options.bits[..i].contains(v))
            {
                return Err(
                    invalid("bits must contain 1..=8 distinct positions in 0..=7")
                        .parameter("bits"),
                );
            }
            let pixels = region.width as usize * region.height as usize;
            let available_bits = pixels * indices.len() * options.bits.len();
            let remainder = available_bits % 8;
            if remainder != 0 && matches!(options.incomplete, Incomplete::Reject) {
                return Err(invalid(
                    "extraction ends with an incomplete byte; explicitly choose drop or pad",
                )
                .parameter("incomplete"));
            }
            let dropped_bits = if matches!(options.incomplete, Incomplete::Drop) {
                remainder
            } else {
                0
            };
            let valid_bits = available_bits - dropped_bits;
            let mut data = vec![0; valid_bits.div_ceil(8)];
            let mut position = 0;
            for (x, y) in options
                .traversal
                .coordinates(region, start_corner, options.reverse)?
            {
                let source = self.position(x, y)?;
                for channel in &indices {
                    for bit in &options.bits {
                        if position < valid_bits {
                            store(
                                &mut data,
                                position,
                                (self.samples[source + channel] >> bit) & 1,
                                options.bit_order,
                            );
                        }
                        position += 1;
                    }
                }
            }
            Ok(OrderedLsbResult {
                start_corner,
                result: LsbResult {
                    region,
                    channels,
                    bits: options.bits.clone(),
                    traversal: options.traversal,
                    reverse: options.reverse,
                    bit_order: options.bit_order,
                    incomplete: options.incomplete,
                    available_bits,
                    valid_bits,
                    dropped_bits,
                    data,
                },
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.lsb.extract"))
    }
}

use super::analyze::{CHANNEL_NAMES, CHANNELS_PARAMETER, EXPAND_PARAMETER, REGION_PARAMETER};
use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
/// Discovery metadata lives alongside bit semantics.
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "image.bit_plane.extract",
        category: "image",
        description: "Extract one row-major sample bit as 0/1 values and MSB-first zero-padded bytes.",
        input_type: "image.samples",
        output_type: "image.bit_plane",
        deterministic: true,
        reversible: Some(false),
        command: "image bit-plane",
        example: "cube image bit-plane sample.png --channel red --bit 0 --output bits.bin",
        parameters: &[
            Parameter {
                name: "channel",
                description: "One available physical sample channel; indexed colors require explicit expansion.",
                required: true,
                default: None,
                constraints: ParameterType::Text {
                    choices: CHANNEL_NAMES,
                },
            },
            Parameter {
                name: "bit",
                description: "Sample bit index, zero denotes least significant bit.",
                required: true,
                default: None,
                constraints: ParameterType::Unsigned { min: 0, max: 7 },
            },
            REGION_PARAMETER,
            EXPAND_PARAMETER,
        ],
    },
    Descriptor {
        id: "image.lsb.extract",
        category: "image",
        description: "Extract raw bytes under explicit pixel/channel/bit ordering and incomplete-byte policy.",
        input_type: "image.samples",
        output_type: "image.bit_stream",
        deterministic: true,
        reversible: Some(false),
        command: "image lsb",
        example: "cube image lsb sample.png --channels red --bits 0 --bit-order msb --stdout",
        parameters: &[
            REGION_PARAMETER,
            CHANNELS_PARAMETER,
            EXPAND_PARAMETER,
            Parameter {
                name: "bits",
                description: "Ordered distinct sample positions, zero=LSB; nested after channel order.",
                required: false,
                default: Some(ParameterValue::UnsignedList(&[0])),
                constraints: ParameterType::UnsignedList {
                    min: 0,
                    max: 7,
                    min_length: 1,
                    max_length: 8,
                    unique: true,
                },
            },
            super::traversal::TRAVERSAL_PARAMETER,
            super::traversal::CORNER_PARAMETER,
            super::traversal::REVERSE_PARAMETER,
            Parameter {
                name: "bit_order",
                description: "First bit occupies output bit 7 (msb) or bit 0 (lsb).",
                required: false,
                default: Some(ParameterValue::Text("msb")),
                constraints: ParameterType::Text {
                    choices: &["msb", "lsb"],
                },
            },
            Parameter {
                name: "incomplete",
                description: "Reject, drop trailing bits or zero-pad final byte; return valid/dropped-bit counts.",
                required: false,
                default: Some(ParameterValue::Text("reject")),
                constraints: ParameterType::Text {
                    choices: &["reject", "drop", "pad"],
                },
            },
        ],
    },
];
