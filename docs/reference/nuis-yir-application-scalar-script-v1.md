# Headless Application Scalar Script

Contract: `nuis-yir-application-scalar-script-v1`.

This is a bounded, non-window ingress profile over `ApplicationEventPump`.
It selects an embedded application registration and drives its Nuis open/event/close
callbacks. It neither uses `WindowSession` nor implements application state,
image algorithms, provider selection or device cleanup in a new host adapter.

## Packaging And Input

`yir-pack-aot <module.yir> <output-dir> [frame-scale] --headless` requires at
least one registered application session. The output contains an executable,
embedded YIR and the existing statically linked host runtime. Its generated C
source is only a process-entry bridge to `nuis_application_script_main`.
No AppKit/Objective-C host or prerendered fallback frame is selected.
Neither the standalone packager nor ordinary headless `nuis build` emits or
advertises unused CPU LLVM IR/shims, or claims AppKit's affinity worker. The compiler
selects an explicit verified-YIR checkpoint: frontend/NIR checks, registered Nustar
semantics, project links, ABI, application registration and FFI ownership checks
all precede packaging. CPU LLVM is not requested; failure is never caught and
converted into an empty intermediate. Native/window builds still require a real
LLVM checkpoint. Native lowering remains a separate path; an unbound
scalar parameter there now produces a function-context diagnostic, not a panic.

The bundle declares:

```text
cpu_host_binary_mode=embedded_yir_headless
runtime_bootstrap_mode=embedded_yir_session
application_script_contract=nuis-yir-application-scalar-script-v1
application_provider_drain_contract=nuis-yir-provider-session-drain-v1
application_yir_fnv1a64=<embedded-source-content-hash>
```

The process accepts:

- `--application-session ID`: required embedded registration ID.
- `--open-args I64,...`: required; an empty value means no arguments.
- Repeated `--event-args I64,...`: zero to 64 ordered event deliveries.
- `--close-args I64,...`: explicitly call close, then await provider Finish.
- Alternatively, `--cancel-after-events`: abandon without invoking close.
- `--drain-provider`: only with cancellation; observe explicit provider drain.

Each argument list has at most 16 values. The whole script has one 180-second
deadline, not a fresh budget per callback. IDs are 1..128 UTF-8 bytes; argv entries
are bounded to 512 bytes. Duplicate, unknown, excess, malformed and contradictory
options fail before session startup. All open/event/close arguments are checked
against the registered application signature before any callback is admitted.
This profile supports i64 ingress, not arbitrary
strings, pointer arguments, interactive stdin or a general scripting language.

## Termination And Ownership

There is only one outstanding operation and one consumed reply at a time.
The host observes accepted states and trace frames without retaining a history.
RGBA frames report byte count and content hash; they are observations, not a
success artifact or authority to reuse resources.

Close yields success only after lifecycle validation and provider Finish.
Cancellation occurs only after the preceding replies passed validation, drops
the pump and waits on its independent retirement ticket. It does not synthesize
Nuis close, a terminal application outcome or parent delivery. Host-only retirement
is not provider retirement. The headless and AppKit adapters use the same safe
receipt-to-exit classifier: an affirmed drain without a latched fault maps to 130,
not success. Replay-only, failed, missing or mismatched drain evidence maps to 1.
The supervising provider launch policy must also validate its typed Drained
terminal; neither exit130 nor EOF proves retirement.

A callback error or timeout aborts the script before its requested terminal
operation. There is no automatic cleanup, retry, reconnect, budget reset or Finish
fallback. Already admitted effects are not rolled back. An in-flight worker can
outlive a timeout; the bounded host wait does not preempt FFI or device work.

Exactly one nonempty provider source must be explicit:
`NUIS_YIR_PROVIDER_DISPATCH_SOCKET` for the current Unix IPC adapter, or
`NUIS_YIR_PROVIDER_RESULT_STREAM` for identity-bound replay. Neither source,
both sources, or an empty path fails. No reference-device fallback is selected.

## Ordinary Build And Launch

