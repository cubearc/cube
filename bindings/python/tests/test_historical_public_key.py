"""Independent exact cases run through the installed educational native SDK."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3]
        / "crates/cube-core/src/asymmetric/historical_public_key/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
def test_reference_vectors_exact_integers_and_full_candidate_sets(v):
    encrypt = getattr(cube, v["scheme"] + "_encrypt")
    decrypt = getattr(cube, v["scheme"] + "_decrypt")
    parameters = json.dumps(v["parameters"])
    public = json.dumps(v["public_key"]).encode()
    private = json.dumps(v["private_key"]).encode()
    expected = v.get("candidate_tokens", v["tokens"])
    assert decrypt(v["ciphertext_tokens"], private, parameters) == expected
    fresh = encrypt(v["tokens"], public, parameters)
    if v["scheme"] in ["rabin", "merkle_hellman"]:
        assert fresh == v["ciphertext_tokens"]
    assert decrypt(fresh, private, parameters) == expected
    assert encrypt([], public, parameters) == []
    with pytest.raises(ValueError):
        encrypt(v["tokens"], public, json.dumps({**v["parameters"], "max_work": 1}))
    with pytest.raises(ValueError):
        encrypt(
            v["tokens"], public, json.dumps({**v["parameters"], "max_memory_bytes": 1})
        )
    with pytest.raises((ValueError, TypeError)):
        encrypt(["0" * 1235], public, parameters)
    with pytest.raises(TypeError):
        encrypt([20], public, parameters)
