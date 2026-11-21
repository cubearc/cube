use cube_core::{
    binary::compression::{bzip2, lz4},
    errors::ErrorCode,
};

#[test]
fn lz4_raw_matches_a_golden_literal_block() {
    let wire = [0x30, b'a', b'b', b'c'];
    let limits = lz4::Limits::default();
    assert_eq!(
        lz4::decode(&wire, lz4::Decode::Block { expected_length: 3 }, limits).unwrap(),
        b"abc"
    );
    assert_eq!(lz4::encode(b"abc", lz4::Mode::Block, limits).unwrap(), wire);
    assert!(lz4::decode(&wire, lz4::Decode::Block { expected_length: 4 }, limits).is_err());
    assert_eq!(
        lz4::decode(
            &wire,
            lz4::Decode::Block { expected_length: 3 },
            lz4::Limits {
                max_output: 2,
                ..limits
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn lz4_frame_validates_bounds_and_all_selected_checksums() {
    let data = vec![b'x'; 70_000];
    let limits = lz4::Limits::default();
    let encoded = lz4::encode(&data, lz4::Mode::Frame, limits).unwrap();
    assert_eq!(
        lz4::decode(&encoded, lz4::Decode::Frame, limits).unwrap(),
        data
    );
    assert_eq!(
        lz4::decode(
            &encoded,
            lz4::Decode::Frame,
            lz4::Limits {
                max_blocks: 1,
                ..limits
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::ResourceLimit
    );
    let mut changed = encoded.clone();
    changed[14] ^= 1;
    assert!(lz4::decode(&changed, lz4::Decode::Frame, limits).is_err());
    let mut changed = encoded.clone();
    *changed.last_mut().unwrap() ^= 1;
    assert!(lz4::decode(&changed, lz4::Decode::Frame, limits).is_err());
    assert!(lz4::decode(&encoded.repeat(2), lz4::Decode::Frame, limits).is_err());
}

#[test]
fn bzip2_reproduces_the_reference_stream_and_checks_concatenation() {
    let known =
        hex::decode("425a6831314159265359648cbb73000000010038002000219819846177245385090648cbb730")
            .unwrap();
    assert_eq!(bzip2::decode(&known, false).unwrap(), b"abc");
    assert_eq!(bzip2::encode(b"abc", 1).unwrap(), known);
    assert_eq!(bzip2::decode(&known.repeat(2), true).unwrap(), b"abcabc");
    assert!(bzip2::decode(&known.repeat(2), false).is_err());
    let mut broken = known.clone();
    broken[10] ^= 1;
    assert!(bzip2::decode(&broken, false).is_err());
    assert!(bzip2::decode(&known[..known.len() - 1], false).is_err());
    assert!(bzip2::encode(b"abc", 0).is_err());
}

#[test]
fn bzip2_backend_charges_a_block_before_starting_it() {
    use cube_bzip2_backend::{BoundedError, decode_bounded};
    let empty = bzip2::encode(b"", 1).unwrap();
    assert_eq!(
        decode_bounded(&empty, 0, 0).unwrap(),
        (Vec::new(), empty.len(), 0)
    );
    let one = bzip2::encode(b"abc", 1).unwrap();
    assert_eq!(decode_bounded(&one, 3, 0).unwrap_err(), BoundedError::Limit);
    assert_eq!(decode_bounded(&one, 2, 1).unwrap_err(), BoundedError::Limit);
    assert_eq!(
        decode_bounded(&one, 3, 1).unwrap(),
        (b"abc".to_vec(), one.len(), 1)
    );
}
