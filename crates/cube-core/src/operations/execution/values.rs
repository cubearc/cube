use super::{Budget, Failure, Representation, Result};
use crate::operations::contracts::PortContract;
use serde_json::Value;
pub(crate) fn text<'a>(v: &'a Value, name: &str) -> Result<&'a str> {
    v[name].as_str().ok_or(Failure("contract_mismatch"))
}
pub(crate) fn bytes(v: &Value) -> Result<Vec<u8>> {
    crate::encoding::base64_decode(v["base64"].as_str().ok_or(Failure("contract_mismatch"))?)
        .map_err(|_| Failure("invalid_bytes"))
}
pub(crate) fn image(v: &Value) -> Result<crate::image::Image> {
    let number = |name| {
        v[name]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(Failure("invalid_image"))
    };
    crate::image::Image::from_samples(
        number("width")?,
        number("height")?,
        crate::image::ColorModel::parse(v["color_model"].as_str().ok_or(Failure("invalid_image"))?)
            .map_err(|_| Failure("invalid_image"))?,
        bytes(&v["samples"])?,
        serde_json::from_value(v["palette"].clone()).map_err(|_| Failure("invalid_image"))?,
        bytes(&v["transparency"])?,
    )
    .map_err(|_| Failure("invalid_image"))
}
pub(super) fn text_size(value: &Value) -> Result<usize> {
    match value {
        Value::String(s) => Ok(s.len()),
        Value::Array(a) => a.iter().try_fold(0usize, |n, v| {
            n.checked_add(text_size(v)?).ok_or(Failure("input_limit"))
        }),
        Value::Object(o) => o.values().try_fold(0usize, |n, v| {
            n.checked_add(text_size(v)?).ok_or(Failure("input_limit"))
        }),
        _ => Ok(0),
    }
}
pub(super) fn size(
    ports: &[PortContract],
    reps: &[Representation],
    values: &Value,
    budget: Budget,
    include_images: bool,
) -> Result<usize> {
    ports.iter().zip(reps).try_fold(0usize, |sum, (p, r)| {
        let v = values.get(&p.name).ok_or(Failure("contract_mismatch"))?;
        let size = match r {
            Representation::Text => v.as_str().ok_or(Failure("contract_mismatch"))?.len(),
            Representation::Bytes => bytes(v)?.len(),
            Representation::ByteArray => v.as_array().ok_or(Failure("contract_mismatch"))?.len(),
            Representation::Json => serde_json::to_vec(v)
                .map_err(|_| Failure("contract_mismatch"))?
                .len(),
            Representation::ImageSamples => {
                let length = bytes(&v["samples"])?.len();
                if length > budget.image_sample_bytes {
                    return Err(Failure("input_limit"));
                }
                if include_images { length } else { 0 }
            }
        };
        sum.checked_add(size).ok_or(Failure("input_limit"))
    })
}
