# Native Checked Resource State V1

This beta-0.18.9 worktree follow-up adds explicit Nuis configuration errors to the
[mixed-State profile](nuis-native-mixed-resource-state-v1.md). The resource tensor
coordinate advances to active/80. The other 36 coordinates remain unchanged;
native interactive frame dispatch remains early/10.

This is the original active/80 checkpoint. The subsequent
[guarded resource return proof](nuis-native-guarded-resource-return-v1.md) advances
only the resource coordinate to active/85 with branch-local early Ok execution;
the restrictions and test totals below describe this earlier checkpoint.

## Contract

The [checked canvas](../../examples/projects/domains/native_checked_canvas_session_demo)
declares `enum Result<T, E> { Ok(T), Err(E) }`. Its three callbacks return
`Result<Canvas, i64>` rather than `Canvas`. Nuis checks width before remainder or
division, and returns `Err` before constructing/copying pixel State:

```toml
resource_sessions = ["canvas open=canvas_open event=canvas_event close=canvas_close min=1 max=4096 state=Canvas result=Result"]
```

`result=Result` selects `nuis-native-checked-resource-session-v1`. The nominal
result name is registered, not hard-coded to Canvas or a renderer. The bounded
schema is exactly `Ok(value: State)` and `Err(value: i64)`, using the existing
owned variant-union YIR layout. All three return types and layouts must match.
State remains 1..=16 i64 fields plus one Bytes owner. Slots are derived from the
layout, not hard-coded canvas offsets. Registration, export names and the Result
identity survive manifest/YIR serialization, cache and NSB restoration.

Current conditional admission permits boolean comparisons and `guard_return`
of `Err` when no resource owner is live. A terminal return may be `Ok(State)` or
resource-free `Err`. This retains actual control flow: later division, mutation
and copying do not run after a taken error exit. Returning an owned State early,
merging separately allocated branch resources or leaking a pre-guard owner is
not silently accepted. The existing node/allocation bounds still apply.

## Error Transport And Ownership

The checked entry uses descriptor discriminator 3, a separate capability tag,
`nuis_checked_resource_<hex-id>_<role>_v1` symbols and an additional i64 error
output pointer. Older Bytes-only and mixed callbacks retain their typed ABIs.
Status 0 publishes a complete State; status 4 carries Nuis `Err(code)` and
publishes neither State region. Err(0), negative codes and full-width i64 codes
remain errors. Statuses 1/2/3 remain transport/allocation/malformed-result errors,
not application codes. Tags discriminate trusted static code, not authenticate it.

The host reports `native resource application error (role=open, code=1)` (or the
corresponding role/code), and the packaged entry exits with code 1, not a signal.
The host neither checks canvas width nor substitutes its own State transition.
Both borrowed input regions and staged outputs remain private to the call. Even
a Nuis write before Err cannot mutate accepted State. A failed event stays failed
after successful close; a failed close cannot be retried into success. No failed
session exports a frame.

This work also fixes inactive Bytes slots in owned variant packing: a statically
absent owner is stored as an empty scalar slot, not a null blob allocation that
poisons the entire aggregate. Runtime allocation results still go through the
checked blob setter; a runtime null must fail, never become an absent resource.
Err still needs the small owned aggregate allocation. Allocation failure on that
path remains a transport failure, with no invented application error code.

## Validation

Native regression coverage includes zero/negative/extreme widths, oversized and
non-dividing widths, all valid divisors of six, 80 successful events, full-width
error codes, unchanged output sentinels, event/close failure, scratch mutation
before Err, malformed registrations/layouts, live-owner early-exit rejection and
fault injection for both aggregate and Bytes allocation. Bytes counters return
to zero; that is not a process-wide leak proof. Renaming Result/State, reversing
variant declaration order and moving Bytes to the first field retain correct
native errors and successful scalar/pixel transport.

The packaged workflow checks exact PPM bytes, deterministic non-signal errors,
no frame/state publication on failed open, event/close failures, cache hits,
binary-only execution with empty PATH after deleting the source/build tree, and
source-free NSB materialization. The older resource workflows remain regressions.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --test native_resource_session -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_resource_session_workflow -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p yir-lower-llvm --lib -- owned_return_contract_tests native_session_resource_boundary --test-threads=1
```

Local results: all 28 native resource tests and four complete artifact workflows
passed. The complete LLVM lowering unit suite reports 183 passed and one existing
ignored test; 52 tensor/mainline/CLI tests, four manifest/packaging tests and 40
repository script tests also pass. Both file-budget tests, documentation links,
UTF-8, host-path policy and workspace architecture checks pass. Tensor drift is clean across 4540 checks;
the other 36 coordinate records retain their previous digest.

## Remaining Boundaries

Only resource-free guarded Err and a terminal complete Result are admitted here,
not arbitrary branch-local ownership or guarded successful resource returns.
Next is path-specific cleanup for a concrete early Ok caller, not broad syntax
admission without execution evidence. Resource helpers/loops, resizing,
asynchronous cancellation, native ns-nova windows and device/provider completion
remain separate work. Native traps outside the admitted validation path are not
recoverable errors. This local CPU proof does not claim Linux/GPU parity,
performance improvements, pure Nsld finalization or an all-Nuis runtime.
