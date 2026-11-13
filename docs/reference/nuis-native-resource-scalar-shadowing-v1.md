# Native Resource Scalar Shadowing V1

This beta-0.18.9 worktree follow-up extends the
[paired owned continuation](nuis-native-paired-owned-continuation-v1.md).
The resource coordinate remains active/95. It does not close native window,
provider completion or whole-application gates.

The later [inferred-shadowing proof](nuis-native-resource-inferred-shadowing-v1.md)
adds shared semantic type resolution for inferred and aliased declarations. This
receipt retains the preceding explicit-annotation scope and validation counts.

## Lexical Identity Before Outlining

Registered resource callbacks now preserve explicit `i64` let/const declarations
that shadow an enclosing explicit `i64` binding inside statement-level branches.
An initializer reads the preceding binding; later uses in that branch read the
new identity. Nested scopes restore the enclosing identity on exit. Early
returns use the selected local value, while a shared continuation captures the
original outer value. Local mutation targets the local identity, not its parent.

NIR currently represents declarations and assignments with the same `Let` form.
The [resource binding pass](../../tools/nuisc/src/resource_session_bindings.rs)
therefore runs before AST-to-NIR lowering, not by relaxing the outliner's capture
check. Both registered source compilation and project compilation use it. The
reported source AST and public callback parameters remain unchanged. Ordinary
unregistered compilation is untouched, and an empty root set borrows the AST
without cloning it.

Private names reserve parameters, module constants, later declarations and
variable uses throughout the admitted region. Function symbols, nominal types
and field labels are not rewritten. Unknown lexical regions veto normalization
for that function before any changes, rather than allowing a partial rewrite.

Observable assignments to captured outer variables remain subject to the existing
continuation rejection. Resource names are not normalized: Bytes rebinding and
ownership-changing joins retain independent admission checks. No YIR operation,
callback ABI, host runtime dispatcher or provider capability is added.

## Execution Evidence

The native regression alternates 120 events across both arms and both outcomes:
selected early return and shared fallthrough. It compares complete scalar/pixel
State with an independent model and checks zero tracked live blobs. Additional
cases cover nested and empty-else scopes, repeated declarations, local mutable
assignments, generated-name collisions, captured-write rejection, owned-shadow
rejection and errors that preserve the last committed State even after close.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
now uses branch-local `width` to validate total snapshot words. After the branch,
the canvas width must still be 3, not 6. Its five-event delivery witness expects
State `[3, 2, 5, 2]` and a 3x2 image containing red, green and four black pixels,
through cache reuse, source-free NSB relocation and detached empty-PATH launch.

## Validation

Run from the repository root with one build job and incremental caches disabled:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- resource_session_bindings --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation for this worktree follow-up:

- All four lexical-normalization unit tests and all 68 native resource-session
  tests pass, including five new execution/rejection regressions.
- All five resource-session delivery tests pass with the updated canvas source.
- Both ordinary direct record-return bridge regressions pass; their unregistered
  scalar path remains unchanged.
- All 50 tensor tests, both file-line-budget checks and all 43 repository
  maintenance tests pass.
- Architecture, UTF-8, documentation links, host-absolute-path policy and scoped
  Rust formatting checks pass.
- The tensor reports 4568 drift checks, zero failures and `clean`. The resource
  coordinate remains `active/95`; the other 36 complete coordinates match their
  pre-change content hash.

## Remaining Boundaries

This slice requires explicit `i64` annotations on both the shadow and its
enclosing binding. Inferred/aliased types should next reuse semantic type
resolution rather than duplicating inference in this adapter. Expression-level
control scopes, match/destructuring, loops and lambdas are not normalized here.
The existing root-inclusive depth-8, 32-function and native resource budgets
remain in force.

General resource helpers/loops, mutable resource phi joins, arbitrary layouts,
resize/cancellation and provider/device completion remain separate. Clang host
finalization, allocation compatibility ABI and host transport copies remain.
No fresh Linux/GPU, full-workspace, performance, pure Nsld or all-Nuis-runtime
acceptance is claimed.
