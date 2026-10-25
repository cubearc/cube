//! Conditional scalar transformations with explicit, auditable manual overrides.
use super::{
    check_input,
    positions::{IndexBase, OP, Selector, check_count, invalid},
};
use crate::{
    ciphers,
    errors::OperationError,
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::Serialize;

/// Only existing ASCII ciphers that preserve scalar count and order are permitted.
/// All non-ASCII scalars are preserved exactly; no normalization is performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Transformation {
    Atbash,
    Caesar { shift: i32 },
    Rot13,
    Identity,
}
impl Transformation {
    /// Caesar requires an explicit signed shift; other modes reject a shift.
    pub fn parse(name: &str, shift: Option<i32>) -> Result<Self, OperationError> {
        match (name, shift) {
            ("atbash", None) => Ok(Self::Atbash),
            ("caesar", Some(shift)) => Ok(Self::Caesar { shift }),
            ("rot13", None) => Ok(Self::Rot13),
            ("identity", None) => Ok(Self::Identity),
            _ => Err(invalid(
                "transformation",
                "use atbash, rot13 or identity without a shift, or caesar with an explicit shift",
            )),
        }
    }
    fn apply(self, input: &str) -> Result<String, OperationError> {
        Ok(match self {
            Self::Atbash => ciphers::atbash(input)?,
            Self::Caesar { shift } => ciphers::caesar_encode(input, shift),
            Self::Rot13 => ciphers::rot13(input),
            Self::Identity => input.to_owned(),
        })
    }
}

/// An explicit replacement at a position in the original scalar numbering.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Override {
    pub position: usize,
    pub replacement: char,
}
impl Override {
    /// Validate that an interface replacement contains exactly one Unicode scalar.
    pub fn new(position: usize, replacement: &str) -> Result<Self, OperationError> {
        let mut chars = replacement.chars();
        let replacement = chars
            .next()
            .filter(|_| chars.next().is_none())
            .ok_or_else(|| {
                invalid(
                    "overrides",
                    "replacement must contain exactly one Unicode scalar",
                )
            })?;
        Ok(Self {
            position,
            replacement,
        })
    }
    /// CLI record syntax is POSITION=SCALAR. The scalar may itself be `=`.
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        let (position, replacement) = value
            .split_once('=')
            .ok_or_else(|| invalid("overrides", "override must be POSITION=SCALAR"))?;
        let position = position
            .parse()
            .map_err(|_| invalid("overrides", "override position must be an unsigned integer"))?;
        Self::new(position, replacement)
    }
}

/// A complete request. Every Unicode scalar counts toward positions. Bases apply
/// to selector labels, reported positions and overrides alike.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Config {
    pub index_base: IndexBase,
    pub selector: Selector,
    pub selected: Transformation,
    pub otherwise: Transformation,
    /// Applied in caller order; repeated positions are allowed and recorded separately.
    pub overrides: Vec<Override>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AppliedOverride {
    pub position: usize,
    pub previous: char,
    pub replacement: char,
}
/// The deterministic output remains available even when explicit guesses are applied.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub original_input: String,
    pub config: Config,
    pub selected_positions: Vec<usize>,
    pub deterministic_text: String,
    pub overrides: Vec<AppliedOverride>,
    pub final_text: String,
}

/// Transform selected and other scalars with existing ciphers, then apply explicit
/// overrides. All parameters validate before cipher execution. Input <=1 MiB;
/// explicit lists, selected positions and overrides <=65,536 each. Final output
/// <=4 MiB (one replacement scalar per input scalar), within the 8 MiB text budget.
///
/// ```
/// use cube_core::text::{positions::{IndexBase, Selector}, transform::{self, Config, Transformation}};
/// let report = transform::masked("ABCDEFGHIJKLMNOPQRSTUVWXYZ", &Config {
///     index_base: IndexBase::One,
///     selector: Selector::Fibonacci { exclude: vec![13, 21] },
///     selected: Transformation::Atbash,
///     otherwise: Transformation::Caesar { shift: 3 },
///     overrides: vec![],
/// }).unwrap();
/// assert_eq!(report.final_text, "ZYXGVIJSLMNOPQRSTUVWXYZABC");
/// ```
pub fn masked(input: &str, config: &Config) -> Result<Report, OperationError> {
    check_input(input, OP)?;
    check_count(config.overrides.len(), "overrides")?;
    let length = input.chars().count();
    let positions = config.selector.positions(length, config.index_base)?;
    for item in &config.overrides {
        config
            .index_base
            .resolve(item.position, length, "overrides")?;
    }
    let mut mask = vec![false; length];
    for position in &positions {
        mask[position - config.index_base.offset()] = true;
    }
    // Partition once so each existing cipher is invoked once, without per-scalar allocations.
    let (mut selected, mut otherwise) = (String::new(), String::new());
    for (character, selected_position) in input.chars().zip(&mask) {
        if *selected_position {
            selected.push(character);
        } else {
            otherwise.push(character);
        }
    }
    let selected = config.selected.apply(&selected)?;
    let otherwise = config.otherwise.apply(&otherwise)?;
    let (mut selected, mut otherwise) = (selected.chars(), otherwise.chars());
    let mut final_chars: Vec<char> = mask
        .into_iter()
        .map(|p| {
            if p { selected.next() } else { otherwise.next() }
                .expect("permitted ciphers preserve scalar count")
        })
        .collect();
    let deterministic_text = final_chars.iter().collect();
    let mut overrides = Vec::with_capacity(config.overrides.len());
    for item in &config.overrides {
        let index = item.position - config.index_base.offset();
        overrides.push(AppliedOverride {
            position: item.position,
            previous: final_chars[index],
            replacement: item.replacement,
        });
        final_chars[index] = item.replacement;
    }
    Ok(Report {
        original_input: input.into(),
        config: config.clone(),
        selected_positions: positions,
        deterministic_text,
        overrides,
        final_text: final_chars.into_iter().collect(),
    })
}