Select `--packaging-mode headless-aot-bundle` on `nuis build`, or the same
`packaging_mode` in the project manifest. It actually invokes the headless
packager; it does not relabel a window binary. Explicit packaging/CPU target
overrides participate in the compile-cache identity. A restored manifest must
match the requested profile; default builds cannot adopt a headless cache entry.
Project manifest selections already participate through the project fingerprint.

`nuis run-artifact` accepts the same scalar-script arguments as the binary.
It requires a verified headless build manifest and compiled artifact, one
hash-bound bundle/YIR/binary entry each, exact capability versions and a matching
executable image. A different binary in the same directory is not admitted.
The YIR content hash and all scripted signatures are checked before provider
preparation. These are consistency checks, not cryptographic publisher trust or
protection against concurrent hostile filesystem replacement.

The manifest's `nuis-headless-build-inputs-v2` section declares
`headless_compiler_checkpoint = "verified-yir-v1"` and carries exact bundle,
source, tokens, AST, NIR, YIR and compiler-stage-handoff bytes as hex. Bundle and
handoff are capped at 1 MiB each, source at 16 MiB, and each other input at 32 MiB;
the aggregate decoded limit is 64 MiB. Independent artifact verification and
relocation restore these inputs, verify hashes and the source-to-YIR SHA-256 chain,
check canonical projections and source/token consistency, and reject missing,
duplicate, changed inputs or an LLVM artifact claim. Rebuild older v1 headless
artifacts; an incompatible cached manifest triggers a fresh build.
They never rebuild capability declarations from the current installation or read
the original source directory. Failed verification also removes its owned
temporary directory.

Window, parent-window, JSON and frame-export options cannot be mixed with an
application script. A headless launch without a script is rejected. Live
frontdoor cancellation requires `--drain-provider`: host-only retirement remains
a direct-entry facility, not a substituted live provider receipt. Normal close
and typed non-success drain share the existing prepared-provider lifecycle and
180-second supervisor bound. Cancellation never publishes successful launch,
replay or trace replacement evidence.

## Reproduce

Run from the repository root with one Cargo job. These tests exercise a compiled
headless process with a protocol peer, without needing GPU hardware:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-pack-aot --test headless_session -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --test provider_application_session script:: -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test headless_checkpoint -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test checkpoint_inspection -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test checkpoint_workflow -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib aot_application_bundle -j 1
```

The separate real M2 Metal test builds the Nuis image showcase through the ordinary
headless build route, checks same-profile cache restoration and rejects tampered
capabilities, mismatched binaries and invalid scripts before provider work. Direct
and frontdoor launches verify normal Finish, one/two-frame typed drain, worker
removal and preservation of prior success evidence. The binary's GPU frames have
exact RGBA hashes, no AppKit dependency, and it rejects replay-only drain:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis headless_metal_session_uses_shared_lifecycle_without_appkit -j 1
```

For a live M2 Metal launch from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- build examples/projects/domains/ns_nova_image_showcase build/ns-nova-headless --packaging-mode headless-aot-bundle
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- run-artifact build/ns-nova-headless --application-session window --open-args 160,120 --event-args 0,0 --event-args 1,32 --close-args 1,0
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- run-artifact build/ns-nova-headless --application-session window --open-args 160,120 --event-args 0,0 --cancel-after-events --drain-provider
```

The final command intentionally returns 130, not success. `window` is the sample's
registered ID; the headless launcher does not interpret it as a window policy.

For a manual replay, first build the
[image showcase](../../examples/projects/domains/ns_nova_image_showcase/README.md)
and complete its `--window-events 32` route, producing two identity-matched frames.
Then package the same YIR and replay those callbacks without a window:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p yir-pack-aot -- build/ns-nova-image/ns_nova_image_showcase.yir build/ns-nova-headless --headless
env -u NUIS_YIR_PROVIDER_DISPATCH_SOCKET NUIS_YIR_PROVIDER_RESULT_STREAM=build/ns-nova-image/nuis.runtime.provider-result-stream.toml \
  build/ns-nova-headless/ns_nova_image_showcase --application-session window \
  --open-args 160,120 --event-args 0,0 --event-args 1,32 --close-args 1,0
```

## Source Inspection

