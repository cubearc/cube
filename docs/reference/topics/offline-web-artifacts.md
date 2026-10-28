# Offline web artifacts

Inspect saved HTML and HAR data or extract an explicitly selected embedded asset.
All content remains inert: no script executes, URL is followed, local reference
is opened or asset is downloaded.

## HTML inspection

`web.html.inspect` accepts UTF-8 HTML and reports:

| Result | Meaning |
| --- | --- |
| References | Selected `src`, `href`, `poster`, `data` and `action` values after HTML character decoding |
| Event handlers | Counts only; no execution |
| Text | Tokenized text excluding script and style |
| Inline script/style | Exact source ranges; unterminated bodies are labeled |
| Parse errors | Recoverable observations; duplicate attributes use the tokenizer's first value |

Inspection is not a DOM, rendered visibility analysis, browser-grade repair
algorithm, or full CSS/`srcset` parser. Parent path segments, control characters
and URL schemes are observations, not threat assessments.

### HTML limits

| Resource | Maximum |
| --- | --- |
| Input | 1 MiB |
| Tokens / attributes | 10,000 |
| Attributes per tag | 64 |
| Syntactic nesting | 128 |
| Recoverable errors | 64 |
| Reference / inline records | 256 |
| Individual field, body or text | 64 KiB |
| Aggregate exported text | 128 KiB |

## HAR inspection

`web.har.inspect` reads a HAR 1.2 request/response metadata subset: method, URL,
status, optional MIME/size/time/date fields, and body availability and encoding.
Unknown values are `null`. Headers, cookies, post data and bodies are not exported.

One leading UTF-8 BOM is accepted. Duplicate JSON keys are rejected. This subset
does not establish complete HAR conformance.

### HAR limits

| Resource | Maximum |
| --- | --- |
| Input | 256 KiB |
| JSON depth / nodes | 16 / 4,096 |
| Entries | 128 |
| String / key | 64 KiB / 256 bytes |
| URL | 8,192 bytes |
| Exported text | 128 KiB |

HTML and HAR inspection have no pipeline adapters or privileged I/O in their core
functions. Browser credentials and referenced files are never accessed.

## Embedded asset extraction

Select one asset explicitly:

| Operation and selection | Returned bytes | Requirements |
| --- | --- | --- |
| `web.html.asset`: inline script/style | Exact source UTF-8 | Zero-based record; explicit closing tag |
| `web.html.asset`: `selector=data_url` | Decoded Base64 | Strict standard-Base64 data URL after HTML character decoding |
| `web.har.asset`: Base64 body | Decoded bytes | Entry contains embedded `response.content.text` |
| `web.har.asset`: body without encoding | UTF-8 of the logical JSON string | May differ from original HTTP wire bytes |

Paths, remote URLs, percent decoding, non-Base64 data URLs, unknown HAR encodings
and external-body references are unsupported. Extraction does not decompress or
interpret content.

### Results and output

Output is at most 64 KiB; the original inspection budgets still apply. Direct
results include artifact SHA-256, record or pointer, source range when available,
representation, size and output SHA-256.

Choose one CLI output explicitly:

| Option | Output |
| --- | --- |
| `--stdout` | Raw bytes |
| `--output` | Exclusive new file |
| `--json` | Exact hex and provenance |

The two asset operations have version-1 text-to-bytes adapters under shared
pipeline budgets. Adapters return bytes; use direct extraction when provenance
must be retained. Local files can enter a byte pipeline through explicit input
selection, but references inside an artifact never authorize opening them.
