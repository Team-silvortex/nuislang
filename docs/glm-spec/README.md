# GLM Spec Notes

This directory is reserved for the longer-range `GLM` ownership/lifetime model
notes of the project.

Current state:

* the implementation is already carrying part of this direction through `YIR`
  lifetime edges, ownership-sensitive verifier rules, and CPU/data move
  semantics
* the fuller written spec is not yet extracted into a stable standalone
  document here

For the current implementation truth, prefer:

* [../reference/yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* current verifier and `yir-core` behavior in the repository
