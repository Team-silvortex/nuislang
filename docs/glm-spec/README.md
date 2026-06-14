# GLM Spec Notes

This directory is the longer-range `GLM` design layer.

Use it for the question:

* what `GLM` is supposed to become as the repository keeps tightening its
  heterogeneous flow-graph semantics

Do not use it as the only source for current implementation truth.

For "what is true in checked-in code today?", prefer:

* [../reference/yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [../reference/cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* current verifier / `yir-core` / lowering behavior in the repository

## Reading Order

Start here:

* [glm-heterogeneous-flow-graph-positioning.md](/Users/Shared/chroot/dev/nuislang/docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md)
* [vulpoya-yir-secondary-review-positioning.md](/Users/Shared/chroot/dev/nuislang/docs/glm-spec/vulpoya-yir-secondary-review-positioning.md)

Then cross-check against implementation-facing references:

* [../reference/nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
* [../reference/yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [../reference/cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)

## Boundary

This folder should answer:

* what `GLM` is for
* what it should constrain across heterogeneous domains
* how ownership/lifetime techniques support that goal
* which parts are current implementation, and which parts are still target
  design

This folder should not drift into:

* grammar details
* per-pass code walkthroughs
* historical archive material