Set `packaging_mode = "headless-aot-bundle"` in `nuis.toml` to select the
verified-YIR checkpoint for `nuis check`, `nuis dump-yir`, the `nuisc` AST/NIR/YIR
dumps, benchmark/binding inspection and `nuis workflow --json`. Both a project
directory and its manifest file select this profile. A bare `.ns` input, an
unmarked project, and native/window/self-contained-image profiles retain the
existing LLVM path. A build-only `--packaging-mode` override does not persist into
later inspection commands. Unknown packaging profiles fail rather than being
treated as native defaults.

The selection occurs before compilation; no LLVM error is caught as a headless
fallback. Input resolution, NIR checks, registered YIR semantics, ownership and
application-session registration are still required. Dumps report the checkpoint
on stderr so their stdout remains parseable. Inspection creates no build outputs.

Workflow JSON exposes `compile_pipeline_checkpoint = "verified-yir"`,
`llvm_emit` stage status `not_requested`, and
`compile_pipeline_llvm_ir_bytes = null`. The native AOT heuristic
`compile_pipeline_ready_for_aot` remains false, with `build_headless` as the next
step. Native reports retain their real LLVM byte counts. Neither kind of source
inspection certifies a runnable image; artifact hash/identity checks and provider
admission remain separate and unchanged.

## Bounded Buffer Loops

The [callback fixture](../../tools/nuisc/tests/fixtures/buffer_while.ns) allocates a
small `ref Buffer`, fills it in a counted loop, reads the result and frees the
buffer before returning scalar application state. Supported inline loop bodies
contain scalar temporary bindings, ordered `store_at`/`load_at` operations,
`if`/`else` with branch-local temporaries and nested conditions, and admitted
source-level scalar helper calls.
Scalar integer division and remainder are supported too: constant evaluation
uses checked operations, registered execution reports invalid divisors/overflow,
and native scalar lowering traps before division by zero or `MIN / -1` (also
`MIN % -1`). This does not certify all specialized loop arithmetic opcodes.
Induction is `i64`, with strict `<` and `+ 1`, or strict `>` and `- 1`, and an
invariant scalar bound. These combinations reach the bound without induction
overflow; they are not a general loop-termination proof.

Lowering outlines the body to a collision-safe private helper, revalidates NIR,
and uses the existing `cpu.loop_while_i64_effect ... cpu scoped_call` contract.
Borrowed captures retain GLM lifetime edges. CPU Nustar owns the resumable loop;
the generic executor invokes registered functions with the same invocation fuel.
Buffer reads/writes carry effect edges at the memory operation, including reads
nested in a larger expression, and helper returns follow their effects. Native
`cpu.load_at`/`cpu.store_at` require length metadata and trap on negative or
out-of-range indices; reference execution reports an error instead of accessing memory.
Buffer owner selection merges the pointer and length from the same branch, so a
subsequent loop capture cannot inherit the other candidate's bounds.
This is not a proof of all allocation, pointer or FFI memory safety.

The [regressions](../../tools/nuisc/tests/buffer_while.rs) compare reference/native
results, repeated application callbacks, zero/one/descending loops, per-iteration
read/write order, declaration-order independence, failed-state admission and shared
fuel. Allocation, ownership transfer or arbitrary captured-state rebinding
inside an outlined body remain unsupported; explicit ordered i64 carries are described below. General steps and mutable
header reads are not silently hoisted.

### Branch-Local Execution

An outlined `if` snapshots its condition exactly once before either arm executes.
Each non-empty arm becomes a private registered function with a leading
`cpu.guard_return`; the unselected arm returns before reading, writing or evaluating
its body. Nested conditions stay inside their enclosing arm. Calls pass only named
scalar values and borrowed buffers, never eagerly evaluated branch expressions.
No new YIR instruction or image-specific executor path is introduced.

The generated helpers retain explicit guards rather than speculative selects, and
source-order edges keep even unused checked arithmetic before the helper return.
Dead-binding optimization retains division/remainder that may fail. Guard entries
and nested calls consume the existing shared callback fuel; a failed callback
cannot commit new application state or be retried as successful.

