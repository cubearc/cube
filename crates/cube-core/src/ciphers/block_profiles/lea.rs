//! LEA full-round reference profiles, 128-bit blocks and 128/192/256-bit keys.
//! Original KISA ARX schedule; little-endian words. Unauthenticated reference.
//! Reviewed alongside go-cryptobin e938a803 (Apache-2.0); rewritten in Rust.
use zeroize::Zeroizing;
const DELTA: [u32; 8] = [
    0xc3efe9db, 0x44626b02, 0x79e27c8a, 0x78df30ec, 0x715ea49e, 0xc785da0a, 0xe04ef22a, 0xe5c40957,
];
fn schedule(key: &[u8]) -> Zeroizing<Vec<[u32; 6]>> {
    assert!([16, 24, 32].contains(&key.len()));
    let words = key.len() / 4;
    let rounds = 16 + 2 * words;
    let mut state = Zeroizing::new([0u32; 8]);
    for (i, word) in key.as_chunks::<4>().0.iter().enumerate() {
        state[i] = u32::from_le_bytes(*word);
    }
    let mut keys = Zeroizing::new(vec![[0; 6]; rounds]);
    let rotations = [1, 3, 6, 11, 13, 17];
    for (round, row) in keys.iter_mut().enumerate() {
        let delta = DELTA[round % words];
        let count = if words == 4 { 4 } else { 6 };
        for j in 0..count {
            let index = if words == 8 { (6 * round + j) % 8 } else { j };
            state[index] = state[index]
                .wrapping_add(delta.rotate_left((round + j) as u32))
                .rotate_left(rotations[j]);
            row[j] = state[index];
        }
        if words == 4 {
            *row = [state[0], state[1], state[2], state[1], state[3], state[1]];
        }
    }
    keys
}
fn round(state: [u32; 4], key: &[u32; 6], decrypting: bool) -> [u32; 4] {
    if decrypting {
        let a = state[3];
        let b = state[0].rotate_right(9).wrapping_sub(a ^ key[0]) ^ key[1];
        let c = state[1].rotate_left(5).wrapping_sub(b ^ key[2]) ^ key[3];
        let d = state[2].rotate_left(3).wrapping_sub(c ^ key[4]) ^ key[5];
        [a, b, c, d]
    } else {
        [
            (state[0] ^ key[0])
                .wrapping_add(state[1] ^ key[1])
                .rotate_left(9),
            (state[1] ^ key[2])
                .wrapping_add(state[2] ^ key[3])
                .rotate_right(5),
            (state[2] ^ key[4])
                .wrapping_add(state[3] ^ key[5])
                .rotate_right(3),
            state[0],
        ]
    }
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(16));
    let keys = schedule(key);
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<16>().0 {
        let mut state: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
            u32::from_le_bytes(block[4 * i..4 * i + 4].try_into().unwrap())
        }));
        if decrypting {
            for key in keys.iter().rev() {
                *state = round(*state, key, true);
            }
        } else {
            for key in keys.iter() {
                *state = round(*state, key, false);
            }
        }
        for (word, bytes) in state.iter().zip(block.as_chunks_mut::<4>().0) {
            bytes.copy_from_slice(&word.to_le_bytes());
        }
    }
    output
}
