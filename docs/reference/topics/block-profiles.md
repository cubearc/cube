# Raw block reference profiles

Encrypt or decrypt exact blocks for legacy interoperability and known-answer
tests. Select an explicit profile and supply the exact key bytes. These
unaudited backends do not promise constant-time execution or provide secure
message encryption. Raw blocks have no authentication: a wrong valid key still
produces bytes on decryption.

## Operations and profiles

`cipher.block.encrypt_blocks` and `cipher.block.decrypt_blocks` return exact
bytes under the selected full-round profile. Block widths and key lengths are
explicit below. There is no default profile or key.

### Legacy interoperability

| Profile | Block bytes | Key bytes | Definition |
| --- | --- | --- | --- |
| `des_fips46_v1` | 8 | 8 | DES, 16 rounds; parity bits ignored |
| `tdes_ede2_v1` | 8 | 16 | Triple DES EDE with K1/K2/K1 |
| `tdes_ede3_v1` | 8 | 24 | Triple DES EDE with K1/K2/K3 |
| `blowfish_be_v1` | 8 | 4..56 | Blowfish, 16 rounds, big-endian words |
| `idea_v1` | 8 | 16 | IDEA, eight rounds plus output, big-endian words |
| `rc2_rfc2268_v1` | 8 | 1..128 | RFC 2268, little-endian words; explicit effective bits |
| `cast128_rfc2144_v1` | 8 | 5..16 | CAST-128; 12 rounds for keys up to 10 bytes, otherwise 16 |
| `cast256_rfc2612_v1` | 16 | 16, 20, 24, 28, 32 | CAST-256, 48 rounds, big-endian words |
| `rc5_32_12_16_le_v1` | 8 | 16 | RC5, 32-bit words, 12 rounds, little-endian bytes |
| `rc6_32_20_v1` | 16 | 16, 24, 32 | RC6, 32-bit words, 20 rounds, little-endian bytes |
| `skipjack_nist_v1` | 8 | 10 | NIST SKIPJACK, 32 rounds, big-endian words |

### Full-round 128-bit block profiles

| Profile | Key bytes | Definition |
| --- | --- | --- |
| `camellia_rfc3713_v1` | 16, 24, 32 | RFC 3713; 18 or 24 rounds |
| `serpent_final_v1` | 16, 24, 32 | Final Serpent, 32 rounds; standard external bytes |
| `twofish_v1` | 16, 24, 32 | Original Twofish, 16 rounds; little-endian words |
| `aria_rfc5794_v1` | 16, 24, 32 | RFC 5794; 12, 14 or 16 rounds |
| `sm4_gmt0002_v1` | 16 | GM/T 0002 SM4, 32 rounds; big-endian words |
| `kuznyechik_rfc7801_v1` | 32 | RFC 7801 Kuznyechik, ten round keys |
| `noekeon_direct_v1` | 16 | Direct-key NOEKEON, 16 rounds plus final key step |
| `lea_v1` | 16, 24, 32 | LEA, 24, 28 or 32 rounds; little-endian words |
| `gift128_v1` | 16 | GIFT-128, 40 rounds; original byte representation |

### Original Rijndael widths

| Profile | Block bytes | Key bytes |
| --- | --- | --- |
| `rijndael_128_v1` | 16 | 16, 20, 24, 28, 32 |
| `rijndael_160_v1` | 20 | 16, 20, 24, 28, 32 |
| `rijndael_192_v1` | 24 | 16, 20, 24, 28, 32 |
| `rijndael_224_v1` | 28 | 16, 20, 24, 28, 32 |
| `rijndael_256_v1` | 32 | 16, 20, 24, 28, 32 |

Rijndael uses column-major bytes and `max(block bytes, key bytes) / 4 + 6`
rounds. Its 224-bit block uses row shifts 0/1/2/4; its 256-bit block uses
0/1/3/4. Standard AES tuples use the separate AES backend internally.

### Small and variable block profiles

