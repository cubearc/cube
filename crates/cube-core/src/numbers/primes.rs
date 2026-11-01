//! Local exact sieve for concrete indexed-alphabet workloads, not a prime API.
use crate::errors::{ErrorCode, OperationError};
pub(crate) const MAX_BOUND: u32 = 1_000_000;
pub(crate) const MAX_WORK: u32 = 33_554_432;
fn invalid(op: &'static str) -> OperationError {
    OperationError::new(
        ErrorCode::InvalidParameter,
        "prime index/count must be1..4096 and inclusive bound2..1000000",
    )
    .operation(op)
}
fn charge(work: &mut u32, amount: u32, op: &'static str) -> Result<(), OperationError> {
    *work = work
        .checked_add(amount)
        .filter(|n| *n <= MAX_WORK)
        .ok_or_else(|| {
            OperationError::new(
                ErrorCode::ResourceLimit,
                "bounded prime preprocessing work exhausted",
            )
            .operation(op)
        })?;
    Ok(())
}
/// Return precisely count consecutive primes starting at one-based first.
/// Mark from p², then scan ascending; reject an insufficient bound atomically.
pub(crate) fn indexed(
    first: usize,
    count: usize,
    limit: u32,
    op: &'static str,
) -> Result<Vec<u32>, OperationError> {
    if !(1..=4096).contains(&first)
        || !(1..=4096).contains(&count)
        || !(2..=MAX_BOUND).contains(&limit)
    {
        return Err(invalid(op));
    }
    let len = usize::try_from(limit)
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or_else(|| invalid(op))?;
    let mut work = 0;
    charge(&mut work, len as u32, op)?;
    let mut marked = Vec::new();
    marked.try_reserve_exact(len).map_err(|_| {
        OperationError::new(ErrorCode::ResourceLimit, "bounded sieve allocation refused")
            .operation(op)
    })?;
    marked.resize(len, 0u8);
    let n = limit as usize;
    let mut p = 2usize;
    while p <= n / p {
        charge(&mut work, 1, op)?;
        if marked[p] == 0 {
            for multiple in (p * p..=n).step_by(p) {
                charge(&mut work, 1, op)?;
                marked[multiple] = 1;
            }
        }
        p += 1;
    }
    let mut result = Vec::with_capacity(count);
    let mut seen = 0;
    for (value, &flag) in marked.iter().enumerate().skip(2) {
        charge(&mut work, 1, op)?;
        if flag == 0 {
            seen += 1;
            if seen >= first {
                result.push(value as u32);
            }
            if result.len() == count {
                return Ok(result);
            }
        }
    }
    Err(OperationError::new(
        ErrorCode::InvalidParameter,
        "inclusive prime bound does not contain the complete requested alphabet",
    )
    .operation(op)
    .parameter("max_prime"))
}
