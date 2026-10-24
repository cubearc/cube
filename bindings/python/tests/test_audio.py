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


@pytest.mark.parametrize("width", [1, 2, 3, 4])
def test_sample_ranges_keep_exact_bytes_signed_values_and_channels(width, tmp_path):
    bits = width * 8
    minimum, maximum = -(2 ** (bits - 1)), 2 ** (bits - 1) - 1
    values = [minimum, maximum, 0, -1, maximum, minimum]

    def encode(numbers):
        return b"".join(
            (n + 128).to_bytes(1, "little")
            if width == 1
            else n.to_bytes(width, "little", signed=True)
            for n in numbers
        )

    data = pcm_wave(encode(values), channels=2, width=width)
    result = cube.wave_samples(
        data, channels=[1, 0], start_frame=1, frames=2, include_values=True
    )
    assert result["values"] == [-1, 0, minimum, maximum]
    assert result["data"] == encode([-1, 0, minimum, maximum])
    assert result["channels"] == [1, 0]
    assert result["start_frame"] == 1
    path = tmp_path / "input.wav"
    path.write_bytes(data)
    assert (
        cube.wave_file_samples(
            path, channels=[1, 0], start_frame=1, frames=2, include_values=True
        )
        == result
    )
    assert cube.wave_samples(data, start_frame=3, frames=0)["data"] == b""


def test_sample_validation_and_raw_structured_budgets():
    data = pcm_wave(bytes(4), channels=2, width=2)
    for selection in [
        dict(channels=[]),
        dict(channels=[0, 0]),
        dict(channels=[2]),
        dict(start_frame=2),
        dict(start_frame=1, frames=1),
    ]:
        with pytest.raises(ValueError) as error:
            cube.wave_samples(data, **selection)
        assert error.value.operation == "audio.pcm.extract"
    many = pcm_wave(bytes(10002))
    assert len(cube.wave_samples(many)["data"]) == 10002
    with pytest.raises(ValueError) as error:
        cube.wave_samples(many, include_values=True)
    assert error.value.code == "resource_limit"
