# Nuis Compiler Stage Handoff

`nuis-compiler-stage-handoff-v1` is the first executable producer-neutral
boundary between the Rust-hosted stage0 compiler and a future Nuis-written
stage1 component. Its machine-readable contract is
[nuis-compiler-stage-handoff-v1.toml](nuis-compiler-stage-handoff-v1.toml).
`nuis-compiler-stage-handoff-v2` retains that canonical bundle and adds a
separate selection proof for registered, reversible derived stages. Its
contract is
[nuis-compiler-stage-handoff-v2.toml](nuis-compiler-stage-handoff-v2.toml).

This self-hosting primitive is now consumed by the first bounded stage1 leaf,
not a claim that the whole compiler is stage1.

## Ordered Bundle

Every normal source build emits `nuis.compiler-stage-handoff.toml` beside five
payloads in one fixed order:

| Ordinal | Stage | Encoding |
| --- | --- | --- |
| 0 | `source` | `utf8-lf-v1` |
| 1 | `tokens` | `nuis-token-stream-v1` |
| 2 | `ast` | `nuis-ast-canonical-projection-v1` |
| 3 | `nir` | `nuis-nir-canonical-projection-v1` |
| 4 | `yir` | `yir-text-v1` |

Payload paths are single relative file names. They cannot be absolute, contain
parent traversal, or escape the manifest directory after canonicalization.
Every payload must be UTF-8/LF text without NUL bytes.

Generated test and benchmark harnesses do not invent a source record. They use
the legacy artifact writer without a handoff until an equivalent source-level
producer contract exists.

## Identity Chain

The semantic root binds the handoff protocol, producer contract, module
domain, and module unit. Each record then binds:

* semantic root and previous record identity
* ordinal, stage, and encoding
* payload byte length and SHA-256

The final record identity is the bundle identity. Fields are length-prefixed
before hashing, and numeric fields use fixed little-endian `u64` encoding.

`producer_id` is retained for audit but deliberately excluded from semantic
identity. Two conforming producers that emit the same five payloads therefore
receive the same semantic root, records, and bundle hash.

## Canonical Payloads

The token stream is reversible and independent of Rust enum layout. Words,
strings, and floating literals use lowercase UTF-8 hexadecimal payloads;
integers use canonical decimal; symbols use Unicode scalar integers; arrows
have a dedicated record.

AST and NIR now share `nuis-compiler-structural-projection-v1`. The codec
independently decodes module documentation, imports, the exact module
domain/unit header, and the two-space structural hierarchy. Record depth may
increase by one level at a time; empty records, odd indentation, trailing
whitespace, misplaced imports, NIR documentation, duplicate module headers,
and module-identity drift fail closed. The decoded records canonically
re-render the exact payload without reparsing source or depending on Rust enum
layout. Multiline inline WGSL remains an explicitly framed opaque NIR leaf:
its source lines preserve exact bytes without being mistaken for NIR hierarchy,
and a missing or malformed `})` terminator fails closed.

Both the shared handoff builder and reader invoke this codec. `nuisc` also
compares the decoded payload bytes with the in-memory pipeline projection when
it owns that pipeline object. YIR uses its explicit parser, which does not
synthesize hidden nodes or edges, then crosses YIR verification and an exact
render-parse-render check. Quoted YIR arguments decode and re-encode spaces,
newlines, quotes, tabs, carriage returns, and backslashes canonically.

## Artifact Integration

Normal AOT builds include `compiler_source`, `compiler_tokens`, and
`compiler_stage_handoff` in `nuis.build.manifest.toml` artifact hashes. A
compile-cache hit must restore the same five payloads and preserve the exact
bundle identity.

The shared reader in `nuis-artifact` validates metadata, stage order,
encodings, parent identities, record identities, root containment, byte
lengths, payload hashes, and text policy before returning any payload.

Handoff v2 then binds the v1 bundle, transformation registry, semantic
differential, and one ordered selection for every registered transform. Each
selection includes the canonical source record, transform contract, derived
file and encoding, checkpoint, recovered source hash, and reversible semantic
verdict. Its reader replays all sibling evidence before returning. It does not
rewrite v1 or grant replacement authority.

## Current Limit

The first identity-projection path is now stage1-candidate ready:

* `StdCompilerProjection` now provides a Nuis-owned streaming state machine for
  typed AST/NIR record tags. The checked-in candidate validates valid AST/NIR
  sequences, rejects malformed boundaries, and executes natively through the
  frozen bootstrap and normal AOT pipeline.
* The scalar producer ABI ingests every byte from all five serialized stage
  payloads and computes a Nuis-owned deterministic stage/bundle fold.
* `nuis bootstrap-candidate-build` materializes a separately identified
  candidate handoff and binds it through
  `nuis-compiler-candidate-production-v11`.
* The default production adapter blindly transports one token page and two
  AST/NIR pages. Its additive structural-pagination mode transports a third
  page for each projection without changing production-v11. Nuis
  owns the token records plus canonical emission, serializes an opaque
  eight-lane structural cursor, and resumes both projections into page two.
* The Nuis consumer can resume repeatedly; production-v11 binds two pages for
  each projection, while its successor independently binds page three, compact
  byte-different AST and NIR records, and their `2/2`
  semantic differential. Handoff v2 selects both registered derived records
  without embedding AST, NIR, or transform-specific logic in the protocol.
* Compiler image and dependency-closure identity are added by the separate
  `nuis-compiler-component-build-v1` stage-driver record.
* Replacement still requires the separate differential and authorization
  contracts; matching payload hashes alone never authorize it.

The independent codec, native Nuis consumer, bounded token page, resumable AST
and NIR pages, compact derived records, semantic proof, v2 selection,
production proof, candidate-owned three-page successor, cross-root semantic
agreement, and `13/13` differential close this bounded coordinate at
`stable/100`. Complete-stream structural pagination, independent attester trust,
and reversible replacement authorization remain separate work. See
[Nuis Compiler Candidate Execution](nuis-compiler-candidate-execution.md),
[Nuis Compiler Candidate Production](nuis-compiler-candidate-production.md),
[Nuis Compiler Stage Transformation](nuis-compiler-stage-transformation.md),
and [Nuis Compiler Component Build](nuis-compiler-component-build.md).

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_stage_handoff -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_stage_handoff_v2 -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_structural_projection -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_structural_projection_page -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_structural_pagination -j 1
CARGO_INCREMENTAL=0 cargo test -q -p yir-syntax -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_data_model_bootstrap -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate -j 1
```

The tests cover canonical manifest and AST/NIR structural round trips,
producer-independent identity, malformed hierarchy, module drift, payload
tampering after a recomputed SHA/parent/bundle chain, invalid order and paths,
explicit YIR parsing, empty and escaped YIR arguments, normal AOT artifact
hashing, cache reuse, native execution of the pure Nuis compiler-data
component, native execution plus tamper rejection for the first typed Nuis
structural consumer, and independent agreement on the AST and NIR first-page
continuation states.
