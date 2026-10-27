# Operation reference

Generated from Core descriptors and typed schemas. Edit their owning Rust declarations, then run `cargo run -p cube-core --example operation_contracts`. This reference lists implemented operations only.

Schemas: [versioned catalog](../../contracts/v1/catalog.json). Structural contracts do not grant capabilities or replace operation runtime validation.

## audio.bits.extract

Exact raw bit extraction from native PCM storage; frame/channel/bit order, optional reverse frames, shared MSB/LSB and reject/drop/pad semantics; <=16 Mi bits.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `audio.pcm_samples` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "bit_order": {
      "description": "First bit goes into bit 7 (msb) or bit 0 (lsb) of each output byte.",
      "type": "string",
      "default": "msb",
      "enum": [
        "msb",
        "lsb"
      ]
    },
    "bits": {
      "description": "Ordered unique native sample bit indexes, zero is LSB; must fit selected sample width.",
      "type": "array",
      "default": [
        0
      ],
      "items": {
        "type": "integer",
        "maximum": 31,
        "minimum": 0
      },
      "maxItems": 32,
      "minItems": 1,
      "uniqueItems": true
    },
    "incomplete": {
      "description": "Policy for final incomplete output byte: reject, drop or zero-pad.",
      "type": "string",
      "default": "reject",
      "enum": [
        "reject",
        "drop",
        "pad"
      ]
    },
    "reverse_frames": {
      "description": "Reverse time/frame traversal, preserving channel order inside each frame.",
      "type": "boolean",
      "default": false
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio bits sample.wav --channels 0 --bits 0 --frames 8 --stdout
```

## audio.dtmf.detect

Bounded timed DTMF candidates with nominal-band, level, dominance, twist, purity and duration/gap evidence; uncertainty retained; not a certified receiver.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `audio.pcm_samples` |
| Output | `output` | `audio.dtmf_detection` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "dominance_db": {
      "description": "Required row/column dominance over second strongest nominal frequency.",
      "type": "integer",
      "default": 6,
      "maximum": 30,
      "minimum": 3
    },
    "max_twist_db": {
      "description": "Maximum absolute row/column amplitude imbalance in dB.",
      "type": "integer",
      "default": 8,
      "maximum": 12,
      "minimum": 0
    },
    "min_gap_ms": {
      "description": "Minimum gap from the preceding candidate run.",
      "type": "integer",
      "default": 40,
      "maximum": 1000,
      "minimum": 0
    },
    "min_level_dbfs": {
      "description": "Minimum RMS dB relative to full scale; not calibrated SPL.",
      "type": "integer",
      "default": -40,
      "maximum": 0,
      "minimum": -60
    },
    "min_tone_ms": {
      "description": "Minimum candidate run duration.",
      "type": "integer",
      "default": 80,
      "maximum": 1000,
      "minimum": 40
    },
    "purity_percent": {
      "description": "Minimum fraction of measured energy explained by the pair.",
      "type": "integer",
      "default": 75,
      "maximum": 100,
      "minimum": 50
    },
    "window_ms": {
      "description": "Nonoverlapping rectangular analysis windows; timing tolerance is one window.",
      "type": "integer",
      "default": 40,
      "maximum": 100,
      "minimum": 10
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio dtmf sample.wav --channels 0 --frames 16000 --json
```

## audio.metadata.inspect

Native FLAC metadata only: STREAMINFO and bounded Vorbis comments; unknown duration explicit, audio/MD5/artwork unverified, <=1 MiB metadata/256 records/16 KiB text. MP3/Ogg unsupported.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `audio.metadata` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio metadata sample.flac --json
```

## audio.morse.detect

Detect bounded tone/silence/uncertain intervals at an explicit nominal frequency; quantized timings, no automatic frequency/speed search or transcription.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `audio.pcm_samples` |
| Output | `output` | `audio.tone_intervals` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "frequency_hz": {
      "description": "Caller-supplied expected frequency; must be below actual Nyquist.",
      "type": "integer",
      "maximum": 191999,
      "minimum": 100
    },
    "min_level_dbfs": {
      "description": "RMS full-scale threshold; not calibrated SPL.",
      "type": "integer",
      "default": -40,
      "maximum": 0,
      "minimum": -60
    },
    "purity_percent": {
      "description": "Required nominal tone energy ratio.",
      "type": "integer",
      "default": 80,
      "maximum": 100,
      "minimum": 50
    },
    "window_ms": {
      "description": "Nonoverlapping rectangular windows; timing resolution.",
      "type": "integer",
      "default": 10,
      "maximum": 50,
      "minimum": 5
    }
  },
  "additionalProperties": false,
  "required": [
    "frequency_hz"
  ]
}
```

Example:

```sh
cube audio morse detect sample.wav --channels 0 --frequency-hz 700 --json
```

## audio.morse.transcribe

Separately interpret bounded explicit intervals using caller dot-unit/tolerance, 1/3 tone and 1/3/7 gap timing; reuse existing Morse decoder, retain uncertainty.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `audio.tone_intervals` |
| Output | `output` | `audio.morse_transcription` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "sample_rate": {
      "description": "Time base of interval frame coordinates.",
      "type": "integer",
      "maximum": 384000,
      "minimum": 1
    },
    "tolerance_percent": {
      "description": "Relative duration tolerance; ambiguous matches remain uncertain.",
      "type": "integer",
      "default": 20,
      "maximum": 40,
      "minimum": 0
    },
    "unit_ms": {
      "description": "Caller-supplied dot duration; no automatic speed estimate.",
      "type": "integer",
      "maximum": 1000,
      "minimum": 40
    }
  },
  "additionalProperties": false,
  "required": [
    "sample_rate",
    "unit_ms"
  ]
}
```

Example:

```sh
cube audio morse transcribe --unit-ms 100 < intervals.json
```

## audio.pcm.extract

Extract exact channel/range PCM bytes and signed values; no resampling, <=8 MiB output/2 Mi samples, <=10000 structured values.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `audio.pcm_samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channels": {
      "description": "Ordered unique source channel indexes; omitted preserves all source channels.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 31,
        "minimum": 0
      },
      "maxItems": 32,
      "minItems": 1,
      "uniqueItems": true
    },
    "frames": {
      "description": "Selected frame count; omitted selects to end. At most 1048576 frames, 300 seconds and 64 MiB source bytes.",
      "type": "integer",
      "maximum": 1048576,
      "minimum": 0
    },
    "start_frame": {
      "description": "Absolute zero-based start frame, not byte or single-channel sample offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based start frame, not byte or single-channel sample offset.",
          "type": "integer",
          "default": 0,
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "default": 0,
            "description": "Absolute zero-based start frame, not byte or single-channel sample offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "default": 0,
      "x-cube-integer": {
        "default": 0,
        "description": "Absolute zero-based start frame, not byte or single-channel sample offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio samples sample.wav --channels 1,0 --start-frame 0 --frames 100 --output samples.pcm
```

## audio.reverse

Reverse selected PCM frames without changing sample bytes/channel grouping; generate bounded minimal WAV fmt/data with checked padding; source ancillary chunks are omitted.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `audio.pcm_samples` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio reverse sample.wav --output reversed.wav
```

## audio.spectrogram

Bounded channel/window/bin numeric spectra using shared FFT, absolute window-start axes, explicit drop/pad tails and peak full-scale amplitudes; <=65536 cells/4 Mi work.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `audio.pcm_samples` |
| Output | `output` | `audio.spectrogram` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "fft_size": {
      "description": "Power-of-two FFT window length; independent of total selected frames.",
      "type": "integer",
      "default": 1024,
      "maximum": 16384,
      "minimum": 8
    },
    "final_window": {
      "description": "drop keeps full windows; pad zero-fills tails at each hop start before end. Empty input has no windows.",
      "type": "string",
      "default": "drop",
      "enum": [
        "drop",
        "pad"
      ]
    },
    "hop_size": {
      "description": "Frame distance between window starts, 1..=fft_size.",
      "type": "integer",
      "default": 512,
      "maximum": 16384,
      "minimum": 1
    },
    "window": {
      "description": "Periodic window; coherent-gain normalized one-sided peak amplitude.",
      "type": "string",
      "default": "hann",
      "enum": [
        "rectangular",
        "hann",
        "hamming"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio spectrogram sample.wav --channels 0 --fft-size 1024 --hop-size 512 --frames 4096 --json
```

## audio.spectrum

Bounded one-sided FFT of selected integer PCM; actual sample-rate frequencies, periodic windows, peak full-scale amplitudes, <=65536 cells/4 Mi work.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `audio.pcm_samples` |
| Output | `output` | `audio.spectrum` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "fft_size": {
      "description": "Power of two, exact selected frame count; no implicit padding/truncation.",
      "type": "integer",
      "default": 1024,
      "maximum": 16384,
      "minimum": 8
    },
    "window": {
      "description": "Periodic window; coherent-gain normalized one-sided peak amplitude.",
      "type": "string",
      "default": "hann",
      "enum": [
        "rectangular",
        "hann",
        "hamming"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio spectrum sample.wav --channels 0 --fft-size 1024 --json
```

## audio.wave.inspect

Inspect bounded PCM RIFF/WAVE metadata and chunks without decoding samples; <=1 GiB, <=10000 chunks.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `audio.wave_info` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio inspect sample.wav --json
```

## audio.waveform.envelope

Stream exact integer min/max envelopes per selected channel; preserve frame positions and partial buckets; explicit bounded results.

Category: `audio`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `audio.waveform_envelope` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "bucket_frames": {
      "description": "Positive frames per bucket (1..=1048576), including a final partial bucket without padding.",
      "type": "integer",
      "default": 1024,
      "maximum": 1048576,
      "minimum": 1
    },
    "channels": {
      "description": "Ordered unique source channel indexes; omitted preserves all source channels.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 31,
        "minimum": 0
      },
      "maxItems": 32,
      "minItems": 1,
      "uniqueItems": true
    },
    "frames": {
      "description": "Selected frame count; omitted selects to end. At most 1048576 frames, 300 seconds and 64 MiB source bytes.",
      "type": "integer",
      "maximum": 1048576,
      "minimum": 0
    },
    "max_buckets": {
      "description": "Maximum complete buckets, 1..=10000; total channel/bucket pairs also <=10000. Exceeding either budget fails.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "start_frame": {
      "description": "Absolute zero-based start frame, not byte or single-channel sample offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based start frame, not byte or single-channel sample offset.",
          "type": "integer",
          "default": 0,
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "default": 0,
            "description": "Absolute zero-based start frame, not byte or single-channel sample offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "default": 0,
      "x-cube-integer": {
        "default": 0,
        "description": "Absolute zero-based start frame, not byte or single-channel sample offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube audio waveform sample.wav --channels 0 --bucket-frames 1024 --json
```

## binary.add_wrapping

Add paired big-endian unsigned units modulo their width; <=16 MiB per operand.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "Exact equally long byte operand, including empty/empty. Never repeat or pad.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 255,
        "minimum": 0
      },
      "minItems": 0,
      "writeOnly": true
    },
    "unit_bytes": {
      "description": "Independent unsigned big-endian units: 1/2/4/8 bytes; partial units fail.",
      "type": "integer",
      "default": 1,
      "enum": [
        1,
        2,
        4,
        8
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube binary add-wrapping --key-hex 01 --text A
```

## binary.and

Bounded per-byte AND of equal-length operands; no repetition/padding; <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "Exact equally long byte operand, including empty/empty. Never repeat or pad.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 255,
        "minimum": 0
      },
      "minItems": 0,
      "writeOnly": true
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube binary and --key-hex ff0f --text AB
```

## binary.not

Invert all eight bits per byte; preserve byte length, <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube binary not --text AB
```

## binary.or

Bounded per-byte OR of equal-length operands; no repetition/padding; <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "Exact equally long byte operand, including empty/empty. Never repeat or pad.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 255,
        "minimum": 0
      },
      "minItems": 0,
      "writeOnly": true
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube binary or --key-hex 0080 --text AB
```

## binary.reverse

Reverse byte positions, not bits or Unicode scalars; <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube binary reverse --text AB
```

## binary.rotate_left

Rotate left per independent big-endian unit; u64 count modulo width; <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "count": {
      "description": "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
      "anyOf": [
        {
          "description": "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "unit_bytes": {
      "description": "Independent unsigned big-endian units: 1/2/4/8 bytes; partial units fail.",
      "type": "integer",
      "default": 1,
      "enum": [
        1,
        2,
        4,
        8
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "count"
  ]
}
```

Example:

```sh
cube binary rotate-left --count 1 --text A
```

## binary.rotate_right

Rotate right per independent big-endian unit; u64 count modulo width; <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "count": {
      "description": "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
      "anyOf": [
        {
          "description": "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Unsigned u64 count, reduced modulo unit_bytes*8; pipeline integers are limited to i64::MAX.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "unit_bytes": {
      "description": "Independent unsigned big-endian units: 1/2/4/8 bytes; partial units fail.",
      "type": "integer",
      "default": 1,
      "enum": [
        1,
        2,
        4,
        8
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "count"
  ]
}
```

Example:

```sh
cube binary rotate-right --count 1 --text A
```

## binary.shift_left

Zero-filled left shifts within independent unsigned big-endian units; overflow bits discarded; <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "bits": {
      "description": "Logical shift count, 0..unit_bytes*8; equal width gives zero, larger shifts fail.",
      "type": "integer",
      "maximum": 64,
      "minimum": 0
    },
    "unit_bytes": {
      "description": "Independent unsigned big-endian units: 1/2/4/8 bytes; partial units fail.",
      "type": "integer",
      "default": 1,
      "enum": [
        1,
        2,
        4,
        8
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "bits"
  ]
}
```

Example:

```sh
cube binary shift-left --bits 1 --unit-bytes 2 --text AB
```

## binary.shift_right

Logical right shifts within independent unsigned big-endian units; no signed arithmetic; <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "bits": {
      "description": "Logical shift count, 0..unit_bytes*8; equal width gives zero, larger shifts fail.",
      "type": "integer",
      "maximum": 64,
      "minimum": 0
    },
    "unit_bytes": {
      "description": "Independent unsigned big-endian units: 1/2/4/8 bytes; partial units fail.",
      "type": "integer",
      "default": 1,
      "enum": [
        1,
        2,
        4,
        8
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "bits"
  ]
}
```

Example:

```sh
cube binary shift-right --bits 1 --unit-bytes 2 --text AB
```

## binary.sub_wrapping

Subtract paired big-endian unsigned units modulo their width; <=16 MiB per operand.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "Exact equally long byte operand, including empty/empty. Never repeat or pad.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 255,
        "minimum": 0
      },
      "minItems": 0,
      "writeOnly": true
    },
    "unit_bytes": {
      "description": "Independent unsigned big-endian units: 1/2/4/8 bytes; partial units fail.",
      "type": "integer",
      "default": 1,
      "enum": [
        1,
        2,
        4,
        8
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube binary sub-wrapping --key-hex 01 --text A
```

## binary.swap_endian

Reverse byte order per complete 1/2/4/8-byte unit; unit order unchanged; <=16 MiB.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "unit_bytes": {
      "description": "Independent unsigned big-endian units: 1/2/4/8 bytes; partial units fail.",
      "type": "integer",
      "default": 1,
      "enum": [
        1,
        2,
        4,
        8
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube binary swap-endian --unit-bytes 2 --text ABCD
```

## binary.xor

XOR exact bytes with a nonempty repeating key; not secure encryption.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "Nonempty repeating binary key.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 255,
        "minimum": 0
      },
      "minItems": 1,
      "writeOnly": true
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube binary xor --key-hex ff --text Hello
```

## binary.xor_exact

Exact equal-length byte XOR; no key repetition; <=16 MiB per operand.

Category: `binary`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "Exact equally long byte operand, including empty/empty. Never repeat or pad.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 255,
        "minimum": 0
      },
      "minItems": 0,
      "writeOnly": true
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube binary xor-exact --key-hex ff --text A
```

## blockchain.bitcoin.derive

Derive SEC1 public keys and legacy P2PKH using validated secp256k1, RIPEMD160(SHA256(pubkey bytes)) and Base58Check (0x00/0x6f). Explicit network/compression, exact raw32/hex64/WIF, <=128 transport bytes. Reject conflicting WIF intent; public output only, no network/capture adapter.

Category: `blockchain`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `private.material` |
| Output | `output` | `bitcoin.public_address` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "compressed": {
      "description": "Explicit public-key SEC1 encoding, matching WIF intent when supplied.",
      "type": "boolean"
    },
    "format": {
      "description": "Exact supplied representation; no inference or normalization.",
      "type": "string",
      "default": "raw",
      "enum": [
        "raw",
        "hex",
        "wif"
      ]
    },
    "network": {
      "description": "Explicit P2PKH network version; testnet selects the shared test-family version.",
      "type": "string",
      "enum": [
        "mainnet",
        "testnet"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "network",
    "compressed"
  ]
}
```

Example:

```sh
cube blockchain bitcoin --format hex --network mainnet --compressed --json < public-test-scalar.hex
```

## blockchain.candidates.validate

Ordered caller-supplied WIF/hex64 candidates; <=256 entries, <=128 bytes each, <=32 KiB material, <=1 Mi work units, <=1 MiB bounded result, <=64 KiB strict JSON with depth16/nodes4096. Safe fixed validation facts/errors only, no inputs/fingerprints. Optional explicit Bitcoin/Ethereum public derivation <=32 entries; source intent remains enforced. Incremental results/progress mark incomplete consumption. No generation, I/O, pipeline/capture adapter or zeroization promise.

Category: `blockchain`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `private.candidates` |
| Output | `output` | `key.validation_batch` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "checksum": {
      "description": "Only for Ethereum derivation; defaults to eip55.",
      "type": "string",
      "enum": [
        "lowercase",
        "eip55"
      ]
    },
    "compressed": {
      "description": "Required only for Bitcoin derivation; WIF intent must match.",
      "type": "boolean"
    },
    "derive": {
      "description": "Explicit public-address output; none by default.",
      "type": "string",
      "default": "none",
      "enum": [
        "none",
        "bitcoin",
        "ethereum"
      ]
    },
    "network": {
      "description": "Required only for Bitcoin derivation; WIF intent must match.",
      "type": "string",
      "enum": [
        "mainnet",
        "testnet"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube blockchain candidates --json < public-test-candidates.json
```

## blockchain.ethereum.derive

Offline validated secp256k1 raw32/hex64 scalar -> 64 uncompressed x||y bytes (no 04 prefix) -> Ethereum Keccak-256 (not SHA3) -> final20 bytes. Exact lowercase or ERC-55/EIP-55 text; <=128 transport bytes. Public output only; no network, persistence or pipeline/capture adapter.

Category: `blockchain`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `private.scalar` |
| Output | `output` | `ethereum.public_address` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "checksum": {
      "description": "Lowercase hex or ERC-55/EIP-55 ASCII-hex checksum; no chain-specific checksum.",
      "type": "string",
      "default": "eip55",
      "enum": [
        "lowercase",
        "eip55"
      ]
    },
    "format": {
      "description": "Exact declared scalar representation.",
      "type": "string",
      "default": "raw",
      "enum": [
        "raw",
        "hex"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube blockchain ethereum --format hex --checksum eip55 --json < public-test-scalar.hex
```

## blockchain.private.export

Explicit private export between exact raw32, hex64 and supported WIF. WIF output requires network/compression; conflicting source WIF intent fails. <=128-byte input/output; returned plaintext is caller-owned, Debug redacted/no Serialize. CLI requires --private-output and explicit stdout/new-file destination. No automatic I/O, logging, pipeline/capture adapter or memory-zeroization promise.

Category: `blockchain`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `private.material` |
| Output | `output` | `private.export` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "compressed": {
      "description": "Required WIF-output intent; optional source-WIF expectation, never silently overridden.",
      "type": "boolean"
    },
    "format": {
      "description": "Exact declared input representation.",
      "type": "string",
      "default": "raw",
      "enum": [
        "raw",
        "hex",
        "wif"
      ]
    },
    "network": {
      "description": "Required for WIF output; optional source-WIF expectation for raw/hex output.",
      "type": "string",
      "enum": [
        "mainnet",
        "testnet"
      ]
    },
    "output": {
      "description": "Explicit private output: raw32, lowercase hex64 or WIF; no implicit export.",
      "type": "string",
      "enum": [
        "raw",
        "hex",
        "wif"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "output"
  ]
}
```

Example:

```sh
cube blockchain export --format hex --to wif --network mainnet --compressed --private-output --stdout < public-test-scalar.hex
```

## blockchain.scalar.validate

Validate exactly 32 big-endian raw bytes or 64 strict hex digits using maintained secp256k1 scalar validation (1 <= d < n). Safe fixed structural metadata only; no private bytes/fingerprints, I/O or pipeline adapter.

Category: `blockchain`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `private.scalar` |
| Output | `output` | `key.validation` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "format": {
      "description": "Declared scalar representation; hex preserves all 32 bytes and accepts no prefixes/whitespace.",
      "type": "string",
      "default": "raw",
      "enum": [
        "raw",
        "hex"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube blockchain scalar --format hex --json < public-test-scalar.hex
```

## blockchain.wif.validate

Validate <=128 WIF bytes via Base58Check, supported 0x80/0xef version, 32-byte nonzero secp256k1 scalar < n and optional 0x01 compression marker. Return safe version/network-family/compression metadata, never private bytes, WIF or fingerprints. Shared test versions cannot identify a chain; no I/O or pipeline adapter.

Category: `blockchain`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `private.wif` |
| Output | `output` | `key.validation` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "expected_network": {
      "description": "Optional encoded network-family expectation; testnet means shared testnet/signet/regtest versions.",
      "type": "string",
      "enum": [
        "mainnet",
        "testnet"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube blockchain wif --expected-network mainnet --json < public-test-key.wif
```

## cipher.affine.decode

Invert Affine with a's modular inverse; preserve case and other Unicode, <=1 MiB.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "a": {
      "description": "Multiplier coprime to 26 (1,3,5,7,9,11,15,17,19,21,23,25).",
      "type": "integer",
      "enum": [
        1,
        3,
        5,
        7,
        9,
        11,
        15,
        17,
        19,
        21,
        23,
        25
      ]
    },
    "b": {
      "description": "Additive key in 0..25.",
      "type": "integer",
      "maximum": 25,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": [
    "a",
    "b"
  ]
}
```

Example:

```sh
cube cipher affine --decode --a 5 --b 8 --text IHHWVCSWFRCP
```

## cipher.affine.encode

Affine C=(aP+b) mod 26; preserve case and other Unicode, <=1 MiB.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "a": {
      "description": "Multiplier coprime to 26 (1,3,5,7,9,11,15,17,19,21,23,25).",
      "type": "integer",
      "enum": [
        1,
        3,
        5,
        7,
        9,
        11,
        15,
        17,
        19,
        21,
        23,
        25
      ]
    },
    "b": {
      "description": "Additive key in 0..25.",
      "type": "integer",
      "maximum": 25,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": [
    "a",
    "b"
  ]
}
```

Example:

```sh
cube cipher affine --a 5 --b 8 --text AFFINECIPHER
```

## cipher.atbash

Reverse ASCII alphabets; preserve case and other Unicode, <=1 MiB; not secure encryption.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube cipher atbash --text 'ABC xyz'
```

## cipher.bacon.decode

Five-symbol Bacon alphabet; ASCII case/word-space canonicalization, <=1 MiB input.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "notation": {
      "description": "Explicit AB or binary orientation; no detection.",
      "type": "string",
      "default": "ab",
      "enum": [
        "ab",
        "binary",
        "binary_inverted"
      ]
    },
    "variant": {
      "description": "Required alphabet; classical24 merges I/J and U/V.",
      "type": "string",
      "enum": [
        "classical24",
        "extended26"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "variant"
  ]
}
```

Example:

```sh
cube cipher bacon --variant extended26 --decode --text AAAAA
```

## cipher.bacon.encode

Five-symbol Bacon alphabet; ASCII case/word-space canonicalization, <=1 MiB input.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "notation": {
      "description": "Explicit AB or binary orientation; no detection.",
      "type": "string",
      "default": "ab",
      "enum": [
        "ab",
        "binary",
        "binary_inverted"
      ]
    },
    "variant": {
      "description": "Required alphabet; classical24 merges I/J and U/V.",
      "type": "string",
      "enum": [
        "classical24",
        "extended26"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "variant"
  ]
}
```

Example:

```sh
cube cipher bacon --variant extended26 --text A
```

## cipher.beaufort

Standard reciprocal Beaufort C=(K-P) mod 26, ASCII letters only; <=1 MiB.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "1..=65536 ASCII letters; case-insensitive; advance only on ASCII input letters.",
      "type": "string",
      "writeOnly": true
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube cipher beaufort --key B --text A
```

## cipher.book.decode

Decode caller-supplied book coordinates to exact Unicode scalars; source/coordinates <=1 MiB, <=65536 coordinates/indexed units; no source retrieval.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "index_base": {
      "description": "Indexing base for every coordinate component.",
      "type": "integer",
      "default": 1,
      "maximum": 1,
      "minimum": 0
    },
    "scheme": {
      "description": "Comma-separated pairs/triples of decimal indexes; Unicode scalars, shared word/line tokenization.",
      "type": "string",
      "default": "word_character",
      "enum": [
        "word_character",
        "line_character",
        "line_word_character"
      ]
    },
    "source": {
      "description": "Exact caller-supplied UTF-8 source, <=1 MiB; no normalization. Preserve edition/line endings and record a source hash for reproducibility.",
      "type": "string"
    }
  },
  "additionalProperties": false,
  "required": [
    "source"
  ]
}
```

Example:

```sh
cube cipher book --source source.txt --text '1:1,2:1'
```

## cipher.caesar.decode

Decode ASCII letters with a signed Caesar shift; preserve other Unicode.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "shift": {
      "description": "Signed 32-bit ASCII letter shift, normalized modulo 26.",
      "type": "integer",
      "maximum": 2147483647,
      "minimum": -2147483648
    }
  },
  "additionalProperties": false,
  "required": [
    "shift"
  ]
}
```

Example:

```sh
cube cipher caesar --shift 3 --decode --text Hello
```

## cipher.caesar.encode

Encode ASCII letters with a signed Caesar shift; preserve other Unicode.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "shift": {
      "description": "Signed 32-bit ASCII letter shift, normalized modulo 26.",
      "type": "integer",
      "maximum": 2147483647,
      "minimum": -2147483648
    }
  },
  "additionalProperties": false,
  "required": [
    "shift"
  ]
}
```

Example:

```sh
cube cipher caesar --shift 3 --text Hello
```

## cipher.grandpre.decode

Decode repeated-letter 8/9/10-square coordinates with explicit digit labels; null represents each space.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `tokens` | `cipher.coordinate_tokens` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Definition": {
      "description": "Compact table JSON is <=64 KiB. There are exactly 8, 9 or 10 rows, each with\nthat many uppercase ASCII letters. Repeated cells are allowed. Each label\nstring has exactly that many unique ASCII digits in cell order; no origin,\nword list or all-letter completeness is inferred.",
      "type": "object",
      "properties": {
        "column_labels": {
          "type": "string",
          "maxLength": 10,
          "minLength": 8,
          "pattern": "^[0-9]{8,10}$"
        },
        "row_labels": {
          "type": "string",
          "maxLength": 10,
          "minLength": 8,
          "pattern": "^[0-9]{8,10}$"
        },
        "rows": {
          "type": "array",
          "items": {
            "type": "string",
            "maxLength": 10,
            "minLength": 8,
            "pattern": "^[A-Z]{8,10}$"
          },
          "maxItems": 10,
          "minItems": 8
        }
      },
      "additionalProperties": false,
      "required": [
        "rows",
        "row_labels",
        "column_labels"
      ],
      "x-cube-max-serialized-bytes": 65536
    }
  }
}
```

Example:

```sh
cube cipher grandpre --help
```

## cipher.grandpre.encode

Encode repeated-letter 8/9/10-square coordinates with explicit digit labels; null represents each space.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `tokens` | `cipher.coordinate_tokens` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Definition": {
      "description": "Compact table JSON is <=64 KiB. There are exactly 8, 9 or 10 rows, each with\nthat many uppercase ASCII letters. Repeated cells are allowed. Each label\nstring has exactly that many unique ASCII digits in cell order; no origin,\nword list or all-letter completeness is inferred.",
      "type": "object",
      "properties": {
        "column_labels": {
          "type": "string",
          "maxLength": 10,
          "minLength": 8,
          "pattern": "^[0-9]{8,10}$"
        },
        "row_labels": {
          "type": "string",
          "maxLength": 10,
          "minLength": 8,
          "pattern": "^[0-9]{8,10}$"
        },
        "rows": {
          "type": "array",
          "items": {
            "type": "string",
            "maxLength": 10,
            "minLength": 8,
            "pattern": "^[A-Z]{8,10}$"
          },
          "maxItems": 10,
          "minItems": 8
        }
      },
      "additionalProperties": false,
      "required": [
        "rows",
        "row_labels",
        "column_labels"
      ],
      "x-cube-max-serialized-bytes": 65536
    }
  }
}
```

Example:

```sh
cube cipher grandpre --help
```

## cipher.homophonic.decode

Decode exact owned tokens to canonical text.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `tokens` | `cipher.symbol_tokens` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Definition": {
      "description": "Caller-owned table whose compact JSON is <=64 KiB, with 1..4096 total tokens.\nSource scalars are unique and already canonical. When ascii_uppercase is true,\nASCII source letters must be uppercase and input ASCII letters uppercase;\nother scalars remain exact, without Unicode normalization or case folding.",
      "type": "object",
      "properties": {
        "ascii_uppercase": {
          "type": "boolean"
        },
        "entries": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/Entry"
          },
          "maxItems": 4096,
          "minItems": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "ascii_uppercase",
        "entries"
      ],
      "x-cube-max-serialized-bytes": 65536,
      "x-cube-max-total-tokens": 4096
    },
    "Entry": {
      "description": "One unique canonical scalar with 1..256 nonempty tokens, each <=256 UTF-8\nbytes. Every token is globally unique, including within the same entry.",
      "type": "object",
      "properties": {
        "symbol": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        },
        "tokens": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/TokenText"
          },
          "maxItems": 256,
          "minItems": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "symbol",
        "tokens"
      ]
    },
    "TokenText": {
      "description": "Token string schema mirrors the exact serialized String representation and\nits existing byte/scalar limits; it does not select or execute an operation.",
      "type": "string",
      "maxLength": 256,
      "minLength": 1,
      "x-cube-max-utf8-bytes": 256
    }
  }
}
```

Example:

```sh
cube cipher homophonic --help
```

## cipher.homophonic.encode

Encode canonical scalars to explicit tokens using local per-symbol round robin.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `tokens` | `cipher.symbol_tokens` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "HomophonicEncode",
  "type": "object",
  "properties": {
    "counters": {
      "type": "array",
      "default": [],
      "items": {
        "$ref": "#/$defs/Counter"
      },
      "maxItems": 4096
    },
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Counter": {
      "description": "Optional starting selection indexes; 0..65535, reduced modulo that symbol's list.\nThe bound is exact on every JSON/JavaScript transport; it is not hidden state.",
      "type": "object",
      "properties": {
        "index": {
          "type": "integer",
          "format": "uint16",
          "maximum": 65535,
          "minimum": 0
        },
        "symbol": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "symbol",
        "index"
      ]
    },
    "Definition": {
      "description": "Caller-owned table whose compact JSON is <=64 KiB, with 1..4096 total tokens.\nSource scalars are unique and already canonical. When ascii_uppercase is true,\nASCII source letters must be uppercase and input ASCII letters uppercase;\nother scalars remain exact, without Unicode normalization or case folding.",
      "type": "object",
      "properties": {
        "ascii_uppercase": {
          "type": "boolean"
        },
        "entries": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/Entry"
          },
          "maxItems": 4096,
          "minItems": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "ascii_uppercase",
        "entries"
      ],
      "x-cube-max-serialized-bytes": 65536,
      "x-cube-max-total-tokens": 4096
    },
    "Entry": {
      "description": "One unique canonical scalar with 1..256 nonempty tokens, each <=256 UTF-8\nbytes. Every token is globally unique, including within the same entry.",
      "type": "object",
      "properties": {
        "symbol": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        },
        "tokens": {
          "type": "array",
          "items": {
            "$ref": "#/$defs/TokenText"
          },
          "maxItems": 256,
          "minItems": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "symbol",
        "tokens"
      ]
    },
    "TokenText": {
      "description": "Token string schema mirrors the exact serialized String representation and\nits existing byte/scalar limits; it does not select or execute an operation.",
      "type": "string",
      "maxLength": 256,
      "minLength": 1,
      "x-cube-max-utf8-bytes": 256
    }
  }
}
```

