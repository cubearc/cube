### Audio analysis

`audio.spectrum` analyzes exactly N selected integer PCM frames with a power-of-two
FFT in 8..16,384. It never truncates or pads implicitly. Each selected channel is
independent, in caller order; frequencies are `k*sample_rate/N`, including DC and
Nyquist. Samples use existing signed/centered PCM decoding divided by `2^(bits-1)`.
Rectangular, periodic Hann (default) and periodic Hamming windows are available.
Magnitudes divide by the window sum and double interior bins, giving one-sided
peak full-scale amplitude, not power density or dB. Silence gives zero, and empty
or short selections fail. Numeric results agree within floating-point tolerance;
they are not guaranteed bit-identical across platforms.

At most 65,536 channel/bin cells and 4 Mi `channels*N*log2(N)` work units are
allowed. Existing PCM input/range/sample limits apply first. RustFFT is used without
optional SIMD features or a native backend. Results are structured Rust/Python
numeric arrays and CLI JSON. No pipeline adapter or new pipeline value is added
for sample blocks; existing audio APIs and recipes remain unchanged.


`audio.spectrogram` reuses these FFT/window/amplitude rules. Its matrix is
`[channel][window][bin]`; `frame_starts` and `times_seconds` refer to absolute
source window starts, not centers. Choose `hop_size` in 1..N. `final_window=drop`
(default) keeps full windows and reports the uncovered tail; `pad` visits every
hop start before end and zero-fills tails using full-window normalization.
`valid_frames` records how much source data each window contains. Empty input
produces no windows; short input produces no windows with drop. The same 65,536
cell/4 Mi work budgets cover the entire matrix, checked before FFT planning.
No plotting dependency or second FFT implementation is added.

`audio.bits.extract` visits selected frames, selected channels in their caller
order, then requested unique bit positions. Reverse traversal reverses frames
without reversing channel groups. Bit zero means the native storage LSB, including
unsigned 8-bit PCM and two's-complement 16/24/32-bit PCM; little-endian storage
never gets amplitude centering, sign extension or normalization for extraction.
Packing reuses the image MSB/LSB and reject/drop/zero-pad rules and reports available,
valid and dropped bit counts. It allows 16 Mi visited bits/2 MiB output, in addition
to existing PCM bounds. Output is exact bytes, never guessed text. The same shared
packing types remain available at their existing image API paths.

`audio.reverse` generates a new minimal PCM WAV containing only fmt/data chunks.
It reverses time frames while preserving selected channel order, rate, integer
width and native sample bytes; it never reverses bytes within samples. Reversing
twice restores sample data. Source ancillary metadata is omitted, so this is not
a byte-for-byte container round trip. Existing PCM limits bound allocation to
8 MiB plus the WAV header/pad; sizes and byte rate are checked before writing.
The CLI creates a new file exclusively and removes a partial output on write
failure. The SDK returns new WAV bytes and performs no implicit file write.

