# Native Resource Inferred Shadowing V1

This beta-0.18.9 worktree follow-up extends the
[explicit scalar-shadowing proof](nuis-native-resource-scalar-shadowing-v1.md).
The resource coordinate remains active/95; frame/window and provider milestones
do not advance with this bounded compiler change.

## Shared Type Resolution

Resource callbacks admit branch-local let/const shadows whose enclosing and new
bindings both resolve to canonical `i64`, including inferred declarations and
chained/generic type aliases. Enclosing parameters and local declarations may
also use aliases or inference. Initializers read the preceding identity; selected
early returns read the local identity; shared suffixes retain their outer capture.

The [binding pass](../../tools/nuisc/src/resource_session_bindings.rs) assigns
collision-free candidate identities before declarations and assignments become
the same NIR statement. Its common `lower` entry calls the existing project-aware
semantic lowering once, including normal inference, alias visibility, generic
constraints and mutability checks. It does not implement a second type solver.
Registered source and project compilation both use this entry; the reported AST
and public callback parameters remain unchanged.

The [type gate](../../tools/nuisc/src/resource_session_bindings/types.rs) then
checks canonical NIR types before optimization or ownership cleanup. It tracks
the binding visible at each declaration, not the last occurrence of the same
name in the function. Every candidate must still exist and have two non-ref,
non-optional, non-generic `i64` types. An unused invalid shadow is rejected before
dead-code removal. Bytes, Buffer and other non-i64 shadows are not granted by
alias spelling or missing annotations. True assignments retain their identity
and remain subject to the existing captured-write check.

No YIR operation, callback ABI, runtime dispatcher or provider grant changes.
The earlier function/field-name isolation, name reservation, unknown-region
veto and empty-root no-clone path remain in place.

## Native Evidence

The inferred and alias variants run 120 events each over four paths: early
return and shared fallthrough in both arms. Complete scalar/pixel State matches
an independent oracle, and completed callbacks leave zero tracked live blobs.
Further regressions cover inferred outer locals, nested generic aliases, scalar
helpers, Buffer reads that must not be recomputed after a write, local mutation,
immutable/captured-write rejection and canonical non-i64 rejection.

Faulted suffix allocation preserves the last committed State on both continuing
edges, releases owners and remains failed after close. Independent tests check
imported aliases, declaration-time type visibility, missing functions/declarations,
ref/optional/type drift, alias cycles and invalid inference.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
uses aliased dimensions, inferred heartbeat width and aliased paint width. Its
five-event delivery witness must still produce State `[3, 2, 5, 2]`, a 3x2 image
with red, green and four black pixels, cache reuse, source-free NSB relocation
and detached launch with empty PATH.

## Validation

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- resource_session_bindings --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation for this worktree follow-up:

- All eight binding/type-admission unit tests and all 75 resource-session tests
  pass, including seven new native/admission regressions.
- All five resource-session delivery tests pass with the inferred/alias canvas.
- Both ordinary direct record-return bridge regressions pass, retaining the
  separate scalar-session baseline.
- All 50 tensor tests, both file-line-budget checks and all 43 repository
  maintenance tests pass.
- Architecture, UTF-8, documentation links, host-absolute-path policy and scoped
  Rust formatting checks pass.
- The rebuilt CLI reports 4571 drift checks, zero failures and `clean`.
  The resource coordinate remains `active/95`; all other 36 complete coordinates
  retain their pre-change content hash.

## Remaining Boundaries

At this checkpoint the next case was `measure(width).value`, where the helper
returns a scalar record: inference succeeded but native admission rejected
`call_owned_struct`. The subsequent
[record-helper proof](nuis-native-resource-record-helpers-v1.md) closes the bounded
flat-i64 case using shared validation and allocation-free value transport. The
historical test counts above remain this checkpoint's receipt; nested scalar
records and arbitrary resource helpers are not implied by that follow-up.

Expression-level control scopes, match/destructuring, loops and lambdas are not
normalized here. Non-i64 shadows, resource-changing phi joins, arbitrary layouts,
resize/cancellation and provider/device completion remain separate. Existing
root-inclusive depth-8 and 32-function budgets still apply. Clang finalization,
allocation compatibility ABI and host copies remain. No fresh Linux/GPU,
full-workspace, performance, pure Nsld or all-Nuis-runtime acceptance is claimed.