Example:

```sh
cube cipher homophonic --help
```

## cipher.monome_dinome.decode

Decode with an explicit monome_dinome digit table; reject unassigned/incomplete digit transport.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Alias": {
      "type": "object",
      "properties": {
        "from": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        },
        "to": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "from",
        "to"
      ]
    },
    "Definition": {
      "description": "Compact table JSON is <=64 KiB. Digits permute 0..9; alphabet cells are unique\nprintable ASCII, excluding lowercase. The operation selects the grammar:\nStraddling requires two distinct prefixes and at most 28 cells; Monome-Dinome\nrequires 24 uppercase cells, J->I and one other explicit merge covering A..Z;\nTridigital requires all 26 uppercase letters, empty prefixes and no aliases or\nescape. Aliases map unique noncell uppercase letters to canonical uppercase cells.\nA Straddling escape is a nonalphanumeric canonical cell, with no digit cells.",
      "type": "object",
      "properties": {
        "aliases": {
          "type": "array",
          "default": [],
          "items": {
            "$ref": "#/$defs/Alias"
          },
          "maxItems": 26
        },
        "alphabet": {
          "type": "string",
          "maxLength": 28,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 28
        },
        "digit_escape": {
          "type": [
            "string",
            "null"
          ],
          "default": null,
          "maxLength": 1,
          "minLength": 1
        },
        "digits": {
          "type": "string",
          "maxLength": 10,
          "minLength": 10,
          "pattern": "^[0-9]{10}$"
        },
        "prefixes": {
          "type": "string",
          "maxLength": 2,
          "pattern": "^[0-9]{0,2}$"
        }
      },
      "additionalProperties": false,
      "required": [
        "digits",
        "prefixes",
        "alphabet"
      ],
      "x-cube-max-serialized-bytes": 65536
    }
  }
}
```

Example:

```sh
cube cipher monome-dinome --help
```

## cipher.monome_dinome.encode

Encode with an explicit monome_dinome digit table; reject unassigned/incomplete digit transport.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Alias": {
      "type": "object",
      "properties": {
        "from": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        },
        "to": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "from",
        "to"
      ]
    },
    "Definition": {
      "description": "Compact table JSON is <=64 KiB. Digits permute 0..9; alphabet cells are unique\nprintable ASCII, excluding lowercase. The operation selects the grammar:\nStraddling requires two distinct prefixes and at most 28 cells; Monome-Dinome\nrequires 24 uppercase cells, J->I and one other explicit merge covering A..Z;\nTridigital requires all 26 uppercase letters, empty prefixes and no aliases or\nescape. Aliases map unique noncell uppercase letters to canonical uppercase cells.\nA Straddling escape is a nonalphanumeric canonical cell, with no digit cells.",
      "type": "object",
      "properties": {
        "aliases": {
          "type": "array",
          "default": [],
          "items": {
            "$ref": "#/$defs/Alias"
          },
          "maxItems": 26
        },
        "alphabet": {
          "type": "string",
          "maxLength": 28,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 28
        },
        "digit_escape": {
          "type": [
            "string",
            "null"
          ],
          "default": null,
          "maxLength": 1,
          "minLength": 1
        },
        "digits": {
          "type": "string",
          "maxLength": 10,
          "minLength": 10,
          "pattern": "^[0-9]{10}$"
        },
        "prefixes": {
          "type": "string",
          "maxLength": 2,
          "pattern": "^[0-9]{0,2}$"
        }
      },
      "additionalProperties": false,
      "required": [
        "digits",
        "prefixes",
        "alphabet"
      ],
      "x-cube-max-serialized-bytes": 65536
    }
  }
}
```

