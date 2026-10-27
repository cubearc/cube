//! One validated scalar square for Polybius and the C/K Tap Code convention.
use crate::{
    errors::{ErrorCode, OperationError},
    text::{MAX_INDICES, check_input},
};
use std::collections::BTreeMap;
/// A row/column pair, or a literal ASCII space (`null` in JSON). No guessed pauses.
pub type Token = Option<[u8; 2]>;
/// Immutable validated square; custom alphabets match exact Unicode scalars.
#[derive(Debug)]
pub struct Square {
    side: u8,
    alphabet: Vec<char>,
    positions: BTreeMap<char, usize>,
    ascii: bool,
}
fn error(message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message).parameter("square")
}
impl Square {
    /// Unique side² scalars, side 1..64; aliases are distinct non-cell scalars
    /// pointing directly to a canonical cell. <=4096 aliases, <=64 KiB definitions.
    pub fn new(side: u8, alphabet: &str, aliases: &[(char, char)]) -> Result<Self, OperationError> {
        if !(1..=64).contains(&side)
            || aliases.len() > 4096
            || alphabet.len() > 65536 - aliases.len() * 8
        {
            return Err(error("square exceeds side/definition/alias bounds"));
        }
        let alphabet: Vec<_> = alphabet.chars().collect();
        if alphabet.len() != usize::from(side).pow(2) {
            return Err(error("alphabet must contain exactly side squared scalars"));
        }
        let mut positions = BTreeMap::new();
        for (i, c) in alphabet.iter().copied().enumerate() {
            if c == ' ' || positions.insert(c, i).is_some() {
                return Err(error("square cells must be unique non-space scalars"));
            }
        }
        for &(alias, target) in aliases {
            let index = alphabet
                .iter()
                .position(|c| *c == target)
                .ok_or_else(|| error("alias target must be a canonical cell"))?;
            if alias == ' ' || positions.insert(alias, index).is_some() {
                return Err(error("alias conflicts with another alias or cell"));
            }
        }
        Ok(Self {
            side,
            alphabet,
            positions,
            ascii: false,
        })
    }
    /// `polybius`: ABCDE/FGHIK/LMNOP/QRSTU/VWXYZ, J→I.
    /// `tap`: ABCDE/FGHIJ/LMNOP/QRSTU/VWXYZ, K→C. Both fold ASCII case.
    pub fn preset(name: &str) -> Result<Self, OperationError> {
        let mut square = match name {
            "polybius" => Self::new(5, "ABCDEFGHIKLMNOPQRSTUVWXYZ", &[('J', 'I')])?,
            "tap" => Self::new(5, "ABCDEFGHIJLMNOPQRSTUVWXYZ", &[('K', 'C')])?,
            _ => return Err(error("preset must be polybius or tap")),
        };
        square.ascii = true;
        Ok(square)
    }
    fn base(&self, base: u8) -> Result<(), OperationError> {
        if base > 1 {
            return Err(error("index_base must be 0 or 1").parameter("index_base"));
        }
        Ok(())
    }
    /// Preserve every ASCII space as a distinct token; <=1 MiB and <=65536
    /// total tokens, including spaces. Coordinates are row then column.
    pub fn encode(&self, text: &str, base: u8) -> Result<Vec<Token>, OperationError> {
        const OP: &str = "cipher.polybius.encode";
        check_input(text, OP)?;
        self.base(base).map_err(|e| e.operation(OP))?;
        if text.chars().count() > MAX_INDICES {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "square input exceeds 65536 tokens",
            )
            .operation(OP));
        }
        text.char_indices()
            .map(|(offset, c)| {
                if c == ' ' {
                    return Ok(None);
                }
                let c = if self.ascii {
                    c.to_ascii_uppercase()
                } else {
                    c
                };
                let i = *self.positions.get(&c).ok_or_else(|| {
                    OperationError::new(
                        ErrorCode::InvalidInput,
                        "scalar is not in the square or aliases",
                    )
                    .operation(OP)
                    .offset(offset as u64)
                })?;
                Ok(Some([
                    (i / usize::from(self.side)) as u8 + base,
                    (i % usize::from(self.side)) as u8 + base,
                ]))
            })
            .collect()
    }
    /// Decode bounded structured pairs/spaces; merged presets return canonical cells.
    pub fn decode(&self, tokens: &[Token], base: u8) -> Result<String, OperationError> {
        const OP: &str = "cipher.polybius.decode";
        self.base(base).map_err(|e| e.operation(OP))?;
        if tokens.len() > MAX_INDICES {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "square input exceeds 65536 tokens",
            )
            .operation(OP));
        }
        let mut output = String::new();
        for (i, token) in tokens.iter().enumerate() {
            match token {
                None => output.push(' '),
                Some([r, c]) => {
                    let r = r.checked_sub(base).filter(|r| *r < self.side);
                    let c = c.checked_sub(base).filter(|c| *c < self.side);
                    let (Some(r), Some(c)) = (r, c) else {
                        return Err(OperationError::new(
                            ErrorCode::InvalidInput,
                            "coordinate outside square",
                        )
                        .operation(OP)
                        .parameter("tokens")
                        .offset(i as u64));
                    };
                    output.push(
                        self.alphabet[usize::from(r) * usize::from(self.side) + usize::from(c)],
                    );
                }
            }
        }
        Ok(output)
    }
}
/// Strict CLI grammar: `ROW,COLUMN` or `/`, separated by one ASCII space.
/// Coordinates use unsigned decimal digits only; <=1 MiB and <=65536 tokens.
pub fn parse_tokens(text: &str) -> Result<Vec<Token>, OperationError> {
    const OP: &str = "cipher.polybius.decode";
    check_input(text, OP)?;
    if text.is_empty() {
        return Ok(vec![]);
    }
    let mut result = Vec::new();
    let mut offset = 0;
    for token in text.split(' ') {
        if result.len() == MAX_INDICES {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "square input exceeds 65536 tokens",
            )
            .operation(OP));
        }
        let invalid = || {
            OperationError::new(
                ErrorCode::InvalidInput,
                "expected ROW,COLUMN or /, separated by one ASCII space",
            )
            .operation(OP)
            .offset(offset as u64)
        };
        result.push(if token == "/" {
            None
        } else {
            let (r, c) = token.split_once(',').ok_or_else(invalid)?;
            if r.is_empty()
                || c.is_empty()
                || !r.bytes().all(|b| b.is_ascii_digit())
                || !c.bytes().all(|b| b.is_ascii_digit())
            {
                return Err(invalid());
            }
            Some([
                r.parse().map_err(|_| invalid())?,
                c.parse().map_err(|_| invalid())?,
            ])
        });
        offset += token.len() + 1;
    }
    Ok(result)
}
/// Canonical CLI serialization of bounded coordinate tokens.
pub fn format_tokens(tokens: &[Token]) -> Result<String, OperationError> {
    if tokens.len() > MAX_INDICES {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "square input exceeds 65536 tokens",
        ));
    }
    Ok(tokens
        .iter()
        .map(|t| match t {
            None => "/".into(),
            Some([r, c]) => format!("{r},{c}"),
        })
        .collect::<Vec<_>>()
        .join(" "))
}

