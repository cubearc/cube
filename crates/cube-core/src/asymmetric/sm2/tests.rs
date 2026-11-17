use super::*;
use rand_core::TryRng;
struct FailedEntropy;
impl TryRng for FailedEntropy {
    type Error = super::super::entropy::EntropyError;
    fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), Self::Error> {
        Err(super::super::entropy::EntropyError)
    }
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Err(super::super::entropy::EntropyError)
    }
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Err(super::super::entropy::EntropyError)
    }
}
impl TryCryptoRng for FailedEntropy {}
#[test]
fn entropy_failure_returns_no_ciphertext() {
    let v: serde_json::Value = serde_json::from_str(include_str!("interoperability.json")).unwrap();
    let v = &v[0];
    let public = EncryptionSettings::parse(
        r#"{"profile":"sm2_sm3_c1c3c2"}"#,
        &hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        encrypt_with_rng(b"message", &public, &mut FailedEntropy)
            .unwrap_err()
            .message,
        "SM2 operation failed"
    );
}
#[test]
fn all_zero_kdf_is_rejected_including_single_byte_and_partial_blocks() {
    for length in [1, 31, 32, 33, 65] {
        let input = vec![7; length];
        let mut out = vec![0; length];
        assert!(!kdf_xor(&input, &mut out, |_| [0; 32]));
        assert_eq!(out, input);
        let mut counters = Vec::new();
        assert!(kdf_xor(&input, &mut out, |c| {
            counters.push(c);
            [1; 32]
        }));
        assert_eq!(out, vec![6; length]);
        assert_eq!(
            counters,
            (1..=length.div_ceil(32) as u32).collect::<Vec<_>>()
        );
    }
}
