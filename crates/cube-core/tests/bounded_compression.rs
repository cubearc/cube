use cube_core::{
    binary::compression::{dictionary as lz, packbits, transforms},
    errors::ErrorCode,
};

#[test]
fn packbits_wire_controls_ties_and_limits() {
    let plain = b"QQQXXYZZZZ";
    let compressed = [254, b'Q', 2, b'X', b'X', b'Y', 253, b'Z'];
    assert_eq!(packbits::encode(plain).unwrap(), compressed);
    assert_eq!(packbits::decode(&compressed, Some(10)).unwrap(), plain);
    assert_eq!(
        packbits::decode(&[128, 255, 42, 128], None).unwrap(),
        [42; 2]
    );
    assert_eq!(packbits::encode(&[7; 130]).unwrap(), [129, 7, 1, 7, 7]);
    assert_eq!(packbits::encode(&[]).unwrap(), Vec::<u8>::new());
    let all: Vec<u8> = (0..=255).collect();
    assert_eq!(
        packbits::decode(&packbits::encode(&all).unwrap(), None).unwrap(),
        all
    );
    assert_eq!(
        packbits::decode(&[3, 0], None).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_eq!(
        packbits::decode(&compressed, Some(11)).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_eq!(
        packbits::encode(&vec![0; (1 << 20) + 1]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let bomb = [129, 0].repeat((1 << 17) + 1);
    assert_eq!(
        packbits::decode(&bomb, None).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
}

#[test]
fn bwt_matches_small_rotation_oracle_including_periodic_ties() {
    for plain in [
        b"BANANA".as_slice(),
        b"ABABABAB",
        b"zzzz",
        b"",
        b"MISSISSIPPI",
        &[0, 255, 0, 1],
    ] {
        let mut rotations: Vec<_> = (0..plain.len())
            .map(|i| {
                (
                    plain[i..]
                        .iter()
                        .chain(&plain[..i])
                        .copied()
                        .collect::<Vec<_>>(),
                    i,
                )
            })
            .collect();
        rotations.sort();
        let last: Vec<_> = rotations
            .iter()
            .map(|(bytes, _)| bytes[bytes.len() - 1])
            .collect();
        let first = rotations.iter().position(|(_, i)| *i == 0).unwrap_or(0);
        let (actual, primary) = transforms::bwt(plain).unwrap();
        assert_eq!((actual.as_slice(), primary), (last.as_slice(), first));
        assert_eq!(transforms::inverse_bwt(&actual, primary).unwrap(), plain);
    }
    assert_eq!(
        transforms::inverse_bwt(b"abc", 3).unwrap_err().code,
        ErrorCode::InvalidParameter
    );
    assert_eq!(
        transforms::bwt(&[0; 65_537]).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let input: Vec<_> = (0..=255).cycle().take(65_536).collect();
    let (last, primary) = transforms::bwt(&input).unwrap();
    assert_eq!(transforms::inverse_bwt(&last, primary).unwrap(), input);
}

#[test]
fn move_to_front_has_exact_initial_alphabet_semantics() {
    let bytes = b"banana";
    assert_eq!(
        transforms::mtf(bytes, None, false).unwrap(),
        [98, 98, 110, 1, 1, 1]
    );
    let reverse: Vec<u8> = (0..=255).rev().collect();
    let transformed = transforms::mtf(bytes, Some(&reverse), false).unwrap();
    assert_eq!(
        transforms::mtf(&transformed, Some(&reverse), true).unwrap(),
        bytes
    );
    assert_eq!(
        transforms::mtf(bytes, Some(&[0; 256]), false)
            .unwrap_err()
            .code,
        ErrorCode::InvalidParameter
    );
}

#[test]
fn lz77_overlap_and_lz78_final_suffix_are_exact() {
    use lz::Lz77Token::{Literal, Match};
    let matching = vec![
        Literal { byte: b'A' },
        Match {
            distance: 1,
            length: 5,
        },
    ];
    assert_eq!(lz::lz77_encode(b"AAAAAA", 4096).unwrap(), matching);
    assert_eq!(lz::lz77_decode(&matching, 4096).unwrap(), b"AAAAAA");
    assert_eq!(
        lz::lz77_decode(
            &[Match {
                distance: 1,
                length: 3
            }],
            4096
        )
        .unwrap_err()
        .code,
        ErrorCode::InvalidInput
    );
    let phrase = vec![
        lz::Lz78Token {
            index: 0,
            byte: Some(b'A'),
        },
        lz::Lz78Token {
            index: 1,
            byte: Some(b'A'),
        },
        lz::Lz78Token {
            index: 2,
            byte: Some(b'A'),
        },
        lz::Lz78Token {
            index: 1,
            byte: None,
        },
    ];
    assert_eq!(lz::lz78_encode(b"AAAAAAA").unwrap(), phrase);
    assert_eq!(lz::lz78_decode(&phrase).unwrap(), b"AAAAAAA");
    assert_eq!(
        lz::lz78_decode(&phrase[2..]).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_eq!(
        lz::lz78_decode(&[lz::Lz78Token {
            index: 0,
            byte: None
        }])
        .unwrap_err()
        .code,
        ErrorCode::InvalidInput
    );
}
