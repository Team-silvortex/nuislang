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
* current `nuis / nuisc` tool split
* current LLVM path
* current AOT packaging path
* current preview/export helpers

This reference does not yet attempt to freeze:

* final `nuisc` CLI
* final `nustar` package ABI
* final runtime packaging format

---

# 3. Reference Tools

## Front-door workflow tool

[tools/nuis/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/nuis/src/main.rs)

```text
cargo run -p nuis -- <command>
```

Current reference commands:

* `status`
* `registry`
* `fmt <input>`
* `bindings <input.ns>`
* `pack-nustar <package-id> <output.nustar>`
* `inspect-nustar <input.nustar>`
* `loader-contract <package-id>`
* `verify-build-manifest <nuis.build.manifest.toml>`
* `cache-status [--all] [--verbose-cache] [--json] [input]`
* `clean-cache [--all] [--json] [input]`
* `cache-prune [--all] [--keep N] [--json] [input]`
* `project-status <input.ns|project-dir|nuis.toml>`
* `project-lock-abi <project-dir|nuis.toml>`
* `release-check <input> <output-dir>`
* `check <input.ns>`
* `build <input.ns> <output-dir>`
* `build --cpu-abi <abi> <input> <output-dir>`
* `build --target <triple> <input> <output-dir>`
* `dump-ast <input.ns>`
* `dump-nir <input.ns>`
* `dump-yir <input.ns>`
* `rc ...`
* `galaxy ...`

`nuis` is the current front-door workflow tool. It should grow into the
user-facing toolchain surface while reusing `nuisc` as the compiler core.

Current `nustar` loading policy is:

* static index
* lazy manifest loading
* binding only for families actually required by the current `YIR` graph
* each loaded `nustar` now also declares its `AST surface`, `NIR surface`, `YIR lowering`, and `part verify` responsibilities so `nuisc` can stay mod-agnostic while still seeing the full package role
* each loaded `nustar` also declares unified entry names for those four surfaces, so future `nuisc` loading can bind through stable entry points instead of hard-coded per-mod assumptions

Current `nustar` packaging prototype is:

* one standard binary envelope
* manifest segment
* format version
* ABI tag
* implementation-format tag
* implementation blob segment
* implementation checksum

Current loading-contract direction is:

* `native-dylib` and `llvm-bc` both bind through the same canonical loader ABI
* packages declare a canonical `loader_entry`
* the canonical bootstrap symbol is `nustar.bootstrap.v1`
* the canonical bootstrap signature is `extern "C" fn(*const NustarHostAbiV1, *const u8, usize, *mut NustarBootstrapResultV1) -> i32`
* host/runtime bootstrap stays machine-ABI aware: current `.nustar` packages carry `machine_arch / machine_os / object_format / calling_abi`
* `loader-contract` now also defines per-kind implementation-segment requirements, including container kind, implementation section name, required exports, required metadata, and link mode
* machine ABI compatibility is explicit and inspectable
* `abi_targets` now also participate in package inspection, loader-contract metadata, project auto-resolution, and CPU-target override validation

## Project Workflow Notes

The front-door workflow is now project-aware:

* `check`, `build`, `dump-ast`, `dump-nir`, `dump-yir`, `bindings`, and cache commands all accept single-file `.ns`, project directories, or direct `nuis.toml` inputs where applicable
* `project-status` prints the resolved project graph, effective ABI mode, and per-domain ABI target details
* `project-lock-abi` materializes the currently recommended host-matching ABI set into the project manifest
* `verify-build-manifest` now reports CPU target metadata including ABI, machine arch/os, object format, calling ABI, clang triple, and cross-build flag

## Build Override Notes

Current CPU build override surface:

```text
nuis build --cpu-abi <registered-cpu-abi> <input> <output-dir>
nuis build --target <clang-target-triple> <input> <output-dir>
nuis release-check --cpu-abi <registered-cpu-abi> <input> <output-dir>
nuis release-check --target <clang-target-triple> <input> <output-dir>
nuisc compile --cpu-abi <registered-cpu-abi> <input> <output-dir>
nuisc compile --target <clang-target-triple> <input> <output-dir>
```

Important current rule:

* CPU ABI support is intended to come from `nustar` registration, not hardcoded `nuisc` tables
* explicit CPU overrides are checked against registered `abi_targets`
* project auto-ABI selection also prefers registered `abi_targets`
* current window-hosted AppKit bundle packaging still rejects cross-target output instead of pretending to support it

## Core compiler

[tools/nuisc/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/main.rs)

```text
cargo run -p nuisc -- <command>
```

`nuisc` is the current compiler-core CLI. It still exposes the same minimal
pipeline surface directly while the higher-level `nuis` workflow matures.
Its current command surface broadly mirrors the relevant `nuis` compiler-facing
subcommands:

* `status`
* `registry`
* `fmt`
* `bindings`
* `pack-nustar`
* `inspect-nustar`
* `loader-contract`
* `verify-build-manifest`
* `cache-status`
* `clean-cache`
* `cache-prune`
* `dump-ast`
* `dump-nir`
* `dump-yir`
* `check`
* `compile`

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

* [examples/yir/ball_cpu_driver.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/ball_cpu_driver.yir)
* [examples/yir/cpu_linked_list.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu_linked_list.yir)
* [examples/yir/cpu_linked_list_rustish.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu_linked_list_rustish.yir)

The current hetero render path may still package prerendered or cooked artifacts
for non-CPU slices.

This is expected at the current stage.

---

# 5. AOT Packaging Notes

The current `yir-pack-aot` path is reference-quality packaging, not final
distribution design.

Current behavior:

* pure CPU graphs can be compiled to a native binary through LLVM/clang
* hetero/window demos with `cpu.tick_i64` can now also be packaged into a
  macOS AppKit-hosted single binary with an embedded `YIR` runtime path:
  generated hosts link `libyir_runtime_host.a`, embed the `.yir` module bytes,
  and generate live framebuffer updates in-process instead of shelling out to a
  sidecar exporter
* the `window_controls_demo` project route now builds successfully through this path
* shader packaging already has a contract/package skeleton for future backend
  variants
* shader package manifests may now include per-stage binding layout entries, texture/sampler/geometry binding kinds, minimal render-state metadata, sampler/texture binding details such as filter, address mode, and texture shape, plus top-level fabric handle-table metadata, per-stage fabric table association, and Fabric worker core binding metadata
* current macOS AppKit host stubs read `fabric_worker_core`, start a dedicated
  Fabric worker thread, and apply it as that thread's startup affinity hint;
* current Fabric host booting stays AOT-first: generated host stubs embed a
  static typed action table derived from `data.*` nodes instead of constructing
  a heavyweight dynamic metadata graph at runtime
* the current typed action table also carries a minimal class/slot ABI tag for
  each Fabric action, so host-side dispatch can remain static without falling
  back to string-only pattern matching
* this is still weaker than a strict reserved-core runtime model
* CPU build manifests and `project-status` output now also expose per-domain ABI target details such as backend family and host-adaptive selection

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
