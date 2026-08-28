# Nuis Compiler Candidate Production

`nuis-compiler-candidate-production-v2` is the first attested path from an
executed Nuis compiler-shaped program with a bounded token decoder to a
separately identified `stage1-candidate` leaf component. Its machine-readable
contract is
[nuis-compiler-candidate-production-v2.toml](nuis-compiler-candidate-production-v2.toml).

This closes one leaf production loop. It does not mean that `nuisc` is
self-hosted, and it never authorizes replacing stage0.

## Frontdoor

Build the checked-in bounded token-decoder candidate through both producers
with:

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

The Nuis source exports twelve exact scalar functions: four for stage and
bundle folds, plus eight for a bounded token-stream DFA. Bootstrap subset v3
accepts only those exact function names, symbol names, parameter counts, and
all-`i64` signatures. Arbitrary exports continue to fail as `NBS004`.

The generated host adapter opens the five verified payload files and passes
every byte, in order, through the Nuis fold function. For the token payload it
also carries scalar mode, count, and fold state between calls into the Nuis
decoder. It performs no token classification, diagnostic, stage, or
replacement decision. The Rust host independently recomputes the same stage
folds and token summary before it can materialize a candidate handoff.

`StdCompilerTokens` recognizes the exact `nuis-token-stream-v1` header, all
seven record kinds, tabs and LF boundaries, lowercase even-length hex payload
shape, and signed/unsigned decimal payload shape. It is bounded to 4 MiB and
65,535 records. Its output is a record count plus semantic fold over decoded
hex nibbles, decimal units, and record kinds. Complete UTF-8/numeric value
materialization and canonical token re-emission remain later work.

This split is deliberate:

```text
host adapter = byte transport and scalar-state orchestration
Nuis image = candidate stage folds and token decoding
shared artifact layer = identity and authority verification
```

The adapter is rebuilt without the ordinary process `main`; the normal Nuis
runtime shim remains otherwise unchanged. Its exact binary length and SHA-256
are part of the production proof.

## Promotion

The candidate handoff preserves the producer-neutral semantic bundle while
changing the auditable producer identity to
`nuis-stage1-token-decoder-v2`. The promoted component keeps the same
component identity, native output, dependency closure, and five stage payloads,
but declares the explicit `stage1-candidate` role and uses the executed Nuis
image as its compiler image.

The production proof binds both components, the earlier execution proof, the
candidate image, all five byte lengths/SHA-256/folds, the bundle fold, and the
adapter. V2 additionally binds `nuis-compiler-token-decoder-v1`, its token
record count, and semantic fold. `bootstrap-diff` verifies this proof before
writing its report. Changing the adapter, token summary, a stage payload, role,
producer, component record, or proof therefore fails closed.

## Current Limit

The first producer now performs a real, bounded token-decoding step while
preserving the canonical five-stage bytes required by the current 13/13 gate.
It does not yet materialize complete token values, re-emit a token stream, or
decode AST/NIR bodies. Its reproducible identity remains proven across two
empty, compile-cache-bypassed roots by `nuis bootstrap-reproducibility`; see
[Nuis Compiler Component Reproducibility](nuis-compiler-component-reproducibility.md).
The next compiler step is complete token value materialization or one bounded
AST/NIR structural-body decoder behind the same ABI.

Replacement authorization remains a separate future protocol with rollback
evidence. A `13/13` report is evidence of equivalence, not permission to switch
the active compiler.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_production -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc --lib exact_scalar_candidate_export_is_allowed_but_symbol_spoofing_is_rejected -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate -j 1
```
