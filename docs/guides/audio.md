# Inspect audio samples and signals

Start with the original WAVE fixture. Frame/channel selection and sample interpretation are explicit; raw PCM and a WAVE container are different outputs.

Prepare a small input with [original media fixtures](fixtures.md). Examples with collections, a second image or FLAC require those explicit caller-supplied inputs.

## CLI

### Inspect audio samples

```sh
cube audio inspect sample.wav --json
cube audio samples sample.wav --channels 0 --frames 4 --json
cube audio samples sample.wav --channels 0 --frames 4 --output channel.pcm
cube audio waveform sample.wav --channels 0 --bucket-frames 2 --json
```


```sh
python - <<'PYTHON'
import wave
with wave.open("spectrum.wav", "wb") as output:
    output.setnchannels(1)
    output.setsampwidth(2)
    output.setframerate(8000)
    output.writeframes(b"\x00\x00\x00\x40\x00\x00\x00\xc0" * 2)
PYTHON
cube audio spectrum spectrum.wav --channels 0 --fft-size 8 --json
cube audio spectrogram spectrum.wav --channels 0 --fft-size 8 --hop-size 4 --final-window pad --json
cube audio bits spectrum.wav --channels 0 --bits 15 --frames 8 --json
cube audio reverse spectrum.wav --output reversed-spectrum.wav
cube audio dtmf spectrum.wav --channels 0 --json
```


```sh
python - <<'PYTHON'
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
# tokens: ".", text: "E"
```


```sh
cube audio metadata sample.flac --json
```


## Python

### Work with PCM WAVE data

```python
import cube

wave_info = cube.inspect_wave_file("sample.wav")
assert wave_info["format"]["sample_rate"] == 8000
samples = cube.wave_file_samples("sample.wav", channels=[0], frames=4, include_values=True)
assert samples["values"] == [0, 16384, 0, -16384]
assert isinstance(samples["data"], bytes)
envelope = cube.wave_file_envelope("sample.wav", channels=[0], bucket_frames=2)
assert len(envelope["buckets"]) == 2
```


### Numerical spectrum

```python
import struct
import cube
from io import BytesIO
import wave

buffer = BytesIO()
with wave.open(buffer, "wb") as writer:
    writer.setnchannels(1)
    writer.setsampwidth(2)
    writer.setframerate(8000)
    writer.writeframes(struct.pack("<8h", *([0] * 8)))
spectrum = cube.wave_spectrum(buffer.getvalue(), fft_size=8, channels=[0])
assert spectrum["frequencies_hz"] == [0, 1000, 2000, 3000, 4000]
assert spectrum["amplitudes"] == [[0.0] * 5]
gram = cube.wave_spectrogram(buffer.getvalue(), fft_size=8, hop_size=4, final_window="pad")
assert gram["frame_starts"] == [0, 4]
assert gram["valid_frames"] == [8, 4]
packed = cube.wave_bits(buffer.getvalue(), bits=[0])
assert packed["data"] == b"\x00"
assert packed["valid_bits"] == 8
reversed_wav = cube.reverse_wave(buffer.getvalue())
assert cube.wave_samples(cube.reverse_wave(reversed_wav))["data"] == bytes(16)
assert cube.detect_dtmf(buffer.getvalue())["events"] == []
```


### Inspect tone intervals before transcription

```python
import math
import struct
import wave
from io import BytesIO
import cube

values = [round(12000 * math.sin(math.tau * 700 * i / 8000)) for i in range(800)]
buffer = BytesIO()
with wave.open(buffer, "wb") as writer:
    writer.setnchannels(1)
    writer.setsampwidth(2)
    writer.setframerate(8000)
    writer.writeframes(struct.pack("<800h", *values))
intervals = cube.detect_morse_intervals(buffer.getvalue(), frequency_hz=700)
assert "text" not in intervals
transcript = cube.transcribe_morse(intervals["intervals"], sample_rate=8000, unit_ms=100)
assert transcript["tokens"] == "." and transcript["text"] == "E"
```


### Inspect compressed-audio metadata

```python
from pathlib import Path
import cube

metadata = cube.inspect_audio_metadata(Path("sample.flac").read_bytes())
assert metadata["format"] == "flac"
assert metadata["verification"] == "metadata_only_audio_payload_unverified"
```


## Reference

- [Audio analysis](../reference/topics/audio-analysis.md)
