//! Static operation metadata; direct Rust functions remain the execution API.
//!
//! IDs and type names are compatibility contracts. Add fields compatibly and do
//! not reuse an ID for different semantics. Categories are strings, so extending
//! the catalog does not require changing a central category enum.

pub mod contracts;
mod discovery;
pub mod metadata;
pub use discovery::{
    DiscoveryFormat, DiscoveryRequest, MAX_DISCOVERY_BYTES, MAX_DISCOVERY_QUERY_BYTES,
    discovery_document,
};
use serde::Serialize;

/// A parameter value used only for metadata and validation, not runtime dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ParameterValue<'a> {
    /// Signed integer.
    Integer(i64),
    /// Unsigned integer, including full-width byte offsets.
    Unsigned(u64),
    /// Boolean flag.
    Boolean(bool),
    /// UTF-8 text.
    Text(&'a str),
    /// Exact bytes; JSON represents these as an integer array.
    Bytes(&'a [u8]),
    /// Ordered text choices, such as image channels.
    TextList(&'a [&'a str]),
    /// Ordered integer choices or a fixed coordinate tuple.
    UnsignedList(&'a [u64]),
}

impl std::fmt::Display for ParameterValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer(value) => value.fmt(f),
            Self::Unsigned(value) => value.fmt(f),
            Self::Boolean(value) => value.fmt(f),
            Self::Text(value) => write!(f, "{value:?}"),
            Self::Bytes(value) => write!(f, "hex:{}", hex::encode(value)),
            Self::TextList(value) => write!(f, "{}", value.join(", ")),
            Self::UnsignedList(value) => write!(f, "{value:?}"),
        }
    }
}

/// Parameter constraints, shared by discovery and explicit preflight validation.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParameterType {
    /// Inclusive integer range.
    Integer { min: i64, max: i64 },
    /// Inclusive unsigned range for byte offsets/counts.
    Unsigned { min: u64, max: u64 },
    /// Finite unsigned choices, such as exact quarter-turn angles.
    UnsignedChoices { choices: &'static [u64] },
    /// Boolean flag.
    Boolean,
    /// UTF-8 text; an empty list permits any text.
    Text { choices: &'static [&'static str] },
    /// Exact bytes with a minimum length.
    Bytes { min_length: usize },
    /// Ordered text values with explicit length and uniqueness constraints.
    TextList {
        choices: &'static [&'static str],
        min_length: usize,
        max_length: usize,
        unique: bool,
    },
    /// Ordered unsigned values with explicit length, range and uniqueness rules.
    UnsignedList {
        min: u64,
        max: u64,
        min_length: usize,
        max_length: usize,
        unique: bool,
    },
}

impl std::fmt::Display for ParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Integer { min, max } => write!(f, "integer in [{min}, {max}]"),
            Self::Unsigned { min, max } => write!(f, "unsigned integer in [{min}, {max}]"),
            Self::UnsignedChoices { choices } => write!(f, "unsigned integer, one of: {choices:?}"),
            Self::Boolean => f.write_str("boolean"),
            Self::Text { choices: [] } => f.write_str("UTF-8 text"),
            Self::Text { choices } => write!(f, "one of: {}", choices.join(", ")),
            Self::Bytes { min_length } => write!(f, "bytes, minimum length {min_length}"),
            Self::TextList {
                choices,
                min_length,
                max_length,
                unique,
            } => write!(
                f,
                "ordered text list, length [{min_length}, {max_length}], unique={unique}, choices: {}",
                choices.join(", ")
            ),
            Self::UnsignedList {
                min,
                max,
                min_length,
                max_length,
                unique,
            } => write!(
                f,
                "ordered unsigned list in [{min}, {max}], length [{min_length}, {max_length}], unique={unique}"
            ),
        }
    }
}

