# Docs Index

This folder is the documentation entry point once you move past the top-level
`README`.

The docs are currently split into two broad categories:

* current reference / implementation-facing material
* longer-range design/spec material
* historical archived material

There is also a practical split inside the current tree:

* mainline docs
  these explain the repository paths you should rely on today
* experimental / design docs
  these explain probe directions, future contracts, and semantic sketches that
  are intentionally not fully locked yet

## Read This First

If you want to understand the repository as it exists today, start here:

* [current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [versioning/nuis-0.19.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-snapshot.md)
* [versioning/nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
* [versioning/nuis-0.20.0-abi-compile-vocabulary.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-abi-compile-vocabulary.md)
* [versioning/nuis-0.19.0-workflow-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-workflow-capability-matrix.md)
* [versioning/nuis-0.19.0-project-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-project-capability-matrix.md)
* [reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
* [repo-layout.md](/Users/Shared/chroot/dev/nuislang/docs/repo-layout.md)

If your immediate question is “what is the canonical compile route today?”,
start with the `0.19.0` workflow file before drilling into deeper reference
material.

If your immediate question is “what is the current minor-line history anchor?”,
start with the `0.19.0` snapshot.

If your immediate question is “which ABI words are now preferred before
`0.20.*` broadens the surface further?”, read the
`0.20.0` ABI compile vocabulary file next.

Then branch by the kind of truth you want:

* current runnable project examples
  - [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
* current source-level `.ns` examples
  - [examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* current stdlib/source-asset maps
  - [stdlib/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/README.md)
  - [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* cleanup policy / archiving candidates
  - [repo-cleanup-candidates.md](/Users/Shared/chroot/dev/nuislang/docs/repo-cleanup-candidates.md)
  - [repo-file-line-policy.md](/Users/Shared/chroot/dev/nuislang/docs/repo-file-line-policy.md)

## Grammar And Frontend Notes

Use these when you want parser/frontend context:

* [grammar/README.md](/Users/Shared/chroot/dev/nuislang/docs/grammar/README.md)

## Design / Spec Direction

These folders describe broader architecture direction and are useful, but they
should be read together with the current reference docs above:

* [fabric-spec/README.md](/Users/Shared/chroot/dev/nuislang/docs/fabric-spec/README.md)
* [glm-spec/README.md](/Users/Shared/chroot/dev/nuislang/docs/glm-spec/README.md)
* [versioning/README.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/README.md)
* [yir-spec/README.md](/Users/Shared/chroot/dev/nuislang/docs/yir-spec/README.md)
* [historical/README.md](/Users/Shared/chroot/dev/nuislang/docs/historical/README.md)

Important current reading rule:

* if a broader design note and the current checked-in tool/reference behavior
  differ, prefer the current `reference/` documents plus the implementation
  itself
* `fabric-spec/DFIR.md` is historical draft material, not a current verifier
  contract

If your immediate question is "how do `GLM`, compiler-native `YIR` verification,
and the future `vulpoya` analyzer fit together?", start with:

* [glm-spec/glm-heterogeneous-flow-graph-positioning.md](/Users/Shared/chroot/dev/nuislang/docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md)
* [glm-spec/vulpoya-yir-secondary-review-positioning.md](/Users/Shared/chroot/dev/nuislang/docs/glm-spec/vulpoya-yir-secondary-review-positioning.md)

## Historical Archive

These files are kept on purpose, but they are no longer part of the shortest
path for understanding the current repository:

* [historical/README.md](/Users/Shared/chroot/dev/nuislang/docs/historical/README.md)
* [historical/nuislang-whitepaper-v0.44b.md](/Users/Shared/chroot/dev/nuislang/docs/historical/nuislang-whitepaper-v0.44b.md)
