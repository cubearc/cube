//! Named outputs for actual channel, ordered-pixel and comparison APIs.
use crate::{
    errors::OperationError,
    image::{self, ColorModel, Image},
    operations::contracts::{self as c, Entry, Fixture, Limits, Port, schema, values},
};
use schemars::Schema;
use serde_json::json;

const IMAGE: Port = Port {
    name: "input",
    semantic_type: "image.samples",
    schema: schema::<values::Image>,
    required: true,
};
const DATA: Port = Port {
    name: "data",
    semantic_type: "bytes",
    schema: schema::<values::Bytes>,
    required: true,
};
fn pixels_parameters() -> Schema {
    c::descriptor_parameters(&super::OPERATIONS[1])
}
fn channel_parameters() -> Schema {
    c::descriptor_parameters(&super::OPERATIONS[2])
}
fn compare_parameters() -> Schema {
    c::descriptor_parameters(&super::OPERATIONS[4])
}
fn sample() -> Result<Image, OperationError> {
    Image::from_bytes(2, 1, ColorModel::Rgb, &[1, 2, 3, 10, 20, 30], &[], &[])
}
fn pixels_fixture() -> Result<Fixture, OperationError> {
    let image = sample()?;
    let pixels = image.pixels_with_traversal(
        None,
        None,
        image::Traversal::Row,
        image::Corner::TopLeft,
        false,
    )?;
    assert_eq!(pixels.pixels.data, [1, 2, 3, 10, 20, 30]);
    Ok(Fixture {
        inputs: json!({"input":values::Image::from(&image)}),
        parameters: json!({}),
        outputs: json!({"output":pixels,"data":values::Bytes {data:pixels.pixels.data.clone()}}),
    })
}
fn channel_fixture() -> Result<Fixture, OperationError> {
    let image = sample()?;
    let pixels = image.channel(image::Channel::Red, None)?;
    assert_eq!(pixels.data, [1, 10]);
    Ok(Fixture {
        inputs: json!({"input":values::Image::from(&image)}),
        parameters: json!({"channel":"red"}),
        outputs: json!({"output":pixels,"data":values::Bytes {data:pixels.data.clone()},"image":values::Image::from(&pixels.grayscale_image()?)}),
    })
}
fn compare_fixture() -> Result<Fixture, OperationError> {
    let image = sample()?;
    let comparison = image.compare(&image, None, None, 1000, true)?;
    assert!(comparison.equal);
    let mask = comparison.mask.as_ref().expect("requested mask exists");
    assert_eq!(mask.samples(), [0, 0]);
    Ok(Fixture {
        inputs: json!({"left":values::Image::from(&image),"right":values::Image::from(&image)}),
        parameters: json!({"mask":true}),
        outputs: json!({"output":comparison,"mask":values::Image::from(mask)}),
    })
}
const LIMITS: Limits = Limits {
    max_input_bytes: None,
    max_output_bytes: None,
    max_items: Some(image::MAX_PIXELS),
    max_work: None,
    max_memory_bytes: None,
};
pub(crate) const CONTRACTS: &[Entry] = &[
    Entry {
        operation_version: 1,
        descriptor: super::OPERATIONS[1],
        inputs: &[IMAGE],
        parameters: pixels_parameters,
        outputs: &[
            Port {
                name: "output",
                semantic_type: "image.pixel_data",
                schema: schema::<image::PixelStream>,
                required: true,
            },
            DATA,
        ],
        limits: LIMITS,
        fixture: pixels_fixture,
    },
    Entry {
        operation_version: 1,
        descriptor: super::OPERATIONS[2],
        inputs: &[IMAGE],
        parameters: channel_parameters,
        outputs: &[
            Port {
                name: "output",
                semantic_type: "image.pixel_data",
                schema: schema::<image::PixelData>,
                required: true,
            },
            DATA,
            Port {
                name: "image",
                required: false,
                ..IMAGE
            },
        ],
        limits: LIMITS,
        fixture: channel_fixture,
    },
    Entry {
        operation_version: 1,
        descriptor: super::OPERATIONS[4],
        inputs: &[
            Port {
                name: "left",
                ..IMAGE
            },
            Port {
                name: "right",
                ..IMAGE
            },
        ],
        parameters: compare_parameters,
        outputs: &[
            Port {
                name: "output",
                semantic_type: "image.comparison",
                schema: schema::<image::Comparison>,
                required: true,
            },
            Port {
                name: "mask",
                required: false,
                ..IMAGE
            },
        ],
        limits: LIMITS,
        fixture: compare_fixture,
    },
];
