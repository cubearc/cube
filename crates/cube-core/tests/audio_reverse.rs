#[path = "common/pcm.rs"]
mod fixture;
use cube_core::media::{
    pcm::{self, Selection},
    reverse, wave,
};
use std::io::Cursor;
#[test]
fn reversed_frames_preserve_native_bytes_rate_channel_groups_and_padding() {
    for width in [1usize, 2, 3, 4] {
        for channels in [1u16, 2] {
            for frames in [0usize, 1, 3] {
                let align = width * usize::from(channels);
                let data: Vec<u8> = (0..frames * align).map(|i| (i * 23) as u8).collect();
                let input = fixture::wav(44100, channels, (width * 8) as u16, &data);
                let block = pcm::extract(&mut Cursor::new(&input), &Selection::default()).unwrap();
                let output = reverse::wave(&block).unwrap();
                let info = wave::inspect(&mut Cursor::new(&output)).unwrap();
                assert_eq!(info.format.sample_rate, 44100);
                assert_eq!(info.format.channels, channels);
                assert_eq!(info.format.bits_per_sample, (width * 8) as u16);
                assert_eq!(info.frames, frames as u64);
                assert_eq!(output.len() % 2, 0);
                let back = pcm::extract(&mut Cursor::new(&output), &Selection::default()).unwrap();
                let expected = data
                    .chunks_exact(align)
                    .rev()
                    .flatten()
                    .copied()
                    .collect::<Vec<_>>();
                assert_eq!(back.raw(), expected);
                let second = reverse::wave(&back).unwrap();
                assert_eq!(
                    pcm::extract(&mut Cursor::new(second), &Selection::default())
                        .unwrap()
                        .raw(),
                    data
                );
                assert_eq!(block.raw(), data);
            }
        }
    }
}
#[test]
fn selected_channel_order_and_original_ranges_are_explicit() {
    let input = fixture::wav(8000, 2, 8, &[1, 11, 2, 12, 3, 13]);
    let block = pcm::extract(
        &mut Cursor::new(input),
        &Selection {
            start_frame: 1,
            frames: Some(2),
            channels: Some(vec![1, 0]),
        },
    )
    .unwrap();
    let out = reverse::wave(&block).unwrap();
    assert_eq!(
        pcm::extract(&mut Cursor::new(out), &Selection::default())
            .unwrap()
            .raw(),
        [13, 3, 12, 2]
    );
    assert!(
        cube_core::operations::require("audio.reverse")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
