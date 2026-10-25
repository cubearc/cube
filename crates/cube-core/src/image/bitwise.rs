//! Exact image-pair adapters over the existing byte transformations.

use super::{Image, invalid, malformed, sample_count};
use crate::{
    binary,
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType},
};
use serde::Serialize;

const ID: &str = "image.bitwise";

/// Per-sample operation, including physical alpha and native palette indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BitwiseOperation {
    Xor,
    And,
    Or,
}

impl BitwiseOperation {
    /// Parse one exact lowercase operation name; no default is implied.
    pub fn parse(name: &str) -> Result<Self, OperationError> {
        match name {
            "xor" => Ok(Self::Xor),
            "and" => Ok(Self::And),
            "or" => Ok(Self::Or),
            _ => Err(invalid("operation must be one of: xor, and, or")
                .operation(ID)
                .parameter("operation")),
        }
    }
}

impl Image {
    /// Combine equal 8-bit rasters using the existing byte XOR/AND/OR functions.
    ///
    /// Dimensions, model, sample length, palette and raw tRNS must match exactly.
    /// All physical samples participate, including alpha. Indexed samples combine
    /// as indices; an invalid resulting index is an error. Matching tRNS keys are
    /// retained unchanged, so newly matching output colors become transparent.
    /// Expand palettes explicitly beforehand when color samples are intended.
    ///
    /// Both inputs remain unchanged. The new image retains compatible palette/tRNS
    /// and drops other source metadata, with constructed-sample provenance. There
    /// is no resizing, blending, premultiplication or implicit conversion.
    /// Existing per-image limits apply: 16384 per side, 4 Mi pixels, 16 MiB samples;
    /// the two input buffers plus the result total at most 48 MiB of sample data.
    pub fn bitwise(
        &self,
        other: &Self,
        operation: BitwiseOperation,
    ) -> Result<Self, OperationError> {
        let result = (|| {
            let count = sample_count(self.width, self.height, self.model.channels())?;
            sample_count(other.width, other.height, other.model.channels())?;
            if self.width != other.width || self.height != other.height || self.model != other.model
            {
                return Err(
                    invalid("image pair must have identical dimensions and sample models")
                        .parameter("other"),
                );
            }
            // XOR's public byte API cycles its key; image operands must never cycle.
            if self.samples.len() != count || other.samples.len() != count {
                return Err(
                    invalid("image pair must have equal, complete sample buffers")
                        .parameter("other"),
                );
            }
            if self.palette != other.palette || self.transparency != other.transparency {
                return Err(invalid("image pair must have identical palettes and raw transparency; expand indexed palettes explicitly if needed")
                    .parameter("other"));
            }
            let samples = match operation {
                BitwiseOperation::Xor => binary::xor(&self.samples, &other.samples)
                    .map_err(|e| malformed("XOR requires nonempty image samples").caused_by(e))?,
                BitwiseOperation::And => binary::transform::and(&self.samples, &other.samples)?,
                BitwiseOperation::Or => binary::transform::or(&self.samples, &other.samples)?,
            };
            Self::from_samples(
                self.width,
                self.height,
                self.model,
                samples,
                self.palette.clone(),
                self.transparency.clone(),
            )
        })();
        result.map_err(|e: OperationError| e.operation(ID))
    }
}

/// Direct image-pair support does not imply pipeline execution support.
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: ID,
    category: "image",
    description: "Combine exact 8-bit image samples with XOR/AND/OR, including alpha or native indices; identical geometry/model/palette/tRNS required; <=4 Mi pixels and 16 MiB samples per image. Other source metadata is dropped.",
    input_type: "image.sample_pair",
    output_type: "image.samples",
    parameters: &[Parameter {
        name: "operation",
        description: "Exact byte operation on equally long corresponding samples; no cycling, padding or blending.",
        required: true,
        default: None,
        constraints: ParameterType::Text {
            choices: &["xor", "and", "or"],
        },
    }],
    deterministic: true,
    reversible: None,
    command: "image bitwise",
    example: "cube image bitwise first.png second.png --operation xor --output combined.png",
}];
