"""Run the owning conformance rows through the packaged native Python entry points."""

import base64
import json
from pathlib import Path

import cube
import pytest

FIXTURES = (
    Path(__file__).resolve().parents[3]
    / "crates/cube-core/src/operations/contracts/conformance.json"
)


def _bytes(value):
    return base64.b64decode(value["base64"], validate=True)


def _exact(value):
    return int(value) if isinstance(value, str) else value


def _wire(value):
    if isinstance(value, bytes):
        return {"base64": base64.b64encode(value).decode()}
    if type(value) is int and abs(value) > 2**53 - 1:
        return str(value)
    if isinstance(value, list):
        return [_wire(item) for item in value]
    if isinstance(value, dict):
        return {name: _wire(item) for name, item in value.items()}
    return value


def _bits(function):
    def call(inputs, params):
        output, valid_bits = function(_bytes(inputs["data"]), params)
        return {"output": output, "valid_bits": valid_bits}

    return call


OPERATIONS = {
    "binary.pack_bits": _bits(cube.pack_bits),
    "binary.unpack_bits": _bits(cube.unpack_bits),
    "binary.mask": lambda i, p: {"output": cube.mask_bytes(_bytes(i["data"]), p)},
    "binary.count_bits": lambda i, p: {"counts": cube.count_bits(_bytes(i["data"]), p)},
    "binary.fixed_integer.encode": lambda i, p: {
        "output": cube.fixed_integer_encode([_exact(v) for v in i["values"]], p)
    },
    "binary.fixed_integer.decode": lambda i, p: {
        "result": cube.fixed_integer_decode(_bytes(i["data"]), p)
    },
    "binary.bcd.encode": lambda i, p: {
        "output": cube.bcd_encode(i["digits"], i["negative"], p)
    },
    "binary.bcd.decode": lambda i, p: {"result": cube.bcd_decode(_bytes(i["data"]), p)},
    "binary.varint.encode": lambda i, p: {
        "output": cube.varint_encode([_exact(v) for v in i["values"]], p)
    },
    "binary.varint.decode": lambda i, p: {
        "records": cube.varint_decode(_bytes(i["data"]), p)
    },
    "binary.ieee754.inspect": lambda i, p: {
        "records": cube.ieee754_inspect(_bytes(i["data"]), p)
    },
    "binary.reed_solomon.encode": lambda i, p: {
        "blocks": cube.reed_solomon_encode([_bytes(row) for row in i["blocks"]], p)
    },
    "binary.reed_solomon.decode": lambda i, p: {
        "blocks": cube.reed_solomon_decode(
            [(_bytes(row["codeword"]), row["erasures"]) for row in i["blocks"]], p
        )
    },
    "hash.noncrypto": lambda i, p: {
        "output": cube.noncrypto_hash(_bytes(i["input"]), p)
    },
    "hash.siphash24": lambda i, _: {
        "digest": {"hex": cube.siphash24(_bytes(i["data"]), _bytes(i["key"]))}
    },
    "encoding.base32hex.encode": lambda i, _: {
        "output": cube.base32hex_encode(_bytes(i["input"]))
    },
    "encoding.base32hex.decode": lambda i, _: {
        "output": cube.base32hex_decode(i["input"])
    },
    "encoding.base64url.encode": lambda i, p: {
        "output": cube.base64url_encode(_bytes(i["input"]), p)
    },
    "encoding.base64url.decode": lambda i, p: {
        "output": cube.base64url_decode(i["input"], p)
    },
}


@pytest.fixture(scope="module")
def conformance():
    return {r["operation"]: r for r in json.loads(FIXTURES.read_text())}


@pytest.mark.parametrize("operation", OPERATIONS)
def test_native_formats_match_canonical_fixture(operation, conformance):
    row = conformance[operation]
    actual = OPERATIONS[operation](row["inputs"], row["parameters"])
    assert _wire(actual) == row["outputs"]


@pytest.mark.parametrize("value", [True, False, 1.0, "1", 2**64, -(2**63) - 1])
def test_integer_encoders_reject_coercion(value):
    with pytest.raises(ValueError, match="exact builtin 64-bit integer"):
        cube.fixed_integer_encode(
            [value], {"bits": 64, "byte_order": "little", "signedness": "unsigned"}
        )
    with pytest.raises(ValueError, match="exact builtin 64-bit integer"):
        cube.varint_encode([value], {"profile": "zigzag64"})


def test_binary64_preserves_a_signed_nan_payload_as_builtin_integer():
    raw = 0xFFF8000000000123
    [row] = cube.ieee754_inspect(
        raw.to_bytes(8, "little"), {"format": "binary64", "byte_order": "little"}
    )
    assert (type(row["bits"]), row["bits"], row["fraction"], row["classification"]) == (
        int,
        raw,
        0x8000000000123,
        "quiet_nan",
    )


def test_cross_interface_limits_and_key_policy():
    with pytest.raises(TypeError):
        cube.siphash24(b"", 0)
    with pytest.raises(ValueError, match="16-byte") as failure:
        cube.siphash24(b"message", b"short_secret")
    assert "short_secret" not in str(failure.value)
    metadata = cube.describe_operation("hash.siphash24", include_contracts=True)
    policy = metadata["contract"]["policy"]
    assert policy["private_input"] and policy["private_output"]
    assert not policy["automatic_capture"]
    assert metadata["pipeline"] is None
    with pytest.raises(ValueError, match="1 MiB"):
        cube.pack_bits(bytes((1 << 20) + 1), {"bit_order": "msb"})
    with pytest.raises(ValueError, match="work budget"):
        cube.reed_solomon_encode(
            [b"ok"],
            {"field": {"profile": "qr-rs-gf256"}, "parity_bytes": 2, "work_limit": 261},
        )
    with pytest.raises(ValueError, match="Base64url"):
        cube.base64url_decode("-_9=")
