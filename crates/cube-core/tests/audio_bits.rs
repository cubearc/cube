#[path = "common/pcm.rs"]
mod fixture;
use cube_core::{
    errors::ErrorCode,
    media::{
        bits::{self, BitOrder, Incomplete, Options},
        pcm::{self, Selection},
    },
};
use std::io::Cursor;
fn samples(bits: u16, data: &[u8], channels: u16) -> pcm::Samples {
    pcm::extract(
        &mut Cursor::new(fixture::wav(8000, channels, bits, data)),
        &Selection::default(),
    )
    .unwrap()
}
#[test]
fn native_sample_bits_preserve_all_storage_widths_and_negative_extrema() {
    for width in [1usize, 2, 3, 4] {
        let data = [vec![0u8; width], vec![255; width], {
            let mut b = vec![0; width];
            b[width - 1] = 128;
            b
        }]
        .concat();
        let block = samples((width * 8) as u16, &data, 1);
        let top = bits::extract(
            &block,
            &Options {
                bits: vec![(width * 8 - 1) as u8],
                incomplete: Incomplete::Pad,
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(top.data, [0x60]);
        let all = bits::extract(
            &block,
            &Options {
                bits: (0..width as u8 * 8).rev().collect(),
                ..Options::default()
            },
        )
        .unwrap();
        let expected = data
            .chunks_exact(width)
            .flat_map(|b| b.iter().rev().copied())
            .collect::<Vec<_>>();
        assert_eq!(all.data, expected);
    }
    // Unsigned 8-bit raw 0x41 must not become centered signed storage 0xc1.
    assert_eq!(
        bits::extract(
            &samples(8, b"A", 1),
            &Options {
                bits: (0..8).rev().collect(),
                ..Options::default()
            }
        )
        .unwrap()
        .data,
        b"A"
    );
}
#[test]
fn frame_channel_bit_traversal_and_tail_packing_are_explicit() {
    let data = [1u8, 0, 0, 1, 1, 0, 0, 1];
    let block = samples(8, &data, 2);
    assert_eq!(
        bits::extract(&block, &Options::default()).unwrap().data,
        [0x99]
    );
    let block = samples(8, &[1, 0, 0, 1, 0, 1, 1, 0], 1);
    assert_eq!(
        bits::extract(&block, &Options::default()).unwrap().data,
        [0x96]
    );
    assert_eq!(
        bits::extract(
            &block,
            &Options {
                reverse_frames: true,
                ..Options::default()
            }
        )
        .unwrap()
        .data,
        [0x69]
    );
    let block = samples(8, &[1, 0, 1], 1);
    assert!(bits::extract(&block, &Options::default()).is_err());
    let r = bits::extract(
        &block,
        &Options {
            incomplete: Incomplete::Drop,
            ..Options::default()
        },
    )
    .unwrap();
    assert!(r.data.is_empty());
    assert_eq!((r.available_bits, r.valid_bits, r.dropped_bits), (3, 0, 3));
    assert_eq!(
        bits::extract(
            &block,
            &Options {
                incomplete: Incomplete::Pad,
                ..Options::default()
            }
        )
        .unwrap()
        .data,
        [0xa0]
    );
    assert_eq!(
        bits::extract(
            &block,
            &Options {
                bit_order: BitOrder::Lsb,
                incomplete: Incomplete::Pad,
                ..Options::default()
            }
        )
        .unwrap()
        .data,
        [5]
    );
    assert!(
        bits::extract(&samples(8, &[], 1), &Options::default())
            .unwrap()
            .data
            .is_empty()
    );
}
#[test]
fn invalid_selections_and_bit_work_limits_fail_before_packing() {
    let block = samples(8, &[0; 8], 1);
    for b in [vec![], vec![0, 0], vec![8], vec![255]] {
        assert!(
            bits::extract(
                &block,
                &Options {
                    bits: b,
                    ..Options::default()
                }
            )
            .is_err()
        );
    }
    let block = samples(32, &vec![0; 4 * 1024 * 1024], 1);
    let r = bits::extract(
        &block,
        &Options {
            bits: (0..32).collect(),
            ..Options::default()
        },
    )
    .unwrap_err();
    assert_eq!(r.code, ErrorCode::ResourceLimit);
    assert!(
        cube_core::operations::require("audio.bits.extract")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
