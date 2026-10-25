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
    assert cube.base58_encode is _native.base58_encode
    assert cube.base58_decode is _native.base58_decode
    assert cube.caesar_encode is _native.caesar_encode
    assert cube.caesar_decode is _native.caesar_decode
    assert cube.crc32 is _native.crc32


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


@pytest.mark.parametrize(
    ("data", "encoded"),
    [
        (b"", ""),
        (b"a", "2g"),
        (b"Hello World", "JxF12TrwUP45BMd"),
        (b"\x00", "1"),
        (b"\x00\x00\x00\x00", "1111"),
        (b"\x00\x00\x01", "112"),
        (b"\x00\xff", "15Q"),
        (b"\xff", "5Q"),
    ],
)
def test_base58_known_answers_and_leading_zeroes(data, encoded):
    assert cube.base58_encode(data) == encoded
    assert cube.base58_decode(encoded) == data


def test_base58_is_case_sensitive():
    assert cube.base58_decode("A") == b"\x09"
    assert cube.base58_decode("a") == b"\x21"


@pytest.mark.parametrize(
    ("text", "shift", "encoded"),
    [
        ("Attack at dawn!", 3, "Dwwdfn dw gdzq!"),
        (
            "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
            3,
            "defghijklmnopqrstuvwxyzabcDEFGHIJKLMNOPQRSTUVWXYZABC",
        ),
        ("AaZz", -1, "ZzYy"),
        ("AaZz", 27, "BbAa"),
        ("AaZz", -2147483648, "CcBb"),
        ("AaZz", 2147483647, "XxWw"),
        ("", -2147483648, ""),
        (
            "Olá, 世界 🧊 e\u0301 K Ａ ﬀ 123\x00\t\r\n",
            3,
            "Roá, 世界 🧊 h\u0301 K Ａ ﬀ 123\x00\t\r\n",
        ),
    ],
)
def test_caesar_known_answers_and_round_trips(text, shift, encoded):
    assert cube.caesar_encode(text, shift) == encoded
    assert cube.caesar_decode(encoded, shift) == text
    assert cube.caesar_encode(cube.caesar_decode(text, shift), shift) == text
    assert cube.caesar_encode(text, shift) == cube.caesar_encode(text, shift)
    assert type(cube.caesar_encode(text, shift)) is str
    assert type(cube.caesar_decode(encoded, shift)) is str


@pytest.mark.parametrize("shift", [0, 26, -26, 52, -52])
def test_caesar_zero_and_full_rotations(shift):
    text = "AaZz 世界 e\u0301"
    assert cube.caesar_encode(text, shift) == text
    assert cube.caesar_decode(text, shift) == text


def test_caesar_shift_13_agrees_with_rot13():
    text = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ Olá, 世界 🧊"
    assert cube.caesar_encode(text, 13) == cube.rot13(text)
    assert cube.caesar_decode(text, 13) == cube.rot13(text)
    assert cube.caesar_decode(cube.caesar_encode(text, shift=3), shift=3) == text


@pytest.mark.parametrize("operation", [cube.caesar_encode, cube.caesar_decode])
@pytest.mark.parametrize("shift", [-2147483649, 2147483648, 2**100])
def test_caesar_rejects_out_of_range_shifts(operation, shift):
    with pytest.raises(OverflowError):
        operation("Hello", shift)


@pytest.mark.parametrize("operation", [cube.caesar_encode, cube.caesar_decode])
@pytest.mark.parametrize("shift", ["3", 3.0, None])
def test_caesar_rejects_non_integer_shifts(operation, shift):
    with pytest.raises(TypeError):
        operation("Hello", shift)


@pytest.mark.parametrize(
    ("data", "expected"),
    [
        (b"123456789", "cbf43926"),
        (b"", "00000000"),
        (b"Hello", "f7d18982"),
        (b"Hello\n", "31963516"),
        (b'"', "0762ae69"),
        (b"\x00", "d202ef8d"),
        (b"\x00\xff\x80", "81dda740"),
        (bytes(range(256)), "29058c73"),
        ("Olá, 世界 🧊".encode(), "08b240b4"),
    ],
)
def test_crc32_known_answers_and_binary_data(data, expected):
    checksum = cube.crc32(data)
    assert checksum == expected
    assert cube.crc32(data) == checksum
    assert type(checksum) is str
    assert len(checksum) == 8
    assert set(checksum) <= set("0123456789abcdef")


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
        (cube.base58_encode, cube.base58_decode),
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
    assert cube.crc32(data) == cube.crc32(data)