| Profile | Block bytes | Key bytes | Definition |
| --- | --- | --- | --- |
| `gost28147_test_le_v1` | 8 | 32 | GOST 28147-89 Applied Cryptography test S-box; little-endian words |
| `gost28147_cryptopro_a_le_v1` | 8 | 32 | GOST 28147-89 CryptoPro A; little-endian words |
| `gost28147_cryptopro_b_le_v1` | 8 | 32 | GOST 28147-89 CryptoPro B; little-endian words |
| `gost28147_cryptopro_c_le_v1` | 8 | 32 | GOST 28147-89 CryptoPro C; little-endian words |
| `tea32_be_v1` | 8 | 16 | Original TEA, 32 cycles; big-endian words |
| `shacal2_be_v1` | 32 | 16..64, multiples of 4 | SHACAL-2, 64 rounds; big-endian words and key zero-fill to 64 bytes |
| `twine80_v1` | 8 | 10 | TWINE-80, 36 rounds; high nibble first |
| `twine128_v1` | 8 | 16 | TWINE-128, 36 rounds; high nibble first |
| `hight_v1` | 8 | 16 | HIGHT, 32 rounds and whitening; original byte ordering |
| `misty1_rfc2994_v1` | 8 | 16 | RFC 2994 MISTY1, eight rounds and final FL; big-endian words |
| `kasumi_3gpp_v1` | 8 | 16 | 3GPP KASUMI, eight rounds; big-endian words |
| `simon32_64_be_v1` | 4 | 8 | Original SIMON full-round tuple; big-endian paper words |
| `simon48_72_be_v1` | 6 | 9 | Original SIMON full-round tuple; big-endian paper words |
| `simon48_96_be_v1` | 6 | 12 | Original SIMON full-round tuple; big-endian paper words |
| `simon64_96_be_v1` | 8 | 12 | Original SIMON full-round tuple; big-endian paper words |
| `simon64_128_be_v1` | 8 | 16 | Original SIMON full-round tuple; big-endian paper words |
| `simon96_96_be_v1` | 12 | 12 | Original SIMON full-round tuple; big-endian paper words |
| `simon96_144_be_v1` | 12 | 18 | Original SIMON full-round tuple; big-endian paper words |
| `simon128_128_be_v1` | 16 | 16 | Original SIMON full-round tuple; big-endian paper words |
| `simon128_192_be_v1` | 16 | 24 | Original SIMON full-round tuple; big-endian paper words |
| `simon128_256_be_v1` | 16 | 32 | Original SIMON full-round tuple; big-endian paper words |
| `skinny64_64_v1` | 8 | 8 | Original SKINNY full-round tuple; complete required tweakey bytes |
| `skinny64_128_v1` | 8 | 16 | Original SKINNY full-round tuple; complete required tweakey bytes |
| `skinny64_192_v1` | 8 | 24 | Original SKINNY full-round tuple; complete required tweakey bytes |
| `skinny128_128_v1` | 16 | 16 | Original SKINNY full-round tuple; complete required tweakey bytes |
| `skinny128_256_v1` | 16 | 32 | Original SKINNY full-round tuple; complete required tweakey bytes |
| `skinny128_384_v1` | 16 | 48 | Original SKINNY full-round tuple; complete required tweakey bytes |
| `safer_k64_6_v1` | 8 | 8 | SAFER K; exactly 6 rounds |
| `safer_k128_12_v1` | 8 | 16 | SAFER K; exactly 12 rounds |
| `safer_sk64_6_v1` | 8 | 8 | SAFER SK; exactly 6 rounds |
| `safer_sk128_10_v1` | 8 | 16 | SAFER SK; exactly 10 rounds |
| `safer_k128_10_v1` | 8 | 16 | SAFER K; exactly 10 rounds |
| `safer_sk64_8_v1` | 8 | 8 | SAFER SK; exactly 8 rounds |
| `anubis_original_v1` | 16 | 16, 20, 24, 28, 32, 36, 40 | Original Anubis S-box; 8 + key words rounds |
| `anubis_tweaked_v1` | 16 | 16, 20, 24, 28, 32, 36, 40 | Tweaked Anubis S-box; 8 + key words rounds |
| `khazad_v1` | 8 | 16 | Khazad; full eight rounds, big-endian bytes |
| `square_v1` | 16 | 16 | Square; full eight rounds, big-endian bytes |
| `seed_rfc4269_v1` | 16 | 16 | SEED RFC 4269; 128-bit key, sixteen rounds |
| `clefia_rfc6114_v1` | 16 | 16, 24, 32 | CLEFIA RFC 6114; 18/22/26 rounds |
| `mars_final_le_v1` | 16 | 16, 24, 32 | Final MARS multiplication-key tweak; little-endian words |
| `crypton10_le_v1` | 16 | 16, 24, 32 | CRYPTON version 1.0; twelve rounds, little-endian words |
| `e2_ntt_v1` | 16 | 16, 24, 32 | E2 full twelve rounds and original cumulative key state; standard bytes |
| `craft_2020_v1` | 8 | 16 | Full 32 rounds; explicit required 8-byte tweak |
| `crypton_original_le_v1` | 16 | 16, 24, 32 | Original AES submission; distinct from revised version 1.0 |
| `boron128_v1` | 8 | 16 | Original 25-round BORON; Appendix zero128 discrepancy documented below |
| `boron80_v1` | 8 | 10 | Original 25-round BORON; Appendix zero128 discrepancy documented below |
| `gift64_v1` | 8 | 16 | Original GIFT64, 28 rounds |
| `hierocrypt3_2001_be_v1` | 16 | 16, 24, 32 | Exact NESSIE L1 H4 / Toshiba 2001 Hierocrypt-3; separate profiles |
| `hierocrypt_l1_nessie_be_v1` | 8 | 16 | Exact NESSIE L1 H4 / Toshiba 2001 Hierocrypt-3; separate profiles |
| `katan32_be_v1` | 4 | 10 | Full 254-round original KATAN / KTANTAN; numeric big-endian bytes |
| `katan48_be_v1` | 6 | 10 | Full 254-round original KATAN / KTANTAN; numeric big-endian bytes |
| `katan64_be_v1` | 8 | 10 | Full 254-round original KATAN / KTANTAN; numeric big-endian bytes |
| `klein64_v1` | 8 | 8 | Original KLEIN 12/16/20 rounds |
| `klein80_v1` | 8 | 10 | Original KLEIN 12/16/20 rounds |
| `klein96_v1` | 8 | 12 | Original KLEIN 12/16/20 rounds |
| `ktantan32_be_v1` | 4 | 10 | Full 254-round original KATAN / KTANTAN; numeric big-endian bytes |
| `ktantan48_be_v1` | 6 | 10 | Full 254-round original KATAN / KTANTAN; numeric big-endian bytes |
| `ktantan64_be_v1` | 8 | 10 | Full 254-round original KATAN / KTANTAN; numeric big-endian bytes |
| `lblock_corrected_v1` | 8 | 10 | Corrected original LBlock full 32 rounds |
| `led128_v1` | 8 | 16 | Original LED-64/128 full 32/48 rounds |
| `led64_v1` | 8 | 8 | Original LED-64/128 full 32/48 rounds |
| `midori128_v1` | 16 | 16 | Original Midori128, 20 rounds |
| `midori64_v1` | 8 | 16 | Original Midori64, 16 rounds |
| `noekeon_indirect_be_v1` | 16 | 16 | Original NESSIE indirect working-key derivation, separate from direct |
| `piccolo128_be_v1` | 8 | 16 | Original Piccolo 25/31 rounds; paper BE branch convention |
| `piccolo80_be_v1` | 8 | 10 | Original Piccolo 25/31 rounds; paper BE branch convention |
| `pride_original_v1` | 8 | 16 | Original PRIDE full twenty rounds; published byte convention |
| `prince_original_v1` | 8 | 16 | Original PRINCE full twelve rounds |
| `rectangle128_be_v1` | 8 | 16 | Revised 2016 RECTANGLE full25 rounds; paper BE rows |
| `rectangle80_be_v1` | 8 | 10 | Revised 2016 RECTANGLE full25 rounds; paper BE rows |
| `magma_rfc8891_v1` | 8 | 32 | RFC 8891, fixed TC26 S-box; big-endian words |
| `xtea32_le_v1` | 8 | 16 | XTEA, 32 cycles; little-endian words |
| `present80_v1` | 8 | 10 | PRESENT-80, 31 rounds plus whitening; big-endian bytes |
| `present128_v1` | 8 | 16 | PRESENT-128, 31 rounds plus whitening; big-endian bytes |
| `speck32_64_be_v1` | 4 | 8 | Original full-round SPECK 32/64; big-endian word tuple |
| `speck48_72_be_v1` | 6 | 9 | Original full-round SPECK 48/72; big-endian word tuple |
| `speck48_96_be_v1` | 6 | 12 | Original full-round SPECK 48/96; big-endian word tuple |
| `speck64_96_be_v1` | 8 | 12 | Original full-round SPECK 64/96; big-endian word tuple |
| `speck64_128_be_v1` | 8 | 16 | Original full-round SPECK 64/128; big-endian word tuple |
| `speck96_96_be_v1` | 12 | 12 | Original full-round SPECK 96/96; big-endian word tuple |
| `speck96_144_be_v1` | 12 | 18 | Original full-round SPECK 96/144; big-endian word tuple |
| `speck128_128_be_v1` | 16 | 16 | Original full-round SPECK 128/128; big-endian word tuple |
| `speck128_192_be_v1` | 16 | 24 | Original full-round SPECK 128/192; big-endian word tuple |
| `speck128_256_be_v1` | 16 | 32 | Original full-round SPECK 128/256; big-endian word tuple |
RC2 requires `effective_key_bits` in 1..1024, independently of the key length.
Other profiles reject that setting. GOST selects a fixed named S-box; none is
inferred from a family name or supplied as an arbitrary table. No effective-bit count is inferred from
the key. DES-family parity is not validated or repaired. Weak and degenerate keys
are accepted for compatibility with historical data and test vectors.

