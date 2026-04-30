---

# YIR Reference Index

## Draft Reference v0.01

---

The current `YIR` reference is split into two working documents:

* [YIR LangRef](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [YIR Tools Reference](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)

This split is intentional:

* `YIR LangRef` tracks graph meaning, domain families, op surfaces, and
  verifier-visible semantics
* `YIR Tools Reference` tracks current reference executors, LLVM lowering, AOT
  packaging, and preview/export tooling

Both documents are expected to evolve together with the implementation.

Current emphasis:

* `YIR LangRef` is the better place to look for execution semantics, graph shape,
  lane behavior, and result/observe rules
* `YIR Tools Reference` is the better place to look for workflow, CLI, build
  outputs, cache behavior, packaging, and inspection commands
* async/result normalization is currently moving more of its contract into
  `yir-core`, so some rules that used to feel domain-local are becoming
  language-level `YIR` concepts