Example:

```sh
cube cipher monome-dinome --help
```

## cipher.polybius.decode

Shared Polybius/Tap scalar square; row,column and explicit space tokens, <=65536 tokens.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `polybius.tokens` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "aliases": {
      "description": "Custom ALIAS=CANONICAL scalar rules.",
      "type": "array",
      "items": {
        "type": "string"
      },
      "maxItems": 4096,
      "minItems": 0,
      "uniqueItems": true
    },
    "alphabet": {
      "description": "Custom exact unique non-space scalar square; requires side.",
      "type": "string"
    },
    "index_base": {
      "description": "Coordinate origin.",
      "type": "integer",
      "default": 1,
      "enum": [
        0,
        1
      ]
    },
    "preset": {
      "description": "I/J Polybius or C/K Tap, mutually exclusive with custom alphabet.",
      "type": "string",
      "default": "polybius",
      "enum": [
        "polybius",
        "tap"
      ]
    },
    "side": {
      "description": "Custom square side, 1..64.",
      "type": "integer",
      "maximum": 64,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube cipher polybius --decode --text 1,1
```

## cipher.polybius.encode

Shared Polybius/Tap scalar square; row,column and explicit space tokens, <=65536 tokens.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `polybius.tokens` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "aliases": {
      "description": "Custom ALIAS=CANONICAL scalar rules.",
      "type": "array",
      "items": {
        "type": "string"
      },
      "maxItems": 4096,
      "minItems": 0,
      "uniqueItems": true
    },
    "alphabet": {
      "description": "Custom exact unique non-space scalar square; requires side.",
      "type": "string"
    },
    "index_base": {
      "description": "Coordinate origin.",
      "type": "integer",
      "default": 1,
      "enum": [
        0,
        1
      ]
    },
    "preset": {
      "description": "I/J Polybius or C/K Tap, mutually exclusive with custom alphabet.",
      "type": "string",
      "default": "polybius",
      "enum": [
        "polybius",
        "tap"
      ]
    },
    "side": {
      "description": "Custom square side, 1..64.",
      "type": "integer",
      "maximum": 64,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube cipher polybius --text A
```

## cipher.rot13

Rotate ASCII letters by 13; preserve other Unicode.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube cipher rot13 --text Hello
```

## cipher.rot18

Self-inverse ASCII ROT13 letters and ROT5 digits; preserve other scalars, <=1 MiB.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Empty",
  "type": "object",
  "additionalProperties": false
}
```

Example:

```sh
cube cipher rot18 --help
```

## cipher.rot47

Self-inverse printable ASCII U+0021..U+007E rotation by 47; preserve other scalars, <=1 MiB.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Empty",
  "type": "object",
  "additionalProperties": false
}
```

Example:

```sh
cube cipher rot47 --help
```

## cipher.rot5

Self-inverse ASCII digit rotation by five; preserve other scalars, <=1 MiB.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Empty",
  "type": "object",
  "additionalProperties": false
}
```

Example:

```sh
cube cipher rot5 --help
```

## cipher.straddling.decode

Decode with an explicit straddling digit table; reject unassigned/incomplete digit transport.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Alias": {
      "type": "object",
      "properties": {
        "from": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        },
        "to": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "from",
        "to"
      ]
    },
    "Definition": {
      "description": "Compact table JSON is <=64 KiB. Digits permute 0..9; alphabet cells are unique\nprintable ASCII, excluding lowercase. The operation selects the grammar:\nStraddling requires two distinct prefixes and at most 28 cells; Monome-Dinome\nrequires 24 uppercase cells, J->I and one other explicit merge covering A..Z;\nTridigital requires all 26 uppercase letters, empty prefixes and no aliases or\nescape. Aliases map unique noncell uppercase letters to canonical uppercase cells.\nA Straddling escape is a nonalphanumeric canonical cell, with no digit cells.",
      "type": "object",
      "properties": {
        "aliases": {
          "type": "array",
          "default": [],
          "items": {
            "$ref": "#/$defs/Alias"
          },
          "maxItems": 26
        },
        "alphabet": {
          "type": "string",
          "maxLength": 28,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 28
        },
        "digit_escape": {
          "type": [
            "string",
            "null"
          ],
          "default": null,
          "maxLength": 1,
          "minLength": 1
        },
        "digits": {
          "type": "string",
          "maxLength": 10,
          "minLength": 10,
          "pattern": "^[0-9]{10}$"
        },
        "prefixes": {
          "type": "string",
          "maxLength": 2,
          "pattern": "^[0-9]{0,2}$"
        }
      },
      "additionalProperties": false,
      "required": [
        "digits",
        "prefixes",
        "alphabet"
      ],
      "x-cube-max-serialized-bytes": 65536
    }
  }
}
```

Example:

```sh
cube cipher straddling --help
```

## cipher.straddling.encode

Encode with an explicit straddling digit table; reject unassigned/incomplete digit transport.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Alias": {
      "type": "object",
      "properties": {
        "from": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        },
        "to": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "from",
        "to"
      ]
    },
    "Definition": {
      "description": "Compact table JSON is <=64 KiB. Digits permute 0..9; alphabet cells are unique\nprintable ASCII, excluding lowercase. The operation selects the grammar:\nStraddling requires two distinct prefixes and at most 28 cells; Monome-Dinome\nrequires 24 uppercase cells, J->I and one other explicit merge covering A..Z;\nTridigital requires all 26 uppercase letters, empty prefixes and no aliases or\nescape. Aliases map unique noncell uppercase letters to canonical uppercase cells.\nA Straddling escape is a nonalphanumeric canonical cell, with no digit cells.",
      "type": "object",
      "properties": {
        "aliases": {
          "type": "array",
          "default": [],
          "items": {
            "$ref": "#/$defs/Alias"
          },
          "maxItems": 26
        },
        "alphabet": {
          "type": "string",
          "maxLength": 28,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 28
        },
        "digit_escape": {
          "type": [
            "string",
            "null"
          ],
          "default": null,
          "maxLength": 1,
          "minLength": 1
        },
        "digits": {
          "type": "string",
          "maxLength": 10,
          "minLength": 10,
          "pattern": "^[0-9]{10}$"
        },
        "prefixes": {
          "type": "string",
          "maxLength": 2,
          "pattern": "^[0-9]{0,2}$"
        }
      },
      "additionalProperties": false,
      "required": [
        "digits",
        "prefixes",
        "alphabet"
      ],
      "x-cube-max-serialized-bytes": 65536
    }
  }
}
```

Example:

```sh
cube cipher straddling --help
```

## cipher.substitution.decode

Decode exact Unicode scalars using a validated bijective alphabet; reject by default, <=1 MiB input/4 MiB output.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Substitution",
  "description": "Choose exactly one target or keyword variant. Core checks uniqueness,\nbijection and the combined source+target/source+keyword UTF-8 byte ceiling.",
  "anyOf": [
    {
      "$ref": "#/$defs/Target"
    },
    {
      "$ref": "#/$defs/Keyword"
    }
  ],
  "x-cube-max-combined-alphabet-utf8-bytes": 65536,
  "$defs": {
    "Keyword": {
      "description": "Source has 1..4096 unique scalars; every keyword scalar belongs to it. Keep\nfirst keyword occurrences, then unused source scalars in source order. An\nempty keyword gives identity. Source plus keyword is <=64 KiB of UTF-8.",
      "type": "object",
      "properties": {
        "keyword": {
          "type": "string",
          "maxLength": 65536,
          "x-cube-max-utf8-bytes": 65536
        },
        "source": {
          "type": "string",
          "maxLength": 4096,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 65536
        },
        "unmapped": {
          "$ref": "#/$defs/Unmapped",
          "default": "reject"
        }
      },
      "additionalProperties": false,
      "required": [
        "source",
        "keyword"
      ]
    },
    "Target": {
      "description": "Source/target contain equal counts of 1..4096 unique scalars and form a\nbijection. Their combined UTF-8 length is <=64 KiB. Sets may differ; preserve\nis therefore not always invertible outside the declared alphabet.",
      "type": "object",
      "properties": {
        "source": {
          "type": "string",
          "maxLength": 4096,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 65536
        },
        "target": {
          "type": "string",
          "maxLength": 4096,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 65536
        },
        "unmapped": {
          "$ref": "#/$defs/Unmapped",
          "default": "reject"
        }
      },
      "additionalProperties": false,
      "required": [
        "source",
        "target"
      ]
    },
    "Unmapped": {
      "type": "string",
      "enum": [
        "reject",
        "preserve"
      ]
    }
  }
}
```

Example:

```sh
cube cipher substitution --help
```

## cipher.substitution.encode

Encode exact Unicode scalars using a validated bijective alphabet; reject by default, <=1 MiB input/4 MiB output.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Substitution",
  "description": "Choose exactly one target or keyword variant. Core checks uniqueness,\nbijection and the combined source+target/source+keyword UTF-8 byte ceiling.",
  "anyOf": [
    {
      "$ref": "#/$defs/Target"
    },
    {
      "$ref": "#/$defs/Keyword"
    }
  ],
  "x-cube-max-combined-alphabet-utf8-bytes": 65536,
  "$defs": {
    "Keyword": {
      "description": "Source has 1..4096 unique scalars; every keyword scalar belongs to it. Keep\nfirst keyword occurrences, then unused source scalars in source order. An\nempty keyword gives identity. Source plus keyword is <=64 KiB of UTF-8.",
      "type": "object",
      "properties": {
        "keyword": {
          "type": "string",
          "maxLength": 65536,
          "x-cube-max-utf8-bytes": 65536
        },
        "source": {
          "type": "string",
          "maxLength": 4096,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 65536
        },
        "unmapped": {
          "$ref": "#/$defs/Unmapped",
          "default": "reject"
        }
      },
      "additionalProperties": false,
      "required": [
        "source",
        "keyword"
      ]
    },
    "Target": {
      "description": "Source/target contain equal counts of 1..4096 unique scalars and form a\nbijection. Their combined UTF-8 length is <=64 KiB. Sets may differ; preserve\nis therefore not always invertible outside the declared alphabet.",
      "type": "object",
      "properties": {
        "source": {
          "type": "string",
          "maxLength": 4096,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 65536
        },
        "target": {
          "type": "string",
          "maxLength": 4096,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 65536
        },
        "unmapped": {
          "$ref": "#/$defs/Unmapped",
          "default": "reject"
        }
      },
      "additionalProperties": false,
      "required": [
        "source",
        "target"
      ]
    },
    "Unmapped": {
      "type": "string",
      "enum": [
        "reject",
        "preserve"
      ]
    }
  }
}
```

Example:

```sh
cube cipher substitution --help
```

## cipher.tridigital.decode

Decode with an explicit tridigital digit table; return per-digit alternatives without guessing.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `alternatives` | `cipher.alternatives` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Alias": {
      "type": "object",
      "properties": {
        "from": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        },
        "to": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "from",
        "to"
      ]
    },
    "Definition": {
      "description": "Compact table JSON is <=64 KiB. Digits permute 0..9; alphabet cells are unique\nprintable ASCII, excluding lowercase. The operation selects the grammar:\nStraddling requires two distinct prefixes and at most 28 cells; Monome-Dinome\nrequires 24 uppercase cells, J->I and one other explicit merge covering A..Z;\nTridigital requires all 26 uppercase letters, empty prefixes and no aliases or\nescape. Aliases map unique noncell uppercase letters to canonical uppercase cells.\nA Straddling escape is a nonalphanumeric canonical cell, with no digit cells.",
      "type": "object",
      "properties": {
        "aliases": {
          "type": "array",
          "default": [],
          "items": {
            "$ref": "#/$defs/Alias"
          },
          "maxItems": 26
        },
        "alphabet": {
          "type": "string",
          "maxLength": 28,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 28
        },
        "digit_escape": {
          "type": [
            "string",
            "null"
          ],
          "default": null,
          "maxLength": 1,
          "minLength": 1
        },
        "digits": {
          "type": "string",
          "maxLength": 10,
          "minLength": 10,
          "pattern": "^[0-9]{10}$"
        },
        "prefixes": {
          "type": "string",
          "maxLength": 2,
          "pattern": "^[0-9]{0,2}$"
        }
      },
      "additionalProperties": false,
      "required": [
        "digits",
        "prefixes",
        "alphabet"
      ],
      "x-cube-max-serialized-bytes": 65536
    }
  }
}
```

Example:

```sh
cube cipher tridigital --help
```

## cipher.tridigital.encode

Encode with an explicit tridigital digit table; reject unassigned/incomplete digit transport.

Category: `ciphers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `text` | `text` |
| Output | `text` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "Definition",
  "type": "object",
  "properties": {
    "definition": {
      "$ref": "#/$defs/Definition"
    }
  },
  "additionalProperties": false,
  "required": [
    "definition"
  ],
  "$defs": {
    "Alias": {
      "type": "object",
      "properties": {
        "from": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        },
        "to": {
          "type": "string",
          "maxLength": 1,
          "minLength": 1
        }
      },
      "additionalProperties": false,
      "required": [
        "from",
        "to"
      ]
    },
    "Definition": {
      "description": "Compact table JSON is <=64 KiB. Digits permute 0..9; alphabet cells are unique\nprintable ASCII, excluding lowercase. The operation selects the grammar:\nStraddling requires two distinct prefixes and at most 28 cells; Monome-Dinome\nrequires 24 uppercase cells, J->I and one other explicit merge covering A..Z;\nTridigital requires all 26 uppercase letters, empty prefixes and no aliases or\nescape. Aliases map unique noncell uppercase letters to canonical uppercase cells.\nA Straddling escape is a nonalphanumeric canonical cell, with no digit cells.",
      "type": "object",
      "properties": {
        "aliases": {
          "type": "array",
          "default": [],
          "items": {
            "$ref": "#/$defs/Alias"
          },
          "maxItems": 26
        },
        "alphabet": {
          "type": "string",
          "maxLength": 28,
          "minLength": 1,
          "x-cube-max-utf8-bytes": 28
        },
        "digit_escape": {
          "type": [
            "string",
            "null"
          ],
          "default": null,
          "maxLength": 1,
          "minLength": 1
        },
        "digits": {
          "type": "string",
          "maxLength": 10,
          "minLength": 10,
          "pattern": "^[0-9]{10}$"
        },
        "prefixes": {
          "type": "string",
          "maxLength": 2,
          "pattern": "^[0-9]{0,2}$"
        }
      },
      "additionalProperties": false,
      "required": [
        "digits",
        "prefixes",
        "alphabet"
      ],
      "x-cube-max-serialized-bytes": 65536
    }
  }
}
```

Example:

```sh
cube cipher tridigital --help
```

## cipher.vigenere.decode

Vigenère P=(C-K) mod 26 with the same key advancement; <=1 MiB.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "1..=65536 ASCII letters; case-insensitive; advance only on ASCII input letters.",
      "type": "string",
      "writeOnly": true
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube cipher vigenere --decode --key LEMON --text LXFOPVEFRNHR
```

## cipher.vigenere.encode

Vigenère C=(P+K) mod 26; ASCII letters only, preserve other Unicode; <=1 MiB.

Category: `ciphers`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "key": {
      "description": "1..=65536 ASCII letters; case-insensitive; advance only on ASCII input letters.",
      "type": "string",
      "writeOnly": true
    }
  },
  "additionalProperties": false,
  "required": [
    "key"
  ]
}
```

Example:

```sh
cube cipher vigenere --key LEMON --text ATTACKATDAWN
```

## encoding.base32.decode

Decode base32 with strict documented alphabet rules.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube decode base32 --text MZXW6===
```

## encoding.base32.encode

Encode base32 with strict documented alphabet rules.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube encode base32 --text foo
```

## encoding.base58.decode

Decode base58 with strict documented alphabet rules.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube decode base58 --text bQbp
```

## encoding.base58.encode

Encode base58 with strict documented alphabet rules.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube encode base58 --text foo
```

## encoding.base58check.decode

Verify bounded Bitcoin Base58Check text and return exact version/payload. <=1421 encoded/working bytes, 5..1029 accepted frame bytes, <=1024 payload bytes; no trimming, network inference or authentication claim. Structured direct result, no pipeline adapter.

Category: `encoding`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `encoding.base58check.decoded` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "NoParameters",
  "type": "object",
  "additionalProperties": false
}
```

Example:

```sh
cube decode base58check --text 1FsSia9rv4NeEwvJ2GvXrX7LyxYspbN2mo --json
```

## encoding.base58check.encode

