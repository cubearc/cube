//! Named-port adapters for bounded image profiles.
use super::{
    bytes, field,
    images::{self, image, raster},
};
use crate::{
    errors::{ErrorCode, OperationError},
    image::{
        advanced_catalog as spec, barcode, binary_grid, colorimetry, edges, filters, matching,
        native_samples, optical, perceptual, qr_encode, quality, resample, stereo, traversal,
        voting,
    },
};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

pub(super) fn supports(id: &str) -> bool {
    spec::OPERATIONS.iter().any(|row| row.id == id)
}

fn options<T: DeserializeOwned>(parameters: Value) -> Result<T, OperationError> {
    serde_json::from_value(parameters).map_err(|_| {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "parameters do not match the image profile",
        )
    })
}

fn conflicting_inputs(message: &'static str) -> OperationError {
    OperationError::new(ErrorCode::InvalidInput, message)
}

pub(super) fn execute(
    id: &str,
    inputs: Value,
    parameters: Value,
    _: super::super::Budget,
    _: Option<super::super::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let raster_input = || raster(&inputs["input"]);
    let mask = || bytes(&inputs["mask"]);
    use spec::*;
    Ok(match id {
        "image.resize" => {
            let p: Resize = options(parameters)?;
            json!({"output":image(&resample::resize(&raster_input()?, p.width, p.height, p.kernel)?)})
        }
        "image.warp" => {
            let p: Warp = options(parameters)?;
            let source = raster_input()?;
            let output =
                resample::warp(&source, p.width, p.height, p.transform, p.kernel, &p.border)?;
            let report = WarpReport {
                transform: p.transform,
                source: stereo::Source::from(&source),
            };
            json!({"output":image(&output), "report":report})
        }
        "image.color.convert" => {
            let p: Color = options(parameters)?;
            json!({"report":colorimetry::convert(&raster_input()?, p.target)?})
        }
        "image.quality.measure" => {
            let p: Quality = options(parameters)?;
            let (left, right) = (raster(&inputs["left"])?, raster(&inputs["right"])?);
            json!({"report":quality::measure(&left, &right, p.region.into(), p.channel, p.metric)?})
        }
        "image.filter" => {
            let p: Filter = options(parameters)?;
            json!({"output":image(&filters::apply(&raster_input()?, p.filter, p.channels.as_deref(), p.border)?)})
        }
        "image.edge.response" => {
            let p: Edge = options(parameters)?;
            json!({"report":edges::responses(&raster_input()?, p.region.into(), p.channel, p.kernel, p.border, p.magnitude)?})
        }
        "image.edge.canny" => {
            let p: Canny = options(parameters)?;
            let result = edges::canny(&raster_input()?, p.region.into(), p.channel, p.settings)?;
            json!({"mask":images::bytes(&result.mask),"report":result})
        }
        "image.mask.otsu" => {
            let p: Selected = options(parameters)?;
            let result = binary_grid::otsu(&raster_input()?, p.region.into(), p.channel)?;
            json!({"mask":images::bytes(&result.mask),"report":result})
        }
        "image.mask.morphology" => {
            let p: Morphology = options(parameters)?;
            let result = binary_grid::morphology(
                p.width,
                p.height,
                &mask()?,
                p.operation,
                p.element,
                p.iterations,
            )?;
            json!({"mask":images::bytes(&result.mask),"report":result})
        }
        "image.mask.components" => {
            let p: Components = options(parameters)?;
            json!({"report":binary_grid::components(p.width, p.height, &mask()?, p.connectivity)?})
        }
        "image.mask.flood" => {
            let p: Components = options(parameters)?;
            let seeds: Vec<[u32; 2]> = if inputs.get("seeds").is_some() {
                field(&inputs, "seeds")?
            } else {
                vec![]
            };
            let result = binary_grid::flood(p.width, p.height, &mask()?, p.connectivity, &seeds)?;
            json!({"mask":images::bytes(&result.mask),"report":result})
        }
        "image.hough.lines" => {
            let p: Hough = options(parameters)?;
            json!({"report":voting::hough(p.width, p.height, &mask()?, p.options)?})
        }
        "image.template.match" => {
            let p: Template = options(parameters)?;
            let (input, template) = (raster_input()?, raster(&inputs["template"])?);
            json!({"report":matching::match_template(&input, p.channel, &template, p.template_channel, p.options)?})
        }
        "image.traversal.order" => {
            let p: Order = options(parameters)?;
            let coordinates = p
                .traversal
                .coordinates(p.region.into(), p.corner, p.reverse)?;
            if coordinates.len() > 1 << 20 {
                return Err(OperationError::new(
                    ErrorCode::ResourceLimit,
                    "direct traversal exceeds 1 Mi coordinates",
                ));
            }
            json!({"coordinates":coordinates.collect::<Vec<_>>()})
        }
        "image.traversal.permute" => {
            let p: Permutation = options(parameters)?;
            json!({"coordinates":traversal::permutation(p.region.into(), &field::<Vec<u32>>(&inputs, "indices")?)?})
        }
        "image.png.scanlines" => {
            let p: Scanlines = options(parameters)?;
            let result = native_samples::png_scanlines(&bytes(&inputs["source"])?, p.profile)?;
            json!({"data":images::bytes(&result.data),"report":result.report})
        }
        "image.gif.indices" => {
            let p: Gif = options(parameters)?;
            let result = native_samples::gif_indices(&bytes(&inputs["source"])?, p.frame_index)?;
            json!({"indices":images::bytes(&result.indices),"report":result.report})
        }
        "image.stereo.separate" => {
            let p: Stereo = options(parameters)?;
            let result = stereo::separate(&raster_input()?, p.mode, p.eye_order)?;
            let report = StereoReport {
                mode: result.mode,
                eye_order: result.eye_order,
                source: result.source,
            };
            json!({"left":image(&result.left),"right":image(&result.right),"report":report})
        }
        "image.stereo.anaglyph" => {
            let p: Anaglyph = options(parameters)?;
            let result = stereo::anaglyph(
                &raster(&inputs["left"])?,
                &raster(&inputs["right"])?,
                p.profile,
            )?;
            let report = AnaglyphReport {
                profile: result.profile,
                sources: result.sources,
                matrices: stereo::DUBOIS_REDCYAN_2009,
            };
            json!({"output":image(&result.image),"report":report})
        }
        "image.hash.perceptual" => {
            let p: Perceptual = options(parameters)?;
            json!({"output":images::bytes(&perceptual::digest(&raster_input()?, p.profile)?),
                "report":HashReport {profile:p.profile}})
        }
        "image.qr.encode" => {
            let segments = match (inputs.get("data"), inputs.get("segments")) {
                (Some(data), None) => vec![qr_encode::Segment::Byte { data: bytes(data)? }],
                (None, Some(_)) => field(&inputs, "segments")?,
                _ => return Err(conflicting_inputs("supply exactly one of data or segments")),
            };
            let result = qr_encode::encode(&segments, options(parameters)?)?;
            let mut result_fields =
                json!({"modules":images::bytes(&result.modules),"report":result.report});
            if let Some(output) = result.raster {
                result_fields["raster"] = image(&output);
            }
            result_fields
        }
        "image.barcode.scan" => {
            let p = options(parameters)?;
            let report = match (
                inputs.get("input"),
                inputs.get("rows"),
                inputs.get("scanlines"),
            ) {
                (Some(input), Some(_), None) => {
                    let rows: Vec<u32> = field(&inputs, "rows")?;
                    barcode::image_rows(&raster(input)?, &rows, p)?
                }
                (None, None, Some(_)) => {
                    let rows: Vec<Vec<u8>> = field(&inputs, "scanlines")?;
                    barcode::scanlines(&rows, p)?
                }
                _ => {
                    return Err(conflicting_inputs(
                        "supply scanlines or both input and rows",
                    ));
                }
            };
            json!({"report":report})
        }
        "image.barcode.decode" => {
            let crops = inputs["crops"].as_array().expect("validated crop list");
            let images = crops.iter().map(raster).collect::<Result<Vec<_>, _>>()?;
            let images = images.iter().collect::<Vec<_>>();
            json!({"report":optical::decode(&images, options(parameters)?)?})
        }
        _ => unreachable!("operation is checked against the image profile catalog"),
    })
}
