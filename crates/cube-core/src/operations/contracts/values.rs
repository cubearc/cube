//! Explicit, lossless logical values for types whose Core APIs deliberately use
//! readers, opaque rasters or iterators rather than serde transport objects.
use crate::{image, investigation, media, pipeline};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Bytes {
    #[serde(rename = "base64", serialize_with = "base64")]
    #[schemars(with = "String", extend("contentEncoding" = "base64"))]
    pub data: Vec<u8>,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub color_model: image::ColorModel,
    pub samples: Bytes,
    pub palette: Vec<[u8; 3]>,
    pub transparency: Bytes,
}
impl From<&image::Image> for Image {
    fn from(value: &image::Image) -> Self {
        let info = value.info();
        Self {
            width: info.width,
            height: info.height,
            color_model: info.color_model,
            samples: Bytes {
                data: value.samples().to_vec(),
            },
            palette: info.palette.to_vec(),
            transparency: Bytes {
                data: info.transparency.to_vec(),
            },
        }
    }
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pcm {
    pub format: media::wave::Format,
    pub channels: Vec<u16>,
    pub start_frame: u64,
    pub frames: u64,
    pub data: Bytes,
}
impl From<&media::pcm::Samples> for Pcm {
    fn from(value: &media::pcm::Samples) -> Self {
        Self {
            format: value.format(),
            channels: value.channels().to_vec(),
            start_frame: value.start_frame(),
            frames: value.frames(),
            data: Bytes {
                data: value.raw().to_vec(),
            },
        }
    }
}
/// A caller-supplied reference grants no authority. Each interface resolves it
/// only through its already authorized filesystem or artifact capability.
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FileReference {
    pub reference: String,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineDefinition {
    pub input_type: pipeline::Kind,
    pub steps: Vec<pipeline::Step>,
    pub limits: pipeline::Limits,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PipelineRequest {
    pub definition: PipelineDefinition,
    pub input: pipeline::Value,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecipeRequest {
    #[schemars(schema_with = "crate::pipeline::recipe::contract_definition")]
    pub recipe: serde_json::Value,
    pub bindings: std::collections::BTreeMap<String, pipeline::Value>,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecipeCheck {
    #[schemars(schema_with = "crate::pipeline::recipe::contract_definition")]
    pub recipe: serde_json::Value,
    pub input: pipeline::Value,
    pub expected: pipeline::Value,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordRequest {
    pub input: Bytes,
    pub spec: investigation::session::RunSpec,
    pub limits: investigation::session::SessionLimits,
    pub record_parameters: bool,
    pub retain_result: bool,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplayRequest {
    pub input: Bytes,
    pub session: investigation::session::Session,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProfileRequest {
    pub profile: investigation::profile::Profile,
    pub input_size: u64,
}
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    pub format: String,
    #[schemars(length(max = 128))]
    pub value: String,
}

fn base64<S: serde::Serializer>(value: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&crate::encoding::base64_encode(value))
}
