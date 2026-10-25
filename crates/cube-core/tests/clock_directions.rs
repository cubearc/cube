use cube_core::symbols::clock;
#[test]
fn times_continuous_hands_and_orientation_are_explicit() {
    for (time, direction) in [("12:00", "N"), ("3:00", "E"), ("6:00", "S"), ("9:00", "W")] {
        let r = clock::inspect(Some(time), None, None, "N", true, false).unwrap();
        assert_eq!(r.hour.directions, [direction]);
        assert_eq!(r.minute.directions, ["N"]);
    }
    assert_eq!(
        clock::inspect(Some("2:59"), None, None, "N", true, false)
            .unwrap()
            .hour
            .directions,
        ["E"]
    );
    assert_eq!(
        clock::inspect(Some("2:00"), None, None, "N", true, false)
            .unwrap()
            .hour
            .directions,
        ["NE"]
    );
    assert!(
        clock::inspect(Some("12:45"), None, None, "N", true, false)
            .unwrap()
            .hour
            .ambiguous
    );
    assert_eq!(
        clock::inspect(Some("3:00"), None, None, "N", true, true)
            .unwrap()
            .decoded
            .as_deref(),
        Some("J")
    );
    assert_eq!(
        clock::inspect(None, Some(45000), Some(0), "W", false, false)
            .unwrap()
            .hour
            .directions,
        ["SW"]
    );
    assert_eq!(
        clock::inspect(None, Some(-45000), Some(360000), "N", true, false)
            .unwrap()
            .hour
            .directions,
        ["NW"]
    );
}
#[test]
fn boundaries_and_invalid_modes_never_force_a_symbol() {
    for (angle, ambiguous) in [
        (22498, false),
        (22499, true),
        (22500, true),
        (22501, true),
        (22502, false),
    ] {
        let r = clock::inspect(None, Some(angle), Some(0), "N", true, false).unwrap();
        assert_eq!(r.hour.ambiguous, ambiguous);
        if ambiguous {
            assert_eq!(r.hour.directions, ["N", "NE"]);
            assert!(clock::inspect(None, Some(angle), Some(0), "N", true, true).is_err());
        }
    }
    for time in [
        "13:00", "3:60", "3:00:00", "+3:00", " 3:00", "3:0", "", "NaN",
    ] {
        assert!(clock::inspect(Some(time), None, None, "N", true, false).is_err());
    }
    assert!(clock::inspect(None, None, None, "N", true, false).is_err());
    assert!(clock::inspect(Some("3:00"), Some(0), Some(0), "N", true, false).is_err());
    assert!(clock::inspect(None, Some(0), None, "N", true, false).is_err());
    assert!(clock::inspect(Some("3:00"), None, None, "unknown", true, false).is_err());
    assert!(clock::inspect(Some("12:00"), None, None, "N", true, true).is_err());
    let r = clock::inspect(None, Some(i64::MIN), Some(i64::MAX), "E", false, false).unwrap();
    assert!(r.hour.bearing_millidegrees < 360000 && r.minute.bearing_millidegrees < 360000);
}
