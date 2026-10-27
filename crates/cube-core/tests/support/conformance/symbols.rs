//! Fixed-fixture driver for direct public Core APIs, compiled only by integration tests.
use super::{bytes, field};
use cube_core::{
    ciphers::{bacon, polybius},
    errors::{ErrorCode, OperationError},
    symbols::{self, clock, keypad},
};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Clone, Copy)]
enum Adapter {
    MorseEncode,
    MorseDecode,
    BrailleEncode,
    BrailleDecode,
    BaconEncode,
    BaconDecode,
    PolybiusEncode,
    PolybiusDecode,
    KeypadEncode,
    KeypadDecode,
    Clock,
}
fn adapter(op: &str) -> Option<Adapter> {
    Some(match op {
        "symbol.morse.encode" => Adapter::MorseEncode,
        "symbol.morse.decode" => Adapter::MorseDecode,
        "symbol.braille.encode" => Adapter::BrailleEncode,
        "symbol.braille.decode" => Adapter::BrailleDecode,
        "cipher.bacon.encode" => Adapter::BaconEncode,
        "cipher.bacon.decode" => Adapter::BaconDecode,
        "cipher.polybius.encode" => Adapter::PolybiusEncode,
        "cipher.polybius.decode" => Adapter::PolybiusDecode,
        "symbol.keypad.encode" => Adapter::KeypadEncode,
        "symbol.keypad.decode" => Adapter::KeypadDecode,
        "symbol.clock.directions" => Adapter::Clock,
        _ => return None,
    })
}
pub(super) fn supported(op: &str) -> bool {
    adapter(op).is_some()
}
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "use the exact symbol input, explicit convention and compatible parameter fields",
    )
}
fn string<'a>(v: &'a Value, name: &str) -> Result<&'a str, OperationError> {
    v.get(name).and_then(Value::as_str).ok_or_else(invalid)
}
fn optional_integer(v: &Value, name: &str) -> Result<Option<i64>, OperationError> {
    v.get(name)
        .map(|n| {
            n.as_i64()
                .or_else(|| n.as_str()?.parse().ok())
                .ok_or_else(invalid)
        })
        .transpose()
}
fn square(p: &Value) -> Result<polybius::Square, OperationError> {
    if let Some(alphabet) = p.get("alphabet") {
        let aliases: Vec<String> = p
            .get("aliases")
            .map(|_| field(p, "aliases"))
            .transpose()?
            .unwrap_or_default();
        let aliases = aliases
            .iter()
            .map(|pair| {
                let mut chars = pair.chars();
                match (chars.next(), chars.next(), chars.next(), chars.next()) {
                    (Some(from), Some('='), Some(to), None) => Ok((from, to)),
                    _ => Err(invalid().parameter("aliases")),
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        polybius::Square::new(
            field(p, "side")?,
            alphabet.as_str().ok_or_else(invalid)?,
            &aliases,
        )
    } else {
        if p.get("side").is_some() || p.get("aliases").is_some() {
            return Err(invalid().parameter("alphabet"));
        }
        polybius::Square::preset(string(p, "preset")?)
    }
}
fn output(value: impl Serialize) -> Result<Value, OperationError> {
    serde_json::to_value(value)
        .map(|value| json!({"output": value}))
        .map_err(|_| invalid())
}
pub(super) fn execute(op: &str, inputs: Value, p: Value) -> Result<Value, OperationError> {
    let identity = cube_core::operations::require(op)?.id;
    let selected = adapter(op).ok_or_else(invalid)?;
    let result = (|| match selected {
        Adapter::MorseEncode => output(symbols::morse_encode(string(&inputs, "input")?)?),
        Adapter::MorseDecode => output(symbols::morse_decode(string(&inputs, "input")?)?),
        Adapter::BrailleEncode => output(symbols::braille_encode(&bytes(&inputs["input"])?)?),
        Adapter::BrailleDecode => output(
            json!({"base64":cube_core::encoding::base64_encode(&symbols::braille_decode(string(&inputs,"input")?)?)}),
        ),
        Adapter::BaconEncode => output(bacon::encode(
            string(&inputs, "input")?,
            string(&p, "variant")?,
            string(&p, "notation")?,
        )?),
        Adapter::BaconDecode => output(bacon::decode(
            string(&inputs, "input")?,
            string(&p, "variant")?,
            string(&p, "notation")?,
        )?),
        Adapter::PolybiusEncode => output(polybius::format_tokens(
            &square(&p)?.encode(string(&inputs, "input")?, field(&p, "index_base")?)?,
        )?),
        Adapter::PolybiusDecode => output(square(&p)?.decode(
            &polybius::parse_tokens(string(&inputs, "input")?)?,
            field(&p, "index_base")?,
        )?),
        Adapter::KeypadEncode | Adapter::KeypadDecode => {
            let mapping = p
                .get("mapping")
                .map(|_| keypad::Mapping::parse(string(&p, "mapping")?))
                .transpose()?
                .unwrap_or_else(keypad::Mapping::standard);
            if matches!(selected, Adapter::KeypadEncode) {
                output(mapping.encode(string(&inputs, "input")?, string(&p, "mode")?)?)
            } else {
                output(mapping.decode(string(&inputs, "input")?, string(&p, "mode")?)?)
            }
        }
        Adapter::Clock => output(clock::inspect(
            p.get("time").map(|_| string(&p, "time")).transpose()?,
            optional_integer(&p, "hour_millidegrees")?,
            optional_integer(&p, "minute_millidegrees")?,
            string(&p, "zero_axis")?,
            field(&p, "clockwise")?,
            field(&p, "decode")?,
        )?),
    })();
    result.map_err(|error: OperationError| error.operation(identity))
}
