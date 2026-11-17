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
fn entropy_failure_never_returns_a_ciphertext() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let v = &rows[1];
    let s = ElGamalEncryptionSettings::parse(
        &v["parameters"].to_string(),
        &hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    let input = hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap();
    assert!(elgamal_with_rng(&input, &s, &mut FailedEntropy).is_err());
    let v = &rows[5];
    let s = CramerShoupEncryptionSettings::parse(
        &v["parameters"].to_string(),
        &hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    assert!(cramer_shoup_with_rng(&input, &s, &mut FailedEntropy).is_err());
}

struct ZeroEntropy {
    calls: usize,
}
impl TryRng for ZeroEntropy {
    type Error = super::super::entropy::EntropyError;
    fn try_fill_bytes(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.calls += 1;
        bytes.fill(0);
        Ok(())
    }
    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Ok(0)
    }
    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Ok(0)
    }
}
impl TryCryptoRng for ZeroEntropy {}
#[test]
fn degenerate_entropy_exhausts_the_fixed_attempt_budget_without_ciphertext() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let v = &rows[1];
    let s = ElGamalEncryptionSettings::parse(
        &v["parameters"].to_string(),
        &hex::decode(v["public_key_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    let mut rng = ZeroEntropy { calls: 0 };
    assert!(
        elgamal_with_rng(
            &hex::decode(v["plaintext_hex"].as_str().unwrap()).unwrap(),
            &s,
            &mut rng
        )
        .is_err()
    );
    assert_eq!(rng.calls, 8);
}
#[test]
fn valid_group_malleation_changes_elgamal_plaintext_but_fails_cramer_shoup_authentication() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let g = group();
    let v = &rows[1];
    let mut cipher = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    let e = g.element(&cipher[WIDTH..], true).unwrap();
    let changed = &e * &g.g1;
    cipher[WIDTH..].copy_from_slice(&encode(&changed));
    let s = ElGamalDecryptionSettings::parse(
        &v["parameters"].to_string(),
        &hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    let mut four = vec![0; WIDTH];
    four[WIDTH - 1] = 4;
    assert_eq!(elgamal_decrypt(&cipher, &s).unwrap(), four);
    let v = &rows[5];
    let mut cipher = hex::decode(v["ciphertext_hex"].as_str().unwrap()).unwrap();
    let e = g.element(&cipher[2 * WIDTH..3 * WIDTH], true).unwrap();
    cipher[2 * WIDTH..3 * WIDTH].copy_from_slice(&encode(&(&e * &g.g1)));
    let s = CramerShoupDecryptionSettings::parse(
        &v["parameters"].to_string(),
        &hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        cramer_shoup_decrypt(&cipher, &s).unwrap_err().message,
        "group encryption failed"
    );
}
#[test]
fn exact_scalar_order_boundary_and_degenerate_public_keys_are_refused() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    let q = group().order.to_be_bytes();
    assert!(ElGamalDecryptionSettings::parse(&rows[1]["parameters"].to_string(), &q).is_err());
    let mut public = hex::decode(rows[5]["public_key_hex"].as_str().unwrap()).unwrap();
    public[..WIDTH].fill(0);
    public[WIDTH - 1] = 1;
    assert!(
        CramerShoupEncryptionSettings::parse(&rows[5]["parameters"].to_string(), &public).is_err()
    );
    let mut p = rows[5]["parameters"].clone();
    p["label_hex"] = serde_json::json!("00".repeat(MAX_LABEL));
    let private = hex::decode(rows[5]["private_key_hex"].as_str().unwrap()).unwrap();
    CramerShoupDecryptionSettings::parse(&p.to_string(), &private).unwrap();
    p["label_hex"] = serde_json::json!("00".repeat(MAX_LABEL + 1));
    assert!(CramerShoupDecryptionSettings::parse(&p.to_string(), &private).is_err());
}
