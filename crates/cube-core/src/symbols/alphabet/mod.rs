//! Source-qualified symbolic alphabets over exact presegmented units.
//! No OCR, font inference, case folding, transliteration or implicit segmentation.
mod catalog;
mod fixtures;
use crate::{
    ciphers::mono::{historical as h, homophonic},
    errors::{ErrorCode, OperationError},
};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub const MAX_REPORT: usize = 1048576;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
pub enum Profile {
    #[serde(rename = "al_bhed_english_ffx_v1")]
    AlBhedEnglishFfxV1,
    #[serde(rename = "hylian_skyward_dcode_v1")]
    HylianSkywardDcodeV1,
    #[serde(rename = "sheikah_botw_dcode_v1")]
    SheikahBotwDcodeV1,
    #[serde(rename = "daedric_names_dcode_v1")]
    DaedricNamesDcodeV1,
    #[serde(rename = "futurama_alien1_dcode_v1")]
    FuturamaAlien1DcodeV1,
    #[serde(rename = "bill_cipher_dcode_v1")]
    BillCipherDcodeV1,
    #[serde(rename = "gnommish_disney_2020_v1")]
    GnommishDisney2020V1,
    #[serde(rename = "theban_trithemius_1518_v1")]
    ThebanTrithemius1518V1,
    #[serde(rename = "celestial_agrippa_ch30_v1")]
    CelestialAgrippaCh30V1,
    #[serde(rename = "malachim_agrippa_ch30_v1")]
    MalachimAgrippaCh30V1,
    #[serde(rename = "magi_ambrosius_1539_v1")]
    MagiAmbrosius1539V1,
    #[serde(rename = "enochian_peterson_2022_v1")]
    EnochianPeterson2022V1,
    #[serde(rename = "moon_english_1877_v1")]
    MoonEnglish1877V1,
    #[serde(rename = "dancing_men_doyle_1905_v1")]
    DancingMenDoyle1905V1,
    #[serde(rename = "alphabetum_kaldeorum_hahn_2004_v1")]
    AlphabetumKaldeorumHahn2004V1,
    #[serde(rename = "pigpen_haverhill_pairs_v1")]
    PigpenHaverhillPairsV1,
    #[serde(rename = "tic_tac_toe_dcode_v1")]
    TicTacToeDcodeV1,
    #[serde(rename = "templars_bothwell_gosse_v1")]
    TemplarsBothwellGosseV1,
    #[serde(rename = "betamaze_cliff_2006_v1")]
    BetamazeCliff2006V1,
    #[serde(rename = "clock_quarter_hour_zero_v1")]
    ClockQuarterHourZeroV1,
    #[serde(rename = "birds_nrtech_four_features_v1")]
    BirdsNrtechFourFeaturesV1,
    #[serde(rename = "aurebesh_ucsur_2016_v1")]
    AurebeshUcsur2016V1,
    #[serde(rename = "standard_galactic_ucsur_2019_v1")]
    StandardGalacticUcsur2019V1,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    #[schemars(length(min = 1, max = 1024))]
    pub source_id: String,
    pub profile_id: Profile,
    #[schemars(length(min = 1, max = 1024))]
    pub source_url: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub source_sha256: String,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Encoded {
    pub tokens: h::Tokens,
    pub provenance: Provenance,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Decoded {
    /// Exact supplied tokens, in original order.
    pub tokens: h::Tokens,
    #[schemars(length(max = 65536))]
    pub readings: Vec<h::Unit>,
    pub provenance: Provenance,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    source_id: String,
    source_url: String,
    source_sha256: String,
    definition: h::Definition,
}
fn book(p: &Parameters) -> Result<(h::Book, Provenance), OperationError> {
    let source: Source = serde_json::from_str(match p.profile {
        Profile::AlBhedEnglishFfxV1 => include_str!("al_bhed_english_ffx_v1.json"),
        Profile::HylianSkywardDcodeV1 => include_str!("hylian_skyward_dcode_v1.json"),
        Profile::SheikahBotwDcodeV1 => include_str!("sheikah_botw_dcode_v1.json"),
        Profile::DaedricNamesDcodeV1 => include_str!("daedric_names_dcode_v1.json"),
        Profile::FuturamaAlien1DcodeV1 => include_str!("futurama_alien1_dcode_v1.json"),
        Profile::BillCipherDcodeV1 => include_str!("bill_cipher_dcode_v1.json"),
        Profile::GnommishDisney2020V1 => include_str!("gnommish_disney_2020_v1.json"),
        Profile::ThebanTrithemius1518V1 => include_str!("theban_trithemius_1518_v1.json"),
        Profile::CelestialAgrippaCh30V1 => include_str!("celestial_agrippa_ch30_v1.json"),
        Profile::MalachimAgrippaCh30V1 => include_str!("malachim_agrippa_ch30_v1.json"),
        Profile::MagiAmbrosius1539V1 => include_str!("magi_ambrosius_1539_v1.json"),
        Profile::EnochianPeterson2022V1 => include_str!("enochian_peterson_2022_v1.json"),
        Profile::MoonEnglish1877V1 => include_str!("moon_english_1877_v1.json"),
        Profile::DancingMenDoyle1905V1 => include_str!("dancing_men_doyle_1905_v1.json"),
        Profile::AlphabetumKaldeorumHahn2004V1 => {
            include_str!("alphabetum_kaldeorum_hahn_2004_v1.json")
        }
        Profile::PigpenHaverhillPairsV1 => include_str!("pigpen_haverhill_pairs_v1.json"),
        Profile::TicTacToeDcodeV1 => include_str!("tic_tac_toe_dcode_v1.json"),
        Profile::TemplarsBothwellGosseV1 => include_str!("templars_bothwell_gosse_v1.json"),
        Profile::BetamazeCliff2006V1 => include_str!("betamaze_cliff_2006_v1.json"),
        Profile::ClockQuarterHourZeroV1 => include_str!("clock_quarter_hour_zero_v1.json"),
        Profile::BirdsNrtechFourFeaturesV1 => include_str!("birds_nrtech_four_features_v1.json"),
        Profile::AurebeshUcsur2016V1 => include_str!("aurebesh_ucsur_2016_v1.json"),
        Profile::StandardGalacticUcsur2019V1 => {
            include_str!("standard_galactic_ucsur_2019_v1.json")
        }
    })
    .expect("reviewed fixed source transcription");
    let definition = source.definition;
    homophonic::serialized_size(&definition, 65536, "symbol.alphabet.encode")?;
    let effective = crate::operations::contracts::canonical_value(
        serde_json::json!({"profile":p.profile,"source_sha256":source.source_sha256,"definition":definition}),
    );
    let provenance = Provenance {
        source_id: source.source_id,
        profile_id: p.profile,
        source_url: source.source_url,
        source_sha256: source.source_sha256,
        definition_sha256: crate::hashing::sha256(
            &serde_json::to_vec(&effective).expect("definition"),
        ),
    };
    Ok((h::Book::new(definition)?, provenance))
}
/// Encode exact, presegmented units using the selected dated source table.
/// Digraphs remain distinct from separate letters; unsupported units reject.
pub fn encode(units: &[String], p: &Parameters) -> Result<Encoded, OperationError> {
    let run = || {
        let (book, provenance) = book(p)?;
        let result = Encoded {
            tokens: h::Tokens(book.encode(units).map_err(|e| input_failure(e, "units"))?),
            provenance,
        };
        homophonic::serialized_size(&result, MAX_REPORT, "symbol.alphabet.encode")
            .map_err(|e| e.parameter("output"))?;
        Ok(result)
    };
    run().map_err(|e: OperationError| e.operation("symbol.alphabet.encode"))
}
/// Decode exact tokens, preserving every source-listed alternative and position.
/// Private-use proposals require a matching font for display; none is bundled.
pub fn decode(tokens: &[String], p: &Parameters) -> Result<Decoded, OperationError> {
    let run = || {
        let (book, provenance) = book(p)?;
        let readings = book
            .decode(tokens)
            .map_err(|e| input_failure(e, "tokens"))?
            .units;
        let result = Decoded {
            tokens: h::Tokens(tokens.to_vec()),
            readings,
            provenance,
        };
        homophonic::serialized_size(&result, MAX_REPORT, "symbol.alphabet.decode")
            .map_err(|e| e.parameter("output"))?;
        Ok(result)
    };
    run().map_err(|e: OperationError| e.operation("symbol.alphabet.decode"))
}
fn input_failure(mut error: OperationError, field: &'static str) -> OperationError {
    if error.code == ErrorCode::InvalidParameter {
        error.code = ErrorCode::InvalidInput;
    }
    if matches!(error.parameter, Some("unit" | "items")) {
        error.parameter = Some(field);
    }
    error
}
