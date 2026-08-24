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
on an unstable language feature. V1 is now executable through
`nuisc bootstrap-check`; see
[Nuis Bootstrap Language Subset](nuis-bootstrap-language-subset.md).
This gate is `stable/100`; widening it requires a new protocol version.

### `compiler-data-model`

Coordinate: `standard-library/std/compiler-data-model`.

Provide the minimum owned text, vector, map, arena, source-span, diagnostic,
and path contracts needed by a real compiler component. The bounded v1 model
is now `usable/75`: `StdLanguageCore` owns the representation, the frozen
bootstrap subset accepts it, and
`bootstrap_compiler_data_model_demo` crosses bootstrap-check, NIR/YIR/LLVM,
native build, and deterministic execution without FFI or host collections.
See [Nuis Compiler Data Model](nuis-compiler-data-model.md).

This is deliberately not `stable/100`. V1 proves the ownership and lowering
shape with four-slot vectors/maps and integer map keys. A realistic tokenizer
or parser still needs deterministic page- or chunk-backed growth, larger
pressure fixtures, and preserved differential identities.

### `stage-neutral-ir-boundary`

Coordinate: `language-core/nuisc/stage-neutral-ir-boundary`.

Freeze producer-neutral source, token, AST, NIR, and YIR handoff records. The
serialized identity must not depend on Rust layout so the existing stage0 and
a future Nuis stage1 producer can be compared against the same contract.

This gate is now `early/60`. Normal AOT builds emit the ordered five-stage
`nuis-compiler-stage-handoff-v1` SHA-256 chain, hash its source/token/manifest
artifacts in the build manifest, and preserve bundle identity across cache
hits. The shared `nuis-compiler-structural-projection-v1` codec independently
parses and canonically re-renders AST/NIR hierarchy and module identity without
source reconstruction or producer-private layout. Explicit YIR crosses parse,
verify, and canonical re-render. A Nuis-owned typed consumer and second
producer remain open. See
[Nuis Compiler Stage Handoff](nuis-compiler-stage-handoff.md).

### `stage0-stage1-driver`

Coordinate: `compiler-toolchain/bootstrap/stage0-stage1-driver`.

This gate is now `early/60`. `nuis bootstrap-build` is a dedicated project-only
driver over the frozen bootstrap gate and normal AOT pipeline. It consumes the
five-stage handoff and emits `nuis-compiler-component-build-v1`, binding the
exact stage0 compiler image, native output, build outputs, project/Galaxy/
Nustar dependency closure, a cache-stable reproducible identity, and an exact
audit identity. It also emits `nuis-compiler-diagnostic-report-v1`, bound to
the exact component record and producer. See
[Nuis Compiler Component Build](nuis-compiler-component-build.md).

The driver is not another unchecked application-build alias, but it is still
only the stage0 half. No Nuis stage1 producer or replacement authorization
exists yet.

### `differential-reproducibility-gate`

Coordinate: `developer-system/bootstrap/differential-reproducibility-gate`.

This gate is now `early/60`. `nuis bootstrap-diff` consumes verified stage0 and
explicit `stage1-candidate` records plus their handoffs, payloads, normalized
diagnostics, dependency closures, and native outputs. Its fixed thirteen-check
report emits `blocked-drift` or `equivalent-awaiting-authorization`; both keep
`replacement_authorized = false`. Valid drift is retained as an audit report
and returns a failing command status. See
[Nuis Compiler Component Differential Gate](nuis-compiler-component-differential.md).

The comparison engine is implemented, but there is still no real Nuis stage1
producer. Therefore repository-native cross-producer equivalence and a
separate reversible replacement authorization remain open. The implemented
structural codec means the next weakest work is no longer a missing comparison
or handoff protocol. It is the first real leaf component emitted by a Nuis
stage1-candidate producer.

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
