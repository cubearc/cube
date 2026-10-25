import math
import struct

import cube
import pytest
from test_audio import pcm_wave


def tone(rate=8000, count=256, bins=(5, 9)):
    values = [
        round(16384 * math.sin(math.tau * k * i / 256))
        for i in range(count)
        for k in bins
    ]
    return pcm_wave(
        struct.pack(f"<{len(values)}h", *values), channels=len(bins), width=2, rate=rate
    )


def test_spectrum_has_exact_rate_channels_window_and_peak_units():
    data = tone()
    for window in ["rectangular", "hann", "hamming"]:
        r = cube.wave_spectrum(data, channels=[1, 0], fft_size=256, window=window)
        assert r["channels"] == [1, 0]
        assert r["amplitude_units"] == "one_sided_peak_full_scale"
        assert r["frequencies_hz"][5] == 156.25
        assert r["amplitudes"][0][9] == pytest.approx(0.5, abs=0.00003)
        assert r["amplitudes"][1][5] == pytest.approx(0.5, abs=0.00003)
    for kwargs in [
        {"fft_size": 9},
        {"fft_size": 512},
        {"window": "guess"},
        {"channels": [2]},
    ]:
        with pytest.raises(ValueError):
            cube.wave_spectrum(data, **kwargs)
    assert cube.describe_operation("audio.spectrum")["pipeline"] is None
    r = cube.wave_spectrum(pcm_wave(bytes([128] * 8)), fft_size=8)
    assert r["amplitudes"] == [[0.0] * 5]


def test_spectrogram_has_absolute_start_axes_and_explicit_partial_windows():
    r = cube.wave_spectrogram(
        tone(count=512), fft_size=256, hop_size=256, window="rectangular"
    )
    assert r["frame_starts"] == [0, 256]
    assert r["times_seconds"] == [0, 0.032]
    assert r["amplitudes"][0][1][5] == pytest.approx(0.5, abs=0.00003)
    r = cube.wave_spectrogram(
        pcm_wave(bytes([128] * 20)),
        start_frame=5,
        frames=10,
        fft_size=8,
        hop_size=4,
        final_window="pad",
    )
    assert r["frame_starts"] == [5, 9, 13]
    assert r["valid_frames"] == [8, 6, 2]
    assert r["amplitudes"] == [[[0.0] * 5] * 3]
    assert (
        cube.wave_spectrogram(pcm_wave(), fft_size=8, hop_size=4)["frame_starts"] == []
    )
    for options in [
        {"hop_size": 0},
        {"fft_size": 7},
        {"final_window": "guess"},
        {"surprise": True},
    ]:
        with pytest.raises((ValueError, TypeError)):
            cube.wave_spectrogram(tone(), **options)
