use cube_core::{
    binary::{
        bcd, bit_units, packing::BitOrder, reed_solomon as rs, representation as repr, varint,
    },
    encoding::{hex_decode, rfc4648 as rfc},
    errors::{ErrorCode, OperationError},
    hashing::fingerprints as hash,
    operations::{self, execution},
    pipeline,
};
use serde_json::{Value, json};

fn error<T>(value: Result<T, OperationError>) -> ErrorCode {
    value.err().expect("expected a typed failure").code
}

#[test]
fn bits_order_and_tail_and_unit_bounds() {
    let bits = [1, 0, 1, 1, 0, 0, 0, 0, 1];
    for (order, expected) in [(BitOrder::Msb, [0xb0, 0x80]), (BitOrder::Lsb, [0x0d, 1])] {
        let pad = bit_units::Pack {
            bit_order: order,
            tail: bit_units::Tail::PadZero,
        };
        assert_eq!(bit_units::pack(&bits, pad).unwrap(), (expected.into(), 9));
        let expand = bit_units::Unpack {
            bit_order: order,
            valid_bits: 9,
        };
        assert_eq!(
            bit_units::unpack(&expected, expand).unwrap(),
            (bits.into(), 9)
        );
        assert_eq!(
            error(bit_units::pack(
                &bits,
                bit_units::Pack {
                    tail: bit_units::Tail::Reject,
                    ..pad
                }
            )),
            ErrorCode::InvalidInput
        );
        assert_eq!(error(bit_units::pack(&[2], pad)), ErrorCode::InvalidInput);
        assert_eq!(
            error(bit_units::unpack(&[], expand)),
            ErrorCode::InvalidParameter
        );
        let limit = vec![0; 1 << 20];
        assert_eq!(bit_units::pack(&limit, pad).unwrap().0.len(), 1 << 17);
        assert_eq!(
            bit_units::unpack(
                &limit,
                bit_units::Unpack {
                    valid_bits: 8 << 20,
                    ..expand
                }
            )
            .unwrap()
            .0
            .len(),
            8 << 20
        );
        assert_eq!(
            error(bit_units::pack(&[limit.as_slice(), &[0]].concat(), pad)),
            ErrorCode::ResourceLimit
        );
    }
    for width in [1, 2, 4, 8] {
        let max = u64::MAX >> (64 - 8 * width);
        assert_eq!(
            bit_units::mask(
                &[255; 8],
                bit_units::Mask {
                    unit_bytes: width,
                    mask: max
                }
            )
            .unwrap(),
            [255; 8]
        );
        let data = [vec![0; width], vec![128; width]].concat();
        for (kind, values) in [
            (
                bit_units::CountKind::LeadingZeros,
                vec![8 * width as u32, 0],
            ),
            (
                bit_units::CountKind::TrailingZeros,
                vec![8 * width as u32, 7],
            ),
            (bit_units::CountKind::Population, vec![0, width as u32]),
        ] {
            assert_eq!(
                bit_units::count(
                    &data,
                    bit_units::Count {
                        unit_bytes: width,
                        kind
                    }
                )
                .unwrap(),
                values
            );
        }
    }
    let count = bit_units::Count {
        unit_bytes: 8,
        kind: bit_units::CountKind::LeadingZeros,
    };
    assert_eq!(
        bit_units::count(&vec![0; 65536 * 8], count).unwrap().len(),
        65_536
    );
    assert_eq!(
        error(bit_units::count(&vec![0; 65537 * 8], count)),
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        error(bit_units::count(&[0; 7], count)),
        ErrorCode::InvalidParameter
    );
    assert_eq!(
        error(bit_units::mask(
            &[0xff],
            bit_units::Mask {
                unit_bytes: 1,
                mask: 256
            }
        )),
        ErrorCode::InvalidParameter
    );
}

