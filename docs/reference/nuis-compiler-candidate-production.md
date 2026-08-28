# Nuis Compiler Candidate Production

`nuis-compiler-candidate-production-v7` is the attested path from an executed
Nuis compiler-shaped program to a separately identified `stage1-candidate`
leaf. V7 retains the canonical token page and independently resumes both AST
and NIR into a second structural page through serialized opaque cursors, then
binds the first Nuis-produced non-identity NIR checkpoint transformation.
The machine-readable contract is
[nuis-compiler-candidate-production-v7.toml](nuis-compiler-candidate-production-v7.toml).

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
  scalar adapter, stage-transformation manifest, and production proof
* `nuis.compiler-component-diff.toml`, which must report all thirteen
  comparisons equivalent while retaining `replacement_authorized = false`

## Producer Boundary

The Nuis source exports sixteen exact scalar functions. The first fifteen
preserve the stage, bundle, complete-token-stream DFA, canonical token page,
and first AST/NIR page ABI from v5. The sixteenth is projection-generic: it
accepts a selector, projection kind, eight opaque cursor lanes, a byte length,
and nineteen packed `i64` words. It returns the page identity, one resulting
cursor lane, or cursor identity. Bootstrap subset v7 accepts only these exact
function names, symbols, parameter counts, and all-`i64` signatures. Arbitrary
exports continue to fail as `NBS004`.

The generated host adapter opens the five verified payload files and passes
every byte, in order, through the Nuis fold functions. For the token payload it
also maintains scalar DFA state and blindly packs at most 128 bytes into
nineteen seven-byte little-endian words. It packs the first two AST and NIR
pages through the same generic byte transport, without recognizing
documentation, imports, module headers, indentation, or record kinds. For each
call it only supplies raw words and transports eight returned cursor lanes into
the next call. V7 also prints both NIR cursor lane arrays in exact order so the
Nuis output can become transformation evidence. It does not decode payloads,
canonicalize values, compute any page or cursor identity, or make authority
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

The same `StdCompilerProjection` state machine scans the first NIR page. The
real payload starts with four complete imports and an unfinished fifth record:

```text
complete records = 4
page bytes = 128
projection hash = 568515310
continuation indentation = 0
continuation body bytes = 25
continuation body hash = 671013644
state hash = 1026894471
identity = 1026894471 * 129 + 128 = 132469386887
```

After the first page, Nuis serializes eight state lanes under
`nuis-compiler-structural-cursor-v1`. The lanes retain only the generic scanner
state needed to resume: consumed-byte metadata and flags, previous depth,
cumulative record count and hash, indentation, unfinished body length/hash,
and bounded line-prefix state. The adapter cannot interpret them. It sends the
same lanes back with the next 128 raw bytes, and Nuis validates the cursor
before scanning the continuation.

The real candidate pins both page chains:

```text
AST first cursor identity = 1136712771
AST second page identity = 149528711957
AST second cursor identity = 1472919348
NIR first cursor identity = 754343074
NIR second page identity = 146705724977
NIR second cursor identity = 38998897
```

The artifact layer independently reconstructs both pages and both cursors from
the complete ordinal-two and ordinal-three payload bytes before the producer
can attest them. AST and NIR therefore share one Nuis implementation, one
producer-neutral page contract, and one resumable cursor contract while
remaining separate projection domains.

## Stage Transformation

The Nuis-produced NIR identities and both eight-lane cursors are encoded as a
22-word `nuis-compiler-structural-checkpoint-v1` record. This is a genuine
non-identity representation under `ordered-u64-le-v1`, not a copy of the NIR
text. The versioned
[stage-transformation protocol](nuis-compiler-stage-transformation.md) binds
the source payload and all output words.

The artifact layer independently reconstructs the same NIR pages and cursor
lanes from the original payload. Candidate production then binds the canonical
transformation manifest file length and SHA-256. Manifest word, order, payload,
hash, producer, handoff, or proof drift fails closed.

```text
host adapter = opaque token/AST/NIR byte and cursor transport
Nuis image = folds, token materialization/emission, resumable AST/NIR scanning
artifact layer = independent page-chain replay, verification, and authority
```

The adapter is rebuilt without the ordinary process `main`; the normal Nuis
runtime shim remains otherwise unchanged. Its exact binary length and SHA-256
are part of the production proof.

## Promotion

The candidate handoff preserves the producer-neutral semantic bundle while
changing the auditable producer identity to
`nuis-stage1-nir-checkpoint-materializer-v7`. The promoted
component keeps the same component identity, native output, dependency closure,
and five stage payloads, but declares the explicit `stage1-candidate` role and
uses the executed Nuis image as its compiler image.

The production proof binds both components, the earlier execution proof, the
candidate image, all five byte lengths/SHA-256/folds, the bundle fold, token
count and semantic fold, canonical token-page fields, every AST and NIR page
and first-page field, both cursor identities, both second-page identities, and
the adapter, plus the independently replayed stage-transformation manifest.
`bootstrap-diff` verifies this proof before writing its report. Changing the
adapter, token page, AST/NIR page or cursor, stage payload, role, producer, component
record, or proof therefore fails closed.

## Current Limit

V7 materializes one fixed-capacity token page and binds exactly two fixed-size
structural pages for both AST and NIR. The generic Nuis resume function can
continue again with the resulting cursor. Production now attests a non-identity
checkpoint representation, but the producer-neutral five-stage handoff still
contains unchanged NIR/YIR bytes. A future semantic differential must admit a
changed stage encoding without weakening equivalence. Production also does not
yet attest a third page. The scalar boundary remains intentional until
arbitrary aggregate loop-carried backedges have native lowering; this contract
does not claim that general loop capability.

Local reproducibility remains proven across two empty,
compile-cache-bypassed roots by `nuis bootstrap-reproducibility`. Independent
machine or attester trust and reversible replacement authorization remain
separate future protocols. A `13/13` report is evidence of equivalence, not
permission to switch the active compiler.

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_token_decoder -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_structural_projection_page -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_stage_transformation -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_production -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuisc command_bootstrap -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate -j 1 -- --test-threads=1
```
