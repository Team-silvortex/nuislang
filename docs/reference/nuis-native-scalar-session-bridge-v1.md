# Native Scalar Session Bridge V1

Status: experimental static CPU bridge with explicit build, artifact and launch selection.
This is not the default application host, a native window renderer, or a
self-contained Nsld image. The ordinary compiled image host still runs embedded
YIR callbacks.

## Selection

`yir_lower_llvm::native_session::emit_registered(module, id)` consumes an existing
YIR application-session registration and its shared `ApplicationSessionSignature`.
It emits a complete LLVM unit containing the registered open/event/close helpers,
their admitted helper closure and one static export per role. No source function names or application policies
are hardcoded. Export symbols encode the registration ID as UTF-8 bytes in hex.
The contract identifier is `nuis-native-scalar-session-bridge-v1`.

The first admitted profile is deliberately small:

- CPU helpers containing scalar parameters/constants, selected arithmetic and
  comparisons, struct construction/fields, selection and aggregate returns.
- Nested nonempty scalar state made of `bool`, `i32`, `i64`, `f32` and `f64`.
- Acyclic helper calls with scalar parameters and one declared scalar return,
  including nested calls, zero-argument helpers and guarded scalar returns.
- Checked flat-i64 aggregate helper returns through ordinary or scoped calls,
  including multi-state guarded branches and matching explicit-step continue.
- Checked i64 division/remainder, with exact operands and reached-path failure.
- Counted i64 loops with constant or runtime-checked induction inputs and ordered
  scalar add/multiply carries, with at most 64 carries and 65536 iterations per loop.
- Scoped loop-body calls with a discarded scalar result, one i64 carry, or a
  checked flat aggregate of at most 64 i64 carries returned before the induction step.
- At most 64 input slots, 64 state slots and 4096 body nodes per function.
- At most 64 reachable functions (including registered roots), 16384 total body
  nodes and 32 functions on any root-to-leaf call path.
- No recursion, unbounded loops, resource/provider effects, resource-bearing state
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
Aggregate-helper reachability includes registered host entries and explicit
export/noinline roots, not only main. This is a common lowering rule, not a
hardcoded application-session name. Unsupported outer-state updates reject
instead of degrading to a counter-only loop after scoped lowering declines.
Likewise, a single-carry scoped call cannot absorb another outer-state update
as a temporary prefix; multi-carry updates need the explicit supported shape.
This is not a guarantee that downstream LLVM optimization preserves machine calls.

## Checked Arithmetic

The profile admits existing `cpu.div` and `cpu.rem` only for two exact i64 values.
It reuses the ordinary LLVM scalar emitter: reached operations check zero divisors
and `i64::MIN` with `-1` before `sdiv` or `srem`. Both invalid cases trap for both
operators. Valid signed results truncate toward zero and remainder follows the
dividend. Implicit bool/i32/float conversion and typed i32/f32/f64 division are
not admitted by this profile. No new opcode, callback ABI or arithmetic runtime
is introduced.

Purity is not permission to speculate checked arithmetic. Shared source lowering
propagates division/remainder through a cached, iterative call graph, independent
of declaration order. Eligible acyclic i64/bool scalar helpers reuse the existing
guarded branch/continuation outliner. Prefix work keeps source order; an unselected
branch skips its arithmetic, even when its result is discarded or passed to a
callee that ignores the argument. Work before the guard still executes.

A flat-i64 aggregate helper may return already available fields before a later
checked operation. More general fallible aggregate branch returns have not yet
been normalized: early-return, two-arm and same-callee argument shapes reject
before select-style shortcuts can speculate them. This is a known source-lowering
boundary, not full aggregate control-flow support. Other runtime families retain
their own branch contracts; this analysis is not a whole-language effect proof.

Native failure terminates the process, not a catchable callback result or fuel
error. It cannot promise cleanup, rollback or a returned state. The reference
session instead reports an arithmetic error, preserves its last accepted state
and retains failure through close. These are distinct failure mechanisms.

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
Scalar helper calls may surround loops, including in nested helpers, or become
the admitted scoped body action described below. Other conditional/effectful
loop forms remain separate work.

The CPU module reuses its cooperative registered driver for plain i64 chains,
publishes `LoopState` only at completion and shares reference execution fuel.
No CPU loop dispatch is added to the generic executor or application host. This
per-loop native bound is not whole-callback fuel, native preemption or a wall-time
guarantee; native budgeted calls still reject before entry.

