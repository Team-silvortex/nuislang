# Native Scalar Session Bridge V1

Status: experimental static CPU bridge with explicit build, artifact and launch selection.
This is not the default application host, a native window renderer, or a
self-contained Nsld image. The ordinary compiled image host still runs embedded
YIR callbacks.

## Selection

`yir_lower_llvm::native_session::emit_registered(module, id)` consumes an existing
YIR application-session registration and its shared `ApplicationSessionSignature`.
It emits a complete LLVM unit containing the registered open/event/close helpers,
their admitted scalar helper closure and one static export per role. No source function names or application policies
are hardcoded. Export symbols encode the registration ID as UTF-8 bytes in hex.
The contract identifier is `nuis-native-scalar-session-bridge-v1`.

The first admitted profile is deliberately small:

- CPU helpers containing scalar parameters/constants, selected arithmetic and
  comparisons, struct construction/fields, selection and aggregate returns.
- Nested nonempty scalar state made of `bool`, `i32`, `i64`, `f32` and `f64`.
- Acyclic helper calls with scalar parameters and one declared scalar return,
  including nested calls, zero-argument helpers and guarded scalar returns.
- Counted i64 loops with constant or runtime-checked induction inputs and ordered
  scalar add/multiply carries, with at most 64 carries and 65536 iterations per loop.
- At most 64 input slots, 64 state slots and 4096 body nodes per function.
- At most 64 reachable functions (including registered roots), 16384 total body
  nodes and 32 functions on any root-to-leaf call path.
- No recursion, unbounded/effectful loops, provider effects, resource-bearing state
  or global init.

Each callback must have one canonical owned-aggregate return whose nominal type,
field order and scalar kinds agree with the event/close flattened state signature.
Parameter nodes must match their declared indices, types and function lanes.
Only explicit typed calls may cross a function boundary. Direct value dependencies
and incoming edges from other functions are rejected, not silently initialized or
discarded. Each reachable helper is checked, even if its result is unused or a guard
would bypass the call. Unselected helpers and unrelated main are not emitted or replayed.
The call graph is checked iteratively, with shared callees counted once and the
longest path bounded; it does not recursively traverse an untrusted host stack.

This bridge uses strict native value materialization: every reached value-producing
node must acquire an LLVM value, and the function must emit a terminal return.
It does not decide completeness by scanning diagnostics or LLVM comments.
Helper arguments and scalar returns must have the exact declared materialized kind,
including guarded returns; the generic backend's bool-to-i64 compatibility is not
admitted here. Guards require bool/i64 conditions; scalar helper guards cannot carry
an aggregate layout. General LLVM inspection keeps its existing partial-lowering behavior.
Supported synchronous `@noinline` functions now retain their NIR-to-YIR call boundary.
This is not a guarantee that downstream LLVM optimization preserves machine calls.

## Counted Loops

The profile consumes existing `cpu.loop_while_i64`, `cpu.loop_while_i64_chain`
and `cpu.loop_while_scalar_chain` nodes. For direct constant i64 start, limit and
add/sub step nodes, admission proves termination in wide arithmetic before
emission. Runtime i64 inputs instead receive an O(1) LLVM preflight at the reached
loop site, before the first iteration. The ordinary loop emitter stays unchanged;
general LLVM inspection and reference execution do not acquire this native policy.

Both routes support `eq/ne/lt/le/gt/ge`. Zero-trip loops preserve their seeds,
including inactive zero steps. Active zero steps, wrong directions, unreachable
non-wrapping equality exits, induction overflow and more than 65536 trips reject.
Constants reject during emission; invalid dynamic values execute `llvm.trap`.
That process failure is not an ABI status, catchable callback error or reference
fuel exhaustion, and cannot promise cleanup or recover the process. Earlier
callback work is not rolled back. A prior guarded return bypasses the preflight.

The preflight uses unsigned i64 quotient/remainder with a safe nonzero divisor,
plus i128 distance, step and final-counter arithmetic. It handles `i64::MIN`
subtraction and distances across the entire signed range without signed overflow
or division-by-zero UB. No i128 division runtime helper, loop interpretation or
new opcode is introduced. Constant proofs require no dynamic guard.

Carries have exact i64 seeds. Add/multiply may read the old/new counter, any
previous-iteration carry, or an already-updated sibling carry. Future sibling
reads, implicit float/bool/i32 conversion and payload-bearing carry operations
are not admitted. LLVM reuses the ordinary loop lowering; carry arithmetic wraps
as i64 and no per-iteration owned aggregate is allocated for this flat chain.
Scalar helper calls may surround loops, including in nested helpers, but calls
inside loop bodies and conditional/scoped-effect loop forms remain separate work.

The CPU module reuses its cooperative registered driver for plain i64 chains,
publishes `LoopState` only at completion and shares reference execution fuel.
No CPU loop dispatch is added to the generic executor or application host. This
per-loop native bound is not whole-callback fuel, native preemption or a wall-time
guarantee; native budgeted calls still reject before entry.

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
headless provider hosts are unchanged. The corresponding `nuis build` profile is
described below; neither route is the Nsld self-contained image path.

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

## Build And Launch

The project manifest may select `packaging_mode = "native-session-aot-bundle:counter"`,
or the build command may explicitly override it. The registration suffix is part
of the build manifest, compiled artifact, envelope and cache identity. There is
no implicit first-registration selection and no runtime switch to another native
registration in the same artifact.

