//! NOEKEON direct-key full-round reference, big-endian words.
//! Original NESSIE specification; independently checked published vectors.
//! Reviewed alongside go-cryptobin e938a803 (Apache-2.0); rewritten in Rust.
//! Unauthenticated, unaudited reference implementation.
use zeroize::Zeroizing;
const RC: [u32; 17] = [
    0x80, 0x1b, 0x36, 0x6c, 0xd8, 0xab, 0x4d, 0x9a, 0x2f, 0x5e, 0xbc, 0x63, 0xc6, 0x97, 0x35, 0x6a,
    0xd4,
];
fn theta(state: &mut [u32; 4], key: &[u32; 4]) {
    let mut t = state[0] ^ state[2];
    t ^= t.rotate_left(8) ^ t.rotate_right(8);
    state[1] ^= t ^ key[1];
    state[3] ^= t ^ key[3];
    t = state[1] ^ state[3];
    t ^= t.rotate_left(8) ^ t.rotate_right(8);
    state[0] ^= t ^ key[0];
    state[2] ^= t ^ key[2];
}
fn nonlinear(state: &mut [u32; 4]) {
    let [mut a, mut b, mut c, mut d] = *state;
    b = b.rotate_left(1);
    c = c.rotate_left(5);
    d = d.rotate_left(2);
    b ^= !(d | c);
    a ^= c & b;
    std::mem::swap(&mut a, &mut d);
    c ^= a ^ b ^ d;
    b ^= !(d | c);
    a ^= c & b;
    *state = [a, b.rotate_right(1), c.rotate_right(5), d.rotate_right(2)];
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert_eq!(key.len(), 16);
    assert!(data.len().is_multiple_of(16));
    let mut keys: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
        u32::from_be_bytes(key[4 * i..4 * i + 4].try_into().unwrap())
    }));
    if decrypting {
        theta(&mut keys, &[0; 4]);
    }
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<16>().0 {
        let mut state: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
            u32::from_be_bytes(block[4 * i..4 * i + 4].try_into().unwrap())
        }));
        if decrypting {
            for round in (1..=16).rev() {
                theta(&mut state, &keys);
                state[0] ^= RC[round];
                nonlinear(&mut state);
            }
            theta(&mut state, &keys);
            state[0] ^= RC[0];
        } else {
            for constant in &RC[..16] {
                state[0] ^= constant;
                theta(&mut state, &keys);
                nonlinear(&mut state);
            }
            state[0] ^= RC[16];
            theta(&mut state, &keys);
        }
        for (word, bytes) in state.iter().zip(block.as_chunks_mut::<4>().0) {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
    }
    output
}
