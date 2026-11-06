//! Midori128 full twenty rounds, paper byte order and per-cell S-boxes.
//! Math checked against MIT AutoDiVer b4a509e and the two Appendix A answers.
use zeroize::Zeroizing;
const RC: [u16; 19] = [
    5555, 30912, 42037, 25107, 4175, 53616, 614, 3020, 38017, 16568, 29079, 8846, 20784, 63690,
    57232, 31873, 7204, 9140, 25226,
];
const S: [u8; 16] = [1, 0, 5, 3, 14, 2, 15, 7, 13, 10, 9, 11, 12, 8, 4, 6];
const BITS: [[usize; 8]; 4] = [
    [4, 1, 6, 3, 0, 5, 2, 7],
    [1, 6, 7, 0, 5, 2, 3, 4],
    [2, 3, 4, 1, 6, 7, 0, 5],
    [7, 4, 1, 2, 3, 0, 5, 6],
];
const PERM: [usize; 16] = [0, 10, 5, 15, 14, 4, 11, 1, 9, 3, 12, 6, 7, 13, 2, 8];
fn substitute(x: u8, cell: usize) -> u8 {
    let p = BITS[cell % 4];
    let before = p
        .iter()
        .enumerate()
        .fold(0u8, |v, (i, j)| v | (((x >> (7 - j)) & 1) << (7 - i)));
    let s = (S[(before >> 4) as usize] << 4) | S[(before & 15) as usize];
    p.iter()
        .enumerate()
        .fold(0, |v, (i, j)| v | (((s >> (7 - i)) & 1) << (7 - j)))
}
fn permute(x: &mut [u8; 16], inverse: bool) {
    let old = Zeroizing::new(*x);
    for (i, j) in PERM.into_iter().enumerate() {
        if inverse {
            x[j] = old[i];
        } else {
            x[i] = old[j];
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
        *b ^= key[i] ^ ((RC[r] >> (15 - i)) & 1) as u8;
    }
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let key = Zeroizing::new(<[u8; 16]>::try_from(key).unwrap());
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<16>().0 {
        let mut x = Zeroizing::new(*block);
        for (i, b) in x.iter_mut().enumerate() {
            *b ^= key[i];
        }
        if inverse {
            for (i, b) in x.iter_mut().enumerate() {
                *b = substitute(*b, i);
            }
            for r in (0..19).rev() {
                round_key(&mut x, &key, r);
                mix(&mut x);
                permute(&mut x, true);
                for (i, b) in x.iter_mut().enumerate() {
                    *b = substitute(*b, i);
                }
            }
        } else {
            for r in 0..19 {
                for (i, b) in x.iter_mut().enumerate() {
                    *b = substitute(*b, i);
                }
                permute(&mut x, false);
                mix(&mut x);
                round_key(&mut x, &key, r);
            }
            for (i, b) in x.iter_mut().enumerate() {
                *b = substitute(*b, i);
            }
        }
        for (i, b) in block.iter_mut().enumerate() {
            *b = x[i] ^ key[i];
        }
    }
    out
}
