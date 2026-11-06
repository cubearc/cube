//! LBlock corrected author specification, all 32 rounds, BE key/block bytes.
//! Mathematical nibble tables; no LGPL C implementation is incorporated.
use zeroize::Zeroizing;
const S: [[u8; 16]; 10] = [
    [14, 9, 15, 0, 13, 4, 10, 11, 1, 2, 8, 3, 7, 6, 12, 5],
    [4, 11, 14, 9, 15, 13, 0, 10, 7, 12, 5, 6, 2, 8, 1, 3],
    [1, 14, 7, 12, 15, 13, 0, 6, 11, 5, 9, 3, 2, 4, 8, 10],
    [7, 6, 8, 11, 0, 15, 3, 14, 9, 10, 12, 13, 5, 2, 4, 1],
    [14, 5, 15, 0, 7, 2, 12, 13, 1, 8, 4, 9, 11, 10, 6, 3],
    [2, 13, 11, 12, 15, 14, 0, 9, 7, 10, 6, 3, 1, 8, 4, 5],
    [11, 9, 4, 14, 0, 15, 10, 13, 6, 12, 5, 7, 3, 8, 1, 2],
    [13, 10, 15, 0, 14, 4, 9, 11, 2, 1, 8, 3, 7, 5, 12, 6],
    [8, 7, 14, 5, 15, 13, 0, 6, 11, 12, 9, 10, 2, 4, 1, 3],
    [11, 5, 15, 0, 7, 2, 9, 13, 4, 8, 1, 12, 14, 10, 3, 6],
];
fn schedule(key: &[u8]) -> Zeroizing<[u32; 32]> {
    let mut state = Zeroizing::new(key.iter().fold(0u128, |n, b| (n << 8) | u128::from(*b)));
    let mask = (1u128 << 80) - 1;
    let mut result = Zeroizing::new([0u32; 32]);
    for (r, out) in result.iter_mut().enumerate() {
        *out = (*state >> 48) as u32;
        *state = ((*state << 29) | (*state >> 51)) & mask;
        let high = S[9][((*state >> 76) & 15) as usize];
        let next = S[8][((*state >> 72) & 15) as usize];
        *state =
            (*state & ((1u128 << 72) - 1)) | (u128::from(high) << 76) | (u128::from(next) << 72);
        *state ^= ((r + 1) as u128) << 46;
    }
    result
}
fn function(word: u32) -> u32 {
    let order = [1, 3, 0, 2, 5, 7, 4, 6];
    order.into_iter().enumerate().fold(0, |out, (dst, src)| {
        out | (u32::from(S[src][((word >> (4 * src)) & 15) as usize]) << (4 * dst))
    })
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let keys = schedule(key);
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x = Zeroizing::new([
            u32::from_be_bytes(block[..4].try_into().unwrap()),
            u32::from_be_bytes(block[4..].try_into().unwrap()),
        ]);
        if inverse {
            x.swap(0, 1);
        }
        for i in 0..32 {
            let a = x[0];
            let b = x[1];
            if inverse {
                *x = [b, (a ^ function(b ^ keys[31 - i])).rotate_right(8)];
            } else {
                *x = [function(a ^ keys[i]) ^ b.rotate_left(8), a];
            }
        }
        if !inverse {
            x.swap(0, 1);
        }
        block[..4].copy_from_slice(&x[0].to_be_bytes());
        block[4..].copy_from_slice(&x[1].to_be_bytes());
    }
    out
}
