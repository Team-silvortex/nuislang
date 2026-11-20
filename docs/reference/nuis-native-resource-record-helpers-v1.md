# Native Resource Record Helpers V1

This beta-0.19.0 worktree follow-up closes the `measure(width).value` boundary
recorded in the [inferred-shadowing proof](nuis-native-resource-inferred-shadowing-v1.md).
The resource coordinate remains active/95. Frame/window and provider milestones
do not advance with this bounded native CPU change.

## Admission And Transport

Mixed/checked resource callbacks can call pure helpers with `i64` value
parameters and flat records containing 1..=64 `i64` fields. Declared field names
and order are preserved; the shape is not a fixed set of precombined arities.
Calls may compose through other admitted value-record helpers.

The [resource adapter](../../crates/yir-lower-llvm/src/native_resource_session/admission/value_helpers.rs)
reuses the existing scalar/value-record helper validator and layout contract.
It independently checks callee identity, arity, parameter ownership, result
layout, ready operands, declared nodes, CPU lanes and dependency boundaries.
Only previously verified immutable contract metadata is exempt from executable
ingress checks; those nodes and edges remain in the selected module.

Resource and value helpers share the same root-inclusive depth-8 and 32-function
closure limits, cycle rejection and unique 256-node per-function budget. Value
helpers gain no Buffer/Bytes parameters, external effects, general calls or
loop-work grant. They never consume or alias the caller's live Bytes set.

The emitter reuses private LLVM value transport for these selected helpers,
not an owned heap packet. Their record returns and field projections therefore
need no aggregate allocation, require or drop operation. This is not a new
public ABI: mixed/checked resource returns retain their existing ownership,
allocation-failure and publication contracts. Ordinary module emission and the
separate scalar-session profile keep their previous selection policies.

## Native Evidence

The [regression suite](../../tools/nuisc/tests/native_resource_session/record_helpers.rs)
executes a two-helper chain while the caller holds a Bytes snapshot. Across
120 events, early returns and shared continuations in both arms match an
independent pixel/scalar oracle and leave zero tracked live blobs.

Further checks cover Buffer reads evaluated before subsequent stores, skipped
branches, multi-field declaration/construction order, 1/2/17/64-field records,
rejection at 65 fields, call depth and function
count boundaries, cycles, malformed signatures/layouts, resource operands,
undeclared/cross-lane nodes, executable metadata and lifetime ingress.
Faulting resource result allocation on all four paths preserves both previously
accepted State halves, releases Bytes and stays failed after successful close.
The pure value helpers themselves contain no owned-aggregate allocation path.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
now uses `measure(width, height).words` in both continuing arms while holding
its snapshot. The five-event delivery witness still expects scalar State
`[3, 2, 5, 2]`, a 3x2 image with red, green and four black pixels, cache reuse,
source-free NSB relocation and detached launch with empty PATH.

## Validation

Run from the repository root, sequentially:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_direct_eager_pair --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation for this follow-up:

- All 82 resource-session tests pass, including seven new record-helper tests
  and the previous negative helper witness now checked as an admitted case.
- All five resource delivery tests and both ordinary direct record-return
  native/reference bridge tests pass.
- All 32 enabled native-scalar lowering tests and the cached resource-closure
  depth regression pass. The normally ignored host LLVM typed-record transport
  test was explicitly run and also passes.
- All 50 tensor tests, 43 maintenance tests and both file-line checks pass.
- Architecture, UTF-8, local documentation links, host-absolute-path policy,
  scoped Rust formatting and diff checks pass.
- The ordinary CLI reports 4574 drift checks with zero failures. The resource
  coordinate stays active/95; the other 36 complete coordinates retain their
  pre-change content hash. No release version or Git commit is changed here.

## Remaining Boundaries

At this checkpoint the next reproduced case was `measure0(width).value.count`,
with a nested scalar record. The subsequent
[nested-record proof](nuis-native-resource-nested-records-v1.md) closes the i64-only
case using the same value transport and recursive whole-record projection checks.
The validation counts above remain this flat-record checkpoint's receipt.

Non-i64 captures, resource-changing joins, arbitrary resource helpers/loops,
resize/cancellation, provider completion and native window integration remain
separate. This is local host evidence, not fresh Linux/GPU parity, full-workspace
acceptance, a performance comparison, pure Nsld or an all-Nuis runtime. Clang
finalization, the allocation compatibility ABI and host State copies remain.
