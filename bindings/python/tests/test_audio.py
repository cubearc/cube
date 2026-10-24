"""Original PCM WAVE fixtures; no third-party recordings or external tools."""

import io
import struct
import wave

import cube
import pytest


def pcm_wave(data=b"", channels=1, width=1, rate=8000):
    output = io.BytesIO()
    with wave.open(output, "wb") as stream:
        stream.setnchannels(channels)
        stream.setsampwidth(width)
        stream.setframerate(rate)
        stream.writeframes(data)
    result = output.getvalue()
    # wave's writer omits final RIFF padding for an odd data byte count.
    if len(data) % 2:
        result += b"\x00"
        result = result[:4] + struct.pack("<I", len(result) - 8) + result[8:]
    return result


@pytest.mark.parametrize("width", [1, 2, 3, 4])
def test_wave_inspection_is_exact_across_widths_and_paths(width, tmp_path):
    data = pcm_wave(bytes(3 * 2 * width), channels=2, width=width)
    info = cube.inspect_wave(data)
    assert info["frames"] == 3
    assert info["duration_seconds"] == {"numerator": 3, "denominator": 8000}
    assert info["format"]["bits_per_sample"] == width * 8
    assert info["chunks"][1]["data_offset"] == 44
    path = tmp_path / "sample.wav"
    path.write_bytes(data)
    assert cube.inspect_wave_file(path) == info
    assert path.read_bytes() == data


def test_wave_malformed_and_unsupported_inputs_have_context():
    assert cube.inspect_wave(pcm_wave())["frames"] == 0
    for marker in [b"RIFX", b"RF64"]:
        with pytest.raises(ValueError) as error:
            cube.inspect_wave(marker + pcm_wave()[4:])
        assert error.value.code == "unsupported_format"
    for data in [b"", pcm_wave()[:-1], pcm_wave()[0:40] + b"\xff\xff\xff\xff"]:
        with pytest.raises(ValueError) as error:
            cube.inspect_wave(data)
        assert error.value.operation == "audio.wave.inspect"
