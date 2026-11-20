# Native Resource Boolean Shadowing V1

This beta-0.19.0 worktree follow-up closes same-kind bool local shadowing after
the [capture checkpoint](nuis-native-resource-boolean-captures-v1.md). The
resource coordinate remains active/95; frame dispatch and application closure
are not promoted by this bounded native CPU change.

## Declaration Contract

Resource callbacks preserve separate lexical identities for an inner `let` or
`const` and its outer namesake. An initializer such as `let gate = !gate` reads
the enclosing binding before introducing its own binding. Nested scopes restore
each enclosing value, and the shared suffix still receives the root value.

The [binding validator](../../tools/nuisc/src/resource_session_bindings/types.rs)
now requires matching canonical i64/bool kinds on both sides. Shared semantics
resolves inference and aliases before admission; this is not a source-spelling
check or implicit coercion. Missing, reference, optional, resource and other
scalar types remain rejected, including unused declarations before optimization.
The public AST remains unchanged, and declaration identities are retained before
NIR merges declaration and assignment representations.

An inner mutable binding may be reassigned locally, then shadowed again. That is
not permission to modify an outer capture and publish the change across a join.
Immutable assignment and observable captured writes retain their separate vetoes.
Exported parameters remain i64, owned State remains 1..=16 i64 fields plus Bytes,
and private bool captures remain LLVM `i1`. No opcode, transport ABI, runtime
allocation or ownership-transfer format is added.

## Native Evidence

The [regressions](../../tools/nuisc/tests/native_resource_session/boolean_shadowing.rs)
run 120 events for each explicit, inferred, aliased and mutable/redeclared form.
An independent oracle checks four early-return/continuation paths, scalar and
pixel State, and zero tracked live blobs. AST preservation and YIR roundtrips
are checked; root continuations receive only the outer `gate` identity.

A three-level witness checks restoration of both intermediate and root values.
Errors after branch-local writes preserve previously committed State and release
the held snapshot. Injected Result allocation failures on all four paths preserve
State and release owners; a successful close does not erase a previous failure.
Compiler binding tests cover imported generic aliases and the canonical type
matrix, while native negative cases retain mixed-kind, resource, immutable-write
and observable captured-write rejection.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
uses root `valid` for color validation and same-named branch locals for snapshot
extent validation. Its existing delivery witness checks five events, exact 3x2
pixels and scalar State `[3, 2, 5, 2]` through cache reuse, source-free NSB
relocation and binary-only launch with empty PATH.

## Validation

Run from the repository root, sequentially:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- resource_session_bindings --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation passed 10 binding unit tests, 117 native resource regressions,
195 LLVM lowering tests, 5 delivery workflows, 50 tensor tests, 2 file-line checks
and 43 maintenance tests. The LLVM suite's one explicitly opt-in host transport
test remains ignored; this receipt does not claim it ran.

The rebuilt CLI reports 4,593 drift checks with zero failures, clean inventory
coverage and clean recursive hierarchy validation. Complete values of the other
36 tensor coordinates retain SHA-256
`b3e520bc09f004ae43d004c48106ee50ee202f6e21bcd4c8f8b672c59848e326`.
Workspace architecture (29 packages, 6 domains, 74 normal/build edges), UTF-8,
5,139 local documentation links, scoped formatting and diff whitespace checks
also pass. These are local host receipts, not full-workspace or cross-platform
certification.

## Remaining Boundaries

At this checkpoint short-circuit shadow initializers were the next gap. Replacing
the inner `!gate` with `!gate && choose(width > 0).value` or
`!gate || choose(width < 0).value` failed with
`resource callback does not admit call_bool` (the diagnostic quotes the opcode).
The later [short-circuit proof](nuis-native-resource-short-circuit-v1.md) closes
the pure-helper case with native execution evidence and retains selected RHS
evaluation. Conditional resource reads remain a separate boundary.

General resource loops, broader owned-State layouts, changing resource joins,
resize/cancellation and provider/device completion remain separate. Clang host
finalization, allocation compatibility functions and State copies remain. This
is neither pure Nsld finalization nor an all-Nuis runtime, and it claims no fresh
Linux/GPU execution or measured performance gain.
