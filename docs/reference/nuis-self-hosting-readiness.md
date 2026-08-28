# Nuis Self-Hosting Readiness

This reference defines the executable preparation boundary for beginning the
formal `stage0 -> stage1` compiler migration at `beta-0.10.*`. The checked-in
machine-readable source is
[nuis-self-hosting-readiness.toml](nuis-self-hosting-readiness.toml).
Completed stage2-equivalent compiler ownership remains a later
`gamma-0.5.*` through `gamma-0.10.*` closure window.

## Roadmap Is Not Readiness

`developer-system/dev-tensor/self-hosting-phase-roadmap` is `stable/100`
because the schedule is agreed and versioned. It does not mean a Nuis-written
compiler stage exists.

The readiness manifest therefore owns five separate bootstrap-critical
coordinates with their own evidence, status, progress, blocker, next action,
validation command, and expected artifact. These coordinates begin at honest
nonterminal scores and can only close independently.

## Frontdoor

Use the repository-relative default manifest:

```bash
cargo run -q -p nuis -- bootstrap-status
cargo run -q -p nuis -- bootstrap-status --json
```

The installed public form is `nuis bootstrap-status --json`.

An explicit manifest can be inspected without changing repository state:

```bash
cargo run -q -p nuis -- bootstrap-status --json path/to/readiness.toml
```

The command validates the exact protocol, required gate set, coordinate
identity, unique IDs, field completeness, status vocabulary, progress range,
and declared gate count. A structurally valid but unfinished manifest exits
successfully with `ready=false`; invalid protocol state fails closed.

`ready=true` is emitted only when all five gates are exactly `stable/100`.
The command selects the next incomplete gate in dependency order rather than
using the roadmap score or registration order.

## Required Gates

### `bootstrap-language-subset`

Coordinate: `language-core/nuisc/bootstrap-language-subset`.

Freeze the syntax, type, effect, control-flow, generic, pointer, FFI, and
library surface permitted in compiler-authoring sources. Both accepted and
rejected fixtures are required so stage1 cannot acquire accidental dependency
on an unstable language feature. V3 is now executable through
`nuisc bootstrap-check`; see
[Nuis Bootstrap Language Subset](nuis-bootstrap-language-subset.md).
This gate is `stable/100`; widening it requires a new protocol version.

### `compiler-data-model`

Coordinate: `standard-library/std/compiler-data-model`.

Provide the minimum owned text, vector, map, arena, source-span, diagnostic,
and path contracts needed by a real compiler component. Data model v3 is now
`usable/88`: `StdLanguageCore` owns the foundational representation,
`StdCompilerData` owns materialized token records and payloads,
`StdCompilerTokens` owns the standalone token DFA, and
`StdCompilerTokenEmit` reconstructs canonical `nuis-token-stream-v1` bytes
while materializing a complete bounded page into the owned store. The frozen
subset accepts all four modules, and
`bootstrap_compiler_data_model_demo` crosses bootstrap-check, NIR/YIR/LLVM,
native build, and deterministic exit `122` without FFI or host collections. It
materializes four records, emits 59 canonical bytes, then materializes the
real `use cpu StdLanguageCore;` token prefix and emits its exact 91-byte
canonical page with pinned hash `1277127995`. It decodes those bytes into a
fresh store and re-emits the identical length and hash. A two-vector, seven-byte-word
packing keeps the 128-byte output buffer near the old compile-time baseline.
See [Nuis Compiler Data Model](nuis-compiler-data-model.md).

This is deliberately not `stable/100`. V3 is bounded to four token records and
64 payload bytes and 128 output bytes, while the production candidate adapter
does not yet feed its actual handoff into the owned store. Vectors remain `i64`-specific, maps remain
four-entry, arenas do not yet hold object storage, generic nested-page
specialization still lacks defining-module provenance. The stage-neutral IR boundary is now
the weakest readiness gate.

### `stage-neutral-ir-boundary`

Coordinate: `language-core/nuisc/stage-neutral-ir-boundary`.

Freeze producer-neutral source, token, AST, NIR, and YIR handoff records. The
serialized identity must not depend on Rust layout so the existing stage0 and
a future Nuis stage1 producer can be compared against the same contract.

