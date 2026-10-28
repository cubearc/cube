# Inspect audio samples and signals

Read PCM samples, measure a spectrum, or inspect the timing of a tone.
Create `sample.wav` with the [sample files guide](fixtures.md) first. It contains
four mono samples at 8,000 frames per second.

## Read samples

```sh
cube audio inspect sample.wav --json
cube audio samples sample.wav --channels 0 --frames 4 --json
cube audio waveform sample.wav --channels 0 --bucket-frames 2 --json
```

The sample values are `0, 16384, 0, -16384`. The waveform report groups them into
two buckets and reports each bucket's minimum and maximum.

To save the selected sample bytes:

```sh
cube audio samples sample.wav --channels 0 --frames 4 --output channel.pcm
```

`channel.pcm` contains raw samples, without a WAVE header. Keep the sample rate,
channel count and sample width with it.

## Measure a spectrum

This example needs eight frames. Use Python's standard library to create them:

```sh
python3 - <<'PYTHON'
import wave

with wave.open("spectrum.wav", "wb") as output:
    output.setnchannels(1)
    output.setsampwidth(2)
    output.setframerate(8000)
    output.writeframes(b"\x00\x00\x00\x40\x00\x00\x00\xc0" * 2)
PYTHON
cube audio spectrum spectrum.wav --channels 0 --fft-size 8 --json
cube audio spectrogram spectrum.wav --channels 0 --fft-size 8 --hop-size 4 --final-window pad --json
```

The spectrum reports frequencies from 0 to 4,000 Hz in 1,000 Hz steps.
The spectrogram starts a window every four frames; `pad` fills the final short
window with zeros. Both commands return numerical data.

## Transcribe a known Morse tone

Create a 100 ms tone at 700 Hz, then detect its intervals:

```sh
python3 - <<'PYTHON'
import math
import struct
import wave

values = [round(12000 * math.sin(math.tau * 700 * i / 8000)) for i in range(800)]
with wave.open("morse-dot.wav", "wb") as output:
    output.setnchannels(1)
    output.setsampwidth(2)
    output.setframerate(8000)
    output.writeframes(struct.pack("<800h", *values))
PYTHON
cube audio morse detect morse-dot.wav --frequency-hz 700 --json > intervals.json
cube audio morse transcribe --unit-ms 100 --json < intervals.json
```

With a 100 ms dot duration, the result is `.`, or `E`. Detection measures the
supplied frequency; transcription interprets the timing you choose. Ambiguous
intervals can produce no text.

For bit extraction, reversal, DTMF, FLAC metadata and supported formats, see
[audio analysis](../reference/topics/audio-analysis.md).

<a id="cli"></a>
<a id="inspect-audio-samples"></a>
<a id="inspect-compressed-audio-metadata"></a>
<a id="inspect-tone-intervals-before-transcription"></a>
<a id="numerical-spectrum"></a>
<a id="python"></a>
<a id="reference"></a>
<a id="work-with-pcm-wave-data"></a>
