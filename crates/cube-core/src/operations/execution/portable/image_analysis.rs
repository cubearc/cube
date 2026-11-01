//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use super::{
    field,
    images::{bytes, channels, image, raster, region},
};
use crate::{
    errors::{ErrorCode, OperationError},
    image::{self, Image},
};
use serde_json::{Value, json};
use std::io::Cursor;

enum Adapter {
    Plane,
    Bitwise,
    Colors,
    ColorExports,
    Qr,
    CsvExport,
    CsvImport,
    Geometry,
    Lsb,
    Pairs,
    Png,
    Raw,
    Spatial,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "image.bit_plane.extract" => Adapter::Plane,
        "image.bitwise" => Adapter::Bitwise,
        "image.colors.extract" => Adapter::Colors,
        "image.colors.export" => Adapter::ColorExports,
        "image.qr.decode" => Adapter::Qr,
        "image.csv.export" => Adapter::CsvExport,
        "image.csv.import" => Adapter::CsvImport,
        "image.geometry.inspect" => Adapter::Geometry,
        "image.lsb.extract" => Adapter::Lsb,
        "image.pairs" => Adapter::Pairs,
        "image.png.inspect" => Adapter::Png,
        "image.raw.interpret" => Adapter::Raw,
        "image.spatial.autocorrelation" => Adapter::Spatial,
        _ => return None,
    })
}
pub(super) fn supports(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use exact logical image inputs and parameters",
    )
}
fn text<'a>(p: &'a Value, key: &str) -> Result<&'a str, OperationError> {
    p[key].as_str().ok_or_else(invalid)
}
fn layout(p: &Value) -> Result<image::RawLayout, OperationError> {
    Ok(image::RawLayout {
        width: field(p, "width")?,
        height: field(p, "height")?,
        layout: image::SampleLayout::parse(text(p, "layout")?)?,
        row_stride: p
            .get("row_stride")
            .map(|_| field(p, "row_stride"))
            .transpose()?,
        offset: field(p, "offset")?,
    })
}
pub(super) fn execute(
    op: &str,
    inputs: Value,
    p: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let identity = crate::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| {
        match selected {
            Adapter::CsvImport => {
                return Ok(json!({"output":image(&Image::from_csv(text(&inputs,"input")?)?)}));
            }
            Adapter::Geometry | Adapter::Raw => {
                let data = super::bytes(&inputs["input"])?;
                let layout = layout(&p)?;
                return Ok(match selected {
                    Adapter::Geometry => json!({"output":layout.inspect_bytes(&data)?}),
                    _ => json!({"output":image(&Image::from_raw(&data,&layout)?)}),
                });
            }
            Adapter::Png => {
                return Ok(
                    json!({"output":crate::forensics::png::inspect(&mut Cursor::new(super::bytes(&inputs["input"])?),field(&p,"strict")?,field(&p,"payload_budget")?)?}),
                );
            }
            Adapter::Pairs => {
                let values = inputs["images"].as_array().ok_or_else(invalid)?;
                let images: Vec<Image> = values.iter().map(raster).collect::<Result<_, _>>()?;
                let refs: Vec<&Image> = images.iter().collect();
                let operation = image::pairs::PairOperation::parse(text(&p, "operation")?)?;
                let max_pairs = field(&p, "max_pairs")?;
                let max_differences = field(&p, "max_differences")?;
                let batch = if field(&p, "plan_only")? {
                    image::pairs::Batch {
                        plan: image::pairs::plan(&refs, operation, max_pairs, max_differences)?,
                        successes: 0,
                        failures: 0,
                        encoded_bytes: 0,
                        retained_differences: 0,
                        results: vec![],
                    }
                } else {
                    image::pairs::execute(&refs, operation, max_pairs, max_differences)?
                };
                let outputs: Vec<Value> = batch
                    .results
                    .iter()
                    .map(|r| r.png.as_ref().map(|png| bytes(png)).unwrap_or(Value::Null))
                    .collect();
                return Ok(json!({"output":batch,"images":outputs}));
            }
            _ => {}
        }
        let source = raster(
            &inputs[match selected {
                Adapter::Bitwise => "left",
                Adapter::Spatial => "image",
                _ => "input",
            }],
        )?;
        let expand = p
            .get("expand_palette")
            .map(|_| field(&p, "expand_palette"))
            .transpose()?
            .unwrap_or(false);
        let source = source.view(expand)?;
        Ok(match selected {
            Adapter::Bitwise => {
                json!({"output":image(&source.bitwise(&raster(&inputs["right"])?,image::BitwiseOperation::parse(text(&p,"operation")?)?)?)})
            }
            Adapter::CsvExport => json!({"output":source.to_csv()?}),
            Adapter::Qr => {
                let report = source.qr(field(&p, "max_symbols")?)?;
                let payloads: Vec<Value> = report
                    .symbols
                    .iter()
                    .map(|s| s.payload.as_ref().map(|p| bytes(p)).unwrap_or(Value::Null))
                    .collect();
                json!({"output":report,"payloads":payloads})
            }
            Adapter::ColorExports => {
                let colors = p
                    .get("colors")
                    .map(|_| field::<Vec<String>>(&p, "colors"))
                    .transpose()?;
                let colors = colors
                    .map(|v| {
                        v.iter()
                            .map(|s| {
                                crate::encoding::hex_decode(s).map_err(|e| invalid().caused_by(e))
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                    .transpose()?;
                let result = source.color_layers(
                    colors.as_deref(),
                    image::ColorOutput::parse(text(&p, "representation")?)?,
                    field(&p, "max_colors")?,
                    field(&p, "max_outputs")?,
                )?;
                let outputs: Vec<Value> = result.outputs.iter().map(|v| bytes(&v.png)).collect();
                json!({"output":result,"images":outputs})
            }
            Adapter::Colors => {
                let colors: Vec<String> = field(&p, "colors")?;
                let colors = colors
                    .iter()
                    .map(|s| crate::encoding::hex_decode(s).map_err(|e| invalid().caused_by(e)))
                    .collect::<Result<Vec<_>, _>>()?;
                let layer = source.exact_colors(
                    &colors,
                    image::ColorOutput::parse(text(&p, "representation")?)?,
                )?;
                json!({"output":layer,"image":image(&layer.image)})
            }
            Adapter::Plane => {
                let plane = source.bit_plane(
                    image::Channel::parse(text(&p, "channel")?)?,
                    field(&p, "bit")?,
                    region(&p)?,
                )?;
                json!({"output":plane,"data":bytes(&plane.packed)})
            }
            Adapter::Lsb => {
                let result = source.lsb_with_corner(
                    &image::LsbOptions {
                        region: region(&p)?,
                        channels: channels(&p)?,
                        bits: field(&p, "bits")?,
                        traversal: image::Traversal::parse(text(&p, "traversal")?)?,
                        reverse: field(&p, "reverse")?,
                        bit_order: image::BitOrder::parse(text(&p, "bit_order")?)?,
                        incomplete: image::Incomplete::parse(text(&p, "incomplete")?)?,
                    },
                    image::Corner::parse(text(&p, "start_corner")?)?,
                )?;
                json!({"output":result,"data":bytes(&result.result.data)})
            }
            Adapter::Spatial => {
                let lags = image::spatial::parse_lags(text(&inputs, "lags")?)?;
                json!({"output":source.autocorrelation(text(&p,"region")?.parse()?,image::Channel::parse(text(&p,"channel")?)?,&lags,image::spatial::Limits { max_work:field(&p,"max_work")?, max_memory_bytes:field(&p,"max_memory_bytes")?, max_report_bytes:field(&p,"max_report_bytes")? })?})
            }
            _ => return Err(invalid()),
        })
    })();
    result.map_err(|e: OperationError| e.operation(identity))
}
