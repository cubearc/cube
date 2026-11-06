//! Original Piccolo 80/128, 25/31 rounds; paper BE branch/word convention.
//! Independently derived from Isobe thesis section 7, designer Appendix A KATs.
use zeroize::Zeroizing;
const S: [u8; 16] = [14, 4, 11, 2, 3, 8, 0, 9, 1, 10, 7, 15, 6, 12, 5, 13];
fn gf(mut a: u8, mut b: u8) -> u8 {
    let mut out = 0;
    for _ in 0..4 {
        if b & 1 != 0 {
            out ^= a;
        }
        b >>= 1;
        a <<= 1;
        if a & 16 != 0 {
            a ^= 0x13;
        }
    }
    out
}
fn f(x: u16) -> u16 {
    let n: [u8; 4] = std::array::from_fn(|i| S[((x >> (4 * (3 - i))) & 15) as usize]);
    let matrix = [[2, 3, 1, 1], [1, 2, 3, 1], [1, 1, 2, 3], [3, 1, 1, 2]];
    matrix.iter().fold(0, |out, row| {
        let cell = row.iter().zip(n).fold(0, |a, (m, b)| a ^ gf(*m, b));
        (out << 4) | u16::from(S[cell as usize])
    })
}
fn rp(state: &mut [u16; 4], inverse: bool) {
    let input: [u8; 8] = std::array::from_fn(|i| (state[i / 2] >> (8 * (1 - i % 2))) as u8);
    let mut out = [0u8; 8];
    for (dst, src) in [2, 7, 4, 1, 6, 3, 0, 5].into_iter().enumerate() {
        let (a, b) = if inverse { (src, dst) } else { (dst, src) };
        out[a] = input[b];
    }
    *state = std::array::from_fn(|i| u16::from_be_bytes([out[2 * i], out[2 * i + 1]]));
}
fn schedule(key: &[u8]) -> (Zeroizing<[u16; 4]>, Zeroizing<Vec<u16>>) {
    let mut k = Zeroizing::new(
        key.as_chunks::<2>()
            .0
            .iter()
            .map(|b| u16::from_be_bytes(*b))
            .collect::<Vec<_>>(),
    );
    let short = key.len() == 10;
    let (a, b) = if short { (4, 3) } else { (4, 7) };
    let w = Zeroizing::new([
        (k[0] & 0xff00) | (k[1] & 255),
        (k[1] & 0xff00) | (k[0] & 255),
        (k[a] & 0xff00) | (k[b] & 255),
        (k[b] & 0xff00) | (k[a] & 255),
    ]);
    let nr = if short { 25 } else { 31 };
    let mut rk = Zeroizing::new(vec![0u16; 2 * nr]);
    for r in 0..nr {
        let n = r as u32 + 1;
        let con =
            ((n << 27) | (n << 17) | (n << 10) | n) ^ if short { 0x0f1e2d3c } else { 0x6547a98b };
        for j in 0..2 {
            let i = 2 * r + j;
            let index = if short {
                match r % 5 {
                    0 | 2 => 2 + j,
                    1 | 4 => j,
                    _ => 4,
                }
            } else {
                if (i + 2) % 8 == 0 {
                    let old = k.clone();
                    for (dst, src) in [2, 1, 6, 7, 0, 3, 4, 5].into_iter().enumerate() {
                        k[dst] = old[src];
                    }
                }
                (i + 2) % 8
            };
            rk[i] = k[index] ^ (con >> (16 * (1 - j))) as u16;
        }
    }
    (w, rk)
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let (w, rk) = schedule(key);
    let nr = rk.len() / 2;
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x = Zeroizing::new(std::array::from_fn::<u16, 4, _>(|i| {
            u16::from_be_bytes([block[2 * i], block[2 * i + 1]])
        }));
        if inverse {
            x[0] ^= w[2];
            x[2] ^= w[3];
            for r in (0..nr).rev() {
                if r + 1 < nr {
                    rp(&mut x, true);
                }
                x[1] ^= f(x[0]) ^ rk[2 * r];
                x[3] ^= f(x[2]) ^ rk[2 * r + 1];
            }
            x[0] ^= w[0];
            x[2] ^= w[1];
        } else {
            x[0] ^= w[0];
            x[2] ^= w[1];
            for r in 0..nr {
                x[1] ^= f(x[0]) ^ rk[2 * r];
                x[3] ^= f(x[2]) ^ rk[2 * r + 1];
                if r + 1 < nr {
                    rp(&mut x, false);
                }
            }
            x[0] ^= w[2];
            x[2] ^= w[3];
        }
        for (i, v) in x.iter().enumerate() {
            block[2 * i..2 * i + 2].copy_from_slice(&v.to_be_bytes());
        }
    }
    out
}
