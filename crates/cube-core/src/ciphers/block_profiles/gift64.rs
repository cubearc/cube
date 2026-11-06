//! GIFT-64-128, original full 28 rounds and high-first external bytes.
//! Reviewed against the designers' MIT reference f43c6b3; three designer KATs.
use zeroize::Zeroizing;
const S: [u8; 16] = [1, 10, 4, 12, 6, 15, 3, 9, 2, 13, 11, 7, 5, 0, 8, 14];
const SI: [u8; 16] = [13, 0, 8, 6, 2, 12, 4, 11, 14, 7, 1, 10, 3, 9, 15, 5];
const P: [usize; 64] = [
    0, 17, 34, 51, 48, 1, 18, 35, 32, 49, 2, 19, 16, 33, 50, 3, 4, 21, 38, 55, 52, 5, 22, 39, 36,
    53, 6, 23, 20, 37, 54, 7, 8, 25, 42, 59, 56, 9, 26, 43, 40, 57, 10, 27, 24, 41, 58, 11, 12, 29,
    46, 63, 60, 13, 30, 47, 44, 61, 14, 31, 28, 45, 62, 15,
];
fn schedule(key: &[u8]) -> Zeroizing<[u64; 28]> {
    let mut words: Zeroizing<[u16; 8]> = Zeroizing::new(std::array::from_fn(|i| {
        u16::from_be_bytes(key[14 - 2 * i..16 - 2 * i].try_into().unwrap())
    }));
    let mut result = Zeroizing::new([0u64; 28]);
    let mut c = 1u64;
    for out in result.iter_mut() {
        for i in 0..16 {
            *out |= u64::from((words[0] >> i) & 1) << (4 * i);
            *out |= u64::from((words[1] >> i) & 1) << (4 * i + 1);
        }
        for i in 0..6 {
            *out ^= ((c >> i) & 1) << (4 * i + 3);
        }
        *out ^= 1 << 63;
        c = ((c << 1) & 63) | (((c >> 5) ^ (c >> 4) ^ 1) & 1);
        words.rotate_left(2);
        words[6] = words[6].rotate_right(12);
        words[7] = words[7].rotate_right(2);
    }
    result
}
fn substitute(x: u64, inverse: bool) -> u64 {
    let table = if inverse { &SI } else { &S };
    (0..16).fold(0, |out, i| {
        out | (u64::from(table[((x >> (4 * i)) & 15) as usize]) << (4 * i))
    })
}
fn permutation(x: u64, inverse: bool) -> u64 {
    P.iter().enumerate().fold(0, |out, (i, p)| {
        let (from, to) = if inverse { (*p, i) } else { (i, *p) };
        out | (((x >> from) & 1) << to)
    })
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let keys = schedule(key);
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x = u64::from_be_bytes(*block);
        for r in 0..28 {
            if inverse {
                x = substitute(permutation(x ^ keys[27 - r], true), true);
            } else {
                x = permutation(substitute(x, false), false) ^ keys[r];
            }
        }
        *block = x.to_be_bytes();
    }
    out
}
