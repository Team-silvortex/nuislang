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
* final `GLM`
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

The handwritten parser now also accepts a compact async-first form:

```text
resource <name> <kind>
node <instr> <name> <resource> [args...]
edge <effect|lifetime|xfer> <from> <to>
```

Semantically:

* `resource` declares an execution resource or domain instance
* `<mod>.<instr>` declares a graph node
* `edge` declares graph ordering or cross-domain structure

Text order is not the execution model.

Execution order is derived from the graph.

## Preferred Style

The current repository now tries to keep handwritten `YIR` examples in one
consistent style:

* use explicit resource kinds such as `cpu.arm64`, `shader.render`,
  `kernel.apple`, `data.fabric`
* keep `data.*` nodes on `data.fabric` resources instead of attaching them to
  compute resources directly
* prefer compact handwritten `node <instr> ...` form for new examples when the
  domain/resource already makes the instruction family obvious
* prefer typed scalar constructors such as `cpu.const_i64`,
  `shader.const_i64`, `kernel.const_f32` when the scalar type is known
* prefer canonical ops over compatibility aliases in examples; for example,
  prefer `shader.sample` / `shader.sample_uv` over legacy explicit sampling
  aliases
* group examples in this order: `resource` declarations, configuration nodes,
  value/material/tensor setup, compute or render nodes, then `edge` clauses
* treat handwritten `YIR` as default-async: ordinary data dependencies should
  flow through node arguments, while `effect`, `lifetime`, and `xfer` remain
  the explicit edges for visibility, ownership, and heterogeneous exchange

## Async-First Handwritten Form

For handwritten `YIR`, plain argument flow is now the default dependency model.

Current parser behavior:

* `node <instr> <name> <resource> ...` is accepted as a compact form
* node resources may carry an optional execution-lane suffix such as
  `cpu0@main` or `fabric0@uplink`
* ordinary argument references to earlier node names synthesize implicit
  dependency edges
* if the referenced source and target live in different domain families, that
  implicit dependency becomes an `xfer`
* nodes that share the same `<resource>@<lane>` queue synthesize implicit
  `effect` edges in handwritten order
* `effect`, `lifetime`, and explicit `xfer` edges are still written by hand
  because they carry real asynchronous hardware semantics that should stay
  visible

Current shorthand examples:

```text
resource cpu0 cpu.arm64

node const.i64 tail_value cpu0@mem 30
node alloc.node tail cpu0@mem tail_value nil
node borrow tail_ref cpu0@mem tail
node load.value tail_val cpu0@mem tail_ref
node print out cpu0@main tail_val
```

This compact layer is only a handwritten syntax convenience. Internally, the
module still resolves to canonical domain ops such as `cpu.const_i64`,
`cpu.alloc_node`, and explicit dependency edges inside `YirModule`.

Preferred stable shorthand spellings for new handwritten CPU examples:

* `const.i64`, `const.f32`, `const.bool`
* `alloc.node`, `alloc.buffer`
* `load.value`, `load.next`, `load.len`
* `store.value`, `store.next`

The shorter forms like `const`, `alloc`, and `load` still exist as convenience
aliases, but they are best treated as authoring sugar rather than the most
stable long-term handwritten surface.

Lane suffixes are queue-local scheduling metadata for the async handwritten
layer. They do not change resource ownership, but they let traces and future
schedulers distinguish flows such as host-main, memory, uplink, or render
queues on the same underlying resource. The queue identity is scoped as
`<resource>@<lane>`.

---

# 4. Edge Kinds

## `dep`

Same-domain dependency.

Used when a node depends on another node in the same domain family.

## `effect`

Effect ordering boundary.

Used when visible side effects must be preserved in order.

## `lifetime`

Lifetime and ownership ordering edge.

The current handwritten prototype now uses `lifetime` for a first explicit
`GLM`-shaped ownership layer.

Current minimum rule:

* `dep` or `xfer` says a node result must be available
* `lifetime` says a resource must still be live for an ownership-sensitive use
* `Write` and `Own` resource accesses currently require `lifetime`

