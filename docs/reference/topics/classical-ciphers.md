# Classical ciphers

[Monoalphabetic profiles](../../monoalphabetic.md) cover exact/keyword alphabets, ASCII
rotations, deterministic homophones, digit grammars and Grandpré, with explicit
canonicalization, token transport and bounds.

Use the `ciphers` category when you have a classical letter substitution and a
specified key/alphabet. These operations do not provide secure modern encryption.

```sh
cube cipher vigenere --key LEMON --text ATTACKATDAWN
# LXFOPVEFRNHR
cube cipher affine --decode --a 5 --b 8 --text IHHWVCSWFRCP
# AFFINECIPHER
```

Caesar, ROT13 and Atbash operate on ASCII letters and preserve case and other
Unicode. Vigenère/Beaufort keys contain nonempty ASCII letters; key position
advances only for ASCII input letters. Standard Beaufort is self-inverse, not
Variant Beaufort. Affine uses `C=(aP+b) mod 26` with A=0, `a,b` in 0..25 and `a`
coprime to 26. Use the original parameters to decode. Caesar accepts a signed
32-bit shift; newer keyed ciphers cap input at 1 MiB and keys at 64 KiB.

Book coordinates use the exact caller-supplied UTF-8 source. Choose
`word_character` (`word:scalar`), `line_character` (`line:scalar`) or
`line_word_character` (`line:word:scalar`); separate records with commas. All
components use the explicit base (default 1, optionally 0). Decimal leading zeroes
and surrounding ASCII whitespace are accepted; signs and empty records fail.
Words split on Unicode whitespace and retain punctuation, matching `text.select`.
LF/CRLF line endings are excluded; blank lines count, bare CR and line spaces
remain data, and a final newline creates no extra line. Characters are Unicode
scalars, including separate combining marks, with no normalization.

Record the source edition and a SHA-256 of its exact bytes to reproduce results.
Source and coordinate text each allow 1 MiB, with at most 65,536 coordinates and
indexed units (lines plus words for triples). Output is at most 256 KiB. Invalid
coordinates fail as a whole, with a record number and no guessed substitute.
No book is bundled, fetched or stored. This direct operation has no pipeline
adapter: the separate source stays caller-controlled rather than being embedded
in recipe parameters. Its text result can be passed to ordinary text operations.

Modern encrypted-file formats remain outside the current cipher API. The
[format evaluation](https://github.com/cubearc/cube/issues/78) records a possible
age v1/X25519 subset and the parser-budget/interoperability work needed before
implementation. There is no modern decryption operation in this release.
