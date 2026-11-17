"""Recognized Wycheproof validity through the installed native SDK."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3]
        / "crates/cube-core/src/ciphers/modern_aead/wycheproof_vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
def test_wycheproof_authentication(v):
    def decrypt():
        return cube.aead_decrypt(
            bytes.fromhex(v["ciphertext_hex"]),
            bytes.fromhex(v["aad_hex"]),
            bytes.fromhex(v["tag_hex"]),
            json.dumps(v["parameters"]),
        )

    if v["valid"]:
        assert decrypt() == bytes.fromhex(v["plaintext_hex"])
    else:
        with pytest.raises(ValueError):
            decrypt()
