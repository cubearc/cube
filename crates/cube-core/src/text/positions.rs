//! Bounded position sets, independent of text transformation.
use super::{MAX_INDICES, MAX_INPUT};
use crate::errors::{ErrorCode, OperationError};
use serde::Serialize;

pub(super) const OP: &str = "text.transform.masked";
pub(super) fn invalid(parameter: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
        .operation(OP)
        .parameter(parameter)
}
pub(super) fn check_count(count: usize, parameter: &'static str) -> Result<(), OperationError> {
    if count > MAX_INDICES {
        return Err(OperationError::new(
            ErrorCode::ResourceLimit,
            "position list exceeds 65536 entries",
        )
        .operation(OP)
        .parameter(parameter));
    }
    Ok(())
}

/// Numbering for all scalar positions, including reported positions and overrides.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(into = "u8")]
pub enum IndexBase {
    #[default]
    Zero,
    One,
}
impl From<IndexBase> for u8 {
    fn from(base: IndexBase) -> Self {
        match base {
            IndexBase::Zero => 0,
            IndexBase::One => 1,
        }
    }
}
impl TryFrom<u8> for IndexBase {
    type Error = OperationError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            _ => Err(invalid("index_base", "index base must be 0 or 1")),
        }
    }
}
impl IndexBase {
    pub(super) fn offset(self) -> usize {
        u8::from(self) as usize
    }
    pub(super) fn resolve(
        self,
        position: usize,
        length: usize,
        parameter: &'static str,
    ) -> Result<usize, OperationError> {
        position
            .checked_sub(self.offset())
            .filter(|p| *p < length)
            .ok_or_else(|| {
                invalid(
                    parameter,
                    "scalar position is out of range for the index base",
                )
            })
    }
}

/// Position-set semantics: explicit indexes are sorted/deduplicated, unlike
/// `text::extract`, which preserves list order and duplicates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, schemars::JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Selector {
    Indices {
        indices: Vec<usize>,
    },
    /// The Nth, 2Nth, ... scalar, irrespective of numbering base. N must be positive.
    EveryNth {
        n: usize,
    },
    /// Numeric Fibonacci values 0,1,2,3,5,... serve as position labels. One-based
    /// numbering omits zero; zero-based numbering includes it. Exclusions are
    /// Fibonacci values, not sequence ordinals or an arbitrary position mask.
    Fibonacci {
        exclude: Vec<usize>,
    },
}

fn fibonacci_values(maximum: usize) -> Vec<usize> {
    let mut values = vec![0];
    let (mut previous, mut current) = (0usize, 1usize);
    while current <= maximum {
        if values.last() != Some(&current) {
            values.push(current);
        }
        let Some(sum) = previous.checked_add(current) else {
            break;
        };
        previous = current;
        current = sum;
    }
    values
}
impl Selector {
    /// Adapt named interface options without silently ignoring conflicting fields.
    /// Use enum variants directly in typed Rust code.
    pub fn from_options(
        name: &str,
        indices: Option<Vec<usize>>,
        n: Option<usize>,
        exclude: Option<Vec<usize>>,
    ) -> Result<Self, OperationError> {
        let selector = match (name, indices, n, exclude) {
            ("indices", Some(indices), None, None) => Self::Indices { indices },
            ("every_nth", None, Some(n), None) => Self::EveryNth { n },
            ("fibonacci", None, None, exclude) => Self::Fibonacci {
                exclude: exclude.unwrap_or_default(),
            },
            _ => {
                return Err(invalid(
                    "selector",
                    "use indices with indices, every_nth with nth, or fibonacci with optional exclude; do not mix modes",
                ));
            }
        };
        selector.validate()?;
        Ok(selector)
    }
    /// Validate configuration without requiring input text.
    pub fn validate(&self) -> Result<(), OperationError> {
        match self {
            Self::Indices { indices } => check_count(indices.len(), "indices"),
            Self::EveryNth { n: 0 } => Err(invalid("nth", "nth must be positive")),
            Self::EveryNth { .. } => Ok(()),
            Self::Fibonacci { exclude } => {
                check_count(exclude.len(), "exclude")?;
                let values = fibonacci_values(usize::MAX);
                if exclude.iter().any(|v| values.binary_search(v).is_err()) {
                    return Err(invalid(
                        "exclude",
                        "excluded values must be Fibonacci numbers",
                    ));
                }
                Ok(())
            }
        }
    }
    /// Select sorted unique positions for a scalar length, using the requested
    /// base. All scalars count, including whitespace/punctuation/combining marks.
    /// Length <=1 Mi scalars, configuration and result <=65,536 entries. A result
    /// over the budget is rejected in full, never truncated. No input is normalized.
    pub fn positions(&self, length: usize, base: IndexBase) -> Result<Vec<usize>, OperationError> {
        self.validate()?;
        if length > MAX_INPUT {
            return Err(OperationError::new(
                ErrorCode::ResourceLimit,
                "scalar length exceeds text budget",
            )
            .operation(OP));
        }
        let offset = base.offset();
        let result = match self {
            Self::Indices { indices } => {
                for &index in indices {
                    base.resolve(index, length, "indices")?;
                }
                let mut result = indices.clone();
                result.sort_unstable();
                result.dedup();
                result
            }
            Self::EveryNth { n } => {
                check_count(length / n, "selector")?;
                (0..length)
                    .skip(n - 1)
                    .step_by(*n)
                    .map(|i| i + offset)
                    .collect()
            }
            Self::Fibonacci { exclude } => {
                let mut excluded = exclude.clone();
                excluded.sort_unstable();
                let maximum = length.saturating_sub(1) + offset;
                fibonacci_values(maximum)
                    .into_iter()
                    .filter(|v| length > 0 && *v >= offset && excluded.binary_search(v).is_err())
                    .collect()
            }
        };
        check_count(result.len(), "selector")?;
        Ok(result)
    }
}