#[test]
fn fixed_integer_covers_widths_byte_order_and_exact_transport() {
    use repr::{ByteOrder, Integer, IntegerFormat, Signedness};
    for bits in [8, 16, 32, 64] {
        let shift = 64 - bits;
        let values = [i64::MIN >> shift, -1, 0, 1, i64::MAX >> shift];
        for order in [ByteOrder::Big, ByteOrder::Little] {
            let format = IntegerFormat {
                bits,
                byte_order: order,
                signedness: Signedness::Signed,
            };
            let data = repr::encode(&values.map(Integer::Signed), format).unwrap();
            assert_eq!(data.len(), 5 * usize::from(bits / 8));
            let repr::Integers::Signed { values: roundtrip } = repr::decode(&data, format).unwrap()
            else {
                panic!("expected signed result");
            };
            assert_eq!(roundtrip, values);
            assert_eq!(
                error(repr::encode(&[Integer::Unsigned(u64::MAX)], format)),
                ErrorCode::InvalidInput
            );
            let format = IntegerFormat {
                signedness: Signedness::Unsigned,
                ..format
            };
            assert_eq!(
                repr::encode(&[Integer::Unsigned(u64::MAX >> shift)], format).unwrap(),
                vec![255; usize::from(bits / 8)]
            );
            assert_eq!(
                error(repr::encode(&[Integer::Signed(-1)], format)),
                ErrorCode::InvalidInput
            );
            if bits < 64 {
                assert_eq!(
                    error(repr::encode(&[Integer::Unsigned(1 << bits)], format)),
                    ErrorCode::InvalidInput
                );
            }
        }
    }
    let format = IntegerFormat {
        bits: 16,
        byte_order: ByteOrder::Little,
        signedness: Signedness::Unsigned,
    };
    assert_eq!(
        repr::encode(&[Integer::Unsigned(0x1234)], format).unwrap(),
        [0x34, 0x12]
    );
    assert_eq!(error(repr::decode(&[1], format)), ErrorCode::InvalidInput);
    assert_eq!(
        error(repr::encode(&[], IntegerFormat { bits: 24, ..format })),
        ErrorCode::InvalidParameter
    );
    let dense = vec![Integer::Unsigned(u64::MAX); 65536];
    let wide = IntegerFormat { bits: 64, ..format };
    assert_eq!(error(repr::encode(&dense, wide)), ErrorCode::ResourceLimit);
    assert_eq!(
        error(repr::decode(&vec![255; 8 * 65536], wide)),
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        repr::decode(&vec![0; 65536 * 2], format)
            .map(|value| {
                let repr::Integers::Unsigned { values } = value else {
                    panic!("expected unsigned")
                };
                values.len()
            })
            .unwrap(),
        65536
    );
    assert_eq!(
        error(repr::decode(&vec![0; 65537 * 2], format)),
        ErrorCode::ResourceLimit
    );
}

#[test]
fn ieee_fields_preserve_every_class_and_raw_payload() {
    let vectors = [
        ("00000000", false, 0, 0, "zero"),
        ("80000000", true, 0, 0, "zero"),
        ("00000001", false, 0, 1, "subnormal"),
        ("3f800000", false, 127, 0, "normal"),
        ("c0200000", true, 128, 0x20_0000, "normal"),
        ("ff800000", true, 255, 0, "infinity"),
        ("7fc01001", false, 255, 0x40_1001, "quiet_nan"),
        ("7f800001", false, 255, 1, "signaling_nan"),
    ];
    let layout = repr::FloatLayout {
        format: repr::FloatFormat::Binary32,
        byte_order: repr::ByteOrder::Big,
    };
    let bytes = hex_decode(&vectors.map(|v| v.0).concat()).unwrap();
    let report = repr::inspect_floats(&bytes, layout).unwrap();
    for (record, (bits, sign, exp, fraction, class)) in report.iter().zip(vectors) {
        assert_eq!(
            serde_json::to_value(record).unwrap(),
            json!({
                "bits": u32::from_str_radix(bits, 16).unwrap(), "sign": sign,
                "biased_exponent":exp, "fraction":fraction, "classification":class,
            })
        );
    }
    let little = repr::FloatLayout {
        format: repr::FloatFormat::Binary64,
        byte_order: repr::ByteOrder::Little,
    };
    let data = 0xfff8_0000_0000_0123u64.to_le_bytes();
    assert_eq!(
        serde_json::to_value(&repr::inspect_floats(&data, little).unwrap()[0]).unwrap(),
        json!({"bits":0xfff8_0000_0000_0123u64,"biased_exponent":2047,"fraction":0x8_0000_0000_0123u64,"sign":true,"classification":"quiet_nan"})
    );
    assert_eq!(
        error(repr::inspect_floats(&data[..7], little)),
        ErrorCode::InvalidInput
    );
    assert_eq!(
        error(repr::inspect_floats(&vec![0; 65_537 * 4], layout)),
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        error(repr::inspect_floats(&vec![0; 65_536 * 8], little)),
        ErrorCode::ResourceLimit
    );
}

