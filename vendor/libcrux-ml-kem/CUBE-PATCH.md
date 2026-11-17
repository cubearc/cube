# Libcrux ML-KEM source used by Cube

Source is `libcrux-ml-kem` 0.0.11 from
[Libcrux](https://github.com/celabshq/libcrux/tree/42d68bd49b244b09bd02626c6ffa95e54dc64f91/libcrux-ml-kem).
The published crate checksum is
`4ceed00367748ec0b545bc4a7b0521f8c9fd476d02484340f0a95b0f45f828cd`.

Cube changed `src/variant.rs` and `src/ind_cca.rs` on 2026-11-17 to select the
implicit-rejection derivation by profile. For CRYSTALS-Kyber 3.0 SHAKE the
pre-key on rejection is `z`, followed by the historical KDF with the ciphertext
hash. FIPS 203 keeps its existing `J(z || ciphertext)` secret. The existing
constant-time selection remains the single decision about the ciphertext.

The change is verified against independent valid and modified ciphertexts for
all three historical and all three FIPS 203 parameter sets in the Core test
suite. The local modification has not been included in the upstream proof runs.