### Scoped Calls

The existing `cpu.loop_while_i64_effect` opcode is admitted only when its action
is `cpu.scoped_call`, `cpu.scoped_call_i64_carry`, `cpu.scoped_call_i64_carries`
or `cpu.scoped_call_i64_carries_break`.
This is not blanket admission
of effect metadata. The first form discards a scalar return; the second consumes
the shared `parse_scoped_i64_carry` contract with one named i64 seed and exactly
one `$carry` operand. Neither form introduces an interpreter or new runtime ABI.

Each body target must be a scalar-returning helper or the checked multi-carry
helper described below, in the same admitted closure.
It participates in cycle, depth, function-count and total-node checks, including
when a loop is statically zero-trip or its result is discarded. Hidden printing,
resource access, unknown/drifted signatures and cross-function value captures
reject before emission. The ordinary loop emitter and CPU registered execution
driver are reused, not copied into the application host.

`$current` is the counter before its step; `$carry` is the preceding return or
initial seed. Both require exact i64 helper parameters. Named captures support
`bool/i32/i64/f32/f64` without coercion, including source lowering of a single-i64
carry with i32/float captures. Calls complete, update the carry, then step the
counter. Zero-trip loops preserve the seed and do not invoke the helper. The
native induction preflight runs before the first invocation; invalid values trap
the process. Pure scalar carry transport adds no owned aggregate per iteration.

Multi-carry calls use `parse_scoped_i64_carries`, not a new opcode or ABI. Each
`$owned_struct_carry:N:seed` binds exactly one named i64 seed to the corresponding
flat `carryN:i64` field. The declared helper result must be owned and match the
action's nominal type, field order and kinds; parameters remain exact scalar
values. At most 64 slots are allowed, within the existing 64-parameter bound.
Operand order need not equal field order. Scalar i32/float captures are also
supported by the source multi-carry path, without widening carried state kinds.

The helper sees the previous iteration's carries and pre-step counter. Its own
ordered updates produce one returned aggregate; all leaves are extracted and
the temporary aggregate is dropped before stepping or invoking it again.
Zero-trip loops preserve every seed. Unlike a plain scalar chain or single-i64
call, this route currently allocates one aggregate per iteration. Release
balance is tested; eliminating that allocation remains an optimization, not a
claim of allocation-free execution or whole-program memory safety.

The explicit source shape is a scoped helper call, ordered scalar projections
and a counted induction step. Source guarded breaks can instead use the private
normalization described below. Unrestricted aggregate calls, move/copy/resource
captures and arbitrary conditional loop bodies are not admitted.

### Guarded Break

`scoped_call_i64_carries_break` reuses the same parser, typed layout, bounded helper
closure and aggregate loop emitter. The last i64 slot is private control: its seed
must be zero even on zero trips, and each returned value must be 0 (advance) or 1
(break). A control-only layout is valid and the control slot counts toward the
64-slot/parameter bounds. It is not an arbitrary user-data or boolean carry.

The returned aggregate is unpacked and released, then the control value is checked
before committing any returned carries. A valid return commits every carry; 1 exits
with the current pre-step counter, while 0 reaches the ordinary induction step.
An invalid seed/return traps the process and never publishes callback output.
This is not a catchable callback status or reference-fuel error.

Native admission still requires the **entire induction sequence**, ignoring any
possible early break, to be finite, non-wrapping and within 65536 iterations. An
immediate break does not excuse an active zero step, overflow or excessive bound.
Constant proofs and reached-loop runtime preflight are unchanged.

Pure scalar source loops now reuse the existing effect-loop normalizer for strict
ascending/descending unit steps and i64/bool helper composition. Break-only loops
use one private bit instead of two complementary flags. This avoids an unnecessary
aggregate branch call while preserving selected-path suffix evaluation and nested
loop scope. The compiler still requires generated control provenance; a user
aggregate followed by `if signal == 1 { break; }` is not automatically trusted.
YIR scoped captures retain all five exact scalar kinds, independently of this more
restricted automatic source normalizer.

Branches that update multiple values, including a user carry plus the break bit,
and mixed break/continue normalization can generate ordinary aggregate helper
calls. The checked flat-i64 subset below now admits these calls. A continue still
requires its matching explicit unit step; step-before-break remains unsupported.

### Flat Helper Returns

