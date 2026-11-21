use cube_core::{
    binary::compression::{huffman as h, lzw},
    errors::ErrorCode,
};

#[test]
fn canonical_huffman_ties_singleton_and_wire_bits() {
    let mut counts = [0; 256];
    counts[0..4].fill(7);
    let codes = h::construct(&counts).unwrap();
    assert_eq!(
        codes
            .iter()
            .map(|c| (c.symbol, c.length, c.code))
            .collect::<Vec<_>>(),
        [(0, 2, 0), (1, 2, 1), (2, 2, 2), (3, 2, 3),]
    );
    let data = [0, 1, 2, 3, 1];
    let result = h::encode(&data, &counts).unwrap();
    assert_eq!(result.data, [0x1b, 0x40]);
    assert_eq!(result.bit_count, 10);
    let lengths: Vec<_> = result
        .codes
        .iter()
        .map(|c| h::Length {
            symbol: c.symbol,
            length: c.length,
        })
        .collect();
    assert_eq!(
        h::decode(&result.data, &lengths, 10, data.len()).unwrap(),
        data
    );
    assert_eq!(
        h::decode(&result.data, &lengths[1..], 10, data.len())
            .unwrap_err()
            .code,
        ErrorCode::InvalidParameter
    );
    let singleton = [h::Length {
        symbol: 42,
        length: 1,
    }];
    assert_eq!(h::decode(&[0], &singleton, 3, 3).unwrap(), [42; 3]);
    assert!(h::decode(&[1], &singleton, 3, 3).is_err());
    assert!(h::decode(&[0x80], &singleton, 1, 1).is_err());
    assert_eq!(h::decode(&[], &[], 0, 0).unwrap(), Vec::<u8>::new());
    counts[..2].fill(u64::MAX);
    assert_eq!(
        h::construct(&counts).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
}

#[test]
fn lzw_gif_and_tiff_code_packing_and_end_policy() {
    let gif = lzw::Profile::Gif { min_code_size: 2 };
    let tiff = lzw::Profile::Tiff;
    // clear, 0, 1, 6, end (3, 3, 3, 3, 4 bits, LSB first).
    let gif_wire = [0x44, 0x5c];
    assert_eq!(lzw::encode(&[0, 1, 0, 1], gif).unwrap(), gif_wire);
    assert_eq!(lzw::decode(&gif_wire, gif).unwrap(), [0, 1, 0, 1]);
    // clear, 65, 258, end, each 9 bits, MSB first.
    let tiff_wire = [0x80, 0x10, 0x60, 0x50, 0x10];
    assert_eq!(lzw::encode(b"AAA", tiff).unwrap(), tiff_wire);
    assert_eq!(lzw::decode(&tiff_wire, tiff).unwrap(), b"AAA");
    assert_eq!(
        lzw::decode(&lzw::encode(&[], tiff).unwrap(), tiff).unwrap(),
        Vec::<u8>::new()
    );
    assert!(lzw::encode(&[4], gif).is_err());
    assert!(lzw::decode(&[0x44], gif).is_err());
    assert!(lzw::decode(&[0x80, 0x10, 0x60, 0x50, 0x11], tiff).is_err());
    assert!(lzw::decode(&[0x44, 0x5c, 0], gif).is_err());
}

#[test]
fn lzw_exercises_all_widths_and_a_frozen_gif_dictionary() {
    let mut state = 7u32;
    let data: Vec<u8> = (0..32_000)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state >> 24) as u8
        })
        .collect();
    for profile in [lzw::Profile::Gif { min_code_size: 8 }, lzw::Profile::Tiff] {
        let compressed = lzw::encode(&data, profile).unwrap();
        assert_eq!(lzw::decode(&compressed, profile).unwrap(), data);
        assert!(lzw::decode(&compressed[..compressed.len() - 1], profile).is_err());
    }
}

#[test]
fn huffman_rejects_overlong_and_oversubscribed_tables() {
    let mut frequencies = [0u64; 256];
    frequencies[..2].fill(1);
    for i in 2..35 {
        frequencies[i] = frequencies[i - 1] + frequencies[i - 2];
    }
    assert_eq!(
        h::construct(&frequencies).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let invalid = [
        h::Length {
            symbol: 0,
            length: 1,
        },
        h::Length {
            symbol: 1,
            length: 1,
        },
        h::Length {
            symbol: 2,
            length: 1,
        },
    ];
    assert_eq!(
        h::canonical(&invalid).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
}
