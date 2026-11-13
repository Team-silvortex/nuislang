# Native Guarded Resource Return V1

This beta-0.18.9 worktree follow-up extends the
[checked-State profile](nuis-native-checked-resource-state-v1.md) with early
`Ok(State)` returns that create resources only on the selected path. The resource
coordinate advances to active/85; the other 36 coordinates, including native
frame dispatch at early/10, are unchanged. This is not a release bump.

Historical checkpoint: the later [retained-Bytes proof](nuis-native-retained-bytes-return-v1.md)
advances caller-owned cleanup across the guard to active/90. The no-live-caller
restriction and validation counts below describe this earlier checkpoint.

## Contract And Lowering

The [guarded canvas](../../examples/projects/domains/native_guarded_canvas_session_demo)
uses a zero-valued event as a heartbeat. Nuis returns fresh scalar/pixel State
early, increments its event count and keeps the cursor unchanged. Other events
paint normally. Close also exercises an early owned return. Configuration errors
retain the separate checked Result error channel and never publish failed State.

The source adapter outlines a straight-line, allocation-containing early-return
arm of a registered host entry into an ordinary YIR helper. The common CPU
`guard_call_owned_struct_return` instruction records condition, helper identity,
exact owned return layout and ready captures. The compiler does not embed branch
source strings, introduce a synthetic loop or move resource creation before the
condition. The helper receives one borrowed Buffer and i64 values; no caller-owned
resource is captured. Explicit typed i64 locals may join those captures.

LLVM emits a real conditional call. The selected helper performs its calculations,
copies, writes and drops in source order, packs the Result once and returns its
owned aggregate directly. The caller forwards it unchanged, including allocation
failure. The unselected path does not call the helper. Continuation nodes remain
ordered after the guard. GLM records the Buffer borrow/write and its lifetime edge.
This instruction requires native function execution; the diagnostic interpreter
rejects a taken call rather than fabricating a successful Result.

## Ownership And Admission

Resource admission follows the static helper closure and checks every body, not
just the public callback roots. The closure is acyclic, at most 32 functions and
eight active levels; the existing 256-node/eight-copy per-function limits remain.
Every helper must preserve the exact Result layout, parameter declarations,
resource lane and contract dependencies. Missing helpers, recursion, capture or
layout drift and cross-lane ordering/lifetime edges fail closed.

At each guarded call the caller must have no live owned resource. Each helper
independently proves that its Bytes are dropped or transferred through State and
Ok into its terminal aggregate. Branch-local temporaries can be explicitly
dropped; moved or dropped resources cannot be reused. This proves both the taken
exit and the fallthrough without allocating both alternatives or inventing
cleanup for an owner that belongs to a different branch.

## Validation

Native tests alternate two early Ok arms and the ordinary tail for 120 events,
checking complete scalar/pixel State and zero retained Bytes after each callback.
One arm copies a snapshot before mutating the borrowed buffer: publication must
retain the earlier snapshot. Another creates and drops a separate temporary.
The normal path would divide by zero if an unselected arm were evaluated.

Fault injection separately fails the selected final Bytes allocation and the
selected aggregate allocation. Unselected faulty helpers do not affect execution;
selected failure preserves the previous State, retains failed completion through
close and leaves no retained Bytes. This is not a process-wide leak proof.
Malformed helper identity, recursive calls, layouts, captures, leaked helper
owners and live caller owners are rejection tests. YIR serialization retains the
helper table and guard contract.

The artifact workflow checks exact final PPM bytes, build-cache reuse, detached
binary execution with empty PATH after removing source/build files, and source-free
NSB materialization. The registration and callback ABI are unchanged.

Local validation: 31 resource-session tests and five artifact workflows pass.
CPU domain tests report 112 passed; LLVM lowering reports 184 passed and one
previously ignored test. All 50 tensor/mainline tests, two file-budget tests and
40 repository script tests pass. Documentation links, UTF-8, host-path policy and
workspace architecture checks pass. All 4545 tensor drift checks are clean; the
other 36 coordinate records retain SHA-256
`551021f2e040fc6a2e71e3e3237c9e0fd69329d5901e8b625f9b4eb14ff9f66d`.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_resource_session_workflow --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-domain-cpu --lib --locked -j1 -- guarded_owned_call --test-threads=1
```

## Remaining Boundaries

The source extension is bounded to straight-line allocation-containing arms of
registered host entries, with an empty else and a terminal owned return. Arbitrary
nested owned branches, branch joins and cross-branch caller-owned resource capture
remain unsupported. This does not grant general resource helper calls, resource
loops, resizing, cancellation or provider/device completion. State is still flat
i64 fields plus one Bytes. Native traps are not recoverable application errors.
No fresh Linux/GPU parity, performance claim, native window integration, pure
Nsld finalization or all-Nuis runtime is established by this local CPU proof.
