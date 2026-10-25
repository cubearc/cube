import math
import struct

import cube
import pytest
from test_audio import pcm_wave


def signal(frequencies, count, rate=8000):
    values = [
        round(
            sum(0.2 * math.sin(math.tau * hz * i / rate) for hz in frequencies) * 32768
        )
        for i in range(count)
    ]
    return pcm_wave(struct.pack(f"<{count}h", *values), width=2, rate=rate)


def test_dtmf_returns_candidates_evidence_and_uncertainty_without_forced_text():
    d = cube.detect_dtmf(signal([697, 1209], 1600))
    assert d["events"][0]["symbol"] == "1"
    assert d["events"][0]["status"] == "candidate"
    assert d["events"][0]["frames"] == 1600
    assert d["windows"][0]["row_hz"] == 697
    assert d["timing_tolerance_frames"] == 320
    short = cube.detect_dtmf(signal([697, 1209], 320))
    assert short["events"][0]["symbol"] is None
    assert short["events"][0]["reason"] == "short_run"
    assert cube.detect_dtmf(signal([], 1600))["events"] == []
    assert cube.detect_dtmf(signal([1000], 1600))["events"] == []
    for options in [
        {"window_ms": 0},
        {"min_level_dbfs": -61},
        {"purity_percent": 101},
        {"surprise": True},
    ]:
        with pytest.raises((ValueError, TypeError)):
            cube.detect_dtmf(signal([], 800), **options)
