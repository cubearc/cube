//! Exact raw stream profiles for offline interoperability. All transforms are
//! unauthenticated and malleable. Callers must prevent key/nonce/position reuse.
mod catalog;
use crate::errors::{ErrorCode, OperationError};
pub(crate) use catalog::MODULE;
pub use catalog::OPERATIONS;
use chacha20::cipher::{Block, KeyIvInit, StreamCipherSeekCore, consts::U64};
use zeroize::{Zeroize, Zeroizing};

const CHACHA8_ID: &str = "cipher.chacha8";
const CHACHA12_ID: &str = "cipher.chacha12";
const CHACHA20_ID: &str = "cipher.chacha20";
const XCHACHA20_ID: &str = "cipher.xchacha20";
const SALSA12_ID: &str = "cipher.salsa20_12";
const SALSA20_ID: &str = "cipher.salsa20";
const VERNAM_ID: &str = "cipher.vernam";

pub const MAX_INPUT: usize = crate::binary::transform::MAX_INPUT;

/// Profiles fix rounds, nonce layout and block-counter width. Reduced rounds
/// are experimental interoperability profiles, not secure-encryption defaults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Profile {
    ChaCha8,
    ChaCha12,
    ChaCha20,
    XChaCha20,
    Salsa12,
    Salsa20,
}
impl Profile {
    pub const ALL: [Self; 6] = [
        Self::ChaCha8,
        Self::ChaCha12,
        Self::ChaCha20,
        Self::XChaCha20,
        Self::Salsa12,
        Self::Salsa20,
    ];
    pub const fn id(self) -> &'static str {
        match self {
            Self::ChaCha8 => CHACHA8_ID,
            Self::ChaCha12 => CHACHA12_ID,
            Self::ChaCha20 => CHACHA20_ID,
            Self::XChaCha20 => XCHACHA20_ID,
            Self::Salsa12 => SALSA12_ID,
            Self::Salsa20 => SALSA20_ID,
        }
    }
    pub const fn nonce_bytes(self) -> usize {
        match self {
            Self::XChaCha20 => 24,
            Self::Salsa12 | Self::Salsa20 => 8,
            _ => 12,
        }
    }
    pub const fn max_counter(self) -> u64 {
        match self {
            Self::Salsa12 | Self::Salsa20 => u64::MAX,
            _ => u32::MAX as u64,
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }
}
fn invalid(
    p: Profile,
    parameter: &'static str,
    code: ErrorCode,
    message: &'static str,
) -> OperationError {
    OperationError::new(code, message)
        .operation(p.id())
        .parameter(parameter)
}
fn preflight(
    p: Profile,
    input: &[u8],
    key: &[u8],
    nonce: &[u8],
    counter: u64,
) -> Result<(), OperationError> {
    if input.len() > MAX_INPUT {
        return Err(invalid(
            p,
            "input",
            ErrorCode::ResourceLimit,
            "input exceeds 16 MiB",
        ));
    }
    if key.len() != 32 {
        return Err(invalid(
            p,
            "key",
            ErrorCode::InvalidParameter,
            "key must contain exactly 32 bytes",
        ));
    }
    if nonce.len() != p.nonce_bytes() {
        return Err(invalid(
            p,
            "nonce",
            ErrorCode::InvalidParameter,
            "nonce length does not match the selected profile",
        ));
    }
    let blocks = input.len().div_ceil(64) as u64;
    if counter > p.max_counter()
        || blocks.saturating_sub(1) > p.max_counter().saturating_sub(counter)
    {
        return Err(invalid(
            p,
            "counter",
            ErrorCode::InvalidParameter,
            "block counter would exhaust the selected profile",
        ));
    }
    Ok(())
}
// Preflight proves all generated blocks fit, including the final full or partial
// block. Local backend state is dropped after this one call, never reused after
// its internal counter wraps. No byte-position multiplication can overflow.
fn apply<C: StreamCipherSeekCore<BlockSize = U64> + KeyIvInit>(
    input: &[u8],
    key: &[u8],
    nonce: &[u8],
    counter: C::Counter,
) -> Vec<u8> {
    let mut core =
        C::new_from_slices(key, nonce).expect("preflight checked fixed key/nonce lengths");
    core.set_block_pos(counter);
    let mut out = input.to_vec();
    let mut block = Block::<C>::default();
    for chunk in out.chunks_mut(64) {
        core.write_keystream_block(&mut block);
        for (byte, mask) in chunk.iter_mut().zip(block.iter()) {
            *byte ^= mask;
        }
        block.as_mut_slice().zeroize();
    }
    out
}

/// Transform up to 16 MiB with an explicit 32-byte key, exact nonce and initial
/// block counter. Applying again with identical arguments restores the input.
/// Validate even empty input; reject counter exhaustion before allocation. No
/// implicit nonce, random state, padding, authentication or pipeline capture.
/// Backend key state and temporary keystream are zeroized; caller buffers and
/// returned bytes remain the caller's responsibility. RustCrypto Salsa is not
/// independently audited; see the stream reference for qualification limits.
pub fn transform(
    p: Profile,
    input: &[u8],
    key: &[u8],
    nonce: &[u8],
    counter: u64,
) -> Result<Vec<u8>, OperationError> {
    preflight(p, input, key, nonce, counter)?;
    use chacha20::cipher::consts::{U6, U10};
    use chacha20::{ChaChaCore, R8, R12, R20, variants::Ietf};
    Ok(match p {
        Profile::ChaCha8 => apply::<ChaChaCore<R8, Ietf>>(input, key, nonce, counter as u32),
        Profile::ChaCha12 => apply::<ChaChaCore<R12, Ietf>>(input, key, nonce, counter as u32),
        Profile::ChaCha20 => apply::<ChaChaCore<R20, Ietf>>(input, key, nonce, counter as u32),
        Profile::XChaCha20 => {
            // Reuse the backend's HChaCha primitive and its documented IETF
            // layout: zero word followed by the last eight nonce bytes.
            let subkey = Zeroizing::new(chacha20::hchacha::<R20>(
                key.try_into().expect("checked key"),
                nonce[..16].try_into().expect("checked nonce"),
            ));
            let mut iv = [0; 12];
            iv[4..].copy_from_slice(&nonce[16..]);
            apply::<ChaChaCore<R20, Ietf>>(input, subkey.as_slice(), &iv, counter as u32)
        }
        Profile::Salsa12 => apply::<salsa20::SalsaCore<U6>>(input, key, nonce, counter),
        Profile::Salsa20 => apply::<salsa20::SalsaCore<U10>>(input, key, nonce, counter),
    })
}

/// Educational Vernam: reuse exact-length XOR, with <=16 MiB per operand.
/// Empty/empty is identity. An actual one-time pad requires a uniform,
/// independent pad, delivered securely and used once; this API cannot guarantee
/// any of those conditions. It never generates, stores or tracks pads.
pub fn vernam(input: &[u8], pad: &[u8]) -> Result<Vec<u8>, OperationError> {
    crate::binary::transform::xor_exact(input, pad).map_err(|e| e.operation(VERNAM_ID))
}