#[test]
fn bcd_digits_width_padding_and_signed_zero() {
    use bcd::Profile::*;
    for (profile, text, negative, hex) in [
        (UnsignedPacked, "000999", false, "000999"),
        (UnsignedPacked, "099", false, "0099"),
        (UnsignedUnpacked, "099", false, "000909"),
        (SignedPacked, "123", false, "123c"),
        (SignedPacked, "123", true, "123d"),
        (SignedPacked, "00", true, "000c"),
    ] {
        let digits = text.len();
        let encoded = bcd::encode(text, negative, bcd::Layout { profile, digits }).unwrap();
        assert_eq!(encoded, hex_decode(hex).unwrap());
        let decoded = bcd::decode(
            &encoded,
            bcd::Decode {
                profile,
                digits,
                allow_unsigned_sign: false,
            },
        )
        .unwrap();
        assert_eq!(
            (decoded.digits.as_str(), decoded.negative),
            (text, negative && text != "00")
        );
    }
    let options = bcd::Decode {
        profile: SignedPacked,
        digits: 3,
        allow_unsigned_sign: false,
    };
    assert!(
        bcd::decode(
            &[0x12, 0x3f],
            bcd::Decode {
                allow_unsigned_sign: true,
                ..options
            }
        )
        .is_ok()
    );
    for data in [&[0x12, 0x3f][..], &[0x1a, 0x3c], &[0x12, 0x30], &[0x12]] {
        assert_eq!(error(bcd::decode(data, options)), ErrorCode::InvalidInput);
    }
    assert!(!bcd::decode(&[0, 0x0d], options).unwrap().negative);
    assert_eq!(
        error(bcd::decode(
            &[0x12],
            bcd::Decode {
                profile: UnsignedPacked,
                digits: 1,
                ..options
            }
        )),
        ErrorCode::InvalidInput
    );
    let digits = "0".repeat(4096);
    assert_eq!(
        bcd::encode(
            &digits,
            false,
            bcd::Layout {
                profile: UnsignedUnpacked,
                digits: 4096
            }
        )
        .unwrap()
        .len(),
        4096
    );
    assert_eq!(
        error(bcd::encode(
            "",
            false,
            bcd::Layout {
                profile: UnsignedUnpacked,
                digits: 4097
            }
        )),
        ErrorCode::ResourceLimit
    );
}

