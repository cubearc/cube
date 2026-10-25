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


JPEGS = [
    "/9j/4AAQSkZJRgABAgAAAQABAAD/wAALCAABAAIBAREA/9sAQwABAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/9oACAEBAAA/AP5/6//Z",
    "/9j/4AAQSkZJRgABAgAAAQABAAD/wAALCAABAAIBAREA/9sAQwABAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/9oACAEBAAA/AP6AK//Z",
]


def mov_frames(wide=False, fixed=False, frames=None):
    import base64

    if frames is None:
        frames = [base64.b64decode(JPEGS[i]) for i in [0, 1, 0]]

    def table(kind, rows):
        return atom(kind, bytes(4) + struct.pack(">I", len(rows)) + b"".join(rows))

    result = atom(b"ftyp", b"qt  " + bytes(4))
    first = len(result) + 8
    offsets = []
    data = b""
    for frame in frames:
        offsets.append(first + len(data))
        data += frame
    result += atom(b"mdat", data)
    duration = sum(i * 10 for i in range(1, len(frames) + 1))
    mvhd = bytearray(100)
    mvhd[12:20] = struct.pack(">II", 1000, duration)
    tkhd = bytearray(84)
    tkhd[12:16] = struct.pack(">I", 1)
    mdhd = bytearray(24)
    mdhd[12:20] = struct.pack(">II", 1000, duration)
    hdlr = bytes(8) + b"vide" + bytes(12)
    entry = bytearray(78)
    entry[6:8] = struct.pack(">H", 1)
    entry[24:28] = struct.pack(">HH", 2, 1)
    entry[74:78] = struct.pack(">Hh", 8, -1)
    tables = table(b"stsd", [atom(b"jpeg", entry)])
    sizes = struct.pack(">III", 0, len(frames[0]) if fixed else 0, len(frames))
    if not fixed:
        sizes += b"".join(struct.pack(">I", len(f)) for f in frames)
    tables += atom(b"stsz", sizes)
    tables += table(b"stsc", [struct.pack(">III", 1, 1, 1)])
    tables += table(
        b"co64" if wide else b"stco",
        [struct.pack(">Q" if wide else ">I", n) for n in offsets],
    )
    tables += table(
        b"stts", [struct.pack(">II", 1, i * 10) for i in range(1, len(frames) + 1)]
    )
    minf = atom(b"dinf", table(b"dref", [atom(b"url ", bytes([0, 0, 0, 1]))])) + atom(
        b"stbl", tables
    )
    mdia = atom(b"mdhd", mdhd) + atom(b"hdlr", hdlr) + atom(b"minf", minf)
    return result + atom(
        b"moov",
        atom(b"mvhd", mvhd) + atom(b"trak", atom(b"tkhd", tkhd) + atom(b"mdia", mdia)),
    )


def test_selected_mov_jpegs_are_native_images_with_exact_provenance():
    import hashlib

    for wide in [False, True]:
        for fixed in [False, True]:
            data = mov_frames(wide, fixed)
            frames = cube.extract_video_frames(data, 1, [2, 0, 1])
            assert [f["provenance"]["decode_start_ticks"] for f in frames] == [
                30,
                0,
                10,
            ]
            for frame in frames:
                assert isinstance(frame["image"], cube.Image)
                p = frame["provenance"]
                encoded = data[p["offset"] : p["offset"] + p["length"]]
                assert p["encoded_sha256"] == hashlib.sha256(encoded).hexdigest()
                expected = cube.Image.decode(encoded)
                assert frame["image"].compare(expected)["equal"]
                assert not frame["image"].provenance()["source_samples_preserved"]
    assert cube.describe_operation("video.frames.extract")["pipeline"] is None
    for indexes in [[], [0, 0], [3], [0] * 9]:
        with pytest.raises(ValueError):
            cube.extract_video_frames(mov_frames(), 1, indexes)
    with pytest.raises(ValueError):
        cube.extract_video_frames(mov_frames(), 2, [0])


def test_mov_extraction_validates_tables_and_never_decodes_unselected_samples():
    import base64

    data = mov_frames(frames=[base64.b64decode(JPEGS[0]), b"invalid"])
    assert cube.extract_video_frames(data, 1, [0])
    with pytest.raises(ValueError):
        cube.extract_video_frames(data, 1, [1])
    bad = bytearray(mov_frames())
    p = bad.index(b"stco") + 12
    bad[p : p + 4] = struct.pack(">I", 0)
    with pytest.raises(ValueError) as error:
        cube.extract_video_frames(bytes(bad), 1, [0])
    assert error.value.operation == "video.frames.extract"
