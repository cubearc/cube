//! Explicit coprime affine residues; arithmetic remains in numbers::modular.
use super::*;
use crate::numbers::modular;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    AffineResidueV1,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
    #[schemars(length(min = 2, max = 4096))]
    pub alphabet: String,
    #[schemars(range(min = 2, max = 4096))]
    pub modulus: u16,
    #[schemars(regex(pattern = "^(0|[1-9][0-9]{0,19})$"))]
    pub multiplier: String,
    #[schemars(regex(pattern = "^(0|[1-9][0-9]{0,19})$"))]
    pub offset: String,
    #[schemars(range(min = 0, max = 1))]
    pub token_origin: u8,
}
fn operand(value: &str, name: &'static str, op: &'static str) -> Result<u64, OperationError> {
    let parsed = modular::parse_operand(value, op, name)?;
    if parsed.to_string() != value {
        return Err(invalid(op, name));
    }
    Ok(parsed)
}
fn table(p: &Parameters, op: &'static str) -> Result<Table, OperationError> {
    let symbols = alphabet::symbols(&p.alphabet, op)?;
    let modulus = u64::from(p.modulus);
    if !(2..=4096).contains(&p.modulus)
        || symbols.len() != usize::from(p.modulus)
        || p.token_origin > 1
    {
        return Err(invalid(op, "modulus_or_origin"));
    }
    let a = operand(&p.multiplier, "multiplier", op)?;
    let b = operand(&p.offset, "offset", op)? % modulus;
    if modular::gcd(a, modulus)? != 1 {
        return Err(invalid(op, "multiplier"));
    }
    let tokens = (0..modulus)
        .map(|i| {
            modular::multiply(a, i, modulus)
                .map(|v| ((v + b) % modulus + u64::from(p.token_origin)).to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Table::new(symbols, tokens)
}
/// Encode zero-based source index i as ((a*i+b) mod modulus)+token_origin.
pub fn encode(text: &str, p: &Parameters) -> Result<Encoded, OperationError> {
    const OP: &str = "cipher.modulo.encode";
    let run = || {
        parameters(p, OP)?;
        text_input(text, OP)?;
        table(p, OP)?.encode(text, p, OP)
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
/// Decode the exact inverse table; noncoprime definitions reject in both directions.
pub fn decode(tokens: &[String], p: &Parameters) -> Result<Decoded, OperationError> {
    const OP: &str = "cipher.modulo.decode";
    let run = || {
        parameters(p, OP)?;
        token_input(tokens, OP)?;
        table(p, OP)?.decode(tokens, p, OP)
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
