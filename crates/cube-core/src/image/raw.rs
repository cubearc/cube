//! Explicit unsigned-byte raster reconstruction. No inferred geometry or padding.
use super::{ColorModel, Image, MAX_ENCODED, invalid, limit, sample_count};
use crate::{
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;

/// Physical input channel order. Output uses canonical gray/RGB/RGBA order.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SampleLayout {
    Gray,
    Rgb,
    Bgr,
    Rgba,
    Bgra,
}
impl SampleLayout {
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "gray" => Ok(Self::Gray),
            "rgb" => Ok(Self::Rgb),
            "bgr" => Ok(Self::Bgr),
            "rgba" => Ok(Self::Rgba),
            "bgra" => Ok(Self::Bgra),
            _ => Err(invalid("layout must be gray, rgb, bgr, rgba or bgra").parameter("layout")),
        }
    }
    fn model(self) -> ColorModel {
        match self {
            Self::Gray => ColorModel::Gray,
            Self::Rgb | Self::Bgr => ColorModel::Rgb,
            _ => ColorModel::Rgba,
        }
    }
}
/// Every row occupies `row_stride` bytes, including padding on the final row.
/// The input must end exactly at `offset + row_stride * height`. Prefix bytes
/// and explicitly declared row padding are ignored, never interpreted as samples.
#[derive(Clone, Debug)]
pub struct RawLayout {
    pub width: u32,
    pub height: u32,
    pub layout: SampleLayout,
    pub row_stride: Option<usize>,
    pub offset: usize,
}
#[derive(Debug, Serialize)]
pub struct RawInfo {
    pub width: u32,
    pub height: u32,
    pub layout: SampleLayout,
    pub color_model: ColorModel,
    pub bit_depth: u8,
    pub row_stride: usize,
    pub row_padding: usize,
    pub offset: usize,
    pub input_bytes: usize,
    pub required_bytes: usize,
    pub sample_bytes: usize,
}
impl RawLayout {
    /// Compare parsed header observations with an independently supplied raw
    /// hypothesis. Header failures remain explicit observations; they do not
    /// invalidate an otherwise exact raw layout or imply a repaired source.
    pub fn inspect_bytes(&self, data: &[u8]) -> Result<GeometryReport, OperationError> {
        let candidate = self
            .inspect(data.len())
            .map_err(|e| e.operation("image.geometry.inspect"))?;
        let (declared, header_error) = match super::decode::header_geometry(data) {
            Ok(v) => (v, None),
            Err(e) => (None, Some(e)),
        };
        Ok(GeometryReport {
            declared,
            header_error,
            candidate,
            interpretation: "caller_supplied_hypothesis",
        })
    }
    /// Validate shape and exact source extent before any sample allocation.
    pub fn inspect(&self, input_bytes: usize) -> Result<RawInfo, OperationError> {
        let result = (|| {
            if input_bytes > MAX_ENCODED {
                return Err(limit("raw input exceeds 32 MiB"));
            }
            let model = self.layout.model();
            let sample_bytes = sample_count(self.width, self.height, model.channels())?;
            let packed_row = self.width as usize * model.channels();
            let stride = self.row_stride.unwrap_or(packed_row);
            if stride < packed_row {
                return Err(
                    invalid("row_stride is shorter than one sample row").parameter("row_stride")
                );
            }
            let required = stride
                .checked_mul(self.height as usize)
                .and_then(|n| self.offset.checked_add(n))
                .ok_or_else(|| limit("raw layout extent overflow"))?;
            if required != input_bytes {
                return Err(invalid(
                    "raw input must end after exactly height complete rows, including final row padding; check offset, dimensions and stride",
                ));
            }
            Ok(RawInfo {
                width: self.width,
                height: self.height,
                layout: self.layout,
                color_model: model,
                bit_depth: 8,
                row_stride: stride,
                row_padding: stride - packed_row,
                offset: self.offset,
                input_bytes,
                required_bytes: required,
                sample_bytes,
            })
        })();
        result.map_err(|e: OperationError| e.operation("image.raw.interpret"))
    }
}
#[derive(Debug, Serialize)]
pub struct GeometryReport {
    pub declared: Option<super::decode::DeclaredGeometry>,
    pub header_error: Option<OperationError>,
    pub candidate: RawInfo,
    pub interpretation: &'static str,
}
impl Image {
    /// Reinterpret a validated raw layout without changing the source buffer.
    pub fn from_raw(data: &[u8], layout: &RawLayout) -> Result<Self, OperationError> {
        let info = layout.inspect(data.len())?;
        let channels = info.color_model.channels();
        let mut samples = Vec::with_capacity(info.sample_bytes);
        for row in data[layout.offset..].chunks_exact(info.row_stride) {
            for pixel in row[..layout.width as usize * channels].chunks_exact(channels) {
                if matches!(layout.layout, SampleLayout::Bgr | SampleLayout::Bgra) {
                    samples.extend_from_slice(&[pixel[2], pixel[1], pixel[0]]);
                    if channels == 4 {
                        samples.push(pixel[3]);
                    }
                } else {
                    samples.extend_from_slice(pixel);
                }
            }
        }
        let mut image = Self::from_samples(
            layout.width,
            layout.height,
            info.color_model,
            samples,
            vec![],
            vec![],
        )?;
        image.source_format = "raw";
        Ok(image)
    }
}
const RAW_PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "width",
        description: "Positive candidate width, not an inferred source dimension.",
        required: true,
        default: None,
        constraints: ParameterType::Unsigned {
            min: 1,
            max: super::MAX_DIMENSION as u64,
        },
    },
    Parameter {
        name: "height",
        description: "Positive candidate height; total pixels <= 4 Mi.",
        required: true,
        default: None,
        constraints: ParameterType::Unsigned {
            min: 1,
            max: super::MAX_DIMENSION as u64,
        },
    },
    Parameter {
        name: "layout",
        description: "Unsigned 8-bit physical channel order; export uses canonical order.",
        required: true,
        default: None,
        constraints: ParameterType::Text {
            choices: &["gray", "rgb", "bgr", "rgba", "bgra"],
        },
    },
    Parameter {
        name: "row_stride",
        description: "Full row byte count including padding on every row; omitted means packed.",
        required: false,
        default: None,
        constraints: ParameterType::Unsigned {
            min: 1,
            max: MAX_ENCODED as u64,
        },
    },
    Parameter {
        name: "offset",
        description: "Explicit prefix byte count; remaining data must contain exactly complete rows.",
        required: false,
        default: Some(ParameterValue::Unsigned(0)),
        constraints: ParameterType::Unsigned {
            min: 0,
            max: MAX_ENCODED as u64,
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "image.raw.interpret",
        category: "image",
        description: "Interpret exact unsigned 8-bit raw layouts, with explicit channel order, stride and offset; never pad or truncate.",
        input_type: "bytes",
        output_type: "image.samples",
        deterministic: true,
        reversible: Some(false),
        command: "image raw",
        example: "cube image raw pixels.bin --width 2 --height 1 --layout rgb --output reconstructed.png",
        parameters: RAW_PARAMETERS,
    },
    Descriptor {
        id: "image.geometry.inspect",
        category: "image",
        description: "Compare declared header dimensions with an explicit checked raw-layout hypothesis, without repairing or decoding the source.",
        input_type: "bytes",
        output_type: "image.geometry",
        parameters: RAW_PARAMETERS,
        deterministic: true,
        reversible: None,
        command: "image raw",
        example: "cube image raw pixels.bin --width 2 --height 1 --layout rgba --compare-header",
    },
];
