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
and path contracts needed by a real compiler component. Existing application
IO and collection-like capabilities are evidence, but not a substitute for a
single pressure-tested compiler data model.

### `stage-neutral-ir-boundary`

Coordinate: `language-core/nuisc/stage-neutral-ir-boundary`.

Freeze producer-neutral source, token, AST, NIR, and YIR handoff records. The
serialized identity must not depend on Rust layout so the existing stage0 and
a future Nuis stage1 producer can be compared against the same contract.

### `stage0-stage1-driver`

Coordinate: `compiler-toolchain/bootstrap/stage0-stage1-driver`.

Build one Nuis-written compiler component through stage0 while recording exact
inputs, outputs, compiler identity, dependency closure, and stage identity.
This is a compiler-stage protocol over the existing AOT chain, not another
application build alias.

### `differential-reproducibility-gate`

Coordinate: `developer-system/bootstrap/differential-reproducibility-gate`.

Compare normalized AST, NIR, YIR, diagnostics, and deterministic artifact
identity across stages. Any semantic or reproducibility drift must block
component replacement rather than being hidden by byte-level shell success.

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
