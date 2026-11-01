//! Required finite indexed-prime alphabet, with no global cache or implicit bound.
use super::*;
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Index {
    #[schemars(length(min = 1, max = 4096))]
    pub alphabet: String,
    #[schemars(range(min = 1, max = 4096))]
    pub first_index: u16,
    #[schemars(range(min = 2, max = 1000000))]
    pub max_prime: u32,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    IndexedPrimesV1,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
    pub index: Index,
}
pub(super) fn table(p: &Index, op: &'static str) -> Result<Table, OperationError> {
    let symbols = alphabet::symbols(&p.alphabet, op)?;
    let primes = crate::numbers::primes::indexed(
        usize::from(p.first_index),
        symbols.len(),
        p.max_prime,
        op,
    )?;
    Table::new(symbols, primes.into_iter().map(|p| p.to_string()).collect())
}
/// Encode exact scalars to consecutive indexed primes within the explicit bound.
pub fn encode(text: &str, p: &Parameters) -> Result<Encoded, OperationError> {
    const OP: &str = "cipher.prime_number.encode";
    let run = || {
        parameters(p, OP)?;
        text_input(text, OP)?;
        table(&p.index, OP)?.encode(text, p, OP)
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
/// Decode only exact canonical decimal tokens in the generated table.
pub fn decode(tokens: &[String], p: &Parameters) -> Result<Decoded, OperationError> {
    const OP: &str = "cipher.prime_number.decode";
    let run = || {
        parameters(p, OP)?;
        token_input(tokens, OP)?;
        table(&p.index, OP)?.decode(tokens, p, OP)
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
