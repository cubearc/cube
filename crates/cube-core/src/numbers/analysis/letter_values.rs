//! Explicit letter tables and aggregates over original Unicode scalar positions.
use super::{Unsigned, invalid, overflow, parameter, resource};
use crate::{
    errors::OperationError, hashing, numbers::digital::digital_root, text::values::table_index,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    Ordinal,
    ReverseOrdinal,
    Reduction,
    Pythagorean,
    FullReduction,
    ReverseReduction,
    ChaldeanCheiro,
    HebrewOrdinary,
    #[serde(rename = "hebrew_final_500")]
    HebrewFinal500,
    AbjadEastern,
    AbjadMaghrebi,
}

impl Preset {
    const fn id(self) -> &'static str {
        match self {
            Self::Ordinal => "ordinal",
            Self::ReverseOrdinal => "reverse_ordinal",
            Self::Reduction | Self::Pythagorean | Self::FullReduction => "reduction",
            Self::ReverseReduction => "reverse_reduction",
            Self::ChaldeanCheiro => "chaldean_cheiro",
            Self::HebrewOrdinary => "hebrew_ordinary",
            Self::HebrewFinal500 => "hebrew_final_500",
            Self::AbjadEastern => "abjad_eastern",
            Self::AbjadMaghrebi => "abjad_maghrebi",
        }
    }
}

fn values() -> impl Iterator<Item = u64> {
    (1..10)
        .chain((1..10).map(|i| i * 10))
        .chain((1..=10).map(|i| i * 100))
}

/// Returns the exact Unicode scalar table in scalar order.
pub fn preset_table(preset: Preset) -> Vec<(char, u64)> {
    let mut table: Vec<_> = match preset {
        Preset::Ordinal
        | Preset::ReverseOrdinal
        | Preset::Reduction
        | Preset::Pythagorean
        | Preset::FullReduction
        | Preset::ReverseReduction => ('A'..='Z')
            .zip(1..=26)
            .map(|(ch, i)| {
                let value = match preset {
                    Preset::ReverseOrdinal | Preset::ReverseReduction => 27 - i,
                    _ => i,
                };
                let value = match preset {
                    Preset::Reduction
                    | Preset::Pythagorean
                    | Preset::FullReduction
                    | Preset::ReverseReduction => 1 + (value - 1) % 9,
                    _ => value,
                };
                (ch, value)
            })
            .collect(),
        Preset::ChaldeanCheiro => ('A'..='Z')
            .zip([
                1, 2, 3, 4, 5, 8, 3, 5, 1, 1, 2, 3, 4, 5, 7, 8, 1, 2, 3, 4, 6, 6, 6, 5, 1, 7,
            ])
            .collect(),
        Preset::HebrewOrdinary | Preset::HebrewFinal500 => {
            let finals = if matches!(preset, Preset::HebrewFinal500) {
                [500, 600, 700, 800, 900]
            } else {
                [20, 40, 50, 80, 90]
            };
            "אבגדהוזחטיכלמנסעפצקרשת"
                .chars()
                .zip(values())
                .chain("ךםןףץ".chars().zip(finals))
                .collect()
        }
        Preset::AbjadEastern | Preset::AbjadMaghrebi => {
            if matches!(preset, Preset::AbjadMaghrebi) {
                "ابجدهوزحطيكلمنصعفضقرستثخذظغش"
            } else {
                "ابجدهوزحطيكلمنسعفصقرشتثخذضظغ"
            }
            .chars()
            .zip(values())
            .collect()
        }
    };
    table.sort_unstable_by_key(|row| row.0);
    table
}

#[derive(Deserialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
pub enum Mapping {
    Preset {
        preset: Preset,
    },
    Custom {
        #[schemars(length(max = 4096))]
        table: Vec<(char, u64)>,
    },
}
#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Case {
    #[default]
    Exact,
    AsciiUpper,
}
#[derive(Clone, Copy, Default, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unmapped {
    #[default]
    Error,
    Skip,
}
#[derive(Clone, Copy, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Aggregate {
    #[default]
    Sum,
    Product,
    PositionalWeightedSum,
}
const fn max_sum() -> u64 {
    u64::MAX
}