use crate::operations::{Descriptor, Parameter, ParameterType, ParameterValue};
const PARAMETERS: &[Parameter] = &[
    Parameter {
        name: "preset",
        description: "I/J Polybius or C/K Tap, mutually exclusive with custom alphabet.",
        required: false,
        default: Some(ParameterValue::Text("polybius")),
        constraints: ParameterType::Text {
            choices: &["polybius", "tap"],
        },
    },
    Parameter {
        name: "alphabet",
        description: "Custom exact unique non-space scalar square; requires side.",
        required: false,
        default: None,
        constraints: ParameterType::Text { choices: &[] },
    },
    Parameter {
        name: "side",
        description: "Custom square side, 1..64.",
        required: false,
        default: None,
        constraints: ParameterType::Unsigned { min: 1, max: 64 },
    },
    Parameter {
        name: "aliases",
        description: "Custom ALIAS=CANONICAL scalar rules.",
        required: false,
        default: None,
        constraints: ParameterType::TextList {
            choices: &[],
            min_length: 0,
            max_length: 4096,
            unique: true,
        },
    },
    Parameter {
        name: "index_base",
        description: "Coordinate origin.",
        required: false,
        default: Some(ParameterValue::Unsigned(1)),
        constraints: ParameterType::UnsignedChoices { choices: &[0, 1] },
    },
];
const PROFILED: [crate::operations::metadata::Entry; 2] = [
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.polybius.encode",
            category: "ciphers",
            description: "Shared Polybius/Tap scalar square; row,column and explicit space tokens, <=65536 tokens.",
            input_type: "text",
            output_type: "polybius.tokens",
            parameters: PARAMETERS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher polybius",
            example: "cube cipher polybius --text A",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Polybius square",
                family: "coordinate_substitution",
                aliases: &[
                    crate::operations::metadata::Alias {
                        name: "Tap Code",
                        usage: "Use Square::preset(\"tap\") / preset=tap; coordinate tokens, not an audio recognizer.",
                    },
                    crate::operations::metadata::Alias {
                        name: "Polybius Coordinates",
                        usage: "Use the same explicit row-column token profile.",
                    },
                ],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "polybius preset merges J into I; tap preset merges K into C. Both fold ASCII case. Custom squares use exact unique non-space scalars and explicit aliases.",
                    key: "Explicit side 2..64 and side-squared alphabet, or named preset; row then column, index_base 0/1.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "tokens",
                        maximum: crate::text::MAX_INDICES as u64,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "definition_bytes",
                        maximum: 65536,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "aliases",
                        maximum: 4096,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "input_utf8_bytes",
                        maximum: crate::text::MAX_INPUT as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. Token text parsing additionally caps raw UTF-8 at 1 MiB; definitions charge eight bytes per alias. Presets/custom mappings can canonicalize symbols and are not general inverses of aliases.",
                rust_api: "cube_core::ciphers::polybius::Square::encode",
                python_api: "polybius_encode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
    {
        const DECLARATION: crate::operations::Descriptor = Descriptor {
            id: "cipher.polybius.decode",
            category: "ciphers",
            description: "Shared Polybius/Tap scalar square; row,column and explicit space tokens, <=65536 tokens.",
            input_type: "polybius.tokens",
            output_type: "text",
            parameters: PARAMETERS,
            deterministic: true,
            reversible: Some(false),
            command: "cipher polybius",
            example: "cube cipher polybius --decode --text 1,1",
        };
        crate::operations::metadata::Entry {
            descriptor: DECLARATION,
            metadata: Some(crate::operations::metadata::Metadata {
                schema_version: 1,
                canonical_name: "Polybius square",
                family: "coordinate_substitution",
                aliases: &[
                    crate::operations::metadata::Alias {
                        name: "Tap Code",
                        usage: "Use Square::preset(\"tap\") / preset=tap; coordinate tokens, not an audio recognizer.",
                    },
                    crate::operations::metadata::Alias {
                        name: "Polybius Coordinates",
                        usage: "Use the same explicit row-column token profile.",
                    },
                ],
                security: crate::operations::metadata::Security {
                    classification: crate::operations::metadata::Classification::HistoricalCipher,
                    limitations: "Not secure encryption; use only for historical interoperability, education and challenges. No authentication or certification claim.",
                    private_input: false,
                    private_output: false,
                    sensitive_parameters: &[],
                },
                profile: crate::operations::metadata::Profile {
                    variant: DECLARATION.description,
                    alphabet: "polybius preset merges J into I; tap preset merges K into C. Both fold ASCII case. Custom squares use exact unique non-space scalars and explicit aliases.",
                    key: "Explicit side 2..64 and side-squared alphabet, or named preset; row then column, index_base 0/1.",
                    key_advancement: "Not applicable.",
                    block_bits: None,
                    output_bits: None,
                },
                resources: &[
                    crate::operations::metadata::ResourceBound {
                        name: "tokens",
                        maximum: crate::text::MAX_INDICES as u64,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "definition_bytes",
                        maximum: 65536,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "aliases",
                        maximum: 4096,
                        unit: "items",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                    crate::operations::metadata::ResourceBound {
                        name: "output_utf8_bytes",
                        maximum: 4 * crate::text::MAX_INDICES as u64,
                        unit: "bytes",
                        scope: crate::operations::metadata::Scope::Core,
                    },
                ],
                resource_notes: "Bounds apply to direct Core calls and their CLI/native-Python wrappers. Output bounds may be mathematical upper bounds rather than independent budget options. Token text parsing additionally caps raw UTF-8 at 1 MiB; definitions charge eight bytes per alias. Presets/custom mappings can canonicalize symbols and are not general inverses of aliases.",
                rust_api: "cube_core::ciphers::polybius::Square::decode",
                python_api: "polybius_decode",
                references: &["https://www.cryptogram.org/resource-area/cipher-types/"],
            }),
        }
    },
];
pub const OPERATIONS: &[crate::operations::Descriptor] =
    &crate::operations::metadata::descriptors(&PROFILED);
pub(crate) const MODULE: crate::operations::Module =
    crate::operations::Module::profiled(OPERATIONS, &PROFILED);
