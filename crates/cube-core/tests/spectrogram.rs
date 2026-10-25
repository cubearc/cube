#[path = "common/pcm.rs"]
mod fixture;
use cube_core::media::{
    pcm::{self, Selection},
    spectrogram::{self, FinalWindow, Options},
    spectrum::Window,
};
use std::io::Cursor;
fn block(frames: usize) -> pcm::Samples {
    let mut data = Vec::new();
    for i in 0..frames {
        let k = if i < 256 { 5.0 } else { 9.0 };
        let n = (16384.0 * (std::f64::consts::TAU * k * i as f64 / 256.0).sin()).round() as i16;
        data.extend(n.to_le_bytes());
    }
    pcm::extract(
        &mut Cursor::new(fixture::wav(8000, 1, 16, &data)),
        &Selection::default(),
    )
    .unwrap()
}
#[test]
fn windows_axes_and_changing_frequencies_are_exact() {
    let r = spectrogram::analyze(
        &block(512),
        Options {
            fft_size: 256,
            hop_size: 256,
            window: Window::Rectangular,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(r.frame_starts, [0, 256]);
    assert_eq!(r.times_seconds, [0.0, 0.032]);
    assert_eq!(r.valid_frames, [256, 256]);
    assert_eq!(r.dropped_frames, 0);
    assert!((r.amplitudes[0][0][5] - 0.5).abs() < 0.00003);
    assert!((r.amplitudes[0][1][9] - 0.5).abs() < 0.00003);
}
#[test]
fn short_empty_tail_and_absolute_selection_have_explicit_policies() {
    for n in [0, 1, 8, 9, 12] {
        for policy in [FinalWindow::Drop, FinalWindow::Pad] {
            let r = spectrogram::analyze(
                &block(n),
                Options {
                    fft_size: 8,
                    hop_size: 4,
                    final_window: policy,
                    ..Options::default()
                },
            )
            .unwrap();
            let count = match policy {
                FinalWindow::Drop => {
                    if n < 8 {
                        0
                    } else {
                        1 + (n - 8) / 4
                    }
                }
                FinalWindow::Pad => n.div_ceil(4),
            };
            assert_eq!(r.frame_starts.len(), count);
            assert!(
                r.amplitudes
                    .iter()
                    .flatten()
                    .flatten()
                    .all(|v| v.is_finite())
            );
        }
    }
    let input = fixture::wav(8000, 1, 8, &[128; 20]);
    let samples = pcm::extract(
        &mut Cursor::new(input),
        &Selection {
            start_frame: 5,
            frames: Some(10),
            ..Selection::default()
        },
    )
    .unwrap();
    let r = spectrogram::analyze(
        &samples,
        Options {
            fft_size: 8,
            hop_size: 4,
            final_window: FinalWindow::Pad,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(r.frame_starts, [5, 9, 13]);
    assert_eq!(r.valid_frames, [8, 6, 2]);
    assert_eq!(r.times_seconds[0], 5.0 / 8000.0);
    let r = spectrogram::analyze(
        &samples,
        Options {
            fft_size: 8,
            hop_size: 4,
            ..Options::default()
        },
    )
    .unwrap();
    assert_eq!(r.dropped_frames, 2);
}
#[test]
fn matrix_work_and_bad_options_fail_before_transform() {
    for (fft, hop) in [(7, 1), (8, 0), (8, 9), (usize::MAX, 1)] {
        assert!(
            spectrogram::analyze(
                &block(8),
                Options {
                    fft_size: fft,
                    hop_size: hop,
                    ..Options::default()
                }
            )
            .is_err()
        );
    }
    assert!(spectrogram::FinalWindow::parse("guess").is_err());
    assert!(
        spectrogram::analyze(
            &block(10000),
            Options {
                fft_size: 256,
                hop_size: 1,
                ..Options::default()
            }
        )
        .is_err()
    );
    assert!(
        cube_core::operations::require("audio.spectrogram")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
