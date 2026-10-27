//! Test-only typed invocation drivers. They read fixed synthetic fixtures only;
//! no product endpoint, runtime dispatcher, algorithm or metadata registry is added.
mod audio;
mod blockchain;
mod files;
mod image_analysis;
mod images;
mod linear;
mod numbers;
mod puzzle_codes;
mod reasoning;
mod symbols;
mod text;
mod text_analysis;
mod video;
mod workflows;

use cube_core::errors::{ErrorCode, OperationError};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
fn field<T: DeserializeOwned>(value: &Value, name: &str) -> Result<T, OperationError> {
    serde_json::from_value(value.get(name).cloned().unwrap_or(Value::Null))
        .map_err(|_| OperationError::new(ErrorCode::InvalidInput, "invalid fixed fixture field"))
}
fn bytes(value: &Value) -> Result<Vec<u8>, OperationError> {
    cube_core::encoding::base64_decode(&field::<String>(value, "base64")?)
        .map_err(|_| OperationError::new(ErrorCode::InvalidInput, "invalid fixed fixture bytes"))
}
pub fn run_fixed_fixture(
    operation: &str,
    inputs: Value,
    parameters: Value,
) -> Result<Value, OperationError> {
    if puzzle_codes::supported(operation) {
        return puzzle_codes::execute(operation, inputs, parameters);
    }
    if linear::supported(operation) {
        return linear::execute(operation, inputs, parameters);
    }
    if numbers::supported(operation) {
        return numbers::execute(operation, inputs, parameters);
    }
    if text::supported(operation) {
        return text::execute(operation, inputs, parameters);
    }
    if text_analysis::supported(operation) {
        return text_analysis::execute(operation, inputs, parameters);
    }
    if reasoning::supported(operation) {
        return reasoning::execute(operation, inputs, parameters);
    }
    if symbols::supported(operation) {
        return symbols::execute(operation, inputs, parameters);
    }
    if files::supported(operation) {
        return files::execute(operation, inputs, parameters);
    }
    if audio::supported(operation) {
        return audio::execute(operation, inputs, parameters);
    }
    if image_analysis::supported(operation) {
        return image_analysis::execute(operation, inputs, parameters);
    }
    if images::supported(operation) {
        return images::execute(operation, inputs, parameters);
    }
    if video::supported(operation) {
        return video::execute(operation, inputs, parameters);
    }
    if blockchain::supported(operation) {
        return blockchain::execute(operation, inputs, parameters);
    }
    if workflows::supported(operation) {
        return workflows::execute(operation, inputs, parameters);
    }
    match operation {
        "encoding.base58check.encode" => Ok(
            json!({"output":cube_core::encoding::base58check::encode(field(&parameters,"version")?,&bytes(&inputs["input"])?)?}),
        ),
        "encoding.base58check.decode" => Ok(
            json!({"output":cube_core::encoding::base58check::decode(&field::<String>(&inputs,"input")?)?}),
        ),
        _ => Err(OperationError::new(
            ErrorCode::InvalidInput,
            "fixed fixture requires a direct Core call",
        )),
    }
}
