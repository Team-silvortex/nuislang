# Nuis Compiler Candidate Execution

`nuis-compiler-candidate-execution-v1` records the first native execution of a
compiler-shaped program written in Nuis. Its machine-readable contract is
[nuis-compiler-candidate-execution-v1.toml](nuis-compiler-candidate-execution-v1.toml).

This record remains a candidate **probe**, not a stage1 compiler-component
record. Its authority is fixed to `execution-only-no-component-production`.
The separate candidate-production protocol may consume this proof, but cannot
edit or widen its authority.

## Frontdoor

Build, execute, and attest a bootstrap-constrained project with:

```bash
nuis bootstrap-candidate-probe path/to/project path/to/output
```

The command first runs the ordinary `bootstrap-build` path. It therefore
retains the frozen subset check, normal semantic/NIR/YIR/LLVM pipeline, native
link, complete dependency closure, stage handoff, component record, and clean
diagnostic proof. It then executes the emitted native image with:

* no arguments
* closed stdin
* captured stdout and stderr
* required exit code `0`
* required empty stdout and stderr

Only a successful output-free process receives
`nuis.compiler-candidate-execution.toml`. A stale proof is removed before a
new execution attempt.

## Bound Identity

The canonical sidecar binds:

* exact stage0 component audit and reproducible identities
* component id, bootstrap subset, and source component role
* native image filename, byte length, and SHA-256
* argument and stdin contracts
* exit code plus stdout/stderr lengths and hashes
* a deterministic execution identity over every field above

The reader first verifies the sidecar, then re-reads the sibling component
record. That component reader recursively verifies its handoff, build
manifest, compiled artifact, and native binary. Moving the proof to another
component or changing the executable invalidates the chain.

## First Consumer

`bootstrap_structural_projection_candidate` is the first checked-in probe. It
uses `StdCompilerProjection` to consume typed AST/NIR structural record tags,
enforce hierarchy and opaque-leaf transitions, compute deterministic hashes,
and reject invalid sequences. The project crosses bootstrap-check and the
normal AOT pipeline without FFI or host effects, then exits `0` natively.

The portable Nuis surface uses stable integer protocol tags rather than a host
or language enum layout, keeping producer implementations and later ABI
revisions decoupled from the consumer.

## Honest Boundary

The proof does not relabel the source component. Its component record remains:

```text
stage_role = stage0
producer_id = nuisc-stage0-reference
```

The sidecar role `stage1-candidate-probe` proves only that a Nuis-written
candidate image executed under the frozen boundary. By itself it cannot enter
`bootstrap-diff`, emit a `stage1-candidate` component record, or authorize
replacement.

`nuis bootstrap-candidate-build` now consumes this immutable proof as one input
to `nuis-compiler-candidate-production-v10`. That successor additionally binds
the Nuis scalar producer ABI, every five-stage byte fold, the candidate
handoff/component/diagnostics, complete token pagination plus the preserved
canonical first-page identity, both AST/NIR structural-page chains and opaque
cursor identities, the lossless
derived NIR payload, its semantic differential, and the host adapter. See
[Nuis Compiler Candidate Production](nuis-compiler-candidate-production.md).

## Validation

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuis-artifact compiler_candidate_execution -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test compiler_structural_projection_candidate -j 1
```
