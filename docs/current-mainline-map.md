# Current Mainline Map

This is the short reading map for current implementation, not a history catalog.
When documents disagree, inspect the code/tests and current tensor evidence
before changing a capability claim.

## Fast Reading Order

The current `beta-0.12.*` priority is the ns-nova application-led dependency
chain agreed in [beta 0.11](versioning/nuis-beta-0.11-application-led-mainline.md).
The [beta-0.12 snapshot](versioning/nuis-beta-0.12.0-snapshot.md) records
`505c820c` (`beta-0.12.2`); Git remains authoritative for later patches.

1. [Repository overview](../README.md)
2. [Mainline selection and acceptance coordinates](reference/nuis-development-tensor-mainline.md)
3. [Machine-readable application boundary](reference/nuis-ns-nova-application-lifecycle-v1.toml)
4. [Stateful window contract](reference/nuis-yir-window-session-v3.md)
5. [Cancellation and host retirement](reference/nuis-yir-application-cancellation-v1.md)
6. [Runnable image application](../examples/projects/domains/ns_nova_image_showcase/README.md)
7. [Focused validation checklist](versioning/nuis-beta-0.12.0-release-checklist.md)

Use the [versioning index](versioning/README.md) for the earlier
[beta-0.10 migration entry](versioning/nuis-beta-0.10.0-self-hosting-entry.md),
[beta-0.6 foundation](versioning/nuis-beta-0.6.0-mainline-entry.md) and older
alpha/pre-alpha anchors. Those checkpoints must not replace current behavior.

## Start Here

