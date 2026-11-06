//! Full-round educational reference. Reviewed against go-cryptobin e938a803
//! (Apache-2.0); safe-Rust arithmetic and explicit external bytes. Unaudited.
use zeroize::Zeroizing;
const RC: [u32; 64] = [
    0x428A2F98, 0x71374491, 0xB5C0FBCF, 0xE9B5DBA5, 0x3956C25B, 0x59F111F1, 0x923F82A4, 0xAB1C5ED5,
    0xD807AA98, 0x12835B01, 0x243185BE, 0x550C7DC3, 0x72BE5D74, 0x80DEB1FE, 0x9BDC06A7, 0xC19BF174,
    0xE49B69C1, 0xEFBE4786, 0x0FC19DC6, 0x240CA1CC, 0x2DE92C6F, 0x4A7484AA, 0x5CB0A9DC, 0x76F988DA,
    0x983E5152, 0xA831C66D, 0xB00327C8, 0xBF597FC7, 0xC6E00BF3, 0xD5A79147, 0x06CA6351, 0x14292967,
    0x27B70A85, 0x2E1B2138, 0x4D2C6DFC, 0x53380D13, 0x650A7354, 0x766A0ABB, 0x81C2C92E, 0x92722C85,
    0xA2BFE8A1, 0xA81A664B, 0xC24B8B70, 0xC76C51A3, 0xD192E819, 0xD6990624, 0xF40E3585, 0x106AA070,
    0x19A4C116, 0x1E376C08, 0x2748774C, 0x34B0BCB5, 0x391C0CB3, 0x4ED8AA4A, 0x5B9CCA4F, 0x682E6FF3,
    0x748F82EE, 0x78A5636F, 0x84C87814, 0x8CC70208, 0x90BEFFFA, 0xA4506CEB, 0xBEF9A3F7, 0xC67178F2,
];
fn big0(a: u32) -> u32 {
    a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22)
}
fn big1(a: u32) -> u32 {
    a.rotate_right(6) ^ a.rotate_right(11) ^ a.rotate_right(25)
}
fn choice(a: u32, b: u32, c: u32) -> u32 {
    (a & b) ^ (!a & c)
}
fn majority(a: u32, b: u32, c: u32) -> u32 {
    (a & b) ^ (a & c) ^ (b & c)
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert!((16..=64).contains(&key.len()) && key.len().is_multiple_of(4));
    assert!(data.len().is_multiple_of(32));
    let mut keys = Zeroizing::new([0u32; 64]);
    for (slot, bytes) in keys.iter_mut().zip(key.as_chunks::<4>().0) {
        *slot = u32::from_be_bytes(*bytes);
    }
    for i in 16..64 {
        let a = keys[i - 15];
        let b = keys[i - 2];
        keys[i] = keys[i - 16]
            .wrapping_add(a.rotate_right(7) ^ a.rotate_right(18) ^ (a >> 3))
            .wrapping_add(keys[i - 7])
            .wrapping_add(b.rotate_right(17) ^ b.rotate_right(19) ^ (b >> 10));
    }
    for (word, constant) in keys.iter_mut().zip(RC) {
        *word = word.wrapping_add(constant);
    }
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<32>().0 {
        let mut state: Zeroizing<[u32; 8]> = Zeroizing::new(std::array::from_fn(|i| {
            u32::from_be_bytes(block[4 * i..4 * i + 4].try_into().unwrap())
        }));
        for i in 0..64 {
            let [a, b, c, d, e, f, g, h] = *state;
            if decrypting {
                let t = a.wrapping_sub(big0(b).wrapping_add(majority(b, c, d)));
                let old_h = t
                    .wrapping_sub(big1(f))
                    .wrapping_sub(choice(f, g, h))
                    .wrapping_sub(keys[63 - i]);
                *state = [b, c, d, e.wrapping_sub(t), f, g, h, old_h];
            } else {
                let t = h
                    .wrapping_add(big1(e))
                    .wrapping_add(choice(e, f, g))
                    .wrapping_add(keys[i]);
                *state = [
                    t.wrapping_add(big0(a)).wrapping_add(majority(a, b, c)),
                    a,
                    b,
                    c,
                    d.wrapping_add(t),
                    e,
                    f,
                    g,
                ];
            }
        }
        for (word, bytes) in state.iter().zip(block.as_chunks_mut::<4>().0) {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
    }
    output
}
