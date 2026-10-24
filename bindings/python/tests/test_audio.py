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


def test_waveform_buckets_are_exact_partial_and_channel_aware(tmp_path):
    values = [0, 4, 32767, -4, -32768, 20, 5, -20, -1, 7, 10, 8, 0, -9]
    data = pcm_wave(struct.pack("<14h", *values), channels=2, width=2)
    result = cube.wave_envelope(data, bucket_frames=3)
    assert result["frames"] == 7
    assert result["buckets"] == [
        dict(start_frame=0, frames=3, minimum=[-32768, -4], maximum=[32767, 20]),
        dict(start_frame=3, frames=3, minimum=[-1, -20], maximum=[10, 8]),
        dict(start_frame=6, frames=1, minimum=[0, -9], maximum=[0, -9]),
    ]
    selected = cube.wave_envelope(
        data, start_frame=1, frames=5, channels=[1, 0], bucket_frames=3
    )
    assert selected["buckets"][0]["start_frame"] == 1
    assert selected["buckets"][1]["frames"] == 2
    path = tmp_path / "sample.wav"
    path.write_bytes(data)
    assert cube.wave_file_envelope(path, bucket_frames=3) == result
    assert path.read_bytes() == data
    assert cube.wave_envelope(pcm_wave(), bucket_frames=10)["buckets"] == []


def test_waveform_streaming_and_budget_errors():
    data = pcm_wave(struct.pack("<4h", 1, -2, -1, 2) * 20000, channels=2, width=2)
    result = cube.wave_envelope(data, bucket_frames=30000)
    assert result["frames"] == 40000
    assert [b["frames"] for b in result["buckets"]] == [30000, 10000]
    assert result["buckets"][0]["minimum"] == [-1, -2]
    assert result["buckets"][0]["maximum"] == [1, 2]
    for options in [
        dict(bucket_frames=0),
        dict(bucket_frames=2**64 - 1),
        dict(max_buckets=0),
        dict(max_buckets=10001),
        dict(bucket_frames=1, max_buckets=10000),
    ]:
        with pytest.raises(ValueError) as error:
            cube.wave_envelope(data, **options)
        assert error.value.operation == "audio.waveform.envelope"
