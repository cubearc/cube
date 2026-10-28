### Signed positions inside words

`text.word.extract` reuses Unicode-whitespace tokenization and the book cipher's
origin resolver. Punctuation is data; positions count scalars, not graphemes.
Choose one shared ordered list or an explicit list for every word. Positive
origin defaults to 1 (zero invalid); origin 0 is optional. Negative -1 is always
last, -2 penultimate. Requests preserve order and duplicates, never cycle between
words. `short_word=error` is the default; `skip` retains a null selection record
and increments omitted_count. No placeholder is inferred.

Reports retain word boundaries, original global scalar indexes and UTF-8 byte
offsets. Empty text with shared positions or an empty per-word sequence yields an
empty report. Input <=1 MiB, words/total coordinates <=65,536, source index <8 MiB
and serialized report <=8 MiB. Index construction uses shared word slices and
bounded scalar-byte offsets; each requested coordinate resolves in constant time,
without rescanning a word per request. CLI position JSON is independently bounded
at 2 MiB. Output limits can be stricter than coordinate limits for verbose reports.
This direct structured operation has no pipeline adapter.
