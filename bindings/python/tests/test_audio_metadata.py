import cube
import pytest


def flac_metadata(samples=1600, fields=()):
    info = bytearray(34)
    info[:4] = b"\x00\x10\x00\x10"
    packed = (8000 << 44) | (1 << 41) | (15 << 36) | samples
    info[10:18] = packed.to_bytes(8, "big")
    comments = (
        len(b"Cube").to_bytes(4, "little") + b"Cube" + len(fields).to_bytes(4, "little")
    )
    for field in fields:
        data = field.encode("utf-8")
        comments += len(data).to_bytes(4, "little") + data
    return (
        b"fLaC\x00\x00\x00\x22"
        + bytes(info)
        + bytes([0x84])
        + len(comments).to_bytes(3, "big")
        + comments
    )


def test_flac_metadata_uses_declared_values_and_retains_duplicate_comments():
    r = cube.inspect_audio_metadata(
        flac_metadata(fields=["TITLE=First", "TITLE=Second", "X-CUBE=雪"])
    )
    assert (r["sample_rate"], r["channels"], r["bits_per_sample"]) == (8000, 2, 16)
    assert r["duration_milliseconds"] == 200
    assert r["comments"][:2] == [
        dict(key="TITLE", value="First"),
        dict(key="TITLE", value="Second"),
    ]
    assert r["declared_md5"] is None
    assert r["verification"] == "metadata_only_audio_payload_unverified"
    assert (
        cube.inspect_audio_metadata(flac_metadata(samples=0))["duration_milliseconds"]
        is None
    )
    for data in [
        b"ID3\x00",
        b"OggS",
        flac_metadata()[:-1],
        flac_metadata(fields=["BROKEN"]),
    ]:
        with pytest.raises(ValueError) as error:
            cube.inspect_audio_metadata(data)
        assert error.value.operation == "audio.metadata.inspect"
    with pytest.raises(ValueError, match="16 KiB"):
        cube.inspect_audio_metadata(flac_metadata(fields=["K=" + "x" * 16384]))
