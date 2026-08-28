# Nuis Compiler Candidate Production

`nuis-compiler-candidate-production-v4` is the attested path from an executed
Nuis compiler-shaped program to a separately identified `stage1-candidate`
leaf. V4 retains the materialized canonical token page and adds the first
Nuis-owned AST structural page, including its partial-line continuation state.
The machine-readable contract is
[nuis-compiler-candidate-production-v4.toml](nuis-compiler-candidate-production-v4.toml).

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

The Nuis source exports fourteen exact scalar functions. Thirteen preserve the
stage, bundle, complete-token-stream DFA, and canonical token-page ABI from v3.
The fourteenth accepts an AST byte length plus nineteen `i64` words and returns
the identity of its first 128-byte structural page. Bootstrap subset v5 accepts
only these exact function names, symbols, parameter counts, and all-`i64`
signatures. Arbitrary exports continue to fail as `NBS004`.

The generated host adapter opens the five verified payload files and passes
every byte, in order, through the Nuis fold functions. For the token payload it
also maintains scalar DFA state and blindly packs at most 128 bytes into
nineteen seven-byte little-endian words. It packs the AST prefix through the
same generic byte transport, without recognizing documentation, imports,
module headers, indentation, or record kinds. It does not decode payloads,
canonicalize values, compute either page identity, or make authority decisions.

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

For AST, `StdCompilerProjection` scans the first 128 raw bytes in deterministic
fixed chunks. It commits complete documentation, import, and module-header
records, then retains the unfinished next line as continuation state. The real
candidate page is pinned independently by both implementations:

```text
complete records = 3
page bytes = 128
projection hash = 65460735
continuation indentation = 0
continuation body bytes = 2
continuation body hash = 28497819
state hash = 1349056749
identity = 1349056749 * 129 + 128 = 174028320749
```

Before the native page path was admitted, it exposed a general LLVM bug where
aggregate-valued `cpu.guard_return` nodes discarded their early return. The
backend now emits a real branch and returns the packed structure through the
owned aggregate ABI; the production integration test exercises that path.

```text
host adapter = opaque token/AST byte transport and scalar-state orchestration
Nuis image = folds, token materialization/emission, AST page state and identity
artifact layer = independent token/AST decoding, verification, and authority
```

The adapter is rebuilt without the ordinary process `main`; the normal Nuis
runtime shim remains otherwise unchanged. Its exact binary length and SHA-256
are part of the production proof.

## Promotion

The candidate handoff preserves the producer-neutral semantic bundle while
changing the auditable producer identity to
`nuis-stage1-token-ast-materializer-v4`. The promoted component keeps the same
component identity, native output, dependency closure, and five stage payloads,
but declares the explicit `stage1-candidate` role and uses the executed Nuis
image as its compiler image.

The production proof binds both components, the earlier execution proof, the
candidate image, all five byte lengths/SHA-256/folds, the bundle fold, token
count and semantic fold, canonical token-page fields, every AST page and
continuation field, and the adapter.
`bootstrap-diff` verifies this proof before writing its report. Changing the
adapter, token page, AST page, stage payload, role, producer, component record,
or proof therefore fails closed.

## Current Limit

V4 materializes one fixed-capacity token page and one fixed-size AST structural
page. It does not paginate the rest of either stream, produce a NIR page, or
transform stage bytes. The fixed eight-by-sixteen-byte execution shape remains
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
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_structural_projection_page -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_production -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc command_bootstrap -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate -j 1 -- --test-threads=1
```
