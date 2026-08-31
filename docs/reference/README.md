# Reference Docs

This folder is the current implementation-facing reference layer.

If you want to understand what is true in the repository today, this is usually
the best documentation layer to read first after the top-level
[README.md](../../README.md).

If your immediate question is "what is `GLM` trying to be beyond the current
task/pointer rules?", read:

* [../glm-spec/glm-heterogeneous-flow-graph-positioning.md](../../docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md)

If you want a short current phase summary before drilling into individual
contracts, start with:

* [../versioning/nuis-beta-0.6.0-mainline-entry.md](../../docs/versioning/nuis-beta-0.6.0-mainline-entry.md)
* [../versioning/nuis-beta-0.3.0-mainline-entry.md](../../docs/versioning/nuis-beta-0.3.0-mainline-entry.md)
* [../versioning/nuis-beta-0.1.0-mainline-entry.md](../../docs/versioning/nuis-beta-0.1.0-mainline-entry.md)
* [../versioning/nuis-beta-0.0.1-mainline-entry.md](../../docs/versioning/nuis-beta-0.0.1-mainline-entry.md)
* [../versioning/nuis-alpha-0.20-mainline-entry.md](../../docs/versioning/nuis-alpha-0.20-mainline-entry.md)
* [../versioning/nuis-alpha-0.17-mainline-entry.md](../../docs/versioning/nuis-alpha-0.17-mainline-entry.md)
* [../versioning/nuis-alpha-0.16-mainline-entry.md](../../docs/versioning/nuis-alpha-0.16-mainline-entry.md)
* [../versioning/nuis-alpha-0.10-mainline-entry.md](../../docs/versioning/nuis-alpha-0.10-mainline-entry.md)
* [../versioning/nuis-alpha-0.8-mainline-entry.md](../../docs/versioning/nuis-alpha-0.8-mainline-entry.md)
* [../versioning/nuis-alpha-0.7-mainline-entry.md](../../docs/versioning/nuis-alpha-0.7-mainline-entry.md)
* [../versioning/nuis-alpha-0.6-mainline-entry.md](../../docs/versioning/nuis-alpha-0.6-mainline-entry.md)
* [../versioning/nuis-alpha-0.4-system-inventory.md](../../docs/versioning/nuis-alpha-0.4-system-inventory.md)
* [../versioning/nuis-alpha-0.4-doc-sync-inventory.md](../../docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md)

## Reading Order

Start in this order:

* `YIR` foundation
  - [nuis-development-tensor.md](nuis-development-tensor.md)
  - [source-text-encoding-contract.md](source-text-encoding-contract.md)
  - [yir-reference.md](yir-reference.md)
  - [yir-langref.md](yir-langref.md)
  - [yir-tools-reference.md](yir-tools-reference.md)
  - [nuis-binary-format-protocol.md](nuis-binary-format-protocol.md)
  - [nuis-native-artifact-workflow.md](nuis-native-artifact-workflow.md)
  - [toolchain-galaxy-core-boundary.md](toolchain-galaxy-core-boundary.md)
  - [galaxy-resolution-lock-contract.md](galaxy-resolution-lock-contract.md)
  - [nustar-multi-backend-artifact-contract.md](nustar-multi-backend-artifact-contract.md)
  - [nsld-linker-frontdoor.md](nsld-linker-frontdoor.md)
  - [nsld-binary-assembly-gap-map.md](nsld-binary-assembly-gap-map.md)
  - [nsld-executable-finalizer-registry.md](nsld-executable-finalizer-registry.md)
  - [nuis-packaging-lifecycle-responsibility-map.md](nuis-packaging-lifecycle-responsibility-map.md)
  - [cffi-von-neumann-domain-contract.md](cffi-von-neumann-domain-contract.md)
  - [nsdb-yir-debugger-frontdoor.md](nsdb-yir-debugger-frontdoor.md)
  - [nsbdr-bundler-frontdoor.md](nsbdr-bundler-frontdoor.md)
