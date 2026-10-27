//! Opt-in computed-result transport. A capture is a declaration, not execution
//! attestation, a filesystem capability, or permission to rerun an operation.
use super::{
    Digest, Identity, MAX_INPUT_BYTES, MAX_RECORD_BYTES, Result, invalid, json_bytes, limit,
};
use crate::{
    operations::{
        self,
        contracts::{self, Contract},
    },
    pipeline,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const FORMAT_VERSION: u32 = 1;
pub const ADAPTER_VERSION: u32 = 1;
pub const MAX_BYTES: usize = MAX_RECORD_BYTES;
/// The initial CLI cohort is explicit; this is not an operation dispatcher.
pub const OPERATIONS: &[&str] = &[
    "text.reverse",
    "encoding.hex.decode",
    "image.invert",
    "image.histogram",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub name: String,
    pub semantic_type: String,
    pub digest_representation: String,
    pub identity: Identity,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    pub name: String,
    pub semantic_type: String,
    pub schema_sha256: Digest,
    pub content_sha256: Digest,
    pub byte_size: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedCapture {
    pub schema_version: u32,
    pub adapter_version: u32,
    pub engine_api: u32,
    pub contract_schema_version: u32,
    pub core_version: String,
    pub operation: String,
    pub operation_version: u32,
    pub capture_id: String,
    pub inputs: Vec<Input>,
    pub parameters: Value,
    pub outputs: Value,
    pub ports: Vec<Port>,
    pub replay_gaps: Vec<String>,
}
fn contract(operation: &str) -> Result<Contract> {
    if !OPERATIONS.contains(&operation) {
        return Err(invalid(
            "Typed capture is unavailable for this operation; use ordinary terminal execution",
        ));
    }
    let contract = operations::require(operation)
        .and_then(|d| d.contract())
        .map_err(pipeline::Error::from_operation)?;
    if contract.policy.private_input
        || contract.policy.private_output
        || !contract.policy.sensitive_parameters.is_empty()
    {
        return Err(invalid("Private operations cannot be captured by default"));
    }
    Ok(contract)
}
fn gap(name: &str) -> String {
    format!(
        "Input port '{name}' was not retained; its content identity does not locate or authorize a replay source"
    )
}
fn binary_values(value: &Value) -> Result<()> {
    match value {
        Value::Object(fields) => {
            if fields.len() == 1
                && let Some(encoded) = fields.get("base64")
            {
                let encoded = encoded
                    .as_str()
                    .ok_or_else(|| invalid("Captured bytes require canonical Base64"))?;
                let bytes = crate::encoding::base64_decode(encoded)
                    .map_err(|_| invalid("Captured bytes require canonical Base64"))?;
                if crate::encoding::base64_encode(&bytes) != encoded {
                    return Err(invalid("Captured bytes require canonical Base64"));
                }
            }
            for child in fields.values() {
                binary_values(child)?;
            }
        }
        Value::Array(items) => {
            for child in items {
                binary_values(child)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn image_value(value: &Value) -> Result<()> {
    let field = |key: &str| {
        value
            .get(key)
            .cloned()
            .ok_or_else(|| invalid("Captured image fields are missing"))
    };
    let bytes = |key: &str| {
        crate::encoding::base64_decode(
            value[key]["base64"]
                .as_str()
                .ok_or_else(|| invalid("Captured image bytes are missing"))?,
        )
        .map_err(|_| invalid("Captured image bytes are invalid"))
    };
    let model = crate::image::ColorModel::parse(
        value["color_model"]
            .as_str()
            .ok_or_else(|| invalid("Captured image model is missing"))?,
    )
    .map_err(pipeline::Error::from_operation)?;
    crate::image::Image::from_samples(
        serde_json::from_value(field("width")?)
            .map_err(|_| invalid("Invalid captured image width"))?,
        serde_json::from_value(field("height")?)
            .map_err(|_| invalid("Invalid captured image height"))?,
        model,
        bytes("samples")?,
        serde_json::from_value(field("palette")?)
            .map_err(|_| invalid("Invalid captured image palette"))?,
        bytes("transparency")?,
    )
    .map_err(pipeline::Error::from_operation)?;
    Ok(())
}
impl TypedCapture {
    /// Validate and describe already-computed named values. This function never
    /// executes an algorithm, retains input values, opens files or uses a network.
    pub fn from_result(
        operation: &str,
        capture_id: &str,
        inputs: &Value,
        parameters: &Value,
        outputs: &Value,
    ) -> Result<Self> {
        let contract = contract(operation)?;
        let inputs = contracts::canonical_value(inputs.clone());
        let mut parameters = contracts::canonical_value(parameters.clone());
        let outputs = contracts::canonical_value(outputs.clone());
        json_bytes(&inputs, MAX_INPUT_BYTES)?;
        json_bytes(&outputs, MAX_BYTES)?;
        contract
            .validate_inputs(&inputs)
            .map_err(pipeline::Error::from_operation)?;
        contract
            .validate_parameters(&parameters)
            .map_err(pipeline::Error::from_operation)?;
        let supplied = parameters
            .as_object_mut()
            .ok_or_else(|| invalid("Capture parameters must be an object"))?;
        for (name, schema) in contract.parameters.as_value()["properties"]
            .as_object()
            .ok_or_else(|| invalid("Invalid installed parameter contract"))?
        {
            if let Some(value) = schema.get("default") {
                supplied
                    .entry(name.clone())
                    .or_insert_with(|| value.clone());
            }
        }
        let identities = contract
            .inputs
            .iter()
            .enumerate()
            .filter(|(_, p)| inputs.get(&p.name).is_some())
            .map(|(index, p)| {
                let bytes = json_bytes(&inputs[&p.name], MAX_INPUT_BYTES)?;
                Ok(Input {
                    name: p.name.clone(),
                    semantic_type: p.semantic_type.clone(),
                    digest_representation: "json_wire_value".into(),
                    identity: Identity {
                        id: index as u32 + 1,
                        size: bytes.len() as u64,
                        digest: Some(Digest::bytes(&bytes)),
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let ports = contract
            .outputs
            .iter()
            .filter(|p| outputs.get(&p.name).is_some())
            .map(|p| {
                let bytes = json_bytes(&outputs[&p.name], MAX_BYTES)?;
                Ok(Port {
                    name: p.name.clone(),
                    semantic_type: p.semantic_type.clone(),
                    schema_sha256: Digest::bytes(&json_bytes(&p.schema, MAX_BYTES)?),
                    content_sha256: Digest::bytes(&bytes),
                    byte_size: bytes.len() as u64,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let capture = Self {
            schema_version: FORMAT_VERSION,
            adapter_version: ADAPTER_VERSION,
            engine_api: pipeline::ENGINE_API,
            contract_schema_version: contract.schema_version,
            core_version: env!("CARGO_PKG_VERSION").into(),
            operation: operation.into(),
            operation_version: contract.operation_version,
            capture_id: capture_id.into(),
            replay_gaps: identities.iter().map(|i| gap(&i.name)).collect(),
            inputs: identities,
            parameters,
            outputs,
            ports,
        };
        capture.validate()?;
        Ok(capture)
    }
    pub fn from_json(document: &str) -> Result<Self> {
        if document.len() > MAX_BYTES {
            return Err(limit("Typed capture exceeds 128 KiB"));
        }
        let value = pipeline::json::parse_with_limits(document, MAX_BYTES, 16, 4096)?;
        let capture: Self = serde_json::from_value(value)
            .map_err(|_| invalid("Invalid typed capture fields or versions"))?;
        capture.validate()?;
        Ok(capture)
    }
    pub fn to_json(&self) -> Result<String> {
        self.validate()?;
        String::from_utf8(json_bytes(self, MAX_BYTES)?)
            .map_err(|_| invalid("Invalid capture encoding"))
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != FORMAT_VERSION
            || self.adapter_version != ADAPTER_VERSION
            || self.engine_api != pipeline::ENGINE_API
            || self.contract_schema_version != contracts::FORMAT_VERSION
        {
            return Err(invalid(
                "Unsupported typed capture schema, adapter or engine version",
            ));
        }
        if self.capture_id.is_empty()
            || self.capture_id.len() > 96
            || !self
                .capture_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self.core_version.is_empty()
            || self.core_version.len() > 64
        {
            return Err(invalid("Invalid typed capture identity or package version"));
        }
        let document = json_bytes(self, MAX_BYTES)?;
        pipeline::json::parse_with_limits(
            std::str::from_utf8(&document).map_err(|_| invalid("Invalid capture encoding"))?,
            MAX_BYTES,
            16,
            4096,
        )?;
        let contract = contract(&self.operation)?;
        if self.operation_version != contract.operation_version {
            return Err(invalid("Unsupported captured operation version"));
        }
        contract
            .validate_parameters(&self.parameters)
            .map_err(pipeline::Error::from_operation)?;
        contract
            .validate_outputs(&self.outputs)
            .map_err(pipeline::Error::from_operation)?;
        binary_values(&self.outputs)?;
        let input_names = self
            .inputs
            .iter()
            .map(|i| i.name.as_str())
            .collect::<BTreeSet<_>>();
        if input_names.len() != self.inputs.len()
            || self.inputs.len() > 32
            || contract
                .inputs
                .iter()
                .any(|p| p.required && !input_names.contains(p.name.as_str()))
        {
            return Err(invalid(
                "Captured input identities do not match named ports",
            ));
        }
        let mut total = 0u64;
        for (index, input) in self.inputs.iter().enumerate() {
            let port = contract
                .inputs
                .iter()
                .find(|p| p.name == input.name)
                .ok_or_else(|| invalid("Unknown captured input port"))?;
            if input.identity.id != index as u32 + 1
                || input.semantic_type != port.semantic_type
                || input.digest_representation != "json_wire_value"
                || input.identity.size > MAX_INPUT_BYTES as u64
            {
                return Err(invalid("Invalid captured input representation or bounds"));
            }
            input
                .identity
                .digest
                .as_ref()
                .ok_or_else(|| invalid("Captured input digest is missing"))?
                .validate()?;
            total = total
                .checked_add(input.identity.size)
                .ok_or_else(|| limit("Capture input size overflow"))?;
        }
        if total > MAX_INPUT_BYTES as u64
            || self.replay_gaps != self.inputs.iter().map(|i| gap(&i.name)).collect::<Vec<_>>()
        {
            return Err(invalid(
                "Captured input bounds or replay gaps are inconsistent",
            ));
        }
        let names = self
            .ports
            .iter()
            .map(|p| p.name.as_str())
            .collect::<BTreeSet<_>>();
        if names.len() != self.ports.len()
            || self
                .outputs
                .as_object()
                .is_none_or(|o| o.len() != names.len())
        {
            return Err(invalid(
                "Captured output evidence must match every named value",
            ));
        }
        for port in &self.ports {
            let expected = contract
                .outputs
                .iter()
                .find(|p| p.name == port.name)
                .ok_or_else(|| invalid("Unknown captured output port"))?;
            let value = self
                .outputs
                .get(&port.name)
                .ok_or_else(|| invalid("Captured output value is missing"))?;
            let bytes = json_bytes(value, MAX_BYTES)?;
            if port.semantic_type != expected.semantic_type
                || port.schema_sha256 != Digest::bytes(&json_bytes(&expected.schema, MAX_BYTES)?)
                || port.content_sha256 != Digest::bytes(&bytes)
                || port.byte_size != bytes.len() as u64
            {
                return Err(invalid(
                    "Captured output representation, schema or content digest does not match",
                ));
            }
            if port.semantic_type == "image.samples" {
                image_value(value)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn typed_captures_preserve_exact_values_and_redact_original_inputs_without_execution() {
        let original = "Do not retain this original input.";
        let result = crate::text::reverse(original).unwrap();
        let capture = TypedCapture::from_result(
            "text.reverse",
            "fixture-1",
            &json!({"input":original}),
            &json!({}),
            &json!({"output":result}),
        )
        .unwrap();
        let document = capture.to_json().unwrap();
        assert!(!document.contains(original));
        let restored = TypedCapture::from_json(&document).unwrap();
        assert_eq!(restored.outputs["output"], result);
        assert!(!restored.replay_gaps.is_empty());
        let input = json!({"input":"00ff80"});
        let bytes = crate::encoding::hex_decode("00ff80").unwrap();
        let capture = TypedCapture::from_result(
            "encoding.hex.decode",
            "binary-1",
            &input,
            &json!({}),
            &json!({"output":contracts::values::Bytes{data:bytes}}),
        )
        .unwrap();
        assert_eq!(
            crate::encoding::base64_decode(capture.outputs["output"]["base64"].as_str().unwrap())
                .unwrap(),
            [0, 255, 128]
        );
        let mut changed = capture.clone();
        changed.outputs["output"]["base64"] = json!("AAAA");
        assert!(changed.validate().is_err());
        let mut changed = capture.clone();
        changed.operation_version = 99;
        assert!(changed.validate().is_err());
        let mut changed = capture;
        changed.replay_gaps.clear();
        assert!(changed.validate().is_err());
        assert!(
            TypedCapture::from_result(
                "blockchain.private.export",
                "no-private",
                &json!({}),
                &json!({}),
                &json!({})
            )
            .is_err()
        );
    }
    #[test]
    fn bounded_documents_reject_duplicates_versions_and_one_over_the_wire_limit() {
        let capture = TypedCapture::from_result(
            "text.reverse",
            "limits",
            &json!({"input":"A"}),
            &json!({}),
            &json!({"output":"A"}),
        )
        .unwrap();
        let mut document = capture.to_json().unwrap();
        document.push_str(&" ".repeat(MAX_BYTES - document.len()));
        TypedCapture::from_json(&document).unwrap();
        document.push(' ');
        assert!(TypedCapture::from_json(&document).is_err());
        let duplicate = capture.to_json().unwrap().replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        );
        assert!(TypedCapture::from_json(&duplicate).is_err());
        let mut capture = capture;
        capture.adapter_version = 99;
        assert!(capture.validate().is_err());
    }
    #[test]
    fn capture_provenance_and_logical_image_shape_cannot_be_substituted() {
        let source = crate::image::Image::from_bytes(
            1,
            1,
            crate::image::ColorModel::Rgb,
            &[1, 2, 3],
            &[],
            &[],
        )
        .unwrap();
        let result = source.invert(None).unwrap();
        let inputs = json!({"input":contracts::values::Image::from(&source)});
        let outputs = json!({"output":contracts::values::Image::from(&result)});
        let capture =
            TypedCapture::from_result("image.invert", "image-shape", &inputs, &json!({}), &outputs)
                .unwrap();
        assert_eq!(capture.parameters["expand_palette"], false);
        TypedCapture::from_json(&capture.to_json().unwrap()).unwrap();
        let mut bad = outputs.clone();
        bad["output"]["width"] = json!(2);
        assert!(
            TypedCapture::from_result("image.invert", "bad-shape", &inputs, &json!({}), &bad)
                .is_err()
        );
        bad = outputs;
        bad["output"]["samples"]["base64"] = json!("!invalid!");
        assert!(
            TypedCapture::from_result("image.invert", "bad-bytes", &inputs, &json!({}), &bad)
                .is_err()
        );
        let mut bad = capture.clone();
        bad.ports[0].schema_sha256.value = "0".repeat(64);
        assert!(bad.validate().is_err());
        let mut bad = capture.clone();
        bad.ports.clear();
        assert!(bad.validate().is_err());
        let mut bad = capture.clone();
        bad.inputs.push(bad.inputs[0].clone());
        assert!(bad.validate().is_err());
        let mut bad = capture.clone();
        bad.inputs[0].identity.digest = None;
        assert!(bad.validate().is_err());
        let mut bad = capture.clone();
        bad.inputs[0].identity.size = MAX_INPUT_BYTES as u64 + 1;
        assert!(bad.validate().is_err());
        let mut bad = capture.clone();
        bad.capture_id = "../escape".into();
        assert!(bad.validate().is_err());
        let mut bad = capture;
        bad.parameters["unexpected"] = json!(true);
        assert!(bad.validate().is_err());
    }
}
