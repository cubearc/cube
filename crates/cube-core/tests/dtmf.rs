#[path = "common/pcm.rs"]
mod fixture;
use cube_core::media::{
    dtmf::{self, Options},
    pcm::{self, Selection},
};
use std::io::Cursor;
fn block(rate: u32, hz: &[f64], frames: usize, amplitude: f64) -> pcm::Samples {
    let data: Vec<u8> = (0..frames)
        .flat_map(|i| {
            let v = hz
                .iter()
                .map(|h| (std::f64::consts::TAU * h * i as f64 / f64::from(rate)).sin() * amplitude)
                .sum::<f64>();
            ((v * 32768.0).round() as i16).to_le_bytes()
        })
        .collect();
    pcm::extract(
        &mut Cursor::new(fixture::wav(rate, 1, 16, &data)),
        &Selection::default(),
    )
    .unwrap()
}
#[test]
fn all_sixteen_nominal_pairs_have_bounded_timed_candidates() {
    for rate in [8000u32, 16000, 44100] {
        for (r, row) in [697.0, 770.0, 852.0, 941.0].iter().enumerate() {
            for (c, col) in [1209.0, 1336.0, 1477.0, 1633.0].iter().enumerate() {
                let samples = block(rate, &[*row, *col], (rate / 5) as usize, 0.2);
                let d = dtmf::detect(&samples, Options::default()).unwrap();
                assert_eq!(d.events.len(), 1);
                let e = &d.events[0];
                assert_eq!(
                    e.symbol,
                    Some(char::from(b"123A456B789C*0#D"[r * 4 + c])),
                    "rate {rate}, row {row}, col {col}; {d:?}"
                );
                assert_eq!(e.start_frame, 0);
                assert_eq!(e.frames, u64::from(rate / 5));
                assert_eq!(e.status, "candidate");
            }
        }
    }
}
#[test]
fn silence_noise_off_band_single_ambiguous_and_short_tones_are_not_confirmed() {
    for frequencies in [
        &[][..],
        &[697.0][..],
        &[1000.0][..],
        &[697.0, 770.0, 1209.0][..],
        &[730.0, 1270.0][..],
    ] {
        let d = dtmf::detect(&block(8000, frequencies, 1600, 0.2), Options::default()).unwrap();
        assert!(
            d.events.iter().all(|e| e.symbol.is_none()),
            "{frequencies:?}: {d:?}"
        );
    }
    let short = dtmf::detect(&block(8000, &[697.0, 1209.0], 320, 0.2), Options::default()).unwrap();
    assert_eq!(short.events[0].reason, "short_run");
    assert!(short.events[0].symbol.is_none());
    let mut seed = 1u32;
    let noise: Vec<u8> = (0..1600)
        .flat_map(|_| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            ((seed >> 16) as i16).to_le_bytes()
        })
        .collect();
    let n = pcm::extract(
        &mut Cursor::new(fixture::wav(8000, 1, 16, &noise)),
        &Selection::default(),
    )
    .unwrap();
    assert!(
        dtmf::detect(&n, Options::default())
            .unwrap()
            .events
            .iter()
            .all(|e| e.symbol.is_none())
    );
    let d = dtmf::detect(
        &block(8000, &[697.0, 1209.0], 320, 0.0001),
        Options::default(),
    )
    .unwrap();
    assert!(d.windows.iter().all(|w| w.status == "silence"));
}
#[test]
fn gap_thresholds_partial_windows_and_resource_validation_remain_explicit() {
    let first = block(8000, &[697.0, 1209.0], 640, 0.2);
    let second = block(8000, &[770.0, 1336.0], 640, 0.2);
    let bytes = [first.raw(), second.raw()].concat();
    let samples = pcm::extract(
        &mut Cursor::new(fixture::wav(8000, 1, 16, &bytes)),
        &Selection::default(),
    )
    .unwrap();
    let d = dtmf::detect(&samples, Options::default()).unwrap();
    assert_eq!(d.events[0].symbol, Some('1'));
    assert_eq!(d.events[1].reason, "insufficient_gap");
    assert!(d.events[1].symbol.is_none());
    assert!(
        dtmf::detect(&block(8000, &[], 0, 0.0), Options::default())
            .unwrap()
            .windows
            .is_empty()
    );
    assert!(dtmf::detect(&block(4000, &[], 8, 0.0), Options::default()).is_err());
    assert!(dtmf::detect(&block(8000, &[], 262145, 0.0), Options::default()).is_err());
    for opts in [
        Options {
            window_ms: 0,
            ..Options::default()
        },
        Options {
            purity_percent: 101,
            ..Options::default()
        },
        Options {
            min_level_dbfs: -61,
            ..Options::default()
        },
        Options {
            max_twist_db: 13,
            ..Options::default()
        },
        Options {
            min_tone_ms: 1,
            ..Options::default()
        },
    ] {
        assert!(dtmf::detect(&first, opts).is_err());
    }
    let stereo = pcm::extract(
        &mut Cursor::new(fixture::wav(8000, 2, 16, &[0; 32])),
        &Selection::default(),
    )
    .unwrap();
    assert!(dtmf::detect(&stereo, Options::default()).is_err());
    let partial =
        dtmf::detect(&block(8000, &[697.0, 1209.0], 641, 0.2), Options::default()).unwrap();
    assert_eq!(partial.windows.last().unwrap().frames, 1);
    assert!(
        cube_core::operations::require("audio.dtmf.detect")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