DTMF analysis evaluates the nominal row frequencies 697/770/852/941 Hz and
columns 1209/1336/1477/1633 Hz from [ITU-T Q.23](https://www.itu.int/rec/T-REC-Q.23/en).
It requires exactly one selected channel and a sample rate of at least 8 kHz.
Bounded Goertzel measurements use nonoverlapping rectangular windows; frequency
offsets, speech, distortion and boundary mixtures can be inconclusive. This is
signal-analysis evidence, not a certified telephony receiver or confirmed string.

Defaults: 40 ms windows, minimum 80 ms tones/40 ms gaps, RMS level >=-40 dBFS,
6 dB dominance over the second row/column, <=8 dB absolute twist and >=75% pair
energy ratio. All thresholds are explicit advanced parameters in discovery/help.
Partial tone windows, ambiguous bands, excessive twist, short runs and insufficient
gaps retain reasons and uncertainty. Silence has a separate state. Events retain
candidate symbols and measured windows; only runs meeting all selected thresholds
have a non-null `symbol`, still labeled `candidate`. Timing tolerance is one window.
Limits are 262,144 frames and 4,096 windows, with one fixed set of eight frequency
measurements per frame. There is no automatic frequency search or device access.

Morse-like analysis has two separate operations. `audio.morse.detect` measures a
caller-supplied nominal frequency (>=100 Hz, below Nyquist) on exactly one channel
at >=8 kHz. Defaults are 10 ms rectangular windows, -40 dBFS minimum RMS and 80%
nominal-tone energy ratio. Below-level windows are silence; other audible or
partial windows without enough expected tone energy remain uncertain. Adjacent
same-state windows merge into exact absolute frame intervals. Resolution is one
window, with the same 262,144-frame/4,096-window budget as signal analysis.

`audio.morse.transcribe` separately requires a dot duration in 40..1,000 ms and
relative tolerance in 0..40% (default 20%). It uses 1/3-unit tones and 1/3/7-unit
internal gaps from [International Morse timing](https://www.itu.int/rec/R-REC-M.1677/en).
Boundary silence is ignored. Invalid/noncontiguous/overlapping interval ranges
fail; uncertain timings or ambiguous matches return no text. Recognized tokens
reuse Cube's existing Morse text decoder. Unsupported token sequences preserve
tokens with `unsupported_tokens` and no invented letter. No automatic frequency
search, speed estimation, radio capture or forced transcription occurs.

`audio.metadata.inspect` supports native FLAC STREAMINFO and one optional
Vorbis-comment block through Lofty. MP3, Ogg, ID3-prefixed FLAC and reserved block
types are unsupported. It reports declared sample rate/channels/bit depth, duplicate
key/value comments, block offsets and duration at millisecond precision. Zero
total samples means unknown duration; an absent/all-zero declared MD5 is null.
The audio payload and MD5 are unverified, so a metadata result is not proof of a
valid audio stream. Artwork and noninterpreted blocks are never expanded.

Limits: 1 GiB physical source, 1 MiB total metadata, 256 blocks/comments, 64 KiB per
vendor/comment field and 16 KiB exported text. Framing, extents and comment lengths
are preflighted before maintained parsing; duplicate STREAMINFO/comment blocks,
truncation and invalid text fail. The operation reads no audio payload, performs
no transcoding/decompression and changes no parser-global settings. Lofty's
maintained default feature configuration is required by an upstream build issue;
Cube's native-FLAC-only entry rejects ID3 before parsing, so compressed-tag code
is unreachable. Format rules follow [RFC 9639](https://www.rfc-editor.org/rfc/rfc9639).
This separate operation preserves the existing integer-WAVE `audio inspect` API.

Use `audio` for RIFF/WAVE metadata, exact channel/frame samples and min/max waveform
data. PCM stores sample amplitudes directly, without a compressed audio codec.
Start with the [synthetic WAVE fixture](../../guides/fixtures.md).

```sh
cube audio inspect sample.wav --json
cube audio waveform sample.wav --channels 0 --bucket-frames 2 --json
```

Supported audio is little-endian integer PCM RIFF/WAVE (format tag 1), with
8/16/24/32-bit samples and 1–32 channels. Compressed/float/extensible WAVE, RIFX,
RF64 and multiple data chunks are unsupported. Metadata inspection skips payloads;
physical input is capped at 1 GiB and 10000 chunks.

A frame contains one sample per channel. Selections use zero-based absolute frames
and ordered unique channel indexes. Raw output preserves original little-endian
sample bytes; 8-bit storage is unsigned, while interpreted values subtract 128.
Wider samples use signed two's complement. There is no resampling, normalization
or downmix. Raw PCM output is not a WAVE file; retain its interpretation metadata.

Sample extraction caps selections at 1048576 frames, 300 seconds and 64 MiB source
bytes, with 8 MiB raw output. Structured value listings cap at 10000 samples.
Waveform buckets contain exact min/max signed amplitudes, absolute start frames
and the final partial bucket; the sample rate supplies the time base. Budgets
bound channel/bucket pairs. This is data, not a plot, spectrogram, FFT, Morse/DTMF
detector or playback feature. Video container inspection is described below.
