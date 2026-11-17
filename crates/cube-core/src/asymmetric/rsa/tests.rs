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
fn failed_entropy_returns_no_ciphertext_or_private_result() {
    let v: serde_json::Value = serde_json::from_str(include_str!("interoperability.json")).unwrap();
    let v = &v[0];
    let public = EncryptionSettings::parse(
        &v["public_parameters"].to_string(),
        &hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    let private = DecryptionSettings::parse(
        &v["private_parameters"].to_string(),
        &hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    let ct = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    assert_eq!(
        encrypt_with_rng(b"message", &public, &mut FailedEntropy)
            .unwrap_err()
            .message,
        "RSA operation failed"
    );
    assert_eq!(
        decrypt_with_rng(&ct, &private, &mut FailedEntropy)
            .unwrap_err()
            .message,
        "RSA operation failed"
    );
    let legacy = LegacySettings {
        key: private.key.clone(),
    };
    assert_eq!(
        legacy_decrypt_with_rng(&ct, &legacy, &mut FailedEntropy)
            .unwrap_err()
            .message,
        "RSA operation failed"
    );
}
