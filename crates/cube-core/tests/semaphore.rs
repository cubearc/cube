use cube_core::{
    errors::ErrorCode,
    pipeline::{Kind, Limits, Plan, Step, Value},
    symbols::semaphore as s,
};
#[test]
fn independently_referenced_letter_vectors_and_canonical_round_trips() {
    // Receiver-facing chart, with sender anatomical left/right arm identity.
    let known = "S,SW S,W S,NW S,N NE,S E,S SE,S SW,W SW,NW E,N N,SW NE,SW E,SW SE,SW NW,W N,W NE,W E,W SE,W N,NW NE,NW SE,N E,NE SE,NE E,NW E,SE";
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    assert_eq!(s::encode(alphabet).unwrap(), known);
    assert_eq!(s::decode(known).unwrap(), alphabet);
    assert_eq!(
        s::encode("abc  xyz").unwrap(),
        "S,SW S,W S,NW / / SE,NE E,NW E,SE"
    );
    assert_eq!(
        s::decode(&s::encode("abc  xyz").unwrap()).unwrap(),
        "ABC  XYZ"
    );
    assert_eq!(s::decode_pair("N", "S").unwrap(), 'D');
    assert_eq!(s::encode("D").unwrap(), "S,N");
    assert_eq!(s::encode("I K").unwrap(), "SW,NW / N,SW");
    assert!(s::decode_pair("SW", "N").is_err());
    assert_eq!(s::encode("").unwrap(), "");
    assert_eq!(s::decode("").unwrap(), "");
    for bad in ["0", "é", "A\n", "\t"] {
        assert!(s::encode(bad).is_err());
    }
    for bad in [
        "S,S",
        "SW,S",
        "NE,N",
        "x,N",
        "S,SW ",
        " S,SW",
        "S,SW  S,W",
        "S,SW\n",
        "S,SW,W",
        "/",
    ] {
        if bad == "/" {
            assert_eq!(s::decode(bad).unwrap(), " ");
        } else {
            assert!(s::decode(bad).is_err(), "{bad}");
        }
    }
    assert_eq!(
        s::encode(&"A".repeat(s::MAX_SYMBOLS + 1)).unwrap_err().code,
        ErrorCode::ResourceLimit
    );
    let encoded = s::encode(&"A".repeat(s::MAX_SYMBOLS)).unwrap();
    assert_eq!(s::decode(&encoded).unwrap().len(), s::MAX_SYMBOLS);
    assert!(s::decode(&format!("{encoded} /")).is_err());
}
#[test]
fn typed_adapters_are_bounded_and_old_engine_contracts_remain_valid() {
    let plan = Plan::validate(
        Kind::Text,
        vec![
            Step::new("symbol.semaphore.encode"),
            Step::new("symbol.semaphore.decode"),
        ],
        Limits::default(),
    )
    .unwrap();
    let output = plan
        .execute(Value::Text { data: "A B".into() }, &[0], None)
        .unwrap();
    assert_eq!(output.output, Value::Text { data: "A B".into() });
    assert_eq!(
        output.intermediates[&0],
        Value::Text {
            data: "S,SW / S,W".into()
        }
    );
    assert!(
        Plan::validate(
            Kind::Bytes,
            vec![Step::new("symbol.semaphore.encode")],
            Limits::default()
        )
        .is_err()
    );
    assert!(
        plan.execute(
            Value::Text {
                data: "A".repeat(s::MAX_SYMBOLS + 1)
            },
            &[],
            None
        )
        .is_err()
    );
}
