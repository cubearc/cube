use super::{Profile, Settings};
use aes::cipher::{
    Block, BlockCipherDecrypt, BlockCipherEncrypt, KeyInit,
    consts::{U12, U16, U20, U24, U32},
};
fn blocks<C: BlockCipherEncrypt + BlockCipherDecrypt>(
    cipher: &C,
    data: &[u8],
    decrypting: bool,
) -> Vec<u8> {
    let mut output = data.to_vec();
    for bytes in output.chunks_exact_mut(C::block_size()) {
        let block: &mut Block<C> = bytes.try_into().expect("validated block width");
        if decrypting {
            cipher.decrypt_block(block);
        } else {
            cipher.encrypt_block(block);
        }
    }
    output
}
fn fixed<C: BlockCipherEncrypt + BlockCipherDecrypt + KeyInit>(
    key: &[u8],
    data: &[u8],
    decrypting: bool,
) -> Vec<u8> {
    blocks(
        &C::new_from_slice(key).expect("validated key"),
        data,
        decrypting,
    )
}
macro_rules! sized {
    ($key:expr, $data:expr, $decrypting:expr, $a:ty, $b:ty, $c:ty) => {
        match $key.len() {
            16 => fixed::<$a>($key, $data, $decrypting),
            24 => fixed::<$b>($key, $data, $decrypting),
            32 => fixed::<$c>($key, $data, $decrypting),
            _ => unreachable!("validated key"),
        }
    };
}
pub(super) fn transform(data: &[u8], settings: &Settings, decrypting: bool) -> Vec<u8> {
    let key = &settings.key[..];
    match settings.profile {
        Profile::Des => fixed::<des::Des>(key, data, decrypting),
        Profile::TdesEde2 => fixed::<des::TdesEde2>(key, data, decrypting),
        Profile::TdesEde3 => fixed::<des::TdesEde3>(key, data, decrypting),
        Profile::BlowfishBe => fixed::<blowfish::Blowfish>(key, data, decrypting),
        Profile::Idea => fixed::<idea::Idea>(key, data, decrypting),
        Profile::Rc2 => blocks(
            &rc2::Rc2::new_with_eff_key_len(
                key,
                usize::from(
                    settings
                        .effective_key_bits
                        .expect("validated effective bits"),
                ),
            ),
            data,
            decrypting,
        ),
        Profile::Camellia => sized!(
            key,
            data,
            decrypting,
            camellia::Camellia128,
            camellia::Camellia192,
            camellia::Camellia256
        ),
        Profile::Serpent => fixed::<serpent::Serpent>(key, data, decrypting),
        Profile::Cast128 => fixed::<cast5::Cast5>(key, data, decrypting),
        Profile::Cast256 => fixed::<cast6::Cast6>(key, data, decrypting),
        Profile::Aria => sized!(
            key,
            data,
            decrypting,
            aria::Aria128,
            aria::Aria192,
            aria::Aria256
        ),
        Profile::Twofish => fixed::<twofish::Twofish>(key, data, decrypting),
        Profile::Sm4 => fixed::<sm4::Sm4>(key, data, decrypting),
        Profile::Kuznyechik => fixed::<kuznyechik::Kuznyechik>(key, data, decrypting),
        Profile::Magma => fixed::<magma::Magma>(key, data, decrypting),
        Profile::Gift128 => fixed::<gift_cipher::Gift128>(key, data, decrypting),
        Profile::Xtea => fixed::<xtea::Xtea>(key, data, decrypting),
        Profile::Rc5 => fixed::<rc5::RC5<u32, U12, U16>>(key, data, decrypting),
        Profile::Rc6 => {
            sized!(key,data,decrypting,rc6::RC6<u32,U20,U16>,rc6::RC6<u32,U20,U24>,rc6::RC6<u32,U20,U32>)
        }
        Profile::Speck32_64 => fixed::<speck_cipher::Speck32_64>(key, data, decrypting),
        Profile::Speck48_72 => fixed::<speck_cipher::Speck48_72>(key, data, decrypting),
        Profile::Speck48_96 => fixed::<speck_cipher::Speck48_96>(key, data, decrypting),
        Profile::Speck64_96 => fixed::<speck_cipher::Speck64_96>(key, data, decrypting),
        Profile::Speck64_128 => fixed::<speck_cipher::Speck64_128>(key, data, decrypting),
        Profile::Speck96_96 => fixed::<speck_cipher::Speck96_96>(key, data, decrypting),
        Profile::Speck96_144 => fixed::<speck_cipher::Speck96_144>(key, data, decrypting),
        Profile::Speck128_128 => fixed::<speck_cipher::Speck128_128>(key, data, decrypting),
        Profile::Speck128_192 => fixed::<speck_cipher::Speck128_192>(key, data, decrypting),
        Profile::Speck128_256 => fixed::<speck_cipher::Speck128_256>(key, data, decrypting),
        Profile::GostTest => gost::<magma::Gost89Test>(key, data, decrypting),
        Profile::GostA => gost::<magma::Gost89CryptoProA>(key, data, decrypting),
        Profile::GostB => gost::<magma::Gost89CryptoProB>(key, data, decrypting),
        Profile::GostC => gost::<magma::Gost89CryptoProC>(key, data, decrypting),
        Profile::Tea => super::tea::transform(data, key, decrypting),
        Profile::Shacal2 => super::shacal2::transform(data, key, decrypting),
        Profile::Hight => super::hight::transform(data, key, decrypting),
        Profile::Misty1 => super::misty1::transform(data, key, decrypting),
        Profile::Kasumi => super::kasumi::transform(data, key, decrypting),
        Profile::Twine80 | Profile::Twine128 => super::twine::transform(data, key, decrypting),
        Profile::Simon32_64 => super::simon::transform(data, key, 4, decrypting),
        Profile::Simon48_72 => super::simon::transform(data, key, 6, decrypting),
        Profile::Simon48_96 => super::simon::transform(data, key, 6, decrypting),
        Profile::Simon64_96 => super::simon::transform(data, key, 8, decrypting),
        Profile::Simon64_128 => super::simon::transform(data, key, 8, decrypting),
        Profile::Simon96_96 => super::simon::transform(data, key, 12, decrypting),
        Profile::Simon96_144 => super::simon::transform(data, key, 12, decrypting),
        Profile::Simon128_128 => super::simon::transform(data, key, 16, decrypting),
        Profile::Simon128_192 => super::simon::transform(data, key, 16, decrypting),
        Profile::Simon128_256 => super::simon::transform(data, key, 16, decrypting),
        Profile::Skinny64_64 => super::skinny::transform(data, key, 8, decrypting),
        Profile::Skinny64_128 => super::skinny::transform(data, key, 8, decrypting),
        Profile::Skinny64_192 => super::skinny::transform(data, key, 8, decrypting),
        Profile::Skinny128_128 => super::skinny::transform(data, key, 16, decrypting),
        Profile::Skinny128_256 => super::skinny::transform(data, key, 16, decrypting),
        Profile::Skinny128_384 => super::skinny::transform(data, key, 16, decrypting),
        Profile::SaferK646 => super::safer::transform(data, key, 6, false, decrypting),
        Profile::SaferK12812 => super::safer::transform(data, key, 12, false, decrypting),
        Profile::SaferSk646 => super::safer::transform(data, key, 6, true, decrypting),
        Profile::SaferSk12810 => super::safer::transform(data, key, 10, true, decrypting),
        Profile::SaferK12810 => super::safer::transform(data, key, 10, false, decrypting),
        Profile::SaferSk648 => super::safer::transform(data, key, 8, true, decrypting),
        Profile::AnubisOriginal => super::anubis::transform(data, key, decrypting, false),
        Profile::AnubisTweaked => super::anubis::transform(data, key, decrypting, true),
        Profile::Khazad => super::khazad::transform(data, key, decrypting),
        Profile::Square => super::square::transform(data, key, decrypting),
        Profile::Seed => super::seed::transform(data, key, decrypting),
        Profile::Clefia => super::clefia::transform(data, key, decrypting),
        Profile::Mars => super::mars::transform(data, key, decrypting),
        Profile::Crypton => super::crypton::transform(data, key, decrypting),
        Profile::E2 => super::e2::transform(data, key, decrypting),
        // Standard AES tuples reuse the independently reviewed AES backend.
        Profile::Rijndael128 if matches!(key.len(), 16 | 24 | 32) => {
            sized!(key, data, decrypting, aes::Aes128, aes::Aes192, aes::Aes256)
        }
        Profile::Rijndael128 => super::rijndael::transform(data, key, 16, decrypting),
        Profile::Rijndael160 => super::rijndael::transform(data, key, 20, decrypting),
        Profile::Rijndael192 => super::rijndael::transform(data, key, 24, decrypting),
        Profile::Rijndael224 => super::rijndael::transform(data, key, 28, decrypting),
        Profile::Rijndael256 => super::rijndael::transform(data, key, 32, decrypting),
        Profile::Skipjack => super::skipjack::transform(data, key, decrypting),
        Profile::Noekeon => super::noekeon::transform(data, key, decrypting),
        Profile::Lea => super::lea::transform(data, key, decrypting),
        Profile::Present80 | Profile::Present128 => {
            super::present::transform(data, key, decrypting)
        }
    }
}

// The selected GOST89 wire convention uses LE key words and a left-first LE
// Feistel pair. RustCrypto's Magma layout is BE, right-first: reverse each key
// word and the complete eight-byte external block. No S-box is inferred.
fn gost<C: BlockCipherEncrypt + BlockCipherDecrypt + KeyInit>(
    key: &[u8],
    data: &[u8],
    decrypting: bool,
) -> Vec<u8> {
    let mut key = zeroize::Zeroizing::new(<[u8; 32]>::try_from(key).expect("validated key"));
    for word in key.as_chunks_mut::<4>().0 {
        word.reverse();
    }
    let cipher = C::new_from_slice(&key[..]).expect("validated key");
    let mut output = data.to_vec();
    for bytes in output.as_chunks_mut::<8>().0 {
        bytes.reverse();
        let block: &mut Block<C> = bytes.as_mut_slice().try_into().expect("GOST64 block");
        if decrypting {
            cipher.decrypt_block(block);
        } else {
            cipher.encrypt_block(block);
        }
        bytes.reverse();
    }
    output
}
