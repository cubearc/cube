//! Corrected Block TEA (XXTEA): complete little-endian word sequences.
//! Pure original round arithmetic, qualified against the raw uint32_t kernel
//! in xxtea-c7ec9615 (MIT). No key-string normalization or length suffix.
use zeroize::Zeroizing;
const DELTA: u32 = 0x9e3779b9;
fn mix(z: u32, y: u32, sum: u32, key: &[u32; 4], p: usize) -> u32 {
    ((z >> 5) ^ (y << 2)).wrapping_add((y >> 3) ^ (z << 4))
        ^ ((sum ^ y).wrapping_add(key[(p & 3) ^ ((sum >> 2) & 3) as usize] ^ z))
}
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert_eq!(key.len(), 16);
    if data.is_empty() {
        return Vec::new();
    }
    assert!(data.len() >= 8 && data.len().is_multiple_of(4));
    let keys: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
        u32::from_le_bytes(key[4 * i..4 * i + 4].try_into().unwrap())
    }));
    let mut words = Zeroizing::new(
        data.as_chunks::<4>()
            .0
            .iter()
            .map(|bytes| u32::from_le_bytes(*bytes))
            .collect::<Vec<_>>(),
    );
    let n = words.len();
    let rounds = 6 + 52 / n;
    if decrypting {
        let mut sum = DELTA.wrapping_mul(rounds as u32);
        let mut y = words[0];
        for _ in 0..rounds {
            for p in (1..n).rev() {
                let z = words[p - 1];
                words[p] = words[p].wrapping_sub(mix(z, y, sum, &keys, p));
                y = words[p];
            }
            let z = words[n - 1];
            words[0] = words[0].wrapping_sub(mix(z, y, sum, &keys, 0));
            y = words[0];
            sum = sum.wrapping_sub(DELTA);
        }
    } else {
        let mut sum = 0u32;
        let mut z = words[n - 1];
        for _ in 0..rounds {
            sum = sum.wrapping_add(DELTA);
            for p in 0..n - 1 {
                let y = words[p + 1];
                words[p] = words[p].wrapping_add(mix(z, y, sum, &keys, p));
                z = words[p];
            }
            let y = words[0];
            words[n - 1] = words[n - 1].wrapping_add(mix(z, y, sum, &keys, n - 1));
            z = words[n - 1];
        }
    }
    words.iter().flat_map(|word| word.to_le_bytes()).collect()
}
