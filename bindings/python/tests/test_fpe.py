"""Independent FPE known answers through the installed native extension."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3] / "crates/cube-core/src/ciphers/fpe/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
def test_exact_numeric_profiles(v):
    p = json.dumps(v["parameters"])
    assert cube.fpe_encrypt(v["plaintext"], p) == v["ciphertext"]
    assert cube.fpe_decrypt(v["ciphertext"], p) == v["plaintext"]


def test_domain_and_private_settings():
    p = json.dumps(
        {
            "profile": "ff1_sp800_38g_integer",
            "radix": 10,
            "key_hex": "00" * 16,
            "tweak_hex": "",
        }
    )
    for digits in ([], [0] * 5, [0] * 5 + [10], [0] * 4097):
        with pytest.raises(ValueError):
            cube.fpe_encrypt(digits, p)
    with pytest.raises(ValueError) as e:
        cube.fpe_encrypt([0] * 6, '{"key_hex":"SECRET"}')
    assert "SECRET" not in str(e.value)
