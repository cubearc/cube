//! Original SKINNY-64/128 full-round tweakey profiles, row-major cells.
//! Reference arithmetic and original-paper vectors reviewed alongside MIT
//! rweather/skinny-c99b8070. Unaudited educational safe-Rust implementation.
use zeroize::Zeroizing;
const SMALL: [u8; 16] = [12, 6, 9, 0, 1, 10, 2, 11, 3, 8, 5, 13, 4, 14, 7, 15];
const PERMUTATION: [usize; 16] = [9, 15, 8, 13, 10, 14, 12, 11, 0, 1, 2, 3, 4, 5, 6, 7];
const fn sbox_byte(mut x: u8) -> u8 {
    let mut i = 0;
    while i < 4 {
        x ^= (!(((x >> 1) | x) >> 2)) & 0x11;
        if i != 3 {
            x = ((x & 1) << 2)
                | ((x & 6) << 5)
                | ((x & 0x20) >> 5)
                | ((x & 0xc8) >> 2)
                | ((x & 0x10) >> 1);
        }
        i += 1;
    }
    (x & 0xf9) | ((x >> 1) & 2) | ((x << 1) & 4)
}
const fn big() -> [u8; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        table[i] = sbox_byte(i as u8);
        i += 1;
    }
    table
}
const BIG: [u8; 256] = big();
const fn inverse<const N: usize>(sbox: &[u8; N]) -> [u8; N] {
    let mut out = [0; N];
    let mut i = 0;
    while i < N {
        out[sbox[i] as usize] = i as u8;
        i += 1;
    }
    out
}
const SMALL_INV: [u8; 16] = inverse(&SMALL);
const BIG_INV: [u8; 256] = inverse(&BIG);
fn unpack(input: &[u8], output: &mut [u8], small: bool) {
    if small {
        for (b, pair) in input.iter().zip(output.as_chunks_mut::<2>().0) {
            pair[0] = b >> 4;
            pair[1] = b & 15;
        }
    } else {
        output.copy_from_slice(input);
    }
}
fn schedule(key: &[u8], width: usize) -> Zeroizing<Vec<[u8; 16]>> {
    let rounds = match (width, key.len()) {
        (8, 8) => 32,
        (8, 16) => 36,
        (8, 24) => 40,
        (16, 16) => 40,
        (16, 32) => 48,
        (16, 48) => 56,
        _ => unreachable!("validated full tweakey tuple"),
    };
    let small = width == 8;
    let components = key.len() / width;
    let mut tk = Zeroizing::new([0u8; 48]);
    unpack(key, &mut tk[..16 * components], small);
    let mut keys = Zeroizing::new(vec![[0u8; 16]; rounds]);
    let mut constant = 0u8;
    for row in keys.iter_mut() {
        for component in tk[..16 * components].as_chunks::<16>().0 {
            for (out, cell) in row[..8].iter_mut().zip(&component[..8]) {
                *out ^= cell;
            }
        }
        constant = ((constant << 1) ^ ((constant >> 5) & 1) ^ ((constant >> 4) & 1) ^ 1) & 63;
        row[0] ^= constant & 15;
        row[4] ^= constant >> 4;
        row[8] ^= 2;
        for (part, component) in tk[..16 * components]
            .as_chunks_mut::<16>()
            .0
            .iter_mut()
            .enumerate()
        {
            let before = Zeroizing::new(*component);
            *component = std::array::from_fn(|i| before[PERMUTATION[i]]);
            if part == 0 {
                continue;
            }
            for cell in &mut component[..8] {
                *cell = match (small, part) {
                    (true, 1) => ((*cell << 1) & 15) ^ ((*cell >> 3) & 1) ^ ((*cell >> 2) & 1),
                    (true, 2) => (*cell >> 1) ^ (((*cell & 1) ^ ((*cell >> 3) & 1)) << 3),
                    (false, 1) => (*cell << 1) ^ ((*cell >> 7) & 1) ^ ((*cell >> 5) & 1),
                    (false, 2) => (*cell >> 1) ^ (((*cell & 1) ^ ((*cell >> 6) & 1)) << 7),
                    _ => unreachable!(),
                };
            }
        }
    }
    keys
}
fn shift(state: &mut [u8; 16], inverse: bool) {
    for row in 1..4 {
        if inverse {
            state[4 * row..4 * row + 4].rotate_left(row);
        } else {
            state[4 * row..4 * row + 4].rotate_right(row);
        }
    }
}
fn mix(state: &mut [u8; 16], inverse: bool) {
    for col in 0..4 {
        let [a, b, c, d] = [state[col], state[col + 4], state[col + 8], state[col + 12]];
        let next = if inverse {
            [b, c ^ d ^ b, d ^ b, a ^ d]
        } else {
            [a ^ c ^ d, a, b ^ c, a ^ c]
        };
        for (row, cell) in next.into_iter().enumerate() {
            state[4 * row + col] = cell;
        }
    }
}
pub(super) fn transform(data: &[u8], key: &[u8], width: usize, decrypting: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(width));
    let keys = schedule(key, width);
    let small = width == 8;
    let sbox: &[u8] = match (small, decrypting) {
        (true, false) => &SMALL,
        (true, true) => &SMALL_INV,
        (false, false) => &BIG,
        (false, true) => &BIG_INV,
    };
    let mut output = data.to_vec();
    for block in output.chunks_exact_mut(width) {
        let mut state = Zeroizing::new([0u8; 16]);
        unpack(block, &mut state[..], small);
        for i in 0..keys.len() {
            let round = if decrypting { keys.len() - 1 - i } else { i };
            if decrypting {
                mix(&mut state, true);
                shift(&mut state, true);
                for (cell, key) in state.iter_mut().zip(keys[round]) {
                    *cell = sbox[(*cell ^ key) as usize];
                }
            } else {
                for (cell, key) in state.iter_mut().zip(keys[round]) {
                    *cell = sbox[*cell as usize] ^ key;
                }
                shift(&mut state, false);
                mix(&mut state, false);
            }
        }
        if small {
            for (byte, pair) in block.iter_mut().zip(state.as_chunks::<2>().0) {
                *byte = (pair[0] << 4) | pair[1];
            }
        } else {
            block.copy_from_slice(&state[..]);
        }
    }
    output
}
