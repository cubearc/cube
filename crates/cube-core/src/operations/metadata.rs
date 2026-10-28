//! Optional, module-owned facts about implemented cryptographic/puzzle profiles.
//! These annotations neither execute algorithms nor grant capabilities. Complete
//! invocation schemas/named ports remain the separately scoped contract work.
use super::Descriptor;
use serde::Serialize;
/// Security purpose, not a certification or recommendation to use an algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Classification {
    RawBlockCipher,
    HistoricalCipher,
    Encoding,
    UnkeyedDigest,
    LegacyDigest,
    Checksum,
    BinaryTransform,
    PrivateMaterial,
    PublicDerivation,
    PuzzleCode,
}
/// A name for the existing profile, with its precise parameter/usage qualification.
/// Aliases are descriptive and never accepted in place of stable operation IDs.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Alias {
    pub name: &'static str,
    pub usage: &'static str,
}
/// Known material sensitivity. Unflagged caller data can still be confidential.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Security {
    pub classification: Classification,
    pub limitations: &'static str,
    pub private_input: bool,
    pub private_output: bool,
    pub sensitive_parameters: &'static [&'static str],
}
/// Exact existing variant, including alphabet/key advancement and block meaning.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Profile {
    pub variant: &'static str,
    pub alphabet: &'static str,
    pub key: &'static str,
    pub key_advancement: &'static str,
    /// Compression/absorption block for digests; not evidence of a block cipher.
    /// None means no fixed applicable block size for this profile.
    pub block_bits: Option<u16>,
    pub output_bits: Option<u16>,
}
/// Where a documented numeric ceiling applies. Core applies to every caller of
/// that direct function; CLI/SDK and pipeline bounds can be stricter. Some bounds
/// are mathematical output maxima rather than independently configurable budgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Core,
    Cli,
    Python,
    Pipeline,
}
#[derive(Clone, Copy, Debug, Serialize)]
pub struct ResourceBound {
    pub name: &'static str,
    pub maximum: u64,
    pub unit: &'static str,
    pub scope: Scope,
}
/// Additive metadata version, independent of operation, recipe and adapter versions.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Metadata {
    pub schema_version: u8,
    pub canonical_name: &'static str,
    pub family: &'static str,
    pub aliases: &'static [Alias],
    pub security: Security,
    pub profile: Profile,
    pub resources: &'static [ResourceBound],
    /// Explicitly records missing legacy caps and bounds that depend on parameters.
    pub resource_notes: &'static str,
    pub rust_api: &'static str,
    pub python_api: &'static str,
    /// Primary specifications/maintainer documentation, not backend qualification.
    pub references: &'static [&'static str],
}
/// A single authoring row projects the original Descriptor slice and annotations.
/// There is no second maintained operation-ID map.
#[derive(Clone, Copy)]
pub(crate) struct Entry {
    pub descriptor: Descriptor,
    pub metadata: Option<Metadata>,
}
pub(crate) const fn descriptors<const N: usize>(entries: &[Entry; N]) -> [Descriptor; N] {
    // Called only for concrete, nonempty module-owned declaration arrays.
    let mut descriptors = [entries[0].descriptor; N];
    let mut i = 0;
    while i < N {
        descriptors[i] = entries[i].descriptor;
        i += 1;
    }
    descriptors
}