* `NIR` safety boundary
  - [nuis-self-hosting-readiness.md](nuis-self-hosting-readiness.md)
  - [nuis-bootstrap-language-subset.md](nuis-bootstrap-language-subset.md)
  - [nuis-compiler-data-model.md](nuis-compiler-data-model.md)
  - [nuis-compiler-data-model-v10.toml](nuis-compiler-data-model-v10.toml)
  - [nuis-compiler-stage-handoff.md](nuis-compiler-stage-handoff.md)
  - [nuis-compiler-stage-transformation.md](nuis-compiler-stage-transformation.md)
  - [nuis-compiler-candidate-production.md](nuis-compiler-candidate-production.md)
  - [nuis-compiler-candidate-compile-capability.md](nuis-compiler-candidate-compile-capability.md)
  - [nuis-compiler-candidate-compile-capability-v1.toml](nuis-compiler-candidate-compile-capability-v1.toml)
  - [nuis-compiler-candidate-direct-compile-capability.md](nuis-compiler-candidate-direct-compile-capability.md)
  - [nuis-compiler-candidate-direct-compile-capability-v2.toml](nuis-compiler-candidate-direct-compile-capability-v2.toml)
  - [nuis-compiler-candidate-preselection.md](nuis-compiler-candidate-preselection.md)
  - [nuis-compiler-candidate-preselection-v1.toml](nuis-compiler-candidate-preselection-v1.toml)
  - [nuis-compiler-candidate-successor.md](nuis-compiler-candidate-successor.md)
  - [nuis-compiler-candidate-successor-v1.toml](nuis-compiler-candidate-successor-v1.toml)
  - [nuis-compiler-component-build.md](nuis-compiler-component-build.md)
  - [nuis-compiler-component-differential.md](nuis-compiler-component-differential.md)
  - [nuis-compiler-component-representation-differential-v1.toml](nuis-compiler-component-representation-differential-v1.toml)
  - [nuis-compiler-component-reproducibility.md](nuis-compiler-component-reproducibility.md)
  - [nuis-compiler-component-attestation.md](nuis-compiler-component-attestation.md)
  - [nuis-compiler-component-replacement-authorization.md](nuis-compiler-component-replacement-authorization.md)
  - [nuis-compiler-component-active-state-v1.toml](nuis-compiler-component-active-state-v1.toml)
  - [nuis-compiler-component-transition-v2.toml](nuis-compiler-component-transition-v2.toml)
  - [nuis-compiler-component-dispatch.md](nuis-compiler-component-dispatch.md)
  - [nuis-compiler-component-dispatch-v1.toml](nuis-compiler-component-dispatch-v1.toml)
  - [nuis-compiler-component-compile-dispatch.md](nuis-compiler-component-compile-dispatch.md)
  - [nuis-compiler-component-compile-dispatch-v1.toml](nuis-compiler-component-compile-dispatch-v1.toml)
  - [nir-memory-model.md](nir-memory-model.md)
  - [nir-optimization-contract.md](nir-optimization-contract.md)
  - [control-flow-lowering-contract.md](control-flow-lowering-contract.md)
  - [generic-diagnostic-ownership-contract.md](generic-diagnostic-ownership-contract.md)
  - [host-read-bridge.md](host-read-bridge.md)
  - [ffi-pointer-safety-boundary.md](ffi-pointer-safety-boundary.md)
  - [std-mainline-layering-contract.md](std-mainline-layering-contract.md)
  - [std-host-io-layering-contract.md](std-host-io-layering-contract.md)
  - [std-data-window-fabric-layering-contract.md](std-data-window-fabric-layering-contract.md)
  - [std-net-layering-contract.md](std-net-layering-contract.md)
  - [std-shader-kernel-project-contract.md](std-shader-kernel-project-contract.md)
  - [pixelmagic-mainline-contract.md](pixelmagic-mainline-contract.md)
* task-facing current contract
  - [cpu-task-contract.md](cpu-task-contract.md)
  - [cpu-task-memory-contract.md](cpu-task-memory-contract.md)
  - [cpu-task-glm-contract.md](cpu-task-glm-contract.md)
  - [cpu-task-payload-matrix.md](cpu-task-payload-matrix.md)
  - [cpu-task-scheduler-clock.md](cpu-task-scheduler-clock.md)
  - [std-task-layering-contract.md](std-task-layering-contract.md)
* task-facing future edge
  - [cpu-task-external-handle-contract.md](cpu-task-external-handle-contract.md)
  - [cpu-task-external-handle-glm-sketch.md](cpu-task-external-handle-glm-sketch.md)
  - [annotation-intrinsic-stdlib-sketch.md](annotation-intrinsic-stdlib-sketch.md)
  - [nuis-launcher-container-linker-sketch.md](nuis-launcher-container-linker-sketch.md)
  - [nuis-aot-lifecycle-loop-sketch.md](nuis-aot-lifecycle-loop-sketch.md)
  - [nustar-abi-grain-sketch.md](nustar-abi-grain-sketch.md)
  - [nuis-packaging-lifecycle-responsibility-map.md](nuis-packaging-lifecycle-responsibility-map.md)
  - [trait-generic-monomorphization-sketch.md](trait-generic-monomorphization-sketch.md)
  - [network-domain-contract.md](network-domain-contract.md)
  - [network-runtime-host-validation.md](network-runtime-host-validation.md)
  - [network-profile-contract.md](network-profile-contract.md)
  - [yir-hot-sync-contraction-sketch.md](yir-hot-sync-contraction-sketch.md)
  - [yir-global-clock-negotiation-sketch.md](yir-global-clock-negotiation-sketch.md)

If your question is specifically “what command should I run next for this
project?”, start with
[yir-tools-reference.md](yir-tools-reference.md).

If your question is specifically “how do architecture, module, and function
progress line up right now?”, start with
[nuis-development-tensor.md](nuis-development-tensor.md).