#[test]
fn varint_exact_extrema_shortest_and_consumption() {
    use repr::Integer::*;
    use varint::{Decode, Encode, Mode, Profile::*};
    for (profile, input, expected, value) in [
        (Uleb128, Unsigned(624485), "e58e26", json!(624485)),
        (Sleb128, Signed(-624485), "9bf159", json!(-624485)),
        (ProtobufUint64, Unsigned(150), "9601", json!(150)),
        (Zigzag64, Signed(-1), "01", json!(-1)),
        (Zigzag64, Signed(1), "02", json!(1)),
        (
            Sleb128,
            Signed(i64::MIN),
            "8080808080808080807f",
            json!(i64::MIN),
        ),
        (
            Sleb128,
            Signed(i64::MAX),
            "ffffffffffffffffff00",
            json!(i64::MAX),
        ),
        (
            Zigzag64,
            Signed(i64::MIN),
            "ffffffffffffffffff01",
            json!(i64::MIN),
        ),
        (
            Uleb128,
            Unsigned(u64::MAX),
            "ffffffffffffffffff01",
            json!(u64::MAX),
        ),
    ] {
        let data = varint::encode(&[input], Encode { profile }).unwrap();
        assert_eq!(data, hex_decode(expected).unwrap());
        let mut with_tail = data.clone();
        with_tail.push(0x80);
        let opts = Decode {
            profile,
            mode: Mode::Prefix,
            allow_noncanonical: false,
        };
        let result = varint::decode(&with_tail, opts).unwrap();
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            json!([{"value": value, "consumed": data.len()}])
        );
        assert_eq!(
            error(varint::decode(
                &with_tail,
                Decode {
                    mode: Mode::Batch,
                    ..opts
                }
            )),
            ErrorCode::InvalidInput
        );
    }
    let strict = Decode {
        profile: Uleb128,
        mode: Mode::Batch,
        allow_noncanonical: false,
    };
    for data in [
        vec![0x80],
        vec![0x80, 0],
        vec![0x80; 11],
        hex_decode("ffffffffffffffffff02").unwrap(),
    ] {
        assert_eq!(
            error(varint::decode(&data, strict)),
            ErrorCode::InvalidInput
        );
    }
    for profile in [Uleb128, Sleb128, Zigzag64] {
        assert!(
            varint::decode(
                &[0x80, 0],
                Decode {
                    profile,
                    allow_noncanonical: true,
                    ..strict
                }
            )
            .is_ok()
        );
    }
    let full = Decode {
        mode: Mode::Full,
        ..strict
    };
    assert_eq!(varint::decode(&[0], full).unwrap()[0].consumed, 1);
    assert_eq!(
        error(varint::decode(&[0, 1], full)),
        ErrorCode::InvalidInput
    );
    assert_eq!(varint::decode(&[0, 1], strict).unwrap().len(), 2);
    let report_len = ((1 << 20) - 1) / 25;
    assert_eq!(
        varint::decode(&vec![0; report_len], strict).unwrap().len(),
        report_len
    );
    assert_eq!(
        error(varint::decode(&vec![0; report_len + 1], strict)),
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        error(varint::decode(
            &vec![0; (1 << 20) + 1],
            Decode {
                mode: Mode::Prefix,
                ..strict
            }
        )),
        ErrorCode::ResourceLimit
    );
}

#[test]
fn strict_rfc4648_profiles_and_legacy() {
    use rfc::{Base64Url, Padding};
    for (decoded, encoded) in [
        (b"".as_slice(), ""),
        (b"f", "CO======"),
        (b"fo", "CPNG===="),
        (b"foo", "CPNMU==="),
        (b"foob", "CPNMUOG="),
        (b"fooba", "CPNMUOJ1"),
        (b"foobar", "CPNMUOJ1E8======"),
    ] {
        assert_eq!(rfc::encode_base32hex(decoded).unwrap(), encoded);
        assert_eq!(rfc::decode_base32hex(encoded).unwrap(), decoded);
    }
    for invalid in [
        "co======",
        "CPNMU",
        "CO======CPNG====",
        "CP======",
        "CO======\n",
        "WO======",
        "CO=====",
    ] {
        assert_eq!(
            error(rfc::decode_base32hex(invalid)),
            ErrorCode::InvalidInput
        );
    }
    let pad = Base64Url {
        padding: Padding::Required,
    };
    let bare = Base64Url {
        padding: Padding::Forbidden,
    };
    for (options, text) in [(pad, "-_8="), (bare, "-_8")] {
        assert_eq!(rfc::encode_base64url(&[0xfb, 0xff], options).unwrap(), text);
        assert_eq!(rfc::decode_base64url(text, options).unwrap(), &[0xfb, 0xff]);
        for bad in ["+/8=", "-_9=", "-_9", "-_8==", "-_8=\n", "+_8="] {
            assert_eq!(
                error(rfc::decode_base64url(bad, options)),
                ErrorCode::InvalidInput
            );
        }
    }
    assert!(rfc::decode_base64url("-_8=", bare).is_err());
    assert!(rfc::decode_base64url("-_8", pad).is_err());
    assert_eq!(
        cube_core::encoding::base32_decode("MZXW6===").unwrap(),
        b"foo"
    );
    assert_eq!(
        cube_core::encoding::base64_decode("+/8=").unwrap(),
        &[0xfb, 0xff]
    );
    let mut bytes = vec![255; 1 << 20];
    assert_eq!(
        rfc::decode_base32hex(&rfc::encode_base32hex(&bytes).unwrap()).unwrap(),
        bytes
    );
    assert_eq!(
        rfc::decode_base64url(&rfc::encode_base64url(&bytes, pad).unwrap(), pad).unwrap(),
        bytes
    );
    bytes.push(255);
    assert_eq!(
        error(rfc::encode_base32hex(&bytes)),
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        error(rfc::decode_base64url(&"A".repeat((2 << 20) + 1), bare)),
        ErrorCode::ResourceLimit
    );
}

