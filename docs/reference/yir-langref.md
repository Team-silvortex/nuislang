---

# YIR LangRef

## Draft Reference v0.01

---

# 1. Purpose

This document is the working language reference for the current handwritten
`YIR` prototype in this repository.

It is the closest thing the project currently has to an early `LLVM LangRef`:

* it records what the current `YIR` graph means
* it records which domain surfaces are part of the current reference model
* it records which verifier-visible rules already exist
* it records which parts are intentionally provisional

This file should evolve together with the implementation.

---

# 2. Scope

This reference currently covers:

* handwritten `YIR` source structure
* graph edge semantics
* current standard domain families
* current reference op surfaces
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
cpu.const_bool
cpu.const_i32
cpu.const_i64
cpu.const_f32
cpu.const_f64
cpu.struct
cpu.field
cpu.neg
cpu.not
cpu.add
cpu.sub
cpu.mul
cpu.div
cpu.rem
cpu.eq
cpu.ne
cpu.lt
cpu.gt
cpu.le
cpu.ge
cpu.and
cpu.or
cpu.xor
cpu.shl
cpu.shr
cpu.madd
cpu.select
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
* the current handwritten reference model now also has a minimal typed-value
  surface: `bool`, `i32`, `i64`, `f32`, `f64`, plus named `struct` values
* current LLVM lowering already supports typed constants and struct field access;
  full stable struct aggregate ABI lowering is still provisional

## Addressable-object prototype

The current controlled heap-node prototype is:

```text
cpu.null
cpu.borrow
cpu.move_ptr
cpu.alloc_node
cpu.alloc_buffer
cpu.load_value
cpu.load_next
cpu.buffer_len
cpu.load_at
cpu.store_value
cpu.store_next
cpu.store_at
cpu.is_null
cpu.free
```

Current model:

* a heap node has `{ value: i64, next: ptr }`
* a heap buffer has `[i64; len]`
* `cpu.alloc_node` allocates one node object
* `cpu.alloc_buffer` allocates one buffer object
* pointers are currently modeled as a narrow object-handle surface, not a
  full general memory model

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
* valid buffer example: [examples/cpu_buffer_rustish.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_buffer_rustish.yir)
* invalid borrowed write: [examples/cpu_borrow_write_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_borrow_write_invalid.yir)
* invalid borrowed buffer write: [examples/cpu_buffer_borrow_write_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_buffer_borrow_write_invalid.yir)
* invalid use-after-free: [examples/cpu_use_after_free_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/cpu_use_after_free_invalid.yir)

---

# 7. Shader Reference Surface

The `shader` family is the current backend-agnostic render/shader surface.

Current reference ops:

