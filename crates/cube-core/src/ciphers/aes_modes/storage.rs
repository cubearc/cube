//! IEEE1619 XTS, and its named XEX component restricted to full blocks.
use super::*;
use aes::{
    Aes128, Aes256,
    cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit},
};
pub(super) fn transform(
    data: &[u8],
    s: &Settings,
    inverse: bool,
) -> Result<Vec<u8>, OperationError> {
    let mut buffer = Zeroizing::new(data.to_vec());
    match s.key.len() {
        32 => apply::<Aes128>(&mut buffer, s, inverse)?,
        64 => apply::<Aes256>(&mut buffer, s, inverse)?,
        _ => return Err(invalid()),
    };
    Ok(buffer.to_vec())
}
fn apply<C>(data: &mut [u8], s: &Settings, inverse: bool) -> Result<(), OperationError>
where
    C: BlockCipherEncrypt<BlockSize = aes::cipher::consts::U16> + BlockCipherDecrypt + KeyInit,
{
    let half = s.key.len() / 2;
    let first = C::new_from_slice(&s.key[..half]).map_err(|_| invalid())?;
    let second = C::new_from_slice(&s.key[half..]).map_err(|_| invalid())?;
    let cipher = xts_mode::Xts128::new(first, second);
    if inverse {
        cipher.decrypt_sector(data, s.initial_block.into());
    } else {
        cipher.encrypt_sector(data, s.initial_block.into());
    }
    Ok(())
}
