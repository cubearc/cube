use cube_core::operations::execution::{self, Budget};
use serde_json::json;

#[test]
fn rfc7748_x25519_exact_shared_secret_executes() {
    let peer =
        hex::decode("de9edb7d7b7dc1b4d35b61c2ece435373f8343c85b78674dadfc7e146f882b4f").unwrap();
    let result = execution::execute(
        "crypto.ecdh.agree", 1,
        &json!({"input":{"base64":cube_core::encoding::base64_encode(&peer)}}),
        &json!({"profile":"x25519_rfc7748","private_key_hex":"77076d0a7318a57d3c16c17251b26645df4c2f87ebc0992ab177fba51db92c2a"}),
        Budget::default(),
    ).unwrap();
    let shared =
        hex::decode("4a5d9d5ba4ce2de1728e3bf480350f25e07e21c947d19e3376f09b3c1e161742").unwrap();
    assert_eq!(
        result,
        json!({"output":{"base64":cube_core::encoding::base64_encode(&shared)}})
    );
}

fn vectors() -> Vec<serde_json::Value> {
    serde_json::from_str(include_str!("../src/asymmetric/ecdh/vectors.json")).unwrap()
}

#[test]
fn independent_rfc7748_and_rfc5903_points_match_raw_secrets_and_public_keys() {
    use cube_core::asymmetric::ecdh;
    for v in vectors() {
        let s = ecdh::Settings::parse(&v["parameters"].to_string()).unwrap();
        assert_eq!(
            ecdh::agree(&hex::decode(v["peer_hex"].as_str().unwrap()).unwrap(), &s).unwrap(),
            hex::decode(v["shared_hex"].as_str().unwrap()).unwrap()
        );
        assert_eq!(
            s.public_key(),
            hex::decode(v["public_hex"].as_str().unwrap()).unwrap()
        );
        assert!(!format!("{s:?}").contains(v["parameters"]["private_key_hex"].as_str().unwrap()));
    }
}

#[test]
fn weak_invalid_noncanonical_wrong_curve_and_oversized_keys_fail_without_secrets() {
    use cube_core::{asymmetric::ecdh, errors::ErrorCode};
    for v in vectors() {
        let s = ecdh::Settings::parse(&v["parameters"].to_string()).unwrap();
        for length in [0, 1, 31, 32, 33, 64, 65] {
            assert!(ecdh::agree(&vec![0; length], &s).is_err());
        }
        assert_eq!(
            ecdh::agree(&[0; 66], &s).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
        let mut p = v["parameters"].clone();
        p["private_key_hex"] = json!("SECRET");
        assert!(
            !format!("{:?}", ecdh::Settings::parse(&p.to_string()).unwrap_err()).contains("SECRET")
        );
        let mut doc = v["parameters"].to_string();
        doc.push_str(&" ".repeat(ecdh::MAX_PARAMETERS - doc.len()));
        ecdh::Settings::parse(&doc).unwrap();
        doc.push(' ');
        assert_eq!(
            ecdh::Settings::parse(&doc).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
        p = v["parameters"].clone();
        p["extra"] = json!(true);
        assert!(ecdh::Settings::parse(&p.to_string()).is_err());
    }
    for scalar in [
        "00".repeat(32),
        "ff".repeat(32),
        "ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551".into(),
    ] {
        assert!(
            ecdh::Settings::parse(
                &json!({"profile":"p256_sec1_raw","private_key_hex":scalar}).to_string()
            )
            .is_err()
        );
    }
    let v = &vectors()[0];
    let s = ecdh::Settings::parse(&v["parameters"].to_string()).unwrap();
    let mut low = [0; 32];
    low[0] = 1;
    assert!(ecdh::agree(&low, &s).is_err());
    let mut masked = hex::decode(v["peer_hex"].as_str().unwrap()).unwrap();
    masked[31] |= 128;
    assert_eq!(
        ecdh::agree(&masked, &s).unwrap(),
        hex::decode(v["shared_hex"].as_str().unwrap()).unwrap()
    );
    let v = &vectors()[1];
    let s = ecdh::Settings::parse(&v["parameters"].to_string()).unwrap();
    for prefix in [0, 1, 5, 6, 7] {
        let mut peer = hex::decode(v["peer_hex"].as_str().unwrap()).unwrap();
        peer[0] = prefix;
        assert!(ecdh::agree(&peer, &s).is_err());
    }
    let mut invalid = vec![255; 65];
    invalid[0] = 4;
    assert!(ecdh::agree(&invalid, &s).is_err());
}