Single-byte version plus <=1024 payload bytes, raw double-SHA256 four-byte checksum and Bitcoin Base58. Empty payload allowed; <=1421 encoded bytes, bounded quadratic conversion. Corruption check, not authentication.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "ContractParameters",
  "type": "object",
  "properties": {
    "version": {
      "type": "integer",
      "format": "uint8",
      "maximum": 255,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": [
    "version"
  ]
}
```

Example:

```sh
cube encode base58check --version-byte 0 --text abc
```

## encoding.base64.decode

Decode base64 with strict documented alphabet rules.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube decode base64 --text Zm9v
```

## encoding.base64.encode

Encode base64 with strict documented alphabet rules.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube encode base64 --text foo
```

## encoding.hex.decode

Decode hex with strict documented alphabet rules.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube decode hex --text 666f6f
```

## encoding.hex.encode

Encode hex with strict documented alphabet rules.

Category: `encoding`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube encode hex --text foo
```

## esolang.brainfuck.compare

Compare isolated 8/16/32-bit runs with shared aggregate work/output/memory budgets; incomplete comparisons are explicit.

Category: `esolang`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `brainfuck_comparison` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "eof": {
      "description": "Explicit policy when program input is exhausted.",
      "type": "string",
      "default": "error",
      "enum": [
        "error",
        "zero",
        "unchanged"
      ]
    },
    "max_instructions": {
      "description": "Maximum dispatched commands, including loop tests and failed effects.",
      "type": "integer",
      "default": 1000000,
      "maximum": 1000000,
      "minimum": 0
    },
    "max_memory_bytes": {
      "description": "Conservative total allocation and compact JSON bound, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_output_bytes": {
      "description": "Maximum output bytes; zero permits no output.",
      "type": "integer",
      "default": 1048576,
      "maximum": 1048576,
      "minimum": 0
    },
    "total_instructions": {
      "description": "Shared maximum dispatches across all widths.",
      "type": "integer",
      "default": 1000000,
      "maximum": 1000000,
      "minimum": 0
    },
    "total_memory_bytes": {
      "description": "Aggregate memory bound including retained runs and compact JSON, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "total_output_bytes": {
      "description": "Shared maximum retained output bytes across all widths.",
      "type": "integer",
      "default": 1048576,
      "maximum": 1048576,
      "minimum": 0
    },
    "widths": {
      "description": "Nonempty unique subset of 8/16/32 (other integers in this range fail); ascending execution order.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 32,
        "minimum": 8
      },
      "maxItems": 3,
      "minItems": 1,
      "uniqueItems": true
    }
  },
  "additionalProperties": false,
  "required": [
    "widths"
  ]
}
```

Example:

```sh
cube esolang brainfuck compare --code '-.' --widths 8,16,32
```

## esolang.brainfuck.execute

Strict bounded Brainfuck, <=1 MiB source/input/output, 65536 commands, depth 256, fixed 30000 cells; exact bytes only on completion.

Category: `esolang`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "cell_bits": {
      "description": "Unsigned wrapping width; input zero-extends bytes, output emits low eight bits.",
      "type": "integer",
      "default": 8,
      "enum": [
        8,
        16,
        32
      ]
    },
    "eof": {
      "description": "Explicit policy when program input is exhausted.",
      "type": "string",
      "default": "error",
      "enum": [
        "error",
        "zero",
        "unchanged"
      ]
    },
    "max_instructions": {
      "description": "Maximum dispatched commands, including loop tests and failed effects.",
      "type": "integer",
      "default": 1000000,
      "maximum": 1000000,
      "minimum": 0
    },
    "max_memory_bytes": {
      "description": "Conservative total allocation and compact JSON bound, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_output_bytes": {
      "description": "Maximum output bytes; zero permits no output.",
      "type": "integer",
      "default": 1048576,
      "maximum": 1048576,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube esolang brainfuck run --code '++++++++[>++++++++<-]>+.' --output text
```

## esolang.brainfuck.report

Bounded execution summary, exact tape range and optional post-dispatch traces; incomplete output is labeled.

Category: `esolang`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `brainfuck_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "cell_bits": {
      "description": "Unsigned wrapping width; input zero-extends bytes, output emits low eight bits.",
      "type": "integer",
      "default": 8,
      "enum": [
        8,
        16,
        32
      ]
    },
    "eof": {
      "description": "Explicit policy when program input is exhausted.",
      "type": "string",
      "default": "error",
      "enum": [
        "error",
        "zero",
        "unchanged"
      ]
    },
    "max_instructions": {
      "description": "Maximum dispatched commands, including loop tests and failed effects.",
      "type": "integer",
      "default": 1000000,
      "maximum": 1000000,
      "minimum": 0
    },
    "max_memory_bytes": {
      "description": "Conservative total allocation and compact JSON bound, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_output_bytes": {
      "description": "Maximum output bytes; zero permits no output.",
      "type": "integer",
      "default": 1048576,
      "maximum": 1048576,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative compact JSON report size cap; preflight never silently truncates evidence.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_trace_bytes": {
      "description": "Explicit in-memory trace byte budget, paired with positive trace records.",
      "type": "integer",
      "default": 0,
      "maximum": 1048576,
      "minimum": 0
    },
    "max_trace_records": {
      "description": "Zero disables traces; exhaustion marks trace_complete=false and continues execution.",
      "type": "integer",
      "default": 0,
      "maximum": 4096,
      "minimum": 0
    },
    "tape_len": {
      "description": "Exact number of cells; start+length must not exceed 30000.",
      "type": "integer",
      "default": 0,
      "maximum": 30000,
      "minimum": 0
    },
    "tape_start": {
      "description": "Exact final tape range start, indexed from zero.",
      "type": "integer",
      "default": 0,
      "maximum": 30000,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube esolang brainfuck report --code '-.' --cell-bits 16 --tape-len 1
```

## file.appended.detect

Find trailing bytes after supported PNG, baseline JPEG, GIF or classic ZIP framing; no pixel/entropy decoding.

Category: `file`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `file.png_boundary` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube file appended sample.png
```

## file.binary.compare

Compare corresponding byte offsets; missing tail bytes remain distinct from zero.

Category: `file`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `left` | `byte_reader` |
| Input | `right` | `byte_reader` |
| Output | `output` | `file.comparison` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "length": {
      "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
      "anyOf": [
        {
          "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "max_results": {
      "description": "Maximum returned records; incomplete listings are explicit.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "offset": {
      "description": "Absolute zero-based byte offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based byte offset.",
          "type": "integer",
          "default": 0,
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "default": 0,
            "description": "Absolute zero-based byte offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "default": 0,
      "x-cube-integer": {
        "default": 0,
        "description": "Absolute zero-based byte offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube file compare first.bin second.bin
```

## file.carve

Recover an explicit range or freshly validated framing candidate to a new file with exact range provenance; no decompression.

Category: `file`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `source` | `byte_reader` |
| Output | `output` | `file.carving` |
| Output | `data` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "length": {
      "description": "Exact caller-selected length; omit to validate a supported candidate at offset.",
      "anyOf": [
        {
          "description": "Exact caller-selected length; omit to validate a supported candidate at offset.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Exact caller-selected length; omit to validate a supported candidate at offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Exact caller-selected length; omit to validate a supported candidate at offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "max_bytes": {
      "description": "Maximum output bytes, checked before destination creation.",
      "type": "integer",
      "default": 16777216,
      "maximum": 1073741824,
      "minimum": 0
    },
    "offset": {
      "description": "Absolute zero-based byte offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based byte offset.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Absolute zero-based byte offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Absolute zero-based byte offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    }
  },
  "additionalProperties": false,
  "required": [
    "offset"
  ]
}
```

Example:

```sh
cube file carve sample.bin --offset 128 --output recovered.zip --json
```

## file.exif.inspect

Inspect typed JPEG/TIFF EXIF metadata under explicit resource budgets.

Category: `file`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `file.exif` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube file exif sample.jpg
```

## file.hex.inspect

Display exact bytes as bounded hex rows with absolute offsets.

Category: `file`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `file.hex_rows` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "length": {
      "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
      "anyOf": [
        {
          "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "offset": {
      "description": "Absolute zero-based byte offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based byte offset.",
          "type": "integer",
          "default": 0,
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "default": 0,
            "description": "Absolute zero-based byte offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "default": 0,
      "x-cube-integer": {
        "default": 0,
        "description": "Absolute zero-based byte offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "width": {
      "description": "Bytes per hexadecimal row.",
      "type": "integer",
      "default": 16,
      "maximum": 256,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube file hex sample.bin --length 256
```

## file.info.inspect

Inspect basic filesystem facts and bounded header evidence; no EXIF.

Category: `file`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `file_path` |
| Output | `output` | `file.info` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube inspect sample.bin
```

## file.range.extract

Copy an explicit byte range to a new destination; never overwrite.

Category: `file`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "length": {
      "description": "Exact byte count; streaming supports at most 1 GiB and byte-returning APIs 16 MiB.",
      "type": "integer",
      "maximum": 1073741824,
      "minimum": 0
    },
    "offset": {
      "description": "Absolute zero-based byte offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based byte offset.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Absolute zero-based byte offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Absolute zero-based byte offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    }
  },
  "additionalProperties": false,
  "required": [
    "offset",
    "length"
  ]
}
```

Example:

```sh
cube file extract sample.bin --offset 4 --length 8 --output bytes.bin
```

## file.signature.detect

Detect header signatures as evidence, not structural validity.

Category: `file`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `file.signatures` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube file signature sample.bin
```

## file.signature.scan

Locate bounded embedded signature candidates with absolute offsets.

Category: `file`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `file.signature_scan` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "format": {
      "description": "Exact signature format filter; omitted selects all supported rules.",
      "type": "string",
      "enum": [
        "png",
        "jpeg",
        "gif",
        "pdf",
        "zip",
        "gzip",
        "wave",
        "tiff"
      ]
    },
    "length": {
      "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
      "anyOf": [
        {
          "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "max_results": {
      "description": "Maximum returned records; incomplete listings are explicit.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "offset": {
      "description": "Absolute zero-based byte offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based byte offset.",
          "type": "integer",
          "default": 0,
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "default": 0,
            "description": "Absolute zero-based byte offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "default": 0,
      "x-cube-integer": {
        "default": 0,
        "description": "Absolute zero-based byte offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube file embedded sample.bin
```

## file.strings.extract

Extract ASCII printable runs and exact byte offsets.

Category: `file`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `file.strings` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "length": {
      "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
      "anyOf": [
        {
          "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Selected byte count; omitted means through EOF (hex interface defaults to 256 bytes).",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "max_results": {
      "description": "Maximum returned records; incomplete listings are explicit.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "min_length": {
      "description": "Minimum number of consecutive ASCII printable bytes.",
      "type": "integer",
      "default": 4,
      "maximum": 1048576,
      "minimum": 1
    },
    "offset": {
      "description": "Absolute zero-based byte offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based byte offset.",
          "type": "integer",
          "default": 0,
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "default": 0,
            "description": "Absolute zero-based byte offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "default": 0,
      "x-cube-integer": {
        "default": 0,
        "description": "Absolute zero-based byte offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube file strings sample.bin
```

## file.zip.validate

Validate bounded classic ZIP structure at an explicit candidate origin; no decompression or payload integrity claim.

Category: `file`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `file.zip_structure` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "offset": {
      "description": "Absolute zero-based byte offset.",
      "anyOf": [
        {
          "description": "Absolute zero-based byte offset.",
          "type": "integer",
          "default": 0,
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "default": 0,
            "description": "Absolute zero-based byte offset.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "default": 0,
      "x-cube-integer": {
        "default": 0,
        "description": "Absolute zero-based byte offset.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube file zip sample.bin --offset 128 --json
```

## graph.components

Weak components of explicit ordered directed/undirected multigraph; includes isolates, <=4096 vertices/32768 edges/2 MiB input.

Category: `graph`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `graph_definition` |
| Output | `output` | `graph_components` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative graph/index/traversal/report allocation ceiling, <=32 MiB.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Complete compact report ceiling, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged graph construction and vertex/edge visits; exhaustion is a resource error.",
      "type": "integer",
      "default": 4194304,
      "maximum": 4194304,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube graph components --text '{"vertices":["a"],"directed":false,"edges":[]}'
```

## graph.cycle

Cycle presence with exact ordered edge/vertex witness; loops and parallel undirected edges count.

Category: `graph`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `graph_definition` |
| Output | `output` | `graph_cycle` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative graph/index/traversal/report allocation ceiling, <=32 MiB.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Complete compact report ceiling, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged graph construction and vertex/edge visits; exhaustion is a resource error.",
      "type": "integer",
      "default": 4194304,
      "maximum": 4194304,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube graph cycle --text '{"vertices":[],"directed":true,"edges":[]}'
```

## graph.euler_trail

Hierholzer edge-once trail/circuit with exact orientation, degree/connectivity rules and requested-start validation.

Category: `graph`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `graph_definition` |
| Output | `output` | `graph_euler_trail` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative graph/index/traversal/report allocation ceiling, <=32 MiB.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Complete compact report ceiling, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged graph construction and vertex/edge visits; exhaustion is a resource error.",
      "type": "integer",
      "default": 4194304,
      "maximum": 4194304,
      "minimum": 0
    },
    "start": {
      "description": "Optional existing start; must satisfy directed/undirected degree conditions.",
      "type": "string"
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube graph euler-trail --text '{"vertices":[],"directed":false,"edges":[]}'
```

## graph.shortest_path

Unweighted BFS path with vertex/edge ID evidence, edge input order ties; null means no route.

Category: `graph`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `graph_definition` |
| Output | `output` | `graph_path` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative graph/index/traversal/report allocation ceiling, <=32 MiB.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Complete compact report ceiling, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged graph construction and vertex/edge visits; exhaustion is a resource error.",
      "type": "integer",
      "default": 4194304,
      "maximum": 4194304,
      "minimum": 0
    },
    "source": {
      "description": "Explicit existing source vertex.",
      "type": "string"
    },
    "target": {
      "description": "Explicit existing target vertex.",
      "type": "string"
    }
  },
  "additionalProperties": false,
  "required": [
    "source",
    "target"
  ]
}
```

Example:

```sh
cube graph shortest-path --source a --target a --text '{"vertices":["a"],"directed":false,"edges":[]}'
```

## hash.crc16_ibm3740

CRC-16/IBM-3740 checksum, poly 0x1021, init 0xFFFF, xor 0, unreflected; not cryptographic.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash crc16-ibm3740 --text abc
```

## hash.crc32

CRC-32/ISO-HDLC checksum, not a cryptographic hash.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash crc32 --text Hello
```

## hash.crc32c

CRC-32/ISCSI (CRC32C) checksum, poly 0x1EDC6F41, init/xor 0xFFFFFFFF, reflected; not cryptographic.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash crc32c --text abc
```

## hash.keccak256

Ethereum Keccak-256 (not standardized SHA3-256), exact bytes as 64 lowercase hex digits; <=16 MiB. No protocol-specific prefixes or normalization.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash keccak256 --text abc
```

## hash.md5

Legacy MD5 (32 lowercase hex digits); unsuitable for collision-resistant security.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash md5 --text abc
```

## hash.ripemd160

RIPEMD-160 exact-byte digest as 40 lowercase hex digits for protocol compatibility; <=16 MiB input, not password hashing/authentication.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash ripemd160 --text abc
```

## hash.sha1

Legacy SHA-1 (40 lowercase hex digits); unsuitable for collision-resistant security.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash sha1 --text abc
```

## hash.sha256

SHA-256 digest of exact bytes, not password hashing.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash sha256 --text Hello
```

## hash.sha512

SHA-512 exact-byte digest (128 lowercase hex digits), not password hashing.

Category: `hashing`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube hash sha512 --text abc
```

## image.bit_plane.extract

Extract one row-major sample bit as 0/1 values and MSB-first zero-padded bytes.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.bit_plane` |
| Output | `data` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "bit": {
      "description": "Sample bit index, zero denotes least significant bit.",
      "type": "integer",
      "maximum": 7,
      "minimum": 0
    },
    "channel": {
      "description": "One available physical sample channel; indexed colors require explicit expansion.",
      "type": "string",
      "enum": [
        "red",
        "green",
        "blue",
        "alpha",
        "gray",
        "index"
      ]
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    },
    "region": {
      "description": "Optional [x,y,width,height] rectangle; width/height must be positive and bounds checked against the image.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 4294967295,
        "minimum": 0
      },
      "maxItems": 4,
      "minItems": 4,
      "uniqueItems": false
    }
  },
  "additionalProperties": false,
  "required": [
    "channel",
    "bit"
  ]
}
```

Example:

```sh
cube image bit-plane sample.png --channel red --bit 0 --output bits.bin
```

## image.bitwise

Combine exact 8-bit image samples with XOR/AND/OR, including alpha or native indices; identical geometry/model/palette/tRNS required; <=4 Mi pixels and 16 MiB samples per image. Other source metadata is dropped.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `left` | `image.samples` |
| Input | `right` | `image.samples` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "operation": {
      "description": "Exact byte operation on equally long corresponding samples; no cycling, padding or blending.",
      "type": "string",
      "enum": [
        "xor",
        "and",
        "or"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "operation"
  ]
}
```

Example:

```sh
cube image bitwise first.png second.png --operation xor --output combined.png
```

## image.channel.extract

Extract one exact channel as bytes and optional grayscale PNG.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.pixel_data` |
| Output | `data` | `bytes` |
| Output | `image` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channel": {
      "description": "One named available channel.",
      "type": "string",
      "enum": [
        "red",
        "green",
        "blue",
        "alpha",
        "gray",
        "index"
      ]
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    },
    "region": {
      "description": "Optional [x,y,width,height] rectangle; width/height must be positive and bounds checked against the image.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 4294967295,
        "minimum": 0
      },
      "maxItems": 4,
      "minItems": 4,
      "uniqueItems": false
    }
  },
  "additionalProperties": false,
  "required": [
    "channel"
  ]
}
```

Example:

```sh
cube image channel sample.png --channel red --output red.bin
```

## image.colors.export

Export sorted exact-color masks/layers as PNG bytes and a manifest; <=64 outputs, 16 Mi pixel-work, 64 MiB conservative PNG bound; no implicit writes.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.color_exports` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "colors": {
      "description": "Optional nonempty distinct native hex tuples, all present; omitted selects all colors.",
      "type": "array",
      "items": {
        "type": "string"
      },
      "maxItems": 64,
      "minItems": 1,
      "uniqueItems": true
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    },
    "max_colors": {
      "description": "Complete histogram color cap, including with a subset.",
      "type": "integer",
      "default": 16,
      "maximum": 64,
      "minimum": 1
    },
    "max_outputs": {
      "description": "Maximum selected outputs; rejects rather than truncates.",
      "type": "integer",
      "default": 16,
      "maximum": 64,
      "minimum": 1
    },
    "representation": {
      "description": "0/255 mask or RGBA layer with transparent-black background.",
      "type": "string",
      "default": "mask",
      "enum": [
        "mask",
        "layer"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image color-layers sample.png --output-dir layers
```

## image.colors.extract

Isolate exact native colors as a coordinate-preserving 0/255 mask or RGBA layer with transparent-black background.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.color_layer` |
| Output | `image` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "colors": {
      "description": "1..=256 distinct exact native tuples, rendered as hexadecimal sample bytes. Indexed input matches indices unless explicitly expanded.",
      "type": "array",
      "items": {
        "type": "string"
      },
      "maxItems": 256,
      "minItems": 1,
      "uniqueItems": true
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    },
    "representation": {
      "description": "Mask: selected=255, other=0. Layer: selected original RGBA, other transparent black.",
      "type": "string",
      "default": "mask",
      "enum": [
        "mask",
        "layer"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "colors"
  ]
}
```

Example:

```sh
cube image colors sample.png --colors ff0000,0000ff --output mask.png
```

## image.compare

Compare exact image/channel samples, with bounded differences and an optional binary mask.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `left` | `image.samples` |
| Input | `right` | `image.samples` |
| Output | `output` | `image.comparison` |
| Output | `mask` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channels": {
      "description": "Ordered distinct channels; omitted selects native sample order, including raw palette indices.",
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "red",
          "green",
          "blue",
          "alpha",
          "gray",
          "index"
        ]
      },
      "maxItems": 4,
      "minItems": 1,
      "uniqueItems": true
    },
    "mask": {
      "description": "Create an explicit 0/255 difference-mask image.",
      "type": "boolean",
      "default": false
    },
    "max_results": {
      "description": "Maximum difference records; total counts stay exact.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "region": {
      "description": "Optional [x,y,width,height] rectangle; width/height must be positive and bounds checked against the image.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 4294967295,
        "minimum": 0
      },
      "maxItems": 4,
      "minItems": 4,
      "uniqueItems": false
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image compare first.png second.png --json
```

## image.crop

Extract a checked rectangle without resampling; unselected pixels are discarded.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "region": {
      "description": "Optional [x,y,width,height] rectangle; width/height must be positive and bounds checked against the image.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 4294967295,
        "minimum": 0
      },
      "maxItems": 4,
      "minItems": 4,
      "uniqueItems": false
    }
  },
  "additionalProperties": false,
  "required": [
    "region"
  ]
}
```

Example:

```sh
cube image crop sample.png --region 0,0,1,1 --output crop.png
```

## image.csv.export

Export CSV v1 exact 8-bit gray/gray_alpha/rgb/rgba samples without palette/tRNS; 10000 pixels, 2 MiB, 128-byte rows/16-byte fields. Shape and native channel order retained; encoded/ancillary metadata excluded.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image csv export sample.png --output pixels.csv
```

## image.csv.import

Import a complete CSV v1 grid, validating explicit shape/model/depth, zero-based coordinates and 0..255 samples. LF/CRLF and whole-field quotes; no embedded delimiters/newlines/quotes. 10000 pixels, 2 MiB, 128-byte rows/16-byte fields; constructed samples only.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image csv import --input pixels.csv --output restored.png
```

## image.decode

Decode bounded PNG8, baseline JPEG, single-frame GIF, BMP or static WebP; preserve PNG8 source samples and report decoded raster provenance.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image inspect sample.png --json
```

## image.encode

Encode exact PNG8 samples, preserving palette/tRNS and dropping other metadata.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image convert sample.png --output copy.png
```

## image.geometry.inspect

Compare declared header dimensions with an explicit checked raw-layout hypothesis, without repairing or decoding the source.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `image.geometry` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "height": {
      "description": "Positive candidate height; total pixels <= 4 Mi.",
      "type": "integer",
      "maximum": 16384,
      "minimum": 1
    },
    "layout": {
      "description": "Unsigned 8-bit physical channel order; export uses canonical order.",
      "type": "string",
      "enum": [
        "gray",
        "rgb",
        "bgr",
        "rgba",
        "bgra"
      ]
    },
    "offset": {
      "description": "Explicit prefix byte count; remaining data must contain exactly complete rows.",
      "type": "integer",
      "default": 0,
      "maximum": 33554432,
      "minimum": 0
    },
    "row_stride": {
      "description": "Full row byte count including padding on every row; omitted means packed.",
      "type": "integer",
      "maximum": 33554432,
      "minimum": 1
    },
    "width": {
      "description": "Positive candidate width, not an inferred source dimension.",
      "type": "integer",
      "maximum": 16384,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": [
    "width",
    "height",
    "layout"
  ]
}
```

Example:

```sh
cube image raw pixels.bin --width 2 --height 1 --layout rgba --compare-header
```

## image.grayscale

Lossy color reduction using (77R+150G+29B+128)>>8 without gamma conversion; preserve alpha.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image grayscale sample.png --output gray.png
```

## image.histogram

Count exact selected channel tuples without blending, approximation or quantization.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.histogram` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channels": {
      "description": "Ordered distinct channels; omitted selects native sample order, including raw palette indices.",
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "red",
          "green",
          "blue",
          "alpha",
          "gray",
          "index"
        ]
      },
      "maxItems": 4,
      "minItems": 1,
      "uniqueItems": true
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    },
    "max_colors": {
      "description": "Maximum distinct tuples; exceeding the budget fails rather than returning approximate counts.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "region": {
      "description": "Optional [x,y,width,height] rectangle; width/height must be positive and bounds checked against the image.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 4294967295,
        "minimum": 0
      },
      "maxItems": 4,
      "minItems": 4,
      "uniqueItems": false
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image histogram sample.png --json
```

## image.invert

Invert selected 8-bit samples as 255-value, preserving alpha by default.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channels": {
      "description": "Ordered selected channels; omitted means native color channels excluding alpha.",
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "red",
          "green",
          "blue",
          "alpha",
          "gray",
          "index"
        ]
      },
      "maxItems": 4,
      "minItems": 1,
      "uniqueItems": true
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image invert sample.png --output inverted.png
```

## image.lsb.extract

Extract raw bytes under explicit pixel/channel/bit ordering and incomplete-byte policy.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.bit_stream` |
| Output | `data` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "bit_order": {
      "description": "First bit occupies output bit 7 (msb) or bit 0 (lsb).",
      "type": "string",
      "default": "msb",
      "enum": [
        "msb",
        "lsb"
      ]
    },
    "bits": {
      "description": "Ordered distinct sample positions, zero=LSB; nested after channel order.",
      "type": "array",
      "default": [
        0
      ],
      "items": {
        "type": "integer",
        "maximum": 7,
        "minimum": 0
      },
      "maxItems": 8,
      "minItems": 1,
      "uniqueItems": true
    },
    "channels": {
      "description": "Ordered distinct channels; omitted selects native sample order, including raw palette indices.",
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "red",
          "green",
          "blue",
          "alpha",
          "gray",
          "index"
        ]
      },
      "maxItems": 4,
      "minItems": 1,
      "uniqueItems": true
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    },
    "incomplete": {
      "description": "Reject, drop trailing bits or zero-pad final byte; return valid/dropped-bit counts.",
      "type": "string",
      "default": "reject",
      "enum": [
        "reject",
        "drop",
        "pad"
      ]
    },
    "region": {
      "description": "Optional [x,y,width,height] rectangle; width/height must be positive and bounds checked against the image.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 4294967295,
        "minimum": 0
      },
      "maxItems": 4,
      "minItems": 4,
      "uniqueItems": false
    },
    "reverse": {
      "description": "Reverse the complete pixel coordinate sequence, retaining channel and bit order.",
      "type": "boolean",
      "default": false
    },
    "start_corner": {
      "description": "Starting corner; axis reflection for linear/diagonal orders, rotated starting edge for directed spirals.",
      "type": "string",
      "default": "top_left",
      "enum": [
        "top_left",
        "top_right",
        "bottom_right",
        "bottom_left"
      ]
    },
    "traversal": {
      "description": "Row/column, anti-diagonals, alternating zigzag or clockwise/counterclockwise inward spiral.",
      "type": "string",
      "default": "row",
      "enum": [
        "row",
        "column",
        "diagonal",
        "zigzag",
        "spiral_cw",
        "spiral_ccw"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image lsb sample.png --channels red --bits 0 --bit-order msb --stdout
```

## image.mirror

Mirror exact samples left/right or top/bottom.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "axis": {
      "description": "horizontal reverses x; vertical reverses y.",
      "type": "string",
      "enum": [
        "horizontal",
        "vertical"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "axis"
  ]
}
```

Example:

```sh
cube image mirror sample.png --axis horizontal --output mirrored.png
```

## image.pairs

Plan/execute lexicographic unordered pairs over <=8 explicit image objects; reuse XOR/AND/OR/compare, retain per-pair failures, bounded PNGs and differences; no I/O.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `images` | `image.collection` |
| Output | `output` | `image.pair_batch` |
| Output | `images` | `image.collection` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_differences": {
      "description": "Global retained comparison records; counts remain exact.",
      "type": "integer",
      "default": 1000,
      "maximum": 1000,
      "minimum": 1
    },
    "max_pairs": {
      "description": "Reject excessive exact pair count; 0 allows empty/single inputs.",
      "type": "integer",
      "default": 28,
      "maximum": 28,
      "minimum": 0
    },
    "operation": {
      "description": "Concrete single-pair allowlist; physical alpha and strict existing model rules.",
      "type": "string",
      "enum": [
        "xor",
        "and",
        "or",
        "compare"
      ]
    },
    "plan_only": {
      "description": "Validate budgets/counts without running single-pair operations.",
      "type": "boolean",
      "default": false
    }
  },
  "additionalProperties": false,
  "required": [
    "operation"
  ]
}
```

Example:

```sh
cube image pairs first.png second.png --operation xor --plan
```

## image.palette.expand

Explicitly expand indexed PNG8 samples to unpremultiplied RGBA8.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image convert sample.png --expand-palette --output rgba.png
```

