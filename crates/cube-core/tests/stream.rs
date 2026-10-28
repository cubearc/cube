use cube_core::ciphers::stream::{self, Profile};

#[test]
fn rfc8439_block_and_exact_counter_boundaries() {
    let key: Vec<_> = (0..32).collect();
    let nonce = hex::decode("000000090000004a00000000").unwrap();
    let result = stream::transform(Profile::ChaCha20, &[0; 64], &key, &nonce, 1).unwrap();
    assert_eq!(
        hex::encode(result),
        "10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4ed2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e"
    );
    for p in Profile::ALL {
        let nonce = vec![0; p.nonce_bytes()];
        let last = p.max_counter();
        for n in [0, 1, 63, 64] {
            let data = vec![0xff; n];
            let encrypted = stream::transform(p, &data, &key, &nonce, last).unwrap();
            assert_eq!(
                stream::transform(p, &encrypted, &key, &nonce, last).unwrap(),
                data
            );
        }
        assert!(stream::transform(p, &[0; 65], &key, &nonce, last).is_err());
        assert!(stream::transform(p, &[0; 129], &key, &nonce, last - 1).is_err());
        assert!(stream::transform(p, &[0; 128], &key, &nonce, last - 1).is_ok());
    }
}

#[test]
fn invalid_parameters_empty_and_bounded_binary_inverse() {
    for p in Profile::ALL {
        let nonce = vec![0; p.nonce_bytes()];
        assert!(stream::transform(p, &[], &[0; 31], &nonce, 0).is_err());
        assert!(stream::transform(p, &[], &[0; 32], &nonce[1..], 0).is_err());
        assert!(
            stream::transform(p, &[], &[0; 32], &nonce, 0)
                .unwrap()
                .is_empty()
        );
        if p.max_counter() < u64::MAX {
            assert!(stream::transform(p, &[], &[0; 32], &nonce, u64::MAX).is_err());
        }
        let data = vec![0xa5; stream::MAX_INPUT];
        let encrypted = stream::transform(p, &data, &[0; 32], &nonce, 0).unwrap();
        assert_eq!(
            stream::transform(p, &encrypted, &[0; 32], &nonce, 0).unwrap(),
            data
        );
        assert!(
            stream::transform(p, &vec![0; stream::MAX_INPUT + 1], &[0; 32], &nonce, 0).is_err()
        );
    }
}

#[test]
fn vernam_reuses_exact_xor_and_preserves_repeating_xor() {
    let data: Vec<_> = (0..=255).collect();
    let key = vec![255; 256];
    let output = stream::vernam(&data, &key).unwrap();
    assert_eq!(output, (0..=255).rev().collect::<Vec<_>>());
    assert_eq!(stream::vernam(&output, &key).unwrap(), data);
    assert!(stream::vernam(&[], &[]).unwrap().is_empty());
    assert!(stream::vernam(b"ab", b"x").is_err());
    assert_eq!(cube_core::binary::xor(b"ab", b"x").unwrap(), vec![25, 26]);
}