[Branch regressions](../../tools/nuisc/tests/buffer_while/branches.rs) compare
reference/native nested and empty branches, condition reads followed by mutation,
and untaken invalid indices, zero divisors and signed overflow. Selected invalid
branches still fail or trap. Reversed YIR node/body declarations retain ordering,
and generated names cannot capture user functions or local bindings. Textual lane
edge inference now extends a dependency-topological order instead of creating
backedges against transitive dependencies, including paths through other lanes.
Invalid explicit graphs are left for verification, not repaired. Branch-local
bindings do not escape. The carried-state and guarded-continue extensions below
admit specific rebindings and exits; allocation, ownership transfer, unbounded
loops, `break`, unstepped `continue` and early returns from the loop body remain
outside this Buffer-writing subset.

### Source Scalar Helpers

Source helper composition admits synchronous functions whose concrete parameters
and result are `i64` or `bool`. Bodies contain fresh scalar `let`/`const` bindings,
nested statement `if`/`else` and typed returns, including early returns. Every path
must return; bindings neither escape branches nor rebind captured locals.
Calls may compose other admitted helpers; signature,
arity, expression types and every reachable callee body are checked. Iterative
leaf-to-caller admission excludes missing or invalid callees and recursive cycles,
without recursive call-graph discovery. Buffer parameters, memory operations,
allocation, I/O, FFI, tasks and loops inside these source helpers are not
admitted. Unresolved generic signatures are not admitted either.

Only helpers reached from accepted outlined loop bodies are retained as direct
YIR functions. They are not replaced by an image-specific opcode or an inline-only
workaround. Calls and unused trapping arithmetic preserve source-order edges, and
nested invocations share the callback budget. Arguments may read Buffers, but are
evaluated left to right inside the current iteration and selected branch, never
hoisted across its guard. Loop-header calls remain unsupported; compute an
invariant scalar bound before the loop instead.

[Helper regressions](../../tools/nuisc/tests/buffer_while/scalar_helpers.rs) cover
diamond dependencies, real `call_i64`/`call_bool` nodes, ordered read arguments,
untaken argument/callee traps, rejection of transitive effects and recursion,
reordered YIR declarations, repeated callbacks and failed shared-fuel admission.
Admission unit tests also exercise a 4096-function dependency chain without
recursive catalog traversal; this is not a runtime recursion-depth guarantee.

Scalar-helper control flow is outlined into typed guarded functions. Each condition
is evaluated once. Unselected arms return a typed inert value before branch-local
bindings, arguments or callees run; a final select uses only the guarded results.
Fallthroughs call a shared continuation, while early returns bypass it. Suffixes
are not duplicated into both arms: a 64-branch regression checks linear function
growth. Only named scalar captures cross these generated boundaries. In contrast,
ordinary source-call arguments still evaluate left to right before the callee's
entry guard. A callee's early return never returns from its caller or loop.
Inactive generated guards still consume fuel; this is not a zero-overhead branch
representation or a new runtime opcode.

[Control-flow regressions](../../tools/nuisc/tests/buffer_while/scalar_control.rs)
exercise nested/fallthrough/empty arms, both scalar result types, early returns,
generated-name and sibling-scope isolation, ordered entry arguments, untaken
division/remainder/overflow failures and selected traps. Reordered YIR declarations
and repeated callbacks retain result, effect ordering and shared-fuel failure policy.

Imported CPU modules retain referenced private implementation helpers in an
owner-local signature scope, separate from their export table. Own-module names
win over same-named consumer/import functions. The
[scope regressions](../../tools/nuisc/src/frontend/tests_frontend_core/private_helper_scope.rs)
check transitive private calls through a Buffer loop and reject direct private
access from consumers or sibling modules in either import order. This fixes the
single-file/project discrepancy exposed by PixelMagic, without making its scalar
helpers public or adding a library-specific compiler exception.

### Scalar State Across Buffer Writes

Existing `i64` bindings can now be rebound in source order throughout a bounded Buffer-writing
iteration, including nested `if`/`else` arms and repeated updates. Their updates may use fresh locals,
ordered Buffer reads and admitted scalar helpers, including their guarded branches.
The incoming value is available to the iteration's writes and helpers. The bound
and step cannot depend on these changing bindings. Rebinding fresh iteration locals,
non-i64 state and ownership/control transfers are not admitted by this extension.
The recursive bounded-loop composition described below is the explicit exception
for updating fresh outer-iteration locals through an inner loop.