Equal input blocks produce equal output blocks. Input length must be a multiple
of the profile's block width; empty input returns empty bytes after settings
validation. No mode, padding, IV, nonce, counter, KDF, RNG, tag or serialized
message format is added. See [raw AES blocks](aes-blocks.md) for the separate
FIPS 197 interface, and [XXTEA word sequences](xxtea.md) for its distinct
variable-width raw-block convention.

## Inputs and results

Both version 1 operations name their byte input `input` and byte result `output`.
Canonical JSON uses `{"base64":"..."}` for each byte port and requires
`profile` and `key_hex` parameters. Keys use exact hexadecimal byte pairs,
without prefixes or separators; upper- and lowercase hex letters are accepted.
RC2 additionally requires `effective_key_bits`. CRAFT additionally requires
`tweak_hex` containing exactly eight hexadecimal bytes. Other profiles reject
the tweak field. Unknown or duplicate fields and null settings are rejected.

The CLI reads a raw key file and a payload file or stdin. It writes binary stdout:

```sh
cube cipher block encrypt --profile des_fips46_v1 \
  --key-file key.bin --file blocks.bin > encrypted.bin
cube cipher block decrypt --profile des_fips46_v1 \
  --key-file key.bin --file encrypted.bin > restored.bin
```

With the public eight-byte key `0113b970fd34f2ce` and input
`059b5e0851cf143a`, encryption produces `86a560f10ec6d85b`. The hexadecimal
strings describe bytes; the files contain the corresponding raw bytes.

