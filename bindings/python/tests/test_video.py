import struct

import cube
import pytest


def atom(kind, data):
    return struct.pack(">I", len(data) + 8) + kind + data


def mp4_metadata():
    mvhd = bytearray(100)
    mvhd[12:20] = struct.pack(">II", 1000, 2000)
    tkhd = bytearray(84)
    tkhd[12:16] = struct.pack(">I", 1)
    mdhd = bytearray(24)
    mdhd[12:20] = struct.pack(">II", 48000, 96000)
    hdlr = bytes(8) + b"vide" + bytes(12)
    entry = bytearray(78)
    entry[24:28] = struct.pack(">HH", 640, 480)
    stsd = bytes(4) + struct.pack(">I", 1) + atom(b"avc1", entry)
    mdia = (
        atom(b"mdhd", mdhd)
        + atom(b"hdlr", hdlr)
        + atom(b"minf", atom(b"stbl", atom(b"stsd", stsd)))
    )
    moov = atom(b"mvhd", mvhd) + atom(
        b"trak", atom(b"tkhd", tkhd) + atom(b"mdia", mdia)
    )
    return atom(b"ftyp", b"isom\0\0\0\0") + atom(b"moov", moov)


def test_video_metadata_is_declared_data_without_decoder_or_pipeline_claim():
    r = cube.inspect_video(mp4_metadata())
    assert r["container"] == "mp4"
    assert r["duration_ticks"] == 2000
    assert r["duration_seconds"] == 2.0
    assert r["tracks"][0]["codec_id"] == "avc1"
    assert (r["tracks"][0]["width"], r["tracks"][0]["height"]) == (640, 480)
    assert r["verification"] == "declared_container_metadata_media_payload_unverified"
    assert cube.describe_operation("video.container.inspect")["pipeline"] is None
    for data in [
        b"",
        mp4_metadata()[:-1],
        atom(b"ftyp", b"isom\0\0\0\0") + atom(b"moof", b""),
    ]:
        with pytest.raises(ValueError) as error:
            cube.inspect_video(data)
        assert error.value.operation == "video.container.inspect"
