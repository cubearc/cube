use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn cctv_x25519_file_authenticates_before_output() {
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/age/vectors.json")).unwrap();
    let v = &rows[0];
    let result = execution::execute(
        "cipher.age.decrypt",
        1,
        &json!({"input":{"base64":v["ciphertext_base64"]}}),
        &v["parameters"],
        Budget::default(),
    )
    .unwrap();
    assert_eq!(result, json!({"output":{"base64":v["plaintext_base64"]}}));
}
#[test]
fn all_cctv_vectors_match_independent_go_success_and_failure_outcomes() {
    use cube_core::ciphers::age;
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/age/vectors.json")).unwrap();
    for v in rows {
        let s = age::Settings::parse(&v["parameters"].to_string()).unwrap();
        let input =
            cube_core::encoding::base64_decode(v["ciphertext_base64"].as_str().unwrap()).unwrap();
        assert_eq!(cube_core::hashing::sha256(&input), v["ciphertext_sha256"]);
        let result = age::decrypt(&input, &s);
        if v["expect"] == "success" {
            let output = result.unwrap_or_else(|e| panic!("{} {e}", v["case"]));
            assert_eq!(cube_core::hashing::sha256(&output), v["plaintext_sha256"]);
            assert_eq!(
                cube_core::encoding::base64_encode(&output),
                v["plaintext_base64"]
            );
        } else {
            assert!(result.is_err(), "{}", v["case"]);
        }
    }
}
#[test]
fn preflight_rejects_hostile_kdf_headers_stanzas_and_private_settings() {
    use cube_core::{ciphers::age, errors::ErrorCode};
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/age/vectors.json")).unwrap();
    let p = &rows[0]["parameters"];
    let s = age::Settings::parse(&p.to_string()).unwrap();
    assert!(!format!("{s:?}").contains("AGE-SECRET-KEY-1"));
    assert_eq!(
        age::decrypt(&vec![0; age::MAX_INPUT + 1], &s)
            .unwrap_err()
            .code,
        ErrorCode::ResourceLimit
    );
    let mut oversized = b"age-encryption.org/v1\n".to_vec();
    oversized.resize(age::MAX_HEADER + 1, b'A');
    assert_eq!(
        age::decrypt(&oversized, &s).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let mut hostile = b"age-encryption.org/v1\n-> scrypt AAAAAAAAAAAAAAAAAAAAAA 99\n".to_vec();
    hostile.extend_from_slice(b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n--- AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n");
    hostile.extend([0; 32]);
    assert_eq!(
        age::decrypt(&hostile, &s).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let mut nine = b"age-encryption.org/v1\n".to_vec();
    for _ in 0..9 {
        nine.extend_from_slice(b"-> X25519 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\nAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n");
    }
    nine.extend_from_slice(b"--- AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n");
    nine.extend([0; 32]);
    assert_eq!(
        age::decrypt(&nine, &s).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for document in [
        r#"{"profile":"age_v1_x25519_binary","identity":"SECRET"}"#,
        r#"{"profile":"other","identity":"SECRET"}"#,
    ] {
        let e = age::Settings::parse(document).unwrap_err();
        assert!(!format!("{e:?}").contains("SECRET"));
    }
    let mut p = p.clone();
    p["identity"] = json!(p["identity"].as_str().unwrap().to_lowercase());
    assert!(age::Settings::parse(&p.to_string()).is_err());
    p = rows[0]["parameters"].clone();
    p["unknown"] = json!(true);
    assert!(age::Settings::parse(&p.to_string()).is_err());
}
#[test]
fn exact_ciphertext_ceiling_authenticates_every_chunk_before_return() {
    use cube_core::ciphers::age as bounded;
    use std::io::Write;
    let rows: Vec<serde_json::Value> =
        serde_json::from_str(include_str!("../src/ciphers/age/vectors.json")).unwrap();
    let p = &rows[0]["parameters"];
    let settings = bounded::Settings::parse(&p.to_string()).unwrap();
    let identity: age::x25519::Identity = p["identity"].as_str().unwrap().parse().unwrap();
    let recipient = identity.to_public();
    use std::{cell::RefCell, rc::Rc};
    struct Sink(Rc<RefCell<Vec<u8>>>);
    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.borrow_mut().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let buffer = Rc::new(RefCell::new(Vec::new()));
    let encryptor =
        age::Encryptor::with_recipients(std::iter::once(&recipient as &dyn age::Recipient))
            .unwrap();
    let mut writer = encryptor.wrap_output(Sink(buffer.clone())).unwrap();
    let overhead = buffer.borrow().len();
    let input = vec![0x42; bounded::MAX_INPUT - overhead - 256 * 16];
    writer.write_all(&input).unwrap();
    writer.finish().unwrap();
    let mut ciphertext = buffer.borrow_mut().split_off(0);
    assert_eq!(ciphertext.len(), bounded::MAX_INPUT);
    assert_eq!(bounded::decrypt(&ciphertext, &settings).unwrap(), input);
    let last = ciphertext.len() - 1;
    ciphertext[last] ^= 1;
    assert!(bounded::decrypt(&ciphertext, &settings).is_err());
    ciphertext.push(0);
    assert_eq!(
        bounded::decrypt(&ciphertext, &settings).unwrap_err().code,
        cube_core::errors::ErrorCode::ResourceLimit
    );
}