The outlined helper returns the updated scalar through the explicit action
`cpu.loop_while_i64_effect ... cpu scoped_call_i64_carry <arity> <callee> <seed> <operands...>`.
Arity includes callee and seed; exactly one helper operand is `$carry`, while
`$current` supplies the induction value. Other operands are named scalar or borrowed
Buffer captures, not owned-transfer markers. Shared payload validation keeps the
registry, CPU implementation and LLVM lowering aligned. GLM reads the seed and
captures, not the placeholders. Both seed and result must be i64, with no implicit
i32 widening. No new generic-executor special case is needed.

CPU Nustar updates its private carry only after a successful helper return. LLVM
stores the actual return into an explicit i64 loop slot. Both produce the existing
`LoopState { current, carry0 }` shape; ordinary `cpu.field` projections rebind the
source variables. A zero-trip loop returns the incoming seed, even when the update
would replace rather than read it. Projected values can feed subsequent loops.
Calls still share the enclosing invocation fuel; failed application callbacks do
not commit new scalar state. This is not rollback of Buffer effects already executed.

[Carry regressions](../../tools/nuisc/tests/buffer_while/scalar_carry.rs) cover
zero/one/descending trips, replacing updates, carry-dependent writes, write-before-read
ordering, subsequent loops, malformed metadata, strict seed types, GLM dependencies,
checked failures, shared fuel and failed-state admission in reference/native paths.

Multiple carries use one aggregate-return helper, not multiple independently
evaluated update calls. Rebindings retain source order: a later update sees
the new value of an earlier binding. The action is
`cpu.loop_while_i64_effect ... cpu scoped_call_i64_carries <arity> <callee> <layout> <operands...>`.
The flat layout is `State{carry0:i64;carry1:i64;...}`; every slot appears exactly once
as `$owned_struct_carry:<index>:<named-seed>` in the helper operands. Arity includes
callee and layout. Named captures and `$current` retain their existing meaning.
The shared layout parser bounds metadata size; there is no opcode per state count.
CPU validates the complete returned type, field sequence and strict i64 values
before advancing. LLVM requires the callee's declared return layout to match,
including its early-return paths, then loads the returned aggregate into private loop slots.
Direct LLVM emission now extends the explicit graph's topological order when
adding implicit serial queues, rather than creating textual-order backedges after
declaration reordering. Native entry emission also honors the declared YIR result
instead of whichever scalar happened to be emitted last. Implicit unit-main returns
join the last statement effect before returning zero. When a declared entry result
is not natively lowered, partial LLVM remains inspectable but ends in `llvm.trap`
and `unreachable`, never an unrelated successful result. Results are ordinary
`LoopState { current, carry0, carry1, ... }` fields, including all zero-trip seeds.

This first native implementation reuses the owned-aggregate return ABI: it allocates
and releases an aggregate each iteration, with additional aggregates for multi-state
branch returns. It is not allocation-free or a performance
parity claim. [Multi-carry regressions](../../tools/nuisc/tests/buffer_while/scalar_carries.rs)
cover sequential rebinding, 2/3/12 slots, generated-name isolation, replacing
updates, subsequent loops, reordered declarations, layout/seed drift, ordered traps,
GLM dependencies and shared-fuel failure without callback-state commit.

Branch helpers return the scalar or flat aggregate of the slots changed by either
arm. Their leading guard returns incoming parameter values when the arm is not
selected, rather than resetting a slot to zero. The predicate is captured before
any arm updates state or Buffer contents. Calls and projections remain in source
order; later branches see earlier updates, while branch-local reads, writes and
arithmetic remain behind their guard. Guard seeds admit only existing i64 parameters
or an exactly matching flat i64 aggregate, not calls or speculative expressions.

