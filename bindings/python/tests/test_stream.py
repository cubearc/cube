import cube
import pytest


def test_rfc8439_and_exact_pad():
    key = bytes(range(32))
    nonce = bytes.fromhex("000000090000004a00000000")
    output = cube.stream_transform("chacha20", bytes(64), key, nonce, 1)
    assert output.hex() == (
        "10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4e"
        "d2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e"
    )
    assert cube.stream_transform("chacha20", output, key, nonce, 1) == bytes(64)
    assert cube.vernam(b"\x00\xff", b"\xff\x0f") == b"\xff\xf0"
    assert cube.vernam(b"", b"") == b""
    with pytest.raises(ValueError):
        cube.vernam(b"ab", b"x")


@pytest.mark.parametrize(
    "profile,nonce_size,counter",
    [
        ("chacha8", 12, 2**32 - 1),
        ("chacha12", 12, 2**32 - 1),
        ("chacha20", 12, 2**32 - 1),
        ("xchacha20", 24, 2**32 - 1),
        ("salsa20_12", 8, 2**64 - 1),
        ("salsa20", 8, 2**64 - 1),
    ],
)
def test_explicit_counter_boundaries(profile, nonce_size, counter):
    key, nonce = bytes(32), bytes(nonce_size)
    assert len(cube.stream_transform(profile, bytes(64), key, nonce, counter)) == 64
    with pytest.raises(ValueError):
        cube.stream_transform(profile, bytes(65), key, nonce, counter)
    for bad in [True, -1, 1.0, "1", 2**64]:
        with pytest.raises((ValueError, TypeError, OverflowError)):
            cube.stream_transform(profile, b"", key, nonce, bad)
