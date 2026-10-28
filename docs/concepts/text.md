# Unicode and text preparation

A text operation can count bytes, Unicode values or visible characters. Check its
reference before using a length or position from another operation.

## Bytes, scalars and visible characters

UTF-8 stores a Unicode scalar in one to four bytes. A scalar is one Unicode value;
a visible character can contain several scalars.

| Text | Stored units |
| --- | --- |
| `aaa` | 3 UTF-8 bytes; 3 scalars, one distinct value |
| `é` as U+00E9 | 2 UTF-8 bytes; 1 scalar |
| `e` followed by U+0301, a combining accent | 3 UTF-8 bytes; 2 scalars |

The last two rows can look alike. Scalar-based selection, reversal and comparison
still see different stored sequences. Emoji sequences can also span several
scalars. These operations do not measure rendered character width.

A **byte offset** locates data in the UTF-8 encoding. A **scalar index** counts
Unicode values. For example, in `éa`, `a` has scalar index 1 and byte offset 2
when indexes start at zero. Each operation states its index origin and whether
positions refer to original or prepared text.

## Preparation changes the values being counted

Normalization can combine or decompose scalars. NFC turns `e` followed by U+0301
into `é`; NFD decomposes `é` into those two scalars. NFKC and NFKD also apply
compatibility mappings, which can remove distinctions. Reports identify the
normalization data version where supported.

Case handling and filtering are separate choices. Lowercasing ASCII A–Z does not
fold accented letters or perform language-sensitive case matching. A filter can
remove whitespace or punctuation from the prepared input even though processing
must still visit them.

Preparation is operation-specific. Statistics, multiset comparison and anagram
search use a [shared preparation profile](../reference/text-preparation.md).
Word ladders allow normalization and ASCII lowercasing but retain punctuation.
Classical cryptanalysis has an ASCII-letter view that **uppercases** letters.
Traversal operations use stored scalar order without normalization.

Use [text and Unicode inspection](../guides/text-evidence.md) to inspect the stored
values before selecting a preparation profile.