[Branch-carry regressions](../../tools/nuisc/tests/buffer_while/branch_carries.rs)
cover nested and asymmetric arms, zero/descending trips, replacing/repeated updates,
ordered traps, reordered YIR declarations, subsequent loops, callback fuel and
failed-state admission. A 32-branch case also checks bounded helper growth and name
isolation. It exposed reverse-substitution growth in pure-helper collection: the
collector now preflights the whole body and bounds expression expansion before
cloning. Exceeding that analysis budget declines expression inlining, not the
source program's branch count. This is not a bound on all compiler optimizations.

### Nested Bounded Loops

Counted Buffer-writing loops now compose recursively through the same private
function and scoped-call contracts, including loops inside guarded `if`/`else`
arms. Each level retains its own current value, invariant bound and i64 carries;
there is no flattening, unrolling or fixed two-dimensional image opcode. A child
may perform only scalar updates when its enclosing loop tree also writes a Buffer.
Every level uses strict `<`/`+ 1` or `>`/`- 1`. An inner body or induction step
cannot modify any ancestor's induction variable or bound inputs. A bound may read
an outer current value but is evaluated only when that child loop is reached.

Fresh inner counters reset on each outer iteration. A counter initialized before
the outer loop instead survives as an outer carry, including zero-trip children.
Child results update both its induction and its scalar carries. A guarded branch
returns updated locals already visible at branch entry, but does not export
locals created inside the branch. Untaken branches preserve those incoming seeds
without evaluating child bounds or Buffer accesses. Buffer effects remain ordered
and are not rolled back on a later failure. All levels consume the same callback
fuel; exhaustion does not commit new application scalar state.

[Recursive regressions](../../tools/nuisc/tests/buffer_while/nested_loops.rs) check
reference/native results, zero and descending trips, outer-dependent bounds,
persistent and reset counters, guarded locals, source-ordered traps, reordered YIR,
shared fuel and an eight-level/four-sibling helper-growth case. They exposed stale
literal propagation across NIR branches: joins now invalidate changed literals,
constant-selected branches use the selected outgoing environment, and arm binding
pruning waits for enclosing-block liveness. These are compiler fixes, not runtime
or PixelMagic exceptions. Non-i64 carries and general control exits remain open;
the following subset adds explicit-step `continue`.

### Guarded Continue

A bounded Buffer-writing loop may end an `if`/`else` path with `continue` when its
immediately preceding statement is the same explicit induction update as the
loop's canonical final step. Strict `<`/`+ 1` and `>`/`- 1` remain required:

```nuis
while index < end {
  if skip(index) {
    let total: i64 = total + 1;
    let index: i64 = index + 1;
    continue;
  }
  store_at(buffer, index, value(index));
  let index: i64 = index + 1;
}
```

The compiler removes the duplicated path step and uses a private per-iteration
i64 flag to guard the entire remaining suffix. The existing scoped-call driver
performs the source-requested step exactly once after the helper returns. Nuis does
not invent an increment for an ordinary `while`: missing, mismatched or nonterminal
continue-path steps are rejected by this outlining path. Writes and carry changes
before `continue` persist; skipped reads, arithmetic, source-call arguments and
child-loop bounds are not evaluated. Each nested loop normalizes only its own
exits, resets its flag every iteration, and never exports it into application state.
No YIR instruction or runtime/backend-specific control mechanism is added.

[Continue regressions](../../tools/nuisc/tests/buffer_while/continues.rs) check
reference/native parity, zero/all/no-continue trips, signed integer boundary steps,
nested/sequential predicates, source-ordered traps, nested-loop scope, reordered
YIR, shared fuel, rejected steps and linear helper growth with 32 guards.

### Guarded Break

Bounded Buffer-writing loops also admit terminal guarded `break`, including nested
conditions and coexistence with explicit-step `continue`. Each loop owns its exit;
the breaking iteration retains prefix writes and scalar updates, skips its suffix
and exits before the canonical induction step. A child break leaves the parent
running and exposes the child's actual exit counter. This is not no-op execution
of the remaining iterations. Induction mutation before break, unstepped continue
and arbitrary loop bodies remain outside this subset.

