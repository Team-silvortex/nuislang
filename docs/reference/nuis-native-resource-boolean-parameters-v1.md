# Native Resource Boolean Parameters V1

This beta-0.19.0 worktree follow-up extends the
[boolean-result checkpoint](nuis-native-resource-boolean-records-v1.md) to exact
boolean helper arguments. The resource coordinate remains active/95; this
bounded CPU change does not advance frame, window or provider milestones.

## Typed Helper Calls

Mixed/checked resource callbacks can pass `i64` and `bool` values, in declaration
order, to bounded `call_owned_struct` helpers. For example,
`choose(width < 1).reject` now compiles and executes natively. Canonical bool
aliases work too. Helpers may compose mixed scalar arguments with other pure
record helpers; the result leaves remain nested `i64`/`bool` values.

The [resource adapter](../../crates/yir-lower-llvm/src/native_resource_session/admission/value_helpers.rs)
checks value ownership and each ready argument's exact declared kind, after the
shared target check validates arity. Bool parameters use the existing LLVM `i1`
signature, not an implicit conversion to `i64`. No public ABI, opcode or heap
return packet is added. The 64-argument/64-result-leaf limits, layout-depth
bound, root-inclusive call depth 8 and 32-function closure bound remain.

Mutation testing found that callback admission alone was insufficient: a
helper-to-helper call could still consume a bool through the ordinary integer
compatibility accessor. The [shared call preflight](../../crates/yir-lower-llvm/src/call_lowering.rs)
now requires exact scalar kinds for native value-return helpers before emitting
the call. This applies to the existing scalar-session value helpers too;
ordinary non-native-value compatibility calls retain their previous behavior.
The scalar-kind matrix tests both this strict boundary and retained compatibility.

This does not grant resource parameters, other scalar parameter kinds in the
resource profile, bool owned-State fields or non-i64 shadowing. Bool captures
across resource-return branches were excluded at this checkpoint; the
[capture follow-up](nuis-native-resource-boolean-captures-v1.md) extends that scope.

## Native Evidence

The [parameter regressions](../../tools/nuisc/tests/native_resource_session/boolean_parameters.rs)
exercise mixed bool/i64 helper-to-helper calls over 120 events, covering four
early-return/continuation paths with independent pixel and scalar expectations.
No owned return allocation is emitted inside the pure helpers, and tracked live
blobs return to zero. Separate cases execute direct comparison arguments with a
bool alias for both true and false, and preserve Buffer reads before later writes.

An application error after a private Buffer write preserves both parts of the
last accepted State and releases held Bytes. Injected Result allocation failure
on the error path and four success paths reports allocation failure instead of
an application code. Successful close does not erase the earlier failure.

Rejection tests cover parameter ownership, unsupported types, parameter-node
declaration drift, arity, unavailable operands, buffer operands, integer/bool
substitution and helper-internal coercion. Counts 1, 17 and 64 remain admitted;
65 boolean arguments are rejected by the shared function bound.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
passes its computed extent mismatch into `validate_extent(invalid: bool)` in
both continuing arms while retaining its snapshot. The existing delivery test
checks five events, State `[3, 2, 5, 2]`, and red/green/four-black pixels in a 3x2
image through cache reuse, source-free NSB relocation and an empty-PATH detached
launch.

## Validation

Run from the repository root, sequentially:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation passed 105 native resource regressions, 194 LLVM lowering unit
tests, 5 delivery workflows, 50 tensor tests, 2 file-line checks and 43
maintenance tests. The LLVM suite's one explicitly opt-in host transport test
remains ignored; this receipt does not claim that test was run.

The rebuilt CLI reports 4,584 drift checks with zero failures, clean inventory
coverage and clean recursive hierarchy validation. The other 36 complete tensor
coordinates are unchanged. Workspace architecture (29 packages, 6 domains,
74 normal/build edges), UTF-8, 5,122 local documentation links, scoped formatting
and diff whitespace checks also pass.

## Remaining Boundaries

An immutable bool stored before an outlined branch was the next gap at this checkpoint:

```text
let saved: Bytes = copy_bytes(pixels);
let reject: bool = choose(width < 1).reject;
if mode == 0 {
  if reject { return Result.Err(8); }
}
return Result.Ok(Canvas { width: width, pixels: move(saved) });
```

Source lowering then reported an unbound `reject` variable. The
[capture follow-up](nuis-native-resource-boolean-captures-v1.md) now preserves it
through resource-return helpers with declaration identities, owner partitions,
the shared root continuation and the existing scalar-shadowing boundary intact.

Broader State layouts, captured writes, general resource loops, changing resource
joins, resize/cancellation and provider/device retirement remain separate. This
is local CPU evidence, not fresh Linux/GPU parity, a performance result, pure
Nsld finalization or an all-Nuis runtime. Clang finalization, the allocation
compatibility ABI and host State copies remain.
