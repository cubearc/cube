//! SP800-38G Feistel rounds over the maintained RustCrypto AES primitive.
//! All lengths and domains are checked by the owning module before entry.
use super::*;
use aes::{
    Aes128, Aes192, Aes256,
    cipher::{BlockCipherEncrypt, BlockSizeUser, KeyInit, consts::U16},
};
use num_bigint::BigUint;
pub(super) fn transform(
    input: &[u16],
    s: &Settings,
    inverse: bool,
) -> Result<Vec<u16>, OperationError> {
    let mut key = Zeroizing::new(s.key.to_vec());
    if s.profile != Profile::Ff1 {
        key.reverse();
    }
    match key.len() {
        16 => run::<Aes128>(input, s, inverse, &key),
        24 => run::<Aes192>(input, s, inverse, &key),
        32 => run::<Aes256>(input, s, inverse, &key),
        _ => Err(invalid()),
    }
}
fn run<C: BlockCipherEncrypt + KeyInit + BlockSizeUser<BlockSize = U16>>(
    input: &[u16],
    s: &Settings,
    inverse: bool,
    key: &[u8],
) -> Result<Vec<u16>, OperationError> {
    let cipher = C::new_from_slice(key).map_err(|_| invalid())?;
    let u = if s.profile == Profile::Ff1 {
        input.len() / 2
    } else {
        input.len().div_ceil(2)
    };
    let v = input.len() - u;
    let mut a = Zeroizing::new(input[..u].to_vec());
    let mut b = Zeroizing::new(input[u..].to_vec());
    let radix = BigUint::from(s.radix);
    let moduli = [radix.pow(u as u32), radix.pow(v as u32)];
    let rounds: usize = if s.profile == Profile::Ff1 { 10 } else { 8 };
    for step in 0..rounds {
        let i = if inverse { rounds - 1 - step } else { step };
        let m = if i.is_multiple_of(2) { u } else { v };
        let source = if inverse { &a } else { &b };
        let y = if s.profile == Profile::Ff1 {
            ff1_y(&cipher, source, s, input.len(), u, v, i)
        } else {
            ff3_y(&cipher, source, s, i)
        };
        let old = if inverse { &b } else { &a };
        let reversed = s.profile != Profile::Ff1;
        let value = number(old, s.radix, reversed);
        let modulus = &moduli[i % 2];
        let result = if inverse {
            (value + modulus - (&y % modulus)) % modulus
        } else {
            (value + y) % modulus
        };
        let c = Zeroizing::new(digits(result, s.radix, m, reversed));
        if inverse {
            b = a;
            a = c;
        } else {
            a = b;
            b = c;
        }
    }
    let mut output = Vec::with_capacity(input.len());
    output.extend_from_slice(&a);
    output.extend_from_slice(&b);
    Ok(output)
}
fn number(digits: &[u16], radix: u32, reverse: bool) -> BigUint {
    let mut value = BigUint::from(0u8);
    for index in 0..digits.len() {
        let i = if reverse {
            digits.len() - 1 - index
        } else {
            index
        };
        value = value * radix + u32::from(digits[i]);
    }
    value
}
fn digits(mut value: BigUint, radix: u32, length: usize, reverse: bool) -> Vec<u16> {
    let mut result = vec![0; length];
    for index in (0..length).rev() {
        let n = (&value % radix).to_u32_digits();
        result[index] = n.first().copied().unwrap_or(0) as u16;
        value /= radix;
    }
    if reverse {
        result.reverse();
    }
    result
}
fn block<C: BlockCipherEncrypt + BlockSizeUser<BlockSize = U16>>(cipher: &C, bytes: &mut [u8; 16]) {
    let mut b = (*bytes).into();
    cipher.encrypt_block(&mut b);
    bytes.copy_from_slice(&b);
}
fn ff1_y<C: BlockCipherEncrypt + BlockSizeUser<BlockSize = U16>>(
    cipher: &C,
    source: &[u16],
    s: &Settings,
    n: usize,
    u: usize,
    v: usize,
    i: usize,
) -> BigUint {
    // ceil(log2(radix^v)) is the bit length of radix^v-1, exactly.
    let power = BigUint::from(s.radix).pow(v as u32) - 1u8;
    let bytes = (power.bits() as usize).div_ceil(8);
    let d = 4 * bytes.div_ceil(4) + 4;
    let mut p = [1, 2, 1, 0, 0, 0, 10, u as u8, 0, 0, 0, 0, 0, 0, 0, 0];
    p[3..6].copy_from_slice(&s.radix.to_be_bytes()[1..]);
    p[8..12].copy_from_slice(&(n as u32).to_be_bytes());
    p[12..].copy_from_slice(&(s.tweak.len() as u32).to_be_bytes());
    let mut q = Zeroizing::new(s.tweak.to_vec());
    q.resize(
        s.tweak.len() + (16 - (s.tweak.len() + bytes + 1) % 16) % 16,
        0,
    );
    q.push(i as u8);
    let number = Zeroizing::new(number(source, s.radix, false).to_bytes_be());
    let length = q.len() + bytes - number.len();
    q.resize(length, 0);
    q.extend_from_slice(&number);
    let mut r = Zeroizing::new(p);
    block(cipher, &mut r);
    for chunk in q.as_chunks::<16>().0 {
        for (a, b) in r.iter_mut().zip(chunk) {
            *a ^= *b;
        }
        block(cipher, &mut r);
    }
    let mut stream = Zeroizing::new(Vec::with_capacity(d.div_ceil(16) * 16));
    stream.extend_from_slice(&r[..]);
    for j in 1..d.div_ceil(16) {
        let mut next = Zeroizing::new(*r);
        for (a, b) in next.iter_mut().zip((j as u128).to_be_bytes()) {
            *a ^= b;
        }
        block(cipher, &mut next);
        stream.extend_from_slice(&next[..]);
    }
    BigUint::from_bytes_be(&stream[..d])
}
fn ff3_y<C: BlockCipherEncrypt + BlockSizeUser<BlockSize = U16>>(
    cipher: &C,
    source: &[u16],
    s: &Settings,
    i: usize,
) -> BigUint {
    let mut tweak = Zeroizing::new([0u8; 8]);
    if s.profile == Profile::Ff31 {
        tweak[..3].copy_from_slice(&s.tweak[..3]);
        tweak[3] = s.tweak[3] & 0xf0;
        tweak[4..7].copy_from_slice(&s.tweak[4..]);
        tweak[7] = s.tweak[3] << 4;
    } else {
        tweak.copy_from_slice(&s.tweak);
    }
    let mut p = Zeroizing::new([0u8; 16]);
    p[..4].copy_from_slice(if i.is_multiple_of(2) {
        &tweak[4..]
    } else {
        &tweak[..4]
    });
    p[3] ^= i as u8;
    let num = Zeroizing::new(number(source, s.radix, true).to_bytes_be());
    p[16 - num.len()..].copy_from_slice(&num);
    p.reverse();
    block(cipher, &mut p);
    p.reverse();
    BigUint::from_bytes_be(&p[..])
}
