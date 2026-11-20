//! Strict RFC 4648 extended-hex Base32 and URL-safe Base64.
use crate::{
    errors::OperationError,
    formal_work::{invalid, resource},
};
use base64::{
    Engine as _,
    engine::general_purpose::{GeneralPurpose, URL_SAFE, URL_SAFE_NO_PAD},
};
use schemars::JsonSchema;
use serde::Deserialize;

pub(crate) const BYTE_LIMIT: usize = 1 << 20;
pub(crate) const TEXT_LIMIT: usize = 2 << 20;

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Padding {
    #[default]
    Required,
    Forbidden,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Base64Url {
    #[serde(default)]
    pub padding: Padding,
}

impl Padding {
    fn engine(self) -> &'static GeneralPurpose {
        match self {
            Self::Required => &URL_SAFE,
            Self::Forbidden => &URL_SAFE_NO_PAD,
        }
    }
}

fn check_bytes(data: &[u8]) -> Result<(), OperationError> {
    if data.len() > BYTE_LIMIT {
        return Err(resource("RFC 4648 byte input exceeds 1 MiB"));
    }
    Ok(())
}

fn check_text(input: &str, bits_per_symbol: usize) -> Result<(), OperationError> {
    if input.len() > TEXT_LIMIT {
        return Err(resource("RFC 4648 text input exceeds 2 MiB"));
    }
    if input.trim_end_matches('=').len() * bits_per_symbol / 8 > BYTE_LIMIT {
        return Err(resource("decoded RFC 4648 output exceeds 1 MiB"));
    }
    Ok(())
}

/// Encode canonical uppercase Base32hex with padding.
pub fn encode_base32hex(data: &[u8]) -> Result<String, OperationError> {
    check_bytes(data)?;
    Ok(data_encoding::BASE32HEX.encode(data))
}

/// Decode canonical uppercase Base32hex with padding.
pub fn decode_base32hex(input: &str) -> Result<Vec<u8>, OperationError> {
    check_text(input, 5)?;
    let output = super::strict_base32(input, &data_encoding::BASE32HEX)
        .map_err(|_| invalid("invalid canonical padded Base32hex"))?;
    check_bytes(&output)?;
    Ok(output)
}

/// Encode URL-safe Base64 with the selected padding policy.
pub fn encode_base64url(data: &[u8], options: Base64Url) -> Result<String, OperationError> {
    check_bytes(data)?;
    Ok(options.padding.engine().encode(data))
}

/// Decode canonical URL-safe Base64 with the selected padding policy.
pub fn decode_base64url(input: &str, options: Base64Url) -> Result<Vec<u8>, OperationError> {
    check_text(input, 6)?;
    let output = options
        .padding
        .engine()
        .decode(input)
        .map_err(|_| invalid("invalid canonical Base64url for the selected padding policy"))?;
    check_bytes(&output)?;
    Ok(output)
}