```text
shader.const
shader.add
shader.sub
shader.mul
shader.target
shader.viewport
shader.pipeline
shader.vertex_layout
shader.vertex_buffer
shader.index_buffer
shader.blend_state
shader.depth_state
shader.raster_state
shader.render_state
shader.uv
shader.texture2d
shader.sampler
shader.uniform
shader.storage
shader.attachment
shader.texture_binding
shader.sampler_binding
shader.vertex_layout_binding
shader.vertex_binding
shader.index_binding
shader.bind_set
shader.pack_ball_state
shader.begin_pass
shader.clear
shader.overlay
shader.sample
shader.sample_uv
shader.sample_nearest
shader.sample_uv_nearest
shader.sample_uv_linear
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

`shader.draw_instanced` may additionally consume an optional `bind_set` in the
reference executor so bound geometry/resource inputs can influence validation
and rendering behavior without changing the backend-facing stage identity.

Current reference geometry interpretation is intentionally small but real:

* `pos2f` influences vertex placement in the reference frame
* `color2f` influences vertex marker glyph selection
* `uv2f` influences vertex marker glyph selection when present
* `triangle_strip` currently interprets vertex attributes, draws reference edges,
  and fills a minimal triangle area in the handwritten reference renderer

This is not yet a full graphics pipeline, but it means bound vertex data is no
longer treated as metadata only.

The current minimal texture-resource surface is:

```text
shader.texture2d
shader.sampler
shader.uv
shader.sample
shader.sample_uv
shader.sample_nearest
shader.sample_uv_nearest
shader.sample_uv_linear
```

Preferred reference direction:

* `shader.sample` and `shader.sample_uv` consult `sampler.filter`
* `nearest` selects nearest sampling
* `linear` selects linear sampling
* `shader.sample_nearest`, `shader.sample_uv_nearest`, and
  `shader.sample_uv_linear` remain as compatibility aliases

The current resource-layout surface around that subset is:

```text
shader.uniform
shader.storage
shader.attachment
shader.texture_binding
shader.sampler_binding
shader.bind_set
```

Current package/contract direction for that surface:

* `texture_binding` should carry enough metadata for backend-side texture ABI
  selection, including at least format and shape
* `sampler_binding` should carry enough metadata for backend-side sampler ABI
  selection, including at least filter and address mode

The current minimal render-state surface is:

```text
shader.blend_state
shader.depth_state
shader.raster_state
shader.render_state
```

The current minimal geometry-input surface is:

```text
shader.vertex_layout
shader.vertex_buffer
shader.index_buffer
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
kernel.add
kernel.mul
kernel.matmul
kernel.add_bias
kernel.transpose
kernel.reduce_sum
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

Current reference ops:

```text
data.move
data.copy_window
data.immutable_window
data.marker
data.bind_core
data.output_pipe
data.input_pipe
data.handle_table
```

The architecture term `Fabric` remains valid.

The op-family name `data` is the instruction surface used inside current `YIR`
graphs.

Current reference direction:

* `data.output_pipe` wraps a value as a fabric egress
* `data.input_pipe` consumes an output pipe and re-materializes the payload on
  the fabric side
* `data.marker` is a zero-sized token for fabric-side sequencing
* `data.bind_core` is the current CPU-hosted worker binding token for the
  Fabric plane
* `data.copy_window` / `data.immutable_window` are the first window-shaped
  payload wrappers in the handwritten prototype
* `data.handle_table` is the first resource-indirection carrier for fabric-side
  binding metadata
* the current verifier already rejects `input_pipe` sources that are not
  `output_pipe`, nested pipe formation, and window creation from marker/handle
  carriers
* `data.move` is intentionally narrow: it is the current `MoveValue` surface
  and may not consume `window`, `marker`, `handle_table`, or `pipe` values
* current packaging/lowering reference paths may surface `handle_table`
  contents as top-level fabric-binding metadata in generated manifests
* `data.handle_table` entries must remain resource indirections only: slot names
  must be unique and each entry must name a declared resource, not a data node
* current shader package generation may also associate a stage with the
  `handle_table` that names its backing render resource
* current AOT/package generation may also surface `bind_core` so the Fabric
  worker can be pinned to a concrete CPU core
* current macOS AppKit bundles consume that binding by spawning a dedicated
  Fabric worker thread and applying the core as its affinity hint, not as a
  hard CPU-core reservation
* current host-side Fabric booting is intentionally thin and AOT-first: `data.*`
  nodes are lowered into a static typed boot/action table instead of a
  heavyweight dynamic metadata runtime
* the current boot/action table also carries minimal class/slot tags for
  `handle_table`, `pipe`, `marker`, `window`, `move`, and worker-binding actions

---

# 10. Stability Notes

Most stable current reference surfaces:

* graph-node based `YIR`
* explicit `xfer` edges
* `cpu / shader / kernel / data` family names
* Rust-like ownership direction for the pure `cpu` domain

Clearly provisional current surfaces:

* host UI adapter ops
* full ownership and lifetime model
* full `kernel` lowering contract
* final `nustar` package ABI

---

# 11. Sync Policy

This file should be updated whenever one of these changes:

* a new reference op is added
* an op is removed or renamed
* verifier rules change in a user-visible way
* the standard family taxonomy changes

The goal is for this file to remain a living reference, not a stale description.
