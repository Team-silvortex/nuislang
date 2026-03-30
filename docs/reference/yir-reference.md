---

# YIR Reference

## Draft Reference v0.01

---

# 1. Purpose

This document is the working reference for the current handwritten `YIR`
prototype in this repository.

It is intentionally closer to an early `LLVM LangRef + tool reference` than to
a polished language manual:

* it records what the current prototype means
* it records which ops and tools are considered reference surfaces
* it records which parts are stable enough to build on
* it records which parts are explicitly provisional

This file should evolve together with the implementation.

---

# 2. Scope

This reference currently covers:

* handwritten `YIR` source structure
* graph edge semantics
* current registered domain families
* current reference tools
* current LLVM/AOT path
* current verifier rules for the pure `cpu` memory prototype

This reference does not yet attempt to freeze:

* full `NIR`
* full `GLM`
* full `Fabric verifier`
* final `nustar` package ABI

---

# 3. Module Structure

The current handwritten form is:

```text
yir <version>
resource <name> <kind>
<mod>.<instr> <name> <resource> [args...]
edge <dep|effect|lifetime|xfer> <from> <to>
```

Semantically:

* `resource` declares an execution resource or domain instance
* `<mod>.<instr>` declares a graph node
* `edge` declares graph ordering or cross-domain structure

Text order is not the execution model.

Execution order is derived from the graph.

---

# 4. Edge Kinds

## `dep`

Same-domain dependency.

Used when a node depends on another node in the same domain family.

## `effect`

Effect ordering boundary.

Used when visible side effects must be preserved in order.

## `lifetime`

Reserved lifetime ordering edge.

The handwritten prototype accepts it as a graph edge kind, but the current
reference demos do not yet rely on a full lifetime system.

## `xfer`

Cross-domain exchange edge.

This is the important heterogeneous edge.

If two nodes live in different domain families, the verifier expects a cross-
domain dependency to be represented explicitly as `xfer`.

---

# 5. Domain Families

The current standard domain surface is:

* `cpu`
* `shader`
* `kernel`
* `data`

Future families may include surfaces such as `quantum`.

These names are capability families, not hard-coded hardware products.

Examples of resource kinds:

* `cpu.arm64`
* `shader.render`
* `kernel.apple`
* `data.fabric`

Resource kinds are open-ended and may later lower to different backend/runtime
families.

---

# 6. CPU Reference Surface

The `cpu` family is the current pure host/system and general compute surface.

## Arithmetic and host-facing ops

```text
cpu.const
cpu.add
cpu.sub
cpu.mul
cpu.madd
cpu.target_config
cpu.bind_core
cpu.window
cpu.input_i64
cpu.present_frame
cpu.print
```

Important boundary:

* `cpu.window`, `cpu.input_i64`, and `cpu.present_frame` are current host-side
  extension ops
* they are not `YIR` core semantics
* they are reference adapter surfaces used by the current preview/runtime path

## Addressable-object prototype

The current controlled heap-node prototype is:

```text
cpu.null
cpu.borrow
cpu.move_ptr
cpu.alloc_node
cpu.load_value
cpu.load_next
cpu.store_value
cpu.store_next
cpu.is_null
cpu.free
```

Current model:

* a heap node has `{ value: i64, next: ptr }`
* `cpu.alloc_node` allocates one such node
* pointers are currently modeled as a narrow node-pointer surface, not a
  general memory model

## Rust-like verifier rules

The current verifier treats this surface as an early Rust-like ownership model:

* `cpu.borrow` creates a readable borrowed pointer
* borrowed pointers may be read through
* borrowed pointers may not be written through
* borrowed pointers may not be freed
* `cpu.move_ptr` transfers ownership from the source name to a new name
* after `cpu.move_ptr`, the source name may not be used again
* after `cpu.free`, the owned name is consumed
* reading through a borrow after the owned object has been freed is rejected

This is intentionally partial, but it is already strong enough to guard the
current linked-list prototype.

Reference examples:

* valid: [examples/cpu_linked_list_rustish.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_linked_list_rustish.yir)
* invalid borrowed write: [examples/cpu_borrow_write_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_borrow_write_invalid.yir)
* invalid use-after-free: [examples/cpu_use_after_free_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_use_after_free_invalid.yir)

---

# 7. Shader Reference Surface

The `shader` family is the current backend-agnostic render/shader surface.

Current reference ops:

```text
shader.const
shader.add
shader.mul
shader.target
shader.viewport
shader.pipeline
shader.pack_ball_state
shader.begin_pass
shader.dispatch
shader.draw_instanced
shader.draw_ball
shader.draw_sphere
shader.print
```

The current portable backend subset is:

```text
shader.target
shader.pipeline
shader.begin_pass
shader.draw_instanced
```

This subset is intended to map cleanly to shared `Metal/Vulkan/DirectX/OpenGL`
style render concepts at the contract/package level.

Reference raster ops such as `shader.draw_ball` and `shader.draw_sphere` remain
valid `YIR`, but currently execute through the reference/prerender path.

---

# 8. Kernel Reference Surface

The `kernel` family is the current tensor/numerical-kernel surface.

Current reference ops:

```text
kernel.target_config
kernel.tensor
kernel.fill
kernel.matmul
kernel.add_bias
kernel.relu
kernel.print
```

This is a standard capability surface, not a hardware name.

It may later lower to:

* `npu`
* `gpu-kernel`
* `cpu-kernel`
* future accelerators

without changing the core graph meaning.

---

# 9. Data Reference Surface

The `data` family is the current instruction-level surface for Fabric-style
exchange.

Current reference op:

```text
data.move
```

The architecture term `Fabric` remains valid.

The op-family name `data` is the instruction surface used inside current `YIR`
graphs.

---

# 10. Reference Tools

The current reference toolchain entry points are:

## Parse + verify + execute

[tools/yir-run/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-run/src/main.rs)

```text
cargo run -p yir-run -- <module.yir>
```

This is the main handwritten `YIR` execution entry.

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

This is a reference render artifact path.

## Export UI plan

[tools/yir-export-ui-plan/src/main.rs](/Users/Shared/chroot/dev/nuislang/tools/yir-export-ui-plan/src/main.rs)

This extracts host-side preview plan data from current `cpu` host extension ops.

---

# 11. LLVM Path

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

# 12. AOT Packaging Notes

The current `yir-pack-aot` path is reference-quality packaging, not final
distribution design.

Current behavior:

* pure CPU graphs can be compiled to a native binary through LLVM/clang
* hetero/window demos can be packaged into a macOS AppKit-hosted single binary
  using embedded prerendered framebuffer content
* shader packaging already has a contract/package skeleton for future backend
  variants

This is the beginning of a `YIR`-native toolchain, not the final shape.

---

# 13. Stability Notes

Most stable current reference surfaces:

* graph-node based `YIR`
* explicit `xfer` edges
* `cpu / shader / kernel / data` family names
* handwritten parse -> verify -> execute path
* narrow LLVM lowering for the CPU slice

Clearly provisional current surfaces:

* host UI adapter ops
* exact shader package schema
* full ownership and lifetime model
* full `kernel` lowering contract
* final `nustar` package ABI

---

# 14. Sync Policy

This file should be updated whenever one of these changes:

* a new reference op is added
* an op is removed or renamed
* verifier rules change in a user-visible way
* LLVM lowering meaning changes
* AOT packaging meaning changes
* the standard family taxonomy changes

The goal is for this file to remain a living reference, not a stale description.
