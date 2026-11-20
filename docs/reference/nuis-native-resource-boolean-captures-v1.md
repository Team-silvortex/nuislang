# Native Resource Boolean Captures V1

This beta-0.19.0 worktree follow-up closes the immutable local-bool capture gap
from the [boolean-parameter checkpoint](nuis-native-resource-boolean-parameters-v1.md).
The resource coordinate remains active/95. This is a bounded native CPU change,
not completion of frame dispatch, native windows or provider/device retirement.

## Capture Contract

An already evaluated bool can cross a guarded resource-return arm or its shared
terminal continuation without becoming an integer or being recomputed. For
example, a local `reject = choose(width < 1).reject` can now be used by an inner
checked return while its caller retains a Bytes snapshot.

The [outliner](../../tools/nuisc/src/lowering/guarded_owned_returns.rs) collects
canonical i64/bool local declarations. Shared semantic resolution handles
explicit types, inference and aliases before outlining. Branch-only bindings
stay in their branch and are not added to the root continuation's entry scope.
The externally registered callback parameters remain i64; their ABI and the
flat i64-plus-Bytes owned State layout do not expand with this change.

The [resource admission](../../crates/yir-lower-llvm/src/native_resource_session/admission/guarded_helpers.rs)
matches each captured operand to its declared kind and ownership. The input
segment admits Buffer followed by i64/bool values; transferred owners remain a
separate Bytes-only segment. Bool parameter nodes must agree with the function
declaration. The [native call preflight](../../crates/yir-lower-llvm/src/guarded_owned_call.rs)
uses existing exact scalar argument lowering and passes bools as LLVM `i1`.

The existing guard/tail format and transfer/cleanup trailers are unchanged.
Taken guards and terminal continuations keep their original owner partition;
an untaken guard preserves the caller's owners. No public opcode, runtime
allocation or externally registered ABI is added. The 33-input limit still
includes the Buffer and all scalar captures; depth 8, 32 functions and 8
ownership-trailer slots retain their existing bounds.

## Native Evidence

The [capture regressions](../../tools/nuisc/tests/native_resource_session/boolean_captures.rs)
run 120 events for each of inferred, explicitly typed, aliased-const and
branch-local declarations. Independent expectations check all four paired-arm
return/continuation paths, scalar/pixel State and zero tracked live blobs.
Branch-local flags are absent from the shared root continuation's parameters.

A separate witness computes a predicate from a Buffer read, writes that Buffer
inside a continuing arm, then uses the captured predicate in the suffix. Two
events exercise true and false values and prove the predicate is neither lost
nor recomputed from the changed Buffer.

An error after branch-local writes preserves the previously committed State and
releases snapshots. Injected Result allocation failures on four success paths
and an error path preserve State and remain distinct from application errors.
Successful close does not erase an earlier failure.

Mutation tests reject wrong/missing captured operands, parameter declaration or
ownership drift, scalar/owner segment substitution and exported-root bool
parameters. LLVM preflight rejects bool/integer coercion before emitting any
branch, cleanup or call, for both guard and tail forms. Observable captured
writes and, at this checkpoint, bool shadow declarations were rejected separately. With two i64
inputs already present, 30 bool captures fit and 31 exceed the shared bound.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo/README.md)
now retains `heartbeat` and `first_pixel` flags across branch outlines and the
shared suffix. Each continuing arm also names its local validation flag before
the checked error. The existing five-event delivery witness checks State
`[3, 2, 5, 2]`, exact 3x2 pixels, cache reuse, source-free NSB relocation and
detached launch with empty PATH.

## Validation

Run from the repository root, sequentially:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

Local validation passed 111 native resource regressions, 195 LLVM lowering unit
tests, 5 delivery workflows, 50 tensor tests, 2 file-line checks and 43 maintenance
tests. The LLVM suite's one explicitly opt-in host transport test remains ignored;
this receipt does not claim it was run.

The rebuilt CLI reports 4,589 drift checks with zero failures, clean inventory
coverage and clean recursive hierarchy validation. Complete values of the other
36 tensor coordinates are unchanged. Workspace architecture (29 packages,
6 domains, 74 normal/build edges), UTF-8, 5,132 local documentation links,
scoped formatting and diff whitespace checks also pass.

## Remaining Boundaries

At this checkpoint a distinct branch-local bool declaration could not shadow
an outer bool: the canonical shadow check required i64 on both sides. The later
[boolean-shadowing proof](nuis-native-resource-boolean-shadowing-v1.md) closes
that gap while preserving separate lexical identities and outer suffix values.
Escaping captured writes, mixed-kind shadows and resource shadows remain rejected.

General resource loops, broader owned-State layouts, changing resource joins,
resize/cancellation and provider/device completion remain separate. This is local
CPU evidence, not fresh Linux/GPU execution or a performance claim. Clang
finalization, allocation compatibility functions and host State copies remain;
this is neither pure Nsld finalization nor an all-Nuis runtime.
