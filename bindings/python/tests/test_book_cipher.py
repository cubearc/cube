import cube
import pytest


def test_book_schemes_are_exact_and_do_not_fetch_sources():
    source = "Cube Uses\r\n\nBinary Evidence!\n雪 e\u0301"
    assert cube.book_decode(source, "1:1,2:1,3:1,4:1") == "CUBE"
    assert cube.book_decode(source, "0:0,1:0", index_base=0) == "CU"
    assert (
        cube.book_decode(source, "3:1,4:3,4:4", scheme="line_character") == "Be\u0301"
    )
    assert (
        cube.book_decode(source, "3:2:9,4:2:2", scheme="line_word_character")
        == "!\u0301"
    )
    assert cube.book_decode("", "") == ""
    assert cube.describe_operation("cipher.book.decode")["pipeline"] is None
    for coordinates in ["-1:1", "0:1", "1:99", "1:1,", "1:1:1"]:
        with pytest.raises(ValueError) as exc:
            cube.book_decode(source, coordinates)
        assert exc.value.operation == "cipher.book.decode"
    for kwargs in [{"scheme": "guess"}, {"index_base": 2}]:
        with pytest.raises(ValueError):
            cube.book_decode(source, "1:1", **kwargs)
    with pytest.raises(ValueError, match="1 MiB"):
        cube.book_decode("x" * (1024 * 1024 + 1), "1:1")
    assert source == "Cube Uses\r\n\nBinary Evidence!\n雪 e\u0301"
