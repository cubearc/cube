//! Full-round educational reference. Reviewed against go-cryptobin e938a803
//! (Apache-2.0); safe-Rust arithmetic and explicit external bytes. Unaudited.
use zeroize::Zeroizing;
const SBOX: [u8; 16] = [
    0x0C, 0x00, 0x0F, 0x0A, 0x02, 0x0B, 0x09, 0x05, 0x08, 0x03, 0x0D, 0x07, 0x01, 0x0E, 0x06, 0x04,
];
const SHUFFLE: [usize; 16] = [5, 0, 1, 4, 7, 12, 3, 8, 13, 6, 9, 2, 15, 10, 11, 14];
const INVERSE: [usize; 16] = [1, 2, 11, 6, 3, 0, 9, 4, 7, 10, 13, 14, 5, 8, 15, 12];
const RC: [u8; 36] = [
    0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x03, 0x06, 0x0c, 0x18, 0x30, 0x23, 0x05, 0x0a, 0x14, 0x28,
    0x13, 0x26, 0x0f, 0x1e, 0x3c, 0x3b, 0x35, 0x29, 0x11, 0x22, 0x07, 0x0e, 0x1c, 0x38, 0x33, 0x25,
    0x09, 0x12, 0x24, 0x0b,
];
fn schedule(key: &[u8]) -> Zeroizing<[[u8; 8]; 36]> {
    assert!([10, 16].contains(&key.len()));
    let n = 2 * key.len();
    let mut state = Zeroizing::new([0u8; 32]);
    for (byte, pair) in key.iter().zip(state.as_chunks_mut::<2>().0.iter_mut()) {
        pair[0] = byte >> 4;
        pair[1] = byte & 15;
    }
    let indices = if n == 20 {
        [1, 3, 4, 6, 13, 14, 15, 16]
    } else {
        [2, 3, 12, 15, 17, 18, 28, 31]
    };
    let mut keys = Zeroizing::new([[0u8; 8]; 36]);
    for (round, row) in keys.iter_mut().enumerate() {
        for (out, index) in row.iter_mut().zip(indices) {
            *out = state[index];
        }
        if round == 35 {
            break;
        }
        state[1] ^= SBOX[state[0] as usize];
        state[4] ^= SBOX[state[16] as usize];
        if n == 32 {
            state[23] ^= SBOX[state[30] as usize];
        }
        state[7] ^= RC[round] >> 3;
        state[19] ^= RC[round] & 7;
        state[..4].rotate_left(1);
        state[..n].rotate_left(4);
    }
    keys
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(8));
    let keys = schedule(key);
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<8>().0 {
        let mut state = Zeroizing::new([0u8; 16]);
        for (byte, pair) in block.iter().zip(state.as_chunks_mut::<2>().0.iter_mut()) {
            pair[0] = byte >> 4;
            pair[1] = byte & 15;
        }
        for i in 0..36 {
            let round = if decrypting { 35 - i } else { i };
            for (pair, key) in state.as_chunks_mut::<2>().0.iter_mut().zip(keys[round]) {
                pair[1] ^= SBOX[(pair[0] ^ key) as usize];
            }
            if i != 35 {
                let mut next = Zeroizing::new([0u8; 16]);
                for (index, byte) in state.iter().enumerate() {
                    next[if decrypting {
                        INVERSE[index]
                    } else {
                        SHUFFLE[index]
                    }] = *byte;
                }
                *state = *next;
            }
        }
        for (byte, pair) in block.iter_mut().zip(state.as_chunks::<2>().0) {
            *byte = (pair[0] << 4) | pair[1];
        }
    }
    output
}
