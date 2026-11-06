use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
/// Exact full-round profiles; there is no default or family-name inference.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
pub enum Profile {
    #[serde(rename = "des_fips46_v1")]
    Des,
    #[serde(rename = "tdes_ede2_v1")]
    TdesEde2,
    #[serde(rename = "tdes_ede3_v1")]
    TdesEde3,
    #[serde(rename = "blowfish_be_v1")]
    BlowfishBe,
    #[serde(rename = "idea_v1")]
    Idea,
    #[serde(rename = "rc2_rfc2268_v1")]
    Rc2,
}
impl Profile {
    pub const ALL: &[Self] = &[
        Self::Des,
        Self::TdesEde2,
        Self::TdesEde3,
        Self::BlowfishBe,
        Self::Idea,
        Self::Rc2,
    ];
    /// Parse only a supported exact profile identifier.
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        serde_json::from_value(serde_json::Value::String(value.into()))
            .map_err(|_| super::invalid("profile", "unsupported exact block profile"))
    }
    /// Required whole-block width, in bytes.
    pub const fn block_bytes(self) -> usize {
        8
    }
    fn key_range(self) -> (usize, usize) {
        match self {
            Self::Des => (8, 8),
            Self::TdesEde2 | Self::Idea => (16, 16),
            Self::TdesEde3 => (24, 24),
            Self::BlowfishBe => (4, 56),
            Self::Rc2 => (1, 128),
        }
    }
    pub(super) fn accepts_key(self, bytes: usize) -> bool {
        let (min, max) = self.key_range();
        (min..=max).contains(&bytes)
    }
    pub(super) fn key_pattern(self) -> String {
        let (min, max) = self.key_range();
        format!("^(?:[0-9A-Fa-f]{{2}}){{{min},{max}}}$")
    }
}
