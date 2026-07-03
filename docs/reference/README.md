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

* [../versioning/nuis-alpha-0.7-mainline-entry.md](../../docs/versioning/nuis-alpha-0.7-mainline-entry.md)
* [../versioning/nuis-alpha-0.6-mainline-entry.md](../../docs/versioning/nuis-alpha-0.6-mainline-entry.md)
* [../versioning/nuis-alpha-0.4-system-inventory.md](../../docs/versioning/nuis-alpha-0.4-system-inventory.md)
* [../versioning/nuis-alpha-0.4-doc-sync-inventory.md](../../docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md)

## Reading Order

Start in this order:

* `YIR` foundation
  - [yir-reference.md](yir-reference.md)
  - [yir-langref.md](yir-langref.md)
  - [yir-tools-reference.md](yir-tools-reference.md)
  - [nuis-binary-format-protocol.md](nuis-binary-format-protocol.md)
  - [nuis-native-artifact-workflow.md](nuis-native-artifact-workflow.md)
  - [toolchain-galaxy-core-boundary.md](toolchain-galaxy-core-boundary.md)
  - [nsld-linker-frontdoor.md](nsld-linker-frontdoor.md)
  - [nsdb-yir-debugger-frontdoor.md](nsdb-yir-debugger-frontdoor.md)
* `NIR` safety boundary
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

If your question is specifically “what is the shortest real native binary
closure route today?”, start with
[nuis-native-artifact-workflow.md](nuis-native-artifact-workflow.md).

If your question is specifically “what is the current independent linker
frontdoor?”, start with
[nsld-linker-frontdoor.md](nsld-linker-frontdoor.md).

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
