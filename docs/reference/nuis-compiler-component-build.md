# Nuis Compiler Component Build

`nuis-compiler-component-build-v1` is the first reusable stage-driver record
for a Nuis-written compiler component. Its machine-readable contract is
[nuis-compiler-component-build-v1.toml](nuis-compiler-component-build-v1.toml).

This now supports stage0 plus the first bounded Nuis-produced
`stage1-candidate` leaf. It does not claim that the whole compiler is stage1,
and it grants no component-replacement authority.

The shared record accepts only the explicit roles `stage0` and
`stage1-candidate`. Role and producer identity participate in both component
identities, so a candidate can never masquerade as stage0. `bootstrap-build`
emits stage0; the separately attested `bootstrap-candidate-build` path may
promote only an executed Nuis image whose five-stage production proof verifies.

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
deterministic score 59, then mutates its bytes and proves the record reader
rejects it.

## Honest Boundary

The reusable stage0 half and the first stage1 leaf are now both attested.
`nuis bootstrap-candidate-probe` executes the pure Nuis structural consumer and
writes an explicitly non-authoritative execution proof. The new
`nuis bootstrap-candidate-build` frontdoor then passes every stage byte through
the candidate's exact scalar ABI, independently verifies its folds, and emits
the production proof, candidate component, diagnostics, and differential. See
[Nuis Compiler Candidate Execution](nuis-compiler-candidate-execution.md) and
[Nuis Compiler Candidate Production](nuis-compiler-candidate-production.md).
The fail-closed comparison protocol and `nuis bootstrap-diff` frontdoor now
exist; see
[Nuis Compiler Component Differential Gate](nuis-compiler-component-differential.md).
`nuis bootstrap-reproducibility` now proves the current identity relay across
two empty, compile-cache-bypassed roots; see
[Nuis Compiler Component Reproducibility](nuis-compiler-component-reproducibility.md).
Independent authorization now feeds active-state v1 and signed transition v2;
dispatch v1 resolves their current/forward identities to exact image bytes and
executes the restored stage0 frontdoor. See
[Nuis Compiler Component Dispatch](nuis-compiler-component-dispatch.md).
The current producer is still not a tokenizer/parser replacement. A later
component must own a real transformation and retain clean-build equivalence.
Only an independent reversible authorization record may permit replacement.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_component_build -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_execution -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_production -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_data_model_bootstrap -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc --lib parse_bootstrap_build_command -j 1
```
