# Binary format reference fixtures

The plain-text vectors contain hexadecimal digest bytes from fixed public
reference inputs. The digest tests use an unaligned start and preserve the
reference wire byte order independently of the target's byte order.

| Fixture | Reference and input |
| --- | --- |
| `xxhash-vectors.txt` | [xxHash v0.8.3][xxhash] C `XXH32` and `XXH64` on ascending byte prefixes. Columns are length, XXH32 at seed `0x12345678`, XXH64 at seed `0xfedcba9876543210`. |
| `murmur-vectors.txt` | [Original SMHasher MurmurHash3][murmur], compiled on little-endian hardware over ascending byte prefixes. Columns are length, seed, four x86-32 output bytes, and sixteen x64-128 output bytes. |
| `siphash24.txt` | [Original SipHash tests][sip] with the 16-byte key 00 through 0f and ascending message prefixes of length zero through 63. Each line is the reference eight-byte result in order. |

[xxhash]: https://github.com/Cyan4973/xxHash/tree/v0.8.3
[murmur]: https://github.com/aappleby/smhasher/blob/32d067603b93b47828700880649198e0bfbbcffa/src/MurmurHash3.cpp
[sip]: https://github.com/veorq/SipHash/blob/07bb4de10a63e8cc2e1724865454eba635742383/vectors.h
