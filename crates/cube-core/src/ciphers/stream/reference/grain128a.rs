//! Designers' SKEW2011 Grain128a: full256-step setup,64 initial bits,
//! decimated keystream and optional32-bit accumulator authentication.
use zeroize::Zeroizing;
fn step(state: &mut [u128; 2], initializing: bool) -> u32 {
    let b = |n: usize| ((state[0] >> n) & 1u128) as u32;
    let s = |n: usize| ((state[1] >> n) & 1u128) as u32;
    let y = (b(12) & s(8))
        ^ (s(13) & s(20))
        ^ (b(95) & s(42))
        ^ (s(60) & s(79))
        ^ (b(12) & b(95) & s(94))
        ^ s(93)
        ^ b(2)
        ^ b(15)
        ^ b(36)
        ^ b(45)
        ^ b(64)
        ^ b(73)
        ^ b(89);
    let l = s(0) ^ s(7) ^ s(38) ^ s(70) ^ s(81) ^ s(96);
    let n = s(0)
        ^ b(0)
        ^ b(26)
        ^ b(56)
        ^ b(91)
        ^ b(96)
        ^ (b(3) & b(67))
        ^ (b(11) & b(13))
        ^ (b(17) & b(18))
        ^ (b(27) & b(59))
        ^ (b(40) & b(48))
        ^ (b(61) & b(65))
        ^ (b(68) & b(84))
        ^ (b(88) & b(92) & b(93) & b(95))
        ^ (b(22) & b(24) & b(25))
        ^ (b(70) & b(78) & b(82));
    let feedback = if initializing { y } else { 0 };
    state[0] = (state[0] >> 1) | (u128::from(n ^ feedback) << 127);
    state[1] = (state[1] >> 1) | (u128::from(l ^ feedback) << 127);
    y
}
fn initialize(key: &[u8], iv: &[u8]) -> Zeroizing<[u128; 2]> {
    let key = u128::from_be_bytes(key.try_into().expect("validated key")).reverse_bits();
    let mut padded = Zeroizing::new([0u8; 16]);
    padded[..12].copy_from_slice(iv);
    let l = u128::from_be_bytes(*padded).reverse_bits() | ((u128::from(u32::MAX) >> 1) << 96);
    let mut state = Zeroizing::new([key, l]);
    for _ in 0..256 {
        step(&mut state, true);
    }
    state
}
pub(super) fn apply(
    data: &mut [u8],
    key: &[u8],
    iv: &[u8],
    data_bits: usize,
    decrypting: bool,
) -> Zeroizing<[u8; 4]> {
    let mut state = initialize(key, iv);
    let mut acc = Zeroizing::new(0u32);
    let mut reg = Zeroizing::new(0u32);
    for _ in 0..32 {
        *acc = (*acc << 1) | step(&mut state, false);
    }
    for _ in 0..32 {
        *reg = (*reg << 1) | step(&mut state, false);
    }
    for i in 0..data_bits {
        let input = u32::from((data[i / 8] >> (7 - i % 8)) & 1);
        let stream = step(&mut state, false);
        let plaintext = if decrypting { input ^ stream } else { input };
        data[i / 8] ^= (stream as u8) << (7 - i % 8);
        *acc ^= *reg & plaintext.wrapping_neg();
        *reg = (*reg << 1) | step(&mut state, false);
    }
    Zeroizing::new((*acc ^ *reg).to_be_bytes())
}
