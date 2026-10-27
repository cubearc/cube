use cube_core::media::{pcm::Samples, wave::Format};
fn format(bits: u16) -> Format {
    let block_align = 2 * (bits / 8);
    Format {
        format_tag: 1,
        channels: 2,
        sample_rate: 8000,
        bits_per_sample: bits,
        block_align,
        byte_rate: 8000 * u32::from(block_align),
    }
}
#[test]
fn logical_pcm_preserves_order_origin_and_native_integer_widths() {
    for (bits, data, expected) in [
        (8, vec![0, 255], vec![-128, 127]),
        (16, vec![0, 128, 255, 127], vec![-32768, 32767]),
        (24, vec![0, 0, 128, 255, 255, 127], vec![-8388608, 8388607]),
        (
            32,
            vec![0, 0, 0, 128, 255, 255, 255, 127],
            vec![i32::MIN, i32::MAX],
        ),
    ] {
        let value = Samples::from_raw(format(bits), vec![1, 0], 17, 1, data.clone()).unwrap();
        assert_eq!(value.raw(), data);
        assert_eq!(value.channels(), [1, 0]);
        assert_eq!(value.start_frame(), 17);
        assert_eq!(value.frames(), 1);
        assert_eq!(value.values(), expected);
    }
}
#[test]
fn hostile_logical_shapes_cannot_create_invalid_samples() {
    for (channels, start, frames, data) in [
        (vec![], 0, 0, vec![]),
        (vec![0, 0], 0, 1, vec![0; 4]),
        (vec![2], 0, 1, vec![0; 2]),
        (vec![0], u64::MAX, 1, vec![0; 2]),
        (vec![0], 0, 2, vec![0; 2]),
        (vec![0], 0, 1048577, vec![]),
        (vec![0], 0, 1, vec![0; 8 * 1024 * 1024 + 1]),
    ] {
        assert!(Samples::from_raw(format(16), channels, start, frames, data).is_err());
    }
    let mut invalid = format(16);
    invalid.byte_rate = 1;
    assert!(Samples::from_raw(invalid, vec![0], 0, 0, vec![]).is_err());
    assert!(Samples::from_raw(format(16), vec![0], 0, 0, vec![]).is_ok());
}
