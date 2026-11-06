//! Full-round educational reference. Reviewed against go-cryptobin e938a803
//! (Apache-2.0); safe-Rust arithmetic and explicit external bytes. Unaudited.
use zeroize::Zeroizing;
const DELTA: [u8; 128] = [
    0x5A, 0x6D, 0x36, 0x1B, 0x0D, 0x06, 0x03, 0x41, 0x60, 0x30, 0x18, 0x4C, 0x66, 0x33, 0x59, 0x2C,
    0x56, 0x2B, 0x15, 0x4A, 0x65, 0x72, 0x39, 0x1C, 0x4E, 0x67, 0x73, 0x79, 0x3C, 0x5E, 0x6F, 0x37,
    0x5B, 0x2D, 0x16, 0x0B, 0x05, 0x42, 0x21, 0x50, 0x28, 0x54, 0x2A, 0x55, 0x6A, 0x75, 0x7A, 0x7D,
    0x3E, 0x5F, 0x2F, 0x17, 0x4B, 0x25, 0x52, 0x29, 0x14, 0x0A, 0x45, 0x62, 0x31, 0x58, 0x6C, 0x76,
    0x3B, 0x1D, 0x0E, 0x47, 0x63, 0x71, 0x78, 0x7C, 0x7E, 0x7F, 0x3F, 0x1F, 0x0F, 0x07, 0x43, 0x61,
    0x70, 0x38, 0x5C, 0x6E, 0x77, 0x7B, 0x3D, 0x1E, 0x4F, 0x27, 0x53, 0x69, 0x34, 0x1A, 0x4D, 0x26,
    0x13, 0x49, 0x24, 0x12, 0x09, 0x04, 0x02, 0x01, 0x40, 0x20, 0x10, 0x08, 0x44, 0x22, 0x11, 0x48,
    0x64, 0x32, 0x19, 0x0C, 0x46, 0x23, 0x51, 0x68, 0x74, 0x3A, 0x5D, 0x2E, 0x57, 0x6B, 0x35, 0x5A,
];
fn f0(x: u8) -> u8 {
    x.rotate_left(1) ^ x.rotate_left(2) ^ x.rotate_left(7)
}
fn f1(x: u8) -> u8 {
    x.rotate_left(3) ^ x.rotate_left(4) ^ x.rotate_left(6)
}
fn schedule(key: &[u8]) -> Zeroizing<[u8; 136]> {
    assert_eq!(key.len(), 16);
    let mut keys = Zeroizing::new([0u8; 136]);
    keys[..4].copy_from_slice(&key[12..]);
    keys[4..8].copy_from_slice(&key[..4]);
    for i in 0..8 {
        for j in 0..8 {
            for half in [0, 8] {
                let index = 8 + 16 * i + j + half;
                keys[index] = key[(j + 8 - i) % 8 + half].wrapping_add(DELTA[16 * i + j + half]);
            }
        }
    }
    keys
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert!(data.len().is_multiple_of(8));
    let k = schedule(key);
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<8>().0 {
        let mut s = Zeroizing::new(*block);
        if decrypting {
            *s = [
                block[7],
                block[0].wrapping_sub(k[4]),
                block[1],
                block[2] ^ k[5],
                block[3],
                block[4].wrapping_sub(k[6]),
                block[5],
                block[6] ^ k[7],
            ];
        } else {
            s[0] = s[0].wrapping_add(k[0]);
            s[2] ^= k[1];
            s[4] = s[4].wrapping_add(k[2]);
            s[6] ^= k[3];
        }
        for i in 0..32 {
            let round = if decrypting { 31 - i } else { i };
            // Each round updates four alternating bytes. Its inputs remain unchanged.
            let start = (7 + 8 - round % 8) % 8;
            for j in 0..4 {
                let dest = (start + 8 - 2 * j) % 8;
                let input = (dest + 7) % 8;
                let subkey = k[8 + 4 * round + 3 - j];
                if j % 2 == 0 {
                    s[dest] ^= f0(s[input]).wrapping_add(subkey);
                } else if decrypting {
                    s[dest] = s[dest].wrapping_sub(f1(s[input]) ^ subkey);
                } else {
                    s[dest] = s[dest].wrapping_add(f1(s[input]) ^ subkey);
                }
            }
        }
        if decrypting {
            *block = [
                s[0].wrapping_sub(k[0]),
                s[1],
                s[2] ^ k[1],
                s[3],
                s[4].wrapping_sub(k[2]),
                s[5],
                s[6] ^ k[3],
                s[7],
            ];
        } else {
            *block = [
                s[1].wrapping_add(k[4]),
                s[2],
                s[3] ^ k[5],
                s[4],
                s[5].wrapping_add(k[6]),
                s[6],
                s[7] ^ k[7],
                s[0],
            ];
        }
    }
    output
}
