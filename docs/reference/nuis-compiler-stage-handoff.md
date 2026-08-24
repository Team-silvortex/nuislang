# Nuis Compiler Stage Handoff

`nuis-compiler-stage-handoff-v1` is the first executable producer-neutral
boundary between the Rust-hosted stage0 compiler and a future Nuis-written
stage1 component. Its machine-readable contract is
[nuis-compiler-stage-handoff-v1.toml](nuis-compiler-stage-handoff-v1.toml).

This is an `early` self-hosting primitive, not a claim that stage1 exists.

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

Stage0 regenerates tokens and AST from the source before accepting a bundle.
It compares NIR against the pipeline's canonical projection. YIR uses the
explicit parser that does not synthesize hidden nodes or edges, then crosses
YIR verification and an exact render-parse-render check. Quoted YIR arguments
decode and re-encode spaces, newlines, quotes, tabs, carriage returns, and
backslashes canonically.

## Artifact Integration

Normal AOT builds include `compiler_source`, `compiler_tokens`, and
`compiler_stage_handoff` in `nuis.build.manifest.toml` artifact hashes. A
compile-cache hit must restore the same five payloads and preserve the exact
bundle identity.

The shared reader in `nuis-artifact` validates metadata, stage order,
encodings, parent identities, record identities, root containment, byte
lengths, payload hashes, and text policy before returning any payload.

## Current Limit

The boundary is not yet stage1-ready:

* AST is independently regenerated from source but has no standalone
  structural decoder for its projection.
* NIR is hash-bound and checked against stage0 output but has no standalone
  structural decoder.
* No Nuis-written producer emits this bundle yet.
* Compiler image and dependency-closure identity still belong to the next
  `stage0-stage1-driver` coordinate.

These limits keep the readiness coordinate at `early/45`, while making the
driver, rather than the serialized handoff format, the next weakest
bootstrap-critical task.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_stage_handoff -j 1
CARGO_INCREMENTAL=0 cargo test -q -p yir-syntax -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_data_model_bootstrap -j 1
```

The tests cover canonical manifest round trips, producer-independent identity,
payload tampering, invalid order and paths, explicit YIR parsing, escaped YIR
arguments, normal AOT artifact hashing, cache reuse, and native execution of
the pure Nuis compiler-data component.
