### Keyed skip and Josephus traversal

Both operate on stored Unicode scalar order, without normalization. Supply a
nonempty cyclic list of positive u64 steps; default start is the first scalar,
origin 0, direction forward. Origin 1 and reverse direction are explicit. The
first selection advances step-1 from the start; a step of 1 selects that anchor.
After each selection the anchor advances to the next scalar in the chosen
direction. Keyed skip retains the ring and requires count; Josephus removes the
selected scalar and resumes at the next remaining scalar, defaulting to all.
For ABCDE and step 2, keyed count 5 yields BDACE, Josephus yields BDAEC.
Keyed nonwrapping traversal errors if any next selection leaves the source;
Josephus requires wrapping. Explicit Josephus count can stop early; `remaining`
reports unselected source scalars. Output records keep original zero-based scalar
indexes and UTF-8 byte offsets, independently of the input origin.

Input <=1 MiB; step lists/counts <=65,536. Scratch memory <=8 MiB and serialized
reports <=8 MiB are independent bounds, excluding caller-owned input. Exact
source arrays cost eight bytes per scalar, plus four bytes per Fenwick cell for
Josephus; extraction releases these before reusing `text.extract`. Thus Josephus
can reject a large ASCII source below the byte limit. Work <=16,777,216 units
charges input bytes, step-list validation, source indexing, tree initialization, each selection/tree
visit, extraction visits and a conservative bound for serialized evidence. Rust
options and CLI/native Python limits can only lower ceilings. Any exhausted
budget fails without a partial success. These structured direct APIs have no
pipeline adapters and leave existing stride/recipe contracts unchanged.
