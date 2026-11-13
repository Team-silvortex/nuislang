# Native Nested Owned Return V1

This beta-0.18.9 worktree follow-up extends the
[transferred-Bytes proof](nuis-native-transferred-bytes-return-v1.md) to nested
terminal conditional regions. The resource coordinate remains active/95: evidence
and the next action advance, but nonterminal resource joins are still open. The
other 36 coordinates and release version are unchanged; frame dispatch remains
early/10. This is not a whole-application or GPU acceptance receipt.

Historical checkpoint: the [shared-continuation follow-up](nuis-native-owned-continuation-v1.md)
subsequently admits empty-else nonterminal outer joins. The rejection and next
task described below retain this earlier receipt's scope.

## Terminal Region Contract

A selected arm may contain nested empty-else conditions whose selected arms also
terminate, followed by its own fallback return. The innermost return forwards its
owned Result directly through the helper chain. Outer fallback effects and
allocations execute only when their inner conditions are not taken.

Source outlining now visits generated helpers through a bounded queue. Each
helper reconstructs incoming ownership from its typed Bytes parameters. A region
that still contains nested conditions receives all live ancestor owners rather
than moving path-local drops ahead of those conditions. Each leaf independently
partitions transfer and cleanup; the existing native SSA admission proves exact
live-owner coverage and complete callee consumption. Surviving edges keep their
owners for subsequent reads, moves and cleanup.

The shared contract permits root-inclusive depth 8 and at most 32 functions in
the callback closure. Source outlining checks these bounds before generating an
oversized helper graph; native admission independently rechecks serialized YIR.
Existing per-function node/copy budgets, eight-owner edge budget and cycle
rejection remain. No new instruction, callback ABI or backend-specific branch
implementation is introduced: this uses `guard_call_owned_struct_return` and its
existing `transfer_bytes_v1` / `cleanup_bytes_v1` ownership contract.

## Execution Evidence

The nested native regression runs 120 events across four outcomes in a three-level
condition tree. It captures distinct root, outer and inner snapshots around
buffer mutations and passes a locally computed i64 into the nested helper.
Each return selects the correct snapshot, publishes the exact
scalar/pixel State and leaves zero tracked blobs after the callback. The root
continuation still reads and cleans its retained owner. A separate test creates
Bytes only inside nested arms, without an incoming ancestor owner.

Each level's aggregate allocation is faulted independently. A deeper return
bypasses the faulty parent fallback, while selecting that fallback fails without
publishing either State half. The failure remains visible after close. An inner
Err likewise releases all ancestor owners and retains the previous accepted
State, despite private buffer writes.

Rejection tests remove lifetime edges at every level, change owned helper
parameters to borrowed, abandon a transferred owner, and reuse an owner dropped
before the inner guard. Depth 8 and 32-function closures are admitted; depth 9
and a 33rd function are rejected. A nonterminal outer arm is explicitly rejected
instead of being miscompiled as an unconditional returning helper.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo)
now has a nested heartbeat: cursor zero returns from the inner arm, later
heartbeats use the outer fallback, and paint events use the root continuation.
The artifact workflow checks exact final PPM bytes and scalar State across build,
cache reuse, source-free NSB relocation and detached binary launch with empty PATH.

## Validation

Local host validation passes all 47 resource-session tests (eight added here)
and five artifact workflows. Core reports 98 passed, CPU 114 passed, and LLVM
186 passed with one previously ignored test. All 50 tensor/mainline tests, two
file-budget tests and 40 repository script tests pass. Documentation links,
UTF-8, host-path policy, workspace architecture and formatting checks pass.
All 4558 tensor drift checks are clean. The other 36 coordinate records retain
SHA-256 `551021f2e040fc6a2e71e3e3237c9e0fd69329d5901e8b625f9b4eb14ff9f66d`.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-core -p yir-domain-cpu -p yir-lower-llvm --lib --locked -j1 -- --test-threads=1
```

## Remaining Boundaries

The admitted source regions terminate on every selected outer path. An outer
arm that can rejoin the root continuation after a nested owned return remains
the next concrete task. Nonempty else regions, general resource loops/helpers,
resizing, cancellation and provider/device completion are not established here.
State remains 1..=16 i64 fields plus one Bytes owner, with fixed-extent synchronous
callbacks. Native traps remain process failures, not recoverable Result errors.

Host/state transport still copies; eliminating a branch-local replacement copy
is not a zero-copy or performance claim. Static host finalization still uses
clang and the allocation compatibility ABI. No fresh Linux/GPU parity, native
ns-nova window integration, pure Nsld finalization or all-Nuis runtime is claimed.