impl ParameterType {
    fn accepts(self, value: ParameterValue<'_>) -> bool {
        match (self, value) {
            (Self::UnsignedChoices { choices }, ParameterValue::Unsigned(n)) => {
                choices.contains(&n)
            }
            (Self::UnsignedChoices { choices }, ParameterValue::Integer(n)) => {
                u64::try_from(n).is_ok_and(|n| choices.contains(&n))
            }
            (Self::Integer { min, max }, ParameterValue::Integer(n)) => (min..=max).contains(&n),
            (Self::Unsigned { min, max }, ParameterValue::Unsigned(n)) => (min..=max).contains(&n),
            (Self::Unsigned { min, max }, ParameterValue::Integer(n)) => {
                u64::try_from(n).is_ok_and(|n| (min..=max).contains(&n))
            }
            (Self::Boolean, ParameterValue::Boolean(_)) => true,
            (Self::Text { choices }, ParameterValue::Text(text)) => {
                choices.is_empty() || choices.contains(&text)
            }
            (Self::Bytes { min_length }, ParameterValue::Bytes(bytes)) => bytes.len() >= min_length,
            (
                Self::TextList {
                    choices,
                    min_length,
                    max_length,
                    unique,
                },
                ParameterValue::TextList(values),
            ) => {
                (min_length..=max_length).contains(&values.len())
                    && values.iter().enumerate().all(|(i, v)| {
                        (choices.is_empty() || choices.contains(v))
                            && (!unique || !values[..i].contains(v))
                    })
            }
            (
                Self::UnsignedList {
                    min,
                    max,
                    min_length,
                    max_length,
                    unique,
                },
                ParameterValue::UnsignedList(values),
            ) => {
                (min_length..=max_length).contains(&values.len())
                    && values.iter().enumerate().all(|(i, v)| {
                        (min..=max).contains(v) && (!unique || !values[..i].contains(v))
                    })
            }
            _ => false,
        }
    }
}

/// One named operation parameter. Defaults are explicit and never applied silently.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Parameter {
    pub name: &'static str,
    pub description: &'static str,
    pub required: bool,
    pub default: Option<ParameterValue<'static>>,
    pub constraints: ParameterType,
}

/// Describes an implemented operation without imposing an execution hierarchy.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Descriptor {
    pub id: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    /// Stable semantic type name: `bytes`, `text`, or a named structured result.
    pub input_type: &'static str,
    pub output_type: &'static str,
    pub parameters: &'static [Parameter],
    pub deterministic: bool,
    /// `None` means reversibility is not meaningful for this operation.
    pub reversible: Option<bool>,
    /// Existing human CLI spelling; multiple IDs may share a parameterized command.
    pub command: &'static str,
    pub example: &'static str,
}

/// Pipeline execution availability derived from the executor's actual allowlist.
/// The version identifies the adapter contract, independently of package version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PipelineSupport {
    pub version: u32,
}

/// An additive discovery view; the descriptor and its public layout stay intact.
/// JSON retains every descriptor field and adds `pipeline`: a version object for
/// a bounded adapter, or null when no adapter exists. This is not a dispatcher.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Discovery<'a> {
    #[serde(flatten)]
    pub descriptor: &'a Descriptor,
    pub pipeline: Option<PipelineSupport>,
}

/// Opt-in additive view. Legacy Discovery serialization and literal construction
/// stay unchanged, including for consumers that reject unknown JSON fields.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ProfiledDiscovery<'a> {
    #[serde(flatten)]
    pub discovery: Discovery<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<&'static metadata::Metadata>,
}
/// Explicit opt-in; legacy and profile-only serialization remain intact.
#[derive(Clone, Debug, Serialize)]
pub struct ContractDiscovery<'a> {
    #[serde(flatten)]
    pub profiled: ProfiledDiscovery<'a>,
    pub contract: contracts::Contract,
}
/// Only the existing catalog registration combines descriptor slices and their
/// optional owning-module annotations. This is not an execution registry.
pub(crate) struct Module {
    descriptors: &'static [Descriptor],
    profiles: &'static [metadata::Entry],
    contracts: &'static [contracts::Entry],
}
impl Module {
    pub(crate) const fn with_contracts(mut self, contracts: &'static [contracts::Entry]) -> Self {
        self.contracts = contracts;
        self
    }
    const fn plain(descriptors: &'static [Descriptor]) -> Self {
        Self {
            descriptors,
            profiles: &[],
            contracts: &[],
        }
    }
    pub(crate) const fn profiled(
        descriptors: &'static [Descriptor],
        profiles: &'static [metadata::Entry],
    ) -> Self {
        Self {
            descriptors,
            profiles,
            contracts: &[],
        }
    }
}
/// A precise parameter validation failure, without including its input value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParameterError {
    pub operation: &'static str,
    pub parameter: String,
    pub reason: &'static str,
}