## image.pixels.export

Export selected samples with explicit shape/channel metadata or bounded coordinate records.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.pixel_data` |
| Output | `data` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channels": {
      "description": "Ordered distinct channels; omitted selects native sample order, including raw palette indices.",
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "red",
          "green",
          "blue",
          "alpha",
          "gray",
          "index"
        ]
      },
      "maxItems": 4,
      "minItems": 1,
      "uniqueItems": true
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    },
    "region": {
      "description": "Optional [x,y,width,height] rectangle; width/height must be positive and bounds checked against the image.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 4294967295,
        "minimum": 0
      },
      "maxItems": 4,
      "minItems": 4,
      "uniqueItems": false
    },
    "reverse": {
      "description": "Reverse the complete pixel coordinate sequence, retaining channel and bit order.",
      "type": "boolean",
      "default": false
    },
    "start_corner": {
      "description": "Starting corner; axis reflection for linear/diagonal orders, rotated starting edge for directed spirals.",
      "type": "string",
      "default": "top_left",
      "enum": [
        "top_left",
        "top_right",
        "bottom_right",
        "bottom_left"
      ]
    },
    "traversal": {
      "description": "Row/column, anti-diagonals, alternating zigzag or clockwise/counterclockwise inward spiral.",
      "type": "string",
      "default": "row",
      "enum": [
        "row",
        "column",
        "diagonal",
        "zigzag",
        "spiral_cw",
        "spiral_ccw"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image pixels sample.png --json
```

## image.pixels.read

Read exact native pixel samples at top-left coordinates.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    },
    "x": {
      "description": "Zero-based horizontal coordinate.",
      "type": "integer",
      "maximum": 4294967295,
      "minimum": 0
    },
    "y": {
      "description": "Zero-based vertical coordinate.",
      "type": "integer",
      "maximum": 4294967295,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": [
    "x",
    "y"
  ]
}
```

Example:

```sh
cube image pixel sample.png --x 0 --y 0
```

## image.png.inspect

Inspect PNG chunks, exact offsets and CRC evidence without pixel decoding.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `image.png_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "payload_budget": {
      "description": "Total optional raw payload export budget; zero disables payload export.",
      "type": "integer",
      "default": 0,
      "maximum": 1048576,
      "minimum": 0
    },
    "strict": {
      "description": "Reject CRC errors; false reports mismatches but still rejects unsafe framing.",
      "type": "boolean",
      "default": true
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image chunks sample.png --json
```

## image.qr.decode

Decode bounded QR symbols as raw bytes, separating no detection, undecodable candidates and decoded payloads.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.qr_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_symbols": {
      "description": "Maximum detected candidates; <=262144 pixels, <=1024 per side and <=64 KiB total decoded payload.",
      "type": "integer",
      "default": 8,
      "maximum": 8,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube image qr code.png --json
```

## image.raw.interpret

Interpret exact unsigned 8-bit raw layouts, with explicit channel order, stride and offset; never pad or truncate.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "height": {
      "description": "Positive candidate height; total pixels <= 4 Mi.",
      "type": "integer",
      "maximum": 16384,
      "minimum": 1
    },
    "layout": {
      "description": "Unsigned 8-bit physical channel order; export uses canonical order.",
      "type": "string",
      "enum": [
        "gray",
        "rgb",
        "bgr",
        "rgba",
        "bgra"
      ]
    },
    "offset": {
      "description": "Explicit prefix byte count; remaining data must contain exactly complete rows.",
      "type": "integer",
      "default": 0,
      "maximum": 33554432,
      "minimum": 0
    },
    "row_stride": {
      "description": "Full row byte count including padding on every row; omitted means packed.",
      "type": "integer",
      "maximum": 33554432,
      "minimum": 1
    },
    "width": {
      "description": "Positive candidate width, not an inferred source dimension.",
      "type": "integer",
      "maximum": 16384,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": [
    "width",
    "height",
    "layout"
  ]
}
```

Example:

```sh
cube image raw pixels.bin --width 2 --height 1 --layout rgb --output reconstructed.png
```

## image.reorder

Apply a full sample channel permutation; moving alpha must be explicit.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channels": {
      "description": "Full ordered permutation of native sample channels; output model remains fixed.",
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "red",
          "green",
          "blue",
          "alpha",
          "gray",
          "index"
        ]
      },
      "maxItems": 4,
      "minItems": 1,
      "uniqueItems": true
    },
    "expand_palette": {
      "description": "Explicitly convert indexed samples to RGBA before analysis.",
      "type": "boolean",
      "default": false
    }
  },
  "additionalProperties": false,
  "required": [
    "channels"
  ]
}
```

Example:

```sh
cube image reorder sample.png --channels blue,green,red,alpha --output reordered.png
```

## image.rotate

Rotate exact samples clockwise by 0/90/180/270; no interpolation.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `image.samples` |
| Output | `output` | `image.samples` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "degrees": {
      "description": "Canonical clockwise quarter turn; width/height swap at 90/270.",
      "type": "integer",
      "enum": [
        0,
        90,
        180,
        270
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "degrees"
  ]
}
```

Example:

```sh
cube image rotate sample.png --degrees 90 --output rotated.png
```

## image.spatial.autocorrelation

Ordered signed lag overlaps on one physical channel; exact moments and population covariance, null zero-variance correlation; 1 Mi selected pixels, 256 lags.

Category: `image`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `image` | `image.samples` |
| Input | `lags` | `image.lags` |
| Output | `output` | `spatial_autocorrelation` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channel": {
      "description": "One explicit physical channel; expand palette to interpret indexed colors.",
      "type": "string",
      "enum": [
        "red",
        "green",
        "blue",
        "gray",
        "alpha",
        "index"
      ]
    },
    "max_memory_bytes": {
      "description": "Conservative aggregate scratch and retained-report allocation bound.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete compact JSON report bound.",
      "type": "integer",
      "default": 1048576,
      "maximum": 1048576,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged validation, index and execution work; exhaustion is explicit.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    },
    "region": {
      "description": "Required x,y,width,height rectangle.",
      "type": "string"
    }
  },
  "additionalProperties": false,
  "required": [
    "region",
    "channel"
  ]
}
```

Example:

```sh
cube image autocorrelation --input stripes.png --region 0,0,4,1 --channel gray --lags '[{"dx":1,"dy":0}]'
```

## investigation.profile.plan

Preflight a versioned bounded profile and expose exact ranges, operation units, effective budgets and skipped reasons without reading content.

Category: `investigation`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `investigation.profile_request` |
| Output | `output` | `investigation.profile_plan` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube investigate plan sample.bin --depth preview --json
```

## investigation.profile.run

Run an explicit local metadata/preview profile through shared core operations and record executed/skipped/failed provenance with cooperative limits.

Category: `investigation`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `investigation.session` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube investigate run sample.bin --depth preview --identify --json
```

## investigation.replay