Run from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run -p nuis -- dev-tensor --json
```

The current goal is `standard-library/ns-nova/interactive-image-workflow`.
Its selected prerequisite is
`standard-library/ns-nova/persistent-application-session`.
Selection follows the [declared dependency plan](reference/nuis-development-tensor.mainline.toml),
not the globally lowest percentage. Correctness regressions may interrupt it.

Current session evidence:

- Registered Nuis open/event/close helpers retain scalar aggregate state and
  one provider connection across independent host events.
- Compiled AppKit/Metal regressions check input, actual image pixels, replay,
  worker/cache reuse, explicit cleanup and failure-preserving exit.
- An explicitly selected CPU parent opens before its child and receives one
  owned terminal outcome; its successful cleanup cannot certify child success.
- Pump, window and C ABI expose one independent cancellation ticket.
  It survives window destruction without implicit close, a fabricated terminal
  outcome or parent delivery. The packaged host now consumes it through explicit
  `--window-cancel-after-events`, with real Metal and compiled replay regressions.
- Cancellation admission, host-scope retirement and provider/device resource
  retirement are distinct. Explicit registered-worker drain now has provider-owned
  evidence; an opt-in host-library cancellation ticket separately observes it.

The [provider-session drain boundary](reference/nuis-yir-provider-session-drain-v1.md)
now distinguishes explicit worker-scope retirement from successful completion,
without publishing replacement replay. Pump/window/C ABI drain admission is now
connected through the generic provider-scope boundary, with first-fault and damaged
exchange guards. The packaged script now accepts opt-in `--drain-provider` behind
an exact bundle capability; the shared policy requires a typed Drained terminal
and cancellation exit, rejecting Finish before publication. It never publishes
cancelled work as successful launch/trace evidence. The terminal contract and
launch policy have no OS/window/backend dependency. A separate
[headless scalar-script profile](reference/nuis-yir-application-scalar-script-v1.md)
now packages the same pump without AppKit, with compiled protocol and real M2
image/drain evidence. Ordinary `nuis build/run-artifact` now select
`headless-aot-bundle`, verify exact capabilities, scalar signatures and image
identity before preparing providers, and reuse the existing typed launch policy.
Explicit host selections are cache-isolated. Headless builds now consume a
verified-YIR checkpoint without CPU LLVM codegen. The complete source-to-YIR
handoff travels with the artifact; no empty LLVM file substitutes for a stage.
Manifest-selected headless check/dump, benchmark/binding inspection and workflow JSON
now use the same checkpoint, with `llvm_emit=not_requested` and no invented LLVM
byte count. Default native diagnostics remain in place. Bounded Buffer-writing
callback loops now outline to private helpers under the existing scoped-call YIR
contract. CPU-owned cooperative execution invokes the helpers rather than logging
calls; expression-level memory effects preserve source order and native indices
are checked. PixelMagic now uses these loops instead of recursive pixel filling.
The actual CLI build/run-artifact regression passes two M2 Metal frames and exact
direct-session replay, rejects executable/YIR drift and invalid callback arguments,
and retains prior evidence on replay exhaustion. Independent native/reference
tests compare every generated pixel; invalid scalar division/remainder no longer
panics in constant evaluation or reaches undefined native arithmetic. Conditional
pixel writes now use nested guarded helpers and one-time condition snapshots;
untaken invalid reads/writes or arithmetic do not execute. Scalar-only branch
traps cannot be discarded as dead bindings. Source-level scalar helper composition
now admits synchronous, acyclic `i64`/`bool` callees, with transitive
body checks and real ordered calls under shared fuel. Buffer-read arguments remain
inside the selected branch. PixelMagic's two-level coordinate/color helpers exercise
this surface, with imported private helpers retained in their owner's scope rather
than exposed as public functions or resolved against another module's same-named
implementation. Nested scalar-helper `if`/`else` and early returns now use typed
guarded function blocks with shared fallthrough continuations, rather than
speculating branch-local arguments or callee bodies. Regressions cover scope
isolation, both return types, traps, source order and shared callback fuel.
Multiple source-ordered i64 accumulators now cross Buffer-writing iterations through a
shared scoped-call contract and ordinary LoopState field projections. Updates keep
source order; zero-trip loops preserve all seeds and only fully validated helper
returns update the private state. PixelMagic exposes fill-and-statistics with exact
native/reference pixel, red-count and pixel-sum tests. Native aggregate returns still
allocate and release storage per iteration and multi-state branch return; this is not performance-parity evidence.
The carried generator also passes ordinary M2 build/run-artifact with two exact
Metal frames, direct-session byte parity and unchanged failure admission.
Branch-local and repeated scalar updates now compose with those writes. Guarded
arms preserve incoming seeds when unselected; nested arms and subsequent branches
see source-ordered updates. PixelMagic counts red pixels within its red write arm.
Nested bounded Buffer-writing loops now compose through the same private functions,
with independent counters, scalar carries, guarded child bounds and shared fuel.
PixelMagic uses row/pixel loops, not a fixed two-dimensional runtime operation.
NIR branch joins invalidate stale literals and preserve live outgoing assignments.
Guarded `continue` now preserves prefix writes/carries and skips the whole suffix,
including nested loop bounds, when the source path explicitly performs the matching
unit step. Each loop owns its control scope; no runtime-specific opcode is added.
PixelMagic uses this path after finishing each red pixel.
Guarded `break` now exits before stepping through a shared flat-i64 control contract
consumed by the CPU registered driver and LLVM. Native/reference and session tests
preserve prefix state, reject invalid control values and stop a trillion-bound loop
within 1000 shared fuel. PixelMagic `recolor_run` now passes ordinary M2 headless
build/run-artifact: four same-color pixels are marked before real Metal inversion,
the fifth helper invocation stops at index 4, and exact pixels, writes, identity
admission and replay remain verified. Owned-Bytes cleanup returns now propagate
declared aggregate layouts through LLVM, restoring the default-AOT image checkpoint.
Nested aggregate cleanup helpers have native/reference parity, declaration-order
invariance and zero live native Bytes after execution. The executor asks each
registered module for function exits instead of dispatching on CPU operation names.
A [static scalar callback bridge](reference/nuis-native-scalar-session-bridge-v1.md)
now executes registered open/event/close helpers natively, with nested typed slots,
reference parity, in-place state and rejection before callback entry. This selected
profile now admits bounded acyclic scalar helper calls and bounded i64 loops with
constant or runtime-checked induction, but excludes resources and providers. Explicit packer
selection now uses the shared application-session host with static exports, full
YIR identity admission, last-valid-state preservation and one cleanup attempt.
Production-host tests reject stale objects and native failures without interpreter
fallback. Explicit `native-session-aot-bundle:<id>` selection now passes ordinary
`nuis build` and `run-artifact --native-session`, with real LLVM checkpoint
regeneration, cache registration isolation and standalone materialization after
removing the original output. Required documentation/package metadata is restored
with bound inputs rather than read from the old host paths. Nested helper calls
now retain all five scalar kinds, reject recursive/effectful/type-drifted closures,
and survive cache reuse and independent artifact restoration. Counted i64 loops
now compose with those helpers, source-ordered add/multiply carries and zero-trip
seeds. Constant induction is proven before emission; runtime i64 start, limit
and step receive an in-place preflight before loop entry. Both require finite,
non-wrapping induction within 65536 iterations per loop. The CPU registered driver now returns real
plain-chain state under shared reference fuel instead of logging a unit value;
shared loop input dependencies no longer duplicate effect edges. Real native
executions and the build/run-artifact restoration regression cover this subset.
A 1680-case native guard probe matches checked-step simulation across six
comparisons and add/sub, including signed extremes, zero trips and skipped guards.
Rejected inputs enter no loop iterations; six unmodified native trap executions
confirm process failure rather than a catchable callback/fuel result. Scoped
scalar loop-body calls now reuse the existing YIR action and LLVM emitter, with
discarded scalar results or one carried i64 return. Scoped targets join the same
bounded acyclic closure; captures require exact scalar kinds and caller-local
values. Six native/reference runs cover the scoped Nuis fixture and declaration
reordering. Two additional executables observe 36 callback cases and 84 actual
helper invocations, including all five scalar kinds, NaN/signed-zero bits,
pre-step carry order and zero-trip preservation. Three process-trap cases prove
invalid induction never enters the helper. The build/run-artifact restoration
regression now uses a multi-carry scoped-loop fixture. Flat multi-i64 scoped
returns now use the shared layout/seed contract and the same bounded helper
closure. Six more native/reference runs cover the multi-carry session fixture.
Six native executables observe 144 callback cases and 288 helper invocations
with 2/3/7 carries, reversed argument order, sequential updates, zero/one trips,
exact scalar captures and balanced real aggregate allocation/drop. Three more
process-trap runs reject invalid induction before the first helper call.
Aggregate helper reachability now includes registered, exported and noinline
roots, not only main; counter-only fallback rejects discarded outer updates.
Scoped guarded break now reuses the shared return/control contract: validate the
zero seed and binary returned control, release the aggregate, commit all carries
and exit before stepping. Pure scalar source loops reuse private normalization;
break-only loops need one control bit rather than an aggregate branch result.
The full bounded induction preflight still applies even for an immediate break.
Six native binaries cover 180 callback cases and balanced aggregate release;
eleven real traps cover seed/return/induction rejection. The frontdoor restoration
suite retains the multi-carry fixture and adds the guarded-break fixture.
Checked flat-i64 branch-helper returns now reuse ordinary aggregate calls with
exact return-layout, scalar-parameter and bounded-closure validation. Multi-state
guarded updates, break and explicit-step continue now execute natively, including
nested scope and both induction directions. Six nested-call binaries cover 144
callback cases and 576 inner helper calls with balanced real aggregate release;
selected-path tests skip an unreachable excessive-loop preflight but trap when it
is reached. The frontdoor adds the multi-state branch fixture through cache
isolation and standalone restoration. Checked i64 division/remainder now have
exact-kind native admission and reuse existing zero/overflow guards. Eight native
binaries compare 1840 callbacks against reference execution and a wide-integer
oracle; 20 dynamic and four literal invalid cases trap without returned state.
Four unselected invalid literal cases return safely. Frontend purity no longer
licenses arithmetic speculation: acyclic i64/bool helpers reuse guarded outlining.
Flat-i64 aggregate values now use the same outliner with neutral guards, existing
record captures and shared suffixes. Fourteen more native binaries compare 3220
callbacks across early/two-arm/nested returns, unused results/arguments and direct
registered-state branches; 32 dynamic and four literal invalid cases trap, while
four unselected literal cases return safely. Real allocation/drop probes stay
balanced after every successful callback and before the observed arithmetic leaf.
Ordinary fields retain declared names; scoped carry/control schemas remain strict.
Thirty-two source guards have linear helper growth. Single-induction counted
loops now compose in flat-value helper branches, including inline arms, prefix
work and shared suffixes. An iterative call-graph flag retains the loop boundary
even without checked arithmetic. Independent checked-step/reference probes cover
2090 accepted or skipped callbacks and 26 real traps, including zero iterations
before rejected preflight and the exact 65536-trip bound. Guarded helpers now also
admit ordered linear i64 carry updates through shared chained-loop preparation.
Another 2849 accepted/skipped callbacks cover all carry slots, wrapping arithmetic,
1/3/7-carry widths and selected-path evaluation; 27 real traps retain preflight
before any update and reached arithmetic failure. Extreme seeds exposed and fixed
Rust-debug-dependent scalar integer overflow in the reference CPU. No backend loop
opcode or Buffer admission was added. Full if/else carry updates now reuse the
existing conditional-chain opcode and LLVM emitter, with strict leaf-condition,
exact-i64 and metadata checks before emission. The reference CPU now executes the
same scalar conditional chains cooperatively rather than returning trace-only Unit.
Another 1357 accepted/skipped callbacks and 27 real traps check selected updates,
source order, all six comparisons, reversed operands, wrapping and preflight.
Omitted and explicit empty carry arms now share `keep` preparation without a new
opcode or ABI. Thirty-nine native binaries add 2502 accepted/skipped callbacks
and nine real traps, including own-value retention, earlier updated carry reads,
all six comparisons, variable widths, dynamic stride and the exact trip bound.
Six more typed lifecycle runs and reference-fuel failure preserve accepted state
and cleanup. Both-empty arms, invalid seeds and effectful/fallible updates still
reject. Pure comparison leaves now compose through existing `and/or` condition
trees and short-circuit LLVM blocks. Thirty-one native binaries add 1216
accepted/skipped callbacks and nine real traps; volatile comparison counters
agree with independent short-circuit evaluation, not just final state values.
Nested RHS kind drift rejects even on zero trips. A shared parser depth guard
also protects the general verifier, which runs before native admission.
Nested carry-update arms and arbitrary loop bodies remain fail-closed;
per-return allocation remains an optimization target.
The division, aggregate-division, aggregate-counted, aggregate-carried,
aggregate-conditional, aggregate-one-sided and aggregate-compound fixtures compose
with typed lifecycle and multi-state break/continue. All ten frontdoor regressions
pass, including cache isolation, tamper rejection and standalone restoration.
The passing default image host still executes embedded YIR.
This does not certify arbitrary loop bodies, richer carry payloads or fully native CPU callbacks.
Linux hardware and Windows transport
are not certified by the portable tests.
Cross-session resource reuse remains a separate, unproven boundary.
The default host-only path acknowledges its own scope, while the live provider
still treats EOF without Finish as incomplete execution. Ordinary quit still uses
close/Finish; parent cancellation remains unsupported. Do not import provider
registries, transports or concrete cancellation state into the window adapter.

This does not close resource-capability state, peer recovery, general multi-child
routing, richer image bindings, sustained performance, native CPU frame dispatch
or a self-contained Nsld application image. The broad engine coordinate is not
an engine-completion percentage.

## Current Truth By Layer

| Area | Current Reference |
| --- | --- |
| Development model | [Tensor protocol](reference/nuis-development-tensor.md), [mainline selection](reference/nuis-development-tensor-mainline.md) |
| Production architecture | [Software manufacturing direction](reference/nuis-software-manufacturing-architecture.md), [RC policy and current limits](reference/nuis-rc-cache-lifecycle.md) |
| Frontend/lowering | [Control flow](reference/control-flow-lowering-contract.md), [generic diagnostics](reference/generic-diagnostic-ownership-contract.md), [source encoding](reference/source-text-encoding-contract.md) |
| Memory/task safety | [NIR memory](reference/nir-memory-model.md), [task/GLM](reference/cpu-task-glm-contract.md), [thread/lock boundary](reference/cpu-thread-lock-boundary.md), [FFI pointers](reference/ffi-pointer-safety-boundary.md) |
| Runtime sessions | [Application lifecycle](reference/nuis-yir-application-session-v1.md), [window](reference/nuis-yir-window-session-v3.md), [terminal outcomes](reference/nuis-yir-application-outcome-v1.md), [parent pump](reference/nuis-yir-application-outcome-pump-v1.md), [cancellation](reference/nuis-yir-application-cancellation-v1.md) |
| Nustars | [Capability ownership](reference/nustar-capability-split-boundary.md), [multi-backend artifacts](reference/nustar-multi-backend-artifact-contract.md), [CFFI domain](reference/cffi-von-neumann-domain-contract.md), [provider IPC](reference/nuis-yir-provider-runtime-ipc-v4.md) |
| Linking/launch | [Native workflow](reference/nuis-native-artifact-workflow.md), [NSB protocol](reference/nuis-binary-format-protocol.md), [Nsld](reference/nsld-linker-frontdoor.md), [assembly gaps](reference/nsld-binary-assembly-gap-map.md) |
| Debugging/distribution | [Nsdb](reference/nsdb-yir-debugger-frontdoor.md), [Nsbdr](reference/nsbdr-bundler-frontdoor.md), [reusable capability boundary](reference/toolchain-galaxy-core-boundary.md) |
| Official Galaxies | [Std](../stdlib/std/README.md), [PixelMagic](../stdlib/pixelmagic/README.md), [WitSage](../stdlib/witsage/README.md), [ns-nova](../stdlib/ns-nova/README.md) |
| Compiler migration | [Readiness](reference/nuis-self-hosting-readiness.md), [data model](reference/nuis-compiler-data-model.md), [stage handoff](reference/nuis-compiler-stage-handoff.md), [candidate production](reference/nuis-compiler-candidate-production.md), [candidate-to-Nsld boundary](reference/nuis-compiler-candidate-nsld-materialization.md) |

The five bounded migration-preparation gates remain closed; actual compiler
responsibility transfer is separate. Use the readiness/component references for
canonical payloads, trust pins, differential/reproducibility and rollback
commands rather than reproducing their entire history in this router.

## Fast Example Routes

| Task | Start With | Evidence Boundary |
| --- | --- | --- |
| Basic host CLI | [filesystem_io_report_demo](../examples/projects/tooling/filesystem_io_report_demo), [stdin_runtime_demo](../examples/projects/tooling/stdin_runtime_demo), [cli_report_file_demo](../examples/projects/tooling/cli_report_file_demo) | Native executables with observable I/O and output-file checks. |
| Text statistics | [cli_wc_demo](../examples/projects/tooling/cli_wc_demo) | One 4096-byte read and ASCII separators, not complete streaming `wc`. |
| Simple file/image transform | [cli_pgm_invert_demo](../examples/projects/tooling/cli_pgm_invert_demo) | Checked small PGM input/output, not a general image-codec library. |
| Current application goal | [ns_nova_image_showcase](../examples/projects/domains/ns_nova_image_showcase) | Real Metal processing, compiled export and explicit stateful AppKit window; embedded YIR host runtime remains. |
| Small render/uniform route | [ns_nova_showcase](../examples/projects/domains/ns_nova_showcase) | Bounded frame loop, registered render resources and live/replay evidence. |
| Linker closure | [native_artifact_closure_demo](../examples/projects/tooling/native_artifact_closure_demo) | Inspect the selected finalizer and admission evidence; a planned image is not automatically runnable. |
| Compiler component | [bootstrap_structural_projection_candidate](../examples/projects/tooling/bootstrap_structural_projection_candidate) | Bounded candidate proof, not general self-compilation or replacement authority. |
| Linux device work | [CUDA bring-up](reference/linux-cuda-provider-bringup.md), [domain examples](../examples/projects/domains/README.md) | Run provider-specific gates on suitable hardware; absent hardware is unverified, not fallback success. |

The [observable CLI regression](../tools/nuis/tests/std_filesystem_smoke.rs)
builds and runs seven project routes, including PixelMagic/WitSage reports.
Those reports alone are not device-execution evidence. The separate Metal tests
check actual dispatched image data.

The packaged AppKit application now exposes standalone scripted cancellation
through `--window-cancel-after-events`. This does not add a Nuis intrinsic,
CFFI signature grant, parent cancellation route or device-retirement authority.

## Deep Routers

- [Documentation index](README.md) and [reference index](reference/README.md)
- [Project examples](../examples/projects/README.md), [source examples](../examples/ns/README.md), [freshness audit](examples-freshness-audit.md)
- [Std tooling](../stdlib/std/tooling/README.md), [filesystem](../stdlib/std/filesystem/README.md), [network](../stdlib/std/network/README.md)
- [Repository layout](repo-layout.md) and [file-line policy](repo-file-line-policy.md)
- [Application-led design](versioning/nuis-beta-0.11-application-led-mainline.md) and [long-range OS roadmap](versioning/nuis-long-range-heterogeneous-os-roadmap.md)

Rendering, control, ML and audio are an engine horizon, not four completed
workflows. Shared YIR/GLM/time contracts preserve independent Galaxy/Nustar
ownership. Hardware extensibility and scheduling efficiency require separate
measurements; a smoke test is not an advantage over MLIR or a mature language.

## Cleanup Rule

Keep this map short. Current behavior belongs in focused reference documents;
minor checkpoints belong in versioning, and examples state their own input,
backend and execution limits. Keep paths repository-relative. Update tensor
evidence and documentation drift guards after each accepted change, without
raising capability scores for documentation work.
