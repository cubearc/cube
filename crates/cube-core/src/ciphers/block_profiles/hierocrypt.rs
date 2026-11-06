//! Hierocrypt-L1 NESSIE reference and Hierocrypt-3 September 2001 profiles.
//! Separate widths, 6 / 6,7,8 full rounds, big-endian data and key words.
//! L1 explicitly fixes H4=8af5def7, distinct from the PDF's printed f7def58a.
//! Own literal mathematical implementation, no third-party cipher code port.
use zeroize::Zeroizing;
const S: [u8; 256] = [
    0x07, 0xFC, 0x55, 0x70, 0x98, 0x8E, 0x84, 0x4E, 0xBC, 0x75, 0xCE, 0x18, 0x02, 0xE9, 0x5D, 0x80,
    0x1C, 0x60, 0x78, 0x42, 0x9D, 0x2E, 0xF5, 0xE8, 0xC6, 0x7A, 0x2F, 0xA4, 0xB2, 0x5F, 0x19, 0x87,
    0x0B, 0x9B, 0x9C, 0xD3, 0xC3, 0x77, 0x3D, 0x6F, 0xB9, 0x2D, 0x4D, 0xF7, 0x8C, 0xA7, 0xAC, 0x17,
    0x3C, 0x5A, 0x41, 0xC9, 0x29, 0xED, 0xDE, 0x27, 0x69, 0x30, 0x72, 0xA8, 0x95, 0x3E, 0xF9, 0xD8,
    0x21, 0x8B, 0x44, 0xD7, 0x11, 0x0D, 0x48, 0xFD, 0x6A, 0x01, 0x57, 0xE5, 0xBD, 0x85, 0xEC, 0x1E,
    0x37, 0x9F, 0xB5, 0x9A, 0x7C, 0x09, 0xF1, 0xB1, 0x94, 0x81, 0x82, 0x08, 0xFB, 0xC0, 0x51, 0x0F,
    0x61, 0x7F, 0x1A, 0x56, 0x96, 0x13, 0xC1, 0x67, 0x99, 0x03, 0x5E, 0xB6, 0xCA, 0xFA, 0x9E, 0xDF,
    0xD6, 0x83, 0xCC, 0xA2, 0x12, 0x23, 0xB7, 0x65, 0xD0, 0x39, 0x7D, 0x3B, 0xD5, 0xB0, 0xAF, 0x1F,
    0x06, 0xC8, 0x34, 0xC5, 0x1B, 0x79, 0x4B, 0x66, 0xBF, 0x88, 0x4A, 0xC4, 0xEF, 0x58, 0x3F, 0x0A,
    0x2C, 0x73, 0xD1, 0xF8, 0x6B, 0xE6, 0x20, 0xB8, 0x22, 0x43, 0xB3, 0x33, 0xE7, 0xF0, 0x71, 0x7E,
    0x52, 0x89, 0x47, 0x63, 0x0E, 0x6D, 0xE3, 0xBE, 0x59, 0x64, 0xEE, 0xF6, 0x38, 0x5C, 0xF4, 0x5B,
    0x49, 0xD4, 0xE0, 0xF3, 0xBB, 0x54, 0x26, 0x2B, 0x00, 0x86, 0x90, 0xFF, 0xFE, 0xA6, 0x7B, 0x05,
    0xAD, 0x68, 0xA1, 0x10, 0xEB, 0xC7, 0xE2, 0xF2, 0x46, 0x8A, 0x6C, 0x14, 0x6E, 0xCF, 0x35, 0x45,
    0x50, 0xD2, 0x92, 0x74, 0x93, 0xE1, 0xDA, 0xAE, 0xA9, 0x53, 0xE4, 0x40, 0xCD, 0xBA, 0x97, 0xA3,
    0x91, 0x31, 0x25, 0x76, 0x36, 0x32, 0x28, 0x3A, 0x24, 0x4C, 0xDB, 0xD9, 0x8D, 0xDC, 0x62, 0x2A,
    0xEA, 0x15, 0xDD, 0xC2, 0xA5, 0x0C, 0x04, 0x1D, 0x8F, 0xCB, 0xB4, 0x4F, 0x16, 0xAB, 0xAA, 0xA0,
];
const SMALL: [[u16; 8]; 2] = [
    [174, 223, 231, 93, 213, 234, 253, 171],
    [189, 94, 175, 106, 165, 218, 237, 91],
];
const LARGE: [[u16; 16]; 2] = [
    [
        43743, 56807, 61171, 21934, 64173, 32222, 16111, 58714, 57258, 59357, 62446, 44629, 44538,
        56957, 61246, 23269,
    ],
    [
        24566, 42875, 54069, 48876, 26111, 47735, 23859, 52206, 63071, 31655, 13779, 60606, 65381,
        30650, 13149, 61131,
    ],
];
const H: [u32; 5] = [0x5a827999, 0x6ed9eba1, 0x8f1bbcdc, 0xca62c1d6, 0x8af5def7];
fn permutation(value: u128, bits: usize, inverse: bool) -> u128 {
    let mask = (1u128 << bits) - 1;
    let v: [u128; 4] = std::array::from_fn(|i| (value >> (bits * (3 - i))) & mask);
    let [a, b, c, d] = v;
    let w = if inverse {
        [a ^ b ^ c, a ^ b ^ d, b ^ c, a ^ d]
    } else {
        [a ^ c, b ^ d, b ^ c ^ d, a ^ c ^ d]
    };
    w.into_iter().fold(0, |n, w| (n << bits) | w)
}
fn binary(input: &[u8], rows: &[u16], output: &mut [u8]) {
    for (dst, row) in output.iter_mut().zip(rows) {
        *dst = input.iter().enumerate().fold(0, |out, (j, x)| {
            out ^ if (row >> (input.len() - 1 - j)) & 1 != 0 {
                *x
            } else {
                0
            }
        });
    }
}
fn multiply_word(x: u64, width: usize, inverse: bool) -> u64 {
    let input = Zeroizing::new(x.to_be_bytes());
    let mut output = Zeroizing::new([0u8; 8]);
    for (i, chunk) in input[8 - width..].as_chunks::<4>().0.iter().enumerate() {
        let rows = match (inverse, i) {
            (false, 0) => [10, 13, 14, 5],
            (true, 0) => [5, 10, 13, 11],
            (false, _) => [15, 7, 3, 14],
            (true, _) => [12, 6, 11, 9],
        };
        binary(
            chunk,
            &rows,
            &mut output[8 - width + 4 * i..8 - width + 4 * i + 4],
        );
    }
    u64::from_be_bytes(*output)
}
fn fsigma(x: u64, width: usize) -> u64 {
    let mut bytes = Zeroizing::new(x.to_be_bytes());
    for b in &mut bytes[8 - width..] {
        *b = S[*b as usize];
    }
    permutation(u128::from(u64::from_be_bytes(*bytes)), width * 2, false) as u64
}
fn pair_permutation(a: u64, b: u64, width: usize, inverse: bool) -> [u64; 2] {
    let bits = width * 8;
    let n = permutation((u128::from(a) << bits) | u128::from(b), bits / 2, inverse);
    let mask = (1u128 << bits) - 1;
    [(n >> bits) as u64, (n & mask) as u64]
}
fn schedule(key: &[u8], small: bool) -> Zeroizing<Vec<[u64; 4]>> {
    let width = if small { 4 } else { 8 };
    let nr = if small { 6 } else { key.len() / 8 + 4 };
    let turn = if small || key.len() < 32 { 4 } else { 5 };
    let mut z = Zeroizing::new([0u64; 4]);
    for (i, b) in key.chunks_exact(width).enumerate() {
        z[i] = b.iter().fold(0, |n, b| (n << 8) | u64::from(*b));
    }
    if !small {
        match key.len() {
            16 => {
                z[2] = z[0];
                z[3] = (u64::from(H[3]) << 32) | u64::from(H[2]);
            }
            24 => {
                z[3] = (u64::from(H[2]) << 32) | u64::from(H[3]);
            }
            _ => {}
        }
    }
    let g0 = if small {
        u64::from(H[0])
    } else {
        (u64::from(H[1]) << 32) | u64::from(H[0])
    };
    let old = z.clone();
    z[2] = multiply_word(old[2], width, false) ^ g0;
    z[3] = multiply_word(old[3], width, small);
    z[0] = old[1];
    z[1] = old[0] ^ fsigma(old[1] ^ z[2], width);
    let ids: &[usize] = match key.len() {
        16 => &[0, 1, 2, 3, 3, 2, 1],
        24 => &[1, 0, 3, 2, 2, 3, 0, 1],
        _ => &[4, 0, 2, 1, 3, 3, 1, 2, 0],
    };
    let gpairs = [[3, 0], [2, 1], [1, 3], [0, 2], [2, 3]];
    let mut result = Zeroizing::new(Vec::with_capacity(nr + 1));
    for t in 1..=nr + 1 {
        let g = if small {
            u64::from(H[if t <= 4 { t } else { 9 - t }])
        } else {
            let pair = gpairs[ids[t - 1]];
            (u64::from(H[pair[0]]) << 32) | u64::from(H[pair[1]])
        };
        let old = z.clone();
        if t <= turn {
            let w = Zeroizing::new(pair_permutation(old[2], old[3], width, false));
            z[2] = multiply_word(w[0], width, false) ^ g;
            z[3] = multiply_word(w[1], width, small);
            let v = fsigma(old[1] ^ z[2], width);
            result.push([old[0] ^ v, z[2] ^ v, z[3] ^ v, old[1] ^ z[3]]);
            z[0] = old[1];
            z[1] = old[0] ^ v;
        } else {
            let v = fsigma(old[0] ^ old[2], width);
            z[0] = old[1] ^ v;
            z[1] = old[0];
            let w = Zeroizing::new([
                multiply_word(old[2] ^ g, width, true),
                multiply_word(old[3], width, !small),
            ]);
            let back = Zeroizing::new(pair_permutation(w[0], w[1], width, true));
            z[2] = back[0];
            z[3] = back[1];
            result.push([z[0] ^ old[2], w[0] ^ v, w[1] ^ v, old[0] ^ w[1]]);
        }
    }
    result
}
fn gf(mut a: u8, mut b: u8) -> u8 {
    let mut out = 0;
    for _ in 0..8 {
        if b & 1 != 0 {
            out ^= a;
        }
        b >>= 1;
        let high = a & 128;
        a <<= 1;
        if high != 0 {
            a ^= 0x63;
        }
    }
    out
}
fn lower(x: &mut [u8], inverse: bool) {
    let matrix = if inverse {
        [0x82, 0xc4, 0x34, 0xf6]
    } else {
        [0xc4, 0x65, 0xc8, 0x8b]
    };
    for chunk in x.as_chunks_mut::<4>().0 {
        let old = Zeroizing::new(*chunk);
        for i in 0..4 {
            chunk[i] = (0..4).fold(0, |v, j| v ^ gf(old[j], matrix[(j + 4 - i) % 4]));
        }
    }
}
fn higher(x: &mut [u8], small: bool, inverse: bool) {
    let old = Zeroizing::new(x.to_vec());
    if small {
        binary(&old, &SMALL[usize::from(inverse)], x);
    } else {
        binary(&old, &LARGE[usize::from(inverse)], x);
    }
}
fn xor_half(x: &mut [u8], keys: &[u64], width: usize) {
    for (chunk, key) in x.chunks_exact_mut(width).zip(keys) {
        for (b, k) in chunk.iter_mut().zip(&key.to_be_bytes()[8 - width..]) {
            *b ^= k;
        }
    }
}
fn sub(x: &mut [u8], inverse: bool) {
    for v in x {
        *v = if inverse {
            S.iter().position(|b| b == v).unwrap() as u8
        } else {
            S[*v as usize]
        };
    }
}
pub(super) fn transform(data: &[u8], key: &[u8], small: bool, inverse: bool) -> Vec<u8> {
    let width = if small { 4 } else { 8 };
    let keys = schedule(key, small);
    let nr = keys.len() - 1;
    let mut output = data.to_vec();
    for block in output.chunks_exact_mut(2 * width) {
        if inverse {
            xor_half(block, &keys[nr][..2], width);
            for r in (0..nr).rev() {
                if r + 1 < nr {
                    higher(block, small, true);
                }
                sub(block, true);
                xor_half(block, &keys[r][2..], width);
                lower(block, true);
                sub(block, true);
                xor_half(block, &keys[r][..2], width);
            }
        } else {
            for (r, round) in keys.iter().take(nr).enumerate() {
                xor_half(block, &round[..2], width);
                sub(block, false);
                lower(block, false);
                xor_half(block, &round[2..], width);
                sub(block, false);
                if r + 1 < nr {
                    higher(block, small, false);
                }
            }
            xor_half(block, &keys[nr][..2], width);
        }
    }
    output
}
