use cube_core::{
    blockchain::{self, Network, PrivateInput, PrivateKey, wif},
    encoding::{self, base58check},
    errors::ErrorCode,
    pipeline::{Kind, Limits, Plan, Step},
};
const ORDER: &str = "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141";
#[test]
fn official_wif_vectors_validate_without_private_outputs() {
    // Public Bitcoin Core key_io_valid.json vectors, never production material.
    for (text, hex, network, compressed) in [
        (
            "5JuW2AMDYu4xVwRG9DZW18VbzQrGcd5RCgb99sS6ehJsNQXu5b9",
            "8f8943bf956de595665c38ffff23827e17c10cdc1c27a028caae6c9810626198",
            Network::Mainnet,
            false,
        ),
        (
            "L5nJeqKmpHp4P7F8ZYyjwc5a7P4d8EabuGAzfGJk7yC1BJyzNaEd",
            "ff778740f88ddcf102aeb81daee289c044c4a4571c4b6f287400f4b8e0b843f8",
            Network::Mainnet,
            true,
        ),
        (
            "92ZdE5HoLafywnTBbzPxbvRmp75pSfzvdU3XaZGh1cToipgdHVh",
            "80c32d81e91bdea04cd7a3819b32275fc3298af4c7ec87eb0099527d041ced5c",
            Network::Testnet,
            false,
        ),
        (
            "cV83kKisF3RQSvXbUCm9ox3kaz5JjEUBWcx8tNydfGJcyeUxuH47",
            "e0fcd4ce4e3d0e3de091f21415bb7cd011fac288c42020a879f28c2a4387df9b",
            Network::Testnet,
            true,
        ),
    ] {
        let info = wif::validate(text, Some(network)).unwrap();
        assert_eq!(info.compressed, compressed);
        assert_eq!(info.network_family, network);
        assert_eq!(info.version, network.private_version());
        assert!(info.valid && info.scalar_validated);
        assert_eq!(
            info.test_networks_indistinguishable,
            network == Network::Testnet
        );
        let output = serde_json::to_string(&info).unwrap();
        assert!(!output.contains(text));
        assert!(!output.contains(hex));
        let key = PrivateKey::from_hex(hex).unwrap();
        assert_eq!(format!("{key:?}"), "PrivateKey([REDACTED])");
        assert!(!format!("{info:?}").contains(hex));
        assert_eq!(
            format!("{:?}", PrivateInput::Hex(hex)),
            "PrivateInput([REDACTED])"
        );
    }
}
#[test]
fn scalar_boundaries_are_delegated_to_the_validated_curve_type() {
    for bytes in [
        vec![0; 32],
        encoding::hex_decode(ORDER).unwrap(),
        vec![255; 32],
        vec![1; 31],
        vec![1; 33],
    ] {
        assert!(PrivateKey::from_bytes(&bytes).is_err());
    }
    for text in [
        "0000000000000000000000000000000000000000000000000000000000000001",
        "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
    ] {
        let key = PrivateKey::from_hex(text).unwrap();
        assert!(key.info().valid);
        assert_eq!(key.info().byte_width, 32);
        assert!(PrivateKey::from_hex(&text.to_uppercase()).is_ok());
        assert!(
            blockchain::validate_scalar(PrivateInput::Hex(text))
                .unwrap()
                .valid
        );
        assert!(
            blockchain::validate_scalar(PrivateInput::Raw(&encoding::hex_decode(text).unwrap()))
                .unwrap()
                .valid
        );
    }
    for text in [
        "",
        "00",
        "0x0000000000000000000000000000000000000000000000000000000000000001",
        " 0000000000000000000000000000000000000000000000000000000000000001",
        "gggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggggg",
    ] {
        assert!(PrivateKey::from_hex(text).is_err());
    }
    assert!(PrivateInput::parse("hex", &[255]).is_err());
    assert!(PrivateInput::parse("other", b"test").is_err());
    assert_eq!(
        PrivateInput::parse("raw", &[0; 129]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    assert!(blockchain::validate_scalar(PrivateInput::Wif("test")).is_err());
    assert!(Network::parse("bitcoin").is_err());
    assert_eq!(Network::parse("testnet").unwrap(), Network::Testnet);
}
#[test]
fn wif_shapes_networks_scalars_and_capture_eligibility_fail_safely() {
    let mut one = vec![0; 32];
    one[31] = 1;
    for (version, payload) in [
        (0x81, one.clone()),
        (0x80, vec![0; 32]),
        (0xef, encoding::hex_decode(ORDER).unwrap()),
        (0x80, vec![1; 31]),
        (0x80, vec![1; 34]),
    ] {
        let text = base58check::encode(version, &payload).unwrap();
        let error = wif::validate(&text, None).unwrap_err();
        assert!(!format!("{error:?}").contains(&text));
        assert_eq!(error.operation, Some("blockchain.wif.validate"));
    }
    let mut bad = one.clone();
    bad.push(2);
    assert!(wif::validate(&base58check::encode(0x80, &bad).unwrap(), None).is_err());
    let text = base58check::encode(0x80, &one).unwrap();
    assert!(wif::validate(&text, Some(Network::Testnet)).is_err());
    assert!(wif::validate(&(text.clone() + "\n"), None).is_err());
    assert!(wif::validate(&text[..text.len() - 1], None).is_err());
    assert_eq!(
        wif::validate(&"1".repeat(129), None).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    for (id, kind) in [
        ("blockchain.wif.validate", Kind::Text),
        ("blockchain.scalar.validate", Kind::Bytes),
    ] {
        assert!(Plan::validate(kind, vec![Step::new(id)], Limits::default()).is_err());
    }
}
