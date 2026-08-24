# Nuis Compiler Component Build

`nuis-compiler-component-build-v1` is the first reusable stage-driver record
for a Nuis-written compiler component. Its machine-readable contract is
[nuis-compiler-component-build-v1.toml](nuis-compiler-component-build-v1.toml).

This is an `early` stage0 capability. It does not claim that a Nuis stage1
compiler exists and it grants no component-replacement authority.

The shared record accepts only the explicit roles `stage0` and
`stage1-candidate`. Role and producer identity participate in both component
identities, so a candidate can never masquerade as stage0. The current public
driver emits only `stage0`.

## Frontdoor

Build one project-form compiler component with:

```bash
cargo run -q -p nuis -- bootstrap-build path/to/project path/to/output
```

The installed form is:

```bash
nuis bootstrap-build path/to/project path/to/output
```

Single source files are rejected. A compiler component must have a project
manifest so the driver can attest its complete logical dependency closure.

The driver does not introduce a private compiler pipeline. It first applies
the frozen bootstrap subset gate, then uses the normal semantic, NIR, YIR,
LLVM, native-link, and artifact pipeline. It additionally requires and
consumes the producer-neutral compiler-stage handoff before writing
`nuis.compiler-component-build.toml`. A successful build also writes the
component-bound `nuis.compiler-diagnostics.toml` used by the differential
gate.

Bootstrap validation, cache identity, compilation, and dependency collection
share one resolved project snapshot. The driver does not independently reload
the project for each phase and accidentally attest a different source graph.

## Bound Inputs

The record binds exact bytes and lowercase SHA-256 identities for:

* the currently executing compiler image
* the ordered source, tokens, AST, NIR, and YIR handoff bundle
* the build manifest and compiled artifact container
* the native executable
* the accepted, checked, normalized diagnostic state
* the project manifest and project-local Nuis sources
* the generated Galaxy lock plus resolved Galaxy manifests, sources, and
  libraries
* the Nustar index and every loaded Nustar manifest

Dependencies use logical identities, not workspace paths. They are uniquely
sorted by kind and identity before the dependency-closure hash is computed.
Absolute paths, traversal, duplicate identities, empty payloads, and more than
4096 dependency records fail closed.

## Identity Layers

The protocol deliberately exposes two component-level identities.

`reproducible_build_sha256` binds semantic and executable inputs that must be
stable across an unchanged miss-to-hit cache rebuild: compiler image, stage
bundle, native executable, and complete dependency closure. It excludes
cache-status bookkeeping and the cache-sensitive outer compiled container.

`record_sha256` is the exact audit identity. It additionally binds the exact
build manifest, compiled artifact container, sibling filenames, byte lengths,
and the reproducible identity. It is allowed to change when an operational
cache record changes, even if the reproducible build remains identical.

Short rule:

```text
reproducible identity answers "did the compiler component stay the same?"
audit identity answers "is this exact build record unchanged?"
```

Neither identity alone proves semantic equivalence between two compiler
producers. That belongs to the differential gate.

The diagnostic sidecar binds the exact `record_sha256` rather than changing
component-build v1. This keeps diagnostic normalization independently
versionable while preventing a report from being moved onto another component
record.

## Verification

`read_compiler_component_build` parses the canonical TOML, recomputes all
record identities, resolves only sibling output files, re-reads every bound
payload, verifies the build manifest and stage handoff, and cross-checks their
filenames and hashes. `verify_compiler_component_build_image` additionally
requires exact equality with the compiler image supplied by the caller.

The native regression builds the same pure Nuis compiler-data component twice
in one output directory. The second run exercises the cache-hit path and must
retain compiler-image, dependency-closure, handoff, native-output, and
reproducible-build identities. The test executes the native program with
deterministic score 43, then mutates its bytes and proves the record reader
rejects it.

## Honest Boundary

The current stage0 half is reusable and attested, but stage1 is still absent.
The fail-closed comparison protocol and `nuis bootstrap-diff` frontdoor now
exist; see
[Nuis Compiler Component Differential Gate](nuis-compiler-component-differential.md).
The next boundary is a separately identified Nuis candidate producer. Only a
later reversible authorization record may permit replacing one compiler
component.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_build -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_data_model_bootstrap -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc --lib parse_bootstrap_build_command -j 1
```
