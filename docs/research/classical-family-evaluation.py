"""Evaluate finite original cases using the installed native Cube wheel."""

import base64
import json
import random
import zlib
from collections import Counter

import cube

PROSE = (
    "A careful reader compares each claim with the available evidence. "
    "The notebook keeps the original message beside every proposed reading, "
    "so that another person can repeat the experiment and challenge its assumptions. "
    "Clear boundaries help distinguish an incomplete search from a contradiction. "
    "We record the key choices and preserve punctuation, spaces and uncertain results. "
)
HELD_OUT = (
    "Along the river the small workshop opened before sunrise. "
    "Inside, a carpenter measured the narrow boards while a neighbour repaired "
    "the old window. Neither remembered where the blue notebook had been left. "
    "After lunch they walked across the square and found it under a wooden bench. "
)


def legacy(data):
    letters = bytes(b for b in data.upper() if 65 <= b <= 90)
    counts = Counter(letters)
    n = len(letters)
    if n < 20:
        return []
    ic = sum(c * (c - 1) for c in counts.values()) / (n * (n - 1))
    families = ["monoalphabetic_or_transposition" if ic >= 0.055 else "polyalphabetic"]
    if any(c > 1 for c in Counter(letters[i : i + 3] for i in range(n - 2)).values()):
        families.append("repeating_key")
    return families


def cases():
    rng = random.Random(2701)
    originals = [("original", PROSE), ("held_out", HELD_OUT)]
    for name, text in originals:
        for kind, transformed in [
            ("plaintext", text),
            ("caesar", cube.caesar_encode(text, 7)),
            ("affine", cube.affine_encode(text, 5, 8)),
            ("transposition", text[::-1]),
        ]:
            yield name + "_" + kind, "count_compatible", transformed.encode()
        yield (
            name + "_vigenere",
            "repeating_key",
            cube.vigenere_encode(text * 4, "KEY").encode(),
        )
    for name, data in [
        ("empty", b""),
        ("short", b"HELLO WORLD"),
        ("one_symbol", b"A" * 1024),
        ("two_symbols", b"AB" * 512),
        ("uniform_cycle", b"ABCDEFGHIJKLMNOPQRSTUVWXYZ" * 100),
        ("uniform_random", bytes(rng.randrange(65, 91) for _ in range(4096))),
        ("binary_random", rng.randbytes(4096)),
        ("compressed", zlib.compress(PROSE.encode() * 16)),
        ("hex", PROSE.encode().hex().encode()),
        ("base64", base64.b64encode(PROSE.encode())),
        ("unicode", ("猫 é 🦀 " * 128).encode()),
        # A deliberately structured language-like sequence tests the documented
        # ambiguity. Any family suggestion here counts as an unsupported claim.
        (
            "periodic_language",
            (
                "THE READER CAN CHECK EVERY CLAIM AND COMPARE THE LETTER COUNTS. " * 32
            ).encode(),
        ),
    ]:
        yield name, "negative_control", data


def main():
    rows = []
    for name, expected, data in cases():
        result = cube.family_suggest(
            {"profile": "ascii_classical_evidence_v1", "bytes": list(data)}
        )
        assert result["status"]["complete"], name
        triage = cube.triage_input(data)
        families = [
            family
            for suggestion in result["suggestions"]
            for family in suggestion["compatible_families"]
        ]
        rows.append(
            {
                "case": name,
                "expected": expected,
                "bytes": len(data),
                "legacy_claims": legacy(data),
                "triage_rules": [s["rule"] for s in triage["suggestions"]],
                "delivered_rules": [s["rule"] for s in result["suggestions"]],
                "compatible_families": families,
                "eligibility_failures": result["eligibility_failures"],
                "observations": result["observations"],
                "columns": result["columns"],
            }
        )
    negatives = [r for r in rows if r["expected"] == "negative_control"]
    summary = {
        "cases": len(rows),
        "negative_controls": len(negatives),
        "legacy_negative_claims": sum(bool(r["legacy_claims"]) for r in negatives),
        "delivered_negative_claims": sum(bool(r["delivered_rules"]) for r in negatives),
        "delivered_negative_abstentions": sum(
            not r["delivered_rules"] for r in negatives
        ),
        "count_compatible_hits": sum(
            "monoalphabetic_substitution" in r["compatible_families"]
            for r in rows
            if r["expected"] == "count_compatible"
        ),
        "repeating_key_hits": sum(
            "repeating_key_polyalphabetic" in r["compatible_families"]
            for r in rows
            if r["expected"] == "repeating_key"
        ),
        "triage_family_claims": 0,
        "interpretation": (
            "Finite original held-out and constructed cases only. "
            "Compatibility includes plaintext and alternatives. No calibration. "
            "Header triage suggests inspections, not cipher identities."
        ),
    }
    print(json.dumps({"summary": summary, "cases": rows}, indent=2))


if __name__ == "__main__":
    main()
