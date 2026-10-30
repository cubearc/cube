use crate::operations::execution::{self as e, Entry, Failure, Representation::*, Result};
use serde_json::{Value, json};
fn raster(i: &Value, p: &Value) -> Result<super::Image> {
    e::image(&i["input"])?
        .view(
            p["expand_palette"]
                .as_bool()
                .ok_or(Failure("invalid_parameters"))?,
        )
        .map(|v| v.into_owned())
        .map_err(|_| Failure("invalid_image"))
}
fn pixel(i: &Value, p: &Value, _: e::Budget) -> Result<Value> {
    let coordinate = |k| {
        p[k].as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(Failure("invalid_parameters"))
    };
    let data = raster(i, p)?
        .pixel(coordinate("x")?, coordinate("y")?)
        .map_err(|_| Failure("invalid_parameters"))?;
    Ok(json!({"output":e::byte_value(data)}))
}
fn pixel_defaults(p: &mut Value, _: e::Budget) {
    p["x"] = json!(0);
    p["y"] = json!(0);
}
fn pixel_preflight(inputs: &Value, parameters: &Value, b: e::Budget) -> Result<()> {
    pixel(inputs, parameters, b).map(|_| ())
}
fn export(i: &Value, p: &Value, b: e::Budget) -> Result<Value> {
    let image = raster(i, p)?;
    let region = match p.get("region") {
        None => None,
        Some(v) => {
            let [x, y, width, height]: [u32; 4] =
                serde_json::from_value(v.clone()).map_err(|_| Failure("invalid_parameters"))?;
            Some(super::Region {
                x,
                y,
                width,
                height,
            })
        }
    };
    let channels = p
        .get("channels")
        .map(|v| {
            v.as_array()
                .ok_or(Failure("invalid_parameters"))?
                .iter()
                .map(|v| {
                    super::Channel::parse(v.as_str().ok_or(Failure("invalid_parameters"))?)
                        .map_err(|_| Failure("invalid_parameters"))
                })
                .collect::<Result<Vec<_>>>()
        })
        .transpose()?;
    let selected = region
        .unwrap_or_else(|| super::Region::full(&image))
        .validate(&image)
        .map_err(|_| Failure("invalid_parameters"))?;
    let count = u64::from(selected.width)
        * u64::from(selected.height)
        * channels
            .as_ref()
            .map_or(image.info().color_model.channels(), Vec::len) as u64;
    // Reject before allocating the traversal/output; the final metadata is charged too.
    if count > b.output_bytes as u64 {
        return Err(Failure("output_limit"));
    }
    let output = image
        .pixels_with_traversal(
            region,
            channels.as_deref(),
            super::Traversal::parse(e::text(p, "traversal")?)
                .map_err(|_| Failure("invalid_parameters"))?,
            super::Corner::parse(e::text(p, "start_corner")?)
                .map_err(|_| Failure("invalid_parameters"))?,
            p["reverse"]
                .as_bool()
                .ok_or(Failure("invalid_parameters"))?,
        )
        .map_err(|_| Failure("invalid_parameters"))?;
    Ok(json!({"data":e::byte_value(output.pixels.data.clone()),"output":output}))
}
pub(crate) const ENTRIES: &[Entry] = &[
    Entry {
        defaults: Some(pixel_defaults),
        preflight: Some(pixel_preflight),
        ..Entry::pure("image.pixels.read", &[ImageSamples], &[Bytes], pixel)
    },
    Entry::pure(
        "image.pixels.export",
        &[ImageSamples],
        &[Json, Bytes],
        export,
    ),
];