This is still intentionally smaller than the final whitepaper `GLM`, but it is
already part of the current reference verifier.

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
cpu.cast_i32_to_i64
cpu.cast_i64_to_i32
cpu.cast_i32_to_f32
cpu.cast_i32_to_f64
cpu.cast_f32_to_f64
cpu.cast_f64_to_f32
cpu.neg
cpu.not
cpu.add
cpu.add_i32
cpu.add_f32
cpu.add_f64
cpu.sub
cpu.sub_i32
cpu.sub_f32
cpu.sub_f64
cpu.mul
cpu.mul_i32
cpu.mul_f32
cpu.mul_f64
cpu.div
cpu.div_i32
cpu.div_f32
cpu.div_f64
cpu.rem
cpu.eq
cpu.eq_i32
cpu.eq_f32
cpu.eq_f64
cpu.ne
cpu.lt
cpu.lt_i32
cpu.lt_f32
cpu.lt_f64
cpu.gt
cpu.gt_i32
cpu.gt_f32
cpu.gt_f64
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
cpu.tick_i64
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
* the CPU surface now also has a first typed arithmetic slice for `i32`, `f32`,
  and `f64`
* the CPU surface now also has a first typed comparison/conversion slice for
  `i32`, `f32`, and `f64`
* `cpu.input_i64` accepts either a minimal form
  `cpu.input_i64 <name> <resource> <channel> <default>`
  or a control-shaped form
  `cpu.input_i64 <name> <resource> <channel> <default> <min> <max> <step>`
* `cpu.tick_i64` is a current host-side timing hook that reads `NUIS_TICK`
  and returns `start + tick * step`

## Addressable-object prototype

The current controlled heap-node prototype is:

```text
cpu.null
cpu.borrow
cpu.borrow_end
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
* `cpu.borrow_end` explicitly closes a previously created borrow scope
* borrowed pointers may be read through
* borrowed pointers may not be written through
* borrowed pointers may not be freed
* `cpu.move_ptr` transfers ownership from the source name to a new name
* after `cpu.move_ptr`, the source name may not be used again
* after `cpu.free`, the owned name is consumed
* reading through a borrow after the owned object has been freed is rejected
* while a borrow is active, moving, freeing, or mutating the owned object is rejected
* that ownership-sensitive path becomes legal again once the last borrow use is
  ordered before the owner mutation/free, or when an explicit `cpu.borrow_end`
  node ends the named borrow earlier

## Current GLM Minimum Surface

The current repository now exposes a minimal explicit `GLM` layer.

This layer is still provisional, but it already makes ownership visible in
handwritten `YIR`.

Current concepts:

* `val`: ordinary SSA-like value flow
* `res`: resource/object flow that participates in ownership and lifetime
* `Read`: non-consuming access to a resource
* `Write`: mutating access to a resource
* `Own`: consuming or ownership-transferring access to a resource

Current minimum verifier rules:

* if a node uses another node result, there must be a `dep` or `xfer`
* if that access is `Write` or `Own`, there must also be a `lifetime` edge
* `cpu.move_ptr`, `cpu.free`, and current mutating CPU heap ops therefore
  require explicit lifetime edges
* `data.move` currently participates as an ownership-moving action and is
  modeled as an `Own` access with domain-move effect
* any `Own` access is currently treated as the final graph-visible consume of
  that source; all other uses must be ordered before it

Current canonical handwritten style:

```text
cpu.alloc_buffer buf_raw cpu0 len fill
cpu.move_ptr buf cpu0 buf_raw
cpu.store_at write_slot cpu0 buf idx value

edge dep buf_raw buf
edge lifetime buf_raw buf

edge dep buf write_slot
edge lifetime buf write_slot
```

This style is intentionally explicit: ownership transfer or mutation should be
visible in the graph rather than hidden inside the opcode name.

Current domain-move consequence:

* once `data.move` consumes a value for cross-domain transfer, later uses of
  that source must already be ordered before the move
* this is the current reference approximation of “source domain immediately
  invalid after move”

This is intentionally partial, but it is already strong enough to guard the
current linked-list prototype.

Reference examples:

* valid: [examples/yir/cpu_linked_list_rustish.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu_linked_list_rustish.yir)
* valid buffer example: [examples/yir/cpu_buffer_rustish.yir](/Users/Shared/chroot/dev/nuislang/examples/yir/cpu_buffer_rustish.yir)
* invalid borrowed write: [examples/invalid/yir/cpu_borrow_write_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_borrow_write_invalid.yir)
* invalid owner write while borrowed: [examples/invalid/yir/cpu_owner_write_while_borrowed_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_owner_write_while_borrowed_invalid.yir)
* invalid borrowed buffer write: [examples/invalid/yir/cpu_buffer_borrow_write_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_buffer_borrow_write_invalid.yir)
* invalid post-free access: [examples/invalid/yir/cpu_use_after_free_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_use_after_free_invalid.yir)
* invalid move while borrowed: [examples/invalid/yir/cpu_move_while_borrowed_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_move_while_borrowed_invalid.yir)
* invalid missing lifetime edge: [examples/invalid/yir/cpu_glm_missing_lifetime_invalid.yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir/cpu_glm_missing_lifetime_invalid.yir)

---

# 7. Shader Reference Surface

The `shader` family is the current backend-agnostic render/shader surface.

Current reference ops:

```text
shader.const
shader.add
shader.sub
shader.mul
shader.const_bool
shader.const_i32
shader.const_i64
shader.const_f32
shader.const_f64
shader.add_i32
shader.mul_i32
shader.add_f32
shader.mul_f32
shader.add_f64
shader.mul_f64
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

