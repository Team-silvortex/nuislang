# Nuis Compiler Stage Transformation

`nuis-compiler-stage-transformation-v1` is the producer-neutral registry for
deterministic stage outputs that are derived from, but are not byte copies of,
an existing compiler handoff payload. Its machine-readable contract is
[nuis-compiler-stage-transformation-v1.toml](nuis-compiler-stage-transformation-v1.toml).

The first registered transform is
`nuis-compiler-structural-checkpoint-v1`. The Nuis stage1 candidate consumes
the NIR projection through two 128-byte pages and emits a 22-word ordered
checkpoint:

```text
0       projection kind tag (NIR = 2)
1       page count (2)
2       first page identity
3       first cursor identity
4..11   first cursor lanes
12      continuation page identity
13      continuation cursor identity
14..21  continuation cursor lanes
```

This is a non-identity representation: its encoding is
`ordered-u64-le-v1`, not NIR text. The adapter transports raw bytes and opaque
cursor values only. It does not classify NIR records or construct the
checkpoint from host semantics.

## Verification

The manifest binds its candidate producer, handoff bundle, source payload
length and SHA-256, transform contract, encoding, exact word count, ordered
word SHA-256, every output word, and an aggregate proof identity. The artifact
reader independently reconstructs both NIR pages and cursors from the original
payload before comparing all 22 words.

Canonical parsing rejects unknown record keys, duplicate source stages,
reordered or missing words, noncanonical UTF-8/LF text, payload drift, output
hash drift, proof drift, and any attempt to set `replacement_authorized`.

Candidate production v7 additionally binds the exact manifest file length and
SHA-256. The two-clean-build reproducibility aggregate binds production proof
identity, so the transformation evidence is covered transitively without
adding paths or timestamps to the aggregate.

## Current Boundary

The checkpoint is real Nuis-produced data, but v1 intentionally sits beside
the unchanged five-stage handoff. It does not yet replace NIR or YIR bytes and
does not change the 13 byte-equivalence comparisons. The next protocol step is
to materialize a changed stage payload from this checkpoint and define a
semantic differential that can compare non-byte-identical stage encodings.

No transformation manifest grants compiler replacement authority.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_stage_transformation -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_production -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate pure_nuis_candidate_produces_an_attested_equivalent_stage1_component -j 1 -- --test-threads=1
```
