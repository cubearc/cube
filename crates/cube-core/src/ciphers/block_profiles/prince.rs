//! Original full-round PRINCE; BE input and key halves.
//! Specification math checked against BSD-licensed secworks/prince f40631d.
//! Low-level educational reference, not constant-time or authenticated.
use zeroize::Zeroizing;
const S: [u8; 16] = [11, 15, 3, 2, 10, 12, 9, 1, 6, 7, 8, 0, 14, 5, 13, 4];
const SI: [u8; 16] = [11, 7, 3, 2, 15, 13, 8, 9, 10, 6, 4, 0, 5, 14, 12, 1];
const RC: [u64; 12] = [
    0,
    0x13198a2e03707344,
    0xa4093822299f31d0,
    0x082efa98ec4e6c89,
    0x452821e638d01377,
    0xbe5466cf34e90c6c,
    0x7ef84f78fd955cb1,
    0x85840851f1ac43aa,
    0xc882d32f25323c54,
    0x64a51195e0e3610d,
    0xd3b5a399ca0c2399,
    0xc0ac29b7c97c50dd,
];
const M: [[u8; 3]; 64] = [
    [8, 4, 0],
    [13, 9, 5],
    [14, 10, 2],
    [15, 7, 3],
    [12, 4, 0],
    [9, 5, 1],
    [14, 10, 6],
    [15, 11, 3],
    [12, 8, 0],
    [13, 5, 1],
    [10, 6, 2],
    [15, 11, 7],
    [12, 8, 4],
    [13, 9, 1],
    [14, 6, 2],
    [11, 7, 3],
    [28, 24, 20],
    [29, 25, 17],
    [30, 22, 18],
    [27, 23, 19],
    [24, 20, 16],
    [29, 25, 21],
    [30, 26, 18],
    [31, 23, 19],
    [28, 20, 16],
    [25, 21, 17],
    [30, 26, 22],
    [31, 27, 19],
    [28, 24, 16],
    [29, 21, 17],
    [26, 22, 18],
    [31, 27, 23],
    [44, 40, 36],
    [45, 41, 33],
    [46, 38, 34],
    [43, 39, 35],
    [40, 36, 32],
    [45, 41, 37],
    [46, 42, 34],
    [47, 39, 35],
    [44, 36, 32],
    [41, 37, 33],
    [46, 42, 38],
    [47, 43, 35],
    [44, 40, 32],
    [45, 37, 33],
    [42, 38, 34],
    [47, 43, 39],
    [56, 52, 48],
    [61, 57, 53],
    [62, 58, 50],
    [63, 55, 51],
    [60, 52, 48],
    [57, 53, 49],
    [62, 58, 54],
    [63, 59, 51],
    [60, 56, 48],
    [61, 53, 49],
    [58, 54, 50],
    [63, 59, 55],
    [60, 56, 52],
    [61, 57, 49],
    [62, 54, 50],
    [59, 55, 51],
];
fn substitute(x: u64, inverse: bool) -> u64 {
    let table = if inverse { &SI } else { &S };
    (0..16).fold(0, |out, i| {
        out | (u64::from(table[((x >> (4 * i)) & 15) as usize]) << (4 * i))
    })
}
fn prime(x: u64) -> u64 {
    M.iter().enumerate().fold(0, |out, (i, row)| {
        out | ((((x >> row[0]) ^ (x >> row[1]) ^ (x >> row[2])) & 1) << i)
    })
}
fn shift(x: u64, inverse: bool) -> u64 {
    let map = [0, 5, 10, 15, 4, 9, 14, 3, 8, 13, 2, 7, 12, 1, 6, 11];
    let mut result = 0;
    for (dst, src) in map.into_iter().enumerate() {
        let (a, b) = if inverse { (src, dst) } else { (dst, src) };
        result |= ((x >> (4 * (15 - b))) & 15) << (4 * (15 - a));
    }
    result
}
fn core(mut x: u64, key: u64) -> u64 {
    x ^= key;
    for constant in RC.iter().take(6).skip(1) {
        x = shift(prime(substitute(x, false)), false) ^ constant ^ key;
    }
    x = substitute(prime(substitute(x, false)), true);
    for constant in RC.iter().take(11).skip(6) {
        x = substitute(prime(shift(x ^ constant ^ key, true)), true);
    }
    x ^ key ^ RC[11]
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let mut k = Zeroizing::new([
        u64::from_be_bytes(key[..8].try_into().unwrap()),
        u64::from_be_bytes(key[8..].try_into().unwrap()),
        0,
    ]);
    k[2] = k[0].rotate_right(1) ^ (k[0] >> 63);
    if inverse {
        k.swap(0, 2);
        k[1] ^= RC[11];
    }
    let mut result = data.to_vec();
    for block in result.as_chunks_mut::<8>().0 {
        let state = u64::from_be_bytes(*block) ^ k[0];
        *block = (core(state, k[1]) ^ k[2]).to_be_bytes();
    }
    result
}
