use super::*;
use crate::operations::ParameterType;

pub(super) fn schema(descriptor: &Descriptor) -> Result<Schema, OperationError> {
    let mut properties = serde_json::Map::new();
    let mut required = Vec::new();
    for parameter in descriptor.parameters {
        if properties.contains_key(parameter.name) {
            return Err(invalid("duplicate canonical parameter"));
        }
        let mut value = match parameter.constraints {
            ParameterType::Integer { min, max } => {
                json!({"type":"integer","minimum":min,"maximum":max})
            }
            ParameterType::Unsigned { min, max } => {
                json!({"type":"integer","minimum":min,"maximum":max})
            }
            ParameterType::UnsignedChoices { choices } => json!({"type":"integer","enum":choices}),
            ParameterType::Boolean => json!({"type":"boolean"}),
            ParameterType::Text { choices } => {
                if choices.is_empty() {
                    json!({"type":"string"})
                } else {
                    json!({"type":"string","enum":choices})
                }
            }
            ParameterType::Bytes { min_length } => {
                json!({"type":"array","minItems":min_length,"items":{"type":"integer","minimum":0,"maximum":255}})
            }
            ParameterType::TextList {
                choices,
                min_length,
                max_length,
                unique,
            } => {
                let item = if choices.is_empty() {
                    json!({"type":"string"})
                } else {
                    json!({"type":"string","enum":choices})
                };
                json!({"type":"array","items":item,"minItems":min_length,"maxItems":max_length,"uniqueItems":unique})
            }
            ParameterType::UnsignedList {
                min,
                max,
                min_length,
                max_length,
                unique,
            } => {
                json!({"type":"array","items":{"type":"integer","minimum":min,"maximum":max},"minItems":min_length,"maxItems":max_length,"uniqueItems":unique})
            }
        };
        value["description"] = json!(parameter.description);
        if let Some(default) = parameter.default {
            value["default"] = default_value(default);
        }
        if parameter.required && parameter.default.is_none() {
            required.push(parameter.name);
        }
        if descriptor
            .metadata()
            .is_some_and(|m| m.security.sensitive_parameters.contains(&parameter.name))
        {
            value["writeOnly"] = json!(true);
        }
        properties.insert(parameter.name.into(), value);
    }
    Schema::try_from(json!({"$schema":"https://json-schema.org/draft/2020-12/schema","type":"object","properties":properties,"required":required,"additionalProperties":false})).map_err(|_|invalid("invalid parameter schema"))
}