Rust calls `ciphers::block_profiles::Settings::new`, `Settings::new_with_tweak`
for CRAFT, or `Settings::parse`, then
`encrypt(data, &settings)` or `decrypt(data, &settings)`. Native Python calls
`block_encrypt_blocks(data, key, profile, *, effective_key_bits=None, tweak=None)` or
`block_decrypt_blocks` with the same arguments. JavaScript and TypeScript use
the operation IDs through `cube.execute`, with `Uint8Array` byte ports and the
generated profile-specific parameter types. All interfaces execute Core.

## Limits and privacy

| Resource or failure | Rule |
| --- | --- |
| Input and output | At most 1 MiB each; output has the input length |
| Educational reference cap | 64 KiB for Rijndael, SKIPJACK, NOEKEON, LEA, PRESENT, TEA, SHACAL-2, TWINE, HIGHT, MISTY1, KASUMI, SIMON, SKINNY, SAFER, Anubis, Khazad, Square, SEED, CLEFIA, MARS, CRYPTON, E2, PRINCE, LED, Midori, GIFT64, KATAN/KTANTAN, PRIDE, KLEIN, LBlock, RECTANGLE, BORON, Piccolo, Hierocrypt and CRAFT |
| Work | At most 262,144 independent blocks; each profile also enforces its byte ceiling and width |
| Private parameter JSON | At most 4096 UTF-8 bytes |
| Invalid profile, key or settings | `invalid_parameter`; no secret excerpts |
| Partial block | `invalid_input`; no output or padding |
| Exhausted byte/parameter budget | `resource_limit` before output allocation |

