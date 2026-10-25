//! Physical key/layer remapping on an explicitly shared geometry.
use super::{MAX_OUTPUT, check_input};
use crate::errors::{ErrorCode, OperationError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
/// One physical key in one layer. Layers are literal IDs, e.g. base/shift.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub key: String,
    pub layer: String,
    pub symbol: char,
}
/// Caller-owned layout definition; no OS layout detection or event handling.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub geometry: String,
    pub entries: Vec<Entry>,
}
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).operation("text.keyboard.remap")
}
// Factual US ANSI base/shift positions. Data kept separate from the algorithm.
// Checked against xkeyboard-config symbols/us (basic and dvorak):
// https://gitlab.freedesktop.org/xkeyboard-config/xkeyboard-config/-/blob/master/symbols/us
const QWERTY: [&str; 2] = [
    "`1234567890-=qwertyuiop[]\\asdfghjkl;'zxcvbnm,./",
    "~!@#$%^&*()_+QWERTYUIOP{}|ASDFGHJKL:\"ZXCVBNM<>?",
];
const DVORAK: [&str; 2] = [
    "`1234567890[]',.pyfgcrl/=\\aoeuidhtns-;qjkxbmwvz",
    "~!@#$%^&*(){}\"<>PYFGCRL?+|AOEUIDHTNS_:QJKXBMWVZ",
];
impl Layout {
    /// `qwerty` and `dvorak`: US ANSI 47 printable positions, base/shift.
    /// `qwertz`: only QWERTY Y/Z swapped on the same geometry; not German locale.
    pub fn preset(name: &str) -> Result<Self, OperationError> {
        let layers = match name {
            "qwerty" | "qwertz" => QWERTY,
            "dvorak" => DVORAK,
            _ => return Err(invalid("layout must be qwerty, qwertz or dvorak")),
        };
        let mut entries = Vec::new();
        for (layer, text) in ["base", "shift"].into_iter().zip(layers) {
            for (i, mut symbol) in text.chars().enumerate() {
                if name == "qwertz" {
                    symbol = match symbol {
                        'y' => 'z',
                        'z' => 'y',
                        'Y' => 'Z',
                        'Z' => 'Y',
                        _ => symbol,
                    };
                }
                entries.push(Entry {
                    key: format!("ansi{i:02}"),
                    layer: layer.into(),
                    symbol,
                });
            }
        }
        Ok(Self {
            geometry: "us_ansi_printable47".into(),
            entries,
        })
    }
    /// Preset name or strict JSON Layout (<=64 KiB). Unknown/duplicate fields fail.
    pub fn parse(definition: &str) -> Result<Self, OperationError> {
        if !definition.starts_with('{') {
            return Self::preset(definition);
        }
        if definition.len() > 65536 {
            return Err(invalid("layout definition exceeds 64 KiB"));
        }
        serde_json::from_str(definition)
            .map_err(|_| invalid("invalid layout JSON; expected geometry and entries"))
    }
    fn validate(&self) -> Result<BTreeMap<(String, String), char>, OperationError> {
        if self.entries.is_empty()
            || self.entries.len() > 512
            || self.geometry.is_empty()
            || self.geometry.len() > 128
        {
            return Err(invalid("layout needs a geometry and 1..512 entries"));
        }
        // Serialize through a bounded writer rather than allocating an unchecked document.
        let mut bytes = super::mapping::ByteBudget::new(65536);
        serde_json::to_writer(&mut bytes, self)
            .map_err(|_| invalid("layout exceeds 64 KiB serialized bytes"))?;
        let mut keys = BTreeMap::new();
        let mut symbols = BTreeSet::new();
        for entry in &self.entries {
            if entry.key.is_empty()
                || entry.layer.is_empty()
                || entry.key.len() > 128
                || entry.layer.len() > 128
                || keys
                    .insert((entry.key.clone(), entry.layer.clone()), entry.symbol)
                    .is_some()
                || !symbols.insert(entry.symbol)
            {
                return Err(invalid(
                    "layout has duplicate/empty keys, layers or ambiguous symbols",
                ));
            }
        }
        Ok(keys)
    }
}
/// Map exact source scalars by physical key and layer. Both layouts must have
/// identical geometry and key/layer sets, with unique symbols. `unmapped` is
/// preserve (default at interfaces) or error; errors carry the source byte offset.
/// <=1 MiB text, <=512 entries/64 KiB per layout, <=8 MiB output; no normalization.
pub fn remap(
    text: &str,
    source: &Layout,
    target: &Layout,
    unmapped: &str,
) -> Result<String, OperationError> {
    const OP: &str = "text.keyboard.remap";
    check_input(text, OP)?;
    if !["preserve", "error"].contains(&unmapped) {
        return Err(invalid("unmapped must be preserve or error").parameter("unmapped"));
    }
    let source_keys = source.validate()?;
    let target_keys = target.validate()?;
    if source.geometry != target.geometry || !source_keys.keys().eq(target_keys.keys()) {
        return Err(invalid(
            "layouts require identical geometry and key/layer positions",
        ));
    }
    let map: BTreeMap<_, _> = source_keys
        .iter()
        .map(|(key, c)| (*c, target_keys[key]))
        .collect();
    let mut output = String::new();
    for (offset, c) in text.char_indices() {
        let mapped = match map.get(&c) {
            Some(v) => *v,
            None if unmapped == "preserve" => c,
            None => {
                return Err(OperationError::new(
                    ErrorCode::InvalidInput,
                    "unmapped scalar in source layout",
                )
                .operation(OP)
                .offset(offset as u64));
            }
        };
        if output.len() + mapped.len_utf8() > MAX_OUTPUT {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "keyboard output exceeds 8 MiB",
            )
            .operation(OP));
        }
        output.push(mapped);
    }
    Ok(output)
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "source",
        description: "Required preset name or bounded JSON Layout.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "target",
        description: "Required preset name or bounded JSON Layout.",
        required: true,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "unmapped",
        description: "Explicit handling for scalars outside the source layout.",
        required: false,
        default: Some(ParameterValue::Text("preserve")),
        constraints: ParameterType::Text {
            choices: &["preserve", "error"],
        },
    },
];
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: "text.keyboard.remap",
    category: "text",
    description: "Map physical key/layer positions on identical explicit geometries; no locale guessing or normalization.",
    input_type: "text",
    output_type: "text",
    parameters: PARAMETERS,
    deterministic: true,
    reversible: Some(false),
    command: "text keyboard",
    example: "cube text keyboard --source qwerty --target dvorak --text qwerty",
}];
