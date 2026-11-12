//! GSM A5 raw COUNT22, big-endian key bytes, MSB-first concatenatedDL/UL bits.
//! Offline historical math only; KASUMI stays owned by the block module.
use crate::ciphers::block_profiles::kasumi;
use zeroize::Zeroizing;
const WIDTH: [u32; 4] = [19, 22, 23, 17];
const TAPS: [u32; 4] = [0x072000, 0x300000, 0x700080, 0x010800];
fn majority(a: u32, b: u32, c: u32) -> u32 {
    (a & b) | (a & c) | (b & c)
}
fn bit(r: u32, n: u32) -> u32 {
    (r >> n) & 1
}
fn clock(r: &mut [u32; 4], second: bool, force: bool) {
    let controls = if second {
        [bit(r[3], 10), bit(r[3], 3), bit(r[3], 7)]
    } else {
        [bit(r[0], 8), bit(r[1], 10), bit(r[2], 10)]
    };
    let m = majority(controls[0], controls[1], controls[2]);
    for i in 0..if second { 4 } else { 3 } {
        if i == 3 || force || controls[i] == m {
            r[i] = ((r[i] << 1) & ((1 << WIDTH[i]) - 1)) | ((r[i] & TAPS[i]).count_ones() & 1);
        }
    }
}
fn output(r: &[u32; 4], second: bool) -> u32 {
    let high = bit(r[0], 18) ^ bit(r[1], 21) ^ bit(r[2], 22);
    if second {
        high ^ majority(bit(r[0], 15), bit(r[0], 14) ^ 1, bit(r[0], 12))
            ^ majority(bit(r[1], 16) ^ 1, bit(r[1], 13), bit(r[1], 9))
            ^ majority(bit(r[2], 18), bit(r[2], 16), bit(r[2], 13) ^ 1)
    } else {
        high
    }
}
fn legacy(data: &mut [u8], key: &[u8], count: u32, data_bits: usize, second: bool) {
    let mut r = Zeroizing::new([0u32; 4]);
    for i in 0..86 {
        let b = if i < 64 {
            u32::from((key[7 - i / 8] >> (i % 8)) & 1)
        } else {
            (count >> (i - 64)) & 1
        };
        clock(&mut r, second, true);
        for w in &mut r[..if second { 4 } else { 3 }] {
            *w ^= b;
        }
    }
    if second {
        for (i, n) in [15, 16, 18, 10].iter().enumerate() {
            r[i] |= 1 << n;
        }
    }
    for _ in 0..if second { 99 } else { 100 } {
        clock(&mut r, second, false);
    }
    for i in 0..data_bits {
        clock(&mut r, second, false);
        data[i / 8] ^= (output(&r, second) as u8) << (7 - i % 8);
    }
}
fn encrypt_block(key: &[u8], word: u64) -> u64 {
    let output = Zeroizing::new(kasumi::transform(&word.to_be_bytes(), key, false));
    u64::from_be_bytes(output[..].try_into().expect("one KASUMI block"))
}
fn kgcore(data: &mut [u8], key: &[u8], count: u32, data_bits: usize) {
    let mut ck = Zeroizing::new([0u8; 16]);
    if key.len() == 8 {
        ck[..8].copy_from_slice(key);
        ck[8..].copy_from_slice(key);
    } else {
        ck.copy_from_slice(key);
    }
    let modified = Zeroizing::new(ck.map(|b| b ^ 0x55));
    let a = Zeroizing::new(encrypt_block(
        &*modified,
        (u64::from(count) << 32) | (0xf << 16),
    ));
    let mut block = Zeroizing::new(0u64);
    for i in 0..data_bits {
        if i % 64 == 0 {
            *block = encrypt_block(&*ck, *a ^ ((i / 64) as u64) ^ *block);
        }
        data[i / 8] ^= (((*block >> (63 - i % 64)) & 1) as u8) << (7 - i % 8);
    }
}
pub(super) fn apply(data: &mut [u8], key: &[u8], nonce: &[u8], data_bits: usize, variant: u8) {
    let count = (u32::from(nonce[0]) << 16) | (u32::from(nonce[1]) << 8) | u32::from(nonce[2]);
    if variant <= 2 {
        legacy(data, key, count, data_bits, variant == 2);
    } else {
        kgcore(data, key, count, data_bits);
    }
}