#[test]
fn hashes_match_independent_reference_implementations() {
    use hash::{Options, Profile::*};
    let data: Vec<_> = [255].into_iter().chain(0..=63).collect();
    let digest = |n: usize, profile, seed| {
        hash::noncrypto(&data[1..n + 1], Options { profile, seed })
            .unwrap()
            .hex
    };
    for line in include_str!("fixtures/binary/xxhash-vectors.txt").lines() {
        let [len, h32, h64]: [&str; 3] = line
            .split_ascii_whitespace()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let len = len.parse().unwrap();
        assert_eq!(digest(len, Xxh32, 0x1234_5678), h32);
        assert_eq!(digest(len, Xxh64, 0xfedc_ba98_7654_3210), h64);
    }
    for line in include_str!("fixtures/binary/murmur-vectors.txt").lines() {
        let [len, seed, h32, h128]: [&str; 4] = line
            .split_ascii_whitespace()
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        let (len, seed) = (len.parse().unwrap(), seed.parse().unwrap());
        assert_eq!(digest(len, Murmur3X86_32, seed), h32);
        assert_eq!(digest(len, Murmur3X64_128, seed), h128);
    }
    assert_eq!(digest(0, Xxh32, 0), "02cc5d05");
    assert_eq!(digest(0, Xxh64, 0), "ef46db3751d8e999");
    for profile in [Xxh32, Murmur3X86_32, Murmur3X64_128] {
        assert_eq!(
            error(hash::noncrypto(
                b"",
                Options {
                    profile,
                    seed: 1 << 32
                }
            )),
            ErrorCode::InvalidParameter
        );
    }
    let key = &data[1..17];
    for (n, vector) in include_str!("fixtures/binary/siphash24.txt")
        .lines()
        .enumerate()
    {
        assert_eq!(
            hash::siphash24(&data[1..n + 1], key).unwrap().hex,
            vector,
            "prefix {n}"
        );
    }
    for key in [&data[..15], &data[..17]] {
        assert_eq!(
            error(hash::siphash24(b"", key)),
            ErrorCode::InvalidParameter
        );
    }
    let at = vec![0; 16 << 20];
    assert!(hash::siphash24(&at, key).is_ok());
    assert!(
        hash::noncrypto(
            &at,
            Options {
                profile: Xxh64,
                seed: 0
            }
        )
        .is_ok()
    );
    let over = vec![0; (16 << 20) + 1];
    assert_eq!(error(hash::siphash24(&over, key)), ErrorCode::ResourceLimit);
    assert_eq!(
        error(hash::noncrypto(
            &over,
            Options {
                profile: Xxh64,
                seed: 0
            }
        )),
        ErrorCode::ResourceLimit
    );
}

const QR: rs::Options = rs::Options {
    field: rs::FieldProfile::QrRsGf256,
    parity_bytes: 10,
    work_limit: None,
};
const MSG: &str = "40548454c4c4f0ec11ec11ec11ec11ec";

