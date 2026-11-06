//! Revised full 25-round RECTANGLE; paper rows in big-endian order.
//! Specification tables/Feistel key math, independently checked Table10.
use zeroize::Zeroizing;
const S: [u8; 16] = [6, 5, 12, 10, 1, 14, 7, 9, 11, 0, 3, 13, 8, 15, 4, 2];
const RC: [u32; 25] = [
    1, 2, 4, 9, 18, 5, 11, 22, 12, 25, 19, 7, 15, 31, 30, 28, 24, 17, 3, 6, 13, 27, 23, 14, 29,
];
fn rotate(word: u32, amount: u32, bits: u32) -> u32 {
    if bits == 32 {
        word.rotate_left(amount)
    } else {
        ((word << amount) | (word >> (16 - amount))) & 65535
    }
}
fn substitute(words: &[u32], columns: usize, inverse: bool) -> Zeroizing<[u32; 4]> {
    let mut out: Zeroizing<[u32; 4]> = Zeroizing::new(words[..4].try_into().unwrap());
    for bit in 0..columns {
        let column = (0..4).fold(0, |n, row| n | (((words[row] >> bit) & 1) << row));
        let value = if inverse {
            S.iter().position(|n| u32::from(*n) == column).unwrap() as u8
        } else {
            S[column as usize]
        };
        for row in 0..4 {
            out[row] = (out[row] & !(1 << bit)) | (u32::from((value >> row) & 1) << bit);
        }
    }
    out
}
fn schedule(key: &[u8]) -> Zeroizing<[[u32; 4]; 26]> {
    let short = key.len() == 10;
    let mut state = Zeroizing::new([0u32; 5]);
    if short {
        for (i, b) in key.as_chunks::<2>().0.iter().enumerate() {
            state[i] = u32::from(u16::from_be_bytes(*b));
        }
    } else {
        for (i, b) in key.as_chunks::<4>().0.iter().enumerate() {
            state[i] = u32::from_be_bytes(*b);
        }
    }
    let mut out = Zeroizing::new([[0u32; 4]; 26]);
    for r in 0..=25 {
        for i in 0..4 {
            out[r][i] = state[i] & 65535;
        }
        if r == 25 {
            break;
        }
        let s = substitute(&state[..4], if short { 4 } else { 8 }, false);
        state[..4].copy_from_slice(&s[..]);
        let old = Zeroizing::new(*state);
        if short {
            *state = [
                rotate(old[0], 8, 16) ^ old[1],
                old[2],
                old[3],
                rotate(old[3], 12, 16) ^ old[4],
                old[0],
            ];
        } else {
            *state = [
                rotate(old[0], 8, 32) ^ old[1],
                old[2],
                rotate(old[2], 16, 32) ^ old[3],
                old[0],
                0,
            ];
        }
        state[0] ^= RC[r];
    }
    out
}
pub(super) fn transform(data: &[u8], key: &[u8], inverse: bool) -> Vec<u8> {
    let keys = schedule(key);
    let mut out = data.to_vec();
    for block in out.as_chunks_mut::<8>().0 {
        let mut x: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
            u32::from(u16::from_be_bytes(
                block[2 * i..2 * i + 2].try_into().unwrap(),
            ))
        }));
        if inverse {
            for i in 0..4 {
                x[i] ^= keys[25][i];
            }
            for r in (0..25).rev() {
                for (i, n) in [0, 1, 12, 13].into_iter().enumerate().skip(1) {
                    x[i] = rotate(x[i], 16 - n, 16);
                }
                *x = *substitute(&x[..], 16, true);
                for i in 0..4 {
                    x[i] ^= keys[r][i];
                }
            }
        } else {
            for r in 0..25 {
                for i in 0..4 {
                    x[i] ^= keys[r][i];
                }
                *x = *substitute(&x[..], 16, false);
                for (i, n) in [0, 1, 12, 13].into_iter().enumerate().skip(1) {
                    x[i] = rotate(x[i], n, 16);
                }
            }
            for i in 0..4 {
                x[i] ^= keys[25][i];
            }
        }
        for (i, b) in block.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            *b = (x[i] as u16).to_be_bytes();
        }
    }
    out
}
