"""Exercise the installed Rust extension through the public Python API."""

from importlib.machinery import EXTENSION_SUFFIXES

import cube
import pytest
from cube import _native


def test_native_extension_is_loaded():
    assert any(_native.__file__.endswith(suffix) for suffix in EXTENSION_SUFFIXES)
    assert cube.base64_encode is _native.base64_encode
    assert cube.base32_encode is _native.base32_encode
    assert cube.base32_decode is _native.base32_decode


@pytest.mark.parametrize(
    ("data", "encoded"),
    [
        (b"", ""),
        (b"f", "MY======"),
        (b"fo", "MZXQ===="),
        (b"foo", "MZXW6==="),
        (b"foob", "MZXW6YQ="),
        (b"fooba", "MZXW6YTB"),
        (b"foobar", "MZXW6YTBOI======"),
    ],
)
def test_base32_rfc4648_vectors(data, encoded):
    assert cube.base32_encode(data) == encoded
    assert cube.base32_decode(encoded) == data


def test_known_answers():
    assert cube.base64_encode(b"Hello") == "SGVsbG8="
    assert cube.base64_decode("SGVsbG8=") == b"Hello"
    assert cube.hex_encode(b"Hello") == "48656c6c6f"
    assert cube.hex_decode("00aAFF") == b"\x00\xaa\xff"
    assert cube.rot13("Hello, World!") == "Uryyb, Jbeyq!"
    assert cube.sha256(b"abc") == (
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    )
    assert cube.xor(b"Hello", b"key") == bytes([35, 0, 21, 7, 10])


@pytest.mark.parametrize("data", [b"", b"\x00\xff\x80", bytes(range(256))])
def test_binary_round_trips_and_determinism(data):
    for encode, decode in [
        (cube.base32_encode, cube.base32_decode),
        (cube.base64_encode, cube.base64_decode),
        (cube.hex_encode, cube.hex_decode),
    ]:
        assert decode(encode(data)) == data
        assert type(decode(encode(data))) is bytes
        assert encode(data) == encode(data)
    assert cube.xor(cube.xor(data, b"key"), b"key") == data
    assert type(cube.xor(data, b"key")) is bytes
    assert cube.xor(data, b"key") == cube.xor(data, b"key")
    assert cube.sha256(data) == cube.sha256(data)


def test_unicode_and_empty_text():
    text = "Olá, 世界 🧊 e\u0301"
    assert cube.rot13(text) == "Byá, 世界 🧊 r\u0301"
    assert cube.rot13(cube.rot13(text)) == text
    assert cube.rot13("") == ""
    data = text.encode("utf-8")
    assert cube.base32_decode(cube.base32_encode(data)) == data
    assert cube.base64_decode(cube.base64_encode(data)) == data
    assert cube.hex_decode(cube.hex_encode(data)) == data


def test_empty_inputs():
    assert cube.base32_encode(b"") == ""
    assert cube.base32_decode("") == b""
    assert cube.base64_encode(b"") == ""
    assert cube.base64_decode("") == b""
    assert cube.hex_encode(b"") == ""
    assert cube.hex_decode("") == b""
    assert cube.xor(b"", b"key") == b""
    assert cube.sha256(b"") == (
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    )


@pytest.mark.parametrize(
    "text",
    [
        "mzxw6===",
        "MZxW6===",
        "0A======",
        "é",
        " MY======",
        "MY======\n",
        "MY",
        "MY=======",
        "MY=====A",
        "MY======MY======",
        "MZ======",
        "MZXR====",
        "MZXW7===",
        "MZXW6YR=",
    ],
)
def test_invalid_base32(text):
    with pytest.raises(ValueError, match="invalid Base32"):
        cube.base32_decode(text)


@pytest.mark.parametrize("text", ["!", "Zg", "Zh==", "Zg==\n", "-_8=", "é"])
def test_invalid_base64(text):
    with pytest.raises(ValueError, match="invalid Base64"):
        cube.base64_decode(text)


@pytest.mark.parametrize("text", ["a", "zz", "0x00", "00 ", "é"])
def test_invalid_hex(text):
    with pytest.raises(ValueError, match="invalid hexadecimal"):
        cube.hex_decode(text)


@pytest.mark.parametrize("data", [b"", b"Hello"])
def test_empty_xor_key(data):
    with pytest.raises(ValueError, match="XOR key must not be empty"):
        cube.xor(data, b"")


@pytest.mark.parametrize(
    ("operation", "args"),
    [
        (cube.base32_encode, ("Hello",)),
        (cube.base32_decode, (b"MZXW6===",)),
        (cube.base64_encode, ("Hello",)),
        (cube.base64_decode, (b"SGVsbG8=",)),
        (cube.hex_encode, ("Hello",)),
        (cube.hex_decode, (b"00",)),
        (cube.sha256, ("Hello",)),
        (cube.rot13, (b"Hello",)),
        (cube.xor, (b"Hello", "key")),
    ],
)
def test_text_and_bytes_are_not_silently_coerced(operation, args):
    with pytest.raises(TypeError):
        operation(*args)
