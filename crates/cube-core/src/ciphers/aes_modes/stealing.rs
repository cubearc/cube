//! Exact NIST CBC-CS orderings over a maintained AES/CTS implementation.
use super::*;
use aes::cipher::{
    BlockCipherDecrypt, BlockCipherEncrypt, BlockModeDecrypt, BlockModeEncrypt, KeyInit, KeyIvInit,
    block_padding::NoPadding,
};
use cts::{Decrypt, Encrypt};
pub(super) fn transform<C>(
    data: &mut [u8],
    s: &Settings,
    inverse: bool,
    kind: Stealing,
) -> Result<(), OperationError>
where
    C: BlockCipherEncrypt<BlockSize = aes::cipher::consts::U16> + BlockCipherDecrypt + KeyInit,
{
    // NIST includes the single-block minimum. CTS0.7's CS3 path performs
    // an extra encryption there; the defined result for all variants is CBC.
    if data.len() == 16 {
        if inverse {
            cbc::Decryptor::<C>::new_from_slices(&s.key, &s.initial_block)
                .map_err(|_| invalid())?
                .decrypt_padded::<NoPadding>(data)
                .map_err(|_| failed())?;
        } else {
            cbc::Encryptor::<C>::new_from_slices(&s.key, &s.initial_block)
                .map_err(|_| invalid())?
                .encrypt_padded::<NoPadding>(data, 16)
                .map_err(|_| failed())?;
        }
        return Ok(());
    }
    macro_rules! apply {
        ($mode:ty) => {{
            let cipher =
                <$mode>::new_from_slices(&s.key, &s.initial_block).map_err(|_| invalid())?;
            if inverse {
                cipher.decrypt(data)
            } else {
                cipher.encrypt(data)
            }
        }};
    }
    match kind {
        Stealing::Cs1 => apply!(cts::CbcCs1<C>),
        Stealing::Cs2 => apply!(cts::CbcCs2<C>),
        Stealing::Cs3 => apply!(cts::CbcCs3<C>),
    }
    .map_err(|_| failed())
}
