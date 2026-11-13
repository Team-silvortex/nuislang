# Native Retained Bytes Return V1

This beta-0.18.9 worktree follow-up extends the
[guarded resource return checkpoint](nuis-native-guarded-resource-return-v1.md).
The resource callback coordinate advances to active/90. The other 36 coordinates,
including native frame dispatch at early/10, are unchanged. No release bump or
whole-application completion is implied.

Historical checkpoint: the [transferred-Bytes proof](nuis-native-transferred-bytes-return-v1.md)
subsequently advances selected-arm owner capture/transfer to active/95. The
cleanup-only boundary and validation counts below describe this earlier step.

## Taken Edge And Continuation

A registered resource callback can hold Bytes before a straight-line early
`Ok(State)` or `Err(code)` return. The taken edge releases those caller owners
before invoking the selected helper. The continuation keeps the same owners and
may read, drop or transfer them into its eventual State. Neither the helper nor
its cleanup runs when the guard is false.

The common `guard_call_owned_struct_return` contract retains its original
condition, helper, exact owned layout and scalar/borrowed-Buffer captures. An
optional `cleanup_bytes_v1=N` trailer names 1..=8 caller Bytes owners. Legacy
calls without a trailer remain valid only when no caller owner needs cleanup.
Canonical extent, uniqueness and separation from captures are checked centrally.
All cleanup values are explicit dependencies with GLM lifetime/write access;
they are not unconditional consumes on the surviving edge.

Source cleanup runs before helper extraction. Both explicit drops and automatic
return-tail drops are transferred to the taken-edge contract only when every
caller owner is dropped exactly once and is otherwise unused throughout the
helper. Shadowing, duplicate drops and owner reads/transfers within the selected
arm are not silently rewritten. The source tracker proposes names; admission
independently resolves and verifies the exact live SSA owner set, including moved
identities. It leaves that set intact while checking the continuation.

LLVM emits cleanup inside the selected block, then calls the independently
verified helper and returns its complete owned aggregate. Cleanup precedes
helper allocation, so a selected allocation failure cannot retain the caller's
Bytes. Scalar/pixel publication, the public callback ABI, registration, exports
and NSB identity checks are unchanged.

## Validation

The retained-Bytes native regression runs 120 events alternating implicit early
cleanup, explicit early cleanup and continuation transfer. Two pre-guard owners,
one moved binding, post-guard Bytes reads and snapshot-before-buffer-write
publication are checked, with zero tracked live blobs after each callback.
An early Err with live owners preserves prior State and remains failed after close.

Selected Bytes allocation failure, Ok aggregate allocation failure and Err
aggregate allocation failure preserve both State halves and release the caller
owners. The same faulty helpers do not affect an unselected continuation.
This is targeted allocation accounting, not a process-wide memory-safety proof.

YIR round-trip and rejection tests cover omitted, incomplete, duplicate, stale,
capture-aliasing and unknown owner lists, missing lifetime edges, a leaking
continuation, duplicate source drops, shadowing and selected-arm owner capture.
The guarded canvas now holds a snapshot before the heartbeat branch and reads
and drops it only on continuation. Its artifact workflow exercises build-cache
reuse, exact pixels, source-free NSB relocation and detached binary launch with
empty PATH.

Local validation: all 35 resource-session tests and five artifact workflows pass.
Core contract tests report 97 passed, CPU 113 passed, and LLVM 184 passed with one
previously ignored test. All 50 tensor/mainline tests, two file-budget tests and
40 repository script tests pass. Documentation links, UTF-8, host-path policy and
workspace architecture checks pass. All 4549 tensor drift checks are clean; the
other 36 coordinate records retain SHA-256
`551021f2e040fc6a2e71e3e3237c9e0fd69329d5901e8b625f9b4eb14ff9f66d`.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-core --lib --locked -j1 -- guarded_owned_call --test-threads=1
```

## Remaining Boundaries

This is cleanup on the selected return edge, not transfer of caller Bytes into
that selected result. The latter is the next concrete snapshot/heartbeat case.
The source shape remains registered host entries with straight-line return arms
and an empty else; nested owned control, branch joins, resource loops, arbitrary
owned helper captures, resizing and cancellation remain outside this proof.
State remains 1..=16 i64 fields plus one Bytes. Native traps are process failures,
not recoverable Result errors. No fresh Linux/GPU execution, native ns-nova window
integration, provider/device retirement, performance claim, pure Nsld finalization
or all-Nuis runtime is established here.
