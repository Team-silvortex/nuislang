# Beta 0.11 Application-Led Mainline

This recalibration was agreed on September 7, 2026 against Git checkpoint
`41cf5e9e` (`beta-0.11.3`). It does not create a new release or alter Cargo package
versions. Git history remains authoritative for subsequent checkpoints.

## Direction

`ns-nova application -> exposed foundation gap -> bug fix and regression ->
measured optimization -> incremental Nuis-owned toolchain replacement`

ns-nova is the application driver, not a reason to couple the compiler to one
engine or start four independent large projects. Its intended role remains a
real-time world engine, with scene, rendering and lifecycle orchestration;
the first deliverable is deliberately smaller than the eventual engine.

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
