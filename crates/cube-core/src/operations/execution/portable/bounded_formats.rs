//! Named-port adapters for bounded coding and inert inspection.
use super::*;
use crate::{
    binary::compression::{
        bzip2, dictionary, huffman, lz4, lzw, packbits, probability, transforms,
    },
    forensics::formats::{bmp, executable, mpeg, ogg, tiff, webp},
    operations::bounded_formats as spec,
};

pub(super) fn supports(operation: &str) -> bool {
    spec::OPERATIONS.iter().any(|row| row.id == operation)
}

fn options<T: DeserializeOwned>(parameters: Value) -> Result<T, OperationError> {
    serde_json::from_value(parameters).map_err(|_| {
        OperationError::new(
            ErrorCode::InvalidParameter,
            "parameters do not match the selected format",
        )
    })
}

fn raw(data: Vec<u8>) -> Value {
    json!({"output": crate::operations::execution::byte_value(data)})
}

pub(super) fn execute(
    id: &str,
    inputs: Value,
    parameters: Value,
    _: super::super::Budget,
    _: Option<super::super::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let data = || bytes(&inputs["data"]);
    let frequencies = || spec::frequencies(field(&inputs, "frequencies")?);
    Ok(match id {
        spec::PACKBITS_ENCODE => raw(packbits::encode(&bytes(&inputs["input"])?)?),
        spec::PACKBITS_DECODE => {
            let settings: spec::Expected = options(parameters)?;
            raw(packbits::decode(
                &bytes(&inputs["input"])?,
                settings.expected_length,
            )?)
        }
        "binary.lz77.encode" => {
            let settings: spec::Window = options(parameters)?;
            json!({"tokens":dictionary::lz77_encode(&data()?, settings.window)?})
        }
        "binary.lz77.decode" => {
            let settings: spec::Window = options(parameters)?;
            raw(dictionary::lz77_decode(
                &field::<Vec<dictionary::Lz77Token>>(&inputs, "tokens")?,
                settings.window,
            )?)
        }
        "binary.lz78.encode" => json!({"tokens": dictionary::lz78_encode(&data()?)?}),
        "binary.lz78.decode" => raw(dictionary::lz78_decode(&field::<
            Vec<dictionary::Lz78Token>,
        >(&inputs, "tokens")?)?),
        "binary.bwt.encode" => {
            let (encoded, primary) = transforms::bwt(&data()?)?;
            json!({"output":crate::operations::execution::byte_value(encoded),"primary_index":primary})
        }
        "binary.bwt.decode" => {
            let settings: spec::Primary = options(parameters)?;
            raw(transforms::inverse_bwt(&data()?, settings.primary_index)?)
        }
        "binary.mtf.encode" | "binary.mtf.decode" => {
            let settings: spec::Permutation = options(parameters)?;
            raw(transforms::mtf(
                &data()?,
                settings.alphabet.as_deref(),
                id.ends_with(".decode"),
            )?)
        }
        "binary.lz4.encode" => {
            let settings: spec::Lz4Encode = options(parameters)?;
            raw(lz4::encode(&data()?, settings.mode, settings.limits)?)
        }
        "binary.lz4.decode" => {
            let (mode, limits) = options::<spec::Lz4Decode>(parameters)?.parts();
            raw(lz4::decode(&data()?, mode, limits)?)
        }
        "binary.bzip2.encode" => {
            let settings: spec::Bzip2Level = options(parameters)?;
            raw(bzip2::encode(&data()?, settings.level)?)
        }
        "binary.bzip2.decode" => {
            let settings: spec::Bzip2Members = options(parameters)?;
            raw(bzip2::decode(&data()?, settings.concatenate)?)
        }
        "binary.lzw.encode" => raw(lzw::encode(&data()?, options(parameters)?)?),
        "binary.lzw.decode" => raw(lzw::decode(&data()?, options(parameters)?)?),
        "binary.huffman.construct" => json!({"codes":huffman::construct(&frequencies()?)?}),
        "binary.huffman.encode" => {
            let result = huffman::encode(&data()?, &frequencies()?)?;
            json!({"output":crate::operations::execution::byte_value(result.data), "bit_count":result.bit_count,"codes":result.codes})
        }
        "binary.huffman.decode" => {
            let settings: spec::HuffmanCounts = options(parameters)?;
            raw(huffman::decode(
                &data()?,
                &field::<Vec<huffman::Length>>(&inputs, "lengths")?,
                settings.bit_count,
                settings.symbol_count,
            )?)
        }
        "binary.entropy.encode" => {
            let settings: spec::EntropyMode = options(parameters)?;
            raw(probability::encode(
                &data()?,
                &frequencies()?,
                settings.profile,
            )?)
        }
        "binary.entropy.decode" => {
            let settings: spec::EntropyDecode = options(parameters)?;
            raw(probability::decode(
                &data()?,
                &frequencies()?,
                settings.symbol_count,
                settings.profile,
            )?)
        }
        "file.bmp.inspect" => json!({"report":bmp::inspect(&data()?)?}),
        "file.webp.inspect" => json!({"report":webp::inspect(&data()?)?}),
        "file.tiff.inspect" => json!({"report":tiff::inspect(&data()?)?}),
        "file.executable.inspect" => json!({"report":executable::inspect(&data()?)?}),
        "file.mpeg_audio.inspect" => json!({"report":mpeg::inspect(&data()?)?}),
        "file.ogg.inspect" => json!({"report":ogg::inspect(&data()?)?}),
        _ => unreachable!("operation is checked against the bounded-format catalog"),
    })
}
