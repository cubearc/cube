use cube_core::binary::compression::probability::{self, Profile};

#[test]
fn fixed_probability_wires_are_canonical() {
    let mut counts = [0; 256];
    counts[b'A' as usize] = 1;
    counts[b'B' as usize] = 1;
    assert_eq!(
        probability::encode(b"ABBA", &counts, Profile::Arithmetic16).unwrap(),
        [0x64]
    );
    let state = probability::encode(b"ABBA", &counts, Profile::Range32).unwrap();
    assert_eq!(state, [0, 0x5f, 0xff, 0xff, 0xfe]);
    for profile in [Profile::Arithmetic16, Profile::Range32] {
        let wire = probability::encode(b"ABBA", &counts, profile).unwrap();
        assert_eq!(
            probability::decode(&wire, &counts, 4, profile).unwrap(),
            b"ABBA"
        );
        assert!(probability::decode(&wire[..wire.len() - 1], &counts, 4, profile).is_err());
        let mut trailing = wire;
        trailing.push(0);
        assert!(probability::decode(&trailing, &counts, 4, profile).is_err());
        assert!(probability::encode(b"C", &counts, profile).is_err());
        counts[0] = 16_384;
        assert!(probability::encode(b"A", &counts, profile).is_err());
        counts[0] = 0;
    }
}
