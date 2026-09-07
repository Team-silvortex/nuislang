# Beta 0.11 Application-Led Mainline

This recalibration was agreed on September 7, 2026 against Git checkpoint
`41cf5e9e` (`beta-0.11.3`). It does not create a new release or alter Cargo package
versions. Git history remains authoritative for subsequent checkpoints.

## Direction

`ns-nova application -> exposed foundation gap -> bug fix and regression ->
measured optimization -> incremental Nuis-owned toolchain replacement`

ns-nova is the application driver, not a reason to couple the compiler to one
engine or start four independent large projects. Its intended role is a
real-time heterogeneous world engine combining rendering, control, ML and audio
workflows under shared lifecycle orchestration. The first deliverable is
deliberately smaller than the eventual engine.

Use one growing image application as the initial acceptance workload:

1. Preserve Nuis-owned state across independent host events and frame requests.
2. Validate close, cancellation, provider failure, backpressure and resource release.
3. Support the registered image/parameter resources needed for interactive load,
   zoom, modification, redraw and export.
4. Establish sustained frame-time, memory/resource and data-transfer baselines;
   optimize actual bottlenecks while retaining output and error semantics.
5. Migrate an independently testable compiler component to Nuis and consume it
   in that application's real build, under existing selection/rollback rules.
6. Move the persistent CPU frame/event path from embedded YIR execution to native
   lowering with differential execution evidence.

This is the initial finite plan, not a requirement to finish the entire engine
before any self-hosting work. Review module migration at each accepted application
slice. A ready compiler component can move earlier by an explicit plan edit;
do not indefinitely postpone it by continually expanding the first goal.
Every increment carries its own regression tests and useful measurements;
the later sustained-baseline goal does not defer bug fixes or observed regressions.

The current lifecycle slice separates completed Nuis cleanup from final provider
success. [Terminal outcome v1](../reference/nuis-yir-application-outcome-v1.md)
adds a readonly scalar snapshot and a Nuis decoder, including late Finish failure.
It does not execute an automatic post-close observer: a new function context
would rerun global initialization. Parent Nuis delivery needs an explicit
ownership/effect/budget contract; closed state and resource authority stay separate.

## Engine Capability Horizon

Rendering, control, ML and audio are intended engine capabilities, not four
unrelated demonstrations or a promise that all four are implemented today.
The engine must compose their work inside one Nuis-owned application lifecycle.

| Capability | Engine Responsibility | Independent Capability Owner | Acceptance Boundary |
| --- | --- | --- | --- |
| Rendering | Scene, view, material and presentation policy | Registered Shader providers; PixelMagic for image algorithms | Render results retain input, resource and completion identity through presentation. |
| Control | Input, application state, simulation updates and control routing | Shared event/task contracts and replaceable host adapters | Accepted input changes carried state and affects the intended downstream work. |
| ML | Schedule model work and consume results in application/scene policy | WitSage for ML semantics; registered Kernel providers for execution | Real predictions affect the running application with explicit freshness, failure and resource rules. |
| Audio workflows | Route capture/playback, processing, mixing and timing within the application | Independent audio processing modules and registered device/compatibility providers | Audio buffers, processing dependencies and device completion are accounted for alongside visual and ML work. |

These are open-ended capability families, not a finite list of allowed backend
combinations. A workflow is not a hardware identity: ML does not imply NPU,
audio does not imply CPU, and possession of a device does not prove that a
provider executed on it. Register capabilities and validate actual execution;
do not infer placement from a provider preference or silently replace it.
Choose any new audio Galaxy/Nustar boundary from its contracts when implemented,
not by reserving a package name or embedding an OS audio API in the compiler.

### Shared Contracts, Independent Rates

All four families remain function/node work under YIR, with GLM-owned resources,
global clock contracts and lifecycle hooks. A shared clock contract is a
dependency partial order, not frame lockstep: rendering, audio sample/block
cadence, control events and ML jobs may advance at different rates. Explicit
clock bridges map physical completion evidence without inventing simultaneity
or forcing a global wait after every node.

