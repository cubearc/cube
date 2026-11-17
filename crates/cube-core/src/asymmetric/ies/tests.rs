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
    for v in v.as_array().unwrap() {
        let s = EncryptionSettings::parse(
            &v["parameters"].to_string(),
            &hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            encrypt_with_rng(b"message", &s, &mut FailedEntropy)
                .unwrap_err()
                .message,
            "IES operation failed"
        );
    }
}
