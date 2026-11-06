//! BORON original 25-round 80-/128-bit-key profiles, numeric BE bytes.
//! Paper equations and MIT eymay/boron_cipher 3d6cb45 combinational review.
use zeroize::Zeroizing;
const S: [u8; 16] = [14, 4, 11, 1, 7, 9, 12, 10, 13, 2, 0, 15, 8, 5, 3, 6];
fn schedule(key: &[u8]) -> Zeroizing<[u64; 26]> {
    let mut k = Zeroizing::new(key.iter().fold(0u128, |n, b| (n << 8) | u128::from(*b)));
    let short = key.len() == 10;
    let mut out = Zeroizing::new([0u64; 26]);
    for (r, rk) in out.iter_mut().enumerate() {
        *rk = *k as u64;
        if r == 25 {
            break;
        }
        *k = if short {
            ((*k << 13) | (*k >> 67)) & ((1u128 << 80) - 1)
        } else {
            k.rotate_left(13)
        };
        let count = if short { 1 } else { 2 };
        for i in 0..count {
            let value = u128::from(S[((*k >> (4 * i)) & 15) as usize]);
            *k = (*k & !(15u128 << (4 * i))) | (value << (4 * i));
        }
        *k ^= (r as u128) << 59;
    }
    out
}
fn substitute(x: u64, inverse: bool) -> u64 {
    (0..16).fold(0, |out, i| {
        let cell = ((x >> (4 * i)) & 15) as u8;
        let value = if inverse {
            S.iter().position(|n| *n == cell).unwrap() as u8
        } else {
            S[cell as usize]
        };
        out | (u64::from(value) << (4 * i))
    })
}
fn linear(x: u64, inverse: bool) -> u64 {
    let mut words: Zeroizing<[u16; 4]> =
        Zeroizing::new(std::array::from_fn(|i| (x >> (16 * i)) as u16));
    let shifts = [9, 12, 15, 1];
    if inverse {
        let [a, b, c, d] = *words;
        let w0 = a ^ b;
        let w3 = d ^ c;
        *words = [w0, b ^ w3, c ^ w0, w3];
        for (i, w) in words.iter_mut().enumerate() {
            *w = w.rotate_right(shifts[i]);
        }
    } else {
        for (i, w) in words.iter_mut().enumerate() {
            *w = w.rotate_left(shifts[i]);
        }
        let [a, b, c, d] = *words;
        *words = [d ^ b ^ a, d ^ b, c ^ a, d ^ c ^ a];
    }
    words
        .iter()
        .enumerate()
        .fold(0, |n, (i, w)| n | (u64::from(*w) << (16 * i)))
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let keys = schedule(key);
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x = u64::from_be_bytes(*block);
        if inverse {
            x ^= keys[25];
            for r in (0..25).rev() {
                x = substitute(linear(x, true), true) ^ keys[r];
            }
        } else {
            for r in 0..25 {
                x = linear(substitute(x ^ keys[r], false), false);
            }
            x ^= keys[25];
        }
        *block = x.to_be_bytes();
    }
    out
}
