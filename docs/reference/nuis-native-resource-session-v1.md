# Native Buffer And Bytes Resource Sessions V1

Historical library checkpoint. The [artifact follow-up](nuis-native-resource-session-artifacts-v1.md)
subsequently delivers project/YIR/NSB registration, export identity and source-free
launch, advancing the live coordinate to active/60. The original scope below is
retained as evidence, not a current denial of that later delivery.

This beta-0.18.9 worktree follow-up establishes a separately admitted native
resource callback path. Compiled Nuis `open/event/close` helpers receive a borrowed
Buffer and return owned Bytes; the host publishes only complete successful
results. It is a synchronous library interface, not yet project/NSB delivery or
a native ns-nova frame pipeline.

The live `compiler-toolchain/nuisc/native-resource-session-callbacks` coordinate
advances from early/0 to active/35. This is a conservative scope label, not a
measured percentage of remaining engineering time. The historical
[recalibration](nuis-development-tensor-recalibration-2026-11-13.md) retains its
original rejection-only checkpoint. Scalar/record admission remains unchanged.

## Contract And Layering

`yir_core::native_resource_session::ResourceSessionRegistration` owns the shared
`nuis-native-resource-session-v1` contract: an identifier, three distinct helper
roots in open/event/close order, and inclusive minimum/maximum Buffer extents.
All three signatures must be exactly `(borrowed ref Buffer, value i64) -> owned
Bytes`. The maximum is 4096 i64 cells; a registration may allow zero cells.
These bounds are v1 admission policy, not language-wide resource limits.

The compiler's `compile_resource_session_source` preserves those explicit roots
and verifies their YIR signatures. `yir_lower_llvm::native_resource_session`
performs profile admission and emits native wrappers around ordinary Nuis helper
lowering. `yir_runtime_host::native_resource_session` binds the trusted exports
and owns session state. The runtime does not depend on the LLVM emitter; neither
layer knows ns-nova application logic. No interpreter fallback is used.

The source fixture lives at
`tools/nuisc/tests/native_resource_session/main.ns`. Its event helper is:

```nuis
fn step(buffer: ref Buffer, delta: i64) -> Bytes {
    let value: i64 = load_at(buffer, 0) + delta;
    store_at(buffer, 0, value);
    return copy_bytes(buffer);
}
```

Buffer currently stores **i64 cells**. Bytes preserves their native-endian bit
representation, including negative values and high bits. This is not a u8
Buffer, UTF-8 text, or a portable serialization format. An invocation preserves
its initial cell count; registration bounds are not resize permission.

The first profile admits finite straight-line integer arithmetic, Buffer
length/load/store, Bytes copy/move/length/drop, and a direct owned Bytes return.
Each callback has at most 256 unique CPU nodes and eight Bytes copies. Admission
checks function/lane ownership, operand kinds, linear Bytes consumption and a
complete final return. External executable ordering/lifetime edges are rejected,
not silently discarded while extracting callbacks. Immutable contract-lane text
and its dependency edges are retained, including clock metadata; its label does
not admit executable lookalikes. Shared nodes, leaks, use after consume, arbitrary calls,
loops, providers and allocation/free of the borrowed Buffer are rejected.

The existing helper parameter lowering now recognizes typed `param_buffer_ref`
and `param_owned_bytes` bindings rather than leaving a deferred-lowering marker.
Wrong or missing bindings remain errors; neither resource converts to a scalar
pointer slot. This fix does not admit arbitrary owned-Bytes session parameters.

## Ownership And Failure

The host owns the accepted `Vec<i64>` and provides a private working copy to
each callback. The wrapper validates tag, extent, output capacity, nonnullness
and eight-byte alignment before entry. A nonzero status never publishes that
working copy. On success it validates the returned blob length/data, copies
every cell into host-owned output, releases the temporary Bytes once, and only
then returns success for atomic whole-state replacement.

The raw ABI is `(tag, buffer_ptr, words, argument, output_ptr, output_words) ->
i32`, using u64 counts and an i64 argument. Status 1 rejects preflight, status 2
reports a null returned blob, and status 3 rejects an incompatible returned
length/data descriptor. Valid nonnull blobs are released on both success and
descriptor rejection. This wrapper is for trusted compiled callbacks, not a
validator for arbitrary malicious native pointers or fabricated blob headers.