This gate is now `usable/86`. Normal AOT builds emit the ordered five-stage
`nuis-compiler-stage-handoff-v1` SHA-256 chain, hash its source/token/manifest
artifacts in the build manifest, and preserve bundle identity across cache
hits. The shared `nuis-compiler-structural-projection-v1` codec independently
parses and canonically re-renders AST/NIR hierarchy and module identity without
source reconstruction or producer-private layout. Explicit YIR crosses parse,
verify, and canonical re-render. `StdCompilerProjection` now supplies a typed
streaming consumer whose pure Nuis candidate crosses bootstrap-check, native
AOT execution, malformed-sequence rejection, and tamper-checked execution
proof. The exact scalar producer ABI now consumes every serialized stage byte,
emits a Nuis-owned bundle fold, and drives `StdCompilerTokens` across the exact
token header, seven record kinds, payload shape, and LF boundaries. Candidate
production v2 binds its record count and decoded semantic fold to an independent
artifact-layer result. Separately, the pure Nuis materializer decodes the exact
91-byte candidate prefix into owned records and canonically re-emits the same
hash. Production-bound page identity, changed stage bytes, and structural-body
decoding in Nuis remain open. See
[Nuis Compiler Stage Handoff](nuis-compiler-stage-handoff.md).

### `stage0-stage1-driver`

Coordinate: `compiler-toolchain/bootstrap/stage0-stage1-driver`.

This gate is now `usable/87`. `nuis bootstrap-build` is a dedicated project-only
driver over the frozen bootstrap gate and normal AOT pipeline. It consumes the
five-stage handoff and emits `nuis-compiler-component-build-v1`, binding the
exact stage0 compiler image, native output, build outputs, project/Galaxy/
Nustar dependency closure, a cache-stable reproducible identity, and an exact
audit identity. It also emits `nuis-compiler-diagnostic-report-v1`, bound to
the exact component record and producer. See
[Nuis Compiler Component Build](nuis-compiler-component-build.md).

`nuis bootstrap-candidate-probe` additionally executes the first pure Nuis
candidate with empty argv, closed stdin, required exit `0`, and empty output,
then binds its image and result to
`nuis-compiler-candidate-execution-v1`. See
[Nuis Compiler Candidate Execution](nuis-compiler-candidate-execution.md).

The probe authority remains explicitly execution-only. The separate
`nuis bootstrap-candidate-build` frontdoor feeds all five payloads through the
candidate's exact scalar exports, independently verifies the folds and token
decode summary, emits a
candidate handoff/component/diagnostic set plus
`nuis-compiler-candidate-production-v2`, and then runs the differential gate.
See [Nuis Compiler Candidate Production](nuis-compiler-candidate-production.md).
The first producer is now the bounded `nuis-stage1-token-decoder-v2` leaf; it
still preserves canonical stage bytes and grants no replacement authority.

`nuis bootstrap-reproducibility` now runs that complete production chain in
two initially empty roots with compile-cache read/write bypass. Its path-free
`nuis-compiler-component-reproducibility-v1` aggregate binds distinct local
witnesses, both exact report lineages, and stable component/native identities.
The local witness intentionally carries no independent attester authority.

### `differential-reproducibility-gate`

Coordinate: `developer-system/bootstrap/differential-reproducibility-gate`.

This gate is now `usable/87`. `nuis bootstrap-diff` consumes verified stage0 and
explicit `stage1-candidate` records plus their handoffs, payloads, normalized
diagnostics, dependency closures, and native outputs. Its fixed thirteen-check
report emits `blocked-drift` or `equivalent-awaiting-authorization`; both keep
`replacement_authorized = false`. Valid drift is retained as an audit report
and returns a failing command status. See
[Nuis Compiler Component Differential Gate](nuis-compiler-component-differential.md).

The checked-in Nuis token decoder now enters this path as a real
`stage1-candidate` leaf and reaches repository-native `13/13` equivalence. The
path frontdoor verifies its execution and production proofs, including exact
adapter bytes, all stage folds, and the independently reproduced token summary
before writing the report. Two local clean,
cache-bypassed runs now retain stable reproducible identities and 13/13
verdicts; root or aggregate tampering fails closed. Complete token re-emission,
independent-machine trust, and separate reversible replacement authorization
remain open.

## Migration Rule

The preparation order is:

```text
freeze subset and compiler data contracts
  -> freeze stage-neutral handoffs
  -> build one component through stage0
  -> compare stage0 and candidate stage1 outputs
  -> permit one reversible component replacement
```

The first stage1 component should be deliberately small and leaf-like. It must
exercise source parsing or diagnostic production while avoiding ownership of
the whole compiler driver. Replacement remains reversible until repeated
stage1 builds and differential reports are deterministic.

## Honest Boundary

This protocol is preparation infrastructure. It does not claim that Nuis is
self-hosted, that the bootstrap subset is stable public language surface, or
that `beta-0.10.*` guarantees immediate compiler replacement. It makes missing
prerequisites visible early enough for that minor line to begin migration
without inventing the ground rules at the same time.