`cpu.loop_while_i64_effect` uses the registered action
`cpu scoped_call_i64_carries_break`, sharing the flat `carryN:i64` layout, named
captures and indexed seeds of `scoped_call_i64_carries`. The last slot is private
control: seed `0` is required even for zero trips; return `0` advances and return
`1` exits before stepping. The layout can contain only that control slot. Every
field and the control value must validate before state commit. Other values or
types fail in registered execution and trap or fail lowering in LLVM. Aggregate
storage is released before taking the exit; no generic executor special case or
engine-specific action is introduced. GLM retains the real seeds and Buffer captures.
The compiler records the normalizer's canonical control binding explicitly; neither
a generated-looking function name nor an ordinary user struct with the same shape
authorizes this control interpretation. General integer data is not silently narrowed
to a 0/1 signal.

[Break regressions](../../tools/nuisc/tests/buffer_while/breaks.rs) cover reference
execution, regenerated native LLVM and registered sessions, including a trillion
upper bound that exits after three calls within 1000 shared fuel, skipped traps,
nested scope, mixed exits, malformed controls and reordered YIR. PixelMagic's
packaged recoloring operation below now adds real Metal application evidence.
A fault injection also omits a returned field producer's native function lane.
Declared helper results now use the same fail-closed fallback as entry results:
partial LLVM remains inspectable but traps instead of treating the last integer
as an aggregate pointer. This does not repair or certify the full native callback graph.

### Packaged Pixel Loop

