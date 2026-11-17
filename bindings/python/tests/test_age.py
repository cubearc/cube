"""CCTV/Go age compatibility through the installed native extension."""

import base64
import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3] / "crates/cube-core/src/ciphers/age/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
def test_authenticated_file_or_no_result(v):
    data = base64.b64decode(v["ciphertext_base64"])
    p = json.dumps(v["parameters"])
    if v["expect"] == "success":
        assert cube.age_decrypt(data, p) == base64.b64decode(v["plaintext_base64"])
    else:
        with pytest.raises(ValueError):
            cube.age_decrypt(data, p)


def test_bounds_and_private_identity():
    p = json.dumps(VECTORS[0]["parameters"])
    with pytest.raises(ValueError):
        cube.age_decrypt(bytes(16 * 1024 * 1024 + 1), p)
    with pytest.raises(ValueError) as e:
        cube.age_decrypt(b"", '{"profile":"age_v1_x25519_binary","identity":"SECRET"}')
    assert "SECRET" not in str(e.value)
