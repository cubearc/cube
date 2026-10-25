import hashlib

import cube


def test_new_hashes_match_standard_library_for_exact_binary_inputs():
    for name, width in [("sha512", 128), ("sha1", 40), ("md5", 32)]:
        fn = getattr(cube, name)
        for data in [b"", b"abc", b"a", bytes(range(256)), b"Hello\n"]:
            expected = hashlib.new(name, data, usedforsecurity=False).hexdigest()
            assert fn(data) == expected
            assert len(fn(data)) == width
    assert cube.crc32c(b"123456789") == "e3069283"
    assert cube.crc32c(b"") == "00000000"
    assert cube.crc16_ibm3740(b"123456789") == "29b1"
    assert cube.crc16_ibm3740(b"") == "ffff"
    assert cube.crc16_ibm3740(bytes(range(256))) == "3fbd"
    assert cube.crc32c(bytes(range(256))) == "9c44184b"


def test_hash_adapters_preserve_hex_and_existing_recipe_contracts():
    for name in ["sha512", "sha1", "md5", "crc32c", "crc16_ibm3740"]:
        assert cube.describe_operation(f"hash.{name}")["pipeline"] == {"version": 1}
    for name in ["sha1", "md5"]:
        assert (
            "collision-resistant"
            in cube.describe_operation(f"hash.{name}")["description"]
        )