Host transport budgets apply separately and can be lower. See
[resource limits](../../concepts/resource-limits.md) and the
[canonical contracts](../operations.md) for the exact named schemas.

CLI keys come only from a raw file. Keys cannot be passed as text/hex arguments,
captured with `--lab-capture` or used in pipeline/recipe steps. Play requires its
explicit private-output retention policy; key parameters and literal inputs stay
out of saved configuration and provenance. Byte outputs use the existing viewer.
No network access or automatic capture is introduced.

Owned decoded keys and supported backend schedules are wiped on drop. Temporary backend
key-expansion arrays, caller buffers, parser/interpreter/compiler copies and OS
memory are not guaranteed erased. Returned bytes belong to the caller. Small
block sizes, obsolete algorithms and weak keys retain their original weaknesses.

Sources: [RustCrypto qualification](../../contributing/block-backends.md),
[RC2 specification and vectors](https://www.rfc-editor.org/rfc/rfc2268.txt),
[Blowfish author vectors](https://www.schneier.com/wp-content/uploads/2015/12/vectors-2.txt).


BORON follows the original 25-round equations. Its published Appendix A zero-key,
zero-plaintext 128-bit answer prints `94a105a7d2f2de42`; two independent evaluations
of the specified equations give `94a105a7d1f2de42`. Both reproduce the other three
Appendix answers, including the nonzero-plaintext BORON-128 answer. The shared
corpus labels the inconsistent zero case as equation-derived, not a published
known answer. The algorithm is never modified to fit that digit.

`hierocrypt_l1_nessie_be_v1` explicitly fixes the NESSIE reference convention
`H4 = 0x8af5def7`, which reproduces Toshiba's original section 4 answer. The
September 2001 PDF prints `0xf7def58a`; that literal-constant variant gives other
bytes and is not silently substituted. Hierocrypt-3 retains its distinct
128-bit blocks and all three full-round key widths. Noekeon's direct and
indirect schedules have separate IDs and separate author-published answers.


CRAFT requires an explicit eight-byte tweak for every call. Python accepts the
keyword `tweak=...`, the canonical JS/TS/Wasm request requires `tweak_hex`, and
CLI requires `--tweak-file`. The tweak is separate from the128-bit key. There
is no implicit zero tweak; other profiles reject a tweak. An explicit zero
value is valid reference input. This raw tweakable permutation is not AEAD.
