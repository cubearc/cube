//! E2 full twelve-round educational reference, standard external bytes.
//! Public arithmetic reviewed against Apache-2.0 go-cryptobin e938a803;
//! independently qualified against the original-state C schedule. No C runtime.
use zeroize::Zeroizing;
const S: [u8; 256] = [
    0xe1, 0x42, 0x3e, 0x81, 0x4e, 0x17, 0x9e, 0xfd, 0xb4, 0x3f, 0x2c, 0xda, 0x31, 0x1e, 0xe0, 0x41,
    0xcc, 0xf3, 0x82, 0x7d, 0x7c, 0x12, 0x8e, 0xbb, 0xe4, 0x58, 0x15, 0xd5, 0x6f, 0xe9, 0x4c, 0x4b,
    0x35, 0x7b, 0x5a, 0x9a, 0x90, 0x45, 0xbc, 0xf8, 0x79, 0xd6, 0x1b, 0x88, 0x02, 0xab, 0xcf, 0x64,
    0x09, 0x0c, 0xf0, 0x01, 0xa4, 0xb0, 0xf6, 0x93, 0x43, 0x63, 0x86, 0xdc, 0x11, 0xa5, 0x83, 0x8b,
    0xc9, 0xd0, 0x19, 0x95, 0x6a, 0xa1, 0x5c, 0x24, 0x6e, 0x50, 0x21, 0x80, 0x2f, 0xe7, 0x53, 0x0f,
    0x91, 0x22, 0x04, 0xed, 0xa6, 0x48, 0x49, 0x67, 0xec, 0xf7, 0xc0, 0x39, 0xce, 0xf2, 0x2d, 0xbe,
    0x5d, 0x1c, 0xe3, 0x87, 0x07, 0x0d, 0x7a, 0xf4, 0xfb, 0x32, 0xf5, 0x8c, 0xdb, 0x8f, 0x25, 0x96,
    0xa8, 0xea, 0xcd, 0x33, 0x65, 0x54, 0x06, 0x8d, 0x89, 0x0a, 0x5e, 0xd9, 0x16, 0x0e, 0x71, 0x6c,
    0x0b, 0xff, 0x60, 0xd2, 0x2e, 0xd3, 0xc8, 0x55, 0xc2, 0x23, 0xb7, 0x74, 0xe2, 0x9b, 0xdf, 0x77,
    0x2b, 0xb9, 0x3c, 0x62, 0x13, 0xe5, 0x94, 0x34, 0xb1, 0x27, 0x84, 0x9f, 0xd7, 0x51, 0x00, 0x61,
    0xad, 0x85, 0x73, 0x03, 0x08, 0x40, 0xef, 0x68, 0xfe, 0x97, 0x1f, 0xde, 0xaf, 0x66, 0xe8, 0xb8,
    0xae, 0xbd, 0xb3, 0xeb, 0xc6, 0x6b, 0x47, 0xa9, 0xd8, 0xa7, 0x72, 0xee, 0x1d, 0x7e, 0xaa, 0xb6,
    0x75, 0xcb, 0xd4, 0x30, 0x69, 0x20, 0x7f, 0x37, 0x5b, 0x9d, 0x78, 0xa3, 0xf1, 0x76, 0xfa, 0x05,
    0x3d, 0x3a, 0x44, 0x57, 0x3b, 0xca, 0xc7, 0x8a, 0x18, 0x46, 0x9c, 0xbf, 0xba, 0x38, 0x56, 0x1a,
    0x92, 0x4d, 0x26, 0x29, 0xa2, 0x98, 0x10, 0x99, 0x70, 0xa0, 0xc5, 0x28, 0xc1, 0x6d, 0x14, 0xac,
    0xf9, 0x5f, 0x4f, 0xc4, 0xc3, 0xd1, 0xfc, 0xdd, 0xb2, 0x59, 0xe6, 0xb5, 0x36, 0x52, 0x4a, 0x2a,
];
fn substitution(x: u32) -> u32 {
    u32::from_le_bytes(x.to_le_bytes().map(|b| S[b as usize]))
}
fn linear(mut x: u32, mut y: u32) -> [u32; 2] {
    y ^= x;
    x ^= y.rotate_right(16);
    y ^= x.rotate_right(8);
    x ^= y;
    [x, y]
}
fn spx(x: u32, y: u32) -> [u32; 2] {
    linear(substitution(x), substitution(y))
}
fn g(key: &mut [u32; 8], v: &mut [u32; 2]) -> Zeroizing<[u32; 8]> {
    let mut result = Zeroizing::new([0u32; 8]);
    for i in 0..4 {
        let pair = spx(key[2 * i], key[2 * i + 1]);
        key[2 * i..2 * i + 2].copy_from_slice(&pair);
        *v = spx(v[0], v[1]);
        for j in 0..2 {
            v[j] ^= pair[j];
            result[2 * i + j] = v[j];
        }
    }
    result
}
fn inverse_odd(x: u32) -> u32 {
    let mut y = x;
    for _ in 0..5 {
        y = y.wrapping_mul(2u32.wrapping_sub(x.wrapping_mul(y)));
    }
    y
}
fn schedule(key: &[u8]) -> Zeroizing<[u32; 72]> {
    let mut state = Zeroizing::new([0u32; 8]);
    for (i, word) in key.as_chunks::<4>().0.iter().enumerate() {
        state[i] = u32::from_be_bytes(*word);
    }
    if key.len() == 16 {
        state[4] = 0x30d32e58u32.swap_bytes();
        state[5] = 0xb89e4984u32.swap_bytes();
    }
    if key.len() < 32 {
        state[6] = 0x0957cfecu32.swap_bytes();
        state[7] = 0xd800502eu32.swap_bytes();
    }
    let mut v = Zeroizing::new([0x67452301u32.swap_bytes(), 0xefcdab89u32.swap_bytes()]);
    let _ = g(&mut state, &mut v);
    let mut bytes = Zeroizing::new([0u8; 288]);
    for i in 0..8 {
        let result = g(&mut state, &mut v);
        let mut block = Zeroizing::new([0u8; 32]);
        for (word, out) in result.iter().zip(block.as_chunks_mut::<4>().0) {
            out.copy_from_slice(&word.to_le_bytes());
        }
        for j in 0..4 {
            let k = 2 * (48 - 16 * j as isize + 2 * (i / 2) as isize - (i % 2) as isize);
            for (offset, index) in [
                (3, j),
                (2, j + 16),
                (19, j + 8),
                (18, j + 24),
                (131, j + 4),
                (130, j + 20),
                (147, j + 12),
                (146, j + 28),
            ] {
                bytes[(k + offset) as usize] = block[index];
            }
        }
    }
    let mut schedule: Zeroizing<[u32; 72]> = Zeroizing::new(std::array::from_fn(|i| {
        u32::from_le_bytes(bytes[4 * i..4 * i + 4].try_into().unwrap())
    }));
    for i in 52..60 {
        schedule[i] |= 1;
        schedule[i + 12] = inverse_odd(schedule[i]);
    }
    for i in (0..48).step_by(4) {
        let change = (schedule[i] ^ schedule[i + 1]) & 0x00ff00ff;
        schedule[i] ^= change;
        schedule[i + 1] ^= change;
    }
    schedule
}
fn byte_swap_halves(mut x: [u32; 4], left: u32, right: u32) -> [u32; 4] {
    let a = (x[0] ^ x[2]) & left;
    let b = (x[1] ^ x[3]) & right;
    x[0] ^= a;
    x[2] ^= a;
    x[1] ^= b;
    x[3] ^= b;
    x
}
fn round(x: &mut [u32; 4], r: usize, k: &[u32]) {
    let base = if r.is_multiple_of(2) { 0 } else { 2 };
    let other = 2 - base;
    let a = (x[other] ^ k[0]).to_le_bytes().map(|b| S[b as usize]);
    let b = (x[other + 1] ^ k[1]).to_le_bytes().map(|b| S[b as usize]);
    let pair = linear(
        u32::from_le_bytes([b[0], a[1], b[2], a[3]]),
        u32::from_le_bytes([a[0], b[1], a[2], b[3]]),
    );
    let a = (pair[0] ^ k[2]).to_le_bytes().map(|b| S[b as usize]);
    let b = (pair[1] ^ k[3]).to_le_bytes().map(|b| S[b as usize]);
    x[base] ^= u32::from_le_bytes([a[3], a[0], b[1], a[2]]);
    x[base + 1] ^= u32::from_le_bytes([b[3], b[0], a[1], b[2]]);
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let k = schedule(key);
    let mut result = data.to_vec();
    for block in result.as_chunks_mut::<16>().0 {
        let mut x: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
            u32::from_be_bytes(block[4 * i..4 * i + 4].try_into().unwrap())
        }));
        let (before, mul, last, post) = if inverse {
            (60, 56, 64, 48)
        } else {
            (48, 52, 68, 60)
        };
        for i in 0..4 {
            x[i] = (x[i] ^ k[before + i]).wrapping_mul(k[mul + i]);
        }
        *x = byte_swap_halves(*x, 0x00ffff00, 0x0000ffff);
        for r in 0..12 {
            let index = if inverse { 11 - r } else { r };
            round(&mut x, r, &k[4 * index..4 * index + 4]);
        }
        *x = byte_swap_halves(*x, 0xff0000ff, 0xffff0000);
        for i in 0..4 {
            x[i] = x[i].wrapping_mul(k[last + i]) ^ k[post + i];
            block[4 * i..4 * i + 4].copy_from_slice(&x[i].to_be_bytes());
        }
    }
    result
}