impl std::fmt::Display for ParameterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: parameter '{}': {}",
            self.operation, self.parameter, self.reason
        )
    }
}

impl std::error::Error for ParameterError {}

impl Descriptor {
    /// Complete canonical contract. Unknown schemas fail explicitly.
    pub fn contract(&self) -> Result<contracts::Contract, crate::errors::OperationError> {
        contracts::describe(self)
    }
    /// Add complete typed contracts without changing legacy discovery JSON.
    pub fn contract_discovery(
        &self,
    ) -> Result<ContractDiscovery<'_>, crate::errors::OperationError> {
        Ok(ContractDiscovery {
            profiled: self.profiled_discovery(),
            contract: self.contract()?,
        })
    }
    /// Opt in to verified profile annotations without changing the legacy view.
    pub fn profiled_discovery(&self) -> ProfiledDiscovery<'_> {
        ProfiledDiscovery {
            discovery: self.discovery(),
            metadata: self.metadata(),
        }
    }
    /// Facts apply only to the complete owning declaration, including exact
    /// copies. A caller-created descriptor reusing an ID with changed semantics
    /// does not inherit another operation's profile or security claims.
    pub fn metadata(&self) -> Option<&'static metadata::Metadata> {
        MODULES
            .iter()
            .flat_map(|m| m.profiles)
            .find(|row| row.descriptor == *self)
            .and_then(|row| row.metadata.as_ref())
    }

    /// Describe execution availability without copying metadata or executing code.
    /// Direct Rust APIs and the declared CLI command remain independent of the
    /// pipeline allowlist. SDK entry points are documented in their native stubs.
    pub fn discovery(&self) -> Discovery<'_> {
        Discovery {
            descriptor: self,
            pipeline: crate::pipeline::adapter_version(self.id)
                .map(|version| PipelineSupport { version }),
        }
    }

    /// Validate supplied values without executing an operation or allocating defaults.
    /// Unknown and duplicate names, missing required values and invalid types/ranges
    /// are errors. Optional defaults remain visible in the descriptor.
    pub fn validate(&self, values: &[(&str, ParameterValue<'_>)]) -> Result<(), ParameterError> {
        let error = |name: &str, reason| ParameterError {
            operation: self.id,
            parameter: name.to_owned(),
            reason,
        };
        for (index, (name, value)) in values.iter().enumerate() {
            let parameter = self
                .parameters
                .iter()
                .find(|p| p.name == *name)
                .ok_or_else(|| error(name, "unknown parameter"))?;
            if values[..index].iter().any(|(previous, _)| previous == name) {
                return Err(error(name, "duplicate parameter"));
            }
            if !parameter.constraints.accepts(*value) {
                return Err(error(
                    name,
                    "invalid type or value; see operation constraints",
                ));
            }
        }
        for parameter in self.parameters {
            if parameter.required
                && parameter.default.is_none()
                && !values.iter().any(|(name, _)| *name == parameter.name)
            {
                return Err(error(parameter.name, "required parameter is missing"));
            }
        }
        Ok(())
    }
}

/// Collect metadata from its owning modules in legacy command order.
/// This is metadata assembly, not a dispatcher. Consumers may sort by stable ID.
const MODULES: &[Module] = &[
    crate::encoding::MODULE,
    crate::ciphers::MODULE,
    crate::hashing::MODULE,
    crate::binary::MODULE,
    crate::binary::transform::MODULE,
    Module::plain(crate::esolang::brainfuck::OPERATIONS),
    Module::plain(crate::forensics::OPERATIONS),
    Module::plain(crate::forensics::signatures::OPERATIONS),
    Module::plain(crate::forensics::inspect::OPERATIONS),
    Module::plain(crate::forensics::ranges::OPERATIONS),
    Module::plain(crate::forensics::boundaries::OPERATIONS),
    Module::plain(crate::forensics::png::OPERATIONS),
    Module::plain(crate::forensics::exif::OPERATIONS),
    Module::plain(crate::forensics::zip::OPERATIONS),
    Module::plain(crate::forensics::carving::OPERATIONS),
    Module::plain(crate::image::OPERATIONS),
    Module::plain(crate::image::raw::OPERATIONS),
    Module::plain(crate::image::colors::OPERATIONS),
    Module::plain(crate::image::csv::OPERATIONS),
    Module::plain(crate::image::layers::OPERATIONS),
    Module::plain(crate::image::pairs::OPERATIONS),
    Module::plain(crate::image::qr::OPERATIONS),
    Module::plain(crate::image::analyze::OPERATIONS)
        .with_contracts(crate::image::analyze::CONTRACTS),
    Module::plain(crate::image::bitwise::OPERATIONS),
    Module::plain(crate::image::transform::OPERATIONS),
    Module::plain(crate::image::bits::OPERATIONS),
    crate::ciphers::classical::MODULE,
    crate::ciphers::book::MODULE,
    crate::ciphers::bacon::MODULE,
    crate::ciphers::polybius::MODULE,
    crate::ciphers::mono::MODULE,
    crate::ciphers::poly::MODULE,
    crate::ciphers::transposition::MODULE,
    crate::ciphers::fractionation::MODULE,
    crate::symbols::keypad::MODULE,
    Module::plain(crate::text::keyboard::OPERATIONS),
    Module::plain(crate::text::classes::OPERATIONS),
    crate::text::MODULE,
    Module::plain(crate::text::selection::OPERATIONS),
    Module::plain(crate::text::word::OPERATIONS),
    Module::plain(crate::text::values::OPERATIONS),
    Module::plain(crate::text::unicode::OPERATIONS),
    Module::plain(crate::text::traversal::OPERATIONS),
    Module::plain(crate::text::transform::OPERATIONS),
    Module::plain(crate::text::grid::OPERATIONS),
    Module::plain(crate::text::combinations::OPERATIONS),
    Module::plain(crate::text::statistics::OPERATIONS),
    Module::plain(crate::text::multiset::OPERATIONS),
    Module::plain(crate::text::anagram::OPERATIONS),
    Module::plain(crate::text::word_ladder::OPERATIONS),
    Module::plain(crate::probability::OPERATIONS),
    Module::plain(crate::image::spatial::OPERATIONS),
    crate::symbols::MODULE,
    crate::symbols::a1z26::MODULE,
    crate::symbols::semaphore::MODULE,
    crate::symbols::clock::MODULE,
    Module::plain(crate::numbers::OPERATIONS),
    Module::plain(crate::numbers::combinatorics::OPERATIONS),
    Module::plain(crate::logic::boolean::OPERATIONS),
    Module::plain(crate::graph::OPERATIONS),
    Module::plain(crate::numbers::modular::OPERATIONS),
    Module::plain(crate::numbers::digital::OPERATIONS),
    Module::plain(crate::numbers::alphabet::OPERATIONS),
    Module::plain(crate::numbers::constants::OPERATIONS),
    Module::plain(crate::media::wave::OPERATIONS),
    Module::plain(crate::media::pcm::OPERATIONS),
    Module::plain(crate::media::envelope::OPERATIONS),
    Module::plain(crate::media::spectrum::OPERATIONS),
    Module::plain(crate::media::spectrogram::OPERATIONS),
    Module::plain(crate::media::bits::OPERATIONS),
    Module::plain(crate::media::reverse::OPERATIONS),
    Module::plain(crate::media::dtmf::OPERATIONS),
    Module::plain(crate::media::morse::OPERATIONS),
    Module::plain(crate::media::metadata::OPERATIONS),
    Module::plain(crate::media::video::OPERATIONS),
    Module::plain(crate::media::video::frames::OPERATIONS),
    Module::plain(crate::media::video::comparison::OPERATIONS),
    Module::plain(crate::media::video::audio::OPERATIONS),
    Module::plain(crate::web::html::OPERATIONS),
    Module::plain(crate::web::har::OPERATIONS),
    Module::plain(crate::web::assets::OPERATIONS),
    Module::plain(crate::pipeline::OPERATIONS),
    Module::plain(crate::pipeline::recipe::OPERATIONS),
    Module::plain(crate::investigation::triage::OPERATIONS),
    Module::plain(crate::investigation::session::OPERATIONS),
    Module::plain(crate::investigation::profile::OPERATIONS),
    crate::encoding::base58check::MODULE,
    crate::blockchain::MODULE,
    crate::blockchain::wif::MODULE,
    crate::hashing::protocol::MODULE,
    crate::blockchain::bitcoin::MODULE,
    crate::blockchain::candidates::MODULE,
    crate::blockchain::ethereum::MODULE,
    crate::blockchain::private::MODULE,
];
pub fn catalog() -> Vec<&'static Descriptor> {
    MODULES
        .iter()
        .flat_map(|module| module.descriptors)
        .collect()
}