Current typed-scalar surface inside `shader`:

```text
shader.const_bool
shader.const_i32
shader.const_i64
shader.const_f32
shader.const_f64
shader.add_i32
shader.mul_i32
shader.add_f32
shader.mul_f32
shader.add_f64
shader.mul_f64
```

This is a deliberately small first step so shader-side material/binding setup
can consume typed scalar values directly without collapsing back into the
legacy integer-only path.

The current reference draw path also consumes typed scalar packets directly:

* tuple packets like `(color, speed[, radius_scale])`
* struct packets with at least `color` and `speed`

`color` may currently come from `bool / i32 / i64 / f32 / f64`, while `speed`
and optional `radius_scale` may come from numeric scalar values.

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
kernel.const_bool
kernel.const_i32
kernel.const_i64
kernel.const_f32
kernel.const_f64
kernel.tensor
kernel.fill
kernel.splat
kernel.add
kernel.mul
kernel.add_scalar
kernel.mul_scalar
kernel.add_i32
kernel.mul_i32
kernel.add_f32
kernel.mul_f32
kernel.add_f64
kernel.mul_f64
kernel.matmul
kernel.add_bias
kernel.shape
kernel.rows
kernel.cols
kernel.row
kernel.col
kernel.element_at
kernel.reshape
kernel.slice
kernel.broadcast
kernel.transpose
kernel.reduce_sum
kernel.reduce_sum_axis
kernel.reduce_max
kernel.reduce_max_axis
kernel.reduce_mean
kernel.reduce_mean_axis
kernel.reduce_min
kernel.reduce_min_axis
kernel.argmax
kernel.argmax_axis
kernel.argmin
kernel.argmin_axis
kernel.sort
kernel.topk
kernel.topk_axis
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

The current typed-scalar surface inside `kernel` is:

```text
kernel.const_bool
kernel.const_i32
kernel.const_i64
kernel.const_f32
kernel.const_f64
kernel.add_i32
kernel.mul_i32
kernel.add_f32
kernel.mul_f32
kernel.add_f64
kernel.mul_f64
```

This is intentionally smaller than a full typed tensor design. It establishes
that the `kernel` domain can now consume typed scalar values directly, while
full typed tensor ABI/design remains deferred.

The current tensor-shape and tensor-scalar bridge surface is:

```text
kernel.splat
kernel.add_scalar
kernel.mul_scalar
kernel.shape
kernel.rows
kernel.cols
kernel.row
kernel.col
kernel.element_at
kernel.reshape
kernel.slice
kernel.broadcast
kernel.reduce_sum_axis
kernel.reduce_max
kernel.reduce_max_axis
kernel.reduce_mean
kernel.reduce_mean_axis
kernel.reduce_min
kernel.reduce_min_axis
kernel.argmax
kernel.argmax_axis
kernel.argmin
kernel.argmin_axis
kernel.sort
kernel.topk
kernel.topk_axis
```

This keeps the tensor payload model simple while still making the `kernel`
domain feel more usable for real numerical graph construction.

Current broadcast behavior:

* `kernel.broadcast` remains available as an explicit shape-transform op
* `kernel.add`, `kernel.mul`, and `kernel.add_bias` now also apply automatic
  broadcast alignment when shapes are compatible

Current compatibility note for scalar-fed tensor ops:

* `kernel.fill` accepts either a direct integer literal or a typed scalar value
  reference
* `kernel.add_bias` accepts either a bias tensor or a typed scalar value
  reference

This keeps older examples valid while letting newer graphs move toward a more
uniform typed-scalar setup.

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
