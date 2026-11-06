//! Original CRYPTON AES submission, separate from revised version 1.0.
//! Mathematical implementation checked against Brian Gladman's 1999 C.
//! Acknowledgement: Dr Brian Gladman permits direct/derivative use with origin
//! acknowledgement and compliance with algorithm-originator conditions.
use zeroize::Zeroizing;
const P: [[u8; 16]; 3] = [
    [15, 9, 6, 8, 9, 9, 4, 12, 6, 2, 6, 10, 1, 3, 5, 15],
    [10, 15, 4, 7, 5, 2, 14, 6, 9, 3, 12, 8, 13, 1, 11, 0],
    [0, 4, 8, 4, 2, 15, 8, 13, 1, 1, 15, 7, 2, 11, 14, 15],
];
const MASK: [u32; 4] = [0x3fcff3fc, 0xfc3fcff3, 0xf3fc3fcf, 0xcff3fc3f];
const MB: [u32; 4] = [0xcffccffc, 0xf33ff33f, 0xfccffccf, 0x3ff33ff3];
const fn boxes() -> [[u8; 256]; 2] {
    let mut out = [[0u8; 256]; 2];
    let mut i = 0;
    while i < 256 {
        let left = i >> 4;
        let right = i & 15;
        let r = (right as u8) ^ P[1][left ^ P[0][right] as usize];
        let l = (left as u8) ^ P[0][right] ^ P[2][r as usize];
        let y = r | (l << 4);
        out[0][i] = y;
        out[1][y as usize] = i as u8;
        i += 1;
    }
    out
}
const S: [[u8; 256]; 2] = boxes();
fn byte(x: u32, i: usize) -> usize {
    ((x >> (8 * i)) & 255) as usize
}
fn gamma(x: &[u32], alternate: bool) -> [u32; 4] {
    std::array::from_fn(|i| {
        (0..4).fold(0, |n, j| {
            n | (u32::from(S[(i + j + usize::from(alternate)) % 2][byte(x[j], i)]) << (8 * j))
        })
    })
}
fn phi(x: u32, start: usize) -> u32 {
    (0..4).fold(0, |n, i| {
        n ^ (x.rotate_left((8 * i) as u32) & MB[(start + i) % 4])
    })
}
fn schedule(key: &[u8]) -> (Zeroizing<[u32; 52]>, Zeroizing<[u32; 52]>) {
    let mut initial = Zeroizing::new([0u32; 8]);
    for (i, b) in key.as_chunks::<4>().0.iter().enumerate() {
        initial[(i % 2) * 4 + i / 2] = u32::from_le_bytes(*b);
    }
    let kp = [0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f];
    let kq = [0x9b05688c, 0x1f83d9ab, 0x5be0cd19, 0xcbbb9d5d];
    let mut base = Zeroizing::new([0u32; 8]);
    for half in 0..2 {
        let mut mixed = Zeroizing::new([0u32; 4]);
        for i in 0..4 {
            mixed[i] = (0..4).fold(0, |n, j| {
                n ^ (initial[4 * half + j] & MASK[(i + j + half) % 4])
            }) ^ if half == 0 { kp[i] } else { kq[i] };
        }
        base[4 * half..4 * half + 4].copy_from_slice(&gamma(&mixed[..], half == 1));
    }
    let a = base[..4].iter().fold(0, |n, w| n ^ w);
    let b = base[4..].iter().fold(0, |n, w| n ^ w);
    for i in 0..4 {
        base[i] ^= b;
        base[i + 4] ^= a;
    }
    let mut e = Zeroizing::new([0u32; 52]);
    let mut d = Zeroizing::new([0u32; 52]);
    e[..8].copy_from_slice(&base[..]);
    for n in 0usize..11 {
        let rc = 0x01010101u32 << (n / 2);
        let rotations = [[8, 16], [16, 24], [24, 8]][n % 3];
        let alternate = (n ^ (n >> 1)) & 1 != 0;
        for i in 0..4 {
            let rotate = i % 2 == usize::from(alternate);
            e[4 * n + 8 + i] = if rotate {
                e[4 * n + i].rotate_left(rotations[i / 2])
            } else {
                e[4 * n + i] ^ rc
            };
        }
    }
    for r in 0..13 {
        for i in 0..4 {
            let start = if r % 2 == 1 { (4 - i) % 4 } else { 3 - i };
            d[4 * (12 - r) + i] = phi(e[4 * r + i], start);
        }
    }
    for i in 0..4 {
        d[48 + i] = phi(d[48 + i], 3 - i);
        e[48 + i] = phi(e[48 + i], 3 - i);
    }
    (e, d)
}
fn round(x: &[u32; 4], alternate: bool) -> [u32; 4] {
    std::array::from_fn(|i| {
        (0..4).fold(0, |n, j| {
            let index = (i + j + usize::from(alternate)) % 4;
            n ^ ((u32::from(S[index % 2][byte(x[j], i)]) * 0x01010101) & MASK[index])
        })
    })
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let (e, d) = schedule(key);
    let keys = if inverse { d } else { e };
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<16>().0 {
        let mut x = Zeroizing::new(std::array::from_fn::<u32, 4, _>(|i| {
            u32::from_le_bytes(block[4 * i..4 * i + 4].try_into().unwrap()) ^ keys[i]
        }));
        for r in 1..12 {
            *x = round(&x, r % 2 == 0);
            for i in 0..4 {
                x[i] ^= keys[4 * r + i];
            }
        }
        *x = gamma(&x[..], true);
        for i in 0..4 {
            block[4 * i..4 * i + 4].copy_from_slice(&(x[i] ^ keys[48 + i]).to_le_bytes());
        }
    }
    out
}
