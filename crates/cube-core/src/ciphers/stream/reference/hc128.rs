//! HC-128, Wu's specification sections 2.1–2.3; little-endian words.
use zeroize::Zeroizing;

fn f1(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}
fn f2(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}
fn step(p: &mut [u32; 512], q: &[u32; 512], j: usize, left: bool) -> u32 {
    let [x, y, z] = [3, 10, 511].map(|d| p[(j + 512 - d) & 511]);
    let g = if left {
        (x.rotate_left(10) ^ z.rotate_left(23)).wrapping_add(y.rotate_left(8))
    } else {
        (x.rotate_right(10) ^ z.rotate_right(23)).wrapping_add(y.rotate_right(8))
    };
    p[j] = p[j].wrapping_add(g);
    let x = p[(j + 500) & 511];
    p[j] ^ q[(x & 255) as usize].wrapping_add(q[256 + ((x >> 16) & 255) as usize])
}
pub(super) fn apply(output: &mut [u8], key: &[u8], iv: &[u8]) {
    let mut w = Zeroizing::new([0u32; 1280]);
    for i in 0..8 {
        let n = (i & 3) * 4;
        w[i] = u32::from_le_bytes(key[n..n + 4].try_into().unwrap());
        w[8 + i] = u32::from_le_bytes(iv[n..n + 4].try_into().unwrap());
    }
    for i in 16..1280 {
        w[i] = f2(w[i - 2])
            .wrapping_add(w[i - 7])
            .wrapping_add(f1(w[i - 15]))
            .wrapping_add(w[i - 16])
            .wrapping_add(i as u32);
    }
    let mut p = Zeroizing::new([0u32; 512]);
    let mut q = Zeroizing::new([0u32; 512]);
    p.copy_from_slice(&w[256..768]);
    q.copy_from_slice(&w[768..1280]);
    for j in 0..512 {
        p[j] = step(&mut p, &q, j, false);
    }
    for j in 0..512 {
        q[j] = step(&mut q, &p, j, true);
    }
    for (i, chunk) in output.chunks_mut(4).enumerate() {
        let j = i & 511;
        let word = if i & 1023 < 512 {
            step(&mut p, &q, j, false)
        } else {
            step(&mut q, &p, j, true)
        };
        for (b, k) in chunk.iter_mut().zip(word.to_le_bytes()) {
            *b ^= k;
        }
    }
}
