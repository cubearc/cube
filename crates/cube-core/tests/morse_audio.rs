#[path = "common/pcm.rs"]
mod fixture;
use cube_core::media::{
    morse::{self, Interval, Options, State},
    pcm::{self, Selection},
};
use std::io::Cursor;
fn signal(pattern: &[(bool, usize)], hz: f64) -> pcm::Samples {
    let mut data = Vec::new();
    for (active, units) in pattern {
        for i in 0..units * 800 {
            let value = if *active {
                (std::f64::consts::TAU * hz * i as f64 / 8000.0).sin() * 12000.0
            } else {
                0.0
            };
            data.extend((value.round() as i16).to_le_bytes());
        }
    }
    pcm::extract(
        &mut Cursor::new(fixture::wav(8000, 1, 16, &data)),
        &Selection::default(),
    )
    .unwrap()
}
fn options() -> Options {
    Options {
        frequency_hz: 700,
        window_ms: 10,
        min_level_dbfs: -40,
        purity_percent: 80,
    }
}
#[test]
fn tone_detection_and_existing_morse_decoder_remain_separate() {
    let samples = signal(
        &[
            (false, 2),
            (true, 1),
            (false, 1),
            (true, 1),
            (false, 1),
            (true, 1),
            (false, 3),
            (true, 3),
            (false, 1),
            (true, 3),
            (false, 1),
            (true, 3),
            (false, 3),
            (true, 1),
            (false, 1),
            (true, 1),
            (false, 1),
            (true, 1),
            (false, 2),
        ],
        700.0,
    );
    let r = morse::detect(&samples, options()).unwrap();
    assert_eq!(r.intervals[0].state, State::Silence);
    assert_eq!(r.intervals[1].frames, 800);
    assert_eq!(r.window_frames, 80);
    let text = morse::transcribe(&r.intervals, r.sample_rate, 100, 20).unwrap();
    assert_eq!(text.status, "decoded");
    assert_eq!(text.tokens.as_deref(), Some("... --- ..."));
    assert_eq!(text.text.as_deref(), Some("SOS"));
    let word = [
        Interval {
            start_frame: 0,
            frames: 800,
            state: State::Tone,
        },
        Interval {
            start_frame: 800,
            frames: 5600,
            state: State::Silence,
        },
        Interval {
            start_frame: 6400,
            frames: 2400,
            state: State::Tone,
        },
    ];
    assert_eq!(
        morse::transcribe(&word, 8000, 100, 20)
            .unwrap()
            .text
            .as_deref(),
        Some("E T")
    );
}
#[test]
fn off_band_ambiguous_timing_noise_and_unsupported_tokens_do_not_invent_text() {
    let r = morse::detect(&signal(&[(true, 2)], 1100.0), options()).unwrap();
    assert!(r.intervals.iter().any(|i| i.state == State::Uncertain));
    assert_eq!(
        morse::transcribe(&r.intervals, 8000, 100, 20)
            .unwrap()
            .status,
        "uncertain"
    );
    let r = morse::detect(&signal(&[(true, 2)], 700.0), options()).unwrap();
    let t = morse::transcribe(&r.intervals, 8000, 100, 20).unwrap();
    assert!(t.text.is_none());
    assert_eq!(t.uncertain_segments, [0]);
    let r = morse::detect(&signal(&[(false, 3)], 700.0), options()).unwrap();
    assert_eq!(
        morse::transcribe(&r.intervals, 8000, 100, 20)
            .unwrap()
            .status,
        "no_result"
    );
    let mut intervals = Vec::new();
    let mut frame = 0;
    for i in 0..13 {
        let state = if i % 2 == 0 {
            State::Tone
        } else {
            State::Silence
        };
        intervals.push(Interval {
            start_frame: frame,
            frames: 800,
            state,
        });
        frame += 800;
    }
    let r = morse::transcribe(&intervals, 8000, 100, 20).unwrap();
    assert_eq!(r.status, "unsupported_tokens");
    assert!(r.text.is_none());
    assert!(r.tokens.is_some());
}
#[test]
fn interval_ranges_input_and_timing_budgets_are_checked() {
    let one = Interval {
        start_frame: 0,
        frames: 800,
        state: State::Tone,
    };
    for (rate, unit, tol) in [(0, 100, 20), (8000, 0, 20), (8000, 100, 41)] {
        assert!(morse::transcribe(std::slice::from_ref(&one), rate, unit, tol).is_err());
    }
    for intervals in [
        vec![Interval {
            frames: 0,
            ..one.clone()
        }],
        vec![Interval {
            start_frame: u64::MAX,
            ..one.clone()
        }],
        vec![one.clone(), one.clone()],
        vec![one.clone(); 4097],
        vec![Interval {
            frames: 262145,
            ..one.clone()
        }],
    ] {
        assert!(morse::transcribe(&intervals, 8000, 100, 20).is_err());
    }
    assert!(morse::State::parse("guess").is_err());
    assert!(
        morse::detect(
            &signal(&[(true, 1)], 700.0),
            Options {
                frequency_hz: 4000,
                ..options()
            }
        )
        .is_err()
    );
    assert!(
        morse::detect(&signal(&[], 700.0), options())
            .unwrap()
            .intervals
            .is_empty()
    );
    assert!(
        morse::transcribe(&[], 8000, 100, 20)
            .unwrap()
            .text
            .is_none()
    );
    assert!(
        cube_core::operations::require("audio.morse.detect")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
    assert!(
        cube_core::operations::require("audio.morse.transcribe")
            .unwrap()
            .discovery()
            .pipeline
            .is_none()
    );
}
