"""NIST positive/adversarial key wrapping through the installed native wheel."""

import json
from pathlib import Path

import cube
import pytest

ROOT = Path(__file__).parents[3] / "crates/cube-core/src/ciphers/key_wrap"
VALID = json.loads((ROOT / "vectors.json").read_text())
INVALID = json.loads((ROOT / "invalid_vectors.json").read_text())


@pytest.mark.parametrize("v", VALID)
def test_known_wrapping_and_integrity_checked_key_release(v):
    p = json.dumps(v["parameters"])
    plain, wrapped = bytes.fromhex(v["plaintext_hex"]), bytes.fromhex(v["wrapped_hex"])
    assert cube.aes_key_wrap(plain, p) == wrapped
    assert cube.aes_key_unwrap(wrapped, p) == plain


@pytest.mark.parametrize("v", INVALID)
def test_adversarial_wrapping_never_returns_key_data(v):
    with pytest.raises(ValueError, match="key unwrap integrity check failed"):
        cube.aes_key_unwrap(
            bytes.fromhex(v["wrapped_hex"]), json.dumps(v["parameters"])
        )


def test_lengths_and_private_settings():
    p = json.dumps({"profile": "aes_kwp_rfc5649", "key_hex": "42" * 16})
    for n in (1, 7, 8, 9, 16, 65536):
        data = b"\x24" * n
        wrapped = cube.aes_key_wrap(data, p)
        assert len(wrapped) == (n + 7) // 8 * 8 + 8
        assert cube.aes_key_unwrap(wrapped, p) == data
    for n in (0, 65537):
        with pytest.raises(ValueError):
            cube.aes_key_wrap(bytes(n), p)
    with pytest.raises(ValueError) as error:
        cube.aes_key_wrap(b"", '{"key_hex":"SECRET"}')
    assert "SECRET" not in str(error.value)
