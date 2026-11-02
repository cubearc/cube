//! M-94 fixed wheel facts and row selection: FM 24-5, 19 October 1942,
//! section V, paragraphs 56–61. George Mace's measured alphabet transcription:
//! https://www.jproc.ca/crypto/m94.html. Some other online tables repeat U in
//! wheel5 or K in wheel25; these are rejected by the permutation regression.
use super::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
/// Standard numbered M-94 wheels, clockwise from A. These are device facts,
/// not simulator code. Wheel order is explicitly supplied, never inferred.
pub const M94_WHEELS: [&str; 25] = [
    "ABCEIGDJFVUYMHTQKZOLRXSPWN",
    "ACDEHFIJKTLMOUVYGZNPQXRWSB",
    "ADKOMJUBGEPHSCZINXFYQRTVWL",
    "AEDCBIFGJHLKMRUOQVPTNWYXZS",
    "AFNQUKDOPITJBRHCYSLWEMZVXG",
    "AGPOCIXLURNDYZHWBJSQFKVMET",
    "AHXJEZBNIKPVROGSYDULCFMQTW",
    "AIHPJOBWKCVFZLQERYNSUMGTDX",
    "AJDSKQOIVTZEFHGYUNLPMBXWCR",
    "AKELBDFJGHONMTPRQSVZUXYWIC",
    "ALTMSXVQPNOHUWDIZYCGKRFBEJ",
    "AMNFLHQGCUJTBYPZKXISRDVEWO",
    "ANCJILDHBMKGXUZTSWQYVORPFE",
    "AODWPKJVIUQHZCTXBLEGNYRSMF",
    "APBVHIYKSGUENTCXOWFQDRLJZM",
    "AQJNUBTGIMWZRVLXCSHDEOKFPY",
    "ARMYOFTHEUSZJXDPCWGQIBKLNV",
    "ASDMCNEQBOZPLGVJRKYTFUIWXH",
    "ATOJYLFXNGWHVCMIRBSEKUPDZQ",
    "AUTRZXQLYIOVBPESNHJWMDGFCK",
    "AVNKHRGOXEYBFSJMUDQCLZWTIP",
    "AWVSFDLIEBHKNRJQZGMXPUCOTY",
    "AXKWREVDTUFOYHMLSIQNJCPGBZ",
    "AYJPXMVKBQWUGLOSTECHNZFRID",
    "AZDNBUHYFWJLVGRCQMPSOEXTKI",
];
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum M94Profile {
    #[serde(rename = "m94_v1")]
    M94V1,
}
#[derive(Serialize, JsonSchema)]
#[schemars(transparent)]
struct WheelId(#[schemars(range(min = 1, max = 25))] u8);
#[derive(Serialize, JsonSchema)]
#[schemars(transparent)]
struct RowOffset(#[schemars(range(min = 1, max = 25))] u8);
/// Explicit left-to-right standard wheel order and one clockwise row
/// displacement per successive block of at most25 letters. Decoding requires
/// the same displacements; it never guesses a meaningful-looking row.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct M94Parameters {
    pub profile: M94Profile,
    #[schemars(with = "[WheelId;25]", extend("uniqueItems"=true))]
    pub order: [u8; 25],
    #[schemars(with = "Vec<RowOffset>", length(max = 41944))]
    pub row_offsets: Vec<u8>,
}
#[derive(Debug, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max=1048576),extend("x-cube-max-utf8-bytes"=1048576))]
    pub text: String,
}
/// Encipher prepared ASCII A–Z using standard M-94 wheels. No random row
/// choice, keyword expansion, padding, spelling-out or case folding occurs.
pub fn m94_encode(text: &str, parameters: &M94Parameters) -> Result<Report, OperationError> {
    m94(text, parameters, false)
}
/// Decipher using the sender's exact wheel order and row displacements.
pub fn m94_decode(text: &str, parameters: &M94Parameters) -> Result<Report, OperationError> {
    m94(text, parameters, true)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum JeffersonProfile {
    #[serde(rename = "jefferson_configured_v1")]
    ConfiguredV1,
}
#[derive(Serialize, JsonSchema)]
#[schemars(transparent)]
struct Alphabet(#[schemars(regex(pattern = "^[A-Z]{26}$"))] String);
#[derive(Serialize, JsonSchema)]
#[schemars(transparent)]
struct ConfiguredWheelId(#[schemars(range(min = 1, max = 64))] u8);
/// Jefferson's numbered mixed-alphabet wheels, explicitly supplied as described
/// in his manuscript before22 March1802. No canonical wheel set was prescribed.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct JeffersonParameters {
    pub profile: JeffersonProfile,
    #[schemars(with = "Vec<Alphabet>", length(min = 1, max = 64))]
    pub wheels: Vec<String>,
    #[schemars(with="Vec<ConfiguredWheelId>",length(min=1,max=64),extend("uniqueItems"=true))]
    pub order: Vec<u8>,
    #[schemars(with = "Vec<RowOffset>", length(max = 65536))]
    pub row_offsets: Vec<u8>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
pub enum M138aProfile {
    #[serde(rename = "m138a_configured_v1")]
    ConfiguredV1,
}
#[derive(Serialize, JsonSchema)]
#[schemars(transparent)]
struct StripId(#[schemars(range(min = 1, max = 100))] u8);
/// M-138-A/CSP-845 raw30-track strip mechanism. Supply the complete100-strip
/// series in numbered order and select30 distinct IDs. Message indicators,
/// null insertion and code-word procedures are outside this raw transform.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(extend("x-cube-max-serialized-bytes"=524288))]
pub struct M138aParameters {
    pub profile: M138aProfile,
    #[schemars(regex(pattern = "^[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}$"))]
    pub strip_series: String,
    #[schemars(with = "Vec<Alphabet>", length(min = 100, max = 100))]
    pub strips: Vec<String>,
    #[schemars(with="[StripId;30]",extend("uniqueItems"=true))]
    pub order: [u8; 30],
    #[schemars(with = "Vec<RowOffset>", length(max = 34953))]
    pub row_offsets: Vec<u8>,
}
/// Encipher explicit Jefferson wheels with no row guessing or text preparation.
pub fn jefferson_encode(text: &str, p: &JeffersonParameters) -> Result<Report, OperationError> {
    jefferson(text, p, false)
}
/// Reverse the same configured Jefferson rows.
pub fn jefferson_decode(text: &str, p: &JeffersonParameters) -> Result<Report, OperationError> {
    jefferson(text, p, true)
}
fn jefferson(text: &str, p: &JeffersonParameters, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.jefferson.decode"
    } else {
        "cipher.jefferson.encode"
    };
    parameter_size(p, op)?;
    if p.wheels.is_empty() || p.wheels.len() > 64 || p.order.len() != p.wheels.len() {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "wheels",
            "provide1..64 complete wheels and an order of the same length",
        ));
    }
    if p.row_offsets.len() > 65536 {
        return Err(error(
            op,
            ErrorCode::ResourceLimit,
            "row_offsets",
            "at most65536 configured row displacements",
        ));
    }
    let wheels = ordered_wheels(&p.wheels, &p.order, op, "wheels")?;
    row_transform(text, &wheels, &p.row_offsets, decode, op)
}
/// Encipher the raw M-138-A30-track mechanism with caller-supplied strip facts.
pub fn m138a_encode(text: &str, p: &M138aParameters) -> Result<Report, OperationError> {
    m138a(text, p, false)
}
/// Reverse the same selected M-138-A strips and clockwise row displacements.
pub fn m138a_decode(text: &str, p: &M138aParameters) -> Result<Report, OperationError> {
    m138a(text, p, true)
}
fn m138a(text: &str, p: &M138aParameters, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.m138a.decode"
    } else {
        "cipher.m138a.encode"
    };
    parameter_size(p, op)?;
    let label = p.strip_series.as_bytes();
    if label.is_empty()
        || label.len() > 128
        || !label[0].is_ascii_alphanumeric()
        || label
            .iter()
            .any(|b| !b.is_ascii_alphanumeric() && !b"._:/-".contains(b))
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "strip_series",
            "use an explicit1..128-byte ASCII series identifier",
        ));
    }
    if p.strips.len() != 100 {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "strips",
            "provide all100 numbered strip alphabets",
        ));
    }
    let wheels = ordered_wheels(&p.strips, &p.order, op, "strips")?;
    row_transform(text, &wheels, &p.row_offsets, decode, op)
}
fn ordered_wheels<'a>(
    alphabets: &'a [String],
    order: &[u8],
    op: &'static str,
    field: &'static str,
) -> Result<Vec<&'a [u8]>, OperationError> {
    for alphabet in alphabets {
        let mut sorted = alphabet.as_bytes().to_vec();
        sorted.sort_unstable();
        if sorted != b"ABCDEFGHIJKLMNOPQRSTUVWXYZ" {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                field,
                "every wheel/strip must contain each ASCII letter exactly once",
            ));
        }
    }
    let mut seen = vec![false; alphabets.len()];
    let mut wheels = Vec::with_capacity(order.len());
    for &id in order {
        let index = usize::from(id)
            .checked_sub(1)
            .filter(|&n| n < alphabets.len())
            .ok_or_else(|| {
                error(
                    op,
                    ErrorCode::InvalidParameter,
                    "order",
                    "selected wheel or strip ID is outside the supplied set",
                )
            })?;
        if std::mem::replace(&mut seen[index], true) {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "order",
                "selected wheel or strip IDs must be distinct",
            ));
        }
        wheels.push(alphabets[index].as_bytes());
    }
    Ok(wheels)
}
fn m94(text: &str, p: &M94Parameters, decode: bool) -> Result<Report, OperationError> {
    let op = if decode {
        "cipher.m94.decode"
    } else {
        "cipher.m94.encode"
    };
    parameter_size(p, op)?;
    let mut seen = [false; 25];
    for &id in &p.order {
        let index = usize::from(id)
            .checked_sub(1)
            .filter(|&n| n < 25)
            .ok_or_else(|| {
                error(
                    op,
                    ErrorCode::InvalidParameter,
                    "order",
                    "use each M-94 wheel 1..25 exactly once",
                )
            })?;
        if std::mem::replace(&mut seen[index], true) {
            return Err(error(
                op,
                ErrorCode::InvalidParameter,
                "order",
                "wheel order must be a permutation",
            ));
        }
    }
    let wheels = p
        .order
        .iter()
        .map(|&id| M94_WHEELS[usize::from(id) - 1].as_bytes())
        .collect::<Vec<_>>();
    row_transform(text, &wheels, &p.row_offsets, decode, op)
}
fn row_transform(
    text: &str,
    wheels: &[&[u8]],
    offsets: &[u8],
    decode: bool,
    op: &'static str,
) -> Result<Report, OperationError> {
    text_budget(text, op)?;
    if offsets.len() != text.len().div_ceil(wheels.len())
        || offsets.iter().any(|&v| !(1..=25).contains(&v))
    {
        return Err(error(
            op,
            ErrorCode::InvalidParameter,
            "row_offsets",
            "provide one displacement 1..25 per complete or partial wheel block",
        ));
    }
    if let Some(offset) = text.bytes().position(|b| !b.is_ascii_uppercase()) {
        return Err(error(
            op,
            ErrorCode::InvalidInput,
            "text",
            "use prepared ASCII A-Z only",
        )
        .offset(offset as u64));
    }
    let inverses = wheels
        .iter()
        .map(|wheel| {
            let mut inverse = [0usize; 26];
            for (index, &letter) in wheel.iter().enumerate() {
                inverse[usize::from(letter - b'A')] = index;
            }
            inverse
        })
        .collect::<Vec<_>>();
    let output = text
        .bytes()
        .enumerate()
        .map(|(index, letter)| {
            let slot = index % wheels.len();
            let offset = usize::from(offsets[index / wheels.len()]);
            let displacement = if decode { 26 - offset } else { offset };
            char::from(
                wheels[slot][(inverses[slot][usize::from(letter - b'A')] + displacement) % 26],
            )
        })
        .collect();
    Ok(Report { text: output })
}