#[test]
fn reed_solomon_matches_nayuki_and_corrects_shortening() {
    let data = hex_decode(MSG).unwrap();
    let clean = rs::encode(&[&data], QR).unwrap().remove(0);
    assert_eq!(clean.parity, hex_decode("23732399ec08c9f737df").unwrap());
    assert_eq!(clean.codeword, [data.as_slice(), &clean.parity].concat());
    let padded = [vec![0; 40], data.clone()].concat();
    assert_eq!(rs::encode(&[&padded], QR).unwrap()[0].parity, clean.parity);
    let mut damaged = clean.codeword.clone();
    damaged[3] ^= 43;
    damaged[25] ^= 42;
    let row = rs::Damaged {
        codeword: &damaged,
        erasures: &[25],
    };
    let fixed = rs::decode(&[row], QR).unwrap().remove(0);
    assert_eq!(fixed.message, data);
    assert_eq!(fixed.codeword, clean.codeword);
    assert_eq!(fixed.positions, [3, 25]);
    assert_eq!(
        fixed.initial_syndromes,
        [1, 6, 170, 26, 32, 224, 200, 199, 104, 251]
    );
    assert_eq!(fixed.verified_syndromes, [0; 10]);
    let alternate = rs::encode(&[b"alternate"], QR).unwrap().remove(0);
    let report = rs::decode(
        &[rs::Damaged {
            codeword: &alternate.codeword,
            erasures: &[0],
        }],
        QR,
    )
    .unwrap()
    .remove(0);
    assert_eq!(report.message, b"alternate");
    assert!(report.positions.is_empty());
}

#[test]
fn reed_solomon_error_erasure_limits_and_custom_roots() {
    for (field, parity) in [
        (rs::FieldProfile::QrRsGf256, 2),
        (rs::FieldProfile::QrRsGf256, 64),
        (
            rs::FieldProfile::Custom {
                polynomial: 0x11b,
                generator: 3,
                first_root: 253,
            },
            16,
        ),
    ] {
        let options = rs::Options {
            field,
            parity_bytes: parity,
            ..QR
        };
        let data: Vec<_> = (0..255 - parity)
            .map(|v| (v as u8).wrapping_mul(73))
            .collect();
        let clean = rs::encode(&[&data], options).unwrap().remove(0);
        for erase in 0..=parity {
            let errors = (parity - erase) / 2;
            let mut damaged = clean.codeword.clone();
            let positions: Vec<_> = (0..erase + errors).map(|i| 3 * i).collect();
            for &pos in &positions {
                damaged[pos] ^= (pos as u8) | 1;
            }
            let mut erased = positions[..erase].to_vec();
            erased.reverse();
            let fixed = rs::decode(
                &[rs::Damaged {
                    codeword: &damaged,
                    erasures: &erased,
                }],
                options,
            )
            .unwrap()
            .remove(0);
            assert_eq!(
                fixed.positions, positions,
                "p={parity}, e={errors}, s={erase}"
            );
            assert_eq!(fixed.message, data);
            assert_eq!(fixed.verified_syndromes, vec![0; parity]);
        }
        let mut damaged = clean.codeword.clone();
        for byte in damaged.iter_mut().take(parity / 2 + 1) {
            *byte ^= 1;
        }
        assert_eq!(
            error(rs::decode(
                &[rs::Damaged {
                    codeword: &damaged,
                    erasures: &[]
                }],
                options
            )),
            ErrorCode::InvalidInput
        );
        for erasures in [&[0, 0][..], &[255][..]] {
            assert_eq!(
                error(rs::decode(
                    &[rs::Damaged {
                        codeword: &clean.codeword,
                        erasures
                    }],
                    options
                )),
                ErrorCode::InvalidInput
            );
        }
    }
    for field in [
        rs::FieldProfile::Custom {
            polynomial: 0x100,
            generator: 2,
            first_root: 0,
        },
        rs::FieldProfile::Custom {
            polynomial: 0x11d,
            generator: 1,
            first_root: 0,
        },
        rs::FieldProfile::Custom {
            polynomial: 0x11d,
            generator: 2,
            first_root: 255,
        },
    ] {
        assert_eq!(
            error(rs::encode(&[b"a"], rs::Options { field, ..QR })),
            ErrorCode::InvalidParameter
        );
    }
    let data = hex_decode(MSG).unwrap();
    assert!(
        rs::encode(
            &[&data],
            rs::Options {
                work_limit: Some(470),
                ..QR
            }
        )
        .is_ok()
    );
    assert_eq!(
        error(rs::encode(
            &[&data],
            rs::Options {
                work_limit: Some(469),
                ..QR
            }
        )),
        ErrorCode::ResourceLimit
    );
    assert_eq!(
        error(rs::encode(
            &[&data],
            rs::Options {
                work_limit: Some((16 << 20) + 1),
                ..QR
            }
        )),
        ErrorCode::InvalidParameter
    );
    assert!(rs::encode(&vec![b"a".as_slice(); 4096], QR).is_ok());
    assert_eq!(
        error(rs::encode(&vec![b"a".as_slice(); 4097], QR)),
        ErrorCode::ResourceLimit
    );
}

