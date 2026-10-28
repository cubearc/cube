# Audio analysis

Inspect audio metadata, extract exact PCM samples, or analyze waveforms,
frequencies and signal timing. Use the [synthetic WAVE fixture](../../guides/fixtures.md)
to try the examples.

## Supported audio

| Input | Supported subset |
| --- | --- |
| RIFF/WAVE | Little-endian integer PCM, format tag 1; 8/16/24/32-bit samples, 1–32 channels |
| Native FLAC | STREAMINFO and optional Vorbis comments; metadata only |

WAVE rejects compressed, float and extensible formats, RIFX, RF64 and multiple
data chunks. Metadata inspection skips payloads and admits at most 1 GiB of
physical input and 10,000 chunks.

```sh
cube audio inspect sample.wav --json
cube audio waveform sample.wav --channels 0 --bucket-frames 2 --json
```

## PCM frames and waveform data

A frame contains one sample per channel. Selections use zero-based absolute frame
indexes and ordered, unique channel indexes.

| Representation | Meaning |
| --- | --- |
| Raw sample bytes | Original little-endian storage |
| Interpreted 8-bit samples | Unsigned storage minus 128 |
| Interpreted 16/24/32-bit samples | Signed two's complement |
| Waveform bucket | Exact min/max signed amplitudes and absolute start frame; final partial bucket retained |

There is no resampling, normalization or downmix. Raw PCM is not a WAVE file;
retain its interpretation metadata. Waveform results are data, without plotting
or playback; sample rate supplies the time base.

### PCM extraction limits

| Resource | Maximum |
| --- | --- |
| Selected frames | 1,048,576 |
| Duration | 300 seconds |
| Source bytes | 64 MiB |
| Raw output | 8 MiB |
| Structured sample listing | 10,000 samples |

Waveform budgets also bound channel/bucket pairs. Sample-block operations use
direct APIs and have no pipeline adapters.

## Spectrum

`audio.spectrum` analyzes exactly N selected PCM frames with an FFT. No implicit
padding or truncation is applied; empty or short selections fail.

| Setting or result | Rule |
| --- | --- |
| N | Power of two from 8 to 16,384 |
| Channels | Independent, in caller order |
| Frequencies | `k * sample_rate / N`, including DC and Nyquist |
| Input amplitude | Signed/centered PCM divided by `2^(bits-1)` |
| Window | Periodic Hann by default; rectangular and periodic Hamming available |
| Magnitudes | Divide by window sum; double interior bins |
| Units | One-sided peak full-scale amplitude, not power density or dB |
| Silence | Zero magnitude |

Results are numeric arrays or CLI JSON. Floating-point tolerance is required;
cross-platform bit identity is not guaranteed. FFT computation uses RustFFT
without optional SIMD features or a native backend.

### Spectrum limits

PCM limits apply first. FFT output allows at most 65,536 channel/bin cells and
4 Mi `channels * N * log2(N)` work units.

## Spectrogram

`audio.spectrogram` uses the same FFT, window and amplitude rules. The matrix is
`[channel][window][bin]`. `frame_starts` and `times_seconds` refer to absolute
source starts, not window centers.

| Setting | Behavior |
| --- | --- |
| `hop_size` | 1–N |
| `final_window=drop` (default) | Keep full windows and report uncovered tail |
| `final_window=pad` | Visit each hop start before the end; zero-fill tails with full-window normalization |
| `valid_frames` | Source frames contributing to each window |
| Empty input | No windows |
| Short input with drop | No windows |

The entire matrix shares the 65,536-cell and 4 Mi work budgets, checked before
FFT planning. Results contain data without a plotting dependency.

## Bit extraction

`audio.bits.extract` visits selected frames, then channels in caller order, then
unique selected bit positions. Reverse traversal reverses frames only.

Bit 0 is the native storage LSB. Extraction does not center amplitudes, sign-extend
or normalize little-endian samples.

| Option or result | Rule |
| --- | --- |
| Packing | MSB-first or LSB-first |
| Incomplete byte | Reject, drop or zero-pad |
| Counts | Available, valid and dropped bits |
| Visited bits | At most 16 Mi |
| Output | At most 2 MiB exact bytes, without guessed text |

PCM bounds also apply. Packing uses the shared image packing types.

## Audio reversal

