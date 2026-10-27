//! Exact bounded selection counts. No arrangements or factorial intermediates.
use crate::{
    errors::{ErrorCode, OperationError},
    operations::{Descriptor, Parameter, ParameterType, ParameterValue},
};
use serde::{Serialize, Serializer};
pub const MAX_N: u64 = 128;
pub const MAX_GROUPS: usize = 128;
pub const MAX_WORK: u64 = 65_536;
pub const MAX_REPORT_BYTES: usize = 4096;
pub const MAX_TRIANGLE_BYTES: usize = 1024 * 1024;
const COUNT_REPORT_BOUND: usize = 128;
fn invalid(op: &'static str, name: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::InvalidParameter, message)
        .operation(op)
        .parameter(name)
}
fn limit(op: &'static str, name: &'static str, message: &str) -> OperationError {
    OperationError::new(ErrorCode::ResourceLimit, message)
        .operation(op)
        .parameter(name)
}
fn decimal<S: Serializer>(v: &u128, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&v.to_string())
}
fn rows_decimal<S: Serializer>(rows: &[Vec<u128>], s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::{SerializeSeq, SerializeTuple};
    struct Row<'a>(&'a [u128]);
    impl Serialize for Row<'_> {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            let mut seq = s.serialize_tuple(self.0.len())?;
            for n in self.0 {
                seq.serialize_element(&n.to_string())?;
            }
            seq.end()
        }
    }
    let mut seq = s.serialize_seq(Some(rows.len()))?;
    for row in rows {
        seq.serialize_element(&Row(row))?;
    }
    seq.end()
}
#[derive(Clone, Copy, Debug, Serialize, schemars::JsonSchema)]
pub struct Limits {
    pub max_work: u64,
    pub max_report_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_work: MAX_WORK,
            max_report_bytes: MAX_REPORT_BYTES,
        }
    }
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Count {
    #[serde(serialize_with = "decimal")]
    #[schemars(with = "String", regex(pattern = r"^(0|[1-9][0-9]*)$"))]
    pub value: u128,
    pub work: u64,
    pub report_bytes_upper_bound: usize,
}
struct Budget {
    op: &'static str,
    max: u64,
    work: u64,
}
impl Budget {
    fn new(op: &'static str, limits: Limits) -> Result<Self, OperationError> {
        if limits.max_work > MAX_WORK || limits.max_report_bytes > MAX_REPORT_BYTES {
            return Err(invalid(op, "limits", "count limits exceed hard ceilings"));
        }
        if limits.max_report_bytes < COUNT_REPORT_BOUND {
            return Err(limit(
                op,
                "max_report_bytes",
                "conservative count report bound exceeds budget",
            ));
        }
        Ok(Self {
            op,
            max: limits.max_work,
            work: 0,
        })
    }
    fn charge(&mut self) -> Result<(), OperationError> {
        if self.work == self.max {
            return Err(limit(
                self.op,
                "max_work",
                "arithmetic work budget exhausted",
            ));
        }
        self.work += 1;
        Ok(())
    }
    fn gcd(&mut self, mut a: u128, mut b: u128) -> Result<u128, OperationError> {
        while b != 0 {
            self.charge()?;
            (a, b) = (b, a % b);
        }
        Ok(a)
    }
    fn multiply(&mut self, a: u128, b: u128) -> Result<u128, OperationError> {
        self.charge()?;
        a.checked_mul(b)
            .ok_or_else(|| limit(self.op, "result", "exact count exceeds u128"))
    }
    fn finish(self, value: u128) -> Count {
        Count {
            value,
            work: self.work,
            report_bytes_upper_bound: COUNT_REPORT_BOUND,
        }
    }
}
fn domain(n: u64, k: u64, op: &'static str) -> Result<(), OperationError> {
    if n > MAX_N || k > n {
        Err(invalid(op, "n_k", "require 0<=k<=n<=128"))
    } else {
        Ok(())
    }
}
fn choose(n: u64, k: u64, budget: &mut Budget) -> Result<u128, OperationError> {
    let k = k.min(n - k);
    let mut value = 1;
    for i in 1..=k {
        // Cancel denominator against numerator and previous exact coefficient
        // before multiplication. The residual denominator is mathematically 1.
        budget.charge()?;
        let mut a = u128::from(n - k + i);
        let mut b = u128::from(i);
        let g = budget.gcd(a, b)?;
        a /= g;
        b /= g;
        let g = budget.gcd(value, b)?;
        value /= g;
        b /= g;
        debug_assert_eq!(b, 1);
        value = budget.multiply(value, a)?;
    }
    Ok(value)
}
/// Ordered k-selections from n distinct items. Empty selection is one.
/// Returns exact u128 or a typed range/work error, never an approximation.
pub fn permutations(n: u64, k: u64, limits: Limits) -> Result<Count, OperationError> {
    const OP: &str = "number.permutations";
    domain(n, k, OP)?;
    let mut b = Budget::new(OP, limits)?;
    let mut v = 1;
    for i in 0..k {
        v = b.multiply(v, u128::from(n - i))?;
    }
    Ok(b.finish(v))
}
/// Exact C(n,k), with cancellation so fitting coefficients never fail due to
/// a factorial or multiplication intermediate. All C(128,k) fit u128.
/// ```
/// use cube_core::numbers::combinatorics::{binomial,Limits};
/// assert_eq!(binomial(5,2,Limits::default())?.value,10);
/// # Ok::<(),cube_core::errors::OperationError>(())
/// ```
pub fn binomial(n: u64, k: u64, limits: Limits) -> Result<Count, OperationError> {
    const OP: &str = "number.binomial";
    domain(n, k, OP)?;
    let mut b = Budget::new(OP, limits)?;
    let value = choose(n, k, &mut b)?;
    Ok(b.finish(value))
}
/// Ordered group sizes, <=128 groups and checked total <=128. Empty/zero groups
/// contribute one. Sequential binomial factors avoid overflowing factorials.
pub fn multinomial(groups: &[u64], limits: Limits) -> Result<Count, OperationError> {
    const OP: &str = "number.multinomial";
    if groups.len() > MAX_GROUPS {
        return Err(invalid(OP, "groups", "at most 128 groups are supported"));
    }
    let mut b = Budget::new(OP, limits)?;
    let mut total = 0u64;
    for &g in groups {
        b.charge()?;
        total = total
            .checked_add(g)
            .filter(|n| *n <= MAX_N)
            .ok_or_else(|| invalid(OP, "groups", "group sizes must sum to at most 128"))?;
    }
    let mut sum = 0;
    let mut value = 1;
    for &g in groups {
        sum += g;
        let factor = choose(sum, g, &mut b)?;
        value = b.multiply(value, factor)?;
    }
    Ok(b.finish(value))
}
#[derive(Debug, Serialize, schemars::JsonSchema)]
pub struct Triangle {
    #[serde(serialize_with = "rows_decimal")]
    #[schemars(with = "Vec<Vec<String>>")]
    pub rows: Vec<Vec<u128>>,
    pub work: u64,
    pub report_bytes_upper_bound: usize,
}
/// Return `rows` rows indexed 0..rows (exclusive), at most 129 (through row 128).
/// Zero rows is empty. Pascal recurrence shares the same exact coefficients;
/// at most 8385 checked additions/seed writes plus 129 row visits, <=4 KiB scratch
/// outside retained output. Compact JSON decimal strings are preflighted <=1 MiB.
pub fn pascal(
    rows: u64,
    max_work: u64,
    max_output_bytes: usize,
) -> Result<Triangle, OperationError> {
    const OP: &str = "number.pascal";
    if rows > 129 || max_work > MAX_WORK || max_output_bytes > MAX_TRIANGLE_BYTES {
        return Err(invalid(
            OP,
            "limits",
            "rows<=129, work<=65536 and output<=1 MiB are required",
        ));
    }
    let count = (rows * (rows + 1) / 2) as usize;
    let bound = 128 + rows as usize * 4 + count * 42;
    if bound > max_output_bytes {
        return Err(limit(
            OP,
            "max_output_bytes",
            "conservative Pascal report bound exceeds budget",
        ));
    }
    let mut budget = Budget {
        op: OP,
        max: max_work,
        work: 0,
    };
    let mut result: Vec<Vec<u128>> = Vec::with_capacity(rows as usize);
    for r in 0..rows as usize {
        budget.charge()?;
        let mut row = Vec::with_capacity(r + 1);
        for k in 0..=r {
            budget.charge()?;
            row.push(if k == 0 || k == r {
                1
            } else {
                result[r - 1][k - 1]
                    .checked_add(result[r - 1][k])
                    .ok_or_else(|| limit(OP, "result", "coefficient exceeds u128"))?
            });
        }
        result.push(row);
    }
    Ok(Triangle {
        rows: result,
        work: budget.work,
        report_bytes_upper_bound: bound,
    })
}
const N: Parameter = Parameter {
    name: "n",
    description: "Unsigned integer, at most 128.",
    required: true,
    default: None,
    constraints: ParameterType::Unsigned { min: 0, max: 128 },
};
const K: Parameter = Parameter {
    name: "k",
    description: "Unsigned selection size, k<=n.",
    required: true,
    default: None,
    constraints: ParameterType::Unsigned { min: 0, max: 128 },
};
const WORK: Parameter = Parameter {
    name: "max_work",
    description: "Charged arithmetic, cancellation and validation steps.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_WORK)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_WORK,
    },
};
const REPORT: Parameter = Parameter {
    name: "max_report_bytes",
    description: "Conservative count report cap, default 4096; bound is 128 bytes.",
    required: false,
    default: Some(ParameterValue::Unsigned(MAX_REPORT_BYTES as u64)),
    constraints: ParameterType::Unsigned {
        min: 0,
        max: MAX_REPORT_BYTES as u64,
    },
};
pub const OPERATIONS: &[Descriptor] = &[
    Descriptor {
        id: "number.permutations",
        category: "numbers",
        description: "Exact P(n,k), n<=128, u128 result; overflow is an error, <=64 Ki arithmetic steps.",
        input_type: "unsigned_pair",
        output_type: "exact_count",
        parameters: &[N, K, WORK, REPORT],
        deterministic: true,
        reversible: None,
        command: "number permutations",
        example: "cube number permutations --n 5 --k 2 --json",
    },
    Descriptor {
        id: "number.binomial",
        category: "numbers",
        description: "Exact C(n,k), n<=128, cancellation before multiplication; no factorial overflow.",
        input_type: "unsigned_pair",
        output_type: "exact_count",
        parameters: &[N, K, WORK, REPORT],
        deterministic: true,
        reversible: None,
        command: "number binomial",
        example: "cube number binomial --n 128 --k 64 --json",
    },
    Descriptor {
        id: "number.multinomial",
        category: "numbers",
        description: "Exact multinomial count, <=128 groups with total<=128, checked u128 result.",
        input_type: "unsigned_list",
        output_type: "exact_count",
        parameters: &[
            Parameter {
                name: "groups",
                description: "Explicit group sizes including zeros, checked total<=128; empty gives one.",
                required: true,
                default: None,
                constraints: ParameterType::UnsignedList {
                    min: 0,
                    max: 128,
                    min_length: 0,
                    max_length: 128,
                    unique: false,
                },
            },
            WORK,
            REPORT,
        ],
        deterministic: true,
        reversible: None,
        command: "number multinomial",
        example: "cube number multinomial --groups 2,1,1 --json",
    },
    Descriptor {
        id: "number.pascal",
        category: "numbers",
        description: "Exact Pascal rows 0..rows-1; <=129 rows including row 128; decimal strings, <=1 MiB report.",
        input_type: "unsigned",
        output_type: "pascal_triangle",
        parameters: &[
            Parameter {
                name: "rows",
                description: "Number of rows; zero is empty, 129 includes index 128.",
                required: true,
                default: None,
                constraints: ParameterType::Unsigned { min: 0, max: 129 },
            },
            WORK,
            Parameter {
                name: "max_output_bytes",
                description: "Conservative compact JSON budget, <=1 MiB.",
                required: false,
                default: Some(ParameterValue::Unsigned(MAX_TRIANGLE_BYTES as u64)),
                constraints: ParameterType::Unsigned {
                    min: 0,
                    max: MAX_TRIANGLE_BYTES as u64,
                },
            },
        ],
        deterministic: true,
        reversible: None,
        command: "number pascal",
        example: "cube number pascal --rows 4",
    },
];