/// Look up an exact stable ID; no aliases or heuristic matching are performed.
pub fn describe(id: &str) -> Option<&'static Descriptor> {
    catalog().into_iter().find(|descriptor| descriptor.id == id)
}

/// Filter the maintained catalog in legacy command order, with shared category errors.
pub fn filtered(
    category: Option<&str>,
) -> Result<Vec<&'static Descriptor>, crate::errors::OperationError> {
    let catalog = catalog();
    if category.is_some_and(|category| !catalog.iter().any(|d| d.category == category)) {
        return Err(crate::errors::OperationError::new(
            crate::errors::ErrorCode::InvalidParameter,
            "unknown operation category; list available operations",
        )
        .operation("ops.list")
        .parameter("category"));
    }
    Ok(catalog
        .into_iter()
        .filter(|d| category.is_none_or(|c| d.category == c))
        .collect())
}

/// Resolve a stable ID with consistent error context for CLI and SDK callers.
pub fn require(id: &str) -> Result<&'static Descriptor, crate::errors::OperationError> {
    describe(id).ok_or_else(|| {
        crate::errors::OperationError::new(
            crate::errors::ErrorCode::InvalidParameter,
            "unknown operation ID; list available operations",
        )
        .operation("ops.describe")
        .parameter("id")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn catalog_is_complete_and_self_consistent() {
        let catalog = catalog();
        assert!(catalog.len() >= 14);
        let mut ids = BTreeSet::new();
        for d in catalog {
            assert!(ids.insert(d.id));
            assert!(d.id.split('.').all(|part| {
                !part.is_empty()
                    && part
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
            }));
            assert!(!d.description.is_empty() && !d.category.is_empty() && !d.example.is_empty());
            assert!(!d.input_type.is_empty() && !d.output_type.is_empty());
            let mut names = BTreeSet::new();
            for parameter in d.parameters {
                assert!(names.insert(parameter.name));
                if let Some(default) = parameter.default {
                    assert!(parameter.constraints.accepts(default));
                }
            }
            assert_eq!(describe(d.id).unwrap().id, d.id);
        }
        assert!(describe("missing.operation").is_none());
    }

    #[test]
    fn validate_parameters_before_execution() {
        let caesar = describe("cipher.caesar.encode").unwrap();
        assert!(caesar.validate(&[]).is_err());
        for value in [i32::MIN as i64, 0, i32::MAX as i64] {
            assert!(
                caesar
                    .validate(&[("shift", ParameterValue::Integer(value))])
                    .is_ok()
            );
        }
        for value in [i32::MIN as i64 - 1, i32::MAX as i64 + 1] {
            assert!(
                caesar
                    .validate(&[("shift", ParameterValue::Integer(value))])
                    .is_err()
            );
        }
        assert!(
            caesar
                .validate(&[("shift", ParameterValue::Text("3"))])
                .is_err()
        );
        assert!(
            caesar
                .validate(&[("unknown", ParameterValue::Boolean(true))])
                .is_err()
        );
        assert!(
            caesar
                .validate(&[
                    ("shift", ParameterValue::Integer(1)),
                    ("shift", ParameterValue::Integer(2))
                ])
                .is_err()
        );
        let xor = describe("binary.xor").unwrap();
        assert!(
            xor.validate(&[("key", ParameterValue::Bytes(b""))])
                .is_err()
        );
        assert!(
            xor.validate(&[("key", ParameterValue::Bytes(b"k"))])
                .is_ok()
        );
        let e = xor.validate(&[]).unwrap_err();
        assert!(e.to_string().contains("key"));
        assert!(ParameterType::Boolean.accepts(ParameterValue::Boolean(false)));
        assert!(ParameterType::Text { choices: &["raw"] }.accepts(ParameterValue::Text("raw")));
        assert!(!ParameterType::Text { choices: &["raw"] }.accepts(ParameterValue::Text("other")));
        assert!(ParameterType::Text { choices: &[] }.accepts(ParameterValue::Text("世界")));
    }
}