Verify input identity and all contracts before replaying supported deterministic runs; reject redaction gaps.

Category: `investigation`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `investigation.replay_request` |
| Output | `output` | `investigation.session` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube investigate replay session.json sample.bin --json
```

## investigation.report

Export versioned JSON or escaped text with explicit redactions and per-run result inclusion; no automatic saving.

Category: `investigation`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `investigation.session` |
| Output | `output` | `investigation.report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube investigate report session.json
```

## investigation.session.record

Record bounded operation provenance and separate observations from user hypotheses, without persistence or raw input retention.

Category: `investigation`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `investigation.request` |
| Output | `output` | `investigation.session` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube investigate pipeline sample.bin --steps '[]' --identify --json
```

## investigation.triage

Recommend available inspections from bounded header evidence using explainable rules, without executing suggestions.

Category: `investigation`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `investigation.triage` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_read_bytes": {
      "description": "Maximum header bytes; no more than 12. Insufficient budgets fail before reading.",
      "type": "integer",
      "default": 12,
      "maximum": 12,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube investigate triage sample.bin --json
```

## logic.boolean.analyze

Full bounded truth scan, exact counts/witnesses and optional implication/equivalence comparison; limited row listing is explicitly qualified.

Category: `logic`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `boolean_model` |
| Output | `output` | `boolean_analysis` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative scratch, retained rows and JSON bound, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Complete compact JSON upper bound, <=8 MiB; lower values may reject conservative preflight.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_rows": {
      "description": "Independent truth-row listing cap; exact full-scan counts remain complete.",
      "type": "integer",
      "default": 32,
      "maximum": 4096,
      "minimum": 0
    },
    "max_work": {
      "description": "All formula nodes per assignment, including both trees; no short circuit.",
      "type": "integer",
      "default": 2097152,
      "maximum": 2097152,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube logic boolean analyze --text '{"variables":[],"formula":{"op":"const","value":true}}'
```

## logic.boolean.evaluate

Tagged const/var/not/and/or/xor/implies/iff; explicit variable order and complete named Boolean assignment; <=12 variables, 256 nodes, depth 32, 1 MiB JSON.

Category: `logic`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `model` | `boolean_model` |
| Input | `assignment` | `boolean.assignment` |
| Output | `output` | `boolean_evaluation` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative scratch, retained rows and JSON bound, <=8 MiB.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Complete compact JSON upper bound, <=8 MiB; lower values may reject conservative preflight.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_rows": {
      "description": "Independent truth-row listing cap; exact full-scan counts remain complete.",
      "type": "integer",
      "default": 32,
      "maximum": 4096,
      "minimum": 0
    },
    "max_work": {
      "description": "All formula nodes per assignment, including both trees; no short circuit.",
      "type": "integer",
      "default": 2097152,
      "maximum": 2097152,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube logic boolean evaluate --text '{"variables":[],"formula":{"op":"const","value":true}}' --assignment '[]'
```

## number.alphabet.convert

Exact positional integers over explicit unique scalar alphabets/bases; u128 magnitude, <=4096 digits, no normalization/case folding.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text.integer` |
| Output | `output` | `text.integer` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "from_base": {
      "description": "Explicit active source prefix length.",
      "type": "integer",
      "maximum": 256,
      "minimum": 2
    },
    "source_alphabet": {
      "description": "Ordered 2..256 exact unique non-whitespace scalars, no ASCII signs; <=1024 bytes.",
      "type": "string"
    },
    "target_alphabet": {
      "description": "Ordered target digits; same validation as source.",
      "type": "string"
    },
    "to_base": {
      "description": "Explicit active target prefix length.",
      "type": "integer",
      "maximum": 256,
      "minimum": 2
    }
  },
  "additionalProperties": false,
  "required": [
    "source_alphabet",
    "target_alphabet",
    "from_base",
    "to_base"
  ]
}
```

Example:

```sh
cube number alphabet --source-alphabet 01 --target-alphabet abcdefghij --from-base 2 --to-base 10 --text 1010
```

## number.base.convert

Convert signed ASCII integers between bases 2..36; <=4096 input digits, <=128-bit magnitude; no floats or prefixes.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text.integer` |
| Output | `output` | `text.integer` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "from_base": {
      "description": "Explicit source radix, case-insensitive ASCII digits.",
      "type": "integer",
      "maximum": 36,
      "minimum": 2
    },
    "to_base": {
      "description": "Explicit target radix, uppercase output; at most 128 digits plus sign.",
      "type": "integer",
      "maximum": 36,
      "minimum": 2
    }
  },
  "additionalProperties": false,
  "required": [
    "from_base",
    "to_base"
  ]
}
```

Example:

```sh
cube number base --from-base 16 --to-base 2 --text FF
```

## number.binomial

Exact C(n,k), n<=128, cancellation before multiplication; no factorial overflow.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `n` | `unsigned` |
| Input | `k` | `unsigned` |
| Output | `output` | `exact_count` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_report_bytes": {
      "description": "Conservative count report cap, default 4096; bound is 128 bytes.",
      "type": "integer",
      "default": 4096,
      "maximum": 4096,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged arithmetic, cancellation and validation steps.",
      "type": "integer",
      "default": 65536,
      "maximum": 65536,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number binomial --n 128 --k 64 --json
```

## number.constant_digits

Certified decimal fractional digits of pi/e/phi/sqrt2; index 0 excludes integer part. Explicit max_digits defaults 256, hard cap 4096. Bounded pure-Rust integer intervals, <=8192 series steps/32768-bit operands; ordered selections, exact strings and prefix SHA-256, <=16 KiB result. No I/O or floating approximation.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Output | `output` | `constant.digits` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "constant": {
      "description": "Exact constant name.",
      "type": "string",
      "enum": [
        "pi",
        "e",
        "phi",
        "sqrt2"
      ]
    },
    "count": {
      "description": "Range length, exclusive with indices; checked start+count must fit max_digits.",
      "type": "integer",
      "default": 16,
      "maximum": 4096,
      "minimum": 0
    },
    "indices": {
      "description": "Optional ordered fractional indexes; repeats and empty selections preserved. Every index must be below max_digits.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 4095,
        "minimum": 0
      },
      "maxItems": 1024,
      "minItems": 0,
      "uniqueItems": false
    },
    "max_digits": {
      "description": "Explicit certified prefix budget; overrides default 256 but never hard cap 4096. The complete chosen prefix is generated and hashed.",
      "type": "integer",
      "default": 256,
      "maximum": 4096,
      "minimum": 1
    },
    "start": {
      "description": "Zero-based fractional range start; exclusive with indices. A zero-count range may end at max_digits.",
      "type": "integer",
      "default": 0,
      "maximum": 4096,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": [
    "constant"
  ]
}
```

Example:

```sh
cube number digits pi --start 256 --count 16 --max-digits 512 --json
```

## number.digital_root

Decimal digital root of a signed 128-bit magnitude; <=4096 ASCII input digits, no float conversion; zero maps to zero.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `number.signed_magnitude` |
| Output | `output` | `number.decimal_digit` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "radix": {
      "description": "Only decimal 10; generalized radices are not implemented.",
      "type": "integer",
      "default": 10,
      "enum": [
        10
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number digital-root --text=-38
```

## number.extended_gcd

Exact GCD and signed i128 Bezout coefficients; deterministic Euclid; JSON decimal strings.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `a` | `number.u64` |
| Input | `b` | `number.u64` |
| Output | `output` | `number.bezout` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number extended-gcd --a 240 --b 46 --json
```

## number.gcd

Exact u64 GCD, including gcd(0,0)=0; fixed storage, <=256 Euclidean rounds.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `a` | `number.u64` |
| Input | `b` | `number.u64` |
| Output | `output` | `number.u64` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number gcd --a 48 --b 18
```

## number.mod_inverse

Canonical u64 modular inverse; noncoprime input errors; <=256 Euclidean rounds.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `a` | `number.u64` |
| Input | `modulus` | `number.u64` |
| Output | `output` | `number.u64` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number mod-inverse --a 3 --modulus 11
```

## number.mod_pow

Exact u64 modular power with u128 products, <=64 bit rounds; exponent zero gives 1 mod m.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `base` | `number.u64` |
| Input | `exponent` | `number.u64` |
| Input | `modulus` | `number.u64` |
| Output | `output` | `number.u64` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number mod-pow --base 4 --exponent 13 --modulus 497
```

## number.multinomial

Exact multinomial count, <=128 groups with total<=128, checked u128 result.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `groups` | `unsigned_list` |
| Output | `output` | `exact_count` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_report_bytes": {
      "description": "Conservative count report cap, default 4096; bound is 128 bytes.",
      "type": "integer",
      "default": 4096,
      "maximum": 4096,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged arithmetic, cancellation and validation steps.",
      "type": "integer",
      "default": 65536,
      "maximum": 65536,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number multinomial --groups 2,1,1 --json
```

## number.pascal

Exact Pascal rows 0..rows-1; <=129 rows including row 128; decimal strings, <=1 MiB report.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `rows` | `unsigned` |
| Output | `output` | `pascal_triangle` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_output_bytes": {
      "description": "Conservative compact JSON budget, <=1 MiB.",
      "type": "integer",
      "default": 1048576,
      "maximum": 1048576,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged arithmetic, cancellation and validation steps.",
      "type": "integer",
      "default": 65536,
      "maximum": 65536,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number pascal --rows 4
```

## number.permutations

Exact P(n,k), n<=128, u128 result; overflow is an error, <=64 Ki arithmetic steps.

Category: `numbers`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `n` | `unsigned` |
| Input | `k` | `unsigned` |
| Output | `output` | `exact_count` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_report_bytes": {
      "description": "Conservative count report cap, default 4096; bound is 128 bytes.",
      "type": "integer",
      "default": 4096,
      "maximum": 4096,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged arithmetic, cancellation and validation steps.",
      "type": "integer",
      "default": 65536,
      "maximum": 65536,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number permutations --n 5 --k 2 --json
```

## pipeline.execute

Execute a prevalidated bounded linear plan; explicit typed values and optional bounded captures, no capabilities.

Category: `pipelines`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `pipeline.request` |
| Output | `output` | `pipeline.execution` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube pipeline run --steps '[]' --hex 00ff --format raw
```

## pipeline.validate

Validate all bounded linear step IDs, contract versions, parameters and adjacent types without executing algorithms.

Category: `pipelines`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `pipeline.definition` |
| Output | `output` | `pipeline.plan` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube pipeline validate --input-type bytes --steps '[]'
```

## probability.dice.compare

Strict left-win, tie and loss exact masses across two independent dice sum distributions; checked u128 arithmetic and bounded pair comparisons.

Category: `probability`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `dice_pair` |
| Output | `output` | `dice_comparison` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative aggregate scratch and retained-report allocation bound.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete compact JSON report bound.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged validation, index and execution work; exhaustion is explicit.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube probability dice compare --text '{"left":{"dice":[]},"right":{"dice":[]}}'
```

## probability.dice.sum

Exact bounded independent weighted dice convolution; 8 dice, 64 faces each, 4096 support points; signed i32 faces, no simulation.

Category: `probability`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `dice_model` |
| Output | `output` | `dice_sum_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative aggregate scratch and retained-report allocation bound.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete compact JSON report bound.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged validation, index and execution work; exhaustion is explicit.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube probability dice sum --text '{"dice":[[{"value":1,"weight":1}]]}'
```

## probability.finite.analyze

Exact supplied finite masses, events, conditioning and signed expectation; 4096 outcomes, 1 MiB JSON, checked u128 intermediates.

Category: `probability`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `finite_probability_model` |
| Output | `output` | `finite_probability_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_memory_bytes": {
      "description": "Conservative aggregate scratch and retained-report allocation bound.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete compact JSON report bound.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged validation, index and execution work; exhaustion is explicit.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube probability finite analyze --text '{"outcomes":[{"id":"H","weight":1}],"event_a":["H"]}'
```

## recipe.check

Check an independent community recipe against an exact typed expected output without persisting results.

Category: `recipes`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `recipe.contract` |
| Output | `output` | `recipe.check` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube recipe check recipe.json --hex payload=00ff --expect '{"type":"bytes","base64":"AP8="}'
```

## recipe.execute

Execute a validated recipe with exact caller-supplied typed bindings and bounded local operations.

Category: `recipes`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `recipe.request` |
| Output | `output` | `pipeline.execution` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube recipe run recipe.json --hex payload=00ff --format raw
```

## recipe.validate

Validate strict versioned JSON recipe contracts without executing operations or resolving input slots.

Category: `recipes`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `recipe.document` |
| Output | `output` | `recipe.plan` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube recipe validate recipe.json
```

## symbol.a1z26.decode

Strict canonical 1..26 hyphen tokens to uppercase ASCII; preserve ASCII whitespace, reject malformed tokens; <=3 MiB.

Category: `symbols`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube decode a1z26 --text '8-9 3-21-2-5'
```

## symbol.a1z26.encode

ASCII A-Z to canonical 1..26 hyphen tokens; preserve ASCII whitespace; lowercase loses case; <=1 MiB input/3 MiB output.

Category: `symbols`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube encode a1z26 --text 'Hi Cube'
```

## symbol.braille.decode

Decode U+2800..U+28FF cells to exact mask bytes; reject other characters; <=1 MiB.

Category: `symbols`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube decode braille --text '⠁⣿' --output hex
```

## symbol.braille.encode

Encode bytes as Unicode Braille dot masks, bit n-1=dot n; no language transcription; <=1 MiB.

Category: `symbols`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
printf '\001' | cube encode braille
```

## symbol.clock.directions

Map one exact H:MM reading or two signed millidegree angles to eight receiver-facing sectors; continuous hour motion, explicit orientation and +/-1 millidegree boundary ambiguity. Optional letter-only Semaphore composition is a caller convention.

Category: `symbols`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Output | `output` | `clock.directions` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "clockwise": {
      "description": "Positive angles follow clockwise bearings; false reverses the sense.",
      "type": "boolean",
      "default": true
    },
    "decode": {
      "description": "Explicit hour=sender LH / minute=sender RH Semaphore composition; ambiguity or unsupported pairs fail.",
      "type": "boolean",
      "default": false
    },
    "hour_millidegrees": {
      "description": "Exact signed hour-hand angle; requires minute_millidegrees, conflicts with time.",
      "anyOf": [
        {
          "description": "Exact signed hour-hand angle; requires minute_millidegrees, conflicts with time.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": -9007199254740991,
          "x-cube-maximum-decimal": "9223372036854775807",
          "x-cube-minimum-decimal": "-9223372036854775808"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Exact signed hour-hand angle; requires minute_millidegrees, conflicts with time.",
            "maximum": 9223372036854775807,
            "minimum": -9223372036854775808,
            "type": "integer",
            "x-cube-maximum-decimal": "9223372036854775807",
            "x-cube-minimum-decimal": "-9223372036854775808"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Exact signed hour-hand angle; requires minute_millidegrees, conflicts with time.",
        "maximum": 9223372036854775807,
        "minimum": -9223372036854775808,
        "type": "integer",
        "x-cube-maximum-decimal": "9223372036854775807",
        "x-cube-minimum-decimal": "-9223372036854775808"
      }
    },
    "minute_millidegrees": {
      "description": "Exact signed minute-hand angle, normalized modulo 360000.",
      "anyOf": [
        {
          "description": "Exact signed minute-hand angle, normalized modulo 360000.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": -9007199254740991,
          "x-cube-maximum-decimal": "9223372036854775807",
          "x-cube-minimum-decimal": "-9223372036854775808"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Exact signed minute-hand angle, normalized modulo 360000.",
            "maximum": 9223372036854775807,
            "minimum": -9223372036854775808,
            "type": "integer",
            "x-cube-maximum-decimal": "9223372036854775807",
            "x-cube-minimum-decimal": "-9223372036854775808"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Exact signed minute-hand angle, normalized modulo 360000.",
        "maximum": 9223372036854775807,
        "minimum": -9223372036854775808,
        "type": "integer",
        "x-cube-maximum-decimal": "9223372036854775807",
        "x-cube-minimum-decimal": "-9223372036854775808"
      }
    },
    "time": {
      "description": "H:MM or HH:MM, hour 0..12 (0=12), minute 0..59; mutually exclusive with angles; no seconds.",
      "type": "string"
    },
    "zero_axis": {
      "description": "Receiver-facing zero bearing.",
      "type": "string",
      "default": "N",
      "enum": [
        "N",
        "E",
        "S",
        "W"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube number clock --time 3:00 --decode
```

## symbol.keypad.decode

Explicit telephone or multi-tap groups; decode telephone alternatives without word guessing; <=65536 groups.

Category: `symbols`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `keypad.decoded` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "mapping": {
      "description": "Optional bounded JSON Mapping with explicit ascii_fold and ordered digit/symbol sets.",
      "type": "string"
    },
    "mode": {
      "description": "Telephone alternatives or delimited multi-tap.",
      "type": "string",
      "enum": [
        "telephone",
        "multi_tap"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "mode"
  ]
}
```

Example:

```sh
cube decode keypad --mode multi_tap --text 44
```

## symbol.keypad.encode

Explicit telephone or multi-tap groups; decode telephone alternatives without word guessing; <=65536 groups.

Category: `symbols`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "mapping": {
      "description": "Optional bounded JSON Mapping with explicit ascii_fold and ordered digit/symbol sets.",
      "type": "string"
    },
    "mode": {
      "description": "Telephone alternatives or delimited multi-tap.",
      "type": "string",
      "enum": [
        "telephone",
        "multi_tap"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "mode"
  ]
}
```

Example:

```sh
cube encode keypad --mode multi_tap --text H
```

## symbol.morse.decode

