---

# YIR Tools Reference

## Draft Reference v0.01

---

# 1. Purpose

This document is the working tool reference for the current handwritten `YIR`
prototype in this repository.

It is the closest thing the project currently has to an early `LLVM tools`
reference for the `YIR` layer.

It should evolve together with the implementation.

---

# 2. Scope

This reference currently covers:

* reference command-line entry points
* current LLVM path
* current AOT packaging path
* current preview/export helpers

This reference does not yet attempt to freeze:

* final `nuisc` CLI
* final `nustar` package ABI
* final runtime packaging format

---

# 3. Reference Tools

## Parse + verify + execute

[tools/yir-run/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-run/src/main.rs)

```text
cargo run -p yir-run -- <module.yir>
```

This is the main handwritten `YIR` execution entry.

It performs:

* parse
* verify
* graph execution
* trace/lane/value reporting

## Emit LLVM IR text

[tools/yir-emit-llvm/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-emit-llvm/src/main.rs)

```text
cargo run -p yir-emit-llvm -- <module.yir>
```

This currently lowers the `cpu` slice to LLVM IR text.

## Build AOT bundle

[tools/yir-pack-aot/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-pack-aot/src/main.rs)

```text
cargo run -p yir-pack-aot -- <module.yir> <output-dir> [frame-scale]
```

This packages a small AOT artifact using LLVM/clang where possible and cooked or
prerendered artifacts where necessary.

## Export frame

[tools/yir-export-frame/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-export-frame/src/main.rs)

```text
cargo run -p yir-export-frame -- <module.yir> <output.ppm> [scale]
```

This is the current reference render-artifact path.

## Export UI plan

[tools/yir-export-ui-plan/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-export-ui-plan/src/main.rs)

This extracts host-side preview plan data from current `cpu` host extension ops.

## macOS preview adapter

[tools/yir-preview-macos/PreviewFrame.swift](/Users/Shared/chroot/dev/nuislang/tools/yir-preview-macos/PreviewFrame.swift)

This is a tool-layer adapter.

It is not part of `YIR` core semantics.

---

# 4. LLVM Path

The current LLVM path is intentionally narrow:

* it lowers the `cpu` slice
* it already supports arithmetic
* it already supports the current heap-node prototype
* it already emits `malloc/free + gep/load/store` for the linked-list model

Current examples:

* [examples/ball_cpu_driver.yir](/Users/Shared/chroot/dev/nuislang/examples/ball_cpu_driver.yir)
* [examples/cpu_linked_list.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_linked_list.yir)
* [examples/cpu_linked_list_rustish.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_linked_list_rustish.yir)

The current hetero render path may still package prerendered or cooked artifacts
for non-CPU slices.

This is expected at the current stage.

---

# 5. AOT Packaging Notes

The current `yir-pack-aot` path is reference-quality packaging, not final
distribution design.

Current behavior:

* pure CPU graphs can be compiled to a native binary through LLVM/clang
* hetero/window demos can be packaged into a macOS AppKit-hosted single binary
  using embedded prerendered framebuffer content
* shader packaging already has a contract/package skeleton for future backend
  variants
* shader package manifests may now include per-stage binding layout entries, texture/sampler/geometry binding kinds, minimal render-state metadata, sampler/texture binding details such as filter, address mode, and texture shape, plus top-level fabric handle-table metadata, per-stage fabric table association, and Fabric worker core binding metadata
* current macOS AppKit host stubs read `fabric_worker_core`, start a dedicated
  Fabric worker thread, and apply it as that thread's startup affinity hint;
* current Fabric host booting stays AOT-first: generated host stubs embed a
  static action table derived from `data.*` nodes instead of constructing a
  heavyweight dynamic metadata graph at runtime
* this is still weaker than a strict reserved-core runtime model

This is the beginning of a `YIR`-native toolchain, not the final shape.

---

# 6. Stability Notes

Most stable current reference tool surfaces:

* `yir-run`
* `yir-emit-llvm`
* `yir-pack-aot`
* CPU-slice LLVM lowering

Clearly provisional current tool surfaces:

* preview adapter details
* exact shader package schema
* final bundle/manifest format
* final in-process runtime embedding strategy

---

# 7. Sync Policy

This file should be updated whenever one of these changes:

* a reference tool is added or removed
* CLI behavior changes in a user-visible way
* LLVM lowering meaning changes
* AOT packaging meaning changes
* preview/export adapter meaning changes

The goal is for this file to remain a living reference, not a stale description.
