//! Burrows–Wheeler cyclic rotation ordering and byte move-to-front.
use super::{WORK_LIMIT, check_limit, parameter};
use crate::errors::OperationError;

pub const MAX_BYTES: usize = 65_536;

fn starts(counts: &mut [usize]) {
    let mut total = 0;
    for count in counts {
        let next = total + *count;
        *count = total;
        total = next;
    }
}

fn bucket(
    items: impl Iterator<Item = usize>,
    ranks: &[usize],
    count: &mut [usize],
    order: &mut [usize],
) {
    count.fill(0);
    for &rank in ranks {
        count[rank] += 1;
    }
    starts(count);
    for i in items {
        let start = &mut count[ranks[i]];
        order[*start] = i;
        *start += 1;
    }
}

/// Return the last column and the index of rotation zero. Equal rotations
/// follow their original starting offsets.
pub fn bwt(input: &[u8]) -> Result<(Vec<u8>, usize), OperationError> {
    let n = input.len();
    check_limit(n, MAX_BYTES, "BWT input exceeds 65,536 bytes")?;
    if n == 0 {
        return Ok((Vec::new(), 0));
    }
    let mut ranks: Vec<_> = input.iter().copied().map(usize::from).collect();
    let mut order = vec![0; n];
    let mut next = vec![0; n];
    let mut counts = vec![0; n.max(256)];
    bucket(0..n, &ranks, &mut counts, &mut order);
    let mut span = 1;
    let mut sorted = vec![0; n];
    while span < n {
        let shifted = order.iter().map(|&i| (i + n - span) % n);
        bucket(shifted, &ranks, &mut counts, &mut sorted);
        let key = |i| (ranks[i], ranks[(i + span) % n]);
        let mut rank = 0;
        next[sorted[0]] = rank;
        for pair in sorted.windows(2) {
            rank += usize::from(key(pair[0]) != key(pair[1]));
            next[pair[1]] = rank;
        }
        std::mem::swap(&mut ranks, &mut next);
        std::mem::swap(&mut order, &mut sorted);
        if rank == n - 1 {
            break;
        }
        span *= 2;
    }
    bucket(0..n, &ranks, &mut counts, &mut order);
    let primary = order.iter().position(|&i| i == 0).expect("all rotations");
    let last = order.into_iter().map(|i| input[(i + n - 1) % n]).collect();
    Ok((last, primary))
}

/// Recover the original bytes using the last column and its primary index.
pub fn inverse_bwt(last: &[u8], primary: usize) -> Result<Vec<u8>, OperationError> {
    let n = last.len();
    check_limit(n, MAX_BYTES, "BWT input exceeds 65,536 bytes")?;
    if primary >= n.max(1) {
        return Err(parameter("BWT primary index is outside the block").parameter("primary_index"));
    }
    let mut counts = [0; 256];
    for &byte in last {
        counts[byte as usize] += 1;
    }
    starts(&mut counts);
    let mut next = vec![0; n];
    for (index, &byte) in last.iter().enumerate() {
        let start = &mut counts[byte as usize];
        next[*start] = index;
        *start += 1;
    }
    let mut result = Vec::with_capacity(n);
    let mut row = primary;
    for _ in 0..n {
        row = next[row];
        result.push(last[row]);
    }
    Ok(result)
}

/// Apply or invert byte move-to-front. With no alphabet, use byte order.
pub fn mtf(
    input: &[u8],
    permutation: Option<&[u8]>,
    decode: bool,
) -> Result<Vec<u8>, OperationError> {
    check_limit(
        input.len(),
        MAX_BYTES,
        "move-to-front input exceeds 65,536 bytes",
    )?;
    let mut symbols: [u8; 256] = std::array::from_fn(|i| i as u8);
    if let Some(permutation) = permutation {
        let mut seen = [false; 256];
        if permutation.len() != 256
            || permutation
                .iter()
                .any(|&b| std::mem::replace(&mut seen[b as usize], true))
        {
            return Err(
                parameter("the move-to-front alphabet must be a byte permutation")
                    .parameter("alphabet"),
            );
        }
        symbols.copy_from_slice(permutation);
    }
    // An index up to 255 entails at most 256 units across lookup and update.
    check_limit(input.len() * 256, WORK_LIMIT, "move-to-front work limit")?;
    Ok(input
        .iter()
        .map(|&value| {
            let position = if decode {
                value as usize
            } else {
                symbols
                    .iter()
                    .position(|&candidate| candidate == value)
                    .expect("byte permutation")
            };
            let symbol = symbols[position];
            symbols[..=position].rotate_right(1);
            if decode { symbol } else { position as u8 }
        })
        .collect())
}
