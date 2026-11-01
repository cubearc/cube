//! Three-trit fixed-width words; arithmetic reuses the Core radix converter.
use super::*;
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    OrderedTernaryV1,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
    #[schemars(length(min = 1, max = 27))]
    pub alphabet: String,
    #[schemars(length(min=3,max=3),extend("x-cube-max-utf8-bytes"=12))]
    pub trits: String,
}
fn table(p: &Parameters, op: &'static str) -> Result<Table, OperationError> {
    let symbols = alphabet::symbols(&p.alphabet, op)?;
    let trits = alphabet::symbols(&p.trits, op)?;
    if symbols.len() > 27 || trits.len() != 3 {
        return Err(invalid(op, "alphabet_or_trits"));
    }
    let tokens = (0..symbols.len())
        .map(|i| {
            let digits = crate::numbers::convert_base(&i.to_string(), 10, 3)?;
            Ok(format!("{digits:0>3}")
                .bytes()
                .map(|digit| trits[usize::from(digit - b'0')])
                .collect::<String>())
        })
        .collect::<Result<Vec<_>, OperationError>>()?;
    Table::new(symbols, tokens)
}
/// Encode exactly three ordered trits, retaining every leading zero trit.
pub fn encode(text: &str, p: &Parameters) -> Result<Encoded, OperationError> {
    const OP: &str = "cipher.triliteral.encode";
    let run = || {
        parameters(p, OP)?;
        text_input(text, OP)?;
        table(p, OP)?.encode(text, p, OP)
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
/// Decode only assigned full-width codewords, with no inference or normalization.
pub fn decode(tokens: &[String], p: &Parameters) -> Result<Decoded, OperationError> {
    const OP: &str = "cipher.triliteral.decode";
    let run = || {
        parameters(p, OP)?;
        token_input(tokens, OP)?;
        table(p, OP)?.decode(tokens, p, OP)
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
