use super::*;
use rand_core::TryRng;
use serde_json::{Value, json};

struct TestEntropy<'a>(&'a [u8]);

impl TryRng for TestEntropy<'_> {
    type Error = super::super::entropy::EntropyError;

    fn try_fill_bytes(&mut self, target: &mut [u8]) -> Result<(), Self::Error> {
        if self.0.len() != target.len() {
            return Err(super::super::entropy::EntropyError);
        }
        target.copy_from_slice(self.0);
        self.0 = &[];
        Ok(())
    }

    fn try_next_u32(&mut self) -> Result<u32, Self::Error> {
        Err(super::super::entropy::EntropyError)
    }

    fn try_next_u64(&mut self) -> Result<u64, Self::Error> {
        Err(super::super::entropy::EntropyError)
    }
}
impl TryCryptoRng for TestEntropy<'_> {}

fn vectors() -> Vec<Value> {
    let mut payload: Value = serde_json::from_str(include_str!("vectors.json")).unwrap();
    serde_json::from_value(payload["profiles"].take()).unwrap()
}

fn raw(value: &Value, field: &str) -> Vec<u8> {
    hex::decode(value[field].as_str().unwrap()).unwrap()
}

fn profile(v: &Value) -> Profile {
    Profile::parse(&json!({"profile":v["profile"]}).to_string()).unwrap()
}

#[test]
fn acvp_key_generation_and_encapsulation_known_answers() {
    for v in vectors() {
        let profile = profile(&v);
        let g = &v["keygen"];
        let seed = [raw(g, "d"), raw(g, "z")].concat();
        let pair = generate(profile, &mut TestEntropy(&seed)).unwrap();
        assert!(
            pair.public_key == raw(g, "ek") && *pair.private_key == raw(g, "dk"),
            "ACVP generation case {}",
            g["tcId"]
        );

        let e = &v["encapsulation"];
        let enc =
            encapsulate_with_rng(&raw(e, "ek"), profile, &mut TestEntropy(&raw(e, "m"))).unwrap();
        assert!(
            enc.ciphertext == raw(e, "c") && enc.shared_secret[..] == raw(e, "k"),
            "ACVP encapsulation case {}",
            e["tcId"]
        );
    }
}

#[test]
fn malformed_encodings_and_missing_entropy_fail() {
    for v in vectors() {
        let profile = profile(&v);
        let e = &v["encapsulation"];
        let mut public = raw(e, "ek");
        assert_eq!(
            encapsulate_with_rng(&public, profile, &mut TestEntropy(&[]))
                .err()
                .unwrap()
                .code,
            ErrorCode::Io
        );
        public[0] = 0xff;
        public[1] |= 0x0f;
        assert_eq!(
            encapsulate_with_rng(&public, profile, &mut TestEntropy(&[]))
                .err()
                .unwrap()
                .code,
            ErrorCode::InvalidInput
        );

        let mut private = raw(e, "dk");
        let index = private.len() - 64;
        private[index] ^= 1;
        let ciphertext = raw(e, "c");
        assert_eq!(
            decapsulate(&ciphertext, &private, profile)
                .err()
                .unwrap()
                .code,
            ErrorCode::InvalidInput
        );
        assert!(decapsulate(&ciphertext[1..], &raw(e, "dk"), profile).is_err());
        assert!(decapsulate(&ciphertext, &[0; 64], profile).is_err());
        assert_eq!(
            generate(profile, &mut TestEntropy(&[])).err().unwrap().code,
            ErrorCode::Io
        );
    }
}
