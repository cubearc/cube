//! SOSEMANUK specification sections 2.1–2.6, full Serpent24 initialization.
use zeroize::Zeroizing;

const SBOXES: [[u8; 16]; 8] = [
    [3, 8, 15, 1, 10, 6, 5, 11, 14, 13, 4, 2, 7, 0, 9, 12],
    [15, 12, 2, 7, 9, 0, 5, 10, 1, 11, 14, 8, 6, 13, 3, 4],
    [8, 6, 7, 9, 3, 12, 10, 15, 13, 1, 14, 4, 0, 11, 5, 2],
    [0, 15, 11, 8, 12, 9, 6, 3, 13, 1, 2, 4, 10, 7, 5, 14],
    [1, 15, 8, 3, 12, 0, 11, 6, 2, 5, 4, 10, 9, 14, 7, 13],
    [15, 5, 2, 11, 4, 10, 9, 12, 0, 3, 14, 8, 13, 6, 7, 1],
    [7, 2, 12, 5, 8, 4, 6, 11, 14, 9, 1, 15, 13, 3, 10, 0],
    [1, 13, 15, 0, 14, 8, 2, 11, 7, 4, 12, 10, 9, 3, 5, 6],
];
fn sbox(x: [u32; 4], index: usize) -> [u32; 4] {
    let mut y = [0; 4];
    for bit in 0..32 {
        let nibble = (0..4).fold(0, |v, i| v | (((x[i] >> bit) & 1) << i));
        let z = SBOXES[index][nibble as usize];
        for (i, w) in y.iter_mut().enumerate() {
            *w |= u32::from((z >> i) & 1) << bit;
        }
    }
    y
}
fn linear(x: &mut [u32; 4]) {
    x[0] = x[0].rotate_left(13);
    x[2] = x[2].rotate_left(3);
    x[1] ^= x[0] ^ x[2];
    x[3] ^= x[2] ^ (x[0] << 3);
    x[1] = x[1].rotate_left(1);
    x[3] = x[3].rotate_left(7);
    x[0] ^= x[1] ^ x[3];
    x[2] ^= x[3] ^ (x[1] << 7);
    x[0] = x[0].rotate_left(5);
    x[2] = x[2].rotate_left(22);
}
const fn gf_mul(mut a: u8, mut b: u8) -> u8 {
    let mut r = 0;
    while b != 0 {
        if b & 1 != 0 {
            r ^= a;
        }
        a = (a << 1) ^ if a & 128 != 0 { 0xa9 } else { 0 };
        b >>= 1;
    }
    r
}
const fn masks(coeff: [u8; 4]) -> [u32; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = u32::from_be_bytes([
            gf_mul(i as u8, coeff[0]),
            gf_mul(i as u8, coeff[1]),
            gf_mul(i as u8, coeff[2]),
            gf_mul(i as u8, coeff[3]),
        ]);
        i += 1;
    }
    table
}
// beta^23, beta^245, beta^48, beta^239 over x^8+x^7+x^5+x^3+1.
const MUL: [u32; 256] = masks([0xe1, 0x9f, 0xcf, 0x13]);
// Inverse alpha derived in the same field: beta^16, beta^39, beta^6, beta^64.
const DIV: [u32; 256] = masks([0x18, 0x0f, 0x40, 0xcd]);
fn schedule(key: &[u8]) -> Zeroizing<[[u32; 4]; 25]> {
    let mut padded = Zeroizing::new([0u8; 32]);
    padded[..key.len()].copy_from_slice(key);
    if key.len() < 32 {
        padded[key.len()] = 1;
    }
    let mut w = Zeroizing::new([0u32; 108]);
    for (i, b) in padded.as_chunks::<4>().0.iter().enumerate() {
        w[i] = u32::from_le_bytes(*b);
    }
    for i in 8..108 {
        w[i] = (w[i - 8] ^ w[i - 5] ^ w[i - 3] ^ w[i - 1] ^ 0x9e3779b9 ^ ((i - 8) as u32))
            .rotate_left(11);
    }
    let mut k = Zeroizing::new([[0; 4]; 25]);
    for i in 0..25 {
        k[i] = sbox(
            w[8 + 4 * i..12 + 4 * i].try_into().unwrap(),
            (3usize.wrapping_sub(i)) & 7,
        );
    }
    k
}
fn initialize(key: &[u8], iv: &[u8]) -> (Zeroizing<[u32; 10]>, Zeroizing<[u32; 2]>) {
    let k = schedule(key);
    let mut x = Zeroizing::new([0; 4]);
    for (i, b) in iv.as_chunks::<4>().0.iter().enumerate() {
        x[i] = u32::from_le_bytes(*b);
    }
    let mut s = Zeroizing::new([0; 10]);
    let mut r = Zeroizing::new([0; 2]);
    for round in 0..24 {
        for i in 0..4 {
            x[i] ^= k[round][i];
        }
        *x = sbox(*x, round & 7);
        linear(&mut x);
        match round {
            11 => s[6..10].copy_from_slice(&[x[3], x[2], x[1], x[0]]),
            17 => {
                s[4] = x[1];
                s[5] = x[3];
                r[0] = x[0];
                r[1] = x[2];
            }
            _ => {}
        }
    }
    for i in 0..4 {
        x[i] ^= k[24][i];
    }
    s[..4].copy_from_slice(&[x[3], x[2], x[1], x[0]]);
    (s, r)
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8]) {
    let (mut s, mut r) = initialize(key, iv);
    for chunk in output.chunks_mut(16) {
        let mut f = Zeroizing::new([0u32; 4]);
        let mut dropped = Zeroizing::new([0u32; 4]);
        for i in 0..4 {
            let next_r1 = r[1].wrapping_add(s[1] ^ if r[0] & 1 != 0 { s[8] } else { 0 });
            r[1] = r[0].wrapping_mul(0x54655307).rotate_left(7);
            r[0] = next_r1;
            f[i] = s[9].wrapping_add(r[0]) ^ r[1];
            dropped[i] = s[0];
            let next = (s[0] << 8)
                ^ MUL[(s[0] >> 24) as usize]
                ^ (s[3] >> 8)
                ^ DIV[(s[3] & 255) as usize]
                ^ s[9];
            s.copy_within(1..10, 0);
            s[9] = next;
        }
        let z = Zeroizing::new(sbox(*f, 2));
        for (i, word) in chunk.chunks_mut(4).enumerate() {
            for (b, k) in word.iter_mut().zip((z[i] ^ dropped[i]).to_le_bytes()) {
                *b ^= k;
            }
        }
    }
}