#[test]
fn all_binary_fixtures_execute_and_private_policy_is_direct_only() {
    let ids: Vec<_> = operations::representation::OPERATIONS
        .iter()
        .map(|entry| entry.id)
        .collect();
    let fixtures = operations::contracts::registered_fixtures().unwrap();
    assert_eq!(ids.len(), 19);
    for id in ids {
        let (_, fixture) = fixtures
            .iter()
            .find(|(operation, _)| operation == id)
            .unwrap();
        let result = execution::invoke(
            id,
            1,
            &fixture.inputs,
            &fixture.parameters,
            execution::Budget::default(),
        )
        .unwrap_or_else(|err| panic!("{id}: {err}"));
        assert_eq!(result, fixture.outputs, "{id}");
    }
    let contract = operations::require("hash.siphash24")
        .unwrap()
        .contract()
        .unwrap();
    assert!(contract.policy.private_input && contract.policy.private_output);
    assert!(!contract.policy.automatic_capture);
    assert_eq!(contract.pipeline_adapter, None);
    let bad_key = json!({"base64":"c2VjcmV0LWtleS1tYXRlcmlhbA=="});
    let err = execution::invoke(
        "hash.siphash24",
        1,
        &json!({"data":{"base64":""},"key":bad_key}),
        &json!({}),
        execution::Budget::default(),
    )
    .unwrap_err();
    assert!(!err.to_string().contains("c2VjcmV0"));
}

#[test]
fn bounded_bytes_text_pipeline_adapters_reuse_direct_formats() {
    let plan = |input: &str, steps: Value| {
        pipeline::Plan::validate(
            input.parse().unwrap(),
            serde_json::from_value(steps).unwrap(),
            pipeline::Limits::default(),
        )
    };
    let original = pipeline::Value::Bytes {
        data: vec![0, 0xfb, 0xff, 0xff],
    };
    for alphabet in ["base32hex", "base64url"] {
        let parameters = if alphabet == "base64url" {
            json!({"padding":"forbidden"})
        } else {
            json!({})
        };
        let steps = json!([
            {"operation":format!("encoding.{alphabet}.encode"),"version":1,"parameters":parameters},
            {"operation":format!("encoding.{alphabet}.decode"),"version":1,"parameters":parameters},
        ]);
        let plan = plan("bytes", steps).unwrap();
        assert_eq!(
            plan.execute(original.clone(), &[], None).unwrap().output,
            original
        );
    }
    let step = |seed| json!([{"operation":"hash.noncrypto","version":1,"parameters":{"profile":"xxh64","seed":seed}}]);
    let hash = plan("bytes", step("18364758544493064720"))
        .unwrap()
        .execute(pipeline::Value::Bytes { data: Vec::new() }, &[], None)
        .unwrap();
    assert_eq!(
        hash.output,
        pipeline::Value::Text {
            data: "7cc8df76db892f66".into()
        }
    );
    assert!(plan("bytes", step("018364758544493064720")).is_err());
    assert!(plan("bytes", json!([{"operation":"hash.siphash24","version":1}])).is_err());
}
