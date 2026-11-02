# Disks and strips

Encipher or decipher a prepared message with explicit wheel order and row
selection. M-94 supplies its standard numbered wheels; Jefferson and M-138-A
require the wheel or strip alphabets for the particular configuration.
These historical devices provide no modern confidentiality or authentication.

## Choose a device

| Device | Operation pair | Required profile |
| --- | --- | --- |
| M-94 / CSP-488 | `cipher.m94.encode`, `cipher.m94.decode` | `m94_v1` |
| Jefferson wheel | `cipher.jefferson.encode`, `cipher.jefferson.decode` | `jefferson_configured_v1` |
| M-138-A / CSP-845 | `cipher.m138a.encode`, `cipher.m138a.decode` | `m138a_configured_v1` |

All operations accept named `text` and `settings` inputs, no parameters, and
return `text`. Native Python accepts the same settings as its second argument;
the CLI reads them from `--parameters`. Text must already contain only
ASCII A–Z. Empty text is valid with an empty `row_offsets` array. Case folding,
space removal, number spelling, padding and message indicators are separate
preparation steps; none are inferred or inserted.

## Select a row

For each block, align the message letters across the ordered wheels or tracks.
`row_offsets` supplies one displacement from 1 through 25 for every block,
including a final partial block. An offset of 1 selects the next letter in each
supplied alphabet; 25 selects the preceding letter. Wheel slots restart at the
first slot for each block.

Decoding requires the same positive displacements used by the sender and reverses
them. A historical recipient might inspect all rows for meaningful text; these
operations require an explicit row and never guess a language or plaintext.
Rows are supplied by the caller, so repeated calls with identical settings
produce identical output.

## Configure M-94

`order` must contain each standard wheel ID from 1 through 25 exactly once,
left to right. A block contains up to 25 letters. The fixed alphabet facts follow
[George Mace's measured wheel matrix](https://www.jproc.ca/crypto/m94.html).

Save the following as `settings.json`:

```json
{"profile":"m94_v1","order":[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25],"row_offsets":[1]}
```

Run `cube cipher m94 encode --parameters settings.json --text AAAAA`.
The JSON result contains `{"text":"BCDEF"}`. Use `decode` with `BCDEF` and the
same settings to recover `AAAAA`.

The primary [FM 24-5, section V, paragraphs 56–61, 19 October 1942](https://www.cryptomuseum.com/crypto/usa/m94/files/FM24-5_1942_pp70-81.pdf)
defines the device and includes a complete worked message. This transform accepts
the resulting numerical wheel order directly; it does not derive a key phrase,
randomize row choices or add the manual's external padding.

## Configure Jefferson wheels

Supply `wheels` as 1–64 numbered alphabets, each containing every ASCII letter
exactly once. `order` must select every supplied wheel once, using IDs starting
at 1. A block uses as many letters as there are wheels.

Jefferson's [original manuscript, before 22 March 1802](https://founders.archives.gov/documents/Jefferson/01-37-02-0082)
specifies mixed alphabets and rearrangeable numbered wheels, rather than a single
canonical alphabet set. Supply the actual alphabets of the intended device;
M-94 wheels are not silently substituted for them.

## Configure M-138-A strips

Supply the complete numbered series as `strips`: exactly 100 alphabets, each
containing every ASCII letter once. List one 26-letter cycle per strip; the
physical second printed copy is implicit. Give the series an explicit
`strip_series` identifier of 1–128 ASCII letters, digits or `._:/-`, beginning
with a letter or digit. The identifier records which supplied series is intended;
it does not download or authenticate a historical key sheet.

`order` selects 30 distinct strip IDs from 1 through 100, left to right. A block
contains up to 30 letters. This profile implements the raw strip displacement
mechanism, with each row chosen explicitly. It does not apply changing wartime
indicator, null or code-word procedures. The [NSA's CSP-845 artifact description](https://www.nsa.gov/History/National-Cryptologic-Museum/Exhibits-Artifacts/Exhibit-View/Article/2719266/csp-845/)
distinguishes the 30-track device and its changing operating procedures.

## Limits and failures

| Resource | Maximum |
| --- | --- |
| Prepared input and output text | 1 MiB each |
| Settings JSON | 512 KiB |
| Jefferson wheels and row offsets | 64 wheels; 65,536 offsets |
| M-94 row offsets | 41,944 |
| M-138-A series, selection and row offsets | 100 strips; 30 selected; 34,953 offsets |

The settings and text ceilings apply separately. Smaller wheel sets can reach
the row-count or settings-byte ceiling before the text-byte ceiling. No trace,
automatic capture or linear recipe adapter is supplied. Direct Play execution
keeps text and settings private and requires consent to retain the result.
Use a structured settings Asset in Play; large key tables are input data and
do not belong in a node's parameter fields. Play applies its own smaller
invocation and storage budgets in addition to these Core ceilings.

Malformed permutations, unknown fields/profiles, repeated IDs and missing or
extra row displacements fail with `invalid_parameter`. Non-A–Z input fails with
`invalid_input` and its UTF-8 byte offset. Excess text or settings returns
`resource_limit`; processing never returns a partial result. Errors identify
fields without echoing text or key values.
Named-port SDK calls can reject malformed `settings` against their input schema
with `invalid_input` before the transform runs.
