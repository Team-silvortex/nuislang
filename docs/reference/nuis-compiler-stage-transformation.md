# Nuis Compiler Stage Transformation

`nuis-compiler-stage-transformation-v2` is the producer-neutral registry for
deterministic stage payloads derived from an existing compiler handoff. Its
machine-readable contract is
[nuis-compiler-stage-transformation-v2.toml](nuis-compiler-stage-transformation-v2.toml).
V1 recorded Nuis-produced checkpoint words only; v2 binds and materializes an
actual byte-different, losslessly decodable payload.

The first registered transform remains
`nuis-compiler-structural-checkpoint-v1`. The Nuis stage1 candidate consumes
the NIR projection through two 128-byte pages and emits this ordered 22-word
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

## Derived Payload

The output encoding is `nuis-derived-structural-stage-payload-v1`. Each
canonical `nuis.compiler-stage-transformation.<ordinal>.bin` file contains:

```text
NSCSTG01                              8-byte magic
projection kind tag                  u64 little-endian
complete source payload byte count   u64 little-endian
checkpoint word count                u64 little-endian
ordered checkpoint words             22 * u64 little-endian
complete source stage payload        raw bytes
```

This is intentionally a conservative first changed representation. The host
adapter only transports the original bytes and the opaque Nuis-produced words;
it does not classify NIR. The artifact layer independently rebuilds every page
and cursor, decodes the binary, compares all words, and requires the embedded
payload to recover the complete original NIR byte-for-byte.

The derived payload is therefore genuinely different from NIR text while
remaining reversible. A later codec may replace embedded text with structured
binary records, but only after it can prove the same producer-neutral semantic
boundary.

## Semantic Differential

[`nuis-compiler-stage-semantic-differential-v1`](nuis-compiler-stage-semantic-differential-v1.toml)
compares source and derived representations. It binds both encodings, files,
lengths and SHA-256 identities, the checkpoint identity, the handoff bundle,
and the transformation proof. Acceptance requires:

* source and derived bytes are not identical
* the derived payload decodes canonically
* every checkpoint word survives independent structural replay
* recovered source bytes equal the complete source payload
* all semantic comparisons pass while `replacement_authorized = false`

This proves lossless representation equivalence, not permission to replace the
active NIR handoff record.

## Trust Chain

Candidate production v8 binds the exact transformation manifest, derived
payload metadata, semantic differential file and semantic proof identity.
`bootstrap-diff` rereads that chain before producing the existing 13/13
component report. The two-clean-build reproducibility aggregate then binds the
production proof transitively without recording paths or timestamps.

Unknown keys, duplicate source stages, reordered words, noncanonical text,
symlink payloads, malformed binary headers, length/hash drift, source recovery
drift, proof drift, and replacement authority all fail closed.

The canonical five-stage handoff remains unchanged in v1 form. A future
handoff v2 may select a derived NIR record only after a separate reversible
replacement protocol exists.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate pure_nuis_candidate_produces_an_attested_equivalent_stage1_component -j 1 -- --test-threads=1
```
