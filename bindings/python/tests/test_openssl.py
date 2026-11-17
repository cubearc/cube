"""Cross-version serialized legacy files through the installed native extension."""

import json
from pathlib import Path

import cube
import pytest

ROOT = Path(__file__).parents[3] / "crates/cube-core/src/ciphers/openssl"
VECTORS = json.loads((ROOT / "vectors.json").read_text())


@pytest.mark.parametrize("v", VECTORS)
def test_explicit_legacy_files(v):
    settings = json.dumps(v["parameters"])
    data = bytes.fromhex(v["ciphertext_hex"])
    assert cube.openssl_decrypt(data, settings) == bytes.fromhex(v["plaintext_hex"])
    with pytest.raises(ValueError):
        cube.openssl_decrypt(data[:-1], settings)


def test_valid_padding_is_not_authentication():
    v = json.loads((ROOT / "valid_pad_wrong_password.json").read_text())
    assert cube.openssl_decrypt(
        bytes.fromhex(v["ciphertext_hex"]), json.dumps(v["parameters"])
    ) == bytes.fromhex(v["wrong_plaintext_hex"])


def test_cost_and_secret_rejections():
    p = {
        "profile": "aes128_cbc_pkcs7_pbkdf2_sha256_salt8",
        "password_hex": "",
        "iterations": 100001,
    }
    with pytest.raises(ValueError):
        cube.openssl_decrypt(b"", json.dumps(p))
    with pytest.raises(ValueError) as e:
        cube.openssl_decrypt(b"", '{"password_hex":"SECRET"}')
    assert "SECRET" not in str(e.value)
