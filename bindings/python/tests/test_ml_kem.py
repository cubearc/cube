"""Test FIPS 203 through the installed native extension."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3]
        / "crates/cube-core/src/asymmetric/ml_kem/vectors.json"
    ).read_text()
)["profiles"]


@pytest.mark.parametrize("vector", VECTORS, ids=lambda v: v["profile"])
def test_acvp_and_fresh_key_exchange(vector):
    parameters = json.dumps({"profile": vector["profile"]})
    for answer in vector["decapsulation"]:
        assert cube.ml_kem_decapsulate(
            bytes.fromhex(answer["c"]), bytes.fromhex(answer["dk"]), parameters
        ) == bytes.fromhex(answer["k"])

    public, private = cube.ml_kem_keygen(parameters)
    assert len(public) * 2 == len(vector["keygen"]["ek"])
    assert len(private) * 2 == len(vector["keygen"]["dk"])
    assert cube.ml_kem_keygen(parameters)[0] != public
    ciphertext, shared = cube.ml_kem_encapsulate(public, parameters)
    assert cube.ml_kem_decapsulate(ciphertext, private, parameters) == shared

    with pytest.raises(ValueError):
        cube.ml_kem_decapsulate(ciphertext[:-1], private, parameters)
    with pytest.raises(ValueError):
        cube.ml_kem_decapsulate(ciphertext, private[:-1], parameters)
    with pytest.raises(ValueError):
        cube.ml_kem_encapsulate(public, '{"profile":"kyber768"}')
