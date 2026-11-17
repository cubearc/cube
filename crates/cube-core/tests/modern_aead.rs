use cube_core::operations::execution::{self, Budget};
use serde_json::json;
#[test]
fn aes_gcm_authenticates_empty_message_and_aad_with_detached_tag() {
    let p = json!({"profile":"aes128_gcm","key_hex":"00".repeat(16),"nonce_hex":"00".repeat(12)});
    let empty = json!({"base64":""});
    let actual = execution::execute(
        "cipher.aead.encrypt",
        1,
        &json!({"input":empty,"aad":empty}),
        &p,
        Budget::default(),
    )
    .unwrap();
    assert_eq!(
        actual,
        json!({"output":empty,"tag":{"base64":cube_core::encoding::base64_encode(&hex::decode("58e2fccefa7e3061367f1d57a4e7455a").unwrap())}})
    );
}
#[test]
fn rfc5297_nonce_free_siv_preserves_component_boundaries() {
    let p = json!({"profile":"aes128_siv_cmac_nonce_free","key_hex":"fffefdfcfbfaf9f8f7f6f5f4f3f2f1f0f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff"});
    let input = hex::decode("112233445566778899aabbccddee").unwrap();
    let aad = hex::decode("101112131415161718191a1b1c1d1e1f2021222324252627").unwrap();
    let result=execution::execute("cipher.aes.siv.encrypt",1,&json!({"input":{"base64":cube_core::encoding::base64_encode(&input)},"aad_components":[aad]}),&p,Budget::default()).unwrap();
    assert_eq!(
        result,
        json!({"output":{"base64":cube_core::encoding::base64_encode(&hex::decode("40c02b9690c4dc04daef7f6afe5c").unwrap())},"tag":{"base64":cube_core::encoding::base64_encode(&hex::decode("85632d07c6e8f37f950acd320a2ecc93").unwrap())}})
    );
}

fn vectors(siv: bool) -> Vec<serde_json::Value> {
    serde_json::from_str(if siv {
        include_str!("../src/ciphers/modern_aead/siv_vectors.json")
    } else {
        include_str!("../src/ciphers/modern_aead/vectors.json")
    })
    .unwrap()
}
fn decode(value: &serde_json::Value) -> Vec<u8> {
    hex::decode(value.as_str().unwrap()).unwrap()
}
fn bytes(value: &[u8]) -> serde_json::Value {
    json!({"base64":cube_core::encoding::base64_encode(value)})
}
#[test]
fn every_exact_aead_profile_executes_independent_ciphertext_tag_answers() {
    use cube_core::ciphers::modern_aead as aead;
    let vectors = vectors(false);
    let found: std::collections::BTreeSet<_> = vectors
        .iter()
        .map(|v| v["parameters"]["profile"].as_str().unwrap())
        .collect();
    assert_eq!(found, aead::Profile::NAMES.iter().copied().collect());
    for v in &vectors {
        let p = &v["parameters"];
        let s = aead::Settings::parse(&p.to_string()).unwrap();
        let plain = decode(&v["plaintext_hex"]);
        let aad = decode(&v["aad_hex"]);
        let ct = decode(&v["ciphertext_hex"]);
        let tag = decode(&v["tag_hex"]);
        let result = aead::encrypt(&plain, &aad, &s).unwrap();
        assert_eq!(result.ciphertext, ct, "{p} {} ", v["source_file"]);
        assert_eq!(result.tag, tag, "{p}");
        assert_eq!(aead::decrypt(&ct, &aad, &tag, &s).unwrap(), plain);
        assert_eq!(
            execution::execute(
                aead::ENCRYPT,
                1,
                &json!({"input":bytes(&plain),"aad":bytes(&aad)}),
                p,
                Budget::default()
            )
            .unwrap(),
            json!({"output":bytes(&ct),"tag":bytes(&tag)})
        );
        assert_eq!(
            execution::execute(
                aead::DECRYPT,
                1,
                &json!({"input":bytes(&ct),"aad":bytes(&aad),"tag":bytes(&tag)}),
                p,
                Budget::default()
            )
            .unwrap(),
            json!({"output":bytes(&plain)})
        );
        let mut forged = tag.clone();
        forged[0] ^= 1;
        assert_eq!(
            aead::decrypt(&ct, &aad, &forged, &s).unwrap_err().message,
            "authentication failed"
        );
    }
}
#[test]
fn siv_independent_answers_authenticate_ordered_components() {
    use cube_core::ciphers::modern_aead as aead;
    let vectors = vectors(true);
    let found: std::collections::BTreeSet<_> = vectors
        .iter()
        .map(|v| v["parameters"]["profile"].as_str().unwrap())
        .collect();
    assert_eq!(found, aead::SivProfile::NAMES.iter().copied().collect());
    for v in vectors {
        let p = &v["parameters"];
        let s = aead::SivSettings::parse(&p.to_string()).unwrap();
        let plain = decode(&v["plaintext_hex"]);
        let ct = decode(&v["ciphertext_hex"]);
        let tag = decode(&v["tag_hex"]);
        let components: Vec<_> = v["aad_components_hex"]
            .as_array()
            .unwrap()
            .iter()
            .map(decode)
            .collect();
        let result = aead::siv_encrypt(&plain, &components, &s).unwrap();
        assert_eq!(result.ciphertext, ct);
        assert_eq!(result.tag, tag);
        assert_eq!(
            aead::siv_decrypt(&ct, &components, &tag, &s).unwrap(),
            plain
        );
        assert_eq!(
            execution::execute(
                aead::SIV_ENCRYPT,
                1,
                &json!({"input":bytes(&plain),"aad_components":components}),
                p,
                Budget::default()
            )
            .unwrap(),
            json!({"output":bytes(&ct),"tag":bytes(&tag)})
        );
        assert_eq!(
            execution::execute(
                aead::SIV_DECRYPT,
                1,
                &json!({"input":bytes(&ct),"aad_components":components,"tag":bytes(&tag)}),
                p,
                Budget::default()
            )
            .unwrap(),
            json!({"output":bytes(&plain)})
        );
        let mut forged = tag.clone();
        forged[0] ^= 1;
        assert!(aead::siv_decrypt(&ct, &components, &forged, &s).is_err());
    }
}

