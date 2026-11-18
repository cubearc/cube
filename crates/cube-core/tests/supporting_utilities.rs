//! Independent wire answers for byte, text, digest, KDF, compression and JPEG profiles.
use cube_core::{
    binary::compression as c,
    ciphers::kdf,
    encoding::byte_text::Profile as e,
    errors::ErrorCode,
    hashing::extended::{MAX_INPUT as MAX_HASH, Profile as h},
    image::jpeg_coefficients as jpeg,
    text::representation as t,
};

#[test]
fn exact_digest_profiles_and_checksums() {
    // OpenSSL and CPython hashlib, the RIPEMD authors' published vectors, the
    // BLAKE3 official vectors, CRC Catalogue and the Fletcher weighted-sum identity.
    let cases: &[(h, &[u8], &str)] = &[
        (
            h::Sha224,
            b"abc",
            "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7",
        ),
        (
            h::Sha384,
            b"abc",
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7",
        ),
        (
            h::Sha512224,
            b"abc",
            "4634270f707b6a54daae7530460842e20e37ed265ceee9a43e8924aa",
        ),
        (
            h::Sha512256,
            b"abc",
            "53048e2681941ef99b2e29b76b4c7dabe4c2d0c634fc6d46e0e2f13107e7af23",
        ),
        (
            h::Sha3224,
            b"abc",
            "e642824c3f8cf24ad09234ee7d3c766fc9a3a5168d0c94ad73b46fdf",
        ),
        (
            h::Sha3256,
            b"abc",
            "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532",
        ),
        (
            h::Sha3384,
            b"abc",
            "ec01498288516fc926459f58e2c6ad8df9b473cb0fc08c2596da7cf0e49be4b298d88cea927ac7f539f1edf228376d25",
        ),
        (
            h::Sha3512,
            b"abc",
            "b751850b1a57168a5693cd924b6b096e08f621827444f70d884f5d0240d2712e10e116e9192af3c91a7ec57647e3934057340b4cf408d5a56592f8274eec53f0",
        ),
        (
            h::Blake2B512,
            b"abc",
            "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d17d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923",
        ),
        (
            h::Blake2S256,
            b"abc",
            "508c5e8c327c14e2e1a72ba34eeb452f37458b209ed63a294d999b4c86675982",
        ),
        (
            h::Keccak224,
            b"abc",
            "c30411768506ebe1c2871b1ee2e87d38df342317300a9b97a95ec6a8",
        ),
        (
            h::Keccak384,
            b"abc",
            "f7df1165f033337be098e7d288ad6a2f74409d7a60b49c36642218de161b1f99f8c681e4afaf31a34db29fb763e3c28e",
        ),
        (
            h::Keccak512,
            b"abc",
            "18587dc2ea106b9a1563e32b3312421ca164c7f1f07bc922a9c83d77cea3a1e5d0c69910739025372dc14ac9642629379540c17e2a65b19d77aa511a9d00bb96",
        ),
        (h::Md4, b"abc", "a448017aaf21d8525fc10ae87aa6729d"),
        (
            h::Whirlpool,
            b"abc",
            "4e2448a4c6f486bb16b6562c73b4020bf3043e3a731bce721ae1b303d97e6d4c7181eebdb6c57e277d0e34957114cbd6c797fc9d95d8b582d225292076d4eef5",
        ),
        (h::Ripemd128, b"", "cdf26213a150dc3ecb610f18f6b38b46"),
        (
            h::Ripemd256,
            b"",
            "02ba4c4e5f8ecd1877fc52d64d30e37a2d9774fb1e5d026380ae0168e3c5522d",
        ),
        (
            h::Ripemd320,
            b"",
            "22d65d5661536cdc75c1fdf5c6de7b41b9f27325ebc61e8557177d705a0ec880151c3a32a00899b8",
        ),
        (
            h::Blake3,
            b"",
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
        ),
        (h::Crc8Smbus, b"123456789", "f4"),
        (h::Crc16Arc, b"123456789", "bb3d"),
        (h::Crc64Ecma182, b"123456789", "6c40df5f0b497347"),
        (h::Adler32, b"Wikipedia", "11e60398"),
        (h::Fletcher16, b"123456789", "1ede"),
        (h::Fletcher32, b"abcdefgh", "e1eb9195"),
    ];
    let mut boundary = vec![0xa5; MAX_HASH];
    for &(profile, input, expected) in cases {
        assert_eq!(profile.digest(input).unwrap(), expected, "{}", profile.id());
        assert_eq!(profile.digest(b"").unwrap().len(), expected.len());
    }
    assert_eq!(h::Sha512256.digest(&boundary).unwrap().len(), 64);
    boundary.push(1);
    for &(profile, _, _) in cases {
        assert_eq!(
            profile.digest(&boundary).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    assert_eq!(h::Fletcher32.digest(b"abc").unwrap_err().offset, Some(2));
    assert_ne!(
        h::Sha3256.digest(b"abc").unwrap(),
        cube_core::hashing::keccak256(b"abc").unwrap()
    );
    assert_ne!(
        h::Sha512256.digest(b"abc").unwrap(),
        cube_core::hashing::sha512(b"abc")[..64]
    );
}

#[test]
fn byte_text_formats_are_strict_and_preserve_binary_values() {
    for (profile, plain, encoded) in [
        (e::Binary, b"\x00\xff".as_slice(), "00000000 11111111"),
        (e::Octal, b"\x00\xff", "000 377"),
        (e::Base36, b"\0\0\xff", "0073"),
        (e::Base62, b"\0\0\xff", "0047"),
        (e::Ascii85, b"\0\0\0\0", "<~z~>"),
        (e::Ascii85Bare, b"Hello", "87cURDZ"),
        (
            e::Base91,
            b"This is a random binary string",
            "nX,<:WRT%yxtYQ:mbr4JB*H[LR/@Qj{o0aU=Z",
        ),
        (e::Base92, b"hello world", "Fc_$aOTdKnsM*k"),
        (e::UrlComponent, b"a+ /\x00\xff", "a%2B%20%2F%00%FF"),
    ] {
        assert_eq!(profile.encode(plain).unwrap(), encoded);
        assert_eq!(profile.decode(encoded).unwrap(), plain);
        assert!(
            profile
                .decode(&profile.encode(b"").unwrap())
                .unwrap()
                .is_empty()
        );
        let all: Vec<u8> = (0..=255).collect();
        for size in [1, 2, 3, 4, 5, 6, 7, 12, 13, 14, 127, 256] {
            let data = &all[..size];
            assert_eq!(
                profile.decode(&profile.encode(data).unwrap()).unwrap(),
                data
            );
        }
        let too_large = vec![0; profile.byte_limit() + 1];
        assert_eq!(
            profile.encode(&too_large).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
        let too_long = "0".repeat(profile.text_limit() + 1);
        assert_eq!(
            profile.decode(&too_long).unwrap_err().code,
            ErrorCode::ResourceLimit
        );
    }
    for (profile, invalid) in [
        (e::Binary, "00000000  00000001"),
        (e::Binary, "00000000 "),
        (e::Binary, "00000002"),
        (e::Octal, "400"),
        (e::Octal, " -01"),
        (e::Base36, "1A"),
        (e::Base62, "1 "),
        (e::Ascii85, "z"),
        (e::Ascii85, "<~uuuuu~>"),
        (e::Ascii85Bare, "!z"),
        (e::Ascii85Bare, "!"),
        (e::Ascii85Bare, "!!!!!"),
        (e::Base91, "dr.J\n"),
        (e::Base92, "}}"),
        (e::Base92, ""),
        (e::UrlComponent, "+"),
        (e::UrlComponent, "%2"),
        (e::UrlComponent, "%xy"),
    ] {
        assert_eq!(
            profile.decode(invalid).unwrap_err().code,
            ErrorCode::InvalidInput,
            "{profile:?} {invalid:?}"
        );
    }
    assert_eq!(e::Base92.encode(b"").unwrap(), "~");
    assert_eq!(e::UrlComponent.decode("%ff").unwrap(), [255]);
    let input = vec![0xff; e::Base62.byte_limit()];
    assert_eq!(
        e::Base62
            .decode(&e::Base62.encode(&input).unwrap())
            .unwrap(),
        input
    );
}

#[test]
fn character_sets_and_escapes_use_exact_bounds_and_offsets() {
    assert_eq!(t::ascii_encode("a\0").unwrap(), b"a\0");
    assert_eq!(t::ascii_decode(b"a\0").unwrap(), "a\0");
    assert_eq!(t::ascii_encode("aé").unwrap_err().offset, Some(1));
    assert_eq!(t::ascii_decode(b"a\xff").unwrap_err().offset, Some(1));
    assert_eq!(
        cube_core::text::from_utf8(b"a\xe2\x82").unwrap_err().offset,
        Some(1)
    );

    let be = t::Utf16Settings {
        byte_order: t::ByteOrder::Be,
        bom: t::Bom::Require,
    };
    let le = t::Utf16Settings {
        byte_order: t::ByteOrder::Le,
        bom: t::Bom::Require,
    };
    assert_eq!(
        t::utf16_encode("A🧊", be).unwrap(),
        hex::decode("feff0041d83eddca").unwrap()
    );
    assert_eq!(
        t::utf16_encode("A🧊", le).unwrap(),
        hex::decode("fffe41003ed8cadd").unwrap()
    );
    assert_eq!(
        t::utf16_decode(&hex::decode("feff0041d83eddca").unwrap(), be).unwrap(),
        "A🧊"
    );
    assert!(t::utf16_decode(b"\xff\xfe", be).is_err());
    assert_eq!(
        t::utf16_decode(b"\xfe\xff\0", be).unwrap_err().offset,
        Some(2)
    );
    assert_eq!(
        t::utf16_decode(b"\xfe\xff\xd8\x3e\0A", be)
            .unwrap_err()
            .offset,
        Some(2)
    );
    assert_eq!(
        t::utf16_decode(b"\xfe\xff\xdd\xca", be).unwrap_err().offset,
        Some(2)
    );
    for (bom, text) in [
        (t::Bom::Forbid, "A"),
        (t::Bom::Strip, "A"),
        (t::Bom::Preserve, "\u{feff}A"),
    ] {
        let settings = t::Utf16Settings { bom, ..be };
        assert_eq!(
            t::utf16_decode(&t::utf16_encode(text, settings).unwrap(), settings).unwrap(),
            text
        );
    }
    let strip = t::Utf16Settings {
        bom: t::Bom::Strip,
        ..be
    };
    assert_eq!(t::utf16_decode(b"\xfe\xff\0A", strip).unwrap(), "A");

    let json = "a\t\n\0\"\\🧊";
    assert_eq!(
        t::json_escape_encode(json, true).unwrap(),
        r#"a\t\n\u0000\"\\\uD83E\uDDCA"#
    );
    assert_eq!(
        t::json_escape_decode(&t::json_escape_encode(json, false).unwrap()).unwrap(),
        json
    );
    assert_eq!(
        t::json_escape_decode(&t::json_escape_encode(json, true).unwrap()).unwrap(),
        json
    );
    for invalid in [r"\x41", r"\uD800", r"\uDC00", r"\u12", "\"\n", "a\n", "\\"] {
        assert_eq!(
            t::json_escape_decode(invalid).unwrap_err().code,
            ErrorCode::InvalidInput
        );
    }
    assert_eq!(
        t::html_entity_encode("<&>\"'").unwrap(),
        "&#x3C;&#x26;&#x3E;&#x22;&#x27;"
    );
    assert_eq!(t::html_entity_decode("&notin; &notit; &NotEqualTilde; &amp= &unknown; <tag>\r\n&#x80;&#0;&#xD800;&#9999999999999999;").unwrap(), "∉ ¬it; ≂̸ &= &unknown; <tag>\r\n€���");
    assert_eq!(
        t::html_entity_decode("&amp;lt; &#123 #123; &#x;").unwrap(),
        "&lt; { #123; &#x;"
    );
    let full = "\0".repeat(t::MAX_TEXT);
    let escaped = t::json_escape_encode(&full, true).unwrap();
    assert_eq!(escaped.len(), 6 * t::MAX_TEXT);
    assert_eq!(t::json_escape_decode(&escaped).unwrap(), full);
    assert_eq!(
        t::ascii_encode(&(full + "x")).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn explicit_compression_round_trips_and_refuses_invalid_streams() {
    let plain = b"Cube\0\xff\x80\n".repeat(19);
    // CPython zlib/gzip/LZMA, and upstream brotli and zstd command-line tools.
    for (profile, wire) in [
        (c::Profile::Deflate, "732e4d4a65f8dfc0e53cc86800"),
        (c::Profile::Zlib, "789c732e4d4a65f8dfc0e53cc86800368c3999"),
        (
            c::Profile::Gzip,
            "1f8b0800000000000213732e4d4a65f8dfc0e53cc868005f8c972e98000000",
        ),
        (
            c::Profile::Brotli,
            "1f970000c4316e5e75b26bc07d603b740b87a23af808",
        ),
        (
            c::Profile::Zstd,
            "28b52ffd04487d0000404375626500ff800a01008d51850bfa501496",
        ),
        (
            c::Profile::LzmaAlone,
            "5d00000400ffffffffffffffff00219d48466d8c5611dd3cc3a16b7bfffff7084000",
        ),
    ] {
        let settings = c::EncodeSettings {
            profile,
            level: None,
        };
        let decode = c::DecodeSettings { profile };
        let source = hex::decode(wire).unwrap();
        assert_eq!(
            c::decompress(&source, decode).unwrap(),
            plain,
            "{profile:?}"
        );
        let packed = c::compress(&plain, settings).unwrap();
        assert_eq!(
            c::decompress(&packed, decode).unwrap(),
            plain,
            "{profile:?}"
        );
        assert_eq!(c::compress(&plain, settings).unwrap(), packed);
        assert!(
            c::decompress(&c::compress(b"", settings).unwrap(), decode)
                .unwrap()
                .is_empty()
        );
        for invalid in [
            &source[..source.len() - 1],
            &[source.as_slice(), b"x"].concat(),
            &[source.as_slice(), &source].concat(),
        ] {
            assert!(c::decompress(invalid, decode).is_err(), "{profile:?}");
        }
    }
    let gzip = c::compress(
        b"abc",
        c::EncodeSettings {
            profile: c::Profile::Gzip,
            level: Some(0),
        },
    )
    .unwrap();
    assert_eq!(&gzip[..10], &[31, 139, 8, 0, 0, 0, 0, 0, 4, 255]);
    let mut corrupt = gzip.clone();
    let crc = corrupt.len() - 8;
    corrupt[crc] ^= 1;
    assert!(
        c::decompress(
            &corrupt,
            c::DecodeSettings {
                profile: c::Profile::Gzip
            }
        )
        .is_err()
    );
    assert_eq!(
        c::compress(
            b"abc",
            c::EncodeSettings {
                profile: c::Profile::Zstd,
                level: Some(1)
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidParameter
    );
    assert_eq!(
        c::decompress(
            &[0x28, 0xb5, 0x2f, 0xfd, 0x00, 0xf8],
            c::DecodeSettings {
                profile: c::Profile::Zstd
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    let mut oversized_dict = [0; 13];
    oversized_dict[0] = 0x5d;
    oversized_dict[1..5].copy_from_slice(&(1u32 << 26).to_le_bytes());
    assert_eq!(
        c::decompress(
            &oversized_dict,
            c::DecodeSettings {
                profile: c::Profile::LzmaAlone
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        c::compress(
            &vec![0; 256 * 1024],
            c::EncodeSettings {
                profile: c::Profile::Zlib,
                level: None
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn kdf_answers_and_private_settings() {
    let pb = kdf::Pbkdf2 {
        profile: kdf::Sha2::Sha512,
        iterations: 2,
        output_bytes: 73,
    };
    let key = kdf::pbkdf2(b"pass\0word", b"salt\xff", pb).unwrap();
    assert_eq!(
        hex::encode(key.as_slice()),
        "a022a43473d5fdf16387a4ffe8b9d64a671a880b944069cf02659dfe2befcd0671b99eafc3fa688894e06253df182d80968908c6a5f63294d543d9044002d65050b995c4edd16c2787"
    );
    assert_eq!(format!("{key:?}"), "SecretBytes { [REDACTED] }");
    assert_eq!(
        kdf::pbkdf2(
            b"p",
            b"s",
            kdf::Pbkdf2 {
                iterations: 10_000_000,
                ..pb
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    let ikm = [0x0b; 22];
    let salt = hex::decode("000102030405060708090a0b0c").unwrap();
    let info = hex::decode("f0f1f2f3f4f5f6f7f8f9").unwrap();
    let hk = kdf::hkdf(
        &ikm,
        &salt,
        &info,
        kdf::Hkdf {
            profile: kdf::Sha2::Sha256,
            output_bytes: 42,
        },
    )
    .unwrap();
    assert_eq!(
        hex::encode(hk.as_slice()),
        "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
    );
    let salt = b"12345678";
    let sc = kdf::Scrypt {
        n: 16,
        r: 1,
        p: 1,
        output_bytes: 32,
    };
    let a2 = kdf::Argon2id {
        memory_kib: 32,
        iterations: 2,
        lanes: 1,
        output_bytes: 32,
    };
    assert_eq!(
        hex::encode(kdf::scrypt(b"password", salt, sc).unwrap().as_slice()),
        "61d075cbc3c14bbccd226827726a404c95c5deb5410b3b7eb55d70511d568c6a"
    );
    assert_eq!(
        hex::encode(kdf::argon2id(b"password", salt, a2).unwrap().as_slice()),
        "a76b1652c8a7f4fac2d0805f6982e354cb0d9d580cc5553a197f4d79e599e534"
    );
    assert_eq!(
        kdf::scrypt(b"p", salt, kdf::Scrypt { n: 15, ..sc })
            .unwrap_err()
            .parameter,
        Some("n")
    );
    assert_eq!(
        kdf::scrypt(
            b"p",
            salt,
            kdf::Scrypt {
                n: 524288,
                r: 1024,
                ..sc
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        kdf::argon2id(
            b"p",
            salt,
            kdf::Argon2id {
                memory_kib: u32::MAX,
                ..a2
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        kdf::argon2id(b"p", b"short", a2).unwrap_err().parameter,
        Some("salt")
    );
    let evp = kdf::EvpBytesToKey {
        digest: kdf::LegacyDigest::Md5,
        count: 1,
        key_bytes: 32,
        iv_bytes: 16,
    };
    let derived = kdf::evp_bytes_to_key(b"password", salt, evp).unwrap();
    assert_eq!(
        hex::encode([derived.key(), derived.iv()].concat()),
        "bbb0ddff1b944b3cc68eaaeb7ac20099e5155bae6ed28f668b60cbc4157de1bdfa1dd9ab5d62d787256eb8aec82de442"
    );
    assert_eq!(format!("{derived:?}"), "KeyIv { [REDACTED] }");
    assert_eq!(
        kdf::evp_bytes_to_key(
            b"password",
            salt,
            kdf::EvpBytesToKey {
                count: 1_000_000,
                ..evp
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn jpeg_coefficients_match_libjpeg_and_reject_invalid_frames() {
    let input = include_bytes!("../src/image/jpeg_coefficients/fixture.jpg");
    let settings = jpeg::Settings {
        profile: jpeg::Profile::BaselineAcParity,
        component_id: 1,
        start_bit: 3,
        max_bits: 19,
    };
    let r = jpeg::extract(input, settings).unwrap();
    assert_eq!(hex::encode(r.bits), "a44860");
    assert_eq!(r.report.bit_count, 19);
    assert_eq!(r.report.candidate_bits, 193);
    assert_eq!(
        (
            r.report.width,
            r.report.height,
            r.report.block_columns,
            r.report.block_rows
        ),
        (19, 9, 3, 2)
    );
    assert_eq!(
        jpeg::extract(
            input,
            jpeg::Settings {
                component_id: 2,
                ..settings
            }
        )
        .unwrap_err()
        .parameter,
        Some("component_id")
    );
    assert_eq!(
        jpeg::extract(
            input,
            jpeg::Settings {
                start_bit: 193,
                ..settings
            }
        )
        .unwrap()
        .report
        .bit_count,
        0
    );
    assert!(jpeg::extract(&input[..input.len() - 1], settings).is_err());
    let mut trailing = input.to_vec();
    trailing.push(0);
    assert!(jpeg::extract(&trailing, settings).is_err());
    let mut progressive = input.to_vec();
    let sof = progressive
        .windows(2)
        .position(|w| w == [0xff, 0xc0])
        .unwrap();
    progressive[sof + 1] = 0xc2;
    assert_eq!(
        jpeg::extract(&progressive, settings).unwrap_err().code,
        ErrorCode::UnsupportedFormat
    );
    let mut huge = input.to_vec();
    huge[sof + 5..sof + 9].fill(0xff);
    assert_eq!(
        jpeg::extract(&huge, settings).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}