The shared tag is **not a secret or unforgeable capability**. Authority comes
from trusted static code provenance and exact registration/module binding.
`NativeResourceBindings::from_static` is unsafe: its caller must supply exports
from the matching module, keep their code alive, guarantee full output on
success, prohibit borrowed-pointer escape and unwinding, and release temporary
Bytes before ordinary return. Public safe session methods never expose those
pointers. Bindings are thread-local; this is not concurrent callback admission.

Successful open starts completion sequence 1. Each successful event or close
increments it, while failed publication leaves the accepted state unchanged.
The sequence is session-local order, not a physical clock, provider completion
or GPU fence. An event error faults the session, rejects further events, and
allows one close using the last accepted state. Close is attempted once even if
it fails; repeated calls cannot run it again or erase its error. A successful
close does not erase an earlier event failure. Host Drop releases storage but
does not silently execute Nuis close or claim lifecycle completion.

Native bounds/arithmetic traps remain **process failures**. They do not unwind
through this ABI or execute recoverable Bytes cleanup. The process owns any
remaining allocation until OS teardown. The trap test allocates private Bytes
before an out-of-bounds write and verifies process termination and the absence
of a post-return marker. Recoverable trap handling is not claimed.

## Execution Evidence

The focused integration suite compiles the Nuis fixture through verified YIR
and LLVM into real host machine code, then invokes it through the production
resource-session API. A Unix test-only loader keeps the compiled code alive for
the session lifetime. It is not a production dynamic-loading design. Tests reuse
the existing generated compatibility runtime for owned blobs; no new C or
Objective-C application implementation replaces the Nuis callbacks.

Checks cover native/reference content agreement, 80 consecutive events,
complete signed/full-width contents, zero/one/4096-cell sessions, reordered YIR
storage, private Bytes release, invalid registration/signature/ownership,
preflight nonentry, failed open/event/close, and sticky failure/idempotent close.
The production live-blob counter is zero after ordinary completions. Null-return
and malformed-length fault injection test the wrapper's failure paths; the
null-return probe is not a process-wide malloc-failure experiment.

## Not Yet Delivered

The next slice is registration/export identity in project/YIR/NSB artifacts and
fresh/cache/source-free native resource-session launch. The current registration
is an explicit in-process library value; it is not serialized into the scalar
application-session manifest. The test loader and ordinary AOT link do not prove
Nsld-only resource-session delivery.

Bounded resource loops, arbitrary helpers, mixed scalar/resource State, resize,
asynchronous ownership, cancellation, provider/device completion, and native
frame integration remain separate work. No fresh Linux/GPU execution or
cross-platform parity is claimed by this local receipt.

## Reproduce

Run from the repository root with an available host LLVM/Clang toolchain:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_resource_session --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- param_lowering::tests native_session_resource_boundary --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_application_session --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_direct_eager_pair --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor mainline --test-threads=1
```

Native execution tests are Unix-gated. A non-Unix run of the same target does
not certify those executions; signature and admission tests still run there.

## Local Validation Receipt

At this worktree checkpoint, 11 resource-session tests pass, including real
native invocation and the isolated trap child. Four parameter/scalar-boundary
checks, seven existing runtime native-session tests, two direct scalar/record
native/reference regressions, 51 tensor/mainline tests, 40 script tests, two
file-budget tests and the host-path policy test also pass. Formatting, UTF-8,
local documentation links and the workspace dependency audit pass.

The rebuilt ordinary CLI reports 37 coordinates and 4521 clean drift checks.
All previous 4515 check IDs retain their order, the other 36 cells are identical,
and the historical bootstrap-critical count/average remain 21/100. Coverage,
hierarchy and task readiness are clean; the selected prerequisite remains this
resource callback coordinate with artifact registration as its next action.

The broader strict Clippy invocation including `nuisc` is not green: it reports
30 warnings-as-errors in existing control-flow lowering code, including argument
counts and type complexity. Those files are unchanged here; the new resource
admission warning was corrected rather than suppressed. This receipt is not an
all-workspace test or warning-free compiler certification.
The separate runtime-host strict check also reports its existing large-enum
warning in `application_session/execution.rs`; that file is unchanged.
