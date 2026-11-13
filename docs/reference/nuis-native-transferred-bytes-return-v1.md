# Native Transferred Bytes Return V1

This beta-0.18.9 worktree follow-up extends the
[retained-Bytes cleanup proof](nuis-native-retained-bytes-return-v1.md).
The resource callback coordinate advances to active/95. The other 36 coordinates,
including native frame dispatch at early/10, are unchanged. This is not a release
bump or a claim that general resource control is complete.

Historical checkpoint: the [nested-return follow-up](nuis-native-nested-owned-return-v1.md)
subsequently admits nested terminal arms and retains active/95. The straight-line
restriction and next action below describe this receipt's original scope.

## Conditional Ownership Contract

An early `Ok(State)` can now return a Bytes owner already held by the caller.
The selected helper receives a typed owned Bytes parameter instead of allocating
a replacement copy. On the untaken edge the caller keeps that same owner and may
read, drop or transfer it later. Other caller owners may be cleaned on the taken
edge or transferred into the helper for its own reads and explicit cleanup.

`guard_call_owned_struct_return` retains its condition, helper, exact owned layout
and borrowed-Buffer/i64 inputs. An optional `transfer_bytes_v1=N` trailer precedes
the optional `cleanup_bytes_v1=N` trailer. Each nonempty trailer has a canonical
count; together they name at most eight distinct Bytes owners. Transfer order is
the helper's trailing owned-parameter order. The two sets must be disjoint from
each other, the condition and ordinary captures. Legacy cleanup-only calls remain
unchanged. Malformed, reordered, duplicated or unknown trailers fail closed.

Source outlining separates drop-only caller values from values used in the
selected straight-line helper. Cleanup extraction requires exactly one drop for
each cleanup-only owner. Captured Bytes become ordinary owned helper parameters;
shadowed bindings are rejected, not silently replaced. Native admission checks
that cleanup plus transfer exactly covers all live caller owners. It checks each
callee independently, including incoming owner initialization, parameter type,
ownership, local moves, drops and complete return consumption. That proof does
not consume the caller's surviving-edge owners. GLM records resource lifetime
accesses, not unconditional moves.

LLVM preflights typed arguments and allocation aliases before emitting any branch
or cleanup. In the selected block it releases cleanup-only owners, passes the
transferred blob pointers to the helper, then returns the helper's owned aggregate
unchanged. No raw-pointer or scalar coercion grants Bytes ownership. Result packing
continues to release transferred Bytes when aggregate allocation fails.
The public callback ABI, registration and source-free artifact identity are unchanged.

## Validation

Native regression alternates three selected helpers and the continuation for
120 events. Two snapshots are captured around a buffer mutation, so choosing the
wrong owner changes observable pixels. One branch transfers one owner and cleans
the other; another swaps their roles; a third transfers both, reads both and drops
the extra owner. The continuation retains both owners for reads and cleanup.
Every callback publishes the exact scalar/pixel State and leaves zero tracked
live blobs. Selected helpers contain no `copy_buffer_owned` instruction.

Each helper's aggregate allocation is faulted independently: unselected execution
still works, selected failure preserves both State halves and retains failed
completion through close. A helper can also consume transferred Bytes before
returning an explicit Err, without leaking or publishing its private writes.
Rejection tests cover missing/overlapping owners, stale moved identities, missing
lifetime edges, parameter type/ownership drift and a callee abandoning its input.
LLVM preflight tests additionally reject pointer coercion, physical register aliases
and mismatched or hidden parameter contracts without partial emission.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo)
heartbeat now returns `move(snapshot)` directly. Its existing workflow checks
exact PPM output, build-cache reuse, detached binary execution with empty PATH
after removing source/build files, and source-free NSB materialization.
Removing this branch-local copy is not a whole-application zero-copy or benchmark
claim: the existing host/state transport still performs its own copies.

Local validation: 39 resource-session tests and five artifact workflows pass.
Core reports 98 passed, CPU 114 passed, LLVM 186 passed and one previously ignored
test. All 50 tensor/mainline tests, two file-budget tests and 40 repository script
tests pass. Documentation links, UTF-8 and workspace architecture checks pass.
All 4553 tensor drift checks are clean. The other 36 coordinate records retain
SHA-256 `551021f2e040fc6a2e71e3e3237c9e0fd69329d5901e8b625f9b4eb14ff9f66d`.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-core -p yir-domain-cpu -p yir-lower-llvm --lib --locked -j1 -- --test-threads=1
```

## Remaining Boundaries

Source admission still targets registered host entries with straight-line early
return arms and empty else branches. Nested owned conditionals are the next
concrete boundary. General branch joins, resource loops, arbitrary resource
helpers, resizing, cancellation and provider/device completion remain separate.
State remains 1..=16 i64 fields plus one Bytes. Native traps remain process failures,
not recoverable Result errors. No fresh Linux/GPU parity, native ns-nova window
integration, pure Nsld finalization or all-Nuis runtime is established here.