Canonical dot/dash tokens with one space; slash separates words; uppercase A-Z/0-9, <=1 MiB.

Category: `symbols`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube decode morse --text '... --- ... / ..---'
```

## symbol.morse.encode

International Morse A-Z/0-9; collapse ASCII spaces, normalize decoded case; <=1 MiB input/8 MiB output.

Category: `symbols`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube encode morse --text 'SOS 2'
```

## symbol.semaphore.decode

Decode exact LH,RH tokens or /, separated by one ASCII space; <=16384 tokens, literal J, alternate D accepted; no guessing or visual recognition.

Category: `symbols`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube decode semaphore --text 'S,SW / S,W'
```

## symbol.semaphore.encode

Letter-only flag Semaphore: sender anatomical LH,RH, receiver-facing N/NE/E/SE/S/SW/W/NW; ASCII letters uppercase, each space=/, <=16384 symbols; no numeric/control state.

Category: `symbols`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube encode semaphore --text 'A B'
```

## text.anagram.search

Exact caller-dictionary unordered anagrams; 10000 entries, 64 scalars/entry, 4 MiB dictionary, 1 MiB source; incomplete prefixes are explicit.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Input | `dictionary` | `text.dictionary` |
| Output | `output` | `anagram_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "case": {
      "description": "Preserve exact scalars or lowercase ASCII.",
      "type": "string",
      "default": "preserve",
      "enum": [
        "preserve",
        "ascii_lower"
      ]
    },
    "filter": {
      "description": "Keep all scalars or ASCII letters after preparation.",
      "type": "string",
      "default": "all",
      "enum": [
        "all",
        "ascii_letters"
      ]
    },
    "max_memory_bytes": {
      "description": "Conservative aggregate scratch and retained-report allocation bound.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete compact JSON report bound.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_results": {
      "description": "Listed solutions; one-extra-solution proves incompleteness.",
      "type": "integer",
      "default": 100,
      "maximum": 1000,
      "minimum": 0
    },
    "max_words": {
      "description": "Maximum words per unordered solution.",
      "type": "integer",
      "default": 8,
      "maximum": 8,
      "minimum": 1
    },
    "max_work": {
      "description": "Charged validation, index and execution work; exhaustion is explicit.",
      "type": "integer",
      "default": 1000000,
      "maximum": 1000000,
      "minimum": 0
    },
    "normalization": {
      "description": "Explicit bounded shared Unicode normalization.",
      "type": "string",
      "default": "none",
      "enum": [
        "none",
        "NFC",
        "NFD",
        "NFKC",
        "NFKD"
      ]
    },
    "reuse": {
      "description": "Permit repeated use of one canonical word.",
      "type": "boolean",
      "default": false
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text anagram search ab --dictionary '["ab","a","b"]
```

## text.class.map

