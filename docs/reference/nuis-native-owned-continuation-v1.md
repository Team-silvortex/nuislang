# Native Owned Continuation V1

This beta-0.18.9 worktree follow-up extends the
[nested owned-return proof](nuis-native-nested-owned-return-v1.md) to nonterminal
outer arms with an empty else. The resource coordinate retains active/95 while
its evidence and next action advance. No release bump or dependent frame/window
promotion is implied; the other 36 coordinates are unchanged.

The later [paired-continuation proof](nuis-native-paired-owned-continuation-v1.md)
extends this boundary to nonempty else arms. The counts and restrictions below
remain the historical scope of this earlier receipt.

## Shared Continuation

An outer condition may now contain an owned early return without itself ending
in a source return. If the inner return is not taken, its surviving Bytes remain
available to the function's subsequent statements. The outer false path reaches
that same suffix. Only one path runs the suffix, exactly once.

The source outliner moves that suffix into one private function with explicit
borrowed Buffer, i64 and owned Bytes parameters. The selected outer helper and
the root fallthrough both end by calling this shared continuation. Nested open
arms compose into an acyclic function graph, not copied suffix trees. Branch
locals retain scope-exit cleanup; names local to the branch do not become the
continuation's captures. Rebinding an entry capture is rejected until a separate
mutable-carry contract can define the join.

The YIR terminal operation is:

```text
cpu.tail_call_owned_struct_return <callee> <owned-layout> <buffer> [i64-inputs...]
    [transfer_bytes_v1=N owner...] [cleanup_bytes_v1=N owner...]
```

Unlike the guarded call, it has no condition and no continuing caller edge.
GLM therefore records unconditional ownership consumption for transferred and
cleaned Bytes. Native admission independently proves that the two disjoint sets
exactly cover the live caller owners, validates the typed callee and verifies
its complete resource consumption. A terminal call must be the function's
declared final result. Recursive, malformed, stale-owner and signature-drift
calls are rejected. Registered State layout/export identity accepts this terminal
form without changing the public callback ABI.

LLVM preflights captures before emitting cleanup or a call, then forwards the
callee's packed result unchanged. It does not unpack/repack the Result, create
a dummy State or allocate a pending-result sentinel. Allocation failure stays
failure rather than being decoded as a successful continuation. This operation
does not promise machine tail-call optimization or unbounded recursion. The
diagnostic interpreter rejects it rather than pretending native execution.

The existing root-inclusive depth-8 and 32-function limits count both guarded
and terminal edges. Shared suffix memoization cannot hide a longer caller path.
Per-function node/copy bounds and the eight-owner edge budget remain in force.

## Execution Evidence

The native regression alternates four paths for 120 events: two inner owned
returns, outer-entry fallthrough, and outer-condition bypass. Distinct snapshots,
branch-local cleanup and moved ancestor owners are checked against exact pixels
and scalar State. A suffix counter proves once-only execution; YIR contains one
shared suffix body rather than one copy per path. Every completed callback leaves
zero tracked live blobs.

Additional native tests cover nested open arms, branch-local Bytes without an
ancestor owner, and checked suffix arithmetic skipped by a real early return.
An inner Err and a faulted continuation allocation preserve both State halves,
release owners and keep failed completion visible through close. Faulting the
shared suffix does not affect early returns that bypass it.

Rejection tests cover a consumed incoming owner, captured-value rebinding,
missing transfer/lifetime edges, borrowed helper parameters, result-layout drift,
recursive continuations and a terminal call outside its declared result boundary.
Contract and LLVM tests check ownership classification, no partial emission on
invalid input, and unchanged forwarding without a new allocation.

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo)
now rejoins after a noninitial heartbeat. Paint and heartbeat paths share the
subsequent code, while the initial heartbeat still returns early. The artifact
workflow checks exact scalar State and PPM bytes through cache reuse, source-free
NSB relocation and detached binary execution with empty PATH.

## Validation

Local host validation passed:

- 54 native resource-session tests, including the seven shared-continuation tests.
- Five artifact workflows covering cache reuse, source-free NSB relocation and
  detached binary execution.
- YIR core: 99 tests; CPU domain: 115 tests; LLVM lowering: 187 tests, with one
  existing opt-in host-clang test ignored.
- Two ordinary direct-record return regressions and two file-size policy tests.
- 50 tensor tests and 4562 clean drift checks. The resource coordinate remains
  active/95; the other 36 coordinates are unchanged.
- 40 maintenance-script tests, workspace architecture, UTF-8, documentation links
  and host-absolute-path checks. Formatting and diff whitespace checks also pass.

This is targeted validation, not a full-workspace or fresh remote-device run.
The resource/runtime tests count tracked owned blobs, not all process allocations.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-core -p yir-domain-cpu -p yir-lower-llvm --lib --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_direct_eager_pair --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor_ --test-threads=1
```

## Remaining Boundaries

Nonempty else joins, rebinding captured values, resource-changing phi joins and
general resource loops/helpers remain separate. The next concrete task is a
two-sided conditional sharing one continuation while preserving incoming Bytes
on both surviving paths. This receipt covers fixed-extent synchronous State with
1..=16 i64 fields and one Bytes, not resizing, cancellation or provider completion.
Native traps remain process failures, not recoverable Result errors.

Static host finalization still uses clang and the allocation compatibility ABI;
host/state transport still copies. No whole-application zero-copy, performance,
fresh Linux/GPU parity, native ns-nova window integration, pure Nsld finalization
or all-Nuis runtime claim is made.