```sh
nuis build --packaging-mode native-session-aot-bundle:counter path/to/project build/native
nuis run-artifact build/native --native-session counter --open-args '10,-17,true,1.5,-2.25' --event-args '3,false' --close-args '5'
nuis verify-artifact build/native/nuis.compiled.artifact
nuis materialize-artifact build/native/nuis.compiled.artifact build/restored
nuis run-artifact build/restored --native-session counter --open-args '10,-17,true,1.5,-2.25' --close-args '5'
```

`check`/`workflow` honor manifest selection and report a real LLVM checkpoint.
The selected callbacks are lowered directly; an unsupported callback fails the
build rather than falling back to a verified-YIR-only or whole-main path.

The `nuis-native-session-build-inputs-v1` carrier uses the
`native-scalar-llvm-v1` checkpoint. It embeds source, tokens, AST, NIR, YIR,
compiler handoff, selected LLVM and bundle declarations. Present documentation,
import, Galaxy and resolution-lock metadata is also carried and relocated.
Each input has a unique hash row and a canonical output filename, with per-input
bounds and a shared 64 MiB limit. Native YIR retains the host's 8 MiB bound.
Verification regenerates the selected callback LLVM exactly and checks its
registration, scalar layout and static-host declarations. Manifest verification
also compares the executable bytes with the compiled artifact image. These are
integrity/consistency checks, not native-code authentication or a signature scheme.

Standalone verification and `materialize-artifact` do not require the original
build directory. Materialization verifies the restored manifest before granting
executable permissions on Unix. Informational producer paths in bundle text are
not runtime dependencies. The launcher checks the resolved binary identity and
uses the same typed script parser as the packaged entry before open. Unlike the
provider `--application-session` option, `--native-session` admits all five scalar
kinds, prepares no provider, and has no interpreter or Nsld-runner fallback.
Cancellation, window, frame-export and provider options cannot be combined with
this script. Nonzero native process exit remains failure.

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

The [helper fixture](../../tools/nuisc/tests/native_application_bridge/helpers.ns)
retains actual typed YIR/LLVM calls after compilation, not just inline expansions.
Six more native executions cover nested and shared helpers, source early returns,
all five scalar kinds, NaN/signed-zero transport and reversed declarations.
[Closure regressions](../../tools/nuisc/tests/native_application_bridge/helpers.rs)
reject direct/mutual recursion, unknown or drifted signatures, implicit scalar
coercions, foreign-lane/global dependencies, hidden side effects and unmaterialized
helper values. They test accepted/rejected function-count, depth and body-size
boundaries, plus the whole-closure node limit. Unselected recursion stays excluded.

The [loop fixture](../../tools/nuisc/tests/native_application_bridge/loops.ns)
retains five counted loops through source lowering. Six native executions compare
all scalar slots with the reference for ascending, descending, zero-trip, ordered
multi-carry and product loops composed through helpers, including reversed
declarations and float bit transport. [Loop regressions](../../tools/nuisc/tests/native_application_bridge/loops.rs)
reject malformed constant induction, excessive trips, forward carries, float
seeds and hidden helper effects. Budgeted reference exhaustion preserves the
accepted application state and cannot become success through cleanup. Independent small-step simulation checks the
termination proof, including extreme signed inputs and the exact trip bound.

The [dynamic loop fixture](../../tools/nuisc/tests/native_application_bridge/dynamic_loops.ns)
adds runtime start/limit/step parameters from callback values, descending and
inactive zero-step chains. [Dynamic parity tests](../../tools/nuisc/tests/native_application_bridge/dynamic_loops.rs)
perform six additional native runs with exact typed state and reversed declarations,
and retain general-lowering policy isolation and shared bound/carry dependencies.
The native frontdoor/cache/standalone regression now consumes this fixture.
[Guard regressions](../../tools/nuisc/tests/native_application_bridge/dynamic_loop_guard.rs)
run 1680 parameter cases in twelve native executables across six comparisons and
add/sub. Volatile inputs prevent constant-only evaluation. A test-only rejection
sentinel permits comparison with independent checked-step simulation; iteration
counters prove rejected/zero-trip/skipped paths never enter the body. Six separate
executions invoke the real registered export, retain the unmodified trap and
require process failure without a returned ABI status. Boolean induction drift
rejects instead of silently coercing to i64.
Every guard-probe process has a deadline so a regression cannot hang the test suite.

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

The [frontdoor regression](../../tools/nuis/tests/native_session_workflow.rs) builds
the real five-scalar dynamic-loop fixture with two registrations, runs typed events and close,
rejects wrong arguments and changed binary/YIR/LLVM/bundle/metadata before open,
and switches registrations through a shared cache/output directory. It removes
the original output, verifies the standalone compiled artifact, materializes it
elsewhere and runs the restored executable with identical states. Separate
carrier tests reject rehashed LLVM and bundle drift, schema/profile downgrades,
duplicate fields and missing hash rows.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --test owned_cleanup_return -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_host -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib native_ -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --test checkpoint_workflow -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib aot_application_bundle -j 1 -- --test-threads=1
```

Native execution evidence currently comes from Apple Silicon. It does not certify
Linux/Windows execution or device-provider parity.

## Next Boundary

Extend scoped scalar loop-body calls within the selected native profile,
keeping admission explicit and retaining the frontdoor/relocation regressions.
Native scheduling limits, loops, Buffer callbacks, resource
state, provider dispatch and ordinary image-host selection still need separate
implementation and evidence. The bridge alone does not impose call order; the
shared application session host does. Existing embedded-YIR image and guarded-loop
proofs remain required.