[PixelMagic's generator](../../stdlib/pixelmagic/lib/pixels.ns) now uses
bounded row/pixel loops with composed scalar coordinate/color and row-range helpers, branch-local
input guards before coordinate division, a phase-dependent early return, and explicit
guarded red writes followed by explicit-step `continue`; blue writes occur only
on fallthrough, not through recursive pixel filling or arithmetic color selection.
Partial first/last rows are clamped before entering their pixel loop;
an empty range returns without iterating. Both paths assert real source helper calls
and an inner loop inside an outer iteration function in YIR. The
[native/reference test](../../tools/nuisc/tests/pixelmagic_buffer_loop.rs) checks
every pixel for both 32x24 phases, a partial region and an empty region.
`fill_checkerboard_region_stats` returns `CheckerboardStats { red_count, checksum }`
using two carried accumulators during the same pass. The red count updates inside
the selected red-pixel write arm, which also updates checksum before continuing;
fallthrough blue writes update checksum separately. The checksum is the sum of
the packed pixel values, not a cryptographic digest. Invalid input returns `(-1, 0)`
before writes; an empty range returns `(0, 0)`. The red-count and boolean fill APIs
delegate without changing their success contract. Native/reference tests check
exact counts and sums as well as pixel bytes, including partial and empty ranges.
The [CLI regression](../../tools/nuis/tests/headless_image_loop.rs) builds the
ordinary image showcase with `headless-aot-bundle` and launches it through
`run-artifact`. The carried generator and its real branch-local counting helper are present in
packaged YIR, including the red branch's two statistics and private continue flag.
After filling, `recolor_run(pixels, 0, 32, pixels[0], 4278255360)` marks the first
same-color run green, stopping before the first different pixel. The callback
checks `PixelRunStats { end: 4, written: 4, checksum: 17113021440 }` before submitting.
Its shared `scoped_call_i64_carries_break` action remains in packaged YIR, with
two statistics and the private exit slot. The [native/reference color-run test](../../tools/nuisc/tests/pixelmagic_buffer_loop/color_runs.rs)
checks 22 boundary cases, all output pixels and exact write counts; invalid input
does not mutate storage and the nonmatching tail remains untouched.
This combined continue/break workflow passes the M2 route with two actual Metal
output hashes, including the 20x5 magenta marker after inversion. Every output byte
matches the independent oracle and direct-session replay. Replay verifies five
recoloring helper invocations with stop index 4, plus exactly 768 fill writes,
four marker writes and the post-snapshot mutation per callback.
Wrong callback arguments or executable/YIR drift are rejected before
application effects. Exhausted packaged replay produces neither Close nor success,
and preserves the earlier replay stream.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test buffer_while --test pixelmagic_buffer_loop -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p yir-pack-aot -p yir-runtime-host -j 1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test headless_image_loop -j 1 -- --test-threads=1
```

The last command is a macOS Metal device regression, not Linux/Windows evidence.
CPU callbacks in the packaged host still execute embedded YIR; the native parity
test independently compiles the generator, not the complete live callback ABI.
Induction changes before `break` and `continue` without the explicit matching step remain outside this subset;
ordered i64 accumulators do not certify arbitrary loop-carried state or fully native callbacks.
The default-AOT compiled-image regression now passes its LLVM checkpoint without
changing packaging mode. `cpu.guard_drop_owned_bytes_return` and
`cpu.branch_drop_owned_bytes_return` use the declared function aggregate layout,
validate all return arms before emitting cleanup, and pack nested fields in ABI
order. Missing/mismatched layouts and directly known dropped-Bytes aliases reject
rather than treating an unrelated integer as a returned aggregate pointer.
The [cleanup return regression](../../tools/nuisc/tests/owned_cleanup_return.rs)
compiles pure Nuis helpers into a native binary and compares all four branch
outcomes with reference execution, including reversed declarations. A native
live-Bytes probe must be zero at completion. Two terminal source branches now
provide a real helper result and deduplicated, ordered cleanup effects.
The generic executor observes the registered module's `function_exit` hook after
successful node execution; the hook neither repeats work nor resets shared fuel.
Non-CPU registration, nested-call continuation and rejected exits have regressions.
The compiled image host still uses embedded YIR for CPU callbacks. Passing the
LLVM checkpoint is not evidence of complete native callback dispatch; other
aggregate early-return operations and resource-bearing callback states remain open.
The separate [native scalar bridge](nuis-native-scalar-session-bridge-v1.md) now
executes a bounded registered open/event/close fixture through static exports,
without interpreting YIR or replaying main. Its strict scalar-only admission and
typed slot checks have native/reference regressions. A separate explicit packer
profile now links it into the shared application-session lifecycle, with last-state
preservation, one cleanup attempt and exact YIR binding before open. It now has an
explicit `native-session-aot-bundle:<id>` build mode and typed
`run-artifact --native-session <id>` ingress, including bound LLVM checkpoints,
cache identity and independent artifact materialization. This provider
scalar-script host still does not select that profile; native calls do not inherit
reference fuel or provider capabilities, and no interpreter fallback is permitted.

## Current Boundary

Ordinary `nuis build` and `nuis run-artifact` now select and admit this profile.
Headless build selection removes the unused CPU LLVM prerequisite without
fabricating an empty intermediate or weakening the verified stage handoff.
Checkpoint-aware source inspection now follows the explicit manifest selection.
Bounded Buffer-writing callbacks, including composed scalar helpers and branch-local pixel writes, now pass
the packaged build/run-artifact image workflow described above.
Compound-condition `while` callbacks now pass the verified-YIR boundary and execute
their current/carry state rather than returning trace-only unit values. CPU Nustar
owns the scalar loop state machine; the generic reference executor only drives
registered resumes and existing YIR function calls, sharing invocation fuel.
Synchronous and awaited steps, pre/post-update control, `&&`/`||`, mixed break/continue
and add/keep carries have reference/native parity tests. Registered callback tests
include zero iterations and untaken conditions. Arbitrary loop bodies, payload-bearing
carries and mixed-action `flow_and` descriptors are not certified by this fix.
Reference execution also has a 100000-resume per-node ceiling when no session fuel
is supplied; that ceiling is not a native ABI or native loop limit.
Standalone artifact verification preserves all compiler inputs; a relocated full
device launch still needs its provider environment and separate execution evidence.

This is embedded-YIR execution, not fully native CPU callback lowering or a
self-contained Nsld application image. The registered live provider remains an
external execution scope. macOS and Linux have host build branches; the recorded
real hardware evidence is M2 Metal, not Linux GPU or Windows transport certification.
Parent cancellation, arbitrary resource reuse and recovery remain separate work.

Related contracts: [application session](nuis-yir-application-session-v1.md),
[cancellation](nuis-yir-application-cancellation-v1.md),
[provider drain](nuis-yir-provider-session-drain-v1.md).
