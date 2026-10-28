### Offline web artifacts

`web.html.inspect` accepts UTF-8 HTML as inert data. A maintained tokenizer reports
selected src/href/poster/data/action references, event-handler counts, tokenized
text excluding script/style, and exact source ranges for inline script/style.
References use HTML character decoding; text is tokenized, not rendered visibility.
It is not a DOM, browser-grade repair algorithm or full CSS/srcset parser. Duplicate
attributes follow the tokenizer's first-value rule and increment parse errors;
unterminated inline bodies are labeled. Recoverable HTML errors are observations.

HTML limits: 1 MiB input, 10,000 tokens/attributes, 64 attributes/tag, 128 syntactic
nesting, 64 recoverable errors, 256 reference/inline records, 64 KiB fields/body/text
and 128 KiB aggregate exported text. No script/event handler executes, no URL is
followed, no relative path is opened and no assets are downloaded. Parent path
segments, control characters or other schemes are observations, not threat claims.

`web.har.inspect` reads a documented HAR 1.2 request/response metadata subset.
It reports method/URL/status, optional MIME/size/time/date fields and body
availability/encoding, without exporting headers, cookies, post data or bodies.
Unknown values are null. A single leading UTF-8 BOM is accepted. Existing bounded
JSON parsing rejects duplicate keys, depth >16, >4,096 nodes or input >256 KiB;
HAR adds <=128 entries, <=64 KiB strings, <=256-byte keys, <=8,192-byte URLs and
<=128 KiB exported text. This subset is not complete HAR conformance validation.
Browser credentials and referenced files are never accessed. Both inspection
operations have no pipeline adapter or privileged I/O in their core functions.

`web.html.asset` selects a zero-based inline script/style record or reference from
that inspection. Inline selection preserves exact source UTF-8 bytes and requires
an explicit closing tag. `selector=data_url` accepts only strict standard-base64
data URLs after HTML character decoding; paths, remote URLs, percent decoding and
non-base64 data URLs are unsupported. `web.har.asset` selects an entry with an
embedded `response.content.text`. Base64 bodies yield decoded bytes; omitted
encoding yields UTF-8 of the logical JSON string, which may differ from original
HTTP wire bytes. Unknown encodings and external-body references fail. No automatic
decompression, content interpretation or filesystem/network resolution occurs.

Selections return bytes plus artifact SHA-256, record/pointer, source range where
available, representation, size and output SHA-256. Output is at most 64 KiB and
all inspection budgets still apply. CLI output must explicitly use `--stdout`,
`--output` (exclusive new file) or `--json` (exact Hex plus provenance). The two
asset operations have version-1 text-to-bytes adapters that reuse the existing
pipeline engine, value kinds, preflight, captures and lowered budgets. Adapter
output is bytes; use direct extraction first when provenance must be retained.
Caller-selected local files can enter the ordinary byte pipeline separately;
artifact references never authorize opening them. Existing recipes remain valid.