The implementation must define bounded buffering, backpressure, deadlines and
stale-result policy at each crossing. Audio device callbacks must not wait for
an arbitrary render or ML job; late work needs an explicit reported policy,
not a hidden synchronous stall. Underrun/overrun, dropped visual frames and
stale inference results are distinct observations. This is not a hard-real-time
guarantee. Cancellation and close must account for in-flight resource ownership
before reuse or release, across every participating provider.

### Combined Acceptance

A later growing application should let control input change a visual/compute
pipeline, let a WitSage result change visible state or audio parameters, and
process/present those results inside the same session. Audio analysis driving
visualization is another possible dependency, not a mandatory cycle in each
frame. Independent unit demos remain useful but cannot close this combined goal.

Acceptance needs source-level Nuis orchestration, registered artifact dispatch,
identified cross-workflow data and completion edges, checked outputs, timing and
resource measurements, and failure/close evidence. Record backend/device
evidence separately; a single successful run does not establish sustained
stability, hard-real-time behavior or portable backend parity.

This horizon does not enlarge the current image application's acceptance goal
or postpone the finite compiler-module migration plan above. Before scheduling
ML integration, audio integration or their combined workload, register separate
tensor coordinates, dependencies and reproducible acceptance tests. Until then,
the broad engine scope cell and the lifecycle manifest's `[engine_target]`
record these as planned, not verified execution or advertised runtime APIs.

## Ownership Boundaries

* Application, scene and engine policies belong in Nuis `.ns` modules.
* PixelMagic owns image algorithms; WitSage owns classical model/compute policy.
  Neither becomes an internal namespace of ns-nova.
* Nustars own registered device/backend capabilities. Shared semantics, clock,
  GLM and data contracts belong at their existing common layers.
* Rust and platform-language code may remain for the bootstrap compiler/runtime
  and necessary OS adapters, but not as the default for new engine logic.
* Compiler fixes must be general language or lowering capabilities, not switches
  for an ns-nova project, frame helper or demo name.

First validate on available hardware. Report Metal, CUDA, Vulkan and other
backend evidence separately; absent hardware is an unverified lane, not a reason
to claim parity or silently substitute a CPU result. Missing devices need not
block unrelated application work on available devices.

## Quality And Migration Gates

Correctness regressions may interrupt features through the plan's explicit
`interrupts` roots. Each needs a reproducible failure and acceptance evidence;
security, corruption, leaks and invalid synchronization are not deferred merely
because another cell has a lower progress score.

Performance work records the workload, machine, backend, warmup and samples.
Measure cold/warm start, frame-time percentiles, allocations, uploads/copies,
outstanding handles and memory trends. Do not derive a language speed ratio
from a three-frame smoke test or cache-hit count. Do not weaken ownership,
completion or error checks merely to improve benchmark numbers.

Writing an engine in Nuis validates the language; it is not compiler self-hosting.
Count a compiler module as migrated only when its real responsibility has moved:

* name its input/output contract and the old and new implementations;
* use the Nuis implementation in a real application build, not only a fixture;
* compare semantics and failures against the retained reference;
* reproduce the build independently and keep exact rollback identities;
* use existing explicit authorization before production selection changes.

Potential early components include source handling, diagnostics, serialization
and bounded compiler stages. Choose by dependency readiness and application need,
not by Rust line count. Native object production and a general self-compiling
compiler remain distinct, later evidence obligations.

The phase agreement remains unchanged: staged migration started at `beta-0.10.*`;
`gamma-0.5.*` through `gamma-0.10.*` remains the approximate stage2-equivalent
completion window, not a fixed deadline. Vulpoya/Yalivia collaboration and wider
framework maturity remain later work; they do not gate the initial AOT application.

## Executable Planning

The [mainline plan](../reference/nuis-development-tensor.mainline.toml) declares
ordered goals and their prerequisite graph. Its
[selection contract](../reference/nuis-development-tensor-mainline.md) keeps the
current goal separate from the global weakest metric. The first task is the
persistent application session, not a blanket claim that the engine is 99% ready.

Update tensor evidence after each verified slice. Preserve old bounded proofs,
create narrower coordinates when scope grows, and do not equate preparation-gate
closure, runtime execution evidence and module-replacement authority.