If your question is specifically “what is the shortest real native binary
closure route today?”, start with
[nuis-native-artifact-workflow.md](nuis-native-artifact-workflow.md).

If your question is specifically “what is the current independent linker
frontdoor?”, start with
[nsld-linker-frontdoor.md](nsld-linker-frontdoor.md).

If your question is specifically “what remains between the current Nsld
container and a runnable Nuis-owned heterogeneous executable?”, start with
[nsld-binary-assembly-gap-map.md](nsld-binary-assembly-gap-map.md).

If your question is specifically “what is the current early-beta target and
what wording is safe?”, start with
[../versioning/nuis-beta-0.6.0-mainline-entry.md](../../docs/versioning/nuis-beta-0.6.0-mainline-entry.md).

If your question is specifically “what must close before staged compiler
self-hosting can begin?”, start with
[nuis-self-hosting-readiness.md](nuis-self-hosting-readiness.md).

If your question is specifically “how are stage0 and future stage1 compiler
payloads identified without Rust layout coupling?”, start with
[nuis-compiler-stage-handoff.md](nuis-compiler-stage-handoff.md).

If your question is specifically “how does a signed component transition select
and execute exact compiler-image bytes without persisting their path?”, start
with [nuis-compiler-component-dispatch.md](nuis-compiler-component-dispatch.md).

If your question is specifically “how does that selected image compile a real
project without making runtime paths part of the request?”, start with
[nuis-compiler-component-compile-dispatch.md](nuis-compiler-component-compile-dispatch.md).

If your question is specifically “how can the production-bound Nuis stage1
candidate drive the same request while its stage0 dependency remains explicit?”,
start with
[nuis-compiler-candidate-compile-capability.md](nuis-compiler-candidate-compile-capability.md).

If your question is specifically “what can stage1 compile directly without a
runtime stage0 provider?”, start with
[nuis-compiler-candidate-direct-compile-capability.md](nuis-compiler-candidate-direct-compile-capability.md).

If your question is specifically “how does stage0 attest one complete
project-form compiler-component build?”, start with
[nuis-compiler-component-build.md](nuis-compiler-component-build.md).

If your question is specifically “how are stage0 and a candidate stage1
compared without implicitly authorizing replacement?”, start with
[nuis-compiler-component-differential.md](nuis-compiler-component-differential.md).

If your question is specifically “how do two independent cache-bypassed clean
candidate builds prove a stable compiler identity?”, start with
[nuis-compiler-component-reproducibility.md](nuis-compiler-component-reproducibility.md).

If your question is specifically “what did the alpha-0.20 closeout establish?”,
read
[../versioning/nuis-alpha-0.20-mainline-entry.md](../../docs/versioning/nuis-alpha-0.20-mainline-entry.md).

If your question is specifically “what did the alpha-0.17 registered
heterogeneous worker execution target establish?”, read
[../versioning/nuis-alpha-0.17-mainline-entry.md](../../docs/versioning/nuis-alpha-0.17-mainline-entry.md).

If your question is specifically “what did the alpha-0.10 executable artifact
target establish?”, read
[../versioning/nuis-alpha-0.10-mainline-entry.md](../../docs/versioning/nuis-alpha-0.10-mainline-entry.md).

If your question is specifically “what did the alpha-0.8 binary-linking
convergence predecessor establish?”, read
[../versioning/nuis-alpha-0.8-mainline-entry.md](../../docs/versioning/nuis-alpha-0.8-mainline-entry.md).

If your question is specifically “how should Nuis treat the C world and the
classic von-Neumann host stack?”, start with
[cffi-von-neumann-domain-contract.md](cffi-von-neumann-domain-contract.md).

If your question is specifically “how should Nuis debugging work above native
LLDB-style shell debugging?”, start with
[nsdb-yir-debugger-frontdoor.md](nsdb-yir-debugger-frontdoor.md).

If your question is specifically “should linker/debugger capabilities be CLI
commands or reusable galaxy-style toolchain APIs?”, start with
[toolchain-galaxy-core-boundary.md](toolchain-galaxy-core-boundary.md).

If your question is specifically “which `nuis` frontdoor fields should I read
or consume right now?”, start with
[nuis-frontdoor-surface-reference.md](nuis-frontdoor-surface-reference.md).

Shortest rule:

* use this README for the implementation-truth anchor set
* use [docs/current-mainline-map.md](../../docs/current-mainline-map.md)
  for repo-level mainline routing
* use the specific reference file directly once you know which truth layer you
  need

## Boundary

Use this folder for:

* current semantic behavior
* current tool behavior
* current workflow and packaging surfaces

Do not treat this folder as:

* the handwritten grammar source of truth
* long-range architecture argument
* historical archive

For those, see:

* [docs/grammar/README.md](../../docs/grammar/README.md)
* [docs/historical/README.md](../../docs/historical/README.md)
