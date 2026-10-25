use cube_core::{
    binary::{self, transform as t},
    errors::ErrorCode,
    pipeline::{Kind, Limits, Parameter, Plan, Step, Value},
};
#[test]
fn byte_logic_reverse_and_endian_round_trips_are_exact() {
    let bytes = [0, 255, 0x55, 0xaa];
    let key = [255, 15, 0xaa, 0x55];
    assert_eq!(t::and(&bytes, &key).unwrap(), [0, 15, 0, 0]);
    assert_eq!(t::or(&bytes, &key).unwrap(), [255; 4]);
    assert_eq!(t::not(&bytes).unwrap(), [255, 0, 0xaa, 0x55]);
    assert_eq!(t::reverse(&bytes).unwrap(), [0xaa, 0x55, 255, 0]);
    assert!(t::and(&[], &[]).unwrap().is_empty());
    assert!(t::or(&[], &[]).unwrap().is_empty());
    assert!(t::and(&bytes, &[0]).is_err());
    assert!(t::or(&[], &[0]).is_err());
    assert_eq!(t::not(&t::not(&bytes).unwrap()).unwrap(), bytes);
    assert_eq!(t::reverse(&t::reverse(&bytes).unwrap()).unwrap(), bytes);
    let bytes: Vec<_> = (0..32).collect();
    for width in [1, 2, 4, 8] {
        assert_eq!(
            t::swap_endian(&t::swap_endian(&bytes, width).unwrap(), width).unwrap(),
            bytes
        );
    }
    assert_eq!(t::swap_endian(b"ABCD", 2).unwrap(), b"BADC");
    assert!(binary::xor(&[], &[]).is_err());
}
#[test]
fn shifts_have_fixed_unsigned_width_carries_padding_and_overflow_rules() {
    assert_eq!(t::shift_left(&[1, 255], 1, 2).unwrap(), [3, 254]);
    assert_eq!(t::shift_left(&[1, 255], 1, 1).unwrap(), [2, 254]);
    assert_eq!(t::shift_right(&[1, 0], 1, 2).unwrap(), [0, 128]);
    assert_eq!(t::shift_right(&[1, 0], 1, 1).unwrap(), [0, 0]);
    assert_eq!(t::shift_left(&[128, 1], 1, 2).unwrap(), [0, 2]);
    assert_eq!(t::shift_left(&[255], 8, 1).unwrap(), [0]);
    assert_eq!(t::shift_right(&[255], 8, 1).unwrap(), [0]);
    assert_eq!(t::shift_left(&[255], 0, 1).unwrap(), [255]);
    assert_eq!(
        t::shift_left(&1u64.to_be_bytes(), 63, 8).unwrap(),
        (1u64 << 63).to_be_bytes()
    );
    assert_eq!(
        t::shift_right(&(1u64 << 63).to_be_bytes(), 63, 8).unwrap(),
        1u64.to_be_bytes()
    );
    assert_eq!(
        t::shift_left(&u64::MAX.to_be_bytes(), 64, 8).unwrap(),
        [0; 8]
    );
    assert!(t::shift_left(&[], 9, 1).is_err());
    assert!(t::shift_right(&[1], 0, 2).is_err());
    for width in [0, 3, 16] {
        assert!(t::swap_endian(&[], width).is_err());
    }
    assert!(t::shift_left(&[], 0, 1).unwrap().is_empty());
}
#[test]
fn allocation_and_pipeline_parameter_bounds_fail_safely() {
    let big = vec![0; t::MAX_INPUT + 1];
    for result in [
        t::and(&big, &[]),
        t::and(&[], &big),
        t::or(&big, &[]),
        t::not(&big),
        t::reverse(&big),
        t::shift_left(&big, 1, 1),
        t::swap_endian(&big, 1),
    ] {
        assert_eq!(result.unwrap_err().code, ErrorCode::ResourceLimit);
    }
    assert_eq!(t::not(&big[..t::MAX_INPUT]).unwrap().len(), t::MAX_INPUT);
    let plan = Plan::validate(
        Kind::Bytes,
        vec![Step::new("binary.not"), Step::new("binary.reverse")],
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        plan.execute(Value::Bytes { data: vec![0, 255] }, &[], None)
            .unwrap()
            .output,
        Value::Bytes { data: vec![0, 255] }
    );
    let mut shift = Step::new("binary.shift_left");
    shift
        .parameters
        .insert("bits".into(), Parameter::Integer(9));
    assert!(Plan::validate(Kind::Bytes, vec![shift.clone()], Limits::default()).is_err());
    shift
        .parameters
        .insert("unit_bytes".into(), Parameter::Integer(2));
    let plan = Plan::validate(Kind::Bytes, vec![shift], Limits::default()).unwrap();
    assert!(
        plan.execute(Value::Bytes { data: vec![1] }, &[], None)
            .is_err()
    );
    let mut and = Step::new("binary.and");
    and.parameters.insert(
        "key".into(),
        Parameter::Bytes {
            data: vec![255; 65537],
        },
    );
    assert!(Plan::validate(Kind::Bytes, vec![and.clone()], Limits::default()).is_err());
    and.parameters
        .insert("key".into(), Parameter::Bytes { data: vec![15] });
    let plan = Plan::validate(Kind::Bytes, vec![and], Limits::default()).unwrap();
    assert_eq!(
        plan.execute(Value::Bytes { data: vec![255] }, &[], None)
            .unwrap()
            .output,
        Value::Bytes { data: vec![15] }
    );
}
