"""Independent raw DH answers execute through the installed native extension."""

import json
from pathlib import Path

import cube
import pytest

VECTORS = json.loads(
    (
        Path(__file__).parents[3] / "crates/cube-core/src/asymmetric/dh/vectors.json"
    ).read_text()
)


@pytest.mark.parametrize("v", VECTORS)
def test_independent_shared_secrets(v):
    assert cube.dh_agree(
        bytes.fromhex(v["peer_hex"]),
        bytes.fromhex(v["private_key_hex"]),
        json.dumps(v["parameters"]),
    ) == bytes.fromhex(v["shared_hex"])


def test_rejects_bad_encodings_and_keys():
    v = VECTORS[0]
    key = bytes.fromhex(v["private_key_hex"])
    parameters = json.dumps(v["parameters"])
    for length in [0, 1, 255, 256, 257, 1025]:
        with pytest.raises(ValueError):
            cube.dh_agree(bytes(length), key, parameters)
    with pytest.raises(ValueError):
        cube.dh_agree(bytes.fromhex(v["peer_hex"]), bytes(32), parameters)
