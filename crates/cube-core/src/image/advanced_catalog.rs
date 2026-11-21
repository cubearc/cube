//! Contracts for bounded image measurements, raster transforms and native samples.
use super::{
    Channel, Corner, Region, Traversal, barcode, binary_grid, colorimetry, edges, filters,
    matching, native_samples, optical, perceptual, qr_encode, quality, resample, stereo, voting,
};
use crate::operations::{
    self as op,
    contracts::{self as c, values},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub const MAX_DOCUMENT: usize = 64 << 20;
const MIB: usize = 1 << 20;

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl From<Rect> for Region {
    fn from(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

macro_rules! settings {
    ($name:ident { $( $(#[$attribute:meta])* $field:ident : $type:ty ),* $(,)? }) => {
        #[derive(Deserialize, JsonSchema)]
        #[serde(deny_unknown_fields)]
        pub struct $name { $( $(#[$attribute])* pub $field: $type ),* }
    };
}
settings!(Resize {
    width: u32,
    height: u32,
    kernel: resample::Kernel
});
settings!(Warp {
    width: u32,
    height: u32,
    transform: resample::Transform,
    #[serde(default)]
    kernel: resample::WarpKernel,
    #[serde(default)]
    border: resample::Border,
});
settings!(Color {
    target: colorimetry::Target
});
settings!(Quality {
    region: Rect,
    channel: Channel,
    metric: quality::Metric
});
settings!(Filter {
    filter: filters::Filter,
    #[schemars(length(max = 4))] channels: Option<Vec<Channel>>,
    #[serde(default)] border: filters::FilterBorder,
});
settings!(Edge {
    region: Rect,
    channel: Channel,
    kernel: edges::EdgeKernel,
    #[serde(default)]
    border: edges::EdgeBorder,
    #[serde(default)]
    magnitude: bool,
});
settings!(Canny {
    region: Rect,
    channel: Channel,
    settings: edges::Canny
});
settings!(Selected {
    region: Rect,
    channel: Channel
});
fn one() -> u8 {
    1
}
settings!(Morphology {
    width: u32,
    height: u32,
    operation: binary_grid::Morphology,
    #[serde(default)]
    element: binary_grid::Element,
    #[serde(default = "one")]
    #[schemars(range(min = 1, max = 16))]
    iterations: u8,
});
settings!(Components {
    width: u32,
    height: u32,
    connectivity: binary_grid::Connectivity
});
settings!(Hough {
    width: u32,
    height: u32,
    options: voting::HoughOptions
});
settings!(Template {
    channel: Channel,
    template_channel: Channel,
    options: matching::TemplateOptions
});
settings!(Order {
    region: Rect,
    traversal: Traversal,
    #[serde(default)]
    corner: Corner,
    #[serde(default)]
    reverse: bool,
});
settings!(Permutation { region: Rect });
settings!(Scanlines {
    profile: native_samples::PngProfile
});
settings!(Gif {
    #[serde(default)]
    #[schemars(range(max = 0))]
    frame_index: u32
});
settings!(Stereo {
    mode: stereo::Mode,
    #[serde(default)]
    eye_order: stereo::EyeOrder
});
settings!(Anaglyph {
    profile: stereo::AnaglyphProfile
});
settings!(Perceptual {
    profile: perceptual::Profile
});

#[derive(Serialize, JsonSchema)]
pub struct WarpReport {
    pub transform: resample::Transform,
    pub source: stereo::Source,
}
#[derive(Serialize, JsonSchema)]
pub struct StereoReport {
    pub mode: stereo::Mode,
    pub eye_order: stereo::EyeOrder,
    pub source: stereo::Source,
}
#[derive(Serialize, JsonSchema)]
pub struct AnaglyphReport {
    pub profile: stereo::AnaglyphProfile,
    pub sources: [stereo::Source; 2],
    pub matrices: [[[f64; 3]; 3]; 2],
}
#[derive(Serialize, JsonSchema)]
pub struct HashReport {
    pub profile: perceptual::Profile,
}

const INPUT: c::Port = image("input");
const OUTPUT: c::Port = image("output");
const LEFT: c::Port = image("left");
const RIGHT: c::Port = image("right");
const MASK: c::Port = bytes::<MIB>("mask");
const SOURCE: c::Port = bytes::<{ 32 * MIB }>("source");

const fn image(name: &'static str) -> c::Port {
    c::Port {
        name,
        semantic_type: "image.samples",
        schema: c::schema::<values::Image>,
        required: true,
    }
}
const fn bytes<const N: usize>(name: &'static str) -> c::Port {
    c::Port {
        name,
        semantic_type: "bytes",
        schema: c::bytes_schema::<N>,
        required: true,
    }
}
const fn report<T: JsonSchema>() -> c::Port {
    typed::<T>("report")
}
const fn typed<T: JsonSchema>(name: &'static str) -> c::Port {
    c::Port {
        name,
        semantic_type: "json",
        schema: c::schema::<T>,
        required: true,
    }
}
fn array<T: JsonSchema, const N: usize>() -> schemars::Schema {
    let mut schema = c::schema::<Vec<T>>().to_value();
    schema["maxItems"] = json!(N);
    schemars::Schema::try_from(schema).expect("bounded array schema")
}
const fn list<T: JsonSchema, const N: usize>(name: &'static str) -> c::Port {
    c::Port {
        name,
        semantic_type: "json",
        schema: array::<T, N>,
        required: true,
    }
}
const fn optional(port: c::Port) -> c::Port {
    c::Port {
        required: false,
        ..port
    }
}

fn fixture(id: &'static str) -> Result<c::Fixture, crate::errors::OperationError> {
    let all: Value = serde_json::from_str(include_str!("advanced-fixtures.json"))
        .expect("reviewed image fixtures");
    let row = &all[id];
    assert!(row.is_object(), "missing reviewed example for {id}");
    Ok(c::Fixture {
        inputs: row["inputs"].clone(),
        parameters: row["parameters"].clone(),
        outputs: row["outputs"].clone(),
    })
}

macro_rules! row {
    ($id:literal, $slug:literal, $description:literal, $settings:ty,
     $inputs:expr => $outputs:expr, $input:expr, $output:expr, $items:expr, $work:expr, $memory:expr) => {
        c::Entry {
            operation_version: 1,
            descriptor: op::Descriptor {
                id: $id,
                category: "image",
                description: $description,
                input_type: "json",
                output_type: "json",
                parameters: &[],
                deterministic: true,
                reversible: None,
                command: concat!("image profile ", $slug),
                example: concat!("cube image profile ", $slug, " --input request.json"),
            },
            inputs: $inputs,
            outputs: $outputs,
            parameters: c::schema::<$settings>,
            limits: c::Limits {
                max_input_bytes: Some($input),
                max_output_bytes: Some($output),
                max_items: Some($items),
                max_work: Some($work),
                max_memory_bytes: Some($memory),
            },
            fixture: || fixture($id),
        }
    };
}

const ROWS: &[c::Entry] = &[
    row!("image.resize", "resize", "Resize physical channels using half-pixel centers and the selected nearest, bilinear or bicubic kernel.", Resize,
        &[INPUT] => &[OUTPUT], 24 << 20, 24 << 20, 4 << 20, 64 << 20, 64 << 20),
    row!("image.warp", "warp", "Apply a finite inverse affine or projective matrix with an explicit interpolation and border.", Warp,
        &[INPUT] => &[OUTPUT, report::<WarpReport>()], 24 << 20, 24 << 20, 4 << 20, 64 << 20, 64 << 20),
    row!("image.color.convert", "color", "Measure explicit sRGB samples as HSV, HSL or Bradford-adapted CIELAB D50.", Color,
        &[INPUT] => &[report::<colorimetry::Report>()], 24 << 20, 8 << 20, 65_536, 16 << 20, 32 << 20),
    row!("image.quality.measure", "quality", "Measure exact squared error, PSNR or single-scale SSIM on a selected physical channel.", Quality,
        &[LEFT, RIGHT] => &[report::<quality::Report>()], 48 << 20, 1 << 20, 1 << 20, 64 << 20, 32 << 20),
    row!("image.filter", "filter", "Smooth selected channels with a bounded Gaussian or median filter and an explicit border.", Filter,
        &[INPUT] => &[OUTPUT], 24 << 20, 24 << 20, 4 << 20, 64 << 20, 32 << 20),
    row!("image.edge.response", "edge", "Report signed Sobel, Prewitt or discrete Laplacian responses for one physical channel.", Edge,
        &[INPUT] => &[report::<edges::Responses>()], 24 << 20, 32 << 20, 1 << 20, 32 << 20, 32 << 20),
    row!("image.edge.canny", "canny", "Detect edges in an explicit grayscale channel with bounded Gaussian, Sobel and hysteresis stages.", Canny,
        &[INPUT] => &[MASK, report::<edges::CannyReport>()], 24 << 20, 4 << 20, 1 << 20, 64 << 20, 48 << 20),
    row!("image.mask.otsu", "otsu", "Threshold one physical channel using exact Otsu histogram comparisons and a stable tie rule.", Selected,
        &[INPUT] => &[MASK, report::<binary_grid::Threshold>()], 24 << 20, 4 << 20, 1 << 20, 4 << 20, 16 << 20),
    row!("image.mask.morphology", "morphology", "Erode or dilate an exact binary mask with a bounded rectangular or cross-shaped element.", Morphology,
        &[MASK] => &[MASK, report::<binary_grid::MorphologyReport>()], 2 << 20, 4 << 20, 1 << 20, 64 << 20, 16 << 20),
    row!("image.mask.components", "components", "Label binary components in first-pixel row-major order and report their area and bounds.", Components,
        &[MASK] => &[report::<binary_grid::Components>()], 2 << 20, 16 << 20, 1 << 20, 16 << 20, 32 << 20),
    row!("image.mask.flood", "flood", "Select the union of the foreground components containing the supplied seeds.", Components,
        &[MASK, optional(list::<[u32;2],65_536>("seeds"))] => &[MASK, report::<binary_grid::Flood>()], 4 << 20, 16 << 20, 1 << 20, 16 << 20, 32 << 20),
    row!("image.hough.lines", "hough", "Vote for polar lines in an explicit binary edge mask and return bounded ordered peaks.", Hough,
        &[MASK] => &[report::<voting::Report>()], 1 << 20, 16 << 20, 1 << 20, 16 << 20, 32 << 20),
    row!("image.template.match", "template", "Compare valid template locations using SSD or zero-mean NCC and exact sample moments.", Template,
        &[INPUT, image("template")] => &[report::<matching::Report>()], 48 << 20, 16 << 20, 1 << 20, 16 << 20, 32 << 20),
    row!("image.traversal.order", "traversal", "List checked coordinates in the selected explicit corner, traversal and reversal order.", Order,
        &[] => &[list::<[u32;2],MIB>("coordinates")], 4096, 32 << 20, 1 << 20, 16 << 20, 16 << 20),
    row!("image.traversal.permute", "permutation", "Validate a rectangular destination-to-source bijection and return its exact source coordinates.", Permutation,
        &[list::<u32,MIB>("indices")] => &[list::<[u32;2],MIB>("coordinates")], 8 << 20, 32 << 20, 1 << 20, 16 << 20, 16 << 20),
    row!("image.png.scanlines", "png-scanlines", "Read strict noninterlaced PNG8 filtered or reconstructed native scanlines.", Scanlines,
        &[SOURCE] => &[bytes::<{16*MIB+16_384}>("data"), report::<native_samples::PngReport>()], 44 << 20, 24 << 20, 4 << 20, 64 << 20, 64 << 20),
    row!("image.gif.indices", "gif-indices", "Read exact deinterlaced GIF indices, active palette and frame offset for one indexed frame.", Gif,
        &[SOURCE] => &[bytes::<{4*MIB}>("indices"), report::<native_samples::GifReport>()], 44 << 20, 16 << 20, 4 << 20, 64 << 20, 64 << 20),
    row!("image.stereo.separate", "stereo", "Separate explicit red-cyan channels or equal-width side-by-side stereo eyes.", Stereo,
        &[INPUT] => &[LEFT, RIGHT, report::<StereoReport>()], 24 << 20, 48 << 20, 4 << 20, 16 << 20, 48 << 20),
    row!("image.stereo.anaglyph", "anaglyph", "Project a pair of opaque sRGB views using the pinned March 2009 Dubois red-cyan matrices.", Anaglyph,
        &[LEFT, RIGHT] => &[OUTPUT, report::<AnaglyphReport>()], 48 << 20, 16 << 20, 1 << 20, 32 << 20, 48 << 20),
    row!("image.hash.perceptual", "perceptual-hash", "Compute a versioned opaque-image average, difference or DCT similarity digest.", Perceptual,
        &[INPUT] => &[bytes::<8>("output"), report::<HashReport>()], 24 << 20, 1024, 1 << 20, 8 << 20, 16 << 20),
    row!("image.qr.encode", "qr-encode", "Generate QR Model 2 modules from explicit segments with a forced or scored mask and an optional gray raster.", qr_encode::Options,
        &[optional(bytes::<2953>("data")), optional(list::<qr_encode::Segment,256>("segments"))]
          => &[bytes::<31_329>("modules"), report::<qr_encode::Report>(), optional(image("raster"))],
        64 << 10, 16 << 20, 31_329, 64 << 20, 32 << 20),
    row!("image.barcode.scan", "barcode-scan", "Read one explicitly selected linear symbology on bounded rows and report checksum evidence.", barcode::ScanOptions,
        &[optional(INPUT), optional(list::<u32,1024>("rows")), optional(list::<Vec<u8>,1024>("scanlines"))]
          => &[report::<barcode::ScanReport>()], 24 << 20, 4 << 20, 1024, 16 << 20, 32 << 20),
    row!("image.barcode.decode", "barcode-decode", "Read Data Matrix ECC200, Aztec or PDF417 from explicitly selected opaque gray crops.", optical::Options,
        &[list::<values::Image,8>("crops")] => &[report::<optical::Report>()],
        4 << 20, 1 << 20, 8, 32 << 20, 32 << 20),
];

const fn descriptors<const N: usize>() -> [op::Descriptor; N] {
    let mut output = [ROWS[0].descriptor; N];
    let mut i = 0;
    while i < N {
        output[i] = ROWS[i].descriptor;
        i += 1;
    }
    output
}

pub const OPERATIONS: &[op::Descriptor] = &descriptors::<{ ROWS.len() }>();
pub(crate) const CONTRACTS: &[c::Entry] = ROWS;

/// Resolve a reviewed command slug without inferring a profile from input.
pub fn command(slug: &str) -> Option<&'static str> {
    OPERATIONS
        .iter()
        .find(|row| row.command.strip_prefix("image profile ") == Some(slug))
        .map(|row| row.id)
}