Map disjoint scalar sets with source positions and explicit preserved/filtered records; <=65536 positions/8 MiB report.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.class.report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "definition": {
      "description": "Optional strict JSON class definition with explicit ASCII folding.",
      "type": "string"
    },
    "unmapped": {
      "description": "Unmapped scalar handling.",
      "type": "string",
      "default": "error",
      "enum": [
        "error",
        "preserve",
        "filter"
      ]
    },
    "y": {
      "description": "Required for Y in preset input; conflicts with custom definitions.",
      "type": "string",
      "enum": [
        "vowel",
        "consonant",
        "unmapped"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text class --text AEIOU
```

## text.codepoints

Exact u32 scalar code points with original offsets and decimal/hex/U+ presentation; <=65536 records, no normalization.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.codepoints` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "format": {
      "description": "Presentation only; numeric fields remain exact integers.",
      "type": "string",
      "default": "decimal",
      "enum": [
        "decimal",
        "hex",
        "unicode"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text codepoints --format unicode --text 'A é'
```

## text.combinations

Incremental ordered Cartesian strings; rightmost group fastest, duplicates retained, zero groups=>one empty string, empty group=>zero. At most 16 groups/4096 fragments/1 MiB input, 10000 results, 64 KiB per result, 16 MiB raw/JSONL output and 32 MiB work; reject excessive plans, never truncate.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text.collections` |
| Output | `output` | `text.iterator` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_results": {
      "description": "Finite result cap, checked before generation.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "separator": {
      "description": "Exact UTF-8 separator, <=4096 bytes; empty by default.",
      "type": "string",
      "default": ""
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text combinations --text '[["a","b"],["1","2"]]'
```

## text.extract

Select Unicode scalar positions (not UTF-8 bytes); preserve list order/duplicates, no normalization; <=1 MiB.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "indices": {
      "description": "Zero-based list, <=65536 items; mutually exclusive with start/stride/stop. An empty list is valid.",
      "type": "array",
      "items": {
        "anyOf": [
          {
            "type": "integer",
            "maximum": 9007199254740991,
            "minimum": 0,
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          },
          {
            "type": "string",
            "maxLength": 40,
            "pattern": "^(0|-?[1-9][0-9]*)$",
            "x-cube-native-integer": {
              "maximum": 18446744073709551615,
              "minimum": 0,
              "type": "integer",
              "x-cube-maximum-decimal": "18446744073709551615",
              "x-cube-minimum-decimal": "0"
            }
          }
        ],
        "x-cube-integer": {
          "maximum": 18446744073709551615,
          "minimum": 0,
          "type": "integer",
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        }
      },
      "maxItems": 65536,
      "minItems": 0,
      "uniqueItems": false
    },
    "start": {
      "description": "Start scalar index for stride mode; default 0 only in stride mode.",
      "anyOf": [
        {
          "description": "Start scalar index for stride mode; default 0 only in stride mode.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Start scalar index for stride mode; default 0 only in stride mode.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Start scalar index for stride mode; default 0 only in stride mode.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "stop": {
      "description": "Exclusive scalar stop in stride mode; omitted means scalar length.",
      "anyOf": [
        {
          "description": "Exclusive scalar stop in stride mode; omitted means scalar length.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Exclusive scalar stop in stride mode; omitted means scalar length.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Exclusive scalar stop in stride mode; omitted means scalar length.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "stride": {
      "description": "Explicit positive stride, required unless indices are provided.",
      "anyOf": [
        {
          "description": "Explicit positive stride, required unless indices are provided.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 1,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "1"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Explicit positive stride, required unless indices are provided.",
            "maximum": 18446744073709551615,
            "minimum": 1,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "1"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Explicit positive stride, required unless indices are provided.",
        "maximum": 18446744073709551615,
        "minimum": 1,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "1"
      }
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text extract --start 1 --stride 2 --text abcdef
```

## text.grid.factors

Enumerate ascending-width factors of 1..=65536 retained Unicode scalars; sides <=16384, <=128 candidates; no inference.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.grid_factors` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "drop_whitespace": {
      "description": "Explicit Rust Unicode White_Space removal; no normalization or other filtering.",
      "type": "boolean",
      "default": false
    },
    "max_candidates": {
      "description": "Reject excessive candidate count, never truncate.",
      "type": "integer",
      "default": 128,
      "maximum": 128,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text grid --text ABCDEF --json
```

## text.grid.render

Place exact scalars using existing image traversal; quoted-cell text or explicit RGBA mapping, <=65536 cells; no fonts or writes.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.grid` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "colors": {
      "description": "Optional explicit JSON array of [single scalar, eight RGBA Hex digits]; <=256 mappings and 8192 bytes.",
      "type": "string"
    },
    "drop_whitespace": {
      "description": "Explicit Rust Unicode White_Space removal; no normalization or other filtering.",
      "type": "boolean",
      "default": false
    },
    "traversal": {
      "description": "Existing coordinate traversal, starting top-left.",
      "type": "string",
      "default": "row",
      "enum": [
        "row",
        "column",
        "diagonal",
        "zigzag",
        "spiral_cw",
        "spiral_ccw"
      ]
    },
    "width": {
      "description": "Explicit divisor of retained length; sides <=16384.",
      "type": "integer",
      "maximum": 16384,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": [
    "width"
  ]
}
```

Example:

```sh
cube text grid --text ABCDEF --width 3
```

## text.josephus.extract

Bounded removal-based scalar traversal, resume next remaining in chosen direction; Fenwick rank selection preserves original positions.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.traversal.report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "count": {
      "description": "Required for keyed skip; Josephus defaults to all, or stops at explicit count.",
      "type": "integer",
      "maximum": 65536,
      "minimum": 0
    },
    "direction": {
      "description": "Direction of traversal and resumption.",
      "type": "string",
      "default": "forward",
      "enum": [
        "forward",
        "reverse"
      ]
    },
    "index_base": {
      "description": "Input start origin; report indexes always zero-based.",
      "type": "integer",
      "default": 0,
      "enum": [
        0,
        1
      ]
    },
    "max_memory": {
      "description": "Lowerable scratch-byte ceiling, excluding bounded retained report.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_output": {
      "description": "Lowerable serialized report-byte ceiling.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Lowerable charged work ceiling including serialized evidence.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    },
    "start": {
      "description": "Initial scalar anchor, default first scalar in explicit origin.",
      "anyOf": [
        {
          "description": "Initial scalar anchor, default first scalar in explicit origin.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Initial scalar anchor, default first scalar in explicit origin.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Initial scalar anchor, default first scalar in explicit origin.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "steps": {
      "description": "Nonempty cyclic positive u64 step list, <=65536 entries.",
      "type": "array",
      "items": {
        "anyOf": [
          {
            "type": "integer",
            "maximum": 9007199254740991,
            "minimum": 1,
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "1"
          },
          {
            "type": "string",
            "maxLength": 40,
            "pattern": "^(0|-?[1-9][0-9]*)$",
            "x-cube-native-integer": {
              "maximum": 18446744073709551615,
              "minimum": 1,
              "type": "integer",
              "x-cube-maximum-decimal": "18446744073709551615",
              "x-cube-minimum-decimal": "1"
            }
          }
        ],
        "x-cube-integer": {
          "maximum": 18446744073709551615,
          "minimum": 1,
          "type": "integer",
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "1"
        }
      },
      "maxItems": 65536,
      "minItems": 1,
      "uniqueItems": false
    },
    "wrap": {
      "description": "Wrap retained ring; Josephus rejects false.",
      "type": "boolean",
      "default": true
    }
  },
  "additionalProperties": false,
  "required": [
    "steps"
  ]
}
```

Example:

```sh
cube text josephus --steps 2 --text ABCDE
```

## text.keyboard.remap

Map physical key/layer positions on identical explicit geometries; no locale guessing or normalization.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "source": {
      "description": "Required preset name or bounded JSON Layout.",
      "type": "string"
    },
    "target": {
      "description": "Required preset name or bounded JSON Layout.",
      "type": "string"
    },
    "unmapped": {
      "description": "Explicit handling for scalars outside the source layout.",
      "type": "string",
      "default": "preserve",
      "enum": [
        "preserve",
        "error"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "source",
    "target"
  ]
}
```

Example:

```sh
cube text keyboard --source qwerty --target dvorak --text qwerty
```

## text.multiset.compare

Exact sorted scalar inventories with equality, containment, residuals and deficits; <=1 MiB per raw input, shared statistics preparation/counting.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `scalar_multiset_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "case": {
      "description": "Preserve case or lowercase ASCII only.",
      "type": "string",
      "default": "preserve",
      "enum": [
        "preserve",
        "ascii_lower"
      ]
    },
    "filter": {
      "description": "All scalars or ASCII letters, after normalization/case.",
      "type": "string",
      "default": "all",
      "enum": [
        "all",
        "ascii_letters"
      ]
    },
    "max_memory_bytes": {
      "description": "Combined scratch, retained inventories and compact JSON bound, <=32 MiB.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete report cap, <=8 MiB; no listing truncation.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Raw byte, prepared scalar and conservative count/merge visits.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    },
    "normalization": {
      "description": "Reuse existing bounded Unicode normalization.",
      "type": "string",
      "default": "none",
      "enum": [
        "none",
        "NFC",
        "NFD",
        "NFKC",
        "NFKD"
      ]
    },
    "requested": {
      "description": "Requested UTF-8 text, <=1 MiB; exact counts use the same profile as source.",
      "type": "string"
    }
  },
  "additionalProperties": false,
  "required": [
    "requested"
  ]
}
```

Example:

```sh
cube text multiset compare --text listen --requested silent
```

## text.multiset.subtract

Exact complete scalar subtraction and deficits; no underflow, hidden normalization or truncated listings.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `scalar_multiset_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "case": {
      "description": "Preserve case or lowercase ASCII only.",
      "type": "string",
      "default": "preserve",
      "enum": [
        "preserve",
        "ascii_lower"
      ]
    },
    "filter": {
      "description": "All scalars or ASCII letters, after normalization/case.",
      "type": "string",
      "default": "all",
      "enum": [
        "all",
        "ascii_letters"
      ]
    },
    "max_memory_bytes": {
      "description": "Combined scratch, retained inventories and compact JSON bound, <=32 MiB.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete report cap, <=8 MiB; no listing truncation.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Raw byte, prepared scalar and conservative count/merge visits.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    },
    "normalization": {
      "description": "Reuse existing bounded Unicode normalization.",
      "type": "string",
      "default": "none",
      "enum": [
        "none",
        "NFC",
        "NFD",
        "NFKC",
        "NFKD"
      ]
    },
    "requested": {
      "description": "Requested UTF-8 text, <=1 MiB; exact counts use the same profile as source.",
      "type": "string"
    }
  },
  "additionalProperties": false,
  "required": [
    "requested"
  ]
}
```

Example:

```sh
cube text multiset subtract --text aab --requested ab
```

## text.normalize

Explicit UAX #15 normalization; Unicode data version exposed by UNICODE_VERSION; <=1 MiB input/8 MiB output.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "form": {
      "description": "Canonical compose/decompose (NFC/NFD) or compatibility forms (NFKC/NFKD); no implicit normalization.",
      "type": "string",
      "enum": [
        "NFC",
        "NFD",
        "NFKC",
        "NFKD"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "form"
  ]
}
```

Example:

```sh
cube text normalize --form NFC --text café
```

## text.reverse

Reverse Unicode scalar values, including combining marks/CRLF; not grapheme or byte reversal; <=1 MiB.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text reverse --text 'Cube 🧊'
```

## text.select

First/last Unicode scalar per word or nonempty LF/CRLF line; punctuation/line spaces retained; no normalization; <=1 MiB.

Category: `text`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "selector": {
      "description": "Words split on Unicode whitespace; line_first is an untrimmed acrostic. Unicode scalars, not grapheme clusters.",
      "type": "string",
      "enum": [
        "word_first",
        "word_last",
        "line_first",
        "line_last"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "selector"
  ]
}
```

Example:

```sh
cube text select --selector word_first --text 'Cube Uses Binary Evidence'
```

## text.skip.keyed

Bounded cyclic-step retained scalar ring with mandatory count and original positions; first selection is step-1 advances from start.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.traversal.report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "count": {
      "description": "Required for keyed skip; Josephus defaults to all, or stops at explicit count.",
      "type": "integer",
      "maximum": 65536,
      "minimum": 0
    },
    "direction": {
      "description": "Direction of traversal and resumption.",
      "type": "string",
      "default": "forward",
      "enum": [
        "forward",
        "reverse"
      ]
    },
    "index_base": {
      "description": "Input start origin; report indexes always zero-based.",
      "type": "integer",
      "default": 0,
      "enum": [
        0,
        1
      ]
    },
    "max_memory": {
      "description": "Lowerable scratch-byte ceiling, excluding bounded retained report.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_output": {
      "description": "Lowerable serialized report-byte ceiling.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Lowerable charged work ceiling including serialized evidence.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    },
    "start": {
      "description": "Initial scalar anchor, default first scalar in explicit origin.",
      "anyOf": [
        {
          "description": "Initial scalar anchor, default first scalar in explicit origin.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Initial scalar anchor, default first scalar in explicit origin.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Initial scalar anchor, default first scalar in explicit origin.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "steps": {
      "description": "Nonempty cyclic positive u64 step list, <=65536 entries.",
      "type": "array",
      "items": {
        "anyOf": [
          {
            "type": "integer",
            "maximum": 9007199254740991,
            "minimum": 1,
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "1"
          },
          {
            "type": "string",
            "maxLength": 40,
            "pattern": "^(0|-?[1-9][0-9]*)$",
            "x-cube-native-integer": {
              "maximum": 18446744073709551615,
              "minimum": 1,
              "type": "integer",
              "x-cube-maximum-decimal": "18446744073709551615",
              "x-cube-minimum-decimal": "1"
            }
          }
        ],
        "x-cube-integer": {
          "maximum": 18446744073709551615,
          "minimum": 1,
          "type": "integer",
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "1"
        }
      },
      "maxItems": 65536,
      "minItems": 1,
      "uniqueItems": false
    },
    "wrap": {
      "description": "Wrap retained ring; Josephus rejects false.",
      "type": "boolean",
      "default": true
    }
  },
  "additionalProperties": false,
  "required": [
    "steps",
    "count"
  ]
}
```

Example:

```sh
cube text keyed-skip --steps 2 --count 5 --text ABCDE
```

## text.statistics

Bounded Unicode-scalar frequencies, overlapping n-grams, IC and entropy under explicit normalization/case/filter policies; numeric observations, never cipher identification.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.statistics` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "case": {
      "description": "Preserve scalar case or lowercase ASCII A-Z only.",
      "type": "string",
      "default": "preserve",
      "enum": [
        "preserve",
        "ascii_lower"
      ]
    },
    "filter": {
      "description": "Keep all scalars or retain ASCII letters only, after normalization/case handling.",
      "type": "string",
      "default": "all",
      "enum": [
        "all",
        "ascii_letters"
      ]
    },
    "max_results": {
      "description": "Return the most frequent values, with explicit completeness; >10000 distinct values fail rather than silently losing counts.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "ngram_size": {
      "description": "Overlapping scalar window width; IC/entropy still use individual scalars.",
      "type": "integer",
      "default": 1,
      "maximum": 16,
      "minimum": 1
    },
    "normalization": {
      "description": "No normalization by default; otherwise reuse the versioned Unicode normalizer.",
      "type": "string",
      "default": "none",
      "enum": [
        "none",
        "NFC",
        "NFD",
        "NFKC",
        "NFKD"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text statistics --text ABAB --ngram-size 2 --json
```

## text.transform.masked

Apply existing ASCII ciphers to bounded scalar position sets and their complement; every scalar counts, no normalization. Explicit overrides are audited separately.

Category: `text`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "exclude": {
      "description": "Excluded Fibonacci numeric values, not sequence ordinals or arbitrary position exclusions. Zero is omitted in base 1.",
      "type": "array",
      "items": {
        "anyOf": [
          {
            "type": "integer",
            "maximum": 9007199254740991,
            "minimum": 0,
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          },
          {
            "type": "string",
            "maxLength": 40,
            "pattern": "^(0|-?[1-9][0-9]*)$",
            "x-cube-native-integer": {
              "maximum": 18446744073709551615,
              "minimum": 0,
              "type": "integer",
              "x-cube-maximum-decimal": "18446744073709551615",
              "x-cube-minimum-decimal": "0"
            }
          }
        ],
        "x-cube-integer": {
          "maximum": 18446744073709551615,
          "minimum": 0,
          "type": "integer",
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        }
      },
      "maxItems": 65536,
      "minItems": 0,
      "uniqueItems": false
    },
    "index_base": {
      "description": "Scalar position numbering, including overrides. Every scalar counts.",
      "type": "integer",
      "default": 0,
      "maximum": 1,
      "minimum": 0
    },
    "indices": {
      "description": "Explicit scalar positions, sorted/deduplicated as a set; out-of-range positions fail.",
      "type": "array",
      "items": {
        "anyOf": [
          {
            "type": "integer",
            "maximum": 9007199254740991,
            "minimum": 0,
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          },
          {
            "type": "string",
            "maxLength": 40,
            "pattern": "^(0|-?[1-9][0-9]*)$",
            "x-cube-native-integer": {
              "maximum": 18446744073709551615,
              "minimum": 0,
              "type": "integer",
              "x-cube-maximum-decimal": "18446744073709551615",
              "x-cube-minimum-decimal": "0"
            }
          }
        ],
        "x-cube-integer": {
          "maximum": 18446744073709551615,
          "minimum": 0,
          "type": "integer",
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        }
      },
      "maxItems": 65536,
      "minItems": 0,
      "uniqueItems": false
    },
    "nth": {
      "description": "Select the Nth, 2Nth, ... scalar; independent of numbering base.",
      "anyOf": [
        {
          "description": "Select the Nth, 2Nth, ... scalar; independent of numbering base.",
          "type": "integer",
          "maximum": 9007199254740991,
          "minimum": 1,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "1"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "description": "Select the Nth, 2Nth, ... scalar; independent of numbering base.",
            "maximum": 18446744073709551615,
            "minimum": 1,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "1"
          }
        }
      ],
      "x-cube-integer": {
        "description": "Select the Nth, 2Nth, ... scalar; independent of numbering base.",
        "maximum": 18446744073709551615,
        "minimum": 1,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "1"
      }
    },
    "otherwise": {
      "description": "Transformation for unselected scalars; Caesar requires otherwise_shift.",
      "type": "string",
      "default": "identity",
      "enum": [
        "atbash",
        "caesar",
        "rot13",
        "identity"
      ]
    },
    "otherwise_shift": {
      "description": "Required only for otherwise Caesar; full signed i32 range, modulo 26.",
      "type": "integer",
      "maximum": 2147483647,
      "minimum": -2147483648
    },
    "overrides": {
      "description": "Ordered explicit POSITION=SCALAR replacements after deterministic transformation; repeated positions are audited separately.",
      "type": "array",
      "items": {
        "type": "string"
      },
      "maxItems": 65536,
      "minItems": 0,
      "uniqueItems": false
    },
    "selected": {
      "description": "Transformation for selected scalars; Caesar requires selected_shift.",
      "type": "string",
      "enum": [
        "atbash",
        "caesar",
        "rot13",
        "identity"
      ]
    },
    "selected_shift": {
      "description": "Required only for selected Caesar; full signed i32 range, modulo 26.",
      "type": "integer",
      "maximum": 2147483647,
      "minimum": -2147483648
    },
    "selector": {
      "description": "Exactly one mode; indices requires indices, every_nth requires nth, fibonacci permits exclude only.",
      "type": "string",
      "enum": [
        "indices",
        "every_nth",
        "fibonacci"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "selector",
    "selected"
  ]
}
```

Example:

```sh
cube text transform masked --text ABCDEFGHIJKLMNOPQRSTUVWXYZ --selector fibonacci --exclude 13,21 --selected atbash --otherwise caesar --otherwise-shift 3 --index-base 1
```

## text.unicode.compare

Exact scalar-index comparison plus optional separately labeled normalized-view differences; original evidence retained, no fuzzy realignment.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Input | `other` | `text` |
| Output | `output` | `text.unicode.comparison` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_output": {
      "description": "Lowerable serialized report-byte bound.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_results": {
      "description": "Shared listing budget; exact counts continue after it is exhausted. Zero is counts-only.",
      "type": "integer",
      "default": 1000,
      "maximum": 65536,
      "minimum": 0
    },
    "max_work": {
      "description": "Lowerable work bound; conservative record/header charges include bounded serialization.",
      "type": "integer",
      "default": 134217728,
      "maximum": 134217728,
      "minimum": 0
    },
    "normalization": {
      "description": "Add a normalized coordinate space; never overwrite original differences.",
      "type": "string",
      "default": "none",
      "enum": [
        "none",
        "NFC",
        "NFD",
        "NFKC",
        "NFKD"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text unicode-compare --text A --other a
```

## text.unicode.inspect

Versioned scalar case/category/control/space/format observations and finite zero-width candidates; exact counts, honest bounded listings.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.unicode.inspection` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_output": {
      "description": "Lowerable serialized report-byte bound.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_results": {
      "description": "Shared listing budget; exact counts continue after it is exhausted. Zero is counts-only.",
      "type": "integer",
      "default": 1000,
      "maximum": 65536,
      "minimum": 0
    },
    "max_work": {
      "description": "Lowerable work bound; conservative record/header charges include bounded serialization.",
      "type": "integer",
      "default": 134217728,
      "maximum": 134217728,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text unicode-inspect --text 'Aaé'
```

## text.utf8.decode

Explicit UTF-8 decoding, reject invalid bytes without replacement; <=1 MiB.

Category: `text`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `bytes` |
| Output | `output` | `text` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
printf Hello | cube text from-bytes
```

## text.utf8.encode

Explicit text-to-UTF-8 bytes, no normalization/newline; <=1 MiB.

Category: `text`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text to-bytes --text Hello --output hex
```

## text.word.extract

Ordered signed positions per Unicode-whitespace word, exact original scalar/byte provenance and explicit short-word omissions; <=65536 coordinates.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.word.extraction` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "index_base": {
      "description": "Positive position origin 0 or 1; negative -1 always last.",
      "type": "integer",
      "default": 1,
      "enum": [
        0,
        1
      ]
    },
    "per_word": {
      "description": "Explicit JSON signed-i64 list for every word; native SDK accepts nested lists, no cycling.",
      "type": "string"
    },
    "positions": {
      "description": "Shared ordered signed-i64 list; CLI JSON text. Choose exactly one of positions/per_word.",
      "type": "string"
    },
    "short_word": {
      "description": "Fail on an absent position or retain an explicit null omission record.",
      "type": "string",
      "default": "error",
      "enum": [
        "error",
        "skip"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text word-extract --positions '[1,3]' --text 'Cube Uses Binary Evidence'
```

## text.word.lengths

Exact Unicode-scalar lengths of whitespace-delimited words with original boundaries; punctuation retained, <=65536 records.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.word.lengths` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text word-lengths --text 'A é'
```

## text.word.values

Ordered per-word scalar/u64 values with original positions, explicit optional tables/omissions and checked sums; <=65536 total values.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `text.word.values` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "max_sum": {
      "description": "Inclusive sum bound; non-default requires sums.",
      "anyOf": [
        {
          "description": "Inclusive sum bound; non-default requires sums.",
          "type": "integer",
          "default": "18446744073709551615",
          "maximum": 9007199254740991,
          "minimum": 0,
          "x-cube-maximum-decimal": "18446744073709551615",
          "x-cube-minimum-decimal": "0"
        },
        {
          "type": "string",
          "maxLength": 40,
          "pattern": "^(0|-?[1-9][0-9]*)$",
          "x-cube-native-integer": {
            "default": 18446744073709551615,
            "description": "Inclusive sum bound; non-default requires sums.",
            "maximum": 18446744073709551615,
            "minimum": 0,
            "type": "integer",
            "x-cube-maximum-decimal": "18446744073709551615",
            "x-cube-minimum-decimal": "0"
          }
        }
      ],
      "default": "18446744073709551615",
      "x-cube-integer": {
        "default": 18446744073709551615,
        "description": "Inclusive sum bound; non-default requires sums.",
        "maximum": 18446744073709551615,
        "minimum": 0,
        "type": "integer",
        "x-cube-maximum-decimal": "18446744073709551615",
        "x-cube-minimum-decimal": "0"
      }
    },
    "sums": {
      "description": "Retain per-character values and add an exact checked u64 word sum.",
      "type": "boolean",
      "default": false
    },
    "table": {
      "description": "Optional JSON [scalar,u64] pairs; <=4096 unique scalars/64 KiB, exact matching.",
      "type": "string"
    },
    "unmapped": {
      "description": "Explicit custom-table policy; skip retains null values and counts.",
      "type": "string",
      "default": "error",
      "enum": [
        "error",
        "skip"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text word-values --sums --text 'A é'
```

## text.word_ladder.shortest

Indexed substitution adjacency and shared graph BFS; lexicographic ties, 32768 edges; resource exhaustion never means no_path.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `dictionary` | `text.dictionary` |
| Input | `start` | `text` |
| Input | `end` | `text` |
| Output | `output` | `word_ladder_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "case": {
      "description": "Preserve exact scalars or lowercase ASCII.",
      "type": "string",
      "default": "preserve",
      "enum": [
        "preserve",
        "ascii_lower"
      ]
    },
    "max_memory_bytes": {
      "description": "Conservative aggregate scratch and retained-report allocation bound.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete compact JSON report bound.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged validation, index and execution work; exhaustion is explicit.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    },
    "normalization": {
      "description": "Explicit bounded shared Unicode normalization.",
      "type": "string",
      "default": "none",
      "enum": [
        "none",
        "NFC",
        "NFD",
        "NFKC",
        "NFKD"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text word-ladder shortest --dictionary '["cat","cot"]' --start cat --end cot
```

## text.word_ladder.validate

Validate exact single-scalar substitutions and caller dictionary membership; 4096 distinct words, 32 scalars/word, 1 MiB dictionary, 4096 path words.

Category: `text`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `dictionary` | `text.dictionary` |
| Input | `path` | `text.list` |
| Output | `output` | `word_ladder_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "case": {
      "description": "Preserve exact scalars or lowercase ASCII.",
      "type": "string",
      "default": "preserve",
      "enum": [
        "preserve",
        "ascii_lower"
      ]
    },
    "max_memory_bytes": {
      "description": "Conservative aggregate scratch and retained-report allocation bound.",
      "type": "integer",
      "default": 33554432,
      "maximum": 33554432,
      "minimum": 0
    },
    "max_report_bytes": {
      "description": "Conservative complete compact JSON report bound.",
      "type": "integer",
      "default": 8388608,
      "maximum": 8388608,
      "minimum": 0
    },
    "max_work": {
      "description": "Charged validation, index and execution work; exhaustion is explicit.",
      "type": "integer",
      "default": 16777216,
      "maximum": 16777216,
      "minimum": 0
    },
    "normalization": {
      "description": "Explicit bounded shared Unicode normalization.",
      "type": "string",
      "default": "none",
      "enum": [
        "none",
        "NFC",
        "NFD",
        "NFKC",
        "NFKD"
      ]
    }
  },
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube text word-ladder validate --dictionary '["cat","cot"]' --path '["cat","cot"]'
```

## video.audio.extract

Remux one explicit self-contained MOV raw-PCM8 mono/stereo track to bounded WAV bytes with exact source/time/hash provenance; no decoding, resampling or sample-byte changes; fixed stsz and delta-1 sample timing required.

Category: `video`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `video.audio_track` |
| Output | `data` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "track_id": {
      "description": "Explicit nonzero raw PCM8 audio track ID.",
      "type": "integer",
      "maximum": 4294967295,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": [
    "track_id"
  ]
}
```

Example:

```sh
cube video audio clip.mov --track-id 2 --output track.wav
```

## video.container.inspect

Bounded declared MP4/MOV or WebM container/track/codec/dimension/time metadata; no decoding or packet validation; fragmentation/encryption and unsupported profiles explicit.

Category: `video`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `video.container_metadata` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube video inspect sample.mp4 --json
```

## video.frames.compare

Compare <=8 explicit MOV/JPEG frame pairs through existing Image comparison, retaining exact extraction provenance; <=8 selected frames, 8 Mi pixel pairs, 10000 difference records; no resizing, perception or lossy-source equivalence claims.

Category: `video`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `video.frame_comparisons` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "channels": {
      "description": "Optional exact channel selection; omitted means existing native image-model semantics.",
      "type": "array",
      "items": {
        "type": "string",
        "enum": [
          "red",
          "green",
          "blue",
          "alpha",
          "gray",
          "index"
        ]
      },
      "maxItems": 4,
      "minItems": 1,
      "uniqueItems": true
    },
    "max_results": {
      "description": "Global retained difference-record budget; exact pixel counts remain available.",
      "type": "integer",
      "default": 1000,
      "maximum": 10000,
      "minimum": 1
    },
    "pairs": {
      "description": "Flattened left/right frame indexes, 1..=8 pairs (even length); CLI uses left:right and SDK uses tuples.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 9999,
        "minimum": 0
      },
      "maxItems": 16,
      "minItems": 2,
      "uniqueItems": false
    },
    "track_id": {
      "description": "Explicit nonzero JPEG video track ID.",
      "type": "integer",
      "maximum": 4294967295,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": [
    "track_id",
    "pairs"
  ]
}
```

Example:

```sh
cube video compare clip.mov --track-id 1 --pairs 0:1,1:2 --max-results 100
```

## video.frames.extract

Extract 1..=8 explicitly indexed independent baseline JPEG frames from bounded self-contained MOV/ISO-BMFF tables; exact range/time/hash provenance; decoded lossy pixels; no edits/composition/encryption/external refs or timestamp seeking.

Category: `video`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `byte_reader` |
| Output | `output` | `video.frames` |
| Output | `images` | `image.collection` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "indexes": {
      "description": "Ordered unique zero-based frame indexes; 1..=8 selections, each <10000.",
      "type": "array",
      "items": {
        "type": "integer",
        "maximum": 9999,
        "minimum": 0
      },
      "maxItems": 8,
      "minItems": 1,
      "uniqueItems": true
    },
    "track_id": {
      "description": "Explicit nonzero video track ID.",
      "type": "integer",
      "maximum": 4294967295,
      "minimum": 1
    }
  },
  "additionalProperties": false,
  "required": [
    "track_id",
    "indexes"
  ]
}
```

Example:

```sh
cube video frames clip.mov --track-id 1 --indexes 0,2 --output-dir frames
```

## web.har.asset

Explicit embedded HAR response body to bytes with logical-text/base64 provenance; no body download/decompression/path access; <=64 KiB.

Category: `web`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "index": {
      "description": "Explicit zero-based HAR entry with embedded response.content.text.",
      "type": "integer",
      "maximum": 127,
      "minimum": 0
    }
  },
  "additionalProperties": false,
  "required": [
    "index"
  ]
}
```

Example:

```sh
cube web asset har saved.har --index 0 --stdout
```

## web.har.inspect

Bounded inert HAR 1.2 metadata observations, explicit missing values/body availability; no requests, headers/cookies/body export or browser access; strict bounded JSON.

Category: `web`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `web.har_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube web har saved.har --json
```

## web.html.asset

Explicit inline source or strict base64 data-URL asset to bounded bytes with source/range/encoding/hash provenance; no URL/path I/O or execution; <=64 KiB.

Category: `web`. Contract: 1. Pipeline adapter: 1.

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `bytes` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "index": {
      "description": "Explicit zero-based embedded record/reference selection; no discovery guess or reference resolution.",
      "type": "integer",
      "maximum": 255,
      "minimum": 0
    },
    "selector": {
      "description": "inline indexes embedded descriptors; data_url indexes references.",
      "type": "string",
      "default": "inline",
      "enum": [
        "inline",
        "data_url"
      ]
    }
  },
  "additionalProperties": false,
  "required": [
    "index"
  ]
}
```

Example:

```sh
cube web asset html saved.html --index 0 --selector inline --stdout
```

## web.html.inspect

Bounded inert UTF-8 HTML token/reference/text/inline-range observations; no DOM/rendering/JS execution/network/path resolution; parse errors and limits explicit.

Category: `web`. Contract: 1. Pipeline adapter: none (direct API).

| Direction | Port | Semantic type |
| --- | --- | --- |
| Input | `input` | `text` |
| Output | `output` | `web.html_report` |

Parameters:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {},
  "additionalProperties": false,
  "required": []
}
```

Example:

```sh
cube web html saved.html --json
```
