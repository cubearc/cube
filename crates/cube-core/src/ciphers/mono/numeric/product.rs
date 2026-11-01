//! Bounded prime products represent multisets, never the original scalar order.
use super::*;
use std::collections::BTreeMap;
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
#[schemars(extend("x-cube-max-serialized-bytes"=1048576))]
pub struct Groups(#[schemars(length(max = 65536))] pub Vec<String>);
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    PrimeMultisetV1,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    pub profile: Profile,
    pub index: prime::Index,
    #[schemars(range(min = 1, max = 1000000))]
    pub max_factor_checks: u32,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Count {
    pub symbol: char,
    #[schemars(range(min = 1, max = 127))]
    pub count: u8,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Group {
    #[schemars(regex(pattern = "^[1-9][0-9]{0,38}$"))]
    pub product: String,
    #[schemars(length(max = 127))]
    pub counts: Vec<Count>,
    #[schemars(length(max = 127))]
    pub canonical_text: String,
    #[schemars(range(min = 0, max = 127))]
    pub scalar_count: u8,
}
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(length(max = 65536))]
    pub groups: Vec<Group>,
    #[schemars(length(min = 1, max = 4096))]
    pub table: Vec<homophonic::Entry>,
    #[schemars(range(min = 0, max = 1000000))]
    pub factor_checks: u32,
    #[schemars(regex(pattern = "^[0-9a-f]{64}$"))]
    pub definition_sha256: String,
}
fn setup(p: &Parameters, values: &[String], op: &'static str) -> Result<Table, OperationError> {
    parameters(p, op)?;
    token_input(values, op)?;
    if !(1..=1000000).contains(&p.max_factor_checks) {
        return Err(invalid(op, "max_factor_checks"));
    }
    prime::table(&p.index, op)
}
fn primes(table: &Table) -> BTreeMap<u32, char> {
    table
        .entries
        .iter()
        .map(|e| (e.tokens[0].parse().expect("generated prime"), e.symbol))
        .collect()
}
fn group(value: u128, counts: BTreeMap<u32, u8>, primes: &BTreeMap<u32, char>) -> Group {
    let mut text = String::new();
    let counts: Vec<_> = counts
        .into_iter()
        .map(|(p, count)| {
            let symbol = primes[&p];
            text.extend(std::iter::repeat_n(symbol, usize::from(count)));
            Count { symbol, count }
        })
        .collect();
    Group {
        product: value.to_string(),
        scalar_count: text.chars().count() as u8,
        canonical_text: text,
        counts,
    }
}
fn resource(op: &'static str) -> OperationError {
    error(
        op,
        ErrorCode::ResourceLimit,
        "products",
        "prime product or factor-work budget exceeded",
    )
}
/// Empty groups encode to1; equal multisets have identical canonical reports.
pub fn encode(groups: &[String], p: &Parameters) -> Result<Report, OperationError> {
    const OP: &str = "cipher.prime_multiplication.encode";
    let run = || {
        let table = setup(p, groups, OP)?;
        let primes = primes(&table);
        let mut units = 0usize;
        let mut result = Vec::new();
        for text in groups {
            units = units.saturating_add(text.chars().count());
            bound(OP, "groups", units, MAX_TOKENS)?;
            let mut value = 1u128;
            let mut counts = BTreeMap::new();
            for token in table.mapping.encode(text, &[])? {
                let prime: u32 = token.parse().expect("generated prime");
                value = value
                    .checked_mul(u128::from(prime))
                    .ok_or_else(|| resource(OP))?;
                *counts.entry(prime).or_insert(0u8) += 1;
            }
            result.push(group(value, counts, &primes));
        }
        complete(
            Report {
                groups: result,
                table: table.entries,
                factor_checks: 0,
                definition_sha256: hash(p),
            },
            OP,
        )
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
/// Trial division is confined to the configured alphabet and charged per group
/// and attempted remainder, including repeated successes and failed divisions.
pub fn decode(products: &[String], p: &Parameters) -> Result<Report, OperationError> {
    const OP: &str = "cipher.prime_multiplication.decode";
    let run = || {
        let table = setup(p, products, OP)?;
        let primes = primes(&table);
        let mut checks = 0u32;
        let mut units = 0usize;
        let mut result = Vec::new();
        let mut charge = || {
            checks += 1;
            if checks > p.max_factor_checks {
                Err(resource(OP))
            } else {
                Ok(())
            }
        };
        for token in products {
            charge()?;
            let value = token
                .parse::<u128>()
                .ok()
                .filter(|n| *n > 0 && n.to_string() == *token)
                .ok_or_else(|| {
                    error(
                        OP,
                        ErrorCode::InvalidInput,
                        "products",
                        "expected canonical positive u128 decimal strings",
                    )
                })?;
            let mut remaining = value;
            let mut counts = BTreeMap::new();
            for &prime in primes.keys() {
                while remaining > 1 {
                    charge()?;
                    if !remaining.is_multiple_of(u128::from(prime)) {
                        break;
                    }
                    remaining /= u128::from(prime);
                    units += 1;
                    bound(OP, "products", units, MAX_TOKENS)?;
                    *counts.entry(prime).or_insert(0u8) += 1;
                }
                if remaining == 1 {
                    break;
                }
            }
            if remaining != 1 {
                return Err(error(
                    OP,
                    ErrorCode::InvalidInput,
                    "products",
                    "product contains a factor outside the configured alphabet",
                ));
            }
            result.push(group(value, counts, &primes));
        }
        complete(
            Report {
                groups: result,
                table: table.entries,
                factor_checks: checks,
                definition_sha256: hash(p),
            },
            OP,
        )
    };
    run().map_err(|e: OperationError| e.operation(OP))
}
