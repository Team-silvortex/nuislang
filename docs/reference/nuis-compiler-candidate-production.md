# Nuis Compiler Candidate Production

`nuis-compiler-candidate-production-v3` is the attested path from an executed
Nuis compiler-shaped program to a separately identified `stage1-candidate`
leaf. V3 proves that the candidate materialized complete token values for one
bounded page and canonically re-emitted them; it no longer attests only a token
count and semantic fold. The machine-readable contract is
[nuis-compiler-candidate-production-v3.toml](nuis-compiler-candidate-production-v3.toml).

This closes one real compiler-data production loop. It does not mean that
`nuisc` is self-hosted, and it never authorizes replacing stage0.

## Frontdoor

Build the checked-in materializing candidate through both producers with:

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

The Nuis source exports thirteen exact scalar functions. Twelve preserve the
stage, bundle, and complete-token-stream DFA ABI from v2. The thirteenth accepts
a byte length plus nineteen `i64` words and returns the canonical identity of
the first four complete token records. Bootstrap subset v4 accepts only these
exact function names, symbols, parameter counts, and all-`i64` signatures.
Arbitrary exports continue to fail as `NBS004`.

The generated host adapter opens the five verified payload files and passes
every byte, in order, through the Nuis fold functions. For the token payload it
also maintains scalar DFA state and blindly packs at most 128 bytes into
nineteen seven-byte little-endian words. It does not classify records, decode
payloads, canonicalize values, compute page identity, or make authority
decisions.

Inside the Nuis image, `StdCompilerTokens` validates the stream grammar and
`CompilerTokenMaterializer` reconstructs four records into
`CompilerTokenStore`. `StdCompilerTokenEmit` then emits canonical
`nuis-token-stream-v1` bytes and computes the page hash. The reference page is
the real candidate prefix `use` / `cpu` / `StdLanguageCore` / semicolon:

```text
records = 4
payload bytes = 21
canonical bytes = 91
canonical hash = 1277127995
identity = 1277127995 * 129 + 91 = 164749511446
```

The artifact layer independently parses the complete token stream,
canonicalizes the same first page, recomputes every field, and rejects any
disagreement before candidate production.

```text
host adapter = opaque byte transport, packing, and scalar-state orchestration
Nuis image = stage folds, token semantics, materialization, emission, identity
artifact layer = independent decoding, identity verification, and authority
```

The adapter is rebuilt without the ordinary process `main`; the normal Nuis
runtime shim remains otherwise unchanged. Its exact binary length and SHA-256
are part of the production proof.

## Promotion

The candidate handoff preserves the producer-neutral semantic bundle while
changing the auditable producer identity to
`nuis-stage1-token-materializer-v3`. The promoted component keeps the same
component identity, native output, dependency closure, and five stage payloads,
but declares the explicit `stage1-candidate` role and uses the executed Nuis
image as its compiler image.

The production proof binds both components, the earlier execution proof, the
candidate image, all five byte lengths/SHA-256/folds, the bundle fold, token
count and semantic fold, canonical page fields, and the adapter.
`bootstrap-diff` verifies this proof before writing its report. Changing the
adapter, token page, stage payload, role, producer, component record, or proof
therefore fails closed.

## Current Limit

V3 materializes one fixed-capacity page: four records, 64 payload bytes, and
128 canonical bytes. It does not paginate the rest of a large token stream or
decode AST/NIR bodies. The fixed eight-by-sixteen-byte execution shape is
intentional until arbitrary aggregate loop-carried backedges have native
lowering; this contract does not claim that general loop capability.

Local reproducibility remains proven across two empty,
compile-cache-bypassed roots by `nuis bootstrap-reproducibility`. Independent
machine or attester trust and reversible replacement authorization remain
separate future protocols. A `13/13` report is evidence of equivalence, not
permission to switch the active compiler.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_token_decoder -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_production -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc command_bootstrap -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate -j 1 -- --test-threads=1
```
