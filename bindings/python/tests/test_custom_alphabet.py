import cube
import pytest


def test_exact_custom_digits_and_magnitude():
    assert cube.convert_alphabet("1010", "01", "abcdefghij", 2, 10) == "ba"
    assert cube.convert_alphabet("-٠٠١٠", "٠١٢٣٤٥٦٧٨٩", "01", 10, 2) == "-1010"
    maximum = str(2**128 - 1)
    bits = cube.convert_alphabet(maximum, "0123456789", "αβ", 10, 2)
    assert len(bits) == 128
    assert cube.convert_alphabet(bits, "αβ", "0123456789", 2, 10) == maximum
    assert cube.convert_alphabet("-000", "01", "αβ", 2, 2) == "α"
    for value in ["", "+", "2", "0x1", "1.0", "0" * 4097, "1" * 129]:
        with pytest.raises(ValueError):
            cube.convert_alphabet(value, "01", "01", 2, 2)
    assert cube.describe_operation("number.alphabet.convert")["pipeline"] is None
