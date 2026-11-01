//! Typed named-port adapter over public Core APIs; no algorithm implementation.
use crate::{
    errors::{ErrorCode, OperationError},
    numbers::{self, combinatorics as counts, constants, modular},
};
use serde_json::{Value, json};
fn invalid() -> OperationError {
    OperationError::new(
        ErrorCode::InvalidInput,
        "use exact numeric ports and parameters",
    )
}
pub(super) fn supports(op: &str) -> bool {
    numbers::OPERATIONS
        .iter()
        .chain(modular::OPERATIONS)
        .chain(counts::OPERATIONS)
        .chain(constants::OPERATIONS)
        .chain(numbers::alphabet::OPERATIONS)
        .chain(numbers::digital::OPERATIONS)
        .any(|d| d.id == op)
}
fn number(value: &Value, name: &str) -> Result<u64, OperationError> {
    let value = value.get(name).ok_or_else(invalid)?;
    value
        .as_u64()
        .or_else(|| value.as_str()?.parse().ok())
        .ok_or_else(invalid)
}
fn text<'a>(value: &'a Value, name: &str) -> Result<&'a str, OperationError> {
    value.get(name).and_then(Value::as_str).ok_or_else(invalid)
}
pub(super) fn execute(
    op: &str,
    inputs: Value,
    p: Value,
    _budget: crate::operations::execution::Budget,
    _clock: Option<crate::operations::execution::Elapsed<'_>>,
) -> Result<Value, OperationError> {
    let a = |name| number(&inputs, name);
    let n = |name| number(&p, name);
    let t = |name| text(&p, name);
    let limits = || -> Result<counts::Limits, OperationError> {
        Ok(counts::Limits {
            max_work: n("max_work")?,
            max_report_bytes: usize::try_from(n("max_report_bytes")?).map_err(|_| invalid())?,
        })
    };
    let value = match op {
        "number.gcd" => json!(modular::gcd(a("a")?, a("b")?)?),
        "number.extended_gcd" => json!(modular::extended_gcd(a("a")?, a("b")?)?),
        "number.mod_inverse" => json!(modular::mod_inverse(a("a")?, a("modulus")?)?),
        "number.mod_pow" => json!(modular::mod_pow(a("base")?, a("exponent")?, a("modulus")?)?),
        "number.binomial" => json!(counts::binomial(a("n")?, a("k")?, limits()?)?),
        "number.permutations" => json!(counts::permutations(a("n")?, a("k")?, limits()?)?),
        "number.multinomial" => {
            let groups = inputs["groups"]
                .as_array()
                .ok_or_else(invalid)?
                .iter()
                .map(|v| number(&json!({"value":v}), "value"))
                .collect::<Result<Vec<_>, _>>()?;
            json!(counts::multinomial(&groups, limits()?)?)
        }
        "number.pascal" => json!(counts::pascal(
            a("rows")?,
            n("max_work")?,
            usize::try_from(n("max_output_bytes")?).map_err(|_| invalid())?
        )?),
        "number.base.convert" => json!(numbers::convert_base(
            text(&inputs, "input")?,
            n("from_base")?.try_into().map_err(|_| invalid())?,
            n("to_base")?.try_into().map_err(|_| invalid())?
        )?),
        "number.alphabet.convert" => json!(numbers::alphabet::convert(
            text(&inputs, "input")?,
            t("source_alphabet")?,
            t("target_alphabet")?,
            n("from_base")?.try_into().map_err(|_| invalid())?,
            n("to_base")?.try_into().map_err(|_| invalid())?
        )?),
        "number.digital_root" => json!(numbers::digital::from_decimal(
            text(&inputs, "input")?,
            n("radix")?.try_into().map_err(|_| invalid())?
        )?),
        "number.constant_digits" => {
            let indices = p
                .get("indices")
                .map(|_| constants::parse_indices(t("indices")?))
                .transpose()?;
            let optional = |name| {
                p.get(name)
                    .map(|_| usize::try_from(n(name)?).map_err(|_| invalid()))
                    .transpose()
            };
            json!(constants::digits(
                constants::Constant::parse(t("constant")?)?,
                constants::DigitOptions {
                    start: optional("start")?,
                    count: optional("count")?,
                    indices: indices.as_deref(),
                    max_digits: n("max_digits")?.try_into().map_err(|_| invalid())?
                }
            )?)
        }
        _ => return Err(invalid()),
    };
    Ok(json!({"output":value}))
}
