//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use cube_core::{
    errors::{ErrorCode, OperationError},
    image::{Channel, ColorModel, Corner, Image, Region, Traversal},
    operations::contracts::values,
};
use serde_json::{Value, json};

pub(super) fn supports(operation: &str) -> bool {
    matches!(
        operation,
        "image.decode"
            | "image.encode"
            | "image.palette.expand"
            | "image.pixels.read"
            | "image.histogram"
            | "image.pixels.export"
            | "image.channel.extract"
            | "image.compare"
            | "image.invert"
            | "image.rotate"
            | "image.mirror"
            | "image.grayscale"
            | "image.crop"
            | "image.reorder"
    )
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use the exact logical raster fields and parameters",
    )
}
pub(super) fn raster(value: &Value) -> Result<Image, OperationError> {
    let model: String = super::field(value, "color_model")?;
    Image::from_samples(
        super::field(value, "width")?,
        super::field(value, "height")?,
        ColorModel::parse(&model)?,
        super::bytes(&value["samples"])?,
        super::field(value, "palette")?,
        super::bytes(&value["transparency"])?,
    )
}
pub(super) fn image(value: &Image) -> Value {
    json!(values::Image::from(value))
}
pub(super) fn bytes(value: &[u8]) -> Value {
    json!({"base64":cube_core::encoding::base64_encode(value)})
}
pub(super) fn region(p: &Value) -> Result<Option<Region>, OperationError> {
    let Some(value) = p.get("region") else {
        return Ok(None);
    };
    let [x, y, width, height]: [u32; 4] =
        serde_json::from_value(value.clone()).map_err(|_| invalid())?;
    Ok(Some(Region {
        x,
        y,
        width,
        height,
    }))
}
pub(super) fn channels(p: &Value) -> Result<Option<Vec<Channel>>, OperationError> {
    p.get("channels")
        .map(|value| {
            let names: Vec<String> =
                serde_json::from_value(value.clone()).map_err(|_| invalid())?;
            names.iter().map(|name| Channel::parse(name)).collect()
        })
        .transpose()
}
pub(super) fn execute(op: &str, inputs: Value, p: Value) -> Result<Value, OperationError> {
    if op == "image.decode" {
        return Ok(
            json!({"output":image(&Image::decode_with_sample_limit(&super::bytes(&inputs["input"] )?, super::super::MAX_CODEC_INPUT)?)}),
        );
    }
    let source = raster(
        &inputs[if op == "image.compare" {
            "left"
        } else {
            "input"
        }],
    )?;
    let expand = p
        .get("expand_palette")
        .map(|_| super::field::<bool>(&p, "expand_palette"))
        .transpose()?
        .unwrap_or(false);
    let source = source.view(expand)?;
    let channels = channels(&p)?;
    let channels = channels.as_deref();
    let region = region(&p)?;
    let output = match op {
        "image.encode" => bytes(&source.encode_png()?),
        "image.palette.expand" => image(&source.expand_palette()?),
        "image.pixels.read" => {
            bytes(&source.pixel(super::field(&p, "x")?, super::field(&p, "y")?)?)
        }
        "image.histogram" => {
            json!(source.histogram(region, channels, super::field(&p, "max_colors")?)?)
        }
        "image.pixels.export" => {
            let traversal: String = super::field(&p, "traversal")?;
            let corner: String = super::field(&p, "start_corner")?;
            let pixels = source.pixels_with_traversal(
                region,
                channels,
                Traversal::parse(&traversal)?,
                Corner::parse(&corner)?,
                super::field(&p, "reverse")?,
            )?;
            return Ok(json!({"output":pixels,"data":bytes(&pixels.pixels.data)}));
        }
        "image.channel.extract" => {
            let channel: String = super::field(&p, "channel")?;
            let pixels = source.channel(Channel::parse(&channel)?, region)?;
            return Ok(
                json!({"output":pixels,"data":bytes(&pixels.data),"image":image(&pixels.grayscale_image()?)}),
            );
        }
        "image.compare" => {
            let other = raster(&inputs["right"])?;
            let comparison = source.compare(
                &other,
                region,
                channels,
                super::field(&p, "max_results")?,
                super::field(&p, "mask")?,
            )?;
            let mut output = json!({"output":comparison});
            if let Some(mask) = &comparison.mask {
                output["mask"] = image(mask);
            }
            return Ok(output);
        }
        "image.rotate" => image(&source.rotate(super::field(&p, "degrees")?)?),
        "image.mirror" => {
            let axis: String = super::field(&p, "axis")?;
            image(&source.mirror(&axis)?)
        }
        "image.crop" => image(&source.crop(region.ok_or_else(invalid)?)?),
        "image.invert" => image(&source.invert(channels)?),
        "image.reorder" => image(&source.reorder(channels.ok_or_else(invalid)?)?),
        "image.grayscale" => image(&source.grayscale()?),
        _ => return Err(invalid()),
    };
    Ok(json!({"output":output}))
}
