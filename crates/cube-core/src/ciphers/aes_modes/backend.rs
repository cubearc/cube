//! RustCrypto mode adapters; no SDK implements a cryptographic algorithm.
use super::*;
use aes::{
    Aes128, Aes192, Aes256,
    cipher::{
        BlockCipherDecrypt, BlockCipherEncrypt, BlockModeDecrypt, BlockModeEncrypt, KeyInit,
        KeyIvInit, StreamCipher,
        block_padding::{NoPadding, Pkcs7},
    },
};
pub(super) fn transform(
    data: &[u8],
    s: &Settings,
    inverse: bool,
) -> Result<Vec<u8>, OperationError> {
    match s.key.len() {
        16 => apply::<Aes128>(data, s, inverse),
        24 => apply::<Aes192>(data, s, inverse),
        _ => apply::<Aes256>(data, s, inverse),
    }
}
fn apply<C>(data: &[u8], s: &Settings, inverse: bool) -> Result<Vec<u8>, OperationError>
where
    C: BlockCipherEncrypt<BlockSize = aes::cipher::consts::U16> + BlockCipherDecrypt + KeyInit,
{
    let mut buffer = Zeroizing::new(data.to_vec());
    // CBC/ECB encrypt into a bounded buffer, then return only validated bytes.
    if matches!(
        s.mode,
        Mode::Ecb(Padding::Pkcs7) | Mode::Cbc(Padding::Pkcs7)
    ) && !inverse
    {
        buffer.resize((data.len() / 16 + 1) * 16, 0);
    }
    let output_len = match s.mode {
        Mode::Ecb(padding) => {
            if inverse {
                unpad(
                    ecb::Decryptor::<C>::new_from_slice(&s.key).map_err(|_| invalid())?,
                    &mut buffer,
                    padding,
                )?
            } else {
                pad(
                    ecb::Encryptor::<C>::new_from_slice(&s.key).map_err(|_| invalid())?,
                    &mut buffer,
                    data.len(),
                    padding,
                )?
            }
        }
        Mode::Cbc(padding) => {
            if inverse {
                unpad(
                    cbc::Decryptor::<C>::new_from_slices(&s.key, &s.iv).map_err(|_| invalid())?,
                    &mut buffer,
                    padding,
                )?
            } else {
                pad(
                    cbc::Encryptor::<C>::new_from_slices(&s.key, &s.iv).map_err(|_| invalid())?,
                    &mut buffer,
                    data.len(),
                    padding,
                )?
            }
        }
        Mode::Cfb128 => {
            if inverse {
                cfb_mode::Decryptor::<C>::new_from_slices(&s.key, &s.iv)
                    .map_err(|_| invalid())?
                    .decrypt(&mut buffer);
            } else {
                cfb_mode::Encryptor::<C>::new_from_slices(&s.key, &s.iv)
                    .map_err(|_| invalid())?
                    .encrypt(&mut buffer);
            }
            data.len()
        }
        Mode::Ofb => {
            ofb::Ofb::<C>::new_from_slices(&s.key, &s.iv)
                .map_err(|_| invalid())?
                .try_apply_keystream(&mut buffer)
                .map_err(|_| failed())?;
            data.len()
        }
        Mode::Ctr128Be => {
            ctr::Ctr128BE::<C>::new_from_slices(&s.key, &s.iv)
                .map_err(|_| invalid())?
                .try_apply_keystream(&mut buffer)
                .map_err(|_| failed())?;
            data.len()
        }
    };
    if output_len > MAX_INPUT && inverse {
        return Err(failed());
    }
    Ok(buffer[..output_len].to_vec())
}

fn pad<C: BlockModeEncrypt<BlockSize = aes::cipher::consts::U16>>(
    cipher: C,
    buffer: &mut [u8],
    length: usize,
    padding: Padding,
) -> Result<usize, OperationError> {
    match padding {
        Padding::None => cipher.encrypt_padded::<NoPadding>(buffer, length),
        Padding::Pkcs7 => cipher.encrypt_padded::<Pkcs7>(buffer, length),
    }
    .map(|bytes| bytes.len())
    .map_err(|_| failed())
}
fn unpad<C: BlockModeDecrypt<BlockSize = aes::cipher::consts::U16>>(
    cipher: C,
    buffer: &mut [u8],
    padding: Padding,
) -> Result<usize, OperationError> {
    match padding {
        Padding::None => cipher.decrypt_padded::<NoPadding>(buffer),
        Padding::Pkcs7 => cipher.decrypt_padded::<Pkcs7>(buffer),
    }
    .map(|bytes| bytes.len())
    .map_err(|_| failed())
}
