use cube_core::{
    blockchain::{Network, PrivateInput, bitcoin},
    encoding::{self, base58check},
    hashing,
    pipeline::{Kind, Limits, Plan, Step, Value},
};
#[test]
fn independent_openssl_public_test_scalars_match_sec1_hash160_and_network_addresses() {
    // Independently checked with OpenSSL secp256k1 and hashlib digests/Base58Check.
    for (scalar, compressed, public, hash, mainnet, testnet, wif_main, wif_test) in [
        (
            1,
            true,
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
            "751e76e8199196d454941c45d1b3a323f1433bd6",
            "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH",
            "mrCDrCybB6J1vRfbwM5hemdJz73FwDBC8r",
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn",
            "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN87JcbXMTcA",
        ),
        (
            1,
            false,
            "0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8",
            "91b24bf9f5288532960ac687abb035127b1d28a5",
            "1EHNa6Q4Jz2uvNExL497mE43ikXhwF6kZm",
            "mtoKs9V381UAhUia3d7Vb9GNak8Qvmcsme",
            "5HpHagT65TZzG1PH3CSu63k8DbpvD8s5ip4nEB3kEsreAnchuDf",
            "91avARGdfge8E4tZfYLoxeJ5sGBdNJQH4kvjJoQFacbgwmaKkrx",
        ),
        (
            2,
            true,
            "02c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee5",
            "06afd46bcdfd22ef94ac122aa11f241244a37ecc",
            "1cMh228HTCiwS8ZsaakH8A8wze1JR5ZsP",
            "mg8Jz5776UdyiYcBb9Z873NTozEiADRW5H",
            "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU74NMTptX4",
            "cMahea7zqjxrtgAbB7LSGbcQUr1uX1ojuat9jZodMN87K7XCyj5v",
        ),
        (
            2,
            false,
            "04c6047f9441ed7d6d3045406e95c07cd85c778e4b8cef3ca7abac09b95c709ee51ae168fea63dc339a3c58419466ceaeef7f632653266d0e1236431a950cfe52a",
            "d6c8e828c1eca1bba065e1b83e1dc2a36e387a42",
            "1LagHJk2FyCV2VzrNHVqg3gYG4TSYwDV4m",
            "n16daMq14zdjocUU5rUDVxts8449SL2VrL",
            "5HpHagT65TZzG1PH3CSu63k8DbpvD8s5ip4nEB3kEsreAvUcVfH",
            "91avARGdfge8E4tZfYLoxeJ5sGBdNJQH4kvjJoQFacbgww7vXtT",
        ),
    ] {
        let mut key = [0u8; 32];
        key[31] = scalar;
        for (network, expected, wif) in [
            (Network::Mainnet, mainnet, wif_main),
            (Network::Testnet, testnet, wif_test),
        ] {
            let result = bitcoin::derive(PrivateInput::Raw(&key), network, compressed).unwrap();
            assert_eq!(result.public_key, encoding::hex_decode(public).unwrap());
            assert_eq!(encoding::hex_encode(&result.hash160), hash);
            assert_eq!(result.address, expected);
            assert_eq!(result.public_key.len(), if compressed { 33 } else { 65 });
            assert_eq!(result.network_family, network);
            assert_eq!(result.compressed, compressed);
            let decoded = base58check::decode(&result.address).unwrap();
            assert_eq!(decoded.version, network.p2pkh_version());
            assert_eq!(decoded.payload, result.hash160);
            assert_eq!(
                bitcoin::derive(PrivateInput::Wif(wif), network, compressed)
                    .unwrap()
                    .address,
                expected
            );
            assert_eq!(
                bitcoin::derive(
                    PrivateInput::Hex(&encoding::hex_encode(&key)),
                    network,
                    compressed
                )
                .unwrap()
                .address,
                expected
            );
            let json = serde_json::to_string(&result).unwrap();
            assert!(!json.contains(wif));
            assert!(!json.contains(&encoding::hex_encode(&key)));
        }
    }
}
#[test]
fn invalid_scalars_and_conflicting_wif_intent_have_non_secret_errors() {
    let wif = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
    for (network, compressed) in [(Network::Testnet, true), (Network::Mainnet, false)] {
        let error = bitcoin::derive(PrivateInput::Wif(wif), network, compressed).unwrap_err();
        assert_eq!(error.operation, Some("blockchain.bitcoin.derive"));
        assert!(!format!("{error:?}").contains(wif));
    }
    for key in [vec![0; 32], vec![255; 32], vec![1; 31]] {
        assert!(bitcoin::derive(PrivateInput::Raw(&key), Network::Mainnet, true).is_err());
    }
    let near_order =
        encoding::hex_decode("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140")
            .unwrap();
    assert!(bitcoin::derive(PrivateInput::Raw(&near_order), Network::Mainnet, true).is_ok());
    assert!(
        Plan::validate(
            Kind::Bytes,
            vec![Step::new("blockchain.bitcoin.derive")],
            Limits::default()
        )
        .is_err()
    );
}
#[test]
fn ripemd_vectors_limits_and_existing_pipeline_composition_are_exact() {
    assert_eq!(
        hashing::ripemd160(b"").unwrap(),
        "9c1185a5c5e9fc54612808977ee8f548b2258d31"
    );
    assert_eq!(
        hashing::ripemd160(b"Hello world!").unwrap(),
        "7f772647d88750add82d8e1a7a3e5c0902a346a3"
    );
    assert_eq!(
        hashing::ripemd160(&vec![0; hashing::protocol::MAX_INPUT])
            .unwrap()
            .len(),
        40
    );
    assert!(hashing::ripemd160(&vec![0; hashing::protocol::MAX_INPUT + 1]).is_err());
    let input = Value::Bytes {
        data: b"Hello world!".to_vec(),
    };
    let result = Plan::validate(
        Kind::Bytes,
        vec![
            Step::new("hash.ripemd160"),
            Step::new("encoding.hex.decode"),
        ],
        Limits::default(),
    )
    .unwrap()
    .execute(input, &[0], None)
    .unwrap();
    assert_eq!(
        result.output,
        Value::Bytes {
            data: encoding::hex_decode("7f772647d88750add82d8e1a7a3e5c0902a346a3").unwrap()
        }
    );
}
