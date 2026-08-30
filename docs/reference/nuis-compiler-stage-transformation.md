# Nuis Compiler Stage Transformation

`nuis-compiler-stage-transformation-v3` is the producer-neutral registry for
deterministic stage payloads derived from an existing compiler handoff. Its
machine-readable contract is
[nuis-compiler-stage-transformation-v3.toml](nuis-compiler-stage-transformation-v3.toml).
V1 recorded checkpoint words only. V2 added a byte-different payload by
embedding the complete source bytes. V3 replaces that conservative envelope
with ordered structural records and keeps v2 frozen as historical evidence.

The current transform is `nuis-compiler-structured-record-codec-v1`. The Nuis
stage1 candidate still consumes the NIR projection through two 128-byte pages
and emits this ordered 22-word checkpoint:

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

## Compact Payload

The output encoding is `nuis-derived-structural-records-v2`. Each canonical
`nuis.compiler-stage-transformation.<ordinal>.bin` file contains:

```text
NSCSTG02                                8-byte magic
projection kind tag                    canonical unsigned LEB128
reconstructed source byte count        canonical unsigned LEB128
checkpoint word count                  canonical unsigned LEB128
structural record count                canonical unsigned LEB128
ordered checkpoint words               22 canonical unsigned LEB128 values
record depth and kind                  packed canonical unsigned LEB128
record body byte count                 canonical unsigned LEB128
record body                            UTF-8 record bytes without LF
```

Records retain source order and use the shared producer-neutral structural
kind vocabulary. Ordinals are implicit in record order; depth reconstructs
two-space indentation for ordinary records; opaque WGSL body records retain
their framing in the body at depth zero. Every record reconstructs exactly one
LF. The payload therefore carries the body bytes needed for lossless replay,
but it does not append or contain one contiguous complete NIR source blob. The
reference fixture is smaller than the equivalent v2 envelope.

The decoder rejects truncated, overflowing, and noncanonical varints, unknown
record kinds, impossible lengths, invalid UTF-8, and trailing bytes. It then
reconstructs the source, reparses it through
`nuis-compiler-structural-projection-v1`, and compares every ordinal, depth,
kind, and body before returning any bytes. The host adapter still transports
only source bytes and opaque Nuis-produced cursor words; it has no NIR
instruction classifier.

## Semantic Differential

[`nuis-compiler-stage-semantic-differential-v1`](nuis-compiler-stage-semantic-differential-v1.toml)
binds source and derived encodings, files, lengths and SHA-256 identities, the
checkpoint identity, handoff bundle, and transformation proof. Acceptance
requires source and derived bytes to differ, canonical record decoding, exact
structural metadata replay, independent checkpoint replay, and complete source
recovery while `replacement_authorized = false`.

This proves lossless representation equivalence, not permission to replace the
active NIR handoff record.

## Trust Chain

Candidate production v11 binds the exact transformation manifest, compact
payload metadata, semantic differential file, handoff v2 selection, and both
proof identities.
`bootstrap-diff` rereads that chain before producing the existing 13/13
component report. The two-clean-build reproducibility aggregate then binds the
production proof transitively without recording paths or timestamps.

Unknown keys, duplicate source stages, reordered words, noncanonical text or
integers, symlink payloads, malformed headers, length/hash drift, record
metadata drift, source recovery drift, proof drift, and replacement authority
all fail closed.

The canonical five-stage handoff remains unchanged in v1 form. Handoff v2 now
selects every registered reversible derived record beside that canonical
bundle, but does not replace it or authorize compiler replacement. Third and
later structural pages, independent attester trust, and reversible replacement
authorization remain open.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_stage_transformation -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_stage_semantic_differential -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate pure_nuis_candidate_produces_an_attested_equivalent_stage1_component -j 1 -- --test-threads=1
```
