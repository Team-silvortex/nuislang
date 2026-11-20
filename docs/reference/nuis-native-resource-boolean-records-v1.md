# Native Resource Boolean Records V1

This beta-0.19.0 worktree follow-up extends the
[nested-record proof](nuis-native-resource-nested-records-v1.md) to boolean
validation results. The resource coordinate remains active/95; frame/window
and provider completion milestones do not advance with this bounded CPU change.

## Admission And Conditions

Mixed/checked resource callbacks can read flat or nested helper results whose
leaves are `i64` or `bool`. A boolean field such as `validate(width).value.valid`
can directly drive an early checked error or owned success return. Whole-record
projection checks still include unselected siblings and reject resources.

The existing private value transport already preserves the scalar kinds. The
[helper adapter](../../crates/yir-lower-llvm/src/native_resource_session/admission/value_helpers.rs)
now retains `bool` as `Kind::Bool`, rather than widening it to an integer. No
opcode, public ABI or heap return packet is added. The 64 total leaf slots,
layout-depth bound, root-inclusive call-depth 8 and 32-function bound remain.

Source boolean negation can lower to equality with `false`. Resource admission
therefore checks `eq`/`ne` with either two bool operands or two i64 operands.
Mixed-kind operands, integer predicates and boolean ordering remain rejected.
This does not grant arbitrary short-circuit helper composition.

At this checkpoint helper parameters still required i64 values; the
[boolean-parameter follow-up](nuis-native-resource-boolean-parameters-v1.md)
extends that boundary. Mixed resource State remains flat
i64 fields plus one Bytes owner; checked error codes remain i64. Projecting a
boolean value does not authorize a boolean State field, implicit integer
conversion, non-i64 local shadow or any change to the ownership partition.

## Native Evidence

The [boolean regression suite](../../tools/nuisc/tests/native_resource_session/boolean_records.rs)
runs a five-leaf helper containing nested flags and scalar values over 120 events.
Direct predicates and negation preserve early-return and continuation paths in
both arms, independent pixel/scalar expectations and zero tracked live blobs.
A separate native witness checks all four boolean equality/inequality cases.

An error path evaluates a boolean helper after taking a Bytes snapshot and
writing the private Buffer. The application error preserves the last accepted
State and releases owners. Successful close does not erase the failure.
Faulting Result allocation on four success paths and the error path instead
reports allocation failure, without publishing State or confusing it with the
application error code.

Independent admission tests keep exact predicate/error/operand types, reject
layout drift and bool ordering/coercion, preserve non-i64 shadow rejection,
and count boolean fields against the existing 1/17/64/65-slot boundary. Unit
tests check boolean projections without granting boolean owned-State fields
or other scalar/resource leaf kinds.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
uses `validate_extent(...).extent.invalid` in both continuing arms while holding
its snapshot. Its existing five-event delivery witness expects State
`[3, 2, 5, 2]` and a 3x2 image with red, green and four black pixels after cache
reuse, source-free NSB relocation and detached launch with empty PATH.

## Validation

Run from the repository root, sequentially:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- native_resource_session --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation for this follow-up passed 97 native resource regressions,
6 resource admission unit tests, 5 delivery workflows, 50 tensor tests and
2 file-line checks. The shared scalar-session suite passed 32 tests with its
one explicitly opt-in host transport test still ignored. The maintenance
suite passed 43 tests.

The rebuilt CLI reports 4,580 drift checks with zero failures, clean coverage
and clean recursive hierarchy validation. The complete values of the other
36 tensor coordinates are unchanged. Architecture, UTF-8, local documentation
links and scoped formatting checks also passed.

## Remaining Boundaries

The next case at this checkpoint was `choose(width < 1).reject`, where `choose`
takes a bool value and returns a boolean record. The
[boolean-parameter follow-up](nuis-native-resource-boolean-parameters-v1.md)
now admits it with exact argument types and no resource or shadowing grants.

Other scalar result kinds, general resource helpers/loops, resource-changing
joins, observable captured writes, resize/cancellation, device retirement and
native window integration remain separate. This is local CPU evidence, not
fresh Linux/GPU parity, a performance claim, pure Nsld or an all-Nuis runtime.
Clang finalization, the allocation compatibility ABI and host State copies remain.
