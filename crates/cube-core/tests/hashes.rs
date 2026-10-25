use cube_core::{
    hashing,
    pipeline::{Kind, Limits, Plan, Step, Value},
};
// SHA vectors: RFC 6234 section 8.5; MD5: RFC 1321 appendix A.5.
#[test]
fn standard_hash_vectors_preserve_widths_and_legacy_metadata() {
    type Vector = (fn(&[u8]) -> String, &'static str, &'static str, usize);
    let cases: [Vector; 3] = [
        (
            hashing::sha512,
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
            128,
        ),
        (
            hashing::sha1,
            "da39a3ee5e6b4b0d3255bfef95601890afd80709",
            "a9993e364706816aba3e25717850c26c9cd0d89d",
            40,
        ),
        (
            hashing::md5,
            "d41d8cd98f00b204e9800998ecf8427e",
            "900150983cd24fb0d6963f7d28e17f72",
            32,
        ),
    ];
    for (hash, empty, abc, width) in cases {
        assert_eq!(hash(b""), empty);
        assert_eq!(hash(b"abc"), abc);
        let binary: Vec<u8> = (0..=255).collect();
        let result = hash(&binary);
        assert_eq!(result.len(), width);
        assert!(
            result
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        );
        assert_eq!(hash(&binary), result);
    }
    assert_eq!(hashing::md5(b"a"), "0cc175b9c0f1b6a831c399e269772661");
    for id in ["hash.md5", "hash.sha1"] {
        assert!(
            cube_core::operations::require(id)
                .unwrap()
                .description
                .contains("unsuitable for collision-resistant security")
        );
    }
}
#[test]
fn named_crc_variants_match_published_catalog_checks() {
    assert_eq!(hashing::crc32c(b""), "00000000");
    assert_eq!(hashing::crc32c(b"123456789"), "e3069283");
    assert_eq!(hashing::crc16_ibm3740(b""), "ffff");
    assert_eq!(hashing::crc16_ibm3740(b"123456789"), "29b1");
    let bytes: Vec<u8> = (0..=255).collect();
    assert_eq!(hashing::crc16_ibm3740(&bytes), "3fbd");
    assert_eq!(hashing::crc32c(&bytes), "9c44184b");
    assert_eq!(hashing::crc32(b"123456789"), "cbf43926");
    // Exact maintained-catalog parameters distinguish each checksum variant.
    let c = crc::CRC_32_ISCSI;
    assert_eq!(
        (c.width, c.poly, c.init, c.refin, c.refout, c.xorout),
        (32, 0x1edc6f41, 0xffffffff, true, true, 0xffffffff)
    );
    let c = crc::CRC_16_IBM_3740;
    assert_eq!(
        (c.width, c.poly, c.init, c.refin, c.refout, c.xorout),
        (16, 0x1021, 0xffff, false, false, 0)
    );
}
#[test]
fn hash_adapters_are_typed_bounded_and_compose_with_existing_hex_decode() {
    for (id, expected) in [
        ("hash.sha512", hashing::sha512(b"abc")),
        ("hash.sha1", hashing::sha1(b"abc")),
        ("hash.md5", hashing::md5(b"abc")),
        ("hash.crc32c", hashing::crc32c(b"abc")),
        ("hash.crc16_ibm3740", hashing::crc16_ibm3740(b"abc")),
    ] {
        let plan = Plan::validate(
            Kind::Bytes,
            vec![Step::new(id), Step::new("encoding.hex.decode")],
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
            Value::Bytes {
                data: hex::decode(&expected).unwrap()
            }
        );
        assert!(Plan::validate(Kind::Text, vec![Step::new(id)], Limits::default()).is_err());
        let limits = Limits {
            max_value_bytes: 1,
            ..Limits::default()
        };
        assert!(
            Plan::validate(Kind::Bytes, vec![Step::new(id)], limits)
                .unwrap()
                .execute(Value::Bytes { data: vec![] }, &[], None)
                .is_err()
        );
    }
}
