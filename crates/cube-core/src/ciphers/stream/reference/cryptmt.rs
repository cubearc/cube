//! CryptMT3 parametermagi06.20060628.01, fixed256-bit key and IV.
//! Portable state math, full1248-byte generator blocks; no SIMD/unsafe casts.
use zeroize::Zeroizing;
const N: usize = 156;
const POSITION: usize = 108;
const MASK: [u32; 4] = [0xffdfafdf, 0xf5dabfff, 0xffdbffff, 0xef7bffff];
const BASE: usize = 10;
const BLOCK: usize = N * 8;
fn boot(s: &mut [[u32; 4]], first: usize, second: usize, count: usize, lung: &mut [u32; 4]) {
    for i in 0..count {
        let mut a = Zeroizing::new([0u32; 4]);
        for j in 0..4 {
            a[j] = s[first + i][j].wrapping_add(s[second + i][j]);
        }
        s[first + i] = *a;
        let mixed = Zeroizing::new([
            a[3] ^ (a[0] >> 13),
            a[0] ^ (a[1] >> 13),
            a[1] ^ (a[2] >> 13),
            a[2] ^ (a[3] >> 13),
        ]);
        let next = Zeroizing::new(s[second + i + 1]);
        let b = Zeroizing::new([
            next[3] ^ (next[0] >> 11),
            next[2] ^ (next[1] >> 11),
            next[0] ^ (next[2] >> 11),
            next[1] ^ (next[3] >> 11),
        ]);
        for j in 0..4 {
            lung[j] = b[j]
                .wrapping_mul(2)
                .wrapping_add(1)
                .wrapping_mul(lung[j])
                .wrapping_add(b[j]);
            s[second + i + 2][j] = mixed[j].wrapping_sub(lung[j]);
        }
    }
}
fn recurse(s: &mut [[u32; 4]], position: usize, second: usize, previous: usize) {
    let a = Zeroizing::new(s[position]);
    let b = Zeroizing::new(s[second]);
    let c = Zeroizing::new(s[previous]);
    let low = ((u64::from(b[1]) << 32) | u64::from(b[0])) >> 3;
    let high = ((u64::from(b[3]) << 32) | u64::from(b[2])) >> 3;
    s[position] = [
        a[1] ^ b[1] ^ (low as u32) ^ (c[0] & MASK[0]),
        a[2] ^ b[3] ^ ((low >> 32) as u32) ^ (c[1] & MASK[1]),
        a[3] ^ b[0] ^ (high as u32) ^ (c[2] & MASK[2]),
        a[0] ^ b[2] ^ ((high >> 32) as u32) ^ (c[3] & MASK[3]),
    ];
}
fn filter(row: &[u32; 4], acc: &mut [u32; 4]) -> Zeroizing<[u16; 4]> {
    let old = Zeroizing::new(*acc);
    let mut words = Zeroizing::new([0u16; 4]);
    for j in 0..4 {
        let x = old[j] ^ (old[(j + 1) % 4] >> 1);
        acc[j] = x
            .wrapping_mul(2)
            .wrapping_add(1)
            .wrapping_mul(row[j])
            .wrapping_add(x);
        words[j] = ((acc[j] >> 16) ^ acc[j]) as u16;
    }
    words
}
fn output(chunk: &mut [u8], s: &[[u32; 4]], base: usize, acc: &mut [u32; 4]) {
    for (i, part) in chunk.chunks_mut(16).enumerate() {
        let a = filter(&s[base + 2 * i], acc);
        let b = filter(&s[base + 2 * i + 1], acc);
        let mut stream = Zeroizing::new([0u8; 16]);
        for j in 0..4 {
            stream[4 * j..4 * j + 2].copy_from_slice(&a[j].to_le_bytes());
            stream[4 * j + 2..4 * j + 4].copy_from_slice(&b[j].to_le_bytes());
        }
        for (byte, k) in part.iter_mut().zip(stream.iter()) {
            *byte ^= k;
        }
    }
}
pub(super) fn apply(data: &mut [u8], key: &[u8], iv: &[u8]) {
    let mut s = Zeroizing::new(vec![[0u32; 4]; N + 32]);
    for (i, material) in [iv, key].iter().enumerate() {
        for (j, word) in material.as_chunks::<4>().0.iter().enumerate() {
            s[2 * i + j / 4][j % 4] = u32::from_le_bytes(*word);
        }
    }
    for i in 0..4 {
        s[i + 4] = s[i];
    }
    for (word, pi) in s[7].iter_mut().zip([314159u32, 265358, 979323, 846264]) {
        *word = word.wrapping_add(pi);
    }
    let mut lung = Zeroizing::new(s[0].map(|w| w | 1));
    boot(&mut s, 0, 6, 10, &mut lung);
    let mut acc = Zeroizing::new(s[17]);
    for (block, chunk) in data.chunks_mut(BLOCK).enumerate() {
        if block == 0 {
            boot(&mut s, BASE, BASE + 6, N, &mut lung);
            output(chunk, &s, BASE, &mut acc);
            s[BASE][3] = 0x4d734e48;
            s[BASE + N] = s[BASE];
            recurse(&mut s, BASE + N, BASE + POSITION, BASE + N - 1);
        } else {
            for i in 0..N {
                recurse(
                    &mut s,
                    BASE + 1 + i,
                    BASE + 1 + (i + POSITION) % N,
                    BASE + 1 + (i + N - 1) % N,
                );
            }
            output(chunk, &s, BASE + 1, &mut acc);
        }
    }
}
