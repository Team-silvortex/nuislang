# Native Scalar Session Bridge V1

Status: experimental static CPU bridge and explicitly selected native session host.
This is not the default application host, a native window renderer, or a
self-contained Nsld image. The ordinary compiled image host still runs embedded
YIR callbacks.

## Selection

`yir_lower_llvm::native_session::emit_registered(module, id)` consumes an existing
YIR application-session registration and its shared `ApplicationSessionSignature`.
It emits a complete LLVM unit containing the registered open/event/close helpers
and one static export per role. No source function names or application policies
are hardcoded. Export symbols encode the registration ID as UTF-8 bytes in hex.
The contract identifier is `nuis-native-scalar-session-bridge-v1`.

The first admitted profile is deliberately small:

- CPU helpers containing scalar parameters/constants, selected arithmetic and
  comparisons, struct construction/fields, selection and aggregate returns.
- Nested nonempty scalar state made of `bool`, `i32`, `i64`, `f32` and `f64`.
- At most 64 input slots, 64 state slots and 4096 body nodes per callback.
- No helper calls, loops, provider effects, resource-bearing state or global init.

Each callback must have one canonical owned-aggregate return whose nominal type,
field order and scalar kinds agree with the event/close flattened state signature.
Parameter nodes must match their declared indices, types and function lanes.
Dependencies from outside a callback are rejected, not silently initialized or
discarded. The unrelated main function is not replayed.

This bridge uses strict native value materialization: every reached value-producing
node must acquire an LLVM value, and the function must emit a terminal return.
It does not decide completeness by scanning diagnostics or LLVM comments.
General LLVM inspection keeps its existing partial-lowering behavior.

## Call ABI

Each export has the same internal native ABI:

```text
i32 bridge(ptr args, i64 argc, ptr out, i64 outc)
```

`args` and `out` contain native-endian 64-bit slots. Counts are exact signature
lengths, not buffer capacities. State output follows the declared layout's
depth-first field order. Event/close inputs start with those state slots, followed
by their additional arguments. An open callback with zero arguments accepts a null
input pointer. Output must be nonnull because empty state is not admitted.

| Scalar | Slot Representation |
| --- | --- |
| `bool` | Exactly 0 or 1. |
| `i32` | Sign-extended to 64 bits. |
| `i64` | Its 64-bit representation. |
| `f32` | IEEE bits in the low 32 bits; upper bits must be zero. |
| `f64` | Its 64-bit IEEE representation. |

Status 0 means success, 1 means invalid counts/pointers, and 2 means noncanonical
input encoding. Counts, null pointers and all scalar encodings are checked before
entering the callback or writing output. All input slots are loaded before output
publication, so in-place state updates are supported. The bridge reads the returned
aggregate, drops its temporary storage, and only then publishes scalar slots.

This is a trusted, statically linked internal host boundary, not a new Nuis cffi
authorization or pointer sandbox. Nonnull pointers must actually reference the
claimed readable/writable memory. The bridge cannot validate arbitrary addresses.
Native traps terminate execution; they are not caught, rolled back, retried or
converted into reference-executor fuel exhaustion.

## Explicit Host

`yir-pack-aot <module.yir> <output-dir> --native-session <registration-id>` now
builds a scalar-session executable on 64-bit macOS/Linux hosts. This selection is
exclusive with `--headless` and frame-scale/window options. The default image and
headless provider hosts are unchanged. This is not yet a `nuis build` manifest
profile or the Nsld self-contained image path.

The packer statically links a callback object, an LLVM process entry and the
existing runtime. The entry consumes a `NativeSessionDescriptorV1` emitted beside
the callbacks: version `u64`, source pointer/`u64` length, registration-ID
pointer/`u64` length, layout pointer/`u64` length, then open/event/close pointers.
There is no dynamic library lookup. The source and descriptor are bounded to 8 MiB
of YIR, 256 ID bytes and 64 KiB of layout respectively at host ingress.

Before open, the host compares the complete parsed YIR graph embedded in the
entry against the graph bound into the callback object, including nodes, edges,
function tables, registrations and lanes. Whitespace differences may parse equally;
declaration reordering is not normalized during identity admission. A newly compiled
reordered object works, but mixing it with a different entry rejects. Matching
names or arity is insufficient. This detects stale/mixed artifacts, not malicious
native-code tampering: trusted static production/linking still establishes that
the descriptor and function pointers belong together. It is not a signature scheme.

