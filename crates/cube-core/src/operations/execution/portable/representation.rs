//! Adapt the named binary-format ports to the owning Core APIs.
use super::*;
use crate::{
    binary::{bcd, bit_units, reed_solomon, representation, varint},
    encoding::rfc4648,
    hashing::fingerprints,
    operations::{contracts::values::Bytes, representation as catalog},
};
use zeroize::Zeroizing;

pub(super) fn supports(id: &str) -> bool {
    catalog::OPERATIONS.iter().any(|row| row.id == id)
}

fn exact_block(value: catalog::BlockBytes) -> Result<Vec<u8>, OperationError> {
    crate::encoding::base64_decode(&value.base64)
        .map_err(|_| crate::formal_work::invalid("use canonical exact Base64 bytes"))
}

fn blocks_encode(inputs: &Value, parameters: Value) -> Result<Value, OperationError> {
    let blocks = field::<Vec<catalog::BlockBytes>>(inputs, "blocks")?
        .into_iter()
        .map(exact_block)
        .collect::<Result<Vec<_>, _>>()?;
    let slices: Vec<_> = blocks.iter().map(Vec::as_slice).collect();
    let outputs: Vec<_> = reed_solomon::encode(&slices, options(parameters)?)?
        .into_iter()
        .map(|out| catalog::Encoded {
            parity: Bytes { data: out.parity },
            codeword: Bytes { data: out.codeword },
        })
        .collect();
    Ok(json!({"blocks": outputs}))
}

fn blocks_decode(inputs: &Value, parameters: Value) -> Result<Value, OperationError> {
    let blocks = field::<Vec<catalog::Erased>>(inputs, "blocks")?
        .into_iter()
        .map(|block| {
            Ok((
                exact_block(block.codeword)?,
                block
                    .erasures
                    .into_iter()
                    .map(usize::from)
                    .collect::<Vec<_>>(),
            ))
        })
        .collect::<Result<Vec<_>, OperationError>>()?;
    let damaged: Vec<_> = blocks
        .iter()
        .map(|(codeword, erasures)| reed_solomon::Damaged { codeword, erasures })
        .collect();
    let outputs: Vec<_> = reed_solomon::decode(&damaged, options(parameters)?)?
        .into_iter()
        .map(|out| catalog::Corrected {
            message: Bytes { data: out.message },
            codeword: Bytes { data: out.codeword },
            positions: out.positions,
            initial_syndromes: out.initial_syndromes,
            verified_syndromes: out.verified_syndromes,
        })
        .collect();
    Ok(json!({"blocks": outputs}))
}

fn options<T: DeserializeOwned>(parameters: Value) -> Result<T, OperationError> {
    serde_json::from_value(parameters).map_err(|_| {
        crate::formal_work::parameter("parameters do not match the selected binary format")
    })
}

fn output(data: Vec<u8>) -> Value {
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
    let codec_bytes = || bytes(&inputs["input"]);
    let codec_text = || field::<String>(&inputs, "input");
    match id {
        "binary.pack_bits" => {
            let (packed, count) = bit_units::pack(&data()?, options(parameters)?)?;
            Ok(
                json!({"output":crate::operations::execution::byte_value(packed),"valid_bits":count}),
            )
        }
        "binary.unpack_bits" => {
            let (unpacked, count) = bit_units::unpack(&data()?, options(parameters)?)?;
            Ok(
                json!({"output":crate::operations::execution::byte_value(unpacked),"valid_bits":count}),
            )
        }
        "binary.mask" => Ok(output(bit_units::mask(&data()?, options(parameters)?)?)),
        "binary.count_bits" => {
            Ok(json!({"counts": bit_units::count(&data()?, options(parameters)?)?}))
        }
        "binary.fixed_integer.encode" => Ok(output(representation::encode(
            &field::<Vec<representation::Integer>>(&inputs, "values")?,
            options(parameters)?,
        )?)),
        "binary.fixed_integer.decode" => {
            Ok(json!({"result":representation::decode(&data()?, options(parameters)?)?}))
        }
        "binary.bcd.encode" => Ok(output(bcd::encode(
            &field::<String>(&inputs, "digits")?,
            field(&inputs, "negative")?,
            options(parameters)?,
        )?)),
        "binary.bcd.decode" => Ok(json!({"result":bcd::decode(&data()?, options(parameters)?)?})),
        "binary.varint.encode" => Ok(output(varint::encode(
            &field::<Vec<representation::Integer>>(&inputs, "values")?,
            options(parameters)?,
        )?)),
        "binary.varint.decode" => {
            Ok(json!({"records":varint::decode(&data()?, options(parameters)?)?}))
        }
        "binary.ieee754.inspect" => {
            Ok(json!({"records":representation::inspect_floats(&data()?, options(parameters)?)?}))
        }
        "binary.reed_solomon.encode" => blocks_encode(&inputs, parameters),
        "binary.reed_solomon.decode" => blocks_decode(&inputs, parameters),
        "hash.noncrypto" => Ok(
            json!({"output":fingerprints::noncrypto(&codec_bytes()?, options(parameters)?)?.hex}),
        ),
        "hash.siphash24" => {
            let key = Zeroizing::new(bytes(&inputs["key"])?);
            Ok(json!({"digest":fingerprints::siphash24(&data()?, key.as_slice())?}))
        }
        "encoding.base32hex.encode" => {
            Ok(json!({"output":rfc4648::encode_base32hex(&codec_bytes()?)?}))
        }
        "encoding.base32hex.decode" => Ok(output(rfc4648::decode_base32hex(&codec_text()?)?)),
        "encoding.base64url.encode" => {
            Ok(json!({"output":rfc4648::encode_base64url(&codec_bytes()?, options(parameters)?)?}))
        }
        "encoding.base64url.decode" => Ok(output(rfc4648::decode_base64url(
            &codec_text()?,
            options(parameters)?,
        )?)),
        _ => Err(crate::formal_work::invalid(
            "unknown binary representation operation",
        )),
    }
}