const LIST: ParameterType = ParameterType::UnsignedList {
    min: 0,
    max: usize::MAX as u64,
    min_length: 0,
    max_length: super::MAX_INDICES,
    unique: false,
};
pub const OPERATIONS: &[Descriptor] = &[Descriptor {
    id: OP,
    category: "text",
    description: "Apply existing ASCII ciphers to bounded scalar position sets and their complement; every scalar counts, no normalization. Explicit overrides are audited separately.",
    input_type: "text",
    output_type: "text",
    parameters: &[
        Parameter {
            name: "selector",
            description: "Exactly one mode; indices requires indices, every_nth requires nth, fibonacci permits exclude only.",
            required: true,
            default: None,
            constraints: ParameterType::Text {
                choices: &["indices", "every_nth", "fibonacci"],
            },
        },
        Parameter {
            name: "index_base",
            description: "Scalar position numbering, including overrides. Every scalar counts.",
            required: false,
            default: Some(ParameterValue::Integer(0)),
            constraints: ParameterType::Integer { min: 0, max: 1 },
        },
        Parameter {
            name: "indices",
            description: "Explicit scalar positions, sorted/deduplicated as a set; out-of-range positions fail.",
            required: false,
            default: None,
            constraints: LIST,
        },
        Parameter {
            name: "nth",
            description: "Select the Nth, 2Nth, ... scalar; independent of numbering base.",
            required: false,
            default: None,
            constraints: ParameterType::Unsigned {
                min: 1,
                max: usize::MAX as u64,
            },
        },
        Parameter {
            name: "exclude",
            description: "Excluded Fibonacci numeric values, not sequence ordinals or arbitrary position exclusions. Zero is omitted in base 1.",
            required: false,
            default: None,
            constraints: LIST,
        },
        Parameter {
            name: "selected",
            description: "Transformation for selected scalars; Caesar requires selected_shift.",
            required: true,
            default: None,
            constraints: ParameterType::Text {
                choices: &["atbash", "caesar", "rot13", "identity"],
            },
        },
        Parameter {
            name: "selected_shift",
            description: "Required only for selected Caesar; full signed i32 range, modulo 26.",
            required: false,
            default: None,
            constraints: ParameterType::Integer {
                min: i32::MIN as i64,
                max: i32::MAX as i64,
            },
        },
        Parameter {
            name: "otherwise",
            description: "Transformation for unselected scalars; Caesar requires otherwise_shift.",
            required: false,
            default: Some(ParameterValue::Text("identity")),
            constraints: ParameterType::Text {
                choices: &["atbash", "caesar", "rot13", "identity"],
            },
        },
        Parameter {
            name: "otherwise_shift",
            description: "Required only for otherwise Caesar; full signed i32 range, modulo 26.",
            required: false,
            default: None,
            constraints: ParameterType::Integer {
                min: i32::MIN as i64,
                max: i32::MAX as i64,
            },
        },
        Parameter {
            name: "overrides",
            description: "Ordered explicit POSITION=SCALAR replacements after deterministic transformation; repeated positions are audited separately.",
            required: false,
            default: None,
            constraints: ParameterType::TextList {
                choices: &[],
                min_length: 0,
                max_length: super::MAX_INDICES,
                unique: false,
            },
        },
    ],
    deterministic: true,
    reversible: Some(false),
    command: "text transform masked",
    example: "cube text transform masked --text ABCDEFGHIJKLMNOPQRSTUVWXYZ --selector fibonacci --exclude 13,21 --selected atbash --otherwise caesar --otherwise-shift 3 --index-base 1",
}];
