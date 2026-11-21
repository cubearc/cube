//! Explicit stereo layouts and the author's pinned Dubois sRGB projection.
use super::{ColorModel, Image, Region, colorimetry, invalid, limit, resample};
use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    RedCyan,
    SideBySide,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EyeOrder {
    #[default]
    LeftRight,
    RightLeft,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema)]
pub enum AnaglyphProfile {
    #[serde(rename = "dubois-redcyan-2009")]
    DuboisRedcyan2009,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct Source {
    pub format: &'static str,
    pub representation: &'static str,
    pub width: u32,
    pub height: u32,
    pub model: ColorModel,
}

impl From<&Image> for Source {
    fn from(image: &Image) -> Self {
        Self {
            format: image.source_format,
            representation: image.representation,
            width: image.width(),
            height: image.height(),
            model: image.model(),
        }
    }
}

#[derive(Debug)]
pub struct StereoPair {
    pub left: Image,
    pub right: Image,
    pub mode: Mode,
    pub eye_order: EyeOrder,
    pub source: Source,
}

#[derive(Debug)]
pub struct Anaglyph {
    pub image: Image,
    pub profile: AnaglyphProfile,
    pub sources: [Source; 2],
}

fn split_channels(image: &Image) -> Result<(Image, Image), OperationError> {
    resample::physical_samples(image)?;
    if !matches!(image.model(), ColorModel::Rgb | ColorModel::Rgba) {
        return Err(invalid("red-cyan separation requires RGB or RGBA samples"));
    }
    let channels = image.model().channels();
    let mut red = Vec::with_capacity(image.samples().len());
    let mut cyan = Vec::with_capacity(image.samples().len());
    for pixel in image.samples().chunks_exact(channels) {
        red.extend([pixel[0], 0, 0]);
        cyan.extend([0, pixel[1], pixel[2]]);
        if channels == 4 {
            red.push(pixel[3]);
            cyan.push(pixel[3]);
        }
    }
    let make = |data| {
        Image::from_samples(
            image.width(),
            image.height(),
            image.model(),
            data,
            vec![],
            vec![],
        )
    };
    Ok((make(red)?, make(cyan)?))
}

fn split_columns(image: &Image) -> Result<(Image, Image), OperationError> {
    if !image.width().is_multiple_of(2) {
        return Err(invalid("side-by-side separation requires even width"));
    }
    let region = Region {
        width: image.width() / 2,
        ..Region::full(image)
    };
    Ok((
        image.crop(region)?,
        image.crop(Region {
            x: region.width,
            ..region
        })?,
    ))
}

/// Separate the selected stereo layout in explicit logical eye order.
pub fn separate(
    image: &Image,
    mode: Mode,
    eye_order: EyeOrder,
) -> Result<StereoPair, OperationError> {
    (|| {
        let (first, second) = match mode {
            Mode::RedCyan => split_channels(image)?,
            Mode::SideBySide => split_columns(image)?,
        };
        let (left, right) = match eye_order {
            EyeOrder::LeftRight => (first, second),
            EyeOrder::RightLeft => (second, first),
        };
        Ok(StereoPair {
            left,
            right,
            mode,
            eye_order,
            source: image.into(),
        })
    })()
    .map_err(|error: OperationError| error.operation("image.stereo.separate"))
}

/// Eric Dubois, March 2009. Row order is output R, G, B for both input eyes.
pub const DUBOIS_REDCYAN_2009: [[[f64; 3]; 3]; 2] = [
    [
        [0.437, 0.449, 0.164],
        [-0.062, -0.062, -0.024],
        [-0.048, -0.050, -0.017],
    ],
    [
        [-0.011, -0.032, -0.007],
        [0.377, 0.761, 0.009],
        [-0.026, -0.093, 1.234],
    ],
];

/// Project equal opaque sRGB eyes. Each eye's linear projection is clipped
/// before addition. The final sRGB samples are rounded half up.
pub fn anaglyph(
    left: &Image,
    right: &Image,
    profile: AnaglyphProfile,
) -> Result<Anaglyph, OperationError> {
    (|| {
        for eye in [left, right] {
            resample::physical_samples(eye)?;
            if eye.model() != ColorModel::Rgb {
                return Err(invalid("Dubois projection requires two opaque RGB views"));
            }
        }
        if (left.width(), left.height()) != (right.width(), right.height()) {
            return Err(invalid("stereo eye dimensions must match"));
        }
        if left.samples().len() / 3 > 1 << 20 {
            return Err(limit("Dubois projection exceeds 1 Mi stereo pixels"));
        }
        let [lm, rm] = DUBOIS_REDCYAN_2009;
        let project = |matrix, rgb: &[u8]| {
            colorimetry::project(
                matrix,
                [rgb[0], rgb[1], rgb[2]].map(colorimetry::decode_srgb),
            )
        };
        let mut output = Vec::with_capacity(left.samples().len());
        for (l, r) in left
            .samples()
            .as_chunks::<3>()
            .0
            .iter()
            .zip(right.samples().as_chunks::<3>().0)
        {
            let l = project(lm, l);
            let r = project(rm, r);
            output.extend((0..3).map(|i| {
                colorimetry::encode_srgb((l[i].clamp(0., 1.) + r[i].clamp(0., 1.)).min(1.))
            }));
        }
        Ok(Anaglyph {
            image: Image::from_samples(
                left.width(),
                left.height(),
                ColorModel::Rgb,
                output,
                vec![],
                vec![],
            )?,
            profile,
            sources: [left.into(), right.into()],
        })
    })()
    .map_err(|error: OperationError| error.operation("image.stereo.anaglyph"))
}
