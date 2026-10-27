//! Shared deterministic bit packing contracts for binary/image/audio extraction.
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
}
/// First extracted bit occupies bit 7 (MSB) or bit 0 (LSB) of an output byte.
#[derive(Clone, Copy, Debug, Default, Serialize, schemars::JsonSchema)]
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
#[derive(Clone, Copy, Debug, Default, Serialize, schemars::JsonSchema)]
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

pub(crate) fn store(bytes: &mut [u8], position: usize, bit: u8, order: BitOrder) {
    let shift = match order {
        BitOrder::Msb => 7 - position % 8,
        BitOrder::Lsb => position % 8,
    };
    bytes[position / 8] |= bit << shift;
}