`audio.reverse` creates a minimal PCM WAV with only `fmt` and `data` chunks.
It reverses frames while preserving selected channel order, sample rate, integer
width and native sample bytes. Bytes within a sample are never reversed.

Reversing twice restores samples, but not omitted ancillary metadata. Allocation
is limited to 8 MiB plus WAV header/padding; sizes and byte rate are checked first.

The CLI creates a new file exclusively and removes its partial output on failure.
The SDK returns WAV bytes without writing a file.

## DTMF analysis

DTMF analysis measures nominal frequencies from
[ITU-T Q.23](https://www.itu.int/rec/T-REC-Q.23/en). It requires one channel at
8 kHz or higher.

| Frequency group | Hz |
| --- | --- |
| Rows | 697, 770, 852, 941 |
| Columns | 1,209, 1,336, 1,477, 1,633 |

Goertzel measurements use nonoverlapping rectangular windows. Frequency offsets,
speech, distortion and mixed boundaries can be inconclusive; results are
signal-analysis evidence, not certified telephony decoding.

### DTMF defaults

| Parameter | Default |
| --- | --- |
| Window | 40 ms |
| Minimum tone / gap | 80 ms / 40 ms |
| Minimum RMS level | −40 dBFS |
| Dominance over second row/column | 6 dB |
| Maximum absolute twist | 8 dB |
| Minimum pair energy ratio | 75% |

Thresholds are explicit parameters in discovery and CLI help. Events retain
candidate symbols, measurements, reasons and uncertainty. Only runs meeting all
thresholds have a non-null `symbol`, still labeled `candidate`. Silence is separate.

Timing tolerance is one window. Limits are 262,144 frames and 4,096 windows, with
eight fixed frequency measurements per frame. There is no automatic frequency
search or device access.

## Morse detection and transcription

Detection measures tone intervals; transcription interprets their timing. These
are separate operations.

### Tone detection

`audio.morse.detect` requires one channel at 8 kHz or higher and a caller-supplied
frequency of at least 100 Hz and below Nyquist.

| Parameter | Default |
| --- | --- |
| Rectangular window | 10 ms |
| Minimum RMS level | −40 dBFS |
| Nominal-tone energy ratio | 80% |

Below-level windows are silence. Audible or partial windows without enough
expected tone energy remain uncertain. Adjacent equal states merge into absolute
frame intervals. Resolution is one window; limits are 262,144 frames and 4,096 windows.

### Timing interpretation

`audio.morse.transcribe` requires a dot duration of 40–1,000 ms. Relative tolerance
is 0–40%, default 20%.

[International Morse timing](https://www.itu.int/rec/R-REC-M.1677/en) uses 1/3-unit
tones and 1/3/7-unit internal gaps. Boundary silence is ignored.

Invalid, noncontiguous or overlapping ranges fail. Uncertain or ambiguous timing
returns no text. Recognized tokens use the Morse text decoder; unsupported
sequences retain tokens and `unsupported_tokens` without inventing letters.

No frequency search, speed estimation, radio capture or forced transcription occurs.

## FLAC metadata

`audio.metadata.inspect` reads native FLAC STREAMINFO and one optional
Vorbis-comment block. MP3, Ogg, ID3-prefixed FLAC and reserved block types are
unsupported.

| Result | Interpretation |
| --- | --- |
| Rate, channels, bit depth | Declared metadata |
| Comments | Duplicate key/value entries preserved |
| Block offsets | Source positions |
| Duration | Millisecond precision; zero total samples means unknown |
| Declared MD5 | `null` when absent or all zero; never verified |

A metadata result does not establish a valid audio stream. Audio payload,
artwork and uninterpreted blocks are not expanded.

### FLAC limits

| Resource | Maximum |
| --- | --- |
| Physical source | 1 GiB |
| Total metadata | 1 MiB |
| Blocks / comments | 256 |
| Vendor or comment field | 64 KiB |
| Exported text | 16 KiB |

Framing, extents and comment lengths are checked before Lofty parsing.
Duplicate STREAMINFO/comment blocks, truncation and invalid text fail.
No audio payload is read, transcoded or decompressed.

The native-FLAC entry rejects ID3 before parsing and changes no parser-global
settings. Format rules follow [RFC 9639](https://www.rfc-editor.org/rfc/rfc9639).
For media containers, see [Video](video-container-metadata.md).
