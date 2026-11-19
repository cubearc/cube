//! Numeric operation contracts and independent small known answers.
use super::*;
use crate::operations::{self as o, contracts as c};
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct NoParameters {}

struct Row {
    contract: c::Entry,
    execute: fn(Value, &'static str) -> Result<Value, OperationError>,
}

const fn declare(id: &'static str, mut row: Row) -> Row {
    row.contract.descriptor.id = id;
    row
}

macro_rules! profile {
    ($name:literal, $slug:literal, $description:literal,
     $request:ty => $report:ty, $run:path,
     ($input:expr, $output:expr, $items:expr, $work:expr, $memory:expr),
     $sample:literal => $answer:literal) => {
        Row {
            contract: c::Entry {
                operation_version: 1,
                descriptor: o::Descriptor {
                    id: "",
                    category: "numbers",
                    description: $description,
                    input_type: "json",
                    output_type: "json",
                    parameters: &[],
                    deterministic: true,
                    reversible: Some(false),
                    command: concat!("number analysis ", $slug),
                    example: concat!("cube number analysis ", $slug, " --text '", $sample, "'"),
                },
                inputs: &[c::Port {
                    name: "request",
                    semantic_type: concat!("number.", $name, ".request"),
                    schema: c::schema::<$request>,
                    required: true,
                }],
                parameters: c::schema::<NoParameters>,
                outputs: &[c::Port {
                    name: "report",
                    semantic_type: concat!("number.", $name, ".report"),
                    schema: c::schema::<$report>,
                    required: true,
                }],
                limits: c::Limits {
                    max_input_bytes: Some($input),
                    max_output_bytes: Some($output),
                    max_items: Some($items),
                    max_work: Some($work),
                    max_memory_bytes: Some($memory),
                },
                fixture: || Ok(c::Fixture {
                    inputs: json!({"request": serde_json::from_str::<Value>($sample).expect("fixture request")}),
                    parameters: json!({}),
                    outputs: json!({"report": serde_json::from_str::<Value>($answer).expect("fixture answer")}),
                }),
            },
            execute: |value, operation| {
                let request: $request = serde_json::from_value(value)
                    .map_err(|_| invalid("request does not match the selected numeric operation")
                        .operation(operation))?;
                let report = $run(&request).map_err(|e| e.operation(operation))?;
                check_report(&report, $output).map_err(|e| e.operation(operation))?;
                serde_json::to_value(report)
                    .map_err(|_| invalid("numeric report could not be serialized").operation(operation))
            },
        }
    };
}

const ROWS: [Row; 23] = [
    declare(
        "number.roman",
        profile!("roman", "roman", "Read or write canonical Roman numerals from 1 through 3999.",
    representations::RomanRequest => representations::RomanReport, representations::roman,
    (1024, 1024, 15, 256, 1024),
    r#"{"direction":"decode","text":"MCMXCIV"}"#
    => r#"{"value":1994,"text":"MCMXCIV"}"#),
    ),
    declare(
        "number.balanced_ternary",
        profile!("balanced_ternary", "balanced-ternary", "Convert signed magnitudes to or from exact balanced ternary trits.",
    representations::TernaryRequest => representations::TernaryReport, representations::balanced_ternary,
    (8192, 1024, 82, 4352, 4096),
    r#"{"direction":"encode","value":"-2"}"#
    => r#"{"value":"-2","text":"T1","trits":[-1,1]}"#),
    ),
    declare(
        "number.gray",
        profile!("gray", "gray", "Encode or decode a width-specific reflected Gray code.",
    representations::GrayRequest => representations::GrayReport, representations::gray,
    (1024, 1024, 128, 128, 1024),
    r#"{"direction":"encode","width":4,"value":"14","binary":true}"#
    => r#"{"value":"9","binary":"1001"}"#),
    ),
    declare(
        "number.decimal_check",
        profile!("decimal_check", "decimal-check", "Summarize decimal digits or test an explicit equation modulo nine.",
    representations::DecimalRequest => representations::DecimalReport, representations::decimal_check,
    (16 << 10, 1024, 12288, 12320, 4096),
    r#"{"action":"summarize","value":"-38"}"#
    => r#"{"action":"summarize","digit_sum":11,"digital_root":2,"residue9":7}"#),
    ),
    declare(
        "number.crt",
        profile!("crt", "crt", "Merge generalized Chinese remainder constraints, including noncoprime moduli.",
    arithmetic::CrtRequest => arithmetic::CrtReport, arithmetic::crt,
    (64 << 10, 8 << 10, 256, 65536, 64 << 10),
    r#"{"pairs":[{"residue":2,"modulus":6},{"residue":8,"modulus":9}]}"#
    => r#"{"status":"solution","residue":8,"modulus":18}"#),
    ),
    declare(
        "number.primality",
        profile!("primality", "primality", "Test an exact u32 by trial division or fixed-base Miller–Rabin.",
    arithmetic::PrimeRequest => arithmetic::PrimeReport, arithmetic::primality,
    (1024, 1024, 1, 65534, 1024),
    r#"{"value":4294967291,"method":"miller_rabin"}"#
    => r#"{"method":"miller_rabin","prime":true}"#),
    ),
    declare(
        "number.sieve",
        profile!("sieve", "sieve", "List bounded primes using Eratosthenes or Atkin.",
    arithmetic::SieveRequest => arithmetic::SieveReport, arithmetic::sieve,
    (1024, 1100256, 100000, 32 << 20, 4 << 20),
    r#"{"limit":20,"method":"atkin"}"#
    => r#"{"method":"atkin","primes":[2,3,5,7,11,13,17,19]}"#),
    ),
    declare(
        "number.factor",
        profile!("factor", "factor", "Factor u32 by trial division or search for one bounded Pollard rho divisor.",
    arithmetic::FactorRequest => arithmetic::FactorReport, arithmetic::factor,
    (8192, 1 << 20, 17, 16 << 20, 1 << 20),
    r#"{"method":"trial_division","value":360}"#
    => r#"{"method":"trial_division","factors":[{"prime":2,"multiplicity":3},{"prime":3,"multiplicity":2},{"prime":5,"multiplicity":1}]}"#),
    ),
    declare(
        "number.recurrence",
        profile!("recurrence", "recurrence", "Generate checked progressions, Fibonacci, Lucas or bounded Collatz prefixes.",
    arithmetic::RecurrenceRequest => arithmetic::RecurrenceReport, arithmetic::recurrence,
    (4096, 1 << 20, 4096, 16 << 20, 32 << 20),
    r#"{"sequence":"fibonacci","count":7}"#
    => r#"{"kind":"unsigned","values":[0,1,1,2,3,5,8]}"#),
    ),
    declare(
        "number.lehmer",
        profile!("lehmer", "lehmer", "Rank and unrank distinct permutations with exact Lehmer digits.",
    combinatorial::LehmerRequest => combinatorial::LehmerReport, combinatorial::lehmer,
    (8 << 10, 16 << 10, 34, 65536, 64 << 10),
    r#"{"action":"rank","permutation":[2,0,1]}"#
    => r#"{"rank":"4","digits":[2,0,0],"permutation":[2,0,1]}"#),
    ),
    declare(
        "number.factoradic",
        profile!("factoradic", "factoradic", "Convert exact unsigned integers and canonical factoradic digits.",
    combinatorial::FactoradicRequest => combinatorial::FactoradicReport, combinatorial::factoradic,
    (8 << 10, 16 << 10, 35, 65536, 64 << 10),
    r#"{"direction":"encode","value":"463"}"#
    => r#"{"value":"463","digits":[3,4,1,0,1,0]}"#),
    ),
    declare(
        "number.lexicographic_rank",
        profile!("lexicographic_rank", "lexicographic-rank", "Rank or unrank a multiset of scalars in an explicit alphabet order.",
    combinatorial::LexRequest => combinatorial::LexReport, combinatorial::lexicographic_rank,
    (16 << 10, 16 << 10, 34, 65536, 64 << 10),
    r#"{"action":"rank","alphabet":"BA","multiset":"ABA","text":"ABA"}"#
    => r#"{"rank":"1","total":"3","text":"ABA"}"#),
    ),
    declare(
        "number.partitions",
        profile!("partitions", "partitions", "Count and enumerate bounded integer partitions in reverse lexicographic order.",
    combinatorial::PartitionRequest => combinatorial::PartitionReport, combinatorial::partitions,
    (1 << 20, 8 << 20, 10000, 1 << 20, 16 << 20),
    r#"{"action":"enumerate","n":4,"part_count":2}"#
    => r#"{"count":"2","partitions":[[3,1],[2,2]],"complete":true,"stop_reason":"complete"}"#),
    ),
    declare(
        "number.de_bruijn",
        profile!("de_bruijn", "de-bruijn", "Generate a canonical FKM de Bruijn cycle over an explicit scalar alphabet.",
    combinatorial::BruijnRequest => combinatorial::BruijnReport, combinatorial::de_bruijn,
    (8192, 8 << 20, 1 << 20, 16 << 20, 32 << 20),
    r#"{"alphabet":"01","order":2,"linearized":true}"#
    => r#"{"sequence":"00110","cyclic_symbols":4,"linearized":true}"#),
    ),
    declare(
        "number.permutation_list",
        profile!("permutation_list", "permutation-list", "Enumerate distinct multiset or adjacent-swap permutations with bounded prefixes.",
    combinatorial::PermRequest => combinatorial::PermReport, combinatorial::permutation_list,
    (1 << 20, 8 << 20, 10000, 1 << 20, 32 << 20),
    r#"{"mode":"multiset","tokens":[2,1,2]}"#
    => r#"{"mode":"multiset","count":"3","permutations":[[1,2,2],[2,1,2],[2,2,1]],"sources":[{"token":1,"source_indices":[1]},{"token":2,"source_indices":[0,2]}],"complete":true,"stop_reason":"complete"}"#),
    ),
    declare(
        "number.matrix",
        profile!("matrix", "matrix", "Compute exact rational matrix operations and deterministic elimination.",
    linear::MatrixRequest => linear::MatrixReport, linear::matrix,
    (1 << 20, 1 << 20, 1024, 4 << 20, 16 << 20),
    r#"{"action":"determinant","matrix":{"rows":2,"cols":2,"cells":[1,2,3,4]}}"#
    => r#"{"action":"determinant","value":{"numerator":"-2","denominator":"1"}}"#),
    ),
    declare(
        "number.interpolation",
        profile!("interpolation", "interpolation", "Return exact forward differences or evaluate a Lagrange or Newton interpolant.",
    linear::PolynomialRequest => linear::PolynomialReport, linear::interpolation,
    (1 << 20, 1 << 20, 256, 4 << 20, 16 << 20),
    r#"{"action":"interpolate","method":"newton","samples":[{"x":0,"y":1},{"x":2,"y":5}],"points":[3],"coefficients":true}"#
    => r#"{"action":"interpolate","method":"newton","values":[{"numerator":"7","denominator":"1"}],"coefficients":[{"numerator":"1","denominator":"1"},{"numerator":"2","denominator":"1"}]}"#),
    ),
    declare(
        "number.entropy",
        profile!("entropy", "entropy", "Estimate entropy from counts with Shannon, Rényi or Miller–Madow.",
    statistics::EntropyRequest => statistics::EntropyReport, statistics::entropy,
    (1 << 20, 1 << 20, 10000, 1 << 20, 16 << 20),
    r#"{"method":"shannon","counts":[2,0,2]}"#
    => r#"{"method":"shannon","alpha":null,"samples":"4","nonzero_bins":2,"bits":1.0}"#),
    ),
    declare(
        "number.correlation",
        profile!("correlation", "correlation", "Compute ordered lag overlaps or explicit Pearson and Spearman correlations.",
    statistics::CorrelationRequest => statistics::CorrelationReport, statistics::correlation,
    (8 << 20, 8 << 20, 65536, 16 << 20, 32 << 20),
    r#"{"method":"lag","x":[1,2,4],"lags":[-1,0,1]}"#
    => r#"{"kind":"lags","lags":[{"lag":-1,"overlap":2,"value":10.0},{"lag":0,"overlap":3,"value":21.0},{"lag":1,"overlap":2,"value":10.0}]}"#),
    ),
    declare(
        "number.runs",
        profile!("runs", "runs", "Count Wald–Wolfowitz runs and return explicitly defined evidence.",
    statistics::RunsRequest => statistics::RunsReport, statistics::runs,
    (12 << 20, 1 << 20, 1 << 20, 4 << 20, 16 << 20),
    r#"{"input":"binary","values":[0,0,0]}"#
    => r#"{"runs":1,"n0":3,"n1":0,"dropped":0,"expected":1.0,"variance":0.0,"z":null,"large_sample":false,"undefined_reason":"single_category"}"#),
    ),
    declare(
        "number.benford",
        profile!("benford", "benford", "Measure exact first significant decimal digits and return Benford weights.",
    statistics::BenfordRequest => statistics::BenfordReport, statistics::benford,
    (2 << 20, 1 << 20, 65536, 4 << 20, 16 << 20),
    r#"{"values":["-010","+0200","027","0"]}"#
    => r#"{"counts":[1,2,0,0,0,0,0,0,0],"included":3,"zeros":1,"probabilities":[0.3010299956639812,0.17609125905568124,0.12493873660829995,0.09691001300805642,0.07918124604762483,0.0669467896306132,0.057991946977686754,0.05115252244738129,0.04575749056067512]}"#),
    ),
    declare(
        "number.markov",
        profile!("markov", "markov", "Build ordered empirical transition counts and exact row probabilities.",
    statistics::MarkovRequest => statistics::MarkovReport, statistics::markov,
    (2 << 20, 8 << 20, 65536, 4 << 20, 16 << 20),
    r#"{"states":["a","b"],"sequences":[["a","b"],["a","a"]]}"#
    => r#"{"states":["a","b"],"sequence_length":4,"sequence_count":2,"rows":[{"total":2,"counts":[1,1],"probabilities":[{"numerator":1,"denominator":2},{"numerator":1,"denominator":2}]},{"total":0,"counts":[0,0],"probabilities":null}]}"#),
    ),
    declare(
        "number.letter_values",
        profile!("letter_values", "letter-values", "Apply explicit scalar value tables with checked aggregate and position evidence.",
    letter_values::Request => letter_values::Report, letter_values::letter_values,
    (4 << 20, 8 << 20, 65536, 4 << 20, 16 << 20),
    r#"{"action":"evaluate","mapping":{"preset":"chaldean_cheiro"},"text":"A!Z","unmapped":"skip","aggregate":"positional_weighted_sum","reduce":true}"#
    => r#"{"action":"evaluate","profile":"chaldean_cheiro","table_sha256":"442b7d9d79d5de77f3adb4b149bb60b1f487ff97e66c360311622c98223d0e2c","values":[{"scalar_index":0,"byte_offset":0,"scalar":"A","value":1},{"scalar_index":1,"byte_offset":1,"scalar":"!","value":null},{"scalar_index":2,"byte_offset":2,"scalar":"Z","value":7}],"omitted_count":1,"sum":8,"aggregate":"positional_weighted_sum","value":"22","digital_root":4}"#),
    ),
];

const fn contracts() -> [c::Entry; ROWS.len()] {
    let mut entries = [ROWS[0].contract; ROWS.len()];
    let mut index = 1;
    while index < ROWS.len() {
        entries[index] = ROWS[index].contract;
        index += 1;
    }
    entries
}

const fn descriptors() -> [o::Descriptor; ROWS.len()] {
    let mut entries = [ROWS[0].contract.descriptor; ROWS.len()];
    let mut index = 1;
    while index < ROWS.len() {
        entries[index] = ROWS[index].contract.descriptor;
        index += 1;
    }
    entries
}

pub const OPERATIONS: &[o::Descriptor] = &descriptors();
pub(crate) const MODULE: o::Module =
    o::Module::profiled(OPERATIONS, &[]).with_contracts(&contracts());

pub(super) fn invoke(operation: &str, value: Value) -> Result<Value, OperationError> {
    let row = ROWS
        .iter()
        .find(|row| row.contract.descriptor.id == operation)
        .ok_or_else(|| invalid("unknown numeric operation"))?;
    (row.execute)(value, row.contract.descriptor.id)
}
