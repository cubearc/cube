//! Ordered keypad groups; telephone alternatives never expand into guessed words.
use crate::{
    errors::{ErrorCode, OperationError},
    text::{MAX_INDICES, MAX_OUTPUT, check_input, mapping::ByteBudget},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
/// One digit key and its ordered exact character alternatives.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub digit: char,
    pub symbols: String,
}
/// Custom mapping must explicitly choose ASCII folding or exact scalar identity.
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub ascii_fold: bool,
    pub keys: Vec<Key>,
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Decoded {
    pub text: Option<String>,
    pub alternatives: Vec<String>,
    pub reversible: bool,
}
fn invalid(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).parameter("mapping")
}
type Forward = BTreeMap<char, (char, usize)>;
type Reverse = BTreeMap<char, Vec<char>>;
impl Mapping {
    /// ITU E.161 telephone letters: ABC=2 ... PQRS=7, TUV=8, WXYZ=9.
    /// No default spaces or literal digits; custom disjoint sets can provide them.
    pub fn standard() -> Self {
        Self {
            ascii_fold: true,
            keys: ["ABC", "DEF", "GHI", "JKL", "MNO", "PQRS", "TUV", "WXYZ"]
                .iter()
                .enumerate()
                .map(|(i, s)| Key {
                    digit: (b'2' + i as u8) as char,
                    symbols: (*s).into(),
                })
                .collect(),
        }
    }
    /// Strict JSON Mapping, <=64 KiB before parsing.
    pub fn parse(definition: &str) -> Result<Self, OperationError> {
        if definition.len() > 65536 {
            return Err(invalid("mapping exceeds 64 KiB"));
        }
        serde_json::from_str(definition).map_err(|_| invalid("invalid keypad mapping JSON"))
    }
    fn validated(&self) -> Result<(Forward, Reverse), OperationError> {
        if self.keys.is_empty() || self.keys.len() > 10 {
            return Err(invalid("mapping needs 1..10 digit keys"));
        }
        let mut budget = ByteBudget::new(65536);
        serde_json::to_writer(&mut budget, self).map_err(|_| invalid("mapping exceeds 64 KiB"))?;
        let mut forward = BTreeMap::new();
        let mut reverse = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for key in &self.keys {
            if !key.digit.is_ascii_digit()
                || reverse.contains_key(&key.digit)
                || key.symbols.is_empty()
            {
                return Err(invalid(
                    "keys must be unique digits with nonempty symbol sets",
                ));
            }
            let mut symbols = Vec::new();
            for c in key.symbols.chars() {
                if forward.len() == 256 {
                    return Err(invalid("mapping exceeds 256 scalars"));
                }
                let c = if self.ascii_fold {
                    c.to_ascii_uppercase()
                } else {
                    c
                };
                if !seen.insert(c) {
                    return Err(invalid("duplicate character after configured folding"));
                }
                forward.insert(c, (key.digit, symbols.len() + 1));
                symbols.push(c);
            }
            reverse.insert(key.digit, symbols);
        }
        Ok((forward, reverse))
    }
    /// Encode telephone as contiguous digits; multi_tap as explicitly `|`-delimited
    /// repeated-digit groups. <=1 MiB input, <=65536 characters, <=8 MiB output.
    pub fn encode(&self, text: &str, mode: &str) -> Result<String, OperationError> {
        const OP: &str = "symbol.keypad.encode";
        check_input(text, OP)?;
        mode_check(mode).map_err(|e| e.operation(OP))?;
        let (forward, _) = self.validated().map_err(|e| e.operation(OP))?;
        if text.chars().count() > MAX_INDICES {
            return Err(limit(OP));
        }
        let mut output = String::new();
        for (i, (offset, c)) in text.char_indices().enumerate() {
            let c = if self.ascii_fold {
                c.to_ascii_uppercase()
            } else {
                c
            };
            let &(digit, count) = forward.get(&c).ok_or_else(|| {
                OperationError::new(ErrorCode::InvalidInput, "unsupported keypad scalar")
                    .operation(OP)
                    .offset(offset as u64)
            })?;
            let count = if mode == "telephone" { 1 } else { count };
            let extra = count + usize::from(mode == "multi_tap" && i > 0);
            if output.len() + extra > MAX_OUTPUT {
                return Err(limit(OP));
            }
            if mode == "multi_tap" && i > 0 {
                output.push('|');
            }
            output.extend(std::iter::repeat_n(digit, count));
        }
        Ok(output)
    }
    /// Telephone returns per-digit alternatives, text=null and reversible=false.
    /// Multi-tap returns canonical text and empty alternatives, reversible=true.
    /// This flag concerns token inversion, not restoration of input letter case.
    pub fn decode(&self, text: &str, mode: &str) -> Result<Decoded, OperationError> {
        const OP: &str = "symbol.keypad.decode";
        check_input(text, OP)?;
        mode_check(mode).map_err(|e| e.operation(OP))?;
        let (_, reverse) = self.validated().map_err(|e| e.operation(OP))?;
        let mut result = Decoded {
            text: if mode == "multi_tap" {
                Some(String::new())
            } else {
                None
            },
            alternatives: vec![],
            reversible: mode == "multi_tap",
        };
        if text.is_empty() {
            return Ok(result);
        }
        let tokens: Box<dyn Iterator<Item = &str>> = if mode == "telephone" {
            Box::new(text.char_indices().map(|(i, c)| &text[i..i + c.len_utf8()]))
        } else {
            Box::new(text.split('|'))
        };
        let mut offset = 0;
        let mut output_bytes = 0;
        for (i, token) in tokens.enumerate() {
            if i == MAX_INDICES {
                return Err(limit(OP));
            }
            let invalid = || {
                OperationError::new(ErrorCode::InvalidInput,"invalid key/count; multi_tap requires one | separated repeated-digit group per scalar").operation(OP).offset(offset as u64)
            };
            let digit = token.chars().next().ok_or_else(invalid)?;
            let symbols = reverse.get(&digit).ok_or_else(invalid)?;
            if !token.chars().all(|c| c == digit) {
                return Err(invalid());
            }
            if mode == "telephone" {
                let alternatives: String = symbols.iter().collect();
                output_bytes += alternatives.len();
                if output_bytes > MAX_OUTPUT {
                    return Err(limit(OP));
                }
                result.alternatives.push(alternatives);
            } else {
                let c = symbols.get(token.len() - 1).ok_or_else(invalid)?;
                result.text.as_mut().expect("multi-tap text").push(*c);
            }
            offset += token.len() + usize::from(mode == "multi_tap");
        }
        let mut budget = ByteBudget::new(MAX_OUTPUT);
        serde_json::to_writer(&mut budget, &result).map_err(|_| limit(OP))?;
        Ok(result)
    }
}
fn mode_check(mode: &str) -> Result<(), OperationError> {
    if ["telephone", "multi_tap"].contains(&mode) {
        Ok(())
    } else {
        Err(invalid("mode must be telephone or multi_tap").parameter("mode"))
    }
}
fn limit(op: &'static str) -> OperationError {
    OperationError::new(
        ErrorCode::ResourceLimit,
        "keypad exceeds 65536 groups or 8 MiB output/report",
    )
    .operation(op)
}

