use cube_core::media::{
    pcm::{self, Selection},
    spectrum::{self, Options, Window},
};
use std::io::Cursor;
fn samples(rate: u32, channels: u16, frames: usize) -> pcm::Samples {
    let mut data = Vec::new();
    for i in 0..frames {
        for c in 0..channels {
            let k = if c == 0 { 5.0 } else { 9.0 };
            let v = (16384.0 * (std::f64::consts::TAU * k * i as f64 / 256.0).sin()).round() as i16;
            data.extend(v.to_le_bytes());
        }
    }
    let mut wav = Vec::new();
    wav.extend(b"RIFF");
    wav.extend((36 + data.len() as u32).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes());
    wav.extend(channels.to_le_bytes());
    wav.extend(rate.to_le_bytes());
    wav.extend((rate * u32::from(channels) * 2).to_le_bytes());
    wav.extend((channels * 2).to_le_bytes());
    wav.extend(16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend((data.len() as u32).to_le_bytes());
    wav.extend(data);
    pcm::extract(&mut Cursor::new(wav), &Selection::default()).unwrap()
}
#[test]
fn periodic_windows_preserve_bin_centered_peak_amplitudes_and_actual_rate() {
    for rate in [8000, 16000, 44100] {
        for window in [Window::Rectangular, Window::Hann, Window::Hamming] {
            let samples = samples(rate, 2, 256);
            let r = spectrum::analyze(
                &samples,
                Options {
                    fft_size: 256,
                    window,
                },
            )
            .unwrap();
            assert_eq!(r.frequencies_hz.len(), 129);
            assert_eq!(r.frequencies_hz[5], 5.0 * f64::from(rate) / 256.0);
            for (channel, bin) in [(0, 5), (1, 9)] {
                assert!((r.amplitudes[channel][bin] - 0.5).abs() < 0.00003);
                assert_eq!(
                    r.amplitudes[channel]
                        .iter()
                        .enumerate()
                        .max_by(|a, b| a.1.total_cmp(b.1))
                        .unwrap()
                        .0,
                    bin
                );
            }
        }
    }
}
#[test]
fn exact_shape_limits_and_parameter_validation_precede_transform() {
    let samples = samples(8000, 1, 256);
    for n in [0, 1, 7, 9, 16385, usize::MAX] {
        assert!(
            spectrum::analyze(
                &samples,
                Options {
                    fft_size: n,
                    ..Options::default()
                }
            )
            .is_err()
        );
    }
    assert!(Window::parse("guess").is_err());
    for n in [128, 512] {
        assert!(
            spectrum::analyze(
                &samples,
                Options {
                    fft_size: n,
                    ..Options::default()
                }
            )
            .is_err()
        );
    }
    assert!(spectrum::analyze(&self::samples(8000, 1, 0), Options::default()).is_err());
    assert!(
        spectrum::analyze(
            &self::samples(8000, 32, 4096),
            Options {
                fft_size: 4096,
                ..Options::default()
            }
        )
        .is_err()
    );
}
#[test]
fn dc_nyquist_and_silence_are_not_doubled_or_nonfinite() {
    // Reuse a valid block through the same PCM reader, changing only exact sample bytes.
    for (signal, bin, expected) in [
        (vec![0i16; 8], 0, 0.0),
        (vec![16384i16; 8], 0, 0.5),
        ([16384i16, -16384].repeat(4), 4, 0.5),
    ] {
        let mut wav = b"RIFF".to_vec();
        wav.extend(52u32.to_le_bytes());
        wav.extend(b"WAVEfmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend([1, 0, 1, 0]);
        wav.extend(8000u32.to_le_bytes());
        wav.extend(16000u32.to_le_bytes());
        wav.extend([2, 0, 16, 0]);
        wav.extend(b"data");
        wav.extend(16u32.to_le_bytes());
        for v in signal {
            wav.extend(v.to_le_bytes());
        }
        let block = pcm::extract(&mut Cursor::new(wav), &Selection::default()).unwrap();
        let r = spectrum::analyze(
            &block,
            Options {
                fft_size: 8,
                window: Window::Rectangular,
            },
        )
        .unwrap();
        assert!((r.amplitudes[0][bin] - expected).abs() < 1e-12);
        assert!(r.amplitudes[0].iter().all(|n| n.is_finite()));
    }
    assert!(
        cube_core::operations::require("audio.spectrum")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
