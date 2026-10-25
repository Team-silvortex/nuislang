# Reference Contracts And Evidence

This directory holds implementation contracts, versioned protocol material and
scoped validation receipts. A receipt preserves what its own checkout tested; it
does not make every statement here current forever. Use code/tests, the live
tensor and later profile-specific follow-ups to resolve apparent contradictions.

## Read First

The current line is beta-0.17. Git identity belongs in the dated snapshot, not
a copied patch number in every index.

| Question | Start With |
| --- | --- |
| What can I run and what remains open? | [Mainline map](../current-mainline-map.md) |
| Which committed patch and worktree follow-up does this audit describe? | [Dated snapshot](../versioning/nuis-beta-0.17.0-snapshot.md) |
| Which checks should I choose? | [Operating checklist](../versioning/nuis-beta-0.17.0-release-checklist.md) |
| Where are scoped native CPU proofs? | [Proof catalog](nuis-native-session-proof-index.md) |
| Which task is selected and why? | [Tensor selection](nuis-development-tensor-mainline.md) |

Native proof history is catalogued once in the proof index. A page's presence,
a schema version or a successful inspection is not runtime/device acceptance.
Each contract identifies its active profile; linked payload files are versioned
examples and must not be selected merely because their filename looks newest.

## YIR And Binary Contracts

- [Artifact File Publication](nuis-artifact-file-publication-v1.md)
- [YIR Reference Overview](yir-reference.md)
- [YIR LangRef](yir-langref.md)
- [YIR Tools Reference](yir-tools-reference.md)
- [Nuis Binary Format Protocol](nuis-binary-format-protocol.md)
- [`nuis` Native Artifact Workflow](nuis-native-artifact-workflow.md)

## Compiler Migration

- [Nuis Self-Hosting Readiness](nuis-self-hosting-readiness.md)
- [Nuis Bootstrap Language Subset](nuis-bootstrap-language-subset.md)
- [Nuis Compiler Data Model](nuis-compiler-data-model.md)
- [nuis-compiler-data-model-v11.toml](nuis-compiler-data-model-v11.toml)
- [nuis-compiler-data-model-v10.toml](nuis-compiler-data-model-v10.toml)
- [Nuis Compiler Stage Handoff](nuis-compiler-stage-handoff.md)
- [nuis-compiler-candidate-structural-pagination-v1.toml](nuis-compiler-candidate-structural-pagination-v1.toml)
- [Nuis Compiler Stage Transformation](nuis-compiler-stage-transformation.md)
- [Nuis Compiler Candidate Production](nuis-compiler-candidate-production.md)
- [Nuis Compiler Candidate Compile Capability](nuis-compiler-candidate-compile-capability.md)
- [nuis-compiler-candidate-compile-capability-v1.toml](nuis-compiler-candidate-compile-capability-v1.toml)
- [Nuis Compiler Candidate Direct Compile Capability](nuis-compiler-candidate-direct-compile-capability.md)
- [nuis-compiler-candidate-direct-compile-capability-v2.toml](nuis-compiler-candidate-direct-compile-capability-v2.toml)
- [Nuis Compiler Candidate Preselection](nuis-compiler-candidate-preselection.md)
- [nuis-compiler-candidate-preselection-v1.toml](nuis-compiler-candidate-preselection-v1.toml)
- [Nuis Compiler Candidate Successor](nuis-compiler-candidate-successor.md)
- [nuis-compiler-candidate-successor-v1.toml](nuis-compiler-candidate-successor-v1.toml)
- [Nuis Compiler Candidate Fresh-Source Capability](nuis-compiler-candidate-fresh-source-capability.md)
- [nuis-compiler-candidate-fresh-source-capability-v1.toml](nuis-compiler-candidate-fresh-source-capability-v1.toml)
- [Nuis Compiler Candidate to Nsld Materialization](nuis-compiler-candidate-nsld-materialization.md)
- [nuis-compiler-candidate-nsld-input-v1.toml](nuis-compiler-candidate-nsld-input-v1.toml)
- [nuis-compiler-candidate-nsld-materialization-v1.toml](nuis-compiler-candidate-nsld-materialization-v1.toml)
- [Nuis Compiler Component Build](nuis-compiler-component-build.md)
- [Nuis Compiler Component Differential Gate](nuis-compiler-component-differential.md)
- [nuis-compiler-component-representation-differential-v1.toml](nuis-compiler-component-representation-differential-v1.toml)
- [Nuis Compiler Component Reproducibility](nuis-compiler-component-reproducibility.md)
- [nuis-compiler-component-reproducibility-v2.toml](nuis-compiler-component-reproducibility-v2.toml)
- [Nuis Compiler Component Attestation](nuis-compiler-component-attestation.md)
- [Nuis Compiler Component Replacement Authorization](nuis-compiler-component-replacement-authorization.md)
- [nuis-compiler-component-active-state-v1.toml](nuis-compiler-component-active-state-v1.toml)
- [nuis-compiler-component-transition-v2.toml](nuis-compiler-component-transition-v2.toml)
- [Nuis Compiler Component Dispatch](nuis-compiler-component-dispatch.md)
- [nuis-compiler-component-dispatch-v1.toml](nuis-compiler-component-dispatch-v1.toml)
- [Nuis Compiler Component Compile Dispatch](nuis-compiler-component-compile-dispatch.md)
- [nuis-compiler-component-compile-dispatch-v1.toml](nuis-compiler-component-compile-dispatch-v1.toml)