use crate::operations::{Descriptor, Parameter, ParameterType};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "mode",
        description: "Telephone alternatives or delimited multi-tap.",
        required: true,
        default: None,
        constraints: ParameterType::Text {
            choices: &["telephone", "multi_tap"],
        },
    },
    Parameter {
        name: "mapping",
        description: "Optional bounded JSON Mapping with explicit ascii_fold and ordered digit/symbol sets.",
        required: false,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
];
const PROFILED: [crate::operations::metadata::Entry; 2] = [
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "symbol.keypad.encode",
            category: "symbols",
            description: "Explicit telephone or multi-tap groups; decode telephone alternatives without word guessing; <=65536 groups.",
            input_type: "text",
            output_type: "text",
            parameters: PARAMETERS,
            deterministic: true,
            reversible: Some(false),
            command: "encode keypad",
            example: "cube encode keypad --mode multi_tap --text H",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Explicit keypad mapping",
                family: "puzzle_representations",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::PuzzleCode,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Caller-selected exact groups or the built-in telephone table; explicit multi-tap/T9 mode and case policy. T9 returns alternatives without dictionary inference.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "symbols_or_groups",
                        maximum: MAX_INDICES as u64,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "mapping_bytes",
                        maximum: 65536,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "mapping_keys",
                        maximum: 10,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "mapping_scalars",
                        maximum: 256,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_bytes",
                        maximum: crate::text::MAX_OUTPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. These are the documented supported subsets, not linguistic decipherment, predictive text or a general secure cipher.",
                rust_api: "cube_core::symbols::keypad::Mapping::encode",
                python_api: "keypad_encode",
                references: &["https://www.itu.int/rec/T-REC-E.161-200102-I/en"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "symbol.keypad.decode",
            category: "symbols",
            description: "Explicit telephone or multi-tap groups; decode telephone alternatives without word guessing; <=65536 groups.",
            input_type: "text",
            output_type: "keypad.decoded",
            parameters: PARAMETERS,
            deterministic: true,
            reversible: Some(false),
            command: "decode keypad",
            example: "cube decode keypad --mode multi_tap --text 44",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Explicit keypad mapping",
                family: "puzzle_representations",
                aliases: &[],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::PuzzleCode,
                    limitations: "Representation only; encoding and corruption checks do not provide secrecy or authenticity.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "Caller-selected exact groups or the built-in telephone table; explicit multi-tap/T9 mode and case policy. T9 returns alternatives without dictionary inference.",
                    key: "No key.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "symbols_or_groups",
                        maximum: MAX_INDICES as u64,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "mapping_bytes",
                        maximum: 65536,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "mapping_keys",
                        maximum: 10,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "mapping_scalars",
                        maximum: 256,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_bytes",
                        maximum: crate::text::MAX_OUTPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. These are the documented supported subsets, not linguistic decipherment, predictive text or a general secure cipher.",
                rust_api: "cube_core::symbols::keypad::Mapping::decode",
                python_api: "keypad_decode",
                references: &["https://www.itu.int/rec/T-REC-E.161-200102-I/en"],
            }),
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
