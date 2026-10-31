//! Reproducible contract fixtures call the real APIs; never a user dispatcher.
use super::{checkerboard, grandpre, homophonic, params, rotations};
use crate::{
    errors::{ErrorCode, OperationError},
    operations::contracts::Fixture,
};
use serde_json::{Value, json};
fn answer(
    op: &'static str,
    inputs: Value,
    parameters: Value,
    name: &str,
    actual: Value,
    expected: &str,
) -> Result<Fixture, OperationError> {
    let expected: Value = serde_json::from_str(expected).expect("static known-answer JSON");
    if actual != expected {
        return Err(super::error(
            op,
            ErrorCode::InvalidInput,
            "fixture",
            "operation disagrees with its independent known answer",
        ));
    }
    let mut outputs = serde_json::Map::new();
    outputs.insert(name.into(), actual);
    Ok(Fixture {
        inputs,
        parameters,
        outputs: Value::Object(outputs),
    })
}

pub(crate) fn substitution_encode() -> Result<Fixture, OperationError> {
    let parameters: params::Substitution =
        serde_json::from_str("{\"source\":\"ABC\",\"target\":\"CAB\"}")
            .expect("static typed parameters");
    let map = parameters.mapping()?;
    let actual = map.encode("ABC", parameters.unmapped())?;
    let parameters = serde_json::to_value(&parameters).expect("typed parameters");
    answer(
        "cipher.substitution.encode",
        serde_json::from_str("{\"text\":\"ABC\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"CAB\"",
    )
}
pub(crate) fn substitution_decode() -> Result<Fixture, OperationError> {
    let parameters: params::Substitution =
        serde_json::from_str("{\"source\":\"ABC\",\"target\":\"CAB\"}")
            .expect("static typed parameters");
    let map = parameters.mapping()?;
    let actual = map.decode("CAB", parameters.unmapped())?;
    let parameters = serde_json::to_value(&parameters).expect("typed parameters");
    answer(
        "cipher.substitution.decode",
        serde_json::from_str("{\"text\":\"CAB\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"ABC\"",
    )
}
pub(crate) fn rot5() -> Result<Fixture, OperationError> {
    let actual = rotations::rot5("A1")?;
    let parameters = json!({});
    answer(
        "cipher.rot5",
        serde_json::from_str("{\"text\":\"A1\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"A6\"",
    )
}
pub(crate) fn rot18() -> Result<Fixture, OperationError> {
    let actual = rotations::rot18("A1")?;
    let parameters = json!({});
    answer(
        "cipher.rot18",
        serde_json::from_str("{\"text\":\"A1\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"N6\"",
    )
}
pub(crate) fn rot47() -> Result<Fixture, OperationError> {
    let actual = rotations::rot47("!")?;
    let parameters = json!({});
    answer(
        "cipher.rot47",
        serde_json::from_str("{\"text\":\"!\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"P\"",
    )
}
pub(crate) fn homophonic_encode() -> Result<Fixture, OperationError> {
    let parameters:params::HomophonicEncode=serde_json::from_str("{\"definition\":{\"ascii_uppercase\":true,\"entries\":[{\"symbol\":\"A\",\"tokens\":[\"1\",\"2\"]},{\"symbol\":\"B\",\"tokens\":[\"3\"]}]}}").expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let map = homophonic::Mapping::new(parameters.definition)?;
    let actual = map.encode("AABA", &parameters.counters)?;
    let parameters = value;
    answer(
        "cipher.homophonic.encode",
        serde_json::from_str("{\"text\":\"AABA\"}").expect("static input"),
        parameters,
        "tokens",
        json!(actual),
        "[\"1\",\"2\",\"3\",\"1\"]",
    )
}
pub(crate) fn homophonic_decode() -> Result<Fixture, OperationError> {
    let parameters:params::Definition<homophonic::Definition>=serde_json::from_str("{\"definition\":{\"ascii_uppercase\":true,\"entries\":[{\"symbol\":\"A\",\"tokens\":[\"1\",\"2\"]},{\"symbol\":\"B\",\"tokens\":[\"3\"]}]}}").expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let map = homophonic::Mapping::new(parameters.definition)?;
    let tokens = super::symbol_tokens("[\"1\",\"2\"]")?;
    let actual = map.decode(&tokens)?;
    let parameters = value;
    answer(
        "cipher.homophonic.decode",
        serde_json::from_str("{\"tokens\":[\"1\",\"2\"]}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"AA\"",
    )
}
pub(crate) fn straddling_encode() -> Result<Fixture, OperationError> {
    let parameters: params::Definition<checkerboard::Definition> = serde_json::from_str(
        "{\"definition\":{\"digits\":\"0123456789\",\"prefixes\":\"37\",\"alphabet\":\"ABC\"}}",
    )
    .expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let table = checkerboard::Table::new(checkerboard::Profile::Straddling, parameters.definition)?;
    let actual = table.encode("AB")?;
    let parameters = value;
    answer(
        "cipher.straddling.encode",
        serde_json::from_str("{\"text\":\"AB\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"01\"",
    )
}
pub(crate) fn straddling_decode() -> Result<Fixture, OperationError> {
    let parameters: params::Definition<checkerboard::Definition> = serde_json::from_str(
        "{\"definition\":{\"digits\":\"0123456789\",\"prefixes\":\"37\",\"alphabet\":\"ABC\"}}",
    )
    .expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let table = checkerboard::Table::new(checkerboard::Profile::Straddling, parameters.definition)?;
    let actual = table.decode("01")?;
    let parameters = value;
    answer(
        "cipher.straddling.decode",
        serde_json::from_str("{\"text\":\"01\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"AB\"",
    )
}
pub(crate) fn monome_dinome_encode() -> Result<Fixture, OperationError> {
    let parameters:params::Definition<checkerboard::Definition>=serde_json::from_str("{\"definition\":{\"digits\":\"6318927054\",\"prefixes\":\"63\",\"alphabet\":\"NOTARIESBCDFGHKLMPQUVWXY\",\"aliases\":[{\"from\":\"J\",\"to\":\"I\"},{\"from\":\"Z\",\"to\":\"Y\"}]}}").expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let table =
        checkerboard::Table::new(checkerboard::Profile::MonomeDinome, parameters.definition)?;
    let actual = table.encode("HIGHF")?;
    let parameters = value;
    answer(
        "cipher.monome_dinome.encode",
        serde_json::from_str("{\"text\":\"HIGHF\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"600676062\"",
    )
}
pub(crate) fn monome_dinome_decode() -> Result<Fixture, OperationError> {
    let parameters:params::Definition<checkerboard::Definition>=serde_json::from_str("{\"definition\":{\"digits\":\"6318927054\",\"prefixes\":\"63\",\"alphabet\":\"NOTARIESBCDFGHKLMPQUVWXY\",\"aliases\":[{\"from\":\"J\",\"to\":\"I\"},{\"from\":\"Z\",\"to\":\"Y\"}]}}").expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let table =
        checkerboard::Table::new(checkerboard::Profile::MonomeDinome, parameters.definition)?;
    let actual = table.decode("600676062")?;
    let parameters = value;
    answer(
        "cipher.monome_dinome.decode",
        serde_json::from_str("{\"text\":\"600676062\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"HIGHF\"",
    )
}
pub(crate) fn tridigital_encode() -> Result<Fixture, OperationError> {
    let parameters:params::Definition<checkerboard::Definition>=serde_json::from_str("{\"definition\":{\"digits\":\"6703528149\",\"prefixes\":\"\",\"alphabet\":\"DRAGONFLYBCEHIJKMPQSTUVWXZ\"}}").expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let table = checkerboard::Table::new(checkerboard::Profile::Tridigital, parameters.definition)?;
    let actual = table.encode("THE")?;
    let parameters = value;
    answer(
        "cipher.tridigital.encode",
        serde_json::from_str("{\"text\":\"THE\"}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"030\"",
    )
}
pub(crate) fn tridigital_decode() -> Result<Fixture, OperationError> {
    let parameters:params::Definition<checkerboard::Definition>=serde_json::from_str("{\"definition\":{\"digits\":\"6703528149\",\"prefixes\":\"\",\"alphabet\":\"DRAGONFLYBCEHIJKMPQSTUVWXZ\"}}").expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let table = checkerboard::Table::new(checkerboard::Profile::Tridigital, parameters.definition)?;
    let actual = table.alternatives("0")?;
    let parameters = value;
    answer(
        "cipher.tridigital.decode",
        serde_json::from_str("{\"text\":\"0\"}").expect("static input"),
        parameters,
        "alternatives",
        json!(actual),
        "[[\"A\",\"E\",\"T\"]]",
    )
}
pub(crate) fn grandpre_encode() -> Result<Fixture, OperationError> {
    let parameters:params::Definition<grandpre::Definition>=serde_json::from_str("{\"definition\":{\"rows\":[\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\"],\"row_labels\":\"12345678\",\"column_labels\":\"12345678\"}}").expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let square = grandpre::Square::new(parameters.definition)?;
    let actual = square.encode("A A")?;
    let parameters = value;
    answer(
        "cipher.grandpre.encode",
        serde_json::from_str("{\"text\":\"A A\"}").expect("static input"),
        parameters,
        "tokens",
        json!(actual),
        "[[1,1],null,[1,2]]",
    )
}
pub(crate) fn grandpre_decode() -> Result<Fixture, OperationError> {
    let parameters:params::Definition<grandpre::Definition>=serde_json::from_str("{\"definition\":{\"rows\":[\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\",\"AAAAAAAA\"],\"row_labels\":\"12345678\",\"column_labels\":\"12345678\"}}").expect("static typed parameters");
    let value = serde_json::to_value(&parameters).expect("typed parameters");
    let square = grandpre::Square::new(parameters.definition)?;
    let tokens = super::coordinate_tokens("[[1,1],null]")?;
    let actual = square.decode(&tokens)?;
    let parameters = value;
    answer(
        "cipher.grandpre.decode",
        serde_json::from_str("{\"tokens\":[[1,1],null]}").expect("static input"),
        parameters,
        "text",
        json!(actual),
        "\"A \"",
    )
}

pub(crate) fn rot8000() -> Result<Fixture, OperationError> {
    answer(
        "cipher.rot8000",
        json!({"text":"A 😀"}),
        json!({}),
        "text",
        json!(rotations::rot8000("A 😀")?),
        "\"籊 😀\"",
    )
}