#[test]
fn all_profiles_reject_changed_ciphertext_aad_nonce_key_and_tag_without_plaintext() {
    use cube_core::ciphers::modern_aead as aead;
    let vectors = vectors(false);
    for profile in aead::Profile::NAMES {
        let v = vectors
            .iter()
            .find(|v| {
                v["parameters"]["profile"] == *profile
                    && !v["plaintext_hex"].as_str().unwrap().is_empty()
            })
            .unwrap();
        let p = &v["parameters"];
        let s = aead::Settings::parse(&p.to_string()).unwrap();
        let ct = decode(&v["ciphertext_hex"]);
        let aad = decode(&v["aad_hex"]);
        let tag = decode(&v["tag_hex"]);
        let mut bad = ct.clone();
        bad[0] ^= 1;
        assert_eq!(
            aead::decrypt(&bad, &aad, &tag, &s).unwrap_err().message,
            "authentication failed"
        );
        let mut changed_aad = aad.clone();
        changed_aad.push(1);
        assert!(aead::decrypt(&ct, &changed_aad, &tag, &s).is_err());
        for name in ["key_hex", "nonce_hex"] {
            let mut p = p.clone();
            let mut bytes = decode(&p[name]);
            bytes[0] ^= 1;
            p[name] = json!(hex::encode(bytes));
            let s = aead::Settings::parse(&p.to_string()).unwrap();
            assert_eq!(
                aead::decrypt(&ct, &aad, &tag, &s).unwrap_err().message,
                "authentication failed",
                "{profile}"
            );
        }
        for length in [0, tag.len() - 1, tag.len() + 1] {
            assert_eq!(
                aead::decrypt(&ct, &aad, &vec![0; length], &s)
                    .unwrap_err()
                    .message,
                "authentication failed"
            );
        }
        let mut bad_tag = tag.clone();
        bad_tag[0] ^= 1;
        assert!(
            execution::execute(
                aead::DECRYPT,
                1,
                &json!({"input":bytes(&ct),"aad":bytes(&aad),"tag":bytes(&bad_tag)}),
                p,
                Budget::default()
            )
            .is_err()
        );
    }
}
#[test]
fn nonce_free_siv_distinguishes_component_count_order_and_boundaries() {
    use cube_core::ciphers::modern_aead as aead;
    for profile in aead::SivProfile::NAMES {
        let size = if profile.starts_with("aes128") {
            32
        } else {
            64
        };
        let p = json!({"profile":profile,"key_hex":"42".repeat(size)});
        let s = aead::SivSettings::parse(&p.to_string()).unwrap();
        let parts = vec![vec![1, 2], vec![3]];
        let plain = b"exact message";
        let encrypted = aead::siv_encrypt(plain, &parts, &s).unwrap();
        for components in [
            vec![vec![3], vec![1, 2]],
            vec![vec![1, 2, 3]],
            vec![vec![1], vec![2, 3]],
            vec![],
        ] {
            assert!(
                aead::siv_decrypt(&encrypted.ciphertext, &components, &encrypted.tag, &s).is_err()
            );
        }
        let no_components = aead::siv_encrypt(&[], &[], &s).unwrap();
        let one_empty = aead::siv_encrypt(&[], &[vec![]], &s).unwrap();
        assert_ne!(no_components.tag, one_empty.tag);
        let max = vec![vec![]; aead::MAX_AAD_COMPONENTS];
        let result = aead::siv_encrypt(&[], &max, &s).unwrap();
        assert_eq!(
            aead::siv_decrypt(&result.ciphertext, &max, &result.tag, &s).unwrap(),
            Vec::<u8>::new()
        );
        assert!(aead::siv_encrypt(&[], &vec![vec![]; aead::MAX_AAD_COMPONENTS + 1], &s).is_err());
        let mut p = p.clone();
        p["nonce_hex"] = json!("00".repeat(16));
        assert!(aead::SivSettings::parse(&p.to_string()).is_err());
    }
}
#[test]
fn settings_are_private_and_all_exact_profile_lengths_match_derived_schemas() {
    use cube_core::ciphers::modern_aead as aead;
    for siv in [false, true] {
        let op = if siv {
            aead::SIV_ENCRYPT
        } else {
            aead::ENCRYPT
        };
        let contract = cube_core::operations::require(op)
            .unwrap()
            .contract()
            .unwrap();
        assert!(
            contract.policy.private_input
                && contract.policy.private_output
                && !contract.policy.automatic_capture
        );
        assert!(contract.pipeline_adapter.is_none());
        for v in vectors(siv) {
            let good = &v["parameters"];
            contract.validate_parameters(good).unwrap();
            for name in if siv {
                &["profile", "key_hex"][..]
            } else {
                &["profile", "key_hex", "nonce_hex"][..]
            } {
                let mut p = good.clone();
                p.as_object_mut().unwrap().remove(*name);
                assert!(contract.validate_parameters(&p).is_err());
            }
            for change in [
                json!({"key_hex":"SECRET"}),
                json!({"profile":"unknown"}),
                json!({"extra":"SECRET"}),
                json!({"nonce_hex":"00"}),
            ] {
                let mut p = good.clone();
                p.as_object_mut()
                    .unwrap()
                    .extend(change.as_object().unwrap().clone());
                assert!(contract.validate_parameters(&p).is_err());
                let error = if siv {
                    aead::SivSettings::parse(&p.to_string()).unwrap_err()
                } else {
                    aead::Settings::parse(&p.to_string()).unwrap_err()
                };
                assert!(!format!("{error:?}").contains("SECRET"));
            }
        }
    }
    let p = json!({"profile":"aes128_gcm","key_hex":"42".repeat(16),"nonce_hex":"24".repeat(12)});
    let s = aead::Settings::parse(&p.to_string()).unwrap();
    assert!(!format!("{s:?}").contains("424242"));
    let mut doc = p.to_string();
    doc.push_str(&" ".repeat(aead::MAX_PARAMETERS - doc.len()));
    aead::Settings::parse(&doc).unwrap();
    doc.push(' ');
    assert!(aead::Settings::parse(&doc).is_err());
    assert!(
        aead::Settings::parse(
            r#"{"profile":"aes128_gcm","key_hex":"SECRET","key_hex":"SECRET","nonce_hex":"SECRET"}"#
        )
        .is_err()
    );
}
#[test]
fn combined_body_aad_and_ccm_lengths_fail_at_exact_boundaries() {
    use cube_core::{ciphers::modern_aead as aead, errors::ErrorCode};
    let p = json!({"profile":"aes128_ccm_nonce13_tag8","key_hex":"42".repeat(16),"nonce_hex":"24".repeat(13)});
    let s = aead::Settings::parse(&p.to_string()).unwrap();
    let plain = vec![0x42; 65535];
    let encrypted = aead::encrypt(&plain, &[], &s).unwrap();
    assert_eq!(
        aead::decrypt(&encrypted.ciphertext, &[], &encrypted.tag, &s).unwrap(),
        plain
    );
    assert_eq!(
        aead::encrypt(&vec![0; 65536], &[], &s).err().unwrap().code,
        ErrorCode::ResourceLimit
    );
    let p = json!({"profile":"aes128_gcm","key_hex":"42".repeat(16),"nonce_hex":"24".repeat(12)});
    let s = aead::Settings::parse(&p.to_string()).unwrap();
    let body = vec![0x42; aead::MAX_INPUT];
    let encrypted = aead::encrypt(&body, &[], &s).unwrap();
    assert_eq!(
        aead::decrypt(&encrypted.ciphertext, &[], &encrypted.tag, &s).unwrap(),
        body
    );
    assert!(aead::encrypt(&body, &[0], &s).is_err());
    assert!(aead::encrypt(&[0], &body, &s).is_err());
    assert!(aead::decrypt(&vec![0; aead::MAX_INPUT + 1], &[], &[0; 16], &s).is_err());
    let p = json!({"profile":"aes128_ccm_nonce12_tag16","key_hex":"42".repeat(16),"nonce_hex":"24".repeat(12)});
    let s = aead::Settings::parse(&p.to_string()).unwrap();
    assert!(aead::encrypt(&body, &[], &s).is_err());
}