#[derive(Deserialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    Table {
        preset: Preset,
    },
    Evaluate {
        mapping: Mapping,
        text: String,
        #[serde(default)]
        case: Case,
        #[serde(default)]
        unmapped: Unmapped,
        #[serde(default)]
        aggregate: Aggregate,
        #[serde(default)]
        weights: Option<Vec<u64>>,
        #[serde(default = "max_sum")]
        max_sum: u64,
        #[serde(default)]
        reduce: bool,
    },
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Report {
    Table {
        profile: &'static str,
        table: Vec<(char, u64)>,
        table_sha256: String,
    },
    Evaluate {
        profile: &'static str,
        table_sha256: String,
        values: Vec<crate::text::values::CharacterValue>,
        omitted_count: usize,
        sum: u64,
        aggregate: Aggregate,
        value: Unsigned,
        digital_root: Option<u8>,
    },
}

fn hash(table: &[(char, u64)]) -> String {
    let bytes: Vec<_> = table
        .iter()
        .flat_map(|&(scalar, value)| {
            (scalar as u32)
                .to_be_bytes()
                .into_iter()
                .chain(value.to_be_bytes())
        })
        .collect();
    hashing::sha256(&bytes)
}

/// Evaluate one exact table with checked aggregation and explicit omission.
pub fn letter_values(request: &Request) -> Result<Report, OperationError> {
    const OP: &str = "number.letter_values";
    let Request::Evaluate {
        mapping,
        text,
        case,
        unmapped,
        aggregate,
        weights,
        max_sum,
        reduce,
    } = request
    else {
        let Request::Table { preset } = *request else {
            unreachable!()
        };
        let table = preset_table(preset);
        let table_sha256 = hash(&table);
        return Ok(Report::Table {
            profile: preset.id(),
            table,
            table_sha256,
        });
    };
    if text.len() > 1 << 20 {
        return Err(resource("letter input exceeds 1 MiB"));
    }
    let length = text.chars().count();
    if length > 65536 {
        return Err(resource("letter input exceeds 65536 scalar records"));
    }
    if weights.as_ref().is_some_and(|values| {
        !matches!(aggregate, Aggregate::PositionalWeightedSum) || values.len() != length
    }) {
        return Err(parameter(
            "weights require positional_weighted_sum and one weight per source scalar",
        ));
    }
    let (id, preset);
    let table = match mapping {
        Mapping::Preset { preset: profile } => {
            id = profile.id();
            preset = preset_table(*profile);
            &preset
        }
        Mapping::Custom { table } => {
            id = "custom";
            table
        }
    };
    let index = table_index(table, OP)?;
    let ordered: Vec<_> = index.iter().map(|(&ch, &n)| (ch, n)).collect();
    let mut sum = 0_u64;
    let mut omitted_count = 0;
    let mut values = Vec::with_capacity(length);
    for (scalar_index, (byte_offset, scalar)) in text.char_indices().enumerate() {
        let key = match case {
            Case::Exact => scalar,
            Case::AsciiUpper => scalar.to_ascii_uppercase(),
        };
        let value = index.get(&key).copied();
        if let Some(n) = value {
            sum = sum
                .checked_add(n)
                .filter(|n| n <= max_sum)
                .ok_or_else(|| resource("letter value sum exceeds max_sum"))?;
        } else {
            if matches!(unmapped, Unmapped::Error) {
                return Err(invalid("source contains a scalar missing from the table"));
            }
            omitted_count += 1;
        }
        values.push(crate::text::values::CharacterValue {
            scalar_index,
            byte_offset,
            scalar,
            value,
        });
    }
    let value = match aggregate {
        Aggregate::Sum => u128::from(sum),
        Aggregate::Product if values.iter().any(|entry| entry.value == Some(0)) => 0,
        Aggregate::Product => values
            .iter()
            .filter_map(|entry| entry.value)
            .try_fold(1_u128, |product, n| product.checked_mul(u128::from(n)))
            .ok_or_else(overflow)?,
        Aggregate::PositionalWeightedSum => values
            .iter()
            .filter_map(|entry| {
                entry.value.map(|n| {
                    let weight = weights
                        .as_ref()
                        .map_or((entry.scalar_index + 1) as u64, |w| w[entry.scalar_index]);
                    u128::from(n) * u128::from(weight)
                })
            })
            .try_fold(0_u128, u128::checked_add)
            .ok_or_else(overflow)?,
    };
    Ok(Report::Evaluate {
        profile: id,
        table_sha256: hash(&ordered),
        values,
        omitted_count,
        sum,
        aggregate: *aggregate,
        value: Unsigned(value),
        digital_root: reduce.then(|| digital_root(value, 10)).transpose()?,
    })
}