`ApplicationSession::open_native_registered` consumes a thread-local
`NativeSessionBindings` instead of constructing a `FunctionSession` interpreter.
It reuses the existing state machine: invalid caller arguments do not enter a
callback or consume close; callback failure faults the session, retains the last
accepted state, and permits one explicit close. Failed close is not retried. Cleanup
success cannot erase an event failure. Drop does not run a Nuis callback. Native
output is decoded in separate storage before state publication. Native budgeted
event/close calls reject before callback entry rather than pretending reference
fuel can preempt native instructions.

Scalar slot encoding and nested state reconstruction now live in `yir-core`.
The runtime does not depend on LLVM or repeat the CPU operation admission table.
The unsafe static-binding constructor is a compiler/host trust boundary, not new
Nuis `unsafe` syntax or authorization for arbitrary C ABI calls. Existing Rust
host implementation and aggregate allocation compatibility routines remain.

The native process accepts `--application-session`, `--open-args`, repeatable
`--event-args`, and `--close-args`. Arguments are comma-separated, typed according
to the registered signature; booleans are `true`/`false`, integers and floats are
parsed as their declared kinds. An empty string supplies zero arguments. All calls
are checked before open, with at most 64 events and 64 slots per callback. The
host reports accepted states and `native_session_completed=1` only after successful
explicit close. It offers no provider transport, cancellation, rollback, retry,
wall-time preemption or invented per-node reference trace.

For the nested scalar fixture compiled to `session.yir`:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p yir-pack-aot -- session.yir build/native --native-session counter
build/native/session --application-session counter --open-args '10,-17,true,1.5,-2.25' --event-args '3,false' --close-args '5'
```

## Evidence

The [native regression](../../tools/nuisc/tests/native_application_bridge.rs) uses
a [pure Nuis fixture](../../tools/nuisc/tests/native_application_bridge/main.ns).
A static LLVM test driver invokes the exported bridges in one executable, passing
actual returned state into subsequent calls. It does not interpret YIR. Existing
host object emission and runtime support are used; no new C/Objective-C shim is added.

The regression compares open, three separate events (including an early return)
and close against `ApplicationSession` reference execution. It covers nested state,
all five scalar kinds, finite floats, signed zero, NaN payload bits, in-place calls,
reordered declarations and unrelated-main exclusion. Invalid arguments preserve
the output; an entry counter confirms they never call the native helper.
Separate checks cover zero-argument native open, the 64/65-slot boundary, resource
and layout drift, parameter/lane drift, external initialization dependencies and
unsupported side effects. A malformed unused field operation must fail strict
native materialization instead of silently disappearing.

The [production-host regression](../../tools/nuisc/tests/native_application_host.rs)
compiles that Nuis fixture and invokes the real packer for normal and reversed YIR
declarations. Actual statically linked host calls match reference states. Invalid
later arguments reject before open. A generated-object event failure preserves the
last accepted state, runs cleanup and exits unsuccessfully without reference
fallback. Mixing an entry with a different YIR graph rejects before any callback.
Separate [host policy tests](../../crates/yir-runtime-host/src/native_application_session/tests.rs)
use fault-injecting test exports to cover malformed output, failed/duplicate close,
open failure, Drop, descriptor drift, argument validation and fuel rejection.
These test doubles are policy evidence, not additional lowering proofs.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --test owned_cleanup_return -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_host -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib native_ -j 1 -- --test-threads=1
```

Native execution evidence currently comes from Apple Silicon. It does not certify
Linux/Windows execution or device-provider parity.

## Next Boundary

Thread the explicit native profile through identity-checked `nuis build` and
`nuis run-artifact`, without relabeling it as the existing provider script host.
Native scheduling limits, helper composition, loops, Buffer callbacks, resource
state, provider dispatch and ordinary image-host selection still need separate
implementation and evidence. The bridge alone does not impose call order; the
shared application session host does. Existing embedded-YIR image and guarded-loop
proofs remain required.
