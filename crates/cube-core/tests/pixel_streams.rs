use cube_core::{
    image::{Channel, ColorModel, Corner, Image, Region, Traversal},
    pipeline::{Kind, Limits, Plan, Step, Value},
};
#[test]
fn ordered_channel_streams_reuse_coordinates_and_feed_typed_pipelines() {
    let image = Image::from_bytes(
        2,
        2,
        ColorModel::Rgba,
        &(1..=16).collect::<Vec<u8>>(),
        &[],
        &[],
    )
    .unwrap();
    let stream = image
        .pixels_with_traversal(
            None,
            Some(&[Channel::Alpha, Channel::Red]),
            Traversal::SpiralCw,
            Corner::TopLeft,
            false,
        )
        .unwrap();
    assert_eq!(stream.pixels.data, [4, 1, 8, 5, 16, 13, 12, 9]);
    let records = stream.records().unwrap();
    assert_eq!((records[2].x, records[2].y), (1, 1));
    assert_eq!(records[2].values, [16, 13]);
    let plan = Plan::validate(
        Kind::Bytes,
        vec![Step::new("encoding.hex.encode")],
        Limits::default(),
    )
    .unwrap();
    assert_eq!(
        plan.execute(
            Value::Bytes {
                data: stream.pixels.data
            },
            &[],
            None
        )
        .unwrap()
        .output,
        Value::Text {
            data: "04010805100d0c09".into()
        }
    );
    assert_eq!(image.pixels(None, None).unwrap().data, image.samples());
    for name in cube_core::image::traversal::NAMES {
        let traversal = Traversal::parse(name).unwrap();
        let channels = vec![Channel::Blue, Channel::Red];
        let stream = image
            .pixels_with_traversal(None, Some(&channels), traversal, Corner::BottomLeft, true)
            .unwrap();
        let bits = image
            .lsb_with_corner(
                &cube_core::image::LsbOptions {
                    channels: Some(channels),
                    bits: (0..8).rev().collect(),
                    traversal,
                    reverse: true,
                    ..Default::default()
                },
                Corner::BottomLeft,
            )
            .unwrap();
        assert_eq!(stream.pixels.data, bits.result.data);
    }
    let region = Region {
        x: 1,
        y: 0,
        width: 1,
        height: 2,
    };
    let stream = image
        .pixels_with_traversal(
            Some(region),
            None,
            Traversal::Column,
            Corner::BottomRight,
            false,
        )
        .unwrap();
    assert_eq!(
        (
            stream.records().unwrap()[0].x,
            stream.records().unwrap()[0].y
        ),
        (1, 1)
    );
    assert!(
        image
            .pixels_with_traversal(
                None,
                Some(&[Channel::Index]),
                Traversal::Row,
                Corner::TopLeft,
                false
            )
            .is_err()
    );
    let large = Image::from_bytes(10001, 1, ColorModel::Gray, &vec![0; 10001], &[], &[]).unwrap();
    assert!(
        large
            .pixels_with_traversal(None, None, Traversal::Row, Corner::TopLeft, false)
            .unwrap()
            .records()
            .is_err()
    );
}
