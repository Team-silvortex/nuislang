# Native Resource Nested Records V1

This beta-0.19.0 worktree follow-up closes the nested i64 projection boundary
from the [flat record-helper proof](nuis-native-resource-record-helpers-v1.md).
The resource coordinate remains active/95; native frame/window and device
completion milestones do not advance with this bounded CPU change.

## Admission And Ownership

Mixed/checked resource callbacks can read nested helper results, including
`measure(width).value.count`, retain a pure nested subrecord locally, and reuse
its fields. Every leaf must still be `i64`; helper parameters remain i64 values.
No Buffer/Bytes parameter or resource-containing helper result is admitted.

The [helper adapter](../../crates/yir-lower-llvm/src/native_resource_session/admission/value_helpers.rs)
reconstructs nested kinds from the existing checked layout. The shared contract
still limits the complete result to 1..=64 scalar leaves, not 64 per subtree,
and retains the layout parser's maximum depth of 64. Nominal child types,
declared field order, duplicate/empty fields and call signatures remain checked.
This record-depth bound is distinct from the existing root-inclusive call-depth
bound of 8, 32 reachable functions and 256 unique nodes per function.

The [projection check](../../crates/yir-lower-llvm/src/native_resource_session/mixed_admission.rs)
requires the source record to be ready and the requested immediate field to
exist. It recursively checks the entire record, including unselected siblings.
A scalar projection cannot hide a Bytes/Buffer owner in a different subtree.
Each projection leaves the caller's ownership set unchanged. Resource-bearing
State construction and transfer still use the existing separate checks.

Nested values reuse the existing private LLVM value transport. There is no new
YIR instruction, public callback ABI, allocation helper or nested-record runtime.
Pure helper results remain allocation-free; resource Result packets retain the
existing allocation, failure cleanup and atomic publication behavior.

## Native Evidence

The [nested regression suite](../../tools/nuisc/tests/native_resource_session/nested_records.rs)
runs two value helpers with a six-leaf nested result over 120 events. Early
return and fallthrough paths in both arms preserve the held Bytes snapshot,
lexical scalar captures and the shared suffix. Pixel/scalar State matches an
independent oracle and completed callbacks leave zero tracked live blobs.

Additional witnesses read every nested field, including repeated leaf names in
different branches and construction order different from declaration order.
They retain a pure nested subrecord locally and verify that a later Buffer store
does not recompute the earlier helper argument. Faulting resource Result
allocation on each of the four paths preserves both accepted State halves,
releases Bytes and stays failed after successful close.

Rejection tests cover wrong nested names/order/paths, duplicate/empty subrecords,
hidden resource leaves and the 65-leaf boundary. Twelve-level source records
compile; direct admission tests retain the common 64/65 depth boundary and reject
unready projections or resource-containing unselected siblings.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
now reads `measure(width, height).extent.words` while holding its snapshot in
both continuing arms. Its delivery witness expects State `[3, 2, 5, 2]` and a
3x2 image with red, green and four black pixels after cache reuse, source-free
NSB relocation and detached binary launch with empty PATH.

## Validation

Run from the repository root, sequentially:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- native_resource_session --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation for this follow-up:

- All 88 resource-session tests pass, including six new nested-record tests and
  the previously rejected nested projection now checked as an admitted case.
- All four resource-admission unit tests pass, including the three new recursive
  projection, isolation and depth checks.
- All 32 enabled native-scalar lowering tests pass; the separately opt-in host
  LLVM transport test was not rerun in this follow-up.
- All five resource-session delivery tests pass with the nested canvas helper.
- All 50 tensor tests, both file-line checks and all 43 repository maintenance
  tests pass.
- Architecture, UTF-8, documentation links, host-absolute-path policy, scoped
  Rust formatting and diff checks pass.
- The rebuilt CLI reports 4577 clean drift checks and no coverage or hierarchy
  gaps. The resource coordinate remains active/95; the other 36 complete
  coordinates retain their pre-change content hash.

## Remaining Boundaries

At this checkpoint the next reproduced case was `if validate(width).value.valid`,
with a nested boolean validation flag while the callback holds Bytes. The
[boolean-record follow-up](nuis-native-resource-boolean-records-v1.md) admits that
result/predicate composition while retaining strict types, i64-only shadows and
the resource boundary. The counts above remain this i64-only checkpoint's receipt.

General resource helpers/loops, resource-changing joins, observable captured
writes, resize/cancellation, device retirement and native window integration
remain separate. This is local CPU evidence, not new Linux/GPU parity or a
performance claim. Clang finalization, allocation compatibility and host State
copies remain; pure Nsld and an all-Nuis runtime are not established here.
