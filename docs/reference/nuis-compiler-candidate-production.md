# Nuis Compiler Candidate Production

`nuis-compiler-candidate-production-v1` is the first attested path from an
executed Nuis compiler-shaped program to a separately identified
`stage1-candidate` leaf component. Its machine-readable contract is
[nuis-compiler-candidate-production-v1.toml](nuis-compiler-candidate-production-v1.toml).

This closes one leaf production loop. It does not mean that `nuisc` is
self-hosted, and it never authorizes replacing stage0.

## Frontdoor

Build the checked-in projection relay through both producers with:

```bash
nuis bootstrap-candidate-build path/to/project path/to/output
```

The output root contains:

* `stage0/`, including the ordinary component, execution, handoff, diagnostic,
  and native-image evidence
* `stage1-candidate/`, including the candidate component, handoff, diagnostics,
  scalar adapter, and production proof
* `nuis.compiler-component-diff.toml`, which must report all thirteen
  comparisons equivalent while retaining `replacement_authorized = false`

## Producer Boundary

The Nuis source exports four exact scalar functions for stage seed/fold and
bundle seed/fold. Bootstrap subset v1 accepts only those exact function names,
symbol names, parameter counts, and all-`i64` signatures. Arbitrary exports
continue to fail as `NBS004`.

The generated host adapter opens the five verified payload files and passes
every byte, in order, through the Nuis fold function. It performs no parsing,
diagnostic, stage, or replacement decision. The Rust host independently
recomputes the same folds before it can materialize a candidate handoff.

This split is deliberate:

```text
host adapter = byte transport
Nuis image = candidate projection computation
shared artifact layer = identity and authority verification
```

The adapter is rebuilt without the ordinary process `main`; the normal Nuis
runtime shim remains otherwise unchanged. Its exact binary length and SHA-256
are part of the production proof.

## Promotion

The candidate handoff preserves the producer-neutral semantic bundle while
changing the auditable producer identity to
`nuis-stage1-projection-relay-v1`. The promoted component keeps the same
component identity, native output, dependency closure, and five stage payloads,
but declares the explicit `stage1-candidate` role and uses the executed Nuis
image as its compiler image.

The production proof binds both components, the earlier execution proof, the
candidate image, all five byte lengths/SHA-256/folds, the bundle fold, and the
adapter. `bootstrap-diff` verifies this proof before writing its report.
Changing the adapter, a stage payload, role, producer, component record, or
proof therefore fails closed.

## Current Limit

The first producer is an identity projection relay: Nuis consumes every
serialized stage byte and owns the deterministic bundle fold, while the shared
host codec still owns token and structural-body decoding. The next compiler
step is to move one real token or structural transformation behind the same
ABI, then prove candidate reproducibility across independent clean builds.

Replacement authorization remains a separate future protocol with rollback
evidence. A `13/13` report is evidence of equivalence, not permission to switch
the active compiler.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_production -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc --lib exact_scalar_candidate_export_is_allowed_but_symbol_spoofing_is_rejected -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate -j 1
```
