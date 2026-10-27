use super::*;
use crate::{
    blockchain, encoding, esolang, forensics, graph, image, investigation, logic, media, numbers,
    pipeline, probability, symbols, text, web,
};

fn simple<T: JsonSchema>() -> Schema {
    schema::<T>()
}
fn root(semantic: &str) -> Result<Schema, OperationError> {
    let value = match semantic {
        "text" | "text.integer" | "polybius.tokens" | "private.wif" | "number.signed_magnitude" => {
            simple::<String>()
        }
        "bytes" | "byte_reader" | "private.scalar" | "private.material" | "private.export" => {
            simple::<values::Bytes>()
        }
        "file_path" => simple::<values::FileReference>(),
        "image.samples" => simple::<values::Image>(),
        "audio.pcm_samples" => simple::<values::Pcm>(),
        "unsigned" | "number.u64" => simple::<u64>(),
        "unsigned_list" => simple::<Vec<u64>>(),
        "number.decimal_digit" => {
            Schema::try_from(json!({"type":"integer","minimum":0,"maximum":9}))
                .expect("digit schema")
        }
        "number.bezout" => simple::<numbers::modular::ExtendedGcd>(),
        "exact_count" => simple::<numbers::combinatorics::Count>(),
        "pascal_triangle" => simple::<numbers::combinatorics::Triangle>(),
        "constant.digits" => simple::<numbers::constants::DigitSelection>(),
        "finite_probability_model" => simple::<probability::FiniteModel>(),
        "finite_probability_report" => simple::<probability::FiniteReport>(),
        "dice_model" => simple::<probability::DiceModel>(),
        "dice_pair" => simple::<probability::DicePair>(),
        "dice_sum_report" => simple::<probability::SumReport>(),
        "dice_comparison" => simple::<probability::Comparison>(),
        "graph_definition" => simple::<graph::Definition>(),
        "graph_components" => simple::<graph::QueryReport<Vec<Vec<String>>>>(),
        "graph_path" | "graph_cycle" => simple::<graph::QueryReport<Option<graph::Path>>>(),
        "graph_euler_trail" => simple::<graph::QueryReport<graph::EulerResult>>(),
        "encoding.base58check.decoded" => simple::<encoding::base58check::Decoded>(),
        "file.png_boundary" => simple::<forensics::png::Boundary>(),
        "file.carving" => simple::<forensics::carving::Extraction>(),
        "file.signatures" => simple::<Vec<forensics::signatures::Signature>>(),
        "file.signature_scan" => simple::<forensics::signatures::Scan>(),
        "file.info" => simple::<forensics::inspect::FileInfo>(),
        "file.zip_structure" => simple::<forensics::zip::Archive>(),
        "file.exif" => simple::<forensics::exif::Metadata>(),
        "file.hex_rows" => simple::<Vec<forensics::ranges::HexRow>>(),
        "file.strings" => simple::<forensics::ranges::Strings>(),
        "file.comparison" => simple::<forensics::ranges::Comparison>(),
        "image.png_report" => simple::<forensics::png::Report>(),
        "image.geometry" => simple::<image::raw::GeometryReport>(),
        "image.histogram" => simple::<image::Histogram>(),
        "image.comparison" => simple::<image::Comparison>(),
        "image.color_exports" => simple::<image::layers::ColorExports>(),
        "image.qr_report" => simple::<image::qr::QrReport>(),
        "spatial_autocorrelation" => simple::<image::spatial::Report>(),
        "image.pair_batch" => simple::<image::pairs::Batch>(),
        "image.pixel_data" => simple::<image::PixelData>(),
        "image.bit_plane" => simple::<image::BitPlane>(),
        "image.bit_stream" => simple::<image::bits::OrderedLsbResult>(),
        "image.color_layer" => simple::<image::colors::ColorLayer>(),
        "investigation.session" => simple::<investigation::session::Session>(),
        "investigation.request" => simple::<values::RecordRequest>(),
        "investigation.replay_request" => simple::<values::ReplayRequest>(),
        "investigation.profile_request" => simple::<values::ProfileRequest>(),
        "investigation.profile_plan" => simple::<investigation::profile::DryRun>(),
        "investigation.report" => investigation::session::contract_report(),
        "investigation.triage" => simple::<investigation::triage::Triage>(),
        "key.validation" => schema_union(&[
            simple::<blockchain::ScalarInfo>(),
            simple::<blockchain::wif::WifInfo>(),
        ]),
        "private.candidates" => simple::<Vec<values::Candidate>>(),
        "key.validation_batch" => simple::<Vec<blockchain::candidates::CandidateResult>>(),
        "bitcoin.public_address" => simple::<blockchain::bitcoin::BitcoinAddress>(),
        "ethereum.public_address" => simple::<blockchain::ethereum::EthereumAddress>(),
        "clock.directions" => simple::<symbols::clock::Clock>(),
        "keypad.decoded" => simple::<symbols::keypad::Decoded>(),
        "anagram_report" => simple::<text::anagram::Report>(),
        "scalar_multiset_report" => simple::<text::multiset::Report>(),
        "word_ladder_report" => simple::<text::word_ladder::Report>(),
        "text.word.lengths" => simple::<Vec<text::values::Length>>(),
        "text.codepoints" => simple::<Vec<text::values::Codepoint>>(),
        "text.word.values" => simple::<Vec<text::values::WordValues>>(),
        "text.unicode.inspection" => simple::<text::unicode::Inspection>(),
        "text.unicode.comparison" => simple::<text::unicode::Comparison>(),
        "text.grid_factors" => simple::<text::grid::Factors>(),
        "text.grid" => simple::<text::grid::Grid>(),
        "text.word.extraction" => simple::<text::word::Report>(),
        "text.class.report" => simple::<text::classes::Report>(),
        "text.statistics" => simple::<text::statistics::Statistics>(),
        "text.traversal.report" => simple::<text::traversal::Report>(),
        "text.iterator" => simple::<Vec<String>>(),
        "text.collections" => simple::<Vec<Vec<String>>>(),
        "audio.wave_info" => simple::<media::wave::Info>(),
        "audio.metadata" => simple::<media::metadata::Metadata>(),
        "audio.spectrogram" => simple::<media::spectrogram::Spectrogram>(),
        "audio.spectrum" => simple::<media::spectrum::Spectrum>(),
        "audio.tone_intervals" => simple::<media::morse::Detection>(),
        "audio.morse_transcription" => simple::<media::morse::Transcription>(),
        "audio.dtmf_detection" => simple::<media::dtmf::Detection>(),
        "audio.waveform_envelope" => simple::<media::envelope::Envelope>(),
        "boolean_model" => simple::<logic::boolean::Model>(),
        "boolean_evaluation" => simple::<logic::boolean::Evaluation>(),
        "boolean_analysis" => simple::<logic::boolean::Analysis>(),
        "brainfuck_report" => simple::<esolang::brainfuck::Report>(),
        "brainfuck_comparison" => simple::<esolang::brainfuck::Comparison>(),
        "video.container_metadata" => simple::<media::video::Info>(),
        "video.frames" => simple::<Vec<media::video::frames::Frame>>(),
        "video.audio_track" => simple::<media::video::audio::AudioTrack>(),
        "video.frame_comparisons" => simple::<media::video::comparison::Report>(),
        "web.html_report" => simple::<web::html::Report>(),
        "web.har_report" => simple::<web::har::Report>(),
        "pipeline.definition" => simple::<values::PipelineDefinition>(),
        "pipeline.request" => simple::<values::PipelineRequest>(),
        "pipeline.plan" => simple::<pipeline::PlanInfo<'static>>(),
        "pipeline.execution" => simple::<pipeline::Execution>(),
        "recipe.document" => crate::pipeline::recipe::contract_document(),
        "recipe.plan" => simple::<pipeline::recipe::Info<'static>>(),
        "recipe.check" => simple::<pipeline::recipe::Check>(),
        "recipe.request" => simple::<values::RecipeRequest>(),
        "recipe.contract" => simple::<values::RecipeCheck>(),
        _ => {
            return Err(invalid(&format!(
                "no concrete schema is registered for semantic type {semantic}"
            )));
        }
    };
    Ok(wire::schema(value))
}
fn schema_union(items: &[Schema]) -> Schema {
    let mut variants = items
        .iter()
        .map(|s| s.as_value().clone())
        .collect::<Vec<_>>();
    for (index, variant) in variants.iter_mut().enumerate() {
        rebase_references(variant, &format!("#/anyOf/{index}"));
    }
    Schema::try_from(json!({"anyOf":variants})).expect("union schema")
}
fn named(name: &str, semantic: &str) -> Result<PortContract, OperationError> {
    Ok(PortContract {
        name: name.into(),
        semantic_type: semantic.into(),
        required: true,
        schema: root(semantic)?,
    })
}
/// Composite direct arguments become actual named ports rather than opaque
/// pseudo-types. Legacy parameter values for promoted ports are omitted below.
pub(super) fn ports(semantic: &str, name: &str) -> Result<Vec<PortContract>, OperationError> {
    if name == "output" {
        let mut output = vec![named(name, semantic)?];
        match semantic {
            "image.pixel_data" | "image.bit_plane" | "image.bit_stream" | "video.audio_track"
            | "file.carving" => output.push(named("data", "bytes")?),
            "image.color_layer" => output.push(named("image", "image.samples")?),
            "video.frames" => output.push(PortContract {
                name: "images".into(),
                semantic_type: "image.collection".into(),
                required: true,
                schema: schema::<Vec<values::Image>>(),
            }),
            "image.pair_batch" => output.push(PortContract {
                name: "images".into(),
                semantic_type: "image.collection".into(),
                required: true,
                schema: schema::<Vec<Option<values::Bytes>>>(),
            }),
            _ => (),
        }
        return Ok(output);
    }
    let components: &[(&str, &str)] = match semantic {
        "image.sample_pair" => &[("left", "image.samples"), ("right", "image.samples")],
        "byte_reader_pair" => &[("left", "byte_reader"), ("right", "byte_reader")],
        "text.pair" => &[("input", "text"), ("other", "text")],
        "unsigned_pair" => &[("n", "unsigned"), ("k", "unsigned")],
        "unsigned" => &[("rows", "unsigned")],
        "unsigned_list" => &[("groups", "unsigned_list")],
        "image.collection" => {
            return Ok(vec![PortContract {
                name: "images".into(),
                semantic_type: "image.collection".into(),
                required: true,
                schema: schema::<Vec<values::Image>>(),
            }]);
        }
        "number.u64_pair" => &[("a", "number.u64"), ("b", "number.u64")],
        "number.modular_inverse_request" => &[("a", "number.u64"), ("modulus", "number.u64")],
        "number.modular_power_request" => &[
            ("base", "number.u64"),
            ("exponent", "number.u64"),
            ("modulus", "number.u64"),
        ],
        "file.carve_request" => &[("source", "byte_reader")],
        "constant.selection" | "clock.reading" => &[],
        "text_and_dictionary"
        | "dictionary_and_path"
        | "dictionary_and_endpoints"
        | "boolean_model_and_assignment"
        | "image_and_lags" => return composite(semantic),
        _ => return Ok(vec![named(name, semantic)?]),
    };
    components
        .iter()
        .map(|(name, semantic)| named(name, semantic))
        .collect()
}
fn composite(semantic: &str) -> Result<Vec<PortContract>, OperationError> {
    let array = |name: &str, semantic: &str| PortContract {
        name: name.into(),
        semantic_type: semantic.into(),
        required: true,
        schema: schema::<Vec<String>>(),
    };
    Ok(match semantic {
        "text_and_dictionary" => vec![
            named("input", "text")?,
            array("dictionary", "text.dictionary"),
        ],
        "dictionary_and_path" => vec![
            array("dictionary", "text.dictionary"),
            array("path", "text.list"),
        ],
        "dictionary_and_endpoints" => vec![
            array("dictionary", "text.dictionary"),
            named("start", "text")?,
            named("end", "text")?,
        ],
        "boolean_model_and_assignment" => vec![
            named("model", "boolean_model")?,
            PortContract {
                name: "assignment".into(),
                semantic_type: "boolean.assignment".into(),
                required: true,
                schema: schema::<Vec<logic::boolean::Binding>>(),
            },
        ],
        "image_and_lags" => vec![
            named("image", "image.samples")?,
            PortContract {
                name: "lags".into(),
                semantic_type: "image.lags".into(),
                required: true,
                schema: schema::<Vec<image::spatial::Lag>>(),
            },
        ],
        _ => return Err(invalid("unknown composite input")),
    })
}
