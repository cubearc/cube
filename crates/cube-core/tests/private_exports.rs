use cube_core::{
    blockchain::{
        Network, PrivateInput,
        private::{self, ExportedPrivate, OutputFormat},
    },
    encoding,
    errors::ErrorCode,
    pipeline::{Kind, Limits, Plan, Step},
};
#[test]
fn deliberate_exports_round_trip_every_scalar_byte_and_wif_intent() {
    let mut bytes = [0u8; 32];
    bytes[31] = 1;
    for (network, compressed, expected) in [
        (
            Network::Mainnet,
            true,
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn",
        ),
        (
            Network::Mainnet,
            false,
            "5HpHagT65TZzG1PH3CSu63k8DbpvD8s5ip4nEB3kEsreAnchuDf",
        ),
        (
            Network::Testnet,
            true,
            "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN87JcbXMTcA",
        ),
        (
            Network::Testnet,
            false,
            "91avARGdfge8E4tZfYLoxeJ5sGBdNJQH4kvjJoQFacbgwmaKkrx",
        ),
    ] {
        let output = private::export(
            PrivateInput::Raw(&bytes),
            OutputFormat::Wif,
            Some(network),
            Some(compressed),
        )
        .unwrap();
        assert_eq!(format!("{output:?}"), "ExportedPrivate([REDACTED])");
        assert_eq!(output.into_private_bytes(), expected.as_bytes());
        let raw = private::export(
            PrivateInput::Wif(expected),
            OutputFormat::Raw,
            Some(network),
            Some(compressed),
        )
        .unwrap();
        assert_eq!(raw.into_private_bytes(), bytes);
        let hex =
            private::export(PrivateInput::Wif(expected), OutputFormat::Hex, None, None).unwrap();
        assert_eq!(
            hex.into_private_bytes(),
            encoding::hex_encode(&bytes).as_bytes()
        );
        let same = private::export(
            PrivateInput::Wif(expected),
            OutputFormat::Wif,
            Some(network),
            Some(compressed),
        )
        .unwrap();
        assert_eq!(same.into_private_bytes(), expected.as_bytes());
    }
    for text in [
        "0000000000000000000000000000000000000000000000000000000000000001",
        "FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364140",
    ] {
        let raw = private::export(PrivateInput::Hex(text), OutputFormat::Raw, None, None).unwrap();
        assert!(matches!(raw, ExportedPrivate::Raw(_)));
        assert_eq!(
            raw.into_private_bytes(),
            encoding::hex_decode(text).unwrap()
        );
        let hex = private::export(PrivateInput::Hex(text), OutputFormat::Hex, None, None).unwrap();
        assert_eq!(hex.into_private_bytes(), text.to_lowercase().as_bytes());
    }
}
#[test]
fn missing_or_conflicting_intent_and_invalid_inputs_never_export() {
    let scalar = [1u8; 32];
    assert!(
        private::export(
            PrivateInput::Raw(&scalar),
            OutputFormat::Wif,
            None,
            Some(true)
        )
        .is_err()
    );
    assert!(
        private::export(
            PrivateInput::Raw(&scalar),
            OutputFormat::Wif,
            Some(Network::Mainnet),
            None
        )
        .is_err()
    );
    assert!(
        private::export(
            PrivateInput::Raw(&scalar),
            OutputFormat::Hex,
            Some(Network::Mainnet),
            None
        )
        .is_err()
    );
    for bad in [vec![0; 32], vec![255; 32], vec![1; 31]] {
        assert!(private::export(PrivateInput::Raw(&bad), OutputFormat::Raw, None, None).is_err());
    }
    let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
    for (network, compressed) in [(Network::Mainnet, false), (Network::Testnet, true)] {
        let error = private::export(
            PrivateInput::Wif(wif),
            OutputFormat::Wif,
            Some(network),
            Some(compressed),
        )
        .unwrap_err();
        assert!(!format!("{error:?}").contains(wif));
        assert_eq!(error.operation, Some("blockchain.private.export"));
    }
    assert_eq!(
        OutputFormat::parse("other").unwrap_err().code,
        ErrorCode::InvalidParameter
    );
    assert!(
        Plan::validate(
            Kind::Bytes,
            vec![Step::new("blockchain.private.export")],
            Limits::default()
        )
        .is_err()
    );
}