def test_unicode_and_empty_text():
    text = "Olá, 世界 🧊 e\u0301"
    assert cube.rot13(text) == "Byá, 世界 🧊 r\u0301"
    assert cube.rot13(cube.rot13(text)) == text
    assert cube.rot13("") == ""
    data = text.encode("utf-8")
    assert cube.base32_decode(cube.base32_encode(data)) == data
    assert cube.base58_decode(cube.base58_encode(data)) == data
    assert cube.base64_decode(cube.base64_encode(data)) == data
    assert cube.hex_decode(cube.hex_encode(data)) == data


def test_empty_inputs():
    assert cube.base58_encode(b"") == ""
    assert cube.base58_decode("") == b""
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


@pytest.mark.parametrize(
    "text",
    [
        "0",
        "O",
        "I",
        "l",
        "2g=",
        "!",
        "é",
        "世界",
        "2g ",
        " 2g",
        "2 g",
        "2g\n",
        "2\tg",
        "\x00",
    ],
)
def test_invalid_base58(text):
    with pytest.raises(ValueError, match="invalid Base58"):
        cube.base58_decode(text)


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
        (cube.base58_encode, ("Hello",)),
        (cube.base58_decode, (b"2g",)),
        (cube.base64_encode, ("Hello",)),
        (cube.base64_decode, (b"SGVsbG8=",)),
        (cube.hex_encode, ("Hello",)),
        (cube.hex_decode, (b"00",)),
        (cube.sha256, ("Hello",)),
        (cube.crc32, ("Hello",)),
        (cube.rot13, (b"Hello",)),
        (cube.caesar_encode, (b"Hello", 3)),
        (cube.caesar_decode, (b"Hello", 3)),
        (cube.xor, (b"Hello", "key")),
    ],
)
def test_text_and_bytes_are_not_silently_coerced(operation, args):
    with pytest.raises(TypeError):
        operation(*args)


def test_operation_discovery_uses_core_metadata():
    descriptors = cube.operations()
    ids = [d["id"] for d in descriptors]
    assert ids == sorted(set(ids))
    assert len(ids) >= 14
    assert cube.operations("binary") == [cube.describe_operation("binary.xor")]
    assert cube.describe_operation("encoding.base32.decode")["output_type"] == "bytes"
    assert cube.describe_operation("hash.crc32")["reversible"] is False
    shift = cube.describe_operation("cipher.caesar.encode")["parameters"][0]
    assert shift["constraints"] == {
        "type": "integer",
        "min": -2147483648,
        "max": 2147483647,
    }
    with pytest.raises(ValueError, match="unknown"):
        cube.operations("missing")
    with pytest.raises(ValueError, match="unknown"):
        cube.describe_operation("missing")
    with pytest.raises(TypeError):
        cube.describe_operation(b"binary.xor")
    assert cube.operations is _native.operations


@pytest.mark.parametrize(
    ("call", "operation"),
    [
        (lambda: cube.base32_decode("bad"), "encoding.base32.decode"),
        (lambda: cube.base58_decode("0"), "encoding.base58.decode"),
        (lambda: cube.base64_decode("!"), "encoding.base64.decode"),
        (lambda: cube.hex_decode("!"), "encoding.hex.decode"),
        (lambda: cube.xor(b"data", b""), "binary.xor"),
    ],
)
def test_operation_errors_preserve_exception_types_and_add_context(call, operation):
    with pytest.raises(ValueError) as captured:
        call()
    assert captured.value.code == "invalid_input"
    assert captured.value.operation == operation
    assert captured.value.parameter is None
    assert captured.value.offset is None


def test_discovery_errors_and_file_parameters_share_core_contracts():
    assert len(cube.operations()) >= 23
    assert len(cube.operations("file")) == 10
    for call, operation, parameter in [
        (lambda: cube.operations("missing"), "ops.list", "category"),
        (lambda: cube.describe_operation("missing"), "ops.describe", "id"),
    ]:
        with pytest.raises(ValueError) as error:
            call()
        assert error.value.code == "invalid_parameter"
        assert error.value.operation == operation
        assert error.value.parameter == parameter
    params = cube.describe_operation("file.signature.scan")["parameters"]
    assert {p["name"] for p in params} == {"offset", "length", "format", "max_results"}
    assert params[0]["constraints"]["max"] == 18446744073709551615