## Memory Tasks And Source Rules

- [Source Text Encoding Contract](source-text-encoding-contract.md)
- [NIR Memory Model](nir-memory-model.md)
- [NIR Optimization Contract](nir-optimization-contract.md)
- [Control-Flow Lowering Contract](control-flow-lowering-contract.md)
- [Generic Diagnostic Ownership Contract](generic-diagnostic-ownership-contract.md)
- [FFI Pointer Safety Boundary](ffi-pointer-safety-boundary.md)
- [CPU Task Contract](cpu-task-contract.md)
- [CPU Task Memory Contract](cpu-task-memory-contract.md)
- [CPU Task GLM Contract](cpu-task-glm-contract.md)
- [CPU Task Payload Matrix](cpu-task-payload-matrix.md)
- [CPU Task, Scheduler, and Clock Contract](cpu-task-scheduler-clock.md)
- [CPU Task External Handle Contract](cpu-task-external-handle-contract.md)

## Runtime And Provider Sessions

- [Registered Window Sessions](nuis-yir-window-session-v3.md)
- [Application Cancellation And Host Retirement](nuis-yir-application-cancellation-v1.md)
- [Provider Session Drain](nuis-yir-provider-session-drain-v1.md)
- [Headless Application Scalar Script](nuis-yir-application-scalar-script-v1.md)
- [Application Outcome Pump](nuis-yir-application-outcome-pump-v1.md)
- [nuis-ns-nova-application-lifecycle-v1.toml](nuis-ns-nova-application-lifecycle-v1.toml)
- [nuis-data-provider-inventory-v1.toml](nuis-data-provider-inventory-v1.toml)
- [nuis-data-provider-conformance-v1.toml](nuis-data-provider-conformance-v1.toml)

## Nustars Libraries And Toolchain

