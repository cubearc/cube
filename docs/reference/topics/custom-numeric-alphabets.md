### Custom numeric alphabets

`number.alphabet.convert` uses explicit ordered Unicode-scalar alphabets and bases
(2..256, not exceeding the corresponding alphabet length). Only the first base
entries are active. Alphabets require unique scalars, no ASCII +/- or Unicode
whitespace, and at most 1,024 UTF-8 bytes. Custom digits match exactly, without
normalization or case folding; the existing ASCII converter keeps its behavior.
Magnitude remains <=2^128-1; larger integers are deferred because no concrete
workflow justifies expanding the existing integer contract. Input allows an
optional ASCII sign and <=4,096 digits/16,385 bytes. Leading zeros and plus signs
canonicalize; negative zero becomes the target zero digit. Output is <=128 digits
plus sign/513 UTF-8 bytes. This integer operation has no pipeline adapter.
