# Reference Docs

This folder is the current implementation-facing reference layer.

If you want to understand what is true in the repository today, this is usually
the best documentation layer to read first after the top-level
[README.md](/Users/Shared/chroot/dev/nuislang/README.md).

## Reading Order

Start in this order:

* [yir-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-reference.md)
  overview/index for the current `YIR` reference set
* [yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
  graph meaning, execution semantics, domain families, and verifier-visible
  rules
* [yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
  workflow commands, compiler/tool boundaries, packaging, cache, and inspection
  behavior
* [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
  current `NIR`-level ownership, borrow, move, and verifier-enforced aliasing
  rules
* [nir-optimization-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-optimization-contract.md)
  current `NIR` optimization safety boundary, including canonical pure /
  read-only / stateful expression classes
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
  current bridge between compiler-recognized host reads and `std` host facade
  modules
* [cpu-task-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-contract.md)
  current `cpu` task semantics, including `spawn/join/timeout/join_result`
  boundaries and the line between async expression support and true
  concurrency runtime
* [cpu-task-memory-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-memory-contract.md)
  current ownership boundary for task inputs, including why `spawn(...)`
  currently rejects borrowed and `ref` values
* [cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
  current `GLM` reading of `Task<T>` / `TaskResult<T>`, including the current
  observation boundary and what is still missing before tasks become a fuller
  ownership/lifetime object
* [cpu-task-payload-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-payload-matrix.md)
  current allowed/rejected task payload families with concrete positive and
  negative examples
* [cpu-task-external-handle-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-external-handle-contract.md)
  current design direction for resource-bearing task payload families that may
  later need an explicit external-handle contract instead of plain value
  semantics
* [cpu-task-scheduler-clock.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-scheduler-clock.md)
  current relationship between task semantics, lane/scheduler surfaces, and
  clock/timeout bridges

If your question is specifically “what command should I run next for this
project?”, start with
[yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md).

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
