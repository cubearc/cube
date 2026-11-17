use cube_core::{
    asymmetric::dh,
    operations::execution::{self, Budget},
};
use serde_json::{Value, json};
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!("../src/asymmetric/dh/vectors.json")).unwrap()
}
#[test]
fn independent_all_named_groups_match_public_keys_and_shared_secrets() {
    for v in vectors() {
        let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
        let input = hex::decode(v["peer_hex"].as_str().unwrap()).unwrap();
        let s = dh::Settings::parse(&v["parameters"].to_string(), &key).unwrap();
        assert_eq!(hex::encode(s.public_key()), v["public_hex"]);
        let result=execution::execute(dh::AGREE,1,&json!({"input":{"base64":cube_core::encoding::base64_encode(&input)},"key":{"base64":cube_core::encoding::base64_encode(&key)}}),&v["parameters"],Budget::default()).unwrap();
        assert_eq!(
            result,
            json!({"output":{"base64":cube_core::encoding::base64_encode(&hex::decode(v["shared_hex"].as_str().unwrap()).unwrap())}})
        );
        assert!(!format!("{s:?}").contains(v["private_key_hex"].as_str().unwrap()));
    }
}
#[test]
fn invalid_subgroup_encodings_settings_and_secret_bounds_fail() {
    let v = &vectors()[0];
    let key = hex::decode(v["private_key_hex"].as_str().unwrap()).unwrap();
    let p = v["parameters"].to_string();
    let s = dh::Settings::parse(&p, &key).unwrap();
    for length in [0, 1, 255, 257, 1024, 1025] {
        assert!(dh::agree(&vec![0; length], &s).is_err());
    }
    let modulus = hex::decode(v["modulus_hex"].as_str().unwrap()).unwrap();
    for input in [vec![0; 256], modulus.clone()] {
        assert!(dh::agree(&input, &s).is_err());
    }
    let mut input = vec![0; 256];
    input[255] = 1;
    assert!(dh::agree(&input, &s).is_err());
    let mut nonmember = modulus;
    nonmember[255] -= 2;
    assert!(dh::agree(&nonmember, &s).is_err());
    for length in [0, 1, 31, 33, 64, 65] {
        assert!(dh::Settings::parse(&p, &vec![2; length]).is_err());
    }
    for last in [0, 1] {
        let mut key = vec![0; 32];
        key[31] = last;
        assert!(dh::Settings::parse(&p, &key).is_err());
    }
    for doc in [
        "{}".to_string(),
        "{\"profile\":\"custom\"}".into(),
        format!("{p}{}", " ".repeat(257)),
        "{\"profile\":\"ffdhe2048_rfc7919_raw\",\"extra\":1}".into(),
    ] {
        assert!(dh::Settings::parse(&doc, &key).is_err());
    }
}
