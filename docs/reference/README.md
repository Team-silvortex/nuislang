# Reference Docs

This folder is the current implementation-facing reference layer.

If you want to understand what is true in the repository today, this is usually
the best documentation layer to read first after the top-level
[README.md](/Users/Shared/chroot/dev/nuislang/README.md).

If you want a short phase summary before drilling into individual
contracts, start with:

* [../versioning/nuis-0.13.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.13.0-snapshot.md)

## Reading Order

Start in this order:

* `YIR` foundation
  - [yir-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-reference.md)
  - [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
  - [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* `NIR` safety boundary
  - [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
  - [nir-optimization-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-optimization-contract.md)
  - [control-flow-lowering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/control-flow-lowering-contract.md)
  - [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
  - [std-mainline-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-mainline-layering-contract.md)
  - [std-host-io-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-host-io-layering-contract.md)
  - [std-data-window-fabric-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-data-window-fabric-layering-contract.md)
  - [std-net-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-net-layering-contract.md)
  - [std-shader-kernel-project-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-shader-kernel-project-contract.md)
* task-facing current contract
  - [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
  - [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
  - [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
  - [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)
  - [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
  - [std-task-layering-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/std-task-layering-contract.md)
* task-facing future edge
  - [cpu-task-external-handle-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-contract.md)
  - [cpu-task-external-handle-glm-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-glm-sketch.md)
  - [annotation-intrinsic-stdlib-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/annotation-intrinsic-stdlib-sketch.md)
  - [nuis-launcher-container-linker-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-launcher-container-linker-sketch.md)
  - [nuis-aot-lifecycle-loop-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-aot-lifecycle-loop-sketch.md)
  - [nustar-abi-grain-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nustar-abi-grain-sketch.md)
  - [nuis-packaging-lifecycle-responsibility-map.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-packaging-lifecycle-responsibility-map.md)
  - [trait-generic-monomorphization-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/trait-generic-monomorphization-sketch.md)
  - [network-domain-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-domain-contract.md)
  - [network-runtime-host-validation.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-runtime-host-validation.md)
  - [network-profile-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-profile-contract.md)
  - [yir-hot-sync-contraction-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-hot-sync-contraction-sketch.md)
  - [yir-global-clock-negotiation-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-global-clock-negotiation-sketch.md)

If your question is specifically “what command should I run next for this
project?”, start with
[yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md).

Shortest rule:

* use this README for the implementation-truth anchor set
* use [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
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

* [docs/grammar/README.md](/Users/Shared/chroot/dev/nuislang/docs/grammar/README.md)
* [docs/historical/README.md](/Users/Shared/chroot/dev/nuislang/docs/historical/README.md)
