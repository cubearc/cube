//! Full-round educational reference. Reviewed against go-cryptobin e938a803
//! (Apache-2.0); safe-Rust arithmetic and explicit external bytes. Unaudited.
use zeroize::Zeroizing;
pub(super) fn transform(data: &[u8], key: &[u8], decrypting: bool) -> Vec<u8> {
    assert_eq!(key.len(), 16);
    assert!(data.len().is_multiple_of(8));
    let k: Zeroizing<[u32; 4]> = Zeroizing::new(std::array::from_fn(|i| {
        u32::from_be_bytes(key[4 * i..4 * i + 4].try_into().unwrap())
    }));
    let mut output = data.to_vec();
    for block in output.as_chunks_mut::<8>().0 {
        let mut state = Zeroizing::new([
            u32::from_be_bytes(block[..4].try_into().unwrap()),
            u32::from_be_bytes(block[4..].try_into().unwrap()),
        ]);
        let mut sum = if decrypting { 0xc6ef3720u32 } else { 0 };
        let f = |word: u32, lo: u32, hi: u32, sum: u32| {
            (word << 4).wrapping_add(lo) ^ word.wrapping_add(sum) ^ (word >> 5).wrapping_add(hi)
        };
        for _ in 0..32 {
            if decrypting {
                state[1] = state[1].wrapping_sub(f(state[0], k[2], k[3], sum));
                state[0] = state[0].wrapping_sub(f(state[1], k[0], k[1], sum));
                sum = sum.wrapping_sub(0x9e3779b9);
            } else {
                sum = sum.wrapping_add(0x9e3779b9);
                state[0] = state[0].wrapping_add(f(state[1], k[0], k[1], sum));
                state[1] = state[1].wrapping_add(f(state[0], k[2], k[3], sum));
            }
        }
        block[..4].copy_from_slice(&state[0].to_be_bytes());
        block[4..].copy_from_slice(&state[1].to_be_bytes());
    }
    output
}
