use crate::errors::OperationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

enum KeyBytes {
    Set(&'static [usize]),
    Range(usize, usize),
}
impl KeyBytes {
    fn accepts(&self, bytes: usize) -> bool {
        match self {
            Self::Set(sizes) => sizes.contains(&bytes),
            Self::Range(min, max) => (*min..=*max).contains(&bytes),
        }
    }
    fn pattern(&self) -> String {
        match self {
            Self::Set(sizes) => format!(
                "^(?:{})$",
                sizes
                    .iter()
                    .map(|n| format!("[0-9A-Fa-f]{{{}}}", 2 * n))
                    .collect::<Vec<_>>()
                    .join("|")
            ),
            Self::Range(min, max) => format!("^(?:[0-9A-Fa-f]{{2}}){{{min},{max}}}$"),
        }
    }
}
// One finite declaration owns exact IDs, external block widths and key sizes.
macro_rules! profiles {
    ($(($variant:ident, $name:literal, $block:literal, $key:expr)),+ $(,)?) => {
        /// Exact full-round profiles; no default or family-name inference.
        #[derive(Clone, Copy, Debug, Deserialize, Serialize, JsonSchema, PartialEq, Eq)]
        pub enum Profile { $(#[serde(rename = $name)] $variant,)+ }
        impl Profile {
            pub const ALL: &[Self] = &[$(Self::$variant,)+];
            pub const NAMES: &[&str] = &[$($name,)+];
            /// Required whole-block width, in bytes.
            pub const fn block_bytes(self) -> usize { match self { $(Self::$variant => $block,)+ } }
            fn key_bytes(self) -> KeyBytes { match self { $(Self::$variant => $key,)+ } }
        }
    }
}
use KeyBytes::{Range, Set};
profiles! {
    (Des, "des_fips46_v1", 8, Set(&[8])),
    (TdesEde2, "tdes_ede2_v1", 8, Set(&[16])),
    (TdesEde3, "tdes_ede3_v1", 8, Set(&[24])),
    (BlowfishBe, "blowfish_be_v1", 8, Range(4, 56)),
    (Idea, "idea_v1", 8, Set(&[16])),
    (Rc2, "rc2_rfc2268_v1", 8, Range(1, 128)),
    (Camellia, "camellia_rfc3713_v1", 16, Set(&[16, 24, 32])),
    (Serpent, "serpent_final_v1", 16, Set(&[16, 24, 32])),
    (Cast128, "cast128_rfc2144_v1", 8, Range(5, 16)),
    (Cast256, "cast256_rfc2612_v1", 16, Set(&[16, 20, 24, 28, 32])),
    (Aria, "aria_rfc5794_v1", 16, Set(&[16, 24, 32])),
    (Twofish, "twofish_v1", 16, Set(&[16, 24, 32])),
    (Sm4, "sm4_gmt0002_v1", 16, Set(&[16])),
    (Kuznyechik, "kuznyechik_rfc7801_v1", 16, Set(&[32])),
    (Magma, "magma_rfc8891_v1", 8, Set(&[32])),
    (Gift128, "gift128_v1", 16, Set(&[16])),
    (Xtea, "xtea32_le_v1", 8, Set(&[16])),
    (Rc5, "rc5_32_12_16_le_v1", 8, Set(&[16])),
    (Rc6, "rc6_32_20_v1", 16, Set(&[16, 24, 32])),
    (Speck32_64, "speck32_64_be_v1", 4, Set(&[8])),
    (Speck48_72, "speck48_72_be_v1", 6, Set(&[9])),
    (Speck48_96, "speck48_96_be_v1", 6, Set(&[12])),
    (Speck64_96, "speck64_96_be_v1", 8, Set(&[12])),
    (Speck64_128, "speck64_128_be_v1", 8, Set(&[16])),
    (Speck96_96, "speck96_96_be_v1", 12, Set(&[12])),
    (Speck96_144, "speck96_144_be_v1", 12, Set(&[18])),
    (Speck128_128, "speck128_128_be_v1", 16, Set(&[16])),
    (Speck128_192, "speck128_192_be_v1", 16, Set(&[24])),
    (Speck128_256, "speck128_256_be_v1", 16, Set(&[32])),
    (Rijndael128, "rijndael_128_v1", 16, Set(&[16, 20, 24, 28, 32])),
    (Rijndael160, "rijndael_160_v1", 20, Set(&[16, 20, 24, 28, 32])),
    (Rijndael192, "rijndael_192_v1", 24, Set(&[16, 20, 24, 28, 32])),
    (Rijndael224, "rijndael_224_v1", 28, Set(&[16, 20, 24, 28, 32])),
    (Rijndael256, "rijndael_256_v1", 32, Set(&[16, 20, 24, 28, 32])),
    (Skipjack, "skipjack_nist_v1", 8, Set(&[10])),
    (Noekeon, "noekeon_direct_v1", 16, Set(&[16])),
    (Lea, "lea_v1", 16, Set(&[16, 24, 32])),
    (Present80, "present80_v1", 8, Set(&[10])),
    (Present128, "present128_v1", 8, Set(&[16])),
    (GostTest, "gost28147_test_le_v1", 8, Set(&[32])),
    (GostA, "gost28147_cryptopro_a_le_v1", 8, Set(&[32])),
    (GostB, "gost28147_cryptopro_b_le_v1", 8, Set(&[32])),
    (GostC, "gost28147_cryptopro_c_le_v1", 8, Set(&[32])),
    (Tea, "tea32_be_v1", 8, Set(&[16])),
    (Shacal2, "shacal2_be_v1", 32, Set(&[16, 20, 24, 28, 32, 36, 40, 44, 48, 52, 56, 60, 64])),
    (Twine80, "twine80_v1", 8, Set(&[10])),
    (Twine128, "twine128_v1", 8, Set(&[16])),
    (Hight, "hight_v1", 8, Set(&[16])),
    (Misty1, "misty1_rfc2994_v1", 8, Set(&[16])),
    (Kasumi, "kasumi_3gpp_v1", 8, Set(&[16])),
}
impl Profile {
    /// Parse only a supported exact profile identifier.
    pub fn parse(value: &str) -> Result<Self, OperationError> {
        serde_json::from_value(serde_json::Value::String(value.into()))
            .map_err(|_| super::invalid("profile", "unsupported exact block profile"))
    }
    pub(super) fn accepts_key(self, bytes: usize) -> bool {
        self.key_bytes().accepts(bytes)
    }
    pub(super) fn key_pattern(self) -> String {
        self.key_bytes().pattern()
    }
    /// Maximum input bytes before expansion or output allocation.
    /// Slow educational reference implementations have a smaller explicit cap.
    pub const fn max_input_bytes(self) -> usize {
        match self {
            Self::Rijndael128
            | Self::Rijndael160
            | Self::Rijndael192
            | Self::Rijndael224
            | Self::Rijndael256
            | Self::Skipjack
            | Self::Noekeon
            | Self::Lea
            | Self::Present80
            | Self::Present128
            | Self::Tea
            | Self::Shacal2
            | Self::Twine80
            | Self::Twine128
            | Self::Hight
            | Self::Misty1
            | Self::Kasumi => 64 * 1024,
            _ => super::MAX_INPUT,
        }
    }
}