- [Toolchain Galaxy Core Boundary](toolchain-galaxy-core-boundary.md)
- [Nustar Multi-Backend Artifact Contract](nustar-multi-backend-artifact-contract.md)
- [Nsld Linker Frontdoor](nsld-linker-frontdoor.md)
- [Nsld Binary Assembly Gap Map](nsld-binary-assembly-gap-map.md)
- [Nsld Executable Finalizer Registry](nsld-executable-finalizer-registry.md)
- [Nuis Packaging / Lifecycle Responsibility Map](nuis-packaging-lifecycle-responsibility-map.md)
- [CFFI / Von Neumann Domain Contract](cffi-von-neumann-domain-contract.md)
- [Nsdb YIR Debugger Frontdoor](nsdb-yir-debugger-frontdoor.md)
- [Nsbdr Bundler Frontdoor](nsbdr-bundler-frontdoor.md)
- [`std` Mainline Layering Contract](std-mainline-layering-contract.md)
- [`std` Host I/O Layering Contract](std-host-io-layering-contract.md)
- [`std` Data / Window / Fabric Layering Contract](std-data-window-fabric-layering-contract.md)
- [`std` Net Layering Contract](std-net-layering-contract.md)
- [Shader / Kernel Project-First Contract](std-shader-kernel-project-contract.md)
- [`PixelMagic` Mainline Contract](pixelmagic-mainline-contract.md)
- [`std` Task Layering Contract](std-task-layering-contract.md)
- [Network Domain Contract](network-domain-contract.md)
- [Network Runtime Host Validation](network-runtime-host-validation.md)
- [Network Profile Contract](network-profile-contract.md)

## Manufacturing And Development

- [nuislang](../../README.md)
- [Nuis Development Tensor](nuis-development-tensor.md)
- [Nuis Software Manufacturing Architecture](nuis-software-manufacturing-architecture.md)
- [Galaxy Resolution Lock Contract](galaxy-resolution-lock-contract.md)
- [Nuis-Rc Heap/Stack Cache Lifecycle](nuis-rc-cache-lifecycle.md)
- [galaxy-provider-trust-state-v1.toml](galaxy-provider-trust-state-v1.toml)
- [nuis-development-tensor.mainline.toml](nuis-development-tensor.mainline.toml)
- [Host Read Bridge](host-read-bridge.md)
- [`nuis` Frontdoor Surface Reference](nuis-frontdoor-surface-reference.md)
- [Current Mainline Map](../../docs/current-mainline-map.md)

## Design And Historical Anchors

- [GLM As Heterogeneous Flow-Graph Semantics](../../docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md)
- [Versioning Notes](../versioning/README.md)
- [CPU Task External Handle GLM Sketch](cpu-task-external-handle-glm-sketch.md)
- [Annotation / Intrinsic Stdlib Sketch](annotation-intrinsic-stdlib-sketch.md)
- [Nuis Launcher / Container / Linker Sketch](nuis-launcher-container-linker-sketch.md)
- [Nuis AOT Lifecycle Loop Sketch](nuis-aot-lifecycle-loop-sketch.md)
- [Nustar ABI Grain Sketch](nustar-abi-grain-sketch.md)
- [Trait And Generic Monomorphization Sketch](trait-generic-monomorphization-sketch.md)
- [YIR Hot Sync Contraction Sketch](yir-hot-sync-contraction-sketch.md)
- [YIR Global Clock Negotiation Sketch](yir-global-clock-negotiation-sketch.md)
- [`nuis` `beta-0.6.*` Mainline Entry](../versioning/nuis-beta-0.6.0-mainline-entry.md)
- [`nuis` `alpha-0.20.*` Mainline Entry](../../docs/versioning/nuis-alpha-0.20-mainline-entry.md)
- [`nuis` `alpha-0.17.*` Mainline Entry](../../docs/versioning/nuis-alpha-0.17-mainline-entry.md)
- [`nuis` `alpha-0.10.*` Mainline Entry](../../docs/versioning/nuis-alpha-0.10-mainline-entry.md)
- [`nuis` `alpha-0.8.*` Mainline Entry](../../docs/versioning/nuis-alpha-0.8-mainline-entry.md)
- [Grammar And Frontend Notes](../../docs/grammar/README.md)
- [Historical Docs](../../docs/historical/README.md)

## Documentation Roles

Use current semantic/tool contracts for implementation behavior, the proof index
for bounded execution history and the versioning index for dated releases and
roadmaps. Grammar, broader design and archived whitepapers retain their own
layers; they are not substitutes for a current verifier or runtime contract.

Keep entry routes short. Add a focused contract or receipt to its owning section
rather than copying the full proof narrative into every README.
