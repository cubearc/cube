//! In-place maintained backends; the owning caller wipes failed plaintext.
use super::*;
use aes_gcm::aead::{AeadInOut, KeyInit, Nonce, Tag};
pub(super) fn standard<A: AeadInOut + KeyInit>(
    s: &Settings,
    aad: &[u8],
    buffer: &mut [u8],
    tag: Option<&[u8]>,
) -> Result<Vec<u8>, OperationError> {
    let cipher = A::new_from_slice(&s.key).map_err(|_| invalid())?;
    let nonce = Nonce::<A>::try_from(s.nonce.as_slice()).map_err(|_| invalid())?;
    match tag {
        None => cipher
            .encrypt_inout_detached(&nonce, aad, buffer.into())
            .map(|tag| tag.to_vec())
            .map_err(|_| authentication_failed()),
        Some(tag) => {
            let tag = Tag::<A>::try_from(tag).map_err(|_| authentication_failed())?;
            cipher
                .decrypt_inout_detached(&nonce, aad, buffer.into(), &tag)
                .map_err(|_| authentication_failed())?;
            Ok(Vec::new())
        }
    }
}
pub(super) fn legacy_eax<A: eax::aead::AeadInPlace + eax::aead::KeyInit>(
    s: &Settings,
    aad: &[u8],
    buffer: &mut [u8],
    tag: Option<&[u8]>,
) -> Result<Vec<u8>, OperationError> {
    let cipher = A::new_from_slice(&s.key).map_err(|_| invalid())?;
    let nonce: &eax::aead::Nonce<A> = s.nonce.as_slice().into();
    match tag {
        None => cipher
            .encrypt_in_place_detached(nonce, aad, buffer)
            .map(|tag| tag.to_vec())
            .map_err(|_| authentication_failed()),
        Some(tag) => {
            let tag: &eax::aead::Tag<A> = tag.into();
            cipher
                .decrypt_in_place_detached(nonce, aad, buffer, tag)
                .map_err(|_| authentication_failed())?;
            Ok(Vec::new())
        }
    }
}
