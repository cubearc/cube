use cube_core::{
    blockchain::{
        PrivateInput,
        ethereum::{self, Checksum},
    },
    encoding, hashing,
    pipeline::{Kind, Limits, Plan, Step, Value},
};
use sha3::{Digest, Keccak256, Sha3_256};
#[test]
fn independent_public_vectors_exclude_sec1_prefix_and_sha3() {
    // Public synthetic scalar=1, and go-ethereum crypto/crypto_test.go TestNewContractAddress.
    for (scalar, address) in [
        (
            format!("{:064x}", 1),
            "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf",
        ),
        (
            "289c2857d4598e37fb9647507e47a309d6133539bf21a8b9cb6df88fd5232032".into(),
            "0x970e8128ab834e8eac17ab8e3812f010678cf791",
        ),
    ] {
        let bytes = encoding::hex_decode(&scalar).unwrap();
        let result = ethereum::derive(PrivateInput::Raw(&bytes), Checksum::Lowercase).unwrap();
        assert_eq!(result.address, address);
        assert_eq!(result.public_key_xy.len(), 64);
        assert_eq!(encoding::hex_encode(&result.address_bytes), &address[2..]);
        let keccak = Keccak256::digest(&result.public_key_xy);
        assert_eq!(&result.address_bytes, &keccak[12..]);
        let sha3 = Sha3_256::digest(&result.public_key_xy);
        assert_ne!(&result.address_bytes, &sha3[12..]);
        let mut with_prefix = vec![4];
        with_prefix.extend(&result.public_key_xy);
        assert_ne!(&result.address_bytes, &Keccak256::digest(with_prefix)[12..]);
        assert_eq!(
            ethereum::derive(PrivateInput::Hex(&scalar), Checksum::Lowercase)
                .unwrap()
                .address,
            address
        );
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains(&scalar));
    }
    let scalar = format!("{:064x}", 1);
    let result = ethereum::derive(PrivateInput::Hex(&scalar), Checksum::Eip55).unwrap();
    assert_eq!(result.address, "0x7E5F4552091A69125d5DfCb7b8C2659029395Bdf");
    assert_eq!(
        encoding::hex_encode(&result.public_key_xy),
        "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8"
    );
}
#[test]
fn digest_vectors_limits_and_pipeline_are_exact() {
    assert_eq!(
        hashing::keccak256(b"").unwrap(),
        "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
    );
    assert_eq!(
        hashing::keccak256(b"abc").unwrap(),
        "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"
    );
    assert_ne!(
        hashing::keccak256(b"").unwrap(),
        encoding::hex_encode(&Sha3_256::digest(b""))
    );
    let plan = Plan::validate(
        Kind::Bytes,
        vec![Step::new("hash.keccak256")],
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        plan.execute(
            Value::Bytes {
                data: b"abc".to_vec()
            },
            &[],
            None
        )
        .unwrap()
        .output,
        Value::Text {
            data: hashing::keccak256(b"abc").unwrap()
        }
    );
    assert!(hashing::keccak256(&vec![0; hashing::protocol::MAX_INPUT + 1]).is_err());
    assert!(hashing::keccak256(&vec![0; hashing::protocol::MAX_INPUT]).is_ok());
}
#[test]
fn strict_scalar_checksum_and_secret_input_boundaries() {
    for key in [vec![0; 32], vec![255; 32], vec![1; 31], vec![1; 33]] {
        let e = ethereum::derive(PrivateInput::Raw(&key), Checksum::Eip55).unwrap_err();
        assert_eq!(e.operation, Some("blockchain.ethereum.derive"));
    }
    for hex in [
        "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364140",
        "0000000000000000000000000000000000000000000000000000000000000001",
    ] {
        assert!(ethereum::derive(PrivateInput::Hex(hex), Checksum::Eip55).is_ok());
    }
    for bad in [
        "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141".into(),
        "g".repeat(64),
        "a".repeat(129),
    ] {
        let e = ethereum::derive(PrivateInput::Hex(&bad), Checksum::Eip55).unwrap_err();
        assert!(!format!("{e:?}").contains(&bad));
    }
    assert!(Checksum::parse("sha3").is_err());
    assert_eq!(Checksum::parse("lowercase").unwrap(), Checksum::Lowercase);
    assert_eq!(Checksum::parse("eip55").unwrap(), Checksum::Eip55);
    assert!(ethereum::derive(PrivateInput::Wif("not echoed"), Checksum::Eip55).is_err());
    assert!(
        Plan::validate(
            Kind::Bytes,
            vec![Step::new("blockchain.ethereum.derive")],
            Limits::default()
        )
        .is_err()
    );
}
