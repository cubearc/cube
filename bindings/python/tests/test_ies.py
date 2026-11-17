"""Independent Bouncy Castle suites execute through the installed Core SDK."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3]
        / "crates/cube-core/src/asymmetric/ies/interoperability.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
def test_bouncy_castle_answers_fresh_ciphertexts_and_context_authentication(v):
    p = json.dumps(v["parameters"])
    private = bytes.fromhex(v["private_key_hex"])
    public = bytes.fromhex(v["public_key_hex"])
    plain = bytes.fromhex(v["plaintext_hex"])
    cipher = bytes.fromhex(v["ciphertext_hex"])
    assert cube.ies_decrypt(cipher, private, p) == plain
    first = cube.ies_encrypt(plain, public, p)
    assert first != cube.ies_encrypt(plain, public, p)
    assert cube.ies_decrypt(first, private, p) == plain
    with pytest.raises(ValueError):
        cube.ies_decrypt(cipher[:-1] + bytes([cipher[-1] ^ 1]), private, p)
    for field in ["derivation_hex", "encoding_hex"]:
        wrong = {**v["parameters"], field: v["parameters"][field] + "00"}
        with pytest.raises(ValueError):
            cube.ies_decrypt(cipher, private, json.dumps(wrong))
