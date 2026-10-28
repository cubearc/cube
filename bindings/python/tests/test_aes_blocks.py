"""Real native raw AES blocks; no text conversion or authenticated-success claim."""

import cube
import pytest


@pytest.mark.parametrize(
    ("key", "expected"),
    [
        (bytes(range(16)), "69c4e0d86a7b0430d8cdb78070b4c55a"),
        (bytes(range(24)), "dda97ca4864cdfe06eaf70a0ec0d7191"),
        (bytes(range(32)), "8ea2b7ca516745bfeafc49904b496089"),
    ],
)
def test_fips_vectors_empty_and_multiple_blocks(key, expected):
    plain = bytes.fromhex("00112233445566778899aabbccddeeff")
    encrypted = bytes.fromhex(expected)
    assert cube.aes_encrypt_blocks(plain, key) == encrypted
    assert cube.aes_decrypt_blocks(encrypted, key) == plain
    assert cube.aes_encrypt_blocks(plain * 3, key) == encrypted * 3
    assert cube.aes_decrypt_blocks(b"", key) == b""


def test_invalid_keys_blocks_and_limits_do_not_echo_secrets():
    for function in (cube.aes_encrypt_blocks, cube.aes_decrypt_blocks):
        for data, key, code in [
            (b"", b"SECRET", "invalid_parameter"),
            (b"SECRET", bytes(16), "invalid_input"),
            (bytes(16 * 1024 * 1024 + 1), bytes(16), "resource_limit"),
        ]:
            with pytest.raises(ValueError) as caught:
                function(data, key)
            assert caught.value.code == code
            assert "SECRET" not in str(caught.value)
