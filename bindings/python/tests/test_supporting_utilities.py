"""Exercise the installed native adapters with independent answers."""

import hashlib
from pathlib import Path

import cube
import pytest


def test_exact_byte_text_and_hash_wrappers():
    assert cube.binary_encode(b"\0\xff") == "00000000 11111111"
    assert cube.octal_decode("000 377") == b"\0\xff"
    assert cube.base36_encode(b"\0\0\xff") == "0073"
    assert cube.base62_decode("0047") == b"\0\0\xff"
    assert cube.ascii85_encode(bytes(4)) == "<~z~>"
    assert cube.ascii85_bare_decode("87cURDZ") == b"Hello"
    assert cube.base91_decode("nX,<:WRT%yxtYQ:mbr4JB*H[LR/@Qj{o0aU=Z") == (
        b"This is a random binary string"
    )
    assert cube.base92_decode("Fc_$aOTdKnsM*k") == b"hello world"
    assert cube.url_component_decode("%00%fF%2B") == b"\0\xff+"
    assert cube.reverse_bits(b"\x01\x02\x80\x00", unit_bytes=2) == b"\x40\x80\x00\x01"
    assert cube.sha512_224(b"abc") == hashlib.new("sha512_224", b"abc").hexdigest()
    assert cube.sha3_256(b"abc") == hashlib.sha3_256(b"abc").hexdigest()
    assert cube.crc8_smbus(b"123456789") == "f4"
    with pytest.raises(ValueError) as error:
        cube.fletcher32(b"odd")
    assert error.value.offset == 2
    with pytest.raises(ValueError) as error:
        cube.base36_encode(bytes(4097))
    assert error.value.code == "resource_limit"


def test_charset_and_escape_wrappers():
    assert cube.ascii_decode(cube.ascii_encode("A\0")) == "A\0"
    assert cube.utf16_encode("A🧊", byte_order="le", bom="require") == bytes.fromhex(
        "fffe41003ed8cadd"
    )
    assert cube.utf16_decode(b"\xfe\xff\0A", byte_order="be", bom="strip") == "A"
    assert cube.json_escape_encode("🧊") == r"\uD83E\uDDCA"
    assert cube.json_escape_encode("🧊", ascii_only=False) == "🧊"
    assert cube.json_escape_decode(r"\uD83E\uDDCA") == "🧊"
    assert cube.html_entity_decode("&notin; &#128; &notit;") == "∉ € ¬it;"
    with pytest.raises(ValueError) as error:
        cube.utf16_decode(b"\x00", byte_order="be", bom="forbid")
    assert error.value.code == "invalid_input"
    assert error.value.offset == 0


@pytest.mark.parametrize(
    "profile", ["deflate", "zlib", "gzip", "brotli", "zstd", "lzma_alone"]
)
def test_compression_wrappers(profile):
    raw = b"Cube\x00\xff\x80\n" * 19
    wire = cube.compress_bytes(raw, profile=profile)
    assert cube.decompress_bytes(wire, profile=profile) == raw
    with pytest.raises(ValueError) as error:
        cube.decompress_bytes(wire + b"x", profile=profile)
    assert error.value.code == "invalid_input"


def test_secret_kdf_and_coefficient_evidence():
    raw = cube.kdf_pbkdf2(b"p", b"s", profile="sha512", iterations=2, output_bytes=73)
    assert raw == hashlib.pbkdf2_hmac("sha512", b"p", b"s", 2, 73)
    assert (
        cube.kdf_argon2id(
            b"password",
            b"12345678",
            memory_kib=32,
            iterations=2,
            lanes=1,
            output_bytes=32,
        ).hex()
        == "a76b1652c8a7f4fac2d0805f6982e354cb0d9d580cc5553a197f4d79e599e534"
    )
    with pytest.raises(ValueError) as error:
        cube.kdf_scrypt(
            b"test-secret", b"12345678", n=524288, r=8, p=2, output_bytes=32
        )
    assert error.value.code == "resource_limit"
    assert "test-secret" not in str(error.value)
    jpeg = (
        Path(__file__).resolve().parents[3]
        / "crates/cube-core/src/image/jpeg_coefficients/fixture.jpg"
    ).read_bytes()
    result = cube.image_jpeg_coefficients(
        jpeg, profile="baseline_ac_parity", component_id=1, start_bit=3, max_bits=19
    )
    assert result["bits"] == bytes.fromhex("a44860")
    assert result["report"]["candidate_bits"] == 193