Ordinary `cpu.call_owned_struct` and scoped multi-carry calls share one native
return-layout validator. The helper must return an owned nominal aggregate with
1..64 ordered `carry0:i64` through `carryN:i64` fields. The call, declared result,
terminal return and any explicit guarded-return layout must agree. Actual return
leaves and all scalar input values retain exact types; there is no implicit
bool/i32/float conversion, resource input or nested aggregate admission.

This is a structural contract, not an allowlist of generated helper names or
precombined arities. Ordinary and mixed scoped edges share the existing acyclic
closure and size/depth limits, including discarded and guard-bypassed calls.
Unknown targets, foreign-lane values, hidden effects and signature drift reject.
The generic LLVM call lowering unpacks and drops each temporary return immediately;
no interpreter, host dispatcher, new opcode or new aggregate ABI is introduced.

Multi-state guarded break and explicit-step continue now preserve source-ordered
updates, selected-path suffix evaluation and child-loop exit scope. A guarded
return bypasses later dynamic induction preflight; reaching an excessive bound
still traps the process. These checks do not add callback fuel/preemption or
resource-bearing returns. Every reached aggregate return still allocates, even
when nested inside one loop iteration; release balance is not allocation freedom.

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
The native frontdoor/cache/standalone regression also covered this fixture before
advancing to scoped loop calls.
[Guard regressions](../../tools/nuisc/tests/native_application_bridge/dynamic_loop_guard.rs)
run 1680 parameter cases in twelve native executables across six comparisons and
add/sub. Volatile inputs prevent constant-only evaluation. A test-only rejection
sentinel permits comparison with independent checked-step simulation; iteration
counters prove rejected/zero-trip/skipped paths never enter the body. Six separate
executions invoke the real registered export, retain the unmodified trap and
require process failure without a returned ABI status. Boolean induction drift
rejects instead of silently coercing to i64.
Every guard-probe process has a deadline so a regression cannot hang the test suite.

The [scoped-loop fixture](../../tools/nuisc/tests/native_application_bridge/scoped_loops.ns)
retains four scoped loops, shared/nested helpers, a zero-argument discarded call,
single-i64 carry returns and dynamic induction. Six native/reference executions
retain exact session slots and reversed-declaration parity.
[Admission regressions](../../tools/nuisc/tests/native_application_bridge/scoped_admission.rs)
check scoped-only and mixed direct/scoped cycles, exact depth/function boundaries,
unknown/drifted targets, malformed captures, hidden effects and caller-lane isolation.
[Execution regressions](../../tools/nuisc/tests/native_application_bridge/scoped_execution.rs)
observe the actual helper arguments in 36 callback cases and 84 invocations across
two native binaries, with old-counter/old-carry ordering, wrapping i64 carries,
all five scalar kinds, signed zero and NaN payloads. Three more binaries retain
real traps and prove invalid induction enters no helper or publication path.
Reference fuel exhaustion retains the accepted state and cleanup cannot clear
the failure. This is not native fuel/preemption evidence.

The [multi-carry fixture](../../tools/nuisc/tests/native_application_bridge/multi_loops.ns)
adds shared aggregate-returning helpers with two carries, ascending/descending
loops and zero-trip seeds. Six more native/reference runs retain exact session
slots and reversed-declaration parity. The
[multi-carry execution tests](../../tools/nuisc/tests/native_application_bridge/multi_execution.rs)
run 144 callback cases and 288 actual helper invocations across six native
executables with 2/3/7 carries, reversed operand order, source-ordered updates,
wrapping carried arithmetic, zero/one trips and all five scalar capture kinds.
Test-only wrappers delegate to the real aggregate allocator/drop functions,
checking release before the next helper and equal allocation/drop counts after
each export. Three additional binaries retain real preflight traps.
[Multi-carry admission tests](../../tools/nuisc/tests/native_application_bridge/multi_admission.rs)
reject layout/ownership/seed/parameter drift, recursive zero-trip edges, hidden
effects, foreign lanes and attempts to enable general aggregate calls. They also
guard against silently discarded outer-state updates in counted-loop fallback.

