//! Midori64 original sixteen-round profile, high-first nibble representation.
//! Constants and three independent answers: BSD ImpeccableCircuits 33619f7.
use zeroize::Zeroizing;
const S: [u8; 16] = [12, 10, 13, 3, 14, 11, 15, 7, 8, 9, 1, 5, 0, 2, 4, 6];
const RC: [u16; 16] = [
    0, 5555, 30912, 42037, 25107, 4175, 53616, 614, 3020, 38017, 16568, 29079, 8846, 20784, 63690,
    57232,
];
const PERM: [usize; 16] = [0, 10, 5, 15, 14, 4, 11, 1, 9, 3, 12, 6, 7, 13, 2, 8];
fn cells(data: &[u8]) -> [u8; 16] {
    std::array::from_fn(|i| {
        if i % 2 == 0 {
            data[i / 2] >> 4
        } else {
            data[i / 2] & 15
        }
    })
}
fn permute(x: &mut [u8; 16], inverse: bool) {
    let old = Zeroizing::new(*x);
    for (i, p) in PERM.into_iter().enumerate() {
        if inverse {
            x[p] = old[i];
        } else {
            x[i] = old[p];
        }
    }
}
fn mix(x: &mut [u8; 16]) {
    for col in x.as_chunks_mut::<4>().0 {
        let sum = col.iter().fold(0, |a, b| a ^ b);
        for b in col {
            *b ^= sum;
        }
    }
}
fn round_key(x: &mut [u8; 16], key: &[u8; 16], r: usize) {
    for (i, b) in x.iter_mut().enumerate() {
        *b ^= key[i] ^ ((RC[r + 1] >> (15 - i)) & 1) as u8;
    }
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let halves = Zeroizing::new([cells(&key[..8]), cells(&key[8..])]);
    let white: Zeroizing<[u8; 16]> =
        Zeroizing::new(std::array::from_fn(|i| halves[0][i] ^ halves[1][i]));
    let mut result = data.to_vec();
    for block in result.as_chunks_mut::<8>().0 {
        let mut x = Zeroizing::new(cells(block));
        for (i, b) in x.iter_mut().enumerate() {
            *b ^= white[i];
        }
        if inverse {
            for b in x.iter_mut() {
                *b = S[*b as usize];
            }
            for r in (0..15).rev() {
                round_key(&mut x, &halves[r % 2], r);
                mix(&mut x);
                permute(&mut x, true);
                for b in x.iter_mut() {
                    *b = S[*b as usize];
                }
            }
        } else {
            for r in 0..15 {
                for b in x.iter_mut() {
                    *b = S[*b as usize];
                }
                permute(&mut x, false);
                mix(&mut x);
                round_key(&mut x, &halves[r % 2], r);
            }
            for b in x.iter_mut() {
                *b = S[*b as usize];
            }
        }
        for (i, b) in block.iter_mut().enumerate() {
            *b = ((x[2 * i] ^ white[2 * i]) << 4) | (x[2 * i + 1] ^ white[2 * i + 1]);
        }
    }
    result
}
