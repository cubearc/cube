//! SAFER K/SK reference with explicitly selected key width and round count.
//! Byte schedule and PHT reviewed against go-cryptobin e938a803 (Apache-2.0),
//! with pinned Crypto++ and LibTomCrypt published answers. Unaudited reference.
use zeroize::Zeroizing;
const fn exponent() -> [u8; 256] {
    let mut table = [0; 256];
    let mut value = 1u16;
    let mut i = 0;
    while i < 256 {
        table[i] = value as u8;
        value = (value * 45) % 257;
        i += 1;
    }
    table
}
const EXP: [u8; 256] = exponent();
const fn logarithm() -> [u8; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        table[EXP[i] as usize] = i as u8;
        i += 1;
    }
    table
}
const LOG: [u8; 256] = logarithm();
const PAIRS: [[(usize, usize); 4]; 3] = [
    [(0, 1), (2, 3), (4, 5), (6, 7)],
    [(0, 2), (4, 6), (1, 3), (5, 7)],
    [(0, 4), (1, 5), (2, 6), (3, 7)],
];
const PERM: [usize; 8] = [0, 4, 1, 5, 2, 6, 3, 7];
const INV: [usize; 8] = [0, 2, 4, 6, 1, 3, 5, 7];
fn schedule(key: &[u8], rounds: usize, strengthened: bool) -> Zeroizing<Vec<[u8; 8]>> {
    assert!([8, 16].contains(&key.len()));
    assert!((6..=13).contains(&rounds));
    let second = if key.len() == 8 { key } else { &key[8..] };
    let mut a = Zeroizing::new([0u8; 9]);
    let mut b = Zeroizing::new([0u8; 9]);
    for i in 0..8 {
        a[i] = key[i].rotate_left(5);
        a[8] ^= a[i];
        b[i] = second[i];
        b[8] ^= b[i];
    }
    let mut keys = Zeroizing::new(vec![[0u8; 8]; 2 * rounds + 1]);
    keys[0].copy_from_slice(second);
    for r in 1..=rounds {
        for value in a.iter_mut().chain(b.iter_mut()) {
            *value = value.rotate_left(6);
        }
        for j in 0..8 {
            let ai = if strengthened { (2 * r - 1 + j) % 9 } else { j };
            let bi = if strengthened { (2 * r + j) % 9 } else { j };
            keys[2 * r - 1][j] = a[ai].wrapping_add(EXP[EXP[(18 * r + j + 1) & 255] as usize]);
            keys[2 * r][j] = b[bi].wrapping_add(EXP[EXP[(18 * r + j + 10) & 255] as usize]);
        }
    }
    keys
}
fn xor_position(index: usize) -> bool {
    matches!(index, 0 | 3 | 4 | 7)
}
fn mix_key(state: &mut [u8; 8], key: &[u8; 8], inverse: bool) {
    for (i, (cell, key)) in state.iter_mut().zip(key).enumerate() {
        if xor_position(i) {
            *cell ^= key;
        } else if inverse {
            *cell = cell.wrapping_sub(*key);
        } else {
            *cell = cell.wrapping_add(*key);
        }
    }
}
fn diffusion(state: &mut [u8; 8], inverse: bool) {
    for step in 0..3 {
        let layer = if inverse { 2 - step } else { step };
        for (a, b) in PAIRS[layer] {
            if inverse {
                state[a] = state[a].wrapping_sub(state[b]);
                state[b] = state[b].wrapping_sub(state[a]);
            } else {
                state[b] = state[b].wrapping_add(state[a]);
                state[a] = state[a].wrapping_add(state[b]);
            }
        }
    }
}
fn permute(state: &mut [u8; 8], inverse: bool) {
    let before = Zeroizing::new(*state);
    let permutation = if inverse { INV } else { PERM };
    *state = std::array::from_fn(|i| before[permutation[i]]);
}
pub(super) fn transform(
    data: &[u8],
    key: &[u8],
    rounds: usize,
    strengthened: bool,
    decrypting: bool,
) -> Vec<u8> {
    assert!(data.len().is_multiple_of(8));
    let keys = schedule(key, rounds, strengthened);
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<8>().0 {
        let mut state = Zeroizing::new(*block);
        if decrypting {
            mix_key(&mut state, &keys[2 * rounds], true);
            for r in (0..rounds).rev() {
                permute(&mut state, true);
                diffusion(&mut state, true);
                for (i, cell) in state.iter_mut().enumerate() {
                    *cell = if xor_position(i) {
                        LOG[cell.wrapping_sub(keys[2 * r + 1][i]) as usize] ^ keys[2 * r][i]
                    } else {
                        EXP[(*cell ^ keys[2 * r + 1][i]) as usize].wrapping_sub(keys[2 * r][i])
                    };
                }
            }
        } else {
            for r in 0..rounds {
                mix_key(&mut state, &keys[2 * r], false);
                for (i, cell) in state.iter_mut().enumerate() {
                    *cell = if xor_position(i) {
                        EXP[*cell as usize].wrapping_add(keys[2 * r + 1][i])
                    } else {
                        LOG[*cell as usize] ^ keys[2 * r + 1][i]
                    };
                }
                diffusion(&mut state, false);
                permute(&mut state, false);
            }
            mix_key(&mut state, &keys[2 * rounds], false);
        }
        *block = *state;
    }
    output
}