The [guarded-break fixture](../../tools/nuisc/tests/native_application_bridge/break_loops.ns)
adds source-normalized two-carry and control-only loops. Six native/reference runs
retain typed state and reversed-declaration parity. The
[break execution tests](../../tools/nuisc/tests/native_application_bridge/break_execution.rs)
exercise 180 callback cases in six native binaries with 2/3/7 slots, early/middle/
last/no break, zero trips, both directions, reversed operands/declarations, wrapping
user carries and exact five-scalar captures. Real allocation/drop counters include
the exiting iteration and callback State. Eleven unmodified process-trap runs
cover nonzero seeds (including zero trips), invalid returned control and invalid
induction even when the helper would immediately break. Helper-entry probes flush
their output before execution so a trap cannot hide buffered evidence.
[Source tests](../../tools/nuisc/tests/native_application_bridge/break_source.rs)
compare native, reference and independent expected states in 40 cases across eight
binaries: nested/flat loops, both directions, pre-step exit counters, skipped suffix
updates, multi-state guarded break and matching explicit-step continue.
Shared negative checks retain
layout/kind/closure restrictions and reject unmodeled source updates or forged
control provenance. Reference budget exhaustion retains the last accepted state;
native budget/preemption remains unimplemented.

The [multi-state branch fixture](../../tools/nuisc/tests/native_application_bridge/branch_loops.ns)
adds six typed native/reference lifecycle runs with guarded updates and mixed
break/continue. [Nested aggregate execution](../../tools/nuisc/tests/native_application_bridge/aggregate_execution.rs)
uses six native binaries for 144 callback cases and 576 inner helper calls, with
2/3/7 slots, reversed capture/declaration order, wrapping updates, signed-zero/NaN
bits and real allocator/drop balance, including early returns. Entry probes verify
that the previous temporary has already been released before the next inner call.
[Aggregate admission](../../tools/nuisc/tests/native_application_bridge/aggregate_admission.rs)
rejects malformed layouts, result/parameter/return-field drift, hidden effects,
foreign captures and recursive calls, while shared unit tests exercise slot bounds.
[Guard execution](../../tools/nuisc/tests/native_application_bridge/aggregate_guards.rs)
proves that an early return skips a later excessive-loop preflight, while actually
entering that loop traps without publishing callback state.

The [division/remainder tests](../../tools/nuisc/tests/native_application_bridge/division_execution.rs)
compare 1840 callbacks across eight native binaries with reference execution and
an independent i128 oracle. Inputs cover signed extremes, signs, zero, scalar and
flat aggregate returns, unused results and unused arguments. Twenty invalid
dynamic cases and four invalid literal cases trap; four unselected literal cases
return safely. Flushed leaf-entry probes prove actual evaluation order, including
failing prefix work before an unselected branch. Every process run is bounded.
[Admission tests](../../tools/nuisc/tests/native_application_bridge/division_admission.rs)
reject either/both operand kind drift, malformed arity, non-i64 typed opcodes and
unoutlined fallible aggregate return branches. The
[division fixture](../../tools/nuisc/tests/native_application_bridge/division_loops.ns)
adds six typed lifecycle native/reference runs with multi-state loop exits and
reordered declarations; it also passes the real build/cache/standalone workflow.

The [production-host regression](../../tools/nuisc/tests/native_application_host.rs)
compiles the [basic scalar callback fixture](../../tools/nuisc/tests/native_application_bridge/main.ns)
and invokes the real packer for normal and reversed YIR
declarations. Actual statically linked host calls match reference states. Invalid
later arguments reject before open. A generated-object event failure preserves the
last accepted state, runs cleanup and exits unsuccessfully without reference
fallback. Mixing an entry with a different YIR graph rejects before any callback.
Separate [host policy tests](../../crates/yir-runtime-host/src/native_application_session/tests.rs)
use fault-injecting test exports to cover malformed output, failed/duplicate close,
open failure, Drop, descriptor drift, argument validation and fuel rejection.
These test doubles are policy evidence, not additional lowering proofs.

The [frontdoor regression](../../tools/nuis/tests/native_session_workflow.rs) builds
the five-scalar multi-carry, guarded-break, multi-state branch and checked-division
fixtures with two registrations,
runs typed events and close,
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

Normalize fallible flat-i64 aggregate branch returns without speculating branch
expressions or call arguments, retaining exact layout/kind checks and real
zero/overflow execution evidence. Keep checked division/remainder, flat-helper
and scoped-break frontdoor/relocation regressions. Per-return aggregate allocation
is a separate optimization boundary.
Whole-callback native scheduling limits, general loops, Buffer callbacks, resource
state, provider dispatch and ordinary image-host selection still need separate
implementation and evidence. The bridge alone does not impose call order; the
shared application session host does. Existing embedded-YIR image and guarded-loop
proofs remain required.
